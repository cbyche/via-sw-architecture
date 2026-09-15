//! The loopback transport, over a real socket.
//!
//! A port of `server/test/acp-session-tools.test.mjs`, plus the refusals that
//! test does not reach. Every assertion here is about bytes on a TCP
//! connection: this is the only surface a backend agent ever sees, and the
//! catalogue's statements about it — 404 with an empty body, 405 with
//! `Allow: POST`, no token in the URL — are statements about exactly these
//! bytes.

mod support;

use std::sync::Arc;

use serde_json::{Value, json};
use support::{post_mcp, send};
use via_downstream::{CancelOutcome, CancelRoute, CancelTarget};
use via_mcp_tools::testing::{RecordedCall, RecordingContext, Refusal};
use via_mcp_tools::{
    DelegationOutcome, DelegationRecord, SESSION_TOOL_NAMES, SESSION_TOOL_SERVER, SessionSummary,
    SessionToolServer, envelope_text, is_error_envelope,
};

/// The bearer token out of a registration's descriptor, read the way a
/// backend would: off the descriptor, never out of the URL.
fn token_of(descriptor: &Value) -> String {
    descriptor["headers"]
        .as_array()
        .expect("headers is an array of {name,value}")
        .iter()
        .find(|header| header["name"] == "Authorization")
        .and_then(|header| header["value"].as_str())
        .and_then(|value| value.strip_prefix("Bearer "))
        .expect("a bearer token")
        .to_owned()
}

fn call(id: i64, name: &str, arguments: Value) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": { "name": name, "arguments": arguments },
    })
    .to_string()
}

/// The JSON payload the model parses out of a tool result.
fn payload(reply: &Value) -> Value {
    let text = envelope_text(&reply["result"]).expect("a text content block");
    serde_json::from_str(text).expect("the text block is JSON")
}

