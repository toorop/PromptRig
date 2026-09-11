//! Tauri commands for provider status, connection testing, and model listing.

use tauri::State;

use crate::domain::{AppError, ModelInfo, ModelPricing, ProviderId};
use crate::pricing::ModelsDevCache;
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

/// Beyond just listing what a provider offers, this also drops any model models.dev marks
/// `"deprecated"` (no point offering a dead end in the picker — a `"beta"` model is still shown,
/// per the user's explicit call) and fills in each model's `pricing` field (a rate to show in
/// the picker, not a computed cost — see `ModelInfo::pricing`) so the frontend never needs its
/// own copy of the pricing-lookup logic. Providers themselves stay unaware models.dev exists at
/// all — this enrichment happens here, at the command boundary, same separation as `Run`'s cost
/// estimation in `build_new_run`.
#[tauri::command]
#[specta::specta]
pub async fn list_models(
    provider: ProviderId,
    providers: State<'_, ProviderRegistry>,
    models_dev: State<'_, ModelsDevCache>,
) -> Result<Vec<ModelInfo>, AppError> {
    let api_key = super::require_api_key(provider)?;
    let models = providers.get(provider)?.list_models(&api_key).await?;

    let mut kept = Vec::with_capacity(models.len());
    for mut model in models {
        if models_dev.is_deprecated(provider, &model.model_id).await {
            continue;
        }

        model.pricing = models_dev
            .price(provider, &model.model_id)
            .await
            .map(|rate| ModelPricing {
                input_per_million_usd: rate.input_per_million_usd,
                output_per_million_usd: rate.output_per_million_usd,
                is_estimate: false,
            });
        kept.push(model);
    }

    Ok(kept)
}
