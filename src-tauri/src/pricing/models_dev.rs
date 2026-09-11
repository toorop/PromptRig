//! Model metadata + pricing from [models.dev](https://models.dev)'s public `api.json` — a
//! community-maintained (MIT-licensed), continuously-updated registry that indexes models by
//! each provider's own native model id.
//!
//! This replaces an earlier design that used OpenRouter's catalog as a cross-provider pricing
//! stand-in, which needed real fuzzy-matching (normalizing case/punctuation, stripping release
//! dates, resolving a rolling `-latest` alias to whichever dated snapshot looked newest) because
//! OpenRouter's own `vendor/model` ids don't line up with a provider's native ids. models.dev
//! doesn't have that problem: verified directly against a real fetch of its `api.json` that even
//! Mistral's rolling aliases (e.g. `ministral-3b-latest`) are present as their own real entries,
//! not just dated snapshots — so a plain `(provider, model_id)` lookup is enough, no fuzzy
//! matching needed at all. Confirmed coverage is broad (>90% of models carry pricing across
//! OpenAI/Anthropic/Google/Mistral/OpenRouter) and cross-checked against this project's own
//! previously hand-curated `pricing.json` (`gpt-4o-mini` matched exactly) before replacing it.
//!
//! Only [`ModelsDevCache::price`] and [`ModelsDevCache::is_deprecated`] are used today (pricing,
//! and filtering deprecated models out of the picker — see `commands::providers::list_models`).
//! The rest of each entry's fields (modalities, reasoning effort levels, tool-call/structured-
//! output support, context limits, ...) are parsed and kept anyway: the user explicitly asked
//! for this groundwork now, ahead of planned features (multimodal-aware UI, reasoning-effort
//! selection) that will want it without redoing the fetch/parse layer.
//!
//! Fetched at most once per app session and kept in memory for the rest of it — restart the app
//! to check for an update. A failed fetch is remembered for the rest of the session rather than
//! retried on every lookup.
//!
//! The parsed result itself is never persisted anywhere (no SQLite, no relational shape to it —
//! see `ModelsDevCache::cache_dir`'s doc), but the *raw response* is: models.dev serves an
//! `ETag`, so each session sends it back as `If-None-Match` and, on a `304 Not Modified`, reuses
//! the on-disk body instead of re-downloading the ~4.5 MB payload. The same on-disk copy is also
//! the fallback whenever the request can't complete at all (offline, DNS failure, a non-2xx
//! status) — a slightly stale *real* price from models.dev is still real data, unlike the old
//! cross-provider approximation this design replaced, so falling back to it beats showing no
//! price for the whole session.

use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(test)]
use std::sync::Mutex as StdMutex;

use reqwest::header::{ETAG, IF_NONE_MATCH};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::domain::{AppError, AppResult, ProviderId};

use super::ModelPrice;

const MODELS_DEV_API_URL: &str = "https://models.dev/api.json";

