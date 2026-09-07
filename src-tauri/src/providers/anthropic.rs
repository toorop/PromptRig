//! Anthropic provider: Messages API.
//!
//! The wire format differs from OpenAI/Mistral/OpenRouter in several real ways, not just
//! naming:
//! - Auth is an `x-api-key` header, not `Authorization: Bearer` — plus a mandatory
//!   `anthropic-version` header (every request without it is rejected).
//! - The system prompt is a top-level `system` field, not a `{"role": "system", ...}` message.
//! - `max_tokens` is *required* by the API, not optional — we fall back to a default when the
//!   user hasn't set one (see `DEFAULT_MAX_TOKENS`).
//! - `temperature`/`top_p` are deprecated for current models and rejected outright (HTTP 400)
//!   unless left at their defaults, so we simply never send them — `ModelCapabilities` reports
//!   both as unsupported, which keeps the frontend from ever showing those controls for this
//!   provider in the first place.

use std::time::Instant;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::{
    AppError, AppResult, GenerationParams, ModelCapabilities, ModelInfo, ProviderId, RunResult,
    Usage,
};

use super::LlmProvider;

const BASE_URL: &str = "https://api.anthropic.com/v1";
const API_VERSION: &str = "2023-06-01";

/// Used only when the caller didn't set `max_tokens` — the Messages API has no concept of "no
/// limit", it always requires one.
const DEFAULT_MAX_TOKENS: u32 = 4096;

pub struct AnthropicProvider {
    http: Client,
}

impl AnthropicProvider {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }
}

impl Default for AnthropicProvider {
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
    display_name: String,
    max_input_tokens: Option<u32>,
}

/// No generation parameters are supported for now — see the module doc comment. `max_tokens` is
/// deliberately reported as unsupported too, even though the API requires it: from the UI's
/// perspective "supported" means "the user can choose a value", and here they can't skip
/// sending one, so there's nothing to expose as a control either way.
fn capabilities() -> ModelCapabilities {
    ModelCapabilities {
        supports_temperature: false,
        supports_top_p: false,
        supports_max_tokens: false,
        supports_streaming: true,
    }
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct MessagesRequest<'a> {
    model: &'a str,
    system: &'a str,
    messages: Vec<Message<'a>>,
    max_tokens: u32,
}

fn build_request<'a>(
    model_id: &'a str,
    system_prompt: &'a str,
    user_prompt: &'a str,
    params: &GenerationParams,
) -> MessagesRequest<'a> {
    MessagesRequest {
        model: model_id,
        system: system_prompt,
        messages: vec![Message {
            role: "user",
            content: user_prompt,
        }],
        max_tokens: params.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
    }
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
    usage: AnthropicUsage,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn test_connection(&self, api_key: &str) -> AppResult<()> {
        let response = self
            .http
            .get(format!("{BASE_URL}/models"))
            .header("x-api-key", api_key)
            .header("anthropic-version", API_VERSION)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("Anthropic request failed: {e}")))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(AppError::Provider(format!(
                "Anthropic rejected the request (HTTP {})",
                response.status()
            )))
        }
    }

    async fn list_models(&self, api_key: &str) -> AppResult<Vec<ModelInfo>> {
        let response = self
            .http
            .get(format!("{BASE_URL}/models"))
            .header("x-api-key", api_key)
            .header("anthropic-version", API_VERSION)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("Anthropic request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(AppError::Provider(format!(
                "Anthropic rejected the request (HTTP {})",
                response.status()
            )));
        }

        let body: ModelsResponse = response
            .json()
            .await
            .map_err(|e| AppError::Provider(format!("unexpected Anthropic response: {e}")))?;

        let mut models: Vec<ModelInfo> = body
            .data
            .into_iter()
            .map(|listing| ModelInfo {
                provider: ProviderId::Anthropic,
                model_id: listing.id,
                display_name: listing.display_name,
                capabilities: capabilities(),
                // A context window of 0 shows up in some API examples as a placeholder for
                // "unknown" rather than a real limit — treat it the same as absent.
                context_window: listing.max_input_tokens.filter(|&n| n > 0),
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
            .post(format!("{BASE_URL}/messages"))
            .header("x-api-key", api_key)
            .header("anthropic-version", API_VERSION)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("Anthropic request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Provider(format!(
                "Anthropic returned HTTP {status}: {body}"
            )));
        }

        let body: MessagesResponse = response
            .json()
            .await
            .map_err(|e| AppError::Provider(format!("unexpected Anthropic response: {e}")))?;
        let duration_ms = started.elapsed().as_millis() as u32;

        let text = body
            .content
            .into_iter()
            .find(|block| block.kind == "text")
            .and_then(|block| block.text)
            .ok_or_else(|| AppError::Provider("Anthropic response had no text content".into()))?;

        Ok(RunResult {
            text,
            usage: Some(Usage {
                input_tokens: body.usage.input_tokens,
                output_tokens: body.usage.output_tokens,
            }),
            duration_ms,
            ttft_ms: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_puts_system_prompt_at_top_level_not_in_messages() {
        let params = GenerationParams::default();

        let request = build_request("claude-opus-5", "You are terse.", "Hi", &params);
        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["system"], "You are terse.");
        assert_eq!(json["messages"].as_array().unwrap().len(), 1);
        assert_eq!(json["messages"][0]["role"], "user");
        assert_eq!(json["messages"][0]["content"], "Hi");
    }

    #[test]
    fn request_never_includes_temperature_or_top_p() {
        // Even if a caller somehow set these (defense in depth, matching how openai.rs handles
        // reasoning models) - Anthropic rejects them outright for current models.
        let params = GenerationParams {
            temperature: Some(0.7),
            top_p: Some(0.9),
            max_tokens: Some(200),
        };

        let request = build_request("claude-opus-5", "system", "user", &params);
        let json = serde_json::to_value(&request).unwrap();

        assert!(json.get("temperature").is_none());
        assert!(json.get("top_p").is_none());
        assert_eq!(json["max_tokens"], 200);
    }

    #[test]
    fn request_falls_back_to_a_default_max_tokens_when_unset() {
        let params = GenerationParams::default();

        let request = build_request("claude-opus-5", "system", "user", &params);
        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["max_tokens"], DEFAULT_MAX_TOKENS);
    }
}
