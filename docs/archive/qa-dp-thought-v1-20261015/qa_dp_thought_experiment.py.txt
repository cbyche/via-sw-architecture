#!/usr/bin/env python3
"""Recompute authored assumptions and score tables only. No VIA/model/PoC execution."""
from pathlib import Path
from fractions import Fraction as F
import argparse
import hashlib
import json
import re
import qa_candidate_population as population
import check_qa_candidate_package as qa

P=population.P
A_PATH=P/'thought-experiment/assumptions.json'
PAIRS=[('C01','C02'),('C01','C03'),('C02','C03'),('C04','C05'),('C04','C06'),('C05','C06')]
NAMES=['첫 응답 지연시간','지칭 대상 식별 정확도','VIA 처리시간','변경 범위','모델 사용 비용','최대 RAM 사용량','부하 추가 지연','병행 수정 중복','음성 전달 지연']
UNITS=['s','%','s','modules/change','USD/bundle','MB','ms','modules/pair','ms']

def rational(value):
    value=F(value)
    return {'numerator':value.numerator,'denominator':value.denominator,'decimal':float(value)}

def fmt(value,places=4):
    return f'{float(value):.{places}f}'.rstrip('0').rstrip('.')

def readable_json(value,level=0):
    """Keep small numerical records on one line; preserve standard JSON."""
    flat=json.dumps(value,ensure_ascii=False)
    if not isinstance(value,(dict,list)) or len(flat)<=180:
        return flat
    pad='  '*(level+1);close='  '*level
    if isinstance(value,dict):
        parts=[pad+json.dumps(k,ensure_ascii=False)+': '+readable_json(v,level+1) for k,v in value.items()]
        return '{\n'+',\n'.join(parts)+'\n'+close+'}'
    return '[\n'+',\n'.join(pad+readable_json(v,level+1) for v in value)+'\n'+close+']'

def score(index,value):
    path=next(P.glob(f'qa{index:02d}-*.md'))
    scores=qa.score_cells(path.read_text())
    hits=[k for k,v in scores.items() if qa.contains(qa.interval(v[2]),value)]
    assert len(hits)==1,(index,value,hits)
    return hits[0]

def event_record(a,e,dp,opt):
    t=a['first_response_ms']; profile=e['reference_semantic_call']['profile'];sid=e['id'].split('-')[0]
    components=dict(t['profiles'][profile])
    mode='agent' if e['kind']=='agent' else e['input_mode']
    components['input_tail']=t['input_tail_by_mode'][mode]
    warm=e['id'] in t['warm_history_events']
    if warm:components['valid_source_cache']=t['warm_history_delta_ms']
    if e['id'] in a['cost']['baseline_extra_reads']['events']:
        components['bounded_rejudgment']=t['baseline_extra_read_ms']
    if sid in t['base_load_added_ms'] and e['id']==sid+'-U01':
        components['background_interference']=t['base_load_added_ms'][sid]
        if opt=='B':components['background_interference']+=a['load_added_deltas_b_ms'][str(dp)][sid]
    if opt=='B':
        if dp==44:components['structural_delta']=t['dp44_b_wiring_delta_ms']
        if dp==42 and e['kind']=='agent':components['structural_delta']=t['dp42_b_agent_delta_ms']
        if dp==47 and profile=='screen':
            b=t['dp47_b']
            components['producer_binding_delta']=b['native_extra_ms'] if e['id'] in b['native_events'] else (b['reuse_extra_ms'] if e['id'] in b['partial_reuse_events'] else b['fresh_extra_ms'])
            if e['id'] in b['expansion_events']:components['producer_expansion_delta']=b['expansion_extra_ms']
        if dp==45 and profile=='history':
            b=t['dp45_b'];components['published_relation_delta']=b['fresh_delta_ms'] if e['id'] in b['fresh_events'] else b['ready_history_delta_ms']
            if warm and e['id'] not in b['fresh_events']:components['already_warm_adjustment']=b['warm_extra_delta_ms']
    first=sum(components.values());assert first>=0
    c=a['complete_response_ms'];tail=c['semantic_tail_by_profile'][profile]
    if opt=='B' and dp==47 and profile=='screen':tail+=c['dp47_b_tail_delta_ms']
    if opt=='B' and dp==45 and profile=='history':tail+=(c['dp45_b_fresh_tail_delta_ms'] if e['id'] in t['dp45_b']['fresh_events'] else c['dp45_b_ready_tail_delta_ms'])
    # Summary speech streams from the adopted short content while full detailed text continues.
    text_tail=tail+c['text_full_after_content_ms']
    audio_tail=c['audio_full_after_content_ms'] if e['response']['voice_required'] else 0
    completion_tail=max(text_tail,audio_tail)+(c['dp44_b_completion_extra_ms'] if opt=='B' and dp==44 else 0)
    return {'event_id':e['id'],'profile':profile,'first_ms_components':components,'first_ms':first,'full_text_tail_ms':text_tail,'full_audio_tail_ms':audio_tail,'completion_tail_ms':completion_tail,'voice_required':e['response']['voice_required'],'correctness_used_to_modify_time':False}

