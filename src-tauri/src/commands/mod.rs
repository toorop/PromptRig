//! Tauri commands: the thin boundary between the frontend and the backend. Each command just
//! translates its arguments and delegates to a domain/provider/storage/secrets function —
//! business logic itself lives in those modules, not here.

pub mod experiments;
pub mod providers;
pub mod runs;
pub mod secrets;

use chrono::{DateTime, Utc};

use crate::domain::{AppError, AppResult, ExperimentId, GenerationParams, ProviderId, RunResult};
use crate::pricing::ModelsDevCache;
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
/// usage came back, if any, via models.dev (see `pricing::models_dev`). Shared by
/// `runs::run_generation` (one Run) and `experiments::run_experiment` (several Runs sharing one
/// Experiment) — the two commands differ in how they get `outcome` and in error handling
/// *before* this point (a missing API key aborts a solo Playground run outright, but shouldn't
/// sink an entire comparison — see `experiments::run_column`), but converting a finished outcome
/// into a `NewRun` is identical either way.
///
/// `cost_is_estimate` is always `false` now — models.dev gives each provider's own real price
/// directly, unlike the earlier OpenRouter-based cross-provider approximation this replaced. The
/// field (and the frontend's "≈" disclosure) stays in place rather than being ripped out, in
/// case a future pricing gap ever needs a lower-confidence fallback again.
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
    models_dev: &ModelsDevCache,
) -> NewRun {
    let (result, error) = match outcome {
        Ok(result) => (Some(result), None),
        Err(err) => (None, Some(err.to_string())),
    };

    let usage = result.as_ref().and_then(|result| result.usage);
    let estimated_cost_usd = match usage {
        None => None,
        Some(usage) => models_dev
            .price(provider, &model_id)
            .await
            .map(|rate| rate.cost(&usage)),
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
        cost_is_estimate: false,
    }
}
