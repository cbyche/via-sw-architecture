//! The candidate walk, its first-frame probe, and the transport it hands back.
//!
//! Ported from ARGO `tinicore/src/llm/providers/openai_live.rs:1245-1720`
//! (`LiveProvider::open`) against **ARGO bring-up §4**, which is the acceptance
//! spec. `via-realtime` owns the session machine, the registry, the connection
//! status and the reconnect ladder; this file owns the walk that gets a socket
//! into a state the session machine can be handed.
//!
//! # The four things this reproduces, and what each one cost
//!
//! **Binary frames, not Text.** RFC 6455 makes Text/Binary a framing choice,
//! not a content type; both carry bytes and only Text promises UTF-8. The
//! gateway ARGO was brought up against forwards the whole session as **Binary**
//! — `session.created` arrived as 1254 bytes of opcode 2 — and a reader that
//! accepted only Text discarded every one of them. The socket upgraded, frames
//! flowed, and the probe reported *"sent no protocol frame"*: eight rounds of
//! bring-up spent on a working endpoint that looked dead. So the honest test is
//! whether the payload decodes to UTF-8 and parses as this protocol
//! ([`protocol_text`]).
//!
//! **`Sec-WebSocket-Protocol: realtime`.** The upgrade succeeds without it but
//! is not routed. Offered on every candidate; see
//! [`SUBPROTOCOL`](crate::SUBPROTOCOL).
//!
//! **A completed 101 is not a session.** A relay will accept the socket on a
//! path it does not route, leaving the client connected to the proxy and the
//! proxy connected to nothing. Nothing arrives, there is no error, and the walk
//! stops on this "success" without trying the shape that would have worked — so
//! a 101 here is strictly worse than a 403. The protocol gives a free liveness
//! probe: a real endpoint sends `session.created` unprompted, immediately. So
//! the configuration goes out and *some* protocol frame must come back before a
//! candidate is committed to.
//!
//! **Pings do not count.** A gateway sends WebSocket Ping keepalives whether or
//! not it ever connected upstream, so a Ping satisfies a naive liveness check
//! while the session is still dead — the same proxy fooled the probe twice. And
//! the deadline is computed **once, outside the loop**: a per-iteration
//! `timeout(...)` restarts the clock on every frame, so a gateway pinging faster
//! than the timeout holds a dead candidate open for as long as it keeps pinging.
//! Observed on a real proxy: a 3 s probe took 5.65 s and ate half the connect
//! budget, so the later candidates were never given a fair window.
//!
//! # And the fifth, which is not about connecting
//!
//! The in-session schema repair and the duplicate-`response.create` filter both
//! live *below* the session machine, in the transport this module returns —
//! because both are decisions about a frame, and by the time a frame reaches
//! `via-realtime` the decision has to have been made. An `error` before the
//! session is ready is what rejects `connect()`, so a GA-discriminator rejection
//! that reached the session machine would kill the session instead of repairing
//! it. See [`InboundFilter`].

use std::sync::Arc;
use std::time::Duration;

use futures::{Sink, SinkExt, Stream, StreamExt};
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{HeaderName, HeaderValue};
use via_realtime::{
    AgentContext, RealtimeError, RealtimeProtocol, RealtimeProvider, RealtimeSession,
    SessionEvents, SessionOptions, SessionRequest, Transport, ga_realtime_protocol,
    openai_compatible_protocol,
};

use crate::conflict::{CreateAccounting, is_active_response_conflict};
use crate::provider::OpenAiRealtimeProvider;
use crate::schema::{RealtimeSchema, protocol_for_url, rejects_ga_discriminator};

/// The tracing target every `[live]` marker is emitted on.
pub const LOG_TARGET: &str = "via::realtime::openai";

/// How long a freshly upgraded socket has to send its first protocol frame.
///
/// External contract — ARGO `openai_live.rs:88`. A real Realtime endpoint sends
/// `session.created` immediately — a round trip, not a model call — so this is
/// already generous. It is deliberately short because it is paid **per
/// candidate**: the whole walk must stay inside [`CONNECT_TOTAL_BUDGET`], and
/// every second here is a second the session is not yet usable.
///
/// It is a **deadline for the whole probe**, not a per-frame idle timeout — that
/// distinction is the difference between catching a silent gateway and being
/// held open by its keepalives.
pub const FIRST_FRAME_TIMEOUT: Duration = Duration::from_secs(3);

/// Ceiling on the whole candidate walk.
///
/// External contract — ARGO `openai_live.rs:110`. Without it the cost is
/// per-candidate timeout × candidate count, so adding a candidate silently
/// lengthens how long a start can hang — and a user waiting on a microphone has
/// a much shorter patience than that arithmetic.
///
/// It sits inside `via-realtime`'s 25 s
/// [`CONNECT_TIMEOUT`](via_realtime::CONNECT_TIMEOUT), which covers the socket
/// *and* the `session.created` / `session.update` / `session.updated` handshake
/// on top of it.
pub const CONNECT_TOTAL_BUDGET: Duration = Duration::from_secs(12);

/// Advisory ceiling on the serialized `session.update`.
///
/// External contract — ARGO `openai_live.rs:104`. 64 KB is well under any
/// provider limit and well over what a voice session needs: a realtime model
/// choosing between hundreds of tools mid-utterance is a latency and accuracy
/// problem before it is a size problem.
///
/// Not enforced — a host is entitled to advertise its whole toolset — but called
/// out, because the failure it produces is opaque: a provider that refuses an
/// oversized configuration may drop it **without an error event**, leaving a
/// connected session that never responds.
pub const SESSION_UPDATE_SIZE_WARN: usize = 64 * 1024;

/// Cap on how much of a rejected handshake's body is quoted.
///
/// External contract — ARGO `openai_live.rs:204`.
pub const CONNECT_ERROR_BODY_LIMIT: usize = 1_024;

/// How much of a non-protocol first frame is quoted back in the rejection.
pub const PROBE_EXCERPT_CHARS: usize = 200;

/// The header Azure puts a machine-readable reason in.
pub const AZURE_ERROR_CODE_HEADER: &str = "x-ms-error-code";