def goal_record(a,s,g,events,dp,opt):
    # Authored precedence algebra. External domain, user speech and playback durations are zero.
    ids=list(g['member_events']); rule=a['goal_dependencies'].get(g['id'],'serial_user_then_agent')
    if rule=='include_intervening_S24-U02':ids.insert(1,'S24-U02')
    byid={e['id']:e for e in s['events']}; nodes=[];fin={};last_user=None;last_agent=None
    handoff=a['complete_response_ms']['local_agent_handoff_ms']+(a['complete_response_ms']['dp42_b_handoff_extra_ms'] if opt=='B' and dp==42 else 0)
    def node(name,duration,parents):
        finish=duration+max([fin[x] for x in parents] or [0]);fin[name]=finish
        nodes.append({'id':name,'duration_ms':duration,'parents':parents,'finish_ms':finish})
    previous_output=None
    for ix,eid in enumerate(ids):
        er=events[eid];e=byid[eid]
        if e['kind']=='user':
            parents=[previous_output] if previous_output else []
            node(eid+':first',er['first_ms'],parents)
            tail=460 if rule=='interrupt_after_first_content' and ix==0 else er['completion_tail_ms']
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

def cost_rows(a,data,dp,opt):
    quoted,_,_,_,_=population.quote(data)
    rows=[{'label':n,'calls':None,'input_text_tokens':None,'input_image_tokens':None,'output_tokens':None,'cost_usd':str(v)} for n,_,_,v in quoted[:6]]
    c=a['cost'];input_price=F(str(c['luna_input_usd_per_million']));out_price=F(str(c['luna_output_usd_per_million']))
    def add(label,calls,txt,img,out):
        val=(input_price*(txt+img)+out_price*out)/1000000
        rows.append({'label':label,'calls':calls,'input_text_tokens':txt,'input_image_tokens':img,'output_tokens':out,'cost_usd':str(float(val)),'cost_exact':rational(val)})
    for s in data['scenarios']:
        for e in s['events']:
            ec=e['reference_semantic_call'];profile=ec['profile'];txt=ec['text_input_tokens'];img=ec['image_count']*ec['image_tokens_each_max'];out=int(F(ec['total_output_including_reasoning_tokens'])*F(c['expected_output_fraction_of_reference']))
            if e['id'] in a['first_response_ms']['warm_history_events']:txt=c['warm_history_input_tokens']
            if opt=='B' and dp==47 and profile=='screen':
                b=c['dp47_b'];native=e['id'] in a['first_response_ms']['dp47_b']['native_events'];reuse=e['id'] in a['first_response_ms']['dp47_b']['partial_reuse_events']
                # Native or reused speech/screen candidates need no model producer on this request.
                if not (native or reuse):add(e['id']+' producer',1,b['producer_input_each'],img,b['producer_output_each'])
                add(e['id']+' binding',1,b['binder_input_each'],0,b['binder_output_each'])
                if e['id'] in a['first_response_ms']['dp47_b']['expansion_events']:add(e['id']+' raw-source expansion',1,b['expansion_input_each'],b['expansion_image_tokens_each'],b['expansion_output_each'])
            else:
                if opt=='B' and dp==45 and profile=='history':txt=c['dp45_b']['request_history_input_each']
                add(e['id']+' semantic',1,txt,img,out)
    extra=c['baseline_extra_reads']
    for eid in extra['events']:
        if opt=='B' and dp==47 and eid in a['first_response_ms']['dp47_b']['expansion_events']:continue
        if opt=='B' and dp==45 and eid in c['dp45_b']['update_events']:continue
        add(eid+' bounded source reread',1,extra['text_input_each'],extra['image_input_tokens_each'],extra['output_each'])
    if opt=='B' and dp==45:
        b=c['dp45_b']
        for s in data['scenarios']:
            n=b['initial_calls_per_scenario'];add(s['id']+' initial publisher',n,n*b['initial_input_each'],0,n*b['initial_output_each'])
        for eid in b['update_events']:add(eid+' publisher update',1,b['update_input_each'],0,b['update_output_each'])
    return rows

