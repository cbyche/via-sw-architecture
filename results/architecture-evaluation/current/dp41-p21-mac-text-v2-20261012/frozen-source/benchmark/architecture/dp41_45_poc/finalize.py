"""Finalize complete raw-linked QA tables and reusable CSV observations."""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
from pathlib import Path
from statistics import mean

from .evaluate import read_jsonl
from .metrics import aggregate
from .normalize import normalize_trial


def dump(path, value):
    path.write_text(json.dumps(value,ensure_ascii=False,indent=2)+"\n")


def csv_rows(path, rows):
    keys=list(dict.fromkeys(k for row in rows for k in row))
    with path.open("w",newline="") as f:
        writer=csv.DictWriter(f,fieldnames=keys);writer.writeheader()
        for row in rows:
            writer.writerow({k:json.dumps(v,ensure_ascii=False) if isinstance(v,(list,dict)) else v for k,v in row.items()})


def finalize(campaign):
    campaign=campaign.resolve(); output=campaign/"final";output.mkdir(exist_ok=True)
    manifest_path=campaign/"evaluation-manifest.json"
    manifest=json.loads((manifest_path if manifest_path.exists() else campaign/"manifest.json").read_text())
    adjudications=read_jsonl(campaign/"evaluation/adjudications.jsonl")
    overrides=read_jsonl(campaign/"evaluation/trace-adjudication-overrides.jsonl") if (campaign/"evaluation/trace-adjudication-overrides.jsonl").exists() else []
    keyed={(a['variant'],a['repeat'],a['case_id']):a for a in adjudications}
    for override in overrides:
        keyed[(override['variant'],override['repeat'],override['case_id'])]=override
    trials=[]; calculations=[]; memory=[]; calls=[]; conditions=[]
    for worker in sorted((campaign/"workers").iterdir()):
        for raw in read_jsonl(worker/"trials-unjudged.jsonl"):
            key=(raw['variant'],raw['repeat'],raw['case_id']);judged=keyed[key]
            normalized,dag=normalize_trial(raw,judged);trials.append(normalized)
            calculations.extend({'variant':key[0],'repeat':key[1],'case_id':key[2],**d} for d in dag)
            conditions.extend({'variant':key[0],'repeat':key[1],'case_id':key[2],'condition':c,**value} for c,value in normalized['conditions'].items())
        memory += read_jsonl(worker/"memory.jsonl")
        calls += read_jsonl(worker/"model-calls.jsonl")
    calls += read_jsonl(campaign/"evaluation/model-calls.jsonl")
    changes=[json.loads(p.read_text()) for p in sorted((campaign/"changes").glob('CH-*/reviewed-result.json'))]
    result=aggregate(trials,memory,changes,calls,manifest)
    for variant,r in result['variants'].items():
        if not all(r['numeric_completion'].values()):
            raise ValueError(f"Incomplete final numbers for {variant}: {r['numeric_completion']}")
        if any(r[f'qa{i:02d}']['score'] is None for i in range(1,7)):
            raise ValueError('Missing final score')
    dump(output/'qa-results.json',result)
    with (output/'normalized-trials.jsonl').open('w') as f:
        for t in trials:f.write(json.dumps(t,ensure_ascii=False)+'\n')
    with (output/'causal-attribution.jsonl').open('w') as f:
        for t in calculations:f.write(json.dumps(t,ensure_ascii=False)+'\n')
    csv_rows(output/'conditions.csv',conditions)
    csv_rows(output/'reactions.csv',[r for v in result['variants'].values() for r in v['qa02']['observations']])
    csv_rows(output/'cycles.csv',[r for v in result['variants'].values() for r in v['qa03']['observations']])
    csv_rows(output/'changes.csv',[{k:r[k] for k in ('job_id','variant','task_id','changed_unit_set','count','baseline_source_hash','patch_ref','result_ref')} for r in changes])
    csv_rows(output/'model-calls.csv',[{k:c.get(k) for k in ('provider','variant','repeat','case_id','request_id','call_id','role','purpose','attempt','status','discard_reason','usage','cost_usd','enqueue_ns','send_ns','terminal_ns','context','output')} for c in calls])
    csv_rows(output/'repeat-results.csv',[{'variant':v,**r} for v,item in result['variants'].items() for r in item['per_repeat']])
    dump(output/'completion.json',{'status':'COMPLETE','expected_trials':210,'observed_trials':len(trials),'conditions':len(conditions),'reactions':sum(len(t['reactions']) for t in trials),'cycles':sum(t['cycle'] is not None for t in trials),'change_jobs':len(changes),'source_digest':manifest['frozen_source_digest']})
    return result


if __name__ == '__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--campaign',type=Path,required=True)
    result=finalize(parser.parse_args().campaign)
    print(json.dumps({v:{f'qa{i:02d}':r[f'qa{i:02d}']['score'] for i in range(1,7)} for v,r in result['variants'].items()}))