#[tokio::test]
async fn serves_the_five_tools_over_an_authenticated_stateless_endpoint() {
    let server = SessionToolServer::new();
    let context =
        Arc::new(
            RecordingContext::new()
                .labelled("one")
                .with_sessions(vec![SessionSummary::new(
                    "one",
                    "Build the thing",
                    "/srv/project",
                    "2026-08-22",
                )]),
        );
    let registration = server
        .register(context.clone())
        .await
        .expect("loopback binds");
    let descriptor = registration.descriptor_json();
    let address = server.address().await.expect("an address");
    let token = token_of(&descriptor);

    // The descriptor is the catalogued shape.
    assert_eq!(descriptor["type"], "http");
    assert_eq!(descriptor["name"], SESSION_TOOL_SERVER);
    assert_eq!(descriptor["headers"][0]["name"], "Authorization");

    // initialize
    let reply = post_mcp(
        address,
        Some(&token),
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1" },
            },
        })
        .to_string(),
    )
    .await
    .json();
    assert_eq!(reply["result"]["serverInfo"]["name"], SESSION_TOOL_SERVER);
    assert_eq!(reply["result"]["serverInfo"]["version"], "1.0.0");

    // tools/list — the five names, exactly.
    let reply = post_mcp(
        address,
        Some(&token),
        &json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }).to_string(),
    )
    .await
    .json();
    let mut names: Vec<String> = reply["result"]["tools"]
        .as_array()
        .expect("an array")
        .iter()
        .filter_map(|tool| tool["name"].as_str().map(str::to_owned))
        .collect();
    names.sort();
    let mut expected: Vec<String> = SESSION_TOOL_NAMES.iter().map(|s| (*s).to_owned()).collect();
    expected.sort();
    assert_eq!(names, expected);

    // The three property-name sets the upstream test deep-equals.
    let properties = |name: &str| -> Vec<String> {
        let mut keys: Vec<String> = reply["result"]["tools"]
            .as_array()
            .expect("an array")
            .iter()
            .find(|tool| tool["name"] == name)
            .and_then(|tool| tool["inputSchema"]["properties"].as_object())
            .map(|object| object.keys().cloned().collect())
            .unwrap_or_default();
        keys.sort();
        keys
    };
    assert_eq!(properties("via_sessions_list"), ["limit", "query"]);
    assert_eq!(properties("via_session_start"), ["prompt", "title"]);
    assert_eq!(properties("via_session_send"), ["prompt", "session_id"]);

    // tools/call reaches the context and comes back as a text block.
    let reply = post_mcp(
        address,
        Some(&token),
        &call(3, "via_sessions_list", json!({ "query": "project" })),
    )
    .await
    .json();
    assert_eq!(
        context.calls(),
        vec![RecordedCall::ListSessions(
            via_mcp_tools::SessionsListInput {
                query: Some("project".to_owned()),
                limit: None,
            }
        )],
    );
    assert_eq!(payload(&reply)["sessions"][0]["session_id"], "one");
    assert!(!is_error_envelope(&reply["result"]));

    let reply = post_mcp(
        address,
        Some(&token),
        &call(4, "via_session_start", json!({ "prompt": "build project" })),
    )
    .await
    .json();
    assert!(matches!(
        context.last_call(),
        Some(RecordedCall::StartSession(input)) if input.prompt == "build project"
    ));
    assert_eq!(payload(&reply)["status"], "started");
    assert!(payload(&reply)["delegation_id"].is_string());

    // update() — the next call must see the new context.
    let replacement =
        Arc::new(
            RecordingContext::new()
                .labelled("two")
                .with_sessions(vec![SessionSummary::new(
                    "two",
                    "Other",
                    "/srv/other",
                    "2026-08-22",
                )]),
        );
    assert!(registration.update(replacement.clone()).await);
    let reply = post_mcp(
        address,
        Some(&token),
        &call(5, "via_sessions_list", json!({ "query": "other" })),
    )
    .await
    .json();
    assert_eq!(payload(&reply)["sessions"][0]["session_id"], "two");
    assert_eq!(replacement.call_count(), 1);
    // The first context saw nothing further.
    assert_eq!(context.call_count(), 2);

    // release() — the token stops working, and a further update is refused.
    assert!(registration.release().await);
    assert!(!registration.update(replacement).await);
    let refused = post_mcp(
        address,
        Some(&token),
        &json!({ "jsonrpc": "2.0", "id": 6, "method": "tools/list" }).to_string(),
    )
    .await;
    assert_eq!(refused.status, 404);
    assert!(refused.body.is_empty());

    server.close().await;
}

#[tokio::test]
async fn the_token_never_appears_in_the_url_or_the_path() {
    let server = SessionToolServer::new();
    let registration = server
        .register(Arc::new(RecordingContext::new()))
        .await
        .expect("loopback binds");
    let descriptor = registration.descriptor_json();
    let url = descriptor["url"].as_str().expect("a url").to_owned();

    // Upstream's assertion: `assert.doesNotMatch(url, /[?&]token=|\/mcp\/.+/)`.
    assert!(!url.contains("?token="), "{url}");
    assert!(!url.contains("&token="), "{url}");
    assert!(!url.contains("/mcp/"), "{url}");
    assert!(url.ends_with("/mcp"), "{url}");
    assert!(url.starts_with("http://127.0.0.1:"), "{url}");

    // And the token really is elsewhere: on the header, and nowhere in the URL.
    let token = token_of(&descriptor);
    assert!(!url.contains(&token), "{url}");
    assert!(!token.is_empty());

    // `{:?}` on the registration must not leak it either.
    let rendered = format!("{registration:?}");
    assert!(!rendered.contains(&token), "{rendered}");
    assert!(rendered.contains("[REDACTED]"), "{rendered}");

    server.close().await;
}

#[tokio::test]
async fn an_unauthenticated_initialize_is_404_with_an_empty_body() {
    let server = SessionToolServer::new();
    let registration = server
        .register(Arc::new(RecordingContext::new()))
        .await
        .expect("loopback binds");
    let address = server.address().await.expect("an address");

    let response = post_mcp(
        address,
        None,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1" },
            },
        })
        .to_string(),
    )
    .await;
    assert_eq!(response.status, 404);
    assert!(response.body.is_empty(), "{:?}", response.body);
    // 404, not 401: nothing is disclosed, including a `WWW-Authenticate`.
    assert_eq!(response.header("www-authenticate"), None);

    registration.release().await;
    server.close().await;
}

