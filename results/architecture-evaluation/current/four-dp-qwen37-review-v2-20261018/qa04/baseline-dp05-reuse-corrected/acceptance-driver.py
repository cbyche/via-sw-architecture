"""Independent QA04 scratch functional acceptance. No provider or latency evidence."""
import collections,datetime,difflib,hashlib,json,subprocess,sys,time
from pathlib import Path
REPO=Path('/Users/changbae/Workspace/via-sw-architecture')
sys.path.insert(0,str(REPO/'benchmark/architecture/four_dp_poc_v2'))
import run
ROOT=Path('/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/qa04/baseline-dp05-reuse-corrected')
BASE=Path('/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/correction-preparation/dp05-common-retention/source')

def write(p,value):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n')
def result(record):return next(e['data'] for e in reversed(record['events']) if e['kind']=='EPISODE_RESULT')
def data(record,kind):return [e['data'] for e in record['events'] if e['kind']==kind]
def execute(dp,feature,fixture,rep=1):
 scratch=Path('/private/tmp/four-dp-review-qa04-reuse-dp'+dp+'-'+feature);run.CRATE=scratch
 folder=ROOT/('dp'+dp+'-'+feature)
 records=[]
 for variant in ['A','B','C']:
  run.CRATE=Path(str(scratch)+'-'+variant);subprocess.run(['cargo','build','--offline','--manifest-path',str(run.CRATE/'Cargo.toml')],check=True,capture_output=True)
  class MetadataProbeClient(run.OfflineClient):
   def call(self,payload,meta):
    response,receipt=super().call(payload,meta)
    receipt['qa04_actual_payload']=payload
    d=payload.get('input',{})
    catalog=d.get('catalog',d.get('source_catalog',d.get('manifest',[])))
    if feature=='C06' and any(x.get('withdrawn_ranges') for x in catalog):assert 'WITHDRAWN PRIVATE FACT' not in json.dumps(payload),payload
    if feature=='C04' and any(x.get('id')=='artifact/artifact-a/section/summary' and x.get('revision')==2 for x in catalog):assert 'Old public summary.' not in json.dumps(payload),payload
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

# A real 3-line source supports range authority and actual background production.
for dp in ['05']:
 folder=ROOT/('dp'+dp+'-C06');folder.mkdir(exist_ok=True)
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

