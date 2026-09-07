//! SQLite storage for everything that isn't a secret (see `secrets/` for API keys). Each
//! `*_repo` module owns the SQL for one table and translates rows to/from domain types — the
//! rest of the app never writes SQL directly.

pub mod db;
pub mod runs_repo;

pub use db::Database;
