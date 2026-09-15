//! **Security, as a process.**
//!
//! Four properties, each of which a unit test can only approximate:
//!
//! 1. a DNS-rebinding `Origin` is refused **403**, with the catalogued body;
//! 2. an unknown WebSocket upgrade path is **destroyed** — the socket closes
//!    with *no bytes at all*, which is not a response and therefore cannot be
//!    asserted from a handler's return value;
//! 3. the signed owner cookie is **required** on an upgrade, and a forged one is
//!    refused — with the genuine signature computed here, independently of the
//!    code that mints it, so both directions are pinned;
//! 4. **no secret appears in `gateway.log`** — grepped out of the real file
//!    after a real run, which is a redaction gate a unit test cannot give.
//!
//! # Why raw TCP
//!
//! Two of the four are about bytes an HTTP client would hide. `reqwest` cannot
//! send a `Host` that disagrees with the URL it dialled — which is the whole of
//! the rebinding shape — and it reports a destroyed connection as a transport
//! error rather than as *zero bytes read*. So the requests below are written
//! onto a `TcpStream` by hand and the answer is read as bytes.

mod support;

use std::time::Duration;

use base64::Engine as _;
use hmac::Mac as _;
use pretty_assertions::assert_eq;
use support::{BUDGET, Machine, Socket};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use via_realtime_mock::Script;

/// A credential shaped like a real one and unlike anything else in the tree.
///
/// Distinctive on purpose: the grep at the end is only meaningful if a match
/// could not be an accident.
const API_KEY: &str = "sk-e2e-leak-canary-6f1a9c4d2b8e";

/// The HMAC key every client cookie is signed with. 64 hex characters, which is
/// the shape `state.env` generates.
const AUTH_SECRET: &str = "e2e00000000000000000000000000000000000000000000000000000000canary";

/// How long a raw socket is given to answer.
const SOCKET_BUDGET: Duration = Duration::from_secs(10);

/// Send `request` verbatim and read whatever comes back, to EOF or the budget.
///
/// Answers the bytes read. **An empty answer is the interesting one**: it means
/// the peer closed without writing a status line, which is what
/// `socket.destroy()` looks like from the outside.
async fn raw(origin: &str, request: &str) -> Vec<u8> {
    exchange(origin, request, false).await
}

/// [`raw`], stopping at the end of the header block.
///
/// For a request the Gateway answers **without** closing — an accepted upgrade
/// leaves the socket open for the WebSocket that follows — so reading to EOF
/// would sit there until the budget expired on every successful case.
async fn raw_headers(origin: &str, request: &str) -> Vec<u8> {
    exchange(origin, request, true).await
}

async fn exchange(origin: &str, request: &str, stop_at_headers: bool) -> Vec<u8> {
    let address = origin
        .strip_prefix("http://")
        .unwrap_or(origin)
        .trim_end_matches('/');
    let mut stream = tokio::net::TcpStream::connect(address)
        .await
        .expect("the Gateway is listening");
    stream
        .write_all(request.as_bytes())
        .await
        .expect("the request is written");
    stream.flush().await.expect("flush");

    let mut answer = Vec::new();
    let mut chunk = [0u8; 4096];
    let deadline = tokio::time::Instant::now() + SOCKET_BUDGET;
    loop {
        // Bounded: a Gateway that neither answers nor closes would otherwise
        // hang the suite rather than fail it.
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        let Ok(Ok(read)) = tokio::time::timeout(remaining, stream.read(&mut chunk)).await else {
            break;
        };
        if read == 0 {
            // EOF. For a destroyed upgrade this is the *first* thing that
            // happens, and `answer` is still empty — which is the assertion.
            break;
        }
        answer.extend_from_slice(&chunk[..read]);
        if stop_at_headers && answer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    answer
}

/// A raw HTTP/1.1 request with an explicit `Host`.
fn request(method: &str, path: &str, host: &str, headers: &[(&str, &str)]) -> String {
    let mut text = format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\n");
    for (name, value) in headers {
        text.push_str(&format!("{name}: {value}\r\n"));
    }
    text.push_str("Connection: close\r\n\r\n");
    text
}

/// The header block of a WebSocket upgrade, minus `Connection: close`.
fn upgrade(path: &str, host: &str, extra: &[(&str, &str)]) -> String {
    let mut text = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nUpgrade: websocket\r\n\
         Connection: Upgrade\r\nSec-WebSocket-Version: 13\r\n\
         Sec-WebSocket-Key: {}\r\n",
        base64::engine::general_purpose::STANDARD.encode([0u8; 16]),
    );
    for (name, value) in extra {
        text.push_str(&format!("{name}: {value}\r\n"));
    }
    text.push_str("\r\n");
    text
}

