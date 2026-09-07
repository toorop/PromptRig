use serde::Serialize;
use thiserror::Error;

/// The app's single domain-level error type.
///
/// Each variant carries an already-fully-formatted message — callers build the whole sentence
/// at the point where they have the context to make it useful (e.g. `format!("OpenAI request
/// failed: {e}")`), rather than this type gluing a generic prefix onto a fragment. This keeps
/// `Display` (via thiserror, for logs and internal `?` conversions), `Serialize`, and
/// `specta::Type` (for the generated TypeScript bindings) all in agreement automatically: they
/// all just read the same `#[serde(tag = "kind", content = "message")]` shape, with no
/// hand-written impl needed.
#[derive(Debug, Error, Serialize, specta::Type)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    #[error("{0}")]
    Provider(String),

    #[error("{0}")]
    Storage(String),

    #[error("{0}")]
    Secret(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    InvalidInput(String),
}

/// Shorthand for `Result<T, AppError>`, used throughout the backend.
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_as_a_tagged_kind_and_message() {
        let err = AppError::NotFound("model xyz not found".into());

        let json = serde_json::to_value(&err).expect("AppError must serialize");

        assert_eq!(json["kind"], "NotFound");
        assert_eq!(json["message"], "model xyz not found");
    }
}
