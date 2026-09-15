# Fidelity: what is ported, adapted, or dropped

This port aims to be exact where it matters — everything the model, a backend
agent, or an external client can observe — and idiomatic where it does not.
This document records every place the two implementations differ, so a reader
can trust the rest.

Upstream reference: `QwenAudio/qwen-audio-agent` v1.11.0, surveyed 2026-08-22.
Upstream is Apache-2.0; its licence is preserved as
`LICENSE.qwen-audio-agent`.

---

## Per-phase deviation records

Policy lives here; the receipts live per phase.

| Phase | Record | Deviations | Tests |
| --- | --- | ---: | ---: |
| 0 — workspace, gates, leaf crates | [`deviations/phase-0.md`](deviations/phase-0.md) | 77 | 475 |
| 1 — core, i18n, CLI | [`deviations/phase-1.md`](deviations/phase-1.md) | 42 | 578 |
| 2 — Layer 3: seam, ACP, process, backends | [`deviations/phase-2.md`](deviations/phase-2.md) | 52 | 660 |
| 3 — Layer 2: Work, MCP tools, coordinator | [`deviations/phase-3.md`](deviations/phase-3.md) | 39 | 560 |
| 4 — conversation: memory, notes, extractor | [`deviations/phase-4.md`](deviations/phase-4.md) | 24 | 254 |
| 5 — voice: realtime, providers, gateway, chat | [`phase-5-apps-via.md`](deviations/phase-5-apps-via.md) · [`phase-5-via-app.md`](deviations/phase-5-via-app.md) · [`phase-5-via-realtime-dashscope.md`](deviations/phase-5-via-realtime-dashscope.md) · [`phase-5-via-realtime-mock.md`](deviations/phase-5-via-realtime-mock.md) · [`phase-5-via-realtime.md`](deviations/phase-5-via-realtime.md) · [`phase-5-via-voice.md`](deviations/phase-5-via-voice.md) | 119 | 1193 |
| 6 — providers: OpenAI/Azure, wake word, conformance | [`phase-6-via-realtime-openai.md`](deviations/phase-6-via-realtime-openai.md) · [`phase-6-via-wake-word.md`](deviations/phase-6-via-wake-word.md) | 47 | 395 |
| 7 — the Context Engine | [`deviations/phase-7.md`](deviations/phase-7.md) | 11 | 141 |
| 8 — on-device voice | [`deviations/phase-8-via-realtime-local.md`](deviations/phase-8-via-realtime-local.md) | 21 | 199 |
| 9 — conformance close-out, `via-e2e` | [`deviations/phase-9-via-e2e.md`](deviations/phase-9-via-e2e.md) | 9 | 25 |

[`deviations/README.md`](deviations/README.md) sorts every one of these 454
deviations across all sixteen files into three lists — Corrections (49, VIA
does better), Divergences (375, VIA does differently on purpose) and Gaps (30
historical entries, plus the live pending-contract list that supersedes them)
— which is the fastest way to read this whole record without opening sixteen
files.

Three phase-0 findings are worth promoting, because they change what a later
phase should expect:

**`WorkState` has five variants, not four.** Upstream computes it as
`ACTIVE.has(status) ? 'active' : status` (`task-manager.mjs:58`), and `scheduled`
is not in `ACTIVE` — so a scheduled Work publishes `workState: "scheduled"`
verbatim. `contracts.json` records five; the brief that went to the crate said
four. The contract won, which is the right precedence.

**The file lock is mkdir-based, deliberately.** `std::fs::File::lock` is stable
and would be the idiomatic choice, but the on-disk mechanism is itself a
compatibility surface: the other side of that lock may be a Node Gateway, an
older VIA, or the CLI, and two processes using different primitives on the same
path exclude nothing.

**~43% of the catalogue is prose, not literals.** A reported heuristic across all 707
contracts finds 404 literal-shaped and 303 prose-shaped values. So "707 contracts asserted"
was never going to be 707 byte comparisons — roughly two fifths need behavioural tests in
their owning crates. As of phase 9 the split is **158 locked** (112 asserted + 46 divergent,
proven by `via-conformance` itself), **472 behavioural** (each row naming the shipped crate's
own test, resolved against the tree so a renamed test orphans nothing silently), 25 partial
and **52 pending** — every one of the twenty-six crates in scope, `docs/deviations/README.md`
naming the reason behind each of the 52. Phase 6's numbers were 85 / 432 / 14 / 176: phase 9's
conformance close-out very nearly doubled locked and cut pending by more than two-thirds,
without touching the catalogue itself — `docs/reference/contracts.json` still carries the same
707 records it did at phase 0. Behavioural deliberately does **not** count as locked, so the
headline number still means "what this gate itself proves".

