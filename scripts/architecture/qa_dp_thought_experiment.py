#!/usr/bin/env python3
"""Deterministic arithmetic of authored forecasts. Never runs VIA, models or a PoC."""
from pathlib import Path
from fractions import Fraction as F
import argparse
import hashlib
import json
import qa_candidate_population as population
import check_qa_candidate_package as qa

P=population.P
A_PATH=P/'thought-experiment/assumptions.json'
NAMES=['첫 응답 지연시간','지칭 대상 식별 정확도','VIA 처리시간','변경 범위','모델 사용 비용','최대 RAM 사용량','부하 추가 지연','병행 수정 중복','음성 전달 지연']
UNITS=['s','%','s','modules/change','USD/COMMON-28 bundle','MB','ms','modules/pair','ms']
DOTS=['○○○','◐○○','●○○','●◐○','●●○','●●◐','●●●']

def rational(value):
    value=F(value)
    return {'numerator':value.numerator,'denominator':value.denominator,'decimal':float(value)}

def fmt(value,places=3):
    return f'{float(value):.{places}f}'.rstrip('0').rstrip('.')

def readable_json(value,level=0):
    flat=json.dumps(value,ensure_ascii=False)
    if not isinstance(value,(dict,list)) or len(flat)<=200:return flat
    pad='  '*(level+1);close='  '*level
    if isinstance(value,dict):
        return '{\n'+',\n'.join(pad+json.dumps(k,ensure_ascii=False)+': '+readable_json(v,level+1) for k,v in value.items())+'\n'+close+'}'
    return '[\n'+',\n'.join(pad+readable_json(v,level+1) for v in value)+'\n'+close+']'

def score(index,value):
    path=next(P.glob(f'qa{index:02d}-*.md'))
    cells=qa.score_cells(path.read_text())
    hits=[k for k,v in cells.items() if qa.contains(qa.interval(v[2]),value)]
    assert len(hits)==1,(index,value,hits)
    return hits[0]

def class_for(a,eid,which):
    return next((k for k,v in a[which].items() if eid in v),None)

def load_parts(a,dp,opt,sid):
    return a['load_components'].get(f'{dp}{opt}',{}).get(sid,a['load_components']['common'][sid])

