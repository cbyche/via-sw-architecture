# Phase 5 deviations — `via-voice`

Layer 1 — the voice session layer: the nine frontend tools, the tool-call
handler, **the Injection Gate**, turn correlation, the response guards, input
arbitration, the sleep controller and `SessionMode` dispatch.

Read [`phase-0.md`](phase-0.md) for what a deviation record is and is not.

*336 tests · clippy clean · `brand_leak.py` clean · 31/31 mutations killed · 14 deviations*

Ported from `server/src/voice/{realtime-gateway, frontend-tools,
response-context, response-lifecycle, turn-correlation, active-voice-clients,
input-arbitration, input-asset-registry, input-transcript, sleep-controller,
session-permission-policy}.mjs`,
`server/src/voice/tools/{tool-call-handler, turn-transcripts}.mjs`,
`server/src/voice/response-guards/{index, action-promise}.mjs`,
`server/src/voice/announcement/{announcement-manager, announcement-window}.mjs`
and `shared/input-parts.mjs`.

## The one thing this crate exists to get right

**The Injection Gate.** A delegation result must never land mid-sentence, and
"mid-sentence" is only decidable with playback position from the client.
`announcement/window.rs` is the predicate
(`user_speaking || turn_pending || audio_responses.nonEmpty`), `gate.rs` wraps it
with `sleeping || waking || !output_enabled`, `announcement/manager.rs` is the
delivery engine, and `via_audio::PlaybackCursor` is the drain predicate between
the `playback.started` and `playback.ended` receipts. Three properties are the
whole point and each has a test that fails without it:

- delivery is confirmed by a **playback** receipt, not by `response.done`
  (`delivery_is_confirmed_only_by_a_playback_receipt`);
- retries are **bounded**, and exhausting them releases the claim so the queue
  moves on (`retries_are_bounded_and_the_batch_is_abandoned`,
  `a_later_result_is_delivered_after_an_abandoned_one`);
- the acknowledgement timeout **re-arms** rather than retrying while the gate is
  blocked (`the_acknowledgement_timeout_re_arms_while_the_gate_is_blocked`).

---

## Structural

- **`via-realtime` owns the provider seam; this crate declares no second one.**
  `docs/architecture.md` §3 and §9 place `RealtimeProvider`, `RealtimeProtocol`,
  the registry and the session machine in `via-realtime`. `via-voice` declares
  only what it *consumes*: `provider::ProviderView` (key, label, the two sample
  rates, the five capability flags, the error classification) and
  `frontend::VoiceFrontend` (the eleven calls the gateway and the announcement
  manager make). `via-app` implements both over `via_realtime`'s session. The
  alternative — importing the provider trait directly — would make every
  gateway branch recompile on a transport change and would make this crate
  untestable without a socket.

  `provider::ProviderCapabilities` does restate `DEFAULT_CAPABILITIES`, because
  two of its flags (`per_response_instructions`, `single_response_slot`) are
  branched on *here* and the struct is five `bool`s. When `via-realtime`'s
  equivalent is stable this becomes a `From` impl, not a rewrite.

- **Layer 3 facts arrive as traits.** `docs/architecture.md` §9's adjacency
  table gives `voice → conversation, core, shared, task, voice` — no `agent`.
  Three things upstream reads directly from Layer 3 are therefore ports here,
  implemented in `via-app`: `tools::BackendAvailability` (over
  `via_backends::BackendAvailability`), `tools::PermissionResponder` (over
  `via_coordinator::PermissionBroker`) and `tools::DelegationRunners` (over
  `via_coordinator::Coordinator`). `via-arch-test`'s
  `layer1_does_not_reach_layer3_directly` is what this is answering to.

- **`RealtimeFrontend` is not ported here.** It is the session machine, and
  §3's table puts it in `via-realtime`. What this crate ports is everything
  *above* a normalized event.

