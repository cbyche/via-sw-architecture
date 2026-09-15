# Phase 9 deviations — `via-e2e`

The end-to-end suite: the built `via` binary driven the way a consumer drives
it. Read [`phase-0.md`](phase-0.md) for what a deviation record is and is not.

## `via-e2e`

*25 tests across 7 integration binaries · one test-only binary · `publish = false`
· clippy clean · 9 hand-applied mutants, all killed · 9 deviations*

The suite spawns **processes**, opens **sockets**, and reads **files** back off
disk. `apps/via` already owns the two suites that do not need a process
boundary — `tests/milestone.rs` (the phase-5 milestone, in-process) and
`tests/chat_client.rs` (the client, from inside the package) — and neither is
duplicated here.

| File | Tests | What only a process can show |
| --- | --- | --- |
| `tests/first_run.rs` | 3 | every path an empty machine ends up with, audited against the `file-path` rows, plus its mode |
| `tests/lease.rs` | 3 | a lease surviving an **unclean** death, and both arms of `kill(pid, 0)` against the real kernel |
| `tests/conversation.rs` | 3 | the five Work routes serializing their catalogued shapes to a client that never linked `via-work` |
| `tests/cancellation.rs` | 2 | `cancelling` observed **before** `cancelled`, and a reminder cancelled through the identical endpoint |
| `tests/restart.rs` | 3 | SIGKILL, then `tasks.json` read back and re-published by a different process |
| `tests/security.rs` | 3 | a destroyed upgrade with **zero bytes** on the wire, a forged cookie refused, and a credential grep over the real `gateway.log` |
| `tests/locales.rs` | 8 | five surfaces × three languages, through the binary |

---

### 1. A test-only Gateway binary exists — `via-gateway-e2e`

**The deviation.** Five of the seven cases drive the shipped `via` binary
unmodified. Three of them — the conversation, cancellation across the process
boundary, and restart recovery — need a realtime model and a backend, and the
shipped binary has neither in a test: its realtime providers dial real
endpoints and Layer 3 resolves `AGENT_PROTOCOL` to an ACP child process.

**Why not substitute in-process.** That is what
`apps/via/tests/milestone.rs` already does, and it is the right shape for what
that file asserts. Repeating it here would produce a second copy of a suite
that already exists and would prove nothing new about `tasks.json`, the lease,
the signals or the accept loop.

**What the binary is.** `crates/via-e2e/src/harness_gateway.rs` calls
[`via::gateway::serve`] — the same function `via gateway` calls — after the
same steps in the same order: sample the environment, load
`.env.local` / `.env` / `config.env` and scaffold, resolve the configuration,
run the setup gate, take the lease, compose, bind, publish the origin, print
the banner, install the signal handlers, serve, close. It differs from the
product in `Composition`'s **two seams and nothing else**:

| Seam | `via gateway` | `via-gateway-e2e` |
| --- | --- | --- |
| `via::gateway::SessionOpener` | `ConnectOpener` — a WebSocket to the provider | a `via-realtime-mock` `Script`, read from `VIA_E2E_SCRIPT` |
| `via_downstream::DownstreamAgent` | an ACP child process | a `ScriptedHarness`, read from `VIA_E2E_HARNESS` |

Both are **files on disk** rather than Rust values, because the process that
authors the fixture is not the process that runs it. The logger is built
identically to the product's, including the `is_test_process` check, which is
what makes `logs/gateway.log` a real file `tests/security.rs` can grep.

The bin target lives at `src/harness_gateway.rs` rather than `src/main.rs` or
`src/bin/`: the name says what it is, and a file directly under `src/` adds no
new directory, so `scripts/risky_unwrap.py`'s `SUBSYSTEM_DIRS` /
`KNOWN_NON_ADVERSARIAL_DIRS` tables need no entry. It is held to the production
panic budget regardless — no `unwrap`, no `expect`, no `panic!`.

[`via::gateway::serve`]: ../../apps/via/src/gateway/mod.rs

### 2. The shipped binary is found by path, not by `CARGO_BIN_EXE_via`

**The deviation.** `apps/via`'s own tests use `env!("CARGO_BIN_EXE_via")`.
`via-e2e` cannot: cargo sets that variable only for integration tests of the
package that *declares* the bin, and that package is `apps/via`.

**What is done instead.** `support::via_binary()` resolves
`<target>/<profile>/via` from `env!("CARGO_BIN_EXE_via-gateway-e2e")` — which
*is* exact, because that bin belongs to this package — and runs
`cargo build --package via --bin via` once if it is not there. Cargo releases
the target-directory lock before running tests, which is why `escargot` and
`trycmd` work the same way. Without the fallback, `cargo test -p via-e2e` on
its own would fail for a reason unrelated to the product, and a suite that
fails for the wrong reason is not a gate.

### 3. `via` and `via-conformance` are path dependencies, not workspace ones

**The deviation.** Every other internal edge in the workspace is
`{ workspace = true }`. These two are `{ path = … }`.

**Why.** The root manifest's `[workspace.dependencies]` has rows for the 25
shipping crates and no row for `via` or for the test-band crates. Adding either
would put an App-band binary and a Tests-band crate into the table every
shipping crate inherits from, for one consumer. `via-arch-test` reads real
`cargo metadata` edges, so a path dependency is checked by exactly the same
rules — and `crate_graph.rs` already names this edge by name: *"a test crate
must be able to depend on another test crate — `via-e2e` on
`via-conformance`."*

**Reported**: the root `Cargo.toml` was **not** edited.

### 4. The *unrecoverable delegated work* restart sentence is unreachable from `via gateway`

**The finding, and it is a defect rather than a decision.**
`docs/reference/contracts.json` carries two restart force-fail sentences:

| Contract | Set by |
| --- | --- |
| *restart force-fail error (interactive work)* | `Actor::restore` — any active Work that cannot be recovered |
| *restart force-fail error (unrecoverable delegated work)* | `Actor::recover_delegated` — a `delegated` Work that **was** addressable and whose recovery declined |

`WorkManager::recover_delegated` documents itself as *"Call once, at
composition: the candidate list is drained"*, and **nothing in `apps/via` calls
it**. So a `delegated` Work with both ids is restored as `queued`, added to
`recovery_candidates`, and left there — never re-attached, never force-failed,
and never carrying the second sentence.

`tests/restart.rs` asserts what the product *does* — an active Work is
force-failed with the interactive sentence, byte for byte against the
catalogue, with `notificationStatus: "pending"` — and
`the_two_restart_reasons_stay_distinct` pins the pair so a build cannot collapse
them. The gap itself belongs to `apps/via`: the fix is one
`work.recover_delegated(runner).await` at composition, beside
`configure_scheduled_task_runner` and `configure_coordinator_query`.

### 5. `logs/cli.log` is created and is not catalogued

`tests/first_run.rs` audits every path a first run creates against the three
`file-path` rows that enumerate them, and one entry has no row: `logs/cli.log`.
It is the **CLI's** log file — `cli/bin/<binary>.mjs` builds its logger with
`fileName: 'cli.log'`, which `apps/via` reproduces — and the survey catalogued
only the Gateway's `logs/gateway.log`. The audit carries it as a single named
exception, derived from `via::LOG_COMPONENT` rather than spelled, so a renamed
component moves the exception with it. Everything else the product creates is
named by a contract.

### 6. The upgrade refusal's header names are lower-case

The catalogued refusal is
`HTTP/1.1 401 Unauthorized\r\nConnection: close\r\nContent-Type: text/plain\r\n\r\nidentity required`.
Upstream writes those bytes onto the raw socket; VIA builds a response and hyper
serializes it, lower-casing the header names and adding `content-length` and
`date`. That deviation is already recorded in
[`phase-5-via-app.md`](phase-5-via-app.md); `tests/security.rs` compares the
header names case-insensitively and everything a client parses — the status,
the connection close, the media type and the body — exactly.

### 7. The log directory is **not** redirected

`apps/via`'s fixture points `VIA_LOG_DIR` at a scratch directory.
`via-e2e`'s does not, because `<config>/logs/gateway.log` is itself a catalogued
path: `tests/first_run.rs` audits it and `tests/security.rs` greps it.
Redirecting it would make both assert against a location the product never uses.

### 8. The mutation check is hand-applied, and it runs in a copy of the tree

**The deviation.** The other crates report `cargo mutants` scores over their own
source. `via-e2e` has almost none: one test-only binary and seven `tests/`
files. Mutating *this* crate would measure nothing, because the code under test
is the other twenty-five crates and the binary.

So the check is nine mutations applied by hand to the **product**, each flipping
one decision that exactly one case claims to protect:

| # | Mutation | Killed by |
| --- | --- | --- |
| 0 | `SignalOutcome::is_alive` reads `EPERM` as dead | `tests/lease.rs` |
| 1 | `destroys_socket` always answers `false` | `tests/security.rs` |
| 2 | the implicit same-origin path drops its loopback restriction | `tests/security.rs` |
| 3 | `resolve_token` stops comparing the signature | `tests/security.rs` |
| 4 | `restore()` uses the *delegated-lost* sentence for interactive work | `tests/restart.rs` |
| 5 | a `running` Work is cancelled without passing through `cancelling` | `tests/cancellation.rs` |
| 6 | `project_timeline` sorts by `createdAt` rather than `completedAt` | `tests/conversation.rs` |
| 7 | `FILE_MODE` becomes `0o644` | `tests/first_run.rs` |
| 8 | the deliberately-un-localized 404 body gets a `zh` translation | `tests/locales.rs` |

**All nine were killed; no survivors.**

It runs against a `cp -R` of the workspace under `CARGO_TARGET_DIR` of its own,
because concurrent agents build the real tree and a mutation left in a shared
crate for the length of one `cargo test` would break *their* build, not just
this suite's. The script is not committed: it is a one-shot audit, not a gate,
and a committed script that mutates `crates/` is a footgun.

### 9. Nothing is `#[ignore]`d

Every case runs by default. The slowest — the full conversation — is bounded by
the announcement window's own cadence rather than by anything the suite does,
and the whole suite is a couple of minutes on a warm build. A suite everyone
skips is not a gate, and none of these cases needs a network, a credential, an
audio device, a model, or a backend binary.

**One thing the suite had to learn the hard way**, and it is the phase-6 lesson
again: the banner read in `support::Machine::start` was originally unbounded and
matched the `en` banner only. A Gateway started under `VIA_LOCALE=zh` prints a
Chinese banner, so the read blocked forever and the suite hung instead of
failing. It now matches all three locales *and* carries `START_BUDGET`, after
which the child is killed and the refusal is reported. Every other wait in the
suite is bounded the same way: `wait_until` and `Socket::wait_for` by their
caller's budget, the SSE read and the raw-socket exchange by their own, and
every HTTP request and WebSocket upgrade by `REQUEST_BUDGET` — the case neither
of the others covers, a Gateway that accepts the connection and then answers
nothing at all.

`FILE_BUDGET` is the one budget that is **derived** rather than chosen:
`via_work::store::DEFERRED_DELAY.saturating_mul(80)`. A test waiting on
`tasks.json` is waiting on the coalescing timer plus a real `write` + `rename`
in another process, and a hand-picked number would silently stop being a margin
the day that delay changed.
