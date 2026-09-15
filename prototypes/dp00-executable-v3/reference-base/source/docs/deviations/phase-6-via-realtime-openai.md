# Phase 6 deviations — `via-realtime-openai`

Layer 1 — the OpenAI / Azure / litellm realtime provider. **The one crate whose
upstream is ARGO rather than qwen-audio-agent** (`docs/architecture.md` §3): its
`tinicore/src/llm/providers/openai_live.rs` is a 2,704-line client that encodes
four litellm-gateway properties which each cost a device run to discover, and
writing VIA's realtime client from the qwen JavaScript would rediscover every one
of them.

Read [`phase-0.md`](phase-0.md) for what a deviation record is and is not.


## `via-realtime-openai`

*158 tests · clippy clean · 33/33 mutations killed · 14 deviations*

Ported from `tinicore/src/llm/providers/openai_live.rs`, with
`tinicore-traits/src/live.rs` for its event vocabulary and
**`docs/plans/VOICE_AGENT_BRINGUP.md` as the acceptance spec**
(`docs/architecture.md` §14: *"Ported from ARGO as a specification, with its
bring-up doc as the acceptance spec: the dialect walk, the first-frame probe,
GA/pre-GA dual schema, and the duplicate-`response.create` filter."*).

It ships as **one `via_realtime::RealtimeProvider` behind qwen's interface**, not
as a replacement for it. `via-realtime` still owns the session machine, the
registry, the connection status, the response watchdogs, the busy-retry ladder
and the reconnect backoff. This crate owns the **dialect and the connect walk**.

ARGO is credited in `NOTICE` as VIA's Layer 1 upstream — *"Supplies VIA's Layer 1:
the realtime transport and its dialect handling"* — and every ported construct
names its source line in its own doc comment.


### What it is built on rather than restating

| Owned by | What |
| --- | --- |
| `via-catalog` | the `openai` provider row (key, label, default endpoint, identity shape) and `RealtimeIdentity`'s hash and key order |
| `via-core` | the environment reading (`resolve_realtime_frontend`) and `Secret` |
| `via-realtime` | both wire dialects, the session machine, `Transport`, `RealtimeError`, `ErrorClass`, the shared inactivity closure |
| `via-i18n` | every sentence a person or the model reads |
| `via-audio` | `SampleRate` |

**The event decoder is deliberately not ported.** ARGO's `decode_server_event`
maps the wire into ARGO's own `LiveInboundEvent` vocabulary; VIA's Layer 1
consumes provider events directly and already accepts **both** the pre-GA and the
GA event spellings — `via-realtime`'s `RESPONSE_ACTIVITY_TYPES` and `via-voice`'s
response tables each list `response.audio.delta` beside
`response.output_audio.delta` and `response.audio_transcript.*` beside
`response.output_audio_transcript.*`. Porting the decoder would have been a
second vocabulary for the same events, which is the definition of a reimplementation.

The one piece of ARGO's decoder that *is* here is its judgement, not its
mechanism: `is_benign_cancel_race` becomes an `ErrorClass::NoActiveResponse`
classification rather than a swallow, because `via-realtime` already suppresses
that class.


### Structural

- **The walk returns a `Transport`; `RealtimeSession::open` is the door.**
  `via-realtime`'s `RealtimeSession::connect` opens **one** socket to
  `provider.url()`. A candidate walk cannot be expressed through it, and
  `Transport` is the seam `via-realtime` made for exactly this
  (`phase-5-via-realtime.md`, *"`Transport` is a seam"*). So
  `connect::connect` walks, probes, and hands back a transport that has already
  proved itself; `connect::open_session` is walk-then-`open`.

  `RealtimeProvider::url()` still answers — with the **first** candidate — so a
  caller that goes through `RealtimeSession::connect` gets the endpoint the walk
  would have dialled first, rather than an error.

- **The schema repair and the conflict filter live *below* the session machine.**
  Both are decisions about a frame, and by the time a frame reaches
  `via-realtime` the decision has to have been made: an `error` arriving before
  the session is ready is what rejects `connect()`
  (`session/state.rs`, `if kind == "error" && !self.ready`). A GA-discriminator
  rejection that reached the session machine would kill the session the repair
  exists to fix. `connect::InboundFilter` is that layer.

- **`ARGO`'s two-constructor `Dialect` becomes a settings field.** ARGO has
  `OpenAiRealtimeProvider::new` and `::azure` because its `ProviderConfig` already
  carried a provider *label*. VIA has one provider key, `openai`, so the choice is
  `OpenAiSettings::dialect`, defaulted from the host by `OpenAiDialect::for_host`.
  That is the same rule stated in the same direction — bring-up §4's *"dialect
  follows the host, not the provider label"* — with `api.openai.com` (and an
  unconfigured endpoint) getting one candidate and every other host getting the
  four-candidate walk, because every other host is either a first-party Azure
  resource or a gateway and both need it.

