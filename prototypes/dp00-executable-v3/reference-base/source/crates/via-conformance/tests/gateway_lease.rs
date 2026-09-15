//! The single-instance Gateway lease — `<configDir>/gateway.lock`.
//!
//! Twelve catalogue rows land on one small file, and the file is the whole
//! contract: a Node Gateway, an older VIA and the CLI all read and write it, so
//! the field order, the trailing newline, the mode bits and the transient
//! names are as external as any wire event.
//!
//! Five of the rows are `Divergent` rather than `Asserted` for exactly one
//! reason: `docs/rebrand.md` renames the upstream schema prefix and the
//! upstream error-code prefix. Nothing else about the lease moves, so each test
//! derives VIA's value from the upstream one with
//! [`rebranded_schema`](via_conformance::value::rebranded_schema) /
//! [`rebranded`](via_conformance::value::rebranded) rather than retyping it. A
//! *partial* rename then fails, which is the failure mode with real risk
//! (`docs/architecture.md` §13).

use std::fs;
use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use pretty_assertions::assert_eq;
use serde_json::{Map, Value, json};

use via_conformance::value::{
    field_name, js_object_fields, quoted_literals, rebranded, rebranded_schema, unquote,
};
use via_conformance::{expect_contract, records_for};
use via_lock::{
    AcquireOptions, Clock, DEFAULT_LEASE_OWNER, FindOptions, GATEWAY_HEARTBEAT_INTERVAL,
    GATEWAY_LOCK_FILE_NAME, GATEWAY_LOCK_SCHEMA, GatewayLeaseHandle,
    HEALTH_FIELDS_READ_BY_THIS_CRATE, HEALTH_INSTANCE_ID_FIELD, LEASE_FILE_MODE, LEASE_STATE_READY,
    LEASE_STATE_STARTING, LeaseError, LeaseUpdate, ProcessProbe, SignalOutcome,
    VIA_GATEWAY_ALREADY_RUNNING, acquire_gateway_lease, find_running_gateway, gateway_lock_path,
    health_instance_id, lease_stale_path, lease_temp_path, read_gateway_lease,
};

/// A clock frozen at one instant, so `startedAt` / `heartbeatAt` are literals.
#[derive(Debug)]
struct FrozenClock(DateTime<Utc>);

impl Clock for FrozenClock {
    fn now(&self) -> DateTime<Utc> {
        self.0
    }
}

fn frozen() -> Box<dyn Clock> {
    Box::new(FrozenClock(
        Utc.with_ymd_and_hms(2026, 8, 22, 10, 36, 0)
            .single()
            .expect("2026-08-22T10:36:00Z is a real, unambiguous instant"),
    ))
}

/// A probe with a fixed answer, so "is the incumbent alive?" is a test input.
#[derive(Debug)]
struct Fixed(SignalOutcome);

impl ProcessProbe for Fixed {
    fn signal_zero(&self, _pid: i64) -> SignalOutcome {
        self.0
    }
}

fn dead() -> Box<dyn ProcessProbe> {
    Box::new(Fixed(SignalOutcome::NoSuchProcess))
}

fn alive() -> Box<dyn ProcessProbe> {
    Box::new(Fixed(SignalOutcome::Delivered))
}

const INSTANCE_ID: &str = "11111111-2222-3333-4444-555555555555";
const CHALLENGER_ID: &str = "22222222-3333-4444-5555-666666666666";

fn take_lease(directory: &Path, owner: &str) -> GatewayLeaseHandle {
    acquire_gateway_lease(
        directory,
        AcquireOptions::new()
            .pid(4242)
            .owner(owner)
            .instance_id(INSTANCE_ID)
            .clock(frozen())
            .probe(dead()),
    )
    .expect("an empty directory has no incumbent")
}

