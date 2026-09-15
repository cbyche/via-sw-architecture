//! The route table, exercised against a real socket.
//!
//! Every assertion here is a catalogued `exactValue` from
//! `docs/reference/contracts.json`. Ported from
//! `server/test/{gateway-application, embedded-gateway, request-security}.test.mjs`.

mod support;

use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use support::{Harness, client};
use via_app::health::HEALTH_FIELD_ORDER;
use via_app::testing::{
    PRIVATE_PROVIDER, ScriptedBackend, TEST_PROVIDER, test_config, test_services,
};
use via_app::{InstanceIdentity, Services};

async fn get(origin: &str, path: &str) -> reqwest::Response {
    client()
        .get(format!("{origin}{path}"))
        .send()
        .await
        .expect("a response")
}

async fn json_body(response: reqwest::Response) -> Value {
    response.json().await.expect("a JSON body")
}

// ── the two probes ─────────────────────────────────────────────────────────

#[tokio::test]
async fn livez_and_readyz_are_the_catalogued_literals() {
    let harness = Harness::start().await;

    let live = get(&harness.origin, "/livez").await;
    assert_eq!(live.status(), 200);
    assert_eq!(
        json_body(live).await,
        json!({ "ok": true, "status": "live" }),
    );

    let ready = get(&harness.origin, "/readyz").await;
    assert_eq!(ready.status(), 200);
    assert_eq!(
        json_body(ready).await,
        json!({ "ok": true, "status": "ready" }),
    );

    harness.stop().await;
}

#[tokio::test]
async fn a_disallowed_origin_gets_403_even_on_livez() {
    let harness = Harness::start().await;
    let response = client()
        .get(format!("{}/livez", harness.origin))
        .header("origin", "https://attacker.example")
        .send()
        .await
        .expect("a response");
    assert_eq!(
        response.status(),
        403,
        "the origin check is the FIRST middleware, so it covers the probes",
    );
    assert!(
        response.headers().get("x-request-id").is_none(),
        "the identity middleware never ran, so it never set the header",
    );
    assert_eq!(
        json_body(response).await,
        json!({ "error": "origin not allowed" }),
    );
    harness.stop().await;
}

#[tokio::test]
async fn a_dns_rebinding_host_is_refused() {
    let harness = Harness::start().await;
    let response = client()
        .get(format!("{}/api/health", harness.origin))
        .header("host", "attacker.example:3101")
        .send()
        .await
        .expect("a response");
    assert_eq!(response.status(), 403);
    harness.stop().await;
}

// ── /api/health ────────────────────────────────────────────────────────────

#[tokio::test]
async fn the_health_field_order_is_the_contract() {
    let harness = Harness::start().await;
    let response = get(&harness.origin, "/api/health").await;
    assert_eq!(response.status(), 200);
    assert!(
        response.headers().get("x-request-id").is_some(),
        "every request past the origin check carries one",
    );

    let body = json_body(response).await;
    let object = body.as_object().expect("an object");
    let keys: Vec<&str> = object.keys().map(String::as_str).collect();
    assert_eq!(keys[..HEALTH_FIELD_ORDER.len()], HEALTH_FIELD_ORDER);
    assert_eq!(body["ok"], true);
    assert_eq!(body["status"], "ready");
    assert_eq!(
        body["protocolVersion"],
        via_protocol::GATEWAY_PROTOCOL_VERSION
    );
    assert_eq!(
        body["capabilities"],
        json!(via_protocol::GATEWAY_CAPABILITIES),
    );
    harness.stop().await;
}

#[tokio::test]
async fn health_echoes_the_lease_instance_and_start_time() {
    let harness = Harness::with_instance(InstanceIdentity {
        instance_id: Some("instance-under-test".to_owned()),
        started_at: Some("2026-08-22T00:00:00.000Z".to_owned()),
    })
    .await;
    let body = json_body(get(&harness.origin, "/api/health").await).await;
    assert_eq!(body["gatewayInstanceId"], "instance-under-test");
    assert_eq!(body["gatewayStartedAt"], "2026-08-22T00:00:00.000Z");
    harness.stop().await;
}

