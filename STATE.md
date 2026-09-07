# PromptRig — Project State

Hand-off document: what's done, what's in progress, decisions made, and known issues — enough
for a new session to resume cleanly. **Update this file in every commit that changes project
state.** See [TODO.md](./TODO.md) for the detailed task checklist, [AGENTS.md](./AGENTS.md) for
the working agreement (commit/state discipline, destructive-command precautions, one step at a
time with a pause for review before commit/push), and
`/home/toorop/.claude/plans/purrfect-frolicking-donut.md` for the full architecture plan
approved by the user.

## Current step

Steps 0–6 are complete, committed, and pushed. Step 6 was manually verified end-to-end by the
user with a real OpenAI key. About to start **Step 7 — Side-by-side comparison**.

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

**Step 1 — Rust domain & errors** (committed & pushed, `45dff8f`):
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

**Step 2 — Secrets** (committed & pushed, `8391eda`):
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

**Step 3 — Storage** (committed & pushed, `4baaea7`):
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

**Step 4 — First provider (OpenAI) + registry** (committed & pushed, `71f4b0f`):
- `providers/mod.rs` — `LlmProvider` trait (`async-trait`, so it can be a trait object) with
  `test_connection`/`list_models`/`generate`, all taking `api_key: &str` explicitly (providers
  never touch the keyring themselves — the caller looks the key up via `secrets::get_api_key`).
  `ProviderRegistry` maps `ProviderId` → `Box<dyn LlmProvider>`; adding a provider is one new
  module + one line in `ProviderRegistry::new()`.
- `providers/openai.rs`:
  - `list_models` calls OpenAI's real `/v1/models` (the actual models available to that key),
    filters out non-chat models (audio/image/embedding/moderation) via a substring heuristic
    (`is_chat_model`), and infers `ModelCapabilities` from the model id (`infer_capabilities`) —
    OpenAI's API doesn't expose capabilities, and a hand-maintained exact-match table would go
    stale fast given how often the catalog changes.
  - `is_reasoning_model` (name-prefix heuristic: `o1`/`o3`/`o4`/`gpt-5.6`/`gpt-6`) drives both
    capability inference and `build_request`: reasoning models get `max_completion_tokens`
    instead of `max_tokens`, and never get `temperature`/`top_p` even if `params` has them set
    (belt and suspenders beyond the frontend only showing supported controls).
  - `generate` calls Chat Completions, measures wall-clock duration with `std::time::Instant`,
    maps `usage.{prompt_tokens,completion_tokens}` to our `Usage`.
  - **Caveat, flagged to the user:** the current OpenAI model catalog and pricing (gpt-6-astra,
    gpt-5.6-sol/terra/luna, gpt-4o-mini) were sourced via web search + OpenAI's docs page,
    since this is beyond the assistant's training cutoff. Worth double-checking against
    OpenAI's live pricing/docs pages before relying on it for real spend decisions.
- `pricing/` — `PricingTable` loaded from an embedded `pricing.json` (`(provider, model_id)` →
  input/output price per million tokens), `estimate_cost(provider, model_id, usage) -> Option<f64>`
  (`None` for anything not in the table — missing pricing is expected and fine per the spec).
  Runtime-overridable pricing file (without recompiling) is deferred to when Tauri path
  resolution is wired in (Step 5/6) — same "storage layer exists before its full integration"
  pattern as Step 3.
- New dependencies: `async-trait`, `reqwest` (`default-features = false`, `json` + `rustls`
  features — avoids needing OpenSSL). Also added `aws-lc-rs` as a direct dependency purely to
  enable its `prebuilt-nasm` feature: reqwest's `rustls` feature now pulls in `aws-lc-rs` as its
  crypto backend, which needs NASM to build assembly-optimized code on Windows, and GitHub's
  `windows-latest` runners don't ship NASM by default — `prebuilt-nasm` ships precompiled
  objects instead, sidestepping that CI landmine before we ever hit it.
- Neither `providers` nor `pricing` are wired into the Tauri app yet — no command needs them
  until Step 5, consistent with how `storage` was handled in Step 3.
- 10 new unit tests (23 total, 1 ignored by design): chat-model filter, reasoning-model
  heuristic, both request-body shapes (regular vs reasoning model), registry resolution
  (implemented + not-yet-implemented provider), pricing load/estimate/malformed-JSON.
  `cargo check`/`clippy --all-targets`/`fmt --check` all clean.
- **Not done:** a real end-to-end call against the live OpenAI API (costs real money, needs a
  real key) — offered to the user to test manually if they want, same pattern as the keyring
  `#[ignore]`d integration test in Step 2.

