//! OpenAI provider: Chat Completions API.
//!
//! Model catalog note: OpenAI's `/v1/models` endpoint lists every model the account can use
//! (including audio, image, embedding, and moderation models) with no structured field saying
//! which ones are chat models or what parameters they accept. `is_chat_model` and
//! `is_reasoning_model` below are pragmatic heuristics over the model id string rather than a
//! hardcoded per-model table — the model catalog changes too often for a hand-maintained exact
//! list to stay accurate (see docs/start.md's Model Registry section).

use std::time::Instant;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::{
    AppError, AppResult, GenerationParams, ModelCapabilities, ModelInfo, ProviderId, RunResult,
    Usage,
};

use super::LlmProvider;

const BASE_URL: &str = "https://api.openai.com/v1";

/// Substrings that mark a model id as *not* a chat/text-generation model.
const NON_CHAT_MODEL_MARKERS: &[&str] = &[
    "whisper",
    "tts",
    "dall-e",
    "embedding",
    "moderation",
    "davinci",
    "babbage",
    "curie",
    "ada",
    "audio",
    "realtime",
    "transcribe",
    "image",
];

pub struct OpenAiProvider {
    http: Client,
}

impl OpenAiProvider {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }
}

impl Default for OpenAiProvider {
    fn default() -> Self {
        Self::new()
    }
}

fn is_chat_model(model_id: &str) -> bool {
    let lower = model_id.to_lowercase();
    !NON_CHAT_MODEL_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
}

/// OpenAI's "reasoning" models (the o-series, and newer flagship models that expose a
/// reasoning-effort control instead of classic sampling) don't accept `temperature`/`top_p` and
/// use `max_completion_tokens` instead of `max_tokens`. There's no API field to detect this, so
/// we key off well-known name prefixes. This is a best-effort heuristic based on OpenAI's
/// current (September 2026) catalog — verify against OpenAI's own docs before relying on it, and
/// update this function as their naming evolves.
fn is_reasoning_model(model_id: &str) -> bool {
    model_id.starts_with("o1")
        || model_id.starts_with("o3")
        || model_id.starts_with("o4")
        || model_id.starts_with("gpt-5.6")
        || model_id.starts_with("gpt-6")
}

fn infer_capabilities(model_id: &str) -> ModelCapabilities {
    let reasoning = is_reasoning_model(model_id);
    ModelCapabilities {
        supports_temperature: !reasoning,
        supports_top_p: !reasoning,
        supports_max_tokens: true,
        supports_streaming: true,
    }
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelListing>,
}

#[derive(Deserialize)]
struct ModelListing {
    id: String,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u32>,
    /// `"minimal"`/`"low"`/`"medium"`/`"high"` (exact allowed set is model-dependent — see
    /// `domain::ModelInfo::reasoning_effort_levels`). A real, top-level Chat Completions field
    /// for reasoning models, verified against OpenAI's docs before wiring this in.
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
}

/// Builds the request body for one generation call. Kept separate from `generate` so the
/// (non-obvious) reasoning-model field-name switch can be unit-tested without a network call.
fn build_request<'a>(
    model_id: &'a str,
    system_prompt: &'a str,
    user_prompt: &'a str,
    params: &GenerationParams,
) -> ChatCompletionRequest<'a> {
    let reasoning = is_reasoning_model(model_id);

    ChatCompletionRequest {
        model: model_id,
        messages: vec![
            ChatMessage {
                role: "system",
                content: system_prompt,
            },
            ChatMessage {
                role: "user",
                content: user_prompt,
            },
        ],
        // Reasoning models reject temperature/top_p outright, so we drop them here even if a
        // caller passed a value in `params` — belt and suspenders alongside the frontend only
        // showing these controls when `ModelCapabilities` says they're supported.
        temperature: if reasoning { None } else { params.temperature },
        top_p: if reasoning { None } else { params.top_p },
        max_tokens: if reasoning { None } else { params.max_tokens },
        max_completion_tokens: if reasoning { params.max_tokens } else { None },
        // Not gated on `reasoning` the way sampling params above are: the frontend only ever
        // sets this from a model's own `reasoning_effort_levels` (populated exactly when
        // models.dev reports an "effort" option), so a value only ever arrives for a model that
        // actually accepts it.
        reasoning_effort: params.reasoning_effort.clone(),
    }
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatCompletionMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ChatCompletionMessage {
    /// `null` in the real API when a reasoning model (o-series, `gpt-5.6`/`gpt-6`) spends its
    /// entire `max_completion_tokens` budget on internal reasoning before emitting any visible
    /// output — was `String` until this was hit live (`finish_reason: "length"`, `content: null`).
    /// See `extract_text` for how this is turned into an actionable error instead of a raw
    /// deserialization failure.
    content: Option<String>,
}

/// Pulls the model's text out of the first choice. Kept separate from `generate` (which owns
/// the actual HTTP call) so this exact shape — including the reasoning-model empty-content case
/// above — can be unit-tested without a real API call.
fn extract_text(choices: Vec<Choice>) -> AppResult<String> {
    let choice = choices
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Provider("OpenAI response had no choices".into()))?;

    choice
        .message
        .content
        .filter(|c| !c.is_empty())
        .ok_or_else(|| {
            if choice.finish_reason.as_deref() == Some("length") {
                AppError::Provider(
                    "OpenAI returned no content — the model likely spent its entire max_tokens \
                 budget on internal reasoning before answering. Try raising max_tokens."
                        .into(),
                )
            } else {
                AppError::Provider("OpenAI response had empty content".into())
            }
        })
}

