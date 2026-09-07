//! Tauri commands exposing API key management to the frontend.
//!
//! The raw key only ever flows one way: from the frontend's input field, through
//! `save_api_key`, into the OS keyring. Nothing here ever sends a key back out — `has_api_key`
//! reports presence as a boolean, never the value itself.

use crate::domain::{AppError, ProviderId};
use crate::secrets;

#[tauri::command]
#[specta::specta]
pub fn save_api_key(provider: ProviderId, api_key: String) -> Result<(), AppError> {
    secrets::save_api_key(provider, &api_key)
}

#[tauri::command]
#[specta::specta]
pub fn delete_api_key(provider: ProviderId) -> Result<(), AppError> {
    secrets::delete_api_key(provider)
}

#[tauri::command]
#[specta::specta]
pub fn has_api_key(provider: ProviderId) -> Result<bool, AppError> {
    secrets::has_api_key(provider)
}
