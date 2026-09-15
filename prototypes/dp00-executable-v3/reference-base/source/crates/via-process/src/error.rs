//! Everything `via-process` refuses to do.
//!
//! Upstream throws plain `Error`s carrying interpolated Chinese sentences. Per
//! `docs/fidelity.md` ("Ad-hoc `{ ok, error, code }` returns → `thiserror`
//! enums with a `code()` accessor") what lives here is the machine-readable
//! discriminant plus the values a message interpolates; the user-facing
//! sentence comes from `via-i18n` through [`ProcessError::message`].
//!
//! Every variant below is one of the refusals catalogued as
//! `error-code/runtime-ownership errors` and `error-code/driver validation
//! errors` in `docs/reference/contracts.json`. They are security-relevant —
//! `Gateway 只能启动本机后台 Agent` is what stops the Gateway launching a
//! process on someone else's machine, and
//! `后台服务地址不能包含用户名或密码` is what stops a credential reaching the
//! process table and the log — so each stays a hard error rather than a
//! warning.

use via_i18n::{Key, Locale, format, keys, t};

/// A refusal from the managed-process layer.
///
/// Ported from `server/src/process/managed-backend.mjs:14-69,205-206`,
/// `server/src/process/backend-drivers/shared.mjs:1-48` and
/// `server/src/process/backend-drivers/registry.mjs:11-62`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ProcessError {
    /// `VIA_BACKEND_PERMISSION_MODE` was neither `native` nor `full`.
    ///
    /// `managed-backend.mjs:14-22`. This is the *second* copy of a check
    /// `via-core`'s resolver also performs; `docs/reference/contracts.json`
    /// (`env-var/QWEN_AUDIO_AGENT_BACKEND_PERMISSION_MODE`) records that both
    /// copies exist and must both be reproduced.
    #[error("unsupported backend permission mode: {requested}")]
    UnsupportedPermissionMode {
        /// The value as requested, lowercased but **not** trimmed — upstream
        /// does not trim here.
        requested: String,
    },

    /// `VIA_BACKEND_OWNERSHIP` was neither empty, `owned` nor `external`.
    ///
    /// `managed-backend.mjs:24-31`.
    #[error("unsupported backend process ownership: {requested}")]
    UnsupportedOwnership {
        /// The value as requested, trimmed and lowercased.
        requested: String,
    },

    /// `external` ownership was requested for a driver that cannot connect to
    /// an already-running service.
    ///
    /// `managed-backend.mjs:32-34`. Upstream interpolates the driver **id**
    /// here, where `shared/backend-catalog.mjs:454` interpolates the catalog
    /// **label** for the same sentence — a real, observable difference between
    /// the two call sites, reproduced rather than harmonised.
    #[error("{label} does not support connecting to an external backend service")]
    ExternalServiceUnsupported {
        /// What the message interpolates: the driver id at this call site.
        label: String,
    },

    /// `full` permission was requested for a backend the Gateway does not own.
    ///
    /// `managed-backend.mjs:52-57,61-69`. Raised from two places, exactly as
    /// upstream does.
    #[error("full permission mode requires a Gateway-launched backend agent")]
    FullPermissionRequiresOwned,

    /// A driver with no service address of its own was asked to run as an
    /// external service.
    ///
    /// `backend-drivers/shared.mjs:41-48` (`managedOnlyBackend`).
    #[error("the {label} backend must be started by the Gateway")]
    MustBeGatewayStarted {
        /// The driver's catalog label.
        label: String,
    },

    /// The Gateway was asked to launch a backend that is not on this machine.
    ///
    /// `managed-backend.mjs:205-206`. The check is on the WHATWG `hostname`,
    /// so `http://localhost`, `http://127.0.0.1` and `http://[::1]` pass and
    /// nothing else does.
    #[error("the Gateway can only start a backend agent on this machine: {url}")]
    LocalOnly {
        /// The base URL that was refused.
        url: String,
    },

    /// A service address used a URL scheme the driver does not admit.
    ///
    /// `backend-drivers/shared.mjs:1-9`. The trailing colon is part of the
    /// value (`new URL(x).protocol`), so the message reads
    /// `…地址协议：wss:`.
    #[error("unsupported backend service address protocol: {protocol}")]
    UnsupportedServiceProtocol {
        /// The scheme, with its trailing colon.
        protocol: String,
    },

    /// A service address carried a username or password.
    ///
    /// `backend-drivers/shared.mjs:10-12`. Credentials in a URL reach the
    /// process table and every log line that echoes the address, so this is a
    /// refusal and not a redaction.
    #[error("the backend service address must not contain a username or password")]
    ServiceUrlHasCredentials,

    /// The Gateway's single permission switch cannot safely turn this backend
    /// into a full-permission one.
    ///
    /// `backend-drivers/registry.mjs:39`.
    #[error("{label} cannot have full permission mode turned on safely from one switch")]
    FullPermissionUnsafe {
        /// The driver's catalog label.
        label: String,
    },

    /// A driver refused for a reason only that driver knows, with a message it
    /// supplied as a `via-i18n` key.
    ///
    /// The escape hatch for `backend-drivers/openclaw.mjs:17-20`, whose
    /// refusal names three of its own configuration sections. The *key* is
    /// carried rather than the rendered sentence, so the refusal is still
    /// localized at the point of display.
    #[error("the driver refused: {}", key.as_str())]
    DriverRefused {
        /// The catalog key naming the refusal.
        key: Key,
        /// Placeholder bindings for that key.
        arguments: Vec<(String, String)>,
    },

    /// A base URL, or a value configured as one, is not a URL.
    ///
    /// Upstream reaches `new URL(value)` and lets a `TypeError` escape
    /// (`backend-drivers/shared.mjs:3`); VIA refuses with a coded error
    /// instead, because an operator typo in `OPENCODE_BASE_URL` should not
    /// read as a crash.
    #[error("invalid backend service address: {value}")]
    InvalidServiceUrl {
        /// The value as configured.
        value: String,
    },

    /// A driver id that is not in the backend catalog.
    ///
    /// `backend-drivers/registry.mjs:56-62`.
    #[error("unsupported backend agent: {protocol}")]
    UnsupportedBackend {
        /// The id as requested, normalised.
        protocol: String,
    },

    /// A registered driver's id does not match the catalog entry it claims.
    ///
    /// `backend-drivers/registry.mjs:12-14`.
    #[error("backend runtime driver id does not match: {id}")]
    DriverIdMismatch {
        /// The offending driver id.
        id: String,
    },

    /// A driver's `supports_external_service` disagrees with the catalog.
    ///
    /// `backend-drivers/registry.mjs:21-26`.
    #[error("backend runtime driver external-service capability does not match: {id}")]
    DriverExternalServiceMismatch {
        /// The offending driver id.
        id: String,
    },

    /// A driver that declares a separate managed process supplied no launch
    /// command.
    ///
    /// `backend-drivers/registry.mjs:27-29`. Upstream's field is
    /// `managedScript`, a `scripts/*.mjs` file name; VIA's is a
    /// [`ManagedLaunch`](crate::driver::ManagedLaunch), because the Node shims
    /// do not survive the port (`docs/architecture.md` §10). The message is
    /// upstream's, unchanged.
    #[error("backend runtime driver has no managed launch command: {id}")]
    DriverMissingManagedLaunch {
        /// The offending driver id.
        id: String,
    },

    /// A launch command could not be found on the child's composed `PATH`.
    ///
    /// VIA-owned: upstream spawns `process.execPath`, which by construction
    /// exists, so it has no counterpart. Reported as
    /// [`BackendStatusCode::NotInstalled`](crate::BackendStatusCode) by
    /// [`backend_failure_code`](crate::backend_failure_code), which is the
    /// same answer upstream reaches from the child's own `ENOENT`.
    #[error("could not find the backend launch command `{command}` on PATH")]
    CommandNotFound {
        /// The command as configured.
        command: String,
    },

    /// A filesystem, socket or spawn operation failed.
    #[error("{operation}: {source}")]
    Io {
        /// What was being attempted, e.g. `spawn` or `allocate address`.
        operation: &'static str,
        /// The underlying error.
        source: std::io::Error,
    },
}

