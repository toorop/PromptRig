# PromptRig — TODO

Detailed, checkable task list. Keep this in sync with reality: check items off as they land,
and update [STATE.md](./STATE.md) in the same commit. Full architecture rationale lives in
`docs/architecture.md` (once written) and in the original spec at `docs/start.md`.

Explicitly **out of scope for now** (do not start early): automatic prompt optimizer, advanced
Experiment UI (multiple prompt variants, parameter matrices), streaming UI.

## Step 0 — Bootstrap

- [x] `git init`
- [x] Scaffold via `create-tauri-app` (template `vue-ts`)
- [x] Rename generated project from `tauri-app` to `promptrig` (package.json, Cargo.toml, lib name, tauri.conf.json)
- [x] Install Tailwind CSS v4 + `@tailwindcss/vite`
- [x] Init shadcn-vue (Reka UI base, Lucide icons, neutral base color, CSS variables theme)
- [x] Drop the Google Fonts CDN import shadcn-vue's init added; use a system font stack instead (offline-friendly desktop app)
- [x] Add base shadcn-vue components: button, input, textarea, select, card, label, separator, badge
- [x] Install Pinia + Vue Router, wire them into `main.ts`
- [x] Configure `@` path alias (vite.config.ts + tsconfig.json)
- [x] Minimal app shell: top nav (Playground / Compare / Settings) + `RouterView`, 3 placeholder views
- [x] Remove the scaffold's placeholder `greet` command
- [x] Verify frontend typechecks and builds (`npm run build`)
- [x] Verify Rust backend compiles (`cargo check`)
- [x] Smoke-test `npm run tauri dev` actually opens a window (needed `WEBKIT_DISABLE_DMABUF_RENDERER=1` on this NVIDIA/Wayland machine — see STATE.md)
- [x] `README.md` — real project description + usage instructions (English, public repo)
- [x] `TODO.md` (this file)
- [x] `STATE.md`
- [x] `LICENSE` (MIT)
- [x] `SECURITY.md`
- [x] `CODE_OF_CONDUCT.md`
- [x] `.github/ISSUE_TEMPLATE/` + `.github/PULL_REQUEST_TEMPLATE.md`
- [x] First commit(s)

## Step 1 — Rust domain & errors

- [x] `domain/error.rs` — `AppError` (thiserror internally, hand-rolled `Serialize` impl using the Display message, so Tauri commands can return it directly)
- [x] `domain/provider.rs` — `ProviderId` enum (OpenAi, Anthropic, Gemini, Mistral, OpenRouter, OpenAiCompatible)
- [x] `domain/model.rs` — `ModelInfo`, `ModelCapabilities`, `GenerationParams`
- [x] `domain/run.rs` — `Run`, `RunId`, `ExperimentId`, `Usage`, `RunResult`
- [x] Unit tests: `AppError` JSON shape, `ProviderId` as_str()/serde consistency, `GenerationParams` defaults (3 tests, all passing)
- [ ] Unit tests for cost calculation (deferred — `pricing/` doesn't exist yet, lands in Step 4)

## Step 2 — Secrets

- [x] `secrets/` module wrapping the `keyring` crate (default `v1` feature — already selects the right per-OS backend, no extra feature flags needed)
- [x] Commands: `save_api_key`, `delete_api_key`, `has_api_key` (never return the raw key to the frontend; `get_api_key` exists for internal use by providers later but is not a registered command)
- [x] Unit tests for the error-mapping logic (NoEntry handling), extracted into pure functions so they don't need a real keyring backend (4 tests)
- [x] Manual, `#[ignore]`d integration test exercising the real OS keyring — run once with `cargo test -- --ignored`, confirmed working against Secret Service on this Linux dev machine

## Step 3 — Storage

- [x] `storage/db.rs` — `Database` (`Arc<Mutex<rusqlite::Connection>>`) + `rusqlite_migration` runner, `open()` and `open_in_memory()` (for tests)
- [x] Initial migration: `prompts`, `test_cases`, `runs`, `experiments`, `model_cache` tables (only `runs` has repository code so far — the rest get Rust types/repos when the features that use them are built)
- [x] `storage/runs_repo.rs` — `NewRun`, `insert_run`, `get_run`; row parsing kept separate from the rusqlite row-mapping closure so JSON/provider/timestamp parse errors become `AppError` cleanly
- [x] Added `domain::ProviderId::parse` (inverse of `as_str()`) and `PartialEq` on the run/model domain types, needed for storage round-trip tests
- [x] Not yet wired into the Tauri app (`storage` isn't in `lib.rs`'s builder) — no command needs it until Step 5/6, so it's only exercised by its own tests for now
- [x] Unit tests: migrations apply on an in-memory DB, Run insert/get round-trip (success case and failed-run case), missing id returns `None` (4 tests, all passing)

## Step 4 — First provider (OpenAI) + registry

- [x] `providers/mod.rs` — `LlmProvider` trait (`async-trait`) + `ProviderRegistry` (keyed by `ProviderId`, one line to add a provider)
- [x] `providers/openai.rs` — `test_connection`, `list_models` (dynamic fetch from `/v1/models`, filtered to chat models, capabilities inferred by name heuristic since OpenAI's API doesn't expose them), `generate` (Chat Completions API, handles the reasoning-model `max_completion_tokens` vs `max_tokens` field-name switch)
- [x] `pricing/` module + `pricing.json` seeded with OpenAI prices (gpt-4o-mini, gpt-5.6-luna/terra/sol, gpt-6-astra) — **sourced via web search since this is beyond training cutoff; worth double-checking against OpenAI's live pricing page**
- [x] Not wired into the Tauri app yet (no command needs it until Step 5), consistent with `storage`
- [x] Added `reqwest` (rustls) + `async-trait`; also added `aws-lc-rs` directly just to enable its `prebuilt-nasm` feature, so the Windows CI build doesn't fail for lack of NASM (reqwest's rustls backend now depends on aws-lc-rs, which needs NASM to build its assembly code on Windows unless prebuilt objects are used)
- [x] Unit tests: chat-model filter, reasoning-model heuristic, request body shape (both branches), registry resolution (implemented + not-yet-implemented provider), pricing load/estimate/malformed-JSON (10 new tests, 23 total)
- [ ] Real end-to-end test against the live OpenAI API — not done automatically (costs real money, needs a real key); offered to the user to test manually if they want

