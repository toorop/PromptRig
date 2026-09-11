//! OpenRouter provider: an aggregator exposing many vendors' models behind one OpenAI-compatible
//! Chat Completions API. This is the friendliest provider to integrate so far: `/v1/models`
//! reports each model's `context_length` and a `supported_parameters` array directly, so
//! capabilities come straight from the API instead of a heuristic (unlike OpenAI) — and since
//! OpenRouter itself translates to whatever wire format the underlying model actually needs, we
//! never need provider-specific quirks like OpenAI's reasoning-model `max_completion_tokens`
//! here; OpenRouter's gateway handles that translation on its side.
//!
//! Pricing note: OpenRouter's `/v1/models` also returns real per-model pricing (`pricing.prompt`
//! / `pricing.completion`, USD per token) — unlike every other provider we support, which don't
//! expose pricing via their API at all. Using that directly (instead of our own `pricing.json`)
//! is a deferred idea (see TODO.md) — not implemented yet, so OpenRouter costs show "—" like any
//! other provider without a `pricing.json` entry, for now.

use std::time::Instant;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::{
    AppError, AppResult, GenerationParams, ModelCapabilities, ModelInfo, ProviderId, RunResult,
    Usage,
};

use super::LlmProvider;

const BASE_URL: &str = "https://openrouter.ai/api/v1";

pub struct OpenRouterProvider {
    http: Client,
}

impl OpenRouterProvider {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }
}

impl Default for OpenRouterProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelListing>,
}

#[derive(Deserialize)]
struct ModelListing {
    id: String,
    name: String,
    context_length: Option<u32>,
    #[serde(default)]
    supported_parameters: Vec<String>,
}

fn capabilities_from(supported_parameters: &[String]) -> ModelCapabilities {
    let supports = |param: &str| supported_parameters.iter().any(|p| p == param);
    ModelCapabilities {
        supports_temperature: supports("temperature"),
        supports_top_p: supports("top_p"),
        supports_max_tokens: supports("max_tokens"),
        supports_streaming: true,
    }
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
    /// OpenRouter's own unified reasoning control — a nested object it translates internally to
    /// whatever the underlying model actually needs (a real, documented OpenRouter feature, not
    /// the deprecated flat `reasoning_effort` field some models used to accept directly).
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<ReasoningConfig>,
}

#[derive(Serialize)]
struct ReasoningConfig {
    effort: String,
}

fn build_request<'a>(
    model_id: &'a str,
    system_prompt: &'a str,
    user_prompt: &'a str,
    params: &GenerationParams,
) -> ChatCompletionRequest<'a> {
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
        temperature: params.temperature,
        top_p: params.top_p,
        max_tokens: params.max_tokens,
        reasoning: params
            .reasoning_effort
            .clone()
            .map(|effort| ReasoningConfig { effort }),
    }
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    usage: Option<OpenRouterUsage>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatCompletionMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ChatCompletionMessage {
    /// `null` in the real API when a reasoning/"thinking" model (OpenRouter routes to plenty —
    /// this is what actually surfaced the bug: a real user report against a Gemini "thinking"
    /// model routed through OpenRouter) spends its entire `max_tokens` budget on internal
    /// reasoning before emitting any visible output. Was `String`, which made this a raw
    /// `error decoding response body` crash instead of an actionable error — see `extract_text`.
    content: Option<String>,
}

/// Pulls the model's text out of the first choice. Kept separate from `generate` so this exact
/// shape — including the reasoning-model empty-content case above — can be unit-tested without a
/// real API call.
fn extract_text(choices: Vec<Choice>) -> AppResult<String> {
    let choice = choices
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Provider("OpenRouter response had no choices".into()))?;

    choice
        .message
        .content
        .filter(|c| !c.is_empty())
        .ok_or_else(|| {
            if choice.finish_reason.as_deref() == Some("length") {
                AppError::Provider(
                "OpenRouter returned no content — the model likely spent its entire max_tokens \
                 budget on internal reasoning before answering. Try raising max_tokens."
                    .into(),
            )
            } else {
                AppError::Provider("OpenRouter response had empty content".into())
            }
        })
}

#[derive(Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[async_trait]
impl LlmProvider for OpenRouterProvider {
    async fn test_connection(&self, api_key: &str) -> AppResult<()> {
        let response = self
            .http
            .get(format!("{BASE_URL}/models"))
            .bearer_auth(api_key)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("OpenRouter request failed: {e}")))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(AppError::Provider(format!(
                "OpenRouter rejected the request (HTTP {})",
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
            .map_err(|e| AppError::Provider(format!("OpenRouter request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(AppError::Provider(format!(
                "OpenRouter rejected the request (HTTP {})",
                response.status()
            )));
        }

        let body: ModelsResponse = response
            .json()
            .await
            .map_err(|e| AppError::Provider(format!("unexpected OpenRouter response: {e}")))?;

        let mut models: Vec<ModelInfo> = body
            .data
            .into_iter()
            .map(|listing| ModelInfo {
                provider: ProviderId::OpenRouter,
                capabilities: capabilities_from(&listing.supported_parameters),
                display_name: listing.name,
                model_id: listing.id,
                context_window: listing.context_length,
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
            // Identifies this app in OpenRouter's dashboards; not required, but polite (and
            // recommended by OpenRouter for anti-abuse / usage attribution). We have no public
            // URL to send as HTTP-Referer, so we skip that one rather than send a misleading value.
            .header("X-Title", "PromptRig")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("OpenRouter request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Provider(format!(
                "OpenRouter returned HTTP {status}: {body}"
            )));
        }

        let body: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| AppError::Provider(format!("unexpected OpenRouter response: {e}")))?;
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
        let err = extract_text(vec![choice(None, Some("stop"))]).unwrap_err();
        assert!(!err.to_string().contains("internal reasoning"));
    }

    #[test]
    fn extract_text_rejects_no_choices() {
        assert!(extract_text(vec![]).is_err());
    }

    #[test]
    fn capabilities_reflect_supported_parameters() {
        let caps = capabilities_from(&["temperature".into(), "max_tokens".into()]);

        assert!(caps.supports_temperature);
        assert!(!caps.supports_top_p);
        assert!(caps.supports_max_tokens);
    }

    #[test]
    fn no_supported_parameters_means_no_capabilities() {
        let caps = capabilities_from(&[]);

        assert!(!caps.supports_temperature);
        assert!(!caps.supports_top_p);
        assert!(!caps.supports_max_tokens);
    }

    #[test]
    fn request_omits_unset_params() {
        let params = GenerationParams::default();

        let request = build_request("openai/gpt-4o-mini", "system", "user", &params);
        let json = serde_json::to_value(&request).unwrap();

        assert!(json.get("temperature").is_none());
        assert!(json.get("top_p").is_none());
        assert!(json.get("max_tokens").is_none());
        assert!(json.get("reasoning").is_none());
    }

    #[test]
    fn reasoning_effort_becomes_a_nested_reasoning_object() {
        let params = GenerationParams {
            temperature: None,
            top_p: None,
            max_tokens: None,
            reasoning_effort: Some("high".into()),
        };

        let request = build_request("openai/gpt-5", "system", "user", &params);
        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["reasoning"]["effort"], "high");
    }
}