def event_record(a,e,dp,opt):
    """Call outputs are dependencies, not a uniform delta pasted on a common latency."""
    t=a['timing_model'];eid=e['id'];sid=eid[:3];screen=class_for(a,eid,'screen_classes');history=class_for(a,eid,'history_classes')
    profile=e['reference_semantic_call']['profile'];nodes=[];fin={};calls=[]
    def node(name,duration,parents=()):
        assert duration>=0
        assert all(p in fin for p in parents)
        finish=duration+max([fin[x] for x in parents] or [0]);fin[name]=finish
        nodes.append({'id':name,'duration_ms':duration,'parents':list(parents),'finish_ms':finish})
        return name
    def model(name,budget,parents,final=False,overlap=0,first_tokens=None):
        b=dict(a['call_budgets'][budget]);b['image']=b.get('image',e['reference_semantic_call']['image_count']*3000)
        b['call_id']=eid+':'+name;b['kind']=budget;b['overlap_before_t0_ms']=overlap
        b['first_content_tokens']=first_tokens or t['first_content_tokens']
        fixed=t['semantic_call_fixed_ms']+(b['text']+b['image'])*t['prefill_ms_per_1000_tokens']//1000
        full=fixed+b['out']*t['decode_ms_per_token'];b['full_duration_ms']=full
        assert 0<=overlap<=full
        calls.append(b)
        if final:
            first=fixed+b['first_content_tokens']*t['decode_ms_per_token'];assert first<=full
            f=node(name+':first',first,parents);done=node(name+':full',full-first,[f]);return f,done
        return node(name,full-overlap,parents)
    mode='agent' if e['kind']=='agent' else e['input_mode']
    start=node('input_tail',t.get('voice_tail_overrides_ms',{}).get(eid,t[mode+'_tail_ms']))
    read=t['screen_read_ms'] if screen else t['context_read_ms'] if history else t['other_read_ms']
    if history=='cache':read=15
    start=node('bounded_source_read',read,[start])
    wire=t['b44_stage_and_join_wiring_ms'] if dp==44 and opt=='B' else t['dispatcher_wiring_ms']
    start=node('execution_wiring',wire,[start])
    if sid in a['load_components']['common'] and eid==sid+'-U01':
        start=node('background_resource_wait',sum(load_parts(a,dp,opt,sid).values()),[start])
    if screen:
        if dp==47 and opt=='B':
            if screen in ['native','partial']:
                ready=node('native_or_valid_partial_candidates',10,[start])
            else:
                speech=model('SpeechReferenceProducer','speech_producer',[start])
                # Screen source can be prepared before final speech evidence; both options retain raw input.
                overlap=t['new_screen_prework_ms'] if screen=='expansion' else t['fresh_screen_prework_ms']
                visual_source=node('independent_screen_read',t['screen_read_ms'])
                visual=model('ScreenTargetProducer','screen_producer',[visual_source],overlap=overlap)
                ready=node('candidate_revision_commit',12,[speech,visual])
            if screen=='expansion':
                proposal=model('binding_needs_expansion','binding_proposal',[ready])
                expanded=model('producer_raw_expansion','screen_raw_expansion',[proposal])
                ready=node('candidate_revision_republish',12,[expanded])
                grounded=model('MultimodalBindingResolver','screen_rebinding',[ready])
            else:grounded=model('MultimodalBindingResolver','binding',[ready])
        else:
            ready=start
            if screen=='expansion':ready=model('joint_bounded_raw_expansion','screen_raw_expansion',[ready])
            budget={'native':'screen_native_joint','partial':'screen_patch_joint'}.get(screen,'screen_joint')
            grounded=model('JointGroundingInterpreter',budget,[ready])
        # A whole GroundingProposal is required before the adopted target/scope card.
        # The common detailed response remains a real additional model call and completion cost.
        first=grounded
        _,full=model('common_detailed_screen_response','screen_detailed_response',[grounded],final=True)
    elif history:
        if dp==45 and opt=='B':
            ready=start
            if history=='new_relation':
                ready=model('MemoryPublisher_update','history_update_publisher',[ready])
                ready=node('publish_and_readback',t['b45_publish_readback_ms'],[ready])
            else:ready=node('published_relation_readback',12,[ready])
            if history=='current_comparison':
                first,full=model('current_comparison_with_originals','history_joint',[ready],final=True,first_tokens=500)
            else:first,full=model('current_request_with_originals','history_response',[ready],final=True)
        elif history in ['semantic','new_relation','updated_ready','current_comparison']:
            # Strong A combines relation interpretation and current response in one model call.
            first,full=model('joint_history_and_current_request','history_joint',[start],final=True,first_tokens=500)
        else:first,full=model('current_request_with_originals','history_response',[start],final=True)
    else:
        first,full=model('current_response',profile,[start],final=True)
    if dp==42 and opt=='B' and e['kind']=='agent':
        first=node('work_fact_ipc_and_dialogue_projection',t['b42_agent_projection_extra_ms'],[first])
        full=node('full_work_fact_projection',t['b42_agent_projection_extra_ms'],[full])
    displayed=node('first_content_ui',t['response_adopt_and_ui_ms'],[first])
    detailed=node('full_text_ui',t['response_adopt_and_ui_ms'],[full])
    finish_parents=[detailed]
    if e['response']['voice_required']:
        finish_parents.append(node('all_summary_audio_ready',t['audio_all_ready_tail_ms'],[displayed]))
    complete=node('response_complete',0,finish_parents)
    return {'event_id':eid,'profile':profile,'mechanism_class':screen or history or profile,'first_ms':fin[displayed],'completion_tail_ms':fin[complete]-fin[displayed],'first_and_completion_nodes':nodes,'semantic_calls':calls,'voice_required':e['response']['voice_required'],'correctness_used_to_modify_time':False}

