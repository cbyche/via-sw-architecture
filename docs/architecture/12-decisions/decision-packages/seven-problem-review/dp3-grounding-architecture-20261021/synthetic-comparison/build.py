"""Author and check synthetic arithmetic; no model inference or benchmark execution."""
import csv
import hashlib
import json
from pathlib import Path
from statistics import mean

HERE = Path(__file__).resolve().parent
KIND = 'SYNTHETIC_AUTHORED_NOT_MEASURED'
CONFIGS = ['A', 'B-R1', 'B-R2', 'B-R3']
FAMILIES = [
    dict(id='S1', title='native 선택이 명확한 두 견적', state='native', ui=0, sam=0, scene=0,
         utterance='지금 선택한 이 두 견적을 설치비는 제외하고 비교표로 만들어줘.',
         targets=['L', 'R'], gesture='native selection = L,R; pointer movement irrelevant',
         a_calls=[(1200, 0, 64)], b_calls=[(1200, 0, 64)], a_errors=[], b_errors=[]),
    dict(id='S2', title='첫 밀집 화면에서 원으로 묶은 두 카드', state='cold', ui=1, sam=0, scene=96,
         utterance='방금 원으로 묶은 이 두 견적을 설치비는 제외하고 비교표로 만들어줘.',
         targets=['L', 'R'], gesture='closed pointer path contains L,R; nearby Z lies outside',
         a_calls=[(1800, 6000, 96)], b_calls=[(1800, 1400, 96)], a_errors=[1,2,3], b_errors=[3]),
    dict(id='S3', title='두 시점의 지칭을 같은 대상으로 연결하고 중복 제거', state='warm', ui=1, sam=2, scene=180,
         utterance='아까와 지금 가리킨 견적들을 중복 없이 묶고 설치비는 제외해서 요약표로 만들어줘.',
         targets=['L'], gesture='point L at -3.8s, viewport shifts at -2.2s, point moved L at -1.8s; old position now Z; no native object IDs or content transform; both point events must be linked before deduplication',
         a_calls=[(1800,4800,96),(1200,2400,80)], b_calls=[(2000,1600,96)], a_errors=[1,2,3,4], b_errors=[3,4]),
    dict(id='S4', title='연속 사용 중 새로 드러난 영역', state='warm', ui=1, sam=0, scene=96,
         utterance='새로 보이는 이 두 견적을 설치비는 제외하고 별도 비교표로 만들어줘.',
         targets=['L', 'R'], gesture='new viewport region was not in previous candidate coverage; circle L,R',
         a_calls=[(1800,3600,96)], b_calls=[(1600,1200,96)], a_errors=[1,2], b_errors=[2]),
    dict(id='S5', title='같은 장면의 후보와 확정 이력 재사용', state='reuse', ui=0, sam=0, scene=180,
         utterance='같은 화면에서 왼쪽의 이 두 견적을 설치비는 제외하고 새 비교표로 만들어줘.',
         targets=['L', 'R'], gesture='same source revision; prior list contains L,R,Z but did not choose current pair',
         a_calls=[(1600,1600,80)], b_calls=[(1400,600,80)], a_errors=[1,2,3], b_errors=[4]),
    dict(id='S6', title='장기 유휴 후 새 화면의 두 UI 항목', state='idle_cold', ui=1, sam=0, scene=48,
         utterance='지금 가리키는 이 두 견적을 설치비는 제외하고 비교표로 만들어줘.',
         targets=['L', 'R'], gesture='two pointer dwell intervals on L,R; detector boxes suffice; no SAM needed',
         a_calls=[(1600,3000,72)], b_calls=[(1400,800,72)], a_errors=[1], b_errors=[]),
]
PROFILE = dict(
    id='VOICE-SCREEN-CPU-SYN-1.0', evidence_kind=KIND,
    device='Authored 8 physical CPU cores, 16 GB RAM, SSD; no GPU',
    visual_cpu_threads_total=4, cpu_admission='one heavy visual stage at a time in all B main profiles',
    common_surroundings='Same request execution and history policies; not a reuse of COMMON-QA-SYN-1.0 BASE',
    qa01='Acoustic utterance end to first adopted concrete text, including incorrect text',
    qa03='One new scored goal per input; full causal completion excludes external Agent work',
    cloud_seconds=dict(setup=.2, input_tokens_per_second=6000, output_tokens_per_second=80),
    local_seconds=dict(final_s2s_text=.3, source_read=.08, adopt_display=.07,
                       runtime_boot=1.2, second_runtime_overlap_extra=.3, ui_load=1.1,
                       sam_load=.65, ui_inference=1.6, sam_new_frame=2.05, release=.15,
                       extra_r3_ipc=.06, raw_expansion_read=.10),
    tariff=dict(text_input_per_million=1, image_equivalent_input_per_million=1,
                text_output_per_million=4, audio_input_per_second=.001,
                audio_generation_per_second=.002),
    memory_mb=dict(core_ui=180, local_asr_audio=220, compressed_source_ring=96,
                   common_request_history=64, cloud_input_buffer=32,
                   runtime_shared=60, runtime_private_per_process=280,
                   ui_model_objects=420, sam_model_objects=170, ui_workspace=260,
                   sam_workspace=680, ui_load_temporary=380, sam_load_temporary=156,
                   ipc_one_worker=24, ipc_two_workers=32),
    notes=['All durations, resident bytes, token quantities and error frequencies are authored assumptions.',
           'Image-equivalent tokens are a fictional accounting bucket, not a provider tokenizer.',
           '120 scored new goals; 60 unscored prior setup activities for S3/S4/S5 are cost and RAM included.',
           'R1 retains the same compatible episode records as R2/R3 even after model unload.',
           'The six family weights are design coverage weights, not observed usage frequency.',
           'No real images, audio or model runs; sources are authored fixture specifications.'])

