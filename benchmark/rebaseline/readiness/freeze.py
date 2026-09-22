"""Content-addressed measurement gate. No installation, network call or benchmark."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
SUFFIXES = {".rs", ".toml", ".lock", ".py", ".json", ".md", ".sh"}
EXCLUDED = {"target", ".git", ".venv", "__pycache__", "results", "node_modules"}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def command(*args: str, cwd: Path | None = None) -> str | None:
    try:
        result = subprocess.run(args, cwd=cwd, text=True, stdout=subprocess.PIPE,
                                stderr=subprocess.DEVNULL, timeout=10, check=False)
        return result.stdout.strip() if result.returncode == 0 else None
    except (OSError, subprocess.TimeoutExpired):
        return None


def host_manifest(root: Path) -> dict[str, Any]:
    """Deliberately omit host/user names, serial numbers, IPs and environment secrets."""
    system = platform.system()
    memory = None
    chip = None
    model = None
    os_version = platform.release()
    if system == "Darwin":
        raw = command("sysctl", "-n", "hw.memsize")
        memory = int(raw) if raw and raw.isdecimal() else None
        chip = command("sysctl", "-n", "machdep.cpu.brand_string")
        model = command("sysctl", "-n", "hw.model")
        os_version = command("sw_vers", "-productVersion") or os_version
    status = command("git", "status", "--porcelain", cwd=root)
    runtime_root = root / "prototype/gate2"
    runtime_cwd = runtime_root if runtime_root.is_dir() else root
    return {
        "schema_version": 1,
        "captured_at_utc": datetime.now(timezone.utc).isoformat(),
        "measurement_os": system,
        "measurement_os_version": os_version,
        "architecture": platform.machine(),
        "cpu_logical_count": os.cpu_count(),
        "memory_bytes": memory,
        "chip_detected": chip,
        "hardware_model_detected": model,
        "python_version": platform.python_version(),
        "git_revision": command("git", "rev-parse", "HEAD", cwd=root),
        "worktree_dirty": None if status is None else bool(status),
        "rust_version": command("rustc", "--version", cwd=runtime_cwd),
        "cargo_version": command("cargo", "--version", cwd=runtime_cwd),
        "declared_product_target": "Windows 11 consumer PC (not this measurement OS)",
        "benchmark_executed": False,
        "score": None,
    }


def inventory(root: Path) -> dict[str, str]:
    paths = ["prototype/gate2", "benchmark/rebaseline/readiness", "benchmark/rebaseline/gate2",
             "benchmark/rebaseline/working12", "docs/rebaseline/12-gate2",
             "docs/rebaseline/11d-working-12-measurement-and-scoring.md",
             "docs/rebaseline/12-00-evaluation-method.md",
             "docs/rebaseline/12-04-measurement-freeze-review.md",
             "docs/rebaseline/12-04a-implementation-mapping-and-fairness.md",
             "docs/rebaseline/11b-test-case-catalog.md", "scripts/gate2"]
    result = {}
    for name in paths:
        item = root / name
        children = [item] if item.is_file() else list(item.rglob("*")) if item.exists() else []
        for path in children:
            relative = path.relative_to(root)
            if path.is_file() and path.suffix in SUFFIXES and not EXCLUDED.intersection(relative.parts):
                if path.is_symlink():
                    raise ValueError(f"refusing symlink in freeze inventory: {relative}")
                result[str(relative)] = digest(path)
    return dict(sorted(result.items()))


def fingerprint(files: dict[str, str]) -> str:
    return hashlib.sha256(json.dumps(files, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def candidate_manifest(root: Path) -> dict[str, Any]:
    files = inventory(root)
    if not files:
        raise ValueError("empty source inventory")
    lock = "prototype/gate2/Cargo.lock"
    return {"schema_version": 1, "purpose": "MEASUREMENT_FREEZE_REVIEW", "files": files,
            "fingerprint": fingerprint(files), "cargo_lock_present": lock in files,
            "comparative_measurement_approved": False, "scores": None}


def authorize_run(manifest: dict[str, Any], approval: dict[str, Any], root: Path, *, real_model: bool = False) -> None:
    """Check a human approval assertion, not a cryptographic authentication scheme."""
    if approval.get("approved") is not True or approval.get("purpose") != "COMPARATIVE_MEASUREMENT":
        raise PermissionError("Measurement Freeze Review has not approved comparison")
    if approval.get("fingerprint") != manifest.get("fingerprint"):
        raise PermissionError("approval is for a different manifest")
    current = inventory(root)
    if current != manifest.get("files") or fingerprint(current) != manifest.get("fingerprint"):
        raise PermissionError("source, fixture or measurement contract changed after freeze")
    if not manifest.get("cargo_lock_present") or "prototype/gate2/Cargo.lock" not in current:
        raise PermissionError("Cargo.lock must be included before comparable execution")
    if real_model:
        profile = approval.get("model_execution_profile")
        if not isinstance(profile, dict):
            raise PermissionError("real-model execution needs a frozen model_execution_profile")
        kind = profile.get("kind")
        if kind == "OPENROUTER_HOSTED_REFERENCE":
            if any(not profile.get(key) for key in ("model_id", "base_url", "provider")):
                raise PermissionError("hosted model profile is incomplete")
            if profile.get("allow_fallbacks") is not False:
                raise PermissionError("hosted comparative measurement must disable provider fallbacks")
            if profile.get("target_pc_absolute_latency_claim") is not False:
                raise PermissionError("hosted reference latency cannot be target-PC absolute latency")
        elif kind == "LOCAL_GGUF":
            if any(not profile.get(key) for key in ("model_sha256", "tokenizer_sha256", "runtime_build_id")):
                raise PermissionError("local model execution needs frozen identities")
        else:
            raise PermissionError("unsupported model execution profile")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=False)
    manifest = candidate_manifest(args.root)
    for name, content in (("source-manifest.json", manifest), ("environment.json", host_manifest(args.root))):
        (args.out / name).write_text(json.dumps(content, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": "REVIEW_BUNDLE_ONLY", "fingerprint": manifest["fingerprint"],
                      "files": len(manifest["files"]), "cargo_lock_present": manifest["cargo_lock_present"],
                      "comparative_benchmark": "NOT_RUN"}, indent=2))


if __name__ == "__main__":
    main()
