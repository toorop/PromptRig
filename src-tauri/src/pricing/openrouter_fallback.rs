//! Cross-provider cost estimates derived from OpenRouter's own published pricing.
//!
//! Most providers (Anthropic, Gemini, Mistral, ...) don't expose per-model pricing through their
//! own API, so a Run against them shows no cost unless it happens to be hand-entered in our own
//! `pricing.json`. OpenRouter, an aggregator that proxies to (almost) every model out there,
//! *does* publish per-model pricing on its public `/models` endpoint (no API key required for
//! the catalog itself) — so its numbers are used here as a stand-in for the equivalent model on
//! its native provider.
//!
//! This is explicitly an **approximation**, not an exact figure, for two reasons the UI must
//! disclose (see `Run::cost_is_estimate`):
//! 1. OpenRouter takes its own commission on top of the underlying provider's price, so a cost
//!    derived from it is usually a *ceiling* — the real provider price is typically at or below
//!    this estimate, never above it.
//! 2. Matching "this OpenRouter listing" to "this native provider model id" is done by
//!    normalizing both strings and comparing (see `normalize_model_id`) — provider APIs and
//!    OpenRouter don't always name the same model exactly the same way (e.g. a native id often
//!    carries a release date OpenRouter's doesn't), so some real matches will still be missed.
//!    When that happens, this just returns `None`, same as no pricing data existing at all —
//!    never a wrong-but-confident number for the wrong model.
//!
//! The catalog is fetched once per app session (same lifetime as the model-list cache in
//! `stores/providers.ts` on the frontend) and kept in memory — restart the app to refresh it.
//! A failed fetch (offline, OpenRouter down) is remembered for the rest of the session rather
//! than retried on every single Run.

use std::collections::HashMap;

#[cfg(test)]
use std::sync::Mutex as StdMutex;

use reqwest::Client;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::domain::{AppError, AppResult, ProviderId, Usage};

use super::ModelPrice;

const OPENROUTER_MODELS_URL: &str = "https://openrouter.ai/api/v1/models";

/// The lookups built from one fetch of OpenRouter's catalog:
/// - `openrouter_exact`: OpenRouter's own model id (e.g. `"openai/gpt-4o-mini"`) → its real
///   price for calling *OpenRouter itself* with that model — exact, not an estimate.
/// - `fallback_by_provider`: `(target provider, normalized native model id)` → OpenRouter's
///   price for the OpenRouter listing that looked like the equivalent model — an approximation
///   for that *other* provider's own pricing.
/// - `latest_by_provider`: `(target provider, normalized base name)` → the price of whichever
///   dated/versioned OpenRouter listing for that base looks newest — used when the native id is
///   a rolling `-latest` alias (common on Mistral: `mistral-large-latest`,
///   `ministral-3b-latest`, ...) that doesn't itself carry a version to match against.
struct PricingData {
    openrouter_exact: HashMap<String, ModelPrice>,
    fallback_by_provider: HashMap<(ProviderId, String), ModelPrice>,
    latest_by_provider: HashMap<(ProviderId, String), ModelPrice>,
}

enum LoadState {
    NotLoaded,
    Loaded(PricingData),
    Failed,
}

pub struct OpenRouterPricingCache {
    http: Client,
    state: Mutex<LoadState>,
    // Lets tests inject a pre-built table without a real HTTP call. A plain (synchronous)
    // std Mutex is fine here — it's only ever locked-and-immediately-unlocked, never held
    // across an `.await`.
    #[cfg(test)]
    test_override: StdMutex<Option<PricingData>>,
}

