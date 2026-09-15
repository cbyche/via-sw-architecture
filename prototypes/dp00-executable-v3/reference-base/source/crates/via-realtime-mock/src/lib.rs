//! A realtime provider that is a script, not a service.
//!
//! **New to VIA.** `qwen-audio-agent` has no equivalent, and the cost of not
//! having one is visible in its own suite: most of its realtime tests need
//! either a live DashScope credential or an ad-hoc fake socket rebuilt at each
//! call site. `docs/architecture.md` §7 names what this replaces —
//! *"`mock` · new · Deterministic replay. Makes the whole gateway testable with
//! no weights, no GPU, no network."*
//!
//! It is what makes phase 5's milestone real: routing, `spawn_thinking`, the
//! Work queue, delegation, the announcement window and `via chat` are all
//! exercised end to end with **no weights, no GPU, no network and no audio
//! hardware**.
//!
//! # What it is
//!
//! One [`RealtimeProvider`](via_realtime::RealtimeProvider) over one
//! [`RealtimeProtocol`](via_realtime::RealtimeProtocol), reached through an
//! in-process channel instead of a socket. The session above it is the *real*
//! [`RealtimeSession`](via_realtime::RealtimeSession) — the same two owning
//! tasks, the same correlation, the same two watchdogs, the same busy-retry
//! ladder — because `via-realtime` made `Transport` a seam for exactly this
//! (`docs/deviations/phase-5-via-realtime.md`).
//!
//! ```
//! use via_realtime::ResponseContext;
//! use via_realtime_mock::{MockRealtime, Script, collect_events, script::turns};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # tokio::runtime::Runtime::new()?.block_on(async {
//! let (session, events, handle) = MockRealtime::new(
//!     Script::conversation().turn(turns::say("On it.")),
//! )
//! .open()
//! .await?
//! .into_parts();
//! let log = collect_events(events);
//!
//! let outcome = session.send_user_text("build the thing", ResponseContext::new(), None).await?;
//! assert!(outcome.is_some_and(|outcome| outcome.is_completed()));
//!
//! // The outcome settled; wait for the log to catch up before reading it.
//! assert!(log.wait_for_turns(1).await);
//! assert_eq!(log.spoken_text(), "On it.");
//! assert_eq!(handle.transcript().await?.count_of("response.create"), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # })
//! # }
//! ```
//!
//! # The four rules
//!
//! 1. **Steps are scanned in declaration order.** The first that matches the
//!    frame and is not exhausted fires, and only that one.
//! 2. **A step fires [`Repeat::Once`] unless it says otherwise**, so a list of
//!    [`Trigger::ResponseCreate`] steps is a list of consecutive model turns.
//! 3. **When nothing matches, the mock behaves like a provider** — it
//!    acknowledges `session.update`, echoes `conversation.item.create`, and
//!    answers `response.create` with `response.created` + `response.done`. An
//!    empty script is a working provider.
//! 4. **Ids are stamped, not written.** Response activity with no response id
//!    gets the id of the response its trigger belongs to; a
//!    `conversation.item.created` with no item id gets the one being
//!    acknowledged. [`Emission::verbatim`] opts out, which is how the
//!    adversarial cases are written.
//!
//! # Determinism
//!
//! No wall clock and no RNG anywhere in this crate. Every id the mock mints is a
//! counter — `resp_1`, `item_mock_1`, `event_mock_1` — and [`CannedAudio::tone`]
//! is integer arithmetic, so the same script produces the same bytes on every
//! run and on every host. Delays are [`tokio::time::sleep`], so under
//! `#[tokio::test(start_paused = true)]` they cost nothing and cooperate with
//! `tokio::time::advance`.
//!
//! There is exactly one thing in the loop the mock does not mint, and it is
//! worth naming: with the baseline `conversation_item_id_echo: true`, the id the
//! mock echoes on `conversation.item.created` is the **client's** — a v4 uuid
//! `RealtimeProtocol::conversation_item_id` minted. Script
//! `conversation_item_id_echo: false` and every id in the stream is the mock's
//! own, which is what `tests/determinism.rs` asserts byte for byte.
//!
//! # Capabilities exist to be exercised
//!
//! [`ProviderCapabilities`](via_realtime::ProviderCapabilities) is five flags
//! and each can be scripted the other way. Four of them change what the mock
//! *does*, not only what it says:
//!
//! | Flag | Scripted the other way |
//! | --- | --- |
//! | `acknowledges_session_update` | no `session.updated`; the session is ready on `session.created` |
//! | `single_response_slot` | a racing `response.create` is refused, driving the busy-retry ladder |
//! | `response_metadata_correlation` | the correlation id is echoed on `response.created` / `response.done` |
//! | `conversation_item_id_echo` | the client's item id is **replaced**, driving the single-waiter fallback |
//! | `per_response_instructions` | declared only — nothing in the session branches on it |
//!
//! [`MockProviderSpec::speech_to_speech_shaped`] is all four at once, on the GA
//! dialect: the capability set `providers/s2s.mjs` declares.
//!
//! # `dictation` is a first-class script kind
//!
//! `docs/architecture.md` §2 makes `dictation` the one mode that mounts no model
//! at all. [`Script::dictation`] is that end to end: the provider declares no
//! model and no voice, the session opens in
//! [`SessionMode::Dictation`](via_protocol::SessionMode::Dictation), every
//! response-creating call answers `skipped / no_model_turn` without touching the
//! transport, and transcripts are scripted on
//! [`Trigger::AudioCommit`].
//!
//! # Real audio
//!
//! [`CannedAudio`] streams base64 PCM16 built through [`via_audio`], so the
//! Injection Gate's drain predicate — [`via_audio::PlaybackCursor::is_draining`],
//! `docs/architecture.md` §3 — has real frame counts to work with rather than a
//! placeholder string.
//!
//! # What it does not restate
//!
//! [`via_catalog`] owns the `mock` provider row and the model profile;
//! [`via_realtime`] owns both dialects, the session and the error corpus;
//! [`via_i18n`] owns every sentence a person reads; [`via_audio`] owns the PCM16
//! conventions; [`via_protocol`] owns [`SessionMode`](via_protocol::SessionMode).
//!
//! Deviations are recorded in `docs/deviations/phase-5-via-realtime-mock.md`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod audio;
pub mod error;
pub mod provider;
pub mod record;
pub mod script;
pub mod server;
pub mod session;

pub use audio::{AudioChunk, CannedAudio, TONE_AMPLITUDE, TONE_HZ, decode_audio, encode_audio};
pub use error::MockError;
pub use provider::{
    ClassificationRule, MOCK_ENDPOINT, MOCK_MODEL, MOCK_PROVIDER_KEY, MockDialect, MockProvider,
    MockProviderSpec, PERMISSION_REQUEST_TAG, dialects,
};
pub use record::{EventLog, POLL_BUDGET, Recorded, collect_events};
pub use script::{
    CANCELLED, COMPLETED, Emission, Repeat, Script, ScriptKind, ScriptStep, Stamp, Trigger, events,
    messages, turns,
};
pub use server::{
    COMMAND_CAPACITY, EVENT_ID_PREFIX, FunctionOutput, ITEM_ID_PREFIX, ITEM_SCOPED_EVENTS,
    RESPONSE_ID_PREFIX, Transcript,
};
pub use session::{MockHandle, MockRealtime, MockSession};
