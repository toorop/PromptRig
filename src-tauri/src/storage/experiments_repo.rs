use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::domain::{AppError, AppResult, ExperimentId};

use super::db::Database;

/// Inserts an Experiment row and returns its id. Runs belonging to it (see `runs_repo::NewRun`)
/// reference this id via a foreign key, so the Experiment must exist before any of its Runs are
/// inserted.
///
/// There's no `get_experiment`/`list_experiments` yet — nothing needs them until a "browse past
/// experiments" feature exists. `run_experiment` (in `commands::experiments`) already knows
/// everything about the Experiment it just created (id, name, timestamp) without reading it
/// back.
pub async fn insert_experiment(
    db: &Database,
    name: String,
    created_at: DateTime<Utc>,
) -> AppResult<ExperimentId> {
    db.with_connection(move |conn| {
        conn.execute(
            "INSERT INTO experiments (name, created_at) VALUES (?1, ?2)",
            params![name, created_at.to_rfc3339()],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;

        Ok(ExperimentId(conn.last_insert_rowid()))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{GenerationParams, ProviderId};
    use crate::storage::runs_repo::{self, NewRun};

    fn sample_run(experiment_id: Option<ExperimentId>) -> NewRun {
        NewRun {
            experiment_id,
            provider: ProviderId::OpenAi,
            model_id: "gpt-4o-mini".into(),
            system_prompt: "system".into(),
            user_prompt: "user".into(),
            params: GenerationParams::default(),
            started_at: Utc::now(),
            result: None,
            error: Some("not actually run".into()),
            estimated_cost_usd: None,
        }
    }

    #[tokio::test]
    async fn insert_experiment_assigns_distinct_ids() {
        let db = Database::open_in_memory().unwrap();

        let first = insert_experiment(&db, "Comparison".into(), Utc::now())
            .await
            .unwrap();
        let second = insert_experiment(&db, "Another comparison".into(), Utc::now())
            .await
            .unwrap();

        assert_ne!(first, second);
    }

    #[tokio::test]
    async fn a_run_can_reference_a_real_experiment() {
        let db = Database::open_in_memory().unwrap();
        let experiment_id = insert_experiment(&db, "Comparison".into(), Utc::now())
            .await
            .unwrap();

        let run_id = runs_repo::insert_run(&db, sample_run(Some(experiment_id)))
            .await
            .expect("inserting a run against a real experiment must succeed");

        let run = runs_repo::get_run(&db, run_id).await.unwrap().unwrap();
        assert_eq!(run.experiment_id, Some(experiment_id));
    }

    /// Guards the `PRAGMA foreign_keys = ON` set in `Database::from_connection` (Step 3): without
    /// it, this insert would silently succeed instead of being rejected.
    #[tokio::test]
    async fn a_run_cannot_reference_a_nonexistent_experiment() {
        let db = Database::open_in_memory().unwrap();
        let bogus_experiment_id = ExperimentId(999_999);

        let result = runs_repo::insert_run(&db, sample_run(Some(bogus_experiment_id))).await;

        assert!(result.is_err(), "foreign key violation should be rejected");
    }
}
