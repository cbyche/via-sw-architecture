# Phase 5 deviations — `via-realtime`

Layer 1 — the realtime transport seam: the provider and protocol traits, the
provider registry, one WebSocket session to one provider, the connection-status
ladder and the reconnect backoff.

Read [`phase-0.md`](phase-0.md) for what a deviation record is and is not.


## `via-realtime`

*255 tests · clippy clean · 44/47 mutations killed (3 equivalent) · 25 deviations*

Ported from `server/src/voice/realtime-provider.mjs`,
`realtime-provider-extension.mjs`,
`providers/{provider-registry,registry,ga-protocol,openai-compatible-protocol}.mjs`,
`realtime-connection-status.mjs`, `realtime-errors.mjs` and
`reconnect-backoff.mjs`, against
`server/test/{realtime-provider,realtime-provider-registry,realtime-connection-status,realtime-errors,reconnect-backoff}.test.mjs`.

`via_catalog` already owns the model profiles, the provider keys and aliases and
the configuration-signature hash; `via_i18n` owns every sentence; `via_protocol`
owns `SessionMode`. None of the three is restated here.

**It ships no provider.** `via-realtime-dashscope`, `via-realtime-mock` and the
local providers each own one; what lives here is the shape they share.

### The two things this crate exists to hold apart

**Which service, and which dialect.** `RealtimeProvider` answers the first —
endpoint, credential, model, voice, session payload, error corpus, capability
declaration. `RealtimeProtocol` answers the second — envelope, event names, id
namespaces, correlation. Splitting them is what lets a local model reuse the
OpenAI-Realtime dialect instead of reimplementing it, and it is why nothing above
this crate ever sees a wire format: the Gateway consumes only what
`normalize_incoming` returns. `tests/extension.rs::two_dialects_live_in_one_registry`
and `::a_per_connection_dialect_tags_and_filters_every_frame` are the two that
pin it.

**Capabilities, never a provider name.** Upstream's own words —
*"optional features require opt-in and providers declare known constraints,
without the frontend ever branching on a provider name."* Every behavioural
difference between the two shipped providers is one of five flags, and the
session reads the flag.

### Structural

- **`response-lifecycle.mjs` lands here, not in `via-voice`.** It is 41 lines
  upstream and sits in `server/src/voice/`, but `handleLifecycle` cannot decide
  whether a response is still producing output without it — a crate that owns
  the session has to own the predicate the session branches on.
  `via-voice` reaches it through `via_realtime::lifecycle`.

- **`AgentContext` carries a *composed* session payload, not the raw context.**
  Upstream's `agentContext` is read by `buildFrontendInstructions` and
  `frontendTools`, both of which live in `server/src/voice/frontend-tools.mjs` —
  `via-voice`'s, per `docs/architecture.md` §9. So `instructions` and `tools`
  arrive here already built, and `extra` carries anything else a provider reads.
  Each provider still shapes the payload: the GA dialect flattens `tools` into
  `{ type, name, description, parameters }` inside its own `build_session`,
  exactly as `providers/s2s.mjs` does. The alternative — depending on the prompt
  layer from the transport layer — inverts the crate graph.

- **`project_user_input`'s *default* is the caller's, not this crate's.**
  Upstream falls back to `frontendInputProjection(parts, options)` from
  `shared/input-parts.mjs`, which composes model-visible prose
  (`用户提交了附件，但没有附带文字说明。`) and is therefore prompt-layer text.
  `RealtimeSession::project_user_input` asks the provider first, exactly as
  upstream, and falls back to a `fallback_text` the caller composed.

- **`RealtimeProvider::preflight()` is the seam for upstream's
  `modelProfile?.family === 'unknown'` gate.** `via-catalog` deliberately has no
  `unknown` family — it answers an unrecognised id with an error rather than an
  all-capabilities-false profile — so the check cannot be phrased as "look at
  the profile the provider returned". The provider raises it, and the ordering is
  kept: `preflight()`, then `is_configured()`, then the socket. Neither ever
  opens a connection it already knows is useless
  (`tests/session.rs::preflight_refuses_before_a_socket_is_opened` asserts that
  nothing was written).

- **The output queue is two owning tasks, not one promise chain.**
  `docs/architecture.md` §11 asks for an owning task per ordering invariant; two
  of the four meet here (the announcement window's `active_responses`, and
  delegation correlation). `session/state.rs` owns the socket and the
  correlation; `session/queue.rs` runs one job at a time. They are separate
  because a job blocks on things only the state task can deliver — an item
  receipt, `response.created`, `response.done` — so running it inside the state
  task deadlocks on the first `await`.
  A step that upstream writes as `await whenIdle()` followed by a synchronous
  check-and-register is **one** command here (`BeginResponse`), parked until idle
  by the state task, because splitting it into two round-trips would open a
  window for an inbound `response.created` to land between them.