impl OpenRouterPricingCache {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
            state: Mutex::new(LoadState::NotLoaded),
            #[cfg(test)]
            test_override: StdMutex::new(None),
        }
    }

    /// Real cost for a Run that itself used the OpenRouter provider — exact, since this *is*
    /// OpenRouter's own price for that model, not a stand-in for someone else's.
    pub async fn estimate_for_openrouter(&self, model_id: &str, usage: &Usage) -> Option<f64> {
        self.lookup_for_openrouter(model_id)
            .await
            .map(|price| price.cost(usage))
    }

    /// Approximate cost for a Run against a *different* provider, derived from whichever
    /// OpenRouter listing looks like the same model. Always an estimate (see module docs).
    pub async fn estimate_fallback(
        &self,
        provider: ProviderId,
        model_id: &str,
        usage: &Usage,
    ) -> Option<f64> {
        self.lookup_fallback(provider, model_id)
            .await
            .map(|price| price.cost(usage))
    }

    /// OpenRouter's own real rate for one of its own listings — the raw counterpart to
    /// `estimate_for_openrouter`, used when only the rate (not a cost) is needed, e.g. to show
    /// in the model picker before anything has run.
    pub(crate) async fn lookup_for_openrouter(&self, model_id: &str) -> Option<ModelPrice> {
        self.with_loaded(|data| data.openrouter_exact.get(model_id).copied())
            .await
    }

    /// The cross-provider approximate rate — the raw counterpart to `estimate_fallback`.
    ///
    /// Tries an exact (date-normalized) match first; if `model_id` is a rolling `-latest` alias
    /// (e.g. Mistral's `ministral-3b-latest`) that didn't match anything directly — OpenRouter
    /// itself only lists dated snapshots like `ministral-3b-2512`, no bare `-latest` entry —
    /// falls back to whichever dated snapshot for that same base name looks newest.
    pub(crate) async fn lookup_fallback(
        &self,
        provider: ProviderId,
        model_id: &str,
    ) -> Option<ModelPrice> {
        let exact_key = (provider, normalize_model_id(model_id));
        let latest_key =
            strip_latest_alias(model_id).map(|base| (provider, normalize_model_id(base)));

        self.with_loaded(|data| {
            data.fallback_by_provider
                .get(&exact_key)
                .or_else(|| {
                    latest_key
                        .as_ref()
                        .and_then(|key| data.latest_by_provider.get(key))
                })
                .copied()
        })
        .await
    }

    /// Loads the catalog on first use (real fetch, or a test override if one was set), then
    /// runs `f` against it. Returns `None` without calling `f` if loading never succeeded.
    async fn with_loaded<T>(&self, f: impl FnOnce(&PricingData) -> Option<T>) -> Option<T> {
        let mut guard = self.state.lock().await;

        #[cfg(test)]
        if matches!(*guard, LoadState::NotLoaded) {
            if let Some(data) = self.test_override.lock().unwrap().take() {
                *guard = LoadState::Loaded(data);
            }
        }

        if matches!(*guard, LoadState::NotLoaded) {
            *guard = match self.fetch().await {
                Ok(data) => LoadState::Loaded(data),
                Err(_) => LoadState::Failed,
            };
        }

        match &*guard {
            LoadState::Loaded(data) => f(data),
            LoadState::NotLoaded | LoadState::Failed => None,
        }
    }

    async fn fetch(&self) -> AppResult<PricingData> {
        let response = self
            .http
            .get(OPENROUTER_MODELS_URL)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("OpenRouter request failed: {e}")))?;

        let body = response
            .text()
            .await
            .map_err(|e| AppError::Provider(format!("unexpected OpenRouter response: {e}")))?;

        build_pricing_data(&body)
    }
}

impl Default for OpenRouterPricingCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Maps a `ProviderId` to the vendor slug OpenRouter prefixes its model ids with. `None` for
/// providers that don't map to a single fixed OpenRouter vendor (OpenRouter itself doesn't need
/// this — see `estimate_for_openrouter` — and a generic OpenAI-compatible endpoint could be
/// pointed at anything).
fn vendor_slug(provider: ProviderId) -> Option<&'static str> {
    match provider {
        ProviderId::OpenAi => Some("openai"),
        ProviderId::Anthropic => Some("anthropic"),
        ProviderId::Gemini => Some("google"),
        ProviderId::Mistral => Some("mistralai"),
        ProviderId::OpenRouter | ProviderId::OpenAiCompatible => None,
    }
}

fn provider_for_vendor_slug(slug: &str) -> Option<ProviderId> {
    [
        ProviderId::OpenAi,
        ProviderId::Anthropic,
        ProviderId::Gemini,
        ProviderId::Mistral,
    ]
    .into_iter()
    .find(|&provider| vendor_slug(provider) == Some(slug))
}

