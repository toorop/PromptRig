use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::experiment::ExperimentId;
use super::id::id_as_string;
use super::model::GenerationParams;
use super::provider::ProviderId;

/// Identifies a `Run` once it has been persisted.
///
/// Wrapping the raw `i64` (SQLite's rowid) in a newtype stops us from accidentally passing a
/// `RunId` where an `ExperimentId` (or any other bare integer) was expected — the compiler will
/// reject it. Assigning the actual value is the storage layer's job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub struct RunId(
    #[serde(with = "id_as_string")]
    #[specta(type = String)]
    pub i64,
);

/// Token counts reported by a provider for one generation call.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// What a provider's `generate()` call returns on success.
///
/// This is deliberately separate from `Run`: `RunResult` is just "what came back from the API
/// call", while `Run` is the full persisted record (request + result). Keeping them separate
/// means a provider implementation never needs to know about `Experiment`s, ids, or storage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct RunResult {
    pub text: String,
    /// `None` when the provider's response didn't include usage information.
    pub usage: Option<Usage>,
    /// `u32` is plenty for a millisecond duration (up to ~49 days) and, unlike `u64`, is safe to
    /// export straight to TypeScript (see `domain::id::id_as_string` for why that distinction
    /// matters for the id types).
    pub duration_ms: u32,
    /// Time to first token. `None` until streaming is implemented (see docs/start.md).
    pub ttft_ms: Option<u32>,
}

/// A single execution of Provider + Model + System Prompt + User Prompt + parameters.
///
/// A Run is the atomic unit the whole app is built around: a solo Playground execution is a Run
/// with `experiment_id: None`, and a side-by-side comparison is simply several Runs sharing the
/// same `experiment_id`. The side-by-side UI (a later step) introduces no separate "comparison"
/// concept in the domain — it's just multiple Runs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Run {
    pub id: RunId,
    pub experiment_id: Option<ExperimentId>,
    pub provider: ProviderId,
    pub model_id: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub params: GenerationParams,
    pub started_at: DateTime<Utc>,
    /// Present when the call succeeded. Mutually exclusive with `error`.
    pub result: Option<RunResult>,
    /// Present when the call failed. Mutually exclusive with `result`.
    pub error: Option<String>,
    /// Estimated cost in USD, computed from `result.usage` by the pricing module (a later step).
    /// `None` when there's no usage yet, or no pricing data for this model.
    pub estimated_cost_usd: Option<f64>,
    /// Always `false` today — `pricing::models_dev` gives each provider's own real price
    /// directly, no cross-provider approximation involved anymore (see its module doc for what
    /// this replaced). Kept, along with the frontend's "≈"/disclosure-tooltip handling for it,
    /// in case a future pricing gap ever needs a lower-confidence fallback again.
    pub cost_is_estimate: bool,
}
