#!/usr/bin/env python3
"""Render/check the documented QA population. No VIA, model or measurement execution."""
from collections import Counter
from decimal import Decimal, ROUND_CEILING
from pathlib import Path
import argparse
import json
import re
ROOT = Path(__file__).resolve().parents[2]
P = ROOT / 'docs/architecture/12-decisions/decision-packages/qa-candidate-package'
D = Decimal

def load():
    return json.loads((P / 'common-scenarios.json').read_text())

def totals(data):
    scenarios = data['scenarios']
    events = [e for s in scenarios for e in s['events']]
    voice_inputs = [e for e in events if e['input_mode'] == 'voice']
    voice_outputs = [e for e in events if e['response']['voice_required']]
    return dict(scenarios=len(scenarios), user=sum(e['kind']=='user' for e in events),
                agent=sum(e['kind']=='agent' for e in events), responses=len(events),
                goals=sum(len(s['goals']) for s in scenarios), bindings=sum(len(s['bindings']) for s in scenarios),
                ambiguous=sum(len(s['ambiguous_events']) for s in scenarios), voices=len(voice_outputs),
                voice_inputs=len(voice_inputs), input_seconds=sum(e['input_audio_seconds'] for e in voice_inputs),
                output_seconds=sum(e['response']['generated_audio_seconds'] for e in voice_outputs),
                rt_calls=len(voice_inputs)+len(voice_outputs),
                luna_text=sum(e['reference_semantic_call']['text_input_tokens'] for e in events),
                images=sum(e['reference_semantic_call']['image_count'] for e in events),
                luna_output=sum(e['reference_semantic_call']['total_output_including_reasoning_tokens'] for e in events))

def quote(data):
    t=totals(data); p=data['reference_price_snapshot']; m=D(1000000)
    rows=[
      ('Realtime 제어 입력',t['rt_calls']*p['rt_control_tokens_per_call'],D(p['rt_text_input_per_million'])),
      ('Realtime 채택 내용 입력',sum(e['response']['reference_adopted_text_tokens'] for s in data['scenarios'] for e in s['events'] if e['response']['voice_required']),D(p['rt_text_input_per_million'])),
      ('Realtime 사용자 음성 입력',t['input_seconds']*p['rt_audio_input_tokens_per_second'],D(p['rt_audio_input_per_million'])),
      ('Realtime 발화 전달 출력',t['voice_inputs']*p['rt_transcript_output_tokens_per_input'],D(p['rt_text_output_per_million'])),
      ('Realtime 음성의 Text 출력',sum(e['response']['reference_audio_text_tokens'] for s in data['scenarios'] for e in s['events']),D(p['rt_text_output_per_million'])),
      ('Realtime 생성 음성 출력',t['output_seconds']*p['rt_audio_output_tokens_per_second'],D(p['rt_audio_output_per_million'])),
      ('Luna 전체 Text 입력',t['luna_text'],D(p['luna_input_quote_per_million'])),
      ('Luna 이미지 입력',sum(e['reference_semantic_call']['image_count']*e['reference_semantic_call']['image_tokens_each_max'] for s in data['scenarios'] for e in s['events']),D(p['luna_input_quote_per_million'])),
      ('Luna 추론 포함 총 출력',t['luna_output'],D(p['luna_output_per_million']))]
    rows=[(n,u,r,D(u)*r/m)for n,u,r in rows]
    rt=sum(v for _,_,_,v in rows[:6]); luna=sum(v for _,_,_,v in rows[6:]); total=rt+luna
    return rows,rt,luna,total,total.quantize(D('.01'),rounding=ROUND_CEILING)

