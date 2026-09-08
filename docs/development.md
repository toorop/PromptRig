# Development setup

## Prerequisites (all platforms)

- **Rust** (stable toolchain) — [rustup.rs](https://rustup.rs/).
- **Node.js 20+** and npm.
- The platform-specific Tauri prerequisites below.

### Linux

Tauri 2 links against WebKitGTK at build time — install these before running anything:

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev
```

(Package names for other distros — Fedora, Arch, ... — are equivalent; see the
[official Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/) if `apt` isn't
your package manager.)

**Known issue — NVIDIA + Wayland**: on some NVIDIA-proprietary-driver + Wayland setups (this
project's own dev machine included, running Hyprland), `npm run tauri dev` crashes immediately
after opening the window with `Gdk-Message: Error 71 (Protocol error) dispatching to Wayland
display`. This is a WebKitGTK/DMA-BUF renderer issue, not a PromptRig bug. Workaround:

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri dev
```

### Windows

Install the [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
(the "Desktop development with C++" workload) and [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)
(pre-installed on current Windows 10/11).

### macOS

Install Xcode Command Line Tools: `xcode-select --install`.

## Running the app

```bash
npm install
npm run tauri dev
```

This starts Vite (frontend, hot-reloading) and compiles/runs the Rust backend together, opening
a native window. First run will take a while (compiling every Rust dependency); subsequent runs
are fast.

In debug builds, every run also regenerates `src/lib/bindings.ts` (the generated
Rust↔TypeScript types and command wrappers — see
[docs/architecture.md](./architecture.md#rust--typescript-type-sync-specta--tauri-specta)) from
whatever's currently annotated in the Rust source. Don't hand-edit that file.

**If the Rust side doesn't seem to pick up a change** (a new file, a `Cargo.toml` edit): the
file watcher backing `tauri dev` occasionally misses new files. Kill the process and re-run
`npm run tauri dev` rather than assuming the change is broken.

## Verifying changes before committing

Frontend:

```bash
npm run build   # vue-tsc typecheck + vite build
```

Backend (from `src-tauri/`):

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

These are exactly what `.github/workflows/ci.yml` runs on every push/PR — running them locally
first avoids a red CI run for something you could have caught immediately.

One Rust test touches the real OS keyring and is `#[ignore]`d by default (CI doesn't have one
available/unlocked):

```bash
cargo test -- --ignored
```

## Project layout

See [docs/architecture.md](./architecture.md) for the full breakdown of `src-tauri/src/` and
`src/`.

## Releasing

See [docs/release.md](./release.md) — versioning discipline, cutting a tagged release, and
what each installer looks like for end users.
