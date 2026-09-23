#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DIR="${1:-$ROOT/models/qwen3-8b}"
FILE="$DIR/Qwen3-8B-Q4_K_M.gguf"
REV="6a569868d07d3bd59e8b97fb001bf8c0b254bb20"
EXPECTED="d98cdcbd03e17ce47681435b5150e34c1417f50b5c0019dd560e4882c5745785"
mkdir -p "$DIR"
if [[ ! -f "$FILE" ]]; then
  echo "Downloading the frozen ~5.03GB Qwen3-8B Q4_K_M artifact..."
  curl -fL --retry 3 -o "$FILE.part" "https://huggingface.co/Qwen/Qwen3-8B-GGUF/resolve/$REV/Qwen3-8B-Q4_K_M.gguf?download=true"
  mv "$FILE.part" "$FILE"
fi
ACTUAL="$(shasum -a 256 "$FILE" | awk '{print $1}')"
if [[ "$ACTUAL" != "$EXPECTED" ]]; then
  echo "SHA256 mismatch. Expected $EXPECTED, got $ACTUAL" >&2
  exit 3
fi
echo "Verified: $FILE"
echo "$ACTUAL"
