//! API key storage backed by the operating system's native keyring: macOS Keychain, Windows
//! Credential Manager, or Secret Service on Linux (via the `keyring` crate's default `v1`
//! feature, which already selects the right backend per platform — see Cargo.toml). Keys are
//! never written to SQLite, a JSON file, or held anywhere in the frontend; see docs/start.md.

use keyring::Entry;

use crate::domain::{AppError, AppResult, ProviderId};

/// Every PromptRig secret is stored under this service name, with the provider id (e.g.
/// `"openai"`) as the account name. This groups all provider keys under one recognizable entry
/// in the OS's credential manager UI, while keeping each provider's key separate.
const SERVICE: &str = "promptrig";

fn entry_for(provider: ProviderId) -> AppResult<Entry> {
    Entry::new(SERVICE, provider.as_str()).map_err(|e| AppError::Secret(e.to_string()))
}

/// Stores (or overwrites) the API key for a provider in the OS keyring.
pub fn save_api_key(provider: ProviderId, api_key: &str) -> AppResult<()> {
    entry_for(provider)?
        .set_password(api_key)
        .map_err(|e| AppError::Secret(e.to_string()))
}

/// Removes the API key for a provider, if one is set. Deleting a key that was never set is not
/// treated as an error: the end state (no key stored) is the same either way.
pub fn delete_api_key(provider: ProviderId) -> AppResult<()> {
    map_delete_result(entry_for(provider)?.delete_credential())
}

/// Retrieves the stored API key for internal use (building provider HTTP requests in a later
/// step). Deliberately **not** registered as a Tauri command — the frontend must never receive
/// a raw key, only ever `has_api_key`'s boolean.
pub fn get_api_key(provider: ProviderId) -> AppResult<Option<String>> {
    map_get_result(entry_for(provider)?.get_password())
}

/// Whether a key is currently stored for this provider.
pub fn has_api_key(provider: ProviderId) -> AppResult<bool> {
    Ok(get_api_key(provider)?.is_some())
}

// The two functions below are pure (no keyring/OS access) so the error-mapping logic — the
// actual business rule here — can be unit-tested without touching a real credential store.
// Real keyring backends (Secret Service, Keychain, Credential Manager) usually aren't available
// or unlocked in a CI runner, so keeping this logic separate from the I/O call is what makes it
// testable at all.

fn map_delete_result(result: Result<(), keyring::Error>) -> AppResult<()> {
    match result {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Secret(e.to_string())),
    }
}

fn map_get_result(result: Result<String, keyring::Error>) -> AppResult<Option<String>> {
    match result {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Secret(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_treats_missing_entry_as_success() {
        assert!(map_delete_result(Ok(())).is_ok());
        assert!(map_delete_result(Err(keyring::Error::NoEntry)).is_ok());
    }

    #[test]
    fn delete_propagates_other_errors() {
        let result = map_delete_result(Err(keyring::Error::Invalid(
            "service".into(),
            "boom".into(),
        )));

        assert!(matches!(result, Err(AppError::Secret(_))));
    }

    #[test]
    fn get_maps_missing_entry_to_none() {
        assert_eq!(
            map_get_result(Ok("secret".into())).unwrap(),
            Some("secret".into())
        );
        assert_eq!(map_get_result(Err(keyring::Error::NoEntry)).unwrap(), None);
    }

    #[test]
    fn get_propagates_other_errors() {
        let result = map_get_result(Err(keyring::Error::Invalid(
            "service".into(),
            "boom".into(),
        )));

        assert!(matches!(result, Err(AppError::Secret(_))));
    }

    /// Exercises the real OS keyring end to end. Ignored by default because CI runners (and
    /// some dev machines) don't have an unlocked Secret Service/Keychain/Credential Manager
    /// available — run it manually with `cargo test -- --ignored` to sanity-check a real
    /// environment. Non-destructive: it restores whatever key was already stored, if any.
    #[test]
    #[ignore = "touches the real OS keyring; run manually with `cargo test -- --ignored`"]
    fn round_trips_against_the_real_os_keyring() {
        let provider = ProviderId::OpenAi;
        let previous = get_api_key(provider).expect("reading the existing key must not fail");

        save_api_key(provider, "promptrig-test-key").expect("save must succeed");
        assert!(has_api_key(provider).expect("has must succeed"));
        assert_eq!(
            get_api_key(provider).expect("get must succeed").as_deref(),
            Some("promptrig-test-key")
        );

        delete_api_key(provider).expect("delete must succeed");
        assert!(!has_api_key(provider).expect("has must succeed"));

        if let Some(key) = previous {
            save_api_key(provider, &key).expect("restoring the previous key must succeed");
        }
    }
}
