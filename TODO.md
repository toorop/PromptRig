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

- [x] `stores/providers.ts` (Pinia): shared `ProviderStatus[]` list, used by both Settings and Playground
- [x] `SettingsView` + `components/settings/ProviderCard.vue`: API key input/save/remove, test connection, configured/not-configured badge — built generically over `list_providers` so Step 8's new providers need zero UI changes, just show up as extra cards
- [x] `PlaygroundView` + `components/playground/ResultPanel.vue`: provider/model picker (models fetched live via `list_models` when provider changes), system/user prompt editors, temperature/top_p/max_tokens inputs (only shown per `ModelCapabilities`, pre-filled with sensible defaults — 0.7 / 1 / 1024 — each with a short visible hint, not a hover tooltip — see below), Run button, result panel (text, latency, tokens in/out, estimated cost, copy button)
- [x] Persist each Run to SQLite (already wired via `run_generation` in Step 5)
- [x] **Manual test, done by the user with a real OpenAI key**: configured `gpt-4o-mini`, ran a "clean up this speech-to-text transcript" system prompt against a messy sample user prompt — got back a correctly cleaned response. Confirms the full chain (key storage → model listing → generation → cost estimate → persistence) works end to end.
- Bug found and fixed along the way: the provider `<Select>` doesn't open when its item list is empty (i.e. no provider configured yet) — not a real bug, just a UX sequencing gap; worth a clearer empty-state hint later (Step 9) so it's not mistaken for broken UI
- Fixed a real UX/safety issue from user testing: the model picker defaulted to `models[0]` of an alphabetically-sorted live list, which could silently land on an expensive flagship model. Now remembers the last provider+model actually used (`localStorage`, per-device) and defaults to that; if nothing's remembered, nothing is pre-selected — an explicit choice beats a guessed one.
- Replaced native `title` tooltips on the param labels (unreliable under WebKitGTK) with always-visible one-line hint text under each input.
- Applied a Nord-inspired color pass (`src/assets/main.css`) to both the light and dark theme variable blocks — first real UI polish pass, ahead of Step 9. User's take: better than plain shadcn neutral, but ended up with *less* contrast than hoped for; explicitly deferred fixing that further to Step 9 rather than iterating more now.
- `npm run build` (vue-tsc + vite) clean throughout

## Step 7 — Side-by-side comparison

