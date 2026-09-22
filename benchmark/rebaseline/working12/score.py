"""Working-12 scoring contract.

NOT_RUN/null remains unscored. Final scoring requires an approved Measurement Freeze
fingerprint. W-07/W-08 aggregation is performed here, not in the pre-freeze raw analyzer.
"""
from __future__ import annotations
import argparse, json, math, sys
from pathlib import Path
from typing import Any

ROOT=Path(__file__).resolve().parents[3]
BASELINE_PATH=ROOT/"benchmark/rebaseline/working12/baseline.json"
sys.path.insert(0,str(ROOT/"benchmark/rebaseline/readiness"))
from freeze import authorize_run  # noqa: E402

WORKING_IDS=tuple(f"W-{i:02d}" for i in range(1,13))

def load_baseline(path:Path=BASELINE_PATH)->dict[str,Any]:
    value=json.loads(path.read_text(encoding="utf-8"));validate_baseline(value);return value

def validate_baseline(baseline:dict[str,Any])->None:
    metrics=baseline.get("metrics")
    if set(metrics or {})!=set(WORKING_IDS): raise ValueError("Working-12 baseline must contain exactly W-01..W-12")
    if baseline.get("not_run_is_score") is not False: raise ValueError("NOT_RUN must never become score 0")
    for wid in WORKING_IDS:
        spec=metrics[wid]
        if spec.get("direction") not in {"lower","higher"}: raise ValueError(f"{wid}: invalid direction")
        bands=spec.get("bands") or []
        if [b.get("score") for b in bands]!=[5,4,3,2,1,0]: raise ValueError(f"{wid}: bands must be 5..0")
        key="max" if spec["direction"]=="lower" else "min"
        if bands[-1].get(key) is not None: raise ValueError(f"{wid}: score-0 boundary must be null")

def score_value(value:float|int|None,spec:dict[str,Any])->int|None:
    if value is None:return None
    if isinstance(value,bool) or not isinstance(value,(int,float)) or not math.isfinite(float(value)):
        raise ValueError("metric value must be finite or null")
    x=float(value)
    for band in spec["bands"]:
        boundary=band["max"] if spec["direction"]=="lower" else band["min"]
        if boundary is None or (x<=float(boundary) if spec["direction"]=="lower" else x>=float(boundary)):
            return int(band["score"])
    raise AssertionError("unreachable")

def score_metrics(metrics:dict[str,Any],baseline:dict[str,Any]|None=None)->dict[str,Any]:
    baseline=baseline or load_baseline()
    unknown=set(metrics)-set(WORKING_IDS)
    if unknown: raise ValueError(f"unknown Working ASR ids: {sorted(unknown)}")
    rows=[]
    for wid in WORKING_IDS:
        spec=baseline["metrics"][wid];value=metrics.get(wid)
        rows.append({"working_asr":wid,"metric":spec["metric"],"metric_value":value,
          "score":score_value(value,spec),"target":spec["target"],"unit":spec["unit"],
          "status":"NOT_RUN" if value is None else "SCORED"})
    return {"baseline_version":baseline["version"],"rows":rows,"winner":None}

def aggregate_change_rows(rows:list[dict[str,Any]])->dict[str,dict[str,float]]:
    by={}
    for row in rows:by.setdefault(row["candidate_id"],[]).append(row)
    out={}
    for candidate,crows in sorted(by.items()):
        agent=[r for r in crows if r.get("family")=="A"]
        non=[r for r in crows if r.get("family") in {"M","C"}]
        if len(agent)!=9 or len(non)!=15: raise ValueError(f"{candidate}: expected 9 Agent and 15 non-Agent rows")
        if any(not isinstance(r.get("unique_count"),int) for r in agent+non): raise ValueError(f"{candidate}: missing unique_count")
        out[candidate]={"W-07":sum(r["unique_count"] for r in agent)/9,
                        "W-08":sum(r["unique_count"] for r in non)/15}
    return out

def require_freeze(manifest_path:Path,approval_path:Path):
    manifest=json.loads(manifest_path.read_text());approval=json.loads(approval_path.read_text())
    authorize_run(manifest,approval,ROOT);return manifest,approval

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--validate-only",action="store_true");p.add_argument("--metrics",type=Path)
    p.add_argument("--change-analysis",type=Path);p.add_argument("--manifest",type=Path);p.add_argument("--approval",type=Path);p.add_argument("--output",type=Path)
    a=p.parse_args();baseline=load_baseline()
    if a.validate_only:
        print(json.dumps({"status":"VALID","version":baseline["version"],"metrics":12}));return
    if a.manifest is None or a.approval is None:p.error("scoring requires --manifest and --approval")
    manifest,_=require_freeze(a.manifest,a.approval);payload={"freeze_fingerprint":manifest["fingerprint"],"winner":None}
    if a.metrics:
        values=json.loads(a.metrics.read_text())
        if not isinstance(values,dict):raise ValueError("--metrics must be object")
        payload["working12"]=score_metrics(values,baseline)
    if a.change_analysis:
        raw=json.loads(a.change_analysis.read_text());agg=aggregate_change_rows(raw["raw_rows"])
        payload["change_metrics"]={c:{w:{"metric_value":v,"score":score_value(v,baseline["metrics"][w])} for w,v in vals.items()} for c,vals in agg.items()}
    if not a.metrics and not a.change_analysis:p.error("provide --metrics and/or --change-analysis")
    if a.output is None:p.error("--output required")
    a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_text(json.dumps(payload,ensure_ascii=False,indent=2)+"\n");print(a.output)
if __name__=="__main__":main()
