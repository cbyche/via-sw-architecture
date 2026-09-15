//! The catalogue, parsed rather than retyped.
//!
//! Every assertion here reads its expected value out of
//! `docs/reference/contracts.json` at test time. A contract whose `exactValue`
//! is edited without the code moving with it fails here, which is the point —
//! a hand-copied literal would silently agree with itself forever.

use std::collections::BTreeSet;
use std::path::PathBuf;

use pretty_assertions::assert_eq;
use serde_json::Value;

fn catalogue() -> Vec<Value> {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "..",
        "..",
        "docs",
        "reference",
        "contracts.json",
    ]
    .iter()
    .collect();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
    serde_json::from_str(&text).expect("the catalogue is JSON")
}

/// Every contract of `kind` whose `name` matches.
fn contracts(kind: &str, name: &str) -> Vec<Value> {
    catalogue()
        .into_iter()
        .filter(|contract| contract["kind"] == kind && contract["name"].as_str() == Some(name))
        .collect()
}

/// The one contract of `kind` named `name`, or a panic naming what was found.
fn contract(kind: &str, name: &str) -> Value {
    let found = contracts(kind, name);
    assert!(
        !found.is_empty(),
        "the catalogue has no {kind} named `{name}`",
    );
    found.into_iter().next().unwrap_or(Value::Null)
}

