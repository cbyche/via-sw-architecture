//! The MCP JSON-RPC surface, as a pure function over one message.
//!
//! Upstream hands a Node request/response pair to
//! `@modelcontextprotocol/sdk`'s `StreamableHTTPServerTransport` in **stateless
//! mode** (`sessionIdGenerator: undefined`) and lets `McpServer` dispatch
//! (`server/src/agent/acp-session-tools.mjs:230-248`). VIA has no such SDK in
//! its dependency set — `docs/architecture.md` §12 names `rmcp`, and the brief
//! for this crate says to build the transport on `axum` — so the dispatch is
//! here, written against the Streamable HTTP specification the SDK implements.
//!
//! Keeping it a **pure function of one JSON value** is what lets the same
//! dispatch serve both transports: [`crate::server`] over loopback HTTP, which
//! is the default because it is what upstream does, and [`crate::stdio`] for
//! `via mcp-serve`, which exists because ACP requires every agent to support
//! stdio. Neither transport knows what a tool is.
//!
//! # What is contract and what is not
//!
//! Catalogued, and reproduced byte for byte: the two JSON-RPC error codes on
//! the HTTP surface (`-32000` with the trailing period in
//! [`METHOD_NOT_ALLOWED_MESSAGE`], and `-32603`), the server identity, and
//! statelessness. See [`crate::server`].
//!
//! Not catalogued, and therefore reproduced from the specification rather than
//! from SDK bytes: the standard JSON-RPC codes below, protocol-version
//! negotiation, and the wording of a bad-arguments failure. Recorded in
//! `docs/deviations/phase-3.md`.

use std::sync::Arc;

use serde_json::{Value, json};
use via_i18n::Locale;

use crate::context::{
    DelegationLookupInput, SessionSendInput, SessionStartInput, SessionToolContext,
    SessionsListInput,
};
use crate::envelope::{error_result, json_result};
use crate::tools::{
    SESSION_TOOL_SERVER, SESSION_TOOL_SERVER_VERSION, SessionTool, tools_list_result,
};

/// The JSON-RPC version every message carries.
pub const JSONRPC_VERSION: &str = "2.0";

/// The MCP protocol revisions this server speaks, oldest first.
///
/// Negotiation is the specification's: echo the client's revision when it is
/// one of these, otherwise answer with [`LATEST_PROTOCOL_VERSION`] and let the
/// client decide whether it can proceed.
pub const SUPPORTED_PROTOCOL_VERSIONS: [&str; 3] = ["2024-11-05", "2025-03-26", "2025-06-18"];

/// The newest revision this server speaks.
pub const LATEST_PROTOCOL_VERSION: &str = SUPPORTED_PROTOCOL_VERSIONS[2];

/// The JSON-RPC method names this server answers.
pub mod methods {
    /// The MCP handshake.
    pub const INITIALIZE: &str = "initialize";
    /// The client's post-handshake notification. Stateless: acknowledged and
    /// discarded.
    pub const INITIALIZED: &str = "notifications/initialized";
    /// Liveness.
    pub const PING: &str = "ping";
    /// The five tool definitions.
    pub const TOOLS_LIST: &str = "tools/list";
    /// Invoke one tool.
    pub const TOOLS_CALL: &str = "tools/call";

    /// Every method, for tests and for a `Method not found` that wants to be
    /// helpful.
    pub const ALL: [&str; 5] = [INITIALIZE, INITIALIZED, PING, TOOLS_LIST, TOOLS_CALL];
}

/// The JSON-RPC error codes this server emits.
pub mod codes {
    /// The body was not JSON.
    pub const PARSE_ERROR: i64 = -32700;
    /// The message was JSON but not a JSON-RPC request.
    pub const INVALID_REQUEST: i64 = -32600;
    /// No such method.
    pub const METHOD_NOT_FOUND: i64 = -32601;
    /// The params did not satisfy the method — including a tool name nobody
    /// serves and arguments that fail a tool's schema.
    pub const INVALID_PARAMS: i64 = -32602;