/// A `/api/health` document carrying `instance_id`, built through the constant
/// so a rename of the field cannot pass unnoticed.
fn health_document(instance_id: &str) -> Value {
    let mut map = Map::new();
    map.insert(
        HEALTH_INSTANCE_ID_FIELD.to_owned(),
        Value::String(instance_id.to_owned()),
    );
    map.insert("ok".to_owned(), Value::Bool(true));
    Value::Object(map)
}

/// The first integer appearing after `marker`, for values like
/// "Heartbeat every 15000 ms".
fn number_after(value: &str, marker: &str) -> u64 {
    let rest = value
        .split_once(marker)
        .unwrap_or_else(|| panic!("`{marker}` is no longer in the catalogued value: {value}"))
        .1;
    let digits: String = rest
        .trim_start()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits
        .parse()
        .unwrap_or_else(|_| panic!("no integer follows `{marker}` in: {value}"))
}

/// Every field name of a catalogued object shape, in catalogue order.
fn field_order(raw: &str) -> Vec<String> {
    js_object_fields(raw)
        .unwrap_or_else(|| panic!("no object shape in: {raw}"))
        .into_iter()
        .map(|field| field_name(field).to_owned())
        .collect()
}

/// The eight lease fields, in the order all three catalogue records write them.
const LEASE_FIELDS: [&str; 8] = [
    "schema",
    "instanceId",
    "pid",
    "owner",
    "state",
    "origin",
    "startedAt",
    "heartbeatAt",
];

#[test]
fn gateway_lock_schema_is_the_rebranded_upstream_string() {
    let contract = expect_contract("file-path", "GATEWAY_LOCK_SCHEMA");
    let upstream = unquote(&contract.exact_value);
    assert_eq!(
        upstream, "qwaudio.gateway-lock/v1",
        "upstream GATEWAY_LOCK_SCHEMA, {}",
        contract.file
    );

    // Derived, not retyped: a half-applied rename fails here.
    assert_eq!(
        GATEWAY_LOCK_SCHEMA,
        rebranded_schema(upstream),
        "docs/rebrand.md renames the product prefix and nothing else"
    );
    assert_ne!(GATEWAY_LOCK_SCHEMA, upstream);
    assert!(
        GATEWAY_LOCK_SCHEMA.ends_with("/v1"),
        "the rename must not move the version"
    );

    // The schema is a *gate*, not a label: a document carrying upstream's
    // string, or anything else, must read as absent rather than half
    // understood. This is what keeps a mixed-version pair of processes from
    // half-understanding each other's lease.
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = gateway_lock_path(dir.path());
    for foreign in [upstream, "via.gateway-lock/v2", "", "via.log/v1"] {
        let document = json!({ "schema": foreign, "instanceId": "x", "pid": 1 });
        fs::write(&path, format!("{document}\n")).expect("seed");
        assert!(
            read_gateway_lease(dir.path()).is_none(),
            "a `{foreign}` document must not read as a lease"
        );
    }
}

#[test]
fn gateway_lock_file_location_and_transient_names() {
    let file = expect_contract("file-path", "gateway lock file");
    assert_eq!(
        file.exact_value.trim(),
        "<configDir>/gateway.lock",
        "{}",
        file.file
    );

    let transient = expect_contract(
        "file-path",
        "gateway lock file location and transient names",
    );
    let segments: Vec<&str> = transient.exact_value.split(';').map(str::trim).collect();
    assert_eq!(segments.len(), 3, "{}", transient.exact_value);
    assert_eq!(segments[0], "<configDirectory>/gateway.lock");
    assert_eq!(segments[1], "temp during update: <path>.<instanceId>.tmp");
    assert_eq!(segments[2], "stale aside: <path>.stale.<token>");

    // The file name is brand-free and KEPT verbatim (docs/rebrand.md), which is
    // what keeps the ~/.config move a pure directory rename.
    assert_eq!(GATEWAY_LOCK_FILE_NAME, "gateway.lock");
    let configured = Path::new("/home/tester/.config/via");
    let lock = gateway_lock_path(configured);
    assert_eq!(lock, Path::new("/home/tester/.config/via/gateway.lock"));
    assert_eq!(
        lease_temp_path(&lock, INSTANCE_ID),
        Path::new(&format!(
            "/home/tester/.config/via/gateway.lock.{INSTANCE_ID}.tmp"
        ))
    );
    assert_eq!(
        lease_stale_path(&lock, INSTANCE_ID),
        Path::new(&format!(
            "/home/tester/.config/via/gateway.lock.stale.{INSTANCE_ID}"
        ))
    );
}