## Audit baseline

Ported from ARGO (standalone Python, no coupling). The baseline ratchets down,
never up; a new site needs a dated justification or a fix.

| Gate | Baseline | Where |
| --- | ---: | --- |
| `panic_filter.py` | 2 | both in `via-conformance`, a test-only crate that must fail loudly on a malformed catalogue |
| `unreachable_filter.py` | 0 | |
| `risky_unwrap.py` | 0 | |
| `brand_leak.py` | 0 | 355 KEEP terms parsed from `rebrand.md` |

---

## Copied verbatim, byte for byte

File copies, not retypings. A test asserts each still matches, with only the
identity substitutions in [`rebrand.md`](rebrand.md) applied.

| Asset | Here | Upstream |
| --- | --- | --- |
| The core policy prompt | `assets/frontend-agent/zh/PROMPT.md` | `config/frontend-agent/PROMPT.md` |
| The assistant profile template | `assets/frontend-agent/zh/ASSISTANT.md` | same |
| OpenClaw bridge config | `assets/openclaw/openclaw.json5` | `config/openclaw/openclaw.json5` |
| DeepSeek Harness config | `assets/deepseek-harness/cordis.yml` | `config/deepseek-harness/cordis.yml` |
| CodeBuddy model table | `assets/codebuddy/workspace/.codebuddy/models.json` | `config/codebuddy/workspace/.codebuddy/models.json` |

`cordis.yml` and `models.json` were planned as the same kind of verbatim copy
from the start and, for a time, were not shipped — `crates/via-backends/assets/`
had only `openclaw/openclaw.json5`, and the Rust side duplicated one fragment of
each by hand instead of reading a packaged file (`DEEPSEEK_DEFAULT_MODEL`, a
plain constant; `CODEBUDDY_MODEL_URL`, an environment-variable name). Both are
now packaged as `include_str!` constants
(`via_backends::driver::DEEPSEEK_HARNESS_CORDIS_YAML` and
`::CODEBUDDY_MODELS_JSON_TEMPLATE`) that the two hand-duplicated fragments are
asserted against — `tests/contracts.rs::the_deepseek_default_model_matches_the_
shipped_cordis_asset` derives `DEEPSEEK_DEFAULT_MODEL` from the shipped YAML's
own `DSH_MODEL` default rather than restating it, and the CodeBuddy template's
two `${…}` placeholders are checked against
`via_core::config::names::DASHSCOPE_API_KEY` and `::CODEBUDDY_MODEL_URL`. The
target path the CodeBuddy contract names — `<codeBuddyWorkspace>/.codebuddy/
models.json` at directory mode `0o700`, file mode `0o600`, exclusive-create
(`'wx'`) — is a real function now too,
`via_core::runtime::seed_codebuddy_models_json`, built on the same
`IfExists::Keep` primitive `load_runtime_environment` already uses for every
other first-run seed (`config.env`, `USER.md`, `MEMORY.md`). `cordis.yml`'s
`!!js "…"` tags are JavaScript the DeepSeek Harness process evaluates, not YAML
this crate parses, so its five contracts and the CodeBuddy template's one are
asserted as literal substrings and byte-for-byte content respectively — the
same shape `the_shipped_openclaw_configuration_carries_every_catalogued_field`
already used for OpenClaw's JSON5 — rather than through a deserialized
document. That closes the seven `via-backends` contracts that quote
`cordis.yml`'s YAML body and the CodeBuddy model table, plus the one `via-core`
contract naming the CodeBuddy write target.

`PROMPT.md` is the assistant's core policy and cannot be overridden by
personalization. Upstream's is Chinese, and the `zh` locale carries it verbatim —
only the sentences that name the product are reworded. `ASSISTANT.md`'s identity
line (`你叫千问Audio`) is one of those.

VIA ships three locales (`en` default, `zh`, `ko`), so `en/` and `ko/` peers sit
beside `zh/`. Those are **authored, not translated**: a machine translation of a
policy prompt is a rewrite with none of the original's precision. Each is written
against the `zh` original as its specification, then checked back against it —
`via-conformance` asserts *structural* parity (same section headings, same
tool-name references, same `{{variable}}` set, same rule count) rather than byte
equality, which only `zh` can have.

Reproduced verbatim **in code**, with tests asserting the exact string:

- Every frontend tool name, description, and full JSON Schema — all eight tools
  the realtime model sees.
