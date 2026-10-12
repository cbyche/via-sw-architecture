"""Readable finished comparison generated from finalized observations."""
from __future__ import annotations
import argparse
from collections import Counter
import json
from pathlib import Path
from .evaluate import read_jsonl


def render(campaign):
    campaign=campaign.resolve(); final=campaign/'final'
    result=json.loads((final/'qa-results.json').read_text());a=result['variants']['A'];b=result['variants']['B']
    manifest=json.loads((campaign/'manifest.json').read_text())
    fmt=lambda v,d=3: f'{v:.{d}f}' if v is not None else '관측 없음'
    clock_cell=lambda row: f"{fmt(row['value'])}초 · {row['n']}" if row['value'] is not None else f"관측 없음 · {row['n']}"
    value=lambda r,i: (r['qa01']['value'] if i==1 else r[f'qa{i:02d}']['cap_mean']['value'] if i in (2,3) else r[f'qa{i:02d}']['value'])
    units={1:'%',2:'초',3:'초',4:'USD/P21',5:'책임/과제',6:'MiB'}
    labels={1:'Request 처리 정확성',2:'상호작용 응답시간 · 실패 cap',3:'VIA Request 처리시간 · 실패 cap',4:'VIA 의미 모델 사용 비용',5:'변경 용이성',6:'로컬 VIA phys_footprint peak'}
    lines=['# DP41 A/B PoC 측정 결과','', '**상태: 측정·판정·집계 완료.** 같은 P21을 방안별 5회 실행했고, 독립 변경 실험 12개를 완료했다. 목표값 달성을 위해 후보 의미나 측정 결과를 보정하지 않았다.','',
           'A는 41A+44A+45A+42A, B는 41B+44A+45A+42A다. 실제 Qwen 의미 호출, transcript 입력과 공통 음성 출력 모형, Mac 실행을 사용했다. 아래 0–6점은 ASR-QA-v6 경계를 이 PoC의 해당 원지표에 적용한 값이며 보호 조건은 별도로 표시한다. 비용은 실제 의미 API usage×고정 가격, 메모리는 Mac 원지표다.','',
           '| QA | A | B | A / B 점수 |','| --- | ---: | ---: | ---: |']
    for i in range(1,7):
        lines.append(f"| {i:02d} {labels[i]} | {fmt(value(a,i),6 if i==4 else 3)} {units[i]} | {fmt(value(b,i),6 if i==4 else 3)} {units[i]} | {a[f'qa{i:02d}']['score']} / {b[f'qa{i:02d}']['score']} |")
    lines += ['', '## 정확성 및 보호 조건','', '| 관측 | A | B |','| --- | ---: | ---: |',
              f"| 통과 조건 / 630 | {a['qa01']['passed']} | {b['qa01']['passed']} |",
              f"| 여섯 조건 모두 통과한 사례 / 105 | {a['qa01']['scenario_passes']} | {b['qa01']['scenario_passes']} |",
              f"| 필수 위반 관측 | {len(a['qa01']['mandatory_violations'])} | {len(b['qa01']['mandatory_violations'])} |"]
    for key in sorted(set(a['qualification'])|set(b['qualification'])):
        lines.append(f"| {key} 보호/자격 충족 | {'통과' if a['qualification'].get(key) else '실패'} | {'통과' if b['qualification'].get(key) else '실패'} |")
    lines += ['', '| 정확성 보호 세부 | A | B |','| --- | --- | --- |']
    for key in a['qa01']['guards']:
        lines.append(f"| {key} | {'통과' if a['qa01']['guards'][key] else '실패'} | {'통과' if b['qa01']['guards'][key] else '실패'} |")
    lines += ['', '낮은 비용이나 짧은 실패 응답만으로 우수 방안을 선정하지 않는다. 정확성·응답·완료의 보호 조건과 각 원지표를 함께 읽으며, 가중 합산 점수는 만들지 않는다.','',
              '## QA02의 세 평균','', '| 방법 | A 평균 · 관측 수 | B 평균 · 관측 수 |','| --- | ---: | ---: |']
    for label,key in [('오답/미전달을 5초로 반영','cap_mean'),('정답 첫 반응만','correct_only_mean'),('오답 포함 실제 첫 핵심 반응','actual_including_wrong_mean')]:
        lines.append(f"| {label} | {clock_cell(a['qa02'][key])} | {clock_cell(b['qa02'][key])} |")
    lines += ['', f"최초 오답 A/B={a['qa02']['initial_wrong_count']}/{b['qa02']['initial_wrong_count']}, 이후 유효 정정={a['qa02']['corrected_count']}/{b['qa02']['corrected_count']}, 유효 반응 누락={a['qa02']['missing_count']}/{b['qa02']['missing_count']}. 실제 시각이 없는 반응에 실측시간을 만들어 넣지 않았다. 유효한 5초 초과 반응은 실제 시간을 유지한다.", '',
              '| 반응시간 보호 관측 | A | B |','| --- | ---: | ---: |',
              f"| 일반 2초 이내 / 전체 · 95% 필요 | {a['qa02']['guards']['ordinary_within_2']} / {a['qa02']['guards']['ordinary_n']} | {b['qa02']['guards']['ordinary_within_2']} / {b['qa02']['guards']['ordinary_n']} |",
              f"| 집중 도착 5초 이내 / 전체 · 모두 필요 | {a['qa02']['guards']['burst_within_5']} / {a['qa02']['guards']['burst_n']} | {b['qa02']['guards']['burst_within_5']} / {b['qa02']['guards']['burst_n']} |",
              f"| 로컬 중단 0.1초 이내 / 전체 · 모두 필요 | {a['qa02']['guards']['local_stop_within_point1']} / {a['qa02']['guards']['local_stop_n']} | {b['qa02']['guards']['local_stop_within_point1']} / {b['qa02']['guards']['local_stop_n']} |",
              f"| 잘못된 첫 반응 또는 유효 반응 누락 | {a['qa02']['missed_endpoint_count']} | {b['qa02']['missed_endpoint_count']} |", '',
              '## QA03의 두 평균','', '| 방법 | A 평균 · 관측 수 | B 평균 · 관측 수 |','| --- | ---: | ---: |']
    for label,key in [('30초 내 미완료를 30초로 반영','cap_mean'),('실제 완료한 사이클만 · 늦은 완료 포함','completed_only_mean')]:
        lines.append(f"| {label} | {clock_cell(a['qa03'][key])} | {clock_cell(b['qa03'][key])} |")
    lines += ['', f"완료 A/B={a['qa03']['completed_count']}/{b['qa03']['completed_count']}, 30초 초과 완료={a['qa03']['late_completed_count']}/{b['qa03']['late_completed_count']}, 관측 종료까지 미완료={a['qa03']['incomplete_count']}/{b['qa03']['incomplete_count']}.", '',
              f"15초 이내 완료 보호 관측: A {a['qa03']['within_15_count']}/95, B {b['qa03']['within_15_count']}/95. 목표 평균 10초, 95%가 15초 이내, 늦은 완료·누락 0을 각각 검사했다.", '',
              '핵심 구절은 judge가 실제 표시·모사 음성에 있는 정확한 구절로 판정하고, 그 첫 문자의 기록된 출력 시점/DAG node를 사용한다. 인사·filler 시작을 핵심 onset으로 대신하지 않는다. VIA 시간은 동일 관측 DAG에서 외부 Agent 작업·사용자 대기·사용자 발화 기간만 0으로 재계산하며, 병렬 join/queue/모델/출력 시간은 남긴다. 원 wall-clock과 계산 경로는 CSV/JSONL에 함께 보존했다.','',
              '## QA04 호출과 비용','', '| 실제 VIA 원장 · 다섯 활동 합 | A | B |','| --- | ---: | ---: |']
    call_rows={v:[] for v in ['A','B']}
    raw_trials=[]
    for worker in sorted((campaign/'workers').iterdir()):
        raw_trials+=read_jsonl(worker/'trials-unjudged.jsonl')
        for c in read_jsonl(worker/'model-calls.jsonl'):call_rows[c['variant']].append(c)
    for label,calculate in [('실제 API attempt 수',lambda rs:len(rs)),('CINTERPRET',lambda rs:sum(r['role']=='CINTERPRET' for r in rs)),('CRESPONSE',lambda rs:sum(r['role']=='CRESPONSE' for r in rs)),('입력 token',lambda rs:sum((r.get('usage') or {}).get('input_tokens',0) for r in rs)),('출력 token',lambda rs:sum((r.get('usage') or {}).get('output_tokens',0) for r in rs)),('재시도 attempt',lambda rs:sum(r['attempt']>1 for r in rs)),('폐기된 반환',lambda rs:sum(bool(r.get('discard_reason')) for r in rs))]:
        lines.append(f"| {label} | {calculate(call_rows['A'])} | {calculate(call_rows['B'])} |")
    lines += ['', f"Qwen: `qwen3.6-35b-a3b`, Singapore/International, 입력 $0.375/M · 출력 $2.25/M. [고정한 Alibaba 가격 근거](https://www.alibabacloud.com/help/en/model-studio/model-pricing). 모델을 한 번만 부르도록 B를 제한하지 않았으며 실제 재조회·부분 판단·확인 질문·폐기를 비용에 포함했다.", '',
              f"OpenAI judge: `gpt-6-luna`, high reasoning. 판정 호출은 VIA 비용에서 분리했다. [모델·가격 근거](https://developers.openai.com/api/docs/models/gpt-6-luna). 본 캠페인 원장 사용량 환산 합계는 Qwen ${result['provider_spend_usd'].get('qwen',0):.6f}, OpenAI ${result['provider_spend_usd'].get('openai',0):.6f}다. 무료 한도/충전 잔액과 구분되는 사용량 환산액이며 음성 API 청구는 없다.", '',
              '## QA05 독립 변경 실험','', '| 변경 과제 | A 책임 수 | B 책임 수 | A / B 변경 책임 |','| --- | ---: | ---: | --- |']
    task_names={'M01':'의미 provider wire 교체','M02':'Agent 명령/receipt wire 교체','M03':'Calendar raw 필드/시간대 교체','M04':'실제 producer 실패 뒤 draft 분기','M05':'Task owner 답변 scope token','M06':'동의 철회·파생·늦은 반환 use fence'}
    for task,name in task_names.items():
        ta,tb=a['qa05']['tasks'][task],b['qa05']['tasks'][task]
        lines.append(f"| {task} {name} | {ta['count']} | {tb['count']} | {', '.join(ta['changed_unit_set']) or '변경 없음'} / {', '.join(tb['changed_unit_set']) or '변경 없음'} |")
    lines += ['', f"과제별 최대 변경 책임 수: A {a['qa05']['maximum']}, B {b['qa05']['maximum']}. 평균 3 이하와 모든 과제 5 이하의 보호 조건을 함께 검사했다."]
    lines += ['', '모든 과제는 같은 고정 원본을 독립 복사해 실행했다. 파일 수 대신 U01~U18의 동일 기능 책임을 세었고, 실제 diff/상위 port 보존/새 기능의 실행 probe/P21의 기록된 의미 재생/두 독립 코드 리뷰로 확정했다. 기존 의미 실패는 회귀 재생에서 그대로 보존한다. M06의 0은 전체 요구를 원본이 충족하는 실행 증거가 있을 때만 인정한다. 새 변경 기능의 실제 Qwen 검증은 별도 원장에 남기며 P21 비용/정확성에 섞지 않는다.','',
              '## QA06 Mac 메모리','', '| 원지표 | A | B |','| --- | ---: | ---: |',
              f"| phys_footprint · 다섯 활동 전체 동시합 최대 | {fmt(a['qa06']['phys_footprint']['peak_mib'])}MiB | {fmt(b['qa06']['phys_footprint']['peak_mib'])}MiB |",
              f"| resident size/RSS · 다섯 활동 전체 동시합 최대 | {fmt(a['qa06']['resident_size_rss']['peak_mib'])}MiB | {fmt(b['qa06']['resident_size_rss']['peak_mib'])}MiB |",
              f"| phys_footprint · 활동별 peak 평균 | {fmt(a['qa06']['phys_footprint']['mean_of_repeat_peaks_mib'])}MiB | {fmt(b['qa06']['phys_footprint']['mean_of_repeat_peaks_mib'])}MiB |"]
    lines += ['', '| Mac 추가 진단 · 주 footprint 점수와 별도 | A | B |','| --- | --- | --- |']
    for key in ('rss_peak_within_6gib_reference','idle_footprint_increase_all_within_16mib_reference'):
        lines.append(f"| {key} | {'통과' if a['qa06']['mac_reference_guards'][key] else '실패'} | {'통과' if b['qa06']['mac_reference_guards'][key] else '실패'} |")
    lines.append(f"| OS 측정 누락 sample | {a['qa06']['mac_reference_guards']['incomplete_os_samples']} | {b['qa06']['mac_reference_guards']['incomplete_os_samples']} |")
    lines += ['', '50ms task_info로 실제 격리 worker를 관측했다. 한 프로세스의 후보·공통 owner/Agent stub·raw recorder를 포함하며 controller와 judge는 제외한다. 20MiB 원문/2,000turn/200 완료 Task/30 source 슬롯과 서로 다른 실제 RGBA 버퍼 15개(311,040,000byte)를 할당했다. 공유 객체는 한 번, 실제 복사본은 OS 사용량 그대로 계산한다. phys_footprint는 프로세스에 귀속되는 물리 메모리 회계, RSS는 현재 resident 페이지다. 공유 페이지를 포함하는 RSS 합을 유일한 물리 RAM으로 해석하지 않는다. 실제 환경/두 지표/기존 MiB 비교 점수 모두 Mac 기준이다.','',
              '## 다섯 쌍의 차이','', '| QA | B−A 평균 차이 | 쌍별 최소 / 최대 | 계약의 실용 차이와 일관성 |','| --- | ---: | ---: | --- |']
    for qa,p in result['paired']['A:B'].items():
        difference=p.get('mean_difference',p.get('candidate_minus_baseline'))
        span=f"{fmt(p['minimum'])} / {fmt(p['maximum'])}" if 'minimum' in p else '독립 변경 6과제'
        unit={'qa01':'%p','qa02':'초','qa03':'초','qa04':'USD/P21','qa05':'책임/과제','qa06_phys_footprint_mib':'MiB','qa06_resident_size_rss_mib':'MiB'}[qa]
        lines.append(f"| {qa} | {fmt(difference,6 if qa=='qa04' else 3)} {unit} | {span} | {'충족' if p.get('consistent_practical_difference',p.get('practical_difference')) else '미충족'} |")
    lines += ['', '차이가 작거나 다섯 쌍에서 방향이 달라지는 축은 동점/변동으로 읽는다. 이 값은 다섯 번 관측과 여섯 변경 과제의 결과이며 통계적 유의성이나 전체 VIA의 A/B 우월성을 뜻하지 않는다.','',
              '## 실제 해석·해결 경로','', '| 다섯 활동 합 | A | B |','| --- | ---: | ---: |']
    mechanisms=json.loads((final/'mechanism-summary.json').read_text())
    for label,key in [('owner 조회 반환','reads'),('조회 item의 cache hit','read_cache_hits'),('확인 질문 요청','clarification_requests'),('출력 계약 거절','schema_rejections'),('해결 실패 사건','resolution_failures'),('늦은 의미 채택 폐기','discarded_meanings'),('의미 반환의 policy 폐기','discarded_model_returns')]:
        lines.append(f"| {label} | {mechanisms['A'][key]} | {mechanisms['B'][key]} |")
    lines += ['', f"A의 논리 모델 호출 목적: `{json.dumps(mechanisms['A']['logical_model_purposes'],ensure_ascii=False)}`. A의 실제 선택: `{json.dumps(mechanisms['A']['a_actions'],ensure_ascii=False)}`.",
              f"B의 논리 모델 호출 목적: `{json.dumps(mechanisms['B']['logical_model_purposes'],ensure_ascii=False)}`. 항목별 상태: `{json.dumps(mechanisms['B']['b_field_states'],ensure_ascii=False)}`.", '',
              '논리 호출 요청 수와 API attempt 수는 재시도·예산 거절에 따라 다를 수 있다. provider의 취소 폐기, 반환 후 의미 채택 폐기는 별도로 기록했다. B의 부분 모델 판단과 갱신 Frame도 실제 경로에 포함한다. [Request별 조회·선택·해결·거절 경로](./request-paths.jsonl)와 [경로 합계](./mechanism-summary.json)를 호출 원장에 연결했다.','',
              '## 사례별 결과','', '| 사례 | A 통과 조건 / 30 | B 통과 조건 / 30 |','| --- | ---: | ---: |']
    normalized=read_jsonl(final/'normalized-trials.jsonl')
    for case in manifest['sequence']:
        totals={v:sum(c['passed'] for r in normalized if r['case_id']==case and r['variant']==v for c in r['conditions'].values()) for v in ['A','B']}
        lines.append(f"| {case} | {totals['A']} | {totals['B']} |")
    lines += ['', '오류 원인 검토는 실제 모델 선택/Frame 해결 상태/거절 사유/수신 payload/최종 전달을 연결해서 한다. 특정 구현의 validator 또는 모델 오류를 구조의 불가능성으로 확대하지 않는다. 유한 계약의 미지원 때문에 필요한 의미를 표현할 수 없는 경우와, 표현 가능한 의미를 모델/코드가 잘못 만든 경우를 구분한다. 상세 조건 판정은 conditions.csv, 후보의 실제 실패/조회/호출 경로는 원장에 보존했다.','',
              '## 고정 계약과 원시 데이터','',
              f"- 계약: `{manifest['campaign_contract']}`, `ASR-QA-v6`, source SHA-256 `{manifest['frozen_source_digest']}`.",
              '- 순서: A/B, B/A, A/B, B/A, A/B. 각 활동의 owner/cache/queue를 독립 초기화하고 timing worker를 순차 실행했다.',
              '- 판정: 보정 사례 16개 → 블라인드 두 prompt pass → 불일치 시 세 번째 trace audit. 같은 judge 모델의 반복을 독립 사람 판정으로 표현하지 않는다.',
              '- [전체 수치와 보호 조건](./qa-results.json), [완료 검수](./completion.json), [조건별 판정](./conditions.csv).',
              '- [반응 raw/세 평균](./reactions.csv), [사이클 raw/두 평균](./cycles.csv), [활동별 결과](./repeat-results.csv).',
              '- [호출 원장 CSV](./model-calls.csv), [변경 책임 CSV](./changes.csv), [인과 시간 재계산](./causal-attribution.jsonl).',
              '- `../workers/`: 원문 gzip, 입력/원천 사건, Context/응답, token/cache/retry/discard, payload/effect, 실제 표시/모사 문자 출력/전달, OS 메모리 JSONL.',
              '- `../changes/`: 동일 baseline/각 변경 source/diff/실행·회귀/두 책임 리뷰. `../evaluation/`: 보정·각 판정·최종 adjudication 원장.',
              '- 준비·중단 기록은 별도 폴더에 보존하고 이 본 측정 모집단/점수에 포함하지 않았다.','']
    path=final/'report.md';path.write_text('\n'.join(lines));return path


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--campaign',type=Path,required=True)
    print(render(parser.parse_args().campaign))