- **The gateway's socket plumbing is `via-app`'s.** `attachRealtimeGateway`'s
  WebSocket server, the `send()` helper and the frame codec are transport;
  `session.rs` carries the decisions that plumbing makes —
  `upgrade_decision`, the seven timing constants, `TurnTracker` — so they are
  testable without a socket and cannot drift when the transport changes.

## Behavioural

- **`schedule_reminder.execute_at` accepts RFC 3339 and two naive forms, not
  everything `Date.parse` does.** Upstream calls `Date.parse`, which also
  accepts `Date.prototype.toString()` output, `"Dec 25, 2026"` and a long tail
  of implementation-defined formats. The tool's own schema says *ISO 8601*, the
  model is told to compute the value from `get_current_time`'s output (which is
  RFC 3339), and widening the parser would only accept values the model is
  instructed not to produce. A value outside the three accepted forms is
  `invalid_time`, which is the same answer upstream gives for anything
  `Date.parse` returns `NaN` for.

- **`backend_unavailable (disconnected)` reuses the not-configured tail.**
  Upstream's disconnected instructions are three sentences, the second and third
  identical to the not-configured variant's. The shipped `via-i18n` catalog
  keeps those two sentences only on `voice.instructions.backend_not_configured`,
  so `instructions::backend_disconnected_instructions` composes
  `head + " " + tail` from the two keys rather than duplicating the text into a
  third. Editing a shipped crate's catalog to un-share them would change what
  every other consumer of that key renders.

  The same catalog joins those sentences with a space where upstream joins with
  `\n`. That choice is `via-i18n`'s and is not re-litigated here.

- **`accepted_instructions` is composed from two keys.** Upstream's four
  sentences are: two shared with the duplicate case, then two specific ones. The
  catalog stores the shared pair inside
  `voice.instructions.duplicate_submission`, so the accepted variant slices them
  back out. `the_accepted_instruction_ends_with_the_not_finished_warning` pins
  the result.

- **`InputArbitration::status()` sorts holders by `(since, owner)`.** Upstream
  iterates a JavaScript `Map`, which is insertion-ordered; the actor holds a
  `HashMap`, which is not. The only order-sensitive readers are `owner` and
  `reason` — a convenience for a host UI that only ever holds one suspension —
  and `/api/health.inputSuspension`, which is diffed across restarts. A stable
  sort is strictly better there than hash order; it is not upstream's order when
  two holders share a `since`, and nothing observable depends on which of two
  simultaneous holders is named.

- **`SessionPermissionPolicy`'s key is not injective for arbitrary input.**
  `${ownerId}\0${sessionId || 'main'}` is upstream's, and it collides for
  ids containing NUL. Neither an owner id (from `via_core::Identity`) nor a
  session id (a URL query parameter) can carry one, and neither reaches the
  function from a model. Recorded rather than "fixed" because changing the
  encoding would change a stored key shape for no reachable gain; the doc
  comment on `key` says so, and
  `ordinary_owner_and_session_ids_never_collide` pins what is actually true.

  *Made legible in the conformance catch-up:* the separator above was written
  as a raw `U+0000`, both here and in `docs/reference/contracts.json`'s own
  `exactValue`, where it renders as nothing at all and reads as a space. It is
  a NUL, the shipped `key` uses `\u{0}`, and the two agree — but a reader who
  took it for a space would conclude this crate diverges, which is the opposite
  of true. Spelled `\0` here.
  `via-conformance`'s `tests/voice_constants.rs::the_session_permission_policy_bounds_are_the_shipped_constants`
  now reads the separator out of the catalogued text and demonstrates the
  collision through the public API, so neither the value nor this note can
  drift. The same raw NUL sits inside `input asset registry limits`'
  `sha256(mime + '\0' + url)`.