/// One model's entry as published by models.dev. Field names match the JSON exactly (`#[serde]`
/// renames aren't needed) so this stays a straightforward mirror of the upstream shape.
///
/// Most of these fields have no reader yet — see the module doc for why they're parsed anyway.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub(crate) struct ModelEntry {
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    /// `"deprecated"`, `"beta"`, or absent (stable). `is_deprecated` is the only thing that reads
    /// this today; a future features might want to surface `"beta"` in the UI.
    pub(crate) status: Option<String>,
    pub(crate) attachment: Option<bool>,
    pub(crate) reasoning: Option<bool>,
    pub(crate) reasoning_options: Option<Vec<ReasoningOption>>,
    pub(crate) tool_call: Option<bool>,
    pub(crate) structured_output: Option<bool>,
    pub(crate) temperature: Option<bool>,
    pub(crate) modalities: Option<Modalities>,
    pub(crate) limit: Option<Limit>,
    pub(crate) knowledge: Option<String>,
    pub(crate) release_date: Option<String>,
    pub(crate) open_weights: Option<bool>,
    pub(crate) cost: Option<Cost>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub(crate) struct ReasoningOption {
    #[serde(rename = "type")]
    pub(crate) kind: String,
    /// Absent for option types that aren't a fixed set of levels (e.g. `"toggle"`,
    /// `"budget_tokens"`) — only an `"effort"`-style option lists concrete `values`. Individual
    /// entries can themselves be `null` in the real data (seen for at least one model, meaning
    /// "no explicit level" alongside named ones like `"low"`/`"medium"`/`"high"`).
    pub(crate) values: Option<Vec<Option<String>>>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub(crate) struct Modalities {
    pub(crate) input: Vec<String>,
    pub(crate) output: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub(crate) struct Limit {
    pub(crate) context: Option<u64>,
    pub(crate) input: Option<u64>,
    pub(crate) output: Option<u64>,
}

/// Only the base rate is used (`ModelPrice`) — models.dev also publishes cache/tiered/context-
/// dependent pricing (`cache_read`, `cache_write`, `tiers`, ...) that isn't parsed here since
/// nothing consumes it yet; add fields as needed rather than guessing the shape ahead of time.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Cost {
    pub(crate) input: f64,
    pub(crate) output: f64,
}

#[derive(Debug, Deserialize)]
struct ProviderEntry {
    models: HashMap<String, ModelEntry>,
}

enum LoadState {
    NotLoaded,
    Loaded(HashMap<(ProviderId, String), ModelEntry>),
    Failed,
}

/// The on-disk copy of the last successful fetch: the raw response body plus the `ETag` that
/// came with it. Kept as two plain files (not one JSON file) so the body never needs escaping —
/// it's already exactly what we'd want to re-parse.
struct DiskCache {
    etag: String,
    body: String,
}

pub struct ModelsDevCache {
    http: Client,
    state: Mutex<LoadState>,
    /// Where the disk cache lives — the same app-data directory `Database` stores its SQLite
    /// file in (see `lib.rs`), not the database itself: this is an opaque blob, not relational
    /// data, so it doesn't belong behind `storage::Database`.
    cache_dir: PathBuf,
    // Lets tests inject a pre-built table without a real HTTP call — see openrouter_fallback's
    // predecessor for the same pattern.
    #[cfg(test)]
    test_override: StdMutex<Option<HashMap<(ProviderId, String), ModelEntry>>>,
}

impl ModelsDevCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            http: Client::new(),
            state: Mutex::new(LoadState::NotLoaded),
            cache_dir,
            #[cfg(test)]
            test_override: StdMutex::new(None),
        }
    }

    fn cache_body_path(&self) -> PathBuf {
        self.cache_dir.join("models_dev_cache.json")
    }

    fn cache_etag_path(&self) -> PathBuf {
        self.cache_dir.join("models_dev_cache.etag")
    }

    /// `None` on first run, or if either file is missing/unreadable — treated the same as "no
    /// cache", never as an error.
    fn read_disk_cache(&self) -> Option<DiskCache> {
        let body = std::fs::read_to_string(self.cache_body_path()).ok()?;
        let etag = std::fs::read_to_string(self.cache_etag_path()).ok()?;
        Some(DiskCache { etag, body })
    }

    /// Best-effort: a failure to persist the cache shouldn't fail the fetch that just succeeded.
    fn write_disk_cache(&self, body: &str, etag: &str) {
        let _ = std::fs::write(self.cache_body_path(), body);
        let _ = std::fs::write(self.cache_etag_path(), etag);
    }

    /// The base per-million-token rate for (provider, model), or `None` if models.dev doesn't
    /// have pricing for it (an unmapped provider, an unlisted model, or a listed model with no
    /// `cost` field — all treated the same as "no price available", same as the old design).
    pub(crate) async fn price(&self, provider: ProviderId, model_id: &str) -> Option<ModelPrice> {
        let entry = self.lookup(provider, model_id).await?;
        let cost = entry.cost?;
        Some(ModelPrice {
            input_per_million_usd: cost.input,
            output_per_million_usd: cost.output,
        })
    }

    /// Whether models.dev marks this model as deprecated. Unmapped providers and unlisted models
    /// are *not* considered deprecated — this only hides models models.dev explicitly flagged,
    /// never hides something just because we have no data on it.
    pub(crate) async fn is_deprecated(&self, provider: ProviderId, model_id: &str) -> bool {
        self.lookup(provider, model_id)
            .await
            .and_then(|entry| entry.status)
            .is_some_and(|status| status == "deprecated")
    }

    async fn lookup(&self, provider: ProviderId, model_id: &str) -> Option<ModelEntry> {
        self.with_loaded(|data| data.get(&(provider, model_id.to_string())).cloned())
            .await
    }

    async fn with_loaded<T>(
        &self,
        f: impl FnOnce(&HashMap<(ProviderId, String), ModelEntry>) -> Option<T>,
    ) -> Option<T> {
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

    /// Conditional GET against the disk cache's `ETag`, when there is one: models.dev answers
    /// `304 Not Modified` (no body) when nothing changed, so an unchanged catalog costs one small
    /// request instead of a fresh 4.5 MB download every session. Any failure to reach the server
    /// at all — offline, DNS, a non-2xx status — falls back to the disk cache if one exists,
    /// rather than leaving every model unpriced for the rest of the session; a slightly stale
    /// *real* price from models.dev is still a real price, unlike the old cross-provider
    /// approximation this design replaced.
    async fn fetch(&self) -> AppResult<HashMap<(ProviderId, String), ModelEntry>> {
        let disk_cache = self.read_disk_cache();

        let mut request = self.http.get(MODELS_DEV_API_URL);
        if let Some(cache) = &disk_cache {
            request = request.header(IF_NONE_MATCH, cache.etag.clone());
        }

        let response = match request.send().await {
            Ok(response) => response,
            Err(e) => {
                return match &disk_cache {
                    Some(cache) => build_models_dev_data(&cache.body),
                    None => Err(AppError::Provider(format!(
                        "models.dev request failed: {e}"
                    ))),
                };
            }
        };

        if response.status() == StatusCode::NOT_MODIFIED {
            if let Some(cache) = &disk_cache {
                return build_models_dev_data(&cache.body);
            }
        }

        if !response.status().is_success() {
            let status = response.status();
            return match &disk_cache {
                Some(cache) => build_models_dev_data(&cache.body),
                None => Err(AppError::Provider(format!(
                    "models.dev returned HTTP {status}"
                ))),
            };
        }

        let etag = response
            .headers()
            .get(ETAG)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);

        let body = response
            .text()
            .await
            .map_err(|e| AppError::Provider(format!("unexpected models.dev response: {e}")))?;

        let data = build_models_dev_data(&body)?;

        if let Some(etag) = etag {
            self.write_disk_cache(&body, &etag);
        }

        Ok(data)
    }
}

