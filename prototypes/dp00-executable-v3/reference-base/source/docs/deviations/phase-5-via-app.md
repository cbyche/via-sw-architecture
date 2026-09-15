# Phase 5 deviations — `via-app`

The composition root and the HTTP/WebSocket server: every service by injection,
the fourteen-route table, `/api/health`'s field order, `WS /api/realtime`, the
accept loop, the offline hand-off, and startup/shutdown in upstream's order.

Read [`phase-0.md`](phase-0.md) for what a deviation record is and is not.

*128 tests · clippy clean · `brand_leak.py` clean · 22/22 mutations killed ·
19 deviations*

Ported from `server/src/app/{gateway-application,bootstrap,offline-notifications}.mjs`
and `server/src/index.mjs`, against
`server/test/{gateway-application,embedded-gateway,service-endpoint,input-suspend-protocol,offline-notifications,request-security}.test.mjs`
and `test/{gateway-contract,gateway-setup}.test.mjs`.

`via_core` owns the origin allow-list, the DNS-rebinding defence, the signed
identity cookie and the setup gate; `via_protocol` owns the 52 wire names, the
protocol version and the capability list; `via_voice` owns the input
arbitration, the voice slot, the Injection Gate, the mode plan and the upgrade
decision; `via_work` owns the Work record and its events; `via_i18n` owns every
sentence. None of them is restated here — this crate is the wiring.

---

## The four things this crate exists to get right

**Importing the factory must not bind a port.** Upstream needs an `autoStart`
flag for it, and `bootstrap.mjs` exists to call the factory at module scope so
that importing it starts a server. Here the separation is structural:
`GatewayApplication::build` owns no socket, `bind` is a separate `async`
fallible call, and `Serving::run` is a third. There is no flag to get wrong and
no module-level side effect to import, so `bootstrap.mjs` has no counterpart and
needs none. `application.rs::binding_port_zero_reports_the_resolved_port` is the
`PORT=0` half.

**`/api/health`'s field order is a contract**, and a `HashMap` cannot hold it.
The payload is a struct — `serde` emits struct fields in declaration order
unconditionally — so the order is the compiler's to keep.
`tests/contracts.rs::the_health_field_order_is_the_catalogued_one` recovers the
twenty-six keys from the catalogue's own prose and compares, so neither the
served order nor the constant can drift alone.

**A WebSocket upgrade to any other path gets no HTTP response at all.** Not a
404 — `socket.destroy()`. An axum handler can only *return a response*, so this
crate drives hyper itself; see the note on the accept loop below.
`tests/realtime.rs::an_upgrade_to_another_path_is_destroyed_with_no_http_response`
reads from a raw `TcpStream` and asserts zero bytes.

**The setup gate runs before the lease is touched.** Upstream's own comment
says why: *"A Gateway that listens but cannot connect its voice is harder to
diagnose than a refusal the user can act on."*
`tests/lifecycle.rs::an_unconfigured_start_is_refused_before_the_lease_is_touched`
asserts the refusal **and** that `gateway.lock` does not exist afterwards.

---

## Structural

