//! What can go wrong on the device.
//!
//! # Two audiences, two strings — `via-wake-word`'s split, kept
//!
//! [`LocalError`]'s `Display` is a **diagnostic**: English, specific, aimed at
//! whoever is reading a log or a test failure. It names the path, the endpoint,
//! the stage.
//!
//! [`LocalError::localized`] is the **sentence a person reads**, and it comes
//! from `via-i18n` like every other one in VIA.
//!
//! The split matters most on the one message this crate exists to get right. A
//! refused connection to a local server must say *"no local server is running at
//! ws://127.0.0.1:8000/v1/realtime"* to the user, and *"Connection refused (os
//! error 61)"* to the operator, and neither is a good substitute for the other.
//!
//! # Why a config failure is a `NotConfigured`, not a transport failure
//!
//! `docs/architecture.md` §7's *"one bug not to port"* is the whole reason this
//! module exists in the shape it does. Upstream answers an unknown model id with
//! an all-capabilities-false profile, and `dashscope.mjs:84-87` then gates
//! `session.turn_detection` on `transportCapabilities.audioInput` — so the
//! session opens, connects, and never hears the user.
//!
//! A local model id is by definition not in that table, so on this path the
//! fallback is reached on the *happy* path. VIA's answer has two halves and both
//! are here: `via-catalog`'s [`local_realtime_model_profile`] carries real
//! capability flags, and a model file that is not on disk is
//! [`ModelFileMissing`](LocalError::ModelFileMissing) — raised from
//! [`preflight`], **before a session is opened**, naming the path. Never a
//! session that connects and hears nothing.
//!
//! [`local_realtime_model_profile`]: via_catalog::local_realtime_model_profile
//! [`preflight`]: via_realtime::RealtimeProvider::preflight

use std::path::{Path, PathBuf};

use via_i18n::{Locale, format as i18n_format, keys};
use via_realtime::RealtimeError;

use crate::mode::PROVIDER_KEY;
use crate::stages::{Stage, StageError};

/// The result type this crate's fallible entry points return.
pub type Result<T> = core::result::Result<T, LocalError>;

/// An on-device configuration or runtime failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum LocalError {
    /// The configured mode is neither `pipeline` nor `endpoint`.
    #[error(
        "unknown local mode `{value}` (expected `pipeline` or `endpoint`, \
         optionally prefixed `local-omni:`)"
    )]
    UnknownMode {
        /// What was configured.
        value: String,
    },

    /// `local-omni:endpoint` was selected with no endpoint to dial.
    ///
    /// `via-catalog`'s `local-omni` row declares **no default URL**, and the
    /// comment says why: *"`pipeline` needs none and `endpoint` must be told
    /// where to look."* Falling back to a guess is the mistake ARGO bring-up §5
    /// records — a missing endpoint that silently became `api.openai.com`
    /// arrived as an accusation about the credential, from a host nobody
    /// configured.
    #[error("no local realtime endpoint is configured; set {variable}")]
    EndpointNotConfigured {
        /// The environment variable to set.
        variable: &'static str,
    },

    /// A model file the pipeline needs is not on disk.
    #[error("the on-device {stage} model file is missing: {}", .path.display())]
    ModelFileMissing {
        /// Which stage would have loaded it.
        stage: Stage,
        /// The path that was looked for.
        path: PathBuf,
    },

    /// The pipeline's model root holds nothing at all.
    ///
    /// Distinct from [`ModelFileMissing`](Self::ModelFileMissing) because the
    /// fix is different: one file missing is a broken install, none is an
    /// install that never happened.
    #[error("the on-device pipeline has no model files under {}", .root.display())]
    PipelineNotInstalled {
        /// The directory that was searched.
        root: PathBuf,
    },

    /// Nothing answered at the local endpoint.
    #[error("no local server is running at {endpoint}: {detail}")]
    ServerUnreachable {
        /// The endpoint that was dialled.
        endpoint: String,
        /// What the transport reported.
        detail: String,
    },

    /// The local endpoint refused a TLS handshake.
    ///
    /// Its own variant rather than a [`ServerUnreachable`](Self::ServerUnreachable)
    /// because the fix is a one-character edit — `wss://` to `ws://` — and a TLS
    /// error rendered verbatim ("invalid peer certificate: UnknownIssuer") says
    /// nothing about that.
    #[error("{endpoint} did not complete a TLS handshake: {detail}")]
    TlsRefused {
        /// The endpoint that was dialled.
        endpoint: String,
        /// What the TLS stack reported.
        detail: String,
    },

    /// A stage refused.
    #[error(transparent)]
    Stage(#[from] StageError),
}

