//! The four documented gateway misbehaviours, against a real socket.
//!
//! ARGO bring-up §4 lists four properties of the litellm gateway, and each one
//! silently broke something for days. Every one of them is a **server**
//! behaviour, so none can be reproduced with a provider double — these tests
//! stand up a `TcpListener` on port 0 and tell the peer to misbehave in one
//! named way at a time.
//!
//! Real timers, not `start_paused`: the first-frame probe races a peer that is
//! genuinely pinging on another task, and a paused clock would let the probe win
//! by construction rather than by being right.

mod common;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use common::{Behaviour, FakeGateway, Rejection};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use via_core::Secret;
use via_realtime::{
    AgentContext, RealtimeError, RealtimeProvider, SessionEvent, SessionEvents, SessionOptions,
};
use via_realtime_openai::{
    CONNECT_TOTAL_BUDGET, ConnectBudget, FIRST_FRAME_TIMEOUT, OpenAiDialect,
    OpenAiRealtimeProvider, OpenAiSettings, RealtimeSchema, connect, connect_within, open_session,
    protocol_for_url,
};

/// A provider pointed at a fake gateway, in the four-candidate dialect.
fn provider_for(gateway: &FakeGateway) -> Arc<OpenAiRealtimeProvider> {
    Arc::new(OpenAiRealtimeProvider::new(OpenAiSettings {
        base_url: gateway.base_url(),
        api_key: Secret::new("sk-test"),
        dialect: OpenAiDialect::Azure,
        model: "gpt-realtime-2.1-mini".to_owned(),
        ..OpenAiSettings::default()
    }))
}

fn options() -> SessionOptions {
    SessionOptions {
        agent_context: AgentContext {
            instructions: "be brief".to_owned(),
            ..AgentContext::default()
        },
        ..SessionOptions::default()
    }
}

/// Drain whatever the session has already emitted, without waiting for more.
async fn drain(events: &mut SessionEvents, within: Duration) -> Vec<SessionEvent> {
    let mut seen = Vec::new();
    let deadline = tokio::time::Instant::now() + within;
    loop {
        let left = deadline.saturating_duration_since(tokio::time::Instant::now());
        if left.is_zero() {
            return seen;
        }
        match tokio::time::timeout(left, events.recv()).await {
            Ok(Some(event)) => seen.push(event),
            Ok(None) | Err(_) => return seen,
        }
    }
}

fn errors(events: &[SessionEvent]) -> Vec<&RealtimeError> {
    events
        .iter()
        .filter_map(|event| match event {
            SessionEvent::Error(error) => Some(error),
            _ => None,
        })
        .collect()
}

fn session_updates(gateway: &FakeGateway) -> Vec<Value> {
    gateway.client_frames_of("session.update")
}

fn carries_discriminator(frame: &Value) -> bool {
    frame
        .get("session")
        .and_then(|session| session.get("type"))
        .is_some()
}

// ── the happy path, and what the upgrade carries ─────────────────────────────

#[tokio::test]
async fn a_healthy_endpoint_configures_a_session() {
    let gateway = FakeGateway::healthy().await;
    let provider = provider_for(&gateway);

    let (session, mut events) = open_session(Arc::clone(&provider), options())
        .await
        .expect("the session opens");

    assert_eq!(session.provider().key(), "openai");
    // The walk committed to the first candidate — a gateway serves OpenAI's own
    // shape whatever it forwards to.
    assert!(
        provider.session_url().contains("/v1/realtime?model="),
        "{}",
        provider.session_url()
    );
    assert!(errors(&drain(&mut events, Duration::from_millis(150)).await).is_empty());
    session.close().await.expect("closes");
}

#[tokio::test]
async fn the_upgrade_offers_the_realtime_subprotocol_and_the_credential() {
    // The header that decides whether a gateway routes the socket at all. A
    // regression presents as a socket that upgrades and then never speaks —
    // the hardest shape to diagnose, because nothing errors.
    let gateway = FakeGateway::healthy().await;
    let (session, _events) = open_session(provider_for(&gateway), options())
        .await
        .expect("the session opens");
    session.close().await.expect("closes");

    let request = gateway
        .requests()
        .into_iter()
        .next()
        .expect("one upgrade was seen");
    assert_eq!(request.header("sec-websocket-protocol"), Some("realtime"));
    // A gateway in front of Azure gets both credentials; neither may be
    // smuggled in the subprotocol name, which every proxy on the path logs.
    assert_eq!(request.header("api-key"), Some("sk-test"));
    assert_eq!(request.header("authorization"), Some("Bearer sk-test"));
    assert!(
        !request
            .header("sec-websocket-protocol")
            .unwrap_or_default()
            .contains("sk-test")
    );
    // The beta interface shut down 2026-05-12.
    assert!(!request.has_header("openai-beta"), "{request:?}");
}

