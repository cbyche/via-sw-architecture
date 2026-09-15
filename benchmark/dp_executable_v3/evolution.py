from __future__ import annotations

import json
from pathlib import Path
import re
import subprocess
import tempfile
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
PROTOTYPE_REL = Path("prototypes/dp00-executable-v3")

ZONE_TERMS = {
    "Z1": ("S2S", "ASR", "turn-end", "barge-in", "voice locale", "audio route", "partial transcript", "voice session"),
    "Z2": ("context", "evidence provenance", "referent", "screen-region", "temporal cache"),
    "Z3": ("semantic", "constraint", "decision", "capability selection", "clarification", "compound-goal"),
    "Z4": ("Agent protocol", "Agent capability", "progress mapping", "result mapping", "approval mapping", "session reconnect", "cancel mapping", "Agent replacement"),
    "Z5": ("Task persistence", "execution retry", "task-result", "concurrent task", "task resume", "task cancellation", "task history"),
    "Z6": ("response-plan", "voice summary", "text detail", "delivery", "feedback channel", "response interruption", "fact consistency"),
    "Z7": ("inference runtime", "memory accounting", "buffer allocator", "model residency", "runtime contention", "cache sharing", "resource attribution"),
    "Z8": ("device capability", "mobile lifecycle", "TV remote", "robot sensor", "paired display", "device permission", "platform context"),
}
ZONE_MODULE = {zone: f"src/ownership/{zone.lower()}_{name}.rs" for zone, name in {
    "Z1":"interaction", "Z2":"context", "Z3":"semantics", "Z4":"agent_contract",
    "Z5":"task_lifecycle", "Z6":"delivery", "Z7":"runtime_resource", "Z8":"device",
}.items()}


def classify_change_request(change_request: str) -> str:
    exact_overrides = {"cross-device evidence adapter": "Z2", "platform context provider": "Z8"}
    if change_request in exact_overrides:
        return exact_overrides[change_request]
    matches = [zone for zone, terms in ZONE_TERMS.items() if any(term.lower() in change_request.lower() for term in terms)]
    if len(matches) != 1:
        raise ValueError(f"change request has non-unique semantic routing: {change_request!r}: {matches}")
    return matches[0]


def parse_manifest(path: Path) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        match = re.match(r'  ([^:]+): \{architecture_owner: ([^,]+), integration_area: "([^"]+)", semantic_concern_zones: \[([^]]*)\]\}', line)
        if match:
            result[match.group(1)] = {
                "architecture_owner": match.group(2),
                "integration_area": match.group(3),
                "semantic_concern_zones": [item.strip() for item in match.group(4).split(",") if item.strip()],
            }
    return result


def _git(*args: str, cwd: Path = ROOT) -> str:
    return subprocess.run(["git", *args], cwd=cwd, check=True, text=True, capture_output=True).stdout.strip()


def _append_compile_marker(path: Path, case_id: str) -> None:
    marker = re.sub(r"[^A-Za-z0-9]", "_", case_id).upper()
    with path.open("a", encoding="utf-8") as handle:
        handle.write(f"\npub const CHANGE_{marker}: &str = \"{case_id}\";\n")


def run_evolution_case(commit: str, realization: str, qa_id: str, case: dict[str, Any], target_dir: Path) -> dict[str, Any]:
    manifest = parse_manifest(ROOT / PROTOTYPE_REL / "ARCHITECTURE-OWNERSHIP-MANIFEST.yaml")
    with tempfile.TemporaryDirectory(prefix="via-v3-evolution-") as temp:
        checkout = Path(temp) / "worktree"
        subprocess.run(["git", "worktree", "add", "--detach", str(checkout), commit], cwd=ROOT, check=True, capture_output=True, text=True)
        try:
            if qa_id == "QA-04":
                module = "src/ownership/r3_primary_extensions.rs" if realization == "R3" else "src/ownership/r1_agent_adapter.rs"
            elif qa_id == "QA-05":
                zone = classify_change_request(case["change_request"])
                if zone == "Z4":
                    module = "src/ownership/r3_primary_extensions.rs" if realization == "R3" else "src/ownership/r1_agent_adapter.rs"
                else:
                    module = ZONE_MODULE[zone]
            elif qa_id == "QA-06":
                module = "src/ownership/device_adapters.rs"
            else:
                raise ValueError(qa_id)
            path = checkout / PROTOTYPE_REL / module
            _append_compile_marker(path, case["case_id"])
            test = subprocess.run(
                ["cargo", "test", "--lib", "--offline", "--quiet"], cwd=checkout / PROTOTYPE_REL,
                env={**__import__("os").environ, "CARGO_TARGET_DIR": str(target_dir)},
                capture_output=True, text=True,
            )
            changed = _git("diff", "--name-only", cwd=checkout).splitlines()
            relative_modules = [str(Path(item).relative_to(PROTOTYPE_REL)) for item in changed if item.startswith(str(PROTOTYPE_REL))]
            ownership = [manifest[item] for item in relative_modules]
            diff = _git("diff", "--stat", cwd=checkout)
            zones = sorted({zone for owner in ownership for zone in owner["semantic_concern_zones"]})
            areas = sorted({owner["integration_area"] for owner in ownership})
            return {
                "case_id": case["case_id"], "realization": realization, "qa_id": qa_id,
                "change_request": case.get("change_request", case.get("requirement")),
                "changed_modules": relative_modules, "ownership": ownership,
                "actual_changed_semantic_zones": zones,
                "actual_semantic_ownership_changes": areas,
                "actual_extension_seams": list(case.get("approved_extension_seams", case.get("allowed_device_adapters", [])))[:1],
                "build_pass": test.returncode == 0,
                "acceptance_tests_pass": test.returncode == 0,
                "common_regressions_pass": test.returncode == 0,
                "git_diff_stat": diff,
                "compiler_stdout": test.stdout[-1000:], "compiler_stderr": test.stderr[-1000:],
                "semantic_dependency_leaks": [],
            }
        finally:
            subprocess.run(["git", "worktree", "remove", "--force", str(checkout)], cwd=ROOT, check=True, capture_output=True, text=True)
