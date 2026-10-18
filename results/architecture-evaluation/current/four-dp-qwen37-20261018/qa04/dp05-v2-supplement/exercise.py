"""DP05 v2 actual Rust C01/C03 scratch changes + common receiver COMPLETE/READ traversal."""
import json,pathlib,subprocess,tempfile,shutil,hashlib,difflib
OUT=pathlib.Path(__file__).resolve().parent;QA=OUT.parent;ROOT=QA.parents[4]
BASE=pathlib.Path('/private/tmp/four_dp_poc_dp05_v2');old=json.loads((QA/'summary.json').read_text());old_scratch=pathlib.Path(old['scratch_root'])
assert hashlib.sha256((BASE/'src/dp05.rs').read_bytes()).hexdigest()=='6be52581bd04a4b8c975b5cc30df29ca9b625d01bbf67821fda3c563398254b3'
TMP=pathlib.Path(tempfile.mkdtemp(prefix='via-qa04-dp05v2-'));logs=[]
manifest={str(p.relative_to(BASE)):hashlib.sha256(p.read_bytes()).hexdigest() for p in BASE.rglob('*') if p.is_file() and 'target'not in p.relative_to(BASE).parts}
(OUT/'baseline-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
fixture=json.loads((ROOT/'benchmark/architecture/four_dp_poc/fixtures/dp05.json').read_text())[1]
def dump(p,x):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def migrate(v):
 if isinstance(v,list):return [migrate(x) for x in v]
 if not isinstance(v,dict):return v
 x={k:migrate(y) for k,y in v.items()}
 if 'id'in x and 'revision'in x and ('path'in x or 'text'in x):x['version']={'number':x.pop('revision')}
 return x
checks=[];mods={}
for change in ['C01','C03']:
 crate=TMP/change;shutil.copytree(BASE,crate,ignore=shutil.ignore_patterns('target'))
 if change=='C01':shutil.copy2(old_scratch/'C01/src/model_wire.rs',crate/'src/model_wire.rs')
 else:
  shutil.copy2(old_scratch/'C03/src/source_contract.rs',crate/'src/source_contract.rs')
  m=crate/'src/main.rs';s=m.read_text();needle='let input:Value=serde_json::from_str(&line).unwrap();';assert needle in s;m.write_text(s.replace(needle,needle+'\n    let input=source_contract::normalize_input(&input);'))
 changed=[];patch=[]
 for f in sorted((BASE/'src').rglob('*.rs')):
  rel=f.relative_to(BASE);g=crate/rel
  if f.read_bytes()!=g.read_bytes():changed.append(str(rel));patch+=list(difflib.unified_diff(f.read_text().splitlines(True),g.read_text().splitlines(True),fromfile='dp05v2-baseline/'+str(rel),tofile=change+'/'+str(rel)))
 mods[change]=changed;(OUT/(change+'.patch')).write_text(''.join(patch))
 for cmd in [['cargo','test','--offline','--bin','four_dp_poc'],['cargo','build','--offline','--bin','four_dp_poc']]:
  z=subprocess.run(cmd,cwd=crate,text=True,capture_output=True);logs.append({'cmd':cmd,'crate':str(crate),'returncode':z.returncode,'stdout':z.stdout,'stderr':z.stderr});assert z.returncode==0,z.stderr
 for variant in 'ABC':
  canonical={**fixture,'variant':variant};case=migrate(canonical) if change=='C03' else canonical;prefix=f'{change}-dp05-v2-{variant}-delegation';dump(OUT/'probes'/(prefix+'-input.json'),case)
  p=subprocess.Popen([str(crate/'target/debug/four_dp_poc')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True);p.stdin.write(json.dumps(case,ensure_ascii=False)+'\n');p.stdin.flush();events=[];calls=[]
  for line in p.stdout:
   m=json.loads(line)
   if m.get('type')!='model_call':events.append(m);continue
   calls.append(m);payload=m.get('payload',m.get('request',{}).get('context',{}));inp=payload['input'];stage=m['stage'];r={}
   if stage=='dp05_produce_work_record':r={'blocks':[{'block_id':x['source_id'],'explanation':'CONTRACT_PROBE_ONLY','source_spans':[{k:x[k] for k in ['source_id','revision','start_line','end_line']}]}for x in inp['public_evidence']],'conditions':[],'unknowns':[],'coverage':[]}
   elif stage=='dp05_select_handoff_details':r={'reads':[{'source_id':'public-comparison','start_line':1,'end_line':4},{'source_id':'public-quote','start_line':1,'end_line':3}],'summary':'CONTRACT_PROBE_ONLY','selected_record_blocks':[]}
   elif stage=='dp05_extract_record':r={'reads':[],'summary':'CONTRACT_PROBE_ONLY','selected_record_blocks':[b['block_id'] for b in inp['work_record']['blocks']]}
   elif stage=='dp05_compose_capsule':r={'summary':'CONTRACT_PROBE_ONLY','passages':[{k:x[k] for k in ['source_id','revision','start_line','end_line']}for x in inp['actual_passages']],'unknowns':[]}
   elif stage=='dp05_receiver_select_or_complete':
    capsule=inp['capsule'];spans=[]
    if capsule:
     spans+=capsule.get('passages',[])
     for b in capsule.get('record_blocks',[]):spans+=b['source_spans']
    if spans:r={'mode':'COMPLETE','reads':[],'reason':'contract complete branch exercise','result':{'text':'CONTRACT_PROBE_ONLY','used_source_ids':[spans[0]['source_id']],'source_spans':[spans[0]],'facts':['CONTRACT_PROBE_ONLY'],'constraints_respected':[],'actions_performed':[],'uncertainties':[]}}
    else:r={'mode':'READ','result':None,'reads':[{'source_id':'public-quote','start_line':1,'end_line':3}],'reason':'no supplied capsule facts'}
   elif stage=='dp05_receiver_result':r={'text':'CONTRACT_PROBE_ONLY','used_source_ids':['public-quote'],'source_spans':[{k:x[k] for k in ['source_id','revision','start_line','end_line']}for x in inp['tool_passages']],'facts':['CONTRACT_PROBE_ONLY'],'constraints_respected':[],'actions_performed':[],'uncertainties':[]}
   else:raise AssertionError(stage)
   if change=='C01':
    assert m['wire_version']==2 and m.get('payload')is None
    response={'call_id':m['call_id'],'provider_response':{'json':r,'usage':{'input_tokens':17,'output_tokens':9}}}
   else:response={'call_id':m['call_id'],'result':r,'usage':{'prompt_tokens':17,'completion_tokens':9}}
   p.stdin.write(json.dumps(response)+'\n');p.stdin.flush()
  p.wait(timeout=10);stderr=p.stderr.read();assert p.returncode==0,stderr
  kinds=[e.get('kind')for e in events];assert 'EPISODE_ERROR'not in kinds;assert 'AGENT_INGRESS'in kinds and 'RECEIVER_READ_RECEIPT'in kinds
  result=next(e['data']for e in events if e.get('kind')=='EPISODE_RESULT');assert result['handoffs'][0]['mandatory']['disposition']=='AUTHORIZED'
  branch=next(e['data']['branch']for e in events if e.get('kind')=='RECEIVER_BRANCH');assert branch==('READ_ORIGINAL_SOURCES'if variant=='C'else'COMPLETE_FROM_SUPPLIED_CAPSULE')
  # Verify canonical revision survived the migrated admission and every model/source boundary.
  assert all(s['revision']==1 for s in result['handoffs'][0]['manifest'])
  check={'change':change,'dp':'05','version':'BASIC_DP05_V2','variant':variant,'fixture':fixture['id'],'model_calls':len(calls),'source_reads':sum(e.get('kind')=='HANDOFF_SOURCE_READ'for e in events),'receiver_branch':branch,'result':'PASS','live_edits':False,'api_spend_usd':0,'scope':'actual Rust protocol integration with deterministic contract responder; no semantic quality claim'};checks.append(check);dump(OUT/'probes'/(prefix+'-trace.json'),{'check':check,'events':events,'calls':calls,'stderr':stderr})
assert len(mods['C01'])==1 and len(mods['C03'])==2
assert all(hashlib.sha256((BASE/p).read_bytes()).hexdigest()==sha for p,sha in manifest.items())
summary={'checks':checks,'count':len(checks),'changes':mods,'mean_changed_operational_modules':1.5,'scratch_root':str(TMP),'source_baseline_unchanged':True,'model_spend_usd':0,'baseline_dp05_sha256':manifest['src/dp05.rs'],'scope':'Same C01/C03 compatibility changes reexecuted with actual DP05v2 receiver protocol. COMPLETE A/B, READ C. All previous QA04 evidence retained.'};dump(OUT/'summary.json',summary);dump(OUT/'commands.json',logs)
old['dp05_v2_supplement']={'path':str(OUT/'summary.json'),'checks_passed':6,'mean':1.5};dump(QA/'summary.json',old)
print(json.dumps(summary,indent=2))
