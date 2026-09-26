#!/usr/bin/env python3
"""Fail closed when a current DP result package is incomplete or ambiguous."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys
from typing import Any


QA_IDS = (
    "QA-01", "QA-02", "QA-03", "QA-04", "QA-05",
    "QA-11", "QA-12", "QA-13", "QA-14", "QA-15",
    "QA-21", "QA-22", "QA-23", "QA-31", "QA-32", "QA-41",
    "QA-51", "QA-61", "QA-62",
)


def load(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def validate(result_dir: Path) -> list[str]:
    errors: list[str] = []
    required = (
        "manifest.json", "raw/trials.jsonl", "summary.json",
        "replay-receipt.json", "report.md", "STATUS.md",
    )
    for relative in required:
        if not (result_dir / relative).is_file():
            errors.append(f"missing required artifact: {relative}")
    if errors:
        return errors

    manifest = load(result_dir / "manifest.json")
    summary = load(result_dir / "summary.json")
    receipt = load(result_dir / "replay-receipt.json")
    if manifest.get("status") != "COMPLETE":
        errors.append(f"manifest status is not COMPLETE: {manifest.get('status')!r}")
    if receipt.get("reproduced") is not True:
        errors.append("independent replay was not reproduced")
    candidates = summary.get("candidate_results")
    if not isinstance(candidates, dict) or len(candidates) < 2:
        errors.append("summary must contain at least two candidate_results")
        return errors
    expected = set(QA_IDS)
    for candidate, result in candidates.items():
        qa = result.get("qa", {})
        missing = expected - set(qa)
        extra = set(qa) - expected
        if missing:
            errors.append(f"{candidate}: missing QA rows: {sorted(missing)}")
        if extra:
            errors.append(f"{candidate}: unknown QA rows: {sorted(extra)}")
        for qa_id, value in qa.items():
            if value.get("status") in {"N/A", "BLOCKED"} and not value.get("reason"):
                errors.append(f"{candidate}/{qa_id}: {value['status']} has no reason")
    raw_lines = [
        line for line in (result_dir / "raw/trials.jsonl").read_text(encoding="utf-8").splitlines()
        if line
    ]
    if not raw_lines:
        errors.append("raw/trials.jsonl is empty")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("result_dir", type=Path)
    args = parser.parse_args()
    errors = validate(args.result_dir.resolve())
    if errors:
        print("evaluation package INVALID", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("evaluation package valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