#[tokio::test]
async fn a_gateway_that_refuses_the_subprotocol_route_never_gets_a_session() {
    // The peer here is stricter than the real one on purpose: the real gateway
    // accepted the socket and silently failed to route it, which no test can
    // distinguish from a silent route. Refusing makes the assertion be about the
    // header, which is the thing that must not regress.
    let gateway = FakeGateway::spawn(
        HashMap::new(),
        Behaviour {
            require_subprotocol: true,
            ..Behaviour::default()
        },
    )
    .await;
    let (session, _events) = open_session(provider_for(&gateway), options())
        .await
        .expect("the offered subprotocol satisfies the route");
    session.close().await.expect("closes");
}

// ── 1. Binary frames, not Text ───────────────────────────────────────────────

#[tokio::test]
async fn a_gateway_that_frames_the_session_as_binary_is_understood() {
    // Days of "connected but silent": `session.created` arrived as 1254 bytes of
    // opcode 2, and a reader that accepted only Text discarded every frame.
    let gateway = FakeGateway::spawn(
        HashMap::new(),
        Behaviour {
            binary_frames: true,
            ..Behaviour::default()
        },
    )
    .await;

    let (session, mut events) = open_session(provider_for(&gateway), options())
        .await
        .expect("a Binary-framed session is still a session");
    assert!(errors(&drain(&mut events, Duration::from_millis(150)).await).is_empty());
    session.close().await.expect("closes");
}

#[tokio::test]
async fn a_binary_frame_that_is_not_utf8_does_not_satisfy_the_probe() {
    // Binary is a framing choice, not a content type — but bytes that are not
    // UTF-8 are not this protocol either, and accepting them would put the
    // 101-is-not-a-session hole back.
    use tokio_tungstenite::tungstenite::Message;
    use via_realtime_openai::{ProbeVerdict, classify_probe_frame};

    assert_eq!(
        classify_probe_frame(&Message::Binary(vec![0xff, 0xff].into())),
        ProbeVerdict::KeepWaiting
    );
}

// ── 4. A completed 101 is not a session, and pings do not count ──────────────

#[tokio::test]
async fn a_socket_that_only_pings_is_rejected_and_the_walk_moves_on() {
    // The property that ended the candidate walk on a dead endpoint: a relay
    // accepts the socket on a path it does not route, and pings forever. The
    // first candidate for a gateway host is `/v1/realtime`; the second is
    // `/openai/v1/realtime`.
    let gateway = FakeGateway::spawn(
        HashMap::from([
            (
                "/v1/realtime",
                Behaviour {
                    silent: true,
                    ..Behaviour::default()
                },
            ),
            ("/openai/v1/realtime", Behaviour::healthy()),
        ]),
        Behaviour {
            reject: Some(Rejection::status(404)),
            ..Behaviour::default()
        },
    )
    .await;

    let provider = provider_for(&gateway);
    let started = std::time::Instant::now();
    // Bounded on purpose. The failure this test exists for — a per-frame idle
    // timeout instead of one deadline — does not make the walk slow, it makes it
    // **never end**, because the peer keeps pinging. An unbounded test would hang
    // rather than fail, and a hang is not a test result.
    let (session, mut events) = tokio::time::timeout(
        CONNECT_TOTAL_BUDGET,
        open_session(Arc::clone(&provider), options()),
    )
    .await
    .expect("the walk must end well inside its own budget")
    .expect("the second candidate answers");
    let elapsed = started.elapsed();

    assert!(
        provider.session_url().contains("/openai/v1/realtime"),
        "the walk must move past the silent route: {}",
        provider.session_url()
    );
    // The pings arrive every 120 ms. A per-frame idle timeout would restart the
    // clock on every one of them and hold the dead candidate open forever; the
    // deadline is computed once, so the probe gives up after three seconds.
    assert!(
        elapsed >= FIRST_FRAME_TIMEOUT,
        "the probe must actually wait: {elapsed:?}"
    );
    assert!(
        elapsed < CONNECT_TOTAL_BUDGET,
        "a pinging gateway must not hold a candidate open: {elapsed:?}"
    );
    assert!(errors(&drain(&mut events, Duration::from_millis(150)).await).is_empty());
    session.close().await.expect("closes");
}

