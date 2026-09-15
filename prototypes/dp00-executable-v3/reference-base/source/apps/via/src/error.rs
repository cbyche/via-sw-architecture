//! What the binary refuses to do, and the exit code it refuses with.
//!
//! # Exit codes
//!
//! **External contract** — `docs/reference/contracts.json` *exit-code/process
//! exit codes*, `cli/bin/<binary>.mjs:16-31`: *"0 = success; 1 = any thrown
//! error (stderr `<binary>: <message>`)"*. Everything else that entry lists —
//! `gateway status` returning 1 when the service is not reachable, foreground
//! `gateway run` returning the child's code — belongs to a command that does
//! not exist until phase 5, and is recorded as owed in
//! `docs/deviations/phase-1.md` rather than invented here.
//!
//! The one place VIA departs from `clap`'s conventions is that a **usage**
//! error also exits 1, not `clap`'s default 2. Upstream throws for an unknown
//! flag exactly as it throws for anything else, and the catalogued contract
//! says every thrown error is 1; a script that branches on the exit code would
//! otherwise see two different numbers for the same class of mistake. See
//! [`crate::cli::parse_from`].

use std::io;
use std::path::PathBuf;

use via_core::CoreError;
use via_i18n::{Locale, format, keys, t};

use crate::phase::Unimplemented;

/// A successful run.
///
/// **External contract** — as above.
pub const EXIT_SUCCESS: u8 = 0;

/// Any refusal at all.
///
/// **External contract** — as above.
pub const EXIT_FAILURE: u8 = 1;

/// The code reported for a command whose body is not written yet.
///
/// VIA-owned: upstream has no unimplemented commands, so there is no upstream
/// code to inherit. The `VIA_CLI_` prefix keeps it from ever being mistaken
/// for a contract a client already knows, following the convention
/// `via-protocol` set for `VIA_PROTOCOL_*`.
pub const CODE_NOT_IMPLEMENTED: &str = "VIA_CLI_NOT_IMPLEMENTED";

/// The code reported when an argument's *value* is rejected.
///
/// VIA-owned, as above. `clap` rejects an argument that does not exist or is
/// missing its value; this code covers the checks `clap`'s type system cannot
/// make — an unparseable URL, a blank `--session`, an unknown `config`
/// subcommand.
pub const CODE_INVALID_ARGUMENT: &str = "VIA_CLI_INVALID_ARGUMENT";

/// The code reported for a filesystem failure.
pub const CODE_IO: &str = "VIA_CLI_IO";

/// Anything the binary declines to do.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// An argument, or a value inside one, that this command cannot accept.
    ///
    /// The message is already localized; the `code` is not (per
    /// `docs/architecture.md` §16, *error codes are not localized — only the
    /// message beside them*).
    #[error("{message}")]
    Refused {
        /// The stable machine-readable code.
        code: &'static str,
        /// The localized sentence.
        message: String,
    },

    /// The command exists, is spelled correctly, and has no body yet.
    #[error("{0}")]
    Unimplemented(Unimplemented),

    /// `via-core` refused: a malformed configuration, or the setup gate.
    #[error("{0}")]
    Core(#[from] CoreError),

    /// A filesystem operation failed, with the path it was operating on.
    #[error("{operation} {}: {source}", path.display())]
    Io {
        /// What was being attempted, e.g. `read`.
        operation: &'static str,
        /// The path involved.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: io::Error,
    },
}

impl From<via_catalog::CatalogError> for CliError {
    /// A catalog lookup failure travels as a [`CoreError::Catalog`], so a
    /// caller that reads `error.code()` gets `VIA_BACKEND_UNSUPPORTED` here
    /// and from the Gateway alike.
    fn from(error: via_catalog::CatalogError) -> Self {
        Self::Core(CoreError::Catalog(error))
    }
}

impl CliError {
    /// Build a localized refusal from a catalog key with no placeholders.
    #[must_use]
    pub fn refused(code: &'static str, locale: Locale, key: via_i18n::Key) -> Self {
        Self::Refused {
            code,
            message: t(locale, key).to_owned(),
        }
    }

    /// Build a localized refusal from a catalog key and its arguments.
    #[must_use]
    pub fn refused_with(
        code: &'static str,
        locale: Locale,
        key: via_i18n::Key,
        args: &[(&str, &str)],
    ) -> Self {
        Self::Refused {
            code,
            message: format(locale, key, args),
        }
    }

    /// Wrap an [`io::Error`] with the operation and path that produced it.
    #[must_use]
    pub fn io(operation: &'static str, path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            operation,
            path: path.into(),
            source,
        }
    }

    /// The stable machine-readable code.
    ///
    /// `via-core`'s and `via-catalog`'s codes are passed through unchanged, so
    /// a caller that already branches on `VIA_GATEWAY_SETUP_REQUIRED` sees the
    /// same string here that it would see from the Gateway itself.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Refused { code, .. } => code,
            Self::Unimplemented(_) => CODE_NOT_IMPLEMENTED,
            Self::Core(error) => error.code(),
            Self::Io { .. } => CODE_IO,
        }
    }

    /// The sentence written after [`crate::STDERR_PREFIX`], in `locale`.
    ///
    /// Most variants carry an already-localized message. [`Self::Core`] is the
    /// exception: `via-core` is a library and its `Display` is developer
    /// English for every variant except the setup gate, whose message the gate
    /// itself rendered. This is where the remaining ones are looked up, keyed
    /// by the code so a new `via-core` variant falls back to its `Display`
    /// rather than to a blank line.
    #[must_use]
    pub fn message(&self, locale: Locale) -> String {
        match self {
            Self::Refused { message, .. } => message.clone(),
            Self::Unimplemented(unimplemented) => unimplemented.message(locale),
            Self::Core(error) => localize_core(locale, error),
            Self::Io { .. } => self.to_string(),
        }
    }
}

