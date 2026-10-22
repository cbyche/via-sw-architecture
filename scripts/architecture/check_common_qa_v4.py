#!/usr/bin/env python3
"""Independently recompute the published analytical ledger; no product runs."""
import csv
import hashlib
import json
import math
from pathlib import Path
from statistics import mean

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/current/evaluation/common-qa-v4'


def eq(a, b):
    assert math.isclose(a, b, abs_tol=1e-7, rel_tol=1e-9), (a, b)


def main():
    spec = json.loads((OUT/'specification.json').read_text())
    oracle = json.loads((OUT/'oracles.json').read_text())
    d = json.loads((OUT/'artifacts/ledger.json').read_text())
    p = spec['profile']
    assert d['evidence_kind'] == 'ANALYTICAL_SIMULATION_NOT_MEASURED'
    for name in ['specification', 'oracles']:
        assert d[name+'_sha256'] == hashlib.sha256((OUT/(name+'.json')).read_bytes()).hexdigest()
    assert d['generator_sha256']==hashlib.sha256((ROOT/'scripts/architecture/build_common_qa_v4.py').read_bytes()).hexdigest()
    assert len(spec['configs']) == 7 and len(spec['scenarios']) == 24
    assert len(d['cases']) == 168 and len(d['changes']) == 42
    assert sum(len(o['referents']) for o in oracle['items']) == 33
    for c in spec['configs']:
        changed = [k for k in ['dp2','dp3','dp5','dp44'] if c[k] != 'A']
        assert len(changed) == (0 if c['id']=='BASE' else 1)
        assert changed == ([] if c['id']=='BASE' else [c['changed_dp'].lower()])
    node_count = usage_count = memory_count = 0
    for c in d['cases']:
        ns = {n['id']: n for n in c['nodes']}
        assert len(ns) == len(c['nodes'])
        assert c['qa02'] is None and c['qa07'] is None
        assert c['accuracy_status'] == 'NO_CANDIDATE_OUTPUT'
        for n in ns.values():
            node_count += 1
            eq(n['finish_s']-n['start_s'], n['duration_s'])
            assert n['duration_s'] >= 0
            earliest = max([n['release_s'], *[ns[k]['finish_s'] for k in n['deps']]])
            assert n['start_s'] >= earliest - 1e-9
            eq(n['resource_wait_s'], n['start_s']-earliest)
            expected = set(n['deps'])
            if n['resource_wait_s'] > 1e-10:
                expected.add(n['resource_predecessor'])
            assert set(n['causal_parents']) == expected
            if n['resource']=='semantic':
                eq(n['duration_s'], p['model_setup_s']+n['tokens_in']/p['input_tokens_per_s']+n['tokens_out']/p['output_tokens_per_s'])
            if n['resource']=='speech':
                setup=p['speech_setup_s']+n['tokens_in']/p['input_tokens_per_s']
                eq(n['duration_s'],setup+n['audio_s']/p['audio_generation_rate'])
                a=next(a for a in c['audio'] if a['node']==n['id'])
                eq(a['setup_s'],setup);eq(a['length_s'],n['audio_s'])
        for resource in ['semantic','speech','visual']:
            pool=[(-30,None)]*p[resource+'_slots']
            jobs=sorted([n for n in ns.values() if n['resource']==resource],key=lambda n:(n['start_s'],list(ns).index(n['id'])))
            for n in jobs:
                assert 0<=n['resource_slot']<len(pool)
                earliest=max([n['release_s'],*[ns[k]['finish_s'] for k in n['deps']]])
                expected=min((max(earliest,finish),slot,pred) for slot,(finish,pred) in enumerate(pool))
                eq(n['start_s'],expected[0]);assert n['resource_slot']==expected[1]
                assert n['resource_predecessor']==expected[2]
                pool[expected[1]]=(n['finish_s'],n['id'])
            for slot in range(p[resource+'_slots']):
                jobs=sorted([n for n in ns.values() if n['resource']==resource and n['resource_slot']==slot],key=lambda n:n['start_s'])
                assert all(a['finish_s']<=b['start_s']+1e-9 for a,b in zip(jobs,jobs[1:]))
        eq(c['qa01'],ns['first_display']['finish_s']-ns['input_end']['finish_s'])
        eq(c['first_voice'],ns['first_audio_output']['finish_s'])
        if 'first_chunk' in ns:
            chunk = ns['first_chunk']; job = ns[chunk['observes_prefix_of']]
            eq(chunk['start_s'],job['start_s'])
            eq(chunk['duration_s'],p['speech_setup_s']+job['tokens_in']/p['input_tokens_per_s']+min(job['audio_s'],p['first_chunk_audio_s'])/p['audio_generation_rate'])
            assert chunk['resource'] is None and chunk['tokens_in']==chunk['audio_s']==0
        eq(ns['first_audio_output']['duration_s'],p['device_s'])
        # Recursive independent projection with explicit cycle detection.
        projected = {}; visiting=set(); origin=c['goal_origin_s']
        def walk(key):
            if key in projected:return projected[key]
            assert key not in visiting, (c['case_id'],key)
            visiting.add(key);n=ns[key]
            parents=max([0,*[walk(k) for k in n['causal_parents']]])
            duration=max(0,n['finish_s']-max(origin,n['start_s'])) if n['kind']=='via' else 0
            result=0 if n['finish_s']<=origin else parents+duration
            visiting.remove(key);projected[key]=result;return result
        eq(c['qa03'],walk(c['terminal_node']))
        # Derive the complete required usage set from nodes and specification;
        # missing/duplicated buckets and altered rates must fail independently.
        expected_usage=[]
        for n in ns.values():
            for bucket,field,rate in [('text_input','tokens_in',p['input_usd_per_token']),('text_output','tokens_out',p['output_usd_per_token']),('audio_output','audio_s',p['audio_output_usd_per_s'])]:
                if n[field]:expected_usage.append((n['id'],n['phase'],bucket,n[field],rate))
        raw=next(s for s in spec['scenarios'] if s['id']==c['case_id'])
        scenario={**spec['scenario_defaults'],**raw}
        if c['modality']=='voice':expected_usage.append(('input_end','input','audio_input',scenario['utterance_s'],p['audio_input_usd_per_s']))
        observed_usage=[(u['node'],u['phase'],u['bucket'],u['amount'],u['rate_usd']) for u in c['usage']]
        assert sorted(observed_usage)==sorted(expected_usage)
        for u in c['usage']:
            usage_count += 1
            eq(u['usd'],u['amount']*u['rate_usd'])
            if u['bucket']=='audio_input':
                assert c['modality']=='voice'
            else:
                key={'text_input':'tokens_in','text_output':'tokens_out','audio_output':'audio_s'}[u['bucket']]
                eq(u['amount'],ns[u['node']][key])
        eq(c['qa05'],sum(u['usd'] for u in c['usage']))
        # Reconstruct the complete piecewise-linear memory envelope, not just
        # the selected peak row. Unique owners prevent invisible double count.
        intervals=c['memory_intervals'];memory_count+=len(intervals)
        assert len({i['owner'] for i in intervals}) == len(intervals)
        owned={i['owner']:i for i in intervals}
        required={'request'}
        cfg=next(cfg for cfg in d['configs'] if cfg['id']==c['config_id'])
        if cfg['dp2']=='B' and (scenario['long_work'] or scenario['overlap'] in ['spare','dependent','saturated']):required.add('request_execution')
        if cfg['dp5']=='B' and scenario['history'] in ['ready','dirty']:required.add('relation_cache')
        if scenario['screen'] in ['cold','tracked','warm']:required.add('cloud_image_buffer')
        for n in ns.values():
            if n['resource']=='semantic':
                owner=n['id']+':client';required.add(owner)
                eq(owned[owner]['mb'],p['call_base_mb']+n['tokens_in']/1000*p['call_mb_per_1000_tokens'])
                eq(owned[owner]['start_s'],n['start_s']);eq(owned[owner]['end_s'],n['finish_s'])
            expected_state={}
            if n['resource']=='visual' and n['id'].endswith('_load'):
                stage=n['id'].split('_')[-2];expected_state[stage+'_load_temp']=p[stage+'_load_temp_mb']
            elif n['resource']=='visual' and n['id'].endswith('_infer'):
                stage=n['id'].split('_')[-2];expected_state[stage+'_workspace']=p[stage+'_workspace_mb']
            elif n['id'] in ['relation_prior','relation_unused','relation_update']:
                expected_state['relation_producer']=p['relation_producer_mb']
            assert n['state']==expected_state,(c['config_id'],c['case_id'],n['id'])
            for label in expected_state:
                owner=n['id']+':'+label;required.add(owner)
                eq(owned[owner]['mb'],p[label+'_mb'])
                eq(owned[owner]['start_s'],n['start_s']);eq(owned[owner]['end_s'],n['finish_s'])
            if n['resource']=='visual' and n['id'].endswith('_boot'):
                required.add('runtime_shared_pages');required.add('vision_ipc')
                stage=n['id'].split('_')[-2]
                required.add('runtime_private:'+ (stage if cfg['dp3']=='B-R3' else 'shared'))
            if n['resource']=='visual' and n['id'].endswith('_load'):
                required.add('weight:'+n['id'][:-4]+'infer')
        assert set(owned)==required,(c['config_id'],c['case_id'],set(owned)^required)
        for owner,row in owned.items():
            if owner in ['request','request_execution']:eq(row['mb'],p['request_mb'])
            elif owner in ['relation_cache','cloud_image_buffer']:eq(row['mb'],p[owner+'_mb'])
            elif owner=='runtime_shared_pages':eq(row['mb'],p['runtime_shared_mb'])
            elif owner.startswith('runtime_private:'):eq(row['mb'],p['runtime_private_mb'])
            elif owner.startswith('weight:'):eq(row['mb'],p[owner.split('_')[-2]+'_weights_mb'])
            elif owner=='vision_ipc':eq(row['mb'],p['ipc_split_mb'] if cfg['dp3']=='B-R3' else p['ipc_shared_mb'])
            end=max(n['finish_s'] for n in ns.values())+0.001
            if owner in ['request','request_execution','relation_cache','cloud_image_buffer','runtime_shared_pages','vision_ipc'] or owner.startswith(('runtime_private:','weight:')):
                if owner=='request':start=0
                elif owner=='request_execution':start=ns['intake']['finish_s']
                elif owner=='relation_cache':start=-8
                elif owner=='cloud_image_buffer':start=ns['read']['finish_s'];end=ns['meaning']['finish_s']
                elif owner in ['runtime_shared_pages','vision_ipc']:start=min(n['start_s'] for n in ns.values() if n['resource']=='visual')
                elif owner.startswith('runtime_private:'):
                    stage=owner.split(':')[1]
                    start=min(n['start_s'] for n in ns.values() if n['id'].endswith('_boot') and (stage=='shared' or n['id'].endswith(stage+'_boot')))
                else:
                    infer=owner.split(':')[1];start=ns[infer[:-5]+'load']['start_s']
                    if cfg['dp3']=='B-R1':end=ns[infer[:-5]+'release']['finish_s']
                eq(row['start_s'],start);eq(row['end_s'],end)
        times={-30,0}
        for i in intervals:times.update([i['start_s'],i['end_s']])
        for a in c['audio']:
            n=ns[a['node']];times.update([n['start_s'],n['start_s']+a['setup_s'],n['finish_s']])
            if a['discard_at'] is not None:times.add(a['discard_at'])
        def ram(t):
            total=sum(p[k] for k in ['core_ui_mb','asr_audio_mb','source_ring_mb','common_history_mb'])
            total+=sum(i['mb'] for i in intervals if i['start_s']<=t<i['end_s'])
            audio=0
            for a in c['audio']:
                n=ns[a['node']]
                if t<n['start_s'] or (a['discard_at'] is not None and t>=a['discard_at']):continue
                length=a['length_s'] if a.get('initial') else min(a['length_s'],max(0,(t-n['start_s']-a['setup_s'])*p['audio_generation_rate']))
                audio+=length*p['pcm_mb_per_audio_s']
            assert audio<=p['audio_cap_mb']
            return total+audio
        eq(c['qa06'],max(ram(t+offset) for t in times for offset in [-1e-8,0]))
        eq(c['qa06'],sum(c['memory_peak']['components_mb'].values()))
        if c['config_id'].startswith('DP3-B'):
            if c['case_id'] in ['U12','U22']:assert 'pre_adopt' in ns
            if c['case_id'] in ['U11','U24']:
                assert ns['expand_need']['resource']=='semantic'
                assert ns['raw_expansion']['deps']==['expand_need']
                assert 'expand_ui_infer' in ns and 'expand_sam_infer' in ns
                assert 'expanded_revision' in ns['meaning']['deps']
        if c['config_id']=='DP44-B':
            if 'next_speech_selection' in ns:
                next_job=ns.get('final_result_audio',ns.get('response_audio'))
                assert next_job['start_s']>=ns['next_speech_selection']['finish_s']-1e-9
            if c['case_id']=='U19':
                eq(ns['old_audio']['audio_s']+ns['resume_tail']['audio_s'],36)
                assert 'first_audio_output' in ns['question_play']['deps']
    for cfg in spec['configs']:
        cid=cfg['id'];rows=[r for r in d['cases'] if r['config_id']==cid]
        assert [r['case_id'] for r in rows] == [r['id'] for r in spec['scenarios']]
        s=d['summary'][cid];cs=[r for r in d['changes'] if r['config_id']==cid]
        assert [r['change_id'] for r in cs] == [f'C0{i}' for i in range(1,7)]
        for r in cs:assert r['module_count']==len(set(r['modules']))
        for q in ['qa01','qa03','first_voice']:eq(s[q],mean(r[q] for r in rows))
        eq(s['qa04'],mean(r['module_count'] for r in cs))
        eq(s['qa05'],sum(r['qa05'] for r in rows));eq(s['qa06'],max(r['qa06'] for r in rows))
        assert s['candidate_outputs']==0 and s['qa02'] is None and s['qa07'] is None
        assert s['receiver_cost'] is None and s['total_system_cost'] is None
        for q in ['qa01','qa03','qa04','qa05','qa06','first_voice']:eq(d['deltas'][cid][q],s[q]-d['summary']['BASE'][q])
    rows=list(csv.DictReader((OUT/'artifacts/comparison.csv').open()))
    assert len(rows)==10 and sum(r['config_id']=='BASE' for r in rows)==4
    for r in rows:
        for q in ['qa01','qa03','qa04','qa05','qa06','first_voice']:eq(float(r[q]),d['summary'][r['config_id']][q])
        assert r['qa02']==r['qa07']==''
    report=dict(status='PASS',configs=7,cases=168,nodes=node_count,cost_rows=usage_count,
                memory_intervals=memory_count,common_change_rows=42,referent_oracles=33,
                model_or_product_run=False,accuracy_outputs=0)
    print(json.dumps(report,ensure_ascii=False,indent=2))


if __name__=='__main__':main()