#[tokio::test]
async fn a_walk_where_every_candidate_is_silent_reports_all_four() {
    let gateway = FakeGateway::spawn(
        HashMap::new(),
        Behaviour {
            silent: true,
            ..Behaviour::default()
        },
    )
    .await;

    let provider = provider_for(&gateway);
    let started = std::time::Instant::now();
    // Bounded for the same reason as above: a probe whose clock restarts on
    // every keepalive never returns at all.
    let error = tokio::time::timeout(
        CONNECT_TOTAL_BUDGET + Duration::from_secs(2),
        connect(&provider, &AgentContext::default()),
    )
    .await
    .expect("the walk must end")
    .expect_err("nothing answered");
    let elapsed = started.elapsed();

    let RealtimeError::Transport { detail } = error else {
        panic!("expected a transport failure");
    };
    assert!(detail.contains("keepalives do not count"), "{detail}");
    // Four candidates × a three-second probe is the budget exactly, so the walk
    // either finishes all four or is cut off by the ceiling — never more.
    assert!(
        elapsed <= CONNECT_TOTAL_BUDGET + Duration::from_secs(1),
        "{elapsed:?}"
    );
    assert!(
        detail.contains("attempt(s)"),
        "every attempt is named: {detail}"
    );
}

// ── the handshake-rejection report ───────────────────────────────────────────

#[tokio::test]
async fn every_refused_candidate_is_named_with_what_it_actually_said() {
    // One status code cannot distinguish a wrong route from a wrong api-version
    // from a refused credential from a proxy that never bridged; the full list
    // can.
    let gateway = FakeGateway::spawn(
        HashMap::from([
            (
                "/v1/realtime",
                Behaviour {
                    reject: Some(Rejection::with_body(
                        403,
                        r#"{"error":{"code":"AuthorizationFailed"}}"#,
                    )),
                    ..Behaviour::default()
                },
            ),
            (
                "/openai/v1/realtime",
                Behaviour {
                    reject: Some(Rejection::with_azure_code(
                        403,
                        "PublicNetworkAccessDisabled",
                    )),
                    ..Behaviour::default()
                },
            ),
        ]),
        Behaviour {
            reject: Some(Rejection::status(404)),
            ..Behaviour::default()
        },
    )
    .await;

    let provider = provider_for(&gateway);
    let error = connect(&provider, &AgentContext::default())
        .await
        .expect_err("nothing answered");
    let RealtimeError::Transport { detail } = error else {
        panic!("expected a transport failure");
    };

    assert!(detail.contains("4 attempt(s)"), "{detail}");
    // tungstenite's own Display prints the status and drops the body.
    assert!(detail.contains("AuthorizationFailed"), "{detail}");
    // Azure puts the reason in a header and sometimes returns no body at all.
    assert!(detail.contains("PublicNetworkAccessDisabled"), "{detail}");
    assert!(detail.contains("empty body"), "{detail}");
    assert!(detail.contains("HTTP 404"), "{detail}");
    // And the whole thing classifies as fatal, so the Gateway stops retrying a
    // credential that will never work.
    assert_eq!(
        provider.classify_error(&detail),
        via_realtime::ErrorClass::Fatal
    );
}

#[tokio::test]
async fn a_transport_fault_stops_the_walk_instead_of_repeating_itself() {
    // A refused connection is a property of the host and would fail identically
    // for every candidate, so retrying only delays the report.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a port");
    let addr = listener.local_addr().expect("an address");
    drop(listener);

    let provider = OpenAiRealtimeProvider::new(OpenAiSettings {
        base_url: format!("ws://{addr}"),
        api_key: Secret::new("sk-test"),
        dialect: OpenAiDialect::Azure,
        ..OpenAiSettings::default()
    });
    let error = connect(&provider, &AgentContext::default())
        .await
        .expect_err("nothing is listening");
    let RealtimeError::Transport { detail } = error else {
        panic!("expected a transport failure");
    };
    assert!(
        detail.contains("1 attempt(s)"),
        "one refused connection, not four: {detail}"
    );
}

// ── 3. GA vs pre-GA, and the in-session repair ───────────────────────────────

