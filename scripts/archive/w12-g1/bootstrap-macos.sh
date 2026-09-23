#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
echo "Gate2 bootstrap check for: $(uname -s) $(uname -m)"
[[ "$(uname -s)" == "Darwin" ]] || { echo "This script targets macOS."; exit 2; }

if ! xcode-select -p >/dev/null 2>&1; then
  echo "MISSING: Xcode Command Line Tools. Run: xcode-select --install"
fi
if ! command -v rustup >/dev/null 2>&1; then
  echo "MISSING: rustup. Install from https://rustup.rs, then rerun."
else
  rustup toolchain install 1.98.1 --profile minimal --component rustfmt,clippy
  (cd "$ROOT/prototype/gate2" && rustup override set 1.98.1)
fi
if ! command -v brew >/dev/null 2>&1; then
  echo "OPTIONAL/MISSING: Homebrew. Needed later for llama.cpp convenience."
else
  echo "Homebrew: $(brew --version | head -1)"
fi
if command -v llama-server >/dev/null 2>&1; then
  echo "llama-server found: $(command -v llama-server)"
else
  echo "IR model phase only: install later with 'brew install llama.cpp' after model/runtime freeze."
fi

echo "No benchmark was run. Next safe check:"
echo "  cd $ROOT/prototype/gate2 && cargo test --workspace"
