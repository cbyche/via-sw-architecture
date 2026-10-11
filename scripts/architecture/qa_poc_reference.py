#!/usr/bin/env python3
"""Reproduce ASR-QA-v6 reference arithmetic, never execute a candidate or API."""
import argparse
from fractions import Fraction as F
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parents[2]
DIR = ROOT / 'docs/architecture/12-decisions/decision-packages'
SOURCE = DIR / '03-02-quality-attribute-definitions.md'
INPUT = DIR / '03-02-poc-inputs.json'
REPORT = DIR / '03-02-poc-calculation-report.md'
OUTPUT = DIR / '03-02-poc-reference-calculations.json'


def cost(inp, out, pricing):
    return (F(str(inp))*F(str(pricing['input_per_million'])) +
            F(str(out))*F(str(pricing['output_per_million'])))/1_000_000


def dag_time(nodes, zero_external=False):
    ends = {}
    for n in nodes:
        if n['id'] in ends:
            raise ValueError('Duplicate node id')
        if any(p not in ends for p in n['parents']):
            raise ValueError('Graph must be topological with valid parents')
        duration = F(str(n['seconds']))
        if duration < 0:
            raise ValueError('Negative duration')
        if zero_external and n['kind'] in ('EXTERNAL_AGENT_WORK', 'EXTERNAL_USER_WAIT', 'EXTERNAL_USER_SPEECH'):
            duration = F(0)
        ends[n['id']] = max((ends[p] for p in n['parents']), default=F(0)) + duration
    return max(ends.values(), default=F(0))


def memory_peak(rows):
    # Allocation identities are unique, even when multiple owners reference one buffer.
    if len({r['allocation_id'] for r in rows}) != len(rows):
        raise ValueError('Duplicate allocation identity')
    events = {}
    for r in rows:
        begin, end, size = F(str(r['start'])), F(str(r['end'])), F(str(r['mib']))
        if end <= begin or size < 0:
            raise ValueError('Invalid allocation lifetime')
        events[begin] = events.get(begin, F(0)) + size
        events[end] = events.get(end, F(0)) - size
    now = peak = F(0)
    for t in sorted(events):
        now += events[t]
        peak = max(peak, now)
    return peak


def score(row, value):
    x = F(str(value))
    if x < 0 or (row['direction'] == 'high' and x > 100):
        raise ValueError('Out of domain')
    for i, bound in enumerate(row['bounds']):
        b = F(str(bound))
        if (x >= b if row['direction'] == 'high' else x <= b):
            return 6-i
    return 0


def compact(x, digits=6):
    return format(float(x), f'.{digits}f').rstrip('0').rstrip('.') or '0'


