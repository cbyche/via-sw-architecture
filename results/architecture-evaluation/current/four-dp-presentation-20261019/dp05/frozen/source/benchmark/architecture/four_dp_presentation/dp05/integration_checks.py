"""Offline protocol only: no model quality, no provider call. Run in exclusive resource window."""
import copy,json,sys,shutil
from pathlib import Path
HERE=Path(__file__).resolve().parent;sys.path.insert(0,str(HERE.parent/'dp01'))
from experiment_common import *
class Probe(harness.OfflineClient):
 def __init__(self,root):super().__init__(root);self.payloads=[]
 def call(self,payload,meta):
  self.payloads.append({'stage':meta['stage'],'payload':payload})
  if meta['stage']=='dp01_adopt_only':
   changed=dict(meta);changed['stage']='dp01_select_adopt_or_complete';value,record=super().call(payload,changed);return {'adoption':value['adoption']},record
  return super().call(payload,meta)
def run():
 records=[]
 for name,dp,eid,tactic,rep in [('C1-unused','dp05','dp05-e1','C1',1),('C1-current','dp05','dp05-e3','C1',1),('C1-revision','dp05','dp05-e3','C1',1),('C1-revoke','dp05','dp05-e3','C1',1),('C1-range','dp05','dp05-e3','C1',1),('C1-repair','dp05','dp05-e3','C1',2),('C2-repeat','dp05','dp05-e3','C2',1),('C2-correction','dp05','dp05-e4','C2',1),('adoption-policy','dp01','dp01-e3',None,1)]:
  case=copy.deepcopy(next(e for e in read(OLD/'fixtures'/f'{dp}.json')if e['id']==eid))
  if tactic=='C1':
   for e in case['events']:
    if e['kind']=='adopt':e['state']['condition_source_refs']=[{'source_id':'delivered-guidance' if eid=='dp05-e3' else 'public-comparison','revision':1,'start_line':1,'end_line':2 if eid=='dp05-e3' else 1}]
  if name in ['C1-revision','C1-revoke','C1-range']:
   source=copy.deepcopy(next(e['source']for e in case['events']if e['kind']=='source'and e['source']['id']=='delivered-guidance'))
   if name=='C1-revision':source['revision']=2
   if name=='C1-range':source['allowed_lines']=[2,3]
   replacement={'kind':'revoke','source_id':'delivered-guidance'}if name=='C1-revoke'else{'kind':'source','source':source}
   index=next(i for i,e in enumerate(case['events'])if e['kind']=='delegate');case['events'].insert(index,replacement)
  if dp=='dp01':case['first_response_policy']='ADOPTION_ONLY'
  root=ROOT/dp/'offline'/name;root.mkdir(parents=True,exist_ok=False);shutil.copy2(ROOT/dp/'freeze.json',root/'freeze.json');c=Probe(root);r=harness.execute(c,case,'C' if dp=='dp05' else 'A',rep,'OFFLINE_PROTOCOL',tactic)
  events=r['events'];complete=[e for e in events if e.get('kind')=='GOAL_COMPLETE'];failed=[e for e in events if e.get('kind') in ['GOAL_FAILED','EPISODE_ERROR']]
  checks={'returncode_zero':r['returncode']==0,'no_terminal_errors':not failed,'expected_terminal_count':len(complete)==(1 if eid=='dp05-e1' else 3 if eid=='dp05-e3' else 2)}
  if name=='C1-unused':checks['actual_unused_prefetch']=any(e.get('kind')=='C1_PREFETCH_COMMIT'and e['data']['passages'] for e in events)and not any(e.get('kind')=='HANDOFF_ADMISSION'for e in events)
  if name=='C1-current':checks['raw_current_text_forwarded']=all(p['payload']['input']['capsule']['passages']for p in c.payloads if p['stage']=='dp05_receiver_select_or_complete')
  if name in ['C1-revision','C1-revoke','C1-range']:
   contracts=[p['payload']['input']for p in c.payloads if p['stage']=='dp05_receiver_select_or_complete'];checks['invalid_prefetch_not_forwarded']=all(not p['capsule']['passages']and not p['capsule']['references']and not p['mandatory']['condition_source_refs']for p in contracts)
  if name=='C2-repeat':checks['actual_hit']=sum(e.get('kind')=='C2_EXACT_READ_HIT'for e in events)>=4
  if name=='C2-correction':
   seen2=False;bad=[]
   for e in events:
    if e.get('kind')=='PUBLIC_ACTIVITY'and e['data'].get('source',{}).get('revision')==2:seen2=True
    if seen2 and e.get('kind')=='HANDOFF_SOURCE_READ'and e['data']['source_id']=='large-public-report'and e['data']['revision']!=2:bad.append(e)
   checks['no_stale_after_revision']=not bad
  if name=='adoption-policy':checks['extra_adoption_only_calls']=sum(c['stage']=='dp01_adopt_only'for c in r['calls'])==2
  record={'name':name,'checks':checks,'pass':all(checks.values()),'run_id':r['run_id'],'API_calls':0,'new_LLM_accuracy_claimed':False};records.append(record);dump(root/'payloads.json',c.payloads)
 dump(ROOT/'dp05/offline-summary.json',{'cases':records,'passed':all(r['pass']for r in records),'API_calls':0});print(json.dumps(records,ensure_ascii=False))
if __name__=='__main__':run()
