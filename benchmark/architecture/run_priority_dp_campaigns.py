#!/usr/bin/env python3
"""Run frozen official reference campaigns for VIA-DP-02/05/09/12/13."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from typing import Any

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO))
sys.path.insert(0, str(REPO / "benchmark" / "architecture"))

from analyze_priority_dp_campaign import analyze, canonical_digest, render_report
from prototypes.candidates.reference_harness.candidates import run_candidate


CONTRACT_DIR = REPO / "benchmark" / "architecture" / "contracts"
RESULT_ROOT = REPO / "results" / "architecture-evaluation" / "current"
ANALYZER = REPO / "benchmark" / "architecture" / "analyze_priority_dp_campaign.py"


def _digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(contract_path: Path) -> Path:
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    result_dir = RESULT_ROOT / contract["campaign_id"]
    if result_dir.exists():
        raise RuntimeError(f"refusing to overwrite campaign: {result_dir}")
    (result_dir / "raw").mkdir(parents=True)
    shutil.copy2(contract_path, result_dir / "contract.json")
    raw_path = result_dir / "raw" / "trials.jsonl"
    with tempfile.TemporaryDirectory(prefix=f"{contract['dp'].lower()}-") as temporary:
        temp_root = Path(temporary)
        with raw_path.open("w", encoding="utf-8") as raw:
            for case in contract["cases"]:
                for trial in range(1, contract["repetitions"] + 1):
                    for candidate in contract["candidates"]:
                        work_dir = temp_root / candidate / case["id"] / f"trial-{trial:02d}"
                        result = run_candidate(contract["dp"], candidate, case, work_dir)
                        result["trial"] = trial
                        raw.write(json.dumps(result, sort_keys=True) + "\n")
                        raw.flush()

    summary = analyze(result_dir)
    core_digest = canonical_digest(summary)
    (result_dir / "core-summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    (result_dir / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    (result_dir / "report.md").write_text(render_report(result_dir, summary), encoding="utf-8")
    replay = subprocess.run(
        [str(REPO / ".venv" / "bin" / "python"), str(ANALYZER), str(result_dir), "--verify", core_digest],
        check=False, capture_output=True, text=True,
    )
    receipt = {
        "reproduced": replay.returncode == 0,
        "expected_core_digest": core_digest,
        "stdout": replay.stdout.strip(),
        "stderr": replay.stderr.strip(),
    }
    (result_dir / "replay-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    if replay.returncode != 0:
        raise RuntimeError(f"independent replay failed: {replay.stderr}")
    manifest: dict[str, Any] = {
        "campaign_id": contract["campaign_id"],
        "dp": contract["dp"],
        "status": "COMPLETE",
        "evidence_label": "MEASURED_REFERENCE_HARNESS",
        "contract_sha256": _digest(result_dir / "contract.json"),
        "candidate_source_sha256": _digest(REPO / "prototypes" / "candidates" / "reference_harness" / "candidates.py"),
        "analyzer_source_sha256": _digest(ANALYZER),
        "raw_sha256": _digest(raw_path),
        "core_summary_sha256": _digest(result_dir / "core-summary.json"),
        "replay_verified": True,
        "git_head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip(),
        "source_tree_note": "campaign source is fixed by candidate/analyzer SHA-256 when git_head predates the campaign commit",
        "target": "current Mac reference harness",
        "shared_models": "one S2S model and one semantic LLM; neither is replicated; these targeted deterministic paths do not invoke model inference",
    }
    (result_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    (result_dir / "STATUS.md").write_text(
        f"# {contract['campaign_id']} status\n\n**COMPLETE** — raw, independent replay, 19-QA table and manifest are present.\n",
        encoding="utf-8",
    )
    return result_dir


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dp", action="append", choices=["02", "05", "09", "12", "13"])
    args = parser.parse_args()
    selected = args.dp or ["05", "09", "02", "13", "12"]
    for number in selected:
        version = "v2" if number == "02" else "v1"
        path = CONTRACT_DIR / f"via-dp-{number}-evaluation-{version}.json"
        result = run(path)
        print(result.relative_to(REPO))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
