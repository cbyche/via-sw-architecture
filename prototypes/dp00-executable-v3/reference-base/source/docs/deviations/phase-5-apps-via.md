# Phase 5 deviations — `apps/via`

`via gateway` and `via chat`, made real. The binary's argument surface shipped
in phase 1 with four verbs refusing and naming a phase; this phase replaces two
of those refusals with the thing they named.

Read [`phase-0.md`](phase-0.md) for what a deviation record is and is not.

*367 tests · clippy clean · `brand_leak.py` clean · `risky_unwrap.py` 0 ·
13/13 mutations killed · 15 deviations*

Ported from `server/src/index.mjs`, the `command === 'gateway'` arm of
`cli/src/launcher.mjs:381-421`, `ensureRuntime` in `cli/src/runtime.mjs:369-430`,
`tui/src/text-cli.mjs` (221 lines), `tui/src/terminal-commands.mjs` and
`shared/gateway-client.mjs`.

---

## The phase-5 milestone

`docs/architecture.md` §15: phase 5 ends when *"`via chat` works end to end — no
audio hardware, no model weights"*. `tests/milestone.rs` is that sentence
executed, and it is deliberately not a mock of itself:

- the Gateway is booted through **[`gateway::boot`]** — the same call
  `via gateway` makes — with the real router, the real accept loop, the real
  `WorkManager`, the real `Coordinator` and the real `AnnouncementManager`;
- exactly **two** things are substituted, each at the seam its own crate
  already provides: `gateway::SessionOpener` (`via-realtime-mock`'s in-process
  transport instead of a WebSocket) and `via_downstream::DownstreamAgent`
  (`ScriptedHarness` instead of an ACP child);
- the client sends `via chat`'s **own** frames, composed by
  `chat::protocol::connect_frame`, so what is asserted is the client a user
  runs.

It asserts, in one test: a direct answer with **no Work created**; a
`spawn_thinking` reaching the Work queue; `running → completed` on the socket
with the backend's own result text on the record; and the result delivered back
**through the announcement window** — asserted twice, once as the spoken
transcript the user reads and once against the mock's transcript, where the
`conversation.item.create` carrying the result proves it reached the model's
conversation rather than only the socket.

`tests/chat_client.rs` asserts the other half, through
`CARGO_BIN_EXE_via`: the binary connects, `/help` `/tasks` `/cancel` `/exit`
answer, autostart starts a Gateway and stops it again, and a second
`via gateway` refuses with `VIA_GATEWAY_ALREADY_RUNNING` while the first is
serving. A green milestone test with a `via chat` that could not open a socket
would be a green test and a broken product.

---

## Two changes to shipped crates

Both are additive, both are things phase 5 explicitly left owed, and neither
changes an existing caller.

- **`via_app::EngineFactory::open_for`, and a shared Injection Gate.**
  `docs/deviations/phase-5-via-app.md` names the one thing it left undone:
  *"**Binding `via_realtime::RealtimeSession` to it is the remaining phase-5
  step**"*. That binding could not be written against `EngineFactory` as it
  stood, because `open(&self, provider: &str)` hands an engine nothing to write
  its frames onto — and a model turn is only observable as frames on a socket.
  Upstream's `ensureFrontend()` is a closure over the whole connection scope; a
  trait cannot close over a scope, so the four things that closure actually uses
  are passed instead, as `EngineContext { provider, owner_id, session_id,
  outbound, gate }`. `open_for` **defaults to `open`**, so `NoEngineFactory` and
  `RecordingEngineFactory` are untouched and `via-app`'s own tests pass
  unchanged.

  The gate is the sharp half. `via_app::SharedGate` makes the connection's
  `InjectionGate` an `Arc<Mutex<…>>` shared with the engine, because
  `docs/architecture.md` §11's third invariant *is* that gate: the socket feeds
  it the playback receipts and the announcement manager reads it. Two gates
  would drift the first time one missed a receipt, and the symptom would be a
  finished result spoken over the user.

- **`via_app::heartbeat` hands the lease back.** It took a
  `GatewayLeaseHandle` **by value** and returned `()`.
  `GatewayLeaseHandle::release` consumes the handle and the type deliberately
  has no `Drop` — *"a clean shutdown must reach this"* is a property `via-lock`
  wants the compiler to hold — so a heartbeat that swallowed the handle made
  releasing the lease unexpressible, and every Gateway would have left a
  `gateway.lock` naming a dead pid behind it. It now returns the lease, which
  `gateway::run` releases as the last step of the close sequence.
  `tests/setup_gate.rs::a_configured_machine_takes_the_lease_holds_it_and_gives
  _it_back` is the assertion, and the mutant that drops the release is killed.

---

## Structural

