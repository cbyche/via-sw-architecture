# Phase 5 deviations — `via-realtime-mock`

Layer 1 — a deterministic realtime provider: a script instead of a service, so
the whole Gateway is testable with no weights, no GPU, no network and no audio
hardware.

Read [`phase-0.md`](phase-0.md) for what a deviation record is and is not.


## `via-realtime-mock`

*230 tests · clippy clean · 36/37 mutations killed (1 equivalent) · 15 deviations*

**There is no upstream to port.** `qwen-audio-agent` has no mock provider, and
the cost of not having one is visible in its own suite: most of its realtime
tests need either a live DashScope credential or an ad-hoc fake socket rebuilt at
each call site. `docs/architecture.md` §7 names what this replaces —
*"`mock` · new · Deterministic replay. Makes the whole gateway testable with no
weights, no GPU, no network."* — and §15 makes it part of the phase-5 milestone.

So the whole crate is "added, which is not a port". What follows records the
design decisions a reviewer would otherwise have to reconstruct, and the four
places where the mock's behaviour is deliberately *unlike* a naive fake.


### What it is built on rather than restating

| Owned by | What |
| --- | --- |
| `via-catalog` | the `mock` provider row (key, label, identity shape) and the `local` model profile |
| `via-realtime` | both dialects, the session, `RESPONSE_ACTIVITY_TYPES`, `realtime_response_id`, `Transport`, and `testing::default_test_classification` |
| `via-i18n` | the two sentences a person reads |
| `via-audio` | `SampleRate`, `WavAudio`, `ChannelCount`, the PCM16 codecs and `PlaybackCursor` |
| `via-protocol` | `SessionMode` |

The mock implements **no dialect of its own**: `MockDialect` chooses between
`openai_compatible_protocol()` and `ga_realtime_protocol()`. It classifies errors
through `via_realtime::testing::default_test_classification` rather than a second
copy of the two shipped corpora. It builds no PCM primitives; `CannedAudio` is
rate + channels + samples plus the two things a socket adds — base64 and block
chunking.


### Structural

- **`RealtimeSession::open` is the door, not `connect`.** `via-realtime` made
  `Transport` a seam for exactly this
  (`phase-5-via-realtime.md`, *"`Transport` is a seam"*), so the session above a
  mock is the **real** session: the same two owning tasks, the same correlation,
  the same two watchdogs, the same busy-retry ladder. Only the bytes' destination
  differs. `RealtimeProvider::url()` therefore names no socket
  (`mock://in-process`), and `RealtimeSession::connect` on a mock fails — which
  is the correct answer to "connect to the mock over TCP", not an omission.

- **The script server is an owning task, not a mutex over a transcript.**
  `docs/architecture.md` §11 asks for an owning task per ordering invariant. This
  one has three that move together — the step cursor, the id counters, the
  emission queue — and the transcript is read *while* the session under test is
  still writing to it. A mutex would hand a test a snapshot torn across a frame
  boundary, which is the exact class of flake a deterministic mock exists to
  remove. `MockHandle` asks over a bounded `mpsc` with `oneshot` replies.

- **One emission FIFO with cumulative deadlines, not N timers.** A delay is
  relative to the *previous emission in the queue*, so a list of `after: 20 ms`
  emissions is a 50 Hz stream and a later emission with a shorter delay can never
  overtake an earlier one. N independent timers would make the arrival order a
  function of the scheduler, which is the property this crate is for.
  `tests/determinism.rs::a_delayed_stream_arrives_in_the_order_it_was_written`.

- **A script is *rules*, not a linear tape.** A realtime service is a function
  from the frames a client writes to the events it sends back, and a bare
  sequence cannot express "answer the *second* `response.create` this way".
  `ScriptStep { on, repeat, emit }` is that function; `Script::turn` /
  `Script::turns` is the linear-tape sugar over it, and it is what almost every
  test actually writes.

- **When no step matches, the mock behaves like a provider.** It acknowledges
  `session.update`, echoes `conversation.item.create`, and answers
  `response.create` with `response.created` + `response.done`. So an empty script
  is a *working* provider rather than a mute one, and a script says only what is
  unusual about the run. The alternative — every test spelling out its own
  handshake — is how the ad-hoc fakes upstream grew.

- **`EventLog` is published, not `#[cfg(test)]`.** `via-realtime` documents one
  rule about its bounded event stream — *do not await a session method from the
  same task that drains it* — and every test that drives a session needs the same
  twenty-line drainer to obey it. `MockSession::with_event_log()` is that
  drainer, once. It is the one place in this crate that uses a mutex, and the
  module says why: the log holds no ordering invariant, because the channel
  already ordered it.

- **`Script` deserializes through a private wire struct** so that `provider` can
  be `Option` on the wire and never in the struct. A fixture that says
  `"kind": "dictation"` and nothing else must get a provider that *mounts no
  model*; `#[serde(default)]` on a plain field would hand it a conversing one.
  That is also why `kind` is a script kind rather than a `SessionMode`.


### Deliberately unlike a naive fake

- **Capabilities are behaviour, not a declaration.** `ProviderCapabilities`
  exists so the frontend never branches on a provider name, which means every
  flag is something a real provider *does*. Four of the five change what the
  script server does: it withholds `session.updated`, refuses a racing
  `response.create`, echoes the correlation id, and replaces the client's item
  id. `tests/capabilities.rs` asserts each from both sides. The fifth,
  `per_response_instructions`, has no session behaviour at all — and that is
  asserted too, because "declared only" is a claim that can go stale.