/// Normalizes a model id for fuzzy cross-provider matching: lowercase, strip everything but
/// letters/digits, and drop a trailing release-date suffix (providers often append one, e.g.
/// `claude-opus-4-5-20260514`, that OpenRouter's own listing for the same model doesn't carry).
/// Two different real models normalizing to the same string is possible but unlikely in
/// practice — worth knowing about if a mismatch is ever reported.
fn normalize_model_id(id: &str) -> String {
    let without_date = strip_trailing_date(id);
    without_date
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Strips a trailing `-YYYYMMDD` (or `YYYYMMDD` with no separator) if the id ends with exactly
/// 8 digits — so we don't chew into a version number like `gpt-4o` or a short numeric suffix
/// that isn't actually a date.
fn strip_trailing_date(id: &str) -> &str {
    let digits_at_end = id.chars().rev().take_while(|c| c.is_ascii_digit()).count();
    if digits_at_end == 8 {
        let cut = id.len() - 8;
        id[..cut].trim_end_matches('-')
    } else {
        id
    }
}

/// If `id` ends with a rolling `-latest` alias (Mistral's convention: `mistral-large-latest`,
/// `ministral-3b-latest`, ...), returns the base name without it.
fn strip_latest_alias(id: &str) -> Option<&str> {
    id.strip_suffix("-latest")
        .or_else(|| id.strip_suffix("-Latest"))
        .or_else(|| id.strip_suffix("-LATEST"))
}

/// Splits a trailing dash-separated run of 2–8 digits off `id` (a version-ish suffix like
/// Mistral's `-2512` snapshot tag or a full `-20260514` release date), returning the base and
/// the digits parsed as a number so multiple candidates for the same base can be compared —
/// larger means newer, true for both `YYMM`-style and `YYYYMMDD`-style suffixes. `None` if the
/// id doesn't end in a separated digit run of that length (e.g. a bare `mistral-large` with no
/// version at all, which we can't rank against dated siblings and so simply skip).
fn trailing_numeric_suffix(id: &str) -> Option<(&str, u64)> {
    let digits_at_end = id.chars().rev().take_while(|c| c.is_ascii_digit()).count();
    if !(2..=8).contains(&digits_at_end) {
        return None;
    }
    let cut = id.len() - digits_at_end;
    let base = id[..cut].strip_suffix('-')?;
    let version: u64 = id[cut..].parse().ok()?;
    Some((base, version))
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelListing>,
}

#[derive(Deserialize)]
struct ModelListing {
    id: String,
    #[serde(default)]
    pricing: Option<Pricing>,
}

#[derive(Deserialize)]
struct Pricing {
    prompt: Option<String>,
    completion: Option<String>,
}

fn build_pricing_data(body: &str) -> AppResult<PricingData> {
    let parsed: ModelsResponse = serde_json::from_str(body)
        .map_err(|e| AppError::Provider(format!("unexpected OpenRouter response: {e}")))?;

    let mut openrouter_exact = HashMap::new();
    let mut fallback_by_provider = HashMap::new();
    // Tracks, per (provider, normalized base), the highest version number seen so far and its
    // price — folded down to just the price in the returned `PricingData` once every listing
    // has been considered.
    let mut latest_candidates: HashMap<(ProviderId, String), (u64, ModelPrice)> = HashMap::new();

    for listing in parsed.data {
        let Some(price) = parse_price(&listing.pricing) else {
            continue;
        };

        if let Some((vendor, native_id)) = listing.id.split_once('/') {
            if let Some(target_provider) = provider_for_vendor_slug(vendor) {
                let key = (target_provider, normalize_model_id(native_id));
                // First match wins on a collision — good enough for a best-effort estimate.
                fallback_by_provider.entry(key).or_insert(price);

                if let Some((base, version)) = trailing_numeric_suffix(native_id) {
                    let base_key = (target_provider, normalize_model_id(base));
                    let is_newer = latest_candidates
                        .get(&base_key)
                        .is_none_or(|&(best_version, _)| version > best_version);
                    if is_newer {
                        latest_candidates.insert(base_key, (version, price));
                    }
                }
            }
        }

        openrouter_exact.insert(listing.id, price);
    }

    let latest_by_provider = latest_candidates
        .into_iter()
        .map(|(key, (_, price))| (key, price))
        .collect();

    Ok(PricingData {
        openrouter_exact,
        fallback_by_provider,
        latest_by_provider,
    })
}

fn parse_price(pricing: &Option<Pricing>) -> Option<ModelPrice> {
    let pricing = pricing.as_ref()?;
    let prompt_per_token: f64 = pricing.prompt.as_ref()?.parse().ok()?;
    let completion_per_token: f64 = pricing.completion.as_ref()?.parse().ok()?;

    Some(ModelPrice {
        input_per_million_usd: prompt_per_token * 1_000_000.0,
        output_per_million_usd: completion_per_token * 1_000_000.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RESPONSE: &str = r#"{
        "data": [
            {
                "id": "openai/gpt-4o-mini",
                "pricing": { "prompt": "0.00000015", "completion": "0.0000006" }
            },
            {
                "id": "anthropic/claude-opus-4.5",
                "pricing": { "prompt": "0.000005", "completion": "0.000025" }
            },
            {
                "id": "some-vendor/unpriced-model",
                "pricing": { "prompt": null, "completion": null }
            },
            {
                "id": "unmapped-vendor/some-model",
                "pricing": { "prompt": "0.000001", "completion": "0.000002" }
            },
            {
                "id": "mistralai/ministral-3b-2407",
                "pricing": { "prompt": "0.0000002", "completion": "0.0000002" }
            },
            {
                "id": "mistralai/ministral-3b-2512",
                "pricing": { "prompt": "0.0000001", "completion": "0.0000001" }
            }
        ]
    }"#;

    #[test]
    fn normalizes_by_stripping_case_punctuation_and_trailing_dates() {
        assert_eq!(
            normalize_model_id("claude-opus-4-5-20260514"),
            normalize_model_id("claude-opus-4.5")
        );
        assert_eq!(normalize_model_id("GPT-4o-Mini"), "gpt4omini");
    }

    #[test]
    fn does_not_strip_short_numeric_suffixes_that_are_not_dates() {
        // "gpt-4o" ends in 2 digits, not 8 — must be left alone.
        assert_eq!(strip_trailing_date("gpt-4o"), "gpt-4o");
    }

    #[test]
    fn strips_the_latest_alias_suffix() {
        assert_eq!(
            strip_latest_alias("ministral-3b-latest"),
            Some("ministral-3b")
        );
        assert_eq!(strip_latest_alias("mistral-large-2407"), None);
    }

    #[test]
    fn extracts_a_trailing_version_number_for_comparison() {
        assert_eq!(
            trailing_numeric_suffix("ministral-3b-2512"),
            Some(("ministral-3b", 2512))
        );
        assert_eq!(
            trailing_numeric_suffix("claude-opus-4-5-20260514"),
            Some(("claude-opus-4-5", 20260514))
        );
        // No separated trailing digit run at all -> nothing to rank against siblings.
        assert_eq!(trailing_numeric_suffix("mistral-large"), None);
    }

    #[test]
    fn builds_both_lookups_from_a_sample_response() {
        let data = build_pricing_data(SAMPLE_RESPONSE).expect("sample response must parse");

        assert!(data.openrouter_exact.contains_key("openai/gpt-4o-mini"));
        assert!(data
            .openrouter_exact
            .contains_key("anthropic/claude-opus-4.5"));
        // No usable price -> skipped entirely, not inserted as a zero-cost entry.
        assert!(!data
            .openrouter_exact
            .contains_key("some-vendor/unpriced-model"));

        let key = (
            ProviderId::Anthropic,
            normalize_model_id("claude-opus-4-5-20260514"),
        );
        assert!(data.fallback_by_provider.contains_key(&key));

        // A vendor slug we don't map to any ProviderId must not show up in the fallback table
        // (it can't be "someone else's" price if we don't know whose model it approximates).
        assert!(data.fallback_by_provider.values().count() < data.openrouter_exact.len());
    }

    #[tokio::test]
    async fn estimate_for_openrouter_is_exact_and_estimate_fallback_is_approximate() {
        let cache = OpenRouterPricingCache::new();
        let data = build_pricing_data(SAMPLE_RESPONSE).unwrap();
        *cache.test_override.lock().unwrap() = Some(data);

        let usage = Usage {
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
        };

        let exact = cache
            .estimate_for_openrouter("openai/gpt-4o-mini", &usage)
            .await
            .expect("must find the exact OpenRouter listing");
        assert!((exact - 0.75).abs() < 1e-9);

        let approx = cache
            .estimate_fallback(ProviderId::Anthropic, "claude-opus-4-5-20260514", &usage)
            .await
            .expect("must fuzzy-match the Anthropic listing");
        assert!((approx - 30.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn latest_alias_resolves_to_the_newest_dated_snapshot() {
        // Mirrors the real bug report: Mistral's API returns a rolling `ministral-3b-latest`
        // alias, but OpenRouter only lists dated snapshots (`-2407`, `-2512`) — never a bare
        // `-latest` entry — so a direct normalized match never hits. The estimate should still
        // resolve, using whichever dated snapshot looks newest (2512 > 2407 here).
        let cache = OpenRouterPricingCache::new();
        let data = build_pricing_data(SAMPLE_RESPONSE).unwrap();
        *cache.test_override.lock().unwrap() = Some(data);

        let usage = Usage {
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
        };

        let cost = cache
            .estimate_fallback(ProviderId::Mistral, "ministral-3b-latest", &usage)
            .await
            .expect("must resolve via the newest dated snapshot");

        // The 2512 snapshot ($0.0000001/$0.0000001), not the older 2407 one ($0.0000002/...).
        assert!((cost - 0.2).abs() < 1e-9);
    }

    #[tokio::test]
    async fn unknown_model_returns_none_rather_than_a_wrong_price() {
        let cache = OpenRouterPricingCache::new();
        let data = build_pricing_data(SAMPLE_RESPONSE).unwrap();
        *cache.test_override.lock().unwrap() = Some(data);

        let usage = Usage {
            input_tokens: 100,
            output_tokens: 100,
        };

        assert_eq!(
            cache
                .estimate_fallback(ProviderId::Mistral, "mistral-large-latest", &usage)
                .await,
            None
        );
    }
}