/// The most candidates any dialect produces.
///
/// [`ConnectBudget::within`] divides by it, so a shrunk budget keeps the same
/// relationship the two constants have: four candidates, each given an equal
/// share, adding up to the whole.
pub const MAX_CANDIDATES: u32 = 4;

/// The two timers that bound a candidate walk.
///
/// [`DEFAULT`](Self::DEFAULT) is ARGO's pair. The type exists so a caller with a
/// shorter deadline of its own can hand the walk a smaller one instead of
/// aborting it from outside — an aborted walk drops its rejection list, which is
/// the only record of how far it got.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectBudget {
    /// Ceiling on the whole walk. Checked before each candidate is dialled.
    pub total: Duration,
    /// How long one candidate has to complete its upgrade, and then how long it
    /// has to send a protocol frame. Two separate windows of the same length: an
    /// upgrade is a round trip and so is `session.created`.
    pub first_frame: Duration,
}

impl ConnectBudget {
    /// [`CONNECT_TOTAL_BUDGET`] and [`FIRST_FRAME_TIMEOUT`].
    pub const DEFAULT: Self = Self {
        total: CONNECT_TOTAL_BUDGET,
        first_frame: FIRST_FRAME_TIMEOUT,
    };

    /// The largest budget that still fits inside a caller's own deadline.
    ///
    /// **A quarter of the caller's window is left over on purpose.** A walk that
    /// runs right up to the caller's deadline is a walk the caller cuts off, and
    /// a cut-off walk reports nothing — the rejection list goes with the dropped
    /// future, which is the exact failure this type exists to avoid. The margin
    /// is what buys the report time to be composed and returned.
    ///
    /// Never larger than [`DEFAULT`](Self::DEFAULT), and never zero: a budget of
    /// zero would report "exhausted" without dialling anything, which is a worse
    /// answer than one attempt that fails honestly.
    #[must_use]
    pub fn within(deadline: Duration) -> Self {
        // The ordinary path is untouched: `via-realtime`'s 25 s connect timeout
        // has room for the whole walk plus one more probe, so ARGO's two
        // constants go through exactly as written.
        if deadline >= CONNECT_TOTAL_BUDGET.saturating_add(FIRST_FRAME_TIMEOUT) {
            return Self::DEFAULT;
        }
        let total = deadline
            .saturating_sub(deadline / MAX_CANDIDATES)
            .max(Duration::from_millis(1));
        Self {
            total,
            // One share more than there are candidates, so a walk that spends
            // every window still finishes inside its own ceiling rather than on
            // it.
            first_frame: FIRST_FRAME_TIMEOUT
                .min(total / (MAX_CANDIDATES + 1))
                .max(Duration::from_millis(1)),
        }
    }
}

impl Default for ConnectBudget {
    fn default() -> Self {
        Self::DEFAULT
    }
}

// ── frame classification ─────────────────────────────────────────────────────

/// The protocol JSON a frame carries, whether the peer framed it as Text or as
/// Binary.
///
/// [`Message::Ping`] / [`Message::Pong`] deliberately yield `None`: a keepalive
/// proves a socket, not a session, which is the distinction the connect probe
/// exists to draw.
#[must_use]
pub fn protocol_text(message: &Message) -> Option<&str> {
    match message {
        Message::Text(text) => Some(text.as_str()),
        Message::Binary(bytes) => core::str::from_utf8(bytes).ok(),
        _ => None,
    }
}

/// Frame kind for log lines — the word that tells a reader whether the peer is
/// framing this session the way OpenAI does or the way a relay does.
#[must_use]
pub fn frame_kind(message: &Message) -> &'static str {
    match message {
        Message::Text(_) => "text",
        Message::Binary(_) => "binary",
        Message::Ping(_) => "ping",
        Message::Pong(_) => "pong",
        Message::Close(_) => "close",
        _ => "raw",
    }
}

/// What the first-frame probe makes of one frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeVerdict {
    /// Proves the socket exists, not that it is routed. Keep waiting **inside
    /// the same deadline** — a keepalive, or binary that is not UTF-8.
    KeepWaiting,
    /// A JSON object with a `type`: something speaking this protocol is on the
    /// other end.
    Accepted {
        /// The event type that proved it, for the log line.
        kind: String,
    },
    /// Decoded to text but not to a protocol event. Reject this candidate.
    NotProtocol {
        /// The first [`PROBE_EXCERPT_CHARS`] characters, for the rejection.
        excerpt: String,
    },
}

/// Whether one frame is evidence that the endpoint speaks this protocol.
///
/// Only a Text **or Binary** frame that parses as a JSON object with a `type`
/// counts. Everything else either keeps the probe waiting or rejects the
/// candidate outright.
#[must_use]
pub fn classify_probe_frame(message: &Message) -> ProbeVerdict {
    let Some(text) = protocol_text(message) else {
        return ProbeVerdict::KeepWaiting;
    };
    match serde_json::from_str::<Value>(text)
        .ok()
        .and_then(|value| value.get("type").and_then(Value::as_str).map(str::to_owned))
    {
        Some(kind) => ProbeVerdict::Accepted { kind },
        None => ProbeVerdict::NotProtocol {
            excerpt: text.chars().take(PROBE_EXCERPT_CHARS).collect(),
        },
    }
}

// ── the inbound filter ───────────────────────────────────────────────────────

/// What the transport does with one decoded inbound event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboundVerdict {
    /// Hand it to the session machine, byte for byte.
    Forward,
    /// The endpoint refused the GA discriminator. Resend the configuration in
    /// the pre-GA schema **on this same socket** and swallow the error.
    RepairSchema,
    /// A second GA-discriminator rejection after the repair already went out.
    /// Swallow it without resending.
    DropRepaired,
    /// The relay's own duplicate `response.create` bouncing off the response its
    /// VAD just opened. Not ours; drop it.
    DropConflict,
}

/// The two frame-level decisions this crate makes below the session machine.
///
/// Both exist because `via-realtime` treats an `error` arriving before the
/// session is ready as a refused connect — correctly, for every other provider —
/// and both of these errors are ones the *client* is supposed to answer rather
/// than surface.
#[derive(Debug)]
pub struct InboundFilter {
    url: String,
    creates: CreateAccounting,
    repaired: bool,
}

