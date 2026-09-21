#!/usr/bin/env bash
# Readiness only: no model calls, installs, benchmark ranking or score computation.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
OUT="${1:-results/gate2-readiness-$(date -u +%Y%m%dT%H%M%SZ)-$$}"
if [ "$#" -gt 2 ] || { [ "$#" -eq 2 ] && [ "$2" != "--with-rust-smoke" ]; }; then
  echo "Usage: bash scripts/gate2/prepare-review-bundle.sh [NEW_OUTPUT_DIR] [--with-rust-smoke]" >&2
  exit 2
fi
if [ -e "$OUT" ]; then echo "Refusing to overwrite: $OUT" >&2; exit 2; fi
mkdir -p "$OUT"
if [ ! -x .venv/bin/python ]; then
  command -v python3 >/dev/null || { echo 'Install Python 3.11+ explicitly, then retry.' >&2; exit 2; }
  python3 -c 'import sys; assert sys.version_info >= (3,11), "Python 3.11+ required"'
  python3 -m venv .venv
fi
PY="$ROOT/.venv/bin/python"
"$PY" -c 'import sys; assert sys.version_info >= (3,11), "Python 3.11+ required"'
"$PY" -m unittest discover -s benchmark/rebaseline/readiness -p 'test_*.py' -v 2>&1 | tee "$OUT/readiness-tests.txt"
if [ "${2:-}" = "--with-rust-smoke" ]; then
  command -v cargo >/dev/null || { echo 'Rust is not installed; review bootstrap instructions first.' >&2; exit 2; }
  # The existing runner tests correctness only and may resolve a local Cargo.lock.
  # Record that exact lock below; this is not a frozen comparative run.
  bash scripts/gate2/run-smoke.sh 2>&1 | tee "$OUT/rust-smoke.txt"
fi
"$PY" benchmark/rebaseline/readiness/freeze.py --root "$ROOT" --out "$OUT/manifest"
printf '%s\n' 'REVIEW_BUNDLE_CREATED' 'Comparative benchmark: NOT_RUN' 'Actual Qwen inference: NOT_RUN' 'No approval was generated.'
printf 'Bundle: %s\n' "$OUT"
