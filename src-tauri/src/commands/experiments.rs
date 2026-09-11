//! The Tauri command behind Compare's "Run all" button.
//!
//! A side-by-side comparison is just one Experiment with several Runs sharing the same system
//! prompt, user prompt, and generation parameters — one Run per Provider+Model column (see
//! docs/start.md's Experiment section). There is no separate "comparison" concept here, only
//! `Experiment` + `Run`.

use chrono::Utc;
use tauri::State;

use crate::domain::{
    AppError, AppResult, Experiment, ExperimentId, GenerationParams, ProviderId, Run, RunId,
    RunResult,
};
use crate::pricing::ModelsDevCache;
use crate::providers::ProviderRegistry;
use crate::storage::{experiments_repo, runs_repo, Database};

/// One column of the comparison: which Provider + Model to run the shared prompt against.
#[derive(Debug, Clone, serde::Deserialize, specta::Type)]
pub struct ComparisonColumn {
    pub provider: ProviderId,
    pub model_id: String,
}

#[derive(Debug, Clone, serde::Deserialize, specta::Type)]
pub struct RunExperimentInput {
    pub system_prompt: String,
    pub user_prompt: String,
    pub params: GenerationParams,
    pub columns: Vec<ComparisonColumn>,
}

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct RunExperimentResult {
    pub experiment: Experiment,
    /// Same order as `RunExperimentInput::columns` — the frontend zips them back together by
    /// index, not by re-matching provider/model (two columns could use the same model).
    pub runs: Vec<Run>,
}

/// Runs one column: looks up its API key, calls the provider, and persists the result (or
/// failure) as a Run against `experiment_id`.
///
/// Unlike `runs::run_generation`, a missing API key or unimplemented provider here does **not**
/// abort the whole comparison — it's folded into the outcome just like a provider call failing,
/// so it shows up as an error on that one column's Run while the other columns still run. One
/// misconfigured column shouldn't sink an entire side-by-side comparison.
#[allow(clippy::too_many_arguments)]
async fn run_column(
    experiment_id: ExperimentId,
    column: ComparisonColumn,
    system_prompt: String,
    user_prompt: String,
    params: GenerationParams,
    providers: &ProviderRegistry,
    models_dev: &ModelsDevCache,
    db: &Database,
) -> AppResult<RunId> {
    let started_at = Utc::now();

    let outcome: AppResult<RunResult> = async {
        let api_key = super::require_api_key(column.provider)?;
        providers
            .get(column.provider)?
            .generate(
                &api_key,
                &column.model_id,
                &system_prompt,
                &user_prompt,
                &params,
            )
            .await
    }
    .await;

    let new_run = super::build_new_run(
        Some(experiment_id),
        column.provider,
        column.model_id,
        system_prompt,
        user_prompt,
        params,
        started_at,
        outcome,
        models_dev,
    )
    .await;

    runs_repo::insert_run(db, new_run).await
}

#[tauri::command]
#[specta::specta]
pub async fn run_experiment(
    input: RunExperimentInput,
    providers: State<'_, ProviderRegistry>,
    models_dev: State<'_, ModelsDevCache>,
    db: State<'_, Database>,
) -> Result<RunExperimentResult, AppError> {
    let created_at = Utc::now();
    let name = format!("Comparison {}", created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    let experiment_id = experiments_repo::insert_experiment(&db, name.clone(), created_at).await?;

    // Run every column concurrently rather than one after another — see docs/start.md's "Run
    // all... idéalement en parallèle". `join_all` polls all of them together within this one
    // async task (no `tokio::spawn`/`'static` needed, since we're not spawning separate tasks),
    // which is enough to have several HTTP requests in flight at once.
    let column_futures = input.columns.into_iter().map(|column| {
        run_column(
            experiment_id,
            column,
            input.system_prompt.clone(),
            input.user_prompt.clone(),
            input.params.clone(),
            &providers,
            &models_dev,
            &db,
        )
    });
    let run_ids = futures::future::join_all(column_futures).await;

    // `run_column` never returns `Err` for a provider/config failure — those become a persisted
    // Run with `error` set. An `Err` here means the Run itself couldn't be written to SQLite,
    // which is worth surfacing as a real command error rather than silently dropping a column.
    let mut runs = Vec::with_capacity(run_ids.len());
    for run_id in run_ids {
        let run_id = run_id?;
        let run = runs_repo::get_run(&db, run_id)
            .await?
            .ok_or_else(|| AppError::Storage("run vanished right after being inserted".into()))?;
        runs.push(run);
    }

    Ok(RunExperimentResult {
        experiment: Experiment {
            id: experiment_id,
            name,
            created_at,
        },
        runs,
    })
}
