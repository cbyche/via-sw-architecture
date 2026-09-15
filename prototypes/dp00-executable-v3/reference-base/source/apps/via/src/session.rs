//! The voice session id `--session` defaults to.

use uuid::Uuid;

use via_i18n::{Locale, keys};

use crate::error::{CODE_INVALID_ARGUMENT, CliError};

/// The prefix every generated session id carries.
///
/// **External contract** — `docs/reference/contracts.json`
/// *default-value/createVoiceSessionId()*, `cli/src/arguments.mjs:31-33`:
/// `` `voice-${randomUUID().replaceAll('-', '')}` ``.
pub const VOICE_SESSION_PREFIX: &str = "voice-";

/// How many hex characters follow the prefix.
///
/// **External contract** — a UUID v4 with its four hyphens removed is 32 hex
/// characters. Stated as a constant because the *shape* is what a peer
/// matches on: the WebUI and the TUI share the id, and a shorter one would
/// still look plausible.
pub const VOICE_SESSION_HEX_LENGTH: usize = 32;

/// The environment variable that supplies a session id.
///
/// **External contract** — `cli/src/arguments.mjs:104`
/// (`env.<PREFIX>_SESSION_ID`), renamed by `docs/rebrand.md` row 93.
pub const SESSION_ID_ENV: &str = "VIA_SESSION_ID";

/// Generate a session id.
///
/// **External contract** — as [`VOICE_SESSION_PREFIX`]. `Uuid::new_v4` is the
/// same CSPRNG-backed v4 `randomUUID()` produces, and `simple()` is the
/// hyphen-free rendering.
#[must_use]
pub fn create_voice_session_id() -> String {
    std::format!("{VOICE_SESSION_PREFIX}{}", Uuid::new_v4().simple())
}

/// Trim a session id and refuse a blank one.
///
/// **External contract** — `cli/src/arguments.mjs:276-277`:
/// `options.sessionId = String(options.sessionId || '').trim()` and then
/// `if (!options.sessionId) throw new Error('--session 不能为空')`. The trim
/// happens *before* the check, so `--session '   '` is a refusal rather than a
/// session whose id is three spaces.
///
/// # Errors
///
/// [`CliError::Refused`] with [`CODE_INVALID_ARGUMENT`] when nothing is left
/// after trimming.
pub fn normalize_session_id(value: &str, locale: Locale) -> Result<String, CliError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CliError::refused(
            CODE_INVALID_ARGUMENT,
            locale,
            keys::CLI_SESSION_CANNOT_BE_EMPTY,
        ));
    }
    Ok(trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn a_generated_id_is_the_prefix_and_thirty_two_lowercase_hex_digits() {
        let id = create_voice_session_id();
        let rest = id
            .strip_prefix(VOICE_SESSION_PREFIX)
            .unwrap_or_else(|| panic!("`{id}` does not start with the catalogued prefix"));
        assert_eq!(rest.len(), VOICE_SESSION_HEX_LENGTH, "{id}");
        assert!(
            rest.chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
            "`{id}` is not lowercase hex"
        );
        // `replaceAll('-', '')`: the UUID's four hyphens are gone. The one
        // left in the id is the prefix's own.
        assert!(!rest.contains('-'), "the hyphens must be removed: {id}");
        assert_eq!(id.matches('-').count(), 1, "{id}");
    }

    #[test]
    fn two_generated_ids_differ() {
        assert_ne!(create_voice_session_id(), create_voice_session_id());
    }

    #[rstest]
    #[case("project-one", "project-one")]
    #[case("  padded  ", "padded")]
    #[case("\tt\n", "t")]
    fn a_session_id_is_trimmed(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            normalize_session_id(input, Locale::En).expect("non-blank"),
            expected
        );
    }

    #[rstest]
    #[case("")]
    #[case(" ")]
    #[case("   \t\n  ")]
    fn a_blank_session_id_is_refused_in_every_locale(#[case] input: &str) {
        let mut rendered = Vec::new();
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            let error =
                normalize_session_id(input, locale).expect_err("blank must not become an id");
            assert_eq!(error.code(), CODE_INVALID_ARGUMENT);
            let message = error.message(locale);
            assert!(message.contains("--session"), "{locale}: {message}");
            rendered.push(message);
        }
        assert_ne!(rendered[0], rendered[1]);
        assert_ne!(rendered[1], rendered[2]);
        assert_eq!(
            normalize_session_id(input, Locale::Zh)
                .expect_err("blank")
                .message(Locale::Zh),
            "--session 不能为空"
        );
    }
}
