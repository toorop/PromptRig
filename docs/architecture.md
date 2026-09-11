# Architecture

PromptRig is a [Tauri 2](https://tauri.app/) desktop app: a Rust backend (in `src-tauri/`) that
does everything that matters — talks to LLM providers, stores data, holds secrets — behind a
typed command boundary, and a Vue 3 frontend (in `src/`) that is essentially a thin, reactive
view over that backend. This document explains the structural decisions and why they were made;
see [docs/start.md](./start.md) for the original product spec and [STATE.md](../STATE.md) for a
detailed, chronological build log.

## The core idea: Run + Experiment, nothing else

Everything in the domain model exists to support one atomic unit: a **Run** — a single call to
one Provider + Model with a system prompt, a user prompt, and generation parameters, plus
whatever came back (text, token usage, latency, estimated cost) or the error if it failed.

A side-by-side comparison (the Compare view) introduces **no separate concept**. It is just an
**Experiment**: a named group of Runs that share the same prompts and were launched together.
The Playground's single-model "Run" button creates a Run with `experiment_id: None`; Compare's
"Run all" creates one Experiment and N Runs (one per column) sharing its id. Every Run is
persisted regardless of success or failure — a failed call is still useful data (what was tried,
what error came back), not something to discard.

This shows up directly in the schema (`src-tauri/src/storage/migrations/0001_initial.sql`):
`runs.experiment_id` is a nullable foreign key into `experiments`. There is deliberately no
richer "comparison" table or UI-level concept layered on top.

## Backend structure (`src-tauri/src/`)

```
domain/       Plain data types + AppError. No I/O, no Tauri dependency.
providers/    One LlmProvider implementation per vendor + the registry that looks them up.
pricing/      Cost estimation, sourced from models.dev's public model/pricing registry.
storage/      SQLite access — the only part of the app that touches rusqlite.
secrets/      OS-native keyring access — the only part of the app that touches API keys directly.
commands/     Tauri commands: thin glue between the frontend and the modules above.
lib.rs        Wires up managed state (Database, ProviderRegistry, ModelsDevCache, ...) and specta.
```

Each module owns one concern and doesn't reach into the others' internals. `domain` has zero
dependencies on the rest — it's just types (`ProviderId`, `ModelInfo`, `Run`, `AppError`, ...)
that everything else shares.

### Provider abstraction

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn test_connection(&self, api_key: &str) -> AppResult<()>;
    async fn list_models(&self, api_key: &str) -> AppResult<Vec<ModelInfo>>;
    async fn generate(&self, api_key: &str, model_id: &str, system_prompt: &str,
                       user_prompt: &str, params: &GenerationParams) -> AppResult<RunResult>;
}
```

`ProviderRegistry` is a `HashMap<ProviderId, Box<dyn LlmProvider>>` built once at startup.
`async-trait` is what makes an async trait usable as a trait object at all — without it, `Box<dyn
LlmProvider>` with async methods wouldn't compile with today's stable Rust. A provider
implementation never touches the OS keyring or SQLite directly; the command layer looks up the
key via `secrets::get_api_key` and passes it in as a plain `&str`. This keeps every provider
testable with a fake key and independent of how secrets are actually stored.

Adding a new provider is: one new module under `providers/` implementing the trait, one new
`ProviderId` variant, and one line in `ProviderRegistry::new()`. See
[docs/adding-a-provider.md](./adding-a-provider.md) for the full walkthrough — five real
providers (OpenAI, Anthropic, Gemini, Mistral, OpenRouter) already follow this shape, each
with its own real wire-format quirks documented inline.

### Storage: `rusqlite` directly, not `tauri-plugin-sql`

SQLite access goes through `rusqlite` directly, wrapped in a small repository layer
(`storage::runs_repo`, `storage::experiments_repo`), rather than `tauri-plugin-sql`. The plugin
exposes raw SQL to the *frontend*, which would mean the query shape (and therefore the schema)
becomes part of the frontend's contract — undesirable for an app whose whole point is a stable,
typed boundary between UI and business logic. Every query the app ever runs lives in Rust.

`rusqlite` is synchronous; Tauri commands are async. `storage::Database::with_connection` is the
one place that bridges this: it wraps a closure in `tauri::async_runtime::spawn_blocking`, so
repositories can be written as plain, easy-to-read synchronous functions while still being safe
to call from async command handlers. A single `Arc<Mutex<Connection>>` is enough concurrency for
a desktop app talking to its own local file — no connection pool needed.

Migrations are managed by `rusqlite_migration` as a growing list of `M::up(...)` SQL files under
`storage/migrations/`. Never edit an existing migration file — add a new one (see
`0002_add_cost_is_estimate.sql` for an example: adding one nullable-by-default column to an
existing table).

### Secrets: the OS keyring, never plaintext

API keys go through the `keyring` crate, which targets the right native backend per platform at
compile time (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux) with no
extra configuration. `secrets::get_api_key` is deliberately not a registered Tauri command — only
`save_api_key`, `delete_api_key`, and `has_api_key` are exposed to the frontend, so a raw key can
never make it into the WebView or a frontend error message. Providers receive the key as a
function argument, already resolved by the command layer.

### Cost estimation: models.dev, looked up directly by native model id

`pricing::models_dev::ModelsDevCache` fetches [models.dev](https://models.dev)'s public
`api.json` — a community-maintained (MIT-licensed), continuously-updated registry covering
OpenAI/Anthropic/Google/Mistral/OpenRouter and 200+ other providers — at most once per app
session and keeps the parsed result in memory (not SQLite — it's not relational data). The raw
response itself *is* persisted to two small files next to the SQLite database
(`models_dev_cache.json`/`.etag` in the app data dir): models.dev serves an `ETag`, so each
session's first fetch sends it back as `If-None-Match` and, on a `304 Not Modified`, reuses the
on-disk body instead of re-downloading the ~4.5 MB payload every launch. The same on-disk copy is
also the fallback whenever the request can't complete at all (offline, DNS failure, a non-2xx
status) — a slightly stale *real* price from models.dev is still real data, unlike the old
cross-provider approximation this design replaced, so falling back to it beats showing no price
for the whole session. Each entry is keyed by the provider's own **native** model id, so looking up
a price is a single `(ProviderId, model_id)` hash-map lookup: no fuzzy matching, no normalizing
punctuation, no resolving a rolling `-latest` alias to a dated snapshot. That's a deliberate
change from an earlier design (see git history around `pricing::openrouter_fallback`) that used
OpenRouter's own catalog as a cross-provider stand-in and needed real fuzzy-matching machinery
for it, since OpenRouter's `vendor/model` ids don't line up with a provider's native ones —
verified directly against a real fetch of models.dev's `api.json` that it doesn't have that
problem (even Mistral's `-latest` aliases are present as their own real entries) before
replacing the old design outright, including deleting the no-longer-needed hand-curated
`pricing.json` (models.dev already covered the same models, cross-checked to match).
`ModelsDevCache::price` returns `None` when a model genuinely isn't listed — there's no further
approximate fallback beyond that, which is expected and acceptable per docs/start.md; `pricing::
ModelPrice`'s `is_estimate` field (`Run.cost_is_estimate`, `ModelInfo.pricing.is_estimate`) is
kept but always `false` now, in case a future pricing gap ever needs a lower-confidence source
again.

The same fetch also drives `commands::providers::list_models` filtering out any model
models.dev marks `"deprecated"` (a `"beta"` model is still shown) — one lookup per model covers
both the price and the deprecation check.

`pricing::models_dev::ModelEntry` parses more of models.dev's per-model data than pricing/status
alone needs today (modalities, reasoning-effort levels, tool-call/structured-output support,
context limits, ...) — deliberately, ahead of planned features (multimodal-aware UI, a
reasoning-effort selector) that will want it without redoing the fetch/parse layer.

### Rust ↔ TypeScript type sync: `specta` + `tauri-specta`

Every domain type that crosses the IPC boundary derives `specta::Type`, and every command is
annotated `#[specta::specta]`. `tauri_specta::Builder` collects them all
(`lib.rs::specta_builder()`) and, in debug builds only, regenerates `src/lib/bindings.ts` on
every build — full TypeScript types plus a typed `commands.xxx(...)` wrapper for every command.
The frontend never hand-writes a type for backend data or calls `invoke()` directly; it imports
from `@/lib/bindings`. This is what keeps the ever-growing domain model (`Run`, `Experiment`,
`ModelInfo`, ...) from silently drifting between the two languages.

