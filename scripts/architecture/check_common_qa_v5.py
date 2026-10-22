#!/usr/bin/env python3
"""Independent v5 arithmetic, population, dependency and preservation audit."""
import csv
import hashlib
import json
import math
from pathlib import Path
from statistics import mean

ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'docs/architecture/12-decisions/decision-packages/current/evaluation/common-qa-v5'

def eq(a,b):
    assert math.isclose(a,b,abs_tol=1e-7,rel_tol=1e-9),(a,b)

def merge(intervals,lo,hi):
    result=[]
    for a,b in sorted((max(a,lo),min(b,hi)) for a,b in intervals):
        if a>=b:continue
        if result and a<=result[-1][1]:result[-1][1]=max(result[-1][1],b)
        else:result.append([a,b])
    return result

def duration(intervals):return sum(b-a for a,b in intervals)

def rating(x,bounds):
    if x is None:return None
    for i,b in enumerate(bounds):
        if x<=b:return 6-i
    return 0

def main():
    spec=json.loads((OUT/'specification.json').read_text());d=json.loads((OUT/'artifacts/ledger.json').read_text());p=spec['profile']
    preserved=json.loads((OUT/'preserved-v4-sha256.json').read_text())
    for name,h in preserved.items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==h,name
    for name in ['specification','oracles']:assert d[name+'_sha256']==hashlib.sha256((OUT/(name+'.json')).read_bytes()).hexdigest()
    assert d['generator_sha256']==hashlib.sha256((ROOT/'scripts/architecture/build_common_qa_v5.py').read_bytes()).hexdigest()
    assert d['contracts']==spec['contracts'] and d['contracts']['qa01']=='QA01-VOICE-v2'
    assert d['evidence_kind']=='ANALYTICAL_SIMULATION_NOT_MEASURED'
    assert len(d['cases'])==168 and len(d['changes'])==42
    assert sum(len(o['referents']) for o in json.loads((OUT/'oracles.json').read_text())['items'])==33
    for bounds in [spec['scores']['qa01_upper_bounds'],spec['scores']['qa03_upper_bounds']]:
        for i,b in enumerate(bounds):
            assert rating(b,bounds)==6-i and rating(b+1e-8,bounds)==5-i
            assert rating(b-1e-8,bounds)==6-i
        assert rating(None,bounds) is None
    count_nodes=count_events=0
    for c in d['cases']:
        raw=next(s for s in spec['scenarios'] if s['id']==c['case_id']);s={**spec['scenario_defaults'],**raw}
        cfg=next(cfg for cfg in spec['configs'] if cfg['id']==c['config_id'])
        ns={n['id']:n for n in c['nodes']};assert len(ns)==len(c['nodes']);count_nodes+=len(ns)
        origin=c['goal_origin_s'];assert origin==(-6 if s['revision_of_existing_goal'] else 0)
        assert c['qa02'] is c['qa07'] is None
        for n in ns.values():
            eq(n['finish_s']-n['start_s'],n['duration_s']);assert n['duration_s']>=0
            assert all(ns[k]['finish_s']<=n['start_s']+1e-7 for k in n['causal_parents'])
            if n['resource']=='semantic':eq(n['duration_s'],p['model_setup_s']+n['tokens_in']/p['input_tokens_per_s']+n['tokens_out']/p['output_tokens_per_s'])
            if n['resource']=='speech':eq(n['duration_s'],p['speech_setup_s']+n['tokens_in']/p['input_tokens_per_s']+n['audio_s']/p['audio_generation_rate'])
            if n.get('observes_prefix_of'):
                job=ns[n['observes_prefix_of']];eq(n['start_s'],job['start_s'])
                seg=next(x for x in c['segments'] if x['id']+'_chunk'==n['id'])
                eq(n['duration_s'],p['speech_setup_s']+job['tokens_in']/p['input_tokens_per_s']+(seg['offset_audio_s']+min(seg['audio_s'],p['first_chunk_audio_s']))/p['audio_generation_rate'])
                assert n['resource'] is None and n['tokens_in']==n['tokens_out']==n['audio_s']==0
            if n['resource_wait_s']>1e-9:assert n['resource_predecessor'] in n['causal_parents']
        for resource in ['semantic','speech','visual']:
            pool=[(-30,None)]*p[resource+'_slots']
            jobs=sorted((n for n in ns.values() if n['resource']==resource),key=lambda n:(n['start_s'],list(ns).index(n['id'])))
            for n in jobs:
                assert 0<=n['resource_slot']<len(pool)
                earliest=max([n['release_s'],*[ns[k]['finish_s'] for k in n['deps']]])
                expected=min((max(earliest,t),slot,pred) for slot,(t,pred) in enumerate(pool))
                eq(n['start_s'],expected[0]);assert n['resource_slot']==expected[1]
                assert n['resource_predecessor']==expected[2];eq(n['resource_wait_s'],n['start_s']-earliest)
                pool[expected[1]]=(n['finish_s'],n['id'])
            for slot in range(p[resource+'_slots']):
                jobs=sorted((n for n in ns.values() if n['resource']==resource and n['resource_slot']==slot),key=lambda n:n['start_s'])
                assert all(a['finish_s']<=b['start_s']+1e-8 for a,b in zip(jobs,jobs[1:]))
        def replay(projected):
            memo={};visiting=set()
            def end(key):
                if key in memo:return memo[key]
                assert key not in visiting;visiting.add(key);n=ns[key]
                if n['finish_s']<=origin:value=0
                else:
                    seconds=max(0,n['finish_s']-max(n['start_s'],origin))
                    if projected and n['kind']!='via':seconds=0
                    value=max([0,*[end(k) for k in n['causal_parents']]])+seconds
                visiting.remove(key);memo[key]=value;return value
            return end(c['terminal_node'])
        eq(replay(False),c['goal_wall_s']);eq(replay(True),c['qa03'])
        terminal=ns[c['terminal_node']]
        assert terminal['deps']==c['required_state_nodes']+c['required_channel_nodes']
        assert 'screen_accept' in c['required_channel_nodes'] and any(k.endswith('_accept') and k!='screen_accept' for k in c['required_channel_nodes'])
        assert 'first_display' not in terminal['deps'] and 'final_screen_display' not in terminal['deps']
        if s['delegate']:
            assert c['required_external_nodes'][:2]==['agent_receipt','result_receive']
            assert ns['agent_result']['deps']==['agent_receipt'] and ns['result_adopt']['deps']==['result_receive']
        play=[[ns[x['play']]['start_s'],ns[x['play']]['finish_s']] for x in c['segments']]
        for a,b in zip(sorted(play),sorted(play)[1:]):assert a[1]<=b[0]+1e-8
        for e in c['events']:
            count_events+=1;assert e['physical_samples']==e['proxy_samples']==0 and e['content_error']=='UNKNOWN'
            expected=s['user_voice_class'] if e['kind']=='USER_INPUT' else s['agent_result_class'] if e['kind']=='AGENT_RESULT' else s['agent_question_class'] if e['kind']=='AGENT_QUESTION' else 'WAIT_USER'
            assert e['profile']==expected
            if e['kind'].startswith('AGENT_'):
                eq(e['received_s']-e['source_s'],p['network_s'])
                assert ns[e['adoption']]['finish_s']>=e['received_s']
            if e['status']=='WITHDRAWN_BEFORE_AUDIO':
                assert s['id']=='U18' and e['L'] is None and e['first_audio_s'] is None
                assert ns['obsolete_source']['finish_s']<0 and e['withdrawal_s']>=0
                continue
            seg=next(x for x in c['segments'] if x['id']==e['segment_id']);start=ns[e['source']]['finish_s'];finish=ns[seg['output']]['finish_s']
            eq(e['L'],finish-start);assert finish>=start
            user=merge([[-s['utterance_s'],0]] if s['modality']=='voice' else [],start,finish)
            previous=merge([[ns[x['play']]['start_s'],ns[x['play']]['finish_s']] for x in c['segments'] if x['id']!=seg['id']],start,finish)
            blocked=merge(user+previous,start,finish)
            eq(e['U'],duration(user));eq(e['P'],duration(previous));eq(e['B'],duration(blocked));eq(e['overlap'],e['U']+e['P']-e['B'])
            eq(e['E'],e['L']-e['B']);eq(e['R'],finish-(blocked[-1][1] if blocked else start))
            adoption=max(start,ns[e['adoption']]['finish_s']);eq(e['D'],finish-adoption-duration(merge(blocked,adoption,finish)))
            assert e['normal_collision']==(e['profile']=='NORMAL' and e['B']>1e-9)
            job=ns[seg['audio_job']];eq(e['prepared_during_wait_s'],duration(merge(blocked,job['start_s'],job['finish_s'])))
        if s['id']=='U23':
            user=next(e for e in c['events'] if e['kind']=='USER_INPUT');question=next(e for e in c['events'] if e['kind']=='AGENT_QUESTION')
            assert question['first_audio_s']<0<user['first_audio_s'] and question['segment_id']!=user['segment_id']
        if s['id']=='U19':
            assert ns['old_audio']['start_s']==-4 and ns['prior_play']['finish_s']==-3
            resume=next(x for x in c['segments'] if x['id']=='resume')
            eq(resume['audio_s']+ns['prior_play']['duration_s'],36)
            eq(ns['old_audio']['audio_s']+ns.get('resume_tail',{}).get('audio_s',0),36)
        if s['revision_of_existing_goal']:
            assert ns['original_work']['resource']=='semantic'
            assert ns['original_work']['tokens_in']==600 and ns['original_work']['tokens_out']==136
            eq(ns['original_work']['duration_s'],2)
        expected=[]
        for n in ns.values():
            for bucket,field,rate in [('text_input','tokens_in',p['input_usd_per_token']),('text_output','tokens_out',p['output_usd_per_token']),('audio_output','audio_s',p['audio_output_usd_per_s'])]:
                if n[field]:expected.append((n['id'],bucket,n[field],rate))
        if s['modality']=='voice':expected.append(('input_end','audio_input',s['utterance_s'],p['audio_input_usd_per_s']))
        assert sorted(expected)==sorted((u['node'],u['bucket'],u['amount'],u['rate_usd']) for u in c['usage'])
        for u in c['usage']:eq(u['usd'],u['amount']*u['rate_usd'])
        eq(c['qa05'],sum(u['usd'] for u in c['usage']))
        ints=c['memory_intervals'];assert len({q['owner'] for q in ints})==len(ints)
        owned={q['owner']:q for q in ints}
        required={'request'}
        if cfg['dp2']=='B' and (s['long_work'] or s['overlap'] in ('spare','dependent','saturated')):required.add('request_execution')
        if cfg['dp5']=='B' and s['history'] in ('ready','dirty'):required.add('relation_cache')
        if s['screen'] in ('cold','tracked','warm'):required.add('cloud_image_buffer')
        for n in ns.values():
            if n['resource']=='semantic':
                required.add(n['id']+':client')
                q=owned[n['id']+':client'];eq(q['mb'],p['call_base_mb']+n['tokens_in']/1000*p['call_mb_per_1000_tokens'])
                eq(q['start_s'],n['start_s']);eq(q['end_s'],n['finish_s'])
            if n['resource']=='visual' and n['id'].endswith(('_infer','_load')):
                stage=n['id'].split('_')[-2];label=stage+('_workspace' if n['id'].endswith('_infer') else '_load_temp')
                assert n['state']=={label:p[label+'_mb']}
            if n['id'] in ('relation_prior','relation_unused','relation_update'):
                assert n['state']=={'relation_producer':p['relation_producer_mb']}
            for owner,amount in n['state'].items():
                required.add(n['id']+':'+owner)
                q=owned[n['id']+':'+owner];eq(q['mb'],amount);eq(q['start_s'],n['start_s']);eq(q['end_s'],n['finish_s'])
            if n['resource']=='visual' and n['id'].endswith('_boot'):
                required.update(['runtime_shared_pages','vision_ipc','visual_retained_state'])
                stage=n['id'].split('_')[-2]
                required.add('runtime_private:'+(stage if cfg['dp3']=='B-R3' else 'shared'))
            if n['resource']=='visual' and n['id'].endswith('_load'):required.add('weight:'+n['id'][:-4]+'infer')
        assert set(owned)==required,(c['config_id'],c['case_id'],set(owned)^required)
        episode_end=max(n['finish_s'] for n in ns.values())+.001
        for owner,q in owned.items():
            if owner in ('request','request_execution','relation_cache','cloud_image_buffer','runtime_shared_pages','vision_ipc','visual_retained_state') or owner.startswith(('runtime_private:','weight:')):
                end=episode_end
                if owner=='request':amount=p['request_mb'];start=0
                elif owner=='request_execution':amount=p['request_mb'];start=ns['intake']['finish_s']
                elif owner=='relation_cache':amount=p['relation_cache_mb'];start=-8
                elif owner=='cloud_image_buffer':amount=p['cloud_image_buffer_mb'];start=ns['read']['finish_s'];end=ns['meaning']['finish_s']
                elif owner in ('runtime_shared_pages','vision_ipc','visual_retained_state'):
                    amount=p['runtime_shared_mb' if owner=='runtime_shared_pages' else 'visual_state_mb' if owner=='visual_retained_state' else 'ipc_split_mb' if cfg['dp3']=='B-R3' else 'ipc_shared_mb']
                    start=min(n['start_s'] for n in ns.values() if n['resource']=='visual')
                elif owner.startswith('runtime_private:'):
                    stage=owner.split(':')[1];amount=p['runtime_private_mb'];start=min(n['start_s'] for n in ns.values() if n['id'].endswith('_boot') and (stage=='shared' or n['id'].endswith(stage+'_boot')))
                else:
                    infer=owner.split(':')[1];stage=infer.split('_')[-2];amount=p[stage+'_weights_mb'];start=ns[infer[:-5]+'load']['start_s']
                    if cfg['dp3']=='B-R1':end=ns[infer[:-5]+'release']['finish_s']
                eq(q['mb'],amount);eq(q['start_s'],start);eq(q['end_s'],end)
        if any(n['resource']=='visual' for n in ns.values()):
            for owner,key in [('runtime_shared_pages','runtime_shared_mb'),('visual_retained_state','visual_state_mb'),('vision_ipc','ipc_split_mb' if cfg['dp3']=='B-R3' else 'ipc_shared_mb')]:eq(owned[owner]['mb'],p[key])
        for q in ints:
            expected_added=q['owner'].startswith(('weight:','runtime_')) or q['owner'] in ('vision_ipc','visual_retained_state') or q['owner'].endswith(('_workspace','_load_temp'))
            assert q['qa08_attributable']==expected_added and q['physical_resource']=='host_ram'
        times={-30,0}
        for q in ints:times.update([q['start_s'],q['end_s']])
        for a in c['audio']:
            n=ns[a['node']];times.update([n['start_s'],n['start_s']+a['setup_s'],n['finish_s']])
            if a['discard_at'] is not None:times.add(a['discard_at'])
        ram=[];added=[]
        for t in [x+offset for x in times for offset in [0,-1e-8]]:
            values=[q for q in ints if q['start_s']<=t<q['end_s']]
            audio=0
            for a in c['audio']:
                n=ns[a['node']]
                if t<n['start_s'] or a['discard_at'] is not None and t>=a['discard_at']:continue
                seconds=a['length_s'] if a.get('initial') else max(0,min(a['length_s'],(t-n['start_s']-a['setup_s'])*p['audio_generation_rate']))
                audio+=seconds*p['pcm_mb_per_audio_s']
            ram.append(sum(p[k] for k in ['core_ui_mb','asr_audio_mb','source_ring_mb','common_history_mb'])+sum(q['mb'] for q in values)+audio)
            added.append(sum(q['mb'] for q in values if q['qa08_attributable']))
        eq(c['qa06'],max(ram));eq(c['qa08'],max(added));assert c['qa08']<=c['qa06']
        eq(c['qa08'],sum(c['qa08_peak']['components_mb'].values()));assert c['qa08_peak']['dedicated_accelerator_mb']==0
        if cfg['dp3']=='A':assert c['qa08']==0
        if s.get('retained_prior_audio_s'):
            a=next(a for a in c['audio'] if a['node']=='retained_prior_audio')
            assert a['initial'] and a['length_s']==s['retained_prior_audio_s'] and a['discard_at'] is None
            assert ns['retained_prior_audio']['finish_s']==-12 and not any(u['node']=='retained_prior_audio' for u in c['usage'])
    for cfg in spec['configs']:
        cid=cfg['id'];rows=[c for c in d['cases'] if c['config_id']==cid];events=[e for c in rows for e in c['events']];s=d['summary'][cid]
        assert [c['case_id'] for c in rows]==[x['id'] for x in spec['scenarios']]
        normal=[e for e in events if e['profile']=='NORMAL'];wait=[e for e in events if e['profile']!='NORMAL' and e['L'] is not None]
        assert len(events)==42 and len(normal)==36 and len(wait)==5
        eq(s['qa01'],mean(e['L'] for e in normal));eq(s['qa03'],mean(c['qa03'] for c in rows))
        eq(s['qa04'],mean(r['module_count'] for r in d['changes'] if r['config_id']==cid));eq(s['qa05'],sum(c['qa05'] for c in rows))
        for q in ['qa06','qa08']:eq(s[q],max(c[q] for c in rows))
        for q in ['qa01','qa03']:assert s[q+'_score']==rating(s[q],spec['scores'][q+'_upper_bounds'])
        for key in ['L','U','P','overlap','B','E','R','D','prepared_during_wait_s']:eq(s['wait'][key],mean(e[key] for e in wait))
        assert s['qa02'] is s['qa07'] is s['qa08_score'] is s['content_errors'] is s['receiver_cost'] is None
    comparison=list(csv.DictReader((OUT/'artifacts/comparison.csv').open()));assert len(comparison)==10
    for row in comparison:
        for q in ['qa01','qa03','qa04','qa05','qa06','qa08']:eq(float(row[q]),d['summary'][row['config_id']][q])
    print(json.dumps(dict(status='PASS',cases=168,events=count_events,nodes=count_nodes,
                         preserved_files=len(preserved),wall_reconstruction=True,physical_observations=0),indent=2))

if __name__=='__main__':main()