def validate(data):
    assert data['version']=='COMMON-28-v1'
    assert [s['id'] for s in data['scenarios']]==[f'S{i:02d}' for i in range(1,29)]
    assert data['repetitions']==5
    all_ids=[]; coverage=set()
    for s in data['scenarios']:
        ids={e['id'] for e in s['events']}; gids={g['id'] for g in s['goals']}
        assert len(ids)==len(s['events'])
        coverage.update(s['uc_ids'])
        for g in s['goals']:
            assert g['start_event'] in ids
            assert set(g['member_events']) <= ids
            assert g['start_event'] in g['member_events']
            for eid in g['member_events']:
                assert next(e for e in s['events'] if e['id']==eid)['goal_id']==g['id']
        for e in s['events']:
            assert e['goal_id'] is None or e['goal_id'] in gids
            if e['kind']=='user': assert e['goal_id'] is not None
            r=e['response'];c=e['reference_semantic_call']
            assert r['voice_required']==(r['generated_audio_seconds']>0)
            if not r['voice_required']: assert r['reference_audio_text_tokens']==0 and r['voice_exclusion_reason']
            assert (e['input_audio_seconds']>0)==(e['input_mode']=='voice')
            assert 0<=c['image_count']<=data['execution_profile']['frame_evidence_limit']
            assert c['text_input_tokens']+c['image_count']*c['image_tokens_each_max'] <= data['reference_price_snapshot']['luna_max_unscaled_input_tokens']
            assert '직전 100ms' not in e['trigger']
            all_ids.extend([e['id'],r['id']])
        for b in s['bindings']:
            assert b['event_id'] in ids and b['expected_set']
            assert len(b['expected_set'])==len(set(b['expected_set']))
            assert b['event_id'] not in s['ambiguous_events']
            all_ids.append(b['id'])
        assert set(s['ambiguous_events'])<=ids
        if s.get('load_probe_event'):
            assert s['load_probe_event'] in ids
            assert len([e for e in s['events']if e.get('background_load')])==2
    assert len(all_ids)==len(set(all_ids))
    for f in data['qualification_variants']:
        assert f['base_scenario'] in {s['id']for s in data['scenarios']}
        coverage.update(f['uc_ids'])
    uc_text=(ROOT/'docs/architecture/05-representative-use-cases.md').read_text()
    required=set(re.findall(r'UC-\d\d\.\d+',uc_text))
    assert required==coverage, f'UC missing={required-coverage}, extra={coverage-required}'
    assert len(required)==94
    return totals(data)

def counts_md(data):
    t=totals(data);r=data['repetitions']
    out=['| QA | 공통 입력과 관측 단위 | 1회 관측 건수 | 5회 또는 개발 평가의 집계 |','| --- | --- | ---: | --- |']
    rows=[('QA01','S01~S28의 모든 지정 응답 사건',t['responses'],f'{t["responses"]*r}개 지연의 최대값'),('QA02','같은 목록의 지칭 표현과 최종 revision',t['bindings'],f'{t["bindings"]*r}개 집합 판단의 일치율'),('QA03','같은 목록의 사용자 완료 목표',t['goals'],f'{t["goals"]*r}개 완료 경로의 최대값'),('QA04','공통 기능에 연결한 C01~C06 변경 과제',6,'6개 변경의 평균 Rust 모듈 수'),('QA05','S01~S28 전체 묶음',1,'5개 묶음 총비용의 평균'),('QA06','QA05와 같은 실행과 관측 범위',1,'5개 묶음 전체에서 동시 RAM 최고량'),('QA07','S05/S11/S18/S19의 첫 입력과 무부하 대조',4,'조건별 4×5건. 네 조건 평균 차이의 증가분 평균'),('QA08','C01~C06으로 정한 CO-01~CO-06',6,'6개 변경 쌍의 모듈 교집합 평균'),('QA09','같은 목록의 의무 음성 요약',t['voices'],f'{t["voices"]*r}개 재생 가능 지연의 평균')]
    out += [f'| {qa} | {source} | {n} | {agg} |'for qa,source,n,agg in rows]
    out += ['',f'한 묶음은 **{t["scenarios"]}개 시나리오 / 사용자 입력 {t["user"]}건 / Agent 사건 {t["agent"]}건 / 지정 응답 {t["responses"]}건 / 완료 목표 {t["goals"]}개 / 지칭 판단 {t["bindings"]}개 / 의무 음성 {t["voices"]}건**이다. 실제 모호한 입력 {t["ambiguous"]}건은 별도 기능 oracle로 남긴다.']
    return '\n'.join(out)

