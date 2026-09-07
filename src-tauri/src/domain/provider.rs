use serde::{Deserialize, Serialize};

/// Identifies which LLM provider a model, API key, or Run belongs to.
///
/// This is a closed enum (not a free-form string) so the compiler catches typos and a future
/// `ProviderRegistry` can exhaustively match over every known provider. Adding a new provider
/// means adding a variant here plus a new module under `providers/` — nothing else needs to
/// change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderId {
    #[serde(rename = "openai")]
    OpenAi,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "mistral")]
    Mistral,
    #[serde(rename = "openrouter")]
    OpenRouter,
    #[serde(rename = "openai_compatible")]
    OpenAiCompatible,
}

impl ProviderId {
    /// All known providers, in the order they should appear in the UI.
    pub const ALL: [ProviderId; 6] = [
        ProviderId::OpenAi,
        ProviderId::Anthropic,
        ProviderId::Gemini,
        ProviderId::Mistral,
        ProviderId::OpenRouter,
        ProviderId::OpenAiCompatible,
    ];

    /// Stable identifier used as a storage key: SQLite rows and OS keyring service names are
    /// keyed by this string, so it must never change for an existing variant (renaming the Rust
    /// variant name is fine, this string is not). Kept in sync by hand with the `#[serde(rename
    /// = ...)]` attributes above — the test below guards that.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderId::OpenAi => "openai",
            ProviderId::Anthropic => "anthropic",
            ProviderId::Gemini => "gemini",
            ProviderId::Mistral => "mistral",
            ProviderId::OpenRouter => "openrouter",
            ProviderId::OpenAiCompatible => "openai_compatible",
        }
    }

    /// Human-readable name for the UI.
    pub fn display_name(&self) -> &'static str {
        match self {
            ProviderId::OpenAi => "OpenAI",
            ProviderId::Anthropic => "Anthropic",
            ProviderId::Gemini => "Google Gemini",
            ProviderId::Mistral => "Mistral",
            ProviderId::OpenRouter => "OpenRouter",
            ProviderId::OpenAiCompatible => "OpenAI-compatible",
        }
    }

    /// The inverse of `as_str()` — used by the storage layer to turn the `provider` column back
    /// into a `ProviderId` when reading a row. `None` for anything that isn't one of the known
    /// strings (e.g. a row written by a future version of the app with a provider we don't know
    /// about yet).
    pub fn parse(s: &str) -> Option<Self> {
        ProviderId::ALL.into_iter().find(|p| p.as_str() == s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `as_str()` and the serde `rename` attribute are two independent, hand-maintained sources
    /// of the same string. If they ever drift, SQLite rows and keyring entries (written via
    /// `as_str()`) would silently stop matching what JSON payloads (written via serde) expect.
    #[test]
    fn as_str_matches_serde_representation() {
        for provider in ProviderId::ALL {
            let json = serde_json::to_string(&provider).expect("ProviderId must serialize");
            let expected = format!("\"{}\"", provider.as_str());
            assert_eq!(
                json, expected,
                "as_str() and serde rename disagree for {provider:?}"
            );
        }
    }

    #[test]
    fn parse_reverses_as_str_for_every_provider() {
        for provider in ProviderId::ALL {
            assert_eq!(ProviderId::parse(provider.as_str()), Some(provider));
        }
    }

    #[test]
    fn parse_rejects_unknown_strings() {
        assert_eq!(ProviderId::parse("not_a_real_provider"), None);
    }
}
