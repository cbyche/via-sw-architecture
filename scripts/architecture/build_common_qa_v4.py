#!/usr/bin/env python3
"""Common-input analytical model. No product, model, provider or device calls.

The authored workload and coefficient file are read, never fitted to results.
QA02/07 remain unavailable without candidate outputs. Oracle is not used by
the scheduler. Every configuration executes the same source event schedule.
"""
from __future__ import annotations
import argparse
import copy
import csv
import hashlib
import json
import math
from decimal import Decimal, ROUND_HALF_UP
from pathlib import Path
from statistics import mean

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/current/evaluation/common-qa-v4'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


class Graph:
    def __init__(self, profile):
        self.p = profile
        self.nodes = []
        self.states = []
        self.audio = []

    def add(self, key, label, seconds=0, *, deps=(), release=-30, resource=None,
            kind='via', tokens_in=0, tokens_out=0, audio_s=0, phase='request', state=None):
        assert not any(n['id'] == key for n in self.nodes), key
        assert seconds >= 0
        self.nodes.append(dict(id=key, label=label, duration_s=seconds, deps=list(deps),
                               release_s=release, resource=resource, kind=kind,
                               tokens_in=tokens_in, tokens_out=tokens_out, audio_s=audio_s,
                               phase=phase, state=state or {}))
        return key

    def model(self, key, label, tin, tout, **kw):
        p = self.p
        return self.add(key, label, p['model_setup_s'] + tin/p['input_tokens_per_s'] + tout/p['output_tokens_per_s'],
                        tokens_in=tin, tokens_out=tout, resource='semantic', **kw)

    def speech(self, key, label, length, *, deps=(), release=-30, phase='request', tin_extra=0):
        p = self.p
        tin = length * 8 + tin_extra
        setup = p['speech_setup_s'] + tin / p['input_tokens_per_s']
        node = self.add(key, label, setup + length/p['audio_generation_rate'], deps=deps,
                        release=release, resource='speech', tokens_in=tin, audio_s=length, phase=phase)
        self.audio.append(dict(node=node, length_s=length, setup_s=setup, discard_at=None))
        return node

    def run(self):
        p = self.p
        pools = {name: [(-30, None)] * p[name + '_slots'] for name in ['semantic','speech','visual']}
        done = {}
        todo = list(self.nodes)
        while todo:
            ready = []
            for order, n in enumerate(todo):
                if all(k in done for k in n['deps']):
                    start = max([n['release_s'], *[done[k]['finish_s'] for k in n['deps']]])
                    slot = None
                    pred = None
                    if n['resource']:
                        candidates = [(max(start, t), j, k) for j, (t,k) in enumerate(pools[n['resource']])]
                        start, slot, pred = min(candidates, key=lambda x: (x[0], x[1]))
                    ready.append((start, order, slot, pred, n))
            assert ready, 'cycle or missing dependency'
            start, _, slot, pred, n = min(ready, key=lambda x: (x[0], x[1]))
            todo.remove(n)
            n['start_s'] = start
            n['finish_s'] = start + n['duration_s']
            n['resource_predecessor'] = pred
            n['resource_slot'] = slot
            earliest=max([n['release_s'],*[done[k]['finish_s'] for k in n['deps']]])
            n['resource_wait_s']=start-earliest
            # A slot used earlier is not a dependency when it was already free.
            # Only real resource blocking creates an additional causal edge.
            blocked=pred and n['resource_wait_s']>1e-10
            n['causal_parents'] = n['deps'] + ([pred] if blocked and pred not in n['deps'] else [])
            if slot is not None:
                pools[n['resource']][slot] = (n['finish_s'], n['id'])
            done[n['id']] = n
        return done

    def projected(self, terminal, origin):
        nodes = {n['id']: n for n in self.nodes}
        memo = {}
        def end(key):
            if key in memo:
                return memo[key]
            n = nodes[key]
            if n['finish_s'] <= origin:
                result = 0
            else:
                dur = max(0, n['finish_s'] - max(n['start_s'], origin)) if n['kind'] == 'via' else 0
                result = max([0, *[end(k) for k in n['causal_parents']]]) + dur
            memo[key] = result
            return result
        return end(terminal)


