#!/usr/bin/env python3
"""Reject superseded metric, requirement, and lifecycle terminology."""

from pathlib import Path
import re
from html import unescape
import xml.etree.ElementTree as ET


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

TARGET = ROOT / "docs/architecture/12-decisions/target-architecture"
COMPONENT_ALIASES = re.compile(
    r"(?<![A-Za-z0-9_])(?:Controller|Interpreter|Gateway|Store|IM|Resolver|Publisher|"
    r"Interaction Runtime|Decision Validator|Context owner)(?![A-Za-z0-9_])",
    re.IGNORECASE,
)


def component_name_violations() -> list[str]:
    """Check target prose and visible figure labels against the Component table.

    Task, Context and Response are also data concepts, so their standalone use
    needs semantic review rather than a blanket ban. Historical designs and XML
    element IDs are outside this check.
    """
    source = (TARGET / "architecture.md").read_text(encoding="utf-8")
    section = source.split("## 4. Component와 상태 소유권\n", 1)[1].split("## 5.", 1)[0]
    table = section.split("| Component |", 1)[1].split("\n\n", 1)[0]
    names = re.findall(r"^\| ([A-Za-z]+(?: [A-Za-z]+)+) \|", table, re.MULTILINE)
    if not names or len(set(names)) != len(names):
        return ["target architecture: missing or duplicate canonical Component names in section 4"]
    canonical = re.compile(
        r"(?<![A-Za-z0-9_])(?:"
        + "|".join(re.escape(name) for name in sorted(names, key=len, reverse=True))
        + r")(?![A-Za-z0-9_])",
        re.IGNORECASE,
    )
    violations = []
    for path in sorted(TARGET.rglob("*")):
        if path.suffix == ".md":
            labels = list(enumerate(path.read_text(encoding="utf-8").splitlines(), 1))
        elif path.suffix in {".svg", ".drawio"}:
            root = ET.fromstring(path.read_text(encoding="utf-8"))
            if path.suffix == ".svg":
                labels = [(n.attrib.get("id", "text"), "".join(n.itertext()))
                          for n in root.iter() if n.tag.rsplit("}", 1)[-1] in {"text", "title", "desc"}]
            else:
                labels = [(n.attrib["id"], unescape(re.sub(r"<[^>]+>", " ", n.attrib.get("value", ""))))
                          for n in root.iter("mxCell")]
        else:
            continue
        for location, label in labels:
            for match in canonical.finditer(label):
                if match.group() not in names:
                    violations.append(f"{path.relative_to(ROOT)}:{location}: noncanonical case: {match.group()}")
            remainder = canonical.sub("", label)
            for match in COMPONENT_ALIASES.finditer(remainder):
                violations.append(f"{path.relative_to(ROOT)}:{location}: abbreviated Component: {match.group()}")
    return violations


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
    violations: list[str] = component_name_violations()
    for path in active_files():
        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            for term in FORBIDDEN:
                if term in line:
                    violations.append(f"{path.relative_to(ROOT)}:{line_number}: {term}")
            for label, pattern in FORBIDDEN_PATTERNS:
                if pattern.search(line):
                    violations.append(f"{path.relative_to(ROOT)}:{line_number}: {label}")

    if violations:
        print("Terminology violations found in active documents:")
        print("\n".join(violations))
        return 1

    print("PASS: active terminology and canonical target Component names are consistent")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
