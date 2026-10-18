"""Supplement QA04 with actual DP05 delegation paths in the same migrated scratch crates."""
import copy,json,pathlib,subprocess
OUT=pathlib.Path(__file__).resolve().parent
ROOT=OUT.parents[4]
summary=json.loads((OUT/'summary.json').read_text());scratch=pathlib.Path(summary['scratch_root'])
fixture=json.loads((ROOT/'benchmark/architecture/four_dp_poc/fixtures/dp05.json').read_text())[1]
def dump(p,x):p.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def migrate(v):
    if isinstance(v,list):return [migrate(x) for x in v]
    if not isinstance(v,dict):return v
    x={k:migrate(y) for k,y in v.items()}
    if 'id'in x and 'revision'in x and ('path'in x or 'text'in x):x['version']={'number':x.pop('revision')}
    return x
checks=[]
for change in ['C01','C03']:
    crate=scratch/change
    for variant in 'ABC':
        canonical={**fixture,'variant':variant};case=migrate(canonical) if change=='C03' else canonical
        prefix=f'{change}-dp05-{variant}-delegation';dump(OUT/'probes'/(prefix+'-input.json'),case);dump(OUT/'probes'/(prefix+'-canonical.json'),canonical)
        if change=='C03':subprocess.run([str(crate/'target/debug/contract_probe'),str(OUT/'probes'/(prefix+'-input.json')),str(OUT/'probes'/(prefix+'-canonical.json'))],check=True,capture_output=True)
        p=subprocess.Popen([str(crate/'target/debug/four_dp_poc')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        p.stdin.write(json.dumps(case,ensure_ascii=False)+'\n');p.stdin.flush();events=[];calls=[]
        for line in p.stdout:
            m=json.loads(line)
            if m.get('type')!='model_call':events.append(m);continue
            calls.append(m);payload=m.get('payload',m.get('request',{}).get('context',{}));inp=payload['input'];stage=m['stage'];r={}
            if stage=='dp05_produce_work_record':
                r={'blocks':[{'block_id':x['source_id'],'explanation':'CONTRACT_PROBE_ONLY','source_spans':[{k:x[k] for k in ['source_id','revision','start_line','end_line']}]} for x in inp['public_evidence']],'conditions':[],'unknowns':[],'coverage':[]}
            elif stage=='dp05_select_handoff_details':r={'reads':[{'source_id':'public-comparison','start_line':1,'end_line':4},{'source_id':'public-quote','start_line':1,'end_line':3}],'summary':'CONTRACT_PROBE_ONLY','selected_record_blocks':[]}
            elif stage=='dp05_extract_record':r={'reads':[],'summary':'CONTRACT_PROBE_ONLY','selected_record_blocks':[b['block_id'] for b in inp['work_record']['blocks']]}
            elif stage=='dp05_compose_capsule':r={'summary':'CONTRACT_PROBE_ONLY','passages':[{k:x[k] for k in ['source_id','revision','start_line','end_line']} for x in inp['actual_passages']],'unknowns':[]}
            elif stage=='dp05_receiver_select_reads':r={'reads':[{'source_id':'public-quote','start_line':1,'end_line':3}],'reason':'contract source read exercise'}
            elif stage=='dp05_receiver_result':r={'text':'CONTRACT_PROBE_ONLY','used_source_ids':['public-quote'],'facts':[],'constraints_respected':[],'actions_performed':[],'uncertainties':[]}
            else:raise AssertionError(stage)
            if change=='C01':
                assert m['wire_version']==2 and m.get('payload') is None
                response={'call_id':m['call_id'],'provider_response':{'json':r,'usage':{'input_tokens':17,'output_tokens':9}}}
            else:response={'call_id':m['call_id'],'result':r,'usage':{'prompt_tokens':17,'completion_tokens':9}}
            p.stdin.write(json.dumps(response)+'\n');p.stdin.flush()
        p.wait(timeout=10);stderr=p.stderr.read();assert p.returncode==0,stderr
        kinds=[e.get('kind') for e in events];assert 'EPISODE_ERROR'not in kinds
        assert 'AGENT_INGRESS'in kinds and 'RECEIVER_READ_RECEIPT'in kinds and 'HANDOFF_SOURCE_READ'in kinds
        result=next(e['data'] for e in events if e.get('kind')=='EPISODE_RESULT')
        assert len(result['handoffs'])==1 and result['handoffs'][0]['mandatory']['disposition']=='AUTHORIZED'
        assert any(c['stage']=='dp05_receiver_result' for c in calls)
        check={'change':change,'dp':'05','variant':variant,'fixture':fixture['id'],'calls':len(calls),'exit':p.returncode,'result':'PASS','source_reads':sum(e.get('kind')=='HANDOFF_SOURCE_READ' for e in events),'wire_actual_traversal':True,'receiver_actual_traversal':True,'model':'NO_MODEL; deterministic contract responder only','semantics_claimed':False}
        checks.append(check);dump(OUT/'probes'/(prefix+'-trace.json'),{'check':check,'calls':calls,'events':events,'stderr':stderr})
summary['supplemental_checks']=checks
summary['total_contract_checks']=len(summary['checks'])+len(checks)
summary['supplement_note']='Original20 probes preserved. Six DP05E2 actualdelegation scratchprobes exercise source reads + modelwire + commonreceiver for ABC under bothmigrations.'
dump(OUT/'summary.json',summary)
with (OUT/'README.md').open('a') as f:f.write('\nSupplement: six DP05 E2 delegation probes add actual wire, authorized source reads and receiver traversal for A/B/C under both migrations. Original twenty traces remain intact. All 26 contract checks pass.\n')
print(json.dumps(checks,indent=2))
