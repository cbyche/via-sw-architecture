"""Finalize complete raw-linked QA tables and reusable CSV observations."""
from __future__ import annotations

import argparse
from collections import Counter
import csv
import hashlib
import json
from pathlib import Path
from statistics import mean

from .evaluate import read_jsonl
from .metrics import aggregate
from .normalize import normalize_trial


def selected_change_records(campaign):
    """Resolve a hash-pinned choice without double-counting preserved alternatives."""
    selection_path = campaign / "qa05-selection-manifest.json"
    if not selection_path.exists():
        return [json.loads(p.read_text()) for p in sorted((campaign / "changes").glob('CH-*/reviewed-result.json'))]
    selection = json.loads(selection_path.read_text())
    manifest = json.loads((campaign / "manifest.json").read_text())
    if selection.get("frozen_source_digest") != manifest["frozen_source_digest"]:
        raise ValueError("QA05 selection belongs to a different measured baseline")
    from .change_experiments import TASKS, source_snapshot, snapshot_digest
    baseline = (campaign / "frozen-source").resolve()
    baseline_hash = snapshot_digest(source_snapshot(baseline))
    expected = {(variant, task) for variant in ("A", "B") for task in TASKS}
    seen, records = set(), []
    for selected in selection["selected"]:
        path = (campaign / selected["reviewed_result_ref"]).resolve()
        if not path.is_relative_to(campaign.resolve()) or not path.is_file():
            raise ValueError("QA05 selected result must be an existing campaign artifact")
        if hashlib.sha256(path.read_bytes()).hexdigest() != selected["reviewed_result_sha256"]:
            raise ValueError("QA05 selected reviewed result changed")
        record = json.loads(path.read_text())
        key = (record["variant"], record["task_id"])
        if (selected["variant"], selected["task_id"]) != key or Path(record["result_ref"]).resolve() != path:
            raise ValueError("QA05 selection identity or result reference differs")
        if key not in expected or key in seen:
            raise ValueError("Duplicate or unexpected QA05 selected job")
        seen.add(key)
        if record.get("status") != "COMPLETED" or not all(record.get(flag) is True for flag in (
            "completion_passed", "regression_passed", "patch_verified", "responsibility_review_passed", "original_baseline_unchanged")):
            raise ValueError("QA05 selected job lacks completed execution and review")
        if Path(record["baseline_source_path"]).resolve() != baseline or record["baseline_source_hash"] != baseline_hash:
            raise ValueError("QA05 selected job baseline differs from measured source")
        if snapshot_digest(source_snapshot(path.parent / "source")) != record["after_source_hash"]:
            raise ValueError("QA05 selected changed source differs from reviewed source")
        if hashlib.sha256(Path(record["patch_ref"]).read_bytes()).hexdigest() != record["patch_sha256"]:
            raise ValueError("QA05 selected patch changed")
        records.append(record)
    if seen != expected:
        raise ValueError("QA05 selection must contain one completed job per variant/task")
    return records


def dump(path, value):
    path.write_text(json.dumps(value,ensure_ascii=False,indent=2)+"\n")


def csv_rows(path, rows):
    keys=list(dict.fromkeys(k for row in rows for k in row))
    with path.open("w",newline="") as f:
        writer=csv.DictWriter(f,fieldnames=keys);writer.writeheader()
        for row in rows:
            writer.writerow({k:json.dumps(v,ensure_ascii=False) if isinstance(v,(list,dict)) else v for k,v in row.items()})


