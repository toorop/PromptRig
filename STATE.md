# PromptRig — Project State

Hand-off document: what's done, what's in progress, decisions made, and known issues — enough
for a new session to resume cleanly. **Update this file in every commit that changes project
state.** See [TODO.md](./TODO.md) for the detailed task checklist, [AGENTS.md](./AGENTS.md) for
the working agreement (commit/state discipline, destructive-command precautions, one step at a
time with a pause for review before commit/push), and
`/home/toorop/.claude/plans/purrfect-frolicking-donut.md` for the full architecture plan
approved by the user.

## Current step

Step 9 (UI polish), in progress, 2026-09-08. Self-hosted typography, the light/dark theme toggle
(dark by default), and a pass of real tooltips + param validation (see below) are done,
committed, and pushed. Currently mid-iteration on Select/Input toolbar font sizing, which the
user has taken over by hand (see TODO.md) — the next UI item to pick up is whatever the user
directs next ("encore d'autres choses concernant l'UI et le design qu'on fera après" — nothing
specific queued yet); don't touch the font-size classes without being asked.

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

**Step 7 — Side-by-side comparison** (committed & pushed, `8891a61`):
- Discussed the Playground↔Compare relationship with the user before building: they initially
  proposed a directed flow (configure in Playground, then switch to Compare which reuses it),
  but agreed a shared store (edit either view, both stay in sync, no required visit order) is
  better — avoids a confusing "Compare is empty because you skipped a step" trap.
- `domain::Experiment` (id, name, created_at) — `ExperimentId` moved out of `run.rs` into a new
  `domain/experiment.rs`; the `id_as_string` serde helper (string-across-IPC for `i64` ids) is
  now in its own `domain/id.rs`, shared by both `RunId` and `ExperimentId` instead of being
  duplicated.
- `storage::experiments_repo::insert_experiment` — deliberately minimal (no `get_experiment`/
  `list_experiments`): `run_experiment` already knows the id/name/timestamp it just created
  without reading it back, and nothing needs to browse past experiments yet. Tests include a
  real foreign-key-violation check (a Run referencing a nonexistent experiment_id is rejected),
  which doubles as a regression guard on the `PRAGMA foreign_keys = ON` set back in Step 3.
