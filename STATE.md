# PromptRig — Project State

Hand-off document: what's done, what's in progress, decisions made, and known issues — enough
for a new session to resume cleanly. **Update this file in every commit that changes project
state.** See [TODO.md](./TODO.md) for the detailed task checklist, [AGENTS.md](./AGENTS.md) for
the working agreement (commit/state discipline, destructive-command precautions, one step at a
time with a pause for review before commit/push), and
`/home/toorop/.claude/plans/purrfect-frolicking-donut.md` for the full architecture plan
approved by the user.

## Current step

Step 9 (UI polish) is done as of 2026-09-08 — the user explicitly signed off ("pour moi c'est
correct... on peut considérer le point 9 pour l'instant fait"), with two known-open items left
on purpose: light-theme contrast (still unresolved, kept open per the user's explicit choice —
see TODO.md) and Select/Input toolbar font sizing (the user is hand-tuning it themselves).

Step 10 (CI/CD) is functionally done, 2026-09-08. `ci.yml` and `release.yml` are both written,
committed, and pushed. `ci.yml` has now run for real on GitHub (all 3 jobs green — see below);
`release.yml` still hasn't (needs a real `v*.*.*` tag push, and there's no version worth
releasing yet).

Before moving to Step 11 (documentation), the user asked for two more MVP-blocking features
first, both now done, tested, and pushed, 2026-09-08:
1. Cross-provider cost estimates via OpenRouter's pricing ("avant de faire une release... il y a
   un truc que j'aimerais que l'on fasse pour le MVP... régler cette histoire de tarifs"),
   including a same-day follow-up fix for Mistral's `-latest` alias models showing no price —
   see below for both.
