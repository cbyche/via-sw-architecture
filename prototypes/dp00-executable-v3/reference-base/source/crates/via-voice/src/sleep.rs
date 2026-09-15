//! The sleep controller — when a voice session may drop to wake-word-only.
//!
//! Ported from `server/src/voice/sleep-controller.mjs`.
//!
//! Sleeping closes the realtime socket and stops paying for an open session
//! nobody is talking into. It is safe **only** when nothing is in flight, which
//! is what the `can_sleep` predicate decides. The controller's own job is
//! narrower: hold the inactivity timer, re-arm it on activity, and retry
//! politely when the predicate says no.
//!
//! # Two properties worth naming
//!
//! **A refused sleep retries; it does not cancel.** If `can_sleep` says no the
//! timer is re-armed at [`SleepControllerConfig::retry_ms`] rather than dropped,
//! so a session that was mid-announcement at the deadline still sleeps a second
//! later instead of staying awake until the next utterance.
//!
//! **A failed `on_sleep` rolls back.** The controller marks itself sleeping
//! *before* awaiting the transition, and un-marks it if the transition throws.
//! Otherwise a failed sleep leaves a session that believes it is asleep, will
//! not answer, and has no timer that would ever wake it.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use via_audio::{ChannelCount, SampleRate};
use via_core::Config;
use via_i18n::{Locale, format as i18n_format, keys};
use via_wake_word::{
    DisabledReason, Keyword, KeywordSet, Resolution, WakeWordDetector, WakeWordError,
    WakeWordSettings, WakeWordStream,
};

/// How soon a refused sleep is retried, in milliseconds.
///
/// **External contract** — `sleep-controller.mjs:4` (`retryMs = 1000`).
pub const DEFAULT_RETRY_MS: u64 = 1_000;

/// The floor on the retry interval.
///
/// **External contract** — `sleep-controller.mjs:9`
/// (`Math.max(10, Number(retryMs) || 1000)`). A zero here would spin.
pub const MIN_RETRY_MS: u64 = 10;

/// Answers "may this session sleep right now?".
pub type CanSleep = Arc<dyn Fn() -> bool + Send + Sync>;

/// Performs the transition. Returning `false` means it failed.
pub type OnSleep = Arc<dyn Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>;

/// The controller's configuration.
#[derive(Clone)]
pub struct SleepControllerConfig {
    /// How long a session may be idle before it sleeps. `0` disables the
    /// automatic timer entirely, which is what a desktop client that owns its
    /// own inactivity policy asks for.
    pub timeout_ms: u64,
    /// How soon a refused sleep is retried.
    pub retry_ms: u64,
    /// The predicate.
    pub can_sleep: CanSleep,
    /// The transition.
    pub on_sleep: OnSleep,
}

impl std::fmt::Debug for SleepControllerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SleepControllerConfig")
            .field("timeout_ms", &self.timeout_ms)
            .field("retry_ms", &self.retry_ms)
            .finish_non_exhaustive()
    }
}

impl SleepControllerConfig {
    /// A configuration with the contract retry interval.
    #[must_use]
    pub fn new(timeout_ms: u64, can_sleep: CanSleep, on_sleep: OnSleep) -> Self {
        Self {
            timeout_ms,
            retry_ms: DEFAULT_RETRY_MS,
            can_sleep,
            on_sleep,
        }
    }

    /// Override the retry interval, floored at [`MIN_RETRY_MS`].
    #[must_use]
    pub fn retry_ms(mut self, retry_ms: u64) -> Self {
        self.retry_ms = retry_ms.max(MIN_RETRY_MS);
        self
    }
}

/// The inputs to the gateway's `can_sleep` predicate.
///
/// **External contract** — `realtime-gateway.mjs:1852-1864`. Eight conditions,
/// and every one of them is a way a session can look idle without being idle:
///
/// - `input_enabled || wake_word_enabled` — a session that can neither hear the
///   user nor a wake word has no way back, so it must not sleep;
/// - `is_active_client` — a client that lost the voice slot is not the one that
///   gets to close the session;
/// - `frontend_ready` — there is nothing to close;
/// - `!user_speaking` and `!window_blocked` — the Injection Gate's own
///   predicate, so sleeping cannot cut a sentence in half;
/// - `!connecting` — a handshake is in flight and closing it mid-flight leaves
///   the reconnect backoff confused;
/// - `!waking` — the session is already coming back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CanSleepInputs {
    /// This client may capture.
    pub input_enabled: bool,
    /// Wake-word detection is configured.
    pub wake_word_enabled: bool,
    /// This client holds the voice slot.
    pub is_active_client: bool,
    /// The realtime session is usable.
    pub frontend_ready: bool,
    /// The user is talking.
    pub user_speaking: bool,
    /// The announcement window is blocked.
    pub window_blocked: bool,
    /// A connect is in flight.
    pub connecting: bool,
    /// A wake is in flight.
    pub waking: bool,
}

