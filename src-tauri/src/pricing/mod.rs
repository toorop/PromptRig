//! Estimated cost per Run, computed from token usage and per-model pricing sourced from
//! [models.dev](https://models.dev) — see `models_dev`'s module doc for the full rationale
//! (why it replaced an earlier OpenRouter-based design, and why it needs no fuzzy id matching).

pub mod models_dev;

use crate::domain::Usage;

pub use models_dev::ModelsDevCache;

/// A per-million-token rate. `models_dev::Cost` is the wire format this is built from.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ModelPrice {
    pub(crate) input_per_million_usd: f64,
    pub(crate) output_per_million_usd: f64,
}

impl ModelPrice {
    pub(crate) fn cost(&self, usage: &Usage) -> f64 {
        let input_cost = (usage.input_tokens as f64 / 1_000_000.0) * self.input_per_million_usd;
        let output_cost = (usage.output_tokens as f64 / 1_000_000.0) * self.output_per_million_usd;
        input_cost + output_cost
    }
}