- **`Transport` is a seam.** Upstream constructs its `ws` socket inline inside
  `connect()`. Everything interesting about the session — correlation, the queue,
  the two watchdogs, the busy-retry ladder — is transport-independent, and a test
  that needs a TCP listener to reach any of it is a test that gets avoided.
  `RealtimeSession::open` takes any `Transport`; `tests/socket.rs` keeps a real
  WebSocket in the suite for the part only a socket proves.

- **The four callbacks became one ordered stream.** `onEvent`, `onError`,
  `onDiagnostic` and `onClose` are `SessionEvent::{Provider, Error, Diagnostic,
  Closed}` on a bounded channel, because the order *between* them is information:
  an error that arrives before `Closed` was the reason for the close. The cost is
  one documented rule — do not await a session method from the task that drains
  the stream.

- **`ResponseContext` is opaque.** Upstream's `context` is an arbitrary object
  the Gateway assembles and this layer never reads; typing its fields here would
  make the transport depend on the Gateway's turn model, which is the coupling
  the correlation seam exists to avoid. Three accessors exist
  (`turn_id`, `task_id`, `authorization_id`) because those are the three keys the
  Gateway reads on *every* correlated event.

- **`testing` is a published module, not `#[cfg(test)]`.**
  `via-realtime-mock`, `via-realtime-dashscope` and `via-voice` all have to drive
  a session, and none should reimplement a provider to do it. `TestProvider` is
  upstream's own `testProvider()` helper grown builder overrides;
  `test_transport()` is the in-memory socket upstream does not need because `ws`
  is trivially stubbed in JavaScript.

- **`emit` takes `&mut self`.** A shared borrow of the state is only `Send` if
  the state is `Sync`, and the state owns a `dyn Sink` that is `Send` but not
  `Sync`. Every caller already holds the exclusive borrow, so this costs nothing
  and keeps the task's future spawnable.

### Corrected, not copied

- **`response.created` with no usable response id no longer hangs.** Upstream
  removes the pending response from the correlation queue and clears its start
  timer *before* checking the id (`realtime-provider.mjs:581-606`), so a pending
  that reaches that branch is never registered anywhere and never settles — the
  caller's promise hangs, and with it the whole output queue, because the queue
  tail awaits that promise. VIA settles it `failed / correlation`. Neither
  shipped dialect can reach the branch; both always carry `response.id`.
  `tests/responses.rs::a_response_created_with_no_id_fails_closed_instead_of_hanging`.

- **A zero sample rate is refused.** Upstream checks `Number.isFinite`, and
  `Number.isFinite(0)` is true, so `inputSampleRate: 0` registers cleanly. It is
  division by zero in the resampler and a `voice.ready` frame that tells every
  client to capture at 0 Hz. `validate_realtime_provider` rejects it with the
  message upstream uses for a *missing* rate, which is what a zero rate is.

### Absorbed by the type system

Upstream spends 190 lines of `provider-registry.mjs` proving that an object
literal really is a provider. Four of its five loops are compile-time here:

| Upstream check | Why it is gone |
| --- | --- |
| each of ten `PROVIDER_METHODS` is a function | the trait |
| each of twelve `PROTOCOL_METHODS` is a function | the trait |
| each of five `CAPABILITY_FLAGS` is a boolean, and no sixth exists | `ProviderCapabilities` |
| each of twelve model/transport capability flags is a boolean | `via_catalog::{ModelCapabilities, TransportCapabilities}` |
| `visibility ∈ {'public','gateway-only'}` | `Visibility` |
| `turnDetection.type` is a non-blank string | `TurnDetectionKind` |

What survives is what a well-typed provider can still get wrong — a blank key or
label, a key a client could never send, a zero sample rate, a zero timeout, a
blank alias, a model profile with a blank id, label or voice — and every one is
tested in `tests/extension.rs`.

The name lists survive as `PROVIDER_METHODS` / `PROTOCOL_METHODS` /
`CAPABILITY_FLAGS` constants, because `via-conformance` asserts the vocabulary
rather than the mechanism.

**One consequence worth stating.** `pending_responses` can hold at most one entry
in VIA, because the queue is a real FIFO rather than a promise chain a test can
push into — so upstream's `Realtime 响应关联冲突` guard is unreachable through the
public API. It is kept anyway, in both places upstream has it
(`begin_response` and `replay_refused`): it is fail-closed behaviour with a
catalogued message, and a future caller that registers a response outside the
queue would need it.

### Added, which is not a port

- **`audio_commit()`.** Upstream never commits an input buffer explicitly —
  both of its providers run server-side turn detection, so the service commits
  for it. `dictation` (`docs/architecture.md` §2) and any push-to-talk client
  need the explicit form, and both dialects spell it the same way, so it is a
  defaulted trait method rather than a per-dialect one.

- **`ActionKind::Barrier`, reached as `RealtimeSession::drain()`.** The Rust
  spelling of upstream's `await frontend.outputQueue`, which reaches into the
  object because JavaScript lets it.