#[tokio::test]
async fn a_stranger_token_a_wrong_path_and_a_malformed_scheme_are_all_404() {
    let server = SessionToolServer::new();
    let registration = server
        .register(Arc::new(RecordingContext::new()))
        .await
        .expect("loopback binds");
    let address = server.address().await.expect("an address");
    let token = token_of(&registration.descriptor_json());
    let list = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }).to_string();

    // A token nobody registered.
    let response = post_mcp(address, Some("00000000-0000-4000-8000-000000000000"), &list).await;
    assert_eq!(response.status, 404);
    assert!(response.body.is_empty());

    // The right token on the wrong path — including the path shapes a token
    // would have been smuggled through.
    for path in ["/", "/mcp/", "/mcp/x", "/MCP", "//mcp", "/mcp2"] {
        let response = send(
            address,
            "POST",
            path,
            &[
                ("Content-Type", "application/json"),
                ("Authorization", &format!("Bearer {token}")),
            ],
            Some(&list),
        )
        .await;
        assert_eq!(response.status, 404, "{path}");
        assert!(response.body.is_empty(), "{path}");
    }

    // A token in the query string is not a token.
    let response = send(
        address,
        "POST",
        &format!("/mcp?token={token}"),
        &[("Content-Type", "application/json")],
        Some(&list),
    )
    .await;
    assert_eq!(response.status, 404);

    // A well-formed header with the wrong scheme.
    for value in [
        format!("Basic {token}"),
        format!("Bearer{token}"),
        format!("Bearer  {token}"),
        format!("Bearer {token} extra"),
    ] {
        let response = send(
            address,
            "POST",
            "/mcp",
            &[
                ("Content-Type", "application/json"),
                ("Authorization", &value),
            ],
            Some(&list),
        )
        .await;
        assert_eq!(response.status, 404, "{value}");
    }

    registration.release().await;
    server.close().await;
}

#[tokio::test]
async fn a_non_post_with_a_good_token_is_405_and_without_one_is_404() {
    let server = SessionToolServer::new();
    let registration = server
        .register(Arc::new(RecordingContext::new()))
        .await
        .expect("loopback binds");
    let address = server.address().await.expect("an address");
    let token = token_of(&registration.descriptor_json());

    for method in ["GET", "DELETE", "PUT", "OPTIONS"] {
        let response = send(
            address,
            method,
            "/mcp",
            &[("Authorization", &format!("Bearer {token}"))],
            None,
        )
        .await;
        assert_eq!(response.status, 405, "{method}");
        assert_eq!(response.header("allow"), Some("POST"), "{method}");
        assert_eq!(
            response.header("content-type"),
            Some("application/json"),
            "{method}",
        );
        assert_eq!(
            response.body,
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32000,"message":"Method not allowed."}}"#,
            "{method}",
        );
    }

    // The token is checked *first*: without it the same request is a 404, so
    // the 405 is not an oracle for "this path exists".
    let response = send(address, "GET", "/mcp", &[], None).await;
    assert_eq!(response.status, 404);
    assert!(response.body.is_empty());
    assert_eq!(response.header("allow"), None);

    registration.release().await;
    server.close().await;
}

#[tokio::test]
async fn a_body_that_is_not_json_is_a_parse_error_and_an_oversized_one_is_a_500() {
    let server = SessionToolServer::new();
    let registration = server
        .register(Arc::new(RecordingContext::new()))
        .await
        .expect("loopback binds");
    let address = server.address().await.expect("an address");
    let token = token_of(&registration.descriptor_json());

    let response = post_mcp(address, Some(&token), "not json").await;
    assert_eq!(response.status, 400);
    assert_eq!(response.json()["error"]["code"], -32700);
    assert_eq!(response.json()["id"], Value::Null);

    let oversized = format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\",\"pad\":\"{}\"}}",
        "x".repeat(via_mcp_tools::MAX_MESSAGE_BYTES + 1),
    );
    let response = post_mcp(address, Some(&token), &oversized).await;
    assert_eq!(response.status, 500);
    assert_eq!(response.json()["error"]["code"], -32603);
    assert_eq!(response.json()["id"], Value::Null);

    registration.release().await;
    server.close().await;
}