One consequence: `i64`/`u64` can't cross this boundary directly (JavaScript numbers lose
precision above 2^53). `RunId` and `ExperimentId` wrap an `i64` but cross IPC as a string (see
`domain::id::id_as_string`, a small serde `with` module) while staying a plain integer
internally and in SQLite; other numeric fields that don't need arbitrary size (like a
millisecond duration) just use `u32` instead.

## Frontend structure (`src/`)

```
views/          One per route: PlaygroundView, CompareView, SettingsView.
components/     Shared pieces (ResultPanel, CopyButton, LabelHint, ...) and shadcn-vue's ui/.
stores/         Pinia: providers, promptDraft, compare, theme.
lib/bindings.ts Generated — see above. Never edit by hand.
```

State that must survive navigating between views lives in a Pinia store, not component state
(Vue destroys a view's local state when you navigate away and back). `stores/promptDraft.ts`
(system/user prompt + generation params) is shared between Playground and Compare with no
explicit hand-off — editing either view updates the same state. `stores/providers.ts` also holds
a session-lifetime, in-memory cache of each provider's model list (`modelsByProvider`), so
picking the same provider on two Compare columns — or on both Playground and Compare — doesn't
re-fetch; it's cleared on app restart by design (`loadModels(provider, { force: true })` is the
manual "↻ Refresh" action).

The UI is dark-by-default (Nord palette, see `src/assets/main.css`) with a light/dark toggle
persisted per device, self-hosted IBM Plex Sans/Mono (no CDN webfont, no system-font-stack
variance), and a frameless window (`decorations: false`) with a custom title bar
(`data-tauri-drag-region` + a themed close button) instead of the native OS chrome.

## Deferred by design

- **Streaming**: `LlmProvider::generate()` returns a complete response today. The trait is
  shaped so `generate_stream()` (emitting Tauri events like `run:{id}:chunk`) can be added later
  without changing any existing command's signature.
- **A generic OpenAI-compatible endpoint** (`ProviderId::OpenAiCompatible` exists but has no
  `LlmProvider` implementation yet): it naturally wants to be *multiple* named user-defined
  instances (a "DeepSeek" card, a separate "local Ollama" card, ...), which is a bigger
  data-model change than every other provider needed. Not started.
- **Auto-update, code signing, an Arch/AUR package**: see [docs/release.md](./release.md)'s
  "Not yet done" section — all deliberately out of scope until there's a real release to justify
  the setup cost.
- Advanced Experiment UI (multiple prompt variants, parameter matrices) and an automatic
  prompt optimizer are explicitly out of scope for the MVP — see docs/start.md.
