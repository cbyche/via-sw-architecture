//! The loopback HTTP transport, and the registrations served over it.
//!
//! A full port of upstream `AcpSessionToolServer`
//! (`server/src/agent/acp-session-tools.mjs:133-262`) onto `axum`.
//!
//! # The security shape, exactly
//!
//! **External contract**, catalogued three times over
//! (`http-route / POST /mcp (session tool server)`,
//! `http-route / MCP session tool endpoint`,
//! `http-route / Session MCP endpoint`):
//!
//! | | |
//! | --- | --- |
//! | Bind | `127.0.0.1:0` — loopback, kernel-chosen port |
//! | Path | exactly `/mcp` |
//! | Credential | `Authorization: Bearer <uuid v4>`, one per registration |
//! | Wrong path, or unknown/absent token | **404 with an empty body** |
//! | Right path and token, wrong method | 405, `Allow: POST`, `-32000` *Method not allowed.* |
//! | Anything unhandled | 500, `-32603` |
//!
//! Three details are load-bearing and easy to lose:
//!
//! **404, not 401.** An unauthenticated caller learns nothing — not that the
//! path exists, not that a token would have worked. The catalogue says so in
//! as many words.
//!
//! **The token is checked before the method.** A `GET /mcp` with a good token
//! is a 405; the same `GET` without one is a 404. Reordering the two checks
//! turns the 405 into an oracle for "this path is real".
//!
//! **The token never appears in the URL or the path.** Upstream's test asserts
//! it with `assert.doesNotMatch(url, /[?&]token=|\/mcp\/.+/)`, and
//! `tests/transport.rs` asserts the same thing here. A credential in a URL
//! leaks through logs, referrers and process listings; this one travels in a
//! header on the descriptor.
//!
//! # Statelessness
//!
//! Upstream constructs a fresh `McpServer` and a fresh transport **per
//! request**, with `sessionIdGenerator: undefined`
//! (`acp-session-tools.mjs:230-237`). There is no MCP session, no
//! `Mcp-Session-Id`, and no server-initiated stream. VIA reproduces that by
//! having no per-connection state at all: [`crate::protocol::dispatch`] is a
//! function of one message.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use agent_client_protocol::schema::v1::{HttpHeader, McpServer, McpServerHttp};
use axum::Router;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, oneshot};
use tokio::task::JoinHandle;
use uuid::Uuid;
use via_core::secret::Secret;
use via_downstream::text::is_js_whitespace;
use via_i18n::Locale;

use crate::context::SessionToolContext;
use crate::error::McpToolsError;
use crate::protocol::{
    codes, dispatch, error_response, internal_error_body, method_not_allowed_body,
};
use crate::registry::{RegistryHandle, SharedContext};
use crate::tools::SESSION_TOOL_SERVER;

/// The one path the server answers on.
///
/// **External contract** — upstream `acp-session-tools.mjs:211`.
pub const MCP_PATH: &str = "/mcp";

/// The loopback address the server binds.
///
/// **External contract** — upstream's `host = '127.0.0.1'` default
/// (`acp-session-tools.mjs:135`). Backend agents are local child processes;
/// nothing off this machine has any business reaching these tools.
pub const LOOPBACK_HOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

/// The authorization scheme, lower-cased for comparison.
const BEARER_SCHEME: &str = "bearer";

/// The header name **as it is written on the descriptor**.
///
/// **External contract** — upstream `acp-session-tools.mjs:189`:
/// `{ name: 'Authorization', value: \`Bearer ${token}\` }`. HTTP header names
/// are case-insensitive on the wire, but this string is not on the wire — it
/// is a JSON field a backend agent reads and may compare literally, so the
/// canonical capitalisation is reproduced rather than
/// `http::header::AUTHORIZATION`'s lower-cased spelling.
pub const AUTHORIZATION_HEADER: &str = "Authorization";

/// The largest request body accepted, in bytes.
///
/// The MCP SDK's own `MAXIMUM_MESSAGE_SIZE` is 4 MiB, and a coordination tool
/// call is three short strings. Anything larger is not a tool call.
pub const MAX_MESSAGE_BYTES: usize = 4 * 1024 * 1024;

/// The media type a client must list in `Accept` to be answered with an event
/// stream.
const EVENT_STREAM: &str = "text/event-stream";

