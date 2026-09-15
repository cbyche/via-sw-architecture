//! `via mcp-serve` — the stdio MCP server.
//!
//! `docs/architecture.md` §12: *"Transport is the loopback HTTP … with `via
//! mcp-serve` over stdio for backends that need it."* The five coordination
//! tools it serves are `via-mcp-tools`, which is phase 3, so the body is.
//!
//! The argument surface is one flag — the Gateway the tools act against —
//! and it is validated here, because a stdio server whose peer address is
//! wrong has no channel to say so on: stdout belongs to the MCP framing.

use crate::cli::McpServeArgs;
use crate::error::CliError;
use crate::host::Host;
use crate::origin::{UrlLabel, clean_origin};
use crate::phase::{Phase, Unimplemented};

/// Run `via mcp-serve`.
///
/// # Errors
///
/// [`CliError::Refused`] for a URL that is not `http(s)`, otherwise
/// [`CliError::Unimplemented`] naming phase 3.
pub fn run(args: &McpServeArgs, host: &Host) -> Result<(), CliError> {
    let _url = clean_origin(&args.url, UrlLabel::Gateway, host.locale())?;
    Err(CliError::Unimplemented(Unimplemented::new(
        "mcp-serve",
        Phase::WorkQueue,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{CODE_INVALID_ARGUMENT, CODE_NOT_IMPLEMENTED};
    use crate::origin::DEFAULT_GATEWAY_URL;
    use via_i18n::Locale;

    fn host() -> Host {
        Host::new(
            via_core::EnvMap::new(),
            "/home/via".into(),
            "/srv/via".into(),
        )
    }

    #[test]
    fn a_valid_invocation_refuses_naming_phase_three() {
        let error = run(
            &McpServeArgs {
                url: DEFAULT_GATEWAY_URL.to_owned(),
            },
            &host(),
        )
        .expect_err("the body lands in phase 3");
        assert_eq!(error.code(), CODE_NOT_IMPLEMENTED);
        let message = error.message(Locale::En);
        assert!(message.contains("mcp-serve"), "{message}");
        assert!(message.contains('3'), "{message}");
    }

    #[test]
    fn a_bad_url_is_refused_before_the_phase_check() {
        let error = run(
            &McpServeArgs {
                url: "stdio://".to_owned(),
            },
            &host(),
        )
        .expect_err("stdio is the transport, not the peer address");
        assert_eq!(error.code(), CODE_INVALID_ARGUMENT);
    }
}
