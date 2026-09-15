//! A harness that answers by **what it was asked**, not by a queue.
//!
//! [`via_downstream::testing::ScriptedHarness`] hands out turns in order across
//! every session, which is exactly right for a single-session test and exactly
//! wrong here: a delegation runs two sessions concurrently, so a positional
//! script would make every assertion about the coordinator a race about task
//! scheduling instead.
//!
//! Upstream's own test double solves it the same way — `fakeAcpClient`
//! (`server/test/acp-backend-adapter.test.mjs:73-205`) branches on the prompt
//! text: `if (prompt.includes('kind="cancel"'))`, `if (prompt.includes(…
//! delegation_result))`, and so on. This is that double, in Rust.

#![allow(dead_code)]

use std::sync::{Arc, Mutex, MutexGuard};

use async_trait::async_trait;
use futures::future::BoxFuture;
use futures::stream::{BoxStream, StreamExt};
use via_downstream::testing::consistent_capabilities;
use via_downstream::{
    CancelOutcome, CancelRoute, CancelScope, CancelTarget, DownstreamAgent, HarnessDescriptor,
    HarnessError, HarnessHealth, HarnessSession, PromptOutcome, PromptRequest, SessionEvent,
    SessionKey, StopReason,
};

/// The catalogued backend this double declares itself as.
///
/// It is named rather than selected because the fixtures assert on the
/// *rendered* session key, which contains the backend id. The choice itself is
/// by property: a `sessionMcp` backend with `nativeDelegation: false` and no
/// service address, which is the shape ten of the twelve share, so nothing here
/// depends on which one it is.
///
/// `via-arch-test`'s no-named-backend rule covers the two generic cores —
/// `via-acp` and `via-process` — and not this crate; the descriptor is still
/// built through [`HarnessDescriptor::declare`] against the real catalog, so
/// the double is validated exactly as a shipped driver is.
pub const BACKEND_ID: &str = "opencode";

/// Its catalogued label.
pub const BACKEND_LABEL: &str = "OpenCode";

/// The session id the coordinator session reports.
pub const COORDINATOR_SESSION_ID: &str = "coordinator-session";

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// How many task polls a `settle_until` may take before it gives up.
///
/// Every test here runs on a paused, single-threaded runtime, so "wait for
/// something another task will do" is `yield_now` in a loop. The loop is
/// **bounded** on purpose: an unbounded one turns a broken invariant into a
/// hang, and a hung test tells nobody anything. The bound is generous — what is
/// being waited for is a handful of polls, never a timer.
pub const MAX_SETTLE_POLLS: usize = 10_000;

/// Yield until `probe` answers true, or fail saying what never happened.
pub async fn settle_until<F, Fut>(what: &str, mut probe: F)
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    for _ in 0..MAX_SETTLE_POLLS {
        if probe().await {
            return;
        }
        tokio::task::yield_now().await;
    }
    panic!("gave up waiting for {what} after {MAX_SETTLE_POLLS} polls");
}

/// How the double answers one prompt.
#[derive(Debug, Clone)]
pub enum Reply {
    /// End the turn with this content.
    Content(String),
    /// End the turn with this content and this stop reason.
    Stopping(String, StopReason),
    /// Fail the turn the way a transport failure arrives.
    Failing(String),
    /// Never answer on its own; end only when the turn is cancelled.
    ///
    /// How a test holds a delegated Session open without a clock.
    Park,
}

impl Reply {
    /// An `end_turn` reply.
    pub fn content(text: &str) -> Self {
        Self::Content(text.to_owned())
    }
}

/// One routing rule: a needle in the prompt, and what to answer.
#[derive(Debug, Clone)]
struct Route {
    needle: String,
    reply: Reply,
}

/// Something the double does **before** answering.
///
/// Upstream's fake client calls `toolServer.context.startSession(…)` from
/// inside its `prompt` (`acp-backend-adapter.test.mjs:176-186`), which is the
/// only way a test can put a tool call *inside* a coordinator turn rather than
/// beside it. This is that.
pub type PromptHook = Arc<dyn Fn() -> BoxFuture<'static, ()> + Send + Sync>;

