use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::id::id_as_string;

/// Identifies an `Experiment` once it has been persisted. See `RunId` for why this wraps `i64`
/// but crosses IPC as a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub struct ExperimentId(
    #[serde(with = "id_as_string")]
    #[specta(type = String)]
    pub i64,
);

/// A group of Runs belonging to the same logical test — see docs/start.md's Experiment section.
///
/// A side-by-side comparison (multiple Provider+Model columns run against the same prompt) is
/// simply an Experiment with several Runs; a solo Playground execution is a Run with no
/// Experiment at all. There is no separate "comparison" concept in the domain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Experiment {
    pub id: ExperimentId,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
