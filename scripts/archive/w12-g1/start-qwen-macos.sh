#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MODEL="${G2_MODEL_PATH:-$ROOT/models/qwen3-8b/Qwen3-8B-Q4_K_M.gguf}"
EXPECTED="d98cdcbd03e17ce47681435b5150e34c1417f50b5c0019dd560e4882c5745785"
[[ -f "$MODEL" ]] || { echo "Missing model: $MODEL. Run scripts/gate2/fetch-qwen-model.sh first."; exit 2; }
command -v llama-server >/dev/null 2>&1 || { echo "Missing llama-server. Install the frozen runtime before measurement."; exit 2; }
ACTUAL="$(shasum -a 256 "$MODEL" | awk '{print $1}')"
[[ "$ACTUAL" == "$EXPECTED" ]] || { echo "Model SHA mismatch: $ACTUAL"; exit 3; }
echo "Starting local Qwen3-8B non-thinking server on 127.0.0.1:8080"
exec llama-server \
  --model "$MODEL" \
  --alias Qwen3-8B-Q4_K_M \
  --host 127.0.0.1 --port 8080 \
  --ctx-size 16384 --parallel 1 \
  --jinja --reasoning off --no-context-shift \
  -ngl 99 -fa on
