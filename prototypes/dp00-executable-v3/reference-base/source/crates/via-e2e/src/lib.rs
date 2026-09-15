//! The end-to-end suite. **Everything is in `tests/`.**
//!
//! `docs/architecture.md` §15 ends phase 9 with *"`via-conformance` complete,
//! `via-e2e`"*. This crate is the second half of that: the product, driven the
//! way a consumer drives it — a process it spawned, a socket it opened, files
//! it read off disk afterwards.
//!
//! # What this crate is *not*
//!
//! `apps/via` already carries two suites, and neither is duplicated here:
//!
//! | | Drives | Proves |
//! | --- | --- | --- |
//! | `apps/via/tests/milestone.rs` | [`via::gateway::boot`], in-process | the phase-5 milestone: the fast path, the Work queue, the announcement window |
//! | `apps/via/tests/chat_client.rs` | `CARGO_BIN_EXE_via`, from inside the package | the client — autostart, the REPL's commands, the single-instance refusal |
//! | **`via-e2e`** | the **built binary**, from outside the package | the things only a process boundary can show |
//!
//! The last row is the whole justification. A green in-process milestone with a
//! binary that cannot reclaim a stale lease, or that writes a credential into
//! `gateway.log`, is a green test and a broken product.
//!
//! # The two seams, and why the harness Gateway exists
//!
//! Five of the seven cases drive the shipped `via` binary unmodified. Three of
//! them — the conversation, the cancellation and the restart — need a realtime
//! model and a backend, and a *shipped* binary has neither in a test: the
//! realtime providers it registers dial real endpoints, and Layer 3 resolves to
//! an ACP child process.
//!
//! So the suite ships one test-only binary, [`via-gateway-e2e`], which calls
//! [`via::gateway::serve`] — the same function `via gateway` calls, with the
//! same gate, the same lease, the same heartbeat, the same banner, the same
//! signal handlers and the same close sequence — and fills
//! [`via::gateway::Composition`]'s two seams from files on disk:
//!
//! | Seam | `via gateway` | `via-gateway-e2e` |
//! | --- | --- | --- |
//! | [`via::gateway::SessionOpener`] | a WebSocket to the provider | a `via-realtime-mock` script, read from `VIA_E2E_SCRIPT` |
//! | [`via_downstream::DownstreamAgent`] | an ACP child process | a [`ScriptedHarness`](via_downstream::testing::ScriptedHarness), read from `VIA_E2E_HARNESS` |
//!
//! Everything between and around them is production code, in a production
//! process: the real `via_realtime::RealtimeSession`, the real
//! `via_work::WorkManager` writing a real `tasks.json`, the real
//! `via_coordinator::Coordinator`, the real router, the real accept loop, the
//! real `via-lock` lease.
//!
//! [`via-gateway-e2e`]: ../../../crates/via-e2e/src/harness_gateway.rs
//!
//! # The seven cases
//!
//! | File | What only a process can show |
//! | --- | --- |
//! | `tests/first_run.rs` | what an empty machine ends up with, path by path and mode by mode |
//! | `tests/lease.rs` | a lease surviving an **unclean** death, and the `EPERM`/`ESRCH` split against the real kernel |
//! | `tests/conversation.rs` | the Work HTTP surface answering its catalogued shapes while a socket watches the same Work move |
//! | `tests/cancellation.rs` | `cancelling` observed *before* `cancelled`, across the process boundary |
//! | `tests/restart.rs` | SIGKILL, then `tasks.json` read back by a new process |
//! | `tests/security.rs` | a destroyed upgrade with no bytes on the wire, and a credential grep over the real `gateway.log` |
//! | `tests/locales.rs` | all three locales through the binary |

#![forbid(unsafe_code)]
#![deny(missing_docs)]
