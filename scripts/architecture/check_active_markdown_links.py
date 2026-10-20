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
    ROOT / "INTENT.md",
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
COMMON15_PACKAGE = CURRENT_EVIDENCE / "common-supplement6-20261019/preservation-final-stopped258-v1"
COMMON15_MANIFEST_SHA = "dc0ad05f5bb4ccab61d367e95771ed65c7f0cf6b19d818464a0d9ebbd2a0aba3"
COMMON15_CAMPAIGNS = (
    CURRENT_EVIDENCE / "common9-poc-20261019",
    CURRENT_EVIDENCE / "common-supplement6-20261019",
    CURRENT_EVIDENCE / "four-dp-prediction-validation-20261019",
)


def published_common15_contexts() -> tuple[dict[Path, dict], set[Path], list[str]]:
    """Resolve published evidence locators without rewriting preserved bytes.

    Parts and inventory must match the published hashes. A packaged-only target
    is an available inventory member, not a claim of filesystem restoration or
    portable analysis. Ordinary unregistered missing links still fail.
    """
    manifest_path = COMMON15_PACKAGE / "chunk-manifest.json"
    if not COMMON15_PACKAGE.exists():
        return {}, set(), []
    try:
        if any(node.is_symlink() for node in (COMMON15_PACKAGE, *COMMON15_PACKAGE.parents)):
            raise ValueError("linked package ancestor")
        if (not manifest_path.is_file() or manifest_path.is_symlink()
                or manifest_path.stat().st_size > 2 * 1024 * 1024):
            raise ValueError("missing/linked published manifest")
        manifest_bytes = manifest_path.read_bytes()
        if hashlib.sha256(manifest_bytes).hexdigest() != COMMON15_MANIFEST_SHA:
            raise ValueError("unbound published manifest")
        manifest = json.loads(manifest_bytes)
        inventory_path = COMMON15_PACKAGE / "PRESERVATION-INVENTORY.json"
        if inventory_path.is_symlink() or inventory_path.stat().st_size > 16 * 1024 * 1024:
            raise ValueError("linked published inventory")
        inventory_bytes = inventory_path.read_bytes()
        if hashlib.sha256(inventory_bytes).hexdigest() != manifest["inventory_sha256"]:
            raise ValueError("inventory hash mismatch")
        inventory = json.loads(inventory_bytes)
        archive_digest = hashlib.sha256()
        archive_bytes = 0
        seen_parts = set()
        for part in manifest["parts"]:
            name = part["name"]
            if Path(name).name != name or name in seen_parts:
                raise ValueError("unsafe/duplicate archive part")
            seen_parts.add(name)
            part_path = COMMON15_PACKAGE / name
            if (part_path.is_symlink() or not 0 < part["bytes"] <= 20 * 1024 * 1024
                    or part_path.stat().st_size != part["bytes"]):
                raise ValueError("invalid archive part")
            digest, count = hashlib.sha256(), 0
            with part_path.open("rb") as stream:
                while chunk := stream.read(1024 * 1024):
                    digest.update(chunk)
                    archive_digest.update(chunk)
                    count += len(chunk)
            if count != part["bytes"] or digest.hexdigest() != part["sha256"]:
                raise ValueError("archive part hash/size mismatch")
            archive_bytes += count
        if archive_bytes != manifest["archive_bytes"] or archive_digest.hexdigest() != manifest["archive_sha256"]:
            raise ValueError("archive hash/size mismatch")
        files, seen_members = {}, set()
        for item in inventory["included_files"]:
            relative = Path(item["path"])
            if relative.is_absolute() or ".." in relative.parts or str(relative) != item["path"]:
                raise ValueError("unsafe inventory path")
            if item["path"] in seen_members:
                raise ValueError("duplicate inventory member")
            seen_members.add(item["path"])
            if item["kind"] == "FILE":
                if not re.fullmatch(r"[0-9a-f]{64}", item["sha256"]):
                    raise ValueError("invalid member hash")
                files[item["path"]] = item
        records = inventory["provenance"]["records"]
        roles = {row["role"]: row for row in records}
        if len(roles) != len(records):
            raise ValueError("duplicate provenance role")
        for role in ("generation_mapping", "joint441_final_freeze"):
            bound = roles[role]
            if (bound["path"] not in files or any(bound[key] != files[bound["path"]][field]
                    for key, field in (("sha256", "sha256"), ("bytes", "size")))):
                raise ValueError("unbound provenance role")
        mapping_ref = roles["generation_mapping"]
        mapping_path = ROOT / mapping_ref["path"]
        if (any(node.is_symlink() for node in (mapping_path, *mapping_path.parents))
                or not mapping_path.resolve().is_relative_to(ROOT.resolve())
                or mapping_path.stat().st_size > 2 * 1024 * 1024):
            raise ValueError("linked generation mapping")
        mapping_bytes = mapping_path.read_bytes()
        if (hashlib.sha256(mapping_bytes).hexdigest() != mapping_ref["sha256"]
                or len(mapping_bytes) != mapping_ref["bytes"]):
            raise ValueError("generation mapping mismatch")
        freeze_ref = json.loads(mapping_bytes)["artifacts"]["formal_freeze"]
        if any(freeze_ref[key] != roles["joint441_final_freeze"][key] for key in ("sha256", "bytes")):
            raise ValueError("recorded freeze association mismatch")
        recorded_freeze = Path(freeze_ref["path"])
        relative_freeze = Path(roles["joint441_final_freeze"]["path"])
        original = recorded_freeze.parents[len(relative_freeze.parts) - 1]
        if not original.is_absolute() or len(original.parts) < 3 or original / relative_freeze != recorded_freeze:
            raise ValueError("invalid recorded repository root")
        targets, checked_nodes = set(), set()
        for relative in files:
            target = ROOT / relative
            for node in (target, *target.parents):
                if node not in checked_nodes:
                    checked_nodes.add(node)
                    if node.is_symlink():
                        raise ValueError("linked inventory file/ancestor")
            if not target.resolve().is_relative_to(ROOT.resolve()):
                raise ValueError("inventory target escapes repository")
            # Keep exact lexical membership; no alias can admit another name.
            targets.add(target)
        contexts = {}
        for relative, item in files.items():
            source = (ROOT / relative).resolve()
            if source.suffix == ".md" and any(source.is_relative_to(campaign) for campaign in COMMON15_CAMPAIGNS):
                contexts[source] = {"sha256": item["sha256"], "original_root": original, "context": None}
        return contexts, targets, []
    except (KeyError, IndexError, TypeError, ValueError, OSError) as exc:
        return {}, set(), [f"{manifest_path.relative_to(ROOT)}: invalid published evidence metadata: {type(exc).__name__}"]


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
    common15_contexts, packaged_targets, package_violations = published_common15_contexts()
    contexts.update(common15_contexts)
    violations.extend(package_violations)
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
                if not resolved.exists() and resolved not in packaged_targets:
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
