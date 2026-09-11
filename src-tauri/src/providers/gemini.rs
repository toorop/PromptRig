//! Google Gemini provider — the "Gemini API" (accessed via Google AI Studio and a plain API
//! key), *not* Vertex AI (Google Cloud's enterprise product, which needs a GCP project and
//! different auth entirely). This is the one users with just a Google account can actually use.
//!
//! Wire-format differences from every other provider so far:
//! - Auth is an `x-goog-api-key` header (there's also a `?key=` query param fallback, but a
//!   header avoids leaking the key into request logs / URL history).
//! - The model id is part of the URL path (`.../models/{id}:generateContent`), not a field in
//!   the JSON body.
//! - The user/system prompts are `contents`/`systemInstruction` objects made of `parts` (Gemini
//!   supports multi-part, multi-modal content — we only ever send one text part), and generation
//!   parameters nest under a `generationConfig` object with camelCase names (`topP`,
//!   `maxOutputTokens`) rather than being top-level snake_case fields.

use std::time::Instant;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::{
    AppError, AppResult, GenerationParams, ModelCapabilities, ModelInfo, ProviderId, RunResult,
    Usage,
};

use super::LlmProvider;

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

pub struct GeminiProvider {
    http: Client,
}

impl GeminiProvider {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }
}

impl Default for GeminiProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct ModelsResponse {
    models: Vec<ModelListing>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelListing {
    /// e.g. `"models/gemini-2.5-flash"` — we strip the `models/` prefix for our own
    /// `ModelInfo::model_id`, and re-add it when building the request URL in `generate`.
    name: String,
    display_name: String,
    input_token_limit: Option<u32>,
    #[serde(default)]
    supported_generation_methods: Vec<String>,
}

fn strip_models_prefix(name: &str) -> &str {
    name.strip_prefix("models/").unwrap_or(name)
}

#[derive(Serialize)]
struct Part<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct Content<'a> {
    role: &'a str,
    parts: Vec<Part<'a>>,
}

#[derive(Serialize)]
struct SystemInstruction<'a> {
    parts: Vec<Part<'a>>,
}

#[derive(Serialize)]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(rename = "topP", skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(rename = "maxOutputTokens", skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(rename = "thinkingConfig", skip_serializing_if = "Option::is_none")]
    thinking_config: Option<ThinkingConfig>,
}

/// `thinkingLevel` (verified against Gemini's own docs) — nests under `generationConfig`, unlike
/// every other provider's reasoning-effort field, which sits at the request's top level.
#[derive(Serialize)]
struct ThinkingConfig {
    #[serde(rename = "thinkingLevel")]
    thinking_level: String,
}

#[derive(Serialize)]
struct GenerateContentRequest<'a> {
    contents: Vec<Content<'a>>,
    #[serde(rename = "systemInstruction")]
    system_instruction: SystemInstruction<'a>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

fn build_request<'a>(
    system_prompt: &'a str,
    user_prompt: &'a str,
    params: &GenerationParams,
) -> GenerateContentRequest<'a> {
    GenerateContentRequest {
        contents: vec![Content {
            role: "user",
            parts: vec![Part { text: user_prompt }],
        }],
        system_instruction: SystemInstruction {
            parts: vec![Part {
                text: system_prompt,
            }],
        },
        generation_config: GenerationConfig {
            temperature: params.temperature,
            top_p: params.top_p,
            max_output_tokens: params.max_tokens,
            thinking_config: params
                .reasoning_effort
                .clone()
                .map(|thinking_level| ThinkingConfig { thinking_level }),
        },
    }
}

#[derive(Deserialize)]
struct GenerateContentResponse {
    candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<UsageMetadata>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: Option<String>,
}