#[test]
fn gateway_lease_document_on_disk() {
    let format = expect_contract("file-format", "gateway.lock lease");
    let schema_doc = expect_contract("json-field", "gateway.lock document schema");
    let lease_doc = expect_contract("json-field", "gateway lease document");

    // All three records spell the same eight fields in the same order. Assert
    // that first: if the catalogue's own records ever disagree, everything
    // below is a comparison against a value nobody actually promised.
    for contract in [format, schema_doc, lease_doc] {
        assert_eq!(
            field_order(&contract.exact_value),
            LEASE_FIELDS,
            "field order in {}",
            contract.file
        );
    }

    // The alternatives each field may carry, pulled out of the record that
    // spells them with quotes rather than retyped.
    let schema_fields = js_object_fields(&schema_doc.exact_value).expect("an object shape");
    let owner_literals = quoted_literals(schema_fields[3]);
    assert_eq!(owner_literals, ["gateway", "desktop", "cli"]);
    assert_eq!(DEFAULT_LEASE_OWNER, owner_literals[0]);
    let state_literals = quoted_literals(schema_fields[4]);
    assert_eq!(state_literals, ["starting", "ready"]);
    assert_eq!(LEASE_STATE_STARTING, state_literals[0]);
    assert_eq!(LEASE_STATE_READY, state_literals[1]);
    let origin_literals = quoted_literals(schema_fields[5]);
    assert_eq!(origin_literals, ["", "http://<host>:<port>"]);

    // The third owner source is an upstream env var, renamed per
    // docs/rebrand.md. Reading it is via-core's job, not this crate's; the
    // mapping is asserted so the two cannot drift apart.
    let upstream_owner_env = "QWEN_AUDIO_GATEWAY_OWNER";
    assert!(format.exact_value.contains(upstream_owner_env));
    assert_eq!(rebranded(upstream_owner_env), "VIA_GATEWAY_OWNER");

    // The file-level facts.
    assert!(
        format
            .exact_value
            .contains("written with a trailing newline")
    );
    assert!(format.exact_value.contains("mode 0o600"));
    assert!(schema_doc.exact_value.contains("file mode 0o600"));
    assert_eq!(LEASE_FILE_MODE, 0o600);
    assert!(
        format
            .exact_value
            .contains("at <configDirectory>/gateway.lock")
    );
    assert_eq!(
        GATEWAY_HEARTBEAT_INTERVAL,
        Duration::from_millis(number_after(&format.exact_value, "Heartbeat every")),
    );

    // Now the bytes. One JSON line, the eight keys in order, plus a newline.
    let dir = tempfile::TempDir::new().expect("tempdir");
    let mut handle = take_lease(dir.path(), "desktop");
    assert_eq!(handle.path(), gateway_lock_path(dir.path()));

    let raw = fs::read_to_string(handle.path()).expect("read the lease");
    assert!(raw.ends_with('\n'), "written with a trailing newline");
    assert_eq!(raw.lines().count(), 1, "one JSON line");

    let parsed: Value = serde_json::from_str(raw.trim_end()).expect("valid JSON");
    let object = parsed.as_object().expect("a JSON object");
    assert_eq!(
        object.keys().map(String::as_str).collect::<Vec<_>>(),
        LEASE_FIELDS,
        "on-disk key order"
    );
    assert_eq!(object["schema"], json!(GATEWAY_LOCK_SCHEMA));
    assert_eq!(object["instanceId"], json!(INSTANCE_ID));
    assert_eq!(object["pid"], json!(4242));
    assert_eq!(object["owner"], json!("desktop"));
    assert_eq!(object["state"], json!("starting"));
    assert_eq!(
        object["origin"],
        json!(""),
        "empty until the listener binds"
    );
    assert_eq!(object["startedAt"], json!("2026-08-22T10:36:00.000Z"));
    assert_eq!(object["heartbeatAt"], json!("2026-08-22T10:36:00.000Z"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(handle.path())
            .expect("metadata")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, LEASE_FILE_MODE, "mode 0o600");
    }

    // `state: 'starting' then 'ready'` and `origin: '' then http://<host>:<port>`
    // — the two transitions the CLI's discovery path depends on. The
    // catalogued observation is `state: 'ready'`, `owner: 'desktop'`.
    assert!(
        handle
            .update(
                LeaseUpdate::new()
                    .state(LEASE_STATE_READY)
                    .origin("http://127.0.0.1:3101"),
            )
            .expect("update"),
        "the lease is still ours"
    );
    let updated: Value =
        serde_json::from_str(fs::read_to_string(handle.path()).expect("read").trim_end())
            .expect("valid JSON");
    assert_eq!(updated["state"], json!("ready"));
    assert_eq!(updated["owner"], json!("desktop"));
    assert_eq!(updated["origin"], json!("http://127.0.0.1:3101"));
    assert_eq!(
        updated
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        LEASE_FIELDS,
        "an update must not reorder the document"
    );

    // A clean shutdown DELETES the file (upstream test/gateway-instance-lock).
    assert!(handle.release().expect("release"));
    assert!(!gateway_lock_path(dir.path()).exists());
}