/// The gateway's `can_sleep` predicate.
///
/// **External contract** — `realtime-gateway.mjs:1852-1864`, condition for
/// condition.
#[must_use]
pub const fn can_sleep(inputs: &CanSleepInputs) -> bool {
    (inputs.input_enabled || inputs.wake_word_enabled)
        && inputs.is_active_client
        && inputs.frontend_ready
        && !inputs.user_speaking
        && !inputs.window_blocked
        && !inputs.connecting
        && !inputs.waking
}

#[derive(Debug, Default)]
struct State {
    enabled: bool,
    sleeping: bool,
    closed: bool,
    timeout_ms: u64,
    /// Bumped whenever a timer is superseded, so a fired timer belonging to an
    /// older arming does nothing.
    generation: u64,
}

/// The inactivity timer that drives a session into sleep.
#[derive(Clone)]
pub struct SleepController {
    state: Arc<std::sync::Mutex<State>>,
    retry_ms: u64,
    can_sleep: CanSleep,
    on_sleep: OnSleep,
}

impl std::fmt::Debug for SleepController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SleepController")
            .field("retry_ms", &self.retry_ms)
            .field(
                "state",
                &self.state.lock().map(|state| format!("{state:?}")),
            )
            .finish_non_exhaustive()
    }
}

impl SleepController {
    /// A disabled controller.
    ///
    /// **External contract** — `sleep-controller.mjs:8-16`: `timeoutMs` is
    /// floored at zero and `retryMs` at [`MIN_RETRY_MS`]; the controller starts
    /// disabled and awake.
    #[must_use]
    pub fn new(config: SleepControllerConfig) -> Self {
        Self {
            state: Arc::new(std::sync::Mutex::new(State {
                timeout_ms: config.timeout_ms,
                ..State::default()
            })),
            retry_ms: config.retry_ms.max(MIN_RETRY_MS),
            can_sleep: config.can_sleep,
            on_sleep: config.on_sleep,
        }
    }

    fn with_state<R>(&self, body: impl FnOnce(&mut State) -> R) -> Option<R> {
        // A poisoned lock means a previous holder panicked while mutating
        // three booleans. Recovering the guard is strictly better than
        // propagating a panic into the socket task: the worst case is a
        // controller that sleeps a beat late.
        match self.state.lock() {
            Ok(mut state) => Some(body(&mut state)),
            Err(poisoned) => Some(body(&mut poisoned.into_inner())),
        }
    }

    /// Arm the controller.
    ///
    /// **External contract** — `sleep-controller.mjs:18-23`. Answers `false`
    /// for a closed controller.
    pub fn enable(&self) -> bool {
        let armed = self.with_state(|state| {
            if state.closed {
                return None;
            }
            state.enabled = true;
            Some(state.timeout_ms)
        });
        match armed.flatten() {
            None => false,
            Some(timeout_ms) => {
                if timeout_ms > 0 {
                    self.record_activity();
                }
                true
            }
        }
    }

    /// Change the timeout, re-arming if the controller is awake and enabled.
    ///
    /// **External contract** — `sleep-controller.mjs:25-33`. A desktop client
    /// that advertises the `sleeping` state sets this to `0`, which owns its
    /// own inactivity policy and disarms the Gateway's timer.
    pub fn set_timeout_ms(&self, timeout_ms: u64) -> u64 {
        let rearm = self.with_state(|state| {
            state.timeout_ms = timeout_ms;
            state.generation += 1;
            state.enabled && !state.sleeping && state.timeout_ms > 0
        });
        if rearm == Some(true) {
            self.record_activity();
        }
        timeout_ms
    }

    /// Postpone the deadline.
    ///
    /// **External contract** — `sleep-controller.mjs:35-43`. A zero timeout
    /// clears the timer rather than scheduling one immediately.
    pub fn record_activity(&self) {
        let delay = self.with_state(|state| {
            if !state.enabled || state.sleeping || state.closed {
                return None;
            }
            state.generation += 1;
            (state.timeout_ms > 0).then_some((state.timeout_ms, state.generation))
        });
        let Some(Some((timeout_ms, generation))) = delay else {
            return;
        };
        self.schedule(timeout_ms, generation);
    }