#[derive(Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn test_connection(&self, api_key: &str) -> AppResult<()> {
        let response = self
            .http
            .get(format!("{BASE_URL}/models"))
            .bearer_auth(api_key)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("OpenAI request failed: {e}")))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(AppError::Provider(format!(
                "OpenAI rejected the request (HTTP {})",
                response.status()
            )))
        }
    }

    async fn list_models(&self, api_key: &str) -> AppResult<Vec<ModelInfo>> {
        let response = self
            .http
            .get(format!("{BASE_URL}/models"))
            .bearer_auth(api_key)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("OpenAI request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(AppError::Provider(format!(
                "OpenAI rejected the request (HTTP {})",
                response.status()
            )));
        }

        let body: ModelsResponse = response
            .json()
            .await
            .map_err(|e| AppError::Provider(format!("unexpected OpenAI response: {e}")))?;

        let mut models: Vec<ModelInfo> = body
            .data
            .into_iter()
            .map(|listing| listing.id)
            .filter(|id| is_chat_model(id))
            .map(|id| ModelInfo {
                provider: ProviderId::OpenAi,
                capabilities: infer_capabilities(&id),
                display_name: id.clone(),
                model_id: id,
                // OpenAI's model list doesn't include context window sizes.
                context_window: None,
                pricing: None,
                reasoning_effort_levels: None,
            })
            .collect();

        models.sort_by(|a, b| a.model_id.cmp(&b.model_id));
        Ok(models)
    }

    async fn generate(
        &self,
        api_key: &str,
        model_id: &str,
        system_prompt: &str,
        user_prompt: &str,
        params: &GenerationParams,
    ) -> AppResult<RunResult> {
        let request_body = build_request(model_id, system_prompt, user_prompt, params);
        let started = Instant::now();

        let response = self
            .http
            .post(format!("{BASE_URL}/chat/completions"))
            .bearer_auth(api_key)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("OpenAI request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Provider(format!(
                "OpenAI returned HTTP {status}: {body}"
            )));
        }

        let body: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| AppError::Provider(format!("unexpected OpenAI response: {e}")))?;
        let duration_ms = started.elapsed().as_millis() as u32;

        let text = extract_text(body.choices)?;

        let usage = body.usage.map(|usage| Usage {
            input_tokens: usage.prompt_tokens,
            output_tokens: usage.completion_tokens,
        });

        Ok(RunResult {
            text,
            usage,
            duration_ms,
            ttft_ms: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choice(content: Option<&str>, finish_reason: Option<&str>) -> Choice {
        Choice {
            message: ChatCompletionMessage {
                content: content.map(str::to_string),
            },
            finish_reason: finish_reason.map(str::to_string),
        }
    }

    #[test]
    fn extract_text_reads_ordinary_content() {
        let text = extract_text(vec![choice(Some("hello"), Some("stop"))]).unwrap();
        assert_eq!(text, "hello");
    }

    #[test]
    fn extract_text_reports_the_reasoning_budget_case() {
        let err = extract_text(vec![choice(None, Some("length"))]).unwrap_err();
        assert!(err.to_string().contains("internal reasoning"));
    }

    #[test]
    fn extract_text_reports_a_generic_error_for_other_empty_content() {
        let err = extract_text(vec![choice(None, Some("content_filter"))]).unwrap_err();
        assert!(!err.to_string().contains("internal reasoning"));
    }

    #[test]
    fn extract_text_rejects_no_choices() {
        assert!(extract_text(vec![]).is_err());
    }

    #[test]
    fn filters_out_non_chat_models() {
        assert!(is_chat_model("gpt-4o-mini"));
        assert!(is_chat_model("gpt-5.6-sol"));
        assert!(!is_chat_model("whisper-1"));
        assert!(!is_chat_model("text-embedding-3-small"));
        assert!(!is_chat_model("dall-e-3"));
        assert!(!is_chat_model("omni-moderation-latest"));
    }

    #[test]
    fn recognizes_reasoning_models() {
        assert!(is_reasoning_model("o1"));
        assert!(is_reasoning_model("o3-mini"));
        assert!(is_reasoning_model("gpt-5.6-sol"));
        assert!(is_reasoning_model("gpt-6-astra"));
        assert!(!is_reasoning_model("gpt-4o-mini"));
    }

    #[test]
    fn regular_model_request_uses_max_tokens_and_sampling_params() {
        let params = GenerationParams {
            temperature: Some(0.7),
            top_p: Some(0.9),
            max_tokens: Some(256),
            reasoning_effort: None,
        };

        let request = build_request("gpt-4o-mini", "system", "user", &params);
        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["temperature"], 0.7);
        assert_eq!(json["top_p"], 0.9);
        assert_eq!(json["max_tokens"], 256);
        assert!(json.get("max_completion_tokens").is_none());
    }

    #[test]
    fn reasoning_model_request_drops_sampling_params_and_uses_max_completion_tokens() {
        let params = GenerationParams {
            temperature: Some(0.7),
            top_p: Some(0.9),
            max_tokens: Some(256),
            reasoning_effort: None,
        };

        let request = build_request("gpt-6-astra", "system", "user", &params);
        let json = serde_json::to_value(&request).unwrap();

        assert!(json.get("temperature").is_none());
        assert!(json.get("top_p").is_none());
        assert!(json.get("max_tokens").is_none());
        assert_eq!(json["max_completion_tokens"], 256);
    }

    #[test]
    fn reasoning_effort_is_forwarded_when_set() {
        let params = GenerationParams {
            temperature: None,
            top_p: None,
            max_tokens: None,
            reasoning_effort: Some("high".into()),
        };

        let request = build_request("gpt-6-astra", "system", "user", &params);
        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["reasoning_effort"], "high");
    }
}