2. Persisting the current System/User prompt draft across app restarts (the deferred idea from
   the previous session), plus a real bug caught and fixed along the way: the frameless window's
   drag region (Step 9) was silently broken on *every* platform (not just untestable on the
   user's tiling WM as first assumed) — `core:window:allow-start-dragging` was never added to
   `capabilities/default.json`, so `data-tauri-drag-region` failed with an unhandled permission
   rejection visible in the dev console.

A third pre-release item followed: a real app icon (the default Tauri scaffold icon had never
been replaced). User generated a source image, it needed a JPEG→PNG conversion and the white
corner cutouts made transparent — done, `npx tauri icon` regenerated every bundled format. User
noticed minor corner artifacts they'll fix by hand in Photoshop later — explicitly not blocking.
**Committed locally but deliberately not pushed** — the user's call, to avoid waiting on CI for
a change with nothing meaningful to verify.

**Step 11 (documentation) is done**, 2026-09-08: `docs/architecture.md`, `docs/development.md`,
and `docs/adding-a-provider.md` written from scratch (facts cross-checked directly against the
current source — migration filenames, function names, `ProviderId::ALL`, etc. — not just
recalled from memory); `README.md` rewritten to reflect the app's actual current state instead
of the Step-0-era "not yet usable" placeholder; `AGENTS.md`'s doc-links section updated to point
at the now-real files instead of "(once written)". `npm run build`/`cargo check` both re-verified
clean after (docs shouldn't affect either, but confirmed rather than assumed).

With Step 11 done, every step in the original plan through "MVP-ready" is complete. The user
added the README's two screenshots directly via GitHub's web editor (drag-and-drop) as planned;
pulled that in locally (`648738c`).

Also fixed, same day: Compare's (and, proactively, Playground's) model `Select` overflowing its
column for a long display name (e.g. an OpenRouter listing like "DeepSeek: DeepSeek V4 Flash
0731 (batch)") — same root cause as the Step 9 vertical-scroll bug, a flex item's default
`min-width: auto` refusing to shrink below its content's width. Fixed with `min-w-0` along the
flex chain in both views. **Committed locally, not pushed** (same reasoning as the icon commit —
user's call to skip the CI wait for a low-risk visual fix); `master` is ahead of
`origin/master` by two commits now (this one + the one below).

Then the user pointed out `README.md` still had no actual **end-user install instructions**
(only a dev setup section) — added an "Installation" section: a link to GitHub's
`/releases/latest`, plus a condensed per-OS quick-start (Windows `.exe`, macOS `.dmg`, Linux
`.deb`/`.rpm`/AppImage), pointing to `docs/release.md` for the full detail. Explicitly flagged
inline that the release link 404s for now, since no version has actually been tagged yet.
Committed locally, also not pushed (no functional reason to — nobody's editing this file via
GitHub the way the screenshots needed pushing to work).

Asked the user directly whether they saw anything else worth adding before handing the project
to developer friends to test. Flagged three things; user's calls on each: (1) cutting and
smoke-testing a real release themselves, before sending any link — they'll do this, "ça ne
devrait pas tarder"; (2) no version number visible anywhere in the app, making bug reports from
testers hard to pin to a build — user agreed, wanted it added, and specifically suggested a
discreet "?" icon over a literal "About" menu entry; (3) whether the NVIDIA+Wayland
`WEBKIT_DISABLE_DMABUF_RENDERER` workaround (documented for `tauri dev`) also applies to the
*packaged* binary — user's call: if it works on their own machine, that's good enough for this
MVP, not worth pre-emptively investigating further.

Implemented (2): a small `CircleHelp` icon button in `App.vue`'s nav (before the theme toggle),
showing "PromptRig v{version}" in a tooltip on hover. Version comes from `@tauri-apps/api/app`'s
built-in `getVersion()` (reads `tauri.conf.json`, no custom Tauri command needed) — added
`core:app:allow-version` to `capabilities/default.json` (confirmed as a real permission id in
the generated schema, same way `allow-close`/`allow-start-dragging` were in Step 9). `npm run
build` clean; verified live via a full `tauri dev` restart (capability changes need one, HMR
doesn't pick them up) — no permission errors, tooltip shows correctly. User confirmed working
("ça fonctionne, c'est bien") before requesting the commit.

**`v0.1.0` has been cut**, 2026-09-08. Pushed the 3 pending commits to `master`, let `ci.yml` go
green on that commit, tagged `v0.1.0`, pushed the tag. `release.yml` ran for the very first
time — **all 4 platform jobs succeeded on the first attempt** (macOS arm64, macOS x64,
ubuntu-22.04, windows-latest, ~8-9 min each). Draft release `PromptRig v0.1.0` created with all
9 expected assets (`.exe`, `.msi`, 2×`.dmg`, 2×`.app.tar.gz`, `.deb`, `.rpm`, `.AppImage`). Its
URL temporarily shows `releases/tag/untagged-<hash>` instead of `v0.1.0` — a known GitHub quirk
for draft releases, resolves once published. **Still a draft** — the user reviews/tests before
publishing, nothing is public yet.

**First real-world bug found via the user's own testing**: downloaded and ran the `.AppImage` on
this same dev machine (NVIDIA + Hyprland/Wayland) — immediate crash: `Could not create GBM EGL
display: EGL_SUCCESS. Aborting... Aborted (core dumped)`. A harsher variant of the already-known
`tauri dev` NVIDIA/Wayland issue (that one is a non-fatal Wayland protocol error; this one is a
hard abort in the packaged binary — same underlying WebKitGTK/NVIDIA-GBM incompatibility, worse
here for reasons not fully understood, possibly related to how the AppImage's bundled runtime
selects its rendering backend vs. the system webkit2gtk used in dev). Reproduced and fixed
directly on this machine (the user asked me to test rather than walk them through it, since it's
the same machine): `WEBKIT_DISABLE_DMABUF_RENDERER=1 ./PromptRig_0.1.0_amd64.AppImage` launches
cleanly (backgrounded it, confirmed the process stays alive with no error output, unlike the
instant abort without the variable). Documented in both `README.md`'s Linux install section and
a new "Troubleshooting" section in `docs/release.md` — explicitly noting it applies to
`.deb`/`.rpm` installs too (same binary, same rendering issue), not just the AppImage.
Committed and pushed (`95ba0f3`) — real bug affecting live testers, pushed immediately rather
than batched.

**Second real bug, from a developer friend's macOS test**: downloaded the `.dmg` via Chrome,
got `"PromptRig" is damaged and can't be opened. You should move it to the Bin.` (screenshot
provided) instead of the "unidentified developer" prompt the README originally described.
Root cause: the app has **no code signature at all** (confirmed decision, see Step 10) — macOS's
right-click → Open bypass only works for an app with *some* trusted signature (even ad-hoc); a
fully unsigned quarantined app instead gets this harsher "damaged" message. Verified the correct
fix via web search before telling the user anything (`xattr -cr /Applications/PromptRig.app`,
strips the browser download's quarantine attribute) rather than guessing. **The README's
original macOS install instructions were wrong** — written before any real unsigned-app testing
happened — corrected in both `README.md` and `docs/release.md`.

Remaining before the user publishes: whatever else surfaces from their own/friends' testing
(two real bugs found so far — NVIDIA/Wayland Linux crash, unsigned macOS Gatekeeper message),
and their planned icon touch-ups (not blocking).

A Mac Silicon friend confirmed the app works well and looks good after the Gatekeeper fix, and a
brainstorm with them produced 4 more ideas, logged in TODO.md's Deferred ideas (none started
except the first, below): a searchable/filterable model picker, remembering window size across
restarts, a slightly larger top-nav font, and pinning a Compare column so "Run all" skips it.
Decided (discussed directly with the user) to keep tracking ideas in TODO.md rather than switch
to GitHub Issues, at least until external testers want to file things themselves.

**Searchable model picker + per-model pricing, implemented 2026-09-08** (committed & pushed) —
see TODO.md's "Searchable/filterable model picker with per-model pricing" entry for the full
design (new `ModelCombobox.vue`, new `ModelInfo.pricing` field, `pricing::resolve_rate` shared
resolver). Went through one live-feedback iteration: the rate first sat beside the model name
and visibly stole its width (screenshot showed long OpenRouter names truncated harder than
before pricing existed) — fixed by stacking the rate below the name instead (name now owns the
full row width and wraps rather than truncating) and dropping the "per 1M" suffix. User's
verdict on the result: "Magnifique!"

Also in this stretch: the user hand-edited `App.vue`'s top-nav font size themselves (as
predicted) — nav links `text-[15px]`, wordmark `text-[16px]` — after confirming with them that
Tailwind has no named step between `text-sm`/14px and `text-base`/16px, only the arbitrary-value
syntax. That TODO item is now resolved.

The user posted a short announcement for the v0.1.0 release in their developer community's
Discord (drafted, reviewed, approved as-is). Immediately after, wanting to bundle the
searchable-model-picker win into a quick follow-up release, they asked whether anything else was
worth adding — small or big. Recommended keeping this release tight (nothing blocking), but
flagged the already-logged "remember window size" idea as a good small addition (official Tauri
plugin exists, likely quick) — user agreed to include it.

**Window size/position persistence, implemented 2026-09-08** (see TODO.md's "Remember window
size across restarts" entry for the full detail): registered `tauri-plugin-window-state` v2.4.1
in `lib.rs`, one line, no custom flags. Verified via source inspection of the plugin itself
(confirmed it hooks window lifecycle events directly, no capability/permission needed since we
don't use its JS-invokable commands) rather than assuming the README was accurate as-is.
`cargo check`/`clippy --all-targets -- -D warnings`/`fmt --check`/`test` all clean. **Live
verification was inconclusive**, honestly reported as such: this dev machine's tiling WM
(Hyprland/Omarchy) doesn't respect requested window geometry, and an attempt to force a
graceful window close via `hyprctl` (to at least confirm a state file gets written) hit a
non-standard Lua-based dispatch syntax on this system — abandoned rather than sunk-cost further
into unrelated WM tooling. Real confirmation needs the user or a friend testing on a
traditional WM/OS. Committed & pushed (`3482fd0`). Proposed tagging `v0.2.0` next (feature
additions, not just fixes) — not yet done, the user wanted to keep going first.

**Pin a Compare column, implemented 2026-09-08** (committed & pushed) — see TODO.md's "Pin a
Compare column so Run all skips it" entry for the full design. Bundled with a persistence
extension (pinned columns survive an app restart) the user asked about in the same breath,
explicitly deferring to KISS if it turned out complex — assessed as low-complexity (same
localStorage watch-and-derive pattern already used 3 times in this app) and built both together.
No backend changes needed at all — `run_experiment` already took an explicit column list, so
"pinned" just means "not included in this request," nothing more.

User specifically asked afterward whether removing a pinned column correctly cleans it out of
the saved state too — traced the actual watcher logic rather than assuming, and confirmed yes:
`savePinnedColumns` re-derives its whole output from current `columns` state on every change
(deep watch), so a removed column disappears from the save on the very next tick with no
special-cased removal-handling code needed. Asked the user whether this default (no extra
confirmation before deleting a pinned card) was fine or needed a safety prompt — they confirmed
the current behavior (KISS) is what they want.

**`v0.2.0` tagged and released, 2026-09-08** — version bumped in all three files (verified
matching locally before pushing), `ci.yml` green on the bump commit, tag pushed, `release.yml`
ran successfully a second time (all 4 platform jobs green again, same 9 installer assets as
`v0.1.0`). Session ended there per the user's request ("on s'arrête là pour aujourd'hui").

**Follow-up same day, outside the repo**: set up local desktop integration on the user's own
Omarchy machine for launching the AppImage (a `promptrig` PATH command that always runs
whichever `PromptRig_*.AppImage` in `~/Downloads` is newest by mtime, an icon installed into
`~/.local/share/icons/hicolor/`, and a `.desktop` launcher entry) — tested working end to end,
including that it correctly picked up `v0.2.0` right after it was downloaded. Generalized (no
hardcoded username) and documented as an optional step in `docs/release.md`'s Linux section for
other Arch/Omarchy users in the same situation, committed & pushed (`4b31cd8`).

**2026-09-10 — short session, research note only, nothing implemented**: the user surfaced
[models.dev](https://models.dev/) (MIT-licensed, community-maintained, powers OpenCode's model
picker) as a possible better pricing/model-metadata source than the current OpenRouter-based
approach. Confirmed its real shape via its GitHub repo before logging anything (plain JSON
endpoints, no API key: `api.json`/`models.json`/`catalog.json`; per-model cost broken out by
input/output/reasoning/cached/audio, context limits, capability flags) rather than trusting the
landing page's marketing copy alone. Logged as a detailed deferred idea in TODO.md (see "Use
models.dev as a pricing/model-metadata source instead of (or alongside) OpenRouter") with the
concrete reason it could matter: it might resolve model ids directly per-provider instead of the
fuzzy vendor/model-id matching `pricing::openrouter_fallback` needs against OpenRouter's own
proxy-shaped catalog — but this hasn't been verified against a real payload yet, only against
the repo's README description, so that's explicitly flagged as the first thing to check before
building anything. The user was explicit they don't have time to implement this now — this is
purely a "remember this for later" entry, not a decision to build it, so **do not start on it
without the user explicitly asking**. Also fixed a small stale doc reference found in passing:
`TODO.md`'s intro line still said `docs/architecture.md` was "(once written)", from before
Step 11 wrote it.

**2026-09-11 — models.dev pricing replacement, implemented, not yet committed**: the user asked
to actually build the models.dev idea logged above. Before writing any code, re-verified the real
payload directly (`curl`+Python on the live 4.5MB `api.json`, not just the README): confirmed
each provider's models are keyed by their own **native** model id (no fuzzy matching needed at
all — even Mistral's `-latest` aliases like `ministral-3b-latest` are present as real entries,
which the old `pricing::openrouter_fallback` design could never resolve directly), confirmed
>90% coverage across OpenAI/Anthropic/Gemini/Mistral/OpenRouter, and cross-checked one known price
(`gpt-4o-mini` = $0.15 / $0.60 per million) against the old hand-curated `pricing.json` before
deleting it. Got two explicit decisions from the user via `AskUserQuestion`: no approximate
fallback when a model is missing from models.dev ("pas de filet"), and delete `pricing.json`
outright rather than keep it as a user override.

The user then asked what other per-model fields models.dev exposes (multimodal support,
reasoning-effort levels, etc.), and after hearing the list, gave an explicit architectural
instruction that shaped the implementation: don't build UI features for that data now, but *do*
make sure the parsing layer captures it now, since multimodal-aware UI and a reasoning-effort
selector are planned follow-ups and shouldn't require redoing the fetch/parse layer later; and
separately, do implement deprecated-model filtering now (hide `"deprecated"` models from the
picker, keep `"beta"` ones) since that's simple and immediately useful.

Implemented: deleted `pricing/openrouter_fallback.rs` and `pricing/pricing.json` outright; wrote
`pricing/models_dev.rs` (`ModelsDevCache`, fetched once per session and memoized — same
fetch-once-remember-failure `LoadState` pattern the old cache used) with a deliberately
over-parsed `ModelEntry` (name, status, attachment, reasoning, reasoning_options, tool_call,
structured_output, modalities, limits, cost, ...) marked `#[allow(dead_code)]` with a doc comment
explaining only price + status are consumed today; `pricing/mod.rs` collapsed down to just
`ModelPrice` + a re-export. Updated every call site (`commands/mod.rs`, `commands/runs.rs`,
`commands/experiments.rs`, `commands/providers.rs`, `lib.rs`) to use the single `ModelsDevCache`
instead of the old `PricingTable` + `OpenRouterPricingCache` pair; `commands::providers::
list_models` now does deprecated-filtering and price-enrichment in the same loop, one lookup per
model. `Run.cost_is_estimate` / `ModelPricing.is_estimate` fields were kept (not removed) but are
always `false` now — avoids a SQLite migration and a multi-file frontend UI rewrite ("≈" badge/
tooltip) that wasn't requested; doc comments explain why they're dormant rather than deleted.
Updated stale doc comments in `domain/model.rs` and `domain/run.rs`, rewrote `docs/
architecture.md`'s "Cost estimation" section and `docs/adding-a-provider.md`'s pricing section
to match. Verified clean: `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt
--check`, `cargo test` (37 passed, 1 ignored — down from 44, net simpler), `npm run build`; a live
`tauri dev` restart confirmed `bindings.ts` is unchanged in shape (frontend needed zero changes).
**Not yet committed** — waiting on the user's review before commit/push, per the project's usual
one-step-at-a-time workflow.

**Regression found and fixed same day**: the user reported prices had vanished entirely from the
model picker. Root cause: `ModelEntry.reasoning_options`' `values` field was typed as a required
`Vec<String>`, but the real `api.json` has `reasoning_options` entries with no `values` at all
(`"toggle"`/`"budget_tokens"` types) and, in at least one case, a `null` inside the `values`
array itself. Since the fetch deserializes the *whole* payload (all 213 providers) in one shot,
one bad entry anywhere failed the entire fetch silently — zero prices for every provider, not
just the affected model. Found by writing a temporary test against the real live endpoint (the
committed unit tests only used a small hand-written sample, which couldn't have caught this).
Fixed by widening the field to `Option<Vec<Option<String>>>`. Re-verified against the real
endpoint (`gpt-4o-mini`, `claude-opus-4-5` both price correctly, all 213 providers parse) and
re-ran the full check suite clean before removing the debug tests.

**Follow-up same day: `ETag`-conditional fetch + on-disk cache.** The user spotted that
models.dev's response carries an `ETag` and asked to use it to skip re-downloading the ~4.5 MB
payload every launch when unchanged — this also surfaced that they'd assumed (incorrectly) that
the data already went through a database step; corrected that first (it was, and still is,
in-memory-only per session). Two explicit decisions via `AskUserQuestion`: persist a small disk
cache (plain files next to the SQLite database, not inside it — not relational data) and fall
back to that disk cache, rather than showing no price, whenever the request fails outright
(offline, non-2xx) and not just on a `304`. Implemented: `models_dev_cache.json` (raw body) +
`models_dev_cache.etag` (the `ETag` value) in the app data dir; `fetch()` sends `If-None-Match`,
reuses the disk body on `304`, falls back to it on any failure, and only overwrites it after a
successful parse of a fresh `200`. `ModelsDevCache::new` now takes the cache directory
(`lib.rs` passes the same `app_data_dir` used for the SQLite file). Verified end-to-end with a
temporary test against the real endpoint (first fetch wrote both files and returned a price; a
second cache instance pointed at the same directory got a real `304` and reused the disk copy)
before removing it; 2 new unit tests cover the disk read/write round trip in isolation. 39 tests
passing (up from 37), `cargo check`/`clippy -D warnings`/`fmt --check` clean.

**Noted, not started: possible UI stall on a slow/no connection.** The user ran out of session
time right after the ETag work and asked to just log this for next time rather than implement it.
Their ask: a Settings toggle to disable the models.dev download outright, for someone on a very
slow connection (the ~4.5 MB first-fetch could otherwise stall the model list/Run path). Worth
checking first: `reqwest::Client::new()` in `ModelsDevCache::new` sets no timeout at all today,
so a stalled connection could hang indefinitely rather than failing into the disk-cache fallback
— a real bug on its own, independent of whatever else gets decided. See TODO.md's "Slow/no
connection could stall the UI on the models.dev fetch" entry for the three options sketched out
(timeout fix / opt-out toggle / non-blocking fetch) — nothing decided yet, pick this up by asking
the user which they want before writing code.

**`CHANGELOG.md` added and `v0.3.0` cut, 2026-09-11.** Created `CHANGELOG.md` (Keep a Changelog
format), backfilling `0.1.0`/`0.2.0` from git history and this file, plus a `0.3.0` entry for the
models.dev pricing replacement + disk cache above. `docs/release.md`'s "Cutting a release" step 1
now also says to add the CHANGELOG entry in the same commit as the version bump, so this becomes
a standing habit rather than a one-off. Also fixed two now-false lines in `README.md` found in
passing: the "Status: pre-release" callout and "No release has been tagged yet" under
Installation — both dated back to before `v0.1.0` ever shipped. Version bumped to `0.3.0` in
`package.json`/`src-tauri/Cargo.toml`/`src-tauri/tauri.conf.json` (+ `Cargo.lock`).

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

**Step 9 (in progress) — whole-window scroll fix** (committed & pushed):
- The bug: reducing the window's height clipped/hid content (e.g. the Temperature/Top P/Max
  tokens row in Playground) with no way to reach it — "j'ai du contenu qui devient invisible."
- **First attempt (per-panel internal scroll) had two real problems**, both found via the user's
  live testing and a screenshot:
  1. A `tailwind-merge` conflict-detection gap: `Card`'s own default class already bakes in
     `overflow-hidden`; overriding it with `overflow-y-auto` doesn't actually override anything
     reliably, because `twMerge` treats `overflow-hidden` (the `overflow` shorthand group) and
     `overflow-y-auto` (the `overflow-y` longhand group) as unrelated, non-conflicting classes —
     both survive in the merged class list, and which one visually wins is decided by whichever
     happens to come later in Tailwind's generated CSS, an implementation detail that isn't
     stable and isn't the same between a full `vite build` and Vite's dev-server HMR CSS
     injection. Confirmed the bug and the fix directly with a `tailwind-merge` Node one-liner:
     `twMerge('overflow-hidden', 'overflow-y-auto')` → keeps both; `twMerge('overflow-hidden',
     'overflow-auto')` → correctly collapses to one. Switched the panel's override to plain
     `overflow-auto` to fix this specific defect.
  2. Even with that fixed, the user's actual screenshot showed it working exactly as coded — a
     visible themed scrollbar sliver on the *prompt panel* — but that's not what they wanted at
     all: **one single scrollbar for the whole window**, not a separate nested scroll region per
     panel. Their own words: "ce n'est pas au niveau du panneau de gauche que je veux une barre
     de scroll, c'est au niveau de la fenêtre... mets une barre de scroll au niveau de la
     fenêtre, sous le header, sous la barre de menu."
- **Final design, reworked accordingly**: `App.vue`'s `<main>` (already the element directly
  under the top nav) is the *only* scroll container app-wide (`overflow-y-auto`, horizontal
  explicitly locked via `overflow-x-hidden` so nothing doubles up with Compare's own intentional
  horizontal column-browsing scroll). `PlaygroundView`/`CompareView` root divs changed from
  `h-full` (hard-clamped to exactly the viewport, the real root cause — content taller than that
  had nowhere to go but get clipped) to `min-h-full` (a floor, not a ceiling — still fills the
  full height when content is short, but is free to grow taller when it isn't, which is what
  lets the extra height become visible overflow that `main` can then scroll to). Removed every
  intermediate `overflow-hidden`/`overflow-auto` boundary that had been acting as its own nested
  scroll region (the Playground two-column grid, the prompt `Card`, `ResultPanel`'s `Card` and
  `CardContent`, Compare's per-column `Card`) — where `Card`'s own default `overflow-hidden`
  would otherwise still clip, each is now explicitly `overflow-visible` instead (confirmed this
  merges cleanly with `twMerge` too, same shorthand-vs-shorthand group). Compare's horizontal
  per-row scroll for browsing many columns side by side (`overflow-x-auto`) is a separate,
  intentional feature and was left untouched — only the vertical clipping was ever the bug.
- `npm run build` clean throughout both attempts. Verified live via full `tauri dev` restarts
  (not HMR) at each step, given the dev-server-vs-build CSS-ordering discrepancy found along the
  way; user confirmed the final version works ("c'est parfait") before requesting the commit.

**Step 9 (closed) — frameless window** (committed & pushed):
- User's idea, described as "headless" (they were thinking of Electron's terminology) — remove
  the native OS title bar (app name + close/minimize/maximize) for a cleaner look. Confirmed
  before implementing (per the user's explicit request not to act until validated) that this is
  Tauri's `decorations: false` window option, and clarified scope via two questions: **close
  button only** (no minimize/maximize — the user doesn't need them), and **design with all 3
  OSes in mind** even though only Linux can actually be tested right now.
- `src-tauri/tauri.conf.json`: `"decorations": false` added to the main window config.
- `src-tauri/capabilities/default.json`: added `"core:window:allow-close"` (confirmed as a real
  permission identifier by grepping the generated `gen/schemas/desktop-schema.json` — `core:default`
  alone doesn't cover window-close under Tauri v2's capability/ACL system).
- `App.vue`: the top nav now doubles as the title bar. `data-tauri-drag-region` on the `<nav>`
  makes its empty space draggable (interactive children — the router links, theme toggle, close
  button — are separate elements and keep working normally, since Tauri's drag-region handling
  only intercepts the exact tagged element, not its children). New close button (`@lucide/vue`'s
  `X` icon, `getCurrentWindow().close()` from `@tauri-apps/api/window`, red-tinted hover) added
  next to the existing theme toggle — the single window control, deliberately no
  minimize/maximize per the user's choice.
- `cargo check` and `npm run build` clean; full `tauri dev` restart required (window-decoration
  and capability changes are Rust/config-side, not picked up by frontend HMR).
- **User-confirmed live**: the close button works correctly. Window-dragging couldn't be
  confirmed on the user's setup — they run Omarchy (an Arch-based Hyprland distro, a tiling
  window manager), where free-floating drag doesn't apply the same way it would under a
  traditional floating WM/DE — not a bug, just not meaningfully testable on that setup. User's
  verdict on the visual result: "beaucoup plus beau."
- **Step 9 closed by the user's explicit sign-off**: "pour moi c'est correct... on peut
  considérer le point 9 pour l'instant fait." Also caught and fixed a real TODO.md bookkeeping
  gap during this closing conversation: "Revisit contrast" had never actually been done (only
  dark becoming the default was done — the light theme's own colors were never retouched since
  Step 6); asked the user how to treat it rather than assuming, and they chose to leave it open
  for a future pass rather than mark it done.

**Step 10 (in progress) — CI/CD** (committed & pushed):
- Before writing anything, walked the user through how Tauri's bundler actually behaves, since
  they explicitly asked to understand the process rather than just receive files: yes, `tauri
  build`/`tauri-action` produce real native installers (`.deb`/`.rpm`/`.AppImage` on Linux,
  NSIS `.exe` on Windows, `.dmg` on macOS), not raw binaries end users have to figure out
  themselves — with the caveat that unsigned installers show an OS security warning on first
  launch. This surfaced three decisions to settle before implementing (each confirmed with the
  user rather than assumed):
  1. **No code signing yet** — needs paid certificates (Apple Developer, Windows), deferred
     until there's real user demand to justify it.
  2. **No native Arch/AUR package** — the user runs Omarchy (Arch-based) and can't use the
     `.deb`/`.rpm`; Tauri's bundler has no `pacman` target at all. Agreed the `.AppImage`
     (works on any distro, no install) is an acceptable interim solution, documented as such;
     a real AUR package logged as a deferred idea (TODO.md).
  3. **CI frontend checks limited to what already exists** — discovered while planning that the
     original architecture plan's "eslint, vue-tsc, frontend tests" assumed tooling (ESLint,
     Vitest) that was never actually set up in this project (no config, no test files). Asked
     the user rather than silently either skipping it or bolting on new tooling/tests just to
     fill out a CI step; they chose to keep CI to what's real (`vue-tsc` + build) and treat
     ESLint/Vitest as a separate future addition if it's ever actually needed.
- `.github/workflows/ci.yml`: 3 jobs on every push/PR to `master` — `frontend` (`npm run build`),
  `rust` (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, with
  the WebKitGTK system packages Tauri needs even to compile), `version-consistency` (fails if
  `package.json`/`Cargo.toml`/`tauri.conf.json`'s version strings don't all match). Every check
  it runs was verified locally first: `cargo fmt --check` clean, `cargo clippy --all-targets -- -D
  warnings` clean (0 warnings — stricter than the plain `cargo clippy --all-targets` used so far
  in this project, and it still passed outright), `cargo test` 36 passed/1 ignored, and the
  version-check shell logic run directly against the repo's current (matching) versions.
- `.github/workflows/release.yml`: triggered by pushing a `v*.*.*` tag. `tauri-apps/tauri-action@v1`
  matrix — macOS (`aarch64-apple-darwin` + `x86_64-apple-darwin` as separate matrix entries),
  `ubuntu-22.04` (deliberately not `-latest`: an older LTS keeps the built Linux binaries'
  glibc-version requirement low enough to run on more users' systems), `windows-latest`. Verified
  the exact current recommended `tauri-action` syntax via the official Tauri v2 docs
  (`v2.tauri.app/distribute/pipelines/github/`) rather than relying on possibly-stale training
  knowledge, given this is exactly the kind of fast-moving tooling detail the project's own
  working method calls for double-checking. Publishes a **draft** GitHub Release (human reviews
  and publishes manually — nothing goes live automatically). No signing secrets configured
  (matches the "no code signing yet" decision above). YAML syntax verified with a Python
  `yaml.safe_load` check; **the workflow itself has never actually run** — that needs a real
  tag push, and there's no version worth releasing yet.
- `docs/release.md` — written now (pulled forward from Step 11, since this step needed the
  process defined anyway): the 3-file version-bump-then-tag procedure, what CI/release do
  automatically, per-OS install instructions for end users (explicitly including the AppImage
  workaround for Arch-based distros), and a "Not yet done" section calling out code signing,
  the Arch/AUR package, and auto-update as deliberate omissions.
- TODO.md's "App versioning strategy" deferred idea marked resolved (pointing at this step and
  `docs/release.md`); a new "Native Arch Linux / AUR package" deferred idea added.
- **Pushing `.github/workflows/*.yml` was rejected once** ("refusing to allow an OAuth App to
  create or update workflow... without `workflow` scope") — the `gh` CLI's stored OAuth token
  lacked the `workflow` scope. Fixed by the user running `gh auth refresh -h github.com -s
  workflow` themselves (needs a browser-based authorization, not something to do on their
  behalf); the push then succeeded.
- **`ci.yml` verified for real on GitHub, not just locally**: watched the actual run
  (`gh run watch`) after pushing — all 3 jobs green (Frontend 24s, Version consistency 3s, Rust
  fmt/clippy/test ~10min, expected for a first cold-cache compile on a fresh runner). Only a
  minor deprecation annotation (Node 20 runtime warning on `actions/checkout@v4`/`setup-node@v4`)
  — not a failure, worth bumping to `@v5` at some point but not urgent.

**Cross-provider cost estimates via OpenRouter pricing** (implemented ahead of Step 11, at the
user's explicit request, 2026-09-08; committed & pushed):
- The user's own words for why this couldn't wait for a later polish pass: "avant de faire une
  release, pour moi, il y a un truc que j'aimerais que l'on fasse pour le MVP... régler cette
  histoire de tarifs parce que c'est quelque chose d'important d'avoir le prix de la requête."
  Confirmed the approach with them first (reaffirming the Step-8-era deferred idea): use
  OpenRouter's own published pricing as an approximate stand-in for providers that don't expose
  pricing themselves (Anthropic, Gemini, Mistral), explicitly disclosed as a ceiling estimate
  via a tooltip, never presented as exact.
- **The real design problem was matching model ids across vendors**, not fetching the data
  (OpenRouter's public `/api/v1/models` — verified live via `curl`, no API key needed — already
  returns per-model `pricing.prompt`/`pricing.completion` in USD/token for ~430 models). Verified
  directly against the real endpoint that OpenRouter's ids are `"<vendor-slug>/<model>"` (e.g.
  `anthropic/claude-opus-4.5`), while a provider's own native id is often shaped differently
  (checked Anthropic's real model-listing code: ids like `claude-opus-4-5-<date>`, dashes and a
  release-date suffix OpenRouter's own listing doesn't carry) — a naive "strip the vendor
  prefix and compare" would rarely match. Solved with `normalize_model_id` (lowercase, strip all
  non-alphanumeric characters, drop a trailing 8-digit date suffix) applied to both sides before
  comparing. Deliberately conservative: a miss just falls back to "no price" (today's behavior)
  rather than ever risking a wrong-model match.
- New module `src-tauri/src/pricing/openrouter_fallback.rs` — `OpenRouterPricingCache`, fetched
  and cached once per app session (same "session-lifetime, refresh on restart" pattern as the
  model-list cache from Step 8), guarded by a `tokio::sync::Mutex` (needed since loading it does
  a network fetch while the lock is held — a plain `std::sync::Mutex` can't be held across an
  `.await`). Builds two lookups from one fetch: OpenRouter's own model id → its real price (used
  when a Run itself targets OpenRouter — exact, not an estimate) and `(target ProviderId,
  normalized native id)` → OpenRouter's price for the closest-looking listing (used for every
  other provider — always an estimate). A failed fetch (offline, etc.) is remembered for the
  rest of the session rather than retried on every single Run.
- `pricing::ModelPrice` (previously private to `pricing::mod`) became `pub(crate)` with its cost
  formula extracted onto the type itself (`ModelPrice::cost(&self, usage)`), shared by both the
  static `pricing.json`-backed table and the new dynamic cache instead of duplicating the
  input/output-tokens-times-price arithmetic.
- **New domain field**: `Run.cost_is_estimate: bool` (migration `0002_add_cost_is_estimate.sql`,
  `runs.cost_is_estimate INTEGER NOT NULL DEFAULT 0`) — `false` for an exact price (our own
  `pricing.json`, or OpenRouter's own real price for its own Runs), `true` only when the number
  came from the cross-provider fallback. `commands::build_new_run` (shared by `run_generation`
  and `run_experiment`/`run_column`) became `async fn` to await the new cache's lookups, tried
  in order: static table → (if provider is OpenRouter) OpenRouter's own exact price → fallback
  estimate. `OpenRouterPricingCache` added as new Tauri-managed state in `lib.rs`, threaded
  through `commands/runs.rs` and `commands/experiments.rs` alongside the existing `PricingTable`.
- `ResultPanel.vue`: the cost `Badge` shows a `≈` prefix and wraps in a `Tooltip` ("Approximate —
  no official pricing for this model, so this uses OpenRouter's price for the closest match.
  Usually a ceiling... not the exact provider cost") only when `run.cost_is_estimate` is true —
  otherwise unchanged. `src/lib/bindings.ts` regenerated automatically via a `tauri dev` restart,
  picked up `cost_is_estimate: boolean` on `Run` with no manual intervention needed.
- 5 new Rust unit tests (`pricing::openrouter_fallback`): normalization (including the
  date-suffix-stripping edge case, and that a short non-date numeric suffix like `gpt-4o`'s "4o"
  isn't wrongly stripped), building both lookups from a sample API response, the exact-vs-
  estimate distinction, and the "unknown model returns `None`, never a wrong price" guarantee.
  41 tests total (was 36), all passing; `cargo fmt --check`/`clippy --all-targets -- -D
  warnings` both clean. `npm run build` clean; verified live via `tauri dev` — user tested
  Anthropic/Gemini/Mistral and confirmed the estimate appears correctly ("Ça fonctionne,
  bravo!").

**Follow-up fix — Mistral's `-latest` alias models showed no estimate** (same day, committed &
pushed):
- The user asked specifically to *diagnose* first ("je voudrais être sûr que c'est parce qu'on
  n'a pas le prix dans la liste OpenRouter, ou c'est le parsing qui n'est pas au point") rather
  than assuming a fix was wanted — checked before touching any code. Fetched OpenRouter's real
  catalog again (`curl`, filtered to `mistralai/*`) and found the true cause: OpenRouter only
  ever lists *dated* Mistral snapshots (`ministral-3b-2512`, `ministral-3b-2407`, `mistral-
  large-2407`, `mistral-large-2512`, ...), never a bare `-latest` entry — while Mistral's own API
  returns rolling aliases like `ministral-3b-latest` (confirmed against this project's own
  `providers/mistral.rs` test fixtures, which already use `"mistral-small-latest"`). So this
  wasn't a missing-data problem or a parsing bug, but a genuine gap in the matching heuristic —
  affecting Mistral's entire lineup, not just the one model reported. Asked the user whether to
  fix now or log it (they'd said "pas critique"); they chose to fix it immediately.
- Deliberately did **not** extend the existing date-stripping normalization to also swallow
  short version-like suffixes (e.g. treating `mistral-large-2407` and `mistral-large-2512` as
  "the same base") — OpenRouter's own data shows those two snapshots have meaningfully different
  prices (4x apart), so collapsing them would risk silently picking a stale price. Instead added
  a separate, narrower path used *only* when the query id itself ends in the literal `-latest`
  alias: `strip_latest_alias` detects that suffix, and a new `latest_by_provider` lookup
  (populated by `trailing_numeric_suffix`, which extracts and compares the version number of
  each dated candidate sharing a base name — works uniformly for both Mistral's short `YYMM`-
  style tags and full `YYYYMMDD` dates, since both compare correctly as plain integers) resolves
  to whichever dated snapshot looks newest. A bare, unversioned listing (e.g. plain
  `mistral-large` with no suffix at all) is simply skipped for this path — there's no way to
  rank its recency against dated siblings, so it's excluded rather than guessed at.
- 3 new unit tests (44 total, up from 41): `-latest` suffix stripping, version-number extraction
  (including that an unversioned id correctly yields `None`), and an end-to-end case proving the
  resolved price is the *newer* of two dated candidates, not just whichever was inserted first.
  `cargo fmt --check`/`clippy --all-targets -- -D warnings`/`test` all clean; verified live via a
  full `tauri dev` restart.

**Drag-region permission bug, found incidentally while testing prompt-draft persistence**
(committed & pushed):
- While restarting `tauri dev` to prove the prompt draft survives a real restart, the dev
  console showed: `Unknown Error: window.start_dragging not allowed. Permissions associated
  with this command: core:window:allow-start-dragging`. This is the frameless-window drag
  region added in Step 9 (`data-tauri-drag-region` on `App.vue`'s `<nav>`) — at the time, the
  user couldn't verify dragging worked because they run a tiling window manager (Hyprland/
  Omarchy) where free-floating drag doesn't really apply, so the omission went unnoticed. This
  console error proves it was actually **broken on every platform**, not just untestable on
  theirs: `capabilities/default.json` only ever gained `core:window:allow-close` (Step 9), never
  the separate `core:window:allow-start-dragging` permission Tauri's ACL system requires for the
  drag gesture itself. Fixed by adding it (confirmed as a real permission id via the generated
  `gen/schemas/desktop-schema.json`, same way `allow-close` was verified in Step 9). `cargo
  check` clean; verified live via `tauri dev` restart — the same console error no longer appears.

**Prompt draft persistence across restarts** (user idea from 2026-09-08's earlier session,
implemented later the same day; committed & pushed):
- `stores/promptDraft.ts`: `systemPrompt`/`userPrompt` now read from and write-through to
  `localStorage` (`promptrig.promptDraft`) — same per-device pattern as the existing "last used
  provider/model" memory in `PlaygroundView.vue`. Deliberately scoped to just these two fields,
  not the generation params, per the user's explicit instruction: "je ne parle pas de pouvoir
  sauvegarder des prompts et de les gérer, mais juste garder le dernier prompt système et le
  dernier user prompt." `npm run build` clean; **user-verified with a real app restart** (typed
  content, closed the app, reopened it, confirmed it came back) — the strongest possible test
  for a "survives a restart" feature.

**Real app icon** (committed locally, deliberately **not pushed** yet — see below):
- The default Tauri scaffold icon (generic Tauri logo) had never been replaced since Step 0 —
  the user flagged this as needed before any real release. Gave them an image-generation prompt
  matching the app's established dark Nord identity (Polar Night `#2E3440` background, Frost
  `#88C0D0` accent): a bold terminal `>` chevron + cursor block, flat/geometric, no text, full
  bleed, 1024×1024.
- User's result came back as a JPEG with rounded corners baked in, leaving pure white in the 4
  cut-off corner triangles. Before touching it, checked (via a NumPy pixel scan) that near-white
  pixels existed *only* in those 4 corner regions and nowhere in the actual artwork (chevron,
  cursor, background all far from white) — confirming a global white-distance threshold couldn't
  accidentally eat into the design. Converted JPEG→RGBA PNG with alpha computed from distance-
  from-white (clamped/scaled, not a hard cutoff, so the rounded-corner edge stays anti-aliased
  instead of jagged). Verified by compositing the result over a solid red background and
  confirming red showed through cleanly in the corners only.
- `npx tauri icon app-icon.png` (Tauri's own icon-generation command) regenerated every bundled
  format (`.icns`, `.ico`, all the PNG sizes, the Windows Store `Square*Logo.png` set) from that
  one source image. It also generates iOS/Android icon sets by default — deleted those, since
  this is a desktop-only app (no mobile target configured anywhere in `tauri.conf.json`).
  Kept `app-icon.png` at the repo root as the regeneratable source.
- `cargo check` clean; verified live via a `tauri dev` restart (icon file changes aren't picked
  up by cargo's incremental build without a real restart). User spotted a few small corner
  artifacts in the result and plans to touch them up by hand in Photoshop later — explicitly
  not blocking, not something to iterate on further right now.
- **User explicitly asked to commit but not push this one**: "je pense qu'il n'y a pas de
  vérification importante à faire et ça nous fait perdre du temps à chaque fois d'attendre que
  le CI se termine." First time in this project a commit was deliberately left unpushed — worth
  remembering for the next session that `master` may be locally ahead of `origin/master`.

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