#[tokio::test]
async fn a_gateway_only_provider_is_active_but_not_offered() {
    let harness = Harness::with(
        {
            let mut services = via_app::testing::test_services();
            services.realtime_provider = Some(PRIVATE_PROVIDER.to_owned());
            services
        },
        InstanceIdentity::default(),
    )
    .await;
    let body = json_body(get(&harness.origin, "/api/health").await).await;
    assert_eq!(body["realtimeProvider"], PRIVATE_PROVIDER);
    let offered: Vec<&str> = body["realtimeProviders"]
        .as_array()
        .expect("an array")
        .iter()
        .filter_map(|provider| provider["key"].as_str())
        .collect();
    assert!(
        !offered.contains(&PRIVATE_PROVIDER),
        "a gateway-only provider is never offered for selection: {offered:?}",
    );
    assert!(offered.contains(&TEST_PROVIDER));
    harness.stop().await;
}

#[tokio::test]
async fn health_leaks_no_credential() {
    let harness = Harness::start().await;
    let body = json_body(get(&harness.origin, "/api/health").await).await;
    let serialized = body.to_string();
    for secret in ["sk-test", &"a".repeat(64)] {
        assert!(
            !serialized.contains(secret),
            "the health payload must not carry a credential",
        );
    }
    harness.stop().await;
}

#[tokio::test]
async fn an_unregistered_default_provider_reaches_the_terminal_error_handler() {
    // **External contract** — `http-behaviour/terminal error handler`:
    // upstream's `app.use((error, req, res, next) => { logger.error(…);
    // next(error) })` logs `http.unhandled_error` and re-delegates to
    // Express's default handler, which answers a bare 500 with **no JSON
    // envelope**. `AppState::health`'s one failure mode — a configured
    // default realtime provider the registry does not carry — is where that
    // reaches this crate: `http::probes::health` logs the same event name and
    // answers the same bare 500, with no router-wide catch-all needed because
    // axum has no equivalent of Express's `next(error)` chain to hang one off.
    let mut services = test_services();
    services.realtime_provider = Some("not-a-registered-provider".to_owned());
    let harness = Harness::with(services, InstanceIdentity::default()).await;

    let response = get(&harness.origin, "/api/health").await;
    assert_eq!(response.status(), 500);
    let body = response.bytes().await.expect("a body");
    assert!(
        body.is_empty(),
        "upstream emits no JSON error envelope here, and neither does VIA: {body:?}",
    );
    harness.stop().await;
}

#[tokio::test]
async fn health_reports_the_session_mode_degradation() {
    let harness = Harness::start().await;
    let body = json_body(get(&harness.origin, "/api/health").await).await;
    let modes = body["sessionModes"].as_array().expect("an array");
    let row = |name: &str| {
        modes
            .iter()
            .find(|mode| mode["requested"] == name)
            .cloned()
            .unwrap_or(Value::Null)
    };
    // No harness is configured in the test services.
    assert_eq!(row("agent")["effective"], "direct");
    assert_eq!(row("agent")["reason"], "no_harness");
    // `interface` no longer degrades: `via-context` ships in this workspace,
    // so the mode reports itself with no reason even though no harness is
    // configured — it never mounts one.
    assert_eq!(row("interface")["effective"], "interface");
    assert_eq!(row("interface")["reason"], Value::Null);
    harness.stop().await;
}

// ── the input control plane ────────────────────────────────────────────────

#[tokio::test]
async fn the_input_control_plane_suspends_resumes_and_refuses() {
    let harness = Harness::start().await;
    let http = client();

    let blank = http
        .post(format!("{}/api/input/suspend", harness.origin))
        .json(&json!({ "reason": "dictation" }))
        .send()
        .await
        .expect("a response");
    assert_eq!(blank.status(), 400);
    let body = json_body(blank).await;
    assert_eq!(body["code"], "VIA_INPUT_OWNER_REQUIRED");
    assert!(body["error"].is_string());

    let suspended = http
        .post(format!("{}/api/input/suspend", harness.origin))
        .json(&json!({ "owner": "host-app", "reason": "dictation", "ttlMs": 60000 }))
        .send()
        .await
        .expect("a response");
    assert_eq!(suspended.status(), 200);
    let body = json_body(suspended).await;
    assert_eq!(body["suspended"], true);
    assert_eq!(body["owner"], "host-app");
    assert_eq!(body["reason"], "dictation");
    assert!(body["expiresAt"].is_number());

    let status = json_body(get(&harness.origin, "/api/input").await).await;
    assert_eq!(status["suspended"], true);

    let resumed = http
        .post(format!("{}/api/input/resume", harness.origin))
        .json(&json!({ "owner": "host-app" }))
        .send()
        .await
        .expect("a response");
    assert_eq!(resumed.status(), 200);
    assert_eq!(json_body(resumed).await["suspended"], false);

    let unknown = http
        .post(format!("{}/api/input/resume", harness.origin))
        .json(&json!({ "owner": "nobody" }))
        .send()
        .await
        .expect("a response");
    assert_eq!(
        unknown.status(),
        200,
        "resuming an unknown owner is a no-op that still answers 200",
    );

    harness.stop().await;
}