def mechanism_rows(raw_trials, calls):
    """Preserve the executed Request paths without interpreting them as wins."""
    paths=[]; call_links={}
    for trial in raw_trials:
        events=trial['events']
        request_ids=sorted({e['request_id'] for e in events if e.get('request_id')})
        for request_id in request_ids:
            request_events=[e for e in events if e.get('request_id')==request_id]
            model_requests=[e for e in request_events if e['kind']=='dp41_model_requested']
            request_calls=[c for c in calls if c.get('request_id')==request_id and c.get('variant')==trial['variant']
                and c.get('repeat')==trial['repeat'] and c.get('case_id')==trial['case_id']]
            discarded=[e for e in request_events if e['kind'] in ('MEANING_DISCARDED','MODEL_DISCARDED')]
            for call in request_calls:
                earlier=[e for e in model_requests if e['monotonic_ns']<=call['enqueue_ns']]
                logical=max(earlier,key=lambda e:e['monotonic_ns']) if earlier and call['role']=='CINTERPRET' else None
                call_links[call['call_id']]={'dp41_purpose':logical['purpose'] if logical else None,
                    'logical_model_ordinal':logical['ordinal'] if logical else None,
                    'resolution_disposition':[{'kind':e['kind'],'reason':e.get('reason'),'evidence_id':e['evidence_id']} for e in discarded]}
            paths.append({'variant':trial['variant'],'repeat':trial['repeat'],'case_id':trial['case_id'],
                'request_id':request_id,'model_call_ids':[c['call_id'] for c in request_calls],
                'logical_model_path':[{'purpose':e['purpose'],'ordinal':e['ordinal'],'evidence_id':e['evidence_id']} for e in model_requests],
                'reads':[{'spec':e['spec'],'result':e['result'],'evidence_id':e['evidence_id']} for e in request_events if e['kind']=='dp41_read_returned'],
                'actions':[{'output':e['output'],'evidence_id':e['evidence_id']} for e in request_events if e['kind']=='dp41_a_model_action'],
                'field_states':[{'field_id':e['field_id'],'state':e['state'],'binding':e.get('binding'),'evidence_id':e['evidence_id']} for e in request_events if e['kind']=='dp41_field_state'],
                'clarifications':[e for e in request_events if e['kind'] in ('dp41_clarification_requested','dp41_clarification_returned')],
                'rejections':[e for e in request_events if e['kind'] in ('dp41_output_rejected','dp41_resolution_failed','REQUEST_FAILED','GOAL_FAILED')],
                'discarded':discarded})
    summaries={}
    for variant in ('A','B'):
        selected=[p for p in paths if p['variant']==variant]
        purposes=Counter(e['purpose'] for p in selected for e in p['logical_model_path'])
        summaries[variant]={'logical_model_purposes':dict(purposes),
            'reads':sum(len(p['reads']) for p in selected),
            'read_cache_hits':sum(i.get('cache_hit') is True for p in selected for r in p['reads'] for i in r['result'].get('items',[])),
            'a_actions':dict(Counter(a['output'].get('action','INVALID') for p in selected for a in p['actions'])),
            'b_field_states':dict(Counter(e['state'] for p in selected for e in p['field_states'])),
            'clarification_requests':sum(e['kind']=='dp41_clarification_requested' for p in selected for e in p['clarifications']),
            'schema_rejections':sum(e['kind']=='dp41_output_rejected' for p in selected for e in p['rejections']),
            'resolution_failures':sum(e['kind']=='dp41_resolution_failed' for p in selected for e in p['rejections']),
            'discarded_meanings':sum(e['kind']=='MEANING_DISCARDED' for p in selected for e in p['discarded']),
            'discarded_model_returns':sum(e['kind']=='MODEL_DISCARDED' for p in selected for e in p['discarded'])}
    return paths,call_links,summaries


