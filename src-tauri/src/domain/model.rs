use serde::{Deserialize, Serialize};

use super::provider::ProviderId;

/// Which generation parameters a given model actually supports.
///
/// The UI uses this to decide which controls to show — see docs/start.md: "Ne force pas
/// l'affichage d'un paramètre qu'un fournisseur ou un modèle ne supporte pas." Pricing and
/// context window are handled separately (see `pricing/`, added in a later step) so that the
/// pricing table can be updated independently of this registry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub supports_temperature: bool,
    pub supports_top_p: bool,
    pub supports_max_tokens: bool,
    /// Whether the provider integration can stream this model's output. Not used until
    /// streaming is implemented, but part of the shape from day one (see docs/start.md's
    /// streaming section).
    pub supports_streaming: bool,
}

/// A model available from a given provider, along with what it supports.
///
/// Instances come from the Model Registry (a later step): either fetched dynamically from a
/// provider's API and cached, or from a small bundled static list for providers/models that
/// don't expose one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub provider: ProviderId,
    /// The exact identifier the provider's API expects, e.g. `"gpt-4o-mini"`.
    pub model_id: String,
    /// Human-readable name for the UI, e.g. `"GPT-4o mini"`.
    pub display_name: String,
    pub capabilities: ModelCapabilities,
    /// Context window size in tokens, when known.
    pub context_window: Option<u32>,
}

/// Generation parameters as configured by the user for a Run.
///
/// Every field is optional: the frontend only sets the ones a model's `ModelCapabilities`
/// mark as supported, and a provider module only includes the fields its API accepts when
/// building the actual HTTP request.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GenerationParams {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_params_default_to_unset() {
        let params = GenerationParams::default();

        assert_eq!(params.temperature, None);
        assert_eq!(params.top_p, None);
        assert_eq!(params.max_tokens, None);
    }
}
