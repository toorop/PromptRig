//! The application's core domain types: providers, models, generation parameters, and runs.
//!
//! Nothing in this module talks to a network, a database, or the OS keyring — it's plain data
//! and the logic that only depends on that data (see `error.rs` for the one exception, which
//! every other layer builds on). Providers (`providers/`), storage (`storage/`), and secrets
//! (`secrets/`) all produce or consume these types without the domain module knowing they exist.

pub mod error;
pub mod model;
pub mod provider;
pub mod run;

pub use error::{AppError, AppResult};
pub use model::{GenerationParams, ModelCapabilities, ModelInfo};
pub use provider::ProviderId;
pub use run::{ExperimentId, Run, RunId, RunResult, Usage};
