//! Tauri commands: the thin boundary between the frontend and the backend. Each command just
//! translates its arguments and delegates to a domain/provider/storage/secrets function —
//! business logic itself lives in those modules, not here.

pub mod experiments;
pub mod providers;
pub mod runs;
pub mod secrets;

use chrono::{DateTime, Utc};

use crate::domain::{AppError, AppResult, ExperimentId, GenerationParams, ProviderId, RunResult};
use crate::pricing::{self, OpenRouterPricingCache, PricingTable};
use crate::secrets as secrets_store;
use crate::storage::runs_repo::NewRun;

/// Looks up the API key for `provider`, turning "none configured" into a clear error instead of
/// letting a provider call fail with a confusing auth error. Shared by every command that needs
/// to actually talk to a provider (`providers::test_provider_connection`, `providers::
/// list_models`, `runs::run_generation`, `experiments::run_experiment`).
fn require_api_key(provider: ProviderId) -> AppResult<String> {
    secrets_store::get_api_key(provider)?.ok_or_else(|| {
        AppError::InvalidInput(format!(
            "No API key configured for {}. Add one in Settings first.",
            provider.display_name()
        ))
    })
}

/// Turns a provider call's outcome into the row `runs_repo::insert_run` expects: a success
/// becomes `result`, a failure becomes `error` (the Run is persisted either way — see
/// docs/start.md, a Run records "erreur éventuelle"), and the cost is estimated from whatever
/// usage came back, if any. Shared by `runs::run_generation` (one Run) and
/// `experiments::run_experiment` (several Runs sharing one Experiment) — the two commands differ
/// in how they get `outcome` and in error handling *before* this point (a missing API key aborts
/// a solo Playground run outright, but shouldn't sink an entire comparison — see
/// `experiments::run_column`), but converting a finished outcome into a `NewRun` is identical
/// either way.
///
/// Cost is resolved in three steps, cheapest/most-trustworthy first: our own hand-curated
/// `pricing.json` (exact), then — for a Run against OpenRouter itself — OpenRouter's own real
/// price for that model (also exact, just fetched dynamically), then — for every other
/// provider — an approximation derived from OpenRouter's price for what looks like the same
/// model elsewhere (see `pricing::openrouter_fallback`; always flagged via `cost_is_estimate`).
#[allow(clippy::too_many_arguments)]
async fn build_new_run(
    experiment_id: Option<ExperimentId>,
    provider: ProviderId,
    model_id: String,
    system_prompt: String,
    user_prompt: String,
    params: GenerationParams,
    started_at: DateTime<Utc>,
    outcome: AppResult<RunResult>,
    pricing: &PricingTable,
    openrouter_pricing: &OpenRouterPricingCache,
) -> NewRun {
    let (result, error) = match outcome {
        Ok(result) => (Some(result), None),
        Err(err) => (None, Some(err.to_string())),
    };

    let usage = result.as_ref().and_then(|result| result.usage);
    let (estimated_cost_usd, cost_is_estimate) = match usage {
        None => (None, false),
        Some(usage) => {
            match pricing::resolve_rate(provider, &model_id, pricing, openrouter_pricing).await {
                Some((rate, is_estimate)) => (Some(rate.cost(&usage)), is_estimate),
                None => (None, false),
            }
        }
    };

    NewRun {
        experiment_id,
        provider,
        model_id,
        system_prompt,
        user_prompt,
        params,
        started_at,
        result,
        error,
        estimated_cost_usd,
        cost_is_estimate,
    }
}
