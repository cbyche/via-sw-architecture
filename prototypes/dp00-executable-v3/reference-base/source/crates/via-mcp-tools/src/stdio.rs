//! The stdio transport behind `via mcp-serve`.
//!
//! **Not the default.** `docs/architecture.md` §12 says the transport is
//! *"qwen's loopback HTTP […] with `via mcp-serve` over stdio for backends
//! that need it"*, and §10 lists the command. Loopback HTTP stays the default
//! because it is what upstream does and because it needs no extra process: the
//! Gateway already has the context in hand and hands the backend a URL.
//!
//! Stdio exists because ACP requires every agent to support it as the baseline
//! MCP transport, so a backend that cannot open an HTTP MCP server — one whose
//! [`external_mcp`](via_downstream::BackendCapabilities::external_mcp) is true
//! but whose HTTP support is not — can still be given the coordination tools
//! by being pointed at `via mcp-serve` as a `command`/`args` descriptor.
//!
//! # The framing
//!
//! MCP's stdio transport is newline-delimited JSON: one JSON-RPC message per
//! line, no embedded newlines, UTF-8. There is no authentication, because
//! there is no network — the peer is the process that spawned this one, and
//! the operating system's process boundary is the credential. That is the
//! whole reason the loopback transport needs a bearer token and this one does
//! not.

use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use via_i18n::Locale;

use crate::context::SessionToolContext;
use crate::protocol::{codes, dispatch, error_response};

/// Serve the coordination tools over one pair of byte streams until `input`
/// ends.
///
/// Messages are handled **one at a time, in arrival order**: a stdio peer has
/// one channel and interleaving replies on it would break the correlation the
/// peer is relying on. That is the same reason
/// [`crate::registry`] is a task rather than a lock.
///
/// A line that is not JSON is answered with a JSON-RPC parse error rather than
/// closing the stream, because a stdio peer has no other channel to be told on.
///
/// # Errors
///
/// Any write failure on `output`. A read failure ends the loop quietly: the
/// peer closed, which is how a stdio server is meant to stop.
pub async fn serve_stdio<R, W>(
    context: Arc<dyn SessionToolContext>,
    input: R,
    mut output: W,
    locale: Locale,
) -> std::io::Result<()>
where
    R: tokio::io::AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut lines = BufReader::new(input).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str(&line) {
            Ok(payload) => dispatch(&context, locale, payload).await,
            Err(error) => Some(error_response(
                serde_json::Value::Null,
                codes::PARSE_ERROR,
                &format!("Parse error: {error}"),
            )),
        };
        let Some(reply) = reply else {
            continue;
        };
        let encoded = serde_json::to_string(&reply).unwrap_or_else(|_| String::from("{}"));
        output.write_all(encoded.as_bytes()).await?;
        output.write_all(b"\n").await?;
        output.flush().await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::RecordingContext;
    use serde_json::Value;

    async fn exchange(input: &str) -> Vec<Value> {
        let context: Arc<dyn SessionToolContext> = Arc::new(RecordingContext::new());
        let mut output = Vec::new();
        serve_stdio(context, input.as_bytes(), &mut output, Locale::En)
            .await
            .expect("writing to a Vec cannot fail");
        String::from_utf8_lossy(&output)
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }

    #[tokio::test]
    async fn one_message_per_line_in_arrival_order() {
        let replies = exchange(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n\
             {\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\"}\n",
        )
        .await;
        assert_eq!(replies.len(), 2);
        assert_eq!(replies[0]["id"], 1);
        assert_eq!(replies[1]["id"], 2);
        assert_eq!(
            replies[1]["result"]["tools"].as_array().map(Vec::len),
            Some(crate::SESSION_TOOL_NAMES.len()),
        );
    }

    #[tokio::test]
    async fn a_notification_produces_no_line() {
        let replies =
            exchange("{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n").await;
        assert!(replies.is_empty());
    }

    #[tokio::test]
    async fn a_blank_line_is_skipped() {
        let replies = exchange("\n   \n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n").await;
        assert_eq!(replies.len(), 1);
    }

    #[tokio::test]
    async fn malformed_json_is_answered_not_fatal() {
        let replies =
            exchange("not json\n{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"ping\"}\n").await;
        assert_eq!(replies.len(), 2);
        assert_eq!(replies[0]["error"]["code"], codes::PARSE_ERROR);
        assert_eq!(replies[0]["id"], Value::Null);
        assert_eq!(replies[1]["id"], 9);
    }

    #[tokio::test]
    async fn a_reply_is_exactly_one_line() {
        let context: Arc<dyn SessionToolContext> = Arc::new(RecordingContext::new());
        let mut output = Vec::new();
        serve_stdio(
            context,
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}\n".as_bytes(),
            &mut output,
            Locale::En,
        )
        .await
        .expect("a Vec sink");
        let text = String::from_utf8_lossy(&output);
        assert_eq!(text.matches('\n').count(), 1);
        assert!(text.ends_with('\n'));
    }
}