def cloud_time(text, image, output):
    return .2 + (text+image)/6000 + output/80

rows, nodes, usage, memory, sources, cases = [], [], [], [], [], []

class Run:
    def __init__(self, config, case, scope, start, scene=0, ready=False, prepared_sam=True):
        self.config, self.case, self.scope, self.t = config, case, scope, start
        self.previous = None
        self.live = dict(core_ui=180, local_asr_audio=220, compressed_source_ring=96,
                         common_request_history=64, input_or_ipc=32 if config in ['A','B-R3'] else 24,
                         runtime_shared=0, runtime_private=0, ui_model=0, sam_model=0,
                         retained_episode=scene if config!='A' else 0)
        self.has_runtime = ready and config!='A'
        self.runtime_count = (2 if config=='B-R3' and prepared_sam else 1) if self.has_runtime else 0
        if self.has_runtime:
            self.live.update(runtime_shared=60, runtime_private=280*self.runtime_count)
        if ready and config in ['B-R2','B-R3']:
            self.live.update(ui_model=420, sam_model=170 if prepared_sam else 0)

    def stage(self, kind, duration, *, included=True, extra=None, note=''):
        ident=f'{self.config}/{self.case}/{self.scope}/{len(nodes):05d}'
        end=self.t+duration
        nodes.append(dict(id=ident, config=self.config, case=self.case, scope=self.scope,
                          kind=kind, start=self.t, end=end, duration=duration,
                          qa03_included=included, parents=[self.previous] if self.previous else [], note=note))
        mem=dict(self.live)
        mem.update(extra or {})
        memory.append(dict(node_id=ident, config=self.config, case=self.case, scope=self.scope,
                           start=self.t, end=end, components_mb=mem, total_mb=sum(mem.values())))
        self.previous, self.t=ident,end
        return ident

    def cost(self, node, bucket, amount, rate, role='VIA'):
        usage.append(dict(node_id=node, config=self.config, case=self.case, scope=self.scope,
                          role=role, bucket=bucket, amount=amount, unit_rate_usd=rate, usd=amount*rate))

    def llm(self, kind, spec):
        tx,im,out=spec
        n=self.stage(kind,cloud_time(tx,im,out))
        for bucket,amount,rate in [('text_input',tx,1e-6),('image_equivalent_input',im,1e-6),('text_output',out,4e-6)]:
            self.cost(n,bucket,amount,rate)

    def vision(self, f):
        if not f['ui'] and not f['sam']:
            return
        c=self.config
        if not self.has_runtime:
            count=2 if c=='B-R3' and f['sam'] else 1
            self.live.update(runtime_shared=60,runtime_private=280*count)
            self.stage('runtime_boot',1.2+(.3 if count==2 else 0),
                       note='R3 two-process initialization overlaps; .3s residual is a stated assumption')
            self.has_runtime=True
        if f['ui']:
            if not self.live['ui_model']:
                self.live['ui_model']=420
                self.stage('ui_load',1.1,extra={'load_temporary':380})
            self.stage('ui_inference',1.6*f['ui'],extra={'workspace':260})
        if f['sam']:
            if c=='B-R1' and self.live['ui_model']:
                self.stage('ui_release_for_sam',.15)
                self.live['ui_model']=0
            if not self.live['sam_model']:
                self.live['sam_model']=170
                self.stage('sam_load',.65,extra={'load_temporary':156})
            self.stage('sam_inference',2.05*f['sam'],extra={'workspace':680})
        if c=='B-R3':
            self.stage('extra_worker_ipc',.06)

    def request(self, f, scored):
        start=self.t
        n=self.stage('s2s_final_text',.3)
        self.cost(n,'audio_input_seconds',6,.001)
        self.llm('screen_gate',(300,0,32))
        self.stage('source_read',.08)
        if self.config!='A':
            self.vision(f)
        calls=f['a_calls'] if self.config=='A' else f['b_calls']
        for i,spec in enumerate(calls):
            if i:
                self.stage('raw_expansion_read',.10)
            self.llm('raw_grounding' if self.config=='A' else 'candidate_binding',spec)
        self.stage('adopt_display',.07)
        first=self.t-start
        # After the input is copied, release is independent of delivery and never delays it.
        releasing=False
        if self.config=='B-R1' and (self.live['ui_model'] or self.live['sam_model']):
            parent, t = self.previous,self.t
            self.stage('async_model_release',.15,note='Parallel with dispatch, not charged twice on QA03 critical path')
            self.previous,self.t=parent,t
            releasing=True
        self.stage('agent_dispatch',.15)
        if releasing:
            self.live.update(ui_model=0,sam_model=0)
        n=self.stage('external_agent_work',8,included=False)
        self.cost(n,'agent_text_input',1000,1e-6,'DownstreamAgent')
        self.cost(n,'agent_text_output',200,4e-6,'DownstreamAgent')
        self.stage('result_link',.10)
        self.llm('final_detail_summary',(1000,0,120))
        n=self.stage('audio_summary_prepare',.35)
        self.cost(n,'audio_generation_seconds',4,.002)
        self.stage('final_ui',.05)
        return dict(first_seconds=first,qa03_seconds=self.t-start-8,wall_seconds=self.t-start)

