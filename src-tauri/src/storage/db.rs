use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

use crate::domain::{AppError, AppResult};

/// The database schema, as a sequence of migrations. Each `M::up(...)` is one forward step;
/// `rusqlite_migration` tracks which have already run (via SQLite's `user_version` pragma) and
/// only applies the new ones. Add a new `M::up(...)` entry — never edit an existing one — when
/// the schema needs to change.
fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(include_str!("migrations/0001_initial.sql")),
        M::up(include_str!("migrations/0002_add_cost_is_estimate.sql")),
    ])
}

/// A handle to the app's SQLite database.
///
/// Cloning a `Database` is cheap and shares the same underlying connection (via `Arc`) — this
/// is what lets it be stored once in Tauri's managed state and handed to every command that
/// needs it. A single `Mutex<Connection>` is enough concurrency for a desktop app talking to
/// its own local file; there's no need for a connection pool here.
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Opens (creating if necessary) the database file at `path` and brings its schema up to
    /// date.
    pub fn open(path: &Path) -> AppResult<Self> {
        let conn = Connection::open(path).map_err(|e| AppError::Storage(e.to_string()))?;
        Self::from_connection(conn)
    }

    /// An in-memory database with the same schema, used by tests so they never touch the
    /// filesystem.
    pub fn open_in_memory() -> AppResult<Self> {
        let conn = Connection::open_in_memory().map_err(|e| AppError::Storage(e.to_string()))?;
        Self::from_connection(conn)
    }

    fn from_connection(mut conn: Connection) -> AppResult<Self> {
        // Off by default in SQLite; without it, `ON DELETE SET NULL` on runs.experiment_id
        // (see the initial migration) would silently not apply.
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(|e| AppError::Storage(e.to_string()))?;

        migrations()
            .to_latest(&mut conn)
            .map_err(|e| AppError::Storage(format!("running migrations failed: {e}")))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Runs `f` with exclusive access to the connection, on a blocking-friendly thread.
    ///
    /// `rusqlite` is synchronous, and Tauri commands are async — calling blocking SQLite I/O
    /// directly on an async task would stall whatever else that task's thread is doing. Routing
    /// every database access through `spawn_blocking` here, in one place, means repositories
    /// (`runs_repo` etc.) can stay plain, easy-to-read synchronous functions internally while
    /// still being safe to call from async command handlers.
    ///
    /// `f` must be `'static` (own everything it needs, no borrows from the caller) because it
    /// may run on a different thread than the one that called `with_connection`.
    pub async fn with_connection<T, F>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> AppResult<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = Arc::clone(&self.conn);

        let join_result = tauri::async_runtime::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|_| AppError::Storage("database connection lock was poisoned".into()))?;
            f(&conn)
        })
        .await;

        match join_result {
            Ok(result) => result,
            Err(e) => Err(AppError::Storage(format!("database task panicked: {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn opens_in_memory_and_applies_migrations() {
        let db = Database::open_in_memory().expect("in-memory database must open");

        // If migrations didn't run, this query would fail with "no such table: runs".
        let table_exists = db
            .with_connection(|conn| {
                conn.query_row(
                    "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'runs'",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|e| AppError::Storage(e.to_string()))
            })
            .await
            .expect("query must succeed");

        assert_eq!(table_exists, 1);
    }
}