- **`RealtimeProtocol` is renamed `RealtimeSchema`.** ARGO's name collides with
  `via_realtime::RealtimeProtocol`, the *wire dialect* trait, and the two are
  genuinely different questions: the dialect decides the envelope, the event names
  and the id namespaces; the schema decides the shape of the one object inside
  `session.update`. The `Debug` spelling of each variant is unchanged, so ARGO's
  `[live] schema=Ga` marker still reads the same.


### Corrected, not copied

- **The upgrade is bounded.** ARGO's walk calls `connect_async` with no timeout of
  its own, so a peer that accepts the TCP connection and never answers the upgrade
  outlives the whole 12 s budget — and bring-up §5's promise, *"Connect is
  bounded. Four candidate URLs × a 3 s first-frame probe, capped at a 12 s total
  budget"*, is not one. The upgrade now gets the same window the probe does: an
  upgrade is a round trip and so is `session.created`.
  `tests/gateway.rs::a_hung_upgrade_is_bounded_and_the_budget_stops_the_walk`
  holds it, against a listener that accepts and never speaks.

- **A second GA-discriminator rejection is swallowed rather than surfaced.** ARGO
  resends the pre-GA payload **once, not in a loop**, and that rule is kept
  exactly — `InboundVerdict::RepairSchema` is returned at most once per socket.
  What is new is `InboundVerdict::DropRepaired`, and VIA needs it because VIA
  sends two GA payloads where ARGO sends one: the walk's probe payload, and the
  session machine's own `session.update` on `session.created`. Both are in flight
  before either can bounce, so the second rejection is a knock-on from a payload
  that was already on the wire — and surfacing it would kill the session the
  repair just fixed. It resends nothing.

- **The walk's budget is derived from the caller's deadline.** `ConnectBudget`
  exists because aborting the walk from outside drops the rejection list, which is
  the only record of how far it got. `ConnectBudget::within` returns ARGO's two
  constants unchanged whenever the caller's window has room for them (which the
  25 s `CONNECT_TIMEOUT` does), and scales both timers when it does not.


### Adapted, with the reason

- **`transcribe_input` defaults to `true`; ARGO's default is `false`.** ARGO's
  reason for `false` is real — it is an extra billed model, and a session that
  never renders the user's words must not pay for them. VIA's reason for `true` is
  stronger: `docs/architecture.md` §2 makes `dictation` a session mode whose
  *entire output* is the user's transcript, and Layer 1 publishes
  `conversation.item.input_audio_transcription.*` to every client. With the flag
  off, a `dictation` session over this provider produces nothing at all — which
  presents as a broken microphone rather than as a configuration. The knob is
  kept.

- **`ws://` is reachable.** ARGO's URL builder hard-codes `wss://`, because its
  endpoint always arrived from a mobile app's HTTPS configuration. VIA's own
  configuration surface already ships a plaintext loopback default for another
  provider (`ws://127.0.0.1:8765/v1/realtime`), and a litellm gateway on loopback
  is an ordinary deployment; forcing TLS there would make a working configuration
  unreachable. It is opt-in and explicit — only an endpoint written as `ws://` or
  `http://` gets it, and a bare host is still `wss`.

- **`response_metadata_correlation` stays at the baseline `false`.** GA echoes
  response metadata and pre-GA does not, and `via-realtime` reads the capability
  declaration **once**, at construction — before the endpoint has said which
  generation it speaks. A wrong `true` mis-correlates silently; FIFO correlation
  is merely less precise. `single_response_slot` and `per_response_instructions`
  are declared, and `tests/contracts.rs` asserts that those are the only two flags
  that differ from the catalogued `DEFAULT_CAPABILITIES`.

