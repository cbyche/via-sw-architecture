//! `via-gateway-e2e` — a real Gateway process with exactly two seams swapped.
//!
//! This is not a second Gateway. It is [`via::gateway::serve`] — the function
//! `via gateway` itself calls — reached through the same steps in the same
//! order: sample the process environment, load `.env.local` / `.env` /
//! `config.env` and scaffold what is missing, resolve the configuration, run
//! the setup gate, take the single-instance lease, compose, bind, publish the
//! origin, print the banner, serve until SIGINT or SIGTERM, close.
//!
//! What differs is [`via::gateway::Composition`], and only in its two seams:
//!
//! | Seam | `via gateway` | here |
//! | --- | --- | --- |
//! | [`SessionOpener`] | [`via::gateway::ConnectOpener`] — a WebSocket to the provider | [`ScriptOpener`] — a `via-realtime-mock` script read from `VIA_E2E_SCRIPT` |
//! | [`DownstreamAgent`] | an ACP child process resolved from `AGENT_PROTOCOL` | a [`ScriptedHarness`] built from `VIA_E2E_HARNESS` |
//!
//! Both are **files on disk**, named by the environment, because the process
//! that authors the fixture is not the process that runs it. A test writes a
//! `Script` and a harness specification as JSON, spawns this binary pointed at
//! them, and observes the result over a socket and in the config directory.
//!
//! # Why a binary at all
//!
//! `docs/architecture.md` §14's discipline is that what an external party can
//! observe is reproduced exactly. Three of `via-e2e`'s seven cases — the
//! conversation, cancellation across the process boundary, and restart recovery
//! after SIGKILL — are only meaningful against a process that has a model and a
//! backend. The shipped binary has neither in a test: its realtime providers
//! dial real endpoints and its Layer 3 spawns an ACP child. Substituting them
//! in-process would give back the suite `apps/via/tests/milestone.rs` already
//! is. Substituting them *inside a spawned process* keeps the lease, the
//! signals, `tasks.json`, the accept loop and the log file real, which is
//! precisely what those three cases are about.
//!
//! # The panic budget applies
//!
//! `scripts/panic_filter.py` and `scripts/risky_unwrap.py` walk `<crate>/src/`,
//! and this file is in it. There is no `unwrap`, no `expect`, no `panic!` and
//! no `unreachable!` below: every fallible step reports on stderr and exits
//! non-zero, exactly as the product does.
//!
//! [`SessionOpener`]: via::gateway::SessionOpener
//! [`DownstreamAgent`]: via_downstream::DownstreamAgent
//! [`ScriptedHarness`]: via_downstream::testing::ScriptedHarness

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::Mutex;
use via::gateway::{Composition, SessionOpener};
use via_downstream::testing::{ScriptedHarness, ScriptedTurn};
use via_downstream::{
    CancelOutcome, CancelScope, DownstreamAgent, HarnessDescriptor, HarnessError, HarnessHealth,
    HarnessSession, PromptOutcome, PromptRequest, SessionEvent, SessionKey,
};
use via_realtime::{
    RealtimeError, RealtimeProvider, RealtimeSession, SessionEvents, SessionOptions,
};
use via_realtime_mock::{MockHandle, MockRealtime, Script};

/// Names the `via-realtime-mock` script file the realtime seam replays.
///
/// Required: a harness Gateway with no script has no model, which is the one
/// thing this binary exists to supply.
pub const SCRIPT_ENV: &str = "VIA_E2E_SCRIPT";

/// Names the JSON file describing the [`ScriptedHarness`] behind the
/// coordinator. Absent means *no harness*, which is upstream's frontend-only
/// mode (`docs/architecture.md` §2 — *"`agent` degrades to `direct` when no
/// harness is configured"*).
pub const HARNESS_ENV: &str = "VIA_E2E_HARNESS";

/// Names a file whose **existence** releases a held backend turn.
///
/// A [`ScriptedHarness`] answers instantly, which is right for almost
/// everything and wrong for the two cases that need a Work to still be *live*
/// when something else acts on it: a cancellation that must find a running
/// delegation, and a SIGKILL that must interrupt one. In-process those tests
/// hold a `Notify`; across a process boundary the rendezvous has to be
/// something both sides can see, and a file is the least surprising one.
///
/// A path that is never created holds the turn open until [`HOLD_LIMIT`].
pub const HOLD_ENV: &str = "VIA_E2E_HOLD";

/// How long a held turn waits before giving up and running anyway.
///
/// A bound rather than `loop {}`: a wedged fixture should fail a test, not
/// leave a Gateway holding a lease and a port for the rest of the run. It is far
/// longer than any budget a test uses, so a passing test never reaches it.
pub const HOLD_LIMIT: Duration = Duration::from_secs(120);

