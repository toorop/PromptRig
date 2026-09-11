# PromptRig — TODO

Detailed, checkable task list. Keep this in sync with reality: check items off as they land,
and update [STATE.md](./STATE.md) in the same commit. Full architecture rationale lives in
[docs/architecture.md](./docs/architecture.md) and in the original spec at `docs/start.md`.

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
- [x] Compare (and Playground) model `Select` overflowing its column when the model's display
  name is long (e.g. an OpenRouter listing like "DeepSeek: DeepSeek V4 Flash 0731 (batch)") —
  same root cause as the earlier vertical-scroll bug (Step 9): a flex item's default
  `min-width: auto` refuses to shrink below its content's width, so the select just grew past
  its column instead of truncating. Fixed with `min-w-0` along the flex chain (column → row →
  `SelectTrigger`) in both `CompareView.vue` and `PlaygroundView.vue` (fixed proactively there
  too, even though only Compare was reported, since the same fixed-width `SelectTrigger`
  pattern is used). User confirmed fixed live.
- [x] No scrolling when the window is smaller than the content — fixed, but not the way first attempted. First pass tried per-panel internal scrolling (`overflow-auto` on the prompt Card and the Result Card), which hit two real bugs before landing on the right design:
  - A `tailwind-merge` gotcha: `Card`'s built-in default class already includes `overflow-hidden`; overriding it with `overflow-y-auto` doesn't work because `twMerge` doesn't treat `overflow-hidden` (shorthand) and `overflow-y-auto` (longhand) as the same conflict group, so both classes survived in the merged output and which one visually won depended on Tailwind's generated CSS order — fragile, and observed to actually fail in practice. `overflow-auto` (matching group) resolves cleanly instead — confirmed directly with a `tailwind-merge` Node one-liner.
  - Even after that fix, the user clarified per-panel scrolling wasn't the design they wanted at all: **one single scrollbar for the whole window**, under the top nav — not nested scroll regions per panel. Reworked accordingly: `App.vue`'s `<main>` is now the only scroll container (`overflow-y-auto`, horizontal locked via `overflow-x-hidden`); `PlaygroundView`/`CompareView` root divs changed from `h-full` to `min-h-full` (so they can grow taller than the viewport instead of being clamped to it); every intermediate `overflow-hidden`/`overflow-auto` boundary (the two-column grid, the prompt `Card`, `ResultPanel`'s `Card`/`CardContent`, Compare's per-column `Card`) was removed or changed to `overflow-visible` so content is free to grow and overflow up to `main`'s single scrollbar instead of clipping or opening its own nested scroll region. Compare's per-row **horizontal** scroll for browsing many columns side by side is intentional and unchanged (`overflow-x-auto` on that row) — only vertical clipping was the bug.
