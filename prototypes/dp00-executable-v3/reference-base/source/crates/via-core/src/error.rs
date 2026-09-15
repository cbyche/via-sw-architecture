//! Core's coded failures.
//!
//! Upstream throws plain `Error`s carrying interpolated Chinese sentences.
//! Per `docs/fidelity.md` ("Ad-hoc `{ ok, error, code }` returns → `thiserror`
//! enums with a `code()` accessor") what lives here is the machine-readable
//! discriminant plus the values a message interpolates; the user-facing
//! sentence is `via-i18n`'s.
//!
//! One `Display` string **is** reproduced verbatim:
//! [`AUTH_SECRET_LENGTH_MESSAGE`](crate::identity::AUTH_SECRET_LENGTH_MESSAGE).
//! `docs/reference/contracts.json` records it as "the literal English error
//! string […] thrown at construction time and reaches the operator", and unlike
//! every other message in this crate it was already English upstream.

use std::path::PathBuf;

use via_catalog::CatalogError;
use via_protocol::{MissingSetting, ProtocolError};

/// Anything `via-core` refuses to do.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// The Gateway may not start: required realtime configuration is missing.
    ///
    /// Code: `VIA_GATEWAY_SETUP_REQUIRED`. `message` is already localized;
    /// [`Self::protocol_error`] converts to the structured
    /// [`ProtocolError::SetupRequired`] a client receives.
    #[error("{message}")]
    GatewaySetupRequired {
        /// Every missing setting, in discovery order.
        missing: Vec<MissingSetting>,
        /// The refusal sentence, rendered in the caller's locale.
        message: String,
    },

    /// `VIA_AUTH_SECRET` is shorter than the 32-character minimum.
    #[error("{0}")]
    AuthSecretTooShort(&'static str),

    /// `VIA_BACKEND_PERMISSION_MODE` was neither `native` nor `full`.
    ///
    /// Upstream message shape (`server/src/core/config.mjs:135-137`):
    /// `不支持的后台权限模式：{requested}（可选 native、full）`.
    #[error("unsupported backend permission mode: {requested}")]
    UnsupportedBackendPermissionMode {
        /// The value as requested, trimmed and lowercased.
        requested: String,
    },

    /// `full` permission was requested for a backend the Gateway does not own.
    ///
    /// Upstream message shape (`server/src/process/managed-backend.mjs:61-69`):
    /// `最高权限模式只支持由 Gateway 启动的后台 Agent`.
    #[error("full permission mode requires a Gateway-launched backend agent")]
    FullPermissionRequiresOwnedBackend,

    /// A backend has no `workspaceEnvironment`, so it has no workspace.
    ///
    /// Upstream message shape (`server/src/core/config.mjs:42`):
    /// `后台 {protocol} 没有 workspace 配置`.
    #[error("backend {protocol} has no workspace configuration")]
    BackendWorkspaceMissing {
        /// The backend protocol id as requested.
        protocol: String,
    },

    /// `ACP_ARGS` opened with `[` but is not valid JSON.
    ///
    /// Upstream message shape (`server/src/core/config.mjs:58`):
    /// `ACP_ARGS 不是有效的 JSON 数组`.
    #[error("ACP_ARGS is not valid JSON")]
    AcpArgsNotJson,

    /// `ACP_ARGS` parsed, but is not an array of strings.
    ///
    /// Upstream message shape (`server/src/core/config.mjs:61`):
    /// `ACP_ARGS 必须是字符串组成的 JSON 数组`.
    #[error("ACP_ARGS must be a JSON array of strings")]
    AcpArgsNotStringArray,

    /// `state.env` exists but does not define the secret it is supposed to
    /// hold.
    ///
    /// Upstream message shape (`shared/runtime-environment.mjs:143`):
    /// `自动生成的本地认证配置无效：{path}`.
    #[error("the generated local identity file is invalid: {path}")]
    InvalidGeneratedSecret {
        /// The `state.env` path that failed to yield a secret.
        path: PathBuf,
    },

    /// A catalog lookup failed — unknown backend, provider, model or ownership.
    #[error(transparent)]
    Catalog(#[from] CatalogError),

    /// A filesystem operation failed, with the path it was operating on.
    #[error("{operation} {path}: {source}")]
    Io {
        /// What was being attempted, e.g. `create`.
        operation: &'static str,
        /// The path involved.
        path: PathBuf,
        /// The underlying error.
        source: std::io::Error,
    },
}

impl CoreError {
    /// A stable machine-readable code.
    ///
    /// Only `VIA_GATEWAY_SETUP_REQUIRED` is inherited from upstream (as
    /// `QWAUDIO_GATEWAY_SETUP_REQUIRED`); the rest are VIA's own and carry a
    /// `VIA_CONFIG_` / `VIA_IDENTITY_` prefix so they are never mistaken for a
    /// contract a client already knows.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::GatewaySetupRequired { .. } => via_protocol::CODE_GATEWAY_SETUP_REQUIRED,
            Self::AuthSecretTooShort(_) => "VIA_IDENTITY_SECRET_TOO_SHORT",
            Self::UnsupportedBackendPermissionMode { .. } => {
                "VIA_CONFIG_BACKEND_PERMISSION_MODE_UNSUPPORTED"
            }
            Self::FullPermissionRequiresOwnedBackend => "VIA_CONFIG_FULL_PERMISSION_REQUIRES_OWNED",
            Self::BackendWorkspaceMissing { .. } => "VIA_CONFIG_BACKEND_WORKSPACE_MISSING",
            Self::AcpArgsNotJson => "VIA_CONFIG_ACP_ARGS_NOT_JSON",
            Self::AcpArgsNotStringArray => "VIA_CONFIG_ACP_ARGS_NOT_STRING_ARRAY",
            Self::InvalidGeneratedSecret { .. } => "VIA_IDENTITY_GENERATED_SECRET_INVALID",
            Self::Catalog(error) => error.code(),
            Self::Io { .. } => "VIA_CONFIG_IO",
        }
    }

    /// The structured protocol error a client receives, where one exists.
    ///
    /// Only the setup gate has a client-visible shape; everything else is an
    /// operator-facing startup failure.
    #[must_use]
    pub fn protocol_error(&self) -> Option<ProtocolError> {
        match self {
            Self::GatewaySetupRequired { missing, .. } => Some(ProtocolError::SetupRequired {
                missing: missing.clone(),
            }),
            _ => None,
        }
    }

    /// Wrap an [`std::io::Error`] with the operation and path that produced it.
    #[must_use]
    pub fn io(operation: &'static str, path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            operation,
            path: path.into(),
            source,
        }
    }
}