def build_case(spec, config, raw):
    p, t = spec['profile'], spec['token_budget']
    s = {**spec['scenario_defaults'], **raw}
    g = Graph(p)
    # Background steps are public initial conditions; no hidden answer is fed
    # into the request. Current A waits for its step, not the whole old task.
    background = []
    if s['overlap'] in ('spare','dependent','saturated'):
        background.append(g.model('bg1','진행 중인 현재 대화의 의미 step',t['background_input'],t['background_output'],release=-1,phase='background'))
        if s['overlap'] == 'saturated':
            background.append(g.model('bg2','다른 대화의 의미 step',t['background_input'],t['background_output'],release=-1,phase='background'))
    prior = []
    if s['revision_of_existing_goal']:
        prior.append(g.add('original_work','원래 목표의 최초 VIA 처리',2,release=-6,phase='prior_goal'))
        if background:
            next(n for n in g.nodes if n['id']==background[0])['deps'].extend(prior)
    g.add('input_end','사용자 입력 종료',kind='external',release=0,deps=prior)
    intake = g.add('intake','입력 확정',p['voice_finalize_s'] if s['modality']=='voice' else p['text_finalize_s'],deps=['input_end'])

    # Production is costed even when unused. A and B both reuse confirmed facts.
    relation_jobs = []
    if config['dp5'] == 'B' and s['history'] in ('ready','dirty'):
        relation_jobs.append(g.model('relation_prior','미확정 피드백 관계 사전 생산',t['relation_production_input'],t['relation_production_output'],release=-8,phase='preparation',state={'relation_producer':p['relation_producer_mb']}))
        relation_jobs.append(g.model('relation_unused','같은 이력의 미사용 관계 생산',t['relation_production_input'],t['relation_production_output'],release=-8,phase='preparation',state={'relation_producer':p['relation_producer_mb']}))
        if s['history'] == 'dirty':
            relation_jobs.append(g.model('relation_update','정정 원본 관계 갱신',t['relation_production_input'],t['relation_production_output'],release=-0.2,phase='preparation',state={'relation_producer':p['relation_producer_mb']}))
    read = g.add('read','공통 허용 원본 조회',p['source_read_s'],deps=[intake])

    # Same CPU allowance and dependency chain for every B runtime. There is no
    # invented R3 parallel speedup. R1 releases weights, not source evidence.
    vision_end = read
    vision_records = []
    boot_nodes = {}
    loaded_models = {}
    def vision(prefix, anchor, release, phase, frames, tracked):
        nonlocal vision_records
        if config['dp3'] == 'A':
            return g.model(prefix+'raw','선행 원본 화면 해석',t['request_base']+t['image_direct']*frames,t['first_adopted_output'],deps=anchor,release=release,phase=phase)
        prev = list(anchor)
        for stage in (['ui','sam'] if tracked else ['ui']):
            runtime_key = stage if config['dp3']=='B-R3' else 'shared'
            boot = None
            if runtime_key not in boot_nodes:
                boot = g.add(prefix+stage+'_boot',stage+' runtime 시작',p['runtime_boot_s'],deps=prev,release=release,resource='visual',phase=phase)
                boot_nodes[runtime_key] = boot
            load_deps = list(dict.fromkeys([*prev,boot_nodes[runtime_key]]))
            load = None
            if stage not in loaded_models or config['dp3']=='B-R1':
                load = g.add(prefix+stage+'_load',stage+' 모델 적재',p[stage+'_load_s'],deps=load_deps,release=release,resource='visual',phase=phase,state={stage+'_load_temp':p[stage+'_load_temp_mb']})
                loaded_models[stage] = load
            infer = g.add(prefix+stage+'_infer',stage+' 후보 생산',p['ui_inference_s'] if stage=='ui' else p['sam_frame_s']*frames,
                          deps=[load] if load else load_deps,release=release,resource='visual',phase=phase,state={stage+'_workspace':p[stage+'_workspace_mb']})
            finish = infer
            if config['dp3'] == 'B-R1':
                finish = g.add(prefix+stage+'_release',stage+' 가중치 회수',p['release_s'],deps=[infer],resource='visual',phase=phase)
            if config['dp3'] == 'B-R3':
                finish = g.add(prefix+stage+'_ipc',stage+' 결과 IPC',p['r3_ipc_s'],deps=[infer],phase=phase)
            vision_records.append(dict(stage=stage,boot=boot,load=load,infer=infer,end=finish,phase=phase))
            prev = [finish]
        return prev[0]
    if s['screen'] in ('warm','cached'):
        pre = vision('pre_',[], -12,'preparation',1,False)
        if config['dp3']!='A':
            pre=g.model('pre_adopt','선행 후보의 요청 의미 확정',t['request_base']+t['image_candidates'],t['first_adopted_output'],deps=[pre],phase='preparation')
        if s['screen']=='warm':
            vision_end = vision('now_',[read,pre],-30,'request',1,False) if config['dp3']!='A' else g.add('read_new_view','새 viewport 원본 조회',p['source_read_s'],deps=[read,pre])
        else:
            vision_end = g.add('reuse_screen','이미 확정한 화면 대상 재사용',p['source_read_s'],deps=[read,pre])
    elif s['screen'] in ('cold','tracked') and config['dp3'] != 'A':
        vision_end = vision('now_',[read],-30,'request',s['frames'],s['screen']=='tracked')
    elif s['screen']=='native':
        vision_end = g.add('native_selection','Native 선택 사실 조회',p['source_read_s'],deps=[read])
    if s['raw_expansion'] and config['dp3'] != 'A':
        expand_need = g.model('expand_need','초기 binder의 부족 판정과 ExpandNeed 생성',t['request_base']+t['image_candidates']*s['frames'],t['expansion_request_output'],deps=[vision_end])
        expanded_read = g.add('raw_expansion','ExpandNeed에 따른 원본 확장 조회',p['source_read_s'],deps=[expand_need])
        vision_end = vision('expand_',[expanded_read],-30,'request',1,s['screen']=='tracked')
        vision_end = g.add('expanded_revision','확장 후보 revision 게시 및 결합 재개',p['adopt_s'],deps=[vision_end])

    tin = t['request_base']
    if s['screen'] in ('cold','tracked','warm'):
        tin += (t['image_direct'] if config['dp3']=='A' else t['image_candidates']) * s['frames']
        if s['raw_expansion'] and config['dp3']!='A':
            tin += t['raw_expansion']
    if s['history'] in ('ready','dirty'):
        tin += t['history_relation'] if config['dp5']=='B' and s['history']=='ready' else t['history_raw']
    if config['dp2']=='B' and (s['long_work'] or background):
        tin += t['dp2_execution_envelope']
    parents = [vision_end]
    if background and (config['dp2']=='A' or s['overlap']=='dependent'):
        parents.append(background[0])
    if config['dp5']=='B' and s['history']=='ready':
        parents.append('relation_prior')
    meaning = g.model('meaning','최초 요청 의미 채택 packet 생성',tin,t['first_adopted_output'],deps=parents)
    adopted = g.add('adopt','현재 revision과 권한 검사 및 채택',p['adopt_s'],deps=[meaning])
    display = g.add('first_display','첫 구체적 채택 내용 화면 표시',p['display_s'],deps=[adopted])
    work = adopted
    if s['long_work']:
        work = g.model('long_work','요청의 남은 VIA 판단',t['long_work_input']+(t['dp2_execution_envelope'] if config['dp2']=='B' else 0),t['long_work_output'],deps=[adopted])
    if s['delegate']:
        ingress = g.add('agent_ingress','Agent 요청 전송',0.04,deps=[work])
        returned = g.add('agent_result','외부 Agent 업무',p['agent_work_s'],deps=[ingress],kind='external')
        result_display = g.add('result_display','최종 결과 상세 표시',p['display_s'],deps=[returned])
        response_ready = returned
    else:
        result_display = display if not s['long_work'] else g.add('result_display','최종 설명 상세 표시',p['display_s'],deps=[work])
        response_ready = work

    # Build requested voice and optional waiting responses. All text comes from
    # a common content source. DP44 does not gain an extra writing LLM.
    voice_source = response_ready
    voice_key = 'response_audio'
    if s['delivery'] in ('queued','cancel','resume'):
        g.add('question_arrival','Agent 질문 텍스트 도착',kind='external',release=-0.5 if s['delivery']=='resume' else -2)
        voice_source = 'question_arrival'
        voice_key = 'question_audio'
    if s['delivery']=='cached':
        g.add('existing_audio','공통 유효 audio 초기 상태',kind='external',release=-12)
        voice_key = 'existing_audio'
        g.audio.append(dict(node=voice_key,length_s=s['audio_s'],setup_s=0,discard_at=None,initial=True))
    else:
        speech_deps = [voice_source] + ([adopted] if config['dp44']=='B' else [])
        g.speech(voice_key,'선택한 응답 음성 생성',s['audio_s'],deps=speech_deps,
                 phase='preparation' if voice_source=='question_arrival' and config['dp44']=='A' else 'request')
    if s['delegate'] and voice_source=='question_arrival':
        g.speech('final_result_audio','새 업무 최종 결과 음성 준비',6,deps=[response_ready])
    if s['delivery']=='cancel' and config['dp44']=='A':
        g.speech('obsolete_audio','철회되는 보고서의 선행 생성',10,release=-2,phase='preparation')
    if s['delivery']=='resume':
        g.speech('old_audio','중단 전 설명 생성',36,release=-2,phase='preparation')
    first_voice_key=voice_key
    if s['delivery']=='new' and (s['delegate'] or s['long_work']):
        first_voice_key=g.speech('intent_audio','채택한 구체적 수행 의도의 짧은 음성',p['intent_audio_s'],deps=[adopted])
    initial = g.run()
    # This milestone observes a prefix of the same speech job. It owns no
    # additional slot, cost or memory. MAX paths do not sum prefix and full job.
    if first_voice_key=='existing_audio':
        prefix=first_voice_key
    else:
        job=initial[first_voice_key]
        a=next(a for a in g.audio if a['node']==first_voice_key)
        prefix=g.add('first_chunk','같은 생성 job의 유효 첫 chunk 관측',a['setup_s']+min(a['length_s'],p['first_chunk_audio_s'])/p['audio_generation_rate'],deps=job['causal_parents'],release=job['release_s'])
        next(n for n in g.nodes if n['id']==prefix)['observes_prefix_of']=first_voice_key
    device = g.add('first_audio_output','공통 장치의 첫 음성 출력',p['device_s'],deps=[adopted,prefix])
    terminal_parents = list(dict.fromkeys([result_display,voice_key,first_voice_key,device]))
    # Shared non-preemptive speech selection: finish the current valid short
    # utterance before selecting the final answer. A may prepare it meanwhile.
    # B must wait for selection even when the text was available earlier.
    next_speech = 'response_audio' if first_voice_key=='intent_audio' else ('final_result_audio' if s['delegate'] and voice_source=='question_arrival' else None)
    if next_speech:
        first_length=next(a['length_s'] for a in g.audio if a['node']==first_voice_key)
        selection=g.add('next_speech_selection','현재 발화 재생 후 다음 응답 선택',first_length,deps=[device],kind='external')
        if config['dp44']=='B':
            next(n for n in g.nodes if n['id']==next_speech)['deps'].append(selection)
    if s['delegate'] and voice_source=='question_arrival':
        terminal_parents.append('final_result_audio')
    if s['delivery']=='cancel' and config['dp44']=='A':
        n=initial['obsolete_audio']; a=next(a for a in g.audio if a['node']=='obsolete_audio')
        end=min(n['finish_s'],initial[adopted]['finish_s']+p['cancel_tail_s'])
        n['duration_s']=end-n['start_s']; n['audio_s']=max(0,min(10,(end-n['start_s']-a['setup_s'])*p['audio_generation_rate']))
        a['length_s']=n['audio_s']; a['discard_at']=initial[adopted]['finish_s']
        terminal_parents.append('obsolete_audio')  # cancellation settlement
    if s['delivery']=='resume':
        n=initial['old_audio']; a=next(a for a in g.audio if a['node']=='old_audio')
        if config['dp44']=='B':
            end=min(n['finish_s'],initial[adopted]['finish_s']+p['cancel_tail_s'])
            n['duration_s']=end-n['start_s']; n['audio_s']=max(0,min(36,(end-n['start_s']-a['setup_s'])*p['audio_generation_rate']))
            a['length_s']=n['audio_s']
        # Actual question playback is external to QA03, but retains ordering.
        play=g.add('question_play','첫 출력부터 질문 순수 재생 및 차례',s['audio_s'],deps=[device],kind='external')
        if config['dp44']=='B' and a['length_s']<36:
            tail=g.speech('resume_tail','보존 buffer 뒤 미생성 후반만 생성',36-a['length_s'],deps=[play,'old_audio'],tin_extra=120)
            terminal_parents.extend([play,tail])
        else:
            terminal_parents.extend([play,'old_audio'])
    terminal=g.add('terminal','공통 목표 최종 상태와 전체 음성 준비',0,deps=terminal_parents)
    nodes=g.run()
    first_voice=nodes[device]['finish_s']
    origin=-6 if s['revision_of_existing_goal'] else 0
    # Every active memory interval owns a disjoint component. Model buffers
    # and retained weights are separate; remote weights and KV are excluded.
    end=max(n['finish_s'] for n in g.nodes)+0.001
    base={k:p[k] for k in ('core_ui_mb','asr_audio_mb','source_ring_mb','common_history_mb')}
    states=[]
    def state(name,amount,start,finish):
        states.append(dict(owner=name,mb=amount,start_s=start,end_s=finish))
    state('request',p['request_mb'],0,end)
    if config['dp2']=='B' and (s['long_work'] or background):
        state('request_execution',p['request_mb'],nodes[intake]['finish_s'],end)
    if config['dp5']=='B' and s['history'] in ('ready','dirty'):
        state('relation_cache',p['relation_cache_mb'],-8,end)
    for n in g.nodes:
        if n['resource']=='semantic':
            state(n['id']+':client',p['call_base_mb']+n['tokens_in']/1000*p['call_mb_per_1000_tokens'],n['start_s'],n['finish_s'])
        for owner,amount in n['state'].items():
            state(n['id']+':'+owner,amount,n['start_s'],n['finish_s'])
    if s['screen'] in ('cold','tracked','warm'):
        state('cloud_image_buffer',p['cloud_image_buffer_mb'],nodes[read]['finish_s'],nodes[meaning]['finish_s'])
    seen_boot=set(); seen_weight=set()
    if vision_records:
        first_start=min(nodes[r['boot'] or r['load'] or r['infer']]['start_s'] for r in vision_records)
        state('runtime_shared_pages',p['runtime_shared_mb'],first_start,end)
        state('vision_ipc',p['ipc_split_mb'] if config['dp3']=='B-R3' else p['ipc_shared_mb'],first_start,end)
        for r in vision_records:
            stage=r['stage']; key=stage if config['dp3']=='B-R3' else 'shared'
            if key not in seen_boot:
                state('runtime_private:'+key,p['runtime_private_mb'],nodes[r['boot'] or r['infer']]['start_s'],end);seen_boot.add(key)
            if config['dp3']=='B-R1' or stage not in seen_weight:
                state('weight:'+r['infer'],p[stage+'_weights_mb'],nodes[r['load'] or r['infer']]['start_s'],nodes[r['end']]['finish_s'] if config['dp3']=='B-R1' else end);seen_weight.add(stage)
    times={-30,0,end}
    for q in states:
        times.update([q['start_s'],q['end_s']])
    for a in g.audio:
        n=nodes[a['node']]
        times.update([n['start_s'],n['start_s']+a['setup_s'],n['finish_s']])
        if a['discard_at'] is not None: times.add(a['discard_at'])
    samples=[]
    for x in sorted(times):
        for x in (x-1e-8,x):
            components=dict(base)
            for q in states:
                if q['start_s']<=x<q['end_s']: components[q['owner']]=q['mb']
            audio_mb=0
            for a in g.audio:
                n=nodes[a['node']]
                if x<n['start_s'] or (a['discard_at'] is not None and x>=a['discard_at']): continue
                seconds=a['length_s'] if a.get('initial') else max(0,min(a['length_s'],(x-n['start_s']-a['setup_s'])*p['audio_generation_rate']))
                audio_mb+=seconds*p['pcm_mb_per_audio_s']
            components['valid_audio']=audio_mb
            assert audio_mb<=p['audio_cap_mb'],('audio cap',s['id'])
            samples.append(dict(time_s=x,components_mb=components,total_mb=sum(components.values())))
    peak=max(samples,key=lambda m:m['total_mb'])
    usage=[]
    for n in g.nodes:
        for bucket,amount,rate in [('text_input',n['tokens_in'],p['input_usd_per_token']),('text_output',n['tokens_out'],p['output_usd_per_token']),('audio_output',n['audio_s'],p['audio_output_usd_per_s'])]:
            if amount:usage.append(dict(node=n['id'],phase=n['phase'],bucket=bucket,amount=amount,rate_usd=rate,usd=amount*rate))
    if s['modality']=='voice':usage.append(dict(node='input_end',phase='input',bucket='audio_input',amount=s['utterance_s'],rate_usd=p['audio_input_usd_per_s'],usd=s['utterance_s']*p['audio_input_usd_per_s']))
    return dict(config_id=config['id'],case_id=s['id'],group=s['group'],modality=s['modality'],
                qa01=nodes[display]['finish_s'],qa02=None,qa03=g.projected(terminal,origin),
                qa05=sum(u['usd'] for u in usage),qa06=peak['total_mb'],qa07=None,first_voice=first_voice,
                goal_origin_s=origin,goal_wall_s=nodes[terminal]['finish_s']-origin,
                first_display_node=display,meaning_node=adopted,terminal_node=terminal,first_audio_node=first_voice_key,
                status='CONDITIONAL_NORMAL_PATH_SIMULATION',accuracy_status='NO_CANDIDATE_OUTPUT',
                nodes=g.nodes,usage=usage,memory_intervals=states,audio=g.audio,memory_peak=peak)


