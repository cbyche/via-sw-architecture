from pathlib import Path
import sys,json
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2/run.py').exists());sys.path.insert(0,str(REPO/'benchmark/architecture/four_dp_poc_v2'))
import run,offline_dp02
ROOT=Path(__file__).resolve().parent;run.CRATE=ROOT/'candidate';episode=json.loads((run.BASE/'fixtures/dp02.json').read_text())[0];text=episode['sources'][0]['text'];a=text.index('cost 120 dollars per month');b=a+len('cost 120 dollars per month')
episode={**episode,'source_range_revocations':[{'source_id':'D1','revision':1,'start_byte':a,'end_byte':b,'trigger':'FIRST_DRAFT_READY','authority':'user_access_event'}]}
original=offline_dp02.respond
# Contract double responds only from currently supplied source; it is never model evidence.
def respond(payload,meta):
 x=payload.get('input',{});sources=x.get('sources',[])
 if meta['stage'] in ('dp02_complete_response','dp02_repair_response') and any('[WITHDRAWN RANGE]' in s.get('text','')for s in sources):
  return {'answer':'The Plan A price range was withdrawn. Plan A recovery checkpoint every 10 seconds and Plan B every 30 seconds remain available.','sections':[{'id':'recovery','text':'Plan A checkpoint 10 seconds; Plan B 30 seconds.'}],'applied_revision':x['current_revision'],'source_claims':[{'source_id':'D1','revision':1,'quote':'recovery checkpoint every 10 seconds'}]}
 return original(payload,meta)
offline_dp02.respond=respond
checks=[]
for variant in ['A','B']:
 r=run.execute(run.OfflineClient(ROOT/'acceptance-evidence'),episode,variant,1,'QA04_OFFLINE')
 kinds=[e.get('kind')for e in r['events']];assert kinds.count('SOURCE_RANGE_REVOKED')==1 and kinds.count('REVOKED_DRAFT_REJECTED')==1
 discarded=next(e for e in r['events']if e.get('kind')=='REVOKED_DRAFT_REJECTED');assert '120' in discarded['data']['draft']['answer']
 terminal=next(e['data']for e in r['events']if e.get('kind')=='GOAL_COMPLETE'and e['data']['goal_id']=='R1');assert '120'not in terminal['text'] and '10 seconds'in terminal['text']
 final_calls=[c for c in r['calls']if c['stage']=='dp02_complete_response'];assert len(final_calls)>=2
 # Bounded rejudgment must follow invalidation, with a real second response operation.
 rejected_at=discarded['elapsed_ns'];assert any(e.get('kind')=='OPERATION_START' and e['elapsed_ns']>rejected_at for e in r['events'])
 assert not any(e.get('kind')=='GOAL_FAILED'for e in r['events'])
 checks.append({'variant':variant,'pass':True,'run_id':r['run_id'],'checks':['versioned_partial_range_event','pending_draft_rejected','actual_second_response_operation','withdrawn_price_not_delivered','allowed_recovery_range_preserved','original_goal_preserved']})
(ROOT/'acceptance-summary.json').write_text(json.dumps({'task':'C06','checks':checks,'provider_calls':0,'not_latency_evidence':True},indent=2));print(json.dumps(checks))