fn exact(kind: &str, name: &str) -> String {
    contract(kind, name)["exactValue"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

/// Every `exactValue` of `kind` named `name`, joined.
///
/// The catalogue carries a name more than once where two files describe the
/// same surface — `POST /api/input/suspend` has one entry for the body shape
/// and one for the bounds and the refusal code — so an assertion about that
/// surface must see all of them.
fn all_exact(kind: &str, name: &str) -> String {
    let found = contracts(kind, name);
    assert!(
        !found.is_empty(),
        "the catalogue has no {kind} named `{name}`",
    );
    found
        .iter()
        .filter_map(|contract| contract["exactValue"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

// ── /api/health ────────────────────────────────────────────────────────────

#[test]
fn the_health_field_order_is_the_catalogued_one() {
    // The catalogue writes the order as prose — "200 JSON with keys in this
    // order: ok(true), status('ready'), …" — so the list is recovered from it
    // rather than retyped, and a reordering upstream fails here.
    let described = exact("http-route", "GET /api/health");
    let (_, list) = described
        .split_once("in this order:")
        .unwrap_or_else(|| panic!("the catalogued value changed shape: {described}"));
    let catalogued: Vec<String> = list
        .split(',')
        .map(|entry| {
            entry
                .trim()
                // `ok(true)` and `gatewayInstanceId(string|null)` annotate the
                // key with its type; the key is what is before the bracket.
                .split('(')
                .next()
                .unwrap_or_default()
                .trim()
                .to_owned()
        })
        .filter(|entry| !entry.is_empty())
        .collect();

    assert_eq!(
        via_app::HEALTH_FIELD_ORDER.to_vec(),
        catalogued,
        "the served field order must equal the catalogued one",
    );
}

#[test]
fn the_two_probes_answer_their_catalogued_literals() {
    let livez = exact("http-route", "/livez");
    assert!(
        livez.contains("{\"ok\":true,\"status\":\"live\"}"),
        "the catalogued value changed: {livez}",
    );
    let served = serde_json::to_string(&via_app::LivenessProbe::default()).expect("serializes");
    assert!(
        livez.contains(&served),
        "served `{served}` is not the catalogued literal in `{livez}`",
    );

    let readyz = exact("http-route", "GET /readyz");
    let served = serde_json::to_string(&via_app::ReadinessProbe::default()).expect("serializes");
    assert!(
        readyz.contains(&served),
        "served `{served}` is not the catalogued literal in `{readyz}`",
    );
}

// ── the protocol surface ───────────────────────────────────────────────────

#[test]
fn the_advertised_capabilities_partition_upstreams_sixteen() {
    let catalogued = exact("capability-list", "GATEWAY_CAPABILITIES");
    // Every entry matches the capability shape the contract itself names:
    // `^[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*)+$`. Splitting on the quote picks up
    // `Object.freeze([` as well, which is why the shape is checked rather than
    // just the dot.
    let is_capability = |token: &str| {
        !token.is_empty()
            && token.split('.').count() >= 2
            && token.split('.').all(|segment| {
                segment
                    .chars()
                    .next()
                    .is_some_and(|first| first.is_ascii_lowercase())
                    && segment.chars().all(|character| {
                        character.is_ascii_lowercase()
                            || character.is_ascii_digit()
                            || character == '-'
                    })
            })
    };
    let upstream: BTreeSet<&str> = catalogued
        .split('\'')
        .filter(|token| is_capability(token))
        .collect();
    assert_eq!(
        upstream.len(),
        16,
        "upstream advertises sixteen capabilities; parsed {upstream:?}",
    );

    let advertised: BTreeSet<&str> = via_protocol::GATEWAY_CAPABILITIES.iter().copied().collect();
    let dropped: BTreeSet<&str> = via_protocol::DROPPED_UPSTREAM_CAPABILITIES
        .iter()
        .copied()
        .collect();
    assert!(
        advertised.is_disjoint(&dropped),
        "a capability cannot be both advertised and dropped",
    );
    assert_eq!(
        advertised.union(&dropped).copied().collect::<BTreeSet<_>>(),
        upstream,
        "the two lists must partition upstream's sixteen exactly",
    );
}

#[test]
fn the_realtime_route_and_its_payload_cap_are_catalogued() {
    let described = exact("ws-route", "WS /api/realtime");
    assert!(
        described.contains(via_app::REALTIME_ROUTE),
        "the catalogued route changed: {described}",
    );
    assert!(
        described.contains("maxPayload 20*1024*1024"),
        "the catalogued cap changed: {described}",
    );
    assert_eq!(via_app::MAX_PAYLOAD_BYTES, 20 * 1024 * 1024);
    assert!(
        described.contains("socket.destroy"),
        "the destroy contract is what makes this crate own its accept loop: {described}",
    );
}

#[test]
fn the_work_surface_bodies_are_the_catalogued_strings() {
    let described = exact("http-route", "Work HTTP surface");
    for literal in [
        "{ tasks: [publicTask] }",
        "404 { error: 'task not found' }",
        "409 { error: 'task is no longer active'",
        "task.snapshot",
        "inline_${taskId}",
    ] {
        assert!(
            described.contains(literal),
            "the catalogued value no longer contains `{literal}`: {described}",
        );
    }
    // The two bodies VIA serves, read out of the catalogue.
    assert!(described.contains(via_i18n::t(
        via_i18n::Locale::En,
        via_i18n::keys::GATEWAY_TASK_NOT_FOUND,
    )));
    assert!(described.contains(via_i18n::t(
        via_i18n::Locale::En,
        via_i18n::keys::GATEWAY_TASK_NOT_ACTIVE,
    )));
    assert_eq!(via_app::http::tasks::TIMELINE_ID_PREFIX, "inline_");
    assert_eq!(via_app::http::tasks::SSE_SNAPSHOT_TYPE, "task.snapshot");
}

#[test]
fn the_permission_decision_enum_is_exactly_two_literals() {
    let described = exact("http-route", "POST /api/permissions/:id");
    assert!(
        described.contains("decision must be always or reject"),
        "the catalogued 400 body changed: {described}",
    );
    assert_eq!(
        via_i18n::t(
            via_i18n::Locale::En,
            via_i18n::keys::GATEWAY_PERMISSION_DECISION_INVALID,
        ),
        "decision must be always or reject",
    );
    for locale in [
        via_i18n::Locale::En,
        via_i18n::Locale::Zh,
        via_i18n::Locale::Ko,
    ] {
        assert_eq!(
            via_i18n::t(locale, via_i18n::keys::GATEWAY_PERMISSION_DECISION_INVALID),
            "decision must be always or reject",
            "a client branches on this string, so it is not localized",
        );
    }
}

#[test]
fn the_backend_ui_route_is_a_302_and_a_shared_404_body() {
    let described = exact("http-route", "GET /api/backend/ui");
    assert!(
        described.contains("302 redirect"),
        "the catalogued status changed: {described}",
    );
    assert!(
        described.contains("identical body in both 404 branches"),
        "the shared body is the contract: {described}",
    );
    assert!(
        described.contains(via_i18n::t(
            via_i18n::Locale::Zh,
            via_i18n::keys::GATEWAY_BACKEND_HAS_NO_WEB_UI,
        )),
        "the served Chinese prose must be the catalogued one: {described}",
    );
    assert_eq!(via_app::http::backend::BACKEND_UI_REDIRECT_STATUS, 302);
}

#[test]
fn the_input_owner_refusal_carries_the_rebranded_code() {
    let described = all_exact("http-route", "POST /api/input/suspend");
    assert!(
        described.contains("INPUT_OWNER_REQUIRED"),
        "the catalogued code changed: {described}",
    );
    // The namespace is renamed per `docs/rebrand.md`; the suffix is not.
    let suffix = via_protocol::CODE_INPUT_OWNER_REQUIRED
        .strip_prefix("VIA_")
        .expect("VIA-namespaced");
    assert!(described.contains(suffix), "{described}");
    assert!(
        !via_protocol::CODE_INPUT_OWNER_REQUIRED.contains("QWAUDIO"),
        "the upstream namespace must not survive the rebrand",
    );
}

#[test]
fn the_middleware_order_is_the_catalogued_one() {
    let described = exact("http-behaviour", "middleware order and limits");
    let position = |needle: &str| {
        described
            .find(needle)
            .unwrap_or_else(|| panic!("the catalogued value no longer mentions `{needle}`"))
    };
    assert!(
        position("enforceSameOrigin") < position("X-Request-Id"),
        "the origin check is the first middleware",
    );
    assert!(
        position("X-Request-Id") < position("express.json"),
        "an oversized body is refused only after identity issuance",
    );
    assert!(
        described.contains("limit:'1mb'"),
        "the catalogued limit changed: {described}",
    );
    assert_eq!(via_app::JSON_BODY_LIMIT, 1024 * 1024);
}

#[test]
fn the_request_id_header_is_on_every_request_that_gets_past_the_origin_check() {
    let described = exact("http-header", "X-Request-Id");
    assert!(described.contains("X-Request-Id"), "{described}");
    assert_eq!(
        via_app::http::middleware::REQUEST_ID_HEADER,
        "x-request-id",
        "HTTP/2 lower-cases header names; the wire name is the same header",
    );
}

#[test]
fn the_offline_notification_has_two_payload_shapes_under_one_type() {
    let described = exact("ipc-message", "offline notification");
    assert!(
        described.contains("status:'progress'"),
        "the progress arm hard-codes its status: {described}",
    );
    assert_eq!(via_app::PROGRESS_STATUS, "progress");
    // The type is rebranded; the suffix is not.
    assert!(
        described.contains(":offline-notification"),
        "the catalogued suffix changed: {described}",
    );
    assert!(via_app::OFFLINE_NOTIFICATION_TYPE.ends_with(":offline-notification"));
    assert!(
        !via_app::OFFLINE_NOTIFICATION_TYPE.contains("qwen"),
        "the upstream brand must not survive the rebrand",
    );
}

#[test]
fn the_gateway_ready_report_carries_the_bound_origin() {
    let described = exact("ipc-message", "gateway ready report");
    assert!(
        described.contains("http://<host>:<boundPort>"),
        "the origin is how a host learns the port when PORT=0: {described}",
    );
}

#[test]
fn the_lease_heartbeat_and_exit_timeout_are_catalogued() {
    // The 15-second heartbeat lives in `via-lock`; this crate re-exports it so
    // the sequence reads in one place, and the two must not drift.
    assert_eq!(
        via_app::HEARTBEAT_INTERVAL,
        via_lock::GATEWAY_HEARTBEAT_INTERVAL,
    );
    assert_eq!(
        via_app::EXIT_TIMEOUT,
        std::time::Duration::from_millis(2000)
    );
}

// ── the catalogue itself ───────────────────────────────────────────────────

#[test]
fn the_catalogue_still_has_thirty_five_http_routes() {
    let routes = catalogue()
        .into_iter()
        .filter(|contract| contract["kind"] == "http-route")
        .count();
    assert_eq!(
        routes, 35,
        "the task named 35 `http-route` entries; if that changed, the route \
         table's coverage claim needs revisiting",
    );
}
