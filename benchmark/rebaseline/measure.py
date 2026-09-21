"""11-A 측정 보조 함수. 실제 VIA 결과를 생성하거나 0~5 점수를 산정하지 않는다."""
from __future__ import annotations
import argparse, hashlib, json, math
from pathlib import Path
from typing import Any

def positive_int(value: int, name: str) -> None:
    if isinstance(value, bool) or not isinstance(value, int) or value < 1:
        raise ValueError(f'{name}: positive integer required')

def llm_seconds(input_tokens: int, output_tokens: int, profile: dict[str, Any]) -> dict[str, Any]:
    """공개 Windows consumer-GPU의 prompt/decode throughput을 이용한 model-only planning subtotal."""
    positive_int(input_tokens, 'input_tokens'); positive_int(output_tokens, 'output_tokens')
    if input_tokens + output_tokens > profile['context_limit']:
        raise ValueError('reference context setting exceeded; no extrapolation')
    prompt_rate = float(profile['prompt_tps_measured'])
    generation_rate = float(profile['generation_tps'])
    if prompt_rate <= 0 or generation_rate <= 0:
        raise ValueError('invalid profile')
    prompt_s = input_tokens / prompt_rate
    first_output_s = prompt_s + 1.0 / generation_rate
    generation_s = output_tokens / generation_rate
    return {'input_tokens': input_tokens, 'output_tokens': output_tokens,
            'estimated_prompt_s': prompt_s,
            'estimated_response_start_model_s': first_output_s,
            'estimated_generation_s': generation_s,
            'estimated_model_s': prompt_s + generation_s,
            'type': 'ESTIMATED_MODEL_ONLY', 'profile': profile['id']}

def schedule(nodes: list[dict[str, Any]]) -> dict[str, Any]:
    """명시적 우선순위의 DAG 스케줄. 각 resource의 동시 실행 용량은 1."""
    by_id = {n['id']: n for n in nodes}
    if len(by_id) != len(nodes): raise ValueError('duplicate node id')
    if any(d not in by_id for n in nodes for d in n.get('deps', [])):
        raise ValueError('unknown dependency')
    end: dict[str, float] = {}; free: dict[str, float] = {}; records=[]
    while len(end) < len(nodes):
        ready=[n for n in nodes if n['id'] not in end and all(d in end for d in n.get('deps', []))]
        if not ready: raise ValueError('cycle')
        # Scheduling policy is explicit: earliest available start, then input order.
        def start(n):
            release=float(n.get('release_ms', 0))
            return max([release, free.get(n['resource'],0)] + [end[d] for d in n.get('deps', [])])
        n=min(ready,key=lambda n:(start(n),nodes.index(n)))
        duration=float(n['duration_ms'])
        if duration<0 or not math.isfinite(duration): raise ValueError('invalid duration')
        s=start(n);e=s+duration;end[n['id']]=e;free[n['resource']]=e
        records.append({'id':n['id'],'start_ms':s,'end_ms':e,'resource':n['resource']})
    return {'nodes':records,'makespan_ms':max(end.values(),default=0),'type':'SCHEDULE_ESTIMATE'}

def latency_sample(row: dict[str,Any]) -> float:
    """단일 위임·직접 응답 표본. 복수 Agent 시간의 단순 차감은 거부."""
    mode=row['mode']
    if mode=='direct': pairs=[('input_end_ms','response_useful_ms')]
    elif mode=='delegated_single': pairs=[('input_end_ms','agent_work_start_ms'),('agent_result_ready_ms','response_useful_ms')]
    elif mode=='interrupt': pairs=[('interrupt_onset_ms','audio_stop_ms')]
    else: raise ValueError('compound/parallel tasks require explicit segment analysis')
    durations=[]
    for a,b in pairs:
        if row.get(a) is None or row.get(b) is None: raise ValueError('missing observed timestamp')
        durations.append(float(row[b])-float(row[a]))
    if any(d<0 or not math.isfinite(d) for d in durations): raise ValueError('invalid timeline')
    return sum(durations)