def build(a,data):
    records=[];summaries=[];byvariant={}
    binding_ids={b['id'] for s in data['scenarios'] for b in s['bindings']}
    for dp in a['comparison']['dps']:
        for opt in ['A','B']:
            tag=f'{dp}{opt}';events={e['id']:event_record(a,e,dp,opt) for s in data['scenarios'] for e in s['events']}
            goals=[goal_record(a,s,g,events,dp,opt) for s in data['scenarios'] for g in s['goals']]
            errors=list(a['accuracy']['baseline_errors'])
            if opt=='B' and dp==47:
                repl=a['accuracy']['dp47_b_replace'];errors=[x for x in errors if x['binding']!=repl['remove']]+[{'binding':repl['add'],'expected_wrong_of_5':1,'reason':repl['reason']}]
            if opt=='B' and dp==45:errors.append(a['accuracy']['dp45_b_extra'])
            assert len({x['binding'] for x in errors})==len(errors)
            assert all(x['binding'] in binding_ids and 0<=x['expected_wrong_of_5']<=5 for x in errors)
            modules=a['module_sets']['reference' if opt=='A' else tag]
            intersections=[{'pair':list(pair),'modules':sorted(set(modules[pair[0]])&set(modules[pair[1]]))} for pair in PAIRS]
            costs=cost_rows(a,data,dp,opt)
            ram=[]
            for s in data['scenarios']:
                r=a['ram'];c=dict(r['baseline_components']);c['screen_decoded_and_compressed_cache']=r['screen_by_scenario'].get(s['id'],r['default_screen_mb']);c['history_source_cache']=r['history_by_scenario'].get(s['id'],r['default_history_mb'])
                if opt=='B':
                    if dp in [44,45,42]:c.update(r[f'dp{dp}_b_extra'])
                    if dp==47 and any(e['reference_semantic_call']['profile']=='screen' for e in s['events']):c.update(r['dp47_b_extra_screen'])
                ram.append({'scenario':s['id'],'simultaneous_components_mb':c,'total_mb':sum(c.values())})
            loads=[]
            for sid,base in a['first_response_ms']['base_load_added_ms'].items():
                added=base+(a['load_added_deltas_b_ms'][str(dp)][sid] if opt=='B' else 0)
                loaded=events[sid+'-U01']['first_ms'];loads.append({'event':sid+'-U01','loaded_ms':loaded,'idle_ms':loaded-added,'added_ms':added,'repetitions':5})
            values=[F(sum(e['first_ms'] for e in events.values()),120000),F(220-sum(x['expected_wrong_of_5'] for x in errors),220)*100,F(sum(g['via_ms'] for g in goals),76000),F(sum(len(v) for v in modules.values()),6),sum(F(r['cost_usd']) for r in costs),F(max(r['total_mb'] for r in ram)),F(sum(x['added_ms'] for x in loads),4),F(sum(len(x['modules']) for x in intersections),6),F(a['audio_eligible_delay_ms']['total'])]
            assert len(events)==120 and len(goals)==76
            assert sum(e['voice_required'] for e in events.values())==109
            assert all(len(ms)==len(set(ms)) for ms in modules.values())
            assert all(e['correctness_used_to_modify_time'] is False for e in events.values())
            one={'dp':dp,'option':opt,'evidence':'THOUGHT_EXPERIMENT','events':list(events.values()),'goals':goals,'hypothetical_errors':errors,'change_sets':modules,'overlaps':intersections,'model_cost_ledger':costs,'simultaneous_ram_ledger':ram,'load_pairs':loads,'audio_eligible_delay_ms':a['audio_eligible_delay_ms']}
            records.append(one);byvariant[tag]=(one,values)
            for index,value in enumerate(values,1):
                summaries.append({'dp':dp,'option':opt,'qa':f'QA{index:02}','name':NAMES[index-1],'unit':UNITS[index-1],'point_estimate':rational(value),'score_0_to_6':score(index,value),'status':'THOUGHT_EXPERIMENT_NOT_MEASURED','timing_failure_penalty_applied':False})
    assert all(byvariant[f'{dp}A'][1]==byvariant['44A'][1] for dp in a['comparison']['dps'])
    paths=[A_PATH,P/'thought-experiment/rationale.json',P/'common-scenarios.json',*sorted(P.glob('qa[0-9][0-9]-*.md'))]
    # The four A controls are identical. Store their detailed ledger once.
    exported=[r if r['option']=='B' or r['dp']==44 else {
        'dp':r['dp'],'option':'A','evidence':'THOUGHT_EXPERIMENT','detail_reference':'44A'
    } for r in records]
    result={'version':a['version'],'status':a['status'],'provenance':{str(f.relative_to(P)):hashlib.sha256(f.read_bytes()).hexdigest() for f in paths},'summary':summaries,'alternatives':exported}
    out=['# 사고 실험 수치 원장','',f'> {a["version"]} / **전 항목 작성자 예상값. 실측 아님.**', '', '이 문서는 [가정 원본](./assumptions.json)과 공통 시나리오를 산술 집계한 결과다. [해석과 DP별 대응](../qa-dp-thought-experiment.md)을 먼저 읽는다. [전체 사건 / 목표 / 비용 / 메모리 JSON](./predictions.json)은 72개 점추정과 그 분자 및 분모를 보존한다. 소수 자릿수는 계산 재현용이며 측정 정밀도가 아니다.','', '## 1. 72개 예상값과 점수','', '| QA / 단위 | 44A | 44B | 47A | 47B | 45A | 45B | 42A | 42B |','| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
    for i,name in enumerate(NAMES,1):
        cells=[]
        for _,v in byvariant.values():cells.append(f'{fmt(v[i-1],6)} ({score(i,v[i-1])}점)')
        out.append(f'| QA{i:02} {name} / {UNITS[i-1]} | '+' | '.join(cells)+' |')
    out+=['','모든 A 열은 같은 분석 기준 구조다. B는 해당 DP만 바꾼 조건이며 여러 B의 차이를 합산하지 않는다. 점수는 canonical QA 문서의 구간으로 반올림 전에 계산한다.','', '## 2. 시나리오별 시간 원장','', '각 셀은 **QA01 평균 초 / QA03 평균 초**다. 전체 값은 시나리오 평균 28개의 단순평균이 아니라 사건 120개 / 목표 76개의 원값 평균이다. 5회는 동일 점추정을 반복한 기대값이며 실행 관측 5회를 꾸며낸 것이 아니다.','', '| S | 사건 / 목표 | 공통 A | 44B | 47B | 45B | 42B |','| --- | ---: | ---: | ---: | ---: | ---: | ---: |']
    for s in data['scenarios']:
        cells=[]
        for key in ['44A','44B','47B','45B','42B']:
            one,_=byvariant[key];es=[e['first_ms'] for e in one['events'] if e['event_id'].startswith(s['id']+'-')];gs=[g['via_ms'] for g in one['goals'] if g['goal_id'].startswith(s['id']+'-')]
            cells.append(f'{fmt(F(sum(es),1000*len(es)))} / {fmt(F(sum(gs),1000*len(gs)))}')
        out.append(f'| {s["id"]} | {len(s["events"])} / {len(s["goals"])} | '+' | '.join(cells)+' |')
    out+=['','## 3. QA02 예상 오답 위치','', '각 항목은 5회 중 1회 틀린다는 정수 근사 가정이다. 실제 실패 기록 / 특정 반복의 관측이 아니다. 나머지 binding은 5/5로 가정한다. 오답의 시간을 삭제하거나 정답 시간으로 바꾸지 않는다.','']
    for key in ['44A','47B','45B']:
        one,values=byvariant[key];out += [f'### {key}'+(' (47A/45A/42A/44B/42B와 동일)' if key=='44A' else ''),'',f'예상 {220-sum(e["expected_wrong_of_5"] for e in one["hypothetical_errors"])}/220 정답 = {fmt(values[1],6)}%, {score(2,values[1])}점.','']
        out += [f'- `{e["binding"]}`: {e["reason"]}.' for e in one['hypothetical_errors']]
        out.append('')
    out+=['## 4. QA04 / QA08 변경 모듈과 교집합','', '**DESIGN_CHANGE_REVIEW**다. 아래 ID는 앞으로 Rust 구현에서 대응시킬 계획 식별자이며 저장소에 실재하는 Rust 파일이라고 주장하지 않는다. `via / lib / via::...`의 세 부분은 package / crate-target / 전체 module path다. 어댑터, 상태, 출력 소유권 기준으로 분리했다. 부모 module의 자체 연결 코드가 바뀌지 않는다고 가정하며 실제 바뀌면 가산해야 한다.','']
    for key in ['44A','44B','47B','45B','42B']:
        one,v=byvariant[key];out += [f'### {key}'+(' (모든 A 공통)' if key=='44A' else ''),'',f'변경 합 {sum(len(z) for z in one["change_sets"].values())}/6 = {fmt(v[3],6)}개, {score(4,v[3])}점. 교집합 합 {sum(len(z["modules"]) for z in one["overlaps"])}/6 = {fmt(v[7],6)}개, {score(8,v[7])}점.','', '| 변경 | 모듈 수 | 계획 모듈 ID |','| --- | ---: | --- |']
        out += [f'| {cid} | {len(ms)} | '+'<br>'.join(f'`{m}`' for m in ms)+' |' for cid,ms in one['change_sets'].items()]
        out+=['', '| 독립 변경 쌍 | 교집합 |','| --- | --- |']+[f'| {" / ".join(z["pair"])} | '+('<br>'.join(f'`{m}`' for m in z['modules']) or '없음')+' |' for z in one['overlaps']]+['']
    out+=['## 5. QA05 토큰 비용 원장 요약','', '같은 28개 묶음 5회의 예상 평균이다. 이전 참조 견적은 출력 예산을 모두 사용했으나 이번 기대 사용량은 총 출력 예산의 1/4이다. reasoning을 제외한 것이 아니며 전체 출력 사용량의 가정이다. Realtime 6개 청구 항목은 참조 원장과 같다. 오류 후 자동 재시도는 가정하지 않지만 채택 전 재판단과 B 생산 / 갱신은 모두 포함한다.','', '| 안 | 의미 호출 수 | 의미 Text 입력 | 의미 이미지 입력 토큰 | 의미 총 출력 | Realtime USD | 의미 USD | 총 USD / 점수 |','| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
    for key in ['44A','44B','47B','45B','42B']:
        one,v=byvariant[key];rs=one['model_cost_ledger'];semantic=rs[6:];rt=sum(F(x['cost_usd']) for x in rs[:6]);sc=v[4]-rt
        out.append(f'| {key} | {sum(r["calls"] for r in semantic)} | {sum(r["input_text_tokens"] for r in semantic)} | {sum(r["input_image_tokens"] for r in semantic)} | {sum(r["output_tokens"] for r in semantic)} | {fmt(rt,6)} | {fmt(sc,6)} | {fmt(v[4],6)} / {score(5,v[4])}점 |')
    out+=['','개별 호출의 토큰 / 단가 산식은 predictions.json의 `model_cost_ledger`에 있다. 준비부터 마지막 후속 청구까지 범위이며 별도 무상 초기 B 결과를 주지 않는다. 가격은 기존 저장소의 2026-10-14 snapshot이다.','', '## 6. QA06 같은 순간 RAM 최고 구성','', '| 안 | 최고 시나리오 | 같은 순간 구성 MB | 합 / 점수 |','| --- | --- | --- | ---: |']
    for key in ['44A','44B','47B','45B','42B']:
        one,v=byvariant[key];peak=max(one['simultaneous_ram_ledger'],key=lambda x:x['total_mb']);out.append(f'| {key} | {peak["scenario"]} | '+'<br>'.join(f'{k}: {x}' for k,x in peak['simultaneous_components_mb'].items())+f' | {peak["total_mb"]} MB / {score(6,v[5])}점 |')
    out+=['','모든 S의 동시 최고 가정을 JSON에 보존했다. 서로 다른 시점의 프로세스 최고값을 더하지 않는다. UI와 공용 라이브러리의 공유 페이지는 한 번만 세고 외부 앱과 클라우드 가중치는 제외한다. 실제 Windows RAM 계측값이 아니다.','', '## 7. QA07 부하 / 무부하 짝','', '| 안 | S05 loaded / idle | S11 loaded / idle | S18 loaded / idle | S19 loaded / idle | 평균 증가 ms / 점수 |','| --- | ---: | ---: | ---: | ---: | ---: |']
    for key in ['44A','44B','47B','45B','42B']:
        one,v=byvariant[key];out.append(f'| {key} | '+' | '.join(f'{x["loaded_ms"]} / {x["idle_ms"]}' for x in one['load_pairs'])+f' | {fmt(v[6])} / {score(7,v[6])}점 |')
    out+=['','loaded는 QA01의 같은 사건 원값이다. 네 조건 각각 5회 동일 기대값을 사용하고 `max(0, mean_loaded−mean_idle)`를 네 조건에 평균한다.','', '## 8. QA09 음성 전달','', '모든 안에서 109개 의무 음성 × 5회 = 545건에 **300ms**를 가정한다. 채택 후 요약 준비 110ms + 첫 음성 생성 / 전송 150ms + 장치 출력 40ms = 300ms → **3점**. 사용자 발화와 앞 음성이 차지한 시간은 제외한다. 전체 음성 데이터 준비 900ms는 QA03의 경계이며 첫 음성 300ms와 다르다. 원본 grounding 이전 시간은 여기에 더하지 않는다. 내용 오류에 시간 벌점을 넣지 않는 사고 실험 원값이며 기능 검사나 실제 음향 관측 통과를 주장하지 않는다.','']
    return result,'\n'.join(out)

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--write',action='store_true');parser.add_argument('--check',action='store_true');args=parser.parse_args()
    a=json.loads(A_PATH.read_text());data=population.load();population.validate(data)
    result,md=build(a,data)
    assert len(result['summary'])==72
    for r in result['summary']:assert 0<=r['score_0_to_6']<=6
    report=P/'qa-dp-thought-experiment.md'
    body=report.read_text()
    matrix=md.split('## 1. 72개 예상값과 점수\n\n')[1].split('\n\n모든 A 열')[0]
    rationale=json.loads((P/'thought-experiment/rationale.json').read_text())
    details=[]
    summary={(r['dp'],r['option'],r['qa']):r for r in result['summary']}
    for dp in a['comparison']['dps']:
        r=rationale[str(dp)]
        details += [f'### DP-{dp}', '', f'[비교 설계 원본](../{r["source"]}). '+r['structure'], '',
                    '| QA | A 예상값 / 점수 | B 예상값 / 점수 | 예상 우열 | 경로와 조건 |',
                    '| --- | ---: | ---: | --- | --- |']
        assert len(r['qas'])==9
        for i,(judgment,reason) in enumerate(r['qas'],1):
            cells=[]
            for opt in ['A','B']:
                row=summary[(dp,opt,f'QA{i:02}')];point=row['point_estimate']
                cells.append(f'{fmt(F(point["numerator"],point["denominator"]),6)} {row["unit"]} / {row["score_0_to_6"]}점')
            details.append(f'| QA{i:02} {NAMES[i-1]} | '+ ' | '.join(cells)+f' | {judgment} | {reason} |')
        details.append('')
    for label,content in [('MATRIX',matrix),('DETAILS','\n'.join(details).rstrip())]:
        begin=f'<!-- BEGIN FORECAST {label} -->';end=f'<!-- END FORECAST {label} -->'
        assert body.count(begin)==body.count(end)==1
        before,tail=body.split(begin);_,after=tail.split(end)
        body=before+begin+'\n'+content+'\n'+end+after
    serialized=readable_json(result)+'\n'
    assert json.loads(serialized)==result
    outputs={P/'thought-experiment/predictions.json':serialized,P/'thought-experiment/calculation-ledger.md':md,report:body}
    for path,body in outputs.items():
        if args.write:path.write_text(body)
        else:assert path.read_text()==body,f'Out of sync: {path}'
    print('PASS: 72 authored point estimates, canonical scoring, 120 event and 76 goal expectations per option, change-set intersections, cost/RAM/load arithmetic.')
    print('This recalculates document assumptions only. No PoC, models, OS measurement or empirical validation ran.')

if __name__=='__main__':main()
