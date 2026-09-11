use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};

use crate::domain::{
    AppError, AppResult, ExperimentId, GenerationParams, ProviderId, Run, RunId, RunResult, Usage,
};

use super::db::Database;

/// Fields needed to insert a new Run — everything `Run` has except `id`, which SQLite assigns
/// on insert.
#[derive(Clone)]
pub struct NewRun {
    pub experiment_id: Option<ExperimentId>,
    pub provider: ProviderId,
    pub model_id: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub params: GenerationParams,
    pub started_at: DateTime<Utc>,
    pub result: Option<RunResult>,
    pub error: Option<String>,
    pub estimated_cost_usd: Option<f64>,
    pub cost_is_estimate: bool,
}

fn json_err(e: serde_json::Error) -> AppError {
    AppError::Storage(format!("JSON (de)serialization failed: {e}"))
}

/// Inserts a Run and returns the id SQLite assigned it.
///
/// Takes `run` by value (rather than `&NewRun`) because the actual insert happens inside
/// `Database::with_connection`'s `spawn_blocking` task, which needs to own everything it
/// touches — it may run on a different thread than the caller.
pub async fn insert_run(db: &Database, run: NewRun) -> AppResult<RunId> {
    let params_json = serde_json::to_string(&run.params).map_err(json_err)?;

    let (result_text, usage_json, duration_ms, ttft_ms) = match &run.result {
        Some(result) => (
            Some(result.text.clone()),
            result
                .usage
                .map(|usage| serde_json::to_string(&usage))
                .transpose()
                .map_err(json_err)?,
            Some(result.duration_ms as i64),
            result.ttft_ms.map(|v| v as i64),
        ),
        None => (None, None, None, None),
    };

    db.with_connection(move |conn| {
        conn.execute(
            "INSERT INTO runs (
                experiment_id, provider, model_id, system_prompt, user_prompt, params_json,
                result_text, usage_json, duration_ms, ttft_ms, cost_estimate_usd,
                cost_is_estimate, error, started_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                run.experiment_id.map(|id| id.0),
                run.provider.as_str(),
                run.model_id,
                run.system_prompt,
                run.user_prompt,
                params_json,
                result_text,
                usage_json,
                duration_ms,
                ttft_ms,
                run.estimated_cost_usd,
                run.cost_is_estimate,
                run.error,
                run.started_at.to_rfc3339(),
            ],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;

        Ok(RunId(conn.last_insert_rowid()))
    })
    .await
}

/// The raw column values for one `runs` row, before any parsing that can fail (JSON, the
/// provider string, the timestamp). Keeping this step separate from `row_to_run` below means
/// the rusqlite row-mapping closure only ever needs to return `rusqlite::Result`, and all of our
/// own `AppError`-producing parsing happens afterward, outside of that closure.
type RawRunRow = (
    i64,
    Option<i64>,
    String,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<i64>,
    Option<i64>,
    Option<f64>,
    bool,
    Option<String>,
    String,
);

const SELECT_RUN_COLUMNS: &str = "id, experiment_id, provider, model_id, system_prompt, \
    user_prompt, params_json, result_text, usage_json, duration_ms, ttft_ms, cost_estimate_usd, \
    cost_is_estimate, error, started_at";

fn row_to_run(row: RawRunRow) -> AppResult<Run> {
    let (
        id,
        experiment_id,
        provider,
        model_id,
        system_prompt,
        user_prompt,
        params_json,
        result_text,
        usage_json,
        duration_ms,
        ttft_ms,
        cost_estimate_usd,
        cost_is_estimate,
        error,
        started_at,
    ) = row;

    let provider = ProviderId::parse(&provider)
        .ok_or_else(|| AppError::Storage(format!("unknown provider in database: {provider}")))?;
    let params: GenerationParams = serde_json::from_str(&params_json).map_err(json_err)?;
    let started_at = DateTime::parse_from_rfc3339(&started_at)
        .map_err(|e| AppError::Storage(format!("invalid started_at timestamp: {e}")))?
        .with_timezone(&Utc);
    let usage = usage_json
        .map(|json| serde_json::from_str::<Usage>(&json))
        .transpose()
        .map_err(json_err)?;

    let result = result_text.map(|text| RunResult {
        text,
        usage,
        duration_ms: duration_ms.unwrap_or(0) as u32,
        ttft_ms: ttft_ms.map(|v| v as u32),
    });

    Ok(Run {
        id: RunId(id),
        experiment_id: experiment_id.map(ExperimentId),
        provider,
        model_id,
        system_prompt,
        user_prompt,
        params,
        started_at,
        result,
        error,
        estimated_cost_usd: cost_estimate_usd,
        cost_is_estimate,
    })
}