#[test]
fn already_running_conflict_code_and_message() {
    // The contract's own *name* is upstream's error-code literal, so the
    // documented rename rule is applied to it directly.
    let upstream_code = "QWAUDIO_GATEWAY_ALREADY_RUNNING";
    let upstream_schema = "qwaudio.gateway-lock/v1";

    // Catalogued twice: once from the module, once from the upstream test.
    let records = records_for("error-code", upstream_code);
    assert_eq!(records.len(), 2, "module + test records");
    let conflict = expect_contract("error-code", "gateway already-running conflict code");
    let state_name = expect_contract("state-name", "gateway lease schema / conflict code");

    assert_eq!(records[0].name, upstream_code);
    assert!(conflict.exact_value.contains(upstream_code));
    assert!(state_name.exact_value.contains(upstream_code));
    assert_eq!(VIA_GATEWAY_ALREADY_RUNNING, rebranded(upstream_code));
    assert_ne!(VIA_GATEWAY_ALREADY_RUNNING, upstream_code);

    // The schema string is the other half of the `state-name` record.
    assert!(state_name.exact_value.contains(upstream_schema));
    assert_eq!(GATEWAY_LOCK_SCHEMA, rebranded_schema(upstream_schema));

    // The message: three literals in the module record —
    // `'已有 Gateway 正在运行' + (existing.origin ? '：' + existing.origin : '')`
    // — the sentence, the fullwidth colon that joins it to the origin, and the
    // empty string the conditional falls back to when there is no origin.
    let literals = quoted_literals(&records[0].exact_value);
    assert_eq!(
        literals,
        ["已有 Gateway 正在运行", "：", ""],
        "{}",
        records[0].file
    );
    assert_eq!(
        literals[1].chars().next(),
        Some('\u{ff1a}'),
        "a fullwidth colon, not an ASCII one"
    );
    assert!(records[1].exact_value.contains("error.lease.instanceId"));

    // Provoke a real conflict: a live incumbent.
    let dir = tempfile::TempDir::new().expect("tempdir");
    let mut incumbent = take_lease(dir.path(), "desktop");
    let error = acquire_gateway_lease(
        dir.path(),
        AcquireOptions::new()
            .pid(99)
            .instance_id(CHALLENGER_ID)
            .clock(frozen())
            .probe(alive()),
    )
    .expect_err("a live incumbent must refuse the lease");

    assert_eq!(error.code(), Some(VIA_GATEWAY_ALREADY_RUNNING));
    assert_eq!(error.to_string(), literals[0], "no origin, no suffix");
    let lease = error.lease().expect("error.lease carries the incumbent");
    assert_eq!(
        lease.instance_id, INSTANCE_ID,
        "error.lease.instanceId is the incumbent"
    );
    assert_eq!(lease.pid, 4242);
    assert_eq!(lease.schema, GATEWAY_LOCK_SCHEMA);

    // With an origin published, the message gains the colon and the origin.
    incumbent
        .update(LeaseUpdate::new().origin("http://127.0.0.1:3101"))
        .expect("publish an origin");
    let error = acquire_gateway_lease(
        dir.path(),
        AcquireOptions::new()
            .pid(99)
            .instance_id(CHALLENGER_ID)
            .clock(frozen())
            .probe(alive()),
    )
    .expect_err("still refused");
    assert_eq!(
        error.to_string(),
        format!("{}{}http://127.0.0.1:3101", literals[0], literals[1])
    );
    assert!(
        conflict
            .exact_value
            .contains("已有 Gateway 正在运行：<origin>")
    );

    // The other two variants carry no code, because upstream invents none.
    assert_eq!(LeaseError::Exhausted.code(), None);
    assert_eq!(LeaseError::Exhausted.lease(), None);

    // A *dead* incumbent is not a conflict: it is reclaimed.
    let handle = acquire_gateway_lease(
        dir.path(),
        AcquireOptions::new()
            .pid(99)
            .instance_id(CHALLENGER_ID)
            .clock(frozen())
            .probe(dead()),
    )
    .expect("a dead incumbent must be reclaimed, not reported as running");
    assert_eq!(handle.instance_id(), CHALLENGER_ID);
    // The evicted holder's handle owns nothing now, and must not delete ours.
    assert!(!incumbent.release().expect("release"));
    assert!(handle.path().exists());
}