#[derive(Deserialize)]
struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: u32,
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn test_connection(&self, api_key: &str) -> AppResult<()> {
        let response = self
            .http
            .get(format!("{BASE_URL}/models"))
            .header("x-goog-api-key", api_key)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("Gemini request failed: {e}")))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(AppError::Provider(format!(
                "Gemini rejected the request (HTTP {})",
                response.status()
            )))
        }
    }

    async fn list_models(&self, api_key: &str) -> AppResult<Vec<ModelInfo>> {
        let response = self
            .http
            .get(format!("{BASE_URL}/models"))
            .header("x-goog-api-key", api_key)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("Gemini request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(AppError::Provider(format!(
                "Gemini rejected the request (HTTP {})",
                response.status()
            )));
        }

        let body: ModelsResponse = response
            .json()
            .await
            .map_err(|e| AppError::Provider(format!("unexpected Gemini response: {e}")))?;

        let mut models: Vec<ModelInfo> = body
            .models
            .into_iter()
            .filter(|listing| {
                listing
                    .supported_generation_methods
                    .iter()
                    .any(|m| m == "generateContent")
            })
            .map(|listing| ModelInfo {
                provider: ProviderId::Gemini,
                model_id: strip_models_prefix(&listing.name).to_string(),
                display_name: listing.display_name,
                // No evidence (unlike OpenAI/Anthropic's reasoning models) that Gemini's
                // "thinking" models restrict classic sampling params, so every model that
                // supports generateContent gets the same capabilities.
                capabilities: ModelCapabilities {
                    supports_temperature: true,
                    supports_top_p: true,
                    supports_max_tokens: true,
                    supports_streaming: true,
                },
                context_window: listing.input_token_limit,
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
        let request_body = build_request(system_prompt, user_prompt, params);
        let started = Instant::now();

        let response = self
            .http
            .post(format!("{BASE_URL}/models/{model_id}:generateContent"))
            .header("x-goog-api-key", api_key)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("Gemini request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Provider(format!(
                "Gemini returned HTTP {status}: {body}"
            )));
        }

        let body: GenerateContentResponse = response
            .json()
            .await
            .map_err(|e| AppError::Provider(format!("unexpected Gemini response: {e}")))?;
        let duration_ms = started.elapsed().as_millis() as u32;

        let text = body
            .candidates
            .into_iter()
            .next()
            .and_then(|candidate| candidate.content.parts.into_iter().next())
            .and_then(|part| part.text)
            .ok_or_else(|| AppError::Provider("Gemini response had no text content".into()))?;

        let usage = body.usage_metadata.map(|usage| Usage {
            input_tokens: usage.prompt_token_count,
            output_tokens: usage.candidates_token_count,
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

    #[test]
    fn strips_the_models_prefix() {
        assert_eq!(
            strip_models_prefix("models/gemini-2.5-flash"),
            "gemini-2.5-flash"
        );
        assert_eq!(strip_models_prefix("gemini-2.5-flash"), "gemini-2.5-flash");
    }

    #[test]
    fn request_nests_params_under_generation_config_with_camel_case_names() {
        let params = GenerationParams {
            temperature: Some(0.5),
            top_p: Some(0.9),
            max_tokens: Some(200),
            reasoning_effort: None,
        };

        let request = build_request("You are terse.", "Hi", &params);
        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["generationConfig"]["temperature"], 0.5);
        assert_eq!(json["generationConfig"]["topP"], 0.9);
        assert_eq!(json["generationConfig"]["maxOutputTokens"], 200);
        assert_eq!(
            json["systemInstruction"]["parts"][0]["text"],
            "You are terse."
        );
        assert_eq!(json["contents"][0]["role"], "user");
        assert_eq!(json["contents"][0]["parts"][0]["text"], "Hi");
    }

    #[test]
    fn reasoning_effort_becomes_a_nested_thinking_level() {
        let params = GenerationParams {
            temperature: None,
            top_p: None,
            max_tokens: None,
            reasoning_effort: Some("high".into()),
        };

        let request = build_request("You are terse.", "Hi", &params);
        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(
            json["generationConfig"]["thinkingConfig"]["thinkingLevel"],
            "high"
        );
    }
}