/// Fetches a Run by id, or `Ok(None)` if it doesn't exist.
pub async fn get_run(db: &Database, id: RunId) -> AppResult<Option<Run>> {
    let raw: Option<RawRunRow> = db
        .with_connection(move |conn| {
            conn.query_row(
                &format!("SELECT {SELECT_RUN_COLUMNS} FROM runs WHERE id = ?1"),
                params![id.0],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                        row.get(10)?,
                        row.get(11)?,
                        row.get(12)?,
                        row.get(13)?,
                        row.get(14)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| AppError::Storage(e.to_string()))
        })
        .await?;

    raw.map(row_to_run).transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_run() -> NewRun {
        NewRun {
            experiment_id: None,
            provider: ProviderId::OpenAi,
            model_id: "gpt-4o-mini".into(),
            system_prompt: "You are a helpful assistant.".into(),
            user_prompt: "Say hi.".into(),
            params: GenerationParams {
                temperature: Some(0.7),
                top_p: None,
                max_tokens: Some(256),
                reasoning_effort: None,
            },
            started_at: DateTime::parse_from_rfc3339("2026-01-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            result: Some(RunResult {
                text: "Hi there!".into(),
                usage: Some(Usage {
                    input_tokens: 12,
                    output_tokens: 3,
                }),
                duration_ms: 450,
                ttft_ms: None,
            }),
            error: None,
            estimated_cost_usd: Some(0.000123),
            cost_is_estimate: false,
        }
    }

    #[tokio::test]
    async fn inserted_run_round_trips_through_get_run() {
        let db = Database::open_in_memory().unwrap();
        let new_run = sample_run();
        // Cloned up front since `insert_run` consumes its input but the test also wants to
        // compare against it afterward.
        let expected = new_run.clone();

        let id = insert_run(&db, new_run).await.expect("insert must succeed");
        let fetched = get_run(&db, id)
            .await
            .expect("get must succeed")
            .expect("run must exist");

        assert_eq!(fetched.id, id);
        assert_eq!(fetched.experiment_id, expected.experiment_id);
        assert_eq!(fetched.provider, expected.provider);
        assert_eq!(fetched.model_id, expected.model_id);
        assert_eq!(fetched.system_prompt, expected.system_prompt);
        assert_eq!(fetched.user_prompt, expected.user_prompt);
        assert_eq!(fetched.params, expected.params);
        assert_eq!(fetched.started_at, expected.started_at);
        assert_eq!(fetched.result, expected.result);
        assert_eq!(fetched.error, expected.error);
        assert_eq!(fetched.estimated_cost_usd, expected.estimated_cost_usd);
        assert_eq!(fetched.cost_is_estimate, expected.cost_is_estimate);
    }

    #[tokio::test]
    async fn failed_run_has_no_result() {
        let db = Database::open_in_memory().unwrap();
        let mut new_run = sample_run();
        new_run.result = None;
        new_run.error = Some("provider returned a 500".into());
        new_run.estimated_cost_usd = None;

        let id = insert_run(&db, new_run).await.unwrap();
        let fetched = get_run(&db, id).await.unwrap().unwrap();

        assert!(fetched.result.is_none());
        assert_eq!(fetched.error.as_deref(), Some("provider returned a 500"));
    }

    #[tokio::test]
    async fn get_run_returns_none_for_missing_id() {
        let db = Database::open_in_memory().unwrap();

        assert_eq!(get_run(&db, RunId(999)).await.unwrap(), None);
    }
}
