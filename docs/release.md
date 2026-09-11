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

1. Add a new section to the top of [`CHANGELOG.md`](../CHANGELOG.md) (`## [x.y.z] - YYYY-MM-DD`,
   following [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)'s Added/Changed/Fixed/
   Removed grouping) summarizing what changed since the last tag — user-visible changes only,
   not internal refactors. Bump the version in the three files above to match, in the same
   commit (e.g. `chore: bump version to 0.2.0`), and push it to `master` like any other change —
   let CI pass on it first.
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
- **macOS**: a `.dmg`. Open it, drag PromptRig to Applications. Because the app has **no code
  signature at all** (not just "unnotarized"), macOS shows *"PromptRig is damaged and can't be
  opened"* on first launch instead of the more familiar "unidentified developer" prompt —
  confirmed via a real user's report (v0.1.0, downloaded via Chrome). The usual right-click →
  Open bypass does **not** work for this message (that trick only works when there's *some*
  signature to trust, even an ad-hoc one). The actual fix is to strip the quarantine attribute
  the browser download added:
  ```bash
  xattr -cr /Applications/PromptRig.app
  ```
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

### Optional: integrate the AppImage with your app launcher (Arch/Omarchy and similar)

An `.AppImage` doesn't register itself as an installed app the way a `.deb`/`.rpm` does — no
`promptrig` command, no entry in your app launcher (`wofi`/`rofi`/`walker`/...). Until there's a
native Arch package, here's a small setup that also re-picks up a newer AppImage automatically
whenever you download one, rather than needing to update a fixed path each release:

1. A launcher script that always runs whichever `PromptRig_*.AppImage` in `~/Downloads` was
   modified most recently:

   ```bash
   mkdir -p ~/.local/bin
   cat > ~/.local/bin/promptrig <<'EOF'
   #!/usr/bin/env bash
   set -euo pipefail
   shopt -s nullglob
   candidates=("$HOME/Downloads"/PromptRig_*.AppImage)
   shopt -u nullglob
   if [ ${#candidates[@]} -eq 0 ]; then
       echo "promptrig: no PromptRig_*.AppImage found in $HOME/Downloads" >&2
       exit 1
   fi
   appimage=$(ls -t -- "${candidates[@]}" | head -n1)
   # Drop the env var below if you don't hit the NVIDIA+Wayland issue described above.
   exec env WEBKIT_DISABLE_DMABUF_RENDERER=1 "$appimage" "$@"
   EOF
   chmod +x ~/.local/bin/promptrig
   ```

   Make sure `~/.local/bin` is on your `PATH` (it already is on most modern distros, including
   Omarchy). You now have a `promptrig` command.

2. Optional icon, extracted straight from the AppImage itself (works for any AppImage, not
   PromptRig-specific):

   ```bash
   cd /tmp && ~/Downloads/PromptRig_*.AppImage --appimage-extract '*.png' >/dev/null
   mkdir -p ~/.local/share/icons/hicolor/256x256/apps
   cp squashfs-root/*.png ~/.local/share/icons/hicolor/256x256/apps/promptrig.png
   rm -rf squashfs-root
   ```

3. A `.desktop` file so it shows up in your app launcher:

   ```bash
   mkdir -p ~/.local/share/applications
   cat > ~/.local/share/applications/promptrig.desktop <<'EOF'
   [Desktop Entry]
   Version=1.0
   Type=Application
   Name=PromptRig
   Comment=Prompt engineering and side-by-side LLM comparison
   Exec=promptrig
   Icon=promptrig
   Terminal=false
   Categories=Development;
   EOF
   update-desktop-database ~/.local/share/applications
   ```

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
