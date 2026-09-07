# PromptRig — Project State

Hand-off document: what's done, what's in progress, decisions made, and known issues — enough
for a new session to resume cleanly. **Update this file in every commit that changes project
state.** See [TODO.md](./TODO.md) for the detailed task checklist, [AGENTS.md](./AGENTS.md) for
the working agreement (commit/state discipline, destructive-command precautions), and
`/home/toorop/.claude/plans/purrfect-frolicking-donut.md` for the full architecture plan
approved by the user.

## Current step

**Step 0 — Bootstrap**, in progress. See TODO.md for the exact checklist.

## Done so far

- Repo initialized (`git init`), Tauri 2 + Vue 3/TS scaffold generated via `create-tauri-app`
  and renamed from the default `tauri-app` to `promptrig` everywhere (package.json,
  `src-tauri/Cargo.toml`, lib name `promptrig_lib`, `tauri.conf.json` productName/title,
  identifier `com.promptrig.app`).
- Tailwind CSS v4 + shadcn-vue initialized (Reka UI base, Lucide icons, neutral base color,
  CSS-variable theme with light/dark already defined by shadcn-vue's `.dark` class). Removed
  the Google Fonts CDN import shadcn-vue's init added by default — using a system font stack
  instead since this is an offline-capable desktop app.
- Base shadcn-vue components added: button, input, textarea, select, card, label, separator,
  badge.
- Pinia + Vue Router installed and wired in `src/main.ts`. `@` → `src/*` path alias configured
  in both `vite.config.ts` (resolve.alias) and `tsconfig.json` (paths).
- Minimal app shell: `src/App.vue` has a top nav (Playground / Compare / Settings) over a
  `RouterView`; three placeholder views exist at `src/views/{Playground,Compare,Settings}View.vue`.
- Removed the scaffold's placeholder `greet` Tauri command from `src-tauri/src/lib.rs` (unused
  once the nav shell replaced the template's demo UI).
- Verified `npm run build` (vue-tsc + vite build) and `cargo check` both succeed.

- `README.md` (real content), `LICENSE` (MIT), `SECURITY.md`, `CODE_OF_CONDUCT.md`, and
  `.github/ISSUE_TEMPLATE/` + `PULL_REQUEST_TEMPLATE.md` all written.
- First commit made (`docs: add original project specification`), restoring `docs/start.md`.

## In progress / not yet done

- `npm run tauri dev` has not been smoke-tested yet. A background run attempt was interrupted
  by the user (mid-way through the process-issue correction below) — re-attempting it should
  wait for an explicit go-ahead since it pops a real window on the user's live desktop session.
- The scaffold + all Step 0 doc/meta files above are written but not yet committed (next action).

## Known issues / incidents

- **Data-loss incident (recovered):** running `create-tauri-app ... --force` in the non-empty
  project directory silently deleted `docs/start.md` (the user's original spec), even though
  that path was unrelated to the Tauri template. Restored verbatim from conversation history
  (verified: 615 lines, matching the original). Lesson recorded in assistant memory — never
  force-scaffold into a non-empty directory without protecting existing files first.

## Key decisions (see the plan file for full rationale)

- SQLite access confined to Rust via `rusqlite` (not `tauri-plugin-sql`).
- Provider abstraction: `trait LlmProvider` (`async-trait`) + `ProviderRegistry` keyed by a
  `ProviderId` enum.
- Rust↔TS type sync via `specta` + `tauri-specta` generated bindings.
- `Experiment` containing N `Run`s from day one; a solo Playground run is a 1-run Experiment.
- Cost = `f64` USD estimate from an externalized `pricing.json`.
- Streaming deferred (ship `generate()` first; `generate_stream()` + Tauri events later).
- License: MIT. Repo meta: SECURITY.md, CODE_OF_CONDUCT.md, and issue/PR templates wanted from
  the start (public repo).
- All docs and code comments are written in English (conversation with the user is in French).

## Next action

Smoke-test `npm run tauri dev` (user confirmed it's fine to open a window on their desktop),
then move to Step 1 (Rust domain & errors).

## Process note

The user asked for an `AGENTS.md` codifying the commit/STATE.md discipline directly in the
repo (not just in assistant memory), after this rule had to be repeated. It's in place at
[AGENTS.md](./AGENTS.md) — follow it.
