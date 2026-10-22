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
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/current/evaluation/common-qa-v5'


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

    def freeze_trace(self, origin):
        """Materialize source-release gaps; causal resource predecessors stay single.

        Every gap has an external source reason; unknown gaps fail closed.
        Full duration replay must reconstruct observed analytical wall time.
        """
        ns={n['id']:n for n in self.nodes}
        for n in list(self.nodes):
            end=max([origin,*[ns[k]['finish_s'] for k in n['causal_parents']]])
            if n['start_s']>end+1e-9:
                reason={'input_end':'user_revision_wait','bg1':'fixed_background_arrival',
                        'bg2':'fixed_background_arrival','question_arrival':'fixed_agent_question_arrival',
                        'obsolete_source':'fixed_report_notice_arrival',
                        'relation_update':'fixed_feedback_arrival'}.get(n['id'])
                assert reason, ('unexplained gap',n['id'],n['start_s'],end)
                key=n['id']+'_source_gap'
                gap=dict(id=key,label=reason,duration_s=n['start_s']-end,deps=list(n['causal_parents']),
                         causal_parents=list(n['causal_parents']),release_s=end,start_s=end,finish_s=n['start_s'],
                         resource=None,kind='external',phase='source_wait',tokens_in=0,tokens_out=0,audio_s=0,state={},
                         resource_predecessor=None,resource_slot=None,resource_wait_s=0)
                self.nodes.append(gap);ns[key]=gap;n['causal_parents']=[key]
        for n in self.nodes:
            n['exclusion_reason']='included_via' if n['kind']=='via' else (
                'pure_device_delivery' if n['kind']=='delivery' else
                'agent_internal_work' if n['id']=='agent_result' else
                'pure_playback' if n['id'].endswith('_play') else 'external_source_or_user_wait')


def union_intervals(intervals, start, end):
    merged=[]
    for a,b in sorted((max(start,a),min(end,b)) for a,b in intervals if b>start and a<end):
        if b<=a:continue
        if merged and a<=merged[-1][1]:merged[-1][1]=max(b,merged[-1][1])
        else:merged.append([a,b])
    return merged