def goal_record(a,s,g,events,dp,opt):
    # Authored precedence algebra. External domain, user speech and playback durations are zero.
    ids=list(g['member_events']); rule=a['goal_dependencies'].get(g['id'],'serial_user_then_agent')
    if rule=='include_intervening_S24-U02':ids.insert(1,'S24-U02')
    byid={e['id']:e for e in s['events']}; nodes=[];fin={};last_user=None;last_agent=None
    handoff=a['timing_model']['local_handoff_ms']+(a['timing_model']['b42_handoff_extra_ms'] if opt=='B' and dp==42 else 0)
    def node(name,duration,parents):
        finish=duration+max([fin[x] for x in parents] or [0]);fin[name]=finish
        nodes.append({'id':name,'duration_ms':duration,'parents':parents,'finish_ms':finish})
    previous_output=None
    for ix,eid in enumerate(ids):
        er=events[eid];e=byid[eid]
        if e['kind']=='user':
            parents=[previous_output] if previous_output else []
            node(eid+':first',er['first_ms'],parents)
            tail=min(er['completion_tail_ms'],a['timing_model']['interrupt_after_first_content_ms']) if rule=='interrupt_after_first_content' and ix==0 else er['completion_tail_ms']
            node(eid+':output',tail,[eid+':first']);last_user=eid;previous_output=eid+':output'
        else:
            if rule=='independent_existing_tasks' or (g['id']=='S17-G05' and eid=='S17-A10'):
                # Existing task source is external, no newly issued request is a causal prerequisite.
                parents=[]
            elif rule=='dependent' and last_agent:
                node(eid+':handoff',handoff,[last_agent+':first']);parents=[eid+':handoff']
            else:
                node(eid+':handoff',handoff,[last_user+':first']);parents=[eid+':handoff']
            node(eid+':first',er['first_ms'],parents)
            node(eid+':output',er['completion_tail_ms'],[eid+':first']);last_agent=eid;previous_output=eid+':output'
    total=max(fin[eid+':output'] for eid in ids)
    return {'goal_id':g['id'],'member_events':g['member_events'],'additional_causal_events':[x for x in ids if x not in g['member_events']],'precedence_assumption':rule,'nodes':nodes,'via_ms':total}

def cost_rows(a,data,all_events,dp,opt):
    quoted,_,_,_,_=population.quote(data)
    rows=[{'label':n,'scope':'common_voice','cost_usd':str(v)} for n,_,_,v in quoted[:6]]
    c=a['cost'];pi=F(c['luna_input_usd_per_million']);po=F(c['luna_output_usd_per_million'])
    calls=[call for e in all_events.values() for call in e['semantic_calls']]
    if dp==45 and opt=='B':
        for s in data['scenarios']:
            for i in range(c['b45_initial_calls_per_episode']):
                b=dict(a['call_budgets']['history_initial_publisher']);b['call_id']=s['id']+f':initial_publisher_{i+1}';b['kind']='initial_preparation';calls.append(b)
        # New relation calls already appear in the foreground DAG and must not be charged twice.
        for eid in c['b45_update_events']:
            if any(x['call_id']==eid+':MemoryPublisher_update' for x in calls):continue
            b=dict(a['call_budgets']['history_update_publisher']);b['call_id']=eid+':background_publisher_update';b['kind']='background_update';calls.append(b)
    for call in calls:
        value=(pi*(call['text']+call['image'])+po*call['out'])/1000000
        rows.append({'label':call['call_id'],'scope':call['kind'],'calls':1,'input_text_tokens':call['text'],'input_image_tokens':call['image'],'output_including_reasoning_tokens':call['out'],'cost_usd':str(float(value)),'cost_exact':rational(value)})
    return rows

def accuracy_rows(a,data,dp,opt,pop):
    rows=[]
    for s in data['scenarios']:
        for b in s['bindings']:
            if b['id'] not in pop['bindings']:continue
            eid=b['event_id'];screen=class_for(a,eid,'screen_classes');history=class_for(a,eid,'history_classes')
            key=screen or ('history_'+history if history else 'other')
            choice=opt if (screen and dp==47) or (history and dp==45) else 'A'
            rule=a['accuracy_rules'][key]
            assert 0<=F(rule[choice])<=100
            rows.append({'binding_id':b['id'],'event_id':eid,'expected_correct_percent':rule[choice],'risk_class':key,'reason':rule['reason']})
    return rows

