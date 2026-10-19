"""Independent QA04 scratch functional acceptance. No provider or latency evidence."""
import collections,datetime,difflib,hashlib,json,subprocess,sys,time
from pathlib import Path
from control import HERE,RESULT,CRATE,REPO
sys.path.insert(0,str(HERE))
import run
ROOT=RESULT/'qa04'
BASE=CRATE

def write(p,value):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n')
def result(record):return next(e['data'] for e in reversed(record['events']) if e['kind']=='EPISODE_RESULT')
def data(record,kind):return [e['data'] for e in record['events'] if e['kind']==kind]
def execute(dp,feature,fixture,rep=1):
 scratch=ROOT/feature

 folder=ROOT/feature
 subprocess.run(['cargo','build','--offline','--manifest-path',str(scratch/'Cargo.toml')],check=True,capture_output=True)
 records=[]
 for variant in ['A','B','C']:
  run.CRATE=scratch/variant/'candidate';subprocess.run(['cargo','build','--offline','--manifest-path',str(run.CRATE/'Cargo.toml')],check=True,capture_output=True)
  class MetadataProbeClient(run.OfflineClient):
   def call(self,payload,meta):
    response,receipt=super().call(payload,meta)
    receipt['qa04_actual_payload']=payload
    d=payload.get('input',{})
    catalog=d.get('catalog',d.get('source_catalog',d.get('manifest',[])))
    if feature=='C06' and any(x.get('withdrawn_ranges') for x in catalog):assert 'WITHDRAWN PRIVATE FACT' not in json.dumps(payload),payload
    if feature=='C06' and meta['stage'] in ('dp05_select_handoff_details','dp05_repair_handoff_selection'):
     response['references']=[{'source_id':x['id'],'revision':x['revision'],'start_line':a,'end_line':b} for x in d['manifest'] for a,b in x.get('allowed_segments',[x['allowed_lines']])]
    if feature=='C06' and meta['stage'] in ('dp05_receiver_select_or_complete','dp05_receiver_recover_read_selection'):
     contract=d.get('contract',d)
     if response.get('mode')=='READ' or meta['stage']=='dp05_receiver_recover_read_selection':response['reads']=[{'source_id':x['id'],'start_line':a,'end_line':b} for x in contract['manifest'] if x['id'] in contract['mandatory']['required_source_ids'] for a,b in x.get('allowed_segments',[x['allowed_lines']])]
    if feature=='C06' and meta['stage'] in ('dp01_publish_relations','dp05_produce_work_record'):
     evidence=payload.get('input',{}).get('observed_evidence',payload.get('input',{}).get('public_evidence',[]))
     if not any('WITHDRAWN PRIVATE FACT' in p['text'] for p in evidence):assert 'WITHDRAWN PRIVATE FACT' not in json.dumps(payload),payload
     blocks=[]
     for passage in evidence:
      for index,line in enumerate(passage['text'].splitlines()):
       span={k:passage[k] for k in ('source_id','revision')};span.update(start_line=passage['start_line']+index,end_line=passage['start_line']+index)
       if dp=='01':blocks.append({'record_id':f"{span['source_id']}:{span['start_line']}",'subject':span['source_id'],'predicate':'observed','object':line,'explanation':line,'source_spans':[span]})
       else:blocks.append({'block_id':f"{span['source_id']}:{span['start_line']}",'explanation':line,'source_spans':[span]})
     response['records' if dp=='01' else 'blocks']=blocks
     if any('WITHDRAWN PRIVATE FACT' in p['text'] for p in evidence):
      if dp=='01':response['coverage']['gaps']=['WITHDRAWN PRIVATE FACT']
      else:response['conditions']=['WITHDRAWN PRIVATE FACT'];response['unknowns']=['WITHDRAWN PRIVATE FACT']
    return response,receipt
  client=MetadataProbeClient(folder);record=run.execute(client,fixture,variant,rep,'QA04_OFFLINE');records.append(record)
  assert not data(record,'EPISODE_ERROR'),record.get('stderr')
 with (folder/'acceptance-runs.jsonl').open('a') as out:
  for r in records:out.write(json.dumps(r,ensure_ascii=False)+'\n')
 return records