impl LocalError {
    /// A missing file for `stage`.
    #[must_use]
    pub fn model_file_missing(stage: Stage, path: impl AsRef<Path>) -> Self {
        Self::ModelFileMissing {
            stage,
            path: path.as_ref().to_path_buf(),
        }
    }

    /// The sentence a person reads.
    ///
    /// Every arm is a `via-i18n` key; there is no user-facing literal in this
    /// crate.
    #[must_use]
    pub fn localized(&self, locale: Locale) -> String {
        match self {
            Self::UnknownMode { value } => i18n_format(
                locale,
                keys::REALTIME_LOCAL_UNKNOWN_MODE,
                &[("value", value.as_str())],
            ),
            Self::EndpointNotConfigured { variable } => i18n_format(
                locale,
                keys::REALTIME_LOCAL_ENDPOINT_MISSING,
                &[("variable", *variable)],
            ),
            Self::ModelFileMissing { path, .. } => i18n_format(
                locale,
                keys::REALTIME_LOCAL_MODEL_FILE_MISSING,
                &[("path", &path.display().to_string())],
            ),
            Self::PipelineNotInstalled { root } => i18n_format(
                locale,
                keys::REALTIME_LOCAL_PIPELINE_NOT_INSTALLED,
                &[("path", &root.display().to_string())],
            ),
            Self::ServerUnreachable { endpoint, .. } => i18n_format(
                locale,
                keys::REALTIME_LOCAL_SERVER_UNREACHABLE,
                &[("endpoint", endpoint.as_str())],
            ),
            Self::TlsRefused { endpoint, .. } => i18n_format(
                locale,
                keys::REALTIME_LOCAL_TLS_REFUSED,
                &[("endpoint", endpoint.as_str())],
            ),
            Self::Stage(stage) => i18n_format(
                locale,
                keys::REALTIME_LOCAL_STAGE_FAILED,
                &[
                    ("stage", stage.stage.as_str()),
                    ("detail", stage.detail.as_str()),
                ],
            ),
        }
    }

    /// Whether this is something the operator has to fix before a session can
    /// open at all.
    ///
    /// The four configuration faults are refused by
    /// [`preflight`](via_realtime::RealtimeProvider::preflight); the two connect
    /// faults and a stage failure are not, because they can only be discovered
    /// by trying.
    #[must_use]
    pub const fn is_configuration_fault(&self) -> bool {
        matches!(
            self,
            Self::UnknownMode { .. }
                | Self::EndpointNotConfigured { .. }
                | Self::ModelFileMissing { .. }
                | Self::PipelineNotInstalled { .. }
        )
    }