def changes(config):
    # Common product modules, counted once per task, not sums of old layer means.
    specs=[('C01','모델 wire 형식 변경',['model_access_wire']),
           ('C02','Agent 요청/취소/receipt 형식 변경',['agent_wire']),
           ('C03','원본 좌표/시각/revision 표현 변경',['source_contract','source_read']),
           ('C04','선택 section만 수정하고 관련 대기 음성 갱신',['response_revision','ui_render','delivery_history']),
           ('C05','실제로 전달한 설명의 원본 위치/revision 표시',['source_citation','ui_render']),
           ('C06','선택 source extent 권한 철회',['source_epoch','response_revision'])]
    answer=[]
    for cid,title,mods in specs:
        if cid in ('C04','C06'):
            mods+=['speech_prepare','prepared_audio_range'] if config['dp44']=='A' else ['speech_turn']
        if config['dp3']!='A' and cid in ('C03','C06'):mods+=['visual_evidence_contract']
        if config['dp5']=='B' and cid in ('C03','C05','C06'):mods+=['relation_provenance']
        # DP2 A/B share the same revision and wire contracts. Do not manufacture
        # a module advantage solely from their Component/process count.
        answer.append(dict(config_id=config['id'],change_id=cid,title=title,modules=['concept::'+v for v in sorted(set(mods))],module_count=len(set(mods)),evidence='DESIGN_IMPACT_ESTIMATE_NOT_RUST_DIFF'))
    return answer


