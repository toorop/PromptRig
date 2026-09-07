//! Tauri commands for provider status, connection testing, and model listing.

use tauri::State;

use crate::domain::{AppError, ModelInfo, ProviderId};
use crate::providers::ProviderRegistry;
use crate::secrets;

/// One provider's status for the Settings screen: whether PromptRig has an implementation for
/// it yet, and whether the user has configured a key for it.
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct ProviderStatus {
    pub provider: ProviderId,
    pub display_name: &'static str,
    pub implemented: bool,
    pub configured: bool,
}

#[tauri::command]
#[specta::specta]
pub fn list_providers(
    providers: State<'_, ProviderRegistry>,
) -> Result<Vec<ProviderStatus>, AppError> {
    ProviderId::ALL
        .into_iter()
        .map(|provider| {
            Ok(ProviderStatus {
                provider,
                display_name: provider.display_name(),
                implemented: providers.get(provider).is_ok(),
                configured: secrets::has_api_key(provider)?,
            })
        })
        .collect()
}

#[tauri::command]
#[specta::specta]
pub async fn test_provider_connection(
    provider: ProviderId,
    providers: State<'_, ProviderRegistry>,
) -> Result<(), AppError> {
    let api_key = super::require_api_key(provider)?;
    providers.get(provider)?.test_connection(&api_key).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_models(
    provider: ProviderId,
    providers: State<'_, ProviderRegistry>,
) -> Result<Vec<ModelInfo>, AppError> {
    let api_key = super::require_api_key(provider)?;
    providers.get(provider)?.list_models(&api_key).await
}