## Step 5 — Tauri commands + generated bindings

- [x] `commands/providers.rs` (`list_providers`, `test_provider_connection`, `list_models`), `commands/runs.rs` (`run_generation`); shared `require_api_key` helper in `commands/mod.rs`
- [x] Wired `specta` + `tauri-specta` (pinned `=2.0.0-rc.25` — the only version compatible with Tauri v2, never left RC despite 25 candidates; user explicitly chose to accept that risk over hand-written TS types), generate `src/lib/bindings.ts` in debug builds via `tauri_specta::Builder`
- [x] Refactored `AppError` to a plain `#[serde(tag = "kind", content = "message")]` derive (dropped the hand-rolled `Serialize` impl) so `specta::Type` can derive automatically too — callers now pre-format the full message rather than relying on a generic Display prefix
- [x] Fixed a real runtime panic caught by the `tauri dev` smoke test: specta refuses to export `i64`/`u64` to TypeScript (precision loss risk). `RunId`/`ExperimentId` now cross the IPC boundary as strings (`#[serde(with = "id_as_string")]` + `#[specta(type = String)]`, internally still `i64`); `RunResult.duration_ms`/`ttft_ms` switched from `u64` to `u32` (plenty for a millisecond duration, and `u32` is safe to export directly)
- [x] `storage::Database::with_connection` now wraps its closure in `tauri::async_runtime::spawn_blocking` (per the original architecture decision) since it's finally being called from real async commands; `runs_repo::insert_run`/`get_run` are now `async fn`, `insert_run` takes `NewRun` by value instead of by reference
- [x] **First real wiring**: `lib.rs`'s `setup()` now resolves the app data dir, opens the real `Database`, and `.manage()`s `Database`/`ProviderRegistry`/`PricingTable` — `storage`/`providers`/`pricing` are no longer inert
- [x] Smoke-tested `npm run tauri dev`: caught and fixed the BigInt panic above; second run launched cleanly (user confirmed), `src/lib/bindings.ts` generated correctly (10.9 KB, all 7 commands + types present)
- [x] `cargo check`/`clippy --all-targets`/`fmt --check`/`test` (23 passed, 1 ignored) and `npm run build` (vue-tsc against the generated bindings) all clean

## Step 6 — Playground vertical slice