impl ProcessError {
    /// A stable machine-readable code.
    ///
    /// VIA-owned: upstream throws untyped `Error`s for every case here. Log
    /// sinks and the HTTP layer branch on these, so they are API.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedPermissionMode { .. } => "VIA_PROCESS_PERMISSION_MODE_UNSUPPORTED",
            Self::UnsupportedOwnership { .. } => "VIA_PROCESS_OWNERSHIP_UNSUPPORTED",
            Self::ExternalServiceUnsupported { .. } => "VIA_PROCESS_EXTERNAL_SERVICE_UNSUPPORTED",
            Self::FullPermissionRequiresOwned => "VIA_PROCESS_FULL_PERMISSION_REQUIRES_OWNED",
            Self::MustBeGatewayStarted { .. } => "VIA_PROCESS_MUST_BE_GATEWAY_STARTED",
            Self::LocalOnly { .. } => "VIA_PROCESS_LOCAL_ONLY",
            Self::UnsupportedServiceProtocol { .. } => "VIA_PROCESS_SERVICE_PROTOCOL_UNSUPPORTED",
            Self::ServiceUrlHasCredentials => "VIA_PROCESS_SERVICE_URL_HAS_CREDENTIALS",
            Self::FullPermissionUnsafe { .. } | Self::DriverRefused { .. } => {
                "VIA_PROCESS_FULL_PERMISSION_UNSAFE"
            }
            Self::InvalidServiceUrl { .. } => "VIA_PROCESS_SERVICE_URL_INVALID",
            Self::UnsupportedBackend { .. } => "VIA_PROCESS_BACKEND_UNSUPPORTED",
            Self::DriverIdMismatch { .. } => "VIA_PROCESS_DRIVER_ID_MISMATCH",
            Self::DriverExternalServiceMismatch { .. } => "VIA_PROCESS_DRIVER_EXTERNAL_MISMATCH",
            Self::DriverMissingManagedLaunch { .. } => "VIA_PROCESS_DRIVER_MISSING_LAUNCH",
            Self::CommandNotFound { .. } => "VIA_PROCESS_COMMAND_NOT_FOUND",
            Self::Io { .. } => "VIA_PROCESS_IO",
        }
    }

    /// The user-facing sentence, rendered in `locale`.
    ///
    /// Every string a person reads comes from `via-i18n`; nothing in this
    /// crate interpolates a literal. `zh` is upstream's own text, so the
    /// catalogued `error-code` contracts are assertable against a VIA build.
    #[must_use]
    pub fn message(&self, locale: Locale) -> String {
        match self {
            Self::UnsupportedPermissionMode { requested } => format(
                locale,
                keys::BACKEND_UNSUPPORTED_PERMISSION_MODE,
                &[("mode", requested)],
            ),
            Self::UnsupportedOwnership { requested } => format(
                locale,
                keys::BACKEND_UNSUPPORTED_OWNERSHIP,
                &[("value", requested)],
            ),
            Self::ExternalServiceUnsupported { label } => format(
                locale,
                keys::BACKEND_EXTERNAL_SERVICE_UNSUPPORTED,
                &[("label", label)],
            ),
            Self::FullPermissionRequiresOwned => {
                t(locale, keys::BACKEND_FULL_PERMISSION_REQUIRES_OWNED).to_owned()
            }
            Self::MustBeGatewayStarted { label } => format(
                locale,
                keys::BACKEND_MUST_BE_GATEWAY_STARTED,
                &[("label", label)],
            ),
            Self::LocalOnly { url } => format(locale, keys::BACKEND_LOCAL_ONLY, &[("url", url)]),
            Self::UnsupportedServiceProtocol { protocol } => format(
                locale,
                keys::BACKEND_UNSUPPORTED_SERVICE_PROTOCOL,
                &[("protocol", protocol)],
            ),
            Self::ServiceUrlHasCredentials => {
                t(locale, keys::BACKEND_SERVICE_URL_HAS_CREDENTIALS).to_owned()
            }
            Self::FullPermissionUnsafe { label } => format(
                locale,
                keys::BACKEND_FULL_PERMISSION_UNSAFE,
                &[("label", label)],
            ),
            Self::DriverRefused { key, arguments } => {
                let bound: Vec<(&str, &str)> = arguments
                    .iter()
                    .map(|(name, value)| (name.as_str(), value.as_str()))
                    .collect();
                format(locale, *key, &bound)
            }
            Self::InvalidServiceUrl { value } => format(
                locale,
                keys::CLI_INVALID_URL,
                &[
                    ("label", t(locale, keys::CLI_LABEL_BACKEND_URL)),
                    ("value", value),
                ],
            ),
            Self::UnsupportedBackend { protocol } => {
                format(locale, keys::BACKEND_UNSUPPORTED_AGENT, &[("id", protocol)])
            }
            Self::DriverIdMismatch { id } => format(
                locale,
                keys::BACKEND_RUNTIME_DRIVER_ID_MISMATCH,
                &[("id", id)],
            ),
            Self::DriverExternalServiceMismatch { id } => format(
                locale,
                keys::BACKEND_RUNTIME_DRIVER_EXTERNAL_MISMATCH,
                &[("id", id)],
            ),
            Self::DriverMissingManagedLaunch { id } => format(
                locale,
                keys::BACKEND_RUNTIME_DRIVER_MISSING_MANAGED_SCRIPT,
                &[("id", id)],
            ),
            Self::CommandNotFound { command } => format(
                locale,
                keys::SETUP_ADAPTER_UNAVAILABLE,
                &[("command", command.as_str())],
            ),
            Self::Io { operation, source } => format(
                locale,
                keys::ACP_PROCESS_SPAWN_FAILED,
                &[("detail", &format!("{operation}: {source}"))],
            ),
        }
    }

    /// Wrap an [`std::io::Error`] with the operation that produced it.
    #[must_use]
    pub fn io(operation: &'static str, source: std::io::Error) -> Self {
        Self::Io { operation, source }
    }
}