def calculate(data):
    p = data['llm_pricing']; a=data['assumptions']; v=data['voice_pricing']; rate=F(str(data['generation_rate_assumption']['base_tokens_per_second']))
    llm_unit=cost(2000,200,p)
    w=json.loads((DIR/data['common_workload']).read_text());manifest=json.loads((DIR/data['workload_manifest']).read_text());u=manifest['summary']
    semantic=cost(sum(c['count']*c['input_tokens'] for c in w['reference_semantic_calls']),sum(c['count']*c['output_tokens'] for c in w['reference_semantic_calls']),p)
    voice=F(u['voice_input_seconds'],60)*F(str(v['audio_input_per_minute']))+F(u['voice_output_reference_seconds'],60)*F(str(v['audio_output_per_minute']))
    voice+=(F(w['reference_voice_text']['input_tokens'])*F(str(v['text_input_per_million']))+F(w['reference_voice_text']['output_tokens'])*F(str(v['text_output_per_million'])))/1_000_000
    activity=semantic+voice
    profiles=[]
    for profile in data['latency_profiles']:
        call=F(str(profile['ttft_seconds']))+F(200)/F(str(profile['tokens_per_second']))
        A=2*call+F('.04')+F('.03')+F(str(profile['voice_seconds']))+F('.20')
        B=call+F('.04')+F('.07')+F(str(profile['voice_seconds']))+F('.20')
        profiles.append({'profile':profile['id'],'A_seconds':compact(A),'B_seconds':compact(B),'A_minus_B_seconds':compact(A-B),'evidence_level':'CALCULATED_REFERENCE'})
    examples=[]
    base=F(a['common_memory_mib'])
    for r in data['memory_examples']:
        delta=F(str(r['B_delta_mib']))
        examples.append({'dp':r['dp'],'A_mib':compact(base),'B_mib':compact(base+delta),'B_minus_A_mib':compact(delta),'evidence_level':'CALCULATED_REFERENCE','assumption':r['basis']})
    A_miss=cost(4000,200,p); B_prod=cost(6000,400,p)
    return {'contract_version':data['contract_version'],'source_sha256':hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
        'common_workload_sha256':manifest['input_sha256'],'workload_manifest_sha256':hashlib.sha256((DIR/data['workload_manifest']).read_bytes()).hexdigest(),'workload_summary':u,
        'evidence_level':'CALCULATED_REFERENCE','not_candidate_results':True,'llm_pricing':p,'voice_pricing':v,
        'semantic_alternative':{**data['semantic_alternative'],'P21_semantic_USD':compact(cost(124000,12400,data['semantic_alternative'])),'P21_base_with_voice_USD':compact(cost(124000,12400,data['semantic_alternative'])+voice)},
        'qwen_speed_reference':{**data['qwen_speed_reference'],'derived_rows':[{**row,'total_seconds':compact(F(row['input_tokens']+data['qwen_speed_reference']['output_tokens'])/F(row['reported_tokens_per_second'])),'effective_output_tokens_per_second':compact(F(data['qwen_speed_reference']['output_tokens'])*F(row['reported_tokens_per_second'])/F(row['input_tokens']+data['qwen_speed_reference']['output_tokens']))} for row in data['qwen_speed_reference']['rows']]},'generation_rate_assumption':data['generation_rate_assumption'],
        'arithmetic':{'llm_2000_200_USD':compact(llm_unit),'generation_only_200_seconds':compact(F(200)/rate),'semantic_activity_USD':compact(semantic),'voice_activity_USD':compact(voice),'activity_total_USD':compact(activity),'activity_headroom_USD':compact(F(str(w['cost_target_usd']))-activity),'reference_reaction_seconds':compact(F('.10')+F('.30')+F(100)/rate+F('.08')+F('.25')),'reference_short_reaction_seconds':compact(F('.10')+F('.30')+F(50)/rate+F('.08')+F('.25'))},
        'dp41_sufficient_typed_evidence':profiles,
        'dp42_valid_answer_seconds':{'case_id':'S6-04','A':'.010','B':'.025','B_minus_A':'.015','assumption':'정상 ANSWER 한 번의 지역 확정/receipt 반영. 모델 호출/외부 접수 시간은 양안 공통.'},
        'dp44_six_normal_transitions_seconds':{'case_id':'S5-04','A':'.06','B':'.05','A_minus_B':'.01','assumption':'A 중앙 callback 6회×0.010초, B stage 6회×0.005초+join 0.020초. 실제 trace로 전이 수/지연을 대체.'},
        'dp45_past_processing_USD':{'case_ids':['S2-02','S2-04','S2-05'],'A_3_misses':compact(3*A_miss),'A_1_miss':compact(A_miss),'B_4_productions':compact(4*B_prod),'break_even_A_misses':compact(4*B_prod/A_miss),'A3_minus_B4':compact(3*A_miss-4*B_prod),'assumption':'P21 안 과거 근거의 추가 처리만 비교: A 3miss 또는 강한 cache 1miss, B 초기/새 근거/갱신을 합친 관계 생산 4회. 원천 변경4건이 생산4회를 강제하지 않음.'},
        'memory_examples':examples,'score_bands':data['score_bands']}


