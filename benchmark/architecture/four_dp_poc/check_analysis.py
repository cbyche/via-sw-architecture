"""Focused regression checks for the two reviewer-found scoring bugs, using retained traces."""
import argparse,copy,json
from pathlib import Path
from analyze import keyed,row,subtract_occupied,BASE
from bridge import dump
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();records=[json.loads(x) for x in (a.results/'runs.jsonl').read_text().splitlines() if x];checks=[]
def check(name,predicate):
    if not predicate:raise AssertionError(name)
    checks.append({'check':name,'status':'PASS'})
check('Overlapping VIA queued/active work is not removed as user wait',subtract_occupied([(10,40)],[(0,50)])==[])
check('Only blocking user-wait fragments remain excluded',subtract_occupied([(10,40)],[(20,30)])==[(10,20),(30,40)])
r=next(r for r in records if r['dp']=='01' and any(e.get('kind')=='EPISODE_RESULT' for e in r['events']));oracle=keyed(BASE/'oracles/dp01.json')[r['episode']];normal=row(r,oracle);truncated=copy.deepcopy(r);truncated['events']=[e for e in truncated['events'] if e.get('kind')!='EPISODE_RESULT'];partial=row(truncated,oracle)
check('Later failure cannot erase previous exact-set adoption',normal['qa02_correct']==partial['qa02_correct'] and normal['qa02_total']==partial['qa02_total'])
onlyfirst=copy.deepcopy(r);first_stimulus=next(i for i,e in enumerate(onlyfirst['events']) if e.get('kind')=='STIMULUS');onlyfirst['events']=onlyfirst['events'][:first_stimulus+1];missing=row(onlyfirst,oracle)
check('Early failure retains full expected referent denominator',missing['qa02_total']==len(oracle['adoptions']) and missing['qa02_correct']==0)
check('Early failure counts expected future missing displays',missing['qa01_noresponse']==len(oracle['adoptions']))
r5=next(r for r in records if r['dp']=='05' and any(e.get('kind')=='HANDOFF_ADMISSION' for e in r['events']));o5=keyed(BASE/'oracles/dp05.json')[r5['episode']];normal5=row(r5,o5);partial5=copy.deepcopy(r5);partial5['events']=[e for e in partial5['events'] if e.get('kind')!='EPISODE_RESULT'];scored5=row(partial5,o5)
check('Later receiver failure cannot erase admitted handoff contract',normal5['nh_correct']==scored5['nh_correct'] and normal5['nh_total']==scored5['nh_total'])
null_targets=copy.deepcopy(r)
next(e for e in null_targets['events'] if e.get('kind')=='SEMANTIC_ADOPTION')['data']['targets']=None
bad=row(null_targets,oracle)
check('Null model targets score wrong without crashing',bad['qa02_details'][0]['correct'] is False and bad['qa02_details'][0]['failure_reason']=='MALFORMED_TARGETS')
version_case=next(r for r in records if r['dp']=='01' and r['episode']=='dp01-e3' and r['phase']=='BASIC')
version_oracle=keyed(BASE/'oracles/dp01.json')['dp01-e3'];null_versions=copy.deepcopy(version_case)
next(e for e in null_versions['events'] if e.get('kind')=='SEMANTIC_ADOPTION')['data']['source_revisions']=None
bad_versions=row(null_versions,version_oracle)
check('Null required source versions score wrong without crashing',bad_versions['qa02_details'][0]['correct'] is False and bad_versions['qa02_details'][0]['failure_reason']=='MALFORMED_SOURCE_REVISIONS')
dump(a.results/'analysis-checks.json',{'checks':checks,'count':len(checks),'model_calls':0,'scope':'Derived-score regression checks; original raw traces unchanged'})
print(json.dumps({'checks_passed':len(checks),'model_calls':0}))