for f in FAMILIES:
    for variant in range(1,21):
        cid=f"{f['id']}-{variant:02d}"
        ids={key:f'{cid}:quote-{key}' for key in ['L','R','Z']}
        targets=[ids[k] for k in f['targets']]
        oracle=dict(targets=targets,goal='요약표 작성' if f['id']=='S3' else '비교표 작성',
                    exclusions=['설치비'],relation='new_goal',preserve_existing_tasks=True)
        if f['id']=='S3': oracle['deduplicate_same_object']=True
        frame_specs=[dict(frame_id=f'{cid}:f0',time=-3.8,source_revision='rev-1',
                          frame_size=[1280,720],visible_regions=[
                              dict(display_text=f'견적 {variant}-가',box=[100,180,220,100]),
                              dict(display_text=f'견적 {variant}-나',box=[340,180,220,100]),
                              dict(display_text=f'견적 {variant}-다',box=[580,180,220,100])])]
        if f['id']=='S3':
            for region in frame_specs[0]['visible_regions']:
                region['display_text']='견적'
            frame_specs[0]['pointer']=[180,230]
            frame_specs[0]['identity_limit']='no unique OCR title, native ID or exposed viewport transform; correspondence is visual'
            frame_specs.append(dict(frame_id=f'{cid}:f1',time=-1.8,source_revision='rev-2',
                                    frame_size=[1280,720],change='same-looking quote cards moved; native identity and scroll transform unavailable',
                                    visible_regions=[dict(display_text='견적',box=[100,340,220,100]),
                                                     dict(display_text='견적',box=[100,180,220,100])],
                                    pointer=[180,390],identity_evidence='visual temporal correspondence to f0; distinct frame-local boxes do not encode persistent identity'))
        source=dict(id=cid,kind='AUTHORED_FIXTURE_SPEC_NOT_PIXELS',frames=frame_specs,
                    action=f['gesture'],coordinate_transform='same desktop/frame scale, no missing transform',
                    utterance_start=-6,utterance_end=0,referent_word_interval=[-4.1,-3.5],
                    asr_text=f['utterance'],asr_timestamps='exact by construction',
                    native_selection=targets if f['state']=='native' else None)
        if f['id']=='S3':
            source['referent_word_intervals']=[[-4.1,-3.5],[-2.1,-1.5]]
            source['content_transform']=None
            source['model_call_reason']='UI detects frame-local seed; SAM derives temporal candidate correspondence for deduplication. It is not needed merely to look up the past frame.'
        sources.append(source)
        has_setup=f['state'] in ['warm','reuse']
        cases.append(dict(id=cid,family=f['id'],variant=variant,input_id=cid+':input',
                          goal_id=cid+':goal',expression_id=cid+':expression',revision=1,
                          utterance=f['utterance'],oracle=oracle,source_id=cid,
                          initial_state=f['state'],setup=has_setup,
                          setup_request=(('이전 별도 두 frame에서 짚은 견적들을 중복 없이 목록으로 정리해줘. 현재 평가 clip과 객체는 아직 도착하지 않음.' if f['id']=='S3' else '직전 화면의 견적 목록을 확인해줘. 미래 평가 입력의 대상 쌍은 지정하지 않음.') if has_setup else None)))
        for config in CONFIGS:
            if has_setup:
                prep=Run(config,cid,'prior_setup',-60,scene=180)
                prior=dict(ui=1,sam=2 if f['id']=='S3' else 0,scene=180,a_calls=[(1800,3600,96)],b_calls=[(1800,1400,96)])
                prep.request(prior,False)
                assert prep.t<0
            run=Run(config,cid,'scored',0,scene=f['scene'],ready=has_setup,prepared_sam=f['id']=='S3')
            timing=run.request(f,True)
            target_error=variant in (f['a_errors'] if config=='A' else f['b_errors'])
            semantic_only=(variant==10 and f['id']!='S6')
            adopted=json.loads(json.dumps(oracle))
            if target_error:
                adopted['targets']=targets[:-1]+[ids['Z']]
            if semantic_only:
                adopted['exclusions']=[]
            reason=''
            if target_error:
                reason=('원본의 경계/집합/스크롤 전후 동일성을 잘못 결합' if config=='A'
                        else '생산 후보 누락 또는 잘못된 경계/track을 결합 단계가 바로잡지 못함')
            if semantic_only:
                reason='대상은 맞지만 설치비 제외 조건을 최초 채택에서 누락'
            names=[t.rsplit(':',1)[1] for t in adopted['targets']]
            content=f"{', '.join(names)}의 "+('설치비를 제외해 ' if adopted['exclusions'] else '설치비를 포함해 ')+adopted['goal']+'을 맡기겠습니다.'
            rows.append(dict(config=config,case=cid,family=f['id'],input_id=cid+':input',
                             goal_id=cid+':goal',expression_id=cid+':expression',**timing,
                             oracle=oracle,first_adopted=adopted,first_response=content,
                             qa02_success=adopted['targets']==targets,
                             qa07_success=adopted==oracle,error_reason=reason,
                             semantic_error_shared_id=cid+':target-error' if target_error else None,
                             terminal='completed_as_adopted',semantic_success=adopted==oracle))