- [x] Frameless window (the user's "headless" idea, confirmed as Tauri's `decorations: false`) — the native OS title bar (app name + close/minimize/maximize) is gone. `App.vue`'s top nav now doubles as the title bar: `data-tauri-drag-region` makes its empty space draggable, and a custom "×" close button (`@tauri-apps/api/window`'s `getCurrentWindow().close()`, `core:window:allow-close` added to `capabilities/default.json`) is the only window control — deliberately **close only**, no minimize/maximize, per the user's explicit choice. Confirmed working live (close button tested and works); window-dragging couldn't be tested on the user's tiling window manager (Hyprland/Omarchy), which doesn't support free-floating drag the same way — not a bug, just untestable on that setup for now.
- [x] Visible app version — raised while discussing what an MVP needs before external testers
  (developer friends) get a link, so bug reports can be pinned to a build. User specifically
  wanted a discreet "?" icon over a literal "About" menu entry. A `CircleHelp` button in
  `App.vue`'s nav shows "PromptRig v{version}" in a tooltip on hover — version comes from
  `@tauri-apps/api/app`'s built-in `getVersion()` (reads `tauri.conf.json`), no custom command
  needed; `core:app:allow-version` added to `capabilities/default.json`. User confirmed working.
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
- [x] **Real app icon** — replaced the default Tauri scaffold icon (never customized since Step 0)
  across every bundled format. User generated a source image from a prompt (dark Nord-themed
  square, a terminal `>` chevron + cursor accent, matching the app's own dark-mode identity);
  came back as a 1024×1024 JPEG with white left over in the 4 rounded-corner cutouts. Converted
  to PNG and made those corners transparent (thresholded distance-from-white, with a soft
  falloff for anti-aliasing — verified first that white pixels existed *only* in the 4 corner
  regions, nowhere in the artwork itself, so a global threshold couldn't eat into the design).
  `npx tauri icon app-icon.png` regenerated every platform's icon files from that one source;
  discarded the iOS/Android icon sets it also generates (out of scope — desktop-only app).
  Source `app-icon.png` kept at the repo root for future re-generation. User plans to touch up
  a few small corner artifacts by hand later (Photoshop) — not blocking.
- [x] **First real release cut**: tagged and pushed `v0.1.0`. `release.yml` ran for the first
  time ever — all 4 platform jobs (macOS arm64/x64, Linux, Windows) succeeded on the first
  attempt, draft release created with all 9 expected installer assets. Still a draft, pending
  the user's review/testing before publishing.
  - **Real bug found from this first release**, via the user's own testing on this dev machine:
    the `.AppImage` aborts on launch (`Could not create GBM EGL display: EGL_SUCCESS`) on
    NVIDIA+Wayland — a harsher variant of the known `tauri dev` issue, this time a hard crash in
    the packaged binary rather than a soft protocol error. Reproduced and fixed directly (same
    machine): `WEBKIT_DISABLE_DMABUF_RENDERER=1` before the binary fixes it, same as the dev
    workaround. Documented in `README.md`'s Linux install section and a new "Troubleshooting"
    section in `docs/release.md` (applies to `.deb`/`.rpm` too, not just the AppImage).
  - **Second real bug found, from a friend's testing on macOS**: downloading the `.dmg` via
    Chrome and opening the app shows `"PromptRig" is damaged and can't be opened. You should
    move it to the Bin.` — not the milder "unidentified developer" prompt the docs originally
    described, because the app has *no code signature at all* (not just "unnotarized"); the
    right-click → Open bypass only works when there's some signature to trust. Correct fix:
    `xattr -cr /Applications/PromptRig.app` (strips the quarantine attribute the browser
    download added). The **README's original macOS instructions were wrong** — written before
    any real signed-vs-unsigned testing happened — corrected in both `README.md` and
    `docs/release.md`.

## Step 11 — Documentation

- [x] Finalize `README.md` — rewrote the status line (no longer "not yet usable end-to-end"),
  features list (marked what's actually shipped vs. the still-unimplemented generic
  OpenAI-compatible endpoint), tech stack (added Pinia/specta/tauri-specta), and removed every
  "(once written)" doc link now that they all exist.
  - Two screenshots (Playground, Compare) added by the user directly via GitHub's web editor
    (drag-and-drop) — pulled in locally.
  - Follow-up, same day: the user pointed out the first pass had no actual **end-user install
    instructions** (only a dev setup section). Added an "Installation" section — a link to
    GitHub's `/releases/latest` plus a condensed per-OS quick-start (Windows `.exe`, macOS
    `.dmg`, Linux `.deb`/`.rpm`/AppImage-for-everything-else), pointing to `docs/release.md` for
    the full detail. Flagged inline that the link 404s until a release is actually tagged, since
    none exists yet.
- [x] `docs/development.md` — prerequisites per OS (Linux system packages, the NVIDIA/Wayland
  `WEBKIT_DISABLE_DMABUF_RENDERER=1` workaround, Windows/macOS build tools), running the app,
  the exact verification commands CI runs (so a contributor can catch a red CI run locally
  first), and the keyring integration test's `--ignored` flag.
- [x] `docs/architecture.md` — the Run/Experiment domain model (a comparison is just N Runs
  sharing an Experiment, no separate concept), backend module breakdown, the provider
  abstraction, why `rusqlite` directly instead of `tauri-plugin-sql`, the keyring boundary, the
  two-tier cost estimation design (exact `pricing.json` → OpenRouter's own price when the Run
  is OpenRouter itself → cross-provider approximation → "—"), the specta/tauri-specta type-sync
  mechanism (including why `i64` ids cross IPC as strings), frontend store/state conventions,
  and an explicit "deferred by design" section (streaming, the multi-instance OpenAI-compatible
  endpoint, auto-update/signing/AUR).
- [x] `docs/adding-a-provider.md` — a concrete step-by-step walkthrough (add a `ProviderId`
  variant, write the module, register it, test it) built from what the 5 real provider
  integrations actually had to figure out (auth header shape, where the model id/system prompt
  goes, which params a model rejects, how capabilities are discovered) — a checklist of the real
  gotchas to check against the vendor's actual docs, not assumptions carried over from another
  provider.
- [x] `docs/release.md` — written in Step 10 (see above), ahead of the rest of this step
- [x] Confirm `TODO.md`/`STATE.md` reflect actual project state — this pass

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
  2026-09-08, later fully superseded by models.dev, 2026-09-11 (see below)**. Originally built
  ahead of the Step 11 documentation pass, at the user's explicit request before cutting any
  release ("c'est quelque chose d'important d'avoir le prix de la requête"): fetched OpenRouter's
  public `/models` catalog once per session, fuzzy-matched native model ids against OpenRouter's
  `vendor/model` ids, disclosed via a `Run.cost_is_estimate` flag + a "≈" badge and tooltip in
  the UI. Once the user found models.dev (see "Use models.dev..." below) — which indexes by each
  provider's own native model id directly, needing no fuzzy matching at all — this whole design
  (the `pricing/openrouter_fallback.rs` module, and the now-redundant hand-curated
  `pricing/pricing.json` it existed alongside) was deleted outright rather than kept as a
  secondary fallback, per the user's own KISS call. `cost_is_estimate`/the "≈" UI stayed in
  place (always `false` now) rather than being ripped out too.
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

**Follow-up, 2026-09-08**: set up desktop integration for the AppImage on the user's own machine
(a `promptrig` PATH command that always runs whichever `PromptRig_*.AppImage` in `~/Downloads`
was modified most recently, an icon installed into `~/.local/share/icons/hicolor/`, and a
`.desktop` entry so it shows up in the app launcher) — tested working end to end. Generalized
(no hardcoded username/paths) and added to `docs/release.md`'s Linux section as an optional
step for other Arch/Omarchy-type users in the same situation, since this doesn't require an AUR
account and takes a few minutes.

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

### Searchable/filterable model picker with per-model pricing — implemented 2026-09-08

**Implemented**, combining this idea with a second one the user raised in the same conversation
("dans le Select, une fois déplié, [...] le tarif approximatif de chaque modèle" — a per-model
rate shown in the dropdown, in a smaller font).

- New `src/components/playground/ModelCombobox.vue` (Popover + Command from shadcn-vue —
  there's no standalone "Combobox" component in the registry, it's the standard recipe built
  from those two primitives) replaces the plain `<Select>` for the **model** picker only, in
  both `PlaygroundView.vue` and `CompareView.vue`. The **provider** picker stays a plain
  `<Select>` (5-6 items, no need for search). Typing in the search box filters the list live via
  `Command`'s built-in fuzzy-contains matching.
- Backend: `ModelInfo` gained a new `pricing: Option<ModelPricing>` field
  (`input_per_million_usd`/`output_per_million_usd`/`is_estimate`) — a *rate* for display, not a
  computed cost (no token usage exists yet at picker time, unlike `Run.estimated_cost_usd`).
  Filled in by `commands::providers::list_models` (provider modules themselves stay unaware
  pricing exists), reusing the exact same resolution order as Run cost estimation via a new
  shared `pricing::resolve_rate` (refactored out of `build_new_run` and
  `OpenRouterPricingCache::estimate_for_openrouter`/`estimate_fallback`, which are now thin
  wrappers around new `lookup_for_openrouter`/`lookup_fallback` methods — no duplicated
  exact-then-fallback logic between "compute a cost" and "show a rate").
- **UI iteration from live feedback**: the first version put the rate to the right of the model
  name on the same line, which visibly ate into the name's available width and caused more
  truncation than before pricing existed (confirmed via a screenshot — long OpenRouter names
  like "DeepSeek: DeepSeek V3.2 Exp (free)" were cut down to "DeepSeek: Deep..."). Fixed by
  stacking the rate *below* the name instead of beside it (name gets the full row width and
  wraps instead of truncating — the user was explicit that showing the complete name is a hard
  requirement) and widening the popover (`w-80` → `w-96`). Also dropped the "per 1M" suffix
  (user: not useful, just take up space) — the rate reads as compact `$X.XX/$Y.YY`.
- Adding `command`/`popover` via the shadcn-vue CLI re-triggered the same Google Fonts CDN
  regression seen before with `tooltip` — stripped each time it reappeared (3 times, across
  `command`, `input-group`, and `dialog` — the latter two are real transitive dependencies of
  `command`'s barrel export, not bloat, even though they looked unrelated to a model picker at
  first glance).
- `cargo check`/`clippy --all-targets -- -D warnings`/`fmt --check`/`test` (44 passing, unchanged
  count — pure refactor + additive change, no new backend tests added for the enrichment path
  itself) all clean. `npm run build` clean (one real TS issue caught: specta exports every bare
  `f64` as `number | null`, even non-`Option` ones, since NaN/Infinity have no JSON
  representation — not something `Option`-wrapping the Rust field would have changed). Verified
  live via `tauri dev` restarts at each iteration; user confirmed the final result ("Magnifique!").

### Remember window size across restarts — implemented 2026-09-08

**Implemented**: registered the official `tauri-plugin-window-state` (v2.4.1) in `lib.rs` —
`.plugin(tauri_plugin_window_state::Builder::default().build())`, no other wiring needed (it
hooks window creation/move/resize/close itself, restoring size + position + maximized state on
next launch). Default `StateFlags::all()` used rather than restricting to just size, since
remembering position alongside size is the expected companion behavior for basically every app
that does this. `cargo check`/`clippy --all-targets -- -D warnings`/`fmt --check`/`test` (44,
unchanged) all clean.

**Not meaningfully verified live**: this dev machine runs Hyprland (tiling), where window
size/position is managed by the compositor regardless of what an app requests, so restore
behavior isn't visible here — and forcing a "graceful window close" via `hyprctl` to at least
confirm a state file gets written didn't work either (this system's dispatcher has a
non-standard Lua-based syntax, not stock `hyprctl`). The integration follows the plugin's
official documented usage exactly (a single `.plugin(...)` call, no custom flags needed for the
default behavior), so confidence is reasonably high, but real confirmation needs the user (or a
friend) testing on an actual traditional WM/OS.

