//! Mistral provider: Chat Completions API.
//!
//! Mistral's API closely mirrors OpenAI's, but its `/v1/models` endpoint is far more useful for
//! our purposes: each entry reports a `capabilities.completion_chat` flag and a
//! `max_context_length`, so — unlike OpenAI — we don't need to guess which models are chat
//! models or fabricate capability/context-window data from the model id string.

use std::time::Instant;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::{
    AppError, AppResult, GenerationParams, ModelCapabilities, ModelInfo, ProviderId, RunResult,
    Usage,
};

use super::LlmProvider;

const BASE_URL: &str = "https://api.mistral.ai/v1";

pub struct MistralProvider {
    http: Client,
}

impl MistralProvider {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }
}

impl Default for MistralProvider {
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
    capabilities: ModelListingCapabilities,
    max_context_length: Option<u32>,
}

#[derive(Deserialize)]
struct ModelListingCapabilities {
    completion_chat: bool,
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
    /// `"none"`/`"high"` on the mistral-small/medium-latest line (verified against Mistral's own
    /// reasoning docs) — `magistral-*` models reason natively and reject this field outright, but
    /// they also report no `"effort"` option in models.dev, so the frontend never sets a value
    /// for them and this is never sent in practice.
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
}

/// Builds the request body. Kept separate from `generate` (even though there's no
/// provider-specific branching here, unlike OpenAI's reasoning-model handling) so the JSON
/// shape can be unit-tested without a network call.
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
        reasoning_effort: params.reasoning_effort.clone(),
    }
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    usage: Option<MistralUsage>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatCompletionMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ChatCompletionMessage {
    /// `null` in the real API when a reasoning model (e.g. `magistral-*`) spends its entire
    /// `max_tokens` budget on internal reasoning before emitting any visible output — same shape
    /// as the identical fix in `providers::openai`/`providers::openrouter`. See `extract_text`.
    content: Option<String>,
}

/// Pulls the model's text out of the first choice. Kept separate from `generate` so this exact
/// shape — including the reasoning-model empty-content case above — can be unit-tested without a
/// real API call.
fn extract_text(choices: Vec<Choice>) -> AppResult<String> {
    let choice = choices
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Provider("Mistral response had no choices".into()))?;

    choice
        .message
        .content
        .filter(|c| !c.is_empty())
        .ok_or_else(|| {
            if choice.finish_reason.as_deref() == Some("length") {
                AppError::Provider(
                    "Mistral returned no content — the model likely spent its entire max_tokens \
                 budget on internal reasoning before answering. Try raising max_tokens."
                        .into(),
                )
            } else {
                AppError::Provider("Mistral response had empty content".into())
            }
        })
}

#[derive(Deserialize)]
struct MistralUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[async_trait]
impl LlmProvider for MistralProvider {
    async fn test_connection(&self, api_key: &str) -> AppResult<()> {
        let response = self
            .http
            .get(format!("{BASE_URL}/models"))
            .bearer_auth(api_key)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("Mistral request failed: {e}")))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(AppError::Provider(format!(
                "Mistral rejected the request (HTTP {})",
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
            .map_err(|e| AppError::Provider(format!("Mistral request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(AppError::Provider(format!(
                "Mistral rejected the request (HTTP {})",
                response.status()
            )));
        }

        let body: ModelsResponse = response
            .json()
            .await
            .map_err(|e| AppError::Provider(format!("unexpected Mistral response: {e}")))?;

        let mut models: Vec<ModelInfo> = body
            .data
            .into_iter()
            .filter(|listing| listing.capabilities.completion_chat)
            .map(|listing| ModelInfo {
                provider: ProviderId::Mistral,
                display_name: listing.id.clone(),
                model_id: listing.id,
                capabilities: ModelCapabilities {
                    supports_temperature: true,
                    supports_top_p: true,
                    supports_max_tokens: true,
                    supports_streaming: true,
                },
                context_window: listing.max_context_length,
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
            .map_err(|e| AppError::Provider(format!("Mistral request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Provider(format!(
                "Mistral returned HTTP {status}: {body}"
            )));
        }

        let body: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| AppError::Provider(format!("unexpected Mistral response: {e}")))?;
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
    fn request_omits_unset_params() {
        let params = GenerationParams::default();

        let request = build_request("mistral-small-latest", "system", "user", &params);
        let json = serde_json::to_value(&request).unwrap();

        assert!(json.get("temperature").is_none());
        assert!(json.get("top_p").is_none());
        assert!(json.get("max_tokens").is_none());
    }

    #[test]
    fn request_includes_set_params() {
        let params = GenerationParams {
            temperature: Some(0.5),
            top_p: Some(0.9),
            max_tokens: Some(500),
            reasoning_effort: None,
        };

        let request = build_request("mistral-small-latest", "system", "user", &params);
        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["temperature"], 0.5);
        assert_eq!(json["top_p"], 0.9);
        assert_eq!(json["max_tokens"], 500);
    }

    #[test]
    fn reasoning_effort_is_forwarded_when_set() {
        let params = GenerationParams {
            temperature: None,
            top_p: None,
            max_tokens: None,
            reasoning_effort: Some("high".into()),
        };

        let request = build_request("mistral-medium-latest", "system", "user", &params);
        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["reasoning_effort"], "high");
    }
}