**Step 5 — Tauri commands + generated bindings** (committed & pushed, `29c5273`):
- `commands/providers.rs`: `list_providers` (sync, returns `ProviderStatus` — provider id,
  display name, `implemented` from the registry, `configured` from `has_api_key`),
  `test_provider_connection` and `list_models` (both async, look up the key via the new shared
  `require_api_key` helper in `commands/mod.rs` and delegate to the `ProviderRegistry`).
- `commands/runs.rs`: `run_generation` — the Playground's "Run" button. Looks up the key, calls
  `provider.generate()`, computes cost via `PricingTable::estimate_cost`, persists via
  `runs_repo::insert_run` **regardless of success or failure** (a failed call is still a Run
  worth keeping, per docs/start.md), returns the freshly-fetched persisted `Run`.
- **Real integration for the first time**: `lib.rs`'s `setup()` resolves the Tauri app data dir,
  opens the real SQLite `Database` there, and `.manage()`s `Database`/`ProviderRegistry`/
  `PricingTable`. `storage`, `providers`, and `pricing` stop being inert library code exercised
  only by their own tests.
- **tauri-specta wired in**, pinned to the exact version discussed with the user
  (`=2.0.0-rc.25` — the only Tauri-v2-compatible release, still RC after 25 candidates; user
  chose this over hand-written TS types). `tauri_specta::Builder` collects all 7 commands;
  `src/lib/bindings.ts` is regenerated on every debug build.
- **Refactored `AppError`** to enable specta support: dropped the hand-rolled `Serialize` impl
  from Step 1 in favor of a plain `#[serde(tag = "kind", content = "message")]` derive (which
  `specta::Type` can also read automatically). Trade-off: call sites now pre-format the full
  message themselves rather than relying on a generic per-variant Display prefix — audited, and
  every existing call site already did this anyway, so it was a no-op in practice.
- **Caught and fixed a real runtime panic** via the `tauri dev` smoke test (not something
  `cargo check`/`clippy` could catch): specta refuses to export `i64`/`u64` to TypeScript
  (JS number precision loss). Fixed by making `RunId`/`ExperimentId` cross the IPC boundary as
  strings (`#[serde(with = "id_as_string")]` + `#[specta(type = String)]`; still plain `i64`
  internally/in SQLite) and switching `RunResult.duration_ms`/`ttft_ms` from `u64` to `u32`
  (a millisecond duration never remotely approaches `u32`'s ~49-day range, and `u32` exports
  safely).
- **Honored a previously-deferred architecture decision**: `storage::Database::with_connection`
  now wraps its closure in `tauri::async_runtime::spawn_blocking`, since it's finally being
  called from real async Tauri commands (this was always the plan — see the plan file's
  decision #1 — just not needed until now). `runs_repo::insert_run`/`get_run` became `async fn`;
  `insert_run` now takes `NewRun` by value (the closure passed to `spawn_blocking` must be
  `'static`, so it needs to own its data rather than borrow it).
- Smoke-tested `npm run tauri dev` twice: first run hit the BigInt panic above before the app
  window even opened; after the fix, the user confirmed the app launched cleanly, and
  `src/lib/bindings.ts` was generated correctly (10.9 KB, all 7 commands + types, including the
  `Run` return type coming through as `Run_Serialize` — specta's conservative handling of the
  custom `id_as_string` serde `with` module splits some types into `_Serialize`/`_Deserialize`
  variants even though ours are symmetric; cosmetic verbosity in the generated file, not a
  correctness issue — worth revisiting only if it becomes annoying to use from the frontend).
- New dev dependency: `tokio` (`macros`, `rt-multi-thread`) for `#[tokio::test]` in the now-async
  storage tests.
- `cargo check`/`clippy --all-targets`/`fmt --check`/`test` (23 passed, 1 ignored) and
  `npm run build` (vue-tsc typechecks the generated bindings) all clean.

**Step 6 — Playground vertical slice** (committed & pushed, `021e4ff`):
- `src/stores/providers.ts` — Pinia store holding `ProviderStatus[]`, shared by Settings (which
  writes) and Playground (which only reads), so Playground reflects a newly-configured key
  without needing to know Settings exists.
- `src/components/settings/ProviderCard.vue` + `SettingsView.vue` — API key save/remove, test
  connection, configured/not-configured badge. Built generically over `list_providers`, so
  Step 8's new providers require zero UI changes — they just appear as additional cards.
- `src/components/playground/ResultPanel.vue` + `PlaygroundView.vue` — provider/model picker
  (models fetched live via `list_models` when the provider selection changes), system/user
  prompt editors, temperature/top_p/max_tokens inputs (shown only when `ModelCapabilities` says
  the model supports them, pre-filled with sensible defaults — 0.7 / 1 / 1024 — each with a
  hover tooltip explaining what it does), Run button, and the result panel (text, latency,
  token usage, estimated cost, copy-to-clipboard button).
- **Manual end-to-end test, done by the user with their own OpenAI key**: saved a key in
  Settings, selected `gpt-4o-mini` in Playground, ran a "clean up this speech-to-text
  transcript" system prompt against a deliberately messy sample user prompt — got back a
  correctly cleaned, reformulated response. Confirms the whole chain works: key storage → live
  model listing → generation → cost estimate → SQLite persistence.
- **Non-bug found during testing**: the provider `<Select>` appeared completely unresponsive to
  clicks. Root cause: no API key was configured yet, so its item list was empty — Reka UI's
  Select won't open with zero items. Not a code defect, but a UX sequencing trap (nothing told
  the user to configure a provider first) — the empty-state hint text already exists
  ("No provider is configured yet. Go to Settings…") but is easy to miss; worth making more
  prominent in Step 9 polish.