- **`response_metadata_correlation` is gated on the dialect as well as the
  flag.** The beta dialect has no portable correlation contract, so a beta
  provider that declared the flag would simply never correlate — the same
  constraint the real dialect has. A mock that echoed metadata the beta dialect
  cannot carry would make a test pass against a shape that never travels.

- **Ids are stamped, and `Stamp::Verbatim` opts out.** An emitted event that is
  response activity and carries no response id gets the trigger's; a
  `conversation.item.created` with no item id gets the one being acknowledged.
  Without the opt-out the adversarial cases are unscriptable — most of all a
  `response.created` with **no** `response.id`, which is the one place
  `via-realtime` had to correct upstream rather than copy it
  (`phase-5-via-realtime.md`, *"Corrected, not copied"*). An event that already
  names its response through any of `realtime_response_id`'s three paths is left
  alone, so it can never correlate two ways at once.

- **The one-slot refusal runs before the script is consulted, and consumes no
  step.** It is the service's own state rather than script content, and leaving
  the step unconsumed is what lets the session's busy-retry ladder replay *into*
  it. `server.rs::one_response_slot_refuses_a_racing_create_without_consuming_a_step`.


### Identity and localization

- **Two i18n keys are used and one was added.**
  `realtime.missing_configuration` already took a `{label}`, so the mock uses it
  unchanged. There was no *generic* connect-timeout key — the catalogue carried
  only `realtime.connect_timeout_dashscope` and
  `realtime.connect_timeout_speech_to_speech`, both of which name someone else's
  product — so **`realtime.connect_timeout`** was added with a `{label}`
  placeholder, modelled on the DashScope one. Its `zh` value carries Han
  characters, so it needs no `NOT_CHINESE_PROSE` entry. `local-omni` and `openai`
  will want it too.

- **`script::messages` is simulated third-party text and does **not** go through
  `via-i18n`.** The six phrases are what the two shipped providers'
  `classifyError` corpora match on (`providers/dashscope.mjs:16-29`,
  `providers/s2s.mjs:14-23`), reproduced so a mock refusal is classified the same
  way a real one is. Translating them would stop them classifying — the same
  reason `RealtimeError::Transport` carries the transport's own text.
  `tests/contracts.rs::the_classification_vocabulary_is_closed_and_every_member_is_reachable`
  pins the pairing: each phrase reaches exactly one catalogued `ErrorClass`.

- **`MockError` has no `message(locale)`.** Every other VIA error type splits a
  developer sentence from a person's sentence because both audiences exist. Here
  only the first does: a `MockError` means a *fixture* is broken, and a fixture
  has no user. The module says so in as many words.


### The lag a consumer has to know about

`EventLog` drains the session's bounded event stream on **its own task**, because
`via-realtime` forbids the task that awaits session methods from also draining
them. That has a consequence worth spelling out, because it is invisible until it
flakes: **a settled outcome is not a drained log.** A session method resolves
when its response outcome settles, and the session settles it inside its own
state task — before the drainer has necessarily forwarded the events that led to
it.

Everything up to and including that turn's `response.done` is already *queued* on
the event channel at that moment, so it is only a question of the drainer
catching up, and the channel is FIFO. `EventLog::wait_for_turns(n)` is therefore
the one line to write between an awaited call and an assertion about what the
model said — seeing the `response.done` proves every delta before it is in the
log too.

This is recorded rather than merely fixed because it was found the honest way: a
doc example asserted `spoken_text()` straight after `send_user_text().await` and
failed on roughly one run in twenty. Every call site in the crate now waits, the
`record` module docs lead with it, and `spoken_text` says so at the point of use.
A turn the provider *refused* never reaches `response.done`, so those wait on
`wait_for_kind("error")` instead.

### Determinism, and its one named boundary

No wall clock and no RNG anywhere in the crate. Every id the mock mints is a
counter (`resp_1`, `item_mock_1`, `event_mock_1`), and `CannedAudio::tone` is
integer arithmetic — a sine would be one line shorter and would make the fixture
depend on the platform's libm in its last ulp.

There is exactly one value in the loop the mock does not mint, and it is named
rather than hidden: with the baseline `conversation_item_id_echo: true`, the id
echoed on `conversation.item.created` is the **client's**, a v4 uuid
`RealtimeProtocol::conversation_item_id` minted. Script
`conversation_item_id_echo: false` and the whole inbound stream is the mock's
own, which `tests/determinism.rs` asserts byte for byte;
`session.rs::the_only_thing_that_differs_between_runs_is_an_id_the_client_minted`
pins the boundary itself.


### The one mutant that survives

**`tone`: `phase < half` → `phase <= half`.** Equivalent, not untested: at
`phase == half`, `period - phase` is `half` as well, so both arms yield the same
sample. The comparison is commented at the point it matters.


### Gates

`scripts/risky_unwrap.py`, `panic_filter.py`, `unreachable_filter.py` and
`brand_leak.py` are all clean and unchanged — this crate adds no subsystem
directory and no `unwrap()`, `expect()`, `panic!` or `unreachable!` outside
tests.


### Root manifest

Unchanged. Every dependency this crate takes was already in the workspace table.
