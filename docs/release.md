# Releasing PromptRig

## Versioning

PromptRig follows [semantic versioning](https://semver.org/) (`MAJOR.MINOR.PATCH`). The version
is currently duplicated in three files with nothing enforcing they match except CI:

- `src-tauri/Cargo.toml` (`[package].version`) — **the source of truth**. Bump this one first.
- `package.json` (`version`)
- `src-tauri/tauri.conf.json` (`version`)

When bumping, update all three to the same value in one commit. CI's `version-consistency` job
(`.github/workflows/ci.yml`) fails the build if they ever drift apart.

There is no automatic version-propagation script yet — with only three small files, keeping them
in sync by hand (and letting CI catch mistakes) is simpler than adding tooling for it. Revisit
this if it becomes error-prone in practice.

## Cutting a release

1. Bump the version in the three files above, in one commit (e.g. `chore: bump version to
   0.2.0`), and push it to `master` like any other change — let CI pass on it first.
2. Tag that commit and push the tag:
   ```
   git tag v0.2.0
   git push origin v0.2.0
   ```
3. Pushing a tag matching `v*.*.*` triggers `.github/workflows/release.yml`, which builds
   PromptRig for macOS (arm64 + x64), Linux (x86_64), and Windows (x86_64) and creates a
   **draft** GitHub Release with all the installers attached.
4. Review the draft release on GitHub (edit the release notes if needed), then publish it
   manually. It's a draft on purpose — nothing goes live without a manual check first.

## What gets built, and how to install it

None of these installers are code-signed yet (see "Not yet done" below), so first launch shows
an extra OS warning (Gatekeeper on macOS, SmartScreen on Windows). The app still installs and
runs fine after clicking through it.

- **Windows**: a `.exe` installer (NSIS). Download, double-click, follow the installer.
- **macOS**: a `.dmg`. Open it, drag PromptRig to Applications. First launch: right-click the
  app → *Open* to get past the "unidentified developer" warning (only needed once).
- **Linux**:
  - Debian/Ubuntu and derivatives: the `.deb` package (`sudo dpkg -i promptrig_*.deb` or your
    package manager's GUI).
  - Fedora/openSUSE and derivatives: the `.rpm` package.
  - **Any other distro, including Arch-based ones (Arch, Manjaro, Omarchy, ...)**: use the
    `.AppImage`. It's a portable, self-contained binary — no installation, no package manager
    involved:
    ```
    chmod +x PromptRig_*.AppImage
    ./PromptRig_*.AppImage
    ```
    There is no native Arch package (`.pkg.tar.zst`) yet — see "Not yet done" below.

## Troubleshooting

**Linux + NVIDIA + Wayland**: the app can abort immediately on launch with
`Could not create GBM EGL display: EGL_SUCCESS. Aborting...`. This is a WebKitGTK issue with
NVIDIA's proprietary driver on Wayland (the driver's GBM support doesn't work the way WebKitGTK
expects when picking a hardware-accelerated rendering path), not a PromptRig bug — confirmed by
reproducing it directly with the `v0.1.0` `.AppImage` on this project's own dev machine (NVIDIA +
Hyprland/Wayland). Fixed by launching with the renderer's DMA-BUF path disabled:

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 ./PromptRig_*.AppImage
```

This applies regardless of install method (`.deb`/`.rpm`/`.AppImage`) — set the environment
variable before launching the `promptrig` binary either way. The same underlying issue, in a
milder form (a Wayland protocol error rather than a hard abort), is also why `docs/development.md`
recommends the same variable for `npm run tauri dev` on this kind of setup.

## Not yet done (deliberately deferred)

- **Code signing** (Apple Developer certificate, Windows code-signing certificate). Needed to
  remove the Gatekeeper/SmartScreen warnings above. Deferred until there's real user demand to
  justify the cost/setup — revisit `release.yml`'s `tauri-apps/tauri-action` step, which accepts
  signing secrets once they exist.
- **Arch Linux / AUR package**: Tauri's bundler doesn't produce a `pacman` package natively
  (only `.deb`, `.rpm`, `.AppImage` for Linux). Arch-based users can already use the AppImage
  above; a proper AUR package (typically a `-bin` PKGBUILD wrapping the AppImage or Release
  binary, published to `aur.archlinux.org`) is a separate, mostly-manual maintenance job — not
  started yet.
- **Auto-update**: needs the signing above plus a hosted update manifest
  (`tauri-plugin-updater`). Only worth building once there's a real release history to update
  *from*.