- Verified directly against the real SQLite file (`~/.local/share/com.promptrig.app/promptrig.sqlite`)
  that `system_prompt`/`user_prompt` are persisted exactly as submitted — used this to diagnose
  the user's first test (where both fields held identical text, so the model correctly asked
  for the actual content instead of "ignoring" the system prompt).
- **Follow-up fixes from live user testing, same step:**
  - Model picker no longer defaults to `models[0]` of the alphabetically-sorted live list (a
    real safety issue — could silently land on an expensive flagship model). Now remembers the
    last provider+model actually used, per device, via `localStorage`
    (`promptrig.playground.lastSelection`); if nothing's remembered yet, nothing is
    pre-selected rather than guessing.
  - Native `title`-attribute tooltips on the param inputs don't render reliably under
    WebKitGTK — replaced with always-visible one-line hint text under each input instead.
  - First visual polish pass (ahead of the dedicated Step 9): nicer tab-style top nav
    (`App.vue`), toolbar/prompt areas grouped into `Card`s, native number-input spinners
    hidden, and — per the user's request — a Nord-inspired color palette
    (nordtheme.com: Polar Night/Snow Storm for background+text, Frost blue for the primary
    accent) applied to both the light and dark `main.css` variable blocks, so the dark theme
    (added in Step 0, unused until a toggle exists) is already Nord-consistent whenever Step 9
    wires up the toggle. User's verdict: nicer than plain neutral shadcn, but ended up with
    *less* contrast than they wanted — explicitly deferred fixing that further to Step 9 rather
    than iterating more now.
  - User pushed back on the framing that the `artifact-design`/`design` skills don't apply
    here ("it's still Vue/HTML/CSS rendered somewhere") — fair point technically; the real
    reason they weren't used is those skills are wired to Artifact-tool-specific mechanics
    (CSP, host-driven theming, etc.) that don't exist in a Tauri app, not that the underlying
    design fundamentals don't transfer. Handled this styling pass with direct CSS/Tailwind
    knowledge instead of loading either skill.
- `npm run build` (vue-tsc + vite) clean throughout this whole step. No Rust changes.

## In progress / not yet done

- Nothing in progress right now — Steps 0–6 are all committed and pushed. Step 7 hasn't started.

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

Start Step 7 (side-by-side comparison: extend `experiments_repo`, `run_experiment` command,
`CompareView`).

## Deferred ideas (see TODO.md's "Deferred ideas" section for detail)

Discussed 2026-09-07 — not scheduled, don't start without the user asking:
- Two separate UI actions: "Refresh models" (per provider, live API call) vs "Update pricing"
  (fetches a `pricing.json` maintained in the PromptRig GitHub repo). OpenRouter is a confirmed
  exception — its `/api/v1/models` already returns per-model pricing, so it doesn't need the
  external pricing file at all.
- User plans to eventually build a separate bot/agent to keep that repo-hosted pricing.json
  up to date — out of scope for the app itself.
- Two more pricing-sourcing ideas noted (full detail in TODO.md): using OpenRouter's own
  per-model pricing as an approximate cross-provider stand-in (with an "approximate" disclaimer
  in the UI), and LiteLLM's `model_prices_and_context_window.json` as a reference/cross-check
  only — user deliberately does not want a hard runtime dependency on an external, unmaintained-
  risk project for something as central as pricing data.