def catalog_md(data):
    out=['| S | 대표 시나리오 | 분리한 이유 | 관련 DP | 응답 / 목표 / 지칭 / 음성 |','| --- | --- | --- | --- | --- |']
    for s in data['scenarios']:
        es=s['events']; n=f'{len(es)} / {len(s["goals"])} / {len(s["bindings"])} / {sum(e["response"]["voice_required"]for e in es)}'
        out.append(f'| [{s["id"]}](#{s["id"].lower()}) | {s["title"]} | {s["selection_reason"]} | '+ '/'.join(map(str,s['affected_dps']))+f' | {n} |')
    for s in data['scenarios']:
        out+=['',f'<a id="{s["id"].lower()}"></a>',f'### {s["id"]}. {s["title"]}','',f'**초기 상태:** {s["initial_state"]}',f'**UC:** '+', '.join(s['uc_ids']),'','| 사건 | 입력과 외부 사실 | 발생 조건 | 필수 응답 | 목표 | 음성 |','| --- | --- | --- | --- | --- | --- |']
        for e in s['events']:
            mode=f'Voice {e["input_audio_seconds"]}초' if e['input_mode']=='voice' else ('Text' if e['input_mode']=='text' else 'Agent source')
            out.append(f'| {e["id"]}<br>{mode} | {e["input"]} | {e["trigger"]} | {e["response"]["required_content"]} | {e["goal_id"] or "기존 Task 사건"} | '+('요약' if e['response']['voice_required'] else 'Text만')+' |')
        out+=['','| 완료 목표 | 시작 / 포함 사건 | 필수 완료 조건 |','| --- | --- | --- |']
        out += [f'| {g["id"]} | 시작 {g["start_event"]}<br>'+', '.join(g['member_events'])+f' | {g["completion"]} |'for g in s['goals']]
        if s['bindings']:
            out+=['','| 지칭 판단 | 입력 / 표현 | 정답 대상 집합 |','| --- | --- | --- |']
            for b in s['bindings']:
                rule='<br>'+b['oracle_rule'] if b.get('oracle_rule') else ''
                out.append(f'| {b["id"]} | {b["event_id"]} / {b["expression"]} | '+', '.join(b['expected_set'])+rule+' |')
        if s['ambiguous_events']:out+=['','**확인 질문 필수:** '+', '.join(s['ambiguous_events'])+'. 대상 집합 일치율의 분모에는 넣지 않는다.']
    return '\n'.join(out)

def coverage_md(data):
    out=['| 변형 | 기반 S | 관련 UC | 바꿀 입력과 기대 행동 |','| --- | --- | --- | --- |']
    for f in data['qualification_variants']:out.append(f'| {f["id"]} | {f["base_scenario"]} | '+', '.join(f['uc_ids'])+f' | {f["change_and_oracle"]} |')
    out+=['','| 필수 UC 변형 | 대응 S 또는 F |','| --- | --- |']
    mapping={}
    for s in data['scenarios']:
        for u in s['uc_ids']:mapping.setdefault(u,[]).append(s['id'])
    for f in data['qualification_variants']:
        for u in f['uc_ids']:mapping.setdefault(u,[]).append(f['id'])
    for u in sorted(mapping,key=lambda x:tuple(map(int,re.findall(r'\d+',x)))):out.append(f'| {u} | '+', '.join(mapping[u])+' |')
    return '\n'.join(out)