- **`OpenAiSettings::from_frontend` substitutes OpenAI's defaults for DashScope's.**
  `RealtimeFrontend` is provider-agnostic but its field *names* are DashScope's:
  `via_core::config::resolve_realtime_frontend` reads **one** environment chain
  for all five providers (`VIA_REALTIME_BASE_URL` / `VIA_REALTIME_URL` /
  `VIA_REALTIME_MODEL`) and records DashScope's defaults in
  `dashscope_realtime_url` / `dashscope_model`. So a value still equal to the
  DashScope default means *"the operator set nothing"*, and this substitutes
  OpenAI's for it. The alternative — `openai_*` fields on `RealtimeFrontend` — was
  not taken because `via-core` is shipped and this crate must not edit it. The one
  configuration that mis-reads is an operator deliberately pointing `openai` at
  `dashscope.aliyuncs.com`, which is not a configuration anyone has.

- **There is no environment variable for the Azure `api-version` or the dialect.**
  `via-core` owns every `VIA_*` name and is shipped; adding two would mean editing
  it. Both are `OpenAiSettings` fields with the catalogued defaults, and the
  dialect is derived from the host, so a first-party Azure resource and a gateway
  both work with nothing but `VIA_REALTIME_BASE_URL` set. A settings surface for
  the `api-version` is a follow-up, exactly as ARGO's bring-up doc records for its
  own per-provider realtime model setting.

- **The retired `gpt-4o-realtime-preview*` family is documented, not enforced.**
  Bring-up §5 is explicit that the deployment id *"must be a `gpt-realtime*` id"*.
  A name check would refuse the primary deployment shape this provider exists to
  reach: **on Azure the deployment name is arbitrary**, so a resource whose
  realtime deployment is called `voice-prod` is the ordinary case. The constraint
  is recorded on `DEFAULT_OPENAI_REALTIME_MODEL` instead.

- **The quota sentence upstream classifies as `other` is `fatal` here.**
  `realtime provider fatal error classification` locks
  `'You exceeded your current quota, please check your plan.'` to `other` — and it
  is right to, because that is **OpenAI's** wording, which DashScope never sends,
  so for DashScope's corpus it is an unrecognised message. For this provider it is
  the endpoint's own billing message and no amount of reconnecting will fix it.
  The reverse also holds: DashScope's two quota sentences are in this corpus,
  because a gateway relays its upstream's prose verbatim and litellm is a
  multi-upstream proxy. `tests/contracts.rs` asserts both directions against the
  parsed contract.

