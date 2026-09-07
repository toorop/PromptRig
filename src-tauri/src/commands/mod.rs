//! Tauri commands: the thin boundary between the frontend and the backend. Each command just
//! translates its arguments and delegates to a domain/provider/storage/secrets function —
//! business logic itself lives in those modules, not here.

pub mod providers;
pub mod runs;
pub mod secrets;

use crate::domain::{AppError, AppResult, ProviderId};
use crate::secrets as secrets_store;

/// Looks up the API key for `provider`, turning "none configured" into a clear error instead of
/// letting a provider call fail with a confusing auth error. Shared by every command that needs
/// to actually talk to a provider (`providers::test_provider_connection`, `providers::
/// list_models`, `runs::run_generation`).
fn require_api_key(provider: ProviderId) -> AppResult<String> {
    secrets_store::get_api_key(provider)?.ok_or_else(|| {
        AppError::InvalidInput(format!(
            "No API key configured for {}. Add one in Settings first.",
            provider.display_name()
        ))
    })
}