- **The lease lives in this binary, not in `via_app::Startup`.**
  `Startup::begin` runs the gate and takes the lease, and `Runtime` keeps the
  handle privately — but `heartbeat` needs it by value and there is no way to
  take it out. Rather than grow a second accessor, `gateway::boot` reproduces
  `index.mjs`'s ordering literally: gate, then lease, then compose, then bind,
  then publish the origin. `Startup::detached` remains what an embedder that
  owns both uses.

- **`main` no longer holds the process-wide stdout lock.** It did —
  `std::io::stdout().lock()` for the whole of `dispatch` — which is harmless for
  five verbs and a **deadlock** for the sixth: `via gateway` and the Gateway are
  one process here, where upstream's CLI is the Gateway's *parent*, so the
  Gateway's console log sink writes to the same stdout from a worker thread and
  blocks on a lock the main thread never releases. It hung on the first
  `gateway.ready` line. `Stdout::write_fmt` takes the lock once for a whole
  format, so the banner is still written atomically.

- **The signal handlers are installed before the banner.** `tokio::signal`
  registers on first *poll*, so a future created and then awaited after the
  banner leaves a window in which SIGTERM takes its default disposition and
  kills the process with the lease still on disk. `gateway::shutdown_signal`
  opens both streams eagerly and returns a future over them. The mutant that
  removes the handler is killed by
  `sigint_shuts_a_running_gateway_down_as_cleanly_as_sigterm`.

- **A reactor per command, not `#[tokio::main]`.** Four of the six verbs need no
  reactor at all, and `via config` paying for a thread pool to print a path is
  paying for nothing. `gateway::run` and `chat::run` each build one.