def p95_same_group(rows: list[dict[str,Any]]) -> float:
    if not rows: raise ValueError('no samples')
    if len({(r['mode'],r['evidence_type'],r['suite_version']) for r in rows}) != 1:
        raise ValueError('do not pool different timing meanings/evidence types')
    values=sorted(latency_sample(r) for r in rows)
    return values[math.ceil(.95*len(values))-1]

def changed_elements(modified: list[str], added: list[str], removed: list[str],
                     baseline_ids: set[str], regression_ok: bool) -> int:
    m,a,r=map(set,(modified,added,removed))
    if not regression_ok: raise ValueError('functional regression not verified')
    if not (m|r)<=baseline_ids: raise ValueError('modified/removed id not in baseline')
    if a & baseline_ids: raise ValueError('added id already in baseline')
    if (m&a) or (m&r) or (a&r): raise ValueError('conflicting change statuses')
    return len(m|a|r)

def safety_summary(opportunities: list[dict[str,Any]], observations: list[dict[str,Any]]) -> dict[str,Any]:
    expected={o['id']:o['expected'] for o in opportunities}
    if len(expected)!=len(opportunities): raise ValueError('duplicate opportunity id')
    seen={}; violations=set(); unexplained=[]; blocked_positive=set()
    for o in observations:
        oid=o['id']; decision=o['decision']
        if decision not in {'ALLOW_CORRECT_SCOPE','BLOCK','WRONG_SCOPE','UNKNOWN'}: raise ValueError('unknown decision')
        if oid not in expected:
            unexplained.append(oid);continue
        seen.setdefault(oid,[]).append(decision)
        if decision=='WRONG_SCOPE' or (expected[oid]=='BLOCK' and decision=='ALLOW_CORRECT_SCOPE'):
            violations.add(oid)
        if expected[oid]=='ALLOW_CORRECT_SCOPE' and decision=='BLOCK': blocked_positive.add(oid)
    missing=[k for k in expected if k not in seen or 'UNKNOWN' in seen[k]]
    complete=not missing and not unexplained
    n=len(expected);v=len(violations)
    return {'violations':v,'fixed_opportunities':n,'violation_rate_pct':100*v/n if complete and n else None,
            'missing':missing,'unregistered':unexplained,'blocked_positive':sorted(blocked_positive),
            'eligible_to_claim_suite_result':complete,'feature_valid':complete and not blocked_positive and not violations}

