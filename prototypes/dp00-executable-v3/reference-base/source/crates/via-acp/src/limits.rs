//! The timeouts and bounds the ACP client is built around.
//!
//! **External contract** — catalogued as *"timeouts and limits"*
//! (`docs/reference/contracts.json`), sourced from
//! `server/src/agent/acp-process-client.mjs:14,17,18,71,224,428,498,528,539,552`.
//! Every one of these changes observable behaviour under load or failure, so
//! they are named constants rather than literals at their call sites, and
//! `tests/limits.rs` asserts each against the catalogue.
//!
//! The adapter-owned half of that contract (`MAX_SESSION_RESULTS`,
//! `MAX_DELEGATION_RESULT_CHARS`, the health-failure backoff, the permission
//! broker's `resolvedLimit`, the session-tool server's host and port) belongs
//! to the crates that own those surfaces; only the process client's values are
//! here.

use std::time::Duration;

/// Default per-request deadline — `AcpProcessClient.timeoutMs = 300_000`.
///
/// `acp-process-client.mjs:71`. Also the adapter's default, which is why a
/// harness that overrides one must override both.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_millis(300_000);

/// `initialize` deadline — `AbortSignal.timeout(15_000)`.
///
/// `acp-process-client.mjs:224`. Deliberately *not* [`DEFAULT_TIMEOUT`]: a
/// backend that never answers `initialize` must fail fast enough to report
/// "CLI not installed / not authenticated" rather than hang a voice turn.
pub const INITIALIZE_TIMEOUT: Duration = Duration::from_millis(15_000);

/// `session/list` per-page deadline — `acp-process-client.mjs:428`.
pub const SESSION_LIST_TIMEOUT: Duration = Duration::from_millis(15_000);

/// `session/set_config_option` deadline — `acp-process-client.mjs:528`.
pub const SET_CONFIG_OPTION_TIMEOUT: Duration = Duration::from_millis(15_000);

/// `session/set_model` deadline — `acp-process-client.mjs:539`.
pub const SET_MODEL_TIMEOUT: Duration = Duration::from_millis(15_000);

/// `session/close` deadline — `acp-process-client.mjs:552`.
pub const CLOSE_SESSION_TIMEOUT: Duration = Duration::from_millis(5_000);

/// How much of the child's stderr is retained — `MAX_STDERR_CHARS = 12_000`.
///
/// `acp-process-client.mjs:14`. The tail is kept, not the head: the last thing
/// a failing backend printed is the actionable part.
pub const MAX_STDERR_CHARS: usize = 12_000;

/// How long a process tree gets between `SIGTERM` and `SIGKILL`.
///
/// `acp-process-client.mjs:17` (`PROCESS_TREE_GRACE_MS = 750`). The value is
/// chosen to fit inside the Gateway's 2 s hard shutdown deadline with room for
/// adapter and logger cleanup afterwards, and that is the whole reason it is
/// 750 rather than a round second.
pub const PROCESS_TREE_GRACE: Duration = Duration::from_millis(750);

/// How often liveness is re-checked during that grace — `PROCESS_TREE_POLL_MS = 25`.
///
/// `acp-process-client.mjs:18`.
pub const PROCESS_TREE_POLL: Duration = Duration::from_millis(25);

/// Default `limit` for [`list_sessions`](crate::AcpProcessClient::list_sessions)
/// — `acp-process-client.mjs:415`.
pub const DEFAULT_SESSION_LIST_LIMIT: usize = 100;

/// Hard per-page cap on `session/list` — `Math.min(100, …)`.
///
/// `acp-process-client.mjs:426`. Observable to the agent: it is the largest
/// `_meta.limit` the Gateway will ever ask for, no matter what the caller
/// requested.
pub const SESSION_LIST_PAGE_CAP: usize = 100;