base_modules=[['model_wire'],['agent_wire'],['source_normalization'],
              ['response_sections','delivered_history','speech_queue'],
              ['delivered_history','response_sections'],
              ['permission_scope','evidence_access','publication_guard']]
changes=[]
for config in CONFIGS:
    for i,mods in enumerate(base_modules,1):
        mods=list(mods)
        if config!='A' and i==6: mods.append('candidate_dependencies')
        changes.append(dict(config=config,change_id=f'C{i:02d}',modules=['concept::'+m for m in mods],
                            count=len(mods),evidence_kind='SYNTHETIC_DESIGN_CHANGE_REVIEW',
                            reason='C01~C06 same acceptance; B C06 additionally changes extent-based candidate invalidation' if i==6 else 'same normalized contract owner'))

summary={}
for config in CONFIGS:
    r=[x for x in rows if x['config']==config]
    u=[x for x in usage if x['config']==config]
    mem=[x for x in memory if x['config']==config]
    s=dict(qa01_seconds=mean(x['first_seconds'] for x in r),qa02_correct=sum(x['qa02_success'] for x in r),
           qa02_denominator=len(r),qa03_seconds=mean(x['qa03_seconds'] for x in r),
           qa04_modules=mean(x['count'] for x in changes if x['config']==config),
           qa05_usd=sum(x['usd'] for x in u if x['role']=='VIA'),
           qa05_scored_usd=sum(x['usd'] for x in u if x['role']=='VIA' and x['scope']=='scored'),
           qa05_setup_usd=sum(x['usd'] for x in u if x['role']=='VIA' and x['scope']=='prior_setup'),
           downstream_agent_usd=sum(x['usd'] for x in u if x['role']=='DownstreamAgent'),
           qa06_mb=max(x['total_mb'] for x in mem),
           qa07_correct=sum(x['qa07_success'] for x in r),qa07_denominator=len(r))
    s['peak_witness']=max(mem,key=lambda x:x['total_mb'])
    s['families']={f['id']:dict(qa01_seconds=mean(x['first_seconds'] for x in r if x['family']==f['id']),
                                      qa03_seconds=mean(x['qa03_seconds'] for x in r if x['family']==f['id']),
                                      qa02_correct=sum(x['qa02_success'] for x in r if x['family']==f['id']),
                                      qa07_correct=sum(x['qa07_success'] for x in r if x['family']==f['id']),n=20)
                         for f in FAMILIES}
    summary[config]=s