def length(intervals):
    return sum(b-a for a,b in intervals)


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
        prior.append(g.model('original_work','원래 목표의 최초 의미 해석',t['original_goal_input'],t['original_goal_output'],release=-6,phase='prior_goal'))
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
    display = g.add('first_display','첫 구체적 채택 내용 화면 표시',p['display_s'],deps=[adopted],kind='delivery')
    work = adopted
    if s['long_work']:
        work = g.model('long_work','요청의 남은 VIA 판단',t['long_work_input']+(t['dp2_execution_envelope'] if config['dp2']=='B' else 0),t['long_work_output'],deps=[adopted])
    # Required Agent receipt and result reflection are VIA/network work.
    if s['delegate']:
        ingress=g.add('agent_ingress','현재 revision 업무/갱신 전송',p['network_s'],deps=[work])
        receipt=g.add('agent_receipt','Agent 접수 확인 수신',p['receipt_s'],deps=[ingress])
        returned=g.add('agent_result','외부 Agent 업무와 결과 최초 제공',p['agent_work_s'],deps=[receipt],kind='external')
        receive=g.add('result_receive','외부 결과 전송 및 수신',p['network_s'],deps=[returned])
        response_ready=g.add('result_adopt','결과 revision 확인 및 내용 채택',p['adopt_s'],deps=[receive])
    else:
        response_ready=work
    committed=g.add('state_commit','필수 Request/Task 상태 확정',p['state_commit_s'],deps=[response_ready])
    screen=g.add('screen_prepare','최종 상세 내용 전체 준비',p['screen_prepare_s'],deps=[response_ready])
    screen_accept=g.add('screen_accept','상세 화면 전달계층 전체 수용',p['channel_accept_s'],deps=[screen])
    g.add('final_screen_display','실제 최종 화면 표시 진단',p['display_s'],deps=[screen_accept],kind='delivery')

    segments=[]; accepted=[]; playback=[]; event_specs=[]
    origin=-6 if s['revision_of_existing_goal'] else 0
    user_intervals=[[-s['utterance_s'],0]] if s['modality']=='voice' else []
    def segment(key, source, selection, seconds, *, initial=False, job=None, offset=0):
        """One actual semantic content interval, with a streaming prefix milestone."""
        select=g.add(key+'_select','공통 차례 정책에 따른 발화 선택',deps=selection)
        if initial:
            audio_job=source
        elif job:
            audio_job=job
        else:
            audio_job=g.speech(key+'_audio','전체 '+key+' 음성 생성',seconds,
                               deps=[source]+([select] if config['dp44']=='B' else []))
        ns=g.run()
        if initial:
            prefix=audio_job
        else:
            a=next(a for a in g.audio if a['node']==audio_job);n=ns[audio_job]
            duration=a['setup_s']+(offset+min(seconds,p['first_chunk_audio_s']))/p['audio_generation_rate']
            assert duration<=n['duration_s']+1e-8,(key,duration,n['duration_s'])
            prefix=g.add(key+'_chunk','같은 생성 작업의 실질 발화 구간 첫 chunk',duration,
                         deps=n['causal_parents'],release=n['release_s'])
            g.nodes[-1]['observes_prefix_of']=audio_job
        device=g.add(key+'_output','실질 발화 구간의 실제 장치 출력',p['device_s'],deps=[select,prefix],kind='delivery')
        play=g.add(key+'_play','순수 재생 및 다음 차례',seconds,deps=[device],kind='external')
        accept=g.add(key+'_accept','전체 음성의 전달계층 수용',p['channel_accept_s'],deps=[audio_job])
        accepted.append(accept);playback.append(play)
        row=dict(id=key,response_revision=s['id']+':'+key+':r1',source=source,selection=select,
                 audio_job=audio_job,output=device,play=play,accept=accept,audio_s=seconds,offset_audio_s=offset)
        segments.append(row)
        return row

    if s.get('retained_prior_audio_s'):
        g.add('retained_prior_audio','공통 초기 R0 유효 buffer 보존',kind='external',release=-12)
        g.audio.append(dict(node='retained_prior_audio',length_s=s['retained_prior_audio_s'],setup_s=0,discard_at=None,initial=True))
    question=None
    if s['delivery'] in ('queued','cancel','resume'):
        source=g.add('question_arrival','Agent 질문 외부 최초 제공',kind='external',release=-.5 if s['delivery']=='resume' else -2)
        receive_question=g.add('question_receive','Agent 질문의 VIA 수신',p['network_s'],deps=[source])
        question_adopt=g.add('question_adopt','질문 내용과 revision 채택',p['adopt_s'],deps=[receive_question])
        # U23 Text does not barge in. Autonomous notice can precede the input.
        question=segment('question',question_adopt,[question_adopt] if s['id']=='U23' else [adopted,question_adopt],s['audio_s'])
        event_specs.append(dict(id=s['id']+':question',kind='AGENT_QUESTION',source=source,
                                profile=s['agent_question_class'],segment='question',adoption=question_adopt,receive_node=receive_question))
    if s['delivery']=='cancel':
        g.add('obsolete_source','보고서 음성 안내의 최초 제공',kind='external',release=-2)
        g.add('obsolete_receive','보고서 안내의 VIA 수신',p['network_s'],deps=['obsolete_source'])
        g.add('obsolete_adopt','보고서 안내 내용 채택',p['adopt_s'],deps=['obsolete_receive'])
        if config['dp44']=='A':
            old=g.speech('obsolete_audio','철회 안내의 선행 생성',10,deps=['obsolete_adopt'],phase='preparation')
            ns=g.run();n=ns[old];a=next(a for a in g.audio if a['node']==old)
            stop=min(n['finish_s'],ns[adopted]['finish_s']+p['cancel_tail_s'])
            n['duration_s']=stop-n['start_s'];n['audio_s']=max(0,(n['duration_s']-a['setup_s'])*p['audio_generation_rate'])
            a['length_s']=n['audio_s'];a['discard_at']=ns[adopted]['finish_s']
            accepted.append(g.add('cancel_settled','보고서 음성 철회 정산',deps=[adopted,old]))
        else:accepted.append(g.add('cancel_settled','미선택 보고서 음성 의무 철회',deps=[adopted]))
        event_specs.append(dict(id=s['id']+':withdrawn',kind='AGENT_NOTICE',source='obsolete_source',
                                profile='WAIT_USER',segment=None,adoption='obsolete_adopt',receive_node='obsolete_receive',status='WITHDRAWN_BEFORE_AUDIO'))
    if s['delivery']=='resume':
        old=g.speech('old_audio','선행 R0 전체 음성 생성',36,release=-4,phase='preparation')
        ns=g.run();n=ns[old];a=next(a for a in g.audio if a['node']==old)
        old_start=n['start_s']+a['setup_s']+p['first_chunk_audio_s']/p['audio_generation_rate']+p['device_s']
        old_played=-s['utterance_s']-old_start
        assert 0<old_played<36
        if config['dp44']=='B':
            stop=min(n['finish_s'],ns[adopted]['finish_s']+p['cancel_tail_s'])
            n['duration_s']=stop-n['start_s'];n['audio_s']=(n['duration_s']-a['setup_s'])*p['audio_generation_rate'];a['length_s']=n['audio_s']
        g.add('prior_play','선행 R0 실제 재생 후 사용자 발화로 중단',old_played,release=old_start,kind='external')
        # Valid buffered remaining audio is delivered immediately at the new turn.
        resume=segment('resume','old_audio',[question['play'],adopted],36-old_played,job=old,offset=old_played)
        if a['length_s']<36:
            tail=g.speech('resume_tail','보존 buffer 뒤 미생성 후반',36-a['length_s'],deps=[resume['selection'],old],tin_extra=120)
            accepted.append(g.add('resume_tail_accept','재개 후반 전체 전달 수용',p['channel_accept_s'],deps=[tail]))
        # Old R0 is initial state, not a new event with an invented source time.
    if s['delivery']=='cached':
        source=g.add('existing_audio','공통 유효 R0 잔여 audio 초기 상태',kind='external',release=-12)
        g.audio.append(dict(node=source,length_s=s['audio_s'],setup_s=0,discard_at=None,initial=True))
        first=segment('resume',source,[adopted],s['audio_s'],initial=True)
    elif question and s['id']!='U23':
        first=question
    elif s['delegate'] or s['long_work']:
        first=segment('intent',adopted,[adopted]+([question['play']] if question else []),p['intent_audio_s'])
    else:
        first=segment('answer',response_ready,[adopted],s['audio_s'])
    event_specs.insert(0,dict(id=s['id']+':user',kind='USER_INPUT',source='input_end',
                            profile=s['user_voice_class'],segment=first['id'],adoption=adopted))
    if s['delegate'] or s['long_work']:
        final=segment('final',response_ready,[response_ready,first['play']],6 if question else s['audio_s'])
        if s['delegate']:
            event_specs.append(dict(id=s['id']+':result',kind='AGENT_RESULT',source='agent_result',
                                    profile=s['agent_result_class'],segment='final',adoption=response_ready,receive_node='result_receive'))
    terminal=g.add('terminal','全 필수 상태와 화면/음성 준비 수용 완료',deps=[committed,screen_accept,*accepted])
    nodes=g.run()
    # Preserve exact onset plus epistemic status; no acoustic samples exist.
    events=[]
    all_play=[[nodes[k]['start_s'],nodes[k]['finish_s']] for k in playback]
    for e in event_specs:
        start=nodes[e['source']]['finish_s'];e.update(source_s=start,goal_id=s['id'],obligation=True,
            evidence='ANALYTICAL_ASSUMED_CONTENT',content_error='UNKNOWN',physical_samples=0,proxy_samples=0)
        e['received_s']=nodes[e['receive_node']]['finish_s'] if e.get('receive_node') else start
        if e.get('status')=='WITHDRAWN_BEFORE_AUDIO':
            e.update(first_audio_s=None,L=None,U=None,P=None,overlap=None,B=None,E=None,R=None,D=None,
                     prepared_during_wait_s=None,normal_collision=False,withdrawal_s=nodes[adopted]['finish_s'])
        else:
            seg=next(seg for seg in segments if seg['id']==e['segment']);v=nodes[seg['output']]['finish_s']
            assert v>=start,(s['id'],e['id'],v,start)
            u=union_intervals(user_intervals,start,v)
            pi=union_intervals([[nodes[k]['start_s'],nodes[k]['finish_s']] for k in playback if k!=seg['play']],start,v)
            both=union_intervals(u+pi,start,v);b=length(both);l=v-start
            adoption=nodes[e['adoption']]['finish_s'];a=max(start,adoption)
            nj=nodes[seg['audio_job']]
            prep=length(union_intervals(both,nj['start_s'],nj['finish_s']))
            e.update(status='ANALYTICAL_ONSET',first_audio_s=v,L=l,U=length(u),P=length(pi),
                     overlap=length(u)+length(pi)-b,B=b,E=l-b,R=v-(both[-1][1] if both else start),
                     D=max(0,v-a)-length(union_intervals(both,a,v)),prepared_during_wait_s=prep,
                     normal_collision=e['profile']=='NORMAL' and b>1e-9,user_intervals=u,previous_intervals=pi,
                     blocked_intervals=both,response_revision=seg['response_revision'],segment_id=seg['id'])
        events.append(e)
    first_voice=next(e['L'] for e in events if e['kind']=='USER_INPUT')
    g.freeze_trace(origin)
    nodes={n['id']:n for n in g.nodes}
    def ancestors(key):
        answer={key}
        for k in nodes[key]['causal_parents']:answer.update(ancestors(k))
        return answer
    shared=ancestors(terminal)&ancestors(first['output'])
    shared_via=[nodes[k] for k in shared if nodes[k]['kind']=='via' and nodes[k]['duration_s']>0 and nodes[k]['finish_s']>origin]
    overlap_diagnostic=dict(user_first_audio_s=first_voice,
        residual_via_path_s=g.projected(terminal,first_voice),
        largest_shared_stage=max(shared_via,key=lambda n:n['duration_s'])['id'] if shared_via else None,
        shared_stages=sorted(n['id'] for n in shared_via),definition='first audio 이후 남은 DAG만 투영한 진단. QA03-QA01이 아니며 새 QA/점수 아님')
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
        state('visual_retained_state',p['visual_state_mb'],first_start,end)
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
    added_owners={q['owner'] for q in states if q['owner'].startswith(('weight:','runtime_'))
                  or q['owner'] in ('vision_ipc','visual_retained_state')
                  or q['owner'].endswith(('_workspace','_load_temp'))}
    for q in states:
        q['qa08_attributable']=q['owner'] in added_owners
        q['physical_resource']='host_ram';q['shared_physical_count']=1
    added_samples=[dict(time_s=m['time_s'],host_mb=sum(v for k,v in m['components_mb'].items() if k in added_owners),
                        dedicated_accelerator_mb=0,components_mb={k:v for k,v in m['components_mb'].items() if k in added_owners}) for m in samples]
    added_peak=max(added_samples,key=lambda m:m['host_mb'])
    usage=[]
    for n in g.nodes:
        for bucket,amount,rate in [('text_input',n['tokens_in'],p['input_usd_per_token']),('text_output',n['tokens_out'],p['output_usd_per_token']),('audio_output',n['audio_s'],p['audio_output_usd_per_s'])]:
            if amount:usage.append(dict(node=n['id'],phase=n['phase'],bucket=bucket,amount=amount,rate_usd=rate,usd=amount*rate))
    if s['modality']=='voice':usage.append(dict(node='input_end',phase='input',bucket='audio_input',amount=s['utterance_s'],rate_usd=p['audio_input_usd_per_s'],usd=s['utterance_s']*p['audio_input_usd_per_s']))
    return dict(config_id=config['id'],case_id=s['id'],group=s['group'],modality=s['modality'],
                qa01=first_voice,qa02=None,qa03=g.projected(terminal,origin),
                qa05=sum(u['usd'] for u in usage),qa06=peak['total_mb'],qa07=None,qa08=added_peak['host_mb'],
                screen_diagnostic=nodes[display]['finish_s'],first_voice=first_voice,
                goal_origin_s=origin,goal_wall_s=nodes[terminal]['finish_s']-origin,
                first_display_node=display,meaning_node=adopted,terminal_node=terminal,first_audio_node=first['audio_job'],
                status='CONDITIONAL_NORMAL_PATH_SIMULATION',accuracy_status='NO_CANDIDATE_OUTPUT',
                nodes=g.nodes,usage=usage,memory_intervals=states,audio=g.audio,memory_peak=peak,
                qa08_peak=added_peak,events=events,segments=segments,timing_overlap=overlap_diagnostic,required_state_nodes=[committed],
                required_external_nodes=(['agent_receipt','result_receive'] if s['delegate'] else [])+(['question_receive'] if question else [])+(['obsolete_receive'] if s['delivery']=='cancel' else []),
                required_channel_nodes=[screen_accept,*accepted],routing=s['routing'])


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


