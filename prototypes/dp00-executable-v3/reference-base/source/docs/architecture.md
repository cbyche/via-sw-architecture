# VIA architecture

VIA — **V**oice **I**nteraction **A**gent — is a Rust voice-agent gateway with
**two upstreams**:

| Upstream | Supplies | Licence |
| --- | --- | --- |
| [`QwenAudio/qwen-audio-agent`](https://github.com/QwenAudio/qwen-audio-agent) v1.11.0 | Layer 2 — task lifecycle, coordination, delivery | Apache-2.0 |
| `~/Developer/ARGO` (`tinicore`, `tiniffi`) | Layer 1 — the realtime transport and its hard-won gateway knowledge | Apache-2.0, same authorship |

It is structured after **[`VIA_REFERENCE_ARCHITECTURE.html`](../VIA_REFERENCE_ARCHITECTURE.html)**
(2026-08-19), which is the target shape. This document is the reconciliation of
that target against what both upstreams actually contain, verified by reading
their sources on 2026-08-22.

Where VIA and an upstream differ, [`fidelity.md`](fidelity.md) records why. What
an external party can observe — the model, a backend agent, an HTTP/WS client, a
file on disk — is reproduced exactly.

---

## 1. What the reconciliation changed

The reference architecture marks boxes red for "does not exist yet." Verifying
that against ARGO produced four corrections to the original port plan, and each
one moves real work:

**Layer 1 is not a port target — it is a second upstream.** ARGO ships a
production realtime stack: `LiveProvider` / `LiveSessionSink` /
`LiveInboundEvent` in `tinicore-traits/src/live.rs` (1,029 lines), a 2,704-line
OpenAI/Azure provider with 57 tests, and a 2,733-line session host in
`tiniffi/src/live_ffi.rs` that has been brought up on a real device. Writing
VIA's realtime client from the qwen-audio-agent JavaScript would rediscover
every gateway bug ARGO already paid for. **`via-realtime` ports
`openai_live.rs`, not `providers/dashscope.mjs`.**

**The Injection Gate is the single highest-value thing the qwen port supplies.**
ARGO's bring-up doc lists it under "Absent by design — do not debug these":
*"Delegation results can land mid-sentence. Injection is queued the moment the
result exists. Holding it until neither side is speaking needs playback position
from the Kotlin side."* That is exactly qwen's announcement window — and qwen's
client protocol already carries the missing input, as
`playback.started` / `playback.ended` / `playback.cancelled`. The port supplies
both the algorithm and the protocol events that feed it.

**Layer 2's four red boxes are genuinely empty, and emptier than the diagram
suggests.** Beyond TaskManager / Scheduler / Task Store / Reminder Scheduler,
ARGO also lacks: a unified Work `kind` taxonomy, coalesced saves, quarantine on
corrupt, duplicate-submission suppression, terminal retention caps, and
progress-check announcements. All are catalogued qwen contracts.

**The Context Engine is bigger than "new."** Referents and ordered deixis have
**no implementation in either upstream** — qwen has no Context Engine at all,
and ARGO has nothing to extend. It gets its own phase rather than being folded
into another crate.

---

## 2. Session modes

A VIA session runs in exactly one of four modes. The mode is fixed for the
session's lifetime, chosen by the client at connect time, and it decides which
layers are even mounted — so it is a `via-protocol` type, not a runtime flag.

| Mode | Layer 1 | Layer 2 | Layer 3 | What it is |
| --- | --- | --- | --- | --- |
| `dictation` | capture + ASR | — | — | Transcribe and nothing else. No model turn, no tools, no speech out. |
| `direct` | full duplex | control tools only | — | The realtime model converses and answers **itself**. No delegation, no harness. |
| `agent` | full duplex | full Work queue | pluggable harness | The whole stack: fast-path answers plus delegated work. |
| `interface` | full duplex | Context Engine | — | Voice drives the **host's UI** through registered affordances rather than conversing. |

Three notes that make the enum load-bearing rather than decorative:

**`dictation` mounts no model at all.** It is the one mode where the realtime
provider may be a plain streaming ASR rather than a speech-to-speech model, which
is why `via-realtime`'s provider trait must not assume a model turn exists. On the
local pipeline this is the cheapest path by a wide margin — VAD plus ASR, no LLM,
no TTS.

**`agent` degrades to `direct` when no harness is configured**, rather than
failing. This mirrors upstream's `AGENT_PROTOCOL=none` frontend-only mode, and it
is what makes a fresh install useful before any backend is installed. The
degradation is reported on `/api/health`, never silent.

**`interface` is the Context Engine's reason to exist.** Referents and ordered
deixis (§5) are what turn "click that one" into an action against a concrete
on-screen object. In the other three modes the Context Engine assembles context;
in `interface` it also resolves targets.

The wire name is the lowercase variant (`"dictation"`, `"direct"`, `"agent"`,
`"interface"`); it appears in the `connect` client event, in `/api/health`, and
in every Work record so a stored Work can be read back without ambiguity.

---

## 3. Layer 1 — Realtime Frontstage

```
User / mic ──► Host shell ──► Voice Engine ──► [Injection Gate] ──► Voice out
               capture         FloorPolicy       may a finished
               playback        listen/speak      result be spoken
               PCM 24k         barge-in          now?
```

### Which upstream Layer 1 follows

**qwen-audio-agent is the implementation VIA follows.** `via-realtime` and
`via-voice` are ported from `server/src/voice/` — the provider interface, the
realtime gateway's session state machine, the tool-call handler, the response
guards, the announcement window, input arbitration, turn correlation. That is
the structure whose 707 contracts this port is judged against, so following it
is what makes the conformance suite mean something.

ARGO enters at **one specific, later point**: its
`tinicore/src/llm/providers/openai_live.rs` is a 2,704-line OpenAI/Azure client
that encodes four litellm-gateway properties which each cost a device run to
discover (Binary rather than Text frames; a mandatory
`Sec-WebSocket-Protocol: realtime`; a pre-GA schema served on GA's own
`/v1/realtime` route; sockets accepted on routes the proxy never bridges), plus
a candidate-URL walk with a first-frame probe and a filter for the gateway's
self-inflicted duplicate `response.create`.

That knowledge is worth having, and it lands as `via-realtime-openai` — **one
provider behind qwen's provider interface**, not a replacement for it. It is
phase 6 work. Nothing before phase 6 depends on it.

| | Source | Where |
| --- | --- | --- |
| The provider interface, session machine, voice layer | **qwen-audio-agent** | `via-realtime`, `via-voice` |
| The DashScope provider | qwen-audio-agent | `via-realtime-dashscope` |
| The OpenAI/Azure/litellm provider | ARGO | `via-realtime-openai` (phase 6) |

### What VIA adds

**`FloorPolicy` — provider-agnostic.** qwen tracks turn ownership across
`response-lifecycle.mjs`, `turn-correlation.mjs` and the provider's own state;
VIA names it. It must sit *above* the provider, because a provider may emit no
VAD events at all and a policy keyed on speech-start is then silently dead.

**The Injection Gate.** qwen's announcement window, whose blocking predicate is
`userSpeaking || turnPending || audioResponses.nonEmpty`, further gated by
`!sleeping && !waking && outputEnabled`. Its three inputs arrive as the
`playback.started` / `playback.ended` / `playback.cancelled` client events, which
is why porting qwen's WS vocabulary whole is load-bearing rather than
incidental. ARGO lists this gap under "Absent by design" — *delegation results
can land mid-sentence* — so the port closes a known hole rather than
reimplementing a solved problem.

**A live-provider registry** with alias resolution, capability flags and
per-provider session defaults, because VIA ships more providers than upstream's
two.

### The tool-count question

ARGO **deleted "direct mode" on 2026-08-04**: the realtime model no longer
authors tool calls, and gets exactly one tool, `request_delegation`, plus a
capability catalog derived from the tool registry. The reason was cost —
declaring 171 schemas is ~249 KB of `session.update`, charged every turn.

qwen gives its realtime model eight tools. These do not conflict, and VIA takes
both: the 171-schema problem was about exposing the *device tool registry*, not
about small control tools. VIA's Layer 1 declares **one delegation tool plus
seven Layer-2 control tools** — status, cancel, time, memory, notes, reminder,
permission-reply — none of which name a device capability. The capability
catalog stays the mechanism for "what kinds of work exist."

---

## 4. Layer 2 — Middleware & Coordination

This is where the qwen-audio-agent port lands almost whole, because ARGO has
essentially nothing here for the voice path.

### TaskManager

Authority for `work_id` and the nine-state lifecycle:

```
scheduled → queued → running ───────────────────────► completed
              │         └→ delegated → finalizing ──────┘
              └───────────────► cancelling → cancelled
                                          ↘ failed
```

ARGO's nearest is `tinihost::schedule::JobState` with **four** variants
(`Pending`/`Running`/`Done`/`Failed`). It has no `delegated`, no `finalizing`,
and critically no `cancelling` — so "cancelling is a state, not an action" has
no representation anywhere in ARGO today.

**One record, four kinds.** qwen keeps `work | reminder | scheduled_task |
control` in a single Work table with a `kind` discriminator. That is why one
`cancel_agent_task` cancels a reminder and a delegation alike. ARGO's equivalents
are three separate registries with three separate cancel paths. The unified
record is the single biggest structural improvement the port brings.

### Scheduler

Admission only: global cap, per-owner cap, **coordinator lane = 1**. ARGO's
`background::resources::ResourceManager::try_admit` has per-kind caps but no
per-owner dimension and no lane. The mechanism to reuse is ARGO's
`exclusive_lease` — a keyed `OwnedMutexGuard` acquired before dispatch and held
across it — which is well built and queues rather than corrupting.

### Task Store

Atomic write + rename, coalesced saves, quarantine on corrupt with the original
retained, health surface. ARGO's `background::TaskStateStore` has the atomic
write and the health surface but neither coalescing nor quarantine. On a phone
with a 500 ms progress cadence, uncoalesced writes are a real battery and flash
cost — so the generation-tagged deferred-write scheme is ported, not skipped.

Second-order behaviour worth keeping: if quarantine itself fails, **disable
persistence** rather than clobber.

### Reminder Scheduler

One `tokio::time::Sleep` re-armed to the earliest due time, recomputed on every
mutation; never polls; overdue work staggered on restart. ARGO has cron triggers
and timezone validation (`tinihost::schedule::Trigger` + `tinicore::cron`) but
they drive *chat* tools, not a voice-owned reminder surface.

### Coordinator

The reference architecture draws **IntentSet fan-out** — one utterance yielding
N intents — where qwen delegates a single objective string. ARGO's
`live_planner::plan` produces exactly one `DelegationIntent`, validated against a
strict schema with a namespace budget expanded into `AgentLoopConfig::allowed_tools`.

That validated-intent machinery is the right base. VIA widens `INTENT_SCHEMA` to
an array and returns `Vec<DelegationIntent>` — an extension of ARGO's planner,
not a replacement — and the Scheduler admits each intent independently.

### Query / Cancel policy

- **Direct answers** are answered on the fast path; no task is created.
- **Delegated work** is admitted by the Scheduler, or refused with a reason.
- **Cancellation is confirmed, not optimistic.** Work stays `cancelling` until a
  path confirms the stop. ARGO gets this right in exactly one place —
  `argo-a2a-delegation`'s requester/worker pair, including the overtaking case —
  and that discipline is the model for the fan-out version.

---

## 5. The Context Engine

Marked "OURS · NO QWEN EQUIVALENT" in the reference architecture, and the
verification agrees: this is the largest genuinely-new piece.

| Piece | Status |
| --- | --- |
| Prompt assembly | ARGO has it — `PromptBuilder::assemble() -> Vec<(PromptTier, String)>`, `<turn-context>` carrier |
| Memory | ARGO has it — fact store L1/L2, `MemoryManager`, plus qwen's `USER.md`/`MEMORY.md` |
| **ContextPack** | **new** — a typed object with per-section provenance, trust label and token cost |
| **Referents** | **new** — nothing to port from either upstream |
| **Ordered deixis** | **new** — consumes referents plus a screen source |
| Untrusted fencing | ARGO has three *unrelated* conventions covering three narrow slices |

Two design notes that follow from the verification:

**ContextPack is a thin typed layer, not a rewrite.** `PromptBuilder::assemble`
already produces ordered sections; VIA wraps them as
`ContextSection { id, source, trust, tier, tokens, body }` and has assembly emit
the pack. Rewriting prompt assembly would be re-deriving a working
implementation.

**Fencing needs a provenance dimension, not a fourth delimiter.** ARGO's
mechanism is correct and fail-closed; what is missing is *classification* — a
first-party tool that returns third-party bytes is currently trusted. The fix is
to separate content provenance from tool trust, not to invent another `<<< >>>`.

**Today the realtime model has strictly less context than a typed chat turn.**
It gets persona plus tool catalog and nothing else — `gather_delegation_context`
fires only *after* the model has already decided to delegate. The fast path is
where the Context Engine pays for itself.

---

## 6. Layer 3 — Backend Agent Execution

**VIA ships no agent harness of its own.** It does not port ARGO's agent loop, and
it does not reimplement one. Layer 3 is a single extension point —
`trait DownstreamAgent` — and every harness, ARGO's included, is a plugin behind
it.

That is a deliberate narrowing of the reference architecture's "five transports":
the five are not five *implementations VIA owns*, they are five *shapes a plugin
can take*.

```rust
#[async_trait]
pub trait DownstreamAgent: Send + Sync {
    fn descriptor(&self) -> &HarnessDescriptor;      // id, label, capability flags
    async fn open(&self, key: &SessionKey) -> Result<Box<dyn HarnessSession>>;
}

#[async_trait]
pub trait HarnessSession: Send + Sync {
    async fn prompt(&self, req: PromptRequest) -> Result<PromptOutcome>;
    fn events(&self) -> BoxStream<'_, SessionEvent>;  // adapter-normalised
    async fn cancel(&self, scope: CancelScope) -> Result<CancelOutcome>;
}
```

Sessions, events, cancellation and permissions come from Layer 2. A harness
answers prompts and emits events; it never owns a queue, a `work_id`, or a
permission decision.

| Shape | Who implements it | Status |
| --- | --- | --- |
| `acp` | `via-acp` — the qwen port | **VIA ships this.** One client covers 12 backends: OpenCode, OpenClaw, Qoder, Qwen Code, Kimi, Hermes, CodeBuddy, Codex, Claude Code, DeepSeek, Pi, generic ACP. |
| `cli` | `via-backends` launch specs | **VIA ships this.** A harness described by a command, args, and a stdio protocol. |
| `ipc` | plugin | A harness already running, reached over a socket. |
| `a2a` | plugin | ARGO's `argo-a2a-delegation` is the reference implementation, and the one place cancellation-as-state is done right. |
| `direct` | plugin | An in-process harness linked by an embedder. ARGO's `agent_loop` is *a* `direct` plugin — not VIA's implementation of one. |

### What "pluggable" has to mean to be real

A trait alone is not an extension point. Three things make it one:

**Capability flags are declared, and validated at registration.** A harness
declares whether it supports delegation, permissions, session resume, native
session history, and a web UI. A descriptor that is incomplete or internally
inconsistent is rejected at startup, not discovered mid-turn. This is qwen's
backend-driver contract, and it is worth porting exactly.

**Registration is data, not a code change.** A harness is named in configuration
by id; `via-backends` resolves the id to a launch spec. Adding a harness that
speaks ACP or a CLI protocol requires no VIA code at all.

**One dispatcher.** A new shape adds a `DownstreamAgent` impl, never a second
dispatch path. This is ARGO's own rule — *one orchestrator, one registry, one
hook manager* — and `via-arch-test` is VIA's enforcement of it. The failure it
prevents is the one ARGO's `CLAUDE.md` calls out by name: a reviewer finding
`fn dispatch_` in new code and having to ask why it isn't calling the existing
path.

### What Layer 2 keeps regardless of harness

Because these live above the trait, they behave identically no matter what is
plugged in: the `work_id` and its nine states; the per-owner FIFO and the
coordinator lane; the permission relay, modelled on ARGO's
`GatewayApprovalResolver` shape — `register(correlation_id)` *before* emitting,
then `wait(…)`, because that ordering is what closes the race; and the
adapter-normalised event vocabulary, which converges on ARGO's `EventListener`
shape rather than inventing a seventh, because that one already carries
sub-agent `depth`.

One recorded lesson to carry over: with nobody at a keyboard, an inline approval
`await` stalls the loop for up to 600 s. 19 of ARGO's 74 device tools are
approval-gated, and its `voice_relay` mode — auto-approve so the OS dialog is
the real prompt, deny shell at once with a speakable reason, relay `ask_user` as
a question the model asks aloud — is the answer VIA adopts.

## 7. On-device models

Unchanged from the first pass; the reconciliation only added a provider.

Three verified facts: **Qwen3.5-Omni has no open weights** (DashScope-API-only);
**no Rust runtime anywhere runs the Qwen3-Omni Talker** (checked against candle,
mistral.rs, llama.cpp's `mtmd_gen_audio_type`, and mlx-rs); and the official
`sherpa-onnx` crate now covers VAD, wake word, streaming ASR and TTS behind one
maintained interface. So "on-device omni" means Qwen3-Omni-30B-A3B, and the
shipping local path is componentized.

| Provider | Status | What it is |
| --- | --- | --- |
| `openai` | **port from ARGO** | The OpenAI/Azure/litellm realtime client, with its dialect walk and schema repair. The one that works today. |
| `dashscope` | port from qwen | Cloud Qwen realtime. |
| `mock` | new | Deterministic replay. Makes the whole gateway testable with no weights, no GPU, no network. |
| `local-omni:pipeline` | new — **the shipping on-device path** | 100% Rust: sherpa-onnx (Silero VAD → streaming ASR → Kokoro/Piper TTS) + `llama-cpp-2` on Qwen3 GGUF, audio input via `mtmd`. Runs on macOS arm64 today. |
| `local-omni:endpoint` | new — thin | Points the same client at `sgl-omni serve --enable-realtime` for true Qwen3-Omni S2S. CUDA-only; cannot be verified on this machine. |

**One bug not to port.** qwen's `resolveDashScopeRealtimeModelProfile()` returns
an all-capabilities-false profile for an unknown model id, and
`dashscope.mjs:84-87` gates `session.turn_detection` on
`transportCapabilities.audioInput` — so an unknown id opens a session that hears
nothing. A local model id is by definition unknown to that table. The local
family gets a real catalog entry, and the unknown-id fallback becomes an error.

---

## 8. Placement

**VIA is a standalone Cargo workspace at `~/Developer/VIA`. ARGO is a
specification source, not a dependency.** Full reasoning in
[`adr/0001-placement.md`](adr/0001-placement.md); the short version:

- **Not inside ARGO's workspace.** ARGO requires every crate to build cleanly for
  Android arm64/x86_64, macOS arm64/x86_64, Windows-MSVC and Linux-gnu. VIA's
  on-device path is sherpa-onnx + llama-cpp-2 + cpal — three native C/C++ trees.
  Cross-compiling those to Windows-MSVC and Android is a research project, not a
  build step. Add to that a crate-naming convention (`tini*` / `argo-*`) that 21
  `via-*` crates would break, and a workspace-lockstep version group VIA does not
  belong in.
- **Not a literal dependency either.** ARGO is not published, has no internal git
  deps, and no submodules. The only consumption pattern that exists is `argo-pc`'s
  relative paths *from inside the same repo*.
- **So: copy knowledge, not crates.** `openai_live.rs` is ported deliberately —
  same authorship, same licence — with ARGO's bring-up doc §4 and §8b as the
  acceptance spec for `via-realtime`'s conformance tests, keeping the `[live]`
  log markers so the same one-glance diagnostics work.
- **An optional, non-default `argo-bridge` feature** path-deps a sibling ARGO
  checkout for `tinicore-traits` and `tinihost`. It ships only when there is a
  concrete consumer: a non-default feature nothing compiles is worse than an
  absent one, because it advertises an integration that does not exist.
- **Four narrow seams go upstream to ARGO** as independent PRs on ARGO's cadence,
  all product-neutral and all pre-blessed by ARGO's own decision register:
  `SpeechToText`/`TextToSpeech` traits in `tinicore-traits`; the cancellation-scope
  seam plus `PreResult::Detached`; `ProgressSink`/`ProgressTap` promotion; and
  `DeviceKind::{Mic, Screen}`. None is on VIA's critical path.
- **Nothing VIA-branded ever goes upstream** — not `work_id`, the state graph,
  IntentSet, ContextPack, FloorPolicy, the wake word, `VIA_*` env vars, or the
  707 contracts.

**Android integration is a WebSocket client against `via gateway`**, per ARGO's
own decision register — not a second FFI surface in `tiniffi`. Growing one would
make an ARGO workspace member depend on VIA crates and break `tiniffi`'s verified
leaf status, which is what currently lets Android PRs edit it without
Core-maintainer review.

`rust-version = "1.94"` and a pinned `rust-toolchain.toml` to match ARGO, even
though VIA needs only 1.89 — it costs nothing and removes a class of future MSRV
confusion.

---

## 9. Crate graph

Upstream qwen enforces its layering with a test
(`dependency-boundaries.test.mjs`) asserting a fixed adjacency table. That table
is the honest source, because it is executable rather than aspirational:

```
shared        → ∅
core          → core, shared
process       → process, shared
agent         → agent, core, shared
conversation  → conversation, core, shared
task          → agent, core, task
voice         → conversation, core, shared, task, voice
app           → agent, app, conversation, core, task, voice
```

It also asserts that the generic ACP and process cores must not mention a
backend name. In JavaScript that needs a regex over source text; in Rust it is a
crate boundary the compiler enforces.

`Cargo.toml`: `resolver = "3"`, `members = ["crates/*", "apps/*"]`,
`edition = "2024"`, `rust-version = "1.94"`.

| Band | Crates |
| --- | --- |
| **Leaf** | `via-protocol` · `via-catalog` · `via-log` · `via-store` · `via-lock` · `via-audio`* |
| **Core** | `via-core` · `via-i18n`* |
| **Layer 1** | `via-realtime` · `via-realtime-openai`* · `via-realtime-dashscope` · `via-realtime-local`* · `via-realtime-mock`* · `via-wake-word` · `via-voice` |
| **Layer 2** | `via-work` · `via-coordinator`* · `via-context`* · `via-conversation` |
| **Layer 3** | `via-downstream`* · `via-acp` · `via-backends` · `via-process` · `via-mcp-tools` |
| **App** | `via-app` · `apps/via` |
| **Tests** | `via-conformance` · `via-arch-test` · `via-e2e` |

\* = no upstream counterpart in either project.

**25 shipped crates plus three test-only gates, and one binary.** The four
additions over the first pass —
`via-realtime-openai`, `via-coordinator`, `via-context`, `via-downstream` — are
exactly the boxes the reference architecture draws that the qwen port alone
would not have produced.

`via-acp` and `via-process` do not depend on `via-backends`. `via-context` sits
beside `via-conversation` rather than inside it, because referents and memory
have different lifetimes.

---

## 10. The `via` binary

| Command | What |
| --- | --- |
| `via gateway` | Run the Gateway in the foreground |
| `via chat` | Text client — the end-to-end harness, no audio, no weights |
| `via config` | Show / edit `config.env` |
| `via backend <install\|status\|auth>` | Backend lifecycle |
| `via mcp-serve` | stdio MCP server exposing the five coordination tools |
| `via service <install\|start\|stop>` | launchd / systemd unit |

Node's `npm install -g`, the `npx` shims and `scripts/*-acp.mjs` disappear: a
Rust binary is its own installer, and the shims become `via-backends` launch
specs. Where a backend genuinely *is* an npm package, VIA keeps resolving that
exact coordinate.

---

## 11. Concurrency — the four invariants

Each gets an owning task, not a mutex: `tokio::sync::Mutex` gives mutual
exclusion but not FIFO order, and three of the four need order. State held by
value in one task, `enum Command { … reply: oneshot::Sender<R> }` over a bounded
`mpsc`, shutdown by dropping senders, registered on a shared `TaskTracker`.

1. **Per-owner Work FIFO.** One item per owner inside the backend session at a
   time. ARGO's current answer is a `tokio::Mutex` serialising delegations —
   correct for exclusion, silent about order.
2. **The keyed serial executor.** Both the Gateway queue *and* the ACP adapter
   serialize session writes. The double guard is deliberate; porting one of the
   two looks correct until it is under load.
3. **The announcement window.** `!userSpeaking && !turnPending &&
   audioResponses.is_empty()`, gated by `!sleeping && !waking && outputEnabled`.
   Retries bounded; delivered marked only *after playback finishes*.
4. **Delegation correlation.** Only the completion correlated to that delegation
   id may complete the Work. The request timeout applies to the coordinator turn
   and the presentation turn — **not** while waiting on the delegated session.

One recorded ARGO defect Layer 2 must design around: a cloned
`CancellationToken` shares one `Arc<AtomicBool>`, so a detached sub-agent
captures the parent turn's token and **one barge-in kills independent background
work**. VIA's Work items own their own cancellation scope.

---

## 12. Protocol dependencies

**ACP** — `agent-client-protocol` 2.0.0 (Zed's official SDK) with
`agent-client-protocol-schema` 1.7.0, targeting **stable v1**. Protocol v2 is
draft; stay off it. Two caveats: the crate is runtime-agnostic and starts a smol
reactor beside tokio, and its teardown goes straight to SIGKILL after a fixed
1 s grace — so children are wrapped in `process-wrap` 9.1.0 for a real
SIGTERM → wait → SIGKILL ladder, which also fixes Windows grandchild containment.

**MCP** — `rmcp` 3.1.4. Skip `agent-client-protocol-rmcp`: it pins rmcp 2.x.
Transport is qwen's loopback HTTP (`POST /mcp` on `127.0.0.1:0`, `Bearer` token,
404 on anything else) with `via mcp-serve` over stdio for backends that need it.

**Server runtime** — axum 0.8, tower-http, tokio-tungstenite, rustls. Install the
rustls crypto provider on line one of `main()`; if `ring` and `aws-lc-rs` both
land in the graph, `ClientConfig::builder()` panics at runtime. `cargo tree -i ring`
belongs in CI.

---

## 13. Identity

The rule: **no qwen, except the model and the frontend voice engine's qwen
runtime.** 198 strings classified — 133 rename, 65 keep. Full table in
[`rebrand.md`](rebrand.md).

Rename what is ours: `qwen-audio-agent` → VIA, `QWEN_AUDIO_AGENT_*` → `VIA_*`,
`~/.config/qwaudio` → `~/.config/via`, `QWAUDIO_*` → `VIA_*`,
`qwaudio.gateway-lock/v1` → `via.gateway-lock/v1`,
`qwen_audio_agent_session_*` → `via_session_*`,
`<qwen_audio_agent_backend_instructions>` → `<via_backend_instructions>`.

Keep what is not: DashScope model ids and voice ids, `DASHSCOPE_*`, the
`dashscope` provider key **and its `qwen` alias**, the Qwen Code backend and its
`QWEN_CODE_*` namespace, all ACP and MCP wire methods.

Two renames carry real risk: **the five MCP tool names** (the permission broker
auto-approves them via *three* name-shape matches — exact, `endsWith("__<name>")`,
`startsWith("<name> (")` — so a partial rename wedges the coordinator), and
**the wake word**, which needs a regenerated sherpa-onnx keyword model per locale.

Because ARGO is now an upstream, VIA also needs the **reverse** gate: nothing
`argo`- or `tini`-branded may leak into VIA's product strings.

---

## 14. Fidelity policy

- **Copied verbatim, byte for byte**, with a test: the `zh` `PROMPT.md` and
  `ASSISTANT.md`, `openclaw.json5`, `cordis.yml`, the CodeBuddy model table.
- **Reproduced verbatim in code**, with tests: every tool name, description and
  JSON Schema; `backend-agent-instructions`; every error code; the backend
  environment allow-lists; the `/api/health` field order; the
  configuration-signature hash input, whose field order is JS insertion order and
  which a `HashMap` will not reproduce.
- **Ported from ARGO as a specification**, with its bring-up doc as the
  acceptance spec: the dialect walk, the first-frame probe, GA/pre-GA dual
  schema, and the duplicate-`response.create` filter.
- **Adapted / dropped, with the reason recorded** in `fidelity.md`.

The acceptance criteria are [`reference/contracts.md`](reference/contracts.md):
**707 contracts**, machine-readable in `reference/contracts.json`, asserted by
`via-conformance`.

VIA also inherits ARGO's cheap discipline from day one — `via-arch-test` plus
ARGO's `panic_filter.py` / `unreachable_filter.py` / `risky_unwrap.py`, which are
standalone Python with no ARGO coupling. A 40–55k-line port written without them
will accrete exactly the drift they exist to catch.

---

## 15. Delivery plan

| # | Phase | Ends when |
| --- | --- | --- |
| 0 ✓ | Workspace, CI, gates, leaf crates | `via-arch-test` green; contract constants asserted |
| 1 ✓ | `via-core` + `via-i18n` + setup gate + binary skeleton | `via gateway` refuses to start unconfigured with the exact message in all three locales |
| 2 ✓ | `via-acp` + `via-backends` + `via-process` + `via-downstream` | a scripted fake ACP agent completes a prompt turn through the trait |
| 3 ✓ | `via-work` + `via-coordinator` + `via-mcp-tools` | delegation, cancellation and permission relay pass their ported tests |
| 4 ✓ | `via-conversation` | memory and notes resolution pass, including every ambiguity and destructive-intent case |
| 5 ✓ | `via-realtime` + `mock` + `dashscope` + `via-voice` + `via-app` | **`via chat` works end to end** — no audio hardware, no model weights |
| 6 ✓ | `via-realtime-openai` + `via-realtime-s2s` + `via-wake-word` | every provider qwen ships plus the ARGO-sourced one; wake-word pipeline runs |
| 7 ✓ | `via-context` | ContextPack, referents, ordered deixis, provenance fencing — 141 tests |
| 8 ✓ | `via-realtime-local` — `pipeline`, then `endpoint` | on-device voice runs the componentized pipeline end to end on macOS arm64 — 199 tests |
| 9 ✓ | `via-conformance` complete, `via-e2e` | every one of the 26 crates is in scope (`Crate::exists_today` is `true` everywhere); `via-e2e` drives the built binary through processes, sockets and the filesystem — 25 tests. **Not** "all 707 contracts asserted": that phrase was always aspirational — see `docs/fidelity.md`'s coverage paragraph for the honest locked/behavioural/pending split, which this phase moved substantially but did not close to zero. |

**All nine phases have landed** — see `docs/fidelity.md`'s coverage paragraph
and `README.md`'s status line for the measured crate, test and coverage
counts. Landing a phase here means its crate(s) ship with tests passing, not
that every catalogued contract touching it is asserted; `docs/deviations/README.md`
is the honest index of what still isn't.

Two adjustments from the first pass. `via-realtime-dashscope` moved **up** into
phase 5 — qwen's realtime gateway is meaningless without the provider it was
written against, so following that implementation means shipping them together.
And `via-realtime-openai`, the ARGO-sourced client, stays in phase 6 alongside
`via-realtime-s2s` — both are providers behind the same qwen-shaped interface,
and nothing before phase 6 needs either.

---

## 16. Decisions taken

### Locale — English default, plus Chinese and Korean

Three locales: `en` (default), `zh`, `ko`. ARGO already ships `en`/`ko` with
identical key trees, so `ko` is not new ground; `zh` is upstream qwen's own text.

| Kind | Locales | Fidelity |
| --- | --- | --- |
| Ported | `zh` is upstream's own text, byte for byte | `via-conformance` asserts it against upstream; the 76 prompt-text contracts stay comparable |
| Authored | `en` and `ko` peers | structural parity — same headings, tool-name references, `{{variable}}` set, rule count |

`assets/frontend-agent/{en,zh,ko}/{PROMPT,ASSISTANT}.md`, plus a `via-i18n`
catalog for runtime strings. **Error codes are not localized** — only the message
beside them. Locale resolves `VIA_LOCALE` → OS locale → `en`; the spoken language
still follows the user's own utterance and their `USER.md` preference.

### On-device — `local-omni:pipeline` first

The 100% Rust componentized path first, because it runs on the dev box and can
be exercised. `local-omni:endpoint` follows and ships unverified-on-hardware,
saying so.

### Wake word — deferred to phase 6

The phrase is a configuration value, never a literal; the keyword model is a
downloadable checksummed artifact resolved per phrase; and **wake word is
per-locale**, so three locales may mean three models.

---

## 17. Review checklist

1. Can the voice layer still converse while backend work is queued or running?
2. Does every executable request enter the same persistent backend session?
3. Did any client-facing API gain knowledge of session, subagent, permission, or
   execution mode?
4. Are tool events used only for generic progress?
5. Is completion spoken only from a final backend result, **through the Injection
   Gate**?
6. Can interruption postpone speech without cancelling submitted Work — and
   without cancelling *detached* work?
7. Do tests cover FIFO serialization, fixed session reuse, tool animation, and
   delivery retry?
8. Does `via-acp` or `via-process` now depend on `via-backends`? *(It must not.)*
9. Did a child process spawn without `.env_clear()`? Rust's inherit-by-default
   `Command` breaks the credential boundary silently.
10. Did a new transport add a **second dispatcher** instead of a
    `DownstreamAgent` impl?
11. Did anything VIA-branded leak into an upstream PR to ARGO?
