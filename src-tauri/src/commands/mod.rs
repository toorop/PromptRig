//! Tauri commands: the thin boundary between the frontend and the backend. Each command just
//! translates its arguments and delegates to a domain/provider/storage/secrets function —
//! business logic itself lives in those modules, not here.

pub mod secrets;