// ── the Work surface ───────────────────────────────────────────────────────

#[tokio::test]
async fn an_unknown_task_reads_as_404_on_every_verb() {
    let harness = Harness::start().await;
    let http = client();

    for path in ["/api/tasks/work_nope", "/api/tasks/work_nope/events"] {
        let response = get(&harness.origin, path).await;
        assert_eq!(response.status(), 404, "{path}");
        assert_eq!(
            json_body(response).await,
            json!({ "error": "task not found" }),
            "{path}",
        );
    }

    let deleted = http
        .delete(format!("{}/api/tasks/work_nope", harness.origin))
        .send()
        .await
        .expect("a response");
    assert_eq!(deleted.status(), 404);
    assert_eq!(
        json_body(deleted).await,
        json!({ "error": "task not found" }),
    );

    harness.stop().await;
}

#[tokio::test]
async fn a_finished_task_answers_409_with_itself_beside_the_error() {
    let harness = Harness::start().await;
    let owner = harness.services.config.personal_owner_id.clone();
    let accepted = harness
        .services
        .work
        .create(via_work::NewWork::new("summarise the diff", &owner))
        .await
        .expect("the manager is running");
    let work_id = accepted.work.id.clone();
    // Nothing runs it, so it settles as failed with no runner installed.
    let _ = harness.services.work.wait(&work_id).await;

    let response = client()
        .delete(format!("{}/api/tasks/{work_id}", harness.origin))
        .send()
        .await
        .expect("a response");
    assert_eq!(response.status(), 409);
    let body = json_body(response).await;
    assert_eq!(body["error"], "task is no longer active");
    assert_eq!(body["task"]["id"], work_id);

    harness.stop().await;
}

#[tokio::test]
async fn tasks_and_timeline_are_scoped_to_the_caller() {
    let harness = Harness::start().await;
    let owner = harness.services.config.personal_owner_id.clone();
    let _ = harness
        .services
        .work
        .create(via_work::NewWork::new("mine", &owner))
        .await;
    let _ = harness
        .services
        .work
        .create(via_work::NewWork::new("someone else's", "user_other"))
        .await;

    let body = json_body(get(&harness.origin, "/api/tasks").await).await;
    let objectives: Vec<&str> = body["tasks"]
        .as_array()
        .expect("an array")
        .iter()
        .filter_map(|task| task["objective"].as_str())
        .collect();
    assert_eq!(objectives, ["mine"]);

    let timeline = json_body(get(&harness.origin, "/api/timeline").await).await;
    assert_eq!(
        timeline["items"].as_array().map(Vec::len),
        Some(0),
        "no Work has an inline presentation block",
    );

    harness.stop().await;
}

#[tokio::test]
async fn the_active_filter_only_answers_to_the_literal_string_true() {
    let harness = Harness::start().await;
    let owner = harness.services.config.personal_owner_id.clone();
    let accepted = harness
        .services
        .work
        .create(via_work::NewWork::new("settle please", &owner))
        .await
        .expect("the manager is running");
    let _ = harness.services.work.wait(&accepted.work.id).await;

    let all = json_body(get(&harness.origin, "/api/tasks?active=1").await).await;
    assert_eq!(all["tasks"].as_array().map(Vec::len), Some(1));
    let active = json_body(get(&harness.origin, "/api/tasks?active=true").await).await;
    assert_eq!(
        active["tasks"].as_array().map(Vec::len),
        Some(0),
        "a settled Work is not active",
    );

    harness.stop().await;
}

