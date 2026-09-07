# PromptRig — Project State

Hand-off document: what's done, what's in progress, decisions made, and known issues — enough
for a new session to resume cleanly. **Update this file in every commit that changes project
state.** See [TODO.md](./TODO.md) for the detailed task checklist, [AGENTS.md](./AGENTS.md) for
the working agreement (commit/state discipline, destructive-command precautions, one step at a
time with a pause for review before commit/push), and
`/home/toorop/.claude/plans/purrfect-frolicking-donut.md` for the full architecture plan
approved by the user.

## Current step

**Step 3 — Storage**: implemented, verified (tests/clippy/fmt all clean), awaiting user review
before commit. Steps 0, 1, and 2 are complete and pushed.

## Done so far

**Step 0 — Bootstrap** (committed & pushed):
- Repo initialized, Tauri 2 + Vue 3/TS scaffold via `create-tauri-app`, renamed to `promptrig`
  throughout (package.json, `src-tauri/Cargo.toml`, lib name `promptrig_lib`,
  `tauri.conf.json` productName/title, identifier `com.promptrig.app`).
- Tailwind CSS v4 + shadcn-vue (Reka UI base, Lucide icons, neutral base color, light/dark CSS
  variable theme). Dropped the default Google Fonts CDN import for a system font stack.
- Base shadcn-vue components: button, input, textarea, select, card, label, separator, badge.
- Pinia + Vue Router wired in; `@` → `src/*` path alias configured (vite.config.ts + tsconfig.json).
- Minimal app shell (`src/App.vue`: top nav Playground/Compare/Settings + `RouterView`, three
  placeholder views). Removed the scaffold's placeholder `greet` command.
- Repo meta for the public GitHub repo: README, LICENSE (MIT), SECURITY.md, CODE_OF_CONDUCT.md,
  issue/PR templates, `AGENTS.md` (working agreement for AI agents on this repo).
- `npm run build` and `cargo check` verified; `npm run tauri dev` smoke-tested successfully
  (see Known issues for the NVIDIA/Wayland workaround needed on this dev machine).
- GitHub remote configured by the user via VS Code: `origin` → `github.com/toorop/PromptRig`
  (HTTPS). `gh auth setup-git` was run once in this session to let `git push` authenticate
  through the `gh` CLI's stored credentials.
- Untracked `.claude/scheduled_tasks.lock` (assistant session state, accidentally committed via
  the editor) and added it to `.gitignore` alongside `settings.local.json`.

**Step 1 — Rust domain & errors** (implemented, not yet committed — see below):
- `src-tauri/src/domain/error.rs` — `AppError` (thiserror for `Display`/internal `?`
  conversions later) with a hand-rolled `Serialize` impl (`{ kind, message }`) so Tauri commands
  can return it directly and the frontend gets the polished Display message. `AppResult<T>` alias.
- `domain/provider.rs` — `ProviderId` enum (OpenAi, Anthropic, Gemini, Mistral, OpenRouter,
  OpenAiCompatible); each variant has an explicit `#[serde(rename = ...)]` matching its
  `as_str()` value (used for SQLite/keyring keys) — a test asserts these two hand-maintained
  string sources never drift apart.
- `domain/model.rs` — `ModelCapabilities` (which generation params a model supports),
  `ModelInfo` (provider + model id + display name + capabilities + context window —
  deliberately no pricing fields, cost stays independently updatable per the plan),
  `GenerationParams` (temperature/top_p/max_tokens, all `Option`).
- `domain/run.rs` — `RunId`/`ExperimentId` newtypes over `i64` (id assignment is the storage
  layer's job, not designed yet), `Usage`, `RunResult` (provider call output), `Run` (the full
  persisted record: request + result; `experiment_id: Option<ExperimentId>` so a solo
  Playground run is just a 1-run Experiment).
- New Cargo dependencies: `chrono` (with `serde` feature, for `Run.started_at`) and `thiserror`
  — both small, standard, non-structural additions.
- 3 unit tests added, all passing: `AppError` JSON shape, `ProviderId` as_str()/serde
  consistency, `GenerationParams` defaults. `cargo check`, `cargo clippy --all-targets`, and
  `cargo fmt --check` all clean.