    /// **External contract** — upstream `acp-session-tools.mjs:158`. The code
    /// on the 500 an unhandled failure produces.
    pub const INTERNAL_ERROR: i64 = -32603;

    /// **External contract** — upstream `acp-session-tools.mjs:226`. The code
    /// on the 405 a non-`POST` to an authenticated `/mcp` produces. It is in
    /// the implementation-defined `-32000..=-32099` range, not a standard
    /// code.
    pub const METHOD_NOT_ALLOWED: i64 = -32000;
}

/// The message on the 405 body.
///
/// **External contract** — upstream `acp-session-tools.mjs:226`. The trailing
/// period is part of it; `docs/reference/contracts.json` says so explicitly.
pub const METHOD_NOT_ALLOWED_MESSAGE: &str = "Method not allowed.";

/// A JSON-RPC success response.
#[must_use]
pub fn response(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": JSONRPC_VERSION, "id": id, "result": result })
}

/// A JSON-RPC error response.
///
/// Field order is `jsonrpc`, `id`, `error` — the order the catalogued 405 and
/// 500 bodies are written in, so every error this server emits reads the same.
#[must_use]
pub fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": JSONRPC_VERSION,
        "id": id,
        "error": { "code": code, "message": message },
    })
}

/// The body of the 405 a non-`POST` to an authenticated `/mcp` receives.
///
/// **External contract** — `{"jsonrpc":"2.0","id":null,"error":{"code":-32000,
/// "message":"Method not allowed."}}`.
#[must_use]
pub fn method_not_allowed_body() -> Value {
    error_response(
        Value::Null,
        codes::METHOD_NOT_ALLOWED,
        METHOD_NOT_ALLOWED_MESSAGE,
    )
}

/// The body of the 500 an unhandled failure produces.
///
/// **External contract** — `{"jsonrpc":"2.0","id":null,"error":{"code":-32603,
/// "message":"<error.message>"}}`.
#[must_use]
pub fn internal_error_body(message: &str) -> Value {
    error_response(Value::Null, codes::INTERNAL_ERROR, message)
}

/// The `initialize` result.
///
/// The server identity is upstream's
/// `new McpServer({ name: ACP_SESSION_TOOL_SERVER, version: '1.0.0' })`.
/// `listChanged` is `false` and truthfully so: the five tools are compiled in
/// and a stateless transport has no channel to notify a change over.
#[must_use]
pub fn initialize_result(requested: Option<&str>) -> Value {
    json!({
        "protocolVersion": negotiate_protocol_version(requested),
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": {
            "name": SESSION_TOOL_SERVER,
            "version": SESSION_TOOL_SERVER_VERSION,
        },
    })
}

/// The revision to answer `initialize` with.
#[must_use]
pub fn negotiate_protocol_version(requested: Option<&str>) -> &'static str {
    requested
        .and_then(|requested| {
            SUPPORTED_PROTOCOL_VERSIONS
                .into_iter()
                .find(|supported| *supported == requested)
        })
        .unwrap_or(LATEST_PROTOCOL_VERSION)
}

/// Dispatch one parsed JSON-RPC payload — a single message or a batch.
///
/// Returns `None` when there is nothing to answer: the payload held only
/// notifications. A transport turns that into `202 Accepted` with no body
/// (HTTP) or into silence (stdio).
pub async fn dispatch(
    context: &Arc<dyn SessionToolContext>,
    locale: Locale,
    payload: Value,
) -> Option<Value> {
    match payload {
        Value::Array(messages) => {
            if messages.is_empty() {
                return Some(error_response(
                    Value::Null,
                    codes::INVALID_REQUEST,
                    "Invalid Request: batch must not be empty",
                ));
            }
            let mut replies = Vec::with_capacity(messages.len());
            for message in messages {
                if let Some(reply) = dispatch_one(context, locale, message).await {
                    replies.push(reply);
                }
            }
            (!replies.is_empty()).then_some(Value::Array(replies))
        }
        single => dispatch_one(context, locale, single).await,
    }
}