#[test]
fn find_running_gateway_polling_defaults() {
    let contract = expect_contract("default-value", "findRunningGateway polling defaults");
    assert_eq!(
        number_after(&contract.exact_value, "timeoutMs ="),
        3000,
        "{}",
        contract.file
    );
    assert_eq!(number_after(&contract.exact_value, "intervalMs ="), 100);

    let defaults = FindOptions::new();
    assert_eq!(defaults.timeout, Duration::from_millis(3000));
    assert_eq!(defaults.interval, Duration::from_millis(100));

    // "reuse requires health.gatewayInstanceId === lease.instanceId and a
    // non-empty lease.origin" — both halves, and both failure directions.
    assert!(
        contract
            .exact_value
            .contains("health.gatewayInstanceId === lease.instanceId")
    );
    assert!(contract.exact_value.contains("non-empty lease.origin"));

    let dir = tempfile::TempDir::new().expect("tempdir");
    let single_pass = FindOptions::new().timeout(Duration::ZERO);

    // No lease at all: not running, immediately, and without probing anything.
    let never = |_: &str| -> Option<Value> { panic!("no lease means no health probe") };
    assert!(find_running_gateway(dir.path(), &never, single_pass).is_none());

    let mut handle = take_lease(dir.path(), "cli");

    // A lease with an empty origin: still starting, so the probe is not called.
    assert!(find_running_gateway(dir.path(), &never, single_pass).is_none());

    handle
        .update(LeaseUpdate::new().origin("http://127.0.0.1:3101"))
        .expect("publish an origin");

    // An origin that answers with someone else's identity: not our Gateway.
    let stranger = |_: &str| Some(health_document("someone-else"));
    assert!(
        find_running_gateway(dir.path(), &stranger, single_pass).is_none(),
        "a reused port must not be handed back as a Gateway"
    );

    // An origin that does not answer at all.
    let silent = |_: &str| None;
    assert!(find_running_gateway(dir.path(), &silent, single_pass).is_none());

    // Matching identity: reused.
    let ours = |origin: &str| {
        assert_eq!(origin, "http://127.0.0.1:3101");
        Some(health_document(INSTANCE_ID))
    };
    let found = find_running_gateway(dir.path(), &ours, single_pass)
        .expect("a matching instance id is the reuse condition");
    assert_eq!(found.origin, "http://127.0.0.1:3101");
    assert_eq!(found.lease.instance_id, INSTANCE_ID);
    assert_eq!(found.health["ok"], json!(true));
}

