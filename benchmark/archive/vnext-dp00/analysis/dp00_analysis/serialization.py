"""Deterministic derived-output serialization."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any


def write_summary(output_root: str | Path, summary: dict[str, Any]) -> Path:
    root = Path(output_root)
    root.mkdir(parents=True, exist_ok=True)
    destination = root / "analysis-summary.json"
    temporary = root / ".analysis-summary.json.tmp"
    temporary.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    temporary.replace(destination)
    return destination


def read_summary(derived_root: str | Path) -> dict[str, Any]:
    return json.loads((Path(derived_root) / "analysis-summary.json").read_text(encoding="utf-8"))
