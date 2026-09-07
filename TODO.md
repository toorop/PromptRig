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

- [ ] `providers/mod.rs` — `LlmProvider` trait (`async-trait`) + `ProviderRegistry`
- [ ] `providers/openai.rs` — `test_connection`, `list_models` (static + dynamic), `generate`
- [ ] `pricing/` module + `pricing.json` seeded with OpenAI prices

## Step 5 — Tauri commands + generated bindings

- [ ] `commands/providers.rs`, `commands/runs.rs`
- [ ] Wire `specta` + `tauri-specta`, generate `src/lib/bindings.ts`

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
