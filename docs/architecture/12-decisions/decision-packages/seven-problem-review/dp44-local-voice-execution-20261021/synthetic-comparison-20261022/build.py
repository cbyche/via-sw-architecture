"""Offline authored event ledger. No model, microphone, product or network execution."""
from pathlib import Path
import csv, json, copy

HERE = Path(__file__).resolve().parent
ROOT = next(p for p in HERE.parents if (p / 'INTENT.md').exists())
OUT = ROOT / 'docs/presentations_files/dp44-synthetic-backdata-20261022'
VERSION = 'DP44-DELIVERY-SYN-1.0'
KIND = 'SYNTHETIC_AUTHORED_NOT_MEASURED'
ASSUMPTIONS = dict(profile='VOICE-DELIVERY-CLOUD-SYN-1.0', base_ram_mb=560,
    pcm_bytes_per_second=48000, job_metadata_bytes=4096, generation_rate=4,
    first_audio_seconds=0.20, first_latency_base=0.38, text_input_rate=1/1e6,
    text_output_rate=4/1e6, audio_input_rate=0.001, audio_output_rate=0.002,
    output_device_seconds=0.04, cancel_tail_seconds=0.10, interruption_seconds=0.08,
    output_generation_slots=4, audio_ram_cap_mb=16, local_model_weights_mb=0,
    semantic_model='fixed common semantic fixture; no model inference',
    rates_status='authored hypothetical rates, same as common v2 where applicable')
FAMILIES = [
 ('S01','즉시 선택되는 새 응답','현재 보고서 처리 상태를 짧게 읽어줘.','보고서 요약은 완료됐고, 검토가 필요합니다.', 'idle'),
 ('S02','유효한 질문의 선행 준비','대기 중인 메일 질문부터 읽어줘.','수신자는 김 대리인가요?', 'ready'),
 ('S03','보고서 정정과 메일 질문 보존','보고서는 6월 기준으로 다시 해줘. 메일 질문부터 들려줘.','수신자는 김 대리인가요?', 'revision'),
 ('S04','대기 음성 철회','그 보고서 음성 안내는 취소하고, 취소했다고만 알려줘.','보고서 음성 안내를 취소했습니다. 화면 결과는 유지합니다.', 'withdraw'),
 ('S05','긴 설명의 중단과 복귀','메일 질문부터 읽고, 이 설명은 멈춘 곳부터 계속해.','수신자는 김 대리인가요?', 'resume'),
 ('S06','완성 음성 재사용과 짧은 오감지','그 설명을 멈춘 곳부터 이어서 읽어줘.','보고서의 다음 항목은 비용 비교입니다.', 'false_trigger'),
 ('S07','여러 Agent 결과의 순서 지정','일정, 메일, 보고서 순서로 읽어줘.','내일 회의는 오후 세 시입니다.', 'burst'),
 ('S08','선행 준비 여유가 거의 없음','도착하는 메일 질문을 바로 읽어줘.','수신자는 김 대리인가요?', 'late'),
]
DATA = dict(version=VERSION,evidence_kind=KIND,created_on='2026-10-22',actual_model_calls=0,
    actual_product_runs=0,assumptions=ASSUMPTIONS,families=[],inputs=[],sources=[],nodes=[],
    responses=[],goals=[],usage=[],jobs=[],playback=[],memory=[],changes=[],history=[],deliveries=[])

def r(v): return round(v,9)

