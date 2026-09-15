# VIA

**V**oice **I**nteraction **A**gent — a Rust voice-agent gateway.

VIA keeps a conversation flowing while work happens. Questions it can answer, it
answers immediately. Work that needs tools, files, or several minutes is handed
to a backend agent harness, and the result comes back into the same conversation
when there is a safe moment to speak it. The user talks to one assistant
throughout.

> **Status: all nine phases complete.** 28 crates — 25 shipped plus the three
> test-only gates — one binary, 224k lines of Rust, 4,599 tests. `via gateway`
> serves, `via chat` drives it end to end, the Context Engine and the on-device
> providers both ship, and `via-conformance`'s registry locks 158 of the
> catalogue's 707 external contracts outright (472 more asserted behaviourally)
> with 52 honestly named as still pending — see
> [the coverage paragraph](docs/fidelity.md) and
> [the deviations index](docs/deviations/README.md) for the full breakdown.
> See [the delivery plan](docs/architecture.md#15-delivery-plan).

## Modes

A session runs in exactly one of four modes, fixed at connect time.

| Mode | What it is |
| --- | --- |
| `dictation` | Transcribe and nothing else. No model turn, no speech out. |
| `direct` | The realtime model converses and answers itself. No backend. |
| `agent` | The whole stack: fast-path answers plus delegated work. |
| `interface` | Voice drives the host's UI through registered affordances. |

`agent` degrades to `direct` when no harness is configured, so a fresh install is
useful before any backend is installed. The degradation is reported on
`/api/health`, never silent.

## Bring your own agent harness

VIA ships **no agent harness of its own**. Layer 3 is one extension point —
`trait DownstreamAgent` — and every harness is a plugin behind it. A harness
answers prompts and emits events; it never owns a queue, a `work_id`, or a
permission decision, because those live above the trait and behave identically
no matter what is plugged in.

Out of the box VIA speaks **ACP**, which covers twelve existing agents (OpenCode,
OpenClaw, Qoder, Qwen Code, Kimi, Hermes, CodeBuddy, Codex, Claude Code, DeepSeek,
Pi, and any generic ACP agent), and **CLI**, where a harness is described by a
command and a stdio protocol. Adding either kind requires no VIA code — a harness
is named in configuration by id.

## Two upstreams

VIA is a port, and it says so. Both upstreams are Apache-2.0.

| Upstream | Supplies |
| --- | --- |
| [qwen-audio-agent](https://github.com/QwenAudio/qwen-audio-agent) | Layer 2 — task lifecycle, coordination, delivery |
| ARGO | Layer 1 — the realtime transport and its dialect handling |

What either upstream exposes to a model, a backend, or a client is reproduced
exactly: **707 external contracts** are catalogued in
[`docs/reference/contracts.md`](docs/reference/contracts.md) and asserted by
`via-conformance`. Everything else is idiomatic Rust. Every deviation is recorded
in [`docs/fidelity.md`](docs/fidelity.md).

## Documentation

| | |
| --- | --- |
| [`docs/architecture.md`](docs/architecture.md) | The design — layers, crate graph, invariants, delivery plan |
| [`docs/adr/0001-placement.md`](docs/adr/0001-placement.md) | Why VIA is standalone and ARGO is a spec source, not a dependency |
| [`docs/fidelity.md`](docs/fidelity.md) | What was ported, adapted, or dropped — and why |
| [`docs/rebrand.md`](docs/rebrand.md) | 198 identity strings: what is ours, what is not |
| [`docs/reference/contracts.md`](docs/reference/contracts.md) | The 707 acceptance criteria |

## Build

```bash
cargo build --workspace && cargo test --workspace
```

Try it. `via gateway` refuses to start without a realtime credential — that
refusal is the setup gate, and it is a contract — so opt out of it for a
look-around. No audio hardware and no model weights are needed either way:

```bash
VIA_ALLOW_UNCONFIGURED=1 cargo run -p via -- gateway
```

```bash
cargo run -p via -- chat
```

`via chat` starts a Gateway itself if none is running, so the second command
alone is enough once a credential is configured.

The gates that keep the port honest run in CI and locally:

```bash
cargo test -p via-arch-test && cargo test -p via-conformance && python3 scripts/brand_leak.py
```

## Licence

Apache-2.0. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).
