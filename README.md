
# PromptRig

A cross-platform desktop app for prompt engineering and side-by-side LLM comparison.

PromptRig is a fast, developer-oriented playground: pick a provider, pick a model, write a
system prompt and a user prompt, run it, and see the result along with latency, token usage,
and estimated cost. It is deliberately **not** an agent builder, an observability platform, or
a workflow engine — just a focused tool for testing and comparing prompts across models.

> **Status: pre-release.** Core functionality (Playground, side-by-side Compare, five
> providers, cost estimation, secrets/storage) is implemented and working end to end. See
> [TODO.md](./TODO.md) for what's left before a first tagged release and
> [STATE.md](./STATE.md) for a detailed, chronological build log.

## Screenshots

### Playground

<img width="1157" height="732" alt="screenshot-2026-09-08_13-33-22" src="https://github.com/user-attachments/assets/e349101b-b81d-47b7-b4fb-f1cfd24af458" />


### Compare

<img width="1422" height="898" alt="screenshot-2026-09-08_13-34-53" src="https://github.com/user-attachments/assets/c3db5cef-c94e-4170-a968-3e89ef37b4bd" />

## Installation

> No release has been tagged yet — the links below will 404 until then (see
> [STATE.md](./STATE.md) for where things stand). Once one exists, this is how to get it.

Grab the installer for your OS from the
**[latest release](https://github.com/toorop/PromptRig/releases/latest)**.

- **Windows**: download the `.exe`, double-click it, and follow the installer. First launch may
  show a SmartScreen warning (the app isn't code-signed yet) — click "More info" → "Run anyway".
- **macOS**: download the `.dmg`, open it, and drag PromptRig into Applications. First launch:
  right-click the app → **Open** to get past the "unidentified developer" warning (only needed
  once — also because it isn't code-signed yet).
- **Linux**:
  - Debian/Ubuntu and derivatives: download the `.deb` and install it (`sudo dpkg -i
    PromptRig_*.deb`, or open it with your package manager's GUI).
  - Fedora/openSUSE and derivatives: download the `.rpm` and install it with your package manager.
  - **Any other distro, including Arch-based ones** (Arch, Manjaro, Omarchy, ...): there's no
    native package yet, but the `.AppImage` works everywhere — no installation, no package
    manager involved:
    ```bash
    chmod +x PromptRig_*.AppImage
    ./PromptRig_*.AppImage
    ```

See [docs/release.md](./docs/release.md) for the full picture, including what's deliberately not
done yet (code signing, a native Arch/AUR package, auto-update).

## Features

- **Playground**: pick a provider + model, write a system/user prompt, tune the generation
  parameters a model actually supports (temperature, top_p, max tokens), run it, and see the
  result with latency, token usage, and estimated cost. System/user prompt content persists
  across app restarts.
- **Compare**: run the same prompt across several Provider + Model combinations at once (`Run
  all`), each in its own column, add/remove columns freely, rerun a single column.
- **Providers**: OpenAI, Anthropic (Claude), Google Gemini, Mistral, and OpenRouter. A generic
  OpenAI-compatible endpoint slot exists in the domain model but has no UI/implementation yet.
  Adding a new provider doesn't require touching the rest of the app — see
  [docs/adding-a-provider.md](./docs/adding-a-provider.md).
- **Cost estimation**: exact pricing where hand-curated, with an approximate cross-provider
  fallback (clearly marked "≈" with a disclosure tooltip) derived from OpenRouter's published
  pricing for providers that don't expose their own.
- API keys are stored in your operating system's native keyring (macOS Keychain, Windows
  Credential Manager, Secret Service on Linux) — never in plain text, never in the frontend.
- Local SQLite storage for every Run and Experiment — all non-secret data stays on your machine.
- Dark-by-default UI (a light theme is available via a toggle), a frameless window, and
  self-hosted typography (no CDN dependency).

## Tech stack

- [Tauri 2](https://tauri.app/) with a Rust backend
- [Vue 3](https://vuejs.org/) + TypeScript + [Vite](https://vitejs.dev/) + [Pinia](https://pinia.vuejs.org/)
- [shadcn-vue](https://www.shadcn-vue.com/) + Tailwind CSS v4
- SQLite (via `rusqlite`, no ORM) for local, non-secret data
- The OS-native keyring (via the `keyring` crate) for API keys
- [`specta`](https://github.com/oscartbeaumont/specta) + [`tauri-specta`](https://github.com/oscartbeaumont/tauri-specta) for generated, always-in-sync Rust↔TypeScript types

## Getting started (development)

See [docs/development.md](./docs/development.md) for full setup instructions per OS (Linux,
Windows, macOS). Short version, once prerequisites (Rust toolchain, Node.js, platform Tauri
dependencies) are installed:

```bash
npm install
npm run tauri dev
```

## Documentation

- [docs/start.md](./docs/start.md) — original product specification
- [docs/architecture.md](./docs/architecture.md) — architecture overview and key decisions
- [docs/development.md](./docs/development.md) — development setup per OS
- [docs/adding-a-provider.md](./docs/adding-a-provider.md) — how to add a new LLM provider
- [docs/release.md](./docs/release.md) — versioning discipline and how to cut a release

## Contributing

Contributions are welcome once the project reaches a more stable shape. Please read
[CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md) before participating, and see
[SECURITY.md](./SECURITY.md) for how to report a vulnerability. If you're an AI coding agent
working on this repo, read [AGENTS.md](./AGENTS.md) first.

## License

[MIT](./LICENSE)