def build(a,data):
    summary=[];alternatives=[];byvariant={};allids={e['id'] for s in data['scenarios'] for e in s['events']}
    for group in ['screen_classes','history_classes']:
        ids=[eid for values in a[group].values() for eid in values]
        assert len(ids)==len(set(ids)) and set(ids)<=allids
    assert set(sum(a['screen_classes'].values(),[]))==set(a['populations']['47']['events'])
    assert set(sum(a['history_classes'].values(),[]))==set(a['populations']['45']['events'])
    assert a['comparison']['timing_correctness_penalty'] is False
    for dp in a['comparison']['dps']:
        pop=a['populations'][str(dp)];assert set(pop['events'])<=allids
        assert set(pop['events']).isdisjoint(pop['excluded_event_ids'])
        assert set(pop['events'])|set(pop['excluded_event_ids'])==allids
        for opt in ['A','B']:
            tag=f'{dp}{opt}'
            all_events={e['id']:event_record(a,e,dp,opt) for s in data['scenarios'] for e in s['events']}
            events=[all_events[eid] for eid in pop['events']]
            goals=[goal_record(a,s,g,all_events,dp,opt) for s in data['scenarios'] for g in s['goals'] if g['id'] in pop['goals']]
            # Unscored events still appear in complete goal graphs if causally required.
            closure={eid for g in goals for eid in g['member_events']+g['additional_causal_events']}
            accuracy=accuracy_rows(a,data,dp,opt,pop);assert len(accuracy)==len(pop['bindings'])
            key=tag if tag in a['module_sets'] else 'reference'
            modules={cid:a['module_sets'][key][cid] for cid in pop['changes']}
            overlaps=[{'pair':pair,'modules':sorted(set(modules[pair[0]])&set(modules[pair[1]]))} for pair in pop['change_pairs']]
            costs=cost_rows(a,data,all_events,dp,opt)
            ram=[]
            for sid in pop['episodes']:
                r=a['ram'];c=dict(r['baseline_components']);c['screen_decoded_and_compressed_cache']=r['screen_by_scenario'].get(sid,r['default_screen_mb']);c['history_source_cache']=r['history_by_scenario'].get(sid,r['default_history_mb'])
                if opt=='B':
                    if dp in [44,45,42]:c.update(r[f'dp{dp}_b_extra'])
                    if dp==47:c.update(r['dp47_b_extra_screen'])
                ram.append({'scenario':sid,'simultaneous_components_mb':c,'total_mb':sum(c.values())})
            loads=[]
            for sid in pop['loads']:
                parts=load_parts(a,dp,opt,sid);added=sum(parts.values());loaded=all_events[sid+'-U01']['first_ms']
                loads.append({'event':sid+'-U01','loaded_ms':loaded,'idle_ms':loaded-added,'added_ms':added,'components_ms':parts})
            values=[F(sum(e['first_ms'] for e in events),1000*len(events)),sum(F(x['expected_correct_percent']) for x in accuracy)/len(accuracy),F(sum(g['via_ms'] for g in goals),1000*len(goals)),F(sum(len(x) for x in modules.values()),len(modules)),sum(F(x['cost_usd']) for x in costs),F(max(x['total_mb'] for x in ram)),F(sum(x['added_ms'] for x in loads),len(loads)),F(sum(len(x['modules']) for x in overlaps),len(overlaps)),F(a['audio_eligible_delay_ms']['total'])]
            assert all(len(ms)==len(set(ms)) for ms in modules.values())
            selected_semantic_cost=sum(F(x['cost_usd']) for x in costs[6:] if x['label'].split(':')[0] in pop['events'])
            one={'dp':dp,'option':opt,'evidence':'THOUGHT_EXPERIMENT_NOT_MEASURED','population':pop,'events':events,'additional_goal_events':[all_events[eid] for eid in sorted(closure-set(pop['events']))],'goals':goals,'accuracy_priors':accuracy,'change_sets':modules,'overlaps':overlaps,'model_cost_ledger':costs,'related_foreground_semantic_cost_usd':rational(selected_semantic_cost),'simultaneous_ram_ledger':ram,'load_pairs':loads,'audio_eligible_delay_ms':a['audio_eligible_delay_ms'],'audio_count_per_pass':sum(e['voice_required'] for e in events)}
            alternatives.append(one);byvariant[tag]=(one,values)
            for i,v in enumerate(values,1):
                applicability='STRUCTURAL_HYPOTHESIS'
                if i==2:applicability=pop['qa02_class']
                if i==9 or i==5 and dp in [44,42]:applicability='REGRESSION_ONLY'
                summary.append({'dp':dp,'option':opt,'qa':f'QA{i:02}','name':NAMES[i-1],'unit':UNITS[i-1],'point_estimate':rational(v),'score_0_to_6':score(i,v),'rating':DOTS[score(i,v)],'applicability':applicability,'status':'THOUGHT_EXPERIMENT_NOT_MEASURED','timing_failure_penalty_applied':False})
    # The control structure is identical even though selected populations differ.
    controls=[one for one in alternatives if one['option']=='A']
    for left in controls:
        for right in controls:
            lm={e['event_id']:e for e in left['events']};rm={e['event_id']:e for e in right['events']}
            assert all(lm[eid]==rm[eid] for eid in set(lm)&set(rm))
            assert left['model_cost_ledger']==right['model_cost_ledger']
            for cid in set(left['change_sets'])&set(right['change_sets']):
                assert left['change_sets'][cid]==right['change_sets'][cid]
    paths=[A_PATH,P/'thought-experiment/rationale.json',P/'common-scenarios.json',*sorted(P.glob('qa[0-9][0-9]-*.md'))]
    result={'version':a['version'],'status':a['status'],'provenance':{str(f.relative_to(P)):hashlib.sha256(f.read_bytes()).hexdigest() for f in paths},'summary':summary,'alternatives':alternatives}
    matrix=['| QA / 단위 | 44A | 44B | 47A | 47B | 45A | 45B | 42A | 42B |','| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
    for i,name in enumerate(NAMES,1):
        matrix.append(f'| QA{i:02} {name} / {UNITS[i-1]} | '+' | '.join(f'{fmt(v[i-1],6)} ({score(i,v[i-1])}점)' for _,v in byvariant.values())+' |')
    ledger=['# 구조별 사고 실험 계산 원장','',f'> **{a["version"]} / 작성자 예상치 / 실측 아님**','', '[발표용 비교표](../qa-dp-thought-experiment.md) / [가정 원본](./assumptions.json) / [모든 계산 JSON](./predictions.json)','', '시간과 정확도는 DP별 관련 사건을 사용한다. QA03은 선택 목표의 전체 인과 경로를 보존한다. QA05는 기존 배점 단위인 전체 COMMON-28 묶음의 비용이며 관련 전경 의미 호출 비용도 따로 표시한다. 5회 반복은 같은 기대값이며 실제 관측 5개가 아니다.','', '## 1. 전체 결과','',*matrix,'']
    for dp in a['comparison']['dps']:
        pop=a['populations'][str(dp)];ao,av=byvariant[f'{dp}A'];bo,bv=byvariant[f'{dp}B']
        ledger += [f'## DP-{dp}','',f'응답 {len(pop["events"])}개, 목표 {len(pop["goals"])}개, 지칭 {len(pop["bindings"])}개. 선정 이유: {pop["reason"]}.','', '### QA01 사건별 처리 단계','', '모델 호출의 전송 / 첫 토큰 고정 140ms + 입력 1,000토큰당 10ms + 총 출력 토큰당 2ms를 동일 가정한다. 모두 설계용 예산이며 모델 성능 실측이 아니다. 일반 현재 응답의 첫 내용에는 48토큰, 원본 이력의 의미 관계를 함께 푸는 호출에는 500토큰을 예상한다. 화면 지칭은 A/B 모두 64~96토큰의 최종 GroundingProposal 전체가 완성되고 채택된 후 대상과 처리 범위 카드를 표시하며, 그 뒤 공통 상세 설명 호출 650토큰을 별도로 센다. 이 500은 강제 API 제한이 아닌 의미 판단량의 가정이다. producer 중간 결과는 전체 출력 후 결합에 전달한다. A는 원본 이력과 현재 요청을 한 호출로 해석하며 유효 cache와 부분 patch를 허용한다. B도 native 후보와 부분 재사용, 화면 선행 작업을 허용한다.','', '| 사건 | 구조 경로 | A ms | B ms | B−A ms |','| --- | --- | ---: | ---: | ---: |']
        am={x['event_id']:x for x in ao['events']};bm={x['event_id']:x for x in bo['events']}
        for eid in pop['events']:
            ledger.append(f'| {eid} | {am[eid]["mechanism_class"]} | {am[eid]["first_ms"]} | {bm[eid]["first_ms"]} | {bm[eid]["first_ms"]-am[eid]["first_ms"]} |')
        ledger+=['',f'평균: A {sum(x["first_ms"] for x in ao["events"])}/{len(am)} = {fmt(av[0]*1000,6)}ms; B {sum(x["first_ms"] for x in bo["events"])}/{len(bm)} = {fmt(bv[0]*1000,6)}ms. 오답 시간 가산 / 제외 없음.','', '### QA03 목표별 전체 인과 경로','', '| 목표 | A ms | B ms |','| --- | ---: | ---: |']
        for ag,bg in zip(ao['goals'],bo['goals']):
            assert ag['goal_id']==bg['goal_id'];ledger.append(f'| {ag["goal_id"]} | {ag["via_ms"]} | {bg["via_ms"]} |')
        ledger+=['',f'목표 {len(ao["goals"])}개의 원값 평균: A {fmt(av[2],6)}초 / B {fmt(bv[2],6)}초. 선택한 응답 이외의 같은 목표 후속 사건은 JSON의 additional_goal_events와 goal nodes에 보존한다.','', '### QA02 지칭별 정답률 가정','', '아래는 반복 실행의 성공 횟수가 아니라 각 지칭이 최종 대상 집합과 정확히 맞을 주관적 예상 확률이다. 어려운 사례를 따로 구분하고 단순 평균한다. 후보 확장 뒤 잔여 오류 위험이며 첫 판단 오류를 최종 오답으로 세지 않는다.','', '| 지칭 | 위험 유형 | A % | B % |','| --- | --- | ---: | ---: |']
        for ar,br in zip(ao['accuracy_priors'],bo['accuracy_priors']):ledger.append(f'| {ar["binding_id"]} | {ar["risk_class"]} | {ar["expected_correct_percent"]} | {br["expected_correct_percent"]} |')
        for opt,one,v in [('A',ao,av),('B',bo,bv)]:
            ledger += ['',f'### {opt}: QA04 / QA08 계획 변경 집합','', '**DESIGN_CHANGE_REVIEW**. 아래 package / crate / module ID는 계획 식별자다. 실제 Rust 코드 diff가 아니다. 관련 변경 과제의 공통 UI / adapter 수정도 모두 센다.','', '| 과제 | 수 | 계획 module ID |','| --- | ---: | --- |']
            ledger += [f'| {cid} | {len(ms)} | '+'<br>'.join(f'`{m}`' for m in ms)+' |' for cid,ms in one['change_sets'].items()]
            ledger += ['', '| 독립 변경 쌍 | 공통 모듈 |','| --- | --- |']+[f'| {" / ".join(x["pair"])} | '+('<br>'.join(f'`{m}`' for m in x['modules']) or '없음')+' |' for x in one['overlaps']]
            ledger += ['',f'변경 평균 {fmt(v[3],6)}개; 교집합 평균 {fmt(v[7],6)}개.','']
        ledger+=['### QA05 비용과 QA06 RAM','', '| 안 | 전경 관련 의미 호출 USD | 전체 의미 USD | 공통 음성 USD | 전체 USD / 점수 | 최고 RAM / 점수 |','| --- | ---: | ---: | ---: | ---: | ---: |']
        for opt,one,v in [('A',ao,av),('B',bo,bv)]:
            rt=sum(F(x['cost_usd']) for x in one['model_cost_ledger'][:6]);sub=one['related_foreground_semantic_cost_usd']['decimal']
            ledger.append(f'| {opt} | {fmt(sub,6)} | {fmt(v[4]-rt,6)} | {fmt(rt,6)} | {fmt(v[4],6)} / {score(5,v[4])}점 | {fmt(v[5])} MB / {score(6,v[5])}점 |')
            peak=max(one['simultaneous_ram_ledger'],key=lambda x:x['total_mb']);ledger+=[]
        ledger+=['','전체 비용에는 28개 각 초기 상태의 준비와 갱신을 모두 포함한다. DP-45 B는 초기 56회 의미 관계 생산과 S12 원본 변경의 1회 갱신을 센다. 명시 ID와 전달 receipt 게시에는 모델 호출을 강제하지 않는다. 초기 작업은 2회 × 1.28초 = 2.56초로 입력 제공 전 5초 안에 완료된다는 가정이며 무료 ready store가 아니다. foreground에서 이미 센 갱신은 중복 청구하지 않는다. 원본 가격은 2026-10-14 저장소 snapshot이다.','']
        for opt,one in [('A',ao),('B',bo)]:
            peak=max(one['simultaneous_ram_ledger'],key=lambda x:x['total_mb']);ledger.append(f'- {opt} RAM 최고 시점 {peak["scenario"]}: '+ ' + '.join(f'{k} {v}MB' for k,v in peak['simultaneous_components_mb'].items())+f' = {peak["total_mb"]}MB.')
        ledger+=['','### QA07 부하 증가와 QA09 음성 전달','', '| 안 / 사건 | idle ms | loaded ms | 추가 ms | 부하 지연 구성 ms |','| --- | ---: | ---: | ---: | --- |']
        for opt,one in [('A',ao),('B',bo)]:
            ledger += [f'| {opt} / {x["event"]} | {x["idle_ms"]} | {x["loaded_ms"]} | {x["added_ms"]} | '+', '.join(f'{k} {v}' for k,v in x['components_ms'].items())+' |' for x in one['load_pairs']]
        ledger+=['',f'QA09: 양안 {ao["audio_count_per_pass"]}개 의무 음성 각각 요약 준비 110ms + 생성 / 전송 150ms + 장치 40ms = 300ms, 3점. 화면 / 이력 해석은 채택 전이므로 다시 더하지 않는다. 청취 불가능 시간과 오답 시간 벌점을 제외한다.','']
    return result,'\n'.join(ledger),'\n'.join(matrix),byvariant

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--write',action='store_true');parser.add_argument('--check',action='store_true');args=parser.parse_args()
    a=json.loads(A_PATH.read_text());data=population.load();population.validate(data)
    result,ledger,matrix,variants=build(a,data);assert len(result['summary'])==72
    rationale=json.loads((P/'thought-experiment/rationale.json').read_text());details=[]
    for dp in a['comparison']['dps']:
        r=rationale[str(dp)];pop=a['populations'][str(dp)];assert len(r['qas'])==9
        details += [f'### DP-{dp}','',f'[설계 원문](../{r["source"]}). '+r['structure'],'',f'관련 모집단: 응답 {len(pop["events"])}개 / 완료 목표 {len(pop["goals"])}개 / 지칭 {len(pop["bindings"])}개.','', '| QA | A 예상값 / 점수 | B 예상값 / 점수 | 예상 우열 | 구조를 보고 이해하는 한 줄 |','| --- | ---: | ---: | --- | --- |']
        for i,(judgment,reason) in enumerate(r['qas'],1):
            av=variants[f'{dp}A'][1][i-1];bv=variants[f'{dp}B'][1][i-1]
            if judgment.startswith('동점'):assert av==bv,(dp,i,judgment,av,bv)
            if judgment.startswith('A'):assert av>=bv if i==2 else av<=bv
            if judgment.startswith('B'):assert bv>=av if i==2 else bv<=av
            for row in result['summary']:
                if row['dp']==dp and row['qa']==f'QA{i:02}':
                    row['expected_order']=judgment;row['structural_reason']=reason
                    row['comparison_population_id']=f'DP-{dp}-RELATED-v2'
            vals=[]
            for opt in ['A','B']:
                value=variants[f'{dp}{opt}'][1][i-1];s=score(i,value)
                vals.append(f'{fmt(value,4)} {UNITS[i-1]}<br>{DOTS[s]} **{s}/6**')
            details.append(f'| QA{i:02} {NAMES[i-1]} | '+' | '.join(vals)+f' | {judgment} | {reason} |')
        details.append('')
    report=P/'qa-dp-thought-experiment.md';body=report.read_text()
    for label,content in [('MATRIX',matrix),('DETAILS','\n'.join(details).rstrip())]:
        begin=f'<!-- BEGIN FORECAST {label} -->';end=f'<!-- END FORECAST {label} -->'
        assert body.count(begin)==body.count(end)==1
        before,tail=body.split(begin);_,after=tail.split(end);body=before+begin+'\n'+content+'\n'+end+after
    serialized=readable_json(result)+'\n';assert json.loads(serialized)==result
    outputs={P/'thought-experiment/predictions.json':serialized,P/'thought-experiment/calculation-ledger.md':ledger,report:body}
    for path,content in outputs.items():
        if args.write:path.write_text(content)
        else:assert path.read_text()==content,f'Out of sync: {path}'
    print('PASS: 72 authored forecasts; same A/B related populations; canonical score boundaries; stage DAGs; complete goal closure; full-bundle costs; module intersections; RAM/load arithmetic.')
    print('No model call, PoC, measured timing, measured accuracy or implementation diff was produced.')

if __name__=='__main__':main()