def render(report):
    a=report['arithmetic']
    lines=['# ASR-QA-v6 참조 계산 검산', '', '> CALCULATED_REFERENCE / 공개 단가와 작성자 가정의 산술 검산 / 실제 모델/Windows/후보 실행 결과 없음', '',
        '[정의 원본](./03-02-quality-attribute-definitions.md)과 [입력 원장](./03-02-poc-inputs.json)을 따른다. 재현 명령은 `.venv/bin/python scripts/architecture/qa_poc_reference.py --check`다.', '',
        '## 모델 근거와 공통 활동 예산', '', '의미 모델은 Google Gemini 3 Flash Preview Standard 입력 $0.50/출력 $3.00 per 1M token이며 출력 단가는 thinking을 포함한다. 생성률 100/200/400 token/s와 TTFT는 작성자 가정으로, 공개 보장/실측이 아니다. 음성은 GPT Realtime의 별도 단가(audio 입력 $32/출력 $64, Text 입력 $4/출력 $16 per 1M token)다. 신규 audio 길이 환산은 이력 audio 재입력/특수 token을 포함한 총 청구가 아니므로 실제 modality별 usage를 합산한다. 공통 모델/설정은 양안에서 동일하다.', '', 'P21은 메커니즘별로 선정한 21개(사용자19/Agent단독2, 집중장면 내부Agent4사건)의 공통 묶음이다. 전체5회만 반복하며 정확성105/630, 반응135, 처리95의 모든 관측을 함께 읽는다. 계획 발화21개의 입력50초, 출력179초는 fixture/작성자 예산이고 실제 사용통계나 실행결과가 아니다.', '', '| 항목 | 계산값 |', '| --- | --- |',
        f"| 의미 2,000/200 token 1호출 | ${a['llm_2000_200_USD']} |",f"| 200 token의 생성 부분만 | {a['generation_only_200_seconds']}초 |",
        f"| P21 의미 비용 | ${a['semantic_activity_USD']} |", f"| P21 음성/audio/Text 비용 | ${a['voice_activity_USD']} |",
        f"| 신규 audio/청구 Text 포함 기본 소계 / $0.50 예산 여유 | ${a['activity_total_USD']} / ${a['activity_headroom_USD']} |",f"| QA-02 직렬 100 token 예시 (1초 초과) | {a['reference_reaction_seconds']}초 |",f"| 유효한 50 token 의미 출력일 때의 예시 | {a['reference_short_reaction_seconds']}초 |", '',
        '## 비교 참고: 가격이 있는 Qwen과 공개 속도', '',
        'Alibaba Cloud Model Studio Singapore/International의 qwen3-30b-a3b-instruct-2507 공식 단가는 입력 $0.20/출력 $0.80 per 1M token이다. 같은 token 예산에서 P21 의미 비용 $0.03472, GPT Realtime 신규 audio/청구 Text 포함 기본 소계 $0.35184다. 의미 모델을 교체한 후보 실행 결과가 아니며 tokenizer/출력/호출/정확성 동등성도 주장하지 않는다.', '',
        'Qwen 공식 H20/SGLang benchmark는 Qwen3-30B-A3B 원본의 (입력+출력)/전체시간이다. 2048 출력과 BF16 입력1: 137.18로부터 전체 약14.94초/유효 출력률137.11을 복원하고, 입력6144: 490.10으로부터 약16.72초/유효 출력률122.53을 복원한다. Gemini 또는 Instruct-2507 API decode rate로 사용하지 않는다. 가격/속도 reference는 다른 배포 조건이다.', '',
        '## 41의 충분한 typed 근거 조건', '', 'S2-01의 충분한 typed 근거라는 조건에서 A 2호출/B 1호출을 둔 지역 call-plan이다. 실제 모델의 판단이나 정답률을 측정한 것이 아니다. B가 부분 해석을 추가하면 두 호출로 바뀔 수 있다.', '',
        '| profile | A (초) | B (초) | A-B (초) |', '| --- | --- | --- | --- |']
    for r in report['dp41_sufficient_typed_evidence']:
        lines.append(f"| {r['profile']} | {r['A_seconds']} | {r['B_seconds']} | {r['A_minus_B_seconds']} |")
    d=report['dp45_past_processing_USD']
    lines += ['', '## 짧은 전이와 cache/생산 비용', '',
        '42의 S6-04 정상 답변 한 번은 A 0.010초/B 0.025초라는 지역 commit/receipt 가정이며 차이 0.015초다. 44의 S5-04 callback/stage 6회는 A 6×0.010=0.060초/B 6×0.005+join 0.020=0.050초이며 차이 0.010초다. 두 차이는 대표 사이클의 0.5초 차이 기준에 못 미친다. 이 일부 구간을 전체 QA 평균으로 소개하지 않는다.', '',
        f"45의 S2-02/04/05 과거 근거 처리만 보면 A 3회 cache miss ${d['A_3_misses']}, 강한 cache의 A 1회 miss ${d['A_1_miss']}, B가 초기/새 근거/갱신을 합쳐 4회 관계 생산하는 가정 ${d['B_4_productions']}다. A miss {d['break_even_A_misses']}회에서 비용이 같고 실제 생산/재사용 빈도에 따라 순위가 바뀐다. 원천 갱신 수가 생산 수를 강제하지 않는다. 이 비용이 공통 예산 호출과 겹치면 교체하고, 별도 호출이면 추가하여 중복 계산하지 않는다.", '',
        '## 같은 P21의 동시 메모리 가정', '', '| DP | A (MiB) | B (MiB) | B-A (MiB) | 가정 |', '| --- | --- | --- | --- | --- |']
    for r in report['memory_examples']:
        lines.append(f"| {r['dp']} | {r['A_mib']} | {r['B_mib']} | {r['B_minus_A_mib']} | {r['assumption']} |")
    lines += ['', '공통 1,792MiB는 예산 항목을 동시에 할당한 설명 예시다. 실제 최소 크기/peak를 확인한 수치가 아니다. 41/44의 차이는 128MiB 기준보다 작고, 42/45는 추가 크기에 대한 민감도를 확인해야 한다.', '',
        '## 검수 범위와 다음 평가의 입력', '',
        '검산은 단가/합계/병렬 DAG/외부 대기/공유 allocation 중복/등급 경계의 재현성을 확인한다. 정확성 630개와 변경 6과제의 실제 결과는 생성하지 않는다. 후보 trace나 구체 변경 patch가 생기면 원지표로 평가하고 이 설명 예시를 결과로 재라벨하지 않는다.', '',
        f"정의 원본 SHA-256: `{report['source_sha256']}`", '']
    return '\n'.join(lines)