class Episode:
    def __init__(self, family, idx, option):
        self.fid,self.title,self.utterance,self.first_text,self.kind=family
        self.idx=idx; self.option=option; self.eid=f'{self.fid}-{idx:02d}'; self.prefix=f'{self.eid}:{option}'
        self.nodes={}; self.jobs=[]; self.plays=[]; self.usage=[]; self.hist=[]; self.sources=[]
        self.jitter=(idx-5.5)*0.012
        self.root=self.node('origin',[],0,False,at=-6)
        self.origin=self.node('origin_interpretation',[self.root],.84,True)
        self.dispatch=self.node('original_handoff',[self.origin],.12,True)
        self.cost('origin_semantic',self.origin,'semantic',600,16,0,0,'setup')
        self.cost('origin_speech_input',self.root,'speech_input',0,0,2,0,'setup')
        # Same input acquisition and semantic execution. Voice ends at 0 in every case.
        self.end=self.node('evaluation_utterance_end',[self.root],6,False)
        self.final=self.node('final_transcript',[self.end],.30,True)
        self.semantic=self.node('meaning_adoption',[self.final,self.origin],.20+(600+idx*20)/6000+(16+2*(idx%3))/80,True)
        self.commit=self.node('apply_relation',[self.semantic],.04,True)
        self.h=self.nodes[self.commit]['end_s']
        self.ui=self.node('evaluation_first_display',[self.commit],.04,True)
        self.cost('input_semantic',self.semantic,'semantic',600+idx*20,16+2*(idx%3),0,0,'evaluation')
        self.speech_length={'revision':4.4,'resume':3.6,'burst':2.6,'ready':2.3,'withdraw':4.4,'idle':3.0}.get(self.kind,2.0)
        self.cost('input_audio',self.final,'speech_input',0,0,self.speech_length,0,'evaluation')

    def node(self,name,parents,duration,via,at=None):
        nid=self.prefix+':'+name
        assert nid not in self.nodes, nid
        start=max((self.nodes[p]['end_s'] for p in parents),default=(at or 0))
        if at is not None: start=at
        self.nodes[nid]=dict(id=nid,episode=self.eid,option=self.option,name=name,parents=parents,
            start_s=r(start),duration_s=r(duration),end_s=r(start+duration),via=via)
        return nid

    def release(self,name,at,parents=None):
        parents=parents or [self.dispatch]
        current=max(self.nodes[p]['end_s'] for p in parents)
        assert at>=current-1e-8,(name,at,current)
        return self.node(name,parents,at-current,False)

    def cost(self,name,node,role,tin,tout,ain,aout,phase):
        self.usage.append(dict(id=self.prefix+':usage:'+name,episode=self.eid,option=self.option,node_id=node,
            role=role,phase=phase,text_input=tin,text_output=tout,audio_input_s=r(ain),audio_output_s=r(aout),
            input_rate=ASSUMPTIONS['text_input_rate'],output_rate=ASSUMPTIONS['text_output_rate'],
            audio_input_rate=ASSUMPTIONS['audio_input_rate'],audio_output_rate=ASSUMPTIONS['audio_output_rate'],
            amount_usd=r(tin/1e6+tout*4/1e6+ain*.001+aout*.002)))

    def source(self,name,at,text,seconds,parents=None,revision=1):
        nid=self.release('source_'+name,at,parents)
        display=self.node('display_'+name,[nid],.04,True)
        s=dict(id=self.eid+':'+name,node_id=nid,display_node=display,available_s=r(at),text=text,
            spoken_seconds=r(seconds),revision=revision,owner='Downstream Agent',input_tokens=100+len(text)*2)
        self.sources.append(s)
        self.hist.append(dict(episode=self.eid,option=self.option,event='display',response_id=s['id'],at_s=self.nodes[display]['end_s'],extent='full text',claims_heard=False))
        return s

    def audio(self,name,source,parent,total=None,cancel=None,discard=False,token_extra=0):
        total=source['spoken_seconds'] if total is None else total
        tin=source['input_tokens']+token_extra
        first=.38+tin/6000
        start=self.nodes[parent]['end_s']
        end=start+first+max(0,total-.2)/4
        if cancel is not None: end=min(end,cancel+.10)
        assert end>=start+first, (name,'cancel before first not in this corpus')
        n1=self.node(name+'_first',[parent],first,True)
        nf=self.node(name+'_tail',[n1],end-self.nodes[n1]['end_s'],True)
        produced=min(total,.2+max(0,end-self.nodes[n1]['end_s'])*4)
        job=dict(id=self.prefix+':job:'+name,episode=self.eid,option=self.option,source_id=source['id'],
            revision=source['revision'],first_node=n1,full_node=nf,start_s=r(start),first_s=self.nodes[n1]['end_s'],
            end_s=r(end),requested_audio_s=r(total),generated_audio_s=r(produced),discarded=discard,
            cancel_requested_s=cancel,status='cancelled' if produced<total-1e-7 else 'completed',
            release_s=None,input_tokens=tin)
        self.jobs.append(job)
        self.cost(name,nf,'speech_output',tin,0,0,produced,'discarded' if discard else 'retained')
        if discard:
            job['release_s']=r(end)
            self.hist.append(dict(episode=self.eid,option=self.option,event='invalidate',response_id=source['id'],at_s=cancel,extent='old revision; never audible',claims_heard=False))
        return job

    def play(self,name,job,gate,offset=0,seconds=None,false_pause=False):
        length=job['generated_audio_s']-offset if seconds is None else seconds
        n=self.node(name+'_device',[gate,job['first_node']],.04,True)
        start=self.nodes[n]['end_s']
        if false_pause:
            p1=self.node(name+'_part1',[n],.60,False)
            # Acoustic onset at playback +.52, 80 ms local stop, then .30 s false-trigger hold.
            hold=self.node(name+'_false_trigger_hold',[p1],.30,False)
            end=self.node(name+'_part2',[hold],length-.60,False)
            spans=[(start,start+.60,offset),(start+.90,start+.90+length-.60,offset+.60)]
            self.hist.append(dict(episode=self.eid,option=self.option,event='false_trigger',response_id=job['source_id'],at_s=r(start+.52),extent='80ms stop; .30s hold; no semantic input or generation cancellation',claims_heard=False))
        else:
            end=self.node(name+'_played',[n],length,False)
            spans=[(start,start+length,offset)]
        for a,b,o in spans:
            row=dict(episode=self.eid,option=self.option,job_id=job['id'],source_id=job['source_id'],
                start_s=r(a),end_s=r(b),source_offset_s=r(o),audio_seconds=r(b-a),output_epoch=len(self.plays)+1)
            self.plays.append(row)
            self.hist.append(dict(episode=self.eid,option=self.option,event='audio_output',response_id=job['source_id'],at_s=r(b),extent=f'{o:.3f}..{o+b-a:.3f} audio seconds',claims_heard=False))
        job['release_s']=self.nodes[end]['end_s']
        return n,end

    def prepare(self,name,source,gate):
        return self.audio(name,source,source['node_id'] if self.option=='A' else gate)

    def projected(self,nid,t0,memo):
        if nid in memo: return memo[nid]
        n=self.nodes[nid]
        if n['end_s']<=t0: value=0
        else:
            duration=max(0,n['end_s']-max(t0,n['start_s'])) if n['via'] else 0
            value=max((self.projected(p,t0,memo) for p in n['parents']),default=0)+duration
        memo[nid]=value
        return value

    def goal(self,name,start,terminals,description):
        memo={}
        duration=max(self.projected(t,start,memo) for t in terminals)
        DATA['goals'].append(dict(id=self.eid+':'+name,episode=self.eid,option=self.option,start_s=start,
            terminals=terminals,qa03_s=r(duration),description=description,terminal_content_correct=True))

    def run(self):
        k=self.kind; h=self.h; x=self.jitter; gate=self.commit
        qtext='수신자는 김 대리인가요?'; report='5월 보고서의 요약이 준비됐습니다. 비용과 일정의 차이를 확인해 주세요.'
        first_job=None; first_play=None; terminals=[]; extra_goal=False
        if k=='idle':
            s=self.source('status',h,self.first_text,3+x,[self.commit])
            j=self.prepare('status',s,gate); first_job=j
            first_play,end=self.play('status',j,gate); terminals=[j['full_node'],s['display_node'],end]
        elif k in ('ready','late','revision'):
            at=(h-.05 if k=='late' else -1.40)
            s=self.source('question',at,qtext,2.8+x)
            # Late source is not known at the beginning of the input.
            select=self.node('select_question',[gate,s['node_id']],0,True)
            j=self.prepare('question',s,select);first_job=j
            first_play,end=self.play('question',j,select)
            terminals=[j['full_node'],self.ui,end]
            if k=='revision':
                old=self.source('report_v1',-1.0,report,8+x)
                if self.option=='A': self.audio('report_v1',old,old['node_id'],cancel=h,discard=True)
                self.hist.append(dict(episode=self.eid,option=self.option,event='speech_start',response_id=old['id'],at_s=-self.speech_length,extent='block old output epoch; keep unrelated question',claims_heard=False))
                hand=self.node('revision_handoff',[self.commit],.12,True)
                new=self.source('report_v2',self.nodes[hand]['end_s']+2.4,'6월 보고서 요약입니다. 비용은 5월보다 감소했고 일정은 같습니다.',8+x,[hand],2)
                select2=self.node('select_report_v2',[end,new['node_id']],0,True)
                j2=self.prepare('report_v2',new,select2)
                _,end2=self.play('report_v2',j2,select2)
                self.goal('report_original',-6,[j2['full_node'],new['display_node'],end2], '원래 보고서 요청과 정정 전 VIA 처리부터 6월 결과의 표시 및 전체 음성 준비. 외부 업무와 재생 제외')
                extra_goal=True
        elif k=='withdraw':
            old=self.source('report_v1',-1.0,report,10+x)
            if self.option=='A': self.audio('report_v1',old,old['node_id'],cancel=h,discard=True)
            s=self.source('withdraw_ack',h,self.first_text,3.6+x,[gate])
            j=self.prepare('withdraw_ack',s,gate);first_job=j
            first_play,end=self.play('withdraw_ack',j,gate);terminals=[j['full_node'],self.ui,end]
            self.hist.append(dict(episode=self.eid,option=self.option,event='withdraw_output_only',response_id=old['id'],at_s=h,extent='keep UI and business Task',claims_heard=False))
        elif k=='resume':
            s=self.source('long_report',-4.80,'보고서 설명입니다. 먼저 비용입니다. 기본 비용은 지난달과 같지만 설치비가 추가되어 전체 금액이 늘었습니다. 다음은 일정입니다. 납품 예정일은 그대로이며, 검수는 납품 다음 날 시작합니다. 위험 항목은 공급 지연과 검수 인력 부족입니다. 담당자는 다음 주까지 대체 공급 가능 여부를 확인해야 합니다. 마지막으로 비교표의 빈칸은 확인되지 않은 값이며, 비용이 없다는 뜻은 아닙니다. 검토 후 추가 질문을 전달할 수 있습니다.',36+x)
            # Both had selected the same report before the evaluation input.
            j0=self.audio('long_report',s,s['node_id'],cancel=(h if self.option=='B' else None))
            onset=j0['first_s']+.04
            stop=-self.speech_length+.08
            self.plays.append(dict(episode=self.eid,option=self.option,job_id=j0['id'],source_id=s['id'],start_s=r(onset),end_s=stop,source_offset_s=0,audio_seconds=r(stop-onset),output_epoch=0))
            offset=stop-onset
            q=self.source('question',-.90,qtext,2.8+x)
            j=self.prepare('question',q,gate);first_job=j
            first_play,end=self.play('question',j,gate)
            if self.option=='A':
                _,er=self.play('resume_report',j0,end,offset=offset)
                full=j0['full_node']
            else:
                _,cached_end=self.play('resume_cached',j0,end,offset=offset)
                suffix=self.audio('resume_suffix',s,end,total=s['spoken_seconds']-j0['generated_audio_s'],token_extra=240)
                _,er=self.play('resume_suffix',suffix,cached_end)
                full=suffix['full_node']
            terminals=[full,j['full_node'],self.ui,er]
            self.hist.append(dict(episode=self.eid,option=self.option,event='barge_in',response_id=s['id'],at_s=-self.speech_length,extent=f'local halt at {stop:.3f}; {offset:.3f}s actually played; generation switches only at meaning adoption',claims_heard=False))
        elif k=='false_trigger':
            s=self.source('explanation',-4,'계속해서, '+self.first_text,8+x)
            j=self.audio('explanation',s,s['node_id']);first_job=j
            # One second previously played is included in both histories and removed from the retained buffer.
            self.plays.append(dict(episode=self.eid,option=self.option,job_id=j['id'],source_id=s['id'],start_s=-self.speech_length-.92,end_s=-self.speech_length+.08,source_offset_s=0,audio_seconds=1,output_epoch=0))
            first_play,end=self.play('continue',j,gate,offset=1,false_pause=True)
            terminals=[j['full_node'],self.ui,end]
        elif k=='burst':
            sources=[self.source('calendar',-1.4,self.first_text,3+x),self.source('mail',-1.2,'메일 초안이 준비됐습니다. 발송 전 확인해 주세요.',4+x),self.source('report',-.9,report,7+x)]
            for n,s in enumerate(sources):
                j=self.prepare(s['id'].split(':')[-1],s,gate)
                pn,end=self.play('burst'+str(n),j,gate)
                if n==0:first_job=j;first_play=pn
                terminals.extend([j['full_node'],s['display_node'],end]);gate=end
            terminals.append(self.ui)
        assert first_job and first_play and terminals
        goal_name='read_question' if k=='revision' else 'delivery_request'
        self.goal(goal_name,0,terminals,'이번 전달 요청의 의미 채택, 표시와 필수 전체 음성 준비. 재생과 사용자 대기는 제외')
        target_names={'revision':['report_v1','question'],'burst':['calendar','mail','report'],
                 'resume':['question','long_report']}.get(k,[{'withdraw':'report_v1','false_trigger':'explanation','idle':'status'}.get(k,'question')])
        targets=[self.eid+':'+t for t in target_names]
        expression_text={'revision':['보고서','메일 질문'],'burst':['일정','메일','보고서'],'resume':['메일 질문','이 설명'],
            'withdraw':['그 보고서'],'false_trigger':['그 설명'],'idle':['보고서']}.get(k,['메일 질문'])
        meaning=dict(operation=k,targets=targets,preserve_unrelated_tasks=True,mail_send_authorized=False,
            conditions={'revision':{'report_month':6,'question_first':True},'withdraw':{'cancel_audio_only':True,'keep_display':True},
                'resume':{'question_first':True,'resume_from_played_position':True},'burst':{'order':targets},
                'false_trigger':{'resume_from_played_position':True}}.get(k,{'read_selected_content':True}))
        exprs=[dict(id=f'{self.eid}:e{n+1}',expression=expression_text[n],oracle=[t],adopted=[t]) for n,t in enumerate(targets)]
        if self.option=='A':
            DATA['inputs'].append(dict(id=self.eid+':input',episode=self.eid,family=self.fid,text=self.utterance,
                acoustic_start_s=-self.speech_length,acoustic_end_s=0,oracle_meaning=meaning,expressions=exprs,
                first_voice_oracle=self.first_text,fixture_variant=f'fixed wording, duration perturbation {x:+.3f}s; not repeated trials',
                history='input-onset snapshot plus presentation events during input; unplayed preparation excluded',
                setup_request='5월 보고서를 요약해줘.' if k=='revision' else '관련 Agent 업무와 앞선 설명이 이미 존재하는 준비 활동',
                setup_start_s=-6,setup_accuracy_in_headline=False))
        DATA['responses'].append(dict(episode=self.eid,family=self.fid,option=self.option,input_id=self.eid+':input',
            first_ui_s=self.nodes[self.ui]['end_s'],first_voice_s=self.nodes[first_play]['end_s'],
            first_ui_text={'revision':'보고서를 6월 기준으로 다시 요청하고, 메일 질문부터 읽겠습니다.','resume':'메일 질문을 읽은 뒤 보고서 설명을 이어갑니다.','burst':'일정, 메일, 보고서 순서로 읽겠습니다.'}.get(k,self.first_text),
            first_voice_text=self.first_text,voice_job=first_job['id'],adopted_meaning=meaning,
            qa02_correct=len(exprs),qa02_n=len(exprs),qa07_correct=1,
            first_meaning_s=self.nodes[self.semantic]['end_s'],first_response_correct=True))
        # Preserve actual presentation events, not preparation as conversation evidence.
        self.hist.append(dict(episode=self.eid,option=self.option,event='input_snapshot',response_id=None,at_s=-self.speech_length,extent='UI and played audio only; ongoing overlap retained separately',claims_heard=False))
        DATA['sources'].extend(dict(episode=self.eid,option=self.option,**s) for s in self.sources)
        DATA['nodes'].extend(self.nodes.values());DATA['jobs'].extend(self.jobs);DATA['playback'].extend(self.plays)
        DATA['usage'].extend(self.usage);DATA['history'].extend(self.hist)
        for s in self.sources:
            ps=[p for p in self.plays if p['source_id']==s['id']]
            first=min((p['start_s'] for p in ps),default=None)
            DATA['deliveries'].append(dict(episode=self.eid,option=self.option,source_id=s['id'],
                source_available_s=s['available_s'],display_s=self.nodes[s['display_node']]['end_s'],
                first_audio_s=first,source_to_audio_s=r(first-s['available_s']) if first is not None else None,
                state='played' if ps else 'superseded_or_withdrawn',metric_role='agent-source diagnostic; not user-input QA01'))
        self.memory()

    def memory(self):
        times=set([-6,0,self.h])
        for j in self.jobs:times.update([j['start_s'],j['first_s'],j['end_s']]);times.add(j['release_s'] or j['end_s'])
        for p in self.plays:times.update([p['start_s'],p['end_s']])
        # All buffers are piecewise linear; sample both sides of each boundary.
        times=sorted(set(t+eps for t in times for eps in [-1e-7,0,1e-7]))
        for t in times:
            payload=0;live=0;active=0
            for j in self.jobs:
                if t<j['start_s'] or (j['release_s'] is not None and t>j['release_s']):continue
                live+=1;active+=int(t<j['end_s'])
                gen=0 if t<j['first_s'] else min(j['generated_audio_s'],.2+max(0,t-j['first_s'])*4)
                consumed=max([0]+[p['source_offset_s']+min(max(0,t-p['start_s']),p['audio_seconds']) for p in self.plays if p['job_id']==j['id'] and t>=p['start_s']])
                payload+=max(0,gen-consumed)*48000/1e6
            meta=live*4096/1e6
            DATA['memory'].append(dict(episode=self.eid,option=self.option,at_s=r(t),base_mb=560,
                audio_mb=r(payload),metadata_mb=r(meta),total_mb=r(560+payload+meta),active_generation=active,live_jobs=live))