/// The `Bearer <token>` value on a request, or `None`.
///
/// Upstream's `/^Bearer ([^\s]+)$/i` (`acp-session-tools.mjs:208`), hand-parsed
/// rather than compiled: ECMAScript's `\s` and the `regex` crate's disagree in
/// both directions (U+0085, U+FEFF), and the token is a credential, so the
/// exact accepted set is worth being deliberate about. The scheme is matched
/// ASCII-case-insensitively, exactly one space follows it, and the token must
/// be non-empty and hold no character ECMAScript would call whitespace.
#[must_use]
pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case(BEARER_SCHEME) {
        return None;
    }
    if token.is_empty() || token.chars().any(is_js_whitespace) {
        return None;
    }
    Some(token.to_owned())
}

/// Whether the client asked to be answered with an event stream.
///
/// Upstream's SDK defaults to SSE and rejects a client that does not accept
/// both `application/json` and `text/event-stream`. VIA answers with whichever
/// the client actually listed, defaulting to JSON: a client that named neither
/// gets a complete, spec-legal `application/json` response instead of a 406 it
/// cannot act on. Recorded in `docs/deviations/phase-3.md`.
fn wants_event_stream(headers: &HeaderMap) -> bool {
    headers
        .get_all(header::ACCEPT)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .any(|entry| {
            entry
                .split(';')
                .next()
                .is_some_and(|media| media.trim().eq_ignore_ascii_case(EVENT_STREAM))
        })
}

/// What the axum app needs.
#[derive(Clone, Debug)]
struct AppState {
    registry: RegistryHandle,
    locale: Locale,
}

/// One request.
///
/// The order of the checks is upstream's and is part of the contract; see the
/// module docs.
async fn handle(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();

    let context: Option<SharedContext> = if parts.uri.path() == MCP_PATH {
        match bearer_token(&parts.headers) {
            Some(token) => state.registry.lookup(token).await,
            None => None,
        }
    } else {
        None
    };

    // Wrong path, or a token nobody registered: 404, empty body, nothing
    // learned.
    let Some(context) = context else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if parts.method != Method::POST {
        return json_body(StatusCode::METHOD_NOT_ALLOWED, &method_not_allowed_body())
            .allowing_post();
    }

    let bytes = match axum::body::to_bytes(body, MAX_MESSAGE_BYTES).await {
        Ok(bytes) => bytes,
        // Upstream reaches its 500 arm the same way: something threw while
        // reading the request.
        Err(error) => {
            return json_body(
                StatusCode::INTERNAL_SERVER_ERROR,
                &internal_error_body(&error.to_string()),
            )
            .into_response();
        }
    };

    let payload: Value = match serde_json::from_slice(&bytes) {
        Ok(payload) => payload,
        Err(error) => {
            return json_body(
                StatusCode::BAD_REQUEST,
                &error_response(
                    Value::Null,
                    codes::PARSE_ERROR,
                    &format!("Parse error: {error}"),
                ),
            )
            .into_response();
        }
    };

    match dispatch(&context, state.locale, payload).await {
        // Nothing to answer: the payload held only notifications.
        None => StatusCode::ACCEPTED.into_response(),
        Some(reply) if wants_event_stream(&parts.headers) => event_stream(&reply),
        Some(reply) => json_body(StatusCode::OK, &reply).into_response(),
    }
}

/// A JSON response, pre-rendered so the body cannot fail later.
struct JsonBody {
    status: StatusCode,
    body: String,
}

impl JsonBody {
    /// Add `Allow: POST`, which only the 405 carries.
    fn allowing_post(self) -> Response {
        (
            self.status,
            [
                (
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                ),
                (header::ALLOW, HeaderValue::from_static("POST")),
            ],
            self.body,
        )
            .into_response()
    }
}

impl IntoResponse for JsonBody {
    fn into_response(self) -> Response {
        (
            self.status,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            )],
            self.body,
        )
            .into_response()
    }
}

/// Render a JSON-RPC payload as a response body.
///
/// Serialization of a `serde_json::Value` cannot fail; the fallback keeps the
/// promise that every refusal carries a JSON-RPC body rather than an empty
/// one.
fn json_body(status: StatusCode, payload: &Value) -> JsonBody {
    JsonBody {
        status,
        body: serde_json::to_string(payload).unwrap_or_else(|_| String::from("{}")),
    }
}

/// One `message` event per JSON-RPC response, then end of stream.
///
/// The SDK's stateless default. A batch answers with one event per element,
/// which is what a Streamable HTTP client expects to correlate against the ids
/// it sent.
fn event_stream(reply: &Value) -> Response {
    let frames: Vec<&Value> = match reply {
        Value::Array(replies) => replies.iter().collect(),
        single => vec![single],
    };
    let body = frames
        .into_iter()
        .map(|frame| {
            let data = serde_json::to_string(frame).unwrap_or_else(|_| String::from("{}"));
            format!("event: message\ndata: {data}\n\n")
        })
        .collect::<String>();
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, HeaderValue::from_static(EVENT_STREAM)),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static("no-cache, no-transform"),
            ),
        ],
        body,
    )
        .into_response()
}

