# ADR-0001 — VIA is a standalone workspace; ARGO is a specification source

**Status:** accepted · 2026-08-22
**Context:** [`VIA_REFERENCE_ARCHITECTURE.html`](../../VIA_REFERENCE_ARCHITECTURE.html)
names `tiniffi` and `argo-pc` as VIA's host shells, which raised the question of
whether VIA should live inside the ARGO workspace.

## Decision

VIA is a standalone Cargo workspace at `~/Developer/VIA` (`crates/*` + `apps/*`).
It takes **no compiled dependency on ARGO** by default. ARGO is treated as a
*specification source* — code is ported deliberately, with attribution, under a
shared Apache-2.0 licence and shared authorship.

An optional, **non-default** `argo-bridge` feature may path-dep a sibling ARGO
checkout for `tinicore-traits` and `tinihost`. It ships only when a concrete
consumer exists.

A small number of **product-neutral seams** are contributed upstream to ARGO as
independent PRs on ARGO's own cadence. None is on VIA's critical path.

## The rule that decides any future case

> If the diff can be written without the word "VIA", it goes to ARGO.
> If it cannot, it stays in VIA.

## Options considered

### (a) New crates inside the ARGO workspace — rejected

1. **Cross-compilation is a hard blocker.** ARGO's `CLAUDE.md` requires every
   workspace crate to build cleanly for `aarch64`/`x86_64-linux-android`,
   `aarch64`/`x86_64-apple-darwin`, `x86_64-pc-windows-msvc` and
   `x86_64-unknown-linux-gnu`, and requires any new dependency to be pure Rust or
   cross-compile-clean on all four. VIA's shipping on-device path is
   `sherpa-onnx` + `llama-cpp-2` + `cpal` — three native C/C++ trees. Getting
   those through `cargo-xwin` and `cargo-ndk` on ARGO's Linux CI pool is a
   research project, not a build step.
2. **Naming.** ARGO's convention is `tini*` for engine libraries and
   `argo-<surface>` for surfaces. 25 `via-*` crates would be the first family
   that is neither — and `rebrand.md` has already fixed those names as product
   identity that cannot become `argo-*`.
3. **Blast radius.** 35 workspace members today, each PR running
   `cargo build/test/clippy --workspace` on a queue-constrained pool. Adding VIA
   makes every ARGO core edit a potential VIA red and vice versa.
4. **Version discipline.** ARGO's Group A is workspace-lockstep and
   `version-check.yml` fails any PR moving a crate version without a version-bump
   title. VIA is a Group B endpoint by nature.
5. **Repo identity.** ARGO's own `CLAUDE.md` calls a change requiring a
   simultaneous Product-repo edit "a red flag." Hosting a second product's entire
   gateway inside the Core repo inverts that principle rather than bending it.

### (b) Standalone, consuming ARGO as a dependency — rejected as written

There is nothing to consume. ARGO is explicitly not published to crates.io, has
zero internal git dependencies anywhere in its tree, and no `.gitmodules`. The
only working consumption pattern in existence is `argo-pc`'s — relative paths
*from inside the same repo*.

Taken literally, (b) means pinning a git SHA against an enterprise GitHub host
that every dev box and CI runner must authenticate to, against a pre-1.0
workspace whose `VERSIONING.md` says breaking changes "land freely."

It is also badly targeted. The parts VIA wants are narrow —
`tinicore_traits::live` (1,029 lines of event vocabulary), `openai_live.rs`
(2,704 lines of transport), `tinihost::{schedule,session,transport}` — but
reaching any of them through `tinicore` drags a 1,134-file crate with `rusqlite`,
`rquickjs` and `objectbox` in its graph into a product whose whole value
proposition is a small, testable gateway.

### (c) Split — accepted, weighted toward VIA

VIA owns everything VIA-branded. ARGO receives four or five product-neutral
seams. Knowledge crosses the boundary by porting, not by linking.

## What is ported from ARGO, and why porting is right

`tinicore/src/llm/providers/openai_live.rs` encodes four properties of the
litellm→Azure gateway that each cost a device run to discover, recorded in
`VOICE_AGENT_BRINGUP.md` §4:

| Property | What it broke |
| --- | --- |
| Frames the session as **Binary**, not Text | "connected but silent" |
| Requires `Sec-WebSocket-Protocol: realtime` | upgrade succeeded but was not routed |
| Speaks the **pre-GA** schema on GA's own `/v1/realtime` route | `Unknown parameter: 'session.type'` |
| Accepts a socket on routes it does not bridge | a 101 that produced no frames |

Plus the candidate-URL walk with a first-frame protocol probe, per-host schema
learning, and §8b's filter for the gateway's self-inflicted duplicate
`response.create`.

