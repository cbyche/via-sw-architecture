# Internal Rust Voice Agent Reference

## Purpose

This directory records architecture observations derived from the existing
internal Rust-based Voice Interaction Agent prototype.

The prototype is maintained separately in:

`cbyche/via-internal-rust-reference`

Reference snapshot:

- Source branch: `main`
- Source commit: `b6032a4c472e18ec216b3e8592346ab269495342`
- Snapshot repository commit: `37b69d8`
- Imported date: `2026-09-09`

## Role in VIA Architecture

The internal Rust prototype is an **Existing Reference Implementation**.

It is not:

- the VIA approved architecture baseline,
- an automatically accepted architecture decision,
- or a requirement that the VIA architecture must follow.

Design choices observed in the prototype are treated as architecture inputs
and must be evaluated independently against:

- VIA Functional Requirements,
- Constraints,
- Quality Attributes,
- Architectural Decision Points,
- and measured prototype / benchmark results.

The intended reasoning flow is:

```text
VIA Requirements / QA
        +
Existing Rust Prototype
        ↓
Architectural Decision Point
        ↓
Alternative comparison
        ↓
Prototype / Experiment
        ↓
Architecture Decision Record
```

## Planned Reference Documents

This directory will contain:

- `architecture-summary.md`
- `component-map.md`
- `design-assumptions.md`
- `behavioral-scenarios.md`
- `source-index.md`
- `traceability.md`

These documents summarize architecture-relevant information without copying
the reference source code into this repository.