#[tokio::test]
async fn the_sse_stream_opens_with_a_task_snapshot() {
    let harness = Harness::start().await;
    let owner = harness.services.config.personal_owner_id.clone();
    let accepted = harness
        .services
        .work
        .create(via_work::NewWork::new("stream me", &owner))
        .await
        .expect("the manager is running");
    let work_id = accepted.work.id.clone();

    let response = get(&harness.origin, &format!("/api/tasks/{work_id}/events")).await;
    assert_eq!(response.status(), 200);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream"),
    );
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-cache"),
    );

    let chunk = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let mut response = response;
        response.chunk().await
    })
    .await
    .expect("a first frame arrives")
    .expect("no transport error")
    .expect("a chunk");
    let text = String::from_utf8_lossy(&chunk);
    assert!(text.starts_with("data: "), "got {text}");
    assert!(text.ends_with("\n\n"), "frames end with a blank line");
    let payload: Value =
        serde_json::from_str(text.trim_start_matches("data: ").trim()).expect("JSON");
    assert_eq!(payload["type"], "task.snapshot");
    assert_eq!(payload["task"]["id"], work_id);

    harness.stop().await;
}

// ── permissions ────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_decision_outside_the_two_literals_is_a_400() {
    let harness = Harness::start().await;
    let http = client();
    for decision in [json!("once"), json!("allow"), json!(null), json!(true)] {
        let response = http
            .post(format!("{}/api/permissions/auth_1", harness.origin))
            .json(&json!({ "decision": decision }))
            .send()
            .await
            .expect("a response");
        assert_eq!(response.status(), 400, "decision {decision}");
        assert_eq!(
            json_body(response).await,
            json!({ "error": "decision must be always or reject" }),
        );
    }
    harness.stop().await;
}

#[tokio::test]
async fn an_unknown_permission_is_a_404_from_the_backend() {
    let harness = Harness::start().await;
    let response = client()
        .post(format!("{}/api/permissions/auth_nope", harness.origin))
        .json(&json!({ "decision": "always" }))
        .send()
        .await
        .expect("a response");
    assert_eq!(response.status(), 404);
    assert!(json_body(response).await["error"].is_string());
    harness.stop().await;
}

#[tokio::test]
async fn a_relayed_permission_answers_with_the_backends_object() {
    let services = scripted(ScriptedBackend::with_web_ui("http://127.0.0.1:4096/ui"));
    let harness = Harness::with(services, InstanceIdentity::default()).await;
    let response = client()
        .post(format!("{}/api/permissions/auth_1", harness.origin))
        .json(&json!({ "decision": "always" }))
        .send()
        .await
        .expect("a response");
    assert_eq!(response.status(), 200);
    let body = json_body(response).await;
    assert_eq!(body["id"], "auth_1");
    assert_eq!(body["status"], "approved");
    harness.stop().await;
}

// ── the backend web address ────────────────────────────────────────────────

#[tokio::test]
async fn the_backend_ui_route_redirects_with_302() {
    let services = scripted(ScriptedBackend::with_web_ui("http://127.0.0.1:4096/ui"));
    let harness = Harness::with(services, InstanceIdentity::default()).await;
    let response = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("a client")
        .get(format!("{}/api/backend/ui", harness.origin))
        .send()
        .await
        .expect("a response");
    assert_eq!(response.status(), 302, "Express's redirect(302, url)");
    assert_eq!(
        response
            .headers()
            .get("location")
            .and_then(|value| value.to_str().ok()),
        Some("http://127.0.0.1:4096/ui"),
    );
    harness.stop().await;
}

#[tokio::test]
async fn both_404_branches_have_an_identical_body() {
    let no_capability = Harness::with(
        scripted(ScriptedBackend::without_web_ui()),
        InstanceIdentity::default(),
    )
    .await;
    let unresolvable = Harness::with(
        scripted(ScriptedBackend::with_unresolvable_web_ui()),
        InstanceIdentity::default(),
    )
    .await;

    let first = get(&no_capability.origin, "/api/backend/ui").await;
    let second = get(&unresolvable.origin, "/api/backend/ui").await;
    assert_eq!(first.status(), 404);
    assert_eq!(second.status(), 404);
    assert_eq!(
        json_body(first).await,
        json_body(second).await,
        "a client must not learn which of the two happened",
    );

    no_capability.stop().await;
    unresolvable.stop().await;
}