#[tokio::test]
async fn a_pre_ga_endpoint_on_gas_own_route_is_repaired_in_session() {
    // The route says GA. The endpoint says `Unknown parameter: 'session.type'.`
    // The endpoint wins, on the same socket, without the user seeing anything.
    let gateway = FakeGateway::spawn(
        HashMap::new(),
        Behaviour {
            reject_ga_schema: true,
            ..Behaviour::default()
        },
    )
    .await;

    let provider = provider_for(&gateway);
    let (session, mut events) = open_session(Arc::clone(&provider), options())
        .await
        .expect("the repair keeps the session alive");

    let seen = drain(&mut events, Duration::from_millis(200)).await;
    assert!(
        errors(&seen).is_empty(),
        "the rejection is answered, not surfaced: {seen:?}"
    );

    let updates = session_updates(&gateway);
    // Two GA payloads go out, not one: the walk's probe and the session
    // machine's own update on `session.created`. Both bounce, and only the
    // first resends — "once, not in a loop" — so the second rejection has to be
    // swallowed rather than surfaced, which is what the empty error list above
    // proves.
    assert!(
        updates.iter().filter(|u| carries_discriminator(u)).count() >= 2,
        "the GA payload was tried first, twice: {updates:?}"
    );
    assert!(
        updates.iter().any(|update| !carries_discriminator(update)),
        "and the pre-GA payload was resent: {updates:?}"
    );
    // The flat pre-GA shape, not merely a GA payload with the key removed.
    let repaired = updates
        .iter()
        .find(|update| !carries_discriminator(update))
        .expect("a pre-GA update");
    assert_eq!(repaired["session"]["voice"], json!("alloy"));
    assert_eq!(repaired["session"]["modalities"], json!(["text", "audio"]));
    assert!(repaired["session"].get("audio").is_none());

    // The endpoint's verdict is remembered for the process, so the next session
    // opens in the right schema rather than repeating this mid-utterance.
    assert_eq!(
        protocol_for_url(&provider.session_url()),
        RealtimeSchema::Preview
    );
    assert_eq!(provider.schema(), RealtimeSchema::Preview);
    session.close().await.expect("closes");
}

#[tokio::test]
async fn the_repair_happens_once_per_process_rather_than_once_per_session() {
    // The memo is the difference between a reconfiguration that lands in the
    // middle of the user's first sentence and one that never happens again.
    let gateway = FakeGateway::spawn(
        HashMap::new(),
        Behaviour {
            reject_ga_schema: true,
            ..Behaviour::default()
        },
    )
    .await;

    let provider = provider_for(&gateway);
    let (first, _events) = open_session(Arc::clone(&provider), options())
        .await
        .expect("the first session repairs");
    first.close().await.expect("closes");
    let after_first = session_updates(&gateway).len();

    let (second, mut events) = open_session(Arc::clone(&provider), options())
        .await
        .expect("the second session needs no repair");
    let seen = drain(&mut events, Duration::from_millis(200)).await;
    assert!(errors(&seen).is_empty(), "{seen:?}");

    let later: Vec<Value> = session_updates(&gateway).split_off(after_first);
    assert!(!later.is_empty(), "the second session configured itself");
    assert!(
        later.iter().all(|update| !carries_discriminator(update)),
        "the second session must not retry the GA schema: {later:?}"
    );
    second.close().await.expect("closes");
}

#[tokio::test]
async fn the_walk_dials_azures_preview_route_with_the_pre_ga_payload() {
    // The walk varies the ROUTE, and the route is what decides the schema.
    // Holding one payload across candidates would dial every preview route with
    // a GA body it must reject, and the rejection would read as "wrong route".
    let gateway = FakeGateway::spawn(
        HashMap::from([(
            "/openai/realtime",
            Behaviour {
                // A preview endpoint refuses the discriminator, so this proves
                // the walk never sent one here.
                reject_ga_schema: true,
                ..Behaviour::default()
            },
        )]),
        Behaviour {
            reject: Some(Rejection::status(404)),
            ..Behaviour::default()
        },
    )
    .await;

    let provider = provider_for(&gateway);
    let (session, _events) = open_session(Arc::clone(&provider), options())
        .await
        .expect("the classic route answers");
    assert!(
        provider.session_url().contains("/openai/realtime?"),
        "{}",
        provider.session_url()
    );
    let updates = session_updates(&gateway);
    assert!(!updates.is_empty());
    assert!(
        updates.iter().all(|update| !carries_discriminator(update)),
        "a preview route is dialled with the preview payload: {updates:?}"
    );
    session.close().await.expect("closes");
}

// ── 5. The duplicate-`response.create` filter ────────────────────────────────

