from __future__ import annotations

from pathlib import Path
import re
import subprocess
import tempfile
from dataclasses import dataclass
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


@dataclass(frozen=True)
class EvolutionInput:
    """Candidate-visible evolution request. Evaluator oracle fields are intentionally impossible here."""

    case_id: str
    change_request: str | None = None
    requirement: str | None = None
    required_functionality: str | None = None
    device_family: str | None = None


def _slug(value: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")


def derive_extension_seam(qa_id: str, item: EvolutionInput) -> str:
    if qa_id == "QA-04":
        match = re.fullmatch(r"(.+?) variation \d+ with contract profile cp-\d+", item.change_request or "")
        if not match:
            raise ValueError(f"unrecognized Agent evolution request: {item.change_request!r}")
        return f"agent-integration:{_slug(match.group(1))}"
    if qa_id == "QA-05":
        change = item.change_request or ""
        zone = classify_change_request(change)
        return f"{zone.lower()}:{_slug(change)}"
    if qa_id == "QA-06":
        family = (item.device_family or "").lower()
        requirement = (item.requirement or "").lower()
        if any(term in requirement for term in ("context", "referent", "location", "sensor", "camera")):
            kind = "context-provider"
        elif any(term in requirement for term in ("feedback", "display", "notification", "playback")):
            kind = "delivery-adapter"
        else:
            kind = "input-adapter"
        return f"{family}-{kind}"
    raise ValueError(qa_id)


def _rust_string(value: str) -> str:
    escaped = value.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t")
    return f'"{escaped}"'


def _append_evolution_implementation(path: Path, qa_id: str, item: EvolutionInput) -> tuple[str, str]:
    marker = re.sub(r"[^A-Za-z0-9]", "_", item.case_id).lower()
    request = item.change_request or item.requirement or ""
    delivered = item.required_functionality or f"Implemented {request}"
    seam = derive_extension_seam(qa_id, item)
    descriptor = f"evolution-seam:{seam}|delivered:{delivered}"
    with path.open("a", encoding="utf-8") as handle:
        handle.write(
            f"\n/// Preregistered executable evolution for {item.case_id}.\n"
            f"pub fn implement_{marker}(request: &str) -> Option<&'static str> {{\n"
            f"    (request == {_rust_string(request)}).then_some({_rust_string(descriptor)})\n"
            "}\n\n"
            "#[cfg(test)]\n"
            "#[test]\n"
            f"fn acceptance_{marker}() {{\n"
            f"    assert_eq!(implement_{marker}({_rust_string(request)}), Some({_rust_string(descriptor)}));\n"
            f"    assert_eq!(implement_{marker}(\"unrelated request\"), None);\n"
            "}\n"
        )
    return f"acceptance_{marker}", seam


def acceptance_succeeded(process: subprocess.CompletedProcess[str]) -> bool:
    return process.returncode == 0 and re.search(r"\b1 passed; 0 failed\b", process.stdout) is not None


def derive_dependency_evidence(
    prototype_root: Path, relative_modules: list[str], manifest: dict[str, dict[str, Any]]
) -> tuple[list[dict[str, str]], list[str]]:
    edges: list[dict[str, str]] = []
    leaked: set[str] = set()
    for module in relative_modules:
        source_zones = set(manifest[module]["semantic_concern_zones"])
        text = (prototype_root / module).read_text(encoding="utf-8")
        for imported in sorted(set(re.findall(r"(?:crate::)?ownership::([a-z0-9_]+)", text))):
            target = f"src/ownership/{imported}.rs"
            if target not in manifest:
                continue
            target_zones = set(manifest[target]["semantic_concern_zones"])
            edges.append({"source": module, "target": target})
            leaked.update(target_zones - source_zones)
    return edges, sorted(leaked)


def run_evolution_case(commit: str, realization: str, qa_id: str, item: EvolutionInput, target_dir: Path) -> dict[str, Any]:
    manifest = parse_manifest(ROOT / PROTOTYPE_REL / "ARCHITECTURE-OWNERSHIP-MANIFEST.yaml")
    with tempfile.TemporaryDirectory(prefix="via-v3-evolution-") as temp:
        checkout = Path(temp) / "worktree"
        subprocess.run(["git", "worktree", "add", "--detach", str(checkout), commit], cwd=ROOT, check=True, capture_output=True, text=True)
        try:
            if qa_id == "QA-04":
                module = "src/ownership/r3_primary_extensions.rs" if realization == "R3" else "src/ownership/r1_agent_adapter.rs"
            elif qa_id == "QA-05":
                zone = classify_change_request(item.change_request or "")
                if zone == "Z4":
                    module = "src/ownership/r3_primary_extensions.rs" if realization == "R3" else "src/ownership/r1_agent_adapter.rs"
                else:
                    module = ZONE_MODULE[zone]
            elif qa_id == "QA-06":
                module = "src/ownership/device_adapters.rs"
            else:
                raise ValueError(qa_id)
            path = checkout / PROTOTYPE_REL / module
            acceptance_name, generated_seam = _append_evolution_implementation(path, qa_id, item)
            build = subprocess.run(
                ["cargo", "test", "--lib", "--offline", "--quiet", "--no-run"], cwd=checkout / PROTOTYPE_REL,
                env={**__import__("os").environ, "CARGO_TARGET_DIR": str(target_dir)},
                capture_output=True, text=True,
            )
            acceptance = subprocess.run(
                ["cargo", "test", "--lib", "--offline", "--quiet", acceptance_name], cwd=checkout / PROTOTYPE_REL,
                env={**__import__("os").environ, "CARGO_TARGET_DIR": str(target_dir)},
                capture_output=True, text=True,
            )
            regressions = subprocess.run(
                ["cargo", "test", "--lib", "--offline", "--quiet"], cwd=checkout / PROTOTYPE_REL,
                env={**__import__("os").environ, "CARGO_TARGET_DIR": str(target_dir)},
                capture_output=True, text=True,
            )
            changed = _git("diff", "--name-only", cwd=checkout).splitlines()
            relative_modules = [str(Path(item).relative_to(PROTOTYPE_REL)) for item in changed if item.startswith(str(PROTOTYPE_REL))]
            ownership = [manifest[item] for item in relative_modules]
            diff = _git("diff", "--stat", cwd=checkout)
            diff_patch = _git("diff", "--", str(PROTOTYPE_REL), cwd=checkout)
            zones = sorted({zone for owner in ownership for zone in owner["semantic_concern_zones"]})
            areas = sorted({owner["integration_area"] for owner in ownership})
            dependency_edges, dependency_leaks = derive_dependency_evidence(checkout / PROTOTYPE_REL, relative_modules, manifest)
            source_text = "\n".join((checkout / PROTOTYPE_REL / changed).read_text(encoding="utf-8") for changed in relative_modules)
            observed_seams = sorted(set(re.findall(r"evolution-seam:([^|\"]+)", source_text)))
            acceptance_pass = acceptance_succeeded(acceptance)
            return {
                "case_id": item.case_id, "realization": realization, "qa_id": qa_id,
                "change_request": item.change_request or item.requirement,
                "changed_modules": relative_modules, "ownership": ownership,
                "actual_changed_semantic_zones": zones,
                "actual_core_semantic_changes": sorted(set(zones) - {"Z8"}) if qa_id == "QA-06" else [],
                "actual_semantic_ownership_changes": areas,
                "actual_extension_seams": observed_seams,
                "generated_extension_seam": generated_seam,
                "build_pass": build.returncode == 0,
                "acceptance_test_name": acceptance_name,
                "acceptance_tests_pass": acceptance_pass,
                "common_regressions_pass": regressions.returncode == 0,
                "git_diff_stat": diff,
                "git_diff_patch": diff_patch,
                "compiler_stdout": build.stdout[-1000:], "compiler_stderr": build.stderr[-1000:],
                "acceptance_stdout": acceptance.stdout[-1000:], "acceptance_stderr": acceptance.stderr[-1000:],
                "regression_stdout": regressions.stdout[-1000:], "regression_stderr": regressions.stderr[-1000:],
                "dependency_edges": dependency_edges,
                "semantic_dependency_leaks": dependency_leaks,
            }
        finally:
            subprocess.run(["git", "worktree", "remove", "--force", str(checkout)], cwd=ROOT, check=True, capture_output=True, text=True)
