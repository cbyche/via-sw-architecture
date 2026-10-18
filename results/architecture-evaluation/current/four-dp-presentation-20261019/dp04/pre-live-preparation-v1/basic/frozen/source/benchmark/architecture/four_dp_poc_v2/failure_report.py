"""Actual first referent-set failures and surrounding evidence; no inferred causal verdict."""
import argparse,json
from pathlib import Path
from analyze import BASE,keyed,row
from bridge import dump
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();runs=[json.loads(x) for x in (a.results/'runs.jsonl').read_text().splitlines() if x];failures=[]
for r in runs:
    if r['dp'] not in ('01','04'):continue
    oracle=keyed(BASE/'oracles'/f'dp{r["dp"]}.json')[r['episode']];scored=row(r,oracle)
    for judgment in scored['qa02_details']:
        if judgment['correct']:continue
        expected=judgment['expected'];actual=judgment['actual_first_adoption'];eid=expected['expression_id'];rev=expected['revision'];current_input=None;branch=None;query=None;publication=None;reads=[];matched=False
        for event in r['events']:
            kind=event.get('kind');d=event.get('data',{})
            if kind=='RELATION_PUBLICATION':publication=d
            elif kind=='STIMULUS':current_input=d;branch=None;query=None;reads=[]
            elif kind=='EVIDENCE_BRANCH':branch=d
            elif kind=='EVIDENCE_QUERY':query=d
            elif kind in ('EVIDENCE_READ','ROLLING_EVIDENCE_REUSE'):reads.append(d)
            elif kind in ('ADOPTION','SEMANTIC_ADOPTION') and d.get('expression_id')==eid and d.get('revision')==rev:matched=True;break
        want=set(expected.get('targets',[]));got=set(x for x in actual.get('targets',[]) if isinstance(x,str)) if actual and isinstance(actual.get('targets'),list) else set();ids=want|got
        records=[x for x in (publication or {}).get('records',[]) if any(s.get('source_id') in ids for s in x.get('source_spans',[]))]
        failures.append({'run_id':r['run_id'],'phase':r['phase'],'dp':r['dp'],'variant':r['variant'],'episode':r['episode'],'rep':r['repetition'],'expression_id':eid,'revision':rev,'expected':expected,'actual_first_adoption':actual,'missing_targets':sorted(want-got),'extra_targets':sorted(got-want),'classification':judgment.get('failure_reason') or ('MISSING_ADOPTION' if actual is None else 'TARGET_SET_MISMATCH' if want!=got else 'SOURCE_VERSION_MISMATCH'),'actual_stimulus':current_input if matched else None,'evidence_branch':branch if matched else None,'evidence_query':query if matched else None,'actual_original_reads':reads if matched else [],'published_source_vector':(publication or {}).get('source_vector'),'relevant_published_records':records,'raw_trace':str(Path(r['input']).parent/'trace.json'),'inference_limit':'Trace facts only; this does not establish intrinsic architectural cause or producer omission.'})
dump(a.results/'referent-failures.json',failures)
lines=['# First-adoption referent failures','','These are actual adopted sets, not repaired final answers. Source versions and available/read paths are shown as observed. No model reruns or oracle-driven tuning. Explanations below classify set/version differences, not architectural causes.','']
for f in failures:
    ex=f['expected'];act=f['actual_first_adoption'] or {};branch=(f['evidence_branch'] or {}).get('branch','not observed / screen path')
    lines += [f"## {f['phase']} {f['episode']} {f['variant']} r{f['rep']} — {f['expression_id']} rev{f['revision']}", '', f"- Expected: `{json.dumps(ex.get('targets',[]),ensure_ascii=False)}`; first adopted: `{json.dumps(act.get('targets',[]),ensure_ascii=False)}`.",f"- Adopted source versions: `{json.dumps(act.get('source_revisions',[]),ensure_ascii=False)}`; classification: {f['classification']}.",f"- Actual path: {branch}; model requested mode: {(f['evidence_query'] or {}).get('mode','N/A')}; original reads: `{json.dumps([{k:v for k,v in x.items() if k!='text'} for x in f['actual_original_reads']],ensure_ascii=False)}`.",f"- Actual expression/utterance: `{json.dumps(f['actual_stimulus'],ensure_ascii=False)}`.",f"- Model conditions/evidence: `{json.dumps(act.get('conditions',act.get('evidence')),ensure_ascii=False)}`.",f"- Relevant published records: {len(f['relevant_published_records'])}; full exact records in referent-failures.json.",f"- Raw trace: `{f['raw_trace']}`.", '']
(a.results/'referent-failures.md').write_text('\n'.join(lines)+'\n');print(f'{len(failures)} exact-set/version failures exported')