impl InboundFilter {
    /// A filter for the endpoint at `url`.
    #[must_use]
    pub fn new(url: &str, creates: CreateAccounting) -> Self {
        Self {
            url: url.to_owned(),
            creates,
            repaired: false,
        }
    }

    /// Whether the pre-GA repair has already been sent on this socket.
    #[must_use]
    pub fn repaired(&self) -> bool {
        self.repaired
    }

    /// Decide what to do with one inbound event.
    ///
    /// # Why the repair happens once and the *swallow* does not
    ///
    /// ARGO resends the configuration **once, not in a loop**: if the pre-GA
    /// schema is refused too, the endpoint disagrees with both generations and
    /// the error belongs on screen rather than in another silent resend. That is
    /// kept exactly — [`RepairSchema`](InboundVerdict::RepairSchema) is returned
    /// at most once per socket.
    ///
    /// What is new is [`DropRepaired`](InboundVerdict::DropRepaired), and
    /// `docs/deviations/phase-6-via-realtime-openai.md` records why VIA needs
    /// it. ARGO sends exactly one `session.update` per session, so exactly one
    /// GA rejection can ever come back. VIA sends two: the walk's probe payload,
    /// and the session machine's own update on `session.created`. Both are in
    /// flight before either can bounce, so the second rejection is a knock-on
    /// from a payload that was already on the wire — and surfacing it would kill
    /// the session the repair just fixed. It is dropped **without** resending,
    /// so the "once, not in a loop" rule is untouched.
    pub fn apply(&mut self, event: &Value) -> InboundVerdict {
        if event.get("type").and_then(Value::as_str) == Some("response.created") {
            // One create is now answered. Saturating, because server VAD opens
            // responses no client asked for.
            self.creates.answered();
        }

        if rejects_ga_discriminator(event) {
            if self.repaired {
                tracing::debug!(
                    target: LOG_TARGET,
                    url = %self.url,
                    "[live] dropped a second GA-discriminator rejection; the repair is already out"
                );
                return InboundVerdict::DropRepaired;
            }
            self.repaired = true;
            tracing::info!(
                target: LOG_TARGET,
                url = %self.url,
                "[live] endpoint rejected the GA discriminator; resending session.update in the pre-GA schema"
            );
            // Remember it, so the next session opens in the right schema instead
            // of repeating this repair mid-utterance.
            crate::schema::remember_preview_endpoint(&self.url);
            return InboundVerdict::RepairSchema;
        }

        if is_active_response_conflict(event) && !self.creates.conflict_is_ours() {
            tracing::debug!(
                target: LOG_TARGET,
                "[live] dropped an active-response conflict the client did not cause"
            );
            return InboundVerdict::DropConflict;
        }

        InboundVerdict::Forward
    }
}

// ── handshake diagnostics ────────────────────────────────────────────────────

/// Render a failed WebSocket handshake with everything it actually carries.
///
/// External contract — ARGO `openai_live.rs:206-256`.
///
/// `tungstenite::Error::Http`'s own `Display` prints the status and **discards
/// the response body**, which is exactly where a provider explains itself. A
/// rejected realtime upgrade then reads as a bare `403 Forbidden`, which is
/// indistinguishable between a disabled local-auth policy, a network ACL, a
/// gateway that refuses upgrades, and a key without access to the model. All
/// four need different fixes, so the body is the diagnosis.
///
/// `x-ms-error-code` is named explicitly because Azure puts a machine-readable
/// reason there and sometimes returns an empty body.
///
/// Nothing here can leak the credential: it travels in a **request** header, and
/// this only reads the *response*.
#[must_use]
pub fn describe_ws_error(error: &tokio_tungstenite::tungstenite::Error) -> String {
    use tokio_tungstenite::tungstenite::Error as WsError;

    let WsError::Http(response) = error else {
        return error.to_string();
    };

    let mut out = format!("HTTP {}", response.status());
    if let Some(code) = response
        .headers()
        .get(AZURE_ERROR_CODE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|code| !code.is_empty())
    {
        out.push_str(&format!(" [{code}]"));
    }
    match response
        .body()
        .as_deref()
        .map(|body| String::from_utf8_lossy(body).trim().to_owned())
        .filter(|body| !body.is_empty())
    {
        Some(body) => {
            out.push_str(": ");
            if body.len() > CONNECT_ERROR_BODY_LIMIT {
                // Slice on a char boundary — a body can be UTF-8 and cutting
                // mid-sequence would panic.
                let end = (0..=CONNECT_ERROR_BODY_LIMIT)
                    .rev()
                    .find(|index| body.is_char_boundary(*index))
                    .unwrap_or(0);
                out.push_str(&body[..end]);
                out.push_str("… (truncated)");
            } else {
                out.push_str(&body);
            }
        }
        None => out.push_str(" (empty body)"),
    }
    out
}

/// Record a rejected candidate, logging it immediately.
///
/// Immediately, because the aggregated error at the end of the walk is not
/// always reached: a caller that abandons a slow start drops the task mid-walk,
/// and then the whole rejection list — the only record of how far it got — goes
/// with it. Logging per rejection means an abandoned connect still leaves a
/// trail.
fn push_rejection(rejections: &mut Vec<String>, message: String) {
    tracing::warn!(target: LOG_TARGET, "[live] candidate rejected: {message}");
    rejections.push(message);
}

/// The aggregated failure of a whole walk.
///
/// Every attempt with its own outcome. One status code cannot distinguish a
/// wrong route from a wrong `api-version` from a refused credential from a proxy
/// that never bridged; the full list can.
#[must_use]
pub fn connect_failure(rejections: &[String]) -> RealtimeError {
    RealtimeError::Transport {
        detail: format!(
            "no realtime endpoint completed a session ({} attempt(s)): {}",
            rejections.len(),
            rejections.join(" | ")
        ),
    }
}

// ── the payloads ─────────────────────────────────────────────────────────────

