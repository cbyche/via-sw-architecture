#!/usr/bin/env python3
"""Reproduce ASR-QA-v4 reference arithmetic, never execute a candidate or API."""
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
        if zero_external and n['kind'] in ('EXTERNAL_AGENT_WORK', 'EXTERNAL_USER_WAIT'):
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
    p = data['llm_pricing']; a=data['assumptions']; v=data['voice_pricing']
    llm_unit=cost(2000,200,p)
    semantic=cost(600000,60000,p)
    voice=F(30)*F(str(v['audio_input_per_minute']))+F(10)*F(str(v['audio_output_per_minute']))
    voice+=(F(50000)*F(str(v['text_input_per_million']))+F(10000)*F(str(v['text_output_per_million'])))/1_000_000
    daily=semantic+voice
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
        'evidence_level':'CALCULATED_REFERENCE','not_candidate_results':True,
        'arithmetic':{'llm_2000_200_USD':compact(llm_unit),'generation_only_200_seconds':compact(F(200)/450),'semantic_daily_USD':compact(semantic),'voice_daily_USD':compact(voice),'daily_total_USD':compact(daily),'daily_headroom_USD':compact(F('1.50')-daily),'reference_reaction_seconds':compact(F('.10')+F('.30')+F(100)/450+F('.08')+F('.25'))},
        'dp41_sufficient_typed_evidence':profiles,
        'dp42_four_normal_handoffs_seconds':{'A':'.04','B':'.10','B_minus_A':'.06'},
        'dp44_six_normal_transitions_seconds':{'A':'.06','B':'.05','A_minus_B':'.01'},
        'dp45_past_processing_USD':{'A_40_misses':compact(40*A_miss),'A_10_misses':compact(10*A_miss),'B_20_productions':compact(20*B_prod),'break_even_A_misses':compact(20*B_prod/A_miss),'A40_minus_B20':compact(40*A_miss-20*B_prod)},
        'memory_examples':examples,'score_bands':data['score_bands']}


def render(report):
    a=report['arithmetic']
    lines=['# ASR-QA-v4 참조 계산 검산', '', '> CALCULATED_REFERENCE / 공개 단가와 작성자 가정의 산술 검산 / 실제 모델/Windows/후보 실행 결과 없음', '',
        '[정의 원본](./03-02-quality-attribute-definitions.md)과 [입력 원장](./03-02-poc-inputs.json)을 따른다. 재현 명령은 `.venv/bin/python scripts/architecture/qa_poc_reference.py --check`다.', '',
        '## 기본 예산', '', '| 항목 | 계산값 |', '| --- | --- |',
        f"| 의미 2,000/200 token 1호출 | ${a['llm_2000_200_USD']} |",f"| 200 token의 생성 부분만 | {a['generation_only_200_seconds']}초 |",
        f"| 8시간 의미 비용 | ${a['semantic_daily_USD']} |", f"| 8시간 음성/audio/Text 비용 | ${a['voice_daily_USD']} |",
        f"| 기본 총액 / $1.50 예산 여유 | ${a['daily_total_USD']} / ${a['daily_headroom_USD']} |",f"| QA-02 설명용 반응 예산 | {a['reference_reaction_seconds']}초 |", '',
        '## 41의 충분한 typed 근거 조건', '', 'A 2호출/B 1호출이라는 명시적 call-plan이다. 실제 모델의 판단이나 정답률을 측정한 것이 아니다. B가 부분 해석을 추가하면 두 호출로 바뀔 수 있다.', '',
        '| profile | A (초) | B (초) | A-B (초) |', '| --- | --- | --- | --- |']
    for r in report['dp41_sufficient_typed_evidence']:
        lines.append(f"| {r['profile']} | {r['A_seconds']} | {r['B_seconds']} | {r['A_minus_B_seconds']} |")
    d=report['dp45_past_processing_USD']
    lines += ['', '## 짧은 전이와 cache/생산 비용', '',
        '42의 정상 hand-off 4회는 A 0.04초/B 0.10초이며 차이 0.06초다. 44의 6회 전이는 A 0.06초/B 0.05초이며 차이 0.01초다. 두 차이는 대표 사이클의 0.5초 차이 기준에 못 미친다. 이 일부 구간을 전체 QA 평균으로 소개하지 않는다.', '',
        f"45의 과거 처리만 보면 A 40회 cache miss ${d['A_40_misses']}, A 10회 miss ${d['A_10_misses']}, B 20회 관계 생산 ${d['B_20_productions']}다. A miss {d['break_even_A_misses']}회에서 비용이 같고 빈도에 따라 순위가 바뀐다. 현재 의미/음성 비용은 전체 원장에 추가해야 한다.", '',
        '## 동시 메모리 가정', '', '| DP | A (MiB) | B (MiB) | B-A (MiB) | 가정 |', '| --- | --- | --- | --- | --- |']
    for r in report['memory_examples']:
        lines.append(f"| {r['dp']} | {r['A_mib']} | {r['B_mib']} | {r['B_minus_A_mib']} | {r['assumption']} |")
    lines += ['', '공통 1,792MiB는 예산 항목을 동시에 할당한 설명 예시다. 실제 최소 크기/peak를 확인한 수치가 아니다. 41/44의 차이는 128MiB 기준보다 작고, 42/45는 추가 크기에 대한 민감도를 확인해야 한다.', '',
        '## 검수 범위와 다음 평가의 입력', '',
        '검산은 단가/합계/병렬 DAG/외부 대기/공유 allocation 중복/등급 경계의 재현성을 확인한다. 정확성 540개와 변경 6과제의 실제 결과는 생성하지 않는다. 후보 trace나 구체 변경 patch가 생기면 원지표로 평가하고 이 설명 예시를 결과로 재라벨하지 않는다.', '',
        f"정의 원본 SHA-256: `{report['source_sha256']}`", '']
    return '\n'.join(lines)


def self_check(data):
    assert cost(2000,200,data['llm_pricing']) == F('0.0024')
    # Parallel agent work must not erase independently running VIA work.
    dag=[{'id':'start','parents':[],'seconds':'0.1','kind':'VIA'},
         {'id':'agent','parents':['start'],'seconds':'8','kind':'EXTERNAL_AGENT_WORK'},
         {'id':'semantic','parents':['start'],'seconds':'2','kind':'VIA'},
         {'id':'join','parents':['agent','semantic'],'seconds':'.2','kind':'VIA'}]
    assert dag_time(dag)==F('8.3') and dag_time(dag,True)==F('2.3')
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
    assert F(530,540)*100>=98 and F(529,540)*100<98
    assert F(86,90)*100>=95 and F(85,90)*100<95
    assert F(11,540)*100>2
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