`via-realtime` ports that logic rather than writing from the qwen-audio-agent
JavaScript. `VOICE_AGENT_BRINGUP.md` §4 and §8b are the acceptance spec for its
conformance tests, and the `[live] dialect=` / `[live] schema=` log markers are
kept so the same one-glance diagnostics work.

The same applies, more loosely, to `tinihost::schedule`'s lease-based
restart-safe scheduler as the reference shape for `via-work`'s per-owner FIFO —
while noting `JobState`'s four variants are not VIA's nine, and
`SqliteScheduledJobStore` is not `via-store`'s versioned-JSON-with-quarantine.

Everything else in Layer 2 comes from the qwen-audio-agent port, because those
are the contracts VIA is judged against and ARGO has no equivalent for any of
them.

## What goes upstream to ARGO

Four narrow, product-neutral PRs, all pre-blessed by ARGO's own decision register
(`VOICE_VISION_REALTIME_PIPELINE.html` §10 decision ⓪):

1. `SpeechToText` / `TextToSpeech` streaming traits in `tinicore-traits`.
2. The cancellation-scope seam plus `PreResult::Detached` / `is_detachable_in_ctx`
   — this fixes a recorded defect where a cloned `CancellationToken` shares one
   `Arc<AtomicBool>`, so one barge-in kills detached background work.
3. `ProgressSink` / `ProgressTap` promotion into `tinicore`.
4. `DeviceKind::{Mic, Screen}` in `tinicore-traits/src/capability.rs`.

Optionally a fifth: `LiveSessionSink::interrupt`'s signature plus `item_id` on
`AudioChunk`, which `conversation.item.truncate` needs.

**Never upstream:** `work_id`, the TaskManager state graph, IntentSet,
ContextPack, FloorPolicy, InjectionGate, the wake word, `VIA_*` env vars, the
`via.gateway-lock/v1` schema, and the 707 contracts.

And do **not** put a `RealtimeSessionHost` into `tinihost` on VIA's behalf.
`tinihost` is the right home for *ARGO's* hosts, and that is ARGO's decision on
ARGO's schedule; making VIA the reason it grows a realtime loop welds two
products' timelines together for no gain, since VIA's session host is
`via-voice` / `via-app` regardless.

## Host shells

**Desktop:** there is nothing to preserve — `argo-pc` has no voice surface at
all. `via gateway` + `via chat` is the shell; `argo-pc` can attach to the same
HTTP/WS gateway later as one more client.

**Android:** the opposite — a complete voice session already exists at
`tiniffi/src/live_ffi.rs` (2,733 lines). Do **not** grow a second FFI surface
there for VIA. That would put a VIA dependency inside an ARGO workspace member,
reopen every objection to (a), and break `tiniffi`'s verified leaf status — the
property that currently lets Android PRs edit it without Core-maintainer review.

Instead, per ARGO's own decision ②: VIA's Gateway runs as a process and the
Android app is a WebSocket client. That also makes the protocol identical across
desktop, phone and server.

If a VIA shell inside ARGO is ever genuinely wanted, the sanctioned precedent is
`argo-pc`'s shape — a self-rooted `[workspace]` in a subdirectory with its own
`Cargo.lock` and `target/`, gated by a dedicated workflow. Adopt it with eyes
open: `argo-pc/CLAUDE.md` §1 records that root `cargo build/test/clippy
--workspace` and `cargo fmt --all` silently skip it, and `argo-pc-ci.yml`
records the incident where code moved into `argo-a2a/`, matched no `paths:` glob,
and a broken lane went green.

## Consequences

- VIA sets `rust-version = "1.94"` and pins `rust-toolchain.toml` to 1.94.0 to
  match ARGO, even though it needs only 1.89. Free, and removes a class of MSRV
  confusion if the bridge feature is ever built.
- Two live-event vocabularies will coexist: `tinicore_traits::live::LiveInboundEvent`
  and `via-protocol`'s server events. **Write the conversion module now**, inside
  the `argo-bridge` feature, with a round-trip test per event — while both are
  small and while the person who read both is still on the problem. A variant that
  cannot round-trip is a design signal worth recording in `fidelity.md`.
- VIA starts with none of ARGO's audit discipline. Land the cheap gates before
  the first feature crate: `via-arch-test`, plus ARGO's `panic_filter.py` /
  `unreachable_filter.py` / `risky_unwrap.py` copied verbatim (standalone Python,
  no coupling), plus a reverse product-leak gate asserting no `qwen`/`qwaudio`
  string survives outside `rebrand.md`'s KEEP list — and none `argo`/`tini`
  either.
- The `argo-bridge` feature will rot if nothing builds it. ARGO's own repo has
  this exact failure recorded: a `transcribe_input` field broke
  `argo-server --features live-api` across several commits without symptom,
  because the feature was non-default and nothing compiled it. Either build it in
  CI against a pinned ARGO SHA recorded in VIA's repo, or do not ship it.