- **`SessionMode` gating.** The mode is VIA's, so its consequences are too:
  every response-creating call answers `skipped / no_model_turn` in `dictation`
  without touching the socket, and `restore_recent_conversation` is skipped
  there because a session with no model has no conversation to give context to.
  `append_audio`, `commit_audio` and the transcript events keep working —
  `tests/extension.rs::a_dictation_session_transcribes_and_nothing_else`.

- **`OutcomePhase::NoModelTurn`**, the one phase upstream does not have, for
  exactly that.

- **`ErrorClass`.** Upstream's `classifyError` returns a free string that the
  Gateway compares against seven literals; the closed enum is the same
  vocabulary, plus `is_suppressed()`, which restates the Gateway's own branch
  table (`realtime-gateway.mjs:1353-1437`) once instead of at four call sites.

- **`STABLE_CONNECTION_MS`.** The catalogued *"backoff is reset only after a
  connection stayed up >= 10000 ms"* rule lives beside the ladder so the caller
  that owns the reconnect loop does not restate it. The caller still decides when
  to apply it.

- **`ReconnectBackoff`'s jitter source is injected**, as upstream's `random`
  option is, and defaults to `getrandom`. A CSPRNG failure returns `0.5` rather
  than panicking: the un-jittered ladder is exactly what `jitterRatio: 0`
  produces, so the fallback degrades the spread, not the behaviour.

### Identity

- **`RESPONSE_CORRELATION_KEY` stays `qwen_audio_request_id`.** It is **KEEP**
  under `docs/rebrand.md`: the key travels to a third-party endpoint VIA does not
  control and is echoed back by it, so renaming it would silently stop
  correlating against a service that still echoes the old name. It is the one
  upstream-branded string this crate publishes, and
  `tests/contracts.rs::nothing_this_crate_publishes_carries_a_brand_it_should_not`
  is the fence around it.

- **One i18n key was added: `realtime.unsupported_frontend_with_options`.**
  Upstream's message is `不支持的 Realtime 前台：${name}（可选 ${keys.join('、')}）`
  and the catalogue already carried only the half without the options list.
  Losing the list makes a mistyped provider key unanswerable, so the key is
  added with the `zh` column byte-identical to upstream. The provider names are
  joined with `、` in `zh` and `, ` in `en`/`ko`, because `dashscope、mock` in an
  English sentence is wrong in a way a reader notices.

- **Three messages are deliberately not localized.**
  `NotConfigured` and `ConnectTimeout` carry the **provider's** own sentence,
  because upstream reads `provider.missingConfigurationMessage` and
  `provider.connectTimeoutMessage` rather than composing one — the two shipped
  providers name different environment variables and one interpolates its
  endpoint. `Transport` carries the transport's own text, and that one is a
  contract: the catalogued DashScope `fatal` corpus matches on
  `unexpected server response: (?:401|403)`, so translating it would stop an
  expired credential from being recognised.

### One note for `via-realtime-dashscope`

`tungstenite` does **not** phrase a refused upgrade the way Node's `ws` does. The
catalogued `fatal` pattern `unexpected server response: (?:401|403)` is `ws`'s
wording; a Rust provider's `classify_error` has to cover tungstenite's phrasing
as well, or an expired `DASHSCOPE_API_KEY` will be classified `other` and
surfaced as a retryable error instead of blocking the connection.
`tests/socket.rs::a_refused_socket_is_a_transport_failure_carrying_the_status_line`
pins the status text and asserts the wording difference, so it fails loudly if
tungstenite ever changes it.

### The three mutants that survive

Named rather than hidden, because each says something about the code:

- **`(exponential + jitter).round().max(0.0)` → drop the `max`.** Equivalent: a
  negative `f64` cast to `u64` already saturates to zero in Rust. The clamp is
  upstream's `Math.max(0, …)`, kept because it states the intent at the point it
  matters rather than relying on a cast's saturation.

- **`response.created` → drop the `cancel_timer()` before the id check.**
  Equivalent: every path out of that block cancels the timer anyway — the
  empty-id branch through `settle`, the normal branch through
  `arm_inactivity_timeout`, which cancels before it arms. Upstream's
  `clearTimeout(pending?.timer)` sits in the same redundant position.

- **FIFO correlation → LIFO.** Unreachable rather than equivalent:
  `pending_responses` never holds more than one entry through the public API,
  because the output queue is a real FIFO that runs one job to completion, so
  `pop_front` and `pop_back` are the same call. The same property is what makes
  upstream's correlation-conflict guard unreachable here — see above.

### Gates

`scripts/risky_unwrap.py` gained `protocol` to `SUBSYSTEM_DIRS`:
`via-realtime/src/protocol/` decodes every frame a realtime provider sends — a
third-party WebSocket service VIA does not control — so it is adversarial input
by definition. The baseline is unchanged (the dialects use `Value::get` /
`as_str` throughout and never unwrap a parse); the listing puts a future
`.unwrap()` on a provider frame in audit scope.

### Root manifest

Unchanged. Every dependency this crate takes was already in the workspace table.