# 05 C04: section result updates preserve the other section/task and update later handoff evidence.
folder=ROOT/'dp05-C04';path=folder/'original-public.txt';path.write_text('Original public fact.\nOriginal limit remains.\n')
source={'id':'original','revision':1,'title':'Original public source','path':str(path),'line_count':2,'allowed_lines':[1,2],'outline':[{'section':'original facts','lines':[1,2]}],'revoked':False}
span={'source_id':'original','revision':1,'start_line':1,'end_line':1}
sections=[{'section_id':'summary','revision':1,'text':'Old public summary.','source_spans':[span]},{'section_id':'limits','revision':1,'text':'Unchanged public limit.','source_spans':[span]}]
initial={'kind':'artifact_received','artifact_id':'artifact-a','task_id':'T-PC','goal_id':'initial-artifact','stimulus_id':'initial-artifact','sections':sections}
update={'kind':'artifact_section_update','artifact_id':'artifact-a','task_id':'T-PC','section_id':'summary','base_revision':1,'new_revision':2,'text':'New public summary.','source_spans':[span],'goal_id':'section-update','stimulus_id':'section-update'}
state={'task_id':'T-PC','request_revision':1,'condition_revision':1,'access_epoch':1,'constraints':{},'exclusions':[],'allowed_actions':['draft_only'],'already_completed_actions':[],'unknown_actions':[],'required_source_ids':['artifact/artifact-a/section/summary']}
events=[{'kind':'source','source':source},{'kind':'adopt','state':state},initial,{'kind':'acquire','reads':[{'source_id':'artifact/artifact-a/section/summary','start_line':1,'end_line':1}]},{'kind':'queue_section','artifact_id':'artifact-a','task_id':'T-PC','section_id':'summary'},{'kind':'queue_section','artifact_id':'artifact-a','task_id':'T-PC','section_id':'limits'},update,update,{**update,'task_id':'wrong-task'}, {'kind':'delegate','goal_id':'future','stimulus_id':'future','delegation_id':'future-delegation','delegation_revision':1,'utterance':'Use the updated public artifact for a small draft, no external actions.'}]
fixture={'id':'qa04-dp05-C04','dp':'05','description':'Actual section update and subsequent receiver handoff','events':events}
for r in execute('05','C04',fixture):
 assert len(data(r,'ARTIFACT_UPDATE_REJECTED'))==2
 changes=data(r,'PUBLIC_ARTIFACT_CHANGE');assert len(changes)==2
 actual=result(r);art=actual['artifacts']['artifact-a'];assert art['sections']['summary']['revision']==2;assert art['sections']['limits']['revision']==1
 assert len(art['sections']['summary']['delivery_history'])==2;assert len(art['sections']['limits']['delivery_history'])==1
 assert [q['revision'] for q in actual['pending_sections']]==[2,1]
 contract=data(r,'HANDOFF_ADMISSION')[-1];manifest={s['id']:s for s in contract['manifest']};assert manifest['artifact/artifact-a/section/summary']['revision']==2
 assert len(data(r,'GOAL_COMPLETE'))==1
 receiver=data(r,'AGENT_RESULT_AVAILABLE')[-1]['result'];assert 'New public summary.' in receiver['text'];assert 'Old public summary.' not in receiver['text']

# Archive per-alternative actual patches; dormant producer edits are not charged to alternatives without that producer.
for dp,feature in [('05','C04'),('05','C06')]:
 folder=ROOT/('dp'+dp+'-'+feature);changed_by_variant={}
 for variant in ['A','B','C']:
  scratch=Path('/private/tmp/four-dp-review-qa04-reuse-dp'+dp+'-'+feature+'-'+variant);changed=[];diff=[]
  for p in sorted((scratch/'src').rglob('*.rs')):
   old=BASE/p.relative_to(scratch)
   if old.read_bytes()!=p.read_bytes():changed.append(str(p.relative_to(scratch)));diff.extend(difflib.unified_diff(old.read_text().splitlines(True),p.read_text().splitlines(True),fromfile='frozen/'+str(p.relative_to(scratch)),tofile=variant+'/'+str(p.relative_to(scratch))))
  changed_by_variant[variant]=changed
  (folder/('actual-'+variant+'.patch')).write_text(''.join(diff))
  test=subprocess.run(['cargo','test','--offline','--manifest-path',str(scratch/'Cargo.toml'),'--','--nocapture'],capture_output=True,text=True);(folder/('tests-'+variant+'.txt')).write_text(test.stdout+'\n'+test.stderr);assert test.returncode==0
 provenance=json.loads((folder/'provenance.json').read_text());started=datetime.datetime.now(datetime.timezone.utc);now=datetime.datetime.now(datetime.timezone.utc)
 summary={'dp':dp,'feature':feature,'variants':['A','B','C'],'changed_modules_by_variant':changed_by_variant,'changed_module_count_each_variant':{v:len(changed_by_variant[v]) for v in ['A','B','C']},'acceptance':'PASS','test_and_replay_scope':'Code functionality only, not model semantic or latency performance','actual_provider_calls':0,'baseline_freeze_sha256':'corrected-scratch-see-provenance','finished_at':now.isoformat(),'development_elapsed_seconds_including_other_work':(now-started).total_seconds(),'rebase_needed_if_formal_baseline_changes':True,'counting_note':'Exact independent variant patch excludes dormant producer code absent from that alternative; original combined-tree provisional summary retained.'}
 write(folder/'summary.json',summary);print(json.dumps(summary))
