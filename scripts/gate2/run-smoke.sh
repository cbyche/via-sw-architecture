#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT/prototype/gate2"
cargo build -p gate2-worker -p gate2-host -p gate2-bench
cargo test --workspace
cargo run -q -p gate2-bench -- smoke
cargo run -q -p gate2-bench -- exec-smoke --worker "$ROOT/prototype/gate2/target/debug/gate2-worker"
cargo run -q -p gate2-bench -- s2s-smoke --trace "$ROOT/prototype/gate2/fixtures/s2s-delay-smoke.json"
cargo run -q -p gate2-bench -- w04-load-smoke
cargo run -q -p gate2-bench -- w09-whole-restart-smoke
echo "SMOKE ONLY: no comparative benchmark or score was produced."