- **A UTF-16 bound stops before a surrogate pair rather than splitting one.**
  `text::bounded_utf16` reproduces JavaScript's `.slice(0, n)` in units but
  cannot reproduce its output when `n` lands mid-pair: JavaScript yields an
  unpaired surrogate, which `String` cannot hold. The clip is one unit shorter
  in exactly that case. `the_two_bounds_disagree_exactly_where_they_should` and
  `a_utf16_bound_never_splits_a_surrogate_pair` record it.

- **The four literal regexes are `Lazy<Option<Regex>>`, not `expect`.** The
  workspace forbids `unwrap`/`expect` outside tests, and
  `via_conversation::tool::is_sensitive` sets the idiom: hold the compile
  result, and pick the *safe* direction when it is absent. The safe direction
  differs by call site — there, treat everything as sensitive and refuse the
  write; here, `promises_action` answers `false` (no correction) and
  `parse_data_url` answers `None` (refuse the attachment). Both fall closed;
  the patterns are literals and cannot in fact fail.

- **`clean` uses ECMAScript's whitespace set, not Rust's.** The two disagree in
  both directions: JavaScript's `\s` includes U+FEFF (zero-width no-break
  space) where Unicode's `White_Space` does not, and `White_Space` includes
  U+0085 (NEXT LINE) where `\s` does not. Handled explicitly so a BOM pasted
  into an objective collapses the way upstream collapses it.

  *Corrected in the conformance catch-up:* this crate originally spelled the
  predicate `char::is_whitespace() || U+FEFF`, which got the first direction
  right and the second wrong — `clean` collapsed a U+0085 that upstream leaves
  alone. `text::is_js_whitespace` is now a re-export of
  `via_core::text::is_js_whitespace`, which enumerates the specified set;
  `docs/deviations/phase-4.md` carries the consolidation, which also removed
  the duplicate definitions in `via-downstream`, `via-conversation` and
  `via-core::env`.

- **`with_attachment_anchors` writes `-1` when an anchor cannot be located.**
  Upstream's `indexOf` returns `-1` and writes it through into
  `source.text.start`. Reproduced rather than "corrected" to `None`, because the
  offsets are OpenCode-compatible and a client reading them expects the
  sentinel.

- **A malformed instant drops only its `completed_at` line.** Upstream's
  `new Date(ms).toISOString()` throws on an unrepresentable value, which would
  take the whole announcement with it. `format::Announcement::block` omits the
  line and keeps the result, and
  `an_unrepresentable_completion_instant_drops_only_its_line` pins it.

## New to VIA

- **`SessionMode` dispatch** (`mode.rs`). Neither upstream has the concept.
  `ModePlan` resolves the requested mode plus one fact — is a harness
  configured? — into four decisions, and reports **both** degradations rather
  than applying them silently:

  | requested | harness | effective | reason |
  | --- | --- | --- | --- |
  | `agent` | yes | `agent` | — |
  | `agent` | no | `direct` | `no_harness` |
  | `direct` | either | `direct` | — |
  | `dictation` | either | `dictation` | — |
  | `interface` | either | `direct` | `context_engine_unavailable` |

  `interface` is **accepted** and behaves as `direct` until `via-context` lands
  in phase 7; `status::aggregate` puts the reason in
  `/api/health.voiceClients.degradedModes` so a client is told rather than
  guessing. **Superseded in phase 7** — `via-context` landed, the `interface`
  row now reads `interface` / no reason, and `ContextEngineUnavailable` became
  conditional on `ModePlan::with_context_engine`. See
  [`phase-7.md`](phase-7.md) §11. `dictation` declares no tools at all and never speaks, which is what
  `docs/architecture.md` §2's *"no model turn, no tools, no speech out"*
  requires of the tool catalog and of `ToolCallHandler::handle`.

- **`gate::InjectionGate`** names what upstream spells out at four call sites,
  and adds `via_audio::PlaybackCursor` as a second opinion: a `playback.ended`
  lost to a dropped socket leaves the window clean, and the cursor keeps the
  gate honest for the audio's own remaining duration.

- **`status::DegradedMode`** — see above.

