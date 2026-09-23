#!/usr/bin/env python3
"""Reject superseded metric and lifecycle terminology in active documents."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DOC_ROOT = ROOT / "docs" / "architecture"
FORBIDDEN = (
    "Conversational Reaction Responsiveness",
    "Task Handoff Responsiveness",
    "Task Feedback Responsiveness",
    "mean_of_case_p95_response_start_ms",
    "mean_of_case_p95_agent_acceptance_ms",
    "mean_of_case_p95_event_to_visible_ms",
    "11-A",
    "11-B",
    "11-C",
    "11-D",
    "11-E",
    "12-A",
    "12-B",
    "VIA-DESIGN",
)


def main() -> int:
    violations: list[str] = []
    for path in sorted(DOC_ROOT.rglob("*.md")):
        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            for term in FORBIDDEN:
                if term in line:
                    violations.append(f"{path.relative_to(ROOT)}:{line_number}: {term}")

    if violations:
        print("Superseded terminology found in active documents:")
        print("\n".join(violations))
        return 1

    print("PASS: superseded terminology is absent from active architecture documents")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