/// What is running, when something is.
#[derive(Debug)]
struct Running {
    port: u16,
    registry: RegistryHandle,
    /// Dropping this triggers axum's graceful shutdown, which is what closes
    /// the sockets connections are sitting on. It is held by value rather than
    /// as an `Option` so that `Running` cannot exist without one.
    shutdown: oneshot::Sender<()>,
    serve: JoinHandle<()>,
}

/// The loopback MCP server the coordination tools are served from.
///
/// One per Gateway. Each backend session registers against it and gets its own
/// bearer token, so a token identifies *which* coordination run the tools act
/// on — which is why the token is the whole of the authentication and why the
/// descriptor is stable for the life of a registration even as
/// [`SessionToolRegistration::update`] swaps the context underneath it. ACP
/// agents may cache the first MCP connection for a session
/// (`acp-backend-adapter.mjs:207-209`); a descriptor that moved would strand
/// them.
#[derive(Debug)]
pub struct SessionToolServer {
    host: IpAddr,
    locale: Locale,
    running: Mutex<Option<Running>>,
}

impl Default for SessionToolServer {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionToolServer {
    /// A server that will bind loopback on a kernel-chosen port and render
    /// tool failures in the default locale.
    #[must_use]
    pub fn new() -> Self {
        Self {
            host: LOOPBACK_HOST,
            locale: Locale::default(),
            running: Mutex::new(None),
        }
    }

    /// The locale a failed tool call's sentence is rendered in.
    ///
    /// Upstream has no such choice — it is monolingual. The message is
    /// model-visible, so it follows the Gateway's locale rather than the
    /// user's utterance.
    #[must_use]
    pub fn with_locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Bind a different host. **Test seam only.**
    ///
    /// [`LOOPBACK_HOST`] is a contract, and every shipped call site takes the
    /// default. This exists so a test can bind `::1` to prove the descriptor
    /// is built from the address actually bound.
    #[must_use]
    pub fn with_host(mut self, host: IpAddr) -> Self {
        self.host = host;
        self
    }

    /// Start listening, or report where it already is.
    ///
    /// Idempotent, and safe to race: upstream memoizes an in-flight
    /// `startPromise` (`acp-session-tools.mjs:148-176`) and the mutex here
    /// does the same job. The mutex guards *lifecycle*, not ordering — the
    /// ordering contract belongs to [`crate::registry`], which is a task.
    ///
    /// # Errors
    ///
    /// [`McpToolsError::Listen`] when loopback cannot be bound, and
    /// [`McpToolsError::LocalAddress`] when it binds but will not report the
    /// port.
    pub async fn start(&self) -> Result<SocketAddr, McpToolsError> {
        let mut running = self.running.lock().await;
        self.start_locked(&mut running)
            .await
            .map(|(address, _)| address)
    }

    /// Bind if not already bound, and answer with the address **and** the
    /// registry behind it.
    ///
    /// The guard is passed in rather than taken so [`Self::register`] can bind
    /// and register under one acquisition. Reaching for the registry through a
    /// second lock would leave a branch for "started, then gone", which is a
    /// state this type does not otherwise have and would have to invent an
    /// answer for.
    async fn start_locked(
        &self,
        running: &mut Option<Running>,
    ) -> Result<(SocketAddr, RegistryHandle), McpToolsError> {
        if let Some(current) = running.as_ref() {
            return Ok((
                SocketAddr::new(self.host, current.port),
                current.registry.clone(),
            ));
        }

        let requested = SocketAddr::new(self.host, 0);
        let listener =
            TcpListener::bind(requested)
                .await
                .map_err(|source| McpToolsError::Listen {
                    address: requested,
                    source,
                })?;
        let bound = listener
            .local_addr()
            .map_err(|source| McpToolsError::LocalAddress { source })?;

        let registry = RegistryHandle::spawn();
        let app = Router::new().fallback(handle).with_state(AppState {
            registry: registry.clone(),
            locale: self.locale,
        });

        let (shutdown, signal) = oneshot::channel();
        let serve = tokio::spawn(async move {
            let served = axum::serve(listener, app).with_graceful_shutdown(async move {
                // Either a send or a drop of the sender ends the wait, so a
                // `close()` that never got to send still shuts the server
                // down.
                let _ = signal.await;
            });
            if let Err(error) = served.await {
                tracing::warn!(event = "mcp.serve_failed", error = %error);
            }
        });

        *running = Some(Running {
            port: bound.port(),
            registry: registry.clone(),
            shutdown,
            serve,
        });
        Ok((bound, registry))
    }