/// The `session.update` frame this provider would send in one named schema.
///
/// One payload per **generation**, not one per session: the candidate walk
/// varies the route, and the route is what decides the schema. Holding a single
/// payload across candidates would make the walk half a walk — every preview
/// route would be dialled with a GA body it must reject, and the rejection would
/// read as *"wrong route"* rather than *"wrong schema"*.
///
/// # Errors
///
/// [`RealtimeError::Transport`] if the payload cannot be serialized, which for a
/// `serde_json::Value` has no reachable branch.
pub fn session_update_frame(
    provider: &OpenAiRealtimeProvider,
    schema: RealtimeSchema,
    agent_context: &AgentContext,
) -> Result<String, RealtimeError> {
    let session = provider.build_session_for(
        schema,
        &SessionRequest {
            configured: false,
            agent_context,
        },
    );
    // The dialect that matches the schema, so the bytes are the ones the session
    // machine would have written for the same endpoint.
    let frame = match schema {
        RealtimeSchema::Ga => {
            let dialect = ga_realtime_protocol();
            dialect.encode_outgoing(dialect.session_update(session))
        }
        RealtimeSchema::Preview => {
            let dialect = openai_compatible_protocol();
            dialect.encode_outgoing(dialect.session_update(session))
        }
    };
    serde_json::to_string(&frame).map_err(|error| RealtimeError::Transport {
        detail: format!("encode session.update: {error}"),
    })
}

// ── the walk ─────────────────────────────────────────────────────────────────

/// Walk this provider's candidates and hand back a transport that has already
/// proved itself.
///
/// The returned [`Transport`] is not the raw socket: it replays the frame the
/// probe consumed, normalises Binary framing to Text, performs the in-session
/// GA → pre-GA repair, and drops the relay's self-inflicted active-response
/// conflicts. Everything above it — the session machine, the Gateway, a client —
/// sees one well-behaved OpenAI Realtime endpoint.
///
/// # Errors
///
/// [`RealtimeError::Transport`] carrying every rejection the walk collected, or
/// the first unusable URL / header, which would fail identically on every
/// candidate.
pub async fn connect(
    provider: &OpenAiRealtimeProvider,
    agent_context: &AgentContext,
) -> Result<Transport, RealtimeError> {
    connect_within(provider, agent_context, ConnectBudget::DEFAULT).await
}

/// [`connect`], with the two timers named.
///
/// The budget is a parameter rather than two constants for one reason and one
/// convenience. The reason: **a caller's own connect timeout can be shorter than
/// the walk's**, and when it is, aborting the walk from outside throws away the
/// rejection list — the only record of how far it got. [`ConnectBudget::within`]
/// makes the walk finish inside the caller's window and *report*. The
/// convenience is that both timers become testable in milliseconds rather than
/// in twelve seconds of suite time.
///
/// # Errors
///
/// As [`connect`].
pub async fn connect_within(
    provider: &OpenAiRealtimeProvider,
    agent_context: &AgentContext,
    budget: ConnectBudget,
) -> Result<Transport, RealtimeError> {
    // A reconnect must not configure the new socket for the endpoint the old one
    // happened to land on.
    provider.forget_connected();

    let host = provider.host();
    tracing::info!(
        target: LOG_TARGET,
        "[live] dialect={} host={host}",
        provider.settings().dialect
    );

    let candidates = provider.candidate_urls();
    let headers = provider.headers();
    let setup_ga = session_update_frame(provider, RealtimeSchema::Ga, agent_context)?;
    let setup_preview = session_update_frame(provider, RealtimeSchema::Preview, agent_context)?;

    // Sized off the GA payload — the two differ by a few bytes of key names, and
    // GA is what every live route now speaks.
    tracing::debug!(
        target: LOG_TARGET,
        "[live] session.update is {} bytes for {} tool(s)",
        setup_ga.len(),
        agent_context.tools.len()
    );
    if setup_ga.len() > SESSION_UPDATE_SIZE_WARN {
        tracing::warn!(
            target: LOG_TARGET,
            "[live] WARNING session.update is {} KB ({} tools), over the {} KB advisory limit — \
             an oversized configuration can be dropped silently, which presents as a session \
             that never answers",
            setup_ga.len() / 1024,
            agent_context.tools.len(),
            SESSION_UPDATE_SIZE_WARN / 1024
        );
    }

    let mut rejections: Vec<String> = Vec::new();
    let started = tokio::time::Instant::now();

    for (index, url) in candidates.iter().enumerate() {
        tracing::info!(
            target: LOG_TARGET,
            "[live] attempt {}/{} ({}s elapsed): {url}",
            index + 1,
            candidates.len(),
            started.elapsed().as_secs()
        );
        let left = budget.total.saturating_sub(started.elapsed());
        if left.is_zero() {
            push_rejection(
                &mut rejections,
                format!(
                    "gave up before trying {url}: {}ms connect budget exhausted",
                    budget.total.as_millis()
                ),
            );
            break;
        }

        let mut request =
            url.as_str()
                .into_client_request()
                .map_err(|error| RealtimeError::Transport {
                    detail: format!("bad url {url}: {error}"),
                })?;
        for (name, value) in &headers {
            let name = HeaderName::from_bytes(name.as_bytes()).map_err(|error| {
                RealtimeError::Transport {
                    detail: format!("bad header name: {error}"),
                }
            })?;
            let value = HeaderValue::from_str(value).map_err(|error| RealtimeError::Transport {
                detail: format!("bad header value: {error}"),
            })?;
            request.headers_mut().insert(name, value);
        }

        // The handshake is bounded too, by the same clock the probe is. A peer
        // that accepts the TCP connection and never answers the upgrade is the
        // same failure as one that upgrades and never speaks — and `connect_async`
        // has no timeout of its own, so without this a single hung handshake
        // outlives the whole budget and the promise that connect is bounded is
        // not one.
        let handshake = budget.first_frame.min(left);
        let socket = match tokio::time::timeout(
            handshake,
            tokio_tungstenite::connect_async(request),
        )
        .await
        {
            Err(_elapsed) => {
                push_rejection(
                    &mut rejections,
                    format!(
                        "{url} → the upgrade did not complete within {}ms",
                        handshake.as_millis()
                    ),
                );
                continue;
            }
            Ok(Ok((socket, _response))) => socket,
            Ok(Err(error)) => {
                let http_rejection =
                    matches!(error, tokio_tungstenite::tungstenite::Error::Http(_));
                push_rejection(
                    &mut rejections,
                    format!("{url} → {}", describe_ws_error(&error)),
                );
                // Retry ONLY on an HTTP rejection: that is the route /
                // api-version / credential surface, which is what differs
                // between candidates. A transport fault (TLS, DNS, refused
                // connection) is a property of the host and would fail
                // identically for every candidate, so retrying would only delay
                // the report.
                if !http_rejection {
                    break;
                }
                continue;
            }
        };

        let (mut write, mut read) = socket.split();
        let schema = protocol_for_url(url);
        let setup = match schema {
            RealtimeSchema::Ga => &setup_ga,
            RealtimeSchema::Preview => &setup_preview,
        };
        tracing::info!(target: LOG_TARGET, "[live] schema={schema} for {url}");
        if let Err(error) = write.send(Message::Text(setup.clone().into())).await {
            push_rejection(
                &mut rejections,
                format!("{url} → session.update send failed: {error}"),
            );
            continue;
        }

        // The deadline is computed ONCE, outside the loop.
        let deadline = tokio::time::Instant::now() + budget.first_frame;
        let accepted = loop {
            match tokio::time::timeout_at(deadline, read.next()).await {
                Ok(Some(Ok(frame))) => match classify_probe_frame(&frame) {
                    ProbeVerdict::KeepWaiting => continue,
                    ProbeVerdict::Accepted { kind } => {
                        tracing::info!(
                            target: LOG_TARGET,
                            "[live] probe accepted {url} on type={kind} ({} frame)",
                            frame_kind(&frame)
                        );
                        break Some(frame);
                    }
                    ProbeVerdict::NotProtocol { excerpt } => {
                        push_rejection(
                            &mut rejections,
                            format!(
                                "{url} → first {} frame is not a protocol event: {excerpt}",
                                frame_kind(&frame)
                            ),
                        );
                        break None;
                    }
                },
                Ok(Some(Err(error))) => {
                    push_rejection(
                        &mut rejections,
                        format!("{url} → first read failed: {error}"),
                    );
                    break None;
                }
                Ok(None) => {
                    push_rejection(
                        &mut rejections,
                        format!("{url} → closed without sending anything"),
                    );
                    break None;
                }
                Err(_elapsed) => {
                    push_rejection(
                        &mut rejections,
                        format!(
                            "{url} → upgraded but sent no protocol frame within {}ms \
                             (keepalives do not count); a gateway that accepts the socket \
                             without routing it upstream looks exactly like this",
                            budget.first_frame.as_millis()
                        ),
                    );
                    break None;
                }
            }
        };
        let Some(first_frame) = accepted else {
            continue;
        };

        tracing::info!(target: LOG_TARGET, "[live] connected via {url}");
        provider.note_connected(url);
        return Ok(wire(write, read, first_frame, url, setup_preview));
    }

    Err(connect_failure(&rejections))
}