/// How often the held turn re-checks for [`HOLD_ENV`].
pub const HOLD_POLL: Duration = Duration::from_millis(20);

/// The backend id a harness specification defaults to.
///
/// Any catalogued id would do; `opencode` is the one
/// `apps/via/tests/milestone.rs` uses, so the two suites' fixtures describe the
/// same backend.
pub const DEFAULT_BACKEND: &str = "opencode";

/// One scripted backend turn, as the fixture file spells it.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TurnSpec {
    /// The turn answers `content` and ends normally.
    Completed {
        /// What the backend replies with — for a coordinator turn, the decision
        /// envelope as a JSON *string*.
        content: String,
    },
    /// The turn fails the way a transport failure arrives.
    Failing {
        /// The already-localized sentence.
        message: String,
        /// An HTTP status, or `0`.
        #[serde(default)]
        status: u16,
    },
    /// The turn is cancelled.
    Cancelled,
}

impl TurnSpec {
    /// The `via-downstream` turn this describes.
    fn into_turn(self) -> ScriptedTurn {
        match self {
            Self::Completed { content } => ScriptedTurn::completed(&content),
            Self::Failing { message, status } => ScriptedTurn::failing(&message, status),
            Self::Cancelled => ScriptedTurn::cancelled(),
        }
    }
}

/// The harness fixture file's shape.
#[derive(Debug, Clone, Deserialize)]
pub struct HarnessSpec {
    /// A catalogued backend id.
    #[serde(default = "default_backend")]
    pub backend: String,
    /// The turns, consumed in order across every session.
    #[serde(default)]
    pub turns: Vec<TurnSpec>,
}

fn default_backend() -> String {
    DEFAULT_BACKEND.to_owned()
}

/// The realtime seam: one `via-realtime-mock` session per open, from a script
/// re-read off disk each time.
///
/// Re-reading rather than replaying one parsed copy is deliberate. A Gateway
/// serves many sockets over its life, and a test that reconnects must get a
/// session rather than a refusal; handing every open its own script server is
/// what a real provider does. It also means a fixture file rewritten between
/// two connections takes effect, which is how the restart case gives the second
/// process a different model from the first.
#[derive(Debug)]
pub struct ScriptOpener {
    path: PathBuf,
    /// Every script server opened so far, kept alive for the process's life.
    ///
    /// The server task stops when its last handle drops, so dropping these
    /// would close the transport out from under a live session.
    handles: Mutex<Vec<MockHandle>>,
}

impl ScriptOpener {
    /// An opener that replays whatever `path` holds at the moment of each open.
    #[must_use]
    pub fn new(path: PathBuf) -> Arc<Self> {
        Arc::new(Self {
            path,
            handles: Mutex::new(Vec::new()),
        })
    }
}

#[async_trait]
impl SessionOpener for ScriptOpener {
    async fn open(
        &self,
        _provider: &Arc<dyn RealtimeProvider>,
        options: SessionOptions,
    ) -> Result<(RealtimeSession, SessionEvents), RealtimeError> {
        let script = Script::from_json_file(&self.path).map_err(|error| {
            // The refusal a provider whose endpoint will not answer produces.
            // `via-app` reports it to the client as an `error` frame rather than
            // dropping it, so a broken fixture surfaces on the socket instead of
            // as silence.
            RealtimeError::ConnectionClosed {
                label: error.to_string(),
            }
        })?;
        let (handle, opened) = MockRealtime::new(script)
            .with_options(options)
            .try_open()
            .await;
        self.handles.lock().await.push(handle);
        opened
    }
}

/// A [`ScriptedHarness`] whose every turn can be pinned open by a file.
///
/// Everything except `prompt`'s timing is delegated — the descriptor, the
/// session id, the event stream and, importantly, `cancel`, so the cancellation
/// path under test is the harness's own rather than this wrapper's.
pub struct HoldingHarness {
    inner: Arc<dyn DownstreamAgent>,
    hold: Option<PathBuf>,
}

impl std::fmt::Debug for HoldingHarness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoldingHarness")
            .field("backend", &self.inner.descriptor().id())
            .field("hold", &self.hold)
            .finish()
    }
}

impl HoldingHarness {
    /// Wrap `inner`, holding every turn until `hold` exists.
    #[must_use]
    pub fn new(inner: Arc<dyn DownstreamAgent>, hold: Option<PathBuf>) -> Arc<Self> {
        Arc::new(Self { inner, hold })
    }
}

#[async_trait]
impl DownstreamAgent for HoldingHarness {
    fn descriptor(&self) -> &HarnessDescriptor {
        self.inner.descriptor()
    }