def document_tables(spec, oracle, data, directory):
    """Keep human-facing values and ranges generated from the same ledger."""
    directory.mkdir(parents=True,exist_ok=True)
    header='# 공통 입력과 구성\n\n`COMMON-QA-MODEL-4.0` / 분석 모델 / [시작](README.md) / [계약](01-common-contract.md)\n\n'
    header+='7개 구성 모두 아래 24개 입력과 동일한 초기 원본, 사건 시각, 목표 및 출력 의무를 사용한다. A/A/A/A를 기준으로 한 DP만 바꾼다. 각 입력은 한 번 계산하며 독립 통계 반복이나 실사용 빈도 표집이 아니다. `source_family`는 v3의 관련 사례를 추적하는 표식이며 같은 시험이나 같은 원시 데이터라는 뜻이 아니다.\n\n'
    header+='| 구성 | DP2 | DP3 | DP5 | DP44 |\n| --- | --- | --- | --- | --- |\n'
    header+=''.join(f"| {c['id']} | {c['dp2']} | {c['dp3']} | {c['dp5']} | {c['dp44']} |\n" for c in spec['configs'])
    header+='\n## 공통 입력 24개\n\n| ID | 그룹 | 입력 | 공통 사실과 사건 | 지칭 표현 수 |\n| --- | --- | --- | --- | ---: |\n'
    for s,o in zip(spec['scenarios'],oracle['items']):header+=f"| {s['id']} | {s['group']} / {s['modality']} | {s['text']} | {s['facts']} | {len(o['referents'])} |\n"
    voice=sum(s['modality']=='voice' for s in spec['scenarios'])
    header+=f'\n구성당 음성 입력 {voice}개, 텍스트 입력 {24-voice}개다. 입력 24개, 목표 24개, 지칭 표현 33개, 변경 과제 6개의 분모를 구별한다. U01은 지칭 표현이 없으므로 QA02 분모에 0개를 추가하지만 QA01/QA03/QA07과 비용·RAM에서는 포함한다.\n\n'
    header+='## 고정 시간과 완료 조건\n\n모든 현재 입력 종료는 0초다. 선행 화면 처리는 -12초, 관계 원본 도착은 -8초, 대기 응답은 -2초 또는 -0.5초, 진행 중 의미 step은 -1초, 피드백 정정은 -0.2초다. 후보 응답 속도에 맞춰 다음 사용자 입력을 당기지 않는다. U08/U16/U24의 기존 목표는 -6초의 최초 2초 VIA 처리부터 계산하며 정정마다 새 목표로 초기화하지 않는다.\n\n'
    header+='QA03의 공통 terminal은 요청한 결과의 UI 상태와 필요한 전체 음성 준비, 취소 정산 또는 재개 후반 준비다. 장치에서 첫 chunk가 나오는 시간은 별도 first_voice다. 사용자에게 실제 전달하는 순수 재생과 Agent 업무 duration은 QA03에서 0으로 투영하지만 순서 의존은 유지한다.\n\n'
    header+='## 공통 변경 과제\n\n| 과제 | 동일 요청 |\n| --- | --- |\n'+''.join(f"| {c['change_id']} | {c['title']} |\n" for c in data['changes'] if c['config_id']=='BASE')
    header+='\n원본 [specification.json](specification.json), 평가자 전용 [oracles.json](oracles.json). 정답은 simulator의 후보 출력으로 사용하지 않는다. 24개 입력의 실제 WAV·화면 자산, 후보 실행 adapter와 채택 출력은 미작성이다. 기존 COMMON-28의 실행 모집단과 혼합하지 않는다.\n'
    (directory/'02-configurations-and-scenarios.md').write_text(header)
    doc='# 모든 DP와 모든 방안의 QA 수치\n\n`COMMON-QA-MODEL-4.0` / **분석 모델 값이며 실측 아님** / [시작](README.md) / [계약](01-common-contract.md)\n\n'
    doc+='같은 24개 입력에 대한 전체 시스템 구성의 값이다. A 네 행은 같은 BASE 증거를 다시 보여 주며 네 번의 독립 실행이 아니다. QA02/QA07은 실제 최초 채택 출력이 없어 미산출이다. QA04는 개념 모듈 영향 추정이다. 시간의 가정상 완료 경로가 정확도 성공을 증명하지 않는다.\n\n'
    doc+='| DP | 방안 | QA01 화면 s | QA02 지칭 % | QA03 VIA s | QA04 모듈 개 | QA05 묶음 $ | QA06 최대 MB | QA07 의미 % | 첫 음성 s |\n| --- | --- | ---: | --- | ---: | ---: | ---: | ---: | --- | ---: |\n'
    for dp,ids in [('DP2',['DP2-B']),('DP3',['DP3-B-R1','DP3-B-R2','DP3-B-R3']),('DP5',['DP5-B']),('DP44',['DP44-B'])]:
        for cid in ['BASE',*ids]:
            s=data['summary'][cid];alt='A' if cid=='BASE' else cid[len(dp)+1:]
            values={q:display(s[q],6 if q=='qa05' else 3) for q in ['qa01','qa03','qa04','qa05','qa06','first_voice']}
            doc+=f"| {dp} | {alt} | {values['qa01']} | 미산출 | {values['qa03']} | {values['qa04']} | {values['qa05']} | {values['qa06']} | 미산출 | {values['first_voice']} |\n"
    doc+='\n## 이 가정에서 보이는 차이\n\n- DP2 B는 여유 의미 슬롯이 있는 독립 입력에서 먼저 처리하지만 포화와 필수 결과 의존은 기다린다. 추가 실행 상태와 envelope 비용이 있으며 모듈 수 이점은 만들지 않았다.\n- DP3 B의 로컬 후보 생산은 클라우드 입력을 줄이지만 이 CPU 가정의 UI/SAM 처리와 적재 비용이 크다. R1은 가중치 회수, R2는 상주, R3는 별도 runtime이다. R3에 자동 병렬 속도 이득을 주지 않는다.\n- DP5 B는 준비된 미확정 관계를 재사용하는 입력에서 단축되지만 갱신과 미사용 관계 생산 비용도 낸다. 이미 확정한 사실은 A도 재사용한다. 정확도 이득은 미확인이다.\n- DP44의 화면 QA01은 같다. B는 선택한 발화만 생성하여 불필요한 준비를 줄이지만 다음 발화 선택 후의 생성 대기가 늘 수 있다. 유효한 기존 audio는 양안 모두 재사용한다.\n\n'
    doc+='어느 구성도 실제 제품 적합성이나 최종 선택을 뜻하지 않는다. 서로 다른 B의 감소량을 합쳐 결합 성능으로 주장하지 않는다.\n\n## 사례별 수치 범위\n\n| 구성 | QA01 최소–최대 s | QA03 최소–최대 s | 사례 비용 최소–최대 $ | 사례 peak RAM 최소–최대 MB | 첫 음성 최소–최대 s |\n| --- | ---: | ---: | ---: | ---: | ---: |\n'
    for c in spec['configs']:
        rows=[r for r in data['cases'] if r['config_id']==c['id']];values=[]
        for q in ['qa01','qa03','qa05','qa06','first_voice']:
            digits=6 if q=='qa05' else 3;values.append(display(min(r[q] for r in rows),digits)+'–'+display(max(r[q] for r in rows),digits))
        doc+='| '+c['id']+' | '+' | '.join(values)+' |\n'
    doc+='\n최소/최대는 작성한 24개 사례의 범위다. 실제 오차 범위나 신뢰구간이 아니다. 시간은 평균, QA04는 6과제 평균, QA05는 전체 24사례의 비용 합, QA06은 모든 동시 상태 중 최댓값이다. 표시 소수 자릿수는 재현 편의이며 실측 정밀도를 뜻하지 않는다.\n\n'
    doc+='## 범위의 적절성과 남은 한계\n\n공통 기반 RAM은 560 MB라 기존 512 MB 요구를 이미 48 MB 초과한다. 요구를 맞추려고 값을 낮추거나 기존 요구를 변경하지 않았다. 모든 구성의 제품 RAM 적합성은 미입증이며, 실장 계측과 메모리 설계 수정이 필요하다. DP3의 과거 비교용 RAM cap 예외도 일반 요구 삭제가 아니다.\n\n'
    doc+='기존 QA05의 $2.55는 COMMON-28 묶음의 도출값이다. 새 24사례 합계와 직접 합격/불합격 비교하지 않는다. v4의 단가, 토큰 예산, 클라우드 속도, CPU 처리 시간과 RAM은 공개된 작성 가정이며 실제 견적·측정 또는 정답 출력을 근거로 교정되지 않았다. 여기서는 산식과 누락·중복을 검증했으며 숫자가 실제 하드웨어에 맞는지는 검증하지 않았다.\n\n'
    doc+='QA02 33표현과 QA07 24입력의 평가자 정답은 준비했지만 후보 출력은 0개다. 최초 의미 채택 증거를 수집한 뒤 별도 판본에서 판정해야 한다. 오류 사례는 제외하지 않고 원래 분모에 보존하며, timeout과 미채택 처리는 실행 전에 동결한다. 현재 QA07 목표/점수의 별도 제안과 이 24입력 묶음도 자동 호환하지 않는다.\n\n'
    doc+='[Excel 전체비교](artifacts/outputs/01a15dfe-v4/VIA-QA-common-v4.xlsx) / [HTML 사례 비교](artifacts/outputs/01a15dfe-v4/guide.html) / [원장](artifacts/ledger.json) / [검토](04-review-and-validation.md)\n'
    (directory/'03-results-and-ranges.md').write_text(doc)


