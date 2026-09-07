# Working agreement for AI coding agents on this repo

These rules apply to any AI assistant (Claude Code or otherwise) working on PromptRig. They
exist because early in the project an agent scaffold ran a destructive `--force` command in a
non-empty directory and silently deleted `docs/start.md` (the original spec) before any commit
existed to protect it, and then kept working for a while without committing anything.

## Commit and state discipline (non-negotiable)

- **Commit regularly**, after each meaningfully-complete unit of work — not once at the end of
  a long session. Don't let uncommitted work pile up.
- **Update [STATE.md](./STATE.md) in the same commit** whenever project state changes: what's
  done, current step, decisions made, known issues/blockers, next action. STATE.md must always
  reflect reality closely enough that a new session (human or agent) can resume from it alone.
- **Keep [TODO.md](./TODO.md) in sync**: check off items as they're completed, in the same
  commit as the work that completed them.

## Before any destructive or overwrite-capable command

Before running a scaffolding/generator/init tool with a force-overwrite flag (or any command
that can silently clobber files) in a directory that already has content:

1. Run `git status` first. If there's uncommitted work, commit or stash it.
2. Prefer scaffolding into a throwaway temp directory and merging in only the files you want,
   over running `--force` directly on top of an existing project directory.
3. After the command runs, diff/verify that pre-existing paths are still intact before building
   on top of the result.

This applies to `git checkout/restore/reset/clean`, `rm -rf`, and any CLI scaffold tool, not
just the Tauri CLI.

## Documentation language

All documentation and code comments are written in **English** (this is a public repo), even
though the maintainer may converse with the assistant in French or another language.

## Where to look for context

- [docs/start.md](./docs/start.md) — original product specification (source of truth for scope
  and intent; do not edit except to fix a transcription error).
- [TODO.md](./TODO.md) — detailed, checkable implementation plan.
- [STATE.md](./STATE.md) — current project state / session hand-off.
- `docs/architecture.md` (once written) — architecture overview and the structuring decisions
  (provider abstraction, storage approach, type-sync strategy, etc.) with their rationale.