def self_check(data):
    assert cost(2000,200,data['llm_pricing']) == F('0.0016')
    result=calculate(data);a=result['arithmetic']
    assert a['activity_total_USD']=='0.41632' and a['activity_headroom_USD']=='0.08368'
    v=data['voice_pricing']
    assert F(v['audio_input_per_minute'])==F(v['audio_input_per_million'])*v['audio_input_tokens_per_second']*60/1_000_000
    assert F(v['audio_output_per_minute'])==F(v['audio_output_per_million'])*v['audio_output_tokens_per_second']*60/1_000_000
    assert result['semantic_alternative']['P21_semantic_USD']=='0.03472'
    assert result['semantic_alternative']['P21_base_with_voice_USD']=='0.35184'
    assert result['workload_summary']['user_input_scenes']==19 and result['workload_summary']['agent_triggered_scenes']==2
    # Parallel agent work must not erase independently running VIA work.
    dag=[{'id':'start','parents':[],'seconds':'0.1','kind':'VIA'},
         {'id':'agent','parents':['start'],'seconds':'8','kind':'EXTERNAL_AGENT_WORK'},
         {'id':'semantic','parents':['start'],'seconds':'2','kind':'VIA'},
         {'id':'join','parents':['agent','semantic'],'seconds':'.2','kind':'VIA'}]
    assert dag_time(dag)==F('8.3') and dag_time(dag,True)==F('2.3')
    speech=[{'id':'speech','parents':[],'seconds':'2','kind':'EXTERNAL_USER_SPEECH'},{'id':'via','parents':[],'seconds':'.3','kind':'VIA'},{'id':'delivery','parents':['speech','via'],'seconds':'.2','kind':'VIA'}]
    assert dag_time(speech)==F('2.2') and dag_time(speech,True)==F('.5')
    assert memory_peak([{'allocation_id':'shared','start':0,'end':2,'mib':100,'owners':['A','B']}, {'allocation_id':'copy','start':1,'end':3,'mib':50}])==150
    assert memory_peak([{'allocation_id':'first','start':0,'end':1,'mib':100},{'allocation_id':'next','start':1,'end':2,'mib':50}])==100
    try:
        memory_peak([{'allocation_id':'same','start':0,'end':1,'mib':1}]*2)
    except ValueError:
        pass
    else:
        raise AssertionError('Duplicate shared allocation must be rejected')
    for r in data['score_bands']:
        b=list(map(lambda x:F(str(x)),r['bounds']))
        assert len(b)==6 and (all(b[i]>b[i+1] for i in range(5)) if r['direction']=='high' else all(b[i]<b[i+1] for i in range(5)))
        for i,x in enumerate(b):
            assert score(r,x)==6-i
            if i<5:
                assert score(r,(x+b[i+1])/2)==5-i
        target=b[-1];epsilon=F(1,1_000_000)
        assert score(r,target-epsilon if r['direction']=='high' else target+epsilon)==0
    assert F(618,630)*100>=98 and F(617,630)*100<98
    assert F(100,105)*100>=95 and F(99,105)*100<95
    assert F(13,630)*100>2
    rows=[r for r in SOURCE.read_text().splitlines() if r.startswith('| **ASR-QA-')]
    assert len(rows)==6 and all(len(r.split('|'))==8 for r in rows)
    assert all(all(c.strip() for c in r.split('|')[1:-1]) for r in rows)
    assert not any(k in '\n'.join(rows) for k in ('미정','검토 예정','정의 초안'))


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--write',action='store_true');parser.add_argument('--check',action='store_true');args=parser.parse_args()
    data=json.loads(INPUT.read_text());self_check(data);report=calculate(data)
    payload=json.dumps(report,ensure_ascii=False,indent=2)+'\n';md=render(report)
    if args.write:
        OUTPUT.write_text(payload);REPORT.write_text(md)
    else:
        assert OUTPUT.read_text()==payload, 'Stale calculation JSON'
        assert REPORT.read_text()==md, 'Stale calculation report'
    print('PASS: six complete QA rows, exact cost/accuracy/score boundaries, parallel attribution and allocation identity checks; reference outputs synchronized')

if __name__=='__main__':main()