    fn schedule(&self, delay_ms: u64, generation: u64) {
        let controller = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            controller.try_sleep(generation).await;
        });
    }

    async fn try_sleep(&self, generation: u64) {
        let proceed = self.with_state(|state| {
            state.generation == generation && state.enabled && !state.sleeping && !state.closed
        });
        if proceed != Some(true) {
            return;
        }
        if !(self.can_sleep)() {
            let retry = self.with_state(|state| {
                state.generation += 1;
                state.generation
            });
            if let Some(retry) = retry {
                self.schedule(self.retry_ms, retry);
            }
            return;
        }
        self.with_state(|state| state.sleeping = true);
        if (self.on_sleep)().await {
            return;
        }
        // The transition failed. Roll back and try again, rather than leaving
        // a session that believes it is asleep and has no timer.
        let retry = self.with_state(|state| {
            state.sleeping = false;
            state.generation += 1;
            state.generation
        });
        if let Some(retry) = retry {
            self.schedule(self.retry_ms, retry);
        }
    }

    /// Leave sleep. Answers whether it was actually asleep.
    ///
    /// **External contract** — `sleep-controller.mjs:67-73`.
    pub fn wake(&self) -> bool {
        let was = self.with_state(|state| {
            if state.closed {
                return None;
            }
            let was_sleeping = state.sleeping;
            state.sleeping = false;
            state.generation += 1;
            Some((was_sleeping, state.enabled))
        });
        match was.flatten() {
            None => false,
            Some((was_sleeping, enabled)) => {
                if enabled {
                    self.record_activity();
                }
                was_sleeping
            }
        }
    }

    /// Mark the session asleep without running the transition.
    ///
    /// **External contract** — `sleep-controller.mjs:75-81`. Used when the
    /// session entered sleep by another path — a failed wake-connect, an
    /// explicit client request — and the controller only has to stop counting.
    pub fn hold_sleeping(&self) -> bool {
        self.with_state(|state| {
            if !state.enabled || state.closed {
                return false;
            }
            state.generation += 1;
            state.sleeping = true;
            true
        })
        .unwrap_or(false)
    }

    /// Disarm and wake, without closing.
    pub fn disable(&self) {
        self.with_state(|state| {
            state.enabled = false;
            state.sleeping = false;
            state.generation += 1;
        });
    }

    /// Disarm permanently. [`Self::enable`] answers `false` afterwards.
    pub fn close(&self) {
        self.with_state(|state| state.closed = true);
        self.disable();
    }

    /// Whether the session is asleep.
    #[must_use]
    pub fn is_sleeping(&self) -> bool {
        self.with_state(|state| state.sleeping).unwrap_or(false)
    }

    /// Whether the controller is armed.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.with_state(|state| state.enabled).unwrap_or(false)
    }

    /// The current timeout.
    #[must_use]
    pub fn timeout_ms(&self) -> u64 {
        self.with_state(|state| state.timeout_ms).unwrap_or(0)
    }
}

/// Opens a wake-word engine for one keyword.
///
/// The seam a connection is composed against — `docs/deviations/README.md`'s
/// "attach a detector to a live connection". `via-wake-word`'s own default
/// build ships no engine at all (`--features sherpa` is deliberately kept out
/// of every default lane, `docs/adr/0001-placement.md`), so a production
/// Gateway wires [`NoWakeWordEngine`] and a test wires an opener over
/// [`via_wake_word::ScriptedDetector`]. Enabling `sherpa` later is a
/// different opener at composition, not a change here.
#[async_trait]
pub trait WakeWordDetectorOpener: Send + Sync + std::fmt::Debug {
    /// Build a detector listening for `keyword`.
    ///
    /// # Errors
    ///
    /// Whatever the engine failed to start with.
    /// [`WakeWordError::localized`] is the sentence this becomes on the wire.
    async fn open(&self, keyword: &Keyword) -> via_wake_word::Result<Box<dyn WakeWordDetector>>;
}

/// The opener every build without `--features sherpa` composes.
///
/// Answers [`WakeWordError::Engine`] unconditionally — there is no engine in
/// this build to open, which is the point: the seam is real and the engine is
/// opt-in.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoWakeWordEngine;

#[async_trait]
impl WakeWordDetectorOpener for NoWakeWordEngine {
    async fn open(&self, _keyword: &Keyword) -> via_wake_word::Result<Box<dyn WakeWordDetector>> {
        Err(WakeWordError::Engine {
            detail: "this build has no wake-word engine (feature \"sherpa\" is off)".to_owned(),
        })
    }
}

/// A live keyword-spotter, with the host's audio format in front of it.
///
/// The connection-level half of the seam:
/// [`via_wake_word::WakeWordStream`] generic over a boxed
/// [`WakeWordDetector`], so a connection can hold one without ever naming
/// which engine built it.
#[derive(Debug)]
pub struct SleepingWakeWord {
    stream: WakeWordStream<Box<dyn WakeWordDetector>>,
}