## Assets

`assets/frontend-agent/{en,zh,ko}/{PROMPT,ASSISTANT}.md`.

- `zh` is upstream's `config/frontend-agent/` byte for byte, with the single
  rebrand `docs/rebrand.md` mandates: `你叫千问Audio。` → `你叫 VIA。` (the space
  before `VIA` is the entry's own instruction).
- `en` and `ko` are **authored peers**, not machine translations
  (`docs/architecture.md` §16). Structural parity is asserted, not assumed:
  `the_three_locales_have_structural_parity` pins the seven top-level headings,
  the three `ASSISTANT.md` sub-headings, and the twenty-one tool names, tag
  names and field names each prompt must refer to by name.
- `the_packaged_documents_fit_their_bounds` pins that none of the six exceeds
  `via_conversation`'s `MAX_PROMPT_CHARS` / `MAX_ASSISTANT_CHARS`, so nothing is
  silently clipped at assembly.

## What the mutation pass changed

Thirty-one mutants, each a plausible edit to a predicate, a bound or a
lifecycle step. Two survived the first pass against real code — both were
**defects in this crate**, not gaps in the mutants:

- **A retry cancelled the lease heartbeat.** The four announcement timers shared
  one generation counter, so cancelling the acknowledgement timer on a retry
  cancelled the claim renewal with it, and a batch that kept retrying let its
  notification claim expire — which is precisely how two live frontends end up
  presenting the same result. Fixed by giving each timer kind its own epoch
  (`Timer::{DELIVERY, RETRY, ACKNOWLEDGEMENT, LEASE}`); pinned by
  `a_retrying_batch_keeps_renewing_its_claim`.

- **Retiring a batch stranded a queued one.** `finish_acknowledged_batch` and
  `abandon_now` bumped the generation the delivery epilogue checks, and the
  epilogue is the only thing that starts the next delivery once `delivering`
  goes false. A result queued while a delivery was in flight then sat there
  until an unrelated `flush`. Upstream bumps that counter only in `pause()`,
  whose own `pending.clear()` makes the early return safe; restored, and pinned
  by `a_result_queued_during_a_delivery_is_not_stranded_by_the_confirmation`.

A third finding was a test weakness rather than a defect: two mutants made
`InputArbitration`'s edge assertions **hang** instead of fail, because a bare
`broadcast::Receiver::recv().await` waits forever for an edge a broken
implementation never sends. Every such receive now goes through a bounded
`next_edge` helper — under `start_paused` the bound costs no wall time, and a
missing edge is a failure rather than a stall.

## Concurrency

Three owning tasks, per `docs/architecture.md` §11 — bounded `mpsc`, `oneshot`
replies, shutdown by dropping the sender:

| Owner | Why order matters |
| --- | --- |
| `arbitration::InputArbitration` | a `suspend` overtaking its own `resume` leaves the microphone dead |
| `tools::TurnTranscripts` | a waiter registered after the record it waits for would hang for the full timeout |
| `announcement::AnnouncementManager` | the batch, the retry ladder and the claim are one state machine |

**A delivery attempt runs outside the announcement actor's loop.** Awaiting the
frontend inside the loop would hold the actor for as long as the provider takes
to settle a response — up to the response-start timeout — and during that window
a `playback.started` receipt could not confirm delivery, a barge-in could not
dismiss the batch, and `pause` could not release the claim on sleep. The attempt
is spawned and its outcome returns as a `Delivered` command, which is what makes
`delivering` mean anything and what the stranding test above actually exercises.
Upstream gets this for free from the JavaScript event loop.

`SleepController` and `InputAssetRegistry` hold a `std::sync::Mutex` instead:
neither has an ordering invariant, both are read far more than written, and a
task for either would put a channel hop inside the sleep timer's own callback.
A poisoned lock is recovered rather than propagated — the worst case is a
controller that sleeps a beat late, and panicking the socket task instead would
drop a live voice session.
