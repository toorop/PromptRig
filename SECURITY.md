# Security Policy

## Reporting a vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities. Instead, use
[GitHub's private vulnerability reporting](../../security/advisories/new) (Security tab →
"Report a vulnerability") for this repository. We'll acknowledge the report and work with you
on a fix and disclosure timeline.

## Scope notes specific to PromptRig

- **API keys** are stored in the operating system's native keyring (macOS Keychain, Windows
  Credential Manager, Secret Service on Linux) and must never be persisted in SQLite, JSON
  files, `localStorage`, or anywhere in the frontend. If you find a code path that stores a key
  or secret outside the keyring, that's a security bug — please report it.
- When reporting, **never include a real API key** (yours or anyone else's) in an issue,
  advisory, or pull request. Redact secrets from logs and screenshots before sharing them.
- Requests to LLM providers are made directly from the user's machine to the provider's API
  using the user's own key; there is no PromptRig-operated backend that sees your prompts or
  keys.

## Supported versions

This project is in early development (pre-1.0). Security fixes land on the latest release;
there is no long-term support branch yet.
