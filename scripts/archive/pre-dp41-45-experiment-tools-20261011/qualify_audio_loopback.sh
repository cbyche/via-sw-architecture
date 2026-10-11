#!/usr/bin/env bash
set -euo pipefail

readonly root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly source_file="${root}/benchmark/architecture/audio_loopback_probe.m"
readonly binary="/private/tmp/via-audio-loopback-probe"

xcrun clang \
  -O \
  -fobjc-arc \
  -framework Foundation \
  -framework AudioToolbox \
  -framework CoreAudio \
  "${source_file}" \
  -o "${binary}"

exec "${binary}" "$@"