- **The runner owns the cancellation race.** `Coordinator::turn` awaits
  `session.prompt` without watching the Work's signal, because a *delegated*
  stop is the backend's to confirm (`docs/architecture.md` §4). A coordinator
  turn that has not delegated yet has nobody to ask, and upstream's task manager
  aborts it locally (`task-manager.mjs:727-733`, *"`delegated` goes to the
  coordinator, everything else aborts locally"*). `DelegationRunner` races the
  turn against `WorkContext::signal`, and `CoordinatorCanceler` calls
  `CancelRequest::abort()` in **both** branches — `via-work`'s no-canceler path
  raises the abort itself, so an installed canceler inherits that duty.
  Skipping it makes `DELETE /api/tasks/:id` never answer, because the reply
  waits on the *confirmed* stop. That mutant is killed by timeout, which is the
  honest way for it to die.

- **The engine merges a response context only when the merge has something to
  say.** Upstream's `mergeResponseContext` is a JavaScript spread, so a patch
  with no keys changes nothing; `via_voice::merge_response_context` assigns and
  preserves a named list of progress flags, which its own
  `a_later_delta_without_metadata_keeps_the_correlated_context` asserts. The
  difference is invisible until an uncorrelated `response.text.delta` erases the
  turn id its own tool call is checked against — at which point every
  `spawn_thinking` is `Superseded` and nothing delegates. The caller is where
  the rule belongs: merge on first sighting, and afterwards only when the
  provider actually echoed a correlation.

- **The `SinkObserver` is a bounded channel plus one forwarding task.**
  `CoordinationObserver` is synchronous and `WorkEventSink::emit` is `async`. A
  `tokio::spawn` per event would have reordered `delegated` and
  `delegation.completed` under load, and correlation is exactly what §11's
  fourth invariant protects.

---

## Behavioural

- **`via chat` connects with `voiceEnabled: false` and an explicit
  `outputEnabled: true`.** Upstream sends `voiceEnabled: true` with a comment
  explaining that it is overloading the legacy single flag *"to enable the
  output channel to receive task announcements"*. VIA sends the explicit pair
  `active-voice-clients.mjs:46-64` provides for exactly this case.
  `via_voice::client_voice_capabilities` resolves both spellings to the same
  answer — `{input: false, output: true, arbitration: false}` — and
  `the_connect_frame_declares_output_without_voice` asserts that equivalence
  directly, so this is a clearer statement of upstream's behaviour rather than a
  change to it. The mutant that turns output off is killed.

- **`via chat` prints the Work plane.** Upstream's text CLI drops every `task.*`
  frame, so a delegation that takes two minutes looks like a hang. `via chat` is
  the harness the Work queue is debugged with; a harness that cannot see
  `queued → running → completed` cannot do that job. One dim line per move,
  `chat.task_event`.

- **`--no-autostart` is VIA's own flag.** Upstream has no flag because it has no
  choice to express: `tui` always refuses
  (`cli/src/launcher.mjs:427-432`) and the desktop path always starts one
  (`ensureRuntime`). `via chat` is both clients at once, so which one it is has
  to be sayable. The default is autostart, because a user who typed `via chat`
  wants to chat; `--no-autostart` gets the catalogued
  `Gateway is not running: {url}. Run \`via gateway\` first`.

- **`/cancel` will not target a `scheduled` Work.** `text-cli.mjs:58-60` lists
  four statuses and `via_voice::tools::CANCELLABLE_STATUSES` lists five — the
  model's own cancel tool may stop a reminder that has not fired, and
  *"the first cancellable one"* picked out of a list the user cannot see must
  not be a timer they set on purpose. Both constants are asserted against each
  other in `chat::protocol`.

- **`/cancel` has no client-side timeout.** `DELETE /api/tasks/:id` answers only
  when the cancellation is **confirmed** (`docs/architecture.md` §4), so a
  backend that never confirms leaves the command waiting. This is upstream's
  behaviour: only its health probe carries `AbortSignal.timeout(1500)`, and its
  `api()` helper carries none. Recorded rather than fixed, because a client-side
  timeout would report "cancelled" for a stop nothing confirmed, which is the
  optimistic cancellation §4 exists to forbid.

- **A realtime front end that will not open reaches the client.** Upstream
  answers a failed `ensureFrontend()` with `send(ws, {type: 'error', message})`
  (`realtime-gateway.mjs:508`). Logging it and returning no engine — which is
  what the first draft of `RealtimeEngineFactory::open_for` did — leaves a user
  who typed into `via chat` looking at `turn.started` and then silence, which is
  the one failure a text client cannot diagnose. Everything else still works, as
  `docs/deviations/phase-5-via-app.md` promises: the socket stays open, the Work
  plane still forwards, and `/api/health` still answers.

- **The Gateway's console log and the CLI's banner share stdout.** One process,
  two writers. Both write whole records through `write_fmt`, so lines never
  interleave mid-line, but the banner is not guaranteed to be the *first* line —
  `gateway.lease_acquired` is normally ahead of it. `tests/support`'s
  `Fixture::start` therefore finds the banner by its catalogued prefix rather
  than assuming a position.

- **`voice-chat-only mode` is the banner's second half, not `WebUI: …`.**
  `gatewaySummary(health)` is reproduced field for field
  (`cli/src/launcher.mjs:88-102`, joined with ` · `); upstream's `WebUI：{url}/`
  line is dropped with the web UI itself.

---

## Not ported

- **`findRunningGateway` and `assertGatewayCompatibility`.** Upstream's
  autostart falls back to a Gateway that moved to another local port, and
  refuses to reuse one whose backend, ownership, permission mode or realtime
  model differs from what this invocation asked for. Both belong with
  `via backend`, and the catalogue already carries their sentences
  (`cli.reuse_*`). Owed, and recorded here rather than pretended.

- **The input-side transcription branches of the event switch.**
  `conversation.item.input_audio_transcription.{delta,completed,failed}` and
  `input_audio_buffer.committed` shape the *user's* transcript, which a text
  client never produces — it types. `via_voice::TurnCorrelation` and
  `ServerEvent::streaming_input_transcript` are the ported pieces and are
  waiting for them; they land with `via-realtime-openai` in phase 6, which is
  the first provider that will emit one.

- **`inputPartsFromText`.** Upstream's text CLI promotes `@path` mentions and
  pasted absolute paths into file parts. `via-voice` owns the whole input-parts
  surface (`normalize_input_parts`, `input_file_parts`, the anchors) and the
  socket already accepts `input.parts`; what is missing is only the client-side
  path scanning. `via chat` sends text, which is what the milestone needs.

---

## Gates

`scripts/risky_unwrap.py` gained `chat` to `SUBSYSTEM_DIRS`: `apps/via/src/chat/`
reads every byte from a Gateway it does not control — `via chat --url
https://voice.example.com` parses that service's JSON bodies and WebSocket
frames — so it is adversarial input in the same sense `via-app/src/realtime/` is,
in the other direction. The baseline is unchanged (0 hits): every read is
`Value::get` / `as_str` with a default, and an unparseable frame is ignored
rather than unwrapped. `apps/via/src/gateway/` needed no listing: `gateway` was
already in `SUBSYSTEM_DIRS`.

`crates/via-i18n/assets/i18n/chat.json` is new — fourteen keys, none of which
needed a `NOT_CHINESE_PROSE` entry, because every `zh` value carries Han
characters. The `zh` column is upstream's own text from `text-cli.mjs`, byte for
byte, with `qwen-audio-agent` renamed per `docs/rebrand.md`.

## Root manifest

**No dependency was added.** `iana-time-zone` was already in the workspace table
(phase 4 added it for `get_current_time`); `apps/via` names it for the same
reason upstream's text CLI calls
`Intl.DateTimeFormat().resolvedOptions().timeZone` — the zone the client reports
on `connect`. `tokio` gained the `io-std` feature at the package level for
`via chat`'s stdin, which is a feature on an existing workspace dependency
rather than a new row.