/// Dispatch exactly one JSON-RPC message.
async fn dispatch_one(
    context: &Arc<dyn SessionToolContext>,
    locale: Locale,
    message: Value,
) -> Option<Value> {
    let Some(object) = message.as_object() else {
        return Some(error_response(
            Value::Null,
            codes::INVALID_REQUEST,
            "Invalid Request: message must be an object",
        ));
    };

    // A Notification is a request object *without* an `id` member. An `id`
    // that is present but neither a string nor a number matches neither
    // schema, so it is an Invalid Request answered against a null id.
    let id = match object.get("id") {
        None => None,
        Some(id @ (Value::String(_) | Value::Number(_))) => Some(id.clone()),
        Some(_) => {
            return Some(error_response(
                Value::Null,
                codes::INVALID_REQUEST,
                "Invalid Request: id must be a string or a number",
            ));
        }
    };

    let method = object.get("method").and_then(Value::as_str);
    let Some(method) = method else {
        // A malformed notification is discarded in silence; JSON-RPC forbids
        // answering one at all.
        return id.map(|id| {
            error_response(
                id,
                codes::INVALID_REQUEST,
                "Invalid Request: method must be a string",
            )
        });
    };

    let params = object.get("params");

    let Some(id) = id else {
        // Notifications. `notifications/initialized` is the handshake's
        // second half and carries nothing a stateless server can keep;
        // everything else — `notifications/cancelled`, `progress` — is
        // likewise informational here. All are accepted silently.
        return None;
    };

    Some(match method {
        methods::INITIALIZE => {
            let requested = params
                .and_then(|params| params.get("protocolVersion"))
                .and_then(Value::as_str);
            response(id, initialize_result(requested))
        }
        methods::PING => response(id, json!({})),
        methods::TOOLS_LIST => response(id, tools_list_result()),
        methods::TOOLS_CALL => match call_tool(context, locale, params).await {
            Ok(result) => response(id, result),
            Err(failure) => error_response(id, failure.code, &failure.message),
        },
        unknown => error_response(
            id,
            codes::METHOD_NOT_FOUND,
            &format!("Method not found: {unknown}"),
        ),
    })
}

/// A `tools/call` that could not reach a handler at all.
struct CallFailure {
    code: i64,
    message: String,
}

impl CallFailure {
    fn invalid_params(message: String) -> Self {
        Self {
            code: codes::INVALID_PARAMS,
            message,
        }
    }
}

/// Run one `tools/call`.
///
/// The split matters. A call that never reached a handler — no such tool,
/// arguments that fail the schema — is a **JSON-RPC error**. A call that
/// reached a handler and the handler failed is a **result** carrying the error
/// envelope, because upstream wraps every handler in `try/catch` and answers
/// `{status:'failed', error}` (`acp-session-tools.mjs:47-53`).
async fn call_tool(
    context: &Arc<dyn SessionToolContext>,
    locale: Locale,
    params: Option<&Value>,
) -> Result<Value, CallFailure> {
    let Some(params) = params.and_then(Value::as_object) else {
        return Err(CallFailure::invalid_params(
            "Invalid params: tools/call requires an object".to_owned(),
        ));
    };
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        return Err(CallFailure::invalid_params(
            "Invalid params: tools/call requires a tool name".to_owned(),
        ));
    };
    let Some(tool) = SessionTool::from_name(name) else {
        return Err(CallFailure::invalid_params(format!(
            "Tool {name} not found"
        )));
    };

    let arguments = params.get("arguments");
    let result = match tool {
        SessionTool::SessionsList => {
            let input = SessionsListInput::parse(arguments)
                .map_err(|error| CallFailure::invalid_params(error.to_string()))?;
            outcome(context.list_sessions(input).await, locale)
        }
        SessionTool::SessionStart => {
            let input = SessionStartInput::parse(arguments)
                .map_err(|error| CallFailure::invalid_params(error.to_string()))?;
            outcome(context.start_session(input).await, locale)
        }
        SessionTool::SessionSend => {
            let input = SessionSendInput::parse(arguments)
                .map_err(|error| CallFailure::invalid_params(error.to_string()))?;
            outcome(context.send_session(input).await, locale)
        }
        SessionTool::SessionStatus => {
            let input = DelegationLookupInput::parse(tool, arguments)
                .map_err(|error| CallFailure::invalid_params(error.to_string()))?;
            outcome(context.session_status(input).await, locale)
        }
        SessionTool::SessionCancel => {
            let input = DelegationLookupInput::parse(tool, arguments)
                .map_err(|error| CallFailure::invalid_params(error.to_string()))?;
            outcome(context.cancel_session(input).await, locale)
        }
    };
    Ok(result)
}