#[tokio::test]
async fn the_relays_own_active_response_conflict_never_reaches_the_session() {
    // ARGO bring-up §8b: the gateway issues its own duplicate `response.create`
    // against the upstream on each VAD commit, the upstream refuses it, and the
    // gateway relays the refusal to us. Once per utterance, never ours.
    let gateway = FakeGateway::spawn(
        HashMap::new(),
        Behaviour {
            after_ready: vec![
                json!({ "type": "response.created", "response": { "id": "resp_x" } }),
                json!({
                    "type": "error",
                    "error": {
                        "message": "Conversation already has an active response in progress: \
                                    resp_x. Wait until the response is finished before creating \
                                    a new one."
                    }
                }),
            ],
            ..Behaviour::default()
        },
    )
    .await;

    let (session, mut events) = open_session(provider_for(&gateway), options())
        .await
        .expect("the session opens");
    let seen = drain(&mut events, Duration::from_millis(250)).await;

    assert!(
        errors(&seen).is_empty(),
        "the conflict is dropped, not surfaced: {seen:?}"
    );
    // The response the server opened still reaches the session — only the
    // complaint about it is dropped.
    assert!(
        seen.iter().any(|event| matches!(
            event,
            SessionEvent::Provider(provider)
                if provider.event.get("type").and_then(Value::as_str)
                    == Some("response.created")
        )),
        "{seen:?}"
    );
    session.close().await.expect("closes");
}

// ── the connect budget ───────────────────────────────────────────────────────

#[tokio::test]
async fn a_short_deadline_against_a_silent_gateway_still_names_every_candidate() {
    // Not a hang, and not a bare timeout either: the walk finishes inside the
    // caller's window and says what each candidate did.
    let gateway = FakeGateway::spawn(
        HashMap::new(),
        Behaviour {
            silent: true,
            ..Behaviour::default()
        },
    )
    .await;

    let started = std::time::Instant::now();
    let error = open_session(
        provider_for(&gateway),
        SessionOptions {
            connect_timeout: Some(Duration::from_millis(600)),
            ..options()
        },
    )
    .await
    .expect_err("nothing answered inside the budget");
    let elapsed = started.elapsed();

    assert_eq!(error.code(), "VIA_REALTIME_TRANSPORT", "{error:?}");
    let RealtimeError::Transport { detail } = error else {
        panic!("expected a transport failure");
    };
    assert!(detail.contains("keepalives do not count"), "{detail}");
    assert!(detail.contains("4 attempt(s)"), "{detail}");
    assert!(elapsed < Duration::from_millis(600), "{elapsed:?}");
}

#[tokio::test]
async fn a_socket_that_passes_the_probe_and_never_configures_hits_the_connect_timeout() {
    // The walk bounds itself, and the session's own connect timeout is the
    // backstop for everything after it — here, an endpoint that answers the
    // probe and then never acknowledges `session.update`. The budget is shared:
    // whatever the walk did not spend is what the handshake gets.
    let gateway = FakeGateway::spawn(
        HashMap::new(),
        Behaviour {
            never_acknowledge: true,
            ..Behaviour::default()
        },
    )
    .await;

    let started = std::time::Instant::now();
    let error = open_session(
        provider_for(&gateway),
        SessionOptions {
            connect_timeout: Some(Duration::from_millis(400)),
            ..options()
        },
    )
    .await
    .expect_err("the session never became usable");
    let elapsed = started.elapsed();

    assert_eq!(error.code(), "VIA_REALTIME_CONNECT_TIMEOUT");
    assert!(
        matches!(&error, RealtimeError::ConnectTimeout { provider, .. } if provider == "openai"),
        "{error:?}"
    );
    // The walk's own time came out of the same window, so the whole thing
    // resolves inside the caller's deadline rather than after it.
    assert!(elapsed < Duration::from_millis(600), "{elapsed:?}");
    // And the endpoint did receive the configuration — this is not a dead
    // socket, it is a socket that never answered.
    assert!(!session_updates(&gateway).is_empty());
}

#[tokio::test]
async fn an_unconfigured_provider_never_opens_a_socket() {
    // `preflight` then `is_configured`, both before the socket. The peer must
    // see nothing at all.
    let gateway = FakeGateway::healthy().await;
    let provider = Arc::new(OpenAiRealtimeProvider::new(OpenAiSettings {
        base_url: gateway.base_url(),
        api_key: Secret::default(),
        dialect: OpenAiDialect::Azure,
        ..OpenAiSettings::default()
    }));

    let error = open_session(provider, options())
        .await
        .expect_err("no credential");
    assert_eq!(error.code(), "VIA_REALTIME_NOT_CONFIGURED");
    assert!(gateway.requests().is_empty(), "{:?}", gateway.requests());
}

