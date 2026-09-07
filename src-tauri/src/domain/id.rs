//! Shared serde helper for id newtypes that wrap `i64` (see `RunId`, `ExperimentId`).
//!
//! Specta refuses to export `i64`/`u64` straight to TypeScript — JS numbers can't represent the
//! full range without losing precision — so ids cross the Tauri IPC boundary as strings
//! instead. Internally (SQLite, Rust comparisons) they stay plain `i64`. Used as
//! `#[serde(with = "id_as_string")]` on the newtype's single field, alongside `#[specta(type =
//! String)]` to tell specta what TypeScript type to declare for it.
pub(crate) mod id_as_string {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(value: &i64, serializer: S) -> Result<S::Ok, S::Error> {
        value.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}