def score(value, bounds):
    if value is None:return None
    assert value>=0
    return next((6-i for i,limit in enumerate(bounds) if value<=limit),0)


def display(value,digits=4):
    if value is None:return '미산출'
    clean=Decimal(str(value)).quantize(Decimal('0.000000000001'),rounding=ROUND_HALF_UP)
    return format(clean.quantize(Decimal(1).scaleb(-digits),rounding=ROUND_HALF_UP),f'.{digits}f')


ALTERNATIVES=[('DP2',['DP2-B']),('DP3',['DP3-B-R1','DP3-B-R2','DP3-B-R3']),('DP5',['DP5-B']),('DP44',['DP44-B'])]


def document_tables(spec,data,directory):
    directory.mkdir(parents=True,exist_ok=True)
    doc='# 공통 입력, 책임과 음성 사건\n\n[시작](README.md) / [공통 계약](01-common-contract.md) / [원본](specification.json)\n\n'
    doc+='24입력 × 7구성, 24목표, 33지칭 표현, 6변경 과제를 유지한다. 아래 역할은 공통 시험 조건이며 자동 routing 모델의 판단 결과가 아니다. Voice ON에서 입력별 응답 의무를 등록했다. 화면과 음성의 준비가 모두 필요하며 Agent 업무는 결과까지 확인한다.\n\n'
    doc+='| 입력 | 사용자 요청 | 처리 | VIA 책임 | Agent 책임 |\n| --- | --- | --- | --- | --- |\n'
    for s in spec['scenarios']:doc+=f"| {s['id']} | {s['text']} | {s['routing']} | {s['via_work']} | {s['agent_work']} |\n"
    doc+='\nU03/06/07은 제공된 결론의 설명, U08은 VIA 설명의 범위 정정이다. U09~12/21/22/24는 비교 산출물 업무를 위임하도록 고정했다. 비교라는 표현만으로 언제나 위임해야 한다는 제품 정책이 아니다. U16/24는 기존 Agent 업무의 revision 갱신 접수와 결과 반영이 필요하다. U22/23의 질문 읽기는 VIA가 담당한다.\n\n'
    doc+='## 사건과 사전 분류\n\n| 사건 | 입력 ID | 분류 | 구성당 예정 수 |\n| --- | --- | --- | ---: |\n'
    groups={}
    for e in [e for c in data['cases'] if c['config_id']=='BASE' for e in c['events']]:groups.setdefault((e['kind'],e['profile']),[]).append(e['id'])
    for (kind,profile),ids in groups.items():doc+=f"| {kind} | {', '.join(ids)} | {profile} | {len(ids)} |\n"
    doc+='\n42예정 = 사용자24 + Agent 결과12 + Agent 질문5 + 철회 안내1. NORMAL 36, WAIT 6예정 중 5출력·1정상철회이며, 실제 관측 수는 0이다. WAIT_BOTH는 이 묶음에 없으며 시험 커버리지의 한계다. 사용자 입력24를 QA01 분모로 바로 사용하지 않는다. 오류 여부는 내용 출력이 없어 UNKNOWN이다.\n\n'
    doc+='U17/18/19/22의 질문 읽기는 사용자 요청과 Agent 질문을 동시에 충족하는 동일 내용 구간으로 연결한다. 두 사건의 시작 시각은 다르다. U23은 -2초 자동 질문 알림이 Text 제출 전 시작할 수 있다. 새 입력 안내는 질문 이후 별도 구간이며 같은 시작 시각을 복사하지 않는다. U18 보고서 안내는 사용자 발화로 출력이 막힌 동안 접수된 뒤 첫 출력 전에 철회한다. 업무 취소가 아니다.\n\n'
    doc+='## 선행 조건과 변경 이유\n\nU19의 v4 생성 -2초와 사용자 발화 시작 -3초는 중단 순서에 맞지 않았다. v5는 양안 모두 생성 -4초, 발화에 의한 중단 -3초로 수정한다. 그 전에 실제 재생한 prefix를 재개 구간에서 뺀다. U20은 잔여12초 buffer를 공통 초기 상태로 사용한다. 관측창 이전에 처리한 R0 등은 초기 상태이며 새 Agent 사건으로 만들지 않는다. 관측창 안 사건의 source 시각 누락은 제외가 아니라 UNOBSERVABLE이다.\n\n'
    doc+='U08/16/24는 최초 목표 -6초부터 계산한다. 정정으로 시작을 재설정하지 않는다. 배경 업무는 이번 관측창에 새 결과를 제공하지 않는 조건이다. Agent 결과는 해당 요청 접수 후 공통8초 업무가 끝나는 인과 사건이라 절대 도착 시각은 VIA 준비에 따라 달라진다. 사용자 입력 일정과 외부 업무 길이를 후보 결과에 맞춰 바꾸지 않는다.\n\n'
    doc+='7구성은 BASE(A/A/A/A), DP2-B, DP3-B-R1/R2/R3, DP5-B, DP44-B다. 한 DP만 바꾸며 네 A행은 같은 BASE의 재표시다.\n'
    (directory/'02-configurations-and-scenarios.md').write_text(doc)
    doc='# 모든 DP와 모든 방안의 QA 수치\n\n**분석 가정의 계산값과 조건부 점수이며 실측이 아니다.** [계약](01-common-contract.md) / [사례](02-configurations-and-scenarios.md)\n\n'
    doc+='| DP | 방안 | QA01 NORMAL 초 | 점수 /6 | WAIT 총 초 | WAIT B 초 | WAIT R 초 | QA03 초 | 점수 /6 | QA04 개 | QA05 $ | QA06 MB | QA08 MB |\n| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n'
    for dp,ids in ALTERNATIVES:
        for cid in ['BASE',*ids]:
            s=data['summary'][cid];values=[display(s['qa01']),str(s['qa01_score']),display(s['wait']['L']),display(s['wait']['B']),display(s['wait']['R']),display(s['qa03']),str(s['qa03_score']),display(s['qa04']),display(s['qa05'],6),display(s['qa06']),display(s['qa08'])]
            doc+='| '+dp+' | '+('A' if cid=='BASE' else cid[len(dp)+1:])+' | '+' | '.join(values)+' |\n'
    doc+='\nQA01은 36 NORMAL 사건 평균, WAIT는 5출력 사건 평균(별점 없음), QA03은 24목표 평균이다. QA02/QA07은 후보 출력이 없어 모든 구성에서 미산출이다. QA04는 6과제의 개념 모듈 영향 추정, QA05는 24사례 VIA 비용 합, QA06/08은 각 소유 범위의 동일 시각 합계 최고량이다. 수신 Agent 비용과 전체 시스템 비용은 미산출이다.\n\n'
    doc+='## 대기 종류별 결과\n\n| 구성 | 분류 | 예정/출력/철회 | L 초 | U 초 | P 초 | 교집합 초 | B 초 | E 초 | R 초 | D 초 | 준비 겹침 초 |\n| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n'
    for cid,s in data['summary'].items():
        for group,w in s['wait_profiles'].items():doc+='| '+cid+' | '+group+f" | {w['scheduled']}/{w['onsets']}/{w['withdrawn']} | "+' | '.join(display(w[k]) for k in ['L','U','P','overlap','B','E','R','D','prepared_during_wait_s'])+' |\n'
    doc+='\nWAIT_BOTH는 예정0으로 미산출이며 정상 평균에 섞지 않는다. 내부 큐와 모델 대기는 NORMAL에서도 포함한다. NORMAL 중 후보가 만든 차례 충돌도 분모를 유지하며 원장의 normal_collision으로 표시한다.\n\n'
    doc+='## 수치의 해석과 한계\n\nDP2는 실행 슬롯과 의존 관계, DP3는 CPU 후보 생산·적재·회수, DP5는 사전 관계 생산과 재사용, DP44는 선행 음성 준비와 선택 후 생성의 차이를 계산한다. 원하는 차이나 점수를 만들기 위해 속도·사례 비중을 조정하지 않았다. QA01/03의 공통 병목은 그대로 둔다.\n\n'
    doc+='QA01 2초는 4초 참조 대비50% 단축이라는 사용자 합의 목표이며 연구의 달성 성과가 아니다. QA03은 Windows Long wait 상단을 채택한 VIA 처리 예산5초다. 원값 평균에 합의한1초 우수선과 균등 구간을 적용했다. [QA01 근거](../../qa/research-notes/qa01-voice-target-rationale.md), [QA03 근거](../../qa/research-notes/qa03-completion-target-rationale.md).\n\n'
    doc+='QA08은 추가 로컬 시각 모델 runtime, weights, workspace, load temporary, IPC, 보존 상태의 최고 동시 합이다. v4가 별도 명시하지 않은 모델 전용 프레임/추적 상태32MB를 공개 가정으로 추가해 QA06에도 포함했다. 입력/출력 tensor와 추론 임시는 workspace, 전달 복사본은 IPC에 포함한다. 기본 UI/ASR은 모든 구성 공통이고 의미·음성 모델은 원격이라는 범위에서만 비추가 구성의0MB가 성립한다. CPU 외 전용 가속기 메모리는0이다.\n\n'
    doc+='QA08 <1000MB는 요구이며 여기의 추정은 제품 합격 판정이 아니다. 최대 지원 부하·연속운영·5회 실행은 미수행이다. QA06 기반560MB는 기존512MB보다 크며 요구를 낮추지 않았다. QA05의 과거 COMMON-28 $2.55 목표는 이24사례 합계와 직접 비교하지 않는다. QA02/07/08에 미관측을0점으로 넣지 않는다. 실제 WAV·화면 자산과 실행 adapter 및 채택 출력은 미작성이다.\n\n'
    doc+='## 첫 응답과 완료가 공유하는 경로\n\n기준 구성의 모든24목표를 함께 본다. 사용자 사건 첫 음성 이후 남은 DAG만 별도로 투영한 진단이며 QA03−QA01이나 새 점수가 아니다. QA01 대표값은 Agent 사건도 포함하므로 이 표의 사용자 사건 열 평균과 다르다. U08/16/24의 목표는 -6초, 사용자 정정 사건은0초에서 시작한다.\n\n| 입력 | 사용자 첫 음성 초 | QA03 초 | 첫 음성 후 남은 VIA 경로 초 | 공통 경로의 가장 긴 단계 |\n| --- | ---: | ---: | ---: | --- |\n'
    for c in data['cases']:
        if c['config_id']=='BASE':doc+=f"| {c['case_id']} | {display(c['first_voice'])} | {display(c['qa03'])} | {display(c['timing_overlap']['residual_via_path_s'])} | {c['timing_overlap']['largest_shared_stage']} |\n"
    doc+='\n간단한 자체 설명은 같은 의미 확정과 음성 생성 경로를 공유하며, 별도 효과라고 두 번 주장하지 않는다. 위임12목표는 Agent 결과의 VIA 반영과 최종 채널 준비가 추가되고 외부8초 업무는 QA03에서 제외한다. 장문 설명·재개는 남은 음성 준비가 중요하다. 모든 구성의 같은 진단은 원장과 HTML 사례 상세에 남겼다. 이24입력은 실사용 빈도가 아니라 설계 커버리지이므로 실제 QA 선정은 대표 사용 비중과 관측된 병목을 추가 확인해야 한다.\n\n'
    doc+='[Excel](artifacts/outputs/01a15dfe-v5/VIA-QA-common-v5.xlsx) / [HTML](artifacts/outputs/01a15dfe-v5/guide.html) / [원장](artifacts/ledger.json) / [검증](04-review-and-validation.md)\n'
    (directory/'03-results-and-ranges.md').write_text(doc)