/// Wrap a proved socket in the transport `via-realtime` is handed.
///
/// Generic over the two halves so the wrapper's own behaviour — the replay, the
/// repair, the conflict filter, the Binary normalisation — is testable without a
/// TCP listener.
pub fn wire<W, R, E>(
    write: W,
    read: R,
    first_frame: Message,
    url: &str,
    preview_setup: String,
) -> Transport
where
    W: Sink<Message> + Unpin + Send + 'static,
    W::Error: core::fmt::Display,
    R: Stream<Item = Result<Message, E>> + Unpin + Send + 'static,
    E: core::fmt::Display + Send + 'static,
{
    let creates = CreateAccounting::new();
    let (outbound, outbound_rx) = futures::channel::mpsc::unbounded::<Message>();
    let (inbound_tx, inbound_rx) = futures::channel::mpsc::unbounded::<Result<Message, String>>();
    let repair = outbound.clone();

    tokio::spawn(write_frames(write, outbound_rx));
    tokio::spawn(read_frames(ReadContext {
        read,
        first_frame,
        inbound: inbound_tx,
        repair,
        preview_setup,
        filter: InboundFilter::new(url, creates.clone()),
    }));

    // The counting rides on the write half rather than on the session, because
    // the ledger has to see every `response.create` the session emits — including
    // the ones the busy-retry ladder replays.
    let sink = outbound.with(move |message: Message| {
        note_outbound(&creates, &message);
        futures::future::ready(Ok::<Message, futures::channel::mpsc::SendError>(message))
    });
    Transport::new(sink, inbound_rx)
}

/// Log one outbound frame and count it if it is a `response.create`.
fn note_outbound(creates: &CreateAccounting, message: &Message) {
    let Message::Text(text) = message else {
        return;
    };
    let kind = serde_json::from_str::<Value>(text)
        .ok()
        .and_then(|value| value.get("type").and_then(Value::as_str).map(str::to_owned))
        .unwrap_or_else(|| "<unparsed>".to_owned());
    if kind == "response.create" {
        creates.sent();
    }
    // Audio is skipped — it is 50 frames a second and says nothing. Everything
    // else is logged, because the asymmetry cost ARGO a debugging round: a
    // provider complained about a client event and the log could not show
    // whether one had in fact been sent.
    if kind != "input_audio_buffer.append" {
        tracing::debug!(target: LOG_TARGET, "[live] outbound type={kind}");
    }
}

async fn write_frames<W>(
    mut write: W,
    mut outbound: futures::channel::mpsc::UnboundedReceiver<Message>,
) where
    W: Sink<Message> + Unpin,
{
    while let Some(message) = outbound.next().await {
        let is_close = matches!(message, Message::Close(_));
        if write.send(message).await.is_err() || is_close {
            let _ = write.close().await;
            return;
        }
    }
    let _ = write.close().await;
}

/// Everything the reader task owns.
struct ReadContext<R, E>
where
    R: Stream<Item = Result<Message, E>> + Unpin,
{
    read: R,
    first_frame: Message,
    inbound: futures::channel::mpsc::UnboundedSender<Result<Message, String>>,
    repair: futures::channel::mpsc::UnboundedSender<Message>,
    preview_setup: String,
    filter: InboundFilter,
}

