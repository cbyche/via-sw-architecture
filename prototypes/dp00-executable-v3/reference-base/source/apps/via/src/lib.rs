//! VIA's command surface.
//!
//! `docs/architecture.md` §10 names six verbs, and this crate is all six of
//! them:
//!
//! | Command | As of phase 5 |
//! | --- | --- |
//! | [`via gateway`](commands::gateway) | **complete** — boots [`gateway`], holds and heartbeats the lease, serves, and closes on SIGINT/SIGTERM |
//! | [`via chat`](commands::chat) | **complete** — [`chat`], the port of upstream's `tui/src/text-cli.mjs` |
//! | [`via config`](commands::config) | **complete** |
//! | [`via backend`](commands::backend) | argument surface real, body refuses |
//! | [`via mcp-serve`](commands::mcp_serve) | argument surface real, body refuses |
//! | [`via service`](commands::service) | argument surface real, body refuses |
//!
//! # The rule the remaining stubs follow
//!
//! **An unimplemented command exits non-zero and names the phase that
//! implements it.** A stub that exits `0` is worse than no stub at all: a
//! script that treats success as success would then be silently wrong, and the
//! failure would surface somewhere else entirely. [`phase::Unimplemented`] is
//! the one type that produces those refusals, so the rule is enforced in one
//! place rather than remembered three times.
//!
//! # The two milestones this binary carries
//!
//! `docs/architecture.md` §15 ends **phase 1** when *"`via gateway` refuses to
//! start unconfigured with the exact message in all three locales"*. That path
//! is [`commands::gateway::run`], the message is
//! [`via_core::SetupStatus::refusal_message`], and `tests/setup_gate.rs` runs
//! the built binary once per locale to prove it. Every one of those refusals
//! survived phase 5; what changed is what happens when none of them fires.
//!
//! §15 ends **phase 5** when *"`via chat` works end to end — no audio hardware,
//! no model weights"*. `tests/milestone.rs` boots a Gateway through
//! [`gateway::boot`] with the mock realtime provider and a scripted harness,
//! sends one message the model answers itself and one that delegates, and
//! follows the Work to `completed` and back into the conversation through the
//! announcement window. `tests/chat_client.rs` asserts the same client as a
//! process. Deviations are recorded in `docs/deviations/phase-5-apps-via.md`.
//!
//! # Fidelity
//!
//! The binary name, the exit codes, the stderr prefix, the flag surface, the
//! `config.env` rewrite and the `config show` output are external contracts
//! catalogued in `docs/reference/contracts.json`; each carries a doc comment
//! naming the upstream file it came from. VIA's verb set is *not* upstream's —
//! §10 replaces `tui`/`webui`/`status`/`setup`/`install`/`skill` with `chat`,
//! `backend` and `service` — and every difference is recorded in
//! `docs/deviations/phase-1.md`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod chat;
pub mod cli;
pub mod commands;
pub mod configfile;
pub mod error;
pub mod gateway;
pub mod host;
pub mod origin;
pub mod phase;
pub mod session;

pub use error::{CliError, EXIT_FAILURE, EXIT_SUCCESS};
pub use host::Host;
pub use phase::{Phase, Unimplemented};

/// The binary's name, and the token every message and hint embeds.
///
/// **External contract** — `docs/reference/contracts.json`
/// *binary-name*, `cli/package.json:7`. Upstream's value is renamed by
/// `docs/rebrand.md` row 136; the rename is the whole of the change, and
/// `tests/contracts.rs` derives this constant from the catalogue rather than
/// comparing two hand-typed strings.
pub const BINARY_NAME: &str = "via";

/// The prefix an uncaught failure is written to stderr with.
///
/// **External contract** — `docs/reference/contracts.json` *error-code/stderr
/// prefix*, `cli/bin/<binary>.mjs:29`: `` `<binary>: ${error.message}\n` ``.
pub const STDERR_PREFIX: &str = "via: ";

/// The `component` field of every record this binary logs, and the file it
/// writes.
///
/// **External contract** — `cli/bin/<binary>.mjs:6-10`, which creates the
/// logger with `component: 'cli'`, `fileName: 'cli.log'` and
/// `consoleEnabled: false`. The console sink stays off because stdout is the
/// command's *output*: a log line interleaved with `via config`'s answer would
/// corrupt the answer for whatever is reading it.
pub const LOG_COMPONENT: &str = "cli";

/// The `via.log/v1` event names the binary emits, in the order one run
/// produces them.
///
/// **External contract** — `cli/bin/<binary>.mjs:12,18,24`.
pub const LOG_EVENTS: [&str; 3] = ["cli.started", "cli.completed", "cli.failed"];