def group_summary(events):
    onsets=[e for e in events if e['L'] is not None]
    return dict(scheduled=len(events),onsets=len(onsets),withdrawn=sum(e['status']=='WITHDRAWN_BEFORE_AUDIO' for e in events),
                missing=sum(e['status'] not in ('ANALYTICAL_ONSET','WITHDRAWN_BEFORE_AUDIO') for e in events),
                **{k:mean(e[k] for e in onsets) if onsets else None for k in ['L','U','P','overlap','B','E','R','D','prepared_during_wait_s']})


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,default=OUT/'artifacts');args=parser.parse_args()
    spec=json.loads((OUT/'specification.json').read_text());oracle=json.loads((OUT/'oracles.json').read_text())
    cases=[build_case(spec,c,s) for c in spec['configs'] for s in spec['scenarios']]
    change_rows=[v for c in spec['configs'] for v in changes(c)];summary={}
    for cfg in spec['configs']:
        cid=cfg['id'];rows=[c for c in cases if c['config_id']==cid];events=[e for c in rows for e in c['events']]
        normal=group_summary([e for e in events if e['profile']=='NORMAL'])
        wait=group_summary([e for e in events if e['profile']!='NORMAL'])
        qa03=mean(c['qa03'] for c in rows)
        summary[cid]=dict(qa01=normal['L'],qa01_score=score(normal['L'],spec['scores']['qa01_upper_bounds']),
            qa02=None,qa03=qa03,qa03_score=score(qa03,spec['scores']['qa03_upper_bounds']),
            qa04=mean(c['module_count'] for c in change_rows if c['config_id']==cid),qa05=sum(c['qa05'] for c in rows),
            qa06=max(c['qa06'] for c in rows),qa07=None,qa08=max(c['qa08'] for c in rows),qa08_score=None,
            normal=normal,wait=wait,wait_profiles={k:group_summary([e for e in events if e['profile']==k]) for k in ['WAIT_USER','WAIT_PREVIOUS','WAIT_BOTH']},
            normal_collisions=sum(e['normal_collision'] for e in events),scheduled_events=len(events),
            goals=24,inputs=24,referents=33,change_tasks=6,physical_onsets=0,proxy_onsets=0,candidate_outputs=0,
            content_errors=None,goal_errors=None,receiver_cost=None,total_system_cost=None,
            score_evidence='ANALYTICAL_CONDITIONAL_NOT_MEASURED',product_compliance='NOT_VERIFIED')
    deltas={cid:{q:None if v[q] is None else v[q]-summary['BASE'][q] for q in [f'qa0{i}' for i in range(1,9)]} for cid,v in summary.items()}
    data=dict(version=spec['version'],evidence_kind=spec['evidence_kind'],contracts=spec['contracts'],
              specification_sha256=digest(OUT/'specification.json'),oracles_sha256=digest(OUT/'oracles.json'),
              generator_sha256=digest(Path(__file__)),configs=spec['configs'],profile=spec['profile'],
              scenarios=spec['scenarios'],policy=spec['policy'],cases=cases,changes=change_rows,summary=summary,deltas=deltas)
    args.output.mkdir(parents=True,exist_ok=True);save(args.output/'ledger.json',data)
    save(args.output/'summary.json',dict(version=spec['version'],contracts=spec['contracts'],summary=summary,deltas=deltas))
    with (args.output/'comparison.csv').open('w',newline='') as f:
        keys=['qa01','qa01_score','qa02','qa03','qa03_score','qa04','qa05','qa06','qa07','qa08']
        writer=csv.writer(f,lineterminator='\n');writer.writerow(['DP','alternative','config_id',*keys,'wait_L','wait_B','wait_R','evidence_kind'])
        for dp,ids in ALTERNATIVES:
            for cid in ['BASE',*ids]:
                s=summary[cid];writer.writerow([dp,'A' if cid=='BASE' else cid[len(dp)+1:],cid,*[s[k] for k in keys],*[s['wait'][k] for k in ['L','B','R']],spec['evidence_kind']])
    document_tables(spec,data,OUT if args.output==OUT/'artifacts' else args.output/'documents')
    print(json.dumps({cid:{k:v[k] for k in ['qa01','qa01_score','qa03','qa03_score','qa08','scheduled_events','normal_collisions']} for cid,v in summary.items()},ensure_ascii=False,indent=2))


if __name__=='__main__':main()