async fn read_frames<R, E>(mut context: ReadContext<R, E>)
where
    R: Stream<Item = Result<Message, E>> + Unpin,
    E: core::fmt::Display,
{
    // The liveness probe already consumed the first frame; replay it here so
    // nothing is lost. On a real endpoint it is `session.created`, which the
    // session machine answers with its own `session.update` — but it could as
    // easily be an `error` explaining a rejected configuration, and dropping
    // that would turn a diagnosable failure back into silence.
    let mut pending = Some(Ok(context.first_frame));
    loop {
        let frame = match pending.take() {
            Some(frame) => frame,
            None => match context.read.next().await {
                Some(frame) => frame.map_err(|error| error.to_string()),
                None => break,
            },
        };
        let message = match frame {
            Ok(message) => message,
            Err(detail) => {
                let _ = context.inbound.unbounded_send(Err(detail));
                return;
            }
        };
        if matches!(message, Message::Close(_)) {
            let _ = context.inbound.unbounded_send(Ok(message));
            return;
        }
        // Text OR Binary — a relay is free to frame the session either way, and
        // the one this was written for uses Binary.
        let Some(text) = protocol_text(&message).map(str::to_owned) else {
            continue;
        };
        let Ok(event) = serde_json::from_str::<Value>(&text) else {
            // Not JSON at all. Forwarded rather than dropped: `via-realtime`
            // skips what it cannot parse, and swallowing it here would hide a
            // relay that started emitting something else.
            let _ = context
                .inbound
                .unbounded_send(Ok(Message::Text(text.into())));
            continue;
        };
        // Log the type of EVERY frame, including the control events that decode
        // to nothing. Without this, "the server said nothing" and "the server
        // spoke a vocabulary we ignore" are the same observation — and they need
        // opposite fixes. The frame kind rides along for the same reason.
        let kind = event
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("<none>")
            .to_owned();
        tracing::debug!(
            target: LOG_TARGET,
            "[live] inbound type={kind} ({})",
            frame_kind(&message)
        );
        match context.filter.apply(&event) {
            // Forwarded byte for byte, in Text framing: the session machine
            // reads both, and normalising here is what keeps the gateway's
            // Binary framing from being something every layer above has to know
            // about.
            InboundVerdict::Forward => {
                if context
                    .inbound
                    .unbounded_send(Ok(Message::Text(text.into())))
                    .is_err()
                {
                    return;
                }
            }
            InboundVerdict::RepairSchema => {
                if context
                    .repair
                    .unbounded_send(Message::Text(context.preview_setup.clone().into()))
                    .is_err()
                {
                    let _ = context
                        .inbound
                        .unbounded_send(Err("schema retry could not be sent".to_owned()));
                    return;
                }
            }
            InboundVerdict::DropRepaired | InboundVerdict::DropConflict => {}
        }
    }
}

// ── the front door ───────────────────────────────────────────────────────────