impl SleepingWakeWord {
    /// Wrap `detector` behind the host's capture format.
    ///
    /// # Errors
    ///
    /// [`WakeWordError::Audio`] if `capture_rate`/`channels` cannot resample
    /// to the detector's own rate.
    pub fn new(
        detector: Box<dyn WakeWordDetector>,
        capture_rate: SampleRate,
        channels: ChannelCount,
        keywords: KeywordSet,
    ) -> via_wake_word::Result<Self> {
        Ok(Self {
            stream: WakeWordStream::with_channels(detector, capture_rate, channels, keywords)?,
        })
    }

    /// Feed one chunk of interleaved little-endian PCM16 — `audio.append`
    /// while the session sleeps.
    ///
    /// # Errors
    ///
    /// Whatever the detector chain failed with — the caller's cue to disable
    /// wake-word detection for the rest of this connection, mirroring
    /// `acceptSleepingAudio`'s catch (`realtime-gateway.mjs:1976-1988`).
    pub fn accept_pcm16le(
        &mut self,
        bytes: &[u8],
    ) -> via_wake_word::Result<Option<via_wake_word::Detection>> {
        self.stream.accept_pcm16le(bytes)
    }

    /// Feed one chunk of base64-encoded PCM16 — `audio.append`'s `audio`
    /// field, exactly as it arrives on the wire.
    ///
    /// Mirrors upstream's own detector boundary: `accept(audio, sampleRate)`
    /// decodes internally (`pcm16Base64ToFloat32`,
    /// `sherpa-detector.mjs:69-70`) rather than asking its caller to. A
    /// payload that is not valid base64 decodes to nothing, which
    /// [`Self::accept_pcm16le`] already treats as a no-op — the same answer
    /// upstream's own decoder gives for anything it cannot decode.
    ///
    /// # Errors
    ///
    /// Whatever [`Self::accept_pcm16le`] failed with.
    pub fn accept_base64_pcm16le(
        &mut self,
        audio_base64: &str,
    ) -> via_wake_word::Result<Option<via_wake_word::Detection>> {
        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(audio_base64.trim())
            .unwrap_or_default();
        self.accept_pcm16le(&bytes)
    }

    /// Forget everything decoded so far — a sleep re-entered after a failed
    /// wake attempt, mirroring `wakeDetector.reset()` in `enterSleep`.
    pub fn reset(&mut self) {
        self.stream.reset();
    }
}

/// The two upstream guard conditions that make wake-word preparation a no-op
/// before it ever looks at a locale.
///
/// **External contract** — `prepareSleepMode`'s early return
/// (`realtime-gateway.mjs:1632`): `!config.wakeWordEnabled || nonVoiceClient
/// || inputSuspended || <a build is already in flight>`. VIA's lifecycle
/// prepares at most once per connection, so the fourth condition has no
/// counterpart here.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WakeWordPrepareInputs {
    /// The client will never play audio (`nonVoiceClient`).
    pub non_voice_client: bool,
    /// The host holds the microphone (`inputSuspended`).
    pub input_suspended: bool,
}

/// What preparing wake-word detection produced.
#[derive(Debug)]
pub enum WakeWordPrepareOutcome {
    /// The guard refused. No frames at all — upstream sends none either.
    NotApplicable,
    /// Wake word is switched on, but nothing is listening —
    /// `voice.sleep: disabled`.
    Disabled {
        /// The frame's `message` field.
        message: String,
    },
    /// Listening — `voice.sleep: enabled`. Boxed so the `NotApplicable` and
    /// `Disabled` variants do not carry a live detector's size.
    Enabled(Box<WakeWordEnabled>),
}

/// A live wake-word stream and the two fields `voice.sleep: enabled` reports
/// alongside it.
#[derive(Debug)]
pub struct WakeWordEnabled {
    /// The frame's `timeoutMs` field.
    pub timeout_ms: u64,
    /// The keyword this stream is listening for.
    pub keyword: Keyword,
    /// The live stream.
    pub stream: SleepingWakeWord,
}

/// The wake-word half of one connection: settings plus the engine that opens
/// a detector, ready to [`prepare`](Self::prepare) once.
#[derive(Clone)]
pub struct WakeWordLifecycle {
    settings: Arc<WakeWordSettings>,
    opener: Arc<dyn WakeWordDetectorOpener>,
    sleep_timeout_ms: u64,
}

impl std::fmt::Debug for WakeWordLifecycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WakeWordLifecycle")
            .field("enabled", &self.settings.is_enabled())
            .field("sleep_timeout_ms", &self.sleep_timeout_ms)
            .finish_non_exhaustive()
    }
}

impl WakeWordLifecycle {
    /// Build a lifecycle from `settings`, opening detectors through `opener`.
    #[must_use]
    pub const fn new(
        settings: Arc<WakeWordSettings>,
        opener: Arc<dyn WakeWordDetectorOpener>,
        sleep_timeout_ms: u64,
    ) -> Self {
        Self {
            settings,
            opener,
            sleep_timeout_ms,
        }
    }

