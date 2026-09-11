# Changelog

All notable changes to PromptRig are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project follows [Semantic Versioning](https://semver.org/).

## [0.3.0] - 2026-09-11

### Changed

- Model pricing and cost estimates now come from [models.dev](https://models.dev) instead of
  OpenRouter's own catalog. Pricing is looked up directly by each provider's native model id, so
  it's an exact price rather than an approximation for every model models.dev covers — including
  cases the old cross-provider matching could never resolve, like Mistral's rolling `-latest`
  aliases.
- The model picker no longer lists models that models.dev marks as deprecated (a beta model is
  still shown).

### Added

- models.dev's pricing data is now cached to disk between launches and refreshed with a
  conditional request (`ETag`/`If-None-Match`), so an unchanged catalog no longer re-downloads
  the full payload every time the app starts. If the request can't complete at all (offline, a
  server error), the last cached data is used instead of showing no price.

## [0.2.0] - 2026-09-08

### Added

- Searchable, filterable model picker (Playground and Compare), with an approximate per-model
  price shown in the dropdown.
- Window size and position are now remembered across restarts.
- Compare columns can be pinned so "Run all" skips them; a pinned column's state (including its
  last result) survives an app restart.

### Changed

- Slightly larger top navigation font.

## [0.1.0] - 2026-09-08

Initial release.

### Added

- **Playground**: run a single prompt against one provider and model, and see the response,
  token usage, latency, and estimated cost.
- **Compare**: run the same prompt across several provider/model columns side by side.
- Five providers: OpenAI, Anthropic, Google Gemini, Mistral, and OpenRouter.
- API keys stored in the OS-native keyring (Keychain / Credential Manager / Secret Service),
  never in plaintext.
- Run and Experiment history persisted locally in SQLite.
- Cross-provider cost estimates via OpenRouter's own pricing catalog (an approximation for
  non-OpenRouter providers, later replaced in 0.3.0).
- Dark/light theme, self-hosted fonts, and a frameless window with a custom title bar.
- Installers for Windows, macOS (arm64 + x64), and Linux (`.deb`/`.rpm`/`.AppImage`), built and
  published via GitHub Actions.
