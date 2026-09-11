# Adding a provider

PromptRig currently supports OpenAI, Anthropic, Google Gemini, Mistral, and OpenRouter. Adding
another one (or building the deferred generic OpenAI-compatible endpoint) follows the same
shape every time. This walks through it using a real example — treat `providers/mistral.rs` as
the simplest reference implementation to copy from (it has the fewest wire-format quirks).

See [docs/architecture.md](./architecture.md#provider-abstraction) for why the trait is shaped
the way it is before diving in here.

## 1. Add a `ProviderId` variant

In `src-tauri/src/domain/provider.rs`:

```rust
pub enum ProviderId {
    // ...
    #[serde(rename = "your_provider")]
    YourProvider,
}
```

Update `ProviderId::ALL`, `as_str()`, `display_name()`, and `parse()`'s match arms. The existing
test `as_str_matches_serde_representation` will fail loudly if the `#[serde(rename = ...)]`
string and `as_str()`'s string ever disagree — keep them identical. `as_str()`'s value becomes
the SQLite `provider` column value and the OS keyring account name for this provider, so once
shipped, **never change it** for an existing variant.

## 2. Write the provider module

Create `src-tauri/src/providers/your_provider.rs`:

```rust
pub struct YourProviderProvider {
    http: reqwest::Client,
}

impl YourProviderProvider {
    pub fn new() -> Self {
        Self { http: reqwest::Client::new() }
    }
}

#[async_trait]
impl LlmProvider for YourProviderProvider {
    async fn test_connection(&self, api_key: &str) -> AppResult<()> { /* ... */ }
    async fn list_models(&self, api_key: &str) -> AppResult<Vec<ModelInfo>> { /* ... */ }
    async fn generate(&self, api_key: &str, model_id: &str, system_prompt: &str,
                       user_prompt: &str, params: &GenerationParams) -> AppResult<RunResult> {
        /* ... */
    }
}
```

Things every existing provider had to figure out for its specific API — check the real vendor
docs, don't assume any of these:

- **Auth header shape.** Most are `Authorization: Bearer <key>` (`reqwest`'s `.bearer_auth()`).
  Anthropic uses `x-api-key` plus a mandatory `anthropic-version` header. Gemini uses
  `x-goog-api-key`. Verify against the current docs — don't guess from a "similar" provider.
- **Where the model id goes.** Most APIs take it as a `model` field in the JSON body. Gemini
  puts it in the URL path instead (`.../models/{id}:generateContent`).
- **Where the system prompt goes.** Most APIs accept a `{"role": "system", ...}` message.
  Anthropic wants it as a separate top-level `system` field.
- **Which `GenerationParams` fields the model actually accepts**, and what happens if you send
  one it doesn't — Anthropic rejects `temperature`/`top_p` outright (HTTP 400) for current
  models; OpenAI's reasoning models (`o1`, `o3`, ...) reject `temperature`/`top_p` too and want
  `max_completion_tokens` instead of `max_tokens`. Reflect this in `list_models`'s
  `ModelCapabilities` (so the UI doesn't even show the control) *and* in `generate`'s request
  building (so a stale/cached capability can't send something the API will reject).
- **How capabilities are discovered.** The best case (Mistral, OpenRouter, Gemini) is that
  `/models` reports them directly — no guessing. The worst case (OpenAI) is that the API
  reports nothing about capabilities, so `list_models` has to infer them from the model id
  string, which will drift as the vendor ships new naming conventions.
- **Pagination / filtering.** Some `/models` endpoints return things that aren't actually
  chat-completion models (embeddings, moderation, audio, ...) — filter those out rather than
  showing them as selectable.

Extract the pure, no-I/O parts (building the request body, inferring capabilities from a model
id, deciding which params to include) into standalone functions the way every existing provider
does (`build_request`, `capabilities_from`, `is_reasoning_model`, ...) — these are what actually
get unit-tested; the HTTP-calling methods themselves generally aren't.

## 3. Register it

In `src-tauri/src/providers/mod.rs`:

```rust
pub mod your_provider;
// ...
providers.insert(ProviderId::YourProvider, Box::new(your_provider::YourProviderProvider::new()));
```

That's the entire integration point. `commands/providers.rs`, `commands/runs.rs`,
`commands/experiments.rs`, and every frontend view are already generic over `ProviderId` and
`LlmProvider` — none of them need to change.

## 4. Write tests

At minimum, mirror the existing providers' test shape:
- Request-body shape for at least the "normal" case and any special case (a param that gets
  omitted, a reasoning-model-style restriction, etc.) — assert against the serialized JSON, not
  against a live API call.
- Any capability-inference heuristic, with a couple of representative model ids.

Run `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` in
`src-tauri/` before considering the provider done (see [docs/development.md](./development.md)).

## 5. Pricing (usually nothing to do)

Cost estimation comes from [models.dev](https://models.dev)'s public model registry (see
`pricing/models_dev.rs`), looked up by `(ProviderId, model_id)` — nothing to add here for a new
provider unless models.dev doesn't cover it under any of its provider slugs yet
(`provider_slug` in that file), in which case pricing simply shows "—" until it does.

## 6. Test manually against the real API

Automated tests only cover request/response *shape*, not that the real API actually accepts
what you're sending. Configure a real API key in Settings, run a prompt in the Playground, and
confirm: the model list loads, a generation succeeds, token usage and (if applicable) cost show
up correctly, and an intentionally-bad request (e.g. an invalid model id) produces a readable
error rather than a panic.
