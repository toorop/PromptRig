use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::model::GenerationParams;
use super::provider::ProviderId;

/// Identifies a `Run` once it has been persisted.
///
/// Wrapping the raw `i64` (SQLite's rowid) in a newtype stops us from accidentally passing a
/// `RunId` where an `ExperimentId` (or any other bare integer) was expected — the compiler will
/// reject it. Assigning the actual value is the storage layer's job (added in a later step).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RunId(pub i64);

/// Identifies an `Experiment` (a group of Runs — see docs/start.md). The full `Experiment`
/// domain type is introduced when side-by-side comparison is implemented; `Run` only needs the
/// id to record which experiment, if any, it belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExperimentId(pub i64);

/// Token counts reported by a provider for one generation call.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// What a provider's `generate()` call returns on success.
///
/// This is deliberately separate from `Run`: `RunResult` is just "what came back from the API
/// call", while `Run` is the full persisted record (request + result). Keeping them separate
/// means a provider implementation never needs to know about `Experiment`s, ids, or storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub text: String,
    /// `None` when the provider's response didn't include usage information.
    pub usage: Option<Usage>,
    pub duration_ms: u64,
    /// Time to first token. `None` until streaming is implemented (see docs/start.md).
    pub ttft_ms: Option<u64>,
}

/// A single execution of Provider + Model + System Prompt + User Prompt + parameters.
///
/// A Run is the atomic unit the whole app is built around: a solo Playground execution is a Run
/// with `experiment_id: None`, and a side-by-side comparison is simply several Runs sharing the
/// same `experiment_id`. The side-by-side UI (a later step) introduces no separate "comparison"
/// concept in the domain — it's just multiple Runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}
