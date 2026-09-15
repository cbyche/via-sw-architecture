//! What can go wrong outside a tool call.
//!
//! Deliberately small. Everything a *tool* can fail with is a
//! [`HarnessError`](via_downstream::HarnessError) and reaches the model as the
//! error envelope; everything a *request* can fail with is an HTTP status and
//! a JSON-RPC body. What is left is the handful of ways the loopback listener
//! itself refuses to exist, which reach an operator through the Gateway's
//! start-up path.
//!
//! Upstream has no type here at all: `server.once('error', reject)`
//! (`server/src/agent/acp-session-tools.mjs:162`) rejects with whatever Node
//! threw. These variants are VIA's, and their `Display` text is
//! developer-facing English rather than a `via-i18n` key for the same reason
//! `via_lock::LeaseError::Io`'s is — an operator reads it beside the
//! `std::io::Error` it wraps, and there is no upstream sentence to reproduce.

use std::io;
use std::net::SocketAddr;

use thiserror::Error;

/// A failure to run the loopback session-tool server.
#[derive(Debug, Error)]
pub enum McpToolsError {
    /// The loopback listener could not be bound.
    ///
    /// The address is always `127.0.0.1:0`, so in practice this is a sandbox
    /// with no loopback interface or an exhausted file-descriptor table — not
    /// a port conflict, because the port is chosen by the kernel.
    #[error("could not bind the session tool server to {address}")]
    Listen {
        /// The address that was asked for.
        address: SocketAddr,
        /// The underlying error.
        #[source]
        source: io::Error,
    },

    /// The listener bound but would not say where.
    ///
    /// Fatal rather than recoverable: without the port there is no URL to put
    /// on the descriptor, and a descriptor without a URL is a backend that
    /// silently has no coordination tools.
    #[error("the session tool server bound but could not report its address")]
    LocalAddress {
        /// The underlying error.
        #[source]
        source: io::Error,
    },
}

impl McpToolsError {
    /// A stable machine-readable code.
    ///
    /// VIA's own — upstream throws untyped errors for both of these.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Listen { .. } => "VIA_MCP_LISTEN_FAILED",
            Self::LocalAddress { .. } => "VIA_MCP_ADDRESS_UNAVAILABLE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_has_its_own_code() {
        let listen = McpToolsError::Listen {
            address: "127.0.0.1:0".parse().expect("a loopback address"),
            source: io::Error::from(io::ErrorKind::PermissionDenied),
        };
        let address = McpToolsError::LocalAddress {
            source: io::Error::from(io::ErrorKind::Other),
        };
        assert_eq!(listen.code(), "VIA_MCP_LISTEN_FAILED");
        assert_eq!(address.code(), "VIA_MCP_ADDRESS_UNAVAILABLE");
        assert_ne!(listen.code(), address.code());
        assert!(listen.to_string().contains("127.0.0.1:0"));
    }
}
