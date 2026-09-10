# DP-00 Qualification Prototype

Phase 1 provides only the neutral Rust workspace and evaluation-support
infrastructure. Architecture alternatives and their decision logic are deliberately
absent.

## Frozen runtime direction

- Rust 1.94.0, matching the specification/reference `1.94` direction.
- Rust edition 2024.
- Tokio 1.53.1, resolved and frozen by `Cargo.lock`, for the common async substrate.
- `tokio-current-thread-v0` is the Phase 1 worker policy. It removes
  alternative-specific scheduler choices; any later multi-thread/parallel policy
  requires an explicit versioned qualification-profile change.
- Ordinary development uses Cargo's `dev`/`test` profiles.
- Timed qualification uses `--profile qualification`, an optimized performance
  profile distinct from ordinary development. Its flags must remain common across
  all future alternatives and be re-frozen before Pilot.

The timed event path records a monotonic timestamp and appends a typed event to an
in-memory buffer. JSON/JSONL conversion is exposed only through the separate
post-capture serialization API.