# 01 C05: query actual complete output; rejected/never-delivered responses remain unqueryable.
ep=json.loads((run.BASE/'fixtures/dp01.json').read_text())[0]
ep={**ep,'id':'qa04-dp01-C05'};ep['events']=[*ep['events'],{'kind':'query_delivered_provenance','response_goal_id':'e1-q1','goal_id':'lookup','stimulus_id':'lookup'},{'kind':'query_delivered_provenance','response_goal_id':'unpresented','goal_id':'unknown','stimulus_id':'unknown'}]
for rep in [1,3]:
 for r in execute('01','C05',ep,rep):
  queries=data(r,'DELIVERED_PROVENANCE_LOOKUP');assert len(queries)==2
  assert queries[0]['provenance']['status']==('DELIVERED' if rep==1 or r['variant']=='C' else 'NO_MATCH')
  assert queries[1]['provenance']['status']=='NO_MATCH'

# A real 3-line source supports range authority and actual background production.
for dp in ['01']:
 folder=ROOT/'C06';folder.mkdir(exist_ok=True)
 path=folder/'range-source.txt';path.write_text('left public fact\nWITHDRAWN PRIVATE FACT\nright public fact\n')
 source={'id':'range-source','revision':1,'title':'Range authority example','path':str(path),'line_count':3,'delivered_lines':[1,3],'allowed_lines':[1,3],'outline':[{'section':'left','lines':[1,1]},{'section':'private middle','lines':[2,2]},{'section':'right','lines':[3,3]}],'revoked':False}
 events=[{'kind':'source','source':source}]
 if dp=='01':events.append({'kind':'request','conversation':'range','stimulus_id':'before','goal_id':'before','expression_id':'before-ref','revision':1,'utterance':'Describe the supplied example source.','expression':'example source'})
 else:
  events.append({'kind':'adopt','state':{'task_id':'T-RANGE','request_revision':1,'condition_revision':1,'access_epoch':1,'constraints':{},'exclusions':[],'allowed_actions':['draft_only'],'already_completed_actions':[],'unknown_actions':[],'required_source_ids':['range-source']}})
  events.append({'kind':'acquire','reads':[{'source_id':'range-source','start_line':1,'end_line':3}]})
 events.append({'kind':'withdraw_range','source_id':'range-source','revision':1,'start_line':2,'end_line':2,'access_epoch':2})
 # Refresh without a new permission grant must preserve the withdrawal; producer must use segments.
 events.append({'kind':'source','source':source})
 if dp=='05':
  events.append({'kind':'acquire','reads':[{'source_id':'range-source','start_line':2,'end_line':2},{'source_id':'range-source','start_line':1,'end_line':1},{'source_id':'range-source','start_line':3,'end_line':3}]})
 if dp=='01':events.append({'kind':'request','conversation':'range','stimulus_id':'after','goal_id':'after','expression_id':'after-ref','revision':1,'utterance':'Describe only the currently allowed example evidence.','expression':'example evidence'})
 else:events.append({'kind':'delegate','goal_id':'after','stimulus_id':'after','delegation_id':'after-delegation','delegation_revision':1,'utterance':'Prepare a small draft from currently allowed public evidence. No external actions.'})
 fixture={'id':'qa04-dp'+dp+'-C06','dp':dp,'description':'Actual range authority lifecycle/serving/production acceptance','events':events}
 for r in execute(dp,'C06',fixture):
  applied=data(r,'RANGE_WITHDRAWAL_APPLIED');assert len(applied)==1
  filtered=data(r,'RANGE_DEPENDENCIES_FILTERED')[-1]
  assert 'WITHDRAWN PRIVATE FACT' not in json.dumps(filtered),filtered
  if dp=='01' and r['variant']=='C':assert len(filtered['publication']['records'])==2
  if dp=='05' and r['variant']=='B':assert len(filtered['work_record']['blocks'])==2
  if dp=='01' and r['variant']=='A':assert [p['start_line'] for p in filtered['working_view']]==[1,3]
  if dp=='05':
   receipt=data(r,'COMMON_EVIDENCE_ACQUIRED')[-1];assert len(receipt['denials'])==1
   safe=data(r,'HANDOFF_SOURCE_READ')[-2:];assert [p['start_line'] for p in safe]==[1,3]
   contract=data(r,'HANDOFF_ADMISSION')[-1];assert contract['mandatory']['access_epoch']==2
   assert len(data(r,'GOAL_COMPLETE'))==1
  pub_kind='RELATION_PUBLICATION' if dp=='01' else 'WORK_RECORD_PUBLICATION'
  publications=[e for e in r['events'] if e['kind']==pub_kind]
  if (dp=='01' and r['variant']=='C') or (dp=='05' and r['variant']=='B'):
   assert len(publications)>=2
   assert 'WITHDRAWN PRIVATE FACT' not in json.dumps(publications[-1]['data'])