- **The `session.update` payload honours `configured`; ARGO's does not.** ARGO
  holds one payload per generation and resends it whole. VIA follows upstream
  qwen's catalogued rule (`DashScope session.update payload (subsequent,
  configured)`) — modalities, voice and transcription are negotiated once — and for
  this endpoint there is a second reason: **OpenAI refuses a voice change once
  audio has been generated**, so re-sending the audio block on a context refresh is
  the one thing that turns a harmless refresh into a session error. The GA
  discriminator is the exception and rides on every update, because it is what
  says which kind of session this is. The repair payload is built with
  `configured: false`, so it stays a full, byte-identical resend.

- **Tools are flattened in both generations.** `tool schema shape per dialect`
  catalogues the beta dialect as nesting a tool under `function`. That nesting is
  **DashScope's**, not the pre-GA OpenAI Realtime API's, and ARGO's payload builder
  calls `tools` *"the one part of the payload that did not move at GA"* and emits
  the flat shape to both. A tool that is not in the nested shape is passed through
  unchanged rather than dropped, because an empty `{}` on the wire is a schema
  error the service reports while a silently discarded tool is a capability that
  disappears with no diagnostic at all.

- **The input sample rate is 24 000, not the catalogued 16 000.** `audio sample
  rates` records 16 000 in / 24 000 out for *"dashscope and s2s"*; this provider is
  neither. The OpenAI Realtime input buffer is 24 kHz mono PCM16 in both
  generations and GA accepts no other PCM rate, so declaring 16 000 would tell
  every client to capture at a rate the endpoint refuses. The contract's own
  rationale — *"Wire-visible: … the voice.ready handshake all key off
  inputSampleRate"* — is exactly why publishing the right value is safe.

- **The three model-visible sentences are reached by key, not imported.** They come
  from the **qwen** side of the port (`frontend-tools.mjs`); ARGO's realtime
  provider has no injections at all. In VIA that file is `via-voice`'s, and
  `via-voice` sits above the provider crates, so importing it would invert the
  crate graph. `via-voice/src/tools/instructions.rs` and
  `via-realtime-dashscope/src/prompt.rs` already reach the same three `via-i18n`
  keys directly; `crate::prompt` is the third accessor over one definition.


### Added, which is not a port

- **`ConnectBudget`** — see *Corrected, not copied*. Also the reason both timers
  are testable in milliseconds rather than in twelve seconds of suite time.

- **`CreateAccounting`** — ARGO's `creates_in_flight` is a bare `AtomicUsize`
  shared between the sink and the inbound task. Naming it makes the saturating
  decrement and the slot-consuming `conflict_is_ours` a testable unit rather than
  two `fetch_update` call sites that have to agree.

- **`ProbeVerdict` / `InboundVerdict`** — ARGO's probe and inbound filter are
  inline `match` arms inside a 470-line `open()`. Splitting the *decision* from the
  I/O is what lets "a Ping does not satisfy the probe" and "a conflict with an
  empty ledger is not ours" be asserted without a socket.

- **`OpenAiDialect::parse`** accepts `azure-openai` and `azure_openai` alongside
  `azure`, because `azure_openai` is the string ARGO's app used as its provider
  label and an operator migrating a working configuration should not have to
  discover a third spelling.


### Testing

The suite is built around a **scripted fake WebSocket peer** (`tests/common/gateway.rs`,
a `TcpListener` on port 0) that can be told to misbehave in each of the four
documented ways, because every one of them is a *server* behaviour and none can
be reproduced by a provider double.

| Bring-up property | Where it is held |
| --- | --- |
| frames the session as Binary, not Text | `a_gateway_that_frames_the_session_as_binary_is_understood` |
| requires `Sec-WebSocket-Protocol: realtime` | `the_upgrade_offers_the_realtime_subprotocol_and_the_credential` |
| speaks pre-GA on GA's own route | `a_pre_ga_endpoint_on_gas_own_route_is_repaired_in_session` |
| accepts a socket on a route it does not bridge | `a_socket_that_only_pings_is_rejected_and_the_walk_moves_on` |
| §8b duplicate `response.create` | `the_relays_own_active_response_conflict_never_reaches_the_session` |

`tests/transport.rs` drives the same wrapper over in-memory channels, because
every one of its decisions is about a frame and none of them is about TCP.
`tests/contracts.rs` parses every catalogued value out of
`docs/reference/contracts.json` rather than retyping it, including the three
deliberate divergences above.

**Mutation-checked by hand** — `cargo-mutants` is not installed in this
environment, so 33 mutations were applied to the crate's decision points one at a
time and each was confirmed to fail the suite. All 33 were killed. A sample:
invert the route classification; never answer from the endpoint memo; key the
memo on the whole URL; drop the Binary arm of `protocol_text`; let a Ping satisfy
the probe; restart the probe deadline on every frame; commit to a 101 the probe
rejected; leave the upgrade unbounded; never check the connect budget; loop the
schema repair; never repair at all; drop every active-response conflict; drop
none; unsaturate the create ledger; match the cancel race on either half; remove
the subprotocol header; drop the gateway's bearer; omit the GA discriminator;
renegotiate everything on a subsequent update; ignore the schema when picking the
wire adapter; stop flattening tools; over-encode query values; carry DashScope's
defaults through; force TLS on a plaintext endpoint.

**Two of those mutants exposed a test defect before they exposed a code one.**
An unbounded probe deadline and an unbounded upgrade do not make the walk *slow*,
they make it **never end** — so the tests that hold those properties would have
hung the suite rather than failed it, and a hang is not a test result. Both are
now wrapped in a `tokio::time::timeout` of their own, with the reason written at
the call site.