#[derive(Default)]
struct Routing {
    routes: Vec<Route>,
    fallback: Option<Reply>,
    hooks: Vec<(String, PromptHook)>,
}

impl std::fmt::Debug for Routing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Routing")
            .field("routes", &self.routes.len())
            .field("hooks", &self.hooks.len())
            .finish()
    }
}

impl Routing {
    fn reply_for(&self, prompt: &str) -> Reply {
        self.routes
            .iter()
            .find(|route| prompt.contains(&route.needle))
            .map(|route| route.reply.clone())
            .or_else(|| self.fallback.clone())
            .unwrap_or_else(|| Reply::Content(String::new()))
    }

    fn hook_for(&self, prompt: &str) -> Option<PromptHook> {
        self.hooks
            .iter()
            .find(|(needle, _)| prompt.contains(needle))
            .map(|(_, hook)| Arc::clone(hook))
    }
}

/// Builds a [`RoutingHarness`].
#[derive(Debug, Default)]
pub struct RoutingHarnessBuilder {
    routes: Vec<Route>,
    fallback: Option<Reply>,
}

impl RoutingHarnessBuilder {
    /// Answer `reply` to any prompt containing `needle`. First match wins.
    #[must_use]
    pub fn route(mut self, needle: &str, reply: Reply) -> Self {
        self.routes.push(Route {
            needle: needle.to_owned(),
            reply,
        });
        self
    }

    /// Answer `reply` to anything no route matched.
    #[must_use]
    pub fn fallback(mut self, reply: Reply) -> Self {
        self.fallback = Some(reply);
        self
    }

    /// Build.
    #[must_use]
    pub fn build(self) -> Arc<RoutingHarness> {
        let capabilities = consistent_capabilities(via_catalog_definition());
        let descriptor = HarnessDescriptor::declare(BACKEND_ID, BACKEND_LABEL, capabilities)
            .expect("the catalogued backend declares consistently");
        Arc::new(RoutingHarness {
            descriptor,
            routing: Arc::new(Mutex::new(Routing {
                routes: self.routes,
                fallback: self.fallback,
                hooks: Vec::new(),
            })),
            state: Mutex::new(HarnessState::default()),
        })
    }
}

/// The catalog entry for [`BACKEND_ID`].
fn via_catalog_definition() -> &'static via_catalog::BackendDefinition {
    via_catalog::backend_definition(BACKEND_ID).expect("a catalogued backend")
}

#[derive(Debug, Default)]
struct HarnessState {
    sessions: Vec<RoutingSession>,
    opened: Vec<String>,
    project_ordinal: usize,
}

/// A harness that answers by prompt content.
#[derive(Debug)]
pub struct RoutingHarness {
    descriptor: HarnessDescriptor,
    routing: Arc<Mutex<Routing>>,
    state: Mutex<HarnessState>,
}

impl RoutingHarness {
    /// Start building.
    #[must_use]
    pub fn builder() -> RoutingHarnessBuilder {
        RoutingHarnessBuilder::default()
    }

    /// Every session key this harness was asked to open, in order.
    #[must_use]
    pub fn opened(&self) -> Vec<String> {
        lock(&self.state).opened.clone()
    }

    /// Every session it opened, in order.
    #[must_use]
    pub fn sessions(&self) -> Vec<RoutingSession> {
        lock(&self.state).sessions.clone()
    }

    /// Every prompt every session received, in the order they arrived.
    #[must_use]
    pub fn prompts(&self) -> Vec<String> {
        self.sessions()
            .iter()
            .flat_map(RoutingSession::prompts)
            .collect()
    }

    /// The prompts the coordinator session received.
    #[must_use]
    pub fn coordinator_prompts(&self) -> Vec<String> {
        self.sessions()
            .iter()
            .filter(|session| session.session_id() == COORDINATOR_SESSION_ID)
            .flat_map(RoutingSession::prompts)
            .collect()
    }