**Step 2 — Secrets** (implemented, not yet committed — see below):
- `src-tauri/src/secrets/mod.rs` — wraps the `keyring` crate. `SERVICE = "promptrig"`, account
  name = `ProviderId::as_str()`. `save_api_key`, `delete_api_key` (treats "already absent" as
  success), `get_api_key` (internal-only, returns `Option<String>`), `has_api_key` (built on
  top of `get_api_key`). The `NoEntry`-handling logic is extracted into small pure functions
  (`map_delete_result`, `map_get_result`) specifically so it's unit-testable without a real
  keyring backend (CI runners typically don't have one available/unlocked).
- `src-tauri/src/commands/` introduced (new top-level module) with `commands/secrets.rs`
  exposing `save_api_key`, `delete_api_key`, `has_api_key` as Tauri commands — thin wrappers
  with no logic of their own. Registered in `lib.rs`'s `invoke_handler`.
- New dependency: `keyring` v4.2.0, default features only. Its default `v1` feature already
  target-conditionally pulls in the right per-OS backend (Secret Service on Linux, Keychain on
  macOS, Credential Manager on Windows) — confirmed by reading the crate's own Cargo.toml, no
  extra feature flags or platform-specific Cargo.toml stanzas needed on our side.
- 4 unit tests (pure error-mapping logic) + 1 `#[ignore]`d integration test that exercises the
  real OS keyring end to end (save/has/get/delete, non-destructively restoring any pre-existing
  key). Ran it manually once with `cargo test -- --ignored`: confirmed working against Secret
  Service on this Linux dev machine. `cargo check`/`clippy --all-targets`/`fmt --check` clean.

**Step 3 — Storage** (implemented, not yet committed — see below):
- `src-tauri/src/storage/migrations/0001_initial.sql` — `prompts`, `test_cases`, `experiments`,
  `runs`, `model_cache` tables. Only `runs` has Rust repository code so far; the rest exist now
  so the schema doesn't need a disruptive later migration, and get real repos when the features
  that use them (prompt saving, side-by-side comparison, model list caching) are built.
- `storage/db.rs` — `Database` (`Arc<Mutex<rusqlite::Connection>>`), `open()` / `open_in_memory()`
  (tests), enables the `foreign_keys` pragma (off by default in SQLite; needed for `ON DELETE
  SET NULL` on `runs.experiment_id`), runs migrations via `rusqlite_migration`.
  `with_connection()` centralizes locking so a poisoned mutex becomes an `AppError`, not a panic.
- `storage/runs_repo.rs` — `NewRun` (everything `Run` has except `id`, since SQLite assigns
  that on insert), `insert_run`, `get_run`. The rusqlite row-mapping closure only extracts raw
  column values (infallible); JSON/provider-string/timestamp parsing — which can fail — happens
  afterward, outside the closure, so parse errors become plain `AppError`s instead of having to
  be shoehorned into `rusqlite::Error`.
- Added `domain::ProviderId::parse` (the inverse of `as_str()`, needed to read the `provider`
  column back) and `PartialEq` on `ModelCapabilities`, `GenerationParams`, `Usage`, `RunResult`,
  `Run` (needed for the round-trip test assertions).
- New dependencies: `rusqlite` (`bundled` feature — statically compiles SQLite so no system
  libsqlite3 is required on any platform/CI runner) and `rusqlite_migration`.
- **Not wired into the Tauri app yet** (`storage` isn't referenced from `lib.rs`'s builder) —
  deliberately deferred until a command actually needs it (Step 5/6), so it's only exercised by
  its own tests for now.
- 4 unit tests, all passing: migrations apply on an in-memory DB, Run insert/get round-trip
  (success and failed-run cases), missing id returns `None`. `cargo check`/`clippy --all-targets`
  /`fmt --check` all clean.

## In progress / not yet done

- Step 3 changes above are complete but **not yet committed** — awaiting user review per the
  step-by-step workflow (finish a step, stop, wait for go-ahead, then commit + push).

## Known issues / incidents

- **Linux + NVIDIA + Wayland**: `npm run tauri dev` crashes immediately after opening the
  window (`Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display`) on the dev
  machine (NVIDIA proprietary driver, Hyprland/Wayland session) — a known WebKitGTK/DMA-BUF
  renderer issue, not an app bug. Workaround: run with `WEBKIT_DISABLE_DMABUF_RENDERER=1` set.
  To document in `docs/development.md` (Linux troubleshooting) once that file exists.
- **Data-loss incident (recovered):** running `create-tauri-app ... --force` in the non-empty
  project directory silently deleted `docs/start.md` (the user's original spec), even though
  that path was unrelated to the Tauri template. Restored verbatim from conversation history
  (verified: 615 lines, matching the original). Lesson recorded in assistant memory and in
  `AGENTS.md` — never force-scaffold into a non-empty directory without protecting existing
  files first, and commit/update STATE.md regularly.

## Key decisions (see the plan file for full rationale)

- SQLite access confined to Rust via `rusqlite` (not `tauri-plugin-sql`).
- Provider abstraction: `trait LlmProvider` (`async-trait`) + `ProviderRegistry` keyed by a
  `ProviderId` enum.
- Rust↔TS type sync via `specta` + `tauri-specta` generated bindings.
- `Experiment` containing N `Run`s from day one; a solo Playground run is a 1-run Experiment.
- Cost = `f64` USD estimate from an externalized `pricing.json`, kept independent of `ModelInfo`.
- Streaming deferred (ship `generate()` first; `generate_stream()` + Tauri events later).
- License: MIT. Repo meta: SECURITY.md, CODE_OF_CONDUCT.md, and issue/PR templates included
  from the start (public repo).
- All docs and code comments are written in English (conversation with the user is in French,
  dictated via speech-to-text — expect occasional transcription oddities in their messages).
- Work proceeds **one step at a time**: implement, verify, stop and report, wait for the user's
  go-ahead, only then commit + push.

## Next action

Waiting on user review of Step 3 (storage module). Once confirmed, commit + push, then start
Step 4 (first provider — OpenAI — plus the `LlmProvider` trait, `ProviderRegistry`, and
`pricing/` with a seeded `pricing.json`).