/// Upstream's `try { return jsonResult(await …) } catch (error) { return
/// errorResult(error) }`, once, for all five handlers.
fn outcome<T: serde::Serialize>(
    result: Result<T, via_downstream::HarnessError>,
    locale: Locale,
) -> Value {
    match result {
        Ok(value) => json_result(&value),
        Err(error) => error_result(&error.message(locale)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::{envelope_text, is_error_envelope};
    use crate::testing::RecordingContext;

    fn context() -> Arc<dyn SessionToolContext> {
        Arc::new(RecordingContext::new())
    }

    async fn call(payload: Value) -> Option<Value> {
        dispatch(&context(), Locale::En, payload).await
    }

    #[tokio::test]
    async fn initialize_reports_the_server_identity() {
        let reply = call(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "protocolVersion": "2025-03-26", "capabilities": {} },
        }))
        .await
        .expect("a reply");
        assert_eq!(reply["jsonrpc"], "2.0");
        assert_eq!(reply["id"], 1);
        assert_eq!(reply["result"]["serverInfo"]["name"], SESSION_TOOL_SERVER);
        assert_eq!(reply["result"]["serverInfo"]["version"], "1.0.0");
        assert_eq!(reply["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(
            reply["result"]["capabilities"]["tools"]["listChanged"],
            false
        );
    }

    #[tokio::test]
    async fn an_unknown_protocol_revision_gets_the_latest() {
        assert_eq!(negotiate_protocol_version(Some("1999-01-01")), "2025-06-18");
        assert_eq!(negotiate_protocol_version(None), "2025-06-18");
        assert_eq!(negotiate_protocol_version(Some("2024-11-05")), "2024-11-05");
    }

    #[tokio::test]
    async fn tools_list_serves_exactly_the_five() {
        let reply = call(json!({ "jsonrpc": "2.0", "id": "a", "method": "tools/list" }))
            .await
            .expect("a reply");
        let names: Vec<&str> = reply["result"]["tools"]
            .as_array()
            .expect("an array")
            .iter()
            .filter_map(|tool| tool["name"].as_str())
            .collect();
        assert_eq!(names, crate::SESSION_TOOL_NAMES.to_vec());
    }

    #[tokio::test]
    async fn a_notification_is_never_answered() {
        assert!(
            call(json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }))
                .await
                .is_none()
        );
        assert!(
            call(json!({ "jsonrpc": "2.0", "method": "notifications/whatever" }))
                .await
                .is_none()
        );
        assert!(call(json!({ "jsonrpc": "2.0" })).await.is_none());
    }

    #[tokio::test]
    async fn an_unknown_method_is_minus_32601() {
        let reply = call(json!({ "jsonrpc": "2.0", "id": 3, "method": "nope" }))
            .await
            .expect("a reply");
        assert_eq!(reply["error"]["code"], codes::METHOD_NOT_FOUND);
        assert_eq!(reply["id"], 3);
    }

    #[tokio::test]
    async fn a_non_object_message_is_invalid_request_against_a_null_id() {
        let reply = call(json!("hello")).await.expect("a reply");
        assert_eq!(reply["error"]["code"], codes::INVALID_REQUEST);
        assert_eq!(reply["id"], Value::Null);
    }

    #[tokio::test]
    async fn a_null_id_is_neither_request_nor_notification() {
        let reply = call(json!({ "jsonrpc": "2.0", "id": null, "method": "ping" }))
            .await
            .expect("a reply");
        assert_eq!(reply["error"]["code"], codes::INVALID_REQUEST);
        assert_eq!(reply["id"], Value::Null);
    }

    #[tokio::test]
    async fn an_unknown_tool_is_minus_32602() {
        let reply = call(json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": { "name": "rm_rf", "arguments": {} },
        }))
        .await
        .expect("a reply");
        assert_eq!(reply["error"]["code"], codes::INVALID_PARAMS);
        assert_eq!(reply["error"]["message"], "Tool rm_rf not found");
    }

    #[tokio::test]
    async fn a_namespaced_tool_name_is_not_dispatchable() {
        // The three lenient shapes are the permission broker's question. The
        // dispatcher answers only the exact name it advertised.
        let reply = call(json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": { "name": "mcp__via__via_session_start" },
        }))
        .await
        .expect("a reply");
        assert_eq!(reply["error"]["code"], codes::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn bad_arguments_are_a_json_rpc_error_not_a_tool_result() {
        let reply = call(json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": { "name": "via_session_start", "arguments": {} },
        }))
        .await
        .expect("a reply");
        assert_eq!(reply["error"]["code"], codes::INVALID_PARAMS);
        assert_eq!(
            reply["error"]["message"],
            "Invalid arguments for tool via_session_start: prompt: Required",
        );
        assert!(reply.get("result").is_none());
    }

    #[tokio::test]
    async fn a_handler_failure_is_a_result_carrying_the_error_envelope() {
        let context: Arc<dyn SessionToolContext> =
            Arc::new(RecordingContext::new().failing("后台 Agent 拒绝了这次请求"));
        let reply = dispatch(
            &context,
            Locale::Zh,
            json!({
                "jsonrpc": "2.0",
                "id": 7,
                "method": "tools/call",
                "params": { "name": "via_sessions_list", "arguments": {} },
            }),
        )
        .await
        .expect("a reply");
        assert!(reply.get("error").is_none(), "not a transport failure");
        assert!(is_error_envelope(&reply["result"]));
        let text = envelope_text(&reply["result"]).expect("a text block");
        let payload: Value = serde_json::from_str(text).expect("json");
        assert_eq!(payload["status"], "failed");
        assert_eq!(payload["error"], "后台 Agent 拒绝了这次请求");
    }

    #[tokio::test]
    async fn a_batch_answers_only_its_requests() {
        let reply = call(json!([
            { "jsonrpc": "2.0", "method": "notifications/initialized" },
            { "jsonrpc": "2.0", "id": 1, "method": "ping" },
            { "jsonrpc": "2.0", "id": 2, "method": "tools/list" },
        ]))
        .await
        .expect("a reply");
        let replies = reply.as_array().expect("an array");
        assert_eq!(replies.len(), 2);
        assert_eq!(replies[0]["id"], 1);
        assert_eq!(replies[1]["id"], 2);
    }

    #[tokio::test]
    async fn a_batch_of_only_notifications_is_answered_with_nothing() {
        assert!(
            call(json!([{ "jsonrpc": "2.0", "method": "notifications/initialized" }]))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn an_empty_batch_is_invalid() {
        let reply = call(json!([])).await.expect("a reply");
        assert_eq!(reply["error"]["code"], codes::INVALID_REQUEST);
    }

    #[tokio::test]
    async fn tools_call_without_params_is_invalid_params() {
        let reply = call(json!({ "jsonrpc": "2.0", "id": 8, "method": "tools/call" }))
            .await
            .expect("a reply");
        assert_eq!(reply["error"]["code"], codes::INVALID_PARAMS);
    }

    #[test]
    fn the_two_catalogued_bodies_are_exact() {
        assert_eq!(
            serde_json::to_string(&method_not_allowed_body()).expect("json"),
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32000,"message":"Method not allowed."}}"#,
        );
        assert_eq!(
            serde_json::to_string(&internal_error_body("boom")).expect("json"),
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"boom"}}"#,
        );
    }
}