#[tokio::test]
async fn a_notification_only_post_is_accepted_with_no_body() {
    let server = SessionToolServer::new();
    let context = Arc::new(RecordingContext::new());
    let registration = server
        .register(context.clone())
        .await
        .expect("loopback binds");
    let address = server.address().await.expect("an address");
    let token = token_of(&registration.descriptor_json());

    let response = post_mcp(
        address,
        Some(&token),
        &json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }).to_string(),
    )
    .await;
    assert_eq!(response.status, 202);
    assert!(response.body.is_empty());
    assert_eq!(context.call_count(), 0);

    registration.release().await;
    server.close().await;
}

#[tokio::test]
async fn a_client_that_accepts_an_event_stream_is_answered_with_one() {
    let server = SessionToolServer::new();
    let registration = server
        .register(Arc::new(RecordingContext::new()))
        .await
        .expect("loopback binds");
    let address = server.address().await.expect("an address");
    let token = token_of(&registration.descriptor_json());

    let response = send(
        address,
        "POST",
        "/mcp",
        &[
            ("Content-Type", "application/json"),
            ("Accept", "application/json, text/event-stream"),
            ("Authorization", &format!("Bearer {token}")),
        ],
        Some(&json!({ "jsonrpc": "2.0", "id": 1, "method": "ping" }).to_string()),
    )
    .await;
    assert_eq!(response.status, 200);
    assert_eq!(response.header("content-type"), Some("text/event-stream"));
    assert_eq!(
        response.header("cache-control"),
        Some("no-cache, no-transform")
    );
    let messages = response.sse_messages();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["id"], 1);

    // A batch frames one event per reply.
    let response = send(
        address,
        "POST",
        "/mcp",
        &[
            ("Content-Type", "application/json"),
            ("Accept", "text/event-stream"),
            ("Authorization", &format!("Bearer {token}")),
        ],
        Some(
            &json!([
                { "jsonrpc": "2.0", "id": 1, "method": "ping" },
                { "jsonrpc": "2.0", "id": 2, "method": "ping" },
            ])
            .to_string(),
        ),
    )
    .await;
    assert_eq!(response.sse_messages().len(), 2);

    registration.release().await;
    server.close().await;
}

#[tokio::test]
async fn two_registrations_are_isolated_from_each_other() {
    let server = SessionToolServer::new();
    let first = Arc::new(RecordingContext::new().labelled("first"));
    let second = Arc::new(RecordingContext::new().labelled("second"));
    let one = server.register(first.clone()).await.expect("binds");
    let two = server.register(second.clone()).await.expect("binds");
    let address = server.address().await.expect("an address");

    let token_one = token_of(&one.descriptor_json());
    let token_two = token_of(&two.descriptor_json());
    assert_ne!(token_one, token_two);
    // Both descriptors name the same endpoint; only the credential differs.
    assert_eq!(one.descriptor_json()["url"], two.descriptor_json()["url"]);

    post_mcp(
        address,
        Some(&token_one),
        &call(1, "via_sessions_list", json!({})),
    )
    .await;
    assert_eq!(first.call_count(), 1);
    assert_eq!(second.call_count(), 0);

    // Releasing one leaves the other alone.
    one.release().await;
    let refused = post_mcp(
        address,
        Some(&token_one),
        &call(2, "via_sessions_list", json!({})),
    )
    .await;
    assert_eq!(refused.status, 404);
    let served = post_mcp(
        address,
        Some(&token_two),
        &call(3, "via_sessions_list", json!({})),
    )
    .await;
    assert_eq!(served.status, 200);
    assert_eq!(second.call_count(), 1);

    two.release().await;
    server.close().await;
}

