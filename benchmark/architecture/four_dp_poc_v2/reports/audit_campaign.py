"""Independent read-only campaign audit. Default: bounded preparation, --final: all evidence.
Never imports a runner/client, executes candidates, contacts a provider, or prints secrets.
"""
import argparse,collections,hashlib,importlib.util,json,math,re,sys
from decimal import Decimal
from pathlib import Path

HERE=Path(__file__).resolve().parent
REPO=next(p for p in HERE.parents if (p/'benchmark/architecture/four_dp_poc_v2').is_dir())
DEFAULT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
MODEL='qwen3.7-plus-2026-05-26'
Q04_CAPS={'dp04.joint_grounding':2600,'dp04.joint_source_reread':2600,'dp04.screen_producer':3000,'dp04.screen_source_expansion':3000,'dp04.screen_output_repair':3000,'dp04.speech_producer':1400,'dp04.speech_output_repair':1400,'dp04.candidate_binding':2000}
SECRETS=[('authorization_header',re.compile(r'"(?:authorization|proxy.authorization)"\s*:',re.I)),('sensitive_credential_field',re.compile(r'"(?:api[_-]?key|access[_-]?token|client[_-]?secret|DASHSCOPE_API_KEY|OPENAI_API_KEY)"\s*:',re.I)),('environment_dump_field',re.compile(r'"(?:environment_variables|process_environment|os_environ)"\s*:\s*\{',re.I)),('bearer_credential',re.compile(r'\bBearer\s+[A-Za-z0-9._~-]{16,}')),('provider_key_pattern',re.compile(r'\bsk-[A-Za-z0-9_-]{20,}'))]
def read(p,default=None):return json.loads(p.read_text()) if p.is_file() else default
def lines(p):
    with p.open() as f:
        for line in f:
            if line.strip():yield json.loads(line)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def key(r):return (r['dp'],r['episode'],r['variant'],r.get('repetition',r.get('rep')))
def union(spans,lo,hi):
    end=lo;total=0
    for a,b in sorted((max(a,lo),min(b,hi)) for a,b in spans if b>lo and a<hi):
        if b>max(a,end):total+=b-max(a,end);end=max(end,b)
    return total