### Slightly larger top-nav font — resolved by the user directly, 2026-09-08

**Resolved**: the user hand-edited `App.vue` themselves (as they'd said they might) — the nav
links (Playground/Compare/Settings) are `text-[15px]`, the "PromptRig" wordmark `text-[16px]`
(both were `text-sm`/14px). Along the way, confirmed for them that Tailwind has no named size
between `text-sm` (14px) and `text-base` (16px) — the arbitrary-value syntax `text-[15px]` is
the way to hit an in-between value, same technique already used for the toolbar controls below.

### Pin a Compare column so "Run all" skips it — implemented 2026-09-08

**Implemented**, plus a persistence extension the user asked for in the same conversation
("je pense que ce serait bien qu'on retrouve au redémarrage de l'application les cartes qui ont
un PIN... si ça apporte beaucoup de complexité, on ne le fait pas" — assessed as low-complexity
given the pattern was already established elsewhere, so both landed together):

- `CompareColumn` (`stores/compare.ts`) gained `pinned: boolean`. `runAll()` in `CompareView.vue`
  filters to unpinned columns before building `RunExperimentInput` and zips the response back
  against that same filtered array (not the full `columns.value`, which would misalign indices).
  `canRunAll` now also requires at least one unpinned column, so the button disables instead of
  silently no-op'ing when everything is pinned. Turned out the backend needed **no changes at
  all** — `run_experiment` was already called with an explicit column list, not "all columns
  implicitly"; a pinned column's *old* `Run` (from whatever Experiment originally produced it)
  just stays displayed, untouched, since it's simply never included in a new request. The
  "does this interact with `run_experiment` always creating a fresh Experiment" concern noted
  when this idea was first logged turned out to be a non-issue.
- Pin toggle: a `Pin`/`PinOff` icon button next to the existing "✕ remove", plus a colored ring
  around a pinned card's `Card` for at-a-glance visibility. Pinning doesn't restrict anything
  else — a pinned column's provider/model can still be changed and it can still be individually
  rerun via its own "Rerun this column" button; pinning only excludes it from the *bulk* sweep.
- **Persistence**: pinned columns (provider, model, full `Run` result) are saved to
  `localStorage` (`promptrig.compare.pinnedColumns`) via a `watch(columns, ..., { deep: true })`
  that re-derives the whole saved list from current state on every change — deliberately not
  tracking add/remove/toggle as separate special cases. Restored on store init (unpinned columns
  always start fresh, as before this feature — only pinned ones survive a restart). This same
  "re-derive from current state" design is also why removing a pinned column via "✕" correctly
  clears it from the saved list too, with no extra code needed for that case specifically — the
  user asked about this exact scenario and it was already handled by construction, confirmed by
  tracing the watcher rather than assumed.
- User confirmed working live (pinned a column, restarted the app, it was still there).

### Use models.dev as a pricing/model-metadata source instead of (or alongside) OpenRouter — implemented 2026-09-11

The user found [models.dev](https://models.dev/) — the open-source (MIT), community-maintained
model/pricing database that powers OpenCode's own model picker — and flagged it as a possible
better source than the current OpenRouter-based approach (`pricing::openrouter_fallback`,
Step 8/MVP work). Confirmed via its GitHub repo (`github.com/sst/models.dev`) before logging
this, rather than assuming: it publishes plain JSON endpoints with no API key needed —
`https://models.dev/api.json` (provider-inclusive), `models.json` (model-only), and
`catalog.json` (combined) — each entry carrying per-million-token cost (input/output/reasoning/
cached/audio separately), context/output token limits, capability flags (`reasoning`,
`tool_call`, `structured_output`, `temperature`, attachment/modality support), and release/
knowledge-cutoff dates. Updated continuously via GitHub PRs with schema validation, not a fixed
release cadence.

Why this could be a real improvement over the current design, if picked up later: our OpenRouter
fallback (see `pricing/openrouter_fallback.rs`'s module doc) has to *fuzzy-match* a native
provider's model id against OpenRouter's own differently-shaped `vendor/model` ids (normalizing
case/punctuation, stripping dates, resolving `-latest` aliases, etc.) precisely because
OpenRouter's catalog exists to describe *OpenRouter's own* proxied models, not to be a neutral
cross-provider registry — real misses are expected and accepted as a known limitation. models.dev
is explicitly built as that neutral registry instead, so — depending on how it actually keys
entries per provider (not yet checked in detail; the fields above come from the repo's README
description, not a hands-on look at real payload data) — it might resolve directly by
provider+model id with no fuzzy matching needed at all, and could yield *exact* rather than
approximate cross-provider costs, retiring the `cost_is_estimate` ceiling caveat for whichever
models it actually covers. Also separately relevant to the already-logged "update pricing.json
from an external source" idea (see the OpenRouter/LiteLLM entries above) — models.dev's own MIT
license and "this is the whole point of the project" positioning is a stronger fit than LiteLLM's
previously-flagged "reference/cross-check only, no hard runtime dependency" caution, though that
tradeoff (any external dependency risk) is worth revisiting on its own merits when this is
actually picked up, not assumed away here.

**Implemented, 2026-09-11.** Before writing any code, fetched and inspected a real `api.json`
payload (4.5 MB, 213 providers) rather than trusting the README's field list — confirmed it
really does eliminate the fuzzy-matching problem rather than relocating it: models.dev keys
directly by each provider's own native model id, and the exact real-world case that needed the
old alias-resolution hack (Mistral's `ministral-3b-latest`) is present as its own real entry
with real pricing ($0.04/$0.04). Coverage checked broad and reliable (44/48 OpenAI models priced,
14/14 Anthropic, 34/39 Google, 32/34 Mistral, 357/361 OpenRouter); cross-checked `gpt-4o-mini`
against the old hand-curated `pricing.json` and got an exact match before deleting that file.

Decided with the user (two explicit calls, not assumed): **no further fallback** when a model is
missing from models.dev (just "—", same as today — dropped the whole fuzzy-matching apparatus
rather than keeping it as a secondary safety net) and **delete `pricing.json` outright** (one
source instead of two, no more manual upkeep). See `src-tauri/src/pricing/models_dev.rs`'s
module doc for the full design; `docs/architecture.md`'s "Cost estimation" section rewritten to
match.

Also implements the user's follow-up ask in the same conversation: `commands::providers::
list_models` now hides any model models.dev marks `"deprecated"` (a `"beta"` model is still
shown, per their explicit call) — one models.dev lookup per model covers both the price and this
deprecation check.

**Deliberately over-parses for planned future features** (the user's explicit ask, since they
plan to eventually use these): `pricing::models_dev::ModelEntry` captures modalities
(multimodal input/output types), `reasoning`/`reasoning_options` (effort levels like
none/low/medium/high/xhigh/max), `tool_call`/`structured_output`/`temperature` support, and
context/input/output token limits — none of it consumed yet beyond `cost`/`status`, but parsed
now so a later multimodal-aware picker or reasoning-effort selector doesn't need to redo the
fetch/parse layer. Marked `#[allow(dead_code)]` with a comment explaining why, rather than
leaving it to accumulate silent warnings or getting deleted as apparently-unused.

5 new unit tests (`pricing::models_dev`) replacing the 13 that covered the deleted
`openrouter_fallback`/`PricingTable` modules — net simpler test surface, matching the net
simpler implementation. `cargo check`/`clippy --all-targets -- -D warnings`/`fmt --check`/`test`
(37 passing, 1 ignored) and `npm run build` all clean.

**Regression found and fixed the same day**: the user reported prices had disappeared entirely
from the model picker after this landed. Root cause, found by adding a temporary throwaway test
that did a real fetch against the live `api.json` (the shipped unit tests only ever exercised a
small hand-written sample, which never exposed this): `ModelEntry`'s `reasoning_options` field
deserializes the *entire* `api.json` in one `serde_json::from_str` call across all 213 providers,
not just the 5 PromptRig maps — so one malformed entry anywhere in that file fails the whole
fetch, silently, for every provider. Two real-world shapes the strict struct didn't allow for:
a `reasoning_options` entry can be `{"type": "toggle"}` or `{"type": "budget_tokens"}` with no
`values` field at all (only `"effort"`-style options list concrete levels), and even where
`values` is present, an individual entry inside it can itself be `null` (seen on one Sarvam
model: `["values": [null, "low", "medium", "high"]]`, meaning "no explicit level" alongside named
ones). Fixed by widening `ReasoningOption.values` from `Vec<String>` to
`Option<Vec<Option<String>>>` — matches the real upstream shape rather than assuming a cleaner one
from the README. Re-verified against a real fetch (`gpt-4o-mini` → $0.15/$0.60,
`claude-opus-4-5` → $5/$25, all 213 providers parse) before removing the throwaway debug tests.
`cargo check`/`clippy -D warnings`/`fmt --check`/`test` all clean again after the fix.

**Follow-up, same day: `ETag`-based conditional fetch + disk cache.** The user noticed
models.dev's response carries an `ETag` and asked to use it to avoid re-downloading the whole
~4.5 MB payload on every launch when nothing changed — also correcting a wrong assumption of
theirs along the way (there is no database step at all today; `ModelsDevCache` only ever kept the
parsed result in memory for the session). Confirmed the real response does carry a Cloudflare
`ETag` before building anything. Got two explicit decisions from the user via `AskUserQuestion`:
persist a small disk cache (not SQLite — it's not relational data) next to the SQLite file in the
app data dir, and fall back to that disk cache (rather than showing no price) whenever the
request can't complete at all (offline, DNS failure, non-2xx status), not just on `304`.

Implemented in `pricing/models_dev.rs`: two flat files (`models_dev_cache.json` for the raw body,
`models_dev_cache.etag` for the `ETag` value — kept separate so the body never needs
JSON-escaping). `fetch()` now sends `If-None-Match` when a disk cache exists; on `304 Not
Modified` it reparses the disk-cached body instead of downloading again; on any request failure
or non-2xx status, it falls back to the disk cache if one exists (erroring only when there's
truly nothing cached yet); on a fresh `200`, it parses the new body and only *then* overwrites the
disk cache (so a malformed response never clobbers a known-good cache). `ModelsDevCache::new` now
takes the cache directory (`lib.rs` passes the same `app_data_dir` the SQLite file already lives
in) instead of being parameterless.

Verified against the real endpoint with a temporary throwaway test (removed after): first fetch
into an empty directory wrote both cache files and returned a price; a second `ModelsDevCache`
pointed at the same directory got a `304` and returned the identical price from disk, with no
`GET` for the full body. 2 new unit tests cover the disk-cache read/write round trip in isolation
(a temp directory per test, no real network call). `cargo check`/`clippy -D warnings`/
`fmt --check`/`test` (39 passing, 1 ignored) all clean.

### Slow/no connection could stall the UI on the models.dev fetch — user idea, 2026-09-11 (option 1 done 2026-09-11)

Raised right after the `ETag`/disk-cache work above, but deliberately **not implemented yet** —
the user is out of session time and asked to just note it down for a later session, not act on
it now.

The concern: on a fast connection the ~4.5 MB `api.json` download is invisible, but on a very
slow or flaky connection (or a fresh install with genuinely no network yet), the first command
that needs pricing — opening a provider's model list, or running a prompt — awaits
`ModelsDevCache::price`/`is_deprecated`, which awaits the fetch. Two real gaps worth checking
against the current code before doing anything:
- `reqwest::Client::new()` (see `ModelsDevCache::new`) sets **no request/connect timeout at
  all** — a stalled-but-not-dropped connection could hang the awaiting command indefinitely
  rather than failing fast into the disk-cache fallback that was just built. This looks like a
  real, separate bug worth fixing regardless of the rest of this idea (a small `.timeout(...)` on
  the `Client` builder).
- Even with a timeout, `commands::providers::list_models` and the Run-cost path both `.await` the
  models.dev lookup inline before returning anything to the frontend — so a slow-but-eventually-
  successful fetch still delays the whole model list / Run result by however long the download
  takes, not just a hard freeze on a dead connection.

The user's specific ask: a **Settings toggle to disable the models.dev download entirely** —
for someone who'd rather never wait on it (and never see prices) than risk a stall. Options to
weigh next session, not decided yet:
1. Just the timeout fix (fast, uncontroversial, fixes the worst case — an indefinite hang —
   regardless of what else is decided).
2. The user's requested opt-out toggle in Settings, persisted like other settings, checked before
   `ModelsDevCache` ever calls `fetch()` (skips pricing/deprecation-filtering entirely when off).
3. A non-blocking fetch instead of (or alongside) a toggle: return models/Run results immediately
   without price, then fill pricing in asynchronously once the fetch resolves — fixes the freeze
   for everyone by default, no setting required, but is a bigger change (the frontend would need
   to handle a model list / Run whose price arrives later).

No decision needed right now — just verify the timeout gap against the real `reqwest` behavior,
and ask the user which combination of (1)/(2)/(3) they want before writing any code.

**Option 1 done, 2026-09-11**: `ModelsDevCache::new` now builds its `Client` with a 10s
`.timeout(...)` instead of `Client::new()`'s no-timeout default — a stalled connection now fails
into the existing disk-cache fallback in `fetch()` instead of hanging the awaiting command
indefinitely. Options 2 (Settings opt-out toggle) and 3 (non-blocking fetch) remain unstarted;
not decided whether either is still wanted now that the worst case (indefinite hang) is fixed.

### `CHANGELOG.md` + `v0.3.0` release — done 2026-09-11

Added `CHANGELOG.md` at the repo root (Keep a Changelog format), backfilling `0.1.0` and `0.2.0`
from git history/STATE.md and adding a `0.3.0` entry for the models.dev pricing work above.
`docs/release.md` now says to add the CHANGELOG entry as part of the version-bump commit for
every future release. Also fixed two stale README lines noticed in passing ("Status:
pre-release" / "No release has been tagged yet" — both predate `v0.1.0`). Version bumped to
`0.3.0` and tagged.

### Filter the model picker to deprecated + text-only models — implemented 2026-09-11

The user noticed many inactive/wrong-modality models still showing in the picker and asked to
filter them out, using models.dev's own data rather than guessing. Investigated the real
`api.json` before writing anything: confirmed `status: "deprecated"` (already filtered, done in
the earlier models.dev work) correctly flags old models (`gpt-4`, `gpt-3.5-turbo`, `o1`, ...), but
disproved the user's second hypothesis ("no price = inactive") with real counter-examples — Google's
free/open-weights Gemma models and OpenRouter's router pseudo-models (`openrouter/auto`, etc.) have
no fixed price but are very much active — so that idea was dropped rather than implemented (would
have hidden legitimate models). Confirmed via `AskUserQuestion`.

Implemented instead: `ModelsDevCache::is_text_only(provider, model_id)` — keeps a model only if its
models.dev `modalities` data shows it accepts `"text"` input and produces *only* `"text"` output
(excludes image/video/audio-generation and realtime-voice models even when they also happen to
emit text, since this app only ever sends/renders plain text; models with no modality data at all
are kept, same "missing data never hides a model" policy as `is_deprecated`). Wired into
`commands::providers::list_models` right after the existing deprecated-filter. 5 new unit tests.
`cargo check`/`clippy -D warnings`/`fmt --check`/`test` (40 passing) and `npm run build` clean.

### Fix: reasoning ("thinking") models could crash a Run instead of erroring cleanly — implemented 2026-09-11

The user hit `unexpected OpenRouter response: error decoding response body` running a Gemini
"thinking" model through OpenRouter, and correctly guessed it was related to `max_tokens` and
reasoning models. Root cause confirmed: `providers/openai.rs`, `mistral.rs`, and `openrouter.rs`
all typed the response's `message.content` as a required `String`, but the real Chat Completions
API returns `content: null` (with `finish_reason: "length"`) when a reasoning model spends its
entire token budget on internal reasoning before emitting any visible text — `serde` then fails to
deserialize the *whole* response, surfacing as a raw, unhelpful reqwest error instead of a normal
failed Run. (Anthropic and Gemini's native providers already had this field as `Option<String>` —
not affected.)

Fixed all three: `content` is now `Option<String>`, `Choice` also carries `finish_reason`, and a
new pure `extract_text(choices)` function (mirrors `build_request`'s "kept separate so it's
testable without a network call" pattern) turns a null/empty content into a specific, actionable
error — "the model likely spent its entire max_tokens budget on internal reasoning before
answering. Try raising max_tokens." — when `finish_reason == "length"`, or a generic empty-content
error otherwise. The failed Run is still persisted with this message, same as any other provider
error. 10 new unit tests (ordinary content, reasoning-budget case, generic empty-content case, no
choices) across the three files. `cargo check`/`clippy -D warnings`/`fmt --check`/`test`
(50 passing) and `npm run build` clean.

**Follow-up implemented same day, 2026-09-11**: higher default `max_tokens` (1024→4096) and lower
default temperature (0.7→0, better suited to reproducible prompt-testing runs than the old
general-purpose default), plus a reasoning-effort control — see the next entry for the full design.
Still not done: surfacing how much of the token budget went to reasoning vs. the answer (models.dev
doesn't expose this as a static field; would need reading it back from the provider's own usage
response, e.g. OpenAI's `completion_tokens_details.reasoning_tokens`) — not started, no decision
made on whether it's worth it yet.

### Reasoning-effort control (UI + backend), implemented 2026-09-11

Scoped via `AskUserQuestion` to the `"effort"`-style control only (not Anthropic's
`budget_tokens`/`toggle`) — see STATE.md for the full design and the per-provider wire-format
verification (OpenAI/Mistral: top-level `reasoning_effort`; Gemini: nested
`generationConfig.thinkingConfig.thinkingLevel`; OpenRouter: nested `reasoning: {effort}`;
Anthropic explicitly excluded — no real API field to send an effort string to).

New `ReasoningEffortSelect.vue` component, hidden entirely when the selected model reports no
levels (models.dev's `ModelEntry.reasoning_options`, surfaced via the new
`ModelInfo.reasoning_effort_levels`) — most Gemini reasoning models fall in this "no control at
all" bucket, so this is the common case, not a corner case. Playground gets one shared instance;
Compare gets one *per column* (a real `AskUserQuestion` decision — each column can have a different
model with non-overlapping valid values, e.g. OpenAI's minimal/low/medium/high vs. Mistral's
none/high, so a single global control could send an invalid value to some column's model). This
needed a small backend change too: `commands::experiments::ComparisonColumn` gained its own
`reasoning_effort` field, separate from the shared `RunExperimentInput::params`, since
`run_experiment` previously had no way to vary a param per column.

**Correction, same day**: initially assumed Anthropic had no real "effort" field (only the older
`thinking: {budget_tokens}`) and left it out. The user asked about it directly; re-checking
Anthropic's current docs (rather than trusting that stale assumption) found a real top-level
`output_config.effort` field, independent of `thinking` mode, matching models.dev's per-model
`"effort"` list exactly (including correctly excluding Sonnet 4.5/Haiku 4.5, which don't support
it). Wired it in the same simple way as every other provider — see STATE.md for the model list and
implementation detail. All five implemented providers now support reasoning effort; nothing
provider-specific left unimplemented in this feature.

**Deliberately not done, follow-ups if ever needed**: surfacing actual reasoning-token usage after
a Run (e.g. OpenAI's `completion_tokens_details.reasoning_tokens`) — not started, no decision made
on whether it's worth it yet.