assert len(rows)==480 and len(cases)==120 and len(sources)==120
assert len({x['expression_id'] for x in cases})==120
for config in CONFIGS:
    assert summary[config]['qa02_correct']==(107 if config=='A' else 115)
    assert summary[config]['qa07_correct']==(102 if config=='A' else 110)
    assert len({x['case'] for x in nodes if x['config']==config and x['scope']=='prior_setup'})==60
    assert summary[config]['qa06_mb']==dict(A=592,**{'B-R1':1954,'B-R2':2374,'B-R3':2662})[config]
    assert all(x['qa03_seconds']>=x['first_seconds'] for x in rows if x['config']==config)
assert len({round(summary[c]['qa05_usd'],9) for c in CONFIGS[1:]})==1
by_id={n['id']:n for n in nodes}
causal_finish={}
for n in nodes:
    assert n['end']>=n['start']
    for p in n['parents']: assert by_id[p]['end']<=n['start']+1e-8
    causal_finish[n['id']]=(n['duration'] if n['qa03_included'] else 0)+max([causal_finish[p] for p in n['parents']] or [0])
for r in rows:
    end=next(n for n in nodes if n['config']==r['config'] and n['case']==r['case'] and n['scope']=='scored' and n['kind']=='final_ui')
    assert abs(causal_finish[end['id']]-r['qa03_seconds'])<1e-8