/// Maps a `ProviderId` to the top-level provider key models.dev uses. `None` for providers that
/// don't map to a single fixed models.dev entry (a generic OpenAI-compatible endpoint could be
/// pointed at anything).
fn provider_slug(provider: ProviderId) -> Option<&'static str> {
    match provider {
        ProviderId::OpenAi => Some("openai"),
        ProviderId::Anthropic => Some("anthropic"),
        ProviderId::Gemini => Some("google"),
        ProviderId::Mistral => Some("mistral"),
        ProviderId::OpenRouter => Some("openrouter"),
        ProviderId::OpenAiCompatible => None,
    }
}

fn build_models_dev_data(body: &str) -> AppResult<HashMap<(ProviderId, String), ModelEntry>> {
    let raw: HashMap<String, ProviderEntry> = serde_json::from_str(body)
        .map_err(|e| AppError::Provider(format!("unexpected models.dev response: {e}")))?;

    let mut entries = HashMap::new();
    for provider in ProviderId::ALL {
        let Some(slug) = provider_slug(provider) else {
            continue;
        };
        let Some(provider_entry) = raw.get(slug) else {
            continue;
        };
        for (model_id, entry) in &provider_entry.models {
            entries.insert((provider, model_id.clone()), entry.clone());
        }
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RESPONSE: &str = r#"{
        "openai": {
            "models": {
                "gpt-4o-mini": {
                    "status": null,
                    "cost": { "input": 0.15, "output": 0.6 }
                },
                "gpt-4-turbo": {
                    "status": "deprecated",
                    "cost": { "input": 10, "output": 30 }
                },
                "some-preview-model": {
                    "status": "beta"
                }
            }
        },
        "mistral": {
            "models": {
                "ministral-3b-latest": {
                    "cost": { "input": 0.04, "output": 0.04 }
                }
            }
        },
        "some-unmapped-vendor": {
            "models": {
                "whatever": { "cost": { "input": 1, "output": 2 } }
            }
        }
    }"#;

    #[test]
    fn builds_a_lookup_keyed_by_provider_and_native_model_id() {
        let data = build_models_dev_data(SAMPLE_RESPONSE).expect("sample response must parse");

        assert!(data.contains_key(&(ProviderId::OpenAi, "gpt-4o-mini".to_string())));
        // The exact real-world case this design fixes: a rolling `-latest` alias present as its
        // own entry, not needing any fuzzy resolution.
        assert!(data.contains_key(&(ProviderId::Mistral, "ministral-3b-latest".to_string())));
        // 3 openai entries + 1 mistral entry; "some-unmapped-vendor" isn't a slug any
        // `ProviderId` maps to, so its model must not show up under any key at all.
        assert_eq!(data.len(), 4);
    }

    #[tokio::test]
    async fn price_reads_the_base_input_output_rate() {
        let cache = ModelsDevCache::new(std::env::temp_dir());
        *cache.test_override.lock().unwrap() =
            Some(build_models_dev_data(SAMPLE_RESPONSE).unwrap());

        let price = cache
            .price(ProviderId::OpenAi, "gpt-4o-mini")
            .await
            .expect("gpt-4o-mini must be priced");

        assert!((price.input_per_million_usd - 0.15).abs() < f64::EPSILON);
        assert!((price.output_per_million_usd - 0.6).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn price_resolves_a_rolling_latest_alias_directly() {
        let cache = ModelsDevCache::new(std::env::temp_dir());
        *cache.test_override.lock().unwrap() =
            Some(build_models_dev_data(SAMPLE_RESPONSE).unwrap());

        let price = cache
            .price(ProviderId::Mistral, "ministral-3b-latest")
            .await
            .expect("the -latest alias must resolve without any fuzzy matching");

        assert!((price.input_per_million_usd - 0.04).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn unknown_model_has_no_price_and_is_not_deprecated() {
        let cache = ModelsDevCache::new(std::env::temp_dir());
        *cache.test_override.lock().unwrap() =
            Some(build_models_dev_data(SAMPLE_RESPONSE).unwrap());

        assert!(cache
            .price(ProviderId::OpenAi, "nonexistent")
            .await
            .is_none());
        assert!(!cache.is_deprecated(ProviderId::OpenAi, "nonexistent").await);
    }

    #[tokio::test]
    async fn is_deprecated_reflects_status_and_beta_is_not_deprecated() {
        let cache = ModelsDevCache::new(std::env::temp_dir());
        *cache.test_override.lock().unwrap() =
            Some(build_models_dev_data(SAMPLE_RESPONSE).unwrap());

        assert!(cache.is_deprecated(ProviderId::OpenAi, "gpt-4-turbo").await);
        assert!(!cache.is_deprecated(ProviderId::OpenAi, "gpt-4o-mini").await);
        assert!(
            !cache
                .is_deprecated(ProviderId::OpenAi, "some-preview-model")
                .await
        );
    }

    /// A fresh, unique directory per test so parallel test runs never share (or race on) the
    /// same cache files.
    fn temp_cache_dir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "promptrig-models-dev-test-{label}-{:?}",
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn disk_cache_round_trips_through_write_and_read() {
        let cache = ModelsDevCache::new(temp_cache_dir("round-trip"));

        cache.write_disk_cache(SAMPLE_RESPONSE, "\"some-etag\"");
        let read_back = cache
            .read_disk_cache()
            .expect("just-written cache must read back");

        assert_eq!(read_back.body, SAMPLE_RESPONSE);
        assert_eq!(read_back.etag, "\"some-etag\"");
    }

    #[test]
    fn disk_cache_is_absent_before_anything_is_written() {
        let cache = ModelsDevCache::new(temp_cache_dir("absent"));

        assert!(cache.read_disk_cache().is_none());
    }
}