/// Walk the candidates and configure a session on whichever one answers.
///
/// The order is `via-realtime`'s and it matters:
/// [`preflight`](RealtimeProvider::preflight), then
/// [`is_configured`](RealtimeProvider::is_configured), then the socket — a
/// session that is going to fail for a known reason never opens one.
///
/// The walk's own [`CONNECT_TOTAL_BUDGET`] bounds the candidates; whatever is
/// left of [`CONNECT_TIMEOUT`](via_realtime::CONNECT_TIMEOUT) covers the
/// `session.created` / `session.update` / `session.updated` handshake on top.
///
/// # Errors
///
/// [`RealtimeError::NotConfigured`] before any socket;
/// [`RealtimeError::ConnectTimeout`] when the budget runs out;
/// [`RealtimeError::Transport`] carrying every candidate's rejection; and
/// whatever [`RealtimeSession::open`] refuses.
pub async fn open_session(
    provider: Arc<OpenAiRealtimeProvider>,
    options: SessionOptions,
) -> Result<(RealtimeSession, SessionEvents), RealtimeError> {
    provider.preflight()?;
    if !provider.is_configured() {
        return Err(RealtimeError::NotConfigured {
            provider: provider.key().to_owned(),
            message: provider.missing_configuration_message(options.locale),
        });
    }

    let budget = options
        .connect_timeout
        .unwrap_or(via_realtime::CONNECT_TIMEOUT);
    let started = tokio::time::Instant::now();
    // The walk gets a budget that fits inside the caller's, so a short deadline
    // produces the walk's own report — every candidate with what it actually
    // said — rather than a bare timeout. The outer `timeout` stays as the
    // backstop for anything the walk cannot bound itself.
    let walk = connect_within(
        &provider,
        &options.agent_context,
        ConnectBudget::within(budget),
    );
    let transport = match tokio::time::timeout(budget, walk).await {
        Ok(result) => result?,
        Err(_elapsed) => {
            return Err(RealtimeError::ConnectTimeout {
                provider: provider.key().to_owned(),
                message: provider.connect_timeout_message(options.locale),
            });
        }
    };

    let mut options = options;
    options.connect_timeout = Some(budget.saturating_sub(started.elapsed()));
    let dynamic: Arc<dyn RealtimeProvider> = provider;
    RealtimeSession::open(dynamic, options, transport).await
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use tokio_tungstenite::tungstenite::Error as WsError;

    use super::*;

    fn text(value: &Value) -> Message {
        Message::Text(value.to_string().into())
    }

    #[test]
    fn a_binary_frame_carries_protocol_text_exactly_as_a_text_frame_does() {
        let body = json!({ "type": "session.created" }).to_string();
        assert_eq!(
            protocol_text(&Message::Text(body.clone().into())),
            Some(body.as_str())
        );
        assert_eq!(
            protocol_text(&Message::Binary(body.clone().into_bytes().into())),
            Some(body.as_str())
        );
    }

    #[test]
    fn a_keepalive_carries_no_protocol_text() {
        for message in [
            Message::Ping(Vec::new().into()),
            Message::Pong(Vec::new().into()),
            Message::Close(None),
            // Binary that is not UTF-8 is bytes, not a protocol frame.
            Message::Binary(vec![0xff, 0xfe].into()),
        ] {
            assert_eq!(protocol_text(&message), None, "{:?}", frame_kind(&message));
        }
    }

    #[test]
    fn the_frame_kind_names_what_a_relay_did() {
        assert_eq!(frame_kind(&Message::Text("x".into())), "text");
        assert_eq!(frame_kind(&Message::Binary(vec![1].into())), "binary");
        assert_eq!(frame_kind(&Message::Ping(Vec::new().into())), "ping");
        assert_eq!(frame_kind(&Message::Pong(Vec::new().into())), "pong");
        assert_eq!(frame_kind(&Message::Close(None)), "close");
    }

    #[test]
    fn the_probe_accepts_a_protocol_event_in_either_framing() {
        let event = json!({ "type": "session.created", "session": {} });
        for message in [
            Message::Text(event.to_string().into()),
            Message::Binary(event.to_string().into_bytes().into()),
        ] {
            assert_eq!(
                classify_probe_frame(&message),
                ProbeVerdict::Accepted {
                    kind: "session.created".to_owned()
                },
                "{}",
                frame_kind(&message)
            );
        }
    }

    #[test]
    fn the_probe_keeps_waiting_through_keepalives() {
        // A Ping satisfied an earlier version of this check and kept a dead
        // candidate alive. It must never accept.
        for message in [
            Message::Ping(Vec::new().into()),
            Message::Pong(b"keepalive".to_vec().into()),
            Message::Binary(vec![0x80].into()),
        ] {
            assert_eq!(classify_probe_frame(&message), ProbeVerdict::KeepWaiting);
        }
    }

    #[test]
    fn the_probe_rejects_text_that_is_not_a_protocol_event() {
        // A proxy's HTML error page, a bare JSON array, an object with no
        // `type`: all decode, none proves the protocol.
        for body in [
            "<html>502 Bad Gateway</html>",
            "[]",
            r#"{"ok":true}"#,
            "\"session.created\"",
            r#"{"type":42}"#,
        ] {
            let verdict = classify_probe_frame(&Message::Text(body.into()));
            assert!(
                matches!(verdict, ProbeVerdict::NotProtocol { .. }),
                "{body} → {verdict:?}"
            );
        }
    }

    #[test]
    fn a_long_non_protocol_frame_is_excerpted_on_a_character_boundary() {
        let body = "한".repeat(PROBE_EXCERPT_CHARS * 2);
        let ProbeVerdict::NotProtocol { excerpt } =
            classify_probe_frame(&Message::Text(body.into()))
        else {
            panic!("expected a rejection");
        };
        assert_eq!(excerpt.chars().count(), PROBE_EXCERPT_CHARS);
    }

    #[test]
    fn the_filter_forwards_an_ordinary_event() {
        let mut filter = InboundFilter::new("wss://f.invalid/v1/realtime", CreateAccounting::new());
        assert_eq!(
            filter.apply(&json!({ "type": "response.output_audio.delta", "delta": "QUJD" })),
            InboundVerdict::Forward
        );
    }

    #[test]
    fn the_filter_repairs_the_schema_once_and_then_only_swallows() {
        let url = "wss://filter-repair-once.invalid/v1/realtime?model=m";
        let mut filter = InboundFilter::new(url, CreateAccounting::new());
        let rejection = json!({
            "type": "error",
            "error": { "message": "Unknown parameter: 'session.type'." }
        });

        assert_eq!(filter.apply(&rejection), InboundVerdict::RepairSchema);
        assert!(filter.repaired());
        // The memo is the consequence of the repair, not a second thing to keep
        // in sync with it.
        assert_eq!(protocol_for_url(url), RealtimeSchema::Preview);

        // Once, not in a loop: the second rejection resends nothing.
        assert_eq!(filter.apply(&rejection), InboundVerdict::DropRepaired);
        assert_eq!(filter.apply(&rejection), InboundVerdict::DropRepaired);
    }

    #[test]
    fn the_filter_drops_a_conflict_the_client_did_not_cause() {
        let mut filter = InboundFilter::new(
            "wss://filter-conflict.invalid/v1/realtime",
            CreateAccounting::new(),
        );
        // The bring-up §8b shape: server VAD opens a response, then the relay's
        // own duplicate bounces off it.
        assert_eq!(
            filter.apply(&json!({ "type": "response.created", "response": { "id": "resp_x" } })),
            InboundVerdict::Forward
        );
        assert_eq!(
            filter.apply(&json!({
                "type": "error",
                "error": { "message": "Conversation already has an active response in progress: resp_x." }
            })),
            InboundVerdict::DropConflict
        );
    }

    #[test]
    fn the_filter_surfaces_a_conflict_that_could_be_ours() {
        let creates = CreateAccounting::new();
        let mut filter =
            InboundFilter::new("wss://filter-ours.invalid/v1/realtime", creates.clone());
        creates.sent();
        assert_eq!(
            filter.apply(&json!({
                "type": "error",
                "error": { "code": "conversation_already_has_active_response" }
            })),
            InboundVerdict::Forward,
            "a client regression that double-sends must stay visible"
        );
        // And it consumed its slot, so the next one is noise again.
        assert_eq!(
            filter.apply(&json!({
                "type": "error",
                "error": { "code": "conversation_already_has_active_response" }
            })),
            InboundVerdict::DropConflict
        );
    }

    #[test]
    fn a_server_vad_response_never_makes_a_later_conflict_look_like_ours() {
        let creates = CreateAccounting::new();
        let mut filter =
            InboundFilter::new("wss://filter-vad.invalid/v1/realtime", creates.clone());
        for _ in 0..4 {
            filter.apply(&json!({ "type": "response.created" }));
        }
        assert_eq!(creates.in_flight(), 0, "the decrement is saturating");
        creates.sent();
        assert_eq!(
            filter.apply(&json!({
                "type": "error",
                "error": { "message": "already has an active response" }
            })),
            InboundVerdict::Forward
        );
    }

    #[test]
    fn an_outbound_response_create_is_counted_and_nothing_else_is() {
        let creates = CreateAccounting::new();
        note_outbound(&creates, &text(&json!({ "type": "session.update" })));
        note_outbound(
            &creates,
            &text(&json!({ "type": "input_audio_buffer.append", "audio": "QUJD" })),
        );
        note_outbound(&creates, &Message::Binary(vec![1, 2].into()));
        note_outbound(&creates, &Message::Text("not json".into()));
        assert_eq!(creates.in_flight(), 0);

        note_outbound(&creates, &text(&json!({ "type": "response.create" })));
        note_outbound(
            &creates,
            &text(&json!({ "event_id": "event_x", "type": "response.create", "response": {} })),
        );
        assert_eq!(creates.in_flight(), 2);
    }

    #[test]
    fn the_connect_failure_names_every_attempt() {
        let error = connect_failure(&["a → 403".to_owned(), "b → silent".to_owned()]);
        let RealtimeError::Transport { detail } = error else {
            panic!("expected a transport failure");
        };
        assert!(detail.contains("2 attempt(s)"), "{detail}");
        assert!(detail.contains("a → 403"), "{detail}");
        assert!(detail.contains("b → silent"), "{detail}");
    }

    fn http_error(status: u16, body: Option<&str>, ms_code: Option<&str>) -> WsError {
        let mut builder = tokio_tungstenite::tungstenite::http::Response::builder().status(status);
        if let Some(code) = ms_code {
            builder = builder.header(AZURE_ERROR_CODE_HEADER, code);
        }
        WsError::Http(Box::new(
            builder
                .body(body.map(|text| text.as_bytes().to_vec()))
                .expect("a response builds"),
        ))
    }

    #[test]
    fn the_connect_error_quotes_the_response_body() {
        // The whole point: tungstenite's own Display prints the status and drops
        // the body, so a 403 is unattributable without this.
        let error = http_error(
            403,
            Some(r#"{"error":{"code":"AuthorizationFailed"}}"#),
            None,
        );
        let described = describe_ws_error(&error);
        assert!(described.contains("403"), "{described}");
        assert!(described.contains("AuthorizationFailed"), "{described}");
        // Prove the default rendering really is lossy — this is the regression
        // guard, not decoration.
        assert!(!error.to_string().contains("AuthorizationFailed"));
    }

    #[test]
    fn the_connect_error_surfaces_the_azure_error_code_header() {
        let described =
            describe_ws_error(&http_error(403, None, Some("PublicNetworkAccessDisabled")));
        assert!(
            described.contains("PublicNetworkAccessDisabled"),
            "{described}"
        );
        // An empty body must say so rather than trailing a bare colon.
        assert!(described.contains("empty body"), "{described}");
        assert!(!described.ends_with(": "), "{described}");
    }

    #[test]
    fn a_blank_azure_error_code_is_not_rendered_as_empty_brackets() {
        let described = describe_ws_error(&http_error(403, None, Some("   ")));
        assert!(!described.contains("[]"), "{described}");
        assert!(!described.contains("[ "), "{described}");
    }

    #[test]
    fn the_connect_error_truncates_a_huge_body_on_a_char_boundary() {
        let described = describe_ws_error(&http_error(
            500,
            Some(&"x".repeat(CONNECT_ERROR_BODY_LIMIT * 3)),
            None,
        ));
        assert!(
            described.contains("truncated"),
            "should say it cut the body"
        );
        assert!(described.len() < CONNECT_ERROR_BODY_LIMIT * 2);

        // Every char here is 3 bytes, so the limit lands mid-sequence for at
        // least one offset. Slicing there would panic.
        let described = describe_ws_error(&http_error(
            500,
            Some(&"한".repeat(CONNECT_ERROR_BODY_LIMIT)),
            None,
        ));
        assert!(described.contains("truncated"), "{described}");
    }

    #[test]
    fn the_connect_error_passes_non_http_errors_through() {
        let error = WsError::Utf8("bad".into());
        assert_eq!(describe_ws_error(&error), error.to_string());
    }

    #[test]
    fn the_ordinary_connect_timeout_yields_argos_own_two_constants() {
        // `via-realtime`'s 25 s window has room for the whole walk plus one more
        // probe, so nothing is scaled on the path every session takes.
        assert_eq!(
            ConnectBudget::within(via_realtime::CONNECT_TIMEOUT),
            ConnectBudget::DEFAULT
        );
        assert_eq!(ConnectBudget::default(), ConnectBudget::DEFAULT);
        assert_eq!(
            ConnectBudget::within(CONNECT_TOTAL_BUDGET + FIRST_FRAME_TIMEOUT),
            ConnectBudget::DEFAULT
        );
    }

    #[test]
    fn a_short_deadline_scales_both_timers_and_leaves_room_for_the_report() {
        let budget = ConnectBudget::within(Duration::from_millis(400));
        assert_eq!(budget.total, Duration::from_millis(300));
        assert_eq!(budget.first_frame, Duration::from_millis(60));
        // Every candidate can spend its whole window and the walk still ends
        // inside its own ceiling — which is inside the caller's.
        assert!(budget.first_frame * MAX_CANDIDATES < budget.total);
        assert!(budget.total < Duration::from_millis(400));
    }

    #[test]
    fn a_zero_deadline_still_dials_once_rather_than_reporting_nothing() {
        // "Exhausted before anything was tried" is a worse answer than one
        // attempt that fails honestly.
        let budget = ConnectBudget::within(Duration::ZERO);
        assert!(!budget.total.is_zero());
        assert!(!budget.first_frame.is_zero());
    }

    #[test]
    fn the_budget_sits_inside_the_sessions_own_connect_timeout() {
        // The walk must leave room for `session.created` /
        // `session.update` / `session.updated` on top of it.
        assert!(CONNECT_TOTAL_BUDGET < via_realtime::CONNECT_TIMEOUT);
        assert!(FIRST_FRAME_TIMEOUT < CONNECT_TOTAL_BUDGET);
        // Four candidates at three seconds each is twelve, which is the budget:
        // the ceiling exists so the walk cannot cost more than the arithmetic.
        assert_eq!(FIRST_FRAME_TIMEOUT * 4, CONNECT_TOTAL_BUDGET);
    }
}
