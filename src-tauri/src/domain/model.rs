use serde::{Deserialize, Serialize};

use super::provider::ProviderId;

/// Which generation parameters a given model actually supports.
///
/// The UI uses this to decide which controls to show — see docs/start.md: "Ne force pas
/// l'affichage d'un paramètre qu'un fournisseur ou un modèle ne supporte pas." Pricing and
/// context window are handled separately (see `pricing/`, added in a later step) so that the
/// pricing table can be updated independently of this registry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, specta::Type)]
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
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ModelInfo {
    pub provider: ProviderId,
    /// The exact identifier the provider's API expects, e.g. `"gpt-4o-mini"`.
    pub model_id: String,
    /// Human-readable name for the UI, e.g. `"GPT-4o mini"`.
    pub display_name: String,
    pub capabilities: ModelCapabilities,
    /// Context window size in tokens, when known.
    pub context_window: Option<u32>,
    /// A per-token rate, shown next to the model in the picker so the cost of a choice is
    /// visible *before* running anything — unlike `Run.estimated_cost_usd`, which only exists
    /// once a Run has real token usage to multiply against. Filled in by
    /// `commands::providers::list_models` (see `pricing::models_dev`), not by the provider
    /// modules themselves — `providers::*::list_models()` stays unaware pricing exists at all,
    /// same separation as `Run`'s own cost estimation.
    pub pricing: Option<ModelPricing>,
    /// The reasoning-effort levels this model accepts (e.g. `["low", "medium", "high"]`), in
    /// whatever order models.dev lists them — `None` when the model has no such control (either
    /// it's not a reasoning model, or it reasons unconditionally with nothing to tune, like most
    /// Gemini models, or — for Anthropic — its older extended-thinking-only generation, which has
    /// no `effort` concept at all). Filled in by `commands::providers::list_models` (see
    /// `pricing::models_dev::ModelsDevCache::reasoning_effort_levels` and
    /// `commands::providers::supports_reasoning_effort_wiring`) for every implemented provider —
    /// each has a real API field this app can send a resolved value to.
    pub reasoning_effort_levels: Option<Vec<String>>,
}

/// A per-million-token rate for display, not a computed cost (see `ModelInfo::pricing`) —
/// compare `pricing::ModelPrice`, the internal type this is built from, and `Run.cost_is_estimate`,
/// which the `is_estimate` field here mirrors.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct ModelPricing {
    pub input_per_million_usd: f64,
    pub output_per_million_usd: f64,
    /// Always `false` today — models.dev (see `pricing::models_dev`) gives each provider's own
    /// real price directly, no cross-provider approximation involved. Kept rather than removed
    /// in case a future pricing gap ever needs a lower-confidence fallback again.
    pub is_estimate: bool,
}

/// Generation parameters as configured by the user for a Run.
///
/// Every field is optional: the frontend only sets the ones a model's `ModelCapabilities`
/// mark as supported, and a provider module only includes the fields its API accepts when
/// building the actual HTTP request.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, specta::Type)]
pub struct GenerationParams {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<u32>,
    /// One of the model's own `ModelInfo::reasoning_effort_levels` (e.g. `"low"`/`"high"`), or
    /// `None` to let the model reason with its own default behavior. The frontend only lets the
    /// user pick a value the *currently selected* model actually reports supporting — see
    /// `ReasoningEffortSelect.vue` — so a provider module can forward this as-is without
    /// re-validating it against a fixed enum here.
    pub reasoning_effort: Option<String>,
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
        assert_eq!(params.reasoning_effort, None);
    }
}