def quote_md(data):
    t=totals(data);rows,rt,luna,total,budget=quote(data)
    out=['# 공통 28개 시나리오의 모델 비용 참조 견적','',f'> COMMON-28-v1 / COST-COMMON-v4 / 2026-10-14 / **REFERENCE_QUOTE / 미실행**','',
         f'**같은 28개 대표 시나리오 전체를 한 번 수행하는 참조 견적은 ${total:.5f}이며 목표 제안은 ${budget:.2f}/묶음이다.** 이전 20요청 활동의 $0.71을 요청 수에 비례해 환산하지 않았다. [공통 목록](./common-scenarios.md)의 입력과 응답 및 자료량을 각각 합산했다.',
         '', '## 1. 참조 구성과 가격 snapshot','',
         '사용자 지정 `gpt-realtime`과 기존 의미 모델 GPT-5.6 Luna를 유지한다. [이전 공식 출처 조사](./research-notes/qa05-cost-research.md)의 2026-10-14 단가 snapshot을 같은 모델 ID로 사용한다. 이 문서에서 새 API 호출이나 현재 가격 재조회는 하지 않았다. 실제 실행 전 이용 가능성과 단가를 다시 확인하고 양안에 같은 판본을 적용한다.',
         '', '| 모델 | 100만 토큰당 참조 단가 | 근거 |','| --- | --- | --- |',
         '| gpt-realtime | Text 입력 $4 / 출력 $16 / 음성 입력 $32 / 출력 $64 | [공식 모델 페이지](https://developers.openai.com/api/docs/models/gpt-realtime) |',
         '| GPT-5.6 Luna | 입력 $0.25 / 총 출력 $1.20 | [공식 모델 페이지](https://developers.openai.com/api/docs/models/gpt-5.6-luna). 입력 $0.20에 1.25배 cache-write를 보수적으로 적용한 기존 견적 규칙 |',
         '', 'Realtime은 Voice 입력마다 원 발화를 Text로 전달하고 의무 음성 응답마다 채택 내용을 음성화한다. `conversation=none`과 명시적 현재 `input`을 함께 사용한다. 자동 응답과 별도 전사 모델은 참조 구성에 넣지 않는다. 이것은 인식 및 의미 품질 검증이 끝난 제품 구성이라는 뜻이 아니다.',
         '', 'Luna는 사용자 입력 또는 지정 Agent 사건마다 의미 해석과 응답 내용을 한 번 생성하는 참조다. 원본 이력 복원과 인덱스 읽기는 코드 작업으로 둔다. 별도 모델 기반 초기 파생 생산은 이 참조에 없으며 실제 후보에 있으면 전부 비용에 추가한다. 호출 수를 A/B의 의무 구조로 강제하지 않는다.',
         '', '## 2. 입력량의 정의와 한계','',
         f'- 입력 음성 **{t["voice_inputs"]}건 / {t["input_seconds"]}초**, 생성 음성 **{t["voices"]}건 / {t["output_seconds"]}초**다. Text 입력에는 음성 전달 호출이 없다. 진행 UI와 Voice OFF의 응답에는 음성 생성 호출이 없다.',
         f'- Realtime은 발화 전달 {t["voice_inputs"]}회 + 음성 생성 {t["voices"]}회 = **{t["rt_calls"]}회**다. 각 호출 제어 입력 1,000토큰, 발화 전달 출력 100토큰, 음성 생성의 채택 내용 입력 500토큰과 동반 Text 출력 50토큰은 기존 프로토콜 견적 가정이다.',
         '- 초 단위 음성은 [공식 음성 비용 설명](https://developers.openai.com/api/docs/guides/voice-latency-cost?voice-api=realtime)의 입력 약 10토큰/초와 생성 약 20토큰/초로 환산한다. 이는 usage 실측이 아니다. JSON의 길이는 모델에 제출할 유효 발화 규격이며 말미 무음으로 늘리지 않는다. 녹음 제작 시 실제 발화 구간과 제출 길이를 확인한다.',
         '- 생성한 뒤 중단하거나 버린 음성도 실제 청구에는 포함한다. S21/S23의 이전 클립은 초기 원본 fixture여서 새 생성 비용은 없지만 재생 점유와 RAM에 포함한다.',
         '- 모든 Text 입력량은 지시문과 schema 및 직렬화 wrapper와 현재 요청 및 해당 자료까지 포함한 총량 가정이다. 화면과 이력 및 Task별 payload를 실제로 제작해 tokenizer로 센 값은 아니다. 부족하면 결과를 보기 전에 공통 입력과 견적을 함께 개정한다.',
         '', '| 의미 호출 profile | 총 Text 입력 가정 | 추론 포함 총 출력 가정 | 범위 |','| --- | ---: | ---: | --- |',
         '| simple | 2,000 | 800 | 현재 일반 질문과 응답 규칙 |','| screen | 4,000 | 1,600 | 발화와 포인터 및 선택 시각과 화면 metadata. 이미지는 별도 |','| history | 24,000 | 1,600 | 원본 200턴과 완료/진행 Task 및 관련 source를 담는 공통 입력 예산 |','| task | 8,000 | 1,200 | Task 상태와 관련 요청 및 외부 사실과 제어 정보 |','| lookup | 8,000 | 1,200 | 범위가 정해진 허용 자료와 조회 결과 |',
         '', '이미지는 각 사건에 등록한 장수를 각각 입력한 것으로 계산한다. 같은 화면을 다른 호출에 다시 넣으면 다시 센다. [공식 이미지 계산 규칙](https://developers.openai.com/api/docs/guides/images-vision)의 기존 Luna high 최대 3,000토큰/장을 적용한다. 실제 tokenizer 값이나 특정 구조의 최소 입력량이라는 주장은 아니다. 어느 호출도 기존 272,000토큰 경계를 넘지 않는다.',
         '', '## 3. 항목별 합산','', '| 청구 항목 | 총 참조 토큰 | 100만 토큰당 단가 | 비용 |','| --- | ---: | ---: | ---: |']
    out += [f'| {n} | {u:,} | ${r} | ${v:.5f} |'for n,u,r,v in rows]
    out += [f'| **Realtime 소계** | | | **${rt:.5f}** |',f'| **Luna 소계** | | | **${luna:.5f}** |',f'| **전체 합계** | | | **${total:.5f}** |',f'| **센트 단위 올림 목표** | | | **${budget:.2f}** |','',
            '각 S를 먼저 반올림하지 않고 전체 합산 뒤 한 번 올린다. 소비자 구독료나 월 사용량 또는 삼성의 승인 예산으로 해석하지 않는다. 같은 기능의 범용 API 참조 구성 수준 이내로 모델 원가를 관리한다는 제품 요구 제안이다.',
            '', '## 4. 시나리오별 견적 원장','', '| S | Voice 입력 건 / 초 | 음성 생성 건 / 초 | 의미 호출 | Text 입력 | 이미지 입력 회수 | 추론 포함 출력 | 견적 USD |','| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
    for s in data['scenarios']:
        one=dict(data,scenarios=[s]);z=totals(one);cost=quote(one)[3]
        out.append(f'| {s["id"]} | {z["voice_inputs"]} / {z["input_seconds"]} | {z["voices"]} / {z["output_seconds"]} | {z["responses"]} | {z["luna_text"]:,} | {z["images"]} | {z["luna_output"]:,} | {cost:.5f} |')
    out+=['','## 5. 실제 QA05와의 관계','',f'실제 QA05는 **28개 묶음 전체의 모든 청구를 5회 반복한 평균**이다. 참조의 ${budget:.2f}는 목표 제안이고 현재 측정값은 없다. 준비와 갱신 및 후보 확장과 실패와 재시도와 후속 청구를 모두 센다. 참조에 없는 호출도 실제 비용에서는 제외하지 않는다. 기능 미완료와 usage 누락은 저비용 성공이 아니다.',
          '',f'`COST-COMMON-v4` 점수는 목표의 100% / 90% / 80% / 70% / 60% / 50%를 동일 규칙으로 사용한다. 이전 20요청 점수와 비교하지 않는다. 최소 구분 제안은 목표의 5%인 ${budget*D(".05"):.4f}이며 실측 전 비교 규칙이다.',
          '', '정적 검산은 `python3 scripts/architecture/qa_candidate_population.py --check`로 수행한다. 목록과 합계 및 문서 동기화만 확인하며 API 기능 동등성과 음성 길이 및 토큰 입력의 충분성을 증명하지 않는다.']
    return '\n'.join(out)+'\n'

def replace_block(body, name, content):
    start=f'<!-- BEGIN COMMON {name} -->';end=f'<!-- END COMMON {name} -->'
    assert body.count(start)==body.count(end)==1
    a,b=body.split(start);_,c=b.split(end)
    return a+start+'\n'+content+'\n'+end+c

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--write',action='store_true');parser.add_argument('--check',action='store_true');args=parser.parse_args()
    data=load();t=validate(data)
    path=P/'common-scenarios.md';body=path.read_text()
    for label,content in [('COUNTS',counts_md(data)),('CATALOG',catalog_md(data)),('COVERAGE',coverage_md(data))]:body=replace_block(body,label,content)
    outputs={path:body,P/'common-model-cost-quote.md':quote_md(data)}
    for p,expected in outputs.items():
        if args.write:p.write_text(expected)
        else:assert p.exists() and p.read_text()==expected,f'Out of sync: {p}'
    print(json.dumps(t,ensure_ascii=False)); print(f'Quote={quote(data)[3]}, budget={quote(data)[4]}')
    print('PASS: 28 documented scenarios, 94 UC variant links, event/goal/binding references and document/quote arithmetic. No runtime or empirical validation.')
if __name__=='__main__':main()