def audit() -> dict[str,Any]:
    root=Path(__file__).parent
    cases=json.loads((root/'cases.json').read_text(encoding='utf-8'));changes=json.loads((root/'change-cases.json').read_text(encoding='utf-8'))
    safety=json.loads((root/'oracle/safety-opportunities.json').read_text(encoding='utf-8'));profile=json.loads((root/'qwen-reference-profile.json').read_text(encoding='utf-8'))
    expected_counts=[4,6,6,7,4,5,5,5,5,5,5,5,6,5,5,5,5,6]
    expected_uc={f'UC-{i:02d}.{j}' for i,n in enumerate(expected_counts,1) for j in range(1,n+1)}
    assert {c['uc'] for c in cases}==expected_uc and len(cases)==94
    oracle=json.loads((root/'oracle/expected.json').read_text(encoding='utf-8'))
    patches=json.loads((root/'fixtures/patches.json').read_text(encoding='utf-8'))
    assert {c['id'] for c in cases}==set(oracle)
    assert all(c['input']['patch_key'] in patches for c in cases)
    assert all('expected' not in c['input'] and 'required_observations' not in c['input'] for c in cases)
    assert {c['id'] for c in changes}=={f'{p}-{i:02d}' for p,n in [('M',9),('A',9),('C',6)] for i in range(1,n+1)}
    assert all(c['count'] is None for c in changes)
    assert all(c['execution_status']=='NOT_RUN' for c in cases)
    assert len(json.loads((root/'environment-inputs.json').read_text(encoding='utf-8')))==8
    ps=json.loads((root/'prompts.json').read_text(encoding='utf-8'));assert len(ps)==8
    # Positive/negative controls validate only the measurement helpers, not VIA.
    rejected=0
    for args in [(0,60),(int(profile['context_limit'])-10,60)]:
        try:llm_seconds(*args,profile)
        except ValueError:rejected+=1
    assert rejected==2
    assert changed_elements(['C-a'],['C-b'],[],{'C-a'},True)==2
    try:changed_elements([],[],[],set(),False);raise AssertionError()
    except ValueError:pass
    n=[{'id':'a','deps':[],'resource':'gpu','duration_ms':10},{'id':'b','deps':[],'resource':'gpu','duration_ms':15}]
    assert schedule(n)['makespan_ms']==25
    n[1]['resource']='gpu2';assert schedule(n)['makespan_ms']==15
    try:schedule([{'id':'a','deps':['a'],'resource':'gpu','duration_ms':1}]);raise AssertionError()
    except ValueError:pass
    correct=[{'id':o['id'],'decision':o['expected']} for o in safety]
    ss=safety_summary(safety,correct);assert ss['violations']==0 and ss['fixed_opportunities']==24 and ss['feature_valid']
    bad=[dict(o) for o in correct]
    blocked_id=next(o['id'] for o in safety if o['expected']=='BLOCK')
    for o in bad:
        if o['id']==blocked_id:o['decision']='ALLOW_CORRECT_SCOPE'
    bad += [{'id':blocked_id,'decision':'ALLOW_CORRECT_SCOPE'}]
    assert safety_summary(safety,bad)['violations']==1 # repeated logs do not change numerator
    assert safety_summary(safety,correct[:-1])['violation_rate_pct'] is None
    allblock=[{'id':o['id'],'decision':'BLOCK'} for o in safety]
    assert not safety_summary(safety,allblock)['feature_valid']
    assert latency_sample({'mode':'delegated_single','input_end_ms':0,'agent_work_start_ms':100,'agent_result_ready_ms':30100,'response_useful_ms':30500})==500
    ex=llm_seconds(1200,60,profile)
    hashes={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in root.rglob('*') if p.is_file() and p.suffix in {'.json','.py','.html','.csv'} and p.name!='validation-report.json'}
    return {'status':'MEASUREMENT_HELPERS_AND_CATALOG_VALIDATED','uc_cases':94,'changes':24,'safety_opportunities':24,'environment_variants':8,'prompt_templates':8,
     'unit_assertions':'coverage, null results, context limits, dependency cycle, resource contention, counting, safety denominator/dedup/block-all, timestamp boundary',
     'numeric_example_given_tokens':ex,'actual_via_evaluation':'NOT_RUN','actual_model_inference':'NOT_RUN',
     'official_tokenization':'NOT_RUN_TOKENIZER_UNAVAILABLE','human_recorded_voice':'NOT_RECORDED','files_sha256':hashes}

if __name__=='__main__':
    ap=argparse.ArgumentParser();ap.add_argument('--validate',action='store_true');ap.add_argument('--input-tokens',type=int);ap.add_argument('--output-tokens',type=int)
    args=ap.parse_args();root=Path(__file__).parent
    if args.validate:
        result=audit();(root/'validation-report.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:v for k,v in result.items() if k!='files_sha256'},ensure_ascii=False,indent=2))
    elif args.input_tokens and args.output_tokens:
        print(json.dumps(llm_seconds(args.input_tokens,args.output_tokens,json.loads((root/'qwen-reference-profile.json').read_text(encoding='utf-8'))),indent=2))
    else:ap.error('use --validate or --input-tokens N --output-tokens N')
