#!/usr/bin/env python3
"""Combine the two frozen campaigns without pretending they share one fingerprint."""
from __future__ import annotations

import argparse
import json
from pathlib import Path

DP_METRICS = {
    "IR-DP01": ("W-01", "W-02", "W-05", "W-08"),
    "TASK-DP01": ("W-02", "W-04", "W-08", "W-09"),
    "AGENT-DP01": ("W-02", "W-03", "W-07", "W-08"),
    "EXEC-DP01": ("W-01", "W-04", "W-09", "W-10"),
}
CONFIG = {("IR-DP01", "A"): "AAAA", ("IR-DP01", "B"): "BAAA", ("TASK-DP01", "A"): "AAAA", ("TASK-DP01", "B"): "ABAA", ("AGENT-DP01", "A"): "AAAA", ("AGENT-DP01", "B"): "AABA", ("EXEC-DP01", "A"): "AAAA", ("EXEC-DP01", "B"): "AAAB"}


def index_rows(payload, metric):
    return {row["configuration_id"]: row for row in payload["metrics"][metric]["rows"]}


def build(reference, changes, representative):
    decisions = {}
    for dp, metrics in DP_METRICS.items():
        rows, preferences = [], []
        for wid in metrics:
            if wid == "W-05":
                rows.append({"working_asr": wid, "status": "BLOCKED_OPENROUTER_KEY", "candidates": None, "g3": False})
                continue
            if wid in {"W-01", "W-02", "W-03", "W-04"}:
                indexed = index_rows(reference, wid)
                values = {c: indexed[CONFIG[(dp, c)]] for c in "AB"}
                evidence = values["A"]["evidence"]
            elif wid in {"W-07", "W-08"}:
                values = {c: changes["change_metrics"][f"{dp}/{c}"][wid] for c in "AB"}
                evidence = "DESIGN_ANALYSIS"
            else:
                indexed = index_rows(representative, wid)
                values = {c: indexed[CONFIG[(dp, c)]] for c in "AB"}
                evidence = "MEASURED_STRUCTURAL"
            candidates = {c: {"metric_value": values[c]["metric_value"], "score": values[c]["score"]} for c in "AB"}
            g3 = candidates["A"]["score"] != candidates["B"]["score"]
            if g3:
                preferences.append(max(candidates, key=lambda c: candidates[c]["score"]))
            rows.append({"working_asr": wid, "status": "MEASURED", "evidence": evidence, "candidates": candidates, "g3": g3})
        selected = preferences[0] if preferences and len(set(preferences)) == 1 else None
        decisions[dp] = {"metrics": rows, "primary_metrics": [r["working_asr"] for r in rows if r["g3"]], "selected_candidate": selected, "status": "ACCEPTED" if selected else "DEFERRED"}
    system = {wid: reference["metrics"][wid]["rows"] for wid in ("W-06", "W-11", "W-12")}
    return {"version": "G2-COMBINED-SWEEP-v1", "source_fingerprints": {"reference": reference["freeze_fingerprint"], "structural": representative["freeze_fingerprint"]}, "weighted_total": None, "global_winner": None, "decisions": decisions, "system_regressions": system}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ("reference", "changes", "representative", "output"):
        p.add_argument(f"--{name}", type=Path, required=True)
    a = p.parse_args()
    load = lambda path: json.loads(path.read_text(encoding="utf-8"))
    result = build(load(a.reference), load(a.changes), load(a.representative))
    a.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(a.output)


if __name__ == "__main__":
    main()