/// Render a [`CoreError`] in `locale`.
///
/// The setup gate has already rendered its own sentence — it is the one
/// `via-core` variant whose `Display` is user-facing — so it is returned as
/// is. The rest are matched on the *code*, which is the stable discriminant,
/// and interpolated from the `via-i18n` key that carries upstream's wording.
fn localize_core(locale: Locale, error: &CoreError) -> String {
    match error {
        CoreError::GatewaySetupRequired { message, .. } => message.clone(),
        CoreError::UnsupportedBackendPermissionMode { requested } => format(
            locale,
            keys::BACKEND_UNSUPPORTED_PERMISSION_MODE,
            &[("mode", requested.as_str())],
        ),
        CoreError::BackendWorkspaceMissing { protocol } => format(
            locale,
            keys::BACKEND_UNSUPPORTED_AGENT,
            &[("id", protocol.as_str())],
        ),
        CoreError::Catalog(catalog) => localize_catalog(locale, catalog),
        // Every other variant is an operator-facing startup failure with no
        // catalogued sentence: `via-core`'s English `Display` is the most
        // informative thing there is, and inventing a translation for a
        // message nobody catalogued would be inventing a value.
        other => other.to_string(),
    }
}

/// Render a [`via_catalog::CatalogError`] in `locale`.
fn localize_catalog(locale: Locale, error: &via_catalog::CatalogError) -> String {
    use via_catalog::CatalogError as E;
    match error {
        E::UnknownRealtimeModel { model } => format(
            locale,
            keys::REALTIME_UNSUPPORTED_MODEL,
            &[("id", model.as_str())],
        ),
        E::UnsupportedRealtimeProvider { requested } => format(
            locale,
            keys::REALTIME_UNSUPPORTED_FRONTEND,
            &[("name", requested.as_str())],
        ),
        E::UnsupportedBackend { protocol } => format(
            locale,
            keys::BACKEND_UNSUPPORTED_AGENT,
            &[("id", protocol.as_str())],
        ),
        E::UnsupportedBackendOwnership { requested } => format(
            locale,
            keys::BACKEND_UNSUPPORTED_OWNERSHIP,
            &[("value", requested.as_str())],
        ),
        E::ExternalServiceUnsupported { label } => format(
            locale,
            keys::BACKEND_EXTERNAL_SERVICE_UNSUPPORTED,
            &[("label", label)],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_exit_codes_are_zero_and_one() {
        assert_eq!(EXIT_SUCCESS, 0);
        assert_eq!(EXIT_FAILURE, 1);
    }

    #[test]
    fn a_core_setup_refusal_keeps_its_own_rendered_sentence() {
        let error = CliError::Core(CoreError::GatewaySetupRequired {
            missing: Vec::new(),
            message: "已经渲染过的句子".to_owned(),
        });
        assert_eq!(error.code(), via_protocol::CODE_GATEWAY_SETUP_REQUIRED);
        // The gate rendered it; no locale re-renders it.
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            assert_eq!(error.message(locale), "已经渲染过的句子");
        }
    }

    #[test]
    fn every_catalog_variant_localizes_without_a_placeholder_hole() {
        use via_catalog::CatalogError as E;
        let cases = [
            E::UnknownRealtimeModel {
                model: "m".to_owned(),
            },
            E::UnsupportedRealtimeProvider {
                requested: "p".to_owned(),
            },
            E::UnsupportedBackend {
                protocol: "b".to_owned(),
            },
            E::UnsupportedBackendOwnership {
                requested: "o".to_owned(),
            },
            E::ExternalServiceUnsupported { label: "L" },
        ];
        for case in cases {
            for locale in [Locale::En, Locale::Zh, Locale::Ko] {
                let rendered = localize_catalog(locale, &case);
                assert!(
                    !rendered.contains("<via-i18n:"),
                    "{case:?} in {locale} rendered a diagnostic: {rendered}"
                );
                assert!(!rendered.contains('{'), "{case:?} left a hole: {rendered}");
            }
        }
    }

    #[test]
    fn an_uncatalogued_core_variant_falls_back_to_its_display() {
        let error = CoreError::AcpArgsNotJson;
        assert_eq!(localize_core(Locale::Zh, &error), error.to_string());
    }

    #[test]
    fn an_io_failure_names_the_operation_and_the_path() {
        let error = CliError::io(
            "read",
            "/nowhere/config.env",
            io::Error::from(io::ErrorKind::NotFound),
        );
        assert_eq!(error.code(), CODE_IO);
        let message = error.message(Locale::En);
        assert!(
            message.starts_with("read /nowhere/config.env: "),
            "{message}"
        );
    }
}