    /// Run `hook` inside any prompt containing `needle`, before answering it.
    ///
    /// Installed after the coordinator exists, because the hook is usually a
    /// call back into it.
    pub fn on_prompt(&self, needle: &str, hook: PromptHook) {
        lock(&self.routing).hooks.push((needle.to_owned(), hook));
    }
}

#[async_trait]
impl DownstreamAgent for RoutingHarness {
    fn descriptor(&self) -> &HarnessDescriptor {
        &self.descriptor
    }

    async fn open(&self, key: &SessionKey) -> Result<Box<dyn HarnessSession>, HarnessError> {
        let mut state = lock(&self.state);
        state.opened.push(key.as_str().to_owned());
        let session_id = if key.is_coordinator() {
            COORDINATOR_SESSION_ID.to_owned()
        } else {
            match key.session_id().unwrap_or_default() {
                "" => {
                    state.project_ordinal += 1;
                    format!("project-{}", state.project_ordinal)
                }
                existing => existing.to_owned(),
            }
        };
        let session = RoutingSession {
            inner: Arc::new(SessionInner {
                session_id,
                routing: Arc::clone(&self.routing),
                prompts: Mutex::new(Vec::new()),
                cancels: Mutex::new(Vec::new()),
            }),
        };
        state.sessions.push(session.clone());
        Ok(Box::new(session))
    }

    async fn health(&self) -> HarnessHealth {
        HarnessHealth::ready()
    }
}

#[derive(Debug)]
struct SessionInner {
    session_id: String,
    routing: Arc<Mutex<Routing>>,
    prompts: Mutex<Vec<String>>,
    cancels: Mutex<Vec<CancelScope>>,
}

/// One session on a [`RoutingHarness`]. Cheap to clone; every clone is the same
/// session.
#[derive(Debug, Clone)]
pub struct RoutingSession {
    inner: Arc<SessionInner>,
}

impl RoutingSession {
    /// Every prompt this session received, in order.
    #[must_use]
    pub fn prompts(&self) -> Vec<String> {
        lock(&self.inner.prompts).clone()
    }

    /// Every cancel this session was asked for.
    #[must_use]
    pub fn cancels(&self) -> Vec<CancelScope> {
        lock(&self.inner.cancels).clone()
    }
}

#[async_trait]
impl HarnessSession for RoutingSession {
    fn session_id(&self) -> &str {
        &self.inner.session_id
    }

    async fn prompt(&self, request: PromptRequest) -> Result<PromptOutcome, HarnessError> {
        lock(&self.inner.prompts).push(request.text.clone());
        let hook = lock(&self.inner.routing).hook_for(&request.text);
        if let Some(hook) = hook {
            hook().await;
        }
        let reply = lock(&self.inner.routing).reply_for(&request.text);
        match reply {
            Reply::Content(content) => PromptOutcome::new(&content, StopReason::EndTurn),
            Reply::Stopping(content, stop_reason) => PromptOutcome::new(&content, stop_reason),
            Reply::Failing(message) => Err(HarnessError::Agent {
                message,
                status: 0,
                body: String::new(),
                protocol: BACKEND_ID.to_owned(),
            }),
            // Parked: the only way out is the caller's own cancellation, which
            // `Coordinator::delegate` races this future against.
            Reply::Park => std::future::pending().await,
        }
    }

    fn events(&self) -> BoxStream<'static, SessionEvent> {
        futures::stream::empty().boxed()
    }

    async fn cancel(&self, scope: CancelScope) -> Result<CancelOutcome, HarnessError> {
        lock(&self.inner.cancels).push(scope.clone());
        let target = scope.delegation_id().map_or_else(
            || CancelTarget::session(&self.inner.session_id),
            CancelTarget::delegation,
        );
        Ok(CancelOutcome::requested(CancelRoute::Adapter, target))
    }
}