- `commands::experiments::run_experiment`: creates one Experiment, then runs all columns
  *concurrently* via `futures::future::join_all` (new dependency — plain `join_all` over
  borrowed futures within one async task; no `tokio::spawn`/`'static`/`Arc`-cloning needed since
  we're not spawning separate tasks, just polling several HTTP-bound futures together).
  A column's own setup error (missing API key, unimplemented provider) is folded into that
  column's persisted Run (`error` set) rather than aborting the whole comparison — one
  misconfigured column shouldn't sink the others. Results come back in the same order as the
  submitted columns (by re-fetching each known `RunId` in order after `join_all`, not by
  relying on SQLite `rowid` order, which could differ from submission order once several
  provider calls race each other).
- Extracted `commands::build_new_run` (outcome → `NewRun` fields) so `run_generation` and the
  per-column runner share that mapping instead of duplicating it; kept their *differing*
  early-failure handling (a missing key aborts a solo Playground run outright, but must not
  abort a whole comparison) as separate code rather than forcing them into one shared function.
- `stores/promptDraft.ts` (Pinia): system prompt, user prompt, temperature/top_p/max_tokens —
  shared by Playground and Compare.
- `components/playground/GenerationParamsFields.vue` extracted from Playground's inline param
  editor, now used by both views: Playground passes the selected model's `capabilities` (filters
  to supported fields), Compare passes none (shows all three, since columns can have different
  models with different support — the backend's `build_request` already silently drops whatever
  a given model doesn't accept).
- `CompareView.vue`: dynamic columns (add/remove, floor of 1), each with an independent
  provider/model picker (models fetched live per column via `list_models`), shared prompt/params
  section on top, `Run all` (disabled until every column is configured and the user prompt is
  non-empty), per-column `Rerun` (via `run_generation`, not `run_experiment` — becomes a
  standalone Run outside the original Experiment; acceptable for now, no experiment-browsing UI
  exists yet to make that matter).
- `cargo check`/`clippy --all-targets`/`fmt --check`/`test` (26 passed, 1 ignored) and
  `npm run build` all clean. Relaunched `tauri dev` to regenerate `bindings.ts` with the new
  `run_experiment` command/types — no panics, HMR picked up the new views cleanly.
- **Manual test done by the user**: compared multiple OpenAI models side by side, including
  `gpt-3.5-turbo` (apparently still available on their account, despite the assumption in Step 4
  that it was likely deprecated — worth remembering the model catalog is more varied per-account
  than assumed). Confirmed the missing-pricing fallback works as designed: `gpt-3.5-turbo` isn't
  in `pricing.json`, and the UI correctly showed "—" for cost instead of erroring.
- Three polish items noted for Step 9 (not fixed now): `ResultPanel`'s `<pre>` text renders in
  the browser's default monospace font rather than the app's theme font; no scrolling when the
  window is smaller than the content (bottom gets clipped instead of scrolling); both found
  during this manual Compare test.
- **Two more bugs found and fixed from continued live testing:**
  - `CompareView` never called `providersStore.refresh()` on mount (Playground and Settings
    both do) — landing on Compare first showed "no provider configured" even when one was.
  - Compare's column state was local component `ref` state, so switching to Playground and back
    reset it entirely (Vue destroys a view's local state on navigation away). Moved into a new
    `stores/compare.ts` (Pinia) — same "state that must survive navigation lives in a store"
    pattern as `promptDraft`/`providers`. `CompareColumn` interface now lives in that store file
    and is imported by `CompareView.vue`.
- **Model list caching discussed, explicitly deferred to Step 8** (not implemented in Step 7):
  the user noticed `list_models` fires a fresh live API call every time a provider is picked on
  a Compare column — e.g. picking OpenAI on 2 columns makes 2 identical requests. Agreed plan:
  use the already-existing (since Step 3, unused) `model_cache` SQLite table with a ~24h
  freshness window, plus a manual "Refresh models" action to force a re-fetch. Added as the
  first item of Step 8, ahead of adding more providers, so every provider benefits from it.

**Step 8 (in progress) — model list caching** (implemented, not yet committed — see below):
- The user reconsidered the originally-discussed 24h SQLite cache and proposed something
  simpler: an in-memory, session-lifetime cache (Pinia store), cleared on app restart. Agreed
  this is actually the better fit — guarantees a fresh model list on every launch (no
  staleness/TTL logic to get wrong) while fully solving the real observed problem (duplicate
  `list_models` calls when the same provider is picked on multiple Compare columns, or on both
  Playground and Compare).
- `stores/providers.ts`: added `modelsByProvider`/`modelsLoading`/`modelsError` (all keyed by
  `ProviderId`) and `loadModels(provider, { force? })`. `PlaygroundView`/`CompareView` now call
  this instead of `commands.listModels` directly. `CompareColumn` (`stores/compare.ts`) dropped
  its own per-column `models`/`modelsLoading`/`modelsError` fields — reads the shared cache
  keyed by its own `provider` instead.
- Added a manual "↻ Refresh" button next to every model picker (Playground, each Compare
  column). Bug found and fixed along the way: refreshing didn't visibly show "Loading models…"
  in Playground (it did in Compare) — turned out to be because a `Select` only shows its
  placeholder when nothing is selected; with a model already chosen, refreshing just greyed out
  the existing value instead of showing the loading text. Fixed by clearing the selection before
  triggering a forced reload, in both views.
- `model_cache` (the SQLite table from Step 3) stays in the schema, unused — not removed, since
  a future feature (e.g. offline mode) could still want it.
- Hit a Pinia+Vite HMR quirk while iterating: the live store instance in memory didn't pick up
  newly-added fields/methods via hot-reload, throwing `TypeError: providersStore.loadModels is
  not a function`. Not a real code bug — fixed by fully restarting `tauri dev` rather than
  relying on HMR. Worth remembering if similar "function is not defined" errors appear right
  after adding something to a Pinia store during dev.
- `npm run build` clean throughout; manually verified live in `tauri dev` by the user twice
  (once for the cache itself, once for the refresh-button placeholder fix).

**Mistral provider** (committed & pushed, `f769932`):
- `providers/mistral.rs`: same Chat Completions wire format as OpenAI, but `/v1/models` reports
  `capabilities.completion_chat` and `max_context_length` per model directly — no id-string
  heuristics needed (unlike OpenAI's `is_chat_model`/`infer_capabilities`). No reasoning-model
  quirk assumed (found no evidence Mistral has an OpenAI-o-series-style split); registered in
  `ProviderRegistry`. 2 new unit tests (request-shape param omission/inclusion).
- No `pricing.json` entries yet for Mistral — didn't want to guess exact API model-id strings
  without confirming them against a real response first.
- **Manually tested by the user with a real key**: connects/authenticates fine (model list
  loaded correctly). A real generation call currently gets `HTTP 429 rate_limited` (Mistral
  error code `1300`) even after the user added account credit and waited several minutes.
  Diagnosis: this is a *request-rate* limit, not a billing/quota error — adding funds isn't
  expected to fix it; likely needs an explicit plan/workspace activation on Mistral's console
  (recalled from earlier research: Mistral requires explicitly selecting a plan, even the free
  one, separately from adding a payment method). The structured JSON error response proves our
  auth/request formatting work correctly — this is an account-side blocker, not a code bug.
  Left to revisit later; not blocking moving on to the next provider.
- Two small UI fixes from this testing session, unrelated to Mistral specifically:
  - `ResultPanel`'s Copy button only appeared for a successful `run.result`, not for a Run's own
    error text or the top-level error — exactly the case hit here (a 429 response). Now
    `copyableText` covers all three text-display cases.
  - New reusable `components/CopyButton.vue`, added next to the System/User prompt labels in
    both `PlaygroundView` and `CompareView` (user's request — same convenience as the Result
    copy button, for saving a good prompt).
- `cargo check`/`clippy --all-targets`/`fmt --check`/`test` and `npm run build` all clean.
  Verified live in `tauri dev` (a full restart was needed again — the file watcher didn't
  auto-pick-up the new `providers/mistral.rs` file on its own).

**OpenRouter provider** (committed & pushed, `12b6710`):
- `providers/openrouter.rs`: an aggregator behind one OpenAI-compatible Chat Completions API.
  `/v1/models` reports `context_length` and a `supported_parameters` array per model directly —
  the most precise capability data of any provider so far, no heuristics needed. OpenRouter's
  own gateway translates to whatever wire format the underlying model actually needs, so there's
  no OpenAI-style reasoning-model quirk to replicate here. Registered in `ProviderRegistry`.
  3 unit tests.
- Confirmed (real fetch of `/v1/models`, no auth needed for the catalog itself) that pricing is
  per-token USD, encoded as JSON strings (e.g. `"0.00001"`), not numbers — noted for whenever
  the deferred "use OpenRouter's own pricing" idea gets built; not implemented now.
- **Manually tested by the user with a real key — fully working end to end**: configured the
  key, model list loaded, ran a real prompt successfully.
- `cargo check`/`clippy --all-targets`/`fmt --check`/`test` (31 passed, 1 ignored) clean.
  Needed another full `tauri dev` restart (same file-watcher limitation as Mistral).

**Anthropic provider** (committed & pushed, `74c4549`):
- `providers/anthropic.rs`: Messages API. Several real wire-format differences from every other
  provider so far, each independently verified against the official docs (a summarized fetch
  first claimed `Authorization: Bearer` for auth, which turned out wrong — cross-checked against
  multiple independent sources before trusting `x-api-key`):
  - `x-api-key` header (not `Authorization: Bearer`) + mandatory `anthropic-version` header.
  - System prompt is a top-level `system` field, not a `{role: "system"}` message.
  - `max_tokens` is *required* by the API (unlike every OpenAI-shaped provider, where it's
    optional) — falls back to `DEFAULT_MAX_TOKENS = 4096` when the caller hasn't set one.
  - `temperature`/`top_p` are deprecated and rejected outright (HTTP 400) for current models
    unless left at their defaults — `ModelCapabilities` reports both unsupported so the UI never
    shows those controls for Anthropic, and we never send them (defense in depth, same pattern
    as OpenAI's reasoning-model handling).
- Registered in `ProviderRegistry`. 3 unit tests (system-at-top-level shape, temperature/top_p
  never sent, default max_tokens fallback).
- **Manually tested by the user with a real key — fully working end to end.**
- `cargo check`/`clippy --all-targets`/`fmt --check`/`test` (34 passed, 1 ignored) clean.

**Gemini provider** (implemented, not yet committed — awaiting the user's live test):
- Confirmed it's the **Gemini API** (Google AI Studio, plain API key) they want, not Vertex AI
  (which needs a GCP project and OAuth/service-account auth — not realistic for someone with
  "just a Google account").
- `providers/gemini.rs`: `x-goog-api-key` header (a `?key=` query-param fallback exists but
  leaks the key into URLs/logs, so header is the documented-preferred approach); the model id is
  in the URL path (`.../models/{id}:generateContent`), not a JSON body field — `ModelInfo::model_id`
  strips the `models/` prefix `/v1beta/models` returns, and it's re-added when building the
  request URL; `contents`/`systemInstruction` are objects made of `parts` (Gemini is
  multi-modal-capable, we only ever send one text part); generation params nest under a
  `generationConfig` object with camelCase names (`topP`, `maxOutputTokens`).
- `/v1beta/models` reports `inputTokenLimit` and `supportedGenerationMethods` per model directly
  (filtered on containing `"generateContent"`) — similar precision to Mistral/OpenRouter, no
  capability-guessing heuristic needed. No reasoning-model-style param restriction assumed for
  Gemini's "thinking" models — found no evidence of one (unlike OpenAI/Anthropic, both
  confirmed via docs to restrict sampling params for their reasoning-capable models).
- Registered in `ProviderRegistry`. 2 unit tests (prefix stripping, request shape with
  camelCase `generationConfig` fields).
- `cargo check`/`clippy --all-targets`/`fmt --check`/`test` (36 passed, 1 ignored) clean.
- **Manually tested by the user with a real key — fully working end to end.** They also noted
  (again) that Gemini/Anthropic show no cost estimate, reaffirming the deferred idea of using
  OpenRouter's own pricing as a cross-provider stand-in — with an explicit caveat they raised
  this time: OpenRouter takes a commission, so a derived estimate would be a *ceiling*, not
  exact, and must be disclosed as such if this ever gets built (see TODO.md's Deferred ideas).

- Mistral retested by the user later in the session: works correctly now. Confirms the earlier
  `429 rate_limited` was a transient account-side delay (billing/plan propagation), not a code
  issue — no code change needed, just a documentation update.
- **Generic OpenAI-compatible endpoint explicitly deferred**: the user considered testing
  against DeepSeek, then noted OpenRouter already covers nearly everything they'd want to test
  this way, so there's no urgency. Also surfaced a real design question for later: this
  provider naturally wants to support **multiple named custom endpoints** (e.g. "DeepSeek" and
  a separate local endpoint, each with its own name/base URL/key) rather than the single fixed
  slot every other `ProviderId` variant has — a bigger data-model change than a normal provider
  addition (a user-defined list, not a fixed enum variant), with Settings needing to show the
  user-given name per instance plus an "add another" action. Not implemented; noted in TODO.md.

- All Step 8 provider work is committed and pushed. Six future ideas logged in TODO.md's
  "Deferred ideas" section (none implemented): saved named prompt sets ("Tests", now with
  explicit delete support), AI-assisted system prompt improvement, app versioning strategy, an
  auto-update mechanism, persisting the current prompt draft across app restarts, and (from
  Step 8) using OpenRouter's pricing as a cross-provider cost estimate.

**Step 9 (in progress) — UI polish, typography + theme toggle** (committed & pushed):
- Self-hosted fonts: added `@fontsource/ibm-plex-sans` and `@fontsource/ibm-plex-mono` (400/500/600
  weights as needed), imported in `main.ts` before `main.css`. Deliberately not a system-font
  stack (inconsistent rendering across OSes) or a CDN webfont (network dependency at runtime) —
  the user's own idea: "on n'est pas obligé de charger par le réseau des polices, on peut la
  télécharger une fois pour toutes et l'inclure au projet." `main.css`'s `--font-sans`/`--font-mono`
  theme variables point at them; `--font-mono` is reserved specifically for prompt/result
  *content* (the textareas, the result text), not decorative UI labels.
  - Applied `font-mono text-sm` to both prompt Textareas in `PlaygroundView.vue` and
    `CompareView.vue` (previously plain default sans, inconsistent with the Result panel).
  - Fixed `ResultPanel.vue`: only the actual result `<pre>` used `font-mono` before; the other
    3 text states (Running…, top-level error, Run's own error, and the empty-state placeholder)
    now use it too, so switching between states doesn't visibly change font mid-flow.
  - The user asked about a perceived size mismatch between the (now-mono, visually larger)
    prompt textareas and the (sans) toolbar Select/Input controls, both nominally `text-sm`;
    per their explicit preference ("je propose plus de grossir la plus petite... que de réduire
    l'autre") grew the toolbar controls instead of shrinking the textareas. Went `text-base`,
    found too large once seen live, tried an intermediate `text-[15px]` — **still not finalized;
    the user is taking over this specific pixel-tuning by hand** ("le plus simple, ce serait
    peut-être que je fasse ça moi-même à la main, comme ça je vois directement le résultat").
    Don't adjust these classes further without being asked.
  - Also separately confirmed (by reading the code, not a bug) that `CompareView`'s system-prompt
    and user-prompt Textareas share byte-identical classes — a perceived font/color difference
    the user raised is almost certainly the dimmer `placeholder:text-muted-foreground` color
    showing on whichever field was empty at the time, not a real inconsistency.
- Light/dark theme toggle: new `src/lib/theme.ts` (shared `Theme` type + `THEME_STORAGE_KEY` +
  `loadStoredTheme()`/`applyTheme()`) and `src/stores/theme.ts` (Pinia store wrapping it with a
  reactive `theme` ref + `toggle()`). `main.ts` applies the persisted-or-default theme
  synchronously before `createApp(...).mount(...)`, so there's no flash of the wrong theme on
  launch. Toggle button (Sun/Moon icons from the already-installed-but-previously-unused
  `@lucide/vue`) added to `App.vue`'s top nav.
  - **Dark is the default**, not light — matches the user's own long-standing Nord usage
    ("c'est comme ça que j'utilise le thème Nord, je l'ai partout") and their live verdict once
    they saw it: "c'est beaucoup plus beau avec le thème sombre." The `.dark` Nord CSS variables
    already existed since Step 0/6 — this just makes dark the actual default and adds the
    toggle + persistence that were missing.
- `npm run build` (vue-tsc + vite) clean. Verified live via Vite HMR in the already-running
  `tauri dev` session (page-reloaded on the `main.ts` change, hot-updated `App.vue` after) —
  no full restart needed this time. User confirmed the result looked right before requesting
  the commit.

**Step 9 (in progress) — real tooltips + param validation** (committed & pushed):
- Added shadcn-vue's `tooltip` component (`src/components/ui/tooltip/`, built on `reka-ui`'s
  `TooltipRoot`/`TooltipTrigger`/`TooltipContent`/`TooltipProvider` — a real hover/focus-driven
  tooltip, not the native `title` attribute already known not to render reliably under
  WebKitGTK). `App.vue`'s whole tree is wrapped in one `TooltipProvider` at the root.
  - Running the shadcn-vue CLI re-added the Google Fonts CDN `@import` for "Geist" in
    `main.css` that had been deliberately removed twice before (Step 0's system-font-stack
    switch, then Step 9's self-hosted-IBM-Plex switch) — caught immediately via `git diff` and
    stripped back out before committing anything else.
- `GenerationParamsFields.vue`: replaced the always-visible one-line caption under each param
  input with a small "?" icon next to the label that shows the same text as a tooltip — the
  user's explicit design preference ("je n'aime pas du tout les légendes en bas... des petites
  infobulles"). New reusable `components/LabelHint.vue` (label + icon + tooltip) extracted since
  this exact pattern is now used ~7 times across two views.
- **Real input validation added** (a functional gap the user flagged, not just cosmetic): values
  are clamped on the `change` event (i.e. once the user leaves the field) to Temperature 0–2,
  Top P 0–1, Max tokens ≥ 1 rounded to an integer. The native `min`/`max` attributes on a number
  input only constrain the little spinner buttons, not typed/pasted values, so out-of-range
  numbers were previously accepted and sent straight to the provider.
- Extended the same tooltip treatment across the rest of the UI, per the user's request to reuse
  it broadly: the "↻ Refresh" model-list buttons (Playground + every Compare column) became icon
  buttons (`RefreshCw`, spinning while `modelsLoading`) with a tooltip instead of a text button +
  native `title`; Compare's remove-column "✕" button too. Added `LabelHint` explanations for
  Provider, Model, System prompt, and User prompt in both views — Compare's per-column
  Provider/Model selects previously had no label or explanation at all, just a bare placeholder.
- `components/CopyButton.vue` converted from a text "Copy"/"Copied" button to an icon button
  (`Copy` → `Check` while the "copied" flash is active), with the label text now living in the
  tooltip — matches the new icon-first visual language everywhere else, per the user's request.
- New `components/ResetButton.vue` (icon button, `Eraser`, tooltip "Clear this field", disabled
  when the field is already empty) — added next to Copy on both System prompt and User prompt in
  Playground and Compare, the user's request for a one-click way to blank a prompt field.
- `npm run build` (vue-tsc + vite) clean. Verified live: killed and relaunched `tauri dev` twice
  during this work (once mid-edit, showed a transient "Failed to resolve component: Label"
  HMR warning from an in-between save state — expected, not a real bug, confirmed gone once the
  edit finished; once more for a fully clean restart to confirm before reporting back). User
  tested live and confirmed it looks right before requesting the commit.

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

Start Step 8. First task: model list caching via the `model_cache` table (~24h freshness +
manual refresh) — agreed with the user to do this before adding new providers, so they all
benefit from it. Then: Anthropic, Gemini, Mistral, OpenRouter, generic OpenAI-compatible — same
trait, same commands, just new `providers/*.rs` modules + registry entries + pricing rows.

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