#[tokio::test]
async fn an_azure_dialect_with_no_endpoint_is_refused_before_the_walk() {
    // ARGO bring-up §5: before this guard it fell back to `api.openai.com`,
    // which answers `Incorrect API key provided` — an accusation about the
    // credential, from a host nobody configured, after the whole candidate walk
    // had been spent.
    let provider = Arc::new(OpenAiRealtimeProvider::new(OpenAiSettings {
        base_url: String::new(),
        api_key: Secret::new("sk-test"),
        dialect: OpenAiDialect::Azure,
        ..OpenAiSettings::default()
    }));
    let error = open_session(provider, options())
        .await
        .expect_err("no endpoint");
    assert_eq!(error.code(), "VIA_REALTIME_NOT_CONFIGURED");
}

// ── the walk's own budget ────────────────────────────────────────────────────

#[tokio::test]
async fn a_hung_upgrade_is_bounded_and_the_budget_stops_the_walk() {
    // `connect_async` has no timeout of its own. Without the walk bounding the
    // upgrade, one peer that accepts the TCP connection and never answers holds
    // the whole budget — and "connect is bounded" stops being true.
    //
    // Four candidates, a 300 ms ceiling and a 100 ms window each: three are
    // dialled and time out, and the fourth is refused before it is dialled.
    let peer = common::HangingPeer::spawn().await;
    let provider = OpenAiRealtimeProvider::new(OpenAiSettings {
        base_url: peer.base_url(),
        api_key: Secret::new("sk-test"),
        dialect: OpenAiDialect::Azure,
        ..OpenAiSettings::default()
    });

    let started = std::time::Instant::now();
    // Bounded, because the failure this test exists for is not "slow" but
    // "never ends": `connect_async` has no timeout of its own, so an unbounded
    // upgrade against a peer that never answers hangs the suite rather than
    // failing it.
    let error = tokio::time::timeout(
        Duration::from_secs(5),
        connect_within(
            &provider,
            &AgentContext::default(),
            ConnectBudget {
                total: Duration::from_millis(300),
                first_frame: Duration::from_millis(100),
            },
        ),
    )
    .await
    .expect("the walk must bound its own upgrade")
    .expect_err("nothing answered");
    let elapsed = started.elapsed();

    let RealtimeError::Transport { detail } = error else {
        panic!("expected a transport failure");
    };
    assert!(
        detail.contains("the upgrade did not complete within 100ms"),
        "{detail}"
    );
    assert!(
        detail.contains("300ms connect budget exhausted"),
        "the fourth candidate is refused before it is dialled: {detail}"
    );
    assert!(detail.contains("4 attempt(s)"), "{detail}");
    assert!(
        elapsed < Duration::from_secs(2),
        "the walk must not outlive its budget: {elapsed:?}"
    );
}

#[tokio::test]
async fn a_short_caller_deadline_still_produces_the_walks_own_report() {
    // Aborting the walk from outside throws away the rejection list, which is the
    // only record of how far it got. A caller with a short deadline gets a
    // smaller budget instead.
    let peer = common::HangingPeer::spawn().await;
    let provider = Arc::new(OpenAiRealtimeProvider::new(OpenAiSettings {
        base_url: peer.base_url(),
        api_key: Secret::new("sk-test"),
        dialect: OpenAiDialect::Azure,
        ..OpenAiSettings::default()
    }));

    let error = tokio::time::timeout(
        Duration::from_secs(5),
        open_session(
            provider,
            SessionOptions {
                connect_timeout: Some(Duration::from_millis(400)),
                ..options()
            },
        ),
    )
    .await
    .expect("the walk must bound its own upgrade")
    .expect_err("nothing answered");

    assert_eq!(
        error.code(),
        "VIA_REALTIME_TRANSPORT",
        "the walk reported, rather than being cut off: {error:?}"
    );
    let RealtimeError::Transport { detail } = error else {
        panic!("expected a transport failure");
    };
    // Every candidate, with what it actually said — which is the whole reason
    // the walk is given a smaller budget instead of being aborted from outside.
    assert!(detail.contains("4 attempt(s)"), "{detail}");
    assert!(detail.contains("the upgrade did not complete"), "{detail}");
}