- **The accept loop is this crate's, not `axum::serve`'s.** One catalogued
  contract cannot be expressed as a response, and it is the destroy rule above.
  `serve.rs` runs the `TcpListener` loop and hands each connection to
  `hyper_util::server::conn::auto`; the service future for a destroyed upgrade
  **never resolves**, it signals a per-connection `oneshot`, and the connection
  task drops the hyper future — which drops the socket with nothing written.
  Everything else goes to the router untouched. `hyper` and `hyper-util` are
  named in the root manifest for this; see [Root manifest](#root-manifest).

- **Layer 3 arrives as four questions, not as a singleton.** Upstream's
  handlers reach `agent`, a module-level object, and ask it `describe()`,
  `status()`, `uiUrl()` and `respondPermission()` — which is exactly what makes
  `gateway-application.mjs` hard to test and why its factory takes `agent` by
  injection in the first place. `backend::GatewayBackend` is those four calls as
  a trait, with `FrontendOnlyBackend` reproducing `agent-client.mjs:164-189`'s
  literal fallback objects and `HarnessBackend` bridging a validated
  `HarnessDescriptor` plus `via_coordinator::PermissionBroker`.

- **`describe()` answers a `serde_json::Map`, not a struct.**
  `/api/health.backend` is `{...agent.describe(), ...agent.status()}` — a
  *spread* of two objects whose key sets belong to the driver. A struct would
  freeze a shape upstream deliberately leaves open, and `docs/rebrand.md` keeps
  every adapter key (`kind`, `baseUrl`, `acpConnection`, `sessionModel`, …)
  verbatim. The one key the Gateway itself reads is `capabilities.backendUi`,
  and `BackendDescription::backend_ui` is that read expressed once.

- **The realtime route carries none of the HTTP middleware.** Upstream attaches
  `attachRealtimeGateway` to `server.on('upgrade')`, not to Express, so no
  Express middleware ever runs for an upgrade — and the two refusals genuinely
  differ. An HTTP route answers a disallowed origin with
  `{"error":"origin not allowed"}` in JSON; an upgrade answers the same sentence
  in **plain text** after a raw status line. The sharper difference is identity:
  the HTTP layer *issues* a cookie when a request has none, while the upgrade
  path only ever *resolves* one and answers 401 otherwise. Sharing a layer would
  silently mint an owner for every unauthenticated socket. `http::router` merges
  two routers for exactly this.

- **The model turn sits behind `realtime::VoiceEngine`.**
  `docs/deviations/phase-5-via-voice.md` splits `realtime-gateway.mjs` in two:
  *"the gateway's socket plumbing is `via-app`'s"*, and the decisions that
  plumbing makes are `via-voice`'s. The trait is where that split lands at
  runtime. Three reasons, and the third is the one that matters: `dictation`
  mounts no model at all, `agent`-without-a-harness and `interface` are
  `ModePlan` decisions taken *before* an engine is asked for, and a connection
  driven by `RecordingEngine` exercises the whole frame vocabulary with no
  provider, no credential and no network. **Binding `via_realtime::RealtimeSession`
  to it is the remaining phase-5 step**; `NoEngineFactory` is the shipped
  default and is a supported configuration, not a broken one.

- **`taskManager` and `taskStore` are one injected service.** Upstream takes
  both because its `TaskManager` does not own its store; `via_work::WorkManager`
  does, and `store_health()` / `flush()` go through it. `ServicesBuilder::work`
  is the single replacement point.

- **The permission policy is a `tokio::sync::Mutex`, not an owning task.**
  `docs/architecture.md` §11 requires an owning task for *ordering* invariants,
  and names the four that have one. This is not one of them: the policy is a
  bounded LRU whose operations are map reads and writes with no ordering
  requirement between owners.

---

## Behavioural

- **A request body is read with JavaScript's coercions, not serde's.**
  `String(req.body?.decision || '')` turns `{"decision": true}` into `"true"`,
  which then fails the membership test and reaches the **catalogued 400**. A
  typed `Option<String>` field answers 422 instead, and 422 is not in the
  catalogue. `http::coerce` reproduces `String(x || '')` and `Number(x)`; the
  one thing it does not reproduce is `String()` of an array or object, which is
  `"1,2"` / `"[object Object]"` in JavaScript and the value's JSON rendering
  here. Neither form is ever equal to `always`, `reject`, or an owner a host
  would use, so the observable outcome is identical.

- **`/api/health` gains a `sessionModes` block**, appended *after* the
  twenty-six contract keys so their order is untouched.
  `docs/architecture.md` §2 requires the mode degradation to be *"reported on
  `/api/health`, never silent"* and names two — `agent` degrades to `direct`
  with no harness, and `interface` behaves as `direct` until `via-context` lands
  in phase 7. Upstream has no `SessionMode` at all, so there is nothing to
  reproduce. **Superseded in phase 7** for the `interface` row — see
  [`phase-7.md`](phase-7.md) §11. The per-live-session half is `voiceClients.degradedModes`, which is
  `via_voice::aggregate`'s.

- **`gatewayInstanceId` and `gatewayStartedAt` are passed, not read from the
  environment.** Upstream writes `QWEN_AUDIO_GATEWAY_INSTANCE_ID` in
  `index.mjs` and reads it back in `gateway-application.mjs`.
  `std::env::set_var` is `unsafe` in edition 2024, and a process global would
  make two Gateways in one test binary report each other's identity, so
  `InstanceIdentity` is a value the composition root is handed.

- **An unmatched path is a JSON 404.** Upstream's catch-all serves
  `index.html`, so *"unknown `/api` paths therefore return `index.html` rather
  than a JSON 404"* — which the catalogue states as a behaviour. VIA ships no
  web UI, so `/skins/*`, `express.static(web/dist)` and the `GET *` fallback are
  all gone, and with nothing to serve an unmatched path answers
  `{"error":"not found"}` — the body upstream's own `/skins` miss produces,
  reused rather than invented. `http::FALLBACK_IS_JSON_404` names the
  divergence so a conformance row can point at a constant.

- **An origin rejection carries no `X-Request-Id`.** Upstream's
  `enforceSameOrigin` answers 403 and returns without calling `next()`, so the
  header the *next* middleware would have set is never set. This looks like a
  bug and is not; the catalogue's *"on all routes, including 403/404"* is about
  the routes' own refusals, which do carry it.
  `tests/http_routes.rs::a_disallowed_origin_gets_403_even_on_livez` asserts the
  absence so it cannot be "fixed" by accident.

- **The log correlation context is per-call, not ambient.** Upstream wraps
  `next()` in `AsyncLocalStorage`, which follows the value across every `await`.
  `via_log::run_with_log_context` is a thread-local and does not (recorded in
  `via-log`'s own deviations), so `identity_layer` attaches `requestId` and
  `ownerId` to the request as a typed extension and handlers read them there. A
  handler that forgets is missing two fields, not correlating the wrong ones.

- **The upgrade refusals carry two extra headers.** Upstream writes
  `HTTP/1.1 403 Forbidden\r\nConnection: close\r\nContent-Type: text/plain\r\n\r\n…`
  onto the raw socket, so the response has exactly three header lines; here it
  is built normally and hyper adds `content-length` and `date`. The status, the
  `connection: close`, the content type and the body — everything a client
  parses — are identical.

- **The SSE stream sends a keep-alive comment.** Upstream writes nothing between
  events; a `tokio` stream behind a proxy with an idle timeout is closed
  silently. Comments are not events — a client's `EventSource` never surfaces
  them — and the `data: {…}\n\n` frame format the catalogue names is untouched.

- **`voice.deactivated` is sent by the registry, not by the slot holder.**
  Upstream's `deactivate(replacement)` sends `playback.clear` and then
  `voice.deactivated` carrying **the replacement's descriptor**. A
  `&dyn via_voice::VoiceClient` cannot supply a descriptor, so
  `VoiceClientRegistry::activate` sends both frames itself, in that order, once
  it can see who won. Doing it inside `deactivate` with `holder: null` would
  drop the field a client uses to say *"the desktop app took the microphone"*.

- **The offline hand-off is a channel, not `parentPort.postMessage`.** VIA has
  no Electron host. The two payload shapes and the single message type are
  reproduced exactly, `qwen-audio-agent:offline-notification` renamed to
  `via:offline-notification` per `docs/rebrand.md`. The `error` key is
  `Option<Option<String>>` because upstream has **three** states, not two: the
  progress literal has no `error` key at all, and the terminal literal always
  has one and writes `null` when the Work did not fail.

- **The offline timers need no `setTimer` seam.** Upstream injects one so its
  test can fire the timer by hand; `#[tokio::test(start_paused = true)]`
  advances every `tokio::time::sleep` at once, so the production path is the one
  under test.

- **`Startup::detached` exists.** `via chat` runs an in-process Gateway and has
  already taken the CLI lease; taking a second one would refuse itself. It skips
  the gate and the lease and nothing else.

---

## Not ported

- **`/skins/*` and `express.static(web/dist)`** — there is no web UI, and
  `web.same-origin-ui` / `web.skin-assets` are in
  `via_protocol::DROPPED_UPSTREAM_CAPABILITIES` rather than in the advertised
  list. The route table says so rather than pretending the capability exists.

- **The `MemoryExtractor`, `MarkdownContextStore`, `ReminderScheduler` and
  `BackendAvailability` construction** that `gateway-application.mjs:99-177`
  does inline. Each is a `via-conversation` / `via-work` / `via-backends` type
  with its own builder; the composition root takes them already built, which is
  what makes `Services` replaceable field by field.

- **`x-powered-by`** — axum sets no such header, so there is nothing to disable.

---

## The close sequence

`gateway-application.mjs:509-530`, in order. Steps 1, 3 and 5 belong to handles
this crate does not own — the backend-availability probe, the reminder
scheduler, and the realtime gateway, each closed by whoever built it — so what
`application::close` reproduces is steps 2, 4 and 6 in the positions upstream
puts them:

1. ~~`backendAvailability.close()`~~ — the probe's owner's.
2. `unsubscribeOfflineNotifications?.()` → `OfflineNotifications::close`.
3. ~~`reminderScheduler?.close()`~~ — the scheduler's owner's.
4. `inputArbitration.close()` → `release_all()`. Upstream's comment is the
   reason: *"A Gateway that stops serving cannot honour a resume, so held state
   must not survive into the next run."*
5. ~~`realtimeGateway?.close?.()`~~ — the connections end with the accept loop.
6. `taskStore?.flush?.()` → `WorkManager::flush`.
7. stop listening.

The process-level sequence is `runtime::shutdown`: clear the heartbeat **first**,
stop every `Stoppable` **concurrently** (`Promise.all`, not `await` then
`await`), log one `backend.stop_failed` per refusal, flush the logger whatever
happened, release the lease. `runtime::shutdown_with_timeout` arms the
catalogued 2000 ms and gives up, so a harness that never answers cannot wedge
the process.

---

## Gates

`scripts/risky_unwrap.py` gained `realtime` to `SUBSYSTEM_DIRS`:
`via-app/src/realtime/` decodes every WebSocket frame a client sends, and the
client is the outermost untrusted party the Gateway has — any browser, any
script, any process that can reach the port. The baseline is unchanged
(`ClientFrame::decode` answers `None` for every shape it cannot read and never
unwraps a parse); the listing puts a future `.unwrap()` on a client frame in
audit scope. `http` was already listed.

`crates/via-i18n/tests/catalog.rs` gained `gateway.task_not_found` and
`gateway.task_not_active` to `NOT_CHINESE_PROSE`: both are the literal 404/409
bodies of the Work HTTP surface, written in English in every locale because a
client branches on the string. They live in the catalog rather than as `const`s
so the one rule — every user-visible string is a key — has no exceptions.

---

## Root manifest

Two additions, both already in the graph beneath `axum`:

```toml
hyper = { version = "1", features = ["server", "http1"] }
hyper-util = { version = "0.1", features = ["server", "tokio", "service"] }
```

`via-app` owns the accept loop rather than calling `axum::serve`, because one
catalogued contract cannot be expressed as a response: a WebSocket upgrade to
any path other than `/api/realtime` is `socket.destroy()`'d with **no HTTP
response at all**. Reproducing that needs the connection future itself, which is
hyper's. Naming the two crates in the workspace table pins one copy of each.

`tokio-stream` also gained the `sync` feature, for
`wrappers::BroadcastStream` — the Work subscription behind
`GET /api/tasks/{id}/events`.
