//! Two on-device modes behind one provider key.
//!
//! `docs/architecture.md` §7 lists `local-omni:pipeline` and
//! `local-omni:endpoint` as separate rows, and `via-catalog`'s `local-omni` row
//! says why they are nevertheless one key:
//!
//! > Two modes, `local-omni:pipeline` (100% Rust: sherpa-onnx + llama-cpp-2) and
//! > `local-omni:endpoint` (an OpenAI-compatible local server); **the mode is a
//! > session setting, not a separate provider key.**
//!
//! That constraint is not stylistic. [`is_valid_provider_key`] refuses a `:`, so
//! `local-omni:pipeline` could never be a registry key in the first place; and a
//! client that has already stored `local-omni` must keep resolving whichever
//! mode the operator later switches to. So the key is [`PROVIDER_KEY`], the mode
//! is [`LocalMode`], and exactly one of the two providers is registered under
//! the key at a time.
//!
//! [`is_valid_provider_key`]: via_realtime::is_valid_provider_key

use crate::error::LocalError;

/// The provider key both modes register under.
///
/// External contract — `via-catalog`'s `local-omni` row, echoed by
/// `/api/health` and accepted on a client's `connect` frame. Read from the
/// catalog rather than retyped; the constant exists so a reader can see the
/// value without opening another crate, and [`crate::provider_key`] asserts the
/// two agree.
pub const PROVIDER_KEY: &str = "local-omni";

/// The separator between the provider key and the mode.
pub const MODE_SEPARATOR: char = ':';

/// Which on-device path is mounted.
///
/// The default is [`Pipeline`](Self::Pipeline), and `docs/architecture.md` §16
/// is explicit about why: *"The 100% Rust componentized path first, because it
/// runs on the dev box and can be exercised. `local-omni:endpoint` follows and
/// ships unverified-on-hardware, saying so."*
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum LocalMode {
    /// VAD → streaming ASR → the reasoning turn → TTS, in-process, presented to
    /// the Gateway as one realtime session. Runs on a laptop.
    #[default]
    Pipeline,
    /// An OpenAI-Realtime client pointed at a local server. CUDA-only in
    /// practice, and **unverified on this machine** — see
    /// [`crate::endpoint`].
    Endpoint,
}

impl LocalMode {
    /// Both modes, in `docs/architecture.md` §7's order.
    pub const ALL: [Self; 2] = [Self::Pipeline, Self::Endpoint];

    /// The bare mode name, without the provider key.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pipeline => "pipeline",
            Self::Endpoint => "endpoint",
        }
    }

    /// The qualified name `docs/architecture.md` §7 writes: `local-omni:pipeline`.
    #[must_use]
    pub fn qualified(self) -> String {
        format!("{PROVIDER_KEY}{MODE_SEPARATOR}{}", self.name())
    }

    /// Whether this mode has ever been exercised against real hardware in this
    /// repository.
    ///
    /// `false` for [`Endpoint`](Self::Endpoint), and the crate says so in three
    /// places rather than assuming it silently: here, in the module docs, and in
    /// [`crate::LocalHealth`].
    #[must_use]
    pub const fn is_verified_on_this_machine(self) -> bool {
        matches!(self, Self::Pipeline)
    }

    /// Read a mode from configuration.
    ///
    /// Accepts the bare name and the qualified one, trims, and folds ASCII case
    /// — the same leniency [`clean_key`](via_realtime::clean_key) gives a
    /// provider name, because both arrive from the same configuration surface.
    /// An empty value is the default rather than an error: a Gateway that
    /// configured nothing gets the path that runs.
    ///
    /// # Errors
    ///
    /// [`LocalError::UnknownMode`] naming the value and both accepted spellings.
    pub fn parse(value: &str) -> Result<Self, LocalError> {
        let cleaned = value.trim().to_ascii_lowercase();
        if cleaned.is_empty() {
            return Ok(Self::default());
        }
        let bare = cleaned
            .strip_prefix(PROVIDER_KEY)
            .and_then(|rest| rest.strip_prefix(MODE_SEPARATOR))
            .unwrap_or(&cleaned);
        Self::ALL
            .into_iter()
            .find(|mode| mode.name() == bare)
            .ok_or_else(|| LocalError::UnknownMode {
                value: value.trim().to_owned(),
            })
    }
}

impl core::fmt::Display for LocalMode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn the_qualified_names_are_the_two_architecture_rows() {
        assert_eq!(LocalMode::Pipeline.qualified(), "local-omni:pipeline");
        assert_eq!(LocalMode::Endpoint.qualified(), "local-omni:endpoint");
    }

    #[test]
    fn the_qualified_name_is_not_a_legal_provider_key() {
        // The reason the mode cannot be a key of its own: `:` is outside
        // `^[a-z0-9][a-z0-9-]*$`, so a registry could never hold it.
        assert!(via_realtime::is_valid_provider_key(PROVIDER_KEY));
        for mode in LocalMode::ALL {
            assert!(
                !via_realtime::is_valid_provider_key(&mode.qualified()),
                "{mode}"
            );
        }
    }

    #[test]
    fn both_spellings_parse_and_case_and_space_are_forgiven() {
        for (input, expected) in [
            ("pipeline", LocalMode::Pipeline),
            ("endpoint", LocalMode::Endpoint),
            ("local-omni:pipeline", LocalMode::Pipeline),
            ("local-omni:endpoint", LocalMode::Endpoint),
            ("  LOCAL-OMNI:Endpoint \n", LocalMode::Endpoint),
            ("  Pipeline ", LocalMode::Pipeline),
        ] {
            assert_eq!(LocalMode::parse(input), Ok(expected), "{input}");
        }
    }

    #[test]
    fn an_empty_value_is_the_default_rather_than_an_error() {
        for input in ["", "   ", "\t\n"] {
            assert_eq!(
                LocalMode::parse(input),
                Ok(LocalMode::Pipeline),
                "{input:?}"
            );
        }
        assert_eq!(LocalMode::default(), LocalMode::Pipeline);
    }

    #[test]
    fn an_unknown_mode_names_the_value_it_refused() {
        // Including the near misses: a bare key with no mode, and the key
        // spelled with the wrong separator.
        for input in ["local-omni", "local-omni/pipeline", "omni", "streaming"] {
            assert_eq!(
                LocalMode::parse(input),
                Err(LocalError::UnknownMode {
                    value: input.to_owned()
                }),
                "{input}"
            );
        }
    }

    #[test]
    fn only_the_pipeline_claims_to_have_been_verified() {
        assert!(LocalMode::Pipeline.is_verified_on_this_machine());
        assert!(!LocalMode::Endpoint.is_verified_on_this_machine());
    }

    #[test]
    fn the_mode_serializes_as_its_bare_name() {
        assert_eq!(
            serde_json::to_value(LocalMode::Endpoint).expect("serialize"),
            serde_json::json!("endpoint")
        );
    }
}
