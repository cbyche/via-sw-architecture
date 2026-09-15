//! Catalog lookup failures.
//!
//! Upstream throws plain `Error`s carrying interpolated Chinese sentences. Per
//! [`docs/fidelity.md`](../../../docs/fidelity.md) ("Ad-hoc `{ ok, error, code }`
//! returns → `thiserror` enums with a `code()` accessor") the *message text* is
//! `via-i18n`'s to own; what lives here is the machine-readable discriminant plus
//! the values a message would interpolate.
//!
//! The `Display` impls are English developer text, not the user-facing string.

/// Something was asked of a static table that the table does not contain.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CatalogError {
    /// A realtime model id that is not in the DashScope catalog.
    ///
    /// **This is deliberately an error rather than a fallback profile.** Upstream's
    /// `resolveDashScopeRealtimeModelProfile()` returns an all-capabilities-false
    /// profile here, and `server/src/voice/providers/dashscope.mjs:84-87` then gates
    /// `session.turn_detection` on `transportCapabilities.audioInput` — so an
    /// unrecognised id opens a session that connects and never hears anything.
    /// See `docs/architecture.md` §7 "One bug not to port".
    #[error("unknown DashScope realtime model id: {model}")]
    UnknownRealtimeModel {
        /// The id as requested, trimmed.
        model: String,
    },

    /// A realtime provider key or alias that is not registered.
    ///
    /// Upstream message shape (`shared/realtime-provider-catalog.mjs:71-74`):
    /// `不支持的 Realtime 前台：{requested}（可选 {names}）`.
    #[error("unsupported realtime provider: {requested}")]
    UnsupportedRealtimeProvider {
        /// The value as requested, trimmed and lowercased.
        requested: String,
    },

    /// A backend agent id that is not in the backend catalog.
    ///
    /// Upstream message shape (`shared/backend-catalog.mjs:448`):
    /// `不支持的后台 Agent：{protocol}`.
    #[error("unsupported backend agent: {protocol}")]
    UnsupportedBackend {
        /// The value as requested, normalised.
        protocol: String,
    },

    /// A backend process-ownership value other than `owned` or `external`.
    ///
    /// Upstream message shape (`shared/backend-catalog.mjs:450-452`):
    /// `不支持的后台进程归属：{requested}`.
    #[error("unsupported backend process ownership: {requested}")]
    UnsupportedBackendOwnership {
        /// The value as requested, trimmed and lowercased.
        requested: String,
    },

    /// `external` ownership was requested for a backend that cannot be reached as
    /// an already-running service.
    ///
    /// Upstream message shape (`shared/backend-catalog.mjs:454`):
    /// `{label} 不支持连接外部后台服务`.
    #[error("{label} does not support connecting to an external backend service")]
    ExternalServiceUnsupported {
        /// The backend's catalog label, which the message interpolates.
        label: &'static str,
    },
}

impl CatalogError {
    /// A stable machine-readable code for this failure.
    ///
    /// These codes are VIA's own — upstream throws untyped `Error`s for all five
    /// cases. They are stable API: log sinks and the HTTP layer branch on them.
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownRealtimeModel { .. } => "VIA_REALTIME_MODEL_UNKNOWN",
            Self::UnsupportedRealtimeProvider { .. } => "VIA_REALTIME_PROVIDER_UNSUPPORTED",
            Self::UnsupportedBackend { .. } => "VIA_BACKEND_UNSUPPORTED",
            Self::UnsupportedBackendOwnership { .. } => "VIA_BACKEND_OWNERSHIP_UNSUPPORTED",
            Self::ExternalServiceUnsupported { .. } => "VIA_BACKEND_EXTERNAL_SERVICE_UNSUPPORTED",
        }
    }
}
