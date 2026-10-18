"""Offline contract/repair checks; uses the real Rust code and mock external vision.
No provider calls. Results explicitly NOT model performance or experimental BASIC rows.
"""
import json
from pathlib import Path
import offline_dp04
import run

ROOT=run.DEFAULT/'offline-dp04-contract-checks'
ROOT.mkdir(parents=True,exist_ok=True)
fixtures=run.read(run.BASE/'fixtures/dp04.json')
oracles={x['id']:x for x in run.read(run.BASE/'oracles/dp04.json')}
checks=[]
original=offline_dp04.respond
for bad in [False,True]:
    def responder(payload,meta):return original(payload,{**meta,'dp04_probe_bad_initial':bad})
    offline_dp04.respond=responder
    for f in fixtures:
        for variant in ['A','B']:
            r=run.execute(run.OfflineClient(ROOT),f,variant,1,'OFFLINE_BAD_INITIAL' if bad else 'OFFLINE_ACCEPTANCE')
            result=next(e['data'] for e in r['events'] if e['kind']=='EPISODE_RESULT')
            expected=oracles[f['id']]['adoptions']
            actual=result['adoptions']
            exact=all(next((a['targets'] for a in actual if a['expression_id']==e['expression_id'] and a['revision']==e['revision']),None)==e['targets'] for e in expected)
            starts=[e for e in r['events'] if e['kind']=='GOAL_START']
            captures=[e for e in r['events'] if e['kind']=='INPUT_CAPTURED']
            recovery=any(e['kind'] in ['BOUNDED_SOURCE_REREAD','PRODUCER_EXPANSION_REQUEST'] for e in r['events'])
            complete=sum(e['kind']=='GOAL_COMPLETE' for e in r['events'])
            checks.append({'episode':f['id'],'variant':variant,'bad_initial_geometry':bad,'exact_structural_response':exact,'actual_owner_recovery':recovery,'one_initial_goal':len(starts)==1,'start_matches_initial_capture':starts[0]['data']['input_elapsed_ns']==captures[0]['data']['input_elapsed_ns'],'completion_callbacks':complete,'expected_revision_callbacks':len(f['revisions']),'api_calls':0,'run_id':r['run_id']})
# Late semantic adoption must retain the first capture after initial recovery exhausts.
f=next(f for f in fixtures if f['late_result_replay'])
for variant in ['A','B']:
    def responder(payload,meta):return original(payload,{**meta,'dp04_probe_fail_revision_one':True})
    offline_dp04.respond=responder
    r=run.execute(run.OfflineClient(ROOT),f,variant,1,'OFFLINE_FIRST_REVISION_FAILED')
    starts=[e for e in r['events'] if e['kind']=='GOAL_START'];captures=[e for e in r['events'] if e['kind']=='INPUT_CAPTURED']
    fail_count=sum(e['kind']=='GOAL_FAILED' for e in r['events']);complete=sum(e['kind']=='GOAL_COMPLETE' for e in r['events'])
    result=next(e['data'] for e in r['events'] if e['kind']=='EPISODE_RESULT')
    r1failed=all(not a['targets'] for a in result['adoptions'] if a['revision']==1)
    r2ok=all(next((a['targets'] for a in result['adoptions'] if a['expression_id']==e['expression_id'] and a['revision']==2),None)==e['targets'] for e in oracles[f['id']]['adoptions'] if e['revision']==2)
    checks.append({'episode':f['id'],'variant':variant,'failed_first_revision_probe':True,'exact_structural_response':r1failed and r2ok and fail_count==1,'actual_owner_recovery':True,'one_initial_goal':len(starts)==1,'start_matches_initial_capture':len(starts)==1 and starts[0]['data']['input_elapsed_ns']==captures[0]['data']['input_elapsed_ns'],'completion_callbacks':complete,'expected_revision_callbacks':1,'api_calls':0,'run_id':r['run_id']})
offline_dp04.respond=original
passed=all(x['exact_structural_response'] and x['one_initial_goal'] and x['start_matches_initial_capture'] and x['completion_callbacks']==x['expected_revision_callbacks'] and (not x.get('bad_initial_geometry',False) or x['actual_owner_recovery']) for x in checks)
output={'status':'PASS' if passed else 'FAIL','checks':checks,'provider_calls':0,'claim_scope':'Actual Rust guards/repair/callback contract only. Mock vision uses authored SVG source; does not measure model capability.'}
(ROOT/'checks.json').write_text(json.dumps(output,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'status':output['status'],'checks':len(checks),'path':str(ROOT/'checks.json'),'provider_calls':0}))
if not passed:raise SystemExit(1)
