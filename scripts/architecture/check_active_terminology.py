#!/usr/bin/env python3
"""Reject superseded metric, requirement, and lifecycle terminology."""

from pathlib import Path
import re


ROOT = Path(__file__).resolve().parents[2]
ACTIVE_INPUTS = (
    ROOT / "README.md",
    ROOT / "AGENTS.md",
    ROOT / "CONTRIBUTING.md",
    ROOT / ".github" / "workflows",
    ROOT / "docs" / "architecture",
    ROOT / "docs" / "adr",
    ROOT / "benchmark" / "architecture",
    ROOT / "prototypes" / "candidates",
    ROOT / "scripts" / "architecture",
)
TEXT_SUFFIXES = {".md", ".py", ".rs", ".json", ".toml", ".yml", ".yaml"}
FORBIDDEN = (
    "Conversational Reaction Responsiveness",
    "Task Handoff Responsiveness",
    "Task Feedback Responsiveness",
    "mean_of_case_p95_response_start_ms",
    "mean_of_case_p95_agent_acceptance_ms",
    "mean_of_case_p95_event_to_visible_ms",
    "VIA-DESIGN",
)
FORBIDDEN_PATTERNS = (
    ("legacy W-series metric ID", re.compile(r"\bW-(?:0[1-9]|1[0-2])\b")),
    ("numbered ASR ID", re.compile(r"\bASR-\d{2}\b")),
    (
        "legacy QA sub-ID",
        re.compile(r"(?<![A-Za-z0-9-])(?:11-[A-E]|12-[AB])(?![A-Za-z0-9-])"),
    ),
)


def active_files() -> list[Path]:
    files: list[Path] = []
    for item in ACTIVE_INPUTS:
        if item.is_file():
            files.append(item)
            continue
        for path in item.rglob("*"):
            if "target" in path.parts or path.suffix not in TEXT_SUFFIXES:
                continue
            if path.is_file() and path.resolve() != Path(__file__).resolve():
                files.append(path)
    return sorted(set(files))


def main() -> int:
    violations: list[str] = []
    for path in active_files():
        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            for term in FORBIDDEN:
                if term in line:
                    violations.append(f"{path.relative_to(ROOT)}:{line_number}: {term}")
            for label, pattern in FORBIDDEN_PATTERNS:
                if pattern.search(line):
                    violations.append(f"{path.relative_to(ROOT)}:{line_number}: {label}")

    if violations:
        print("Superseded terminology found in active documents:")
        print("\n".join(violations))
        return 1

    print("PASS: superseded terminology is absent from active architecture documents")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