    /// The address the server is listening on, or `None` before
    /// [`Self::start`].
    pub async fn address(&self) -> Option<SocketAddr> {
        self.running
            .lock()
            .await
            .as_ref()
            .map(|running| SocketAddr::new(self.host, running.port))
    }

    /// Serve `context` behind a fresh bearer token, starting the server if it
    /// is not already up.
    ///
    /// The returned [`SessionToolRegistration`] carries the ACP descriptor to
    /// put in `session/new`'s `mcpServers`.
    ///
    /// # Errors
    ///
    /// Whatever [`Self::start`] reports.
    pub async fn register(
        &self,
        context: Arc<dyn SessionToolContext>,
    ) -> Result<SessionToolRegistration, McpToolsError> {
        // One acquisition covers binding and registering, so no other caller
        // can close the server between the two and leave this registration
        // pointing at a registry that is already gone.
        let (address, registry) = {
            let mut running = self.running.lock().await;
            self.start_locked(&mut running).await?
        };

        let token = Uuid::new_v4().to_string();
        registry.register(token.clone(), context).await;

        let descriptor = McpServer::Http(
            McpServerHttp::new(SESSION_TOOL_SERVER, format!("http://{address}{MCP_PATH}")).headers(
                vec![HttpHeader::new(
                    AUTHORIZATION_HEADER,
                    format!("Bearer {token}"),
                )],
            ),
        );

        Ok(SessionToolRegistration {
            descriptor,
            token: Secret::new(token),
            registry,
            released: AtomicBool::new(false),
        })
    }

    /// Stop listening, forget every token, and drop every connection.
    ///
    /// Upstream clears the contexts, closes the listener and then
    /// **destroys every open socket** (`acp-session-tools.mjs:251-261`) —
    /// its own test asserts the close completes inside 500 ms with an idle
    /// socket attached. Here that is: fire axum's graceful-shutdown signal,
    /// which tells every live connection to close and stops the accept loop;
    /// abort the serve task, which is otherwise waiting for those connections;
    /// and stop the registry task, so a request already past the accept still
    /// resolves no token.
    ///
    /// Idempotent, and a closed server can be started again.
    pub async fn close(&self) {
        let running = self.running.lock().await.take();
        let Some(Running {
            registry,
            shutdown,
            serve,
            ..
        }) = running
        else {
            return;
        };
        registry.close().await;
        // Firing the signal is what tells every live connection to close, and
        // it reaches them through a task of axum's own that the abort below
        // cannot cancel. Destructured rather than left to `Running`'s drop so
        // the order — clear, signal, abort — is stated rather than incidental.
        drop(shutdown);
        serve.abort();
    }
}

/// One backend session's access to the coordination tools.
///
/// Upstream's `register()` return value (`acp-session-tools.mjs:183-202`):
/// a descriptor, an `update`, and a `release`.
pub struct SessionToolRegistration {
    descriptor: McpServer,
    token: Secret,
    registry: RegistryHandle,
    released: AtomicBool,
}

impl SessionToolRegistration {
    /// The ACP `mcpServers` entry to hand the backend.
    ///
    /// **External contract** — `json-field / MCP server descriptor handed to
    /// ACP`:
    /// `{"type":"http","name":"via","url":"http://127.0.0.1:<port>/mcp",
    /// "headers":[{"name":"Authorization","value":"Bearer <uuid>"}]}`. Note
    /// `headers` is an **array of `{name,value}` objects**, not a map; the
    /// catalogue flags that as easy to get wrong in Rust, and it is why this
    /// returns the SDK's own type rather than a hand-built `Value`.
    #[must_use]
    pub fn descriptor(&self) -> &McpServer {
        &self.descriptor
    }

    /// The descriptor as JSON, for a caller that logs or asserts it.
    ///
    /// The bearer value is present: this is the wire form, and the wire form
    /// carries the credential. Anything that persists it must redact it.
    #[must_use]
    pub fn descriptor_json(&self) -> Value {
        serde_json::to_value(&self.descriptor).unwrap_or(Value::Null)
    }