def close(a,b):return a is None and b is None or a is not None and b is not None and math.isclose(a,b,abs_tol=1e-6,rel_tol=1e-9)
def keyed(v):return {x['id']:x for x in v} if isinstance(v,list) else v
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('campaign',type=Path,nargs='?',default=DEFAULT);p.add_argument('--final',action='store_true');p.add_argument('--baseline-audit',action='store_true',help='Interim full original BASIC120+calibration26 audit during the approved model-free window; not the final all-phase audit');p.add_argument('--sample-records',type=int,default=2);a=p.parse_args()
    if a.baseline_audit and a.final:raise SystemExit('Choose the interim baseline audit or final audit')
    if a.baseline_audit:a.final=True
    root=a.campaign.resolve();repo=next(x for x in root.parents if (x/'benchmark/architecture').exists());out=root/'analysis/final-review';out.mkdir(exist_ok=True);checks=[];digest_cache={};scanned=set();secret_presence=[]
    def check(name,ok,details=None,limited=False,pending=False):checks.append({'name':name,'status':'PENDING' if pending else 'FAIL' if not ok else 'LIMITED' if limited else 'PASS','details':details})
    def digest(path):
        path=Path(path)
        if path not in digest_cache:digest_cache[path]=sha(path)
        return digest_cache[path]
    def scan(path):
        path=Path(path)
        if not a.final or path in scanned or not path.is_file():return
        scanned.add(path);text=path.read_text(errors='replace')
        for label,pattern in SECRETS:
            count=sum(1 for _ in pattern.finditer(text))
            if count:secret_presence.append({'path':str(path),'pattern':label,'presence_count':count})
    basic=root/'basic-v1';freeze=read(basic/'freeze.json',{});plan=read(basic/'plan.json',{});population=read(basic/'population.json',{});jobs=[key(x) for x in plan.get('jobs',[])];expected=read(out/'expected-basic-population.json',{});expected_keys=set(map(tuple,expected.get('expected_keys',[])))
    check('BASIC planned120 unique and independent manifest key agreement',len(jobs)==len(set(jobs))==120 and set(jobs)==expected_keys,{'planned':len(jobs),'unique':len(set(jobs))})
    q1=q3=0
    for dp,variants in [('01','ABC'),('02','AB'),('04','AB'),('05','ABC')]:
        manifests=keyed(read(basic/f'frozen/source/benchmark/architecture/four_dp_poc_v2/goal_manifests/dp{dp}.json',{}))
        q1+=sum(len(x['qa01_stimuli']) for x in manifests.values())*len(variants)*3;q3+=sum(len(x['qa03_goals']) for x in manifests.values())*len(variants)*3
    check('Fixed QA01/QA03 population independently totals261/243',q1==population.get('qa01_total')==261 and q3==population.get('qa03_total')==243,{'qa01':q1,'qa03':q3})
    phases=[]
    for f in sorted(root.rglob('freeze.json')):
        if any(x in f.relative_to(root).parts for x in ['frozen','qa04','analysis','tactics-preparation']):continue
        if a.baseline_audit and f.parent.name not in ('basic-v1','calibration-v1','calibration-v2','calibration-v3'):continue
        if (f.parent/'MODEL-CALLS-AUTHORIZED.json').is_file() and (f.parent/'runs.jsonl').is_file():phases.append(f.parent)
    all_records=[];global_calls={};call_occurrences=collections.Counter();sample_left=a.sample_records;phase_notes=[];observations=[];failure_records=[];memory_notes=[]
    for phase in phases:
        f=read(phase/'freeze.json');auth=read(phase/'MODEL-CALLS-AUTHORIZED.json');fh=digest(phase/'freeze.json');bad=[]
        for rel,wanted in f.get('source',{}).items():
            source=phase/'frozen/source'/rel
            if not source.is_file() or digest(source)!=wanted:bad.append(rel)
        for source,meta in f.get('evaluator_external_sources',{}).items():
            snap=Path(meta['snapshot']);original=Path(source)
            if not snap.is_file() or digest(snap)!=meta['sha256']:bad.append('external_snapshot:'+source)
            if original.is_file() and digest(original)!=meta['sha256']:bad.append('external_original_changed:'+source)
        binary=Path(f['binary']['path']);check(str(phase.relative_to(root))+': freeze/source/external/binary pins',not bad and binary.is_file() and digest(binary)==f['binary']['sha256'] and auth.get('freeze_sha256')==fh,{'source_files':len(f.get('source',{})),'mismatches':bad,'freeze_sha256':fh,'binary_sha256':f['binary']['sha256']})
        effective=f.get('effective_candidate_source')
        if effective:
            effective_bad=[];snapshot=Path(effective['snapshot']);baseline=Path(effective['baseline_source_snapshot']);pin_map=effective.get('sha256',{})
            for relative,wanted in pin_map.items():
                actual=snapshot/relative
                if not actual.is_file() or digest(actual)!=wanted:effective_bad.append(relative)
            baseline_map={str(p.relative_to(baseline)):digest(p) for p in baseline.rglob('*') if p.is_file() and 'target' not in p.relative_to(baseline).parts}
            actual_changes=sorted(relative for relative in set(baseline_map)|set(pin_map) if baseline_map.get(relative)!=pin_map.get(relative))
            numbered=auth.get('allowed_tactic')=='ORIGINAL_LINE_NUMBERS';corrected=auth.get('phase')=='BASIC_CORRECTED'
            expected_changes=['src/dp01.rs'] if f.get('tactic')=='A2' and not numbered and not corrected else ['src/dp05.rs','src/dp05_handoff.rs','src/dp05_sources.rs'] if numbered or corrected else None
            expected_parent=root/'basic-dp05-reuse-union-corrected-v2/freeze.json' if numbered else basic/'freeze.json'
            effective_ok=bool(pin_map) and not effective_bad and expected_changes is not None and actual_changes==sorted(effective.get('only_baseline_source_change',[]))==expected_changes and expected_parent.is_file() and f.get('baseline_freeze_sha256')==digest(expected_parent)
            check(str(phase.relative_to(root))+': explicit effective candidate source and separately pinned binary',effective_ok,{'effective_source_files':len(pin_map),'mismatches':effective_bad,'actual_changed_candidate_files':actual_changes,'binary_sha256':f['binary']['sha256'],'scope':'manifest.source pins baseline harness/fixtures/source guard. Explicit effective source map pins actual A2 one-file change or corrected DP05 three-file change; numbered diagnostic inherits corrected candidate with separately pinned formatter. No silent BASIC-binary substitution.'})
        harness=f.get('effective_harness_source');formatter=None
        if harness:
            harness_bad=[];snapshot=Path(harness['snapshot'])
            for relative,wanted in harness.get('sha256',{}).items():
                actual=snapshot/relative
                if not actual.is_file() or digest(actual)!=wanted:harness_bad.append(relative)
            diagnostic_baseline=root/'basic-dp05-reuse-union-corrected-v2';diagnostic_freeze=read(diagnostic_baseline/'freeze.json',{})
            check(str(phase.relative_to(root))+': explicitly pinned line-number-only diagnostic harness',not harness_bad and harness.get('candidate_binary_unchanged') is True and f['binary']['sha256']==diagnostic_freeze.get('binary',{}).get('sha256') and (diagnostic_baseline/'freeze.json').is_file() and f.get('baseline_freeze_sha256')==digest(diagnostic_baseline/'freeze.json') and auth.get('allowed_tactic')=='ORIGINAL_LINE_NUMBERS',{'mismatches':harness_bad,'effective_harness_files':len(harness.get('sha256',{})),'candidate_binary_unchanged':harness.get('candidate_binary_unchanged'),'scope':'Diagnostic adds original line notation and its common instruction on model-payload copies. Candidate/host/guards/oracles remain the corrected DP05 baseline version.'})
            if not harness_bad:
                formatter_path=snapshot/'compact_source_lines.py';spec=importlib.util.spec_from_file_location('audited_pure_frozen_line_formatter',formatter_path);formatter=importlib.util.module_from_spec(spec);spec.loader.exec_module(formatter)
        provenance=read(phase/'analysis-provenance.json',{});code_map=provenance.get('code_sha256',{});code_bad=[name for name,wanted in code_map.items() if not(repo/'benchmark/architecture/four_dp_poc_v2'/name).is_file() or digest(repo/'benchmark/architecture/four_dp_poc_v2'/name)!=wanted]
        check(str(phase.relative_to(root))+': explicit derived-analysis/scorer provenance',bool(code_map) and not code_bad and fh in provenance.get('input_freeze_sha256',[]),{'version':provenance.get('version'),'code_files':len(code_map),'mismatches':code_bad},pending=not a.final and not code_map)
        selected=[]
        for r in lines(phase/'runs.jsonl'):
            if not a.final and sample_left<=0:break
            selected.append(r)
            if not a.final:sample_left-=1
        all_records.extend(selected);allowed=set(key(x) for x in auth.get('allowed_jobs',[]));actual=[key(x) for x in selected]
        if a.final:
            stop=read(phase/'PHASE-STOPPED.json',{});stopped=bool(stop)
            if stopped:
                completed=set(key(x) for x in stop.get('completed_jobs',[]));unexecuted=set(key(x) for x in stop.get('unexecuted_jobs',[]))
                stop_ok=stop.get('version')=='EXPLICIT_PARENT_AUTHORIZED_BOUNDARY_STOP' and stop.get('completed_trials')==len(selected)==len(completed) and stop.get('planned_trials')==len(allowed) and stop.get('unexecuted_trials')==len(unexecuted) and completed.isdisjoint(unexecuted) and completed|unexecuted==allowed and set(actual)==completed and set(stop.get('completed_run_ids',[]))=={r['run_id']for r in selected} and stop.get('used_in_final_effective120') is False and stop.get('inflight_at_stop')==0 and stop.get('current_api_cancelled') is False and stop.get('preserved_freeze_sha256')==fh and stop.get('runs_sha256')==digest(phase/'runs.jsonl')
                check(str(phase.relative_to(root))+': explicit boundary stop preserves completed and unexecuted jobs',stop_ok and len(actual)==len(set(actual)),{'actual':len(actual),'planned':len(allowed),'unexecuted':len(unexecuted),'replacement_phase':stop.get('replacement_phase'),'included_in_effective120':False})
            else:check(str(phase.relative_to(root))+': every authorized job once',len(actual)==len(set(actual)) and set(actual)==allowed,{'actual':len(actual),'planned':len(allowed),'missing':[list(x) for x in sorted(allowed-set(actual))],'unexpected':[list(x) for x in sorted(set(actual)-allowed)]})
        metrics={x['run_id']:x for x in read(phase/'episode-metrics.json',[])};manifests={};oracles={};row_bad=[];time_bad=[];q2_bad=[];memory_bad=[];fail_bad=[];raw_bad=[];call_bad=[];diagnostic_bad=[]
        for r in selected:
            rid=r['run_id'];m=metrics.get(rid);events=r['events'];input_path=Path(r.get('input',''));trace_path=input_path.with_name('trace.json');scan(input_path);scan(trace_path)
            if not input_path.is_file() or not trace_path.is_file():raw_bad.append(rid)
            elif read(trace_path)!=r:raw_bad.append(rid+':raw_record_diff')
            if r.get('freeze_sha256')!=fh or r.get('candidate_binary_sha256')!=f['binary']['sha256'] or r.get('postrun_binary_sha256')!=f['binary']['sha256'] or key(r) not in allowed or r.get('phase')!=auth.get('phase') or r.get('tactic')!=auth.get('allowed_tactic') or r.get('offline_responder'):row_bad.append(rid)
            dp=r['dp']
            if dp not in manifests:
                base=phase/'frozen/source/benchmark/architecture/four_dp_poc_v2';manifests[dp]=keyed(read(base/f'goal_manifests/dp{dp}.json'));oracles[dp]=keyed(read(base/f'oracles/dp{dp}.json'))
            manifest=manifests[dp][r['episode']];oracle=oracles[dp].get(r['episode'],{});starts={};displays={};goals=[]
            for e in events:
                d=e.get('data',{});at=e.get('elapsed_ns')
                if at is None:continue
                sid=d.get('stimulus_id')
                if e['kind'] in ['INPUT_CAPTURED','STIMULUS'] and sid:starts.setdefault(str(sid),d.get('input_elapsed_ns',at))
                if e['kind']=='DISPLAY_PROXY' and sid and isinstance(d.get('text'),str) and d['text'].strip() and d.get('content_type') in ['ADOPTED_HANDLING','DETAILED_RESULT','TASK_QUESTION']:displays.setdefault(str(sid),e)
            observed1=[]
            for entry in manifest['qa01_stimuli']:
                sid=str(entry.get('stimulus_id',entry.get('id'))) if isinstance(entry,dict) else str(entry);start=starts.get(sid);end=displays.get(sid,{}).get('elapsed_ns');value=(end-start)/1e6 if end is not None and start is not None else None
                if value is not None:observed1.append(value)
                if m and not close(value,m.get('timing_details_by_stimulus',{}).get(sid,{}).get('qa01_ms')):time_bad.append(rid+':qa01:'+sid)
            offsets=[e['observed_ns']-e['elapsed_ns'] for e in events if 'observed_ns' in e and 'elapsed_ns' in e];origin=min(offsets) if offsets else r['start_ns'];observed3=[]
            for g in manifest['qa03_goals']:
                gid=g['goal_id'];start=starts.get(str(g['origin_stimulus_id']));term=[e for e in events if e['kind']=='GOAL_COMPLETE' and e['data'].get('goal_id')==gid and e['data'].get('content_contract_checked') is True and str(e['data'].get('text','')).strip() and (g.get('final_revision') is None or e['data'].get('revision')==g['final_revision'])];end=term[-1]['elapsed_ns'] if term else None;spans=[(c['send_ns']-origin,c['terminal_ns']-origin) for c in r['calls'] if c.get('role')=='RECEIVER' and gid in c.get('causal_trace',{}).get('goal_ids',[]) and c.get('send_ns') is not None and c.get('terminal_ns') is not None];elapsed=(end-start) if end is not None and start is not None else None;excluded=union(spans,start,end) if elapsed is not None else None;value=(elapsed-excluded)/1e6 if elapsed is not None else None
                if value is not None:observed3.append(value)
                if m:
                    actual=next((x for x in m.get('timing_details_by_goal',[]) if x['goal_id']==gid),{})
                    if not close(value,actual.get('qa03_ms')) or not close(elapsed/1e6 if elapsed is not None else None,actual.get('elapsed_ms')) or not close(excluded/1e6 if excluded is not None else None,actual.get('receiver_shim_internal_excluded_ms')):time_bad.append(rid+':qa03:'+gid)
                goals.append({'goal_id':gid,'qa03_ms':value,'missing':value is None})
            if m and (m['qa01_expected']!=len(manifest['qa01_stimuli']) or m['qa03_expected']!=len(manifest['qa03_goals']) or m['qa01_n']!=len(observed1) or m['qa03_n']!=len(observed3) or m['qa01_noresponse']!=len(manifest['qa01_stimuli'])-len(observed1) or m['qa03_incomplete']!=len(manifest['qa03_goals'])-len(observed3)):time_bad.append(rid+':coverage')
            if m and (not close(m.get('qa01_sum_ms'),sum(observed1)) or not close(m.get('qa03_sum_ms'),sum(observed3)) or not close(m.get('qa01_mean_ms'),sum(observed1)/len(observed1) if observed1 else None) or not close(m.get('qa03_mean_ms'),sum(observed3)/len(observed3) if observed3 else None)):time_bad.append(rid+':sum_mean')
            if a.final and m is None:time_bad.append(rid+':metrics_missing')
            actual_adoptions=[e['data'] for e in events if e['kind'] in ['ADOPTION','SEMANTIC_ADOPTION']]
            if dp=='04':
                sys.path.insert(0,str(repo/'benchmark/architecture/four_dp_poc_v2'));from score_dp04 import canonicalize_adoptions
                identity=canonicalize_adoptions(r,actual_adoptions);actual_adoptions=identity['adoptions']
            first={}
            for v in actual_adoptions:first.setdefault((str(v.get('expression_id')),str(v.get('revision'))),v)
            correct=0
            for wanted in oracle.get('adoptions',[]):
                got=first.get((str(wanted['expression_id']),str(wanted['revision'])));ok=got is not None and isinstance(got.get('targets'),list) and set(got['targets'])==set(wanted['targets']) and not got.get('_target_identity_unresolved')
                versions=(got or {}).get('source_revisions',{});versions={x['source_id']:x['revision'] for x in versions} if isinstance(versions,list) else versions
                if wanted.get('required_source_revisions'):ok=ok and all(versions.get(k)==v for k,v in wanted['required_source_revisions'].items())
                correct+=bool(ok)
            if m and (m['qa02_total']!=len(oracle.get('adoptions',[])) or m['qa02_correct']!=correct):q2_bad.append(rid)
            if m and dp=='02':
                rk=lambda i:tuple(i.get(k) for k in ['relation','target_request_id','question_id','task_id']);nrc=0;wanted_sets=oracle.get('intent_sets',[])
                for wanted in wanted_sets:
                    actual=[rk(e['data'].get('decision',{})) for e in events if e['kind']=='ADOPTION_COMMITTED' and e['data'].get('stimulus_id')==wanted['event_id']];nrc+=collections.Counter(actual)==collections.Counter(rk(i) for i in wanted['intents'])
                if m['nr_total']!=len(wanted_sets) or m['nr_correct']!=nrc:diagnostic_bad.append(rid+':exact_intent_multiset')
            completed=[e['data'] for e in events if e['kind']=='GOAL_COMPLETE']
            if m and completed and m.get('result',{}).get('outputs')!=completed:diagnostic_bad.append(rid+':actual_completion_output_recovery')
            if m and m.get('content_total')!=len(oracle.get('content_checks',[])):diagnostic_bad.append(rid+':fixed_content_opportunities')
            nh_regression=False
            if m and dp=='05':
                sys.path.insert(0,str(repo/'benchmark/architecture/four_dp_poc_v2'));from score_dp05 import score as handoff_score
                nh=handoff_score({'handoffs':[e['data'] for e in events if e['kind']=='HANDOFF_ADMISSION'],'outputs':completed},oracle)
                nh_regression=nh['functional_regression']
                if m['nh_total']!=len(oracle.get('handoffs',[])) or m['nh_correct']!=nh['correct'] or m.get('nh_functional_regression')!=nh['functional_regression']:diagnostic_bad.append(rid+':fixed_handoff_population_or_extras')
            failures=[e for e in events if e['kind'] in ['GOAL_FAILED','EPISODE_ERROR']]
            if failures:failure_records.append({'run_id':rid,'phase':r['phase'],'failure_events':len(failures),'qa01_missing':len(manifest['qa01_stimuli'])-len(observed1),'qa03_missing':len(manifest['qa03_goals'])-len(observed3),'qa02_wrong':len(oracle.get('adoptions',[]))-correct})
            expected_functional_error=bool(failures or r['returncode']!=0 or nh_regression)
            if m and bool(m.get('functional_error'))!=expected_functional_error:fail_bad.append(rid)
            samples=r.get('memory',[]);peak=max(samples,key=lambda x:x['concurrent_sum_bytes']) if samples else None;both=[x for x in samples if len(x.get('rss_bytes_by_pid',{}))>=2];gap=max(((b['at_ns']-a['at_ns'])/1e6 for a,b in zip(samples,samples[1:])),default=None)
            registry={str(x['pid']):x['role'] for x in r.get('process_registry',[])};python_pid=next((int(pid) for pid,role in registry.items() if role=='python_runner_bridge'),None)
            if len(registry)!=2 or set(registry.values())!={'python_runner_bridge','rust_candidate'}:memory_bad.append(rid+':registry')
            for sample in samples:
                if sample['concurrent_sum_bytes']!=sum(sample['rss_bytes_by_pid'].values()) or sample['concurrent_sum_bytes']!=sum(sample.get('rss_bytes_by_role',{}).values()) or sample['sample_end_ns']<sample['at_ns']:memory_bad.append(rid)
                for process in sample.get('processes',[]):
                    if registry.get(str(process['pid']))!=process['role'] or sample['rss_bytes_by_pid'].get(str(process['pid']))!=process['rss_bytes'] or (process['role']=='rust_candidate' and process['ppid']!=python_pid):memory_bad.append(rid+':actual_pid_role_parent')
            if m and (not close(m.get('rss_concurrent_peak_mb'),peak['concurrent_sum_bytes']/1e6 if peak else None) or (m.get('rss_peak_same_sample')!=peak) or (m.get('memory_state')=='SAMPLED_RSS_ESTIMATE')!=bool(both)):memory_bad.append(rid+':peak_or_coverage')
            memory_notes.append({'run_id':rid,'registered_roles':r.get('process_registry'),'samples':len(samples),'both_role_samples':len(both),'peak_bytes':peak['concurrent_sum_bytes'] if peak else None,'peak_all_registered_observed':(peak or {}).get('all_registered_observed'),'max_sample_gap_ms':gap})
            observations.append({'run_id':rid,'phase':r['phase'],'phase_directory':str(phase.relative_to(root)),'dp':dp,'variant':r['variant'],'tactic':r.get('tactic'),'qa01_expected':len(manifest['qa01_stimuli']),'qa01_observed':len(observed1),'qa01_sum_ms':sum(observed1),'qa03_expected':len(manifest['qa03_goals']),'qa03_observed':len(observed3),'qa03_sum_ms':sum(observed3),'qa02_correct':correct,'qa02_expected':len(oracle.get('adoptions',[])),'via_cost_usd':sum(c.get('cost_usd') or 0 for c in r['calls'] if c.get('role')!='RECEIVER'),'receiver_cost_usd':sum(c.get('cost_usd') or 0 for c in r['calls'] if c.get('role')=='RECEIVER'),'model_calls':len(r['calls']),'functional_error':expected_functional_error,'rss_peak_mb':peak['concurrent_sum_bytes']/1e6 if peak else None})
            for c in r['calls']:
                cid=c.get('call_uuid');call_occurrences[cid]+=1
                if not cid:call_bad.append(rid+':missing_uuid');continue
                req=Path(c.get('request_file',''));resp=Path(c.get('response_file',''));scan(req);scan(resp);body=read(req,{})
                if not req.is_file() or body.get('model')!=MODEL or body.get('enable_thinking') is not False or body.get('temperature')!=0 or c.get('model')!=MODEL or c.get('thinking') is not False:call_bad.append(cid+':model_settings_or_request')
                if c.get('stage') in Q04_CAPS and body.get('max_tokens')!=Q04_CAPS[c['stage']]:call_bad.append(cid+':dp04_output_cap')
                if harness:
                    representation=c.get('input_representation_diagnostic',{});before=Path(representation.get('original_payload_file',''));scan(before)
                    if formatter is None or not before.is_file() or digest(before)!=representation.get('original_payload_sha256'):call_bad.append(cid+':original_line_payload_provenance')
                    else:
                        original=read(before);formatted,changes=formatter.transform(original['payload']);original_instruction=formatted.get('instruction','');expected_instruction=original_instruction+' '+formatter.EXPLANATION+' Return valid JSON only.'
                        messages=body.get('messages',[]);actual_user=messages[1].get('content') if len(messages)>1 else None
                        try:actual_user_value=json.loads(actual_user) if isinstance(actual_user,str) else None
                        except (ValueError,TypeError):actual_user_value=None
                        ok=representation.get('formatter_sha256')==digest(formatter_path) and representation.get('version')==formatter.VERSION and representation.get('changed_original_passages')==changes and actual_user_value==formatted.get('input',{}) and messages[0].get('content')==expected_instruction and original_instruction.count(formatter.EXPLANATION)==0 and representation.get('instruction_appended_exactly_once') is True and all(c.get(k)==v for k,v in original.get('meta',{}).items())
                        if not ok:call_bad.append(cid+':actual_numbered_copy_vs_original_request')
                usage=(read(resp,{}) or {}).get('usage',{});cost=c.get('cost_usd')
                if cost is not None:
                    if usage.get('prompt_tokens')!=c.get('input_tokens') or usage.get('completion_tokens')!=c.get('output_tokens'):call_bad.append(cid+':usage')
                    if c.get('input_tokens') is None or c.get('output_tokens') is None:call_bad.append(cid+':known_cost_missing_usage')
                    else:
                        computed=(Decimal(c['input_tokens'])*Decimal('.40')+Decimal(c['output_tokens'])*Decimal('1.60'))/Decimal(1000000)
                        if abs(Decimal(str(cost))-computed)>Decimal('0.000000001'):call_bad.append(cid+':price')
                global_calls[cid]=c
        for label,bad in [('actual raw input/trace',raw_bad),('pertrial pin/job identity',row_bad),('independent callback/finalrevision/receiver timing and fixed coverage',time_bad),('first exact referent sets/current revision',q2_bad),('failure events preserved',fail_bad),('same-sample RSS sums/peak/coverage',memory_bad),('exact N-R/fixed N-H/content event recovery diagnostics',diagnostic_bad),('actual requests/model/usage/price/DP04caps',call_bad)]:check(str(phase.relative_to(root))+': '+label,not bad,{'records_checked':len(selected),'mismatches':bad},pending=not selected)
        phase_notes.append({'phase_directory':str(phase.relative_to(root)),'authorized_jobs':len(allowed),'checked_records':len(selected)})
        if a.final:
            log=list(lines(phase/'model-calls.jsonl'));logged={c['call_uuid']:c for c in log};owned={c['call_uuid']:c for r in selected for c in r['calls']};check(str(phase.relative_to(root))+': actual call log equals episode ownership',len(log)==len(logged) and set(logged)==set(owned),{'call_log_rows':len(log),'owned':len(owned),'unowned_uuid_count':len(set(logged)-set(owned)),'missing_log_count':len(set(owned)-set(logged))})
            summary=read(phase/'summary.json',{});aggregation_bad=[]
            observed_groups={(x['phase'],x['dp'],x['variant'],x['tactic']) for x in observations if x['phase_directory']==str(phase.relative_to(root))};summary_groups=[(s['phase'],s['dp'],s['variant'],s.get('tactic')) for s in summary.get('rows',[])]
            if set(summary_groups)!=observed_groups or len(summary_groups)!=len(set(summary_groups)):aggregation_bad.append({'reason':'MISSING_EXTRA_OR_DUPLICATE_SUMMARY_GROUP','expected_groups':[list(x) for x in sorted(observed_groups)],'actual_groups':[list(x) for x in summary_groups]})
            for s in summary.get('rows',[]):
                group=[x for x in observations if x['phase_directory']==str(phase.relative_to(root)) and (x['phase'],x['dp'],x['variant'],x['tactic'])==(s['phase'],s['dp'],s['variant'],s.get('tactic'))];n1=sum(x['qa01_observed'] for x in group);n3=sum(x['qa03_observed'] for x in group);s1=sum(x['qa01_sum_ms'] for x in group);s3=sum(x['qa03_sum_ms'] for x in group)
                if s['phase'] in ('TACTIC','DIAGNOSTIC') and (s.get('qa04_modules_mean') is not None or s.get('qa04_tasks')):aggregation_bad.append({'reason':'BASELINE_QA04_WRONGLY_ATTRIBUTED_TO_TACTIC','phase':s['phase'],'dp':s['dp'],'variant':s['variant']})
                ok=len(group)==s['episodes'] and n1==s['qa01_n'] and n3==s['qa03_n'] and sum(x['qa01_expected'] for x in group)==s['qa01_expected'] and sum(x['qa03_expected'] for x in group)==s['qa03_expected'] and sum(x['qa02_correct'] for x in group)==s['qa02_correct'] and sum(x['qa02_expected'] for x in group)==s['qa02_total'] and close(s1/n1 if n1 else None,s['qa01_ms']) and close(s3/n3 if n3 else None,s['qa03_ms']) and close(sum(x['via_cost_usd'] for x in group),s['qa05_via_usd']) and close(sum(x['receiver_cost_usd'] for x in group),s['qa05_receiver_usd']) and sum(x['model_calls'] for x in group)==s['model_calls'] and sum(x['functional_error'] for x in group)==s['errors'] and close(max((x['rss_peak_mb'] for x in group if x['rss_peak_mb'] is not None),default=None),s['qa06_peak_rss_mb'])
                if not ok:aggregation_bad.append({'phase':s['phase'],'dp':s['dp'],'variant':s['variant']})
            check(str(phase.relative_to(root))+': summary weighted time/set totals/cost/peak/failure aggregation',bool(summary.get('rows')) and not aggregation_bad,{'groups':len(summary.get('rows',[])),'mismatches':aggregation_bad})
    basics=[r for r in all_records if r['phase']=='BASIC'];calibrations=[r for r in all_records if r['phase'].startswith('CALIBRATION')]
    if a.final:
        actual=[key(r) for r in basics];check('Final BASIC120 keys/no episode loss',len(actual)==len(set(actual))==120 and set(actual)==set(jobs),{'actual':len(actual),'unique':len(set(actual)),'missing':[list(x) for x in sorted(set(jobs)-set(actual))]});check('All26 calibration episodes separately preserved',len(calibrations)==26,{'calibration_rows':len(calibrations),'included_in_basic':False})
        scope=collections.Counter(r['phase'] for r in all_records)
        if a.baseline_audit:
            check('Interim146 original records: BASIC120 + calibration26',len(all_records)==146 and scope.get('BASIC')==120 and len(calibrations)==26,{'actual_records':len(all_records),'phases':dict(scope),'scope':'Original DP0536 is intermediate evidence; this does not claim corrected DP05 or final all-phase completion.'})
        else:
            stopped_phases=[phase for phase in phases if (phase/'PHASE-STOPPED.json').is_file()];stopped_records=[r for phase in stopped_phases for r in lines(phase/'runs.jsonl')];stopped_count=sum(read(phase/'PHASE-STOPPED.json')['completed_trials']for phase in stopped_phases);final_corrected_root=root/'basic-dp05-reuse-union-corrected-v2'
            check('Only the explicitly authorized intermediate correction stop is counted',[(str(phase.relative_to(root)),read(phase/'PHASE-STOPPED.json').get('completed_trials'))for phase in stopped_phases]==[('basic-dp05-reuse-corrected-v1',11)],{'stopped_phase_counts':[(str(phase.relative_to(root)),read(phase/'PHASE-STOPPED.json').get('completed_trials'))for phase in stopped_phases],'scope':'Parent authorized one intermediate11, final36 whole-generation replacement and numbered9; no arbitrary inferred phase substitution.'})
            check('Final all-phase records: fixed218 plus explicitly stopped historical trial count',len(all_records)==218+stopped_count and scope.get('BASIC')==120 and scope.get('BASIC_CORRECTED')==36+stopped_count and scope.get('TACTIC')==15 and scope.get('DIAGNOSTIC')==21 and len(calibrations)==26 and len(stopped_records)==stopped_count,{'actual_records':len(all_records),'expected_records':218+stopped_count,'stopped_history_records':stopped_count,'phases':dict(scope),'planned_post_basic':'A4 TACTIC12 + A2 TACTIC3 + B4 DIAGNOSTIC12 + final DP05 BASIC_CORRECTED36 + numbered-lines DIAGNOSTIC9; calibration26 and explicitly stopped intermediate records separate'})
            corrected=[r for r in all_records if r['phase']=='BASIC_CORRECTED' and Path(r['input']).parents[3].resolve()==final_corrected_root.resolve()];effective=[r for r in basics if r['dp']!='05']+corrected;effective_keys=[key(r) for r in effective]
            effective_ids={r['run_id'] for r in effective};eo=[x for x in observations if x['run_id'] in effective_ids]
            check('Presentation effective120: original01/02/04 84 + corrected05 36, fixed261/243',len(effective)==len(set(effective_keys))==120 and set(effective_keys)==set(jobs) and len(corrected)==36 and all(r['dp']=='05' for r in corrected) and sum(x['qa01_expected'] for x in eo)==261 and sum(x['qa03_expected'] for x in eo)==243,{'effective_records':len(effective),'original_non05_records':len(effective)-len(corrected),'corrected05_records':len(corrected),'qa01_expected':sum(x['qa01_expected'] for x in eo),'qa01_observed':sum(x['qa01_observed'] for x in eo),'qa03_expected':sum(x['qa03_expected'] for x in eo),'qa03_observed':sum(x['qa03_observed'] for x in eo),'scope':'Original BASIC120 remains immutable diagnostic history. This explicit generation selection does not overwrite any original raw run or mix DP05 generations.'})
            selection=read(root/'effective-main-selection.json',{});selected=selection.get('selection',[]);by_id={r['run_id']:r for r in effective};selection_bad=[]
            for item in selected:
                raw=by_id.get(item['run_id']);trace=Path(item['trace']);inp=Path(item['input'])
                if raw is None or not trace.is_file() or not inp.is_file() or digest(trace)!=item['trace_sha256'] or digest(inp)!=item['input_sha256']:selection_bad.append(item['run_id']);continue
                if key(raw)!=(item['dp'],item['episode'],item['variant'],item['repetition']) or raw['phase']!=item['actual_phase'] or raw['freeze_sha256']!=item['freeze_sha256'] or raw['candidate_binary_sha256']!=item['candidate_binary_sha256'] or trace.parents[3].resolve()!=Path(item['source_phase_root']).resolve():selection_bad.append(item['run_id']+':generation_identity')
            check('Publication selection manifest is the exact effective120 with source hashes',selection.get('version')=='EXPLICIT_EFFECTIVE_MAIN_120_AFTER_DP05_REUSE_CORRECTION' and selection.get('selected_trials')==len(selected)==120 and len({x['run_id']for x in selected})==120 and {x['run_id']for x in selected}==set(by_id) and selection.get('expected_qa01')==261 and selection.get('expected_qa03')==243 and not selection_bad,{'mismatches':selection_bad,'manifest':str(root/'effective-main-selection.json'),'raw_generation_preserved':True})
            publication=read(root/'effective-main-comparison/summary.json',{});publication_bad=[];pub_rows=publication.get('rows',[])
            for s in pub_rows:
                group=[x for x in eo if (x['dp'],x['variant'])==(s['dp'],s['variant'])];n1=sum(x['qa01_observed']for x in group);n3=sum(x['qa03_observed']for x in group)
                ok=len(group)==s['episodes']==12 and sum(x['qa01_expected']for x in group)==s['qa01_expected'] and sum(x['qa03_expected']for x in group)==s['qa03_expected'] and n1==s['qa01_n'] and n3==s['qa03_n'] and sum(x['qa02_correct']for x in group)==s['qa02_correct'] and sum(x['qa02_expected']for x in group)==s['qa02_total'] and close(sum(x['qa01_sum_ms']for x in group)/n1 if n1 else None,s['qa01_ms']) and close(sum(x['qa03_sum_ms']for x in group)/n3 if n3 else None,s['qa03_ms']) and close(sum(x['via_cost_usd']for x in group),s['qa05_via_usd']) and close(sum(x['receiver_cost_usd']for x in group),s['qa05_receiver_usd']) and close(max((x['rss_peak_mb']for x in group if x['rss_peak_mb']is not None),default=None),s['qa06_peak_rss_mb'])
                if not ok:publication_bad.append((s['dp'],s['variant']))
            check('Publication headline10 rows use selected120 observed coverage/time/sets/cost/same-peak',publication.get('derived_not_measured_phase') is True and len(pub_rows)==len({(s['dp'],s['variant'])for s in pub_rows})==10 and not publication_bad,{'mismatches':publication_bad,'rows':len(pub_rows),'scope':'Derived selection, never a new measured phase; excludes original DP05 and all calibration/tactic/diagnostic costs from effective120 rows.'})
            final_publication=read(root/'final-measurement-summary.json',{});final_bad=[];stage_bad=[];peak_bad=[];pub_options=final_publication.get('options',[]);raw_by_id={r['run_id']:r for r in effective}
            for option in pub_options:
                pair=(option['dp'],option['variant']);summary_row=next((s for s in pub_rows if (s['dp'],s['variant'])==pair),None);actual_runs=[r for r in effective if (r['dp'],r['variant'])==pair]
                if summary_row is None:final_bad.append([*pair,'unknown_option']);continue
                q1=option['qa01'];q2=option['qa02'];q3=option['qa03'];q5=option['qa05']
                if not(option['episode_trials']==summary_row['episodes']==12 and q1['planned']==summary_row['qa01_expected'] and q1['observed']==summary_row['qa01_n'] and q1['missing']==summary_row['qa01_noresponse'] and close(q1['conditional_mean_ms'],summary_row['qa01_ms']) and q2['correct']==summary_row['qa02_correct'] and q2['planned']==summary_row['qa02_total'] and q3['planned']==summary_row['qa03_expected'] and q3['observed']==summary_row['qa03_n'] and q3['missing']==summary_row['qa03_incomplete'] and close(q3['conditional_via_mean_ms'],summary_row['qa03_ms']) and close(q3['conditional_full_wall_mean_ms'],summary_row['qa03_full_elapsed_ms']) and close(q3['conditional_receiver_union_excluded_mean_ms'],summary_row['receiver_shim_excluded_ms']) and option['failed_contract_or_completion_episodes']==summary_row['errors']):final_bad.append([*pair,'endpoints_or_sets'])
                actual_stages={}
                for raw in actual_runs:
                    for call in raw['calls']:
                        stage=actual_stages.setdefault((call['role'],call['stage']),{'calls':0,'input_tokens':0,'output_tokens':0,'cost':0.0,'unknown':0});stage['calls']+=1;stage['input_tokens']+=call.get('input_tokens')or 0;stage['output_tokens']+=call.get('output_tokens')or 0;stage['cost']+=call.get('cost_usd')or 0;stage['unknown']+=call.get('cost_usd')is None
                published_stages={(s['role'],s['stage']):s for s in q5['stages']}
                if set(actual_stages)!=set(published_stages) or len(published_stages)!=len(q5['stages']):stage_bad.append([*pair,'stage_keys'])
                for stage,actual in actual_stages.items():
                    stated=published_stages.get(stage,{})
                    if not(actual['calls']==stated.get('calls') and actual['input_tokens']==stated.get('input_tokens') and actual['output_tokens']==stated.get('output_tokens') and actual['unknown']==stated.get('unknown_usage_calls') and close(actual['cost'],stated.get('known_usage_usd'))):stage_bad.append([*pair,*stage])
                if not(q5['actual_calls']==sum(s['calls']for s in actual_stages.values()) and q5['input_tokens']==sum(s['input_tokens']for s in actual_stages.values()) and q5['output_tokens']==sum(s['output_tokens']for s in actual_stages.values()) and close(q5['total_usd'],sum(s['cost']for s in actual_stages.values()))):stage_bad.append([*pair,'option_totals'])
                peak_meta=option['qa06_same_peak'];peak_run=raw_by_id.get(peak_meta['peak_run_id']);peak_sample=next((sample for sample in peak_run['memory']if sample['at_ns']==peak_meta['peak_sample_start_ns']),None)if peak_run else None
                if peak_sample is None or not(close(peak_meta['concurrent_peak_mb'],peak_sample['concurrent_sum_bytes']/1e6) and close(peak_meta['python_at_same_peak_mb'],peak_sample['rss_bytes_by_role'].get('python_runner_bridge',0)/1e6) and close(peak_meta['rust_at_same_peak_mb'],peak_sample['rss_bytes_by_role'].get('rust_candidate',0)/1e6) and close(peak_meta['concurrent_peak_mb'],max(sample['concurrent_sum_bytes']for raw in actual_runs for sample in raw['memory'])/1e6)):peak_bad.append(list(pair))
            final_inputs=final_publication.get('input_sha256',{});pin_bad=[path for path,wanted in final_inputs.items()if not Path(path).is_file()or digest(Path(path))!=wanted]
            check('Final publication120 endpoints and raw stage call/token/cost accounting',len(pub_options)==len({(o['dp'],o['variant'])for o in pub_options})==10 and final_publication.get('selected_episode_trials')==120 and final_publication.get('actual_all_episode_trials')==229 and (final_publication.get('qa01_expected'),final_publication.get('qa03_expected'))==(261,243) and not final_bad and not stage_bad and bool(final_inputs)and not pin_bad,{'endpoint_mismatches':final_bad,'stage_mismatches':stage_bad,'input_pin_mismatches':pin_bad})
            check('Final publication role RSS uses the actual same run and sample peak',len(pub_options)==10 and not peak_bad,{'mismatches':peak_bad,'scope':'Python/Rust contributions at one actual sampled total peak, not independent role maxima; no browser/cloud/product memory claim.'})
        b=[o for o in observations if o['phase']=='BASIC'];check('Final fixed261/243 coverage including missing',sum(x['qa01_expected'] for x in b)==261 and sum(x['qa03_expected'] for x in b)==243,{'qa01_expected':sum(x['qa01_expected'] for x in b),'qa01_observed':sum(x['qa01_observed'] for x in b),'qa03_expected':sum(x['qa03_expected'] for x in b),'qa03_observed':sum(x['qa03_observed'] for x in b)})
        ledger=read(root/'budget.json',{});entries=ledger.get('calls',{});unknown=[cid for cid,c in global_calls.items() if c.get('cost_usd') is None];known=sum((Decimal(str(c['cost_usd'])) for c in global_calls.values() if c.get('cost_usd') is not None),Decimal(0));ledger_total=sum((Decimal(str(v['amount_usd'])) for v in entries.values()),Decimal(0));ledger_bad=[cid for cid,c in global_calls.items() if cid in entries and c.get('cost_usd') is not None and (entries[cid].get('state')!='USAGE_RECORDED' or abs(Decimal(str(entries[cid]['amount_usd']))-Decimal(str(c['cost_usd'])))>Decimal('0.000000001'))]
        owned_keys=set(global_calls);ledger_keys=set(entries);ownership_ok=owned_keys<=ledger_keys if a.baseline_audit else owned_keys==ledger_keys
        check('Every scoped actual call UUID ↔ one global ledger entry',ownership_ok and not ledger_bad and all(n==1 for n in call_occurrences.values()),{'actual_unique_calls':len(global_calls),'ledger_entries':len(entries),'duplicate_episode_owned_calls':sum(n-1 for n in call_occurrences.values() if n>1),'ledger_only_count':len(ledger_keys-owned_keys),'call_only_count':len(owned_keys-ledger_keys),'amount_state_mismatches':ledger_bad,'scope':'Baseline-only interim allows newer phase ledger entries but still validates every original call; final audit requires exact global equality.'})
        check('Global $4 cap / known vs unknown reservations',ledger.get('limit_usd')==4 and ledger_total<=Decimal('4') and (a.baseline_audit or bool(unknown) or abs(known-ledger_total)<=Decimal('0.000000001')),{'known_standard_price_usd':str(known),'known_cost_scope':'original BASIC+calibration only' if a.baseline_audit else 'all actual phases','unknown_usage_calls':len(unknown),'ledger_reserved_or_recorded_total_usd':str(ledger_total),'unsettled_states':dict(collections.Counter(v['state'] for v in entries.values() if v['state']!='USAGE_RECORDED'))},limited=bool(unknown))
        if not a.baseline_audit:
            stated_all=final_publication.get('actual_campaign',{});stated_effective=final_publication.get('effective120_cost',{});effective_calls=[c for r in effective for c in r['calls']];effective_via=sum(c.get('cost_usd')or 0 for c in effective_calls if c['role']!='RECEIVER');effective_receiver=sum(c.get('cost_usd')or 0 for c in effective_calls if c['role']=='RECEIVER')
            check('Final publication separates selected120 cost from all229 actual calls and ledger cost',stated_all.get('recorded_actual_episode_trials')==229 and stated_all.get('recorded_actual_calls')==len(global_calls) and close(stated_all.get('usage_confirmed_usd'),float(known)) and stated_all.get('authorized_unexecuted_jobs_stopped_before_start')==25 and stated_effective.get('calls')==len(effective_calls) and close(stated_effective.get('via_usd'),effective_via) and close(stated_effective.get('receiver_usd'),effective_receiver) and close(stated_effective.get('total_usd'),effective_via+effective_receiver),{'selected120_calls':len(effective_calls),'selected120_via_usd':effective_via,'selected120_receiver_usd':effective_receiver,'all229_calls':len(global_calls),'all229_known_usd':str(known),'stopped_unexecuted_jobs':25})
        if not a.baseline_audit:
            scan(root/'demo.html')
        check('No persisted secret/header/environment patterns (presence only)',not secret_presence,{'files_scanned':len(scanned),'standalone_export_scanned':str(root/'demo.html') in {str(p)for p in scanned},'presence':secret_presence,'scope':'Pattern presence scan, not an exhaustive security certification; no matched value is emitted. Final export is included if present.'})
        old=read(out/'old-immutability-baseline.json',{});changed=[path for path,wanted in old.get('files',{}).items() if not (repo/path).is_file() or digest(repo/path)!=wanted];check('Old raw/source invariance from captured baseline',bool(old.get('files')) and not changed,{'files':len(old.get('files',{})),'changed_or_missing_paths':changed,'scope':old.get('scope')})
    else:check('Final all-call/raw/callback/completeness audit deferred',False,{'sample_records_checked':len(all_records),'final_command':'python3 '+str(HERE/'audit_campaign.py')+' '+str(root)+' --final'},pending=True)
    applicability=read(root/'qa04/dp04-rebased-calibration-v3/basic-v1-applicability.json',{});qa04=read(basic/'qa04/summary.json',{});four=[x for x in qa04.get('options',[]) if x['dp']=='04'];check('QA04 DP04 final baseline applicability and four-task mean',applicability.get('candidate_source_byte_identical') is True and applicability.get('basic_freeze_sha256')==digest(basic/'freeze.json') and len(four)==2 and all(x['mean_changed_operational_modules']==2.5 and x['functional_tasks_mean']==3.5 and [t['count'] for t in x['tasks']]==[1,2,2,5] for x in four),{'applicability':applicability,'main_means':[x['mean_changed_operational_modules'] for x in four],'functional_pair_means':[x['functional_tasks_mean'] for x in four]})
    historical=read(root/'calibration-v1/CURRENT-REANALYSIS-20261018.json',{})
    if historical:check('Historical calibration-v1 analysis execution version remains UNKNOWN',historical.get('historical_analysis_execution_provenance')=='UNKNOWN' and historical.get('kind')=='EXPLICIT_CURRENT_REANALYSIS_NOT_HISTORICAL_PROVENANCE_RECONSTRUCTION',{'scope':'Current explicitly pinned reanalysis is audited; missing historical execution provenance is not reconstructed or inferred.','history':historical.get('historical_derived_archive')},limited=True)
    counts=collections.Counter(x['status'] for x in checks);result={'mode':'ORIGINAL_BASIC120_INTERIM' if a.baseline_audit else 'FINAL' if a.final else 'BOUNDED_PREPARATION','audit_script_sha256':sha(Path(__file__)),'checks':checks,'counts':dict(counts),'phases':phase_notes,'records_checked':len(all_records),'failure_records':failure_records,'memory_scope':{'records':memory_notes,'claim':'Actual same-sample process RSS estimates; no physical shared-page dedup, browser or cloud weights; logical buffers are separate harness-inclusive serialization diagnostics.'},'limitations':['Reuses recorded-provenance-verified source-backed DP04 alias and N-H field-validation helpers; first-set choice, exact-set comparison, callback timing, fixed populations and aggregation are independently reconstructed.','No independent general semantic judge; author manual/literal/equivalence diagnostics remain separate.','No OS process-birth timestamp is stored; registry/actual ps PID roles and same-run parent evidence support fresh-process scope, not impossible global PID reuse guarantees.','Old invariance applies only from the captured inventory to this audit.'],'provider_calls_created':0,'candidate_executions_created':0}
    filename=out/('basic120-checks.json' if a.baseline_audit else 'final229-review.json' if a.final else 'preparation-checks.json');filename.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');md=(out/'final229-review.md') if a.final and not a.baseline_audit else root/'analysis'/('basic120-review.md' if a.baseline_audit else 'final-review-preparation.md');md.write_text('# '+result['mode']+' independent campaign audit\n\n'+json.dumps(dict(counts))+'\n\n'+'\n'.join('- '+x['status']+' — '+x['name'] for x in checks)+'\n\nRaw check details: '+str(filename)+'\n\nNo model calls, candidate execution or source changes.\n');print(json.dumps({'mode':result['mode'],'counts':dict(counts),'output':str(filename),'records_checked':len(all_records),'provider_calls_created':0}));return 1 if counts['FAIL'] else 0
if __name__=='__main__':raise SystemExit(main())
