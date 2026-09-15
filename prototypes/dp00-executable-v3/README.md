# DP-00 Executable Reference Architecture Qualification v3

This tree contains immutable imported implementation evidence plus new, separate executable Rust compositions for R1, R3 and the R1+@ tactic. It is intentionally not a candidate-enum simulator.

Build: `cargo build --release --all-targets --offline`.

Preflight: `.venv/bin/python -m benchmark.dp_executable_v3.preflight` from repository root.

Official campaign (clean preregistration commit only): `.venv/bin/python -m benchmark.dp_executable_v3.campaign --architecture-commit <SHA>`.

The campaign is fully offline and contains no Cloud/API integration.
