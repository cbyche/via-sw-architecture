#!/usr/bin/env bash
set -euo pipefail

readonly MODEL_SHA256="d98cdcbd03e17ce47681435b5150e34c1417f50b5c0019dd560e4882c5745785"
readonly DEFAULT_MODEL_PATH="${HOME}/.cache/huggingface/hub/models--Qwen--Qwen3-8B-GGUF/blobs/${MODEL_SHA256}"

model_path="${VIA_SEMANTIC_MODEL_PATH:-${DEFAULT_MODEL_PATH}}"
host="${VIA_SEMANTIC_HOST:-127.0.0.1}"
port="${VIA_SEMANTIC_PORT:-18080}"
context_size="${VIA_SEMANTIC_CONTEXT_SIZE:-16384}"

if ! command -v llama-server >/dev/null 2>&1; then
  echo "llama-server not found. Install it with: brew install llama.cpp" >&2
  exit 1
fi

if [[ ! -f "${model_path}" ]]; then
  echo "Frozen model file not found: ${model_path}" >&2
  echo "Download it once with: llama-server -hf Qwen/Qwen3-8B-GGUF:Q4_K_M" >&2
  exit 1
fi

actual_sha256="$(shasum -a 256 "${model_path}" | awk '{print $1}')"
if [[ "${actual_sha256}" != "${MODEL_SHA256}" ]]; then
  echo "Model SHA-256 mismatch: expected ${MODEL_SHA256}, got ${actual_sha256}" >&2
  exit 1
fi

exec llama-server \
  --model "${model_path}" \
  --host "${host}" \
  --port "${port}" \
  --ctx-size "${context_size}" \
  --reasoning off
