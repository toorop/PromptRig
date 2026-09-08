//! Estimated cost per Run, computed from token usage and a per-model price table.
//!
//! The price table lives in `pricing.json`, not in Rust code, precisely so it can be updated
//! (new models, price changes) independently of the rest of the app — see docs/start.md's Coût
//! section. `pricing.json` is embedded in the binary as a built-in default for now; letting a
//! user override it with a local file (without recompiling) is a natural follow-up once the
//! Tauri app resolves a data directory (Step 5/6), not needed yet.
//!
//! Prices for the models below were sourced from OpenAI's public pricing as of September 2026.
//! They *will* go stale — that's expected and fine, since a missing or outdated entry just
//! means `estimate_cost` returns `None` (see docs/start.md: "il n'est pas nécessaire que tous
//! les providers donnent immédiatement un coût exact").

pub mod openrouter_fallback;

use std::collections::HashMap;

use serde::Deserialize;

use crate::domain::{AppError, AppResult, ProviderId, Usage};

pub use openrouter_fallback::OpenRouterPricingCache;

const DEFAULT_PRICING_JSON: &str = include_str!("pricing.json");

#[derive(Debug, Deserialize)]
struct PricingFile {
    models: Vec<ModelPriceEntry>,
}

#[derive(Debug, Deserialize)]
struct ModelPriceEntry {
    provider: ProviderId,
    model_id: String,
    input_per_million_usd: f64,
    output_per_million_usd: f64,
}

/// Shared by both this module's static table and `openrouter_fallback`'s dynamic one.
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

/// A loaded price table, keyed by (provider, model id).
pub struct PricingTable {
    prices: HashMap<(ProviderId, String), ModelPrice>,
}

impl PricingTable {
    /// The table bundled with the app.
    pub fn load_default() -> AppResult<Self> {
        Self::load_from_str(DEFAULT_PRICING_JSON)
    }

    /// Parses a pricing table from a JSON string — split out from `load_default` so a future
    /// user-supplied override file can be parsed the same way.
    pub fn load_from_str(json: &str) -> AppResult<Self> {
        let file: PricingFile = serde_json::from_str(json)
            .map_err(|e| AppError::InvalidInput(format!("invalid pricing data: {e}")))?;

        let prices = file
            .models
            .into_iter()
            .map(|entry| {
                (
                    (entry.provider, entry.model_id),
                    ModelPrice {
                        input_per_million_usd: entry.input_per_million_usd,
                        output_per_million_usd: entry.output_per_million_usd,
                    },
                )
            })
            .collect();

        Ok(Self { prices })
    }

    /// The raw per-million-token rate for (provider, model), or `None` if it isn't in the
    /// table. Exposed (crate-internal) so `resolve_rate` can share it with the OpenRouter
    /// fallback lookups, and so a rate can be shown in the UI before any usage exists to turn
    /// it into an actual cost (see `ModelInfo::pricing`).
    pub(crate) fn lookup(&self, provider: ProviderId, model_id: &str) -> Option<ModelPrice> {
        self.prices.get(&(provider, model_id.to_string())).copied()
    }

    /// Estimated cost in USD for a completed Run, or `None` if this (provider, model) isn't in
    /// the table.
    pub fn estimate_cost(
        &self,
        provider: ProviderId,
        model_id: &str,
        usage: &Usage,
    ) -> Option<f64> {
        self.lookup(provider, model_id)
            .map(|price| price.cost(usage))
    }
}

/// Resolves the best available per-token rate for (provider, model), trying our own
/// hand-curated table first, then — for OpenRouter itself — its own real price, then — for
/// every other provider — the cross-provider approximation. Returns the rate plus whether it's
/// an estimate. Shared by `commands::build_new_run` (multiplies by real usage to get a cost)
/// and `commands::providers::list_models` (shows the bare rate before anything has run) so the
/// resolution order lives in exactly one place.
pub(crate) async fn resolve_rate(
    provider: ProviderId,
    model_id: &str,
    table: &PricingTable,
    openrouter: &OpenRouterPricingCache,
) -> Option<(ModelPrice, bool)> {
    if let Some(price) = table.lookup(provider, model_id) {
        return Some((price, false));
    }

    if provider == ProviderId::OpenRouter {
        return openrouter
            .lookup_for_openrouter(model_id)
            .await
            .map(|price| (price, false));
    }

    openrouter
        .lookup_fallback(provider, model_id)
        .await
        .map(|price| (price, true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pricing_json_loads_and_parses() {
        // Guards against a typo in pricing.json breaking the app at startup.
        PricingTable::load_default().expect("bundled pricing.json must be valid");
    }

    #[test]
    fn estimates_cost_for_a_known_model() {
        let table = PricingTable::load_default().unwrap();
        let usage = Usage {
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
        };

        let cost = table
            .estimate_cost(ProviderId::OpenAi, "gpt-4o-mini", &usage)
            .expect("gpt-4o-mini must be priced");

        assert!(
            (cost - 0.75).abs() < f64::EPSILON,
            "expected $0.75, got {cost}"
        );
    }

    #[test]
    fn returns_none_for_an_unpriced_model() {
        let table = PricingTable::load_default().unwrap();
        let usage = Usage {
            input_tokens: 100,
            output_tokens: 100,
        };

        assert_eq!(
            table.estimate_cost(ProviderId::OpenAi, "some-future-model", &usage),
            None
        );
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(PricingTable::load_from_str("{ not valid json").is_err());
    }
}