/// `<owner>.<base64url(HMAC-SHA256(secret, owner))>`, unpadded.
///
/// Composed here from `hmac` and `sha2` directly rather than through
/// `via_core::IdentityManager`, so "a genuine cookie is accepted" is not
/// asserted with the same code that mints one. If the two ever disagree, this
/// test is the one that says so.
fn signed_cookie(secret: &str, owner_id: &str) -> String {
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts a key of any length");
    mac.update(owner_id.as_bytes());
    let signature =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    format!(
        "{}={owner_id}.{signature}",
        via_core::identity::IDENTITY_COOKIE_NAME,
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn a_rebinding_origin_is_refused_and_an_unknown_upgrade_is_destroyed() {
    let machine = Machine::new();
    let gateway = machine.start_harness_gateway(&Script::conversation(), None, &[]);
    gateway.require_serving();
    let origin = gateway.origin();
    let authority = origin.trim_start_matches("http://").to_owned();
    let port = authority
        .rsplit(':')
        .next()
        .expect("the origin names a port")
        .to_owned();
    // The status line the catalogue names, built from `via-core`'s own constant
    // rather than typed: `403` is the contract, not a number this file chose.
    let forbidden = format!("HTTP/1.1 {} ", via_core::security::ORIGIN_REJECTION_STATUS);

    // ── 1. the loopback baseline ────────────────────────────────────────────
    // A CLI sends no `Origin` at all and reaches a loopback `Host`; that is the
    // path every non-browser client takes, and it must work or the refusals
    // below prove nothing.
    let allowed = raw(&origin, &request("GET", "/livez", &authority, &[])).await;
    let allowed = String::from_utf8_lossy(&allowed).into_owned();
    assert!(allowed.starts_with("HTTP/1.1 200 "), "{allowed}");

    // ── 2. plain cross-origin ───────────────────────────────────────────────
    let refused = raw(
        &origin,
        &request(
            "GET",
            "/api/tasks",
            &authority,
            &[("Origin", "https://attacker.example")],
        ),
    )
    .await;
    let refused = String::from_utf8_lossy(&refused).into_owned();
    assert!(refused.starts_with(&forbidden), "{refused}");
    assert!(
        refused.ends_with(via_core::security::ORIGIN_REJECTION_BODY),
        "the catalogued body, byte for byte: {refused}",
    );

    // ── 3. DNS rebinding ────────────────────────────────────────────────────
    // The attacker controls `evil.example` and answers with a TTL-0 record
    // pointing at 127.0.0.1. The page's *own* origin now resolves to the
    // Gateway: `Origin` and `Host` **agree**, and a naive same-origin check
    // passes. The defence is that the implicit same-origin path is restricted
    // to *literal* loopback hosts, so a public hostname must be allow-listed
    // whatever it resolves to.
    let rebound = format!("evil.example:{port}");
    let refused = raw(
        &origin,
        &request(
            "GET",
            "/api/tasks",
            &rebound,
            &[("Origin", &format!("http://{rebound}"))],
        ),
    )
    .await;
    let refused = String::from_utf8_lossy(&refused).into_owned();
    assert!(
        refused.starts_with(&forbidden),
        "an Origin that agrees with a *rebound* Host is still refused: {refused}",
    );
    assert!(
        refused.ends_with(via_core::security::ORIGIN_REJECTION_BODY),
        "{refused}"
    );
    // …and the refusal is the first middleware, so it covers the probes too.
    let refused = raw(
        &origin,
        &request(
            "GET",
            "/livez",
            &rebound,
            &[("Origin", &format!("http://{rebound}"))],
        ),
    )
    .await;
    assert!(
        String::from_utf8_lossy(&refused).starts_with(&forbidden),
        "the origin check runs before everything, including `/livez`",
    );

    // ── 4. an unknown upgrade path is destroyed ─────────────────────────────
    // *"a mismatch silently drops the connection with no HTTP status"* — every
    // shipped client hardcodes the path, so the failure a client must recognise
    // is a bare close. Zero bytes is the assertion; anything else, including a
    // 404, would be a response.
    let destroyed = raw(&origin, &upgrade("/socket", &authority, &[])).await;
    assert!(
        destroyed.is_empty(),
        "an upgrade to an unknown path must be destroyed with **no HTTP response \
         at all**, and {} byte(s) came back: {:?}",
        destroyed.len(),
        String::from_utf8_lossy(&destroyed),
    );

    // The right path, same request shape, answers 101 — so the destruction is
    // about the path and not about the request being malformed.
    let accepted = raw_headers(
        &origin,
        &upgrade(via_app::http::REALTIME_ROUTE, &authority, &[]),
    )
    .await;
    assert!(
        String::from_utf8_lossy(&accepted).starts_with("HTTP/1.1 101 "),
        "{:?}",
        String::from_utf8_lossy(&accepted),
    );

    // And a *plain* GET of the same unknown path is an ordinary 404: the rule
    // is "an upgrade to a foreign path", not "a foreign path".
    let missing = raw(&origin, &request("GET", "/socket", &authority, &[])).await;
    let missing = String::from_utf8_lossy(&missing).into_owned();
    assert!(missing.starts_with("HTTP/1.1 404 "), "{missing}");
    assert!(
        missing.ends_with(&format!(
            "{{\"error\":\"{}\"}}",
            via_app::http::NOT_FOUND_BODY
        )),
        "{missing}",
    );

    // An origin-refused request carries no `X-Request-Id`: `enforceSameOrigin`
    // answers and returns without calling `next()`, so the layer that would
    // have set the header never runs.
    let refused = raw(
        &origin,
        &request(
            "GET",
            "/api/tasks",
            &authority,
            &[("Origin", "https://attacker.example")],
        ),
    )
    .await;
    let refused = String::from_utf8_lossy(&refused).to_lowercase();
    assert!(
        !refused.contains(via_app::http::middleware::REQUEST_ID_HEADER),
        "{refused}",
    );

    let _ = gateway.stop();
}

#[tokio::test(flavor = "multi_thread")]
async fn the_upgrade_requires_a_signed_cookie_and_refuses_a_forged_one() {
    // `browser` is the multi-tenant mode where the cookie *is* the identity.
    // In `personal` mode — the default — every request resolves to one owner
    // and an upgrade never returns 401, so this case would assert nothing.
    let machine = Machine::new();
    let gateway = machine.start_harness_gateway(
        &Script::conversation(),
        None,
        &[
            ("VIA_IDENTITY_MODE", "browser"),
            ("VIA_AUTH_SECRET", AUTH_SECRET),
        ],
    );
    gateway.require_serving();
    let origin = gateway.origin();
    let authority = origin.trim_start_matches("http://").to_owned();

    // ── no cookie ───────────────────────────────────────────────────────────
    let refused = raw(
        &origin,
        &upgrade(via_app::http::REALTIME_ROUTE, &authority, &[]),
    )
    .await;
    let refused = String::from_utf8_lossy(&refused).into_owned();
    assert!(
        refused.starts_with("HTTP/1.1 401 "),
        "an upgrade has no response headers to carry a `Set-Cookie`, so an \
         unrecognised client is refused rather than silently given a new owner: \
         {refused}",
    );
    // The catalogued refusal is
    // `HTTP/1.1 401 Unauthorized\r\nConnection: close\r\nContent-Type:
    // text/plain\r\n\r\nidentity required`. Upstream writes those bytes onto
    // the raw socket; here hyper serializes the response and lower-cases the
    // header names and adds `content-length` and `date`
    // (`docs/deviations/phase-5-via-app.md`). Everything a client parses — the
    // status, the connection close, the media type and the body — is identical,
    // so the comparison is case-insensitive on the header names and exact on
    // the rest.
    let headers = refused.to_lowercase();
    assert!(headers.contains("content-type: text/plain"), "{refused}");
    assert!(headers.contains("connection: close"), "{refused}");
    assert!(refused.ends_with("identity required"), "{refused}");

    // ── a forged cookie ─────────────────────────────────────────────────────
    // Three shapes, each of which a sloppy validator lets through: an owner id
    // with no signature at all, a signature for a *different* owner, and one
    // signed with a secret the Gateway does not have.
    let owner = "user_11111111-2222-3333-4444-555555555555";
    let other = "user_99999999-8888-7777-6666-555555555555";
    for forged in [
        format!("{}={owner}", via_core::identity::IDENTITY_COOKIE_NAME),
        format!(
            "{}={owner}.not-a-signature",
            via_core::identity::IDENTITY_COOKIE_NAME
        ),
        signed_cookie(AUTH_SECRET, other).replace(other, owner),
        signed_cookie("a-secret-the-gateway-does-not-have-and-never-had", owner),
    ] {
        let refused = raw(
            &origin,
            &upgrade(
                via_app::http::REALTIME_ROUTE,
                &authority,
                &[("Cookie", &forged)],
            ),
        )
        .await;
        let refused = String::from_utf8_lossy(&refused).into_owned();
        assert!(
            refused.starts_with("HTTP/1.1 401 "),
            "a forged cookie minted an owner: {forged} -> {refused}",
        );
    }

    // ── the genuine one ─────────────────────────────────────────────────────
    // Computed from `hmac` and `sha2` here, not from `via-core`, so this is a
    // second implementation agreeing with the first rather than one agreeing
    // with itself.
    let genuine = signed_cookie(AUTH_SECRET, owner);
    let accepted = raw_headers(
        &origin,
        &upgrade(
            via_app::http::REALTIME_ROUTE,
            &authority,
            &[("Cookie", &genuine)],
        ),
    )
    .await;
    assert!(
        String::from_utf8_lossy(&accepted).starts_with("HTTP/1.1 101 "),
        "a correctly signed cookie must be accepted, or the refusals above are \
         only proving the endpoint is broken: {:?}",
        String::from_utf8_lossy(&accepted),
    );
    // …and the socket the client library opens with it works end to end.
    let mut client = Socket::try_connect(&origin, "voice-e2e-cookie", Some(&genuine))
        .await
        .expect("the upgrade is accepted");
    client.hello(via_i18n::Locale::En).await;

    // The HTTP half *issues* rather than refuses, and the attributes are a
    // catalogued contract of their own.
    let issued = gateway.api().get("/api/health").await;
    assert_eq!(issued.status, 200);
    let set_cookie = issued
        .headers
        .get("set-cookie")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    assert!(
        set_cookie.starts_with(&format!("{}=", via_core::identity::IDENTITY_COOKIE_NAME)),
        "{set_cookie}",
    );
    for attribute in via_core::identity::IDENTITY_COOKIE_ATTRIBUTES {
        assert!(set_cookie.contains(attribute), "{set_cookie}");
    }
    assert!(
        set_cookie.contains(&format!(
            "Max-Age={}",
            via_core::identity::IDENTITY_COOKIE_MAX_AGE_SECONDS
        )),
        "{set_cookie}",
    );
    assert!(
        !set_cookie.contains("Secure"),
        "nothing terminates TLS here and no proxy header was sent: {set_cookie}",
    );

    drop(client);
    let _ = gateway.stop();
}

#[tokio::test(flavor = "multi_thread")]
async fn no_credential_reaches_the_log_file() {
    // The shipped binary, a real credential, a real run, and a grep of the file
    // afterwards. Redaction is a property of what was *written*, so nothing
    // short of reading the file back can assert it.
    let machine = Machine::new();
    let gateway = machine.start_via_gateway(&[
        ("DASHSCOPE_API_KEY", API_KEY),
        ("VIA_AUTH_SECRET", AUTH_SECRET),
        // A closed loopback port, so the realtime connection fails *locally*
        // and at once. The failure path is where a credential is most likely to
        // be logged — an error carrying the URL it tried, or the options it was
        // built from — and it costs no network to reach.
        ("VIA_REALTIME_BASE_URL", "ws://127.0.0.1:1"),
    ]);
    gateway.require_serving();

    // Exercise the surfaces that touch the configuration: the health payload
    // (which publishes a *signature* of the realtime configuration, never the
    // configuration), and a turn that has to try to open a session.
    let health = gateway.api().get("/api/health").await;
    assert_eq!(health.status, 200);
    let rendered = health.body.to_string();
    assert!(
        !rendered.contains(API_KEY) && !rendered.contains(AUTH_SECRET),
        "a credential reached `/api/health`",
    );

    let mut client = Socket::connect(&gateway.origin(), "voice-e2e-redaction").await;
    client.hello(via_i18n::Locale::En).await;
    client.say("hello?").await;
    // The turn fails — there is nothing on port 1 — and the failure is what is
    // being provoked. Whether the client is told is `tests/conversation.rs`'s
    // question; here the only thing that matters is that the attempt happened.
    let _ = client
        .wait_for(BUDGET, |frame| frame["type"] == "error")
        .await;
    drop(client);

    let run = gateway.stop();
    assert_eq!(run.code, Some(0), "stderr: {}", run.stderr);

    // ── the grep ────────────────────────────────────────────────────────────
    let path = machine.gateway_log();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    // Vacuity guard: an empty log contains no secret either.
    let records = machine.gateway_records();
    assert!(
        records.len() >= 2,
        "the Gateway wrote {} record(s); a redaction gate over an empty file asserts \
         nothing",
        records.len(),
    );
    assert!(
        records
            .iter()
            .all(|record| record["schema"] == via_log::LOG_SCHEMA),
        "every line is a `{}` record",
        via_log::LOG_SCHEMA,
    );

    for secret in [API_KEY, AUTH_SECRET] {
        assert!(
            !text.contains(secret),
            "`{}` carries a credential. The offending line(s):\n{}",
            path.display(),
            text.lines()
                .filter(|line| line.contains(secret))
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
    // The CLI's own log is written by the same process and is held to the same
    // rule.
    let cli = std::fs::read_to_string(machine.log_dir().join("cli.log")).unwrap_or_default();
    for secret in [API_KEY, AUTH_SECRET] {
        assert!(!cli.contains(secret), "`cli.log` carries a credential");
    }
    // …and the console half of the same run, which a supervisor captures.
    for secret in [API_KEY, AUTH_SECRET] {
        assert!(
            !run.stdout.contains(secret) && !run.stderr.contains(secret),
            "a credential reached the console",
        );
    }
}
