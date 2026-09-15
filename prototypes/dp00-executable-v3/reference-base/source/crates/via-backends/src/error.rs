//! Everything `via-backends` refuses to do.
//!
//! Upstream throws plain `Error`s carrying interpolated Chinese sentences
//! (`server/src/agent/backends/registry.mjs:22-63`,
//! `generic-acp.mjs:29-33`, `shared/backend-install.mjs:450-659`). Per
//! `docs/fidelity.md` the machine-readable discriminant lives here and the
//! user-facing sentence comes from `via-i18n` through [`BackendsError::message`].

use via_i18n::{Locale, format, keys, t};

/// A refusal from the named-backend layer.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BackendsError {
    /// The configured id is not one of the twelve catalogued backends.
    ///
    /// `server/src/agent/backends/registry.mjs:58-62` —
    /// `不支持的后台 Agent：<id>`.
    #[error("unsupported backend agent: {protocol}")]
    UnsupportedBackend {
        /// The id as normalised (trimmed, lower-cased, `none` → empty).
        protocol: String,
    },

    /// `AGENT_PROTOCOL=acp` with no `ACP_COMMAND`.
    ///
    /// `server/src/agent/backends/generic-acp.mjs:29-31` —
    /// `使用通用 ACP 后端时必须设置 ACP_COMMAND`.
    #[error("ACP_COMMAND must be set when using the generic ACP backend")]
    GenericAcpRequiresCommand,

    /// `full` permission requested for the generic ACP backend.
    ///
    /// `server/src/agent/backends/generic-acp.mjs:32-34` —
    /// `通用 ACP 后端无法安全地统一开启最高权限模式`. A user-supplied agent has
    /// no declared approval gate, so VIA cannot claim one switch turns it on
    /// safely.
    #[error("the generic ACP backend cannot have full permission mode turned on safely")]
    GenericAcpFullPermissionUnsafe,

    /// `full` permission requested for a backend the catalog says cannot take
    /// it.
    ///
    /// `server/src/process/backend-drivers/registry.mjs:39` —
    /// `<label> 无法安全地统一开启最高权限模式`.
    #[error("{label} cannot have full permission mode turned on safely from one switch")]
    FullPermissionUnsafe {
        /// The catalog label.
        label: String,
    },

    /// `full` permission requested for OpenClaw.
    ///
    /// `server/src/process/backend-drivers/openclaw.mjs:17-20`. OpenClaw's own
    /// full permission is three separate settings inside a third-party product;
    /// the Gateway's single switch cannot stand in for them.
    #[error("OpenClaw's full permission cannot be enabled safely from the Gateway's switch")]
    OpenClawFullPermissionUnsafe,

    /// A backend was asked for with no workspace directory resolved.
    ///
    /// `shared/runtime-environment.mjs:286-303` —
    /// `后台 <protocol> 没有 workspace 配置`.
    #[error("backend {protocol} has no workspace configuration")]
    MissingWorkspace {
        /// The backend id.
        protocol: String,
    },

    /// The declaration this crate handed the seam was refused.
    ///
    /// Every one of upstream's `validateBackendDriver` refusals arrives this
    /// way, because `via-downstream` owns that validation.
    #[error(transparent)]
    Harness(#[from] via_downstream::HarnessError),
}

impl BackendsError {
    /// A stable machine-readable code.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedBackend { .. } => "VIA_BACKEND_UNSUPPORTED",
            Self::GenericAcpRequiresCommand => "VIA_BACKEND_ACP_COMMAND_REQUIRED",
            Self::GenericAcpFullPermissionUnsafe | Self::FullPermissionUnsafe { .. } => {
                "VIA_BACKEND_FULL_PERMISSION_UNSAFE"
            }
            Self::OpenClawFullPermissionUnsafe => "VIA_BACKEND_OPENCLAW_FULL_PERMISSION_UNSAFE",
            Self::MissingWorkspace { .. } => "VIA_BACKEND_WORKSPACE_MISSING",
            Self::Harness(error) => error.code(),
        }
    }

    /// The user-facing sentence, rendered in `locale`.
    #[must_use]
    pub fn message(&self, locale: Locale) -> String {
        match self {
            Self::UnsupportedBackend { protocol } => {
                format(locale, keys::BACKEND_UNSUPPORTED_AGENT, &[("id", protocol)])
            }
            Self::GenericAcpRequiresCommand => {
                t(locale, keys::BACKEND_GENERIC_ACP_REQUIRES_COMMAND).to_owned()
            }
            Self::GenericAcpFullPermissionUnsafe => {
                t(locale, keys::BACKEND_GENERIC_ACP_FULL_PERMISSION_UNSAFE).to_owned()
            }
            Self::FullPermissionUnsafe { label } => format(
                locale,
                keys::BACKEND_FULL_PERMISSION_UNSAFE,
                &[("label", label)],
            ),
            Self::OpenClawFullPermissionUnsafe => {
                t(locale, keys::BACKEND_OPENCLAW_FULL_PERMISSION_UNSAFE).to_owned()
            }
            Self::MissingWorkspace { protocol } => format(
                locale,
                keys::BACKEND_MISSING_WORKSPACE,
                &[("protocol", protocol)],
            ),
            Self::Harness(error) => error.message(locale),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_refusal_renders_upstream_chinese() {
        assert_eq!(
            BackendsError::GenericAcpRequiresCommand.message(Locale::Zh),
            "使用通用 ACP 后端时必须设置 ACP_COMMAND"
        );
        assert_eq!(
            BackendsError::GenericAcpFullPermissionUnsafe.message(Locale::Zh),
            "通用 ACP 后端无法安全地统一开启最高权限模式"
        );
        assert_eq!(
            BackendsError::UnsupportedBackend {
                protocol: "nope".to_owned()
            }
            .message(Locale::Zh),
            "不支持的后台 Agent：nope"
        );
    }

    #[test]
    fn a_harness_refusal_keeps_its_own_code() {
        let error = BackendsError::Harness(via_downstream::HarnessError::NotConfigured);
        assert_eq!(error.code(), "VIA_BACKEND_NOT_CONFIGURED");
    }
}
