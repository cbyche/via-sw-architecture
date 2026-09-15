//! Choosing a transport for an ACP connection.
//!
//! Ported from `server/src/agent/acp-client-factory.mjs`, all 22 lines of it.
//!
//! The file is small and load-bearing: it is the one place that decides what
//! kind of ACP connection a profile describes, and it **refuses** anything it
//! does not implement rather than defaulting. A factory that silently fell back
//! to the process transport would turn a typo in configuration into a spawned
//! process with the wrong command.
//!
//! Today there is exactly one kind. `ipc` — an agent already running, reached
//! over a socket — is a plugin shape in `docs/architecture.md` §6, not
//! something this crate grows a branch for.

use crate::client::{AcpProcessClient, AcpProcessClientBuilder};
use crate::error::{AcpError, Result};
use crate::process::SpawnSpec;

/// The one connection kind this crate implements.
///
/// **External contract** — `ACP_CONNECTION_PROCESS`,
/// `acp-client-factory.mjs:4`. It appears in configuration and in the refusal
/// message, so it is a wire value rather than an internal name.
pub const ACP_CONNECTION_PROCESS: &str = "process";

/// How to reach an ACP agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpConnection {
    /// A child process speaking NDJSON on stdio.
    Process(SpawnSpec),
}

impl AcpConnection {
    /// The wire name of this kind.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Process(_) => ACP_CONNECTION_PROCESS,
        }
    }
}

/// Build a client for `connection`.
///
/// Typed rather than string-dispatched, so the "unsupported kind" branch is
/// only reachable through [`create_acp_client_by_kind`], which is what a
/// configuration value actually goes through.
#[must_use]
pub fn create_acp_client(
    connection: AcpConnection,
    builder: AcpProcessClientBuilder,
) -> AcpProcessClient {
    match connection {
        AcpConnection::Process(spec) => builder.spawn_spec(spec).build(),
    }
}

/// Build a client for a connection named by a configuration string.
///
/// **Contract** — `createAcpClient`, `acp-client-factory.mjs:6-22`: the kind is
/// trimmed and lower-cased before comparison, and anything else raises
/// `不支持的 ACP 连接方式：<kind>` with an unset kind rendering as the
/// localized "not configured" token.
///
/// # Errors
///
/// [`AcpError::UnsupportedConnection`] for any kind but
/// [`ACP_CONNECTION_PROCESS`].
pub fn create_acp_client_by_kind(
    kind: &str,
    spec: SpawnSpec,
    builder: AcpProcessClientBuilder,
) -> Result<AcpProcessClient> {
    let normalized = kind.trim().to_lowercase();
    if normalized == ACP_CONNECTION_PROCESS {
        return Ok(create_acp_client(AcpConnection::Process(spec), builder));
    }
    Err(AcpError::UnsupportedConnection { kind: normalized })
}

#[cfg(test)]
mod tests {
    use via_i18n::Locale;

    use super::*;

    fn spec() -> SpawnSpec {
        SpawnSpec::new("example-agent")
            .args(["--acp"])
            .cwd("/workspace")
    }

    #[test]
    fn creates_the_local_stdio_client_from_a_process_connection() {
        // Upstream: server/test/acp-client-factory.test.mjs:9-29.
        let client = create_acp_client_by_kind(
            ACP_CONNECTION_PROCESS,
            spec(),
            AcpProcessClient::builder().label("Example"),
        )
        .expect("the process kind is implemented");
        assert_eq!(client.label(), "Example");
        assert_eq!(client.spawn_spec().command, "example-agent");
        assert_eq!(client.spawn_spec().args, ["--acp"]);
        assert_eq!(
            client.spawn_spec().cwd.as_deref(),
            Some(std::path::Path::new("/workspace"))
        );
    }

    #[test]
    fn the_kind_is_trimmed_and_lowercased_before_it_is_matched() {
        assert!(
            create_acp_client_by_kind(
                "  PROCESS  ",
                spec(),
                AcpProcessClient::builder().label("Example")
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_unknown_connection_kinds_at_the_factory_boundary() {
        // Upstream: server/test/acp-client-factory.test.mjs:31-39.
        let error = create_acp_client_by_kind(
            "WebSocket",
            spec(),
            AcpProcessClient::builder().label("Remote Example"),
        )
        .expect_err("no other kind is implemented");
        assert_eq!(
            error,
            AcpError::UnsupportedConnection {
                kind: "websocket".to_owned()
            }
        );
        assert_eq!(
            error.message(Locale::Zh),
            "不支持的 ACP 连接方式：websocket"
        );
    }

    #[test]
    fn an_unset_kind_reports_itself_as_not_configured() {
        let error =
            create_acp_client_by_kind("   ", spec(), AcpProcessClient::builder().label("x"))
                .expect_err("the empty kind is not a kind");
        assert_eq!(error.message(Locale::Zh), "不支持的 ACP 连接方式：未配置");
        assert_eq!(
            error.message(Locale::En),
            "unsupported ACP connection kind: not configured"
        );
    }

    #[test]
    fn the_typed_connection_reports_its_own_wire_name() {
        assert_eq!(
            AcpConnection::Process(spec()).kind(),
            ACP_CONNECTION_PROCESS
        );
    }
}
