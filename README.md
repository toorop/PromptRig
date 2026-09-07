# PromptRig

A cross-platform desktop app for prompt engineering and side-by-side LLM comparison.

PromptRig is a fast, developer-oriented playground: pick a provider, pick a model, write a
system prompt and a user prompt, run it, and see the result along with latency, token usage,
and estimated cost. It is deliberately **not** an agent builder, an observability platform, or
a workflow engine — just a focused tool for testing and comparing prompts across models.

> **Status: early development.** The project is being built incrementally; see
> [TODO.md](./TODO.md) for the current step and [STATE.md](./STATE.md) for a detailed snapshot
> of what's implemented so far. It is not yet usable end-to-end.

## Features (target for the first usable version)

- Single-model playground: provider + model picker, system/user prompt editors, core
  generation parameters (temperature, max tokens, top_p — only shown when the model supports
  them), run, and inspect the result (text, latency, token usage, estimated cost).
- Side-by-side comparison: run the same prompt across several Provider + Model combinations at
  once (`Run all`), each with its own result column.
- Providers: OpenAI, Anthropic, Google Gemini, Mistral, OpenRouter, and any generic
  OpenAI-compatible endpoint. Adding a new provider does not require touching the rest of the
  app — see [docs/adding-a-provider.md](./docs/adding-a-provider.md) (once written).
- API keys are stored in your operating system's native keyring (macOS Keychain, Windows
  Credential Manager, Secret Service on Linux) — never in plain text, never in the frontend.
- Local SQLite storage for prompts, test cases, runs, and experiments — all non-secret data
  stays on your machine.

## Tech stack

- [Tauri 2](https://tauri.app/) with a Rust backend
- [Vue 3](https://vuejs.org/) + TypeScript + [Vite](https://vitejs.dev/)
- [shadcn-vue](https://www.shadcn-vue.com/) + Tailwind CSS
- SQLite (via `rusqlite`) for local, non-secret data
- The OS-native keyring (via the `keyring` crate) for API keys and other secrets

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
- [docs/architecture.md](./docs/architecture.md) — architecture overview and key decisions (once written)
- [docs/development.md](./docs/development.md) — development setup per OS (once written)
- [docs/adding-a-provider.md](./docs/adding-a-provider.md) — how to add a new LLM provider (once written)
- [docs/release.md](./docs/release.md) — how to cut a release (once written)

## Contributing

Contributions are welcome once the project reaches a more stable shape. Please read
[CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md) before participating, and see
[SECURITY.md](./SECURITY.md) for how to report a vulnerability.

## License

[MIT](./LICENSE)