    /// Build a lifecycle from a resolved [`Config`] and a caller-supplied
    /// keyword table.
    ///
    /// `via-core` carries the switch, the announced phrase and the install
    /// root; the table is the composition root's, because only whatever
    /// installed the keyword models on disk knows which locales actually
    /// have one — this crate does not discover them itself.
    #[must_use]
    pub fn from_config(
        config: &Config,
        keywords: KeywordSet,
        opener: Arc<dyn WakeWordDetectorOpener>,
    ) -> Self {
        Self::new(
            Arc::new(WakeWordSettings::from_config(config, keywords)),
            opener,
            u64::try_from(config.sleep_timeout_ms).unwrap_or(0),
        )
    }

    /// Whether the switch is on at all — `config.wakeWordEnabled`, before any
    /// locale resolves.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.settings.is_enabled()
    }

    /// `prepareSleepMode` — `realtime-gateway.mjs:1632-1657`.
    ///
    /// Always resolves to an outcome; the `preparing` frame is the caller's to
    /// send the instant this is called with anything other than
    /// [`NotApplicable`](WakeWordPrepareOutcome::NotApplicable), because
    /// upstream sends it *before* this async attempt, not after — this method
    /// cannot send it itself without knowing how the caller builds a frame.
    pub async fn prepare(
        &self,
        locale: Locale,
        inputs: WakeWordPrepareInputs,
        capture_rate: SampleRate,
        channels: ChannelCount,
    ) -> WakeWordPrepareOutcome {
        if !self.settings.is_enabled() || inputs.non_voice_client || inputs.input_suspended {
            return WakeWordPrepareOutcome::NotApplicable;
        }
        let keyword = match self.settings.resolve(locale) {
            Resolution::Enabled(keyword) => keyword.clone(),
            Resolution::Disabled(reason) => {
                return WakeWordPrepareOutcome::Disabled {
                    message: disabled_message(locale, &reason),
                };
            }
        };
        match self.opener.open(&keyword).await {
            Ok(detector) => {
                match SleepingWakeWord::new(
                    detector,
                    capture_rate,
                    channels,
                    self.settings.keywords().clone(),
                ) {
                    Ok(stream) => WakeWordPrepareOutcome::Enabled(Box::new(WakeWordEnabled {
                        timeout_ms: self.sleep_timeout_ms,
                        keyword,
                        stream,
                    })),
                    Err(error) => WakeWordPrepareOutcome::Disabled {
                        message: error.localized(locale),
                    },
                }
            }
            Err(error) => WakeWordPrepareOutcome::Disabled {
                message: error.localized(locale),
            },
        }
    }
}

