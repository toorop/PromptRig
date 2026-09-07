//! Provider abstraction: one `LlmProvider` implementation per module (see `openai.rs` for the
//! first one), looked up through a `ProviderRegistry`. See docs/start.md's "Fournisseurs"
//! section — this is deliberately pragmatic rather than trying to hide every difference between
//! providers.

pub mod mistral;
pub mod openai;

use std::collections::HashMap;

use async_trait::async_trait;

use crate::domain::{AppError, AppResult, GenerationParams, ModelInfo, ProviderId, RunResult};

/// What every LLM provider integration must be able to do.
///
/// Implementations never see the OS keyring or SQLite — callers (the command layer, once it
/// exists) look up the API key via `secrets::get_api_key` and pass it in explicitly. This keeps
/// a provider testable with a fake key and independent of how the app stores secrets.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Verifies that `api_key` actually works against this provider, with a lightweight call —
    /// not a real (billed) generation request.
    async fn test_connection(&self, api_key: &str) -> AppResult<()>;

    /// Lists the models this provider currently makes available to `api_key`.
    async fn list_models(&self, api_key: &str) -> AppResult<Vec<ModelInfo>>;

    /// Runs one generation call and returns a normalized result.
    async fn generate(
        &self,
        api_key: &str,
        model_id: &str,
        system_prompt: &str,
        user_prompt: &str,
        params: &GenerationParams,
    ) -> AppResult<RunResult>;
}

/// Looks up the `LlmProvider` implementation for each `ProviderId`.
///
/// Built once (`ProviderRegistry::new()`) and kept around for the app's lifetime. Adding a new
/// provider means adding a variant to `ProviderId`, a new module here implementing
/// `LlmProvider`, and one line in `new()` below — nothing else in the app needs to change.
pub struct ProviderRegistry {
    providers: HashMap<ProviderId, Box<dyn LlmProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut providers: HashMap<ProviderId, Box<dyn LlmProvider>> = HashMap::new();
        providers.insert(ProviderId::OpenAi, Box::new(openai::OpenAiProvider::new()));
        providers.insert(
            ProviderId::Mistral,
            Box::new(mistral::MistralProvider::new()),
        );
        Self { providers }
    }

    pub fn get(&self, provider: ProviderId) -> AppResult<&dyn LlmProvider> {
        self.providers
            .get(&provider)
            .map(|p| p.as_ref())
            .ok_or_else(|| {
                AppError::Provider(format!(
                    "{} is not implemented yet",
                    provider.display_name()
                ))
            })
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_implemented_providers() {
        let registry = ProviderRegistry::new();

        assert!(registry.get(ProviderId::OpenAi).is_ok());
    }

    #[test]
    fn reports_a_clear_error_for_unimplemented_providers() {
        let registry = ProviderRegistry::new();

        // `.err().unwrap()` rather than `.unwrap_err()`: the latter requires the Ok type
        // (`&dyn LlmProvider` here) to implement `Debug`, which it doesn't.
        let err = registry.get(ProviderId::Anthropic).err().unwrap();
        assert!(matches!(err, AppError::Provider(_)));
    }
}
