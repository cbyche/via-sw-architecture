#!/usr/bin/env python3
"""Write a stable hash inventory for local campaign WAV artifacts."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("result_dir", type=Path)
    args = parser.parse_args()
    result_dir = args.result_dir.resolve()
    rows = []
    for path in sorted(result_dir.rglob("*.wav")):
        rows.append({
            "path": path.relative_to(result_dir).as_posix(),
            "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        })
    output = result_dir / "raw/audio-artifact-digests.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps({"artifact_count": len(rows), "artifacts": rows}, indent=2), encoding="utf-8")
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
