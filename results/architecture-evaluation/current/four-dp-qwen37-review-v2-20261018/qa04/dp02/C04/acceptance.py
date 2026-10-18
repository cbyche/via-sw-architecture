from pathlib import Path
import sys,json
REPO=Path(__file__).resolve().parents[8]
# Locate repository without depending on directory-depth assumptions.
REPO=next(p for p in Path(__file__).resolve().parents if (p/'benchmark/architecture/four_dp_poc_v2/run.py').exists())
sys.path.insert(0,str(REPO/'benchmark/architecture/four_dp_poc_v2'))
import run
ROOT=Path(__file__).resolve().parent;run.CRATE=ROOT/'candidate';episode=json.loads((run.BASE/'fixtures/dp02.json').read_text())[0]
episode={**episode,'incremental_updates':[{'goal_id':'R1','section_id':'cost','section_revision':2,'text':'Revised cost section: Plan A $119; Plan B $74.'},{'goal_id':'R1','section_id':'cost','section_revision':1,'text':'stale must not publish'}]}
checks=[]
for variant in ['A','B']:
 r=run.execute(run.OfflineClient(ROOT/'acceptance-evidence'),episode,variant,1,'QA04_OFFLINE')
 result=next(e['data']for e in r['events']if e.get('kind')=='EPISODE_RESULT')
 patches=[e['data']for e in r['events']if e.get('kind')=='CONTENT_SECTION_PATCH'];assert len(patches)==1
 actual=patches[0];assert actual['goal_id']=='R1' and actual['goal_revision']==1
 delivery=actual['full_current_delivery'];sections=delivery['result']['sections'];assert next(s['text']for s in sections if s['id']=='recovery-capacity')=='Plan A 10 seconds/40 requests; Plan B 30 seconds/24 requests.'
 assert next(s['text']for s in sections if s['id']=='cost')=='Revised cost section: Plan A $119; Plan B $74.'
 assert any(v['goal_id']=='R1'and v['section_id']=='cost'and v['section_revision']==2 for v in actual['pending_voice'])
 assert any(v['goal_id']=='R1'and v['section_id']=='recovery-capacity'and v['section_revision']==1 for v in actual['pending_voice'])
 assert not any(v['goal_id']=='R1'and v['section_id']=='cost'and v['section_revision']==1 for v in actual['pending_voice'])
 assert any(e.get('kind')=='SECTION_PATCH_REJECTED'for e in r['events'])
 assert any(e.get('kind')=='GOAL_COMPLETE' and e['data'].get('goal_id')=='U2' for e in r['events'])
 checks.append({'variant':variant,'checks':['actual_patch_callback','goal_preserved','other_section_preserved','pending_voice_section_revision_updated','other_voice_section_preserved','stale_patch_rejected','unrelated_U2_preserved'],'pass':True,'run_id':r['run_id']})
(ROOT/'acceptance-summary.json').write_text(json.dumps({'task':'C04','checks':checks,'calls':'deterministic doubles only','not_latency_evidence':True},indent=2));print(json.dumps(checks))