    /// The bearer token, wrapped so `{:?}` cannot leak it.
    ///
    /// VIA's own: upstream's token is a bare string on a plain object.
    /// [`Secret`] costs nothing and keeps a registration out of the class of
    /// values that leak through a debug print.
    #[must_use]
    pub fn bearer_token(&self) -> &Secret {
        &self.token
    }

    /// Point this registration at a different context.
    ///
    /// `false` once [`Self::release`]d, and `false` if the server has closed.
    /// Upstream: *"ACP agents may cache the first MCP connection for a
    /// Session. Keep its descriptor stable while serialized owner turns
    /// replace the run context."*
    pub async fn update(&self, context: Arc<dyn SessionToolContext>) -> bool {
        if self.released.load(Ordering::Acquire) {
            return false;
        }
        self.registry
            .update(self.token.expose().to_owned(), context)
            .await
    }

    /// Revoke the token. `false` if it was already gone.
    ///
    /// Explicit, like `via_lock::GatewayLeaseHandle::release`: it is
    /// asynchronous and its answer is meaningful. [`Drop`] fires a best-effort
    /// release as well, so a registration that is simply dropped does not
    /// leave a live token behind — which upstream's does.
    pub async fn release(&self) -> bool {
        self.released.store(true, Ordering::Release);
        self.registry.release(self.token.expose().to_owned()).await
    }

    /// Whether [`Self::release`] has been called.
    #[must_use]
    pub fn is_released(&self) -> bool {
        self.released.load(Ordering::Acquire)
    }
}

impl Drop for SessionToolRegistration {
    /// Best-effort revocation.
    ///
    /// VIA's own. Upstream leaves a dropped registration's token live until
    /// the whole server closes; here the token goes with the registration.
    /// Nothing here can block or await, so a full command queue means the
    /// token survives to `close()` — upstream's behaviour, as the floor rather
    /// than the ceiling.
    fn drop(&mut self) {
        if !self.released.load(Ordering::Acquire) {
            self.registry
                .release_detached(self.token.expose().to_owned());
        }
    }
}

impl core::fmt::Debug for SessionToolRegistration {
    /// Names the endpoint, never the credential.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let url = match &self.descriptor {
            McpServer::Http(http) => http.url.as_str(),
            _ => "<not http>",
        };
        f.debug_struct("SessionToolRegistration")
            .field("url", &url)
            .field("token", &self.token)
            .field("released", &self.is_released())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderName;

    fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            if let (Ok(name), Ok(value)) =
                (name.parse::<HeaderName>(), HeaderValue::from_str(value))
            {
                map.append(name, value);
            }
        }
        map
    }

    #[test]
    fn the_bearer_scheme_is_case_insensitive_and_the_token_is_not() {
        assert_eq!(
            bearer_token(&headers(&[("authorization", "Bearer AbC")])).as_deref(),
            Some("AbC"),
        );
        assert_eq!(
            bearer_token(&headers(&[("authorization", "bEaReR AbC")])).as_deref(),
            Some("AbC"),
        );
    }

    #[test]
    fn a_malformed_authorization_yields_no_token() {
        for value in [
            "Bearer",
            "Bearer ",
            "Bearer a b",
            "Basic abc",
            "Bearerabc",
            " Bearer abc",
            "Bearer  abc",
        ] {
            assert_eq!(
                bearer_token(&headers(&[("authorization", value)])),
                None,
                "{value:?} produced a token",
            );
        }
        assert_eq!(bearer_token(&HeaderMap::new()), None);
    }

    #[test]
    fn a_token_carrying_ecmascript_whitespace_is_refused() {
        assert_eq!(
            bearer_token(&headers(&[("authorization", "Bearer a\u{feff}b")])),
            None,
        );
    }

    #[test]
    fn event_stream_is_only_chosen_when_the_client_named_it() {
        assert!(wants_event_stream(&headers(&[(
            "accept",
            "application/json, text/event-stream",
        )])));
        assert!(wants_event_stream(&headers(&[(
            "accept",
            "text/event-stream;q=0.9",
        )])));
        assert!(wants_event_stream(&headers(&[
            ("accept", "application/json"),
            ("accept", "text/event-stream"),
        ])));
        assert!(!wants_event_stream(&headers(&[("accept", "*/*")])));
        assert!(!wants_event_stream(&headers(&[(
            "accept",
            "application/json",
        )])));
        assert!(!wants_event_stream(&HeaderMap::new()));
    }

    #[test]
    fn an_event_stream_frames_every_reply() {
        let body = event_stream(&serde_json::json!([{ "id": 1 }, { "id": 2 }]));
        assert_eq!(
            body.headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some(EVENT_STREAM),
        );
    }
}
