use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use thiserror::Error;

/// The app's single domain-level error type.
///
/// `thiserror` gives us a `Display` impl (the `#[error("...")]` messages below) and lets other
/// modules convert their own errors into this one with `?` once we add `#[from]` conversions
/// (e.g. for `rusqlite::Error` once `storage/` exists). Every fallible operation in the app
/// returns `AppResult<T>` so callers get one consistent, informative error type instead of a
/// mix of raw strings.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("provider error: {0}")]
    Provider(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("secret storage error: {0}")]
    Secret(String),

    #[error("{0} not found")]
    NotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),
}

impl AppError {
    /// A short, stable machine-readable tag for this error's variant. Used by the frontend to
    /// branch on error type without string-matching the (human-readable, possibly-changing)
    /// `message`.
    fn kind(&self) -> &'static str {
        match self {
            AppError::Provider(_) => "provider",
            AppError::Storage(_) => "storage",
            AppError::Secret(_) => "secret",
            AppError::NotFound(_) => "not_found",
            AppError::InvalidInput(_) => "invalid_input",
        }
    }
}

// Tauri commands can return any error type that implements `Serialize` — the value is what the
// frontend's `invoke()` promise rejects with. We hand-roll this impl (rather than
// `#[derive(Serialize)]`) so the JSON always carries the polished `Display` message (e.g.
// "model xyz not found") instead of just the raw string a variant happens to wrap.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("kind", self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

/// Shorthand for `Result<T, AppError>`, used throughout the backend.
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_with_kind_and_display_message() {
        let err = AppError::NotFound("model xyz".into());

        let json = serde_json::to_value(&err).expect("AppError must serialize");

        assert_eq!(json["kind"], "not_found");
        assert_eq!(json["message"], "model xyz not found");
    }
}