#[tokio::test]
async fn a_dropped_registration_takes_its_token_with_it() {
    let server = SessionToolServer::new();
    let address;
    let token;
    {
        let registration = server
            .register(Arc::new(RecordingContext::new()))
            .await
            .expect("binds");
        address = server.address().await.expect("an address");
        token = token_of(&registration.descriptor_json());
        assert_eq!(
            post_mcp(
                address,
                Some(&token),
                &json!({ "jsonrpc": "2.0", "id": 1, "method": "ping" }).to_string(),
            )
            .await
            .status,
            200,
        );
    }
    // Upstream leaves a dropped registration's token live until the whole
    // server closes; VIA revokes it. Recorded in docs/deviations/phase-3.md.
    let refused = post_mcp(
        address,
        Some(&token),
        &json!({ "jsonrpc": "2.0", "id": 2, "method": "ping" }).to_string(),
    )
    .await;
    assert_eq!(refused.status, 404);

    server.close().await;
}

#[tokio::test]
async fn a_handler_failure_reaches_the_model_as_the_error_envelope() {
    let server = SessionToolServer::new();
    let registration = server
        .register(Arc::new(
            RecordingContext::new().failing("the backend refused"),
        ))
        .await
        .expect("binds");
    let address = server.address().await.expect("an address");
    let token = token_of(&registration.descriptor_json());

    let response = post_mcp(
        address,
        Some(&token),
        &call(
            1,
            "via_session_send",
            json!({ "session_id": "s", "prompt": "p" }),
        ),
    )
    .await;
    // 200: the tool answered. The failure is in the envelope, not the
    // transport.
    assert_eq!(response.status, 200);
    let reply = response.json();
    assert!(reply.get("error").is_none());
    assert!(is_error_envelope(&reply["result"]));
    let payload = payload(&reply);
    assert_eq!(payload["status"], "failed");
    assert_eq!(payload["error"], "the backend refused");

    registration.release().await;
    server.close().await;
}

#[tokio::test]
async fn the_status_and_cancel_answers_are_the_shapes_the_model_parses() {
    let server = SessionToolServer::new();
    let record = DelegationRecord::new("acp_run_1", "sess-1", "Build", "/srv/project");
    let context = Arc::new(
        RecordingContext::new()
            .with_delegation(record.clone(), DelegationOutcome::completed("all done"))
            .with_cancel(CancelOutcome::requested(
                CancelRoute::Adapter,
                CancelTarget::delegation("acp_run_1"),
            )),
    );
    let registration = server.register(context).await.expect("binds");
    let address = server.address().await.expect("an address");
    let token = token_of(&registration.descriptor_json());

    let found = payload(
        &post_mcp(
            address,
            Some(&token),
            &call(
                1,
                "via_session_status",
                json!({ "delegation_id": "acp_run_1" }),
            ),
        )
        .await
        .json(),
    );
    assert_eq!(found["status"], "completed");
    assert_eq!(found["delegation_id"], "acp_run_1");
    assert_eq!(found["session_id"], "sess-1");
    assert_eq!(found["title"], "Build");
    assert_eq!(found["directory"], "/srv/project");
    assert_eq!(found["result"], "all done");
    assert!(found.get("error").is_none());

    // A lookup naming nothing answers `not_found`, bare.
    let missing = payload(
        &post_mcp(
            address,
            Some(&token),
            &call(2, "via_session_status", json!({})),
        )
        .await
        .json(),
    );
    assert_eq!(missing, json!({ "status": "not_found" }));

    // Cancel reports `cancelling` for a delivered-but-unconfirmed cancel —
    // via-downstream's recorded divergence from upstream's optimistic
    // `cancelled`.
    let cancelled = payload(
        &post_mcp(
            address,
            Some(&token),
            &call(
                3,
                "via_session_cancel",
                json!({ "delegation_id": "acp_run_1" }),
            ),
        )
        .await
        .json(),
    );
    assert_eq!(cancelled["status"], "cancelling");
    assert_eq!(cancelled["delegation_id"], "acp_run_1");
    assert!(cancelled.get("title").is_none(), "cancel carries no title");

    registration.release().await;
    server.close().await;
}

