#!/usr/bin/env python3
"""Check repository-local links in current, non-archived Markdown documents."""

from pathlib import Path
import re
from urllib.parse import unquote


ROOT = Path(__file__).resolve().parents[2]
ACTIVE_INPUTS = (
    ROOT / "README.md",
    ROOT / "AGENTS.md",
    ROOT / "CONTRIBUTING.md",
    ROOT / "docs" / "architecture",
    ROOT / "docs" / "adr",
    ROOT / "docs" / "references" / "README.md",
    ROOT / "docs" / "archive" / "README.md",
    ROOT / "benchmark" / "README.md",
    ROOT / "benchmark" / "architecture",
    ROOT / "benchmark" / "archive" / "README.md",
    ROOT / "prototypes" / "gate2",
    ROOT / "prototypes" / "archive" / "README.md",
    ROOT / "results" / "gate2" / "README.md",
    ROOT / "results" / "gate2" / "current",
    ROOT / "results" / "gate2" / "archive" / "README.md",
    ROOT / "scripts" / "gate2" / "README.md",
    ROOT / "scripts" / "archive" / "README.md",
)
LINK = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")


def markdown_files() -> list[Path]:
    files: list[Path] = []
    for item in ACTIVE_INPUTS:
        if item.is_file():
            files.append(item)
        elif item.is_dir():
            files.extend(item.rglob("*.md"))
    return sorted(set(files))


def local_target(raw: str) -> str | None:
    target = raw.strip().strip("<>")
    if not target or target.startswith(("#", "http://", "https://", "mailto:")):
        return None
    # Markdown titles are not used in the active corpus; preserve spaces in paths.
    return unquote(target.split("#", 1)[0])


def main() -> int:
    violations: list[str] = []
    for source in markdown_files():
        text = source.read_text(encoding="utf-8")
        for line_number, line in enumerate(text.splitlines(), 1):
            for match in LINK.finditer(line):
                target = local_target(match.group(1))
                if target is None:
                    continue
                resolved = (source.parent / target).resolve()
                try:
                    resolved.relative_to(ROOT.resolve())
                except ValueError:
                    violations.append(
                        f"{source.relative_to(ROOT)}:{line_number}: outside repository: {target}"
                    )
                    continue
                if not resolved.exists():
                    violations.append(
                        f"{source.relative_to(ROOT)}:{line_number}: missing: {target}"
                    )

    if violations:
        print("Broken links in active Markdown:")
        print("\n".join(violations))
        return 1
    print(f"PASS: checked local links in {len(markdown_files())} active Markdown files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
