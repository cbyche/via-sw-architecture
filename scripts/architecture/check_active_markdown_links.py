#!/usr/bin/env python3
"""Check repository-local links in current, non-archived Markdown documents."""

from pathlib import Path
import hashlib
import json
import re
from urllib.parse import unquote


ROOT = Path(__file__).resolve().parents[2]
CURRENT_EVIDENCE = ROOT / "results" / "architecture-evaluation" / "current"
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
    ROOT / "prototypes" / "candidates",
    ROOT / "prototypes" / "archive" / "README.md",
    ROOT / "results" / "gate2" / "README.md",
    ROOT / "results" / "architecture-evaluation" / "current",
    ROOT / "results" / "gate2" / "archive" / "README.md",
    ROOT / "scripts" / "architecture" / "README.md",
    ROOT / "scripts" / "archive" / "README.md",
)
LINK = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
FOUR_DP_BUNDLE = ROOT / "results/architecture-evaluation/current/four-dp-poc-evidence-20261019/manifest.json"
FOUR_DP_CAMPAIGNS = (
    CURRENT_EVIDENCE / "four-dp-qwen37-20261018",
    CURRENT_EVIDENCE / "four-dp-qwen37-review-v2-20261018",
)


def restored_markdown_contexts() -> tuple[dict[Path, dict], list[str]]:
    """Read link context only for exact, SHA-pinned restored four-DP Markdown.

    The bundle preserves measured bytes, including historical absolute links and
    copied source documents. This changes neither those bytes nor normal source
    document validation; every mapped target must still exist inside this repo.
    """
    if not FOUR_DP_BUNDLE.is_file():
        return {}, []
    try:
        manifest = json.loads(FOUR_DP_BUNDLE.read_text(encoding="utf-8"))
        original = Path(manifest["original_repo_root"])
        if not original.is_absolute() or len(original.parts) < 3:
            raise ValueError("invalid recorded repository root")
        contexts = {}
        for item in manifest["files"]:
            relative = Path(item["path"])
            if relative.is_absolute() or ".." in relative.parts:
                raise ValueError("non-local manifest path")
            source = (ROOT / relative).resolve()
            if source.suffix != ".md" or not any(
                source.is_relative_to(campaign) for campaign in FOUR_DP_CAMPAIGNS
            ):
                continue
            digest = item["sha256"]
            if not re.fullmatch(r"[0-9a-f]{64}", digest):
                raise ValueError("invalid Markdown SHA256")
            context = item.get("markdown_link_context")
            if context is not None:
                relative_context = Path(context)
                if relative_context.is_absolute() or ".." in relative_context.parts:
                    raise ValueError("non-local Markdown link context")
                context = (ROOT / relative_context).resolve()
                if not context.is_relative_to(ROOT.resolve()):
                    raise ValueError("Markdown context escapes repository")
            if source in contexts:
                raise ValueError("duplicate restored Markdown entry")
            contexts[source] = {"sha256": digest, "original_root": original, "context": context}
        return contexts, []
    except (KeyError, TypeError, ValueError, OSError) as exc:
        return {}, [f"{FOUR_DP_BUNDLE.relative_to(ROOT)}: invalid link metadata: {type(exc).__name__}"]


def preserved_measurement_snapshot(path: Path) -> bool:
    """Frozen/excluded copies retain their original link context as evidence.

    Campaign reports, raw-data guides and original preparation reviews still
    receive the normal check. This does not change a frozen evidence document.
    """
    if not path.is_relative_to(CURRENT_EVIDENCE):
        return False
    relative = path.relative_to(CURRENT_EVIDENCE)
    if len(relative.parts) < 2 or not relative.parts[0].startswith("dp41-"):
        return False
    campaign = CURRENT_EVIDENCE / relative.parts[0]
    # Retired attempts explicitly carry invalidation provenance, not results.
    if (campaign / "invalidated.json").is_file():
        return True
    # A docs subtree here is an exact measured-source copy, not a standalone
    # Architecture website; its transitive navigation corpus is not copied.
    if "docs" in relative.parts[1:]:
        return True
    if path.parent == campaign / "reviews":
        original = CURRENT_EVIDENCE / "dp41-p21-mac-20261012-preparation" / path.name
        if original.is_file() and original.read_bytes() == path.read_bytes():
            return True
    return False


def markdown_files() -> list[Path]:
    files: list[Path] = []
    for item in ACTIVE_INPUTS:
        if item.is_file():
            files.append(item)
        elif item.is_dir():
            files.extend(item.rglob("*.md"))
    return sorted(path for path in set(files) if not preserved_measurement_snapshot(path))


def local_target(raw: str) -> str | None:
    target = raw.strip().strip("<>")
    if not target or target.startswith(("#", "http://", "https://", "mailto:")):
        return None
    # Markdown titles are not used in the active corpus; preserve spaces in paths.
    # Codex file links may include a one-based source line suffix.
    return re.sub(r":\d+$", "", unquote(target.split("#", 1)[0]))


def main() -> int:
    contexts, violations = restored_markdown_contexts()
    for source in markdown_files():
        preserved = contexts.get(source.resolve())
        if preserved is not None:
            if hashlib.sha256(source.read_bytes()).hexdigest() != preserved["sha256"]:
                violations.append(f"{source.relative_to(ROOT)}: restored Markdown SHA256 mismatch")
                preserved = None
            elif preserved["context"] is not None and not preserved["context"].is_file():
                violations.append(f"{source.relative_to(ROOT)}: recorded Markdown context is missing")
                preserved = None
        text = source.read_text(encoding="utf-8")
        for line_number, line in enumerate(text.splitlines(), 1):
            for match in LINK.finditer(line):
                target = local_target(match.group(1))
                if target is None:
                    continue
                target_path = Path(target)
                if preserved is not None and target_path.is_absolute() and target_path.is_relative_to(preserved["original_root"]):
                    resolved = (ROOT / target_path.relative_to(preserved["original_root"])).resolve()
                else:
                    context = preserved["context"] if preserved is not None else None
                    resolved = ((context or source).parent / target).resolve()
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