def finalize(campaign):
    campaign=campaign.resolve(); output=campaign/"final";output.mkdir(exist_ok=True)
    manifest_path=campaign/"evaluation-manifest.json"
    manifest=json.loads((manifest_path if manifest_path.exists() else campaign/"manifest.json").read_text())
    adjudications=read_jsonl(campaign/"evaluation/adjudications.jsonl")
    overrides=read_jsonl(campaign/"evaluation/trace-adjudication-overrides.jsonl") if (campaign/"evaluation/trace-adjudication-overrides.jsonl").exists() else []
    keyed={(a['variant'],a['repeat'],a['case_id']):a for a in adjudications}
    for override in overrides:
        keyed[(override['variant'],override['repeat'],override['case_id'])]=override
    trials=[]; raw_trials=[]; calculations=[]; memory=[]; calls=[]; conditions=[]
    for worker in sorted((campaign/"workers").iterdir()):
        for raw in read_jsonl(worker/"trials-unjudged.jsonl"):
            raw_trials.append(raw)
            key=(raw['variant'],raw['repeat'],raw['case_id']);judged=keyed[key]
            normalized,dag=normalize_trial(raw,judged);trials.append(normalized)
            calculations.extend({'variant':key[0],'repeat':key[1],'case_id':key[2],**d} for d in dag)
            conditions.extend({'variant':key[0],'repeat':key[1],'case_id':key[2],'condition':c,**value} for c,value in normalized['conditions'].items())
        memory += read_jsonl(worker/"memory.jsonl")
        calls += read_jsonl(worker/"model-calls.jsonl")
    calls += read_jsonl(campaign/"evaluation/model-calls.jsonl")
    paths,call_links,mechanisms=mechanism_rows(raw_trials,calls)
    changes=selected_change_records(campaign)
    result=aggregate(trials,memory,changes,calls,manifest)
    for variant,r in result['variants'].items():
        if not all(r['numeric_completion'].values()):
            raise ValueError(f"Incomplete final numbers for {variant}: {r['numeric_completion']}")
        if any(r[f'qa{i:02d}']['score'] is None for i in range(1,7)):
            raise ValueError('Missing final score')
    dump(output/'qa-results.json',result)
    dump(output/'mechanism-summary.json',mechanisms)
    with (output/'request-paths.jsonl').open('w') as f:
        for p in paths:f.write(json.dumps(p,ensure_ascii=False)+'\n')
    with (output/'normalized-trials.jsonl').open('w') as f:
        for t in trials:f.write(json.dumps(t,ensure_ascii=False)+'\n')
    with (output/'causal-attribution.jsonl').open('w') as f:
        for t in calculations:f.write(json.dumps(t,ensure_ascii=False)+'\n')
    csv_rows(output/'conditions.csv',conditions)
    csv_rows(output/'reactions.csv',[r for v in result['variants'].values() for r in v['qa02']['observations']])
    csv_rows(output/'cycles.csv',[r for v in result['variants'].values() for r in v['qa03']['observations']])
    csv_rows(output/'changes.csv',[{k:r[k] for k in ('job_id','variant','task_id','changed_unit_set','count','baseline_source_hash','patch_ref','result_ref')} for r in changes])
    csv_rows(output/'model-calls.csv',[{**{k:c.get(k) for k in ('provider','variant','repeat','case_id','request_id','call_id','role','purpose','attempt','status','discard_reason','usage','cost_usd','enqueue_ns','send_ns','terminal_ns','context','output')},**call_links.get(c['call_id'],{})} for c in calls])
    csv_rows(output/'repeat-results.csv',[{'variant':v,**r} for v,item in result['variants'].items() for r in item['per_repeat']])
    completion={'status':'COMPLETE','expected_trials':210,'observed_trials':len(trials),'conditions':len(conditions),'reactions':sum(len(t['reactions']) for t in trials),'cycles':sum(t['cycle'] is not None for t in trials),'change_jobs':len(changes),'source_digest':manifest['frozen_source_digest']}
    for key,expected in {'observed_trials':210,'conditions':1260,'reactions':270,'cycles':190,'change_jobs':12}.items():
        if completion[key]!=expected:raise ValueError(f"Final {key} population mismatch: {completion[key]} != {expected}")
    dump(output/'completion.json',completion)
    return result


if __name__ == '__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--campaign',type=Path,required=True)
    result=finalize(parser.parse_args().campaign)
    print(json.dumps({v:{f'qa{i:02d}':r[f'qa{i:02d}']['score'] for i in range(1,7)} for v,r in result['variants'].items()}))