- [x] `domain::Experiment` + `ExperimentId` (moved out of `run.rs` into its own `experiment.rs`; shared `domain::id::id_as_string` serde helper extracted so both id newtypes use it)
- [x] `storage::experiments_repo::insert_experiment` — only what's needed now (no `get_experiment`/`list_experiments` yet, same "storage ahead of use, but only what's needed" pattern as prompts/test_cases in Step 3)
- [x] `commands::experiments::run_experiment` — one Experiment, N Runs (one per column), run concurrently via `futures::future::join_all` (new small dependency); a column's own config error (e.g. missing API key) becomes that column's failed Run instead of aborting the whole comparison
- [x] Shared `commands::build_new_run` helper extracted (used by both `run_generation` and the per-column runner) so the "outcome → NewRun fields" mapping exists once
- [x] `stores/promptDraft.ts` (Pinia): system prompt, user prompt, and base params, shared between Playground and Compare with no explicit hand-off — discussed with the user, who initially proposed a directed Playground→Compare flow but agreed shared state is better (works regardless of which view you open first)
- [x] `components/playground/GenerationParamsFields.vue` extracted (temperature/top_p/max_tokens editor), reused by both Playground (filtered by the selected model's capabilities) and Compare (shows all three unconditionally, since columns can have different models — the backend already drops whatever a column's model doesn't support)
- [x] `CompareView`: dynamic columns (add/remove, minimum 1), each with its own provider/model picker; `Run all`; per-column `Rerun` (via the single-run command — creates a standalone Run rather than reattaching to the original Experiment, acceptable since there's no experiment-browsing UI yet to care)
- [x] **Manual test, done by the user**: compared multiple OpenAI models side by side, including `gpt-3.5-turbo` — confirmed the missing-pricing fallback works as designed (shows "—" for cost instead of erroring, since that model isn't in `pricing.json`)
- Two bugs found and fixed from that live testing:
  - `CompareView` never called `providersStore.refresh()` on mount (unlike Playground/Settings) — landing on Compare first (fresh navigation or reload) showed "no provider configured" even when one was, until another view happened to trigger the fetch.
  - Compare's column state (list of columns, each one's provider/model selection and results) was local component `ref` state, so it reset every time the user navigated away from Compare and back (Vue destroys a view's local state on route change). Moved into a new `stores/compare.ts` (Pinia), matching the same pattern as `promptDraft`/`providers` — state that needs to survive navigation lives in a store, not in the view.
- Noted for Step 8 (not fixed now, explicitly deferred by the user): `list_models` has no caching — picking the same provider on multiple Compare columns fires duplicate live API calls. Plan: use the already-existing `model_cache` table with a ~24h freshness window plus a manual refresh action.
- `cargo check`/`clippy --all-targets`/`fmt --check`/`test` (26 passed, 1 ignored, +3 new experiments_repo tests including a real FK-violation check) and `npm run build` all clean; `tauri dev` relaunched to regenerate bindings, no panics

## Step 8 — Generalize providers

- [x] **Model list caching** — the user reconsidered the original 24h-SQLite-cache plan and proposed something simpler: cache in memory for the app's session only (a Pinia store), cleared on restart. Better fit than the SQLite `model_cache` table idea: guarantees a fresh model list on every launch (no staleness window to reason about), and fully solves the actual observed problem (picking the same provider on 2+ Compare columns, or on both Playground and Compare, no longer re-fetches) with zero backend changes.
  - `stores/providers.ts` extended with `modelsByProvider`/`modelsLoading`/`modelsError` (keyed by `ProviderId`) and `loadModels(provider, { force? })` — returns the cached list unless `force: true` or nothing cached yet.
  - `PlaygroundView` and `CompareView` both call `providersStore.loadModels(...)` instead of `commands.listModels(...)` directly; `CompareColumn` (in `stores/compare.ts`) no longer keeps its own `models`/`modelsLoading`/`modelsError` — reads the shared cache by its `provider` instead.
  - Added a manual "↻ Refresh" button next to each model picker (Playground and every Compare column) that calls `loadModels(provider, { force: true })` — also clears the current model selection first, so the `Select`'s "Loading models…" placeholder actually shows (it only appears when nothing is selected; otherwise the previously-selected value stays displayed, just greyed out) — found via live testing, same session.
  - The `model_cache` SQLite table (Step 3) stays unused for now; not removed, since it could still be useful later (e.g. an offline mode).
  - `npm run build` clean throughout; verified live in `tauri dev` (including working around a Pinia+Vite HMR quirk — a stale in-memory store instance missing new fields — by doing a full dev-server restart rather than relying on hot-reload).
- [x] **Mistral** (`providers/mistral.rs`) — same Chat Completions shape as OpenAI, but `/v1/models` is much more useful: each entry reports `capabilities.completion_chat` and `max_context_length` directly, so no id-string heuristics needed for filtering/capabilities/context window (unlike OpenAI). No reasoning-model quirk assumed (nothing found suggesting Mistral has an OpenAI-o-series-style split) — plain `build_request`, unit-tested for param-omission/inclusion shape.
  - Registered in `ProviderRegistry`. No pricing entries yet — didn't want to guess exact API model-id strings for `pricing.json` without confirming them against a real `list_models` response first.
  - **Manually tested by the user with a real key, fully working end to end.** Initially hit `HTTP 429 rate_limited` even after adding account credit; retesting later (after enough time had passed) worked correctly — confirms it was a transient account-side propagation delay, not a code bug.
  - Two small UI fixes landed alongside this (found via the same testing session): `ResultPanel` copying a *failed* Run's error text didn't work (the Copy button only showed for a successful `run.result`) — now shows for any displayed text (top-level error, Run error, or success). Added `CopyButton.vue` (reusable) next to the System/User prompt labels in both Playground and Compare, per the user's request.
- [x] **OpenRouter** (`providers/openrouter.rs`) — the friendliest provider yet: `/v1/models` reports `context_length` and a `supported_parameters` array per model directly, so capabilities come straight from the API (no heuristics like OpenAI, more precise than even Mistral). OpenRouter itself translates to whatever wire format the underlying model needs, so no reasoning-model quirk to handle on our side either. Registered in `ProviderRegistry`. 3 unit tests (capabilities-from-supported-params, both directions, plus request shape).
  - Pricing: OpenRouter's `/v1/models` also returns real per-model pricing (unlike every other provider) — noted as the deferred "use OpenRouter's own pricing" idea from Step 4, still not implemented; costs show "—" like any unpriced model for now.
  - **Manually tested by the user with a real key, fully working**: configured the key, models loaded, ran a real prompt successfully end to end.
- [x] **Anthropic** (`providers/anthropic.rs`) — Messages API, several real wire-format differences from every other provider so far: `x-api-key` header (not `Authorization: Bearer`) + mandatory `anthropic-version` header; system prompt is a top-level `system` field, not a `role: system` message; `max_tokens` is *required* by the API (we fall back to `DEFAULT_MAX_TOKENS = 4096` when unset); `temperature`/`top_p` are deprecated and rejected outright (HTTP 400) for current models unless left at defaults, so `ModelCapabilities` reports both unsupported and we never send them. Registered in `ProviderRegistry`. 3 unit tests.
  - **Manually tested by the user with a real key, fully working end to end.**
- [x] **Google Gemini** (`providers/gemini.rs`) — confirmed this means the **Gemini API** (Google AI Studio, plain API key), not Vertex AI (needs a GCP project + different auth). More wire-format differences: `x-goog-api-key` header (not Bearer — there's a `?key=` query-param fallback too, but the header avoids leaking the key into URLs/logs); the model id is part of the URL path (`.../models/{id}:generateContent`), not a body field; user/system content is `contents`/`systemInstruction` objects made of `parts`; generation params nest under a `generationConfig` object with camelCase names (`topP`, `maxOutputTokens`). `/v1beta/models` reports `inputTokenLimit` and `supportedGenerationMethods` per model directly (filtered on `generateContent`), similar quality to Mistral/OpenRouter — no capability heuristics needed. No reasoning-model-style param restriction assumed for Gemini's "thinking" models (no evidence found, unlike OpenAI/Anthropic). Registered in `ProviderRegistry`. 2 unit tests.
  - `cargo check`/`clippy`/`fmt --check`/`test` (36 passed, 1 ignored) all clean.
  - **Manually tested by the user with a real key, fully working end to end.**
- [ ] **Generic OpenAI-compatible endpoint — deliberately deferred (decided 2026-09-07)**: the user considered testing it against DeepSeek (a well-known OpenAI-compatible API), then realized OpenRouter already lets them test almost anything they'd want to test this way, so there's no pressing need right now. Also surfaced a real design question for whenever this does get built: `ProviderId::OpenAiCompatible` is currently a single fixed slot (one key, one base URL) like every other provider, but a generic "custom OpenAI-compatible endpoint" naturally wants to be **multiple named instances** (e.g. a "DeepSeek" card and a separate "local Ollama" card, each with its own name/base URL/key) — a bigger data-model change (a list of user-defined endpoints, not a fixed enum variant) than a normal provider addition. Settings' provider card would need to show the user-given name instead of a generic label, plus an "add another" affordance. Not implemented — noted for when it's actually prioritized.
- [ ] Pricing entries for each provider once real model ids are confirmed via live testing

## Step 9 — UI polish

- [x] Self-hosted typography: IBM Plex Sans (UI chrome) + IBM Plex Mono (prompt/result content only, not decorative labels) via `@fontsource/*` npm packages — no CDN webfont, no reliance on whatever happens to be installed on the user's system. Wired through `main.css`'s `--font-sans`/`--font-mono` theme variables. `font-mono` applied to both prompt Textareas (Playground + Compare) and all 4 text states of `ResultPanel` (running/error/run-error/empty-placeholder — previously only the actual result `<pre>` used it, so the empty-state hint looked inconsistent).
- [x] Light/dark theme toggle — `stores/theme.ts` (Pinia) + `lib/theme.ts` (shared storage-key/apply helpers so `main.ts` can apply the persisted/default theme before mount, avoiding a flash of the wrong theme). Sun/Moon icon button (`@lucide/vue`) in `App.vue`'s top nav. **Dark is the default** (not light) — the user's explicit preference, confirmed live: "c'est beaucoup plus beau avec le thème sombre... c'est comme ça que j'utilise le thème Nord, je l'ai partout." Persisted per device via `localStorage`.
- [x] Real tooltips (shadcn-vue `Tooltip`/`TooltipProvider`/`TooltipTrigger`/`TooltipContent`, on top of `reka-ui` — not the native `title` attribute, which doesn't render reliably under WebKitGTK) + input validation for the generation params:
  - `GenerationParamsFields.vue`: the always-visible caption text under Temperature/Top P/Max tokens is gone, replaced by a small "?" icon next to each label that shows the same explanation as a hover/focus tooltip. Extracted the repeated "label + '?' icon + tooltip" markup into a new reusable `components/LabelHint.vue`.
  - Values are now clamped on `@change` (leaving the field) to their real valid range — Temperature 0–2, Top P 0–1, Max tokens ≥ 1 (rounded to an integer) — since the native `min`/`max` attributes on a number input only constrain the spinner buttons, not typed or pasted values. Fixes a real functional gap the user flagged: "on peut mettre n'importe quoi dedans, les champs ne sont pas filtrés."
  - Extended tooltips to the rest of the UI the user pointed at: the model-list "↻ Refresh" buttons (Playground + every Compare column) now use a `RefreshCw` icon (spinning while loading) instead of a plain "↻" text button with a native tooltip; Compare's remove-column "✕" button too. Added `LabelHint`-powered explanations for Provider, Model, System prompt, and User prompt in both Playground and Compare (Compare's Provider/Model selects previously had no label or explanation at all, just a placeholder).
  - `components/CopyButton.vue` converted from a text "Copy"/"Copied" button to an icon button (`Copy`/`Check`) with the same text now living in the tooltip instead of always-visible label — matches the new visual language used everywhere else.
  - New `components/ResetButton.vue` (icon button, `Eraser`, tooltip "Clear this field", disabled when the field is already empty) added next to the Copy button on both System prompt and User prompt in Playground and Compare — the user's request for a quick way to blank a prompt field without manual select-all-delete.
  - Adding the shadcn-vue `tooltip` component (`npx shadcn-vue@latest add tooltip`) re-introduced the Google Fonts CDN `@import` in `main.css` that had been deliberately removed back in Step 0/Step 9's self-hosted-fonts work — caught and stripped immediately, self-hosted IBM Plex fonts are unaffected.
- [ ] Revisit contrast: a first Nord-inspired pass landed in Step 6 (light-mode background now `nord5` instead of white, cards `nord6`), but the user found the result had *less* contrast than intended, not more — acceptable for now, but worth a proper look here rather than more ad hoc tweaking. Less urgent now that dark (already higher-contrast) is the default.
- [ ] Select/Input control font size in the toolbars — mid-iteration: grew from `text-sm` to `text-base`, then to an intermediate `text-[15px]` (still felt slightly large to the user at `text-base`). **User is taking this over by hand** — they have live visual feedback and prefer to fine-tune themselves; don't touch these font-size classes further without asking.
- [x] No scrolling when the window is smaller than the content — fixed, but not the way first attempted. First pass tried per-panel internal scrolling (`overflow-auto` on the prompt Card and the Result Card), which hit two real bugs before landing on the right design:
  - A `tailwind-merge` gotcha: `Card`'s built-in default class already includes `overflow-hidden`; overriding it with `overflow-y-auto` doesn't work because `twMerge` doesn't treat `overflow-hidden` (shorthand) and `overflow-y-auto` (longhand) as the same conflict group, so both classes survived in the merged output and which one visually won depended on Tailwind's generated CSS order — fragile, and observed to actually fail in practice. `overflow-auto` (matching group) resolves cleanly instead — confirmed directly with a `tailwind-merge` Node one-liner.
  - Even after that fix, the user clarified per-panel scrolling wasn't the design they wanted at all: **one single scrollbar for the whole window**, under the top nav — not nested scroll regions per panel. Reworked accordingly: `App.vue`'s `<main>` is now the only scroll container (`overflow-y-auto`, horizontal locked via `overflow-x-hidden`); `PlaygroundView`/`CompareView` root divs changed from `h-full` to `min-h-full` (so they can grow taller than the viewport instead of being clamped to it); every intermediate `overflow-hidden`/`overflow-auto` boundary (the two-column grid, the prompt `Card`, `ResultPanel`'s `Card`/`CardContent`, Compare's per-column `Card`) was removed or changed to `overflow-visible` so content is free to grow and overflow up to `main`'s single scrollbar instead of clipping or opening its own nested scroll region. Compare's per-row **horizontal** scroll for browsing many columns side by side is intentional and unchanged (`overflow-x-auto` on that row) — only vertical clipping was the bug.
- [x] Frameless window (the user's "headless" idea, confirmed as Tauri's `decorations: false`) — the native OS title bar (app name + close/minimize/maximize) is gone. `App.vue`'s top nav now doubles as the title bar: `data-tauri-drag-region` makes its empty space draggable, and a custom "×" close button (`@tauri-apps/api/window`'s `getCurrentWindow().close()`, `core:window:allow-close` added to `capabilities/default.json`) is the only window control — deliberately **close only**, no minimize/maximize, per the user's explicit choice. Confirmed working live (close button tested and works); window-dragging couldn't be tested on the user's tiling window manager (Hyprland/Omarchy), which doesn't support free-floating drag the same way — not a bug, just untestable on that setup for now.
- [ ] Resizable panels
- [ ] Advanced params drawer
- [ ] Easy result copy (basic copy button already added to `ResultPanel` in Step 6 — revisit only if it needs more than that)

## Step 10 — CI/CD

Before writing any workflow, walked the user through how this actually plays out for an end
user (installers vs. raw binaries, versioning, whether auto-update is in scope) since they
explicitly asked to understand the process, not just get files dropped in. Decisions made:
**no code signing yet** (unsigned installers, OS shows a warning on first launch — revisit once
there's real user demand), **no native Arch/AUR package yet** (Arch-based users use the
`.AppImage` — Tauri's bundler doesn't produce a `pacman` package natively; documented as the
workaround, logged as a deferred idea for a real AUR package later), and **CI frontend checks
limited to what already exists** (`vue-tsc` + build) rather than retrofitting ESLint/Vitest,
which were never actually set up despite being in the original architecture plan.

- [x] `.github/workflows/ci.yml`: 3 jobs — frontend (`npm run build`, i.e. `vue-tsc` + Vite),
  Rust (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, with the
  WebKitGTK system packages Tauri needs even just to compile), and a `version-consistency` job
  that fails if `package.json`/`Cargo.toml`/`tauri.conf.json`'s version strings ever drift apart.
  All three checks verified locally before committing (`cargo fmt --check`, `cargo clippy
  --all-targets -- -D warnings` — a stricter gate than previously used locally, passes clean with
  0 warnings —, `cargo test` — 36 passed, 1 ignored —, and the version-check shell logic run
  directly). No `eslint`/frontend-test job (see above) and no full `tauri build` dry run on every
  push (kept CI fast; full bundling only happens in `release.yml`, on an actual version tag).
- [x] `.github/workflows/release.yml`: triggered on pushing a `v*.*.*` tag. `tauri-apps/tauri-action@v1`
  matrix: macOS (`aarch64-apple-darwin` + `x86_64-apple-darwin`, separate matrix entries),
  `ubuntu-22.04` (not `-latest` — deliberately the oldest supported LTS, so the built
  `.deb`/`.rpm`/`.AppImage` don't link against a glibc newer than what many users' systems have),
  `windows-latest`. Creates a **draft** GitHub Release (`releaseDraft: true`) with every
  platform's installer attached — a human still reviews and publishes it manually, nothing goes
  live automatically. Verified via `python3 -c "import yaml; yaml.safe_load(...)"` (syntax only —
  actually exercising this workflow needs a real tag push, not done yet since there's no version
  worth releasing so far).
- [x] `docs/release.md` — written now rather than deferred to Step 11, since the versioning/tagging
  process it documents is exactly what this step needed to define. Covers: the 3-file version
  bump + tag procedure, what CI/release automatically do, per-OS install instructions for end
  users (including the AppImage workaround for Arch-based distros, called out explicitly since
  the user runs Omarchy), and an explicit "not yet done" section (code signing, Arch/AUR package,
  auto-update) so it's clear these are deliberate omissions, not oversights.

## Step 11 — Documentation

- [ ] Finalize `README.md`
- [ ] `docs/development.md` (Linux/Windows/macOS setup)
- [ ] `docs/architecture.md`
- [ ] `docs/adding-a-provider.md`
- [x] `docs/release.md` — written in Step 10 (see above), ahead of the rest of this step
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
- **Use OpenRouter's own pricing as an approximate stand-in for other providers — implemented
  2026-09-08**, ahead of the Step 11 documentation pass, at the user's explicit request before
  cutting any release ("c'est quelque chose d'important d'avoir le prix de la requête"). See
  `src-tauri/src/pricing/openrouter_fallback.rs` and STATE.md for the full design (fetch
  OpenRouter's public `/models` catalog once per session, fuzzy-match native model ids against
  OpenRouter's `vendor/model` ids, disclose via a new `Run.cost_is_estimate` flag + a "≈" badge
  and tooltip in the UI). User confirmed it works live ("Ça fonctionne, bravo!").
  - **Follow-up fix, same day**: user reported Mistral's `ministral-3b-latest` showed no cost.
    Root cause: OpenRouter never lists a bare `-latest` entry, only dated snapshots
    (`ministral-3b-2512`, `ministral-3b-2407`, ...) — Mistral's whole model lineup uses this
    rolling-alias convention (`mistral-large-latest`, `mistral-small-latest`, etc.), so this
    affected all of it, not just one model. Fixed by resolving a `-latest` alias to whichever
    dated snapshot for the same base name has the highest version number (works for both
    `YYMM`-style short tags and full `YYYYMMDD` dates, compared as plain integers). Confirmed
    exact vs. approximate vs. no-match are still handled correctly — never guesses a wrong
    model's price. 3 new tests (44 total, up from 41).
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

### Saved prompt sets ("Tests") — user idea, 2026-09-07 (reaffirmed 2026-09-08)

Save a named (system prompt, user prompt) pair with a description/notes field, so it can be
recalled later — **not** tied to a specific provider/model, just the prompt content itself.
Reaffirmed the next day (independently, before checking whether it was already noted) as an
"archive" you can reload prompts from in a later session — and explicitly, **must support
deleting a saved entry**, not just creating/recalling.

This is the concrete feature that would finally exercise the `prompts` and `test_cases` tables
already sitting in the schema since Step 3 (created ahead of use, per docs/start.md's Prompt /
Test Case sections), but note a real mismatch to resolve when this gets designed: the spec's
original model is `Prompt` = system prompt (named, versionable) and `Test Case` = a user prompt
that tests a Prompt (reusable across several Prompts) — two separate, relatable entities. What
the user actually asked for here is simpler: **one saved unit** bundling both system prompt and
user prompt together under one name + description. Don't just bolt this onto the existing two
tables without checking which shape the user actually wants when this is picked up — it may mean
adding a `description` column and treating a "saved test" as its own thing (perhaps a Prompt +
Test Case pair saved together), rather than assuming the original split design still fits.

### AI-assisted system prompt improvement — user idea, 2026-09-07

Two versions of the same idea, from simpler to more ambitious:
1. A button near the System Prompt editor that sends the current system prompt to a powerful
   model for critique/improvement suggestions.
2. A full loop: describe the desired behavior, a powerful "judge/optimizer" model generates a
   system prompt, you run it against a Test Case, and based on the result the model iterates and
   improves the prompt.

This is exactly docs/start.md's "Évolution prévue : optimisation automatique" section — the
spec already describes this future workflow in detail (including cost/latency/quality tradeoffs
across models) and explicitly says not to build it in the MVP, only to keep the architecture
open to it (Runs must be executable programmatically, not just from the interactive UI — already
true: `commands::runs::run_generation` and `commands::experiments::run_experiment` are plain
async functions under the Tauri command layer, callable from anywhere, not wired to any specific
UI flow). The user re-raised it independently after using the tool for a while, which is a good
signal it's a real want, not just a spec artifact — but per the spec's own instructions, still
explicitly deferred past the MVP.

### App versioning strategy — user idea, 2026-09-08 (resolved in Step 10, 2026-09-08)

**Resolved**: semver, `src-tauri/Cargo.toml` as the source of truth, `package.json`/
`tauri.conf.json` kept in sync by hand and checked by CI's `version-consistency` job. Full
procedure documented in `docs/release.md`. Kept here for history; see Step 10 above and
`docs/release.md` for the actual decision.

### Auto-update mechanism — user idea, 2026-09-08

Something like Electron's auto-updater, so installed users get new versions without manually
re-downloading. This is exactly docs/start.md's own line: "Prépare la structure de façon à
pouvoir ajouter ultérieurement la signature des binaires et l'auto-update" — already anticipated
in the original spec as a Step-10-adjacent follow-up, not MVP. Concretely: Tauri has an official
`tauri-plugin-updater` for this, which needs signed release artifacts (code signing explicitly
deferred, see Step 10/`docs/release.md`) and a hosted update manifest (`latest.json`, typically
published alongside GitHub Release artifacts) — depends on Step 10's release pipeline existing
(it now does) and on code signing actually happening first.

### Native Arch Linux / AUR package — user idea, 2026-09-08

The user runs Omarchy (an Arch-based distro) and can't use the `.deb`/`.rpm` Tauri produces.
Tauri's bundler has no built-in `pacman`/AUR target (only `.deb`, `.rpm`, `.AppImage` for
Linux) — a proper Arch package would mean hand-maintaining a PKGBUILD (typically a `-bin`
variant wrapping the GitHub Release binary/AppImage) and publishing it to `aur.archlinux.org`,
which needs its own AUR maintainer account/SSH setup. Explicitly deferred: the `.AppImage`
(portable, no install, works on any distro including Arch-based ones) is an acceptable interim
solution and is documented as such in `docs/release.md`.

### Persist the current prompt draft across app restarts — user idea, 2026-09-08 (implemented same day)

**Implemented**: `stores/promptDraft.ts` now persists `systemPrompt`/`userPrompt` to
`localStorage` (`promptrig.promptDraft`), read on store init and write-through on every change —
same per-device pattern as the "last used provider/model" selection
(`PlaygroundView.vue`'s `loadLastSelection`/`saveLastSelection`). Deliberately scoped to just the
two prompt fields, not the generation params (temperature/top_p/max_tokens stay session-only), per
the user's explicit ask: "je ne parle pas de pouvoir sauvegarder des prompts... juste garder le
dernier prompt système et le dernier user prompt." User confirmed working after a real app
restart. Distinct from the "saved prompt sets" idea above, which is still not built — that's an
explicit named archive of multiple saved prompts, this is just not losing your current
in-progress draft.
