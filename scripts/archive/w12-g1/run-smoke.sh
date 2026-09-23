#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT/prototype/gate2"
cargo build --locked -p gate2-worker -p gate2-host -p gate2-bench
cargo test --locked --workspace
cargo run --locked -q -p gate2-bench -- smoke
cargo run --locked -q -p gate2-bench -- exec-smoke --worker "$ROOT/prototype/gate2/target/debug/gate2-worker"
cargo run --locked -q -p gate2-bench -- s2s-smoke --trace "$ROOT/prototype/gate2/fixtures/s2s-delay-smoke.json"
cargo run --locked -q -p gate2-bench -- w04-load-smoke
cargo run --locked -q -p gate2-bench -- w09-whole-restart-smoke
cargo run --locked -q -p gate2-bench -- w09-whole-process-smoke
cargo run --locked -q -p gate2-bench -- w09-integration-fatal-smoke --host "$ROOT/prototype/gate2/target/debug/gate2-host" --worker "$ROOT/prototype/gate2/target/debug/gate2-worker"
cargo run --locked -q -p gate2-bench -- w10-containment --spec "$ROOT/benchmark/rebaseline/gate2/w10-containment-cells.json" --host "$ROOT/prototype/gate2/target/debug/gate2-host" --worker "$ROOT/prototype/gate2/target/debug/gate2-worker" --profile smoke
echo "SMOKE ONLY: no comparative benchmark or score was produced."