// ── the fallback ───────────────────────────────────────────────────────────

#[tokio::test]
async fn an_unmatched_path_is_a_json_404_rather_than_a_web_ui() {
    let harness = Harness::start().await;
    let response = get(&harness.origin, "/definitely/not/a/route").await;
    assert_eq!(response.status(), 404);
    assert_eq!(json_body(response).await, json!({ "error": "not found" }));
    harness.stop().await;
}

#[tokio::test]
async fn an_oversized_body_is_refused_after_the_origin_check() {
    let harness = Harness::start().await;
    // Deliberately **valid** JSON: a body that is merely unparseable would be a
    // 400 with or without the limit, and would not tell the two apart.
    let body = json!({ "owner": "x".repeat(via_app::JSON_BODY_LIMIT) }).to_string();
    assert!(body.len() > via_app::JSON_BODY_LIMIT);
    // The refusal is observable two ways, and which one the client sees is a
    // race it does not control: the server answers 413 and closes as soon as
    // the limit trips, so if that close beats the client's remaining body write
    // the write fails with ECONNRESET before the response can be read. Under a
    // loaded `cargo test --workspace` the second outcome is common. Both are
    // the same server behaviour — refused, and refused *early* — so accept
    // either, then prove the Gateway is still serving, which is what tells a
    // refusal apart from a crash.
    match client()
        .post(format!("{}/api/input/suspend", harness.origin))
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
    {
        Ok(response) => assert_eq!(
            response.status(),
            413,
            "the 1 MiB limit refuses it; without the limit it would suspend for \
             a one-megabyte owner name and answer 200",
        ),
        Err(error) => assert!(
            error.is_request(),
            "the only tolerated failure is the server closing on us mid-body: {error:?}",
        ),
    }
    assert!(
        client()
            .get(format!("{}/livez", harness.origin))
            .send()
            .await
            .expect("the Gateway is still serving after refusing an oversized body")
            .status()
            .is_success(),
    );

    // And the origin check runs first. Asserted with a body that fits, because
    // an oversized one races: the origin refusal closes the socket even earlier
    // than the body limit does, so the client can lose the response to an
    // ECONNRESET on its own write and the status that would have proved the
    // ordering is never read. `a_disallowed_origin_gets_403_even_on_livez`
    // carries the "first middleware" half; this carries "on this route, with a
    // body present".
    let refused = client()
        .post(format!("{}/api/input/suspend", harness.origin))
        .header("content-type", "application/json")
        .header("origin", "https://attacker.example")
        .body(json!({ "owner": "x" }).to_string())
        .send()
        .await
        .expect("a response");
    assert_eq!(refused.status(), 403);

    // The oversized-and-disallowed combination is refused too. Which check
    // fired is unobservable when the socket resets, so this asserts only what
    // is always true — that it is refused, and never suspends.
    match client()
        .post(format!("{}/api/input/suspend", harness.origin))
        .header("content-type", "application/json")
        .header("origin", "https://attacker.example")
        .body(json!({ "owner": "x".repeat(via_app::JSON_BODY_LIMIT) }).to_string())
        .send()
        .await
    {
        Ok(response) => assert!(
            matches!(response.status().as_u16(), 403 | 413),
            "refused by one check or the other, never accepted: {}",
            response.status(),
        ),
        Err(error) => assert!(error.is_request(), "{error:?}"),
    }
    let status = json_body(get(&harness.origin, "/api/input").await).await;
    assert_eq!(
        status["suspended"], false,
        "no refused request may have taken the microphone",
    );

    harness.stop().await;
}

fn scripted(backend: ScriptedBackend) -> Services {
    let config = test_config();
    let identity = via_core::IdentityManager::new(
        &"a".repeat(64),
        config.identity_mode,
        &config.personal_owner_id,
    )
    .expect("a long-enough secret");
    Services::builder(config)
        .identity(std::sync::Arc::new(identity))
        .backend(std::sync::Arc::new(backend))
        .realtime_registry(std::sync::Arc::new(via_app::testing::test_registry()))
        .realtime_provider(TEST_PROVIDER)
        .logger(std::sync::Arc::new(via_log::Logger::with_sinks(
            via_log::LoggerOptions::detached("gateway"),
            Vec::new(),
        )))
        .build()
        .expect("services")
}