def display(value,digits):
    clean=Decimal(str(value)).quantize(Decimal('0.000000000001'),rounding=ROUND_HALF_UP)
    return format(clean.quantize(Decimal(1).scaleb(-digits),rounding=ROUND_HALF_UP),f'.{digits}f')


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,default=OUT/'artifacts');args=parser.parse_args()
    spec=json.loads((OUT/'specification.json').read_text())
    oracle=json.loads((OUT/'oracles.json').read_text())
    assert [s['id'] for s in spec['scenarios']]==[o['id'] for o in oracle['items']]
    cases=[build_case(spec,c,s) for c in spec['configs'] for s in spec['scenarios']]
    change_rows=[v for c in spec['configs'] for v in changes(c)]
    summary={}
    for c in spec['configs']:
        rows=[r for r in cases if r['config_id']==c['id']]
        cs=[r for r in change_rows if r['config_id']==c['id']]
        summary[c['id']]={**{k:mean(r[k] for r in rows) for k in ['qa01','qa03','first_voice']},
                           'qa02':None,'qa04':mean(r['module_count'] for r in cs),'qa05':sum(r['qa05'] for r in rows),
                           'qa06':max(r['qa06'] for r in rows),'qa07':None,
                           'inputs':len(rows),'goals':len(rows),'referents':sum(len(o['referents']) for o in oracle['items']),
                           'candidate_outputs':0,'change_tasks':6,'cost_status':'VIA_ONLY_RECEIVER_NOT_MODELED',
                           'receiver_cost':None,'total_system_cost':None}
    deltas={cid:{q:None if v[q] is None else v[q]-summary['BASE'][q] for q in ['qa01','qa02','qa03','qa04','qa05','qa06','qa07','first_voice']} for cid,v in summary.items()}
    data=dict(version=spec['version'],evidence_kind=spec['evidence_kind'],specification_sha256=digest(OUT/'specification.json'),
              oracles_sha256=digest(OUT/'oracles.json'),generator_sha256=digest(Path(__file__)),configs=spec['configs'],profile=spec['profile'],scenarios=spec['scenarios'],
              policy=spec['policy'],cases=cases,changes=change_rows,summary=summary,deltas=deltas)
    args.output.mkdir(parents=True,exist_ok=True)
    save(args.output/'ledger.json',data);save(args.output/'summary.json',dict(version=spec['version'],summary=summary,deltas=deltas))
    with (args.output/'comparison.csv').open('w',newline='') as f:
        writer=csv.writer(f,lineterminator='\n');writer.writerow(['DP','alternative','config_id',*[f'qa0{i}' for i in range(1,8)],'first_voice','evidence_kind'])
        for dp,alternatives in [('DP2',['DP2-B']),('DP3',['DP3-B-R1','DP3-B-R2','DP3-B-R3']),('DP5',['DP5-B']),('DP44',['DP44-B'])]:
            for cid in ['BASE',*alternatives]:writer.writerow([dp,'A' if cid=='BASE' else cid[len(dp)+1:],cid,*[summary[cid][f'qa0{i}'] for i in range(1,8)],summary[cid]['first_voice'],spec['evidence_kind']])
    document_tables(spec,oracle,data,OUT if args.output==OUT/'artifacts' else args.output/'documents')
    print(json.dumps({'configurations':len(summary),'cases_per_config':len(spec['scenarios']),'summary':summary},ensure_ascii=False,indent=2))


if __name__=='__main__':main()