/// The `disabled` message for a settings resolution that never reaches an
/// engine at all.
///
/// Reused from the same catalogued key upstream's engine-failure catch
/// renders into (`realtime.wake_word_detection_stopped`) — `via-wake-word`'s
/// own `error` module docs say why: from the outside, "no phrase is
/// configured" and "the engine could not open" are the same event. Detection
/// is not going to happen, and this is where the client learns it.
fn disabled_message(locale: Locale, reason: &DisabledReason) -> String {
    i18n_format(
        locale,
        keys::REALTIME_WAKE_WORD_DETECTION_STOPPED,
        &[("detail", &reason.to_string())],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One named way to break an otherwise-passing input.
    type Mutation = (&'static str, fn(&mut CanSleepInputs));
    use pretty_assertions::assert_eq;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    struct Harness {
        controller: SleepController,
        allowed: Arc<AtomicBool>,
        slept: Arc<AtomicUsize>,
        succeeds: Arc<AtomicBool>,
    }

    fn harness(timeout_ms: u64) -> Harness {
        let allowed = Arc::new(AtomicBool::new(true));
        let slept = Arc::new(AtomicUsize::new(0));
        let succeeds = Arc::new(AtomicBool::new(true));
        let can_read = Arc::clone(&allowed);
        let slept_write = Arc::clone(&slept);
        let succeeds_read = Arc::clone(&succeeds);
        let controller = SleepController::new(SleepControllerConfig::new(
            timeout_ms,
            Arc::new(move || can_read.load(Ordering::SeqCst)),
            Arc::new(move || {
                let slept = Arc::clone(&slept_write);
                let succeeds = Arc::clone(&succeeds_read);
                Box::pin(async move {
                    slept.fetch_add(1, Ordering::SeqCst);
                    succeeds.load(Ordering::SeqCst)
                })
            }),
        ));
        Harness {
            controller,
            allowed,
            slept,
            succeeds,
        }
    }

    #[tokio::test(start_paused = true)]
    async fn an_idle_session_sleeps_at_the_deadline() {
        let harness = harness(1_000);
        assert!(harness.controller.enable());
        tokio::time::sleep(Duration::from_millis(1_100)).await;
        assert_eq!(harness.slept.load(Ordering::SeqCst), 1);
        assert!(harness.controller.is_sleeping());
    }

    #[tokio::test(start_paused = true)]
    async fn activity_postpones_the_deadline() {
        let harness = harness(1_000);
        harness.controller.enable();
        for _ in 0..5 {
            tokio::time::sleep(Duration::from_millis(600)).await;
            harness.controller.record_activity();
        }
        assert_eq!(
            harness.slept.load(Ordering::SeqCst),
            0,
            "never idle long enough"
        );
        tokio::time::sleep(Duration::from_millis(1_100)).await;
        assert_eq!(harness.slept.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn a_refused_sleep_retries_rather_than_giving_up() {
        let harness = harness(1_000);
        harness.allowed.store(false, Ordering::SeqCst);
        harness.controller.enable();
        tokio::time::sleep(Duration::from_millis(1_100)).await;
        assert_eq!(harness.slept.load(Ordering::SeqCst), 0);
        assert!(!harness.controller.is_sleeping());
        // Still refused across several retries.
        tokio::time::sleep(Duration::from_millis(3_000)).await;
        assert_eq!(harness.slept.load(Ordering::SeqCst), 0);
        harness.allowed.store(true, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(1_100)).await;
        assert_eq!(
            harness.slept.load(Ordering::SeqCst),
            1,
            "the retry caught it"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_failed_transition_rolls_back_and_retries() {
        let harness = harness(1_000);
        harness.succeeds.store(false, Ordering::SeqCst);
        harness.controller.enable();
        tokio::time::sleep(Duration::from_millis(1_100)).await;
        assert_eq!(harness.slept.load(Ordering::SeqCst), 1);
        assert!(
            !harness.controller.is_sleeping(),
            "a failed sleep must not leave the session believing it slept",
        );
        harness.succeeds.store(true, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(1_100)).await;
        assert_eq!(harness.slept.load(Ordering::SeqCst), 2);
        assert!(harness.controller.is_sleeping());
    }

    #[tokio::test(start_paused = true)]
    async fn a_zero_timeout_never_sleeps_on_its_own() {
        let harness = harness(0);
        assert!(harness.controller.enable());
        tokio::time::sleep(Duration::from_secs(600)).await;
        assert_eq!(harness.slept.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(start_paused = true)]
    async fn setting_the_timeout_to_zero_disarms_a_running_timer() {
        let harness = harness(1_000);
        harness.controller.enable();
        assert_eq!(harness.controller.set_timeout_ms(0), 0);
        tokio::time::sleep(Duration::from_secs(60)).await;
        assert_eq!(harness.slept.load(Ordering::SeqCst), 0);
        // And setting it back re-arms.
        harness.controller.set_timeout_ms(1_000);
        tokio::time::sleep(Duration::from_millis(1_100)).await;
        assert_eq!(harness.slept.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn a_disabled_controller_never_sleeps() {
        let harness = harness(1_000);
        harness.controller.enable();
        harness.controller.disable();
        tokio::time::sleep(Duration::from_secs(60)).await;
        assert_eq!(harness.slept.load(Ordering::SeqCst), 0);
        assert!(!harness.controller.is_enabled());
    }

    #[tokio::test(start_paused = true)]
    async fn a_closed_controller_refuses_to_be_enabled() {
        let harness = harness(1_000);
        harness.controller.close();
        assert!(!harness.controller.enable());
        assert!(!harness.controller.hold_sleeping());
        assert!(!harness.controller.wake());
        tokio::time::sleep(Duration::from_secs(60)).await;
        assert_eq!(harness.slept.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(start_paused = true)]
    async fn wake_reports_whether_it_was_asleep_and_re_arms() {
        let harness = harness(1_000);
        harness.controller.enable();
        tokio::time::sleep(Duration::from_millis(1_100)).await;
        assert!(harness.controller.is_sleeping());
        assert!(harness.controller.wake(), "it was asleep");
        assert!(!harness.controller.wake(), "and now it is not");
        tokio::time::sleep(Duration::from_millis(1_100)).await;
        assert_eq!(
            harness.slept.load(Ordering::SeqCst),
            2,
            "the timer re-armed"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn hold_sleeping_marks_without_running_the_transition() {
        let harness = harness(1_000);
        harness.controller.enable();
        assert!(harness.controller.hold_sleeping());
        assert!(harness.controller.is_sleeping());
        tokio::time::sleep(Duration::from_secs(60)).await;
        assert_eq!(harness.slept.load(Ordering::SeqCst), 0, "no transition ran");
    }

    #[tokio::test(start_paused = true)]
    async fn hold_sleeping_needs_an_enabled_controller() {
        let harness = harness(1_000);
        assert!(!harness.controller.hold_sleeping());
    }

    #[test]
    fn the_can_sleep_predicate_needs_every_condition() {
        let idle = CanSleepInputs {
            input_enabled: true,
            wake_word_enabled: false,
            is_active_client: true,
            frontend_ready: true,
            user_speaking: false,
            window_blocked: false,
            connecting: false,
            waking: false,
        };
        assert!(can_sleep(&idle));

        let mutations: [Mutation; 6] = [
            ("no way back in", |inputs| {
                inputs.input_enabled = false;
                inputs.wake_word_enabled = false;
            }),
            ("not the active client", |inputs| {
                inputs.is_active_client = false
            }),
            ("nothing to close", |inputs| inputs.frontend_ready = false),
            ("user speaking", |inputs| inputs.user_speaking = true),
            ("gate blocked", |inputs| inputs.window_blocked = true),
            ("handshake in flight", |inputs| inputs.connecting = true),
        ];
        for (why, mutate) in mutations {
            let mut inputs = idle;
            mutate(&mut inputs);
            assert!(!can_sleep(&inputs), "{why} still permitted sleep");
        }

        let mut waking = idle;
        waking.waking = true;
        assert!(!can_sleep(&waking));
    }

    #[test]
    fn wake_word_alone_is_a_way_back_in() {
        let inputs = CanSleepInputs {
            input_enabled: false,
            wake_word_enabled: true,
            is_active_client: true,
            frontend_ready: true,
            ..CanSleepInputs::default()
        };
        assert!(can_sleep(&inputs));
    }

    #[test]
    fn the_retry_interval_is_floored() {
        let config = SleepControllerConfig::new(
            1_000,
            Arc::new(|| true),
            Arc::new(|| Box::pin(async { true })),
        )
        .retry_ms(0);
        assert_eq!(config.retry_ms, MIN_RETRY_MS);
    }

    // ---- wake-word lifecycle ----

    use std::sync::Mutex;

    use via_wake_word::ScriptedDetector;

    /// An opener that hands out one preset detector, once, and answers
    /// [`WakeWordError::Engine`] for every call after that — enough to drive
    /// [`WakeWordLifecycle::prepare`] with [`ScriptedDetector`] rather than
    /// asserting a value this test built by hand.
    #[derive(Debug)]
    struct ScriptedOpener {
        detector: Mutex<Option<ScriptedDetector>>,
    }

    impl ScriptedOpener {
        fn once(detector: ScriptedDetector) -> Self {
            Self {
                detector: Mutex::new(Some(detector)),
            }
        }
    }

    #[async_trait]
    impl WakeWordDetectorOpener for ScriptedOpener {
        async fn open(
            &self,
            _keyword: &Keyword,
        ) -> via_wake_word::Result<Box<dyn WakeWordDetector>> {
            self.detector
                .lock()
                .expect("not poisoned")
                .take()
                .map(|detector| Box::new(detector) as Box<dyn WakeWordDetector>)
                .ok_or(WakeWordError::Engine {
                    detail: "already opened".to_owned(),
                })
        }
    }

    /// An opener that always refuses, for the "no engine in this build" and
    /// "the engine itself failed" cases.
    #[derive(Debug, Default)]
    struct RefusingOpener;

    #[async_trait]
    impl WakeWordDetectorOpener for RefusingOpener {
        async fn open(
            &self,
            _keyword: &Keyword,
        ) -> via_wake_word::Result<Box<dyn WakeWordDetector>> {
            Err(WakeWordError::Engine {
                detail: "no engine".to_owned(),
            })
        }
    }

    fn en_keyword() -> Keyword {
        Keyword::zh_en("hey via", "HH EY1 V IY1 AH0").expect("a valid fixture keyword")
    }

    fn settings(enabled: bool, keywords: KeywordSet) -> Arc<WakeWordSettings> {
        Arc::new(WakeWordSettings::new(
            enabled,
            "/var/lib/via/wake-word",
            keywords,
        ))
    }

    #[tokio::test]
    async fn the_switch_being_off_is_not_applicable_and_never_touches_the_opener() {
        let lifecycle = WakeWordLifecycle::new(
            settings(false, KeywordSet::new().with(Locale::En, en_keyword())),
            Arc::new(RefusingOpener),
            5_000,
        );
        let outcome = lifecycle
            .prepare(
                Locale::En,
                WakeWordPrepareInputs::default(),
                SampleRate::HZ_16000,
                ChannelCount::MONO,
            )
            .await;
        assert!(matches!(outcome, WakeWordPrepareOutcome::NotApplicable));
    }

    #[tokio::test]
    async fn a_non_voice_client_and_a_suspended_microphone_are_also_not_applicable() {
        let lifecycle = WakeWordLifecycle::new(
            settings(true, KeywordSet::new().with(Locale::En, en_keyword())),
            Arc::new(RefusingOpener),
            5_000,
        );
        for inputs in [
            WakeWordPrepareInputs {
                non_voice_client: true,
                input_suspended: false,
            },
            WakeWordPrepareInputs {
                non_voice_client: false,
                input_suspended: true,
            },
        ] {
            let outcome = lifecycle
                .prepare(Locale::En, inputs, SampleRate::HZ_16000, ChannelCount::MONO)
                .await;
            assert!(matches!(outcome, WakeWordPrepareOutcome::NotApplicable));
        }
    }

    #[tokio::test]
    async fn an_unconfigured_phrase_disables_without_ever_opening_an_engine() {
        let lifecycle = WakeWordLifecycle::new(
            settings(true, KeywordSet::new()),
            Arc::new(RefusingOpener),
            5_000,
        );
        let outcome = lifecycle
            .prepare(
                Locale::En,
                WakeWordPrepareInputs::default(),
                SampleRate::HZ_16000,
                ChannelCount::MONO,
            )
            .await;
        let WakeWordPrepareOutcome::Disabled { message } = outcome else {
            panic!("expected Disabled, got {outcome:?}");
        };
        assert!(
            message.contains("no wake phrase is configured"),
            "got {message:?}",
        );
    }

    #[tokio::test]
    async fn no_engine_in_the_default_build_disables_with_the_localized_message() {
        let lifecycle = WakeWordLifecycle::new(
            settings(true, KeywordSet::new().with(Locale::En, en_keyword())),
            Arc::new(NoWakeWordEngine),
            5_000,
        );
        let outcome = lifecycle
            .prepare(
                Locale::En,
                WakeWordPrepareInputs::default(),
                SampleRate::HZ_16000,
                ChannelCount::MONO,
            )
            .await;
        let WakeWordPrepareOutcome::Disabled { message } = outcome else {
            panic!("expected Disabled, got {outcome:?}");
        };
        assert!(message.contains("sherpa"), "got {message:?}");
    }

    #[tokio::test]
    async fn a_scripted_engine_enables_and_the_stream_detects_the_configured_phrase() {
        let keyword = en_keyword();
        let keywords = KeywordSet::new().with(Locale::En, keyword.clone());
        let lifecycle = WakeWordLifecycle::new(
            settings(true, keywords),
            Arc::new(ScriptedOpener::once(ScriptedDetector::detecting(
                keyword.label(),
            ))),
            7_000,
        );
        let outcome = lifecycle
            .prepare(
                Locale::En,
                WakeWordPrepareInputs::default(),
                SampleRate::HZ_16000,
                ChannelCount::MONO,
            )
            .await;
        let WakeWordPrepareOutcome::Enabled(enabled) = outcome else {
            panic!("expected Enabled, got {outcome:?}");
        };
        let WakeWordEnabled {
            timeout_ms,
            keyword: resolved,
            mut stream,
        } = *enabled;
        assert_eq!(timeout_ms, 7_000);
        assert_eq!(resolved.text(), "hey via");

        // 20 ms of 16 kHz mono silence — no resampler in the path, so the
        // detector sees it on the first call.
        let detection = stream
            .accept_pcm16le(&[0u8; 640])
            .expect("the scripted engine never errors")
            .expect("the script fires on the first non-empty chunk");
        assert_eq!(detection.keyword, keyword.label());
        assert_eq!(detection.locale, Some(Locale::En));
    }

    #[test]
    fn the_base64_boundary_decodes_exactly_like_the_raw_one() {
        use base64::Engine as _;

        let keyword = en_keyword();
        let mut stream = SleepingWakeWord::new(
            Box::new(ScriptedDetector::detecting(keyword.label())),
            SampleRate::HZ_16000,
            ChannelCount::MONO,
            KeywordSet::new().with(Locale::En, keyword.clone()),
        )
        .expect("a matching rate needs no resampler");

        let encoded = base64::engine::general_purpose::STANDARD.encode([0u8; 640]);
        let detection = stream
            .accept_base64_pcm16le(&encoded)
            .expect("decodes")
            .expect("the script fires on the first non-empty chunk");
        assert_eq!(detection.keyword, keyword.label());
    }

    #[test]
    fn audio_that_is_not_base64_is_a_no_op_not_an_error() {
        let keyword = en_keyword();
        let mut stream = SleepingWakeWord::new(
            Box::new(ScriptedDetector::detecting(keyword.label())),
            SampleRate::HZ_16000,
            ChannelCount::MONO,
            KeywordSet::new().with(Locale::En, keyword),
        )
        .expect("a matching rate needs no resampler");

        assert_eq!(
            stream
                .accept_base64_pcm16le("not base64 at all!!")
                .expect("never errors on undecodable input"),
            None,
        );
    }
}