def main():
    for family in FAMILIES:
        DATA['families'].append(dict(id=family[0],name=family[1],input=family[2],kind=family[4],n=10))
        for i in range(1,11):
            for opt in ['A','B']: Episode(family,i,opt).run()
    spec=json.loads((ROOT/'docs/architecture/12-decisions/decision-packages/qa-common-synthetic-20261021/change-spec.json').read_text())
    modules={
        'C01':(['model_wire'],['model_wire']), 'C02':(['agent_wire'],['agent_wire']),
        'C03':(['source_normalization'],['source_normalization']),
        'C04':(['response_sections','delivered_history','speech_playback','speech_preparation','prepared_audio'],['response_sections','delivered_history','turn_execution','turn_audio_buffer']),
        'C05':(['delivered_history','response_sections'],['delivered_history','response_sections']),
        'C06':(['permission_scope','evidence_access','publication_guard','preparation_dependencies'],['permission_scope','evidence_access','publication_guard'])}
    for change in spec['changes']:
        for oi,opt in enumerate(['A','B']):
            DATA['changes'].append(dict(id=change['id'],option=opt,title=change['title'],acceptance=change['acceptance'],
                modules=['concept::'+x for x in modules[change['id']][oi]],count=len(modules[change['id']][oi]),
                status='DESIGN_CHANGE_REVIEW_NOT_RUST_DIFF',
                reason={'C04':'A의 준비 실행, 보관 구간 및 재생 커서 사이 section patch 계약 변경. B는 현재 발화 내부에서 일관된 section patch를 적용한다.',
                        'C06':'A의 응답 단위 준비 의존성 무효화를 범위 단위로 확장. 공통 guard로 흡수 가능하면 추가 변경은 사라짐.'}.get(change['id'],'동일 외부 계약을 공통 adapter 또는 이력 경계에서 변경')))
    summary={}; families=[]
    for opt in ['A','B']:
        resp=[x for x in DATA['responses'] if x['option']==opt];goals=[x for x in DATA['goals'] if x['option']==opt]
        usage=[x for x in DATA['usage'] if x['option']==opt];mem=[x for x in DATA['memory'] if x['option']==opt]
        summary[opt]=dict(qa01_display_s=r(sum(x['first_ui_s'] for x in resp)/len(resp)),
            qa01_voice_s=r(sum(x['first_voice_s'] for x in resp)/len(resp)),qa02=sum(x['qa02_correct'] for x in resp)/sum(x['qa02_n'] for x in resp),
            qa03_s=r(sum(x['qa03_s'] for x in goals)/len(goals)),qa04_modules=sum(x['count'] for x in DATA['changes'] if x['option']==opt)/6,
            qa05_usd=r(sum(x['amount_usd'] for x in usage)),qa06_mb=max(x['total_mb'] for x in mem),qa07=1,
            input_n=len(resp),expression_n=sum(x['qa02_n'] for x in resp),goal_n=len(goals),
            discarded_cost_usd=r(sum(x['amount_usd'] for x in usage if x['phase']=='discarded')),
            audio_output_s=r(sum(x['audio_output_s'] for x in usage)),peak_witness=max(mem,key=lambda x:x['total_mb']))
        for fam in DATA['families']:
            rr=[x for x in resp if x['family']==fam['id']];gg=[x for x in goals if x['episode'].startswith(fam['id'])]
            uu=[x for x in usage if x['episode'].startswith(fam['id'])]
            families.append(dict(family=fam['id'],name=fam['name'],option=opt,n=len(rr),goal_n=len(gg),
                display_s=r(sum(x['first_ui_s'] for x in rr)/len(rr)),voice_s=r(sum(x['first_voice_s'] for x in rr)/len(rr)),
                via_s=r(sum(x['qa03_s'] for x in gg)/len(gg)),cost_usd=r(sum(x['amount_usd'] for x in uu))))
    DATA['summary']=summary; DATA['family_summary']=families
    # Every retained audio span must be ready when played; no same-device overlap.
    nodes={x['id']:x for x in DATA['nodes']};jobs={x['id']:x for x in DATA['jobs']}
    for p in DATA['playback']:
        j=jobs[p['job_id']]
        assert p['start_s']>=j['first_s']-1e-6
        assert p['source_offset_s']+p['audio_seconds']<=j['generated_audio_s']+1e-6
    for eid in {x['episode'] for x in DATA['inputs']}:
        for opt in ['A','B']:
            ps=sorted([p for p in DATA['playback'] if p['episode']==eid and p['option']==opt],key=lambda p:p['start_s'])
            assert all(a['end_s']<=b['start_s']+1e-6 for a,b in zip(ps,ps[1:])),(eid,opt)
    assert max(x['active_generation'] for x in DATA['memory'])<=4
    assert max(x['audio_mb'] for x in DATA['memory'])<16
    OUT.mkdir(parents=True,exist_ok=True)
    (OUT/'synthetic-data.json').write_text(json.dumps(DATA,ensure_ascii=False,indent=2)+'\n')
    (OUT/'summary.json').write_text(json.dumps(dict(version=VERSION,evidence_kind=KIND,summary=summary,family_summary=families),ensure_ascii=False,indent=2)+'\n')
    for key in ['inputs','sources','nodes','responses','goals','usage','jobs','playback','memory','changes','history','deliveries']:
        rows=DATA[key]
        with (OUT/(key+'.csv')).open('w',newline='') as f:
            w=csv.DictWriter(f,fieldnames=list(rows[0]),lineterminator='\n');w.writeheader()
            for row in rows:w.writerow({k:json.dumps(v,ensure_ascii=False) if isinstance(v,(dict,list)) else v for k,v in row.items()})
    # A final manifest also covers report, workbook and validation outputs.
    # Invalidate the former manifest until finalize.py runs after all exports.
    (OUT/'manifest.json').unlink(missing_ok=True)
    print(json.dumps(summary,ensure_ascii=False,indent=2))

if __name__=='__main__':main()