    /// The [`RealtimeError`] the Gateway sees.
    ///
    /// A configuration fault becomes [`RealtimeError::NotConfigured`] carrying
    /// the localized sentence — which is what puts the missing path in front of
    /// the person who has to create it. Everything else becomes
    /// [`RealtimeError::Transport`], whose `detail` is what a provider's
    /// `classify_error` reads, so the localized sentence goes there too and the
    /// English diagnostic rides beside it.
    #[must_use]
    pub fn into_realtime(self, locale: Locale) -> RealtimeError {
        let message = self.localized(locale);
        if self.is_configuration_fault() {
            return RealtimeError::NotConfigured {
                provider: PROVIDER_KEY.to_owned(),
                message,
            };
        }
        RealtimeError::Transport {
            detail: format!("{message} ({self})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const LOCALES: [Locale; 3] = [Locale::En, Locale::Zh, Locale::Ko];

    fn every_error() -> Vec<LocalError> {
        vec![
            LocalError::UnknownMode {
                value: "streaming".to_owned(),
            },
            LocalError::EndpointNotConfigured {
                variable: "VIA_REALTIME_BASE_URL",
            },
            LocalError::model_file_missing(Stage::Asr, "/models/local-omni/encoder.onnx"),
            LocalError::PipelineNotInstalled {
                root: PathBuf::from("/models/local-omni"),
            },
            LocalError::ServerUnreachable {
                endpoint: "ws://127.0.0.1:8000/v1/realtime".to_owned(),
                detail: "Connection refused (os error 61)".to_owned(),
            },
            LocalError::TlsRefused {
                endpoint: "wss://127.0.0.1:8000/v1/realtime".to_owned(),
                detail: "invalid peer certificate".to_owned(),
            },
            LocalError::Stage(StageError::new(Stage::Reasoning, "context overflow")),
        ]
    }

    #[test]
    fn every_error_localizes_in_every_locale_with_no_placeholder_left_behind() {
        for error in every_error() {
            for locale in LOCALES {
                let rendered = error.localized(locale);
                assert!(!rendered.is_empty(), "{error:?} in {locale}");
                assert!(!rendered.contains('{'), "{locale}: {rendered}");
            }
        }
    }

    #[test]
    fn the_refusal_sentence_names_the_endpoint_rather_than_the_transport() {
        let error = LocalError::ServerUnreachable {
            endpoint: "ws://127.0.0.1:8000/v1/realtime".to_owned(),
            detail: "Connection refused (os error 61)".to_owned(),
        };
        assert_eq!(
            error.localized(Locale::En),
            "no local server is running at ws://127.0.0.1:8000/v1/realtime"
        );
        // The operator's half keeps the detail the user's half drops.
        assert!(error.to_string().contains("os error 61"));
        for locale in LOCALES {
            assert!(
                error.localized(locale).contains("ws://127.0.0.1:8000"),
                "{locale}"
            );
        }
    }

    #[test]
    fn a_tls_failure_is_its_own_sentence_because_the_fix_is_the_scheme() {
        let error = LocalError::TlsRefused {
            endpoint: "wss://127.0.0.1:8000/v1/realtime".to_owned(),
            detail: "invalid peer certificate: UnknownIssuer".to_owned(),
        };
        assert!(error.localized(Locale::En).contains("ws://"));
        assert_ne!(
            error.localized(Locale::En),
            LocalError::ServerUnreachable {
                endpoint: "wss://127.0.0.1:8000/v1/realtime".to_owned(),
                detail: "invalid peer certificate: UnknownIssuer".to_owned(),
            }
            .localized(Locale::En)
        );
    }

    #[test]
    fn a_missing_model_file_names_the_path_in_every_locale() {
        let error = LocalError::model_file_missing(Stage::Tts, "/models/local-omni/kokoro.onnx");
        for locale in LOCALES {
            assert!(
                error
                    .localized(locale)
                    .contains("/models/local-omni/kokoro.onnx"),
                "{locale}: {}",
                error.localized(locale)
            );
        }
        assert!(error.to_string().contains("tts"));
    }

    #[test]
    fn only_the_four_configuration_faults_are_refused_before_a_session_opens() {
        let faults: Vec<bool> = every_error()
            .iter()
            .map(LocalError::is_configuration_fault)
            .collect();
        assert_eq!(faults, [true, true, true, true, false, false, false]);
    }

    #[test]
    fn a_configuration_fault_reaches_the_gateway_as_not_configured() {
        let error = LocalError::model_file_missing(Stage::Vad, "/models/local-omni/silero.onnx")
            .into_realtime(Locale::En);
        assert_eq!(
            error,
            RealtimeError::NotConfigured {
                provider: "local-omni".to_owned(),
                message: "the on-device model file is missing: /models/local-omni/silero.onnx"
                    .to_owned(),
            }
        );
    }

    #[test]
    fn a_connect_fault_reaches_the_gateway_as_a_transport_failure_carrying_both_halves() {
        let error = LocalError::ServerUnreachable {
            endpoint: "ws://127.0.0.1:8000".to_owned(),
            detail: "Connection refused".to_owned(),
        }
        .into_realtime(Locale::En);
        let RealtimeError::Transport { detail } = &error else {
            panic!("expected a transport failure, got {error:?}");
        };
        assert!(detail.starts_with("no local server is running at ws://127.0.0.1:8000"));
        assert!(detail.contains("Connection refused"));
        assert_eq!(error.code(), "VIA_REALTIME_TRANSPORT");
    }

    #[test]
    fn a_stage_failure_carries_the_stage_into_the_sentence() {
        let error = LocalError::from(StageError::new(Stage::Asr, "decoder overflow"));
        let rendered = error.localized(Locale::En);
        assert_eq!(rendered, "the on-device asr stage failed: decoder overflow");
        assert!(!error.is_configuration_fault());
    }
}
