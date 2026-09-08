//! The Tauri command behind the Playground's "Run" button.

use chrono::Utc;
use tauri::State;

use crate::domain::{AppError, GenerationParams, ProviderId, Run};
use crate::pricing::{OpenRouterPricingCache, PricingTable};
use crate::providers::ProviderRegistry;
use crate::storage::{runs_repo, Database};

/// What the frontend sends to run one generation. Mirrors `Run`'s "request" half — the
/// "result" half gets filled in here after calling the provider.
#[derive(Debug, Clone, serde::Deserialize, specta::Type)]
pub struct RunGenerationInput {
    pub provider: ProviderId,
    pub model_id: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub params: GenerationParams,
}

#[tauri::command]
#[specta::specta]
pub async fn run_generation(
    input: RunGenerationInput,
    providers: State<'_, ProviderRegistry>,
    pricing: State<'_, PricingTable>,
    openrouter_pricing: State<'_, OpenRouterPricingCache>,
    db: State<'_, Database>,
) -> Result<Run, AppError> {
    let api_key = super::require_api_key(input.provider)?;
    let provider = providers.get(input.provider)?;
    let started_at = Utc::now();

    let outcome = provider
        .generate(
            &api_key,
            &input.model_id,
            &input.system_prompt,
            &input.user_prompt,
            &input.params,
        )
        .await;

    let new_run = super::build_new_run(
        None,
        input.provider,
        input.model_id,
        input.system_prompt,
        input.user_prompt,
        input.params,
        started_at,
        outcome,
        &pricing,
        &openrouter_pricing,
    )
    .await;

    let id = runs_repo::insert_run(&db, new_run).await?;
    runs_repo::get_run(&db, id)
        .await?
        .ok_or_else(|| AppError::Storage("run vanished right after being inserted".into()))
}