#[test]
fn gateway_polling_timeouts_owned_here_and_owed_elsewhere() {
    let contract = expect_contract("default-value", "gateway polling timeouts");

    // The half this crate owns.
    let find = contract
        .exact_value
        .split_once("findRunningGateway:")
        .expect("the catalogue still names findRunningGateway")
        .1;
    assert!(
        find.contains("timeoutMs 3000"),
        "findRunningGateway's default timeout: {find}"
    );
    assert_eq!(number_after(find, "intervalMs"), 100);
    assert_eq!(FindOptions::new().timeout, Duration::from_millis(3000));
    assert_eq!(FindOptions::new().interval, Duration::from_millis(100));

    // The half that is not. These four values live in `cli/src/runtime.mjs`
    // and `cli/src/launcher.mjs` — the launcher that starts and stops a
    // Gateway *process* — which is `apps/via`, not this crate. Asserting they
    // are still catalogued and still unowned keeps the Partial classification
    // honest: the row cannot read as done while these have nowhere to live.
    assert_eq!(
        number_after(&contract.exact_value, "waitForGateway: timeoutMs"),
        45000
    );
    assert_eq!(number_after(&contract.exact_value, "intervalMs"), 200);
    assert_eq!(
        number_after(&contract.exact_value, "waitForGatewayStop: timeoutMs"),
        15000
    );
    assert_eq!(
        number_after(&contract.exact_value, "health fetch timeout"),
        1500
    );
    assert!(contract.file.contains("cli/src/"), "{}", contract.file);
}

#[test]
fn health_instance_id_is_the_field_this_crate_reads() {
    let contract = expect_contract("json-field", "health fields the CLI reads");

    // The one field this crate reads, and the only one it can assert.
    assert!(contract.exact_value.contains(HEALTH_INSTANCE_ID_FIELD));
    assert_eq!(HEALTH_INSTANCE_ID_FIELD, "gatewayInstanceId");
    assert_eq!(HEALTH_FIELDS_READ_BY_THIS_CRATE, [HEALTH_INSTANCE_ID_FIELD]);

    // Everything else on the list is served by `/api/health` and belongs to
    // `via-app`. Named here so the Partial row states what is still owed
    // rather than leaving it to a comment.
    for owed in [
        "backend.",
        "realtimeModelProfile.",
        "realtimeModel",
        "realtimeLabel",
        "realtimeProvider",
        "realtimeConfigurationSignature",
        "voiceConfigured",
        "voiceClients.byType.desktop",
    ] {
        assert!(
            contract.exact_value.contains(owed),
            "`{owed}` is no longer catalogued; the Partial row's remainder is stale"
        );
        assert!(
            !HEALTH_FIELDS_READ_BY_THIS_CRATE.contains(&owed),
            "`{owed}` is not this crate's to serve"
        );
    }

    // The identity check is the reason this crate reads the field at all: a
    // health document without it, or with a non-string, can never match a
    // lease and so can never be mistaken for a Gateway.
    assert_eq!(health_instance_id(&health_document("abc")), Some("abc"));
    let mut wrong_type = Map::new();
    wrong_type.insert(HEALTH_INSTANCE_ID_FIELD.to_owned(), json!(7));
    assert_eq!(health_instance_id(&Value::Object(wrong_type)), None);
    assert_eq!(health_instance_id(&json!({ "ok": true })), None);
}