#[tokio::test]
async fn closing_is_prompt_even_while_a_socket_is_still_connected() {
    let server = SessionToolServer::new();
    let registration = server
        .register(Arc::new(RecordingContext::new()))
        .await
        .expect("binds");
    let address = server.address().await.expect("an address");

    // An idle connection, exactly as upstream's test opens.
    let mut socket = tokio::net::TcpStream::connect(address)
        .await
        .expect("connect");

    tokio::time::timeout(std::time::Duration::from_millis(500), server.close())
        .await
        .expect("close must not block on an idle connection");

    // And the socket is finished, not merely forgotten. Upstream destroys it
    // outright, which a peer sees as a reset; a graceful close is an orderly
    // EOF. Either proves the connection is over — a read that *returns data*
    // or one that never returns would not.
    let mut buffer = [0u8; 1];
    let read = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        tokio::io::AsyncReadExt::read(&mut socket, &mut buffer),
    )
    .await
    .expect("the read must not hang: the socket has to be closed");
    match read {
        Ok(0) => {}
        Err(_) => {}
        Ok(bytes) => panic!("the closed server sent {bytes} byte(s)"),
    }

    // The registration outlives the server harmlessly.
    assert!(!registration.update(Arc::new(RecordingContext::new())).await);
}

#[tokio::test]
async fn a_closed_server_can_be_started_again_on_a_new_port() {
    let server = SessionToolServer::new();
    let first = server.register(Arc::new(RecordingContext::new())).await;
    let first_address = server.address().await.expect("an address");
    drop(first);
    server.close().await;
    assert_eq!(server.address().await, None);

    let second = server
        .register(Arc::new(RecordingContext::new()))
        .await
        .expect("binds again");
    let second_address = server.address().await.expect("an address");
    assert_ne!(first_address, second_address);
    assert_eq!(
        post_mcp(
            second_address,
            Some(&token_of(&second.descriptor_json())),
            &json!({ "jsonrpc": "2.0", "id": 1, "method": "ping" }).to_string(),
        )
        .await
        .status,
        200,
    );
    server.close().await;
}

#[tokio::test]
async fn starting_twice_keeps_one_listener() {
    let server = SessionToolServer::new();
    let first = server.start().await.expect("binds");
    let second = server.start().await.expect("already bound");
    assert_eq!(first, second);
    assert_ne!(first.port(), 0);
    server.close().await;
}

#[tokio::test]
async fn the_servers_locale_decides_the_sentence_the_model_is_shown() {
    // The tool result is model-visible, so a failure the seam localizes must
    // be rendered in the Gateway's locale rather than in a default nobody
    // chose. Upstream has no such choice — it is monolingual.
    for locale in [
        via_i18n::Locale::En,
        via_i18n::Locale::Zh,
        via_i18n::Locale::Ko,
    ] {
        let server = SessionToolServer::new().with_locale(locale);
        let registration = server
            .register(Arc::new(
                RecordingContext::new().refusing(Refusal::Cancelled),
            ))
            .await
            .expect("binds");
        let address = server.address().await.expect("an address");
        let token = token_of(&registration.descriptor_json());

        let reply = post_mcp(
            address,
            Some(&token),
            &call(1, "via_session_status", json!({ "delegation_id": "d" })),
        )
        .await
        .json();
        assert_eq!(
            payload(&reply)["error"],
            via_downstream::HarnessError::Cancelled.message(locale),
            "{locale:?}",
        );
        registration.release().await;
        server.close().await;
    }
}

#[tokio::test]
async fn an_explicit_host_is_the_host_on_the_descriptor() {
    // `with_host` is a test seam; this is what it is for — proving the URL is
    // built from the address actually bound rather than from a constant.
    let server = SessionToolServer::new().with_host(via_mcp_tools::LOOPBACK_HOST);
    let registration = server
        .register(Arc::new(RecordingContext::new()))
        .await
        .expect("binds");
    let address = server.address().await.expect("an address");
    assert_eq!(
        registration.descriptor_json()["url"],
        format!("http://{address}/mcp"),
    );
    assert_eq!(address.ip(), via_mcp_tools::LOOPBACK_HOST);
    registration.release().await;
    server.close().await;
}