- [ ] `SettingsView`: OpenAI provider card (API key, test connection, status)
- [ ] `PlaygroundView`: provider/model picker, system/user prompt editors, basic params, Run button, result panel (text, latency, usage, cost)
- [ ] Persist each Run to SQLite
- [ ] **Manual test**: configure an OpenAI key, run a prompt, see the response + metrics

## Step 7 — Side-by-side comparison

- [ ] Extend `experiments_repo`
- [ ] `run_experiment` command — parallel execution of multiple Runs (`tokio` tasks)
- [ ] `CompareView`: dynamic columns (add/remove), shared prompt editable per column, `Run all`, rerun a single column
- [ ] **Manual test**: compare 2-3 OpenAI models side by side

## Step 8 — Generalize providers

- [ ] Anthropic
- [ ] Google Gemini
- [ ] Mistral
- [ ] OpenRouter
- [ ] Generic OpenAI-compatible endpoint
- [ ] Pricing entries for each

## Step 9 — UI polish

- [ ] Light/dark theme toggle
- [ ] Resizable panels
- [ ] Advanced params drawer
- [ ] Easy result copy

## Step 10 — CI/CD

- [ ] `.github/workflows/ci.yml`: eslint, vue-tsc, frontend tests, `cargo fmt --check`, `cargo clippy`, `cargo test`, `tauri build` dry run
- [ ] `.github/workflows/release.yml`: triggered on tag push, `tauri-action` matrix (Linux x86_64, Windows x86_64, macOS arm64 + x64), publish to GitHub Release

## Step 11 — Documentation

- [ ] Finalize `README.md`
- [ ] `docs/development.md` (Linux/Windows/macOS setup)
- [ ] `docs/architecture.md`
- [ ] `docs/adding-a-provider.md`
- [ ] `docs/release.md`
- [ ] Confirm `TODO.md`/`STATE.md` reflect actual project state

## Deferred ideas (not scheduled — don't start early)

Discussed 2026-09-07, explicitly not to be implemented until the user asks:

- **Two separate update mechanisms in the UI**, per the user's idea: a "Refresh models" action
  per provider (live call to that provider's model-list API, e.g. `list_models()`, refreshing
  `model_cache`) and a separate "Update pricing" action, since models and prices change on
  different schedules and via different sources.
- **Pricing source per provider differs**: OpenRouter's `/api/v1/models` actually returns a
  `pricing` object per model (USD per token, prompt/completion) — confirmed via its docs, so
  OpenRouter pricing can be read directly from its own API, no external file needed. OpenAI
  (confirmed) and, most likely, Anthropic/Gemini/Mistral do **not** expose pricing via API, so
  they still need the "maintained file" approach.
- **A `pricing.json` maintained in the PromptRig GitHub repo**, fetched by the app over HTTP on
  demand (the "Update pricing" action) — decouples price updates from app releases, per
  docs/start.md's original requirement. Local user-supplied override file stays a fallback.
- **A separate bot/agent (the user's idea)** that periodically scrapes/checks provider pricing
  pages and opens a PR (or otherwise updates) that maintained `pricing.json` in the repo — a
  follow-up project of its own, not part of the app itself.
- **Use OpenRouter's own pricing as an approximate stand-in for other providers**, per the
  user's idea: OpenRouter's per-model prices for e.g. OpenAI models track the vendor's own
  pricing fairly closely, so they could seed/cross-check our `pricing.json` for providers that
  don't expose pricing themselves — shown in the UI with a clear "approximate, order of
  magnitude only" disclaimer rather than presented as exact.
- **LiteLLM's `model_prices_and_context_window.json`**
  (raw: `https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json`)
  is a large (thousands of entries, effectively every provider/model) community-maintained
  pricing+metadata file — checked its shape: keyed by model name, fields like
  `input_cost_per_token`, `output_cost_per_token`, `litellm_provider`, `mode`,
  `max_input_tokens`/`max_output_tokens`, capability flags (`supports_vision`, etc.), cache
  pricing where applicable. Tempting as a data source, but the user flagged a real risk: even
  though its license would allow using it, depending on an external project we don't control
  means we're stuck if it goes unmaintained. Decision: don't build a hard runtime dependency on
  it; at most, consult it as a reference/cross-check when hand-updating our own `pricing.json`
  (manually or via the future update-bot).