for u in usage: assert u['node_id'] in by_id
assert all(abs(sum(m['components_mb'].values())-m['total_mb'])<1e-8 for m in memory)
for c in CONFIGS[1:]:
    for r in [r for r in rows if r['config']==c and r['family']=='S5']:
        other=next(x for x in rows if x['config']=='B-R1' and x['case']==r['case'])
        assert r['first_seconds']==other['first_seconds']

data=dict(evidence_kind=KIND,dataset_id='DP3-GROUNDING-CPU-SYN-1.0',profile=PROFILE,
          configs=CONFIGS,families=FAMILIES,cases=cases,sources=sources,responses=rows,
          stages=nodes,usage=usage,memory=memory,changes=changes,summary=summary)
for name,obj in [('synthetic-data.json',data),('summary.json',summary)]:
    (HERE/name).write_text(json.dumps(obj,ensure_ascii=False,indent=2)+'\n')

def csv_dump(name, records):
    with (HERE/name).open('w',newline='',encoding='utf-8-sig') as file:
        w=csv.DictWriter(file,fieldnames=['evidence_kind','dataset_id']+[k for k in records[0] if k!='evidence_kind']);w.writeheader()
        for r in records:
            out={k:json.dumps(v,ensure_ascii=False) if isinstance(v,(dict,list)) else v for k,v in r.items()}
            out['evidence_kind']=r.get('evidence_kind',KIND)
            out['dataset_id']='DP3-GROUNDING-CPU-SYN-1.0'
            w.writerow(out)

csv_dump('responses.csv',rows);csv_dump('stages.csv',nodes);csv_dump('usage.csv',usage)
csv_dump('memory.csv',memory);csv_dump('changes.csv',changes)
lines=['# 네 대안 합성 집계','',f'근거 종류: `{KIND}`. [계약과 해석](README.md).','',
       '| QA | A | B-R1 | B-R2 | B-R3 |','| --- | ---: | ---: | ---: | ---: |']
fields=[('QA01 첫 구체적 표시 s','qa01_seconds'),('QA02 지칭 성공 /120','qa02_correct'),
        ('QA03 목표별 VIA 처리 s','qa03_seconds'),('QA04 예상 Rust 모듈/과제','qa04_modules'),
        ('QA05 USD/120평가+60준비 묶음','qa05_usd'),('QA06 동시 RAM MB','qa06_mb'),
        ('QA07 전체 의미 성공 /120','qa07_correct')]
for label,key in fields:
    lines.append('| '+label+' | '+' | '.join(f'{summary[c][key]:.4f}' if key=='qa05_usd' else f'{summary[c][key]:.3f}' for c in CONFIGS)+' |')
lines+=['','## 유형별 첫 응답','', '| 유형 | A | B-R1 | B-R2 | B-R3 |','| --- | ---: | ---: | ---: | ---: |']
for f in FAMILIES:
    lines.append('| '+f['id']+' '+f['title']+' | '+' | '.join(f"{summary[c]['families'][f['id']]['qa01_seconds']:.3f}" for c in CONFIGS)+' |')
lines+=['','## 비용의 포함 범위','', '| 구성 | 평가 요청 VIA | 준비 이력 VIA | QA05 합계 | Agent 별도 |','| --- | ---: | ---: | ---: | ---: |']
for c,s in summary.items():
    lines.append('| '+c+' | '+' | '.join(f'{s[k]:.5f}' for k in ['qa05_scored_usd','qa05_setup_usd','qa05_usd','downstream_agent_usd'])+' |')
(HERE/'results.md').write_text('\n'.join(lines)+'\n')
manifest=dict(dataset_id=data['dataset_id'],evidence_kind=KIND,counts=dict(scored_cases=120,setup_per_config=60,
              responses=480,stages=len(nodes),usage_rows=len(usage),memory_rows=len(memory)),
              validation='PASS: authored arithmetic, IDs, dependency order, denominators, peak sums; not performance',
              files={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in [HERE/'build.py',HERE/'synthetic-data.json',HERE/'summary.json']})
(HERE/'manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({c:{k:v for k,v in s.items() if k not in ['families','peak_witness']} for c,s in summary.items()},ensure_ascii=False,indent=2))
