"""Actual scratch Rust C05/C06 acceptance, external mock only, zero provider calls."""
import argparse,copy,difflib,hashlib,json,subprocess
from pathlib import Path
import sys
sys.path.insert(0,'/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v2/frozen/source/benchmark/architecture/four_dp_poc_v2')
import offline_dp04
BASE=Path('/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v2/frozen/source/benchmark/architecture/four_dp_poc_v2')
REPO=Path('/Users/changbae/Workspace/via-sw-architecture')
RESULT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
FROZEN=RESULT/'calibration-v2/frozen/source/prototypes/candidates/four_dp_poc_v2'
FIXTURES=json.loads((BASE/'fixtures/dp04.json').read_text())
NODE='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node'
SHARP='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp'

def pixel(path,x,y):
 script="require(process.argv[1])(process.argv[2]).removeAlpha().raw().toBuffer({resolveWithObject:true}).then(({data,info})=>{const n=(Number(process.argv[4])*info.width+Number(process.argv[3]))*info.channels;process.stdout.write(JSON.stringify(Array.from(data.subarray(n,n+3))))});"
 return json.loads(subprocess.check_output([NODE,'-e',script,SHARP,str(path),str(x),str(y)],text=True))

def drive(crate,case,failed_first=False,forbidden_probe=False):
 proc=subprocess.Popen([str(crate/'target/debug/four_dp_poc')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
 proc.stdin.write(json.dumps(case,ensure_ascii=False)+'\n');proc.stdin.flush();events=[];calls=[]
 for line in proc.stdout:
  msg=json.loads(line)
  if msg.get('type')=='model_call':
   payload=msg['payload'];test_payload=copy.deepcopy(payload)
   # Model double only: use authored source geometry for already redacted observation,
   # then remove denied candidates. Candidate message and actual PNG remain redacted.
   if case.get('region_withdrawals'):
    value=test_payload['input'];original=value.get('original',value);r=original.get('observations',original)
    if 'frames' in r:
     for f in r['frames']:
      if f.get('region_redaction_applied'):f['image_path']=str(BASE/'assets/screen'/str(f['frame_id']+'.png'))
   value=offline_dp04.respond(test_payload,{'stage':msg['stage'],'dp04_probe_fail_revision_one':failed_first})
   if case.get('region_withdrawals') and 'candidates' in value and any(f.get('region_redaction_applied') for f in test_payload['input'].get('frames',[])):
    withdrawal=case['region_withdrawals'][0];b=withdrawal['rect']
    def allowed(c):
     a=c['bbox'];return c['frame_id']!=withdrawal['frame_id'] or not (a[0]<b[2] and b[0]<a[2] and a[1]<b[3] and b[1]<a[3])
    value['candidates']=[c for c in value['candidates'] if allowed(c)]
   if case.get('region_withdrawals') and msg['stage']=='dp04.candidate_binding':
    # Mock semantic 'middle' uses actual frame geometry, not candidate count/order.
    data=payload['input'];width=case['revisions'][0]['frames'][0]['width']
    for adoption in value['adoptions']:
     expression=next(x for x in data['speech']['expressions'] if x['expression_id']==adoption['expression_id'])
     if '가운데' in expression['meaning']:
      chosen=min(data['screen']['candidates'],key=lambda c:abs((c['bbox'][0]+c['bbox'][2])/2-width/2))
      adoption['targets']=[chosen['target_id']];adoption['extents']=[chosen]
   if forbidden_probe:
    data=payload['input'];original=data.get('original',data);r=original.get('observations',original)
    redacted=any(f.get('region_redaction_applied') for f in r.get('frames',[]))
    if redacted or (msg['stage']=='dp04.candidate_binding' and any(c['target_id']=='blocked-probe' for c in data['screen']['candidates'])):
     withdrawal=case['region_withdrawals'][0];extent={'target_id':'blocked-probe','frame_id':withdrawal['frame_id'],'bbox':withdrawal['rect'],'label':'Deliberate denied-region negative probe','kind':'card'}
     if 'candidates' in value:value['candidates']=[extent]
     if 'adoptions' in value:
      for adoption in value['adoptions']:adoption['targets']=['blocked-probe'];adoption['extents']=[extent];adoption['evidence']='Negative probe must be rejected by actual source-region policy'
   calls.append({'stage':msg['stage'],'payload':payload,'response':value,'external_mock':True})
   proc.stdin.write(json.dumps({'call_id':msg['call_id'],'result':value},ensure_ascii=False)+'\n');proc.stdin.flush()
  else:events.append(msg)
 proc.wait();stderr=proc.stderr.read();assert proc.returncode==0,stderr
 result=next((e['data'] for e in events if e.get('kind')=='EPISODE_RESULT'),None)
 assert result is not None,events[-1]
 return {'events':events,'calls':calls,'result':result,'provider_calls':0,'stderr':stderr}

def patches(crate,out):
 names=sorted(set(str(p.relative_to(FROZEN)) for p in FROZEN.rglob('*.rs'))|set(str(p.relative_to(crate)) for p in crate.rglob('*.rs') if 'target/' not in str(p)))
 changed=[];diff=[]
 for name in names:
  a=(FROZEN/name).read_text() if (FROZEN/name).exists() else '';b=(crate/name).read_text() if (crate/name).exists() else ''
  if a!=b:changed.append(name);diff.extend(difflib.unified_diff(a.splitlines(True),b.splitlines(True),fromfile='baseline/'+name,tofile='scratch/'+name))
 (out/'actual.patch').write_text(''.join(diff));return changed

def main():
 ap=argparse.ArgumentParser();ap.add_argument('task',choices=['C05','C06']);ap.add_argument('--variant',choices=['A','B']);a=ap.parse_args();variants=[a.variant] if a.variant else ['A','B'];crate=Path('/private/tmp/four_dp_poc_v2_qa04_dp04_calibration_v2_'+a.task+('_'+a.variant if a.variant else ''));out=RESULT/'qa04/dp04-rebased-calibration-v2'/a.task;out=out/a.variant if a.variant else out;out.mkdir(parents=True,exist_ok=True)
 checks=[]
 for variant in variants:
  case=copy.deepcopy(next(f for f in FIXTURES if f['late_result_replay']));case['variant']=variant
  if a.task=='C05':
   case['explanation_queries']=[{'expression_id':e['expression_id'],'revision':r['revision'],'adoption_id':case['id']+':grounding:r'+str(r['revision'])} for r in case['revisions'] for e in r['expressions']]
   case['explanation_queries'].append({'expression_id':'keep-first','revision':1,'adoption_id':'not-delivered'})
   for failed in [False,True]:
    trace=drive(crate,case,failed);(out/(variant+('-failed-first' if failed else '-normal')+'.json')).write_text(json.dumps(trace,ensure_ascii=False,indent=2)+'\n')
    records=trace['result']['explanations'];found=[q['response']['record'] for q in records if q['response']['status']=='FOUND']
    valid_count=2 if failed else 4
    checks.append({'variant':variant,'probe':'failed-proposal-excluded' if failed else 'delivered-provenance-query','passed':len(found)==valid_count and records[-1]['response']['status']=='NOT_FOUND' and all(r['basis']=='ACTUALLY_DISPLAYED_ADOPTION' and len(r['source_extents'])==1 and r['source_extents'][0]['frame_id']=='correction' and r['source_extents'][0]['source_revision']==1 for r in found) and (variant!='B' or all(r['candidate_read_set']['screen_generation']>=1 for r in found))})
  else:
   # A valid two-region binding followed by real external region authorization change.
   case['late_result_replay']=False
   first=copy.deepcopy(case['revisions'][0]);second=copy.deepcopy(first);second['revision']=2;second['transcript']='가운데 그래프만 새 작업의 자료로 써줘.';second['expressions']=[{'expression_id':'permitted-middle','text':'가운데 그래프','start_ms':2000,'end_ms':2500,'time_uncertainty_ms':20}]
   case['revisions']=[first,second]
   event={'frame_id':'correction','rect':[40,110,335,530],'after_revision':1,'origin':'EXTERNAL_AUTHORIZATION_REGION_WITHDRAWAL'}
   case['region_withdrawals']=[event,copy.deepcopy(event)]
   trace=drive(crate,case);(out/(variant+'-region-withdrawal.json')).write_text(json.dumps(trace,ensure_ascii=False,indent=2)+'\n')
   reports=trace['result']['region_reports'];initial=reports[0];view=initial['permitted_reread']['frames'][0]
   black=pixel(view['image_path'],100,150)==[0,0,0]
   same_outside=pixel(view['image_path'],500,150)==pixel(first['frames'][0]['image_path'],500,150)
   reread_calls=[c for c in trace['calls'] if any('via-qa04-region-views' in p for p in c['payload'].get('images',[]))]
   forbidden_publication=initial['held_old_proposal_accepted'] is False and initial['held_old_proposal_rejection']=='STALE_REGION_POLICY'
   kept=initial['prior_binding_admission']['retained_targets']==['C2'] and initial['prior_binding_admission']['denied_expression_ids']==['keep-first']
   candidate_ok=variant=='A' or ('C1' in initial['candidate_admission']['ineligible_candidate_ids'] and 'C2' in initial['candidate_admission']['eligible_candidate_ids'])
   final=next(x for x in trace['result']['adoptions'] if x['expression_id']=='permitted-middle')
   checks.append({'variant':variant,'probe':'actual-redaction-admission-and-permitted-regrounding','passed':black and same_outside and forbidden_publication and kept and candidate_ok and len(reread_calls)>=1 and final['targets']==['C2'] and all(x['policy_generation']==1 for x in reports),'black_pixel_verified':black,'disjoint_pixel_equal_original':same_outside,'held_old_proposal_rejected':forbidden_publication,'disjoint_binding_retained':kept,'producer_candidates_policy_eligible':candidate_ok,'new_model_messages_use_redacted_paths':len(reread_calls),'permitted_regrounding_targets':final['targets'],'idempotent_epoch':all(x['policy_generation']==1 for x in reports)})
   denied=drive(crate,case,forbidden_probe=True);(out/(variant+'-denied-new-proposal.json')).write_text(json.dumps(denied,ensure_ascii=False,indent=2)+'\n')
   failures=[e for e in denied['events'] if e['kind']=='GROUNDING_CONSISTENCY_REJECTED' and 'SOURCE_REGION_WITHDRAWN' in json.dumps(e['data'])]
   finals=[x for x in denied['result']['adoptions'] if x['expression_id']=='permitted-middle']
   denied_pass=bool(failures) and finals and all(not x['targets'] for x in finals) and any(e['kind']=='GOAL_FAILED' and e['data'].get('revision')==2 for e in denied['events'])
   checks.append({'variant':variant,'probe':'new-policy-denied-region-proposal-blocked-after-owner-recovery','passed':bool(denied_pass),'actual_region_guard_rejections':len(failures),'provider_calls':0})
 changed=patches(crate,out)
 command=['cargo','test','--offline','--manifest-path',str(crate/'Cargo.toml'),'dp04'];test=subprocess.run(command,capture_output=True,text=True);(out/'regression.log').write_text(test.stdout+test.stderr)
 shutil_manifest={str(p.relative_to(crate)):hashlib.sha256(p.read_bytes()).hexdigest() for p in crate.rglob('*.rs') if 'target/' not in str(p)}
 summary={'task':a.task,'baseline_calibration_freeze_sha256':'2e2da0eb88895cd02036f2e4219ac80373425202a810cb63b639e908c0a0f0e1','status':'PASS' if all(x['passed'] for x in checks) and test.returncode==0 else 'FAIL','checks':checks,'changed_operating_rust_modules':changed,'changed_operating_module_count':len(changed),'counts_by_alternative':{v:len(changed) for v in variants},'scratch_source_sha256':shutil_manifest,'provider_calls':0,'not_model_quality_or_latency_measurement':True,'scope':'Actual scratch Rust functionality and original regressions; external vision semantic double plus actual PNG redaction/pixel checks.'}
 (out/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:summary[k] for k in ['task','status','changed_operating_rust_modules','counts_by_alternative','provider_calls']}));assert summary['status']=='PASS',checks
if __name__=='__main__':main()