    async fn open(&self, key: &SessionKey) -> Result<Box<dyn HarnessSession>, HarnessError> {
        Ok(Box::new(HoldingSession {
            inner: self.inner.open(key).await?,
            hold: self.hold.clone(),
        }))
    }

    async fn health(&self) -> HarnessHealth {
        self.inner.health().await
    }
}

/// One held session.
struct HoldingSession {
    inner: Box<dyn HarnessSession>,
    hold: Option<PathBuf>,
}

/// Wait until `path` exists, or [`HOLD_LIMIT`] elapses.
async fn wait_for(path: &Path) {
    let deadline = tokio::time::Instant::now() + HOLD_LIMIT;
    while !path.exists() && tokio::time::Instant::now() < deadline {
        tokio::time::sleep(HOLD_POLL).await;
    }
}

#[async_trait]
impl HarnessSession for HoldingSession {
    fn session_id(&self) -> &str {
        self.inner.session_id()
    }

    async fn prompt(&self, request: PromptRequest) -> Result<PromptOutcome, HarnessError> {
        if let Some(hold) = &self.hold {
            wait_for(hold).await;
        }
        self.inner.prompt(request).await
    }

    fn events(&self) -> futures::stream::BoxStream<'static, SessionEvent> {
        self.inner.events()
    }

    async fn cancel(&self, scope: CancelScope) -> Result<CancelOutcome, HarnessError> {
        self.inner.cancel(scope).await
    }
}

/// Read the harness fixture, or answer `None` for frontend-only mode.
fn read_harness(path: &Path) -> Result<Arc<dyn DownstreamAgent>, String> {
    let text =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let spec: HarnessSpec =
        serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut builder = ScriptedHarness::builder(&spec.backend);
    for turn in spec.turns {
        builder = builder.turn(turn.into_turn());
    }
    // A descriptor that is incomplete or internally inconsistent is rejected
    // here, at composition — which is the property `docs/architecture.md` §6
    // asks for, and the reason the fixture is validated rather than trusted.
    let harness = builder.build().map_err(|error| error.to_string())?;
    Ok(Arc::new(harness))
}

/// The whole run, as a `Result` so `main` has one exit path.
fn run() -> Result<(), String> {
    let mut host = via::Host::from_process();
    // `false` — a Gateway is not a read-only inspection, so this scaffolds the
    // configuration directory and generates the auth secret exactly as
    // `via gateway` does.
    host.load_runtime(false)
        .map_err(|error| error.message(via_i18n::Locale::En))?;
    let config = host
        .resolve_config()
        .map_err(|error| error.message(via_i18n::Locale::En))?;
    via_core::assert_gateway_setup(host.env(), host.locale()).map_err(|error| error.to_string())?;

    let script_path = match host.env().get_truthy(SCRIPT_ENV) {
        Some(path) => PathBuf::from(path),
        None => return Err(format!("{SCRIPT_ENV} is required")),
    };
    let script = Script::from_json_file(&script_path)
        .map_err(|error| format!("{}: {error}", script_path.display()))?;
    let provider = MockRealtime::new(script)
        .provider()
        .map_err(|error| error.to_string())?;

    let harness = match host.env().get_truthy(HARNESS_ENV) {
        Some(path) => Some(read_harness(Path::new(path))?),
        None => None,
    };
    let hold = host.env().get_truthy(HOLD_ENV).map(PathBuf::from);
    let harness = harness.map(|inner| HoldingHarness::new(inner, hold) as Arc<dyn DownstreamAgent>);

    // The logger the product builds, built the same way — including the
    // test-process check, so a run under `cargo test`'s own argv still writes
    // nowhere. `via-e2e` spawns this binary by path, which is why
    // `logs/gateway.log` is a real file the security case can grep.
    let mut options = via_log::LoggerOptions::gateway(host.env(), host.home_directory());
    if via_log::is_test_process(host.env(), std::env::args()) {
        options.console_enabled = false;
        options.file_enabled = false;
    }

    let composition = Composition {
        opener: ScriptOpener::new(script_path) as Arc<dyn SessionOpener>,
        providers: vec![provider],
        harness,
        logger: Some(Arc::new(via_log::Logger::new(options))),
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let mut out = std::io::stdout();
    runtime
        .block_on(via::gateway::serve(
            &config,
            host.env(),
            composition,
            &mut out,
        ))
        .map_err(|error| error.message(via_i18n::Locale::En))
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            // The product's own stderr shape, so a failed harness Gateway reads
            // like a failed `via gateway` in a test's captured output.
            eprintln!("{}{message}", via::STDERR_PREFIX);
            ExitCode::from(via::EXIT_FAILURE)
        }
    }
}