- The five coordination MCP tool schemas served to backends.
- `backend-agent-instructions`, which the coordinator injects into the backend
  envelope.
- Every error code, including `VIA_GATEWAY_SETUP_REQUIRED` and
  `VIA_GATEWAY_ALREADY_RUNNING` (upstream's `QWAUDIO_*`).
- The backend environment allow-lists — a security boundary, not a convenience.
- The `/api/health` payload **with its field order**.
- The Work state names and the legal transition graph.
- The `via.gateway-lock/v1` lease document shape.

The full list is [`reference/contracts.md`](reference/contracts.md): **707
contracts**, machine-readable in `reference/contracts.json`, asserted by
`via-conformance`.

---

## Adapted, with the reason

### Type system

| Upstream | Here | Why |
| --- | --- | --- |
| Duck-typed provider objects (`{ url, headers, model, … }`) | `trait RealtimeProvider` + `trait RealtimeProtocol` | Rust needs the seam to be nominal. The split separates *which service* from *which wire dialect*, which is what lets a local model reuse the OpenAI-Realtime dialect. |
| Backend drivers as plain objects with a validated shape | `trait AgentDriver` + `trait RuntimeDriver` + a `BackendCapabilities` struct | Upstream validates driver completeness at startup and rejects incomplete drivers. In Rust the trait does that at compile time. |
| `createBackendProfile` spreads `driver.capabilities` **after** the profile | Capabilities live only on the driver; the profile does not carry those seven fields | Upstream's spread order means a declared capability silently overwrites a same-named profile field. Several profiles set redundant copies that happen to agree today. Removing the duplication removes the question. |
| Ad-hoc `{ ok, error, code }` returns | `thiserror` enums with a `code()` accessor | The codes are contract; the shape around them is not. |

### Runtime

| Upstream | Here | Why |
| --- | --- | --- |
| `process.env` is process-global and is inherited by every child | An explicit `BackendEnv` newtype, and the only spawn helper always calls `.env_clear()` first | Node *replaces* the child environment when given one; Rust's `Command` *inherits* by default. This is the highest-value security property in the port and idiomatic Rust breaks it silently. |
| `child_process.spawn` + `detached` | `process-wrap` with `ProcessGroup::leader()` / `JobObject` + a SIGTERM → wait → SIGKILL ladder | Backends launch through `npx`/`uvx` wrappers, so killing only the immediate child orphans the real agent. |
| Hand-rolled ACP JSON-RPC over stdio | `agent-client-protocol` 2.0.0 (stable v1) | ~1,500 lines plus a JSON-RPC actor engine, for zero differentiation. |
| Hand-rolled MCP server | `rmcp` 3.1.4 | Same reasoning. The `agent-client-protocol-rmcp` bridge is deliberately skipped: it pins rmcp 2.x, and VIA's MCP server is standalone, declared in `session/new`. |
| Promise chains for the serialization invariants | An owning task per invariant: bounded `mpsc` + `oneshot` replies | A `tokio::sync::Mutex` provides mutual exclusion but not FIFO order, and three of the four invariants need order. See [`architecture.md` §6](architecture.md). |
| `process.env.X ?? 'default'` scattered across modules | One `impl Default for Config`, layered file → `VIA_*` env → CLI, snapshot-tested | The upstream defaults are contract. A single snapshot makes any change to any of the 75 catalogued defaults a reviewable diff. |
| npm global install, `npx` shims, `scripts/*-acp.mjs` | A self-contained binary; the shims become `via-backends` launch specs | A Rust binary is its own installer. Where a backend genuinely *is* an npm package, VIA keeps resolving that exact coordinate. |

### Corrected, not copied

`resolveDashScopeRealtimeModelProfile()` returns an all-capabilities-false
profile for an unrecognised model id, and `providers/dashscope.mjs:84-87` gates
`session.turn_detection` on `transportCapabilities.audioInput`. An unknown id
therefore opens a session with no audio input and no turn detection — it
connects and never hears anything. Since a local model id is by definition not
in the DashScope table, VIA gives the local family a real catalog entry and
makes the unknown-id fallback an **error** rather than a silent all-false
profile.

---

## Dropped, and why

The GUI surfaces. Core-only means they have no role:

- `web/` — the React SPA (3,615 lines).
- `desktop/` — the Electron host, the floating orb, the skin store, the
  auto-updater, the settings window (6,416 lines).
- `shared/electron-host.cjs`, `shared/orb-skin-catalog.mjs`, and the
  `qwen-audio-agent/electron`, `/orb/*`, `/skin-store` package entry points.

Consequently six `GATEWAY_CAPABILITIES` entries are not advertised:
`web.same-origin-ui`, `web.skin-assets`, `desktop.orb-shell`,
`desktop.orb-window-factory`, `desktop.orb-placement`,
`desktop.orb-position-store`, `desktop.skin-store`. A removed capability is a
breaking change under upstream's own versioning rule, so **VIA's
`GATEWAY_PROTOCOL_VERSION` starts at its own `1.0.0`** rather than claiming
upstream's `2.0.0`. Clients branch on the capability list, which is exactly what
that list is for.

The Gateway's HTTP and WebSocket protocol is otherwise ported **whole**,
including `/api/tasks`, `/api/timeline`, `/api/backend/ui` and
`/api/permissions/:id`, which only the dropped UIs consumed. It is the seam a
future frontend attaches to.

**Deferred, not dropped:** `tui/` — upstream's terminal UI, including the Swift
and PortAudio native voice bridges. VIA ships `via chat` (a port of upstream's
`tui/src/text-cli.mjs`) so the core is drivable end to end; the full interactive
TUI comes later.

---

## Added, which is not a port

**`via-realtime-mock`.** A deterministic replay provider. Upstream has no
equivalent and, as a result, most of its realtime tests need either a live
DashScope credential or an ad-hoc fake socket. A first-class mock provider makes
the entire Gateway — routing, tool calls, the Work queue, delegation, the
announcement window — testable under `insta` + `rstest` with no weights, no GPU
and no network. It is what makes phase 5 of the delivery plan a real milestone.

**`via-realtime-local`.** The on-device provider. Upstream's only local path is
`speech-to-speech`, which delegates to a separate Python service the user
installs and runs. `local-omni:pipeline` is 100% Rust — sherpa-onnx for VAD,
streaming ASR and TTS, `llama-cpp-2` for the reasoning turn — and
`local-omni:endpoint` points the existing OpenAI-Realtime client at a local
server. See [`architecture.md` §5](architecture.md) for why true end-to-end
Qwen3-Omni cannot be pure Rust today.

**`via-i18n` and the `en` / `ko` locales.** Upstream has no i18n layer at all —
every user-facing string is a Chinese literal at its use site. VIA hoists them
into a keyed catalog so three locales can coexist, with `zh` holding upstream's
exact values. This is the single largest *additive* change in the port, and it is
also where a careless rewrite could quietly weaken `PROMPT.md`'s policy language;
hence the structural-parity assertions.

**`via-arch-test`.** Upstream's `dependency-boundaries.test.mjs` walks import
statements with a regex. Here the layer graph is asserted against real
`cargo metadata` edges, and the rule it approximated with a
`/\b(openclaw|opencode|qoder|…)\b/i` scan over source text — *the generic ACP
and process cores must not bind to a named backend* — is a crate boundary the
compiler enforces.

---

## Known gaps

Tracked openly rather than papered over.

- **True on-device Qwen3-Omni is not pure Rust, and cannot be yet.** No Rust
  runtime runs the Talker — verified against candle, mistral.rs, llama.cpp and
  mlx-rs. `local-omni:endpoint` reaches it through an external server
  (`sgl-omni`, CUDA-only) or a Python `mlx-vlm` sidecar on Apple Silicon.
- **Qwen3.5-Omni has no open weights at all.** It is DashScope-API-only.
  On-device omni means Qwen3-Omni-30B-A3B (Sept 2025).
- **Streaming ASR granularity.** sherpa-onnx exposes Qwen3-ASR as an *offline*
  recognizer driven by a VAD-segmented loop, not a true streaming model.
  Utterance-granularity partials are the trade; the online zipformer and
  Parakeet recognizers are the alternative if lower latency matters more than
  Qwen3-ASR's quality.
- **Model artifacts are not managed by any crate.** Every ONNX model — Silero
  VAD, the keyword-spotting transducer, ASR, Kokoro voices — must be sourced,
  versioned, checksummed and shipped by VIA itself. Upstream's verified-download
  machinery for the wake-word model is ported and generalised to cover them.
- **The wake word needs a new phrase and a regenerated keyword model.** It is
  not a string edit. Open product decision.
- **Windows.** Upstream supports it; VIA's process-group containment,
  directory-fsync-on-rename and advisory-lock behaviour all differ there and are
  the least-exercised surface. Where a behaviour differs it is stated in the
  crate's docs rather than silently stubbed.
- **The `agent-client-protocol` crate is runtime-agnostic** and starts a smol
  reactor beside tokio. Acceptable, but it means tracing context and shutdown
  cross two reactors.
