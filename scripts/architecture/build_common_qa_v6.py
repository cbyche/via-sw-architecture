#!/usr/bin/env python3
"""Extend the preserved v5 analysis with a proposed QA07 case/obligation ledger.

No model calls, simulated successes, new timing calculations or measurement freeze.
"""
import argparse
import copy
import csv
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PARENT = ROOT / 'docs/architecture/12-decisions/decision-packages/current'
BASE = PARENT / 'evaluation/common-qa-v5'
OUT = PARENT / 'evaluation/common-qa-v6'
VERSION = 'COMMON-QA-MODEL-6.0'
STAGES = ['REQUEST_INTERPRETATION', 'RESPONSE_COMPOSITION', 'ACTUAL_DELIVERY']


def read(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')


# Explicit per-fixture oracle requirements; these are evaluator expectations, not outputs.
RESPONSES = [
    'TCP와 UDP의 차이를 간단히 설명한다.',
    '실제 전달한 R1의 결론을 한 문장으로 요약한다. 다른 응답을 요약하지 않는다.',
    '제공된 D1@v1 결론을 설명한다. 원문에 없는 조사 결과를 추가하지 않는다.',
    'TCP의 특징을 설명하고 조사 T1을 유지한다. 조사가 끝났다고 주장하지 않는다.',
    'TCP와 UDP 차이를 설명하고 진행 중인 D1 비교를 유지한다.',
    '제공된 D2 결론을 설명하고 D1 분석을 유지한다. 두 자료의 결론을 섞지 않는다.',
    '제공된 D2의 핵심을 설명하고 기존 업무를 유지한다.',
    '현재 읽기에서 확인한 D1 수치를 사용해 VIA 설명 범위만 비용으로 정정한다. Agent 업무를 수정하지 않는다.',
    '선택된 Q1@v1과 Q2@v1 비교 요청을 안내하고 접수와 실제 결과를 구별한다.',
    '원 안 Q1/Q2만 비교한다. 원 밖 Q3을 포함하지 않는다.',
    '서로 다른 두 시점에 가리킨 Q1/Q2를 비교한다. 현재 화면의 다른 항목으로 바꾸지 않는다.',
    '이미 확정한 Q1/Q2의 납기만 비교한다. 비교 범위를 임의로 넓히지 않는다.',
    'D2 작성 Agent P에게 새 발표를 맡긴다는 요청을 안내한다. 접수만으로 완성됐다고 하지 않는다.',
    'D2의 선호 형식과 외부 고객용 범위를 이번 조사에 적용한다. D3이나 내부 회의 선호로 대체하지 않는다.',
    '이미 확정된 D2 형식을 이번 자료에 적용한다. 불필요한 재확인을 필수 조건으로 만들지 않는다.',
    '최신 F1과 내부 회의용 조건을 반영한다. 내부 회의에 적용하지 않기로 한 D2 선호를 다시 적용하지 않는다.',
    '메일 Q1 질문을 먼저 읽는다. 답을 임의로 만들어 Agent에게 제공하지 않는다.',
    '보고서 R1의 음성 안내만 철회하고 메일 Q1을 읽는다. 보고서 Task 자체를 취소하지 않는다.',
    '메일 Q1을 먼저 읽고 중단한 R0의 남은 유효 내용으로 돌아간다.',
    'R0를 실제 중단 위치부터 이어 읽는다. 이미 전달한 부분을 처음부터 반복하거나 남은 부분을 누락하지 않는다.',
    'Q1/Q2를 D2 표 형식으로 비교하고 별도 보고서 T1을 유지한다.',
    '새 화면 Q1/Q2 비교와 일정 질문 읽기를 각각 연결한다. 일정 질문을 먼저 읽고 결과를 혼동하지 않는다.',
    'D2 형식의 새 자료 작성과 기존 메일 질문 읽기를 연결한다. 입력 전에 정당하게 시작한 질문을 중복 출력하지 않는다.',
    '두 시점의 Q1/Q2와 최신 F1 조건으로 기존 비교 요청을 정정한다. 이전 판본 결과를 현재 결과로 안내하지 않는다.',
]


def make_manifest(base):
    old = {x['id']: x for x in read(BASE / 'oracles.json')['items']}
    prior = {'U08', 'U12', 'U16', 'U17', 'U18', 'U19', 'U20', 'U22', 'U23', 'U24'}
    cases = []
    for i, s in enumerate(base['scenarios']):
        uid = s['id']; cid = 'Q7-' + uid
        fixture = next(c for c in base['cases'] if c['config_id'] == 'BASE' and c['case_id'] == uid)
        obligations = []

        def add(suffix, stage, requirement, **extra):
            obligation = dict(id=cid + ':' + suffix, stage=stage, requirement=requirement,
                              request_ref=cid + ':request', revision='current')
            obligation.update(extra)
            obligations.append(obligation)

        meaning = copy.deepcopy(old[uid]['meaning'])
        if uid == 'U08':
            meaning['request_relation'] = 'revise_VIA_explanation_T1'
        add('I-current', STAGES[0], '최초로 실제 사용한 해석에서 모든 요청, 대상, 제한, 유지와 연결 의미를 충족한다.',
            expected_meaning=meaning, referent_oracle=old[uid]['referents'],
            forbidden='조기 오확정 또는 이후 복구로 최초 오류를 덮어쓰기',
            source_input=uid, legacy_oracle_note='U08의 T1은 v5에 명시된 VIA 설명' if uid == 'U08' else None)
        if uid in prior:
            add('I-origin', STAGES[0], '연결된 원래 요청과 최초 확정 증거를 복원한다. 현재 제어/정정 해석만으로 전체 성공을 인정하지 않는다.',
                source_input=None, expected_meaning=None, prerequisite_status='SOURCE_REQUEST_AND_ORACLE_PENDING')
        add('C-current', STAGES[1], RESPONSES[i],
            allowed='의미를 보존하는 표현, 요약과 사례에 등록된 정상 폐기',
            forbidden='잘못된 Task/판본/수신자 연결, 근거 없는 완료 주장 또는 채택 후 실패 은폐')
        if s['routing'] in ['DELEGATED', 'MIXED']:
            add('C-result', STAGES[1], 'Agent가 실제 제공한 결과/실패 상태와 해당 요청 판본에 맞게 결과 안내를 구성한다. Agent 업무 품질 자체는 별도다.',
                event_ref=uid + ':result', revision='latest_after_correction' if uid in ['U16', 'U24'] else 'current')
        question = next((e for e in fixture['events'] if e['kind'] == 'AGENT_QUESTION'), None)
        if question:
            add('C-question', STAGES[1], '해당 메일/일정 질문의 출처와 현재 유효성을 보존한다. 준비 시점과 전달 시점의 판정을 구별한다.', event_ref=question['id'])
        add('D-screen', STAGES[2], '필수 상세 응답이 올바른 Request/Task UI에 실제 표시된 내용과 판본을 확인한다. accept/queue 기록만으로 통과하지 않는다.', channels=['screen'])
        add('D-audio', STAGES[2], RESPONSES[i] + ' 실제 가청 구간, 순서, 재개 위치와 필수 음성 요약을 확인한다.', channels=['audio'])
        add('D-close', STAGES[2], '마지막 필수 사건과 후속 관측창 종료까지 누락, 중복, 구판본 및 금지 재전달이 없음을 확인한다.',
            channels=['screen', 'audio'], observation_end_s=None)
        if uid == 'U18':
            add('D-withdraw', STAGES[2], 'R1 안내의 정상 폐기를 확인하고 이후 재소비/재개에서도 출력하지 않는다. Q1 전달 의무는 남는다.',
                event_ref='U18:withdrawn', allowed_disposition='NORMAL_DISCARD')
        if uid in ['U19', 'U20']:
            add('D-resume', STAGES[2], '실제 중단까지 전달한 prefix와 재개 offset을 대조한다. 유효 잔여를 빠짐없이 한 번만 전달한다.', channels=['audio'])
        if uid in ['U09', 'U10', 'U11', 'U12', 'U13', 'U14', 'U15', 'U16', 'U21', 'U22', 'U23', 'U24']:
            add('D-result', STAGES[2], 'Agent 결과의 화면 상세와 음성 요약을 실제 전달하고 현재 요청/판본에 연결한다.', channels=['screen', 'audio'], event_ref=uid + ':result')
        blockers = ['CASE_GROUPING_REVIEW', 'REPEAT_COUNT_PENDING', 'FUNCTIONAL_DEADLINE_PENDING',
                    'POST_OBSERVATION_WINDOW_PENDING', 'CONTENT_AND_PHYSICAL_TRACES_ABSENT']
        if uid in prior:
            blockers.append('ORIGIN_REQUEST_EVIDENCE_PENDING')
        if uid in ['U08', 'U16', 'U18', 'U19', 'U20', 'U23', 'U24']:
            blockers.append('RACE_AND_CLOCK_POLICY_PENDING')
        cases.append(dict(case_id=cid, source_case_id=uid, proposed_root_input=uid if uid not in prior else None,
                          linked_inputs=[uid], grouping_reason='독립 fixture의 시작 요청 묶음' if uid not in prior else '선행 요청/정정/전달 제어 연결을 복원한 뒤 사례 경계 검토',
                          independent_fixture=True, shared_context_only=True, routing=s['routing'], text=s['text'],
                          context=s['facts'], obligations=obligations,
                          events=[{k:e.get(k) for k in ['id','kind','profile','source_s','received_s','segment_id','status']} for e in fixture['events']],
                          event_evidence='ANALYTICAL_REFERENCE_NOT_OBSERVATION',
                          source_event_rule='사용자 일정과 외부 업무 길이를 v5대로 유지. 후보별 Agent 결과 수신 시각은 해당 ledger를 참조한다.',
                          required_final_events=[e['id'] for e in fixture['events']],
                          functional_deadline_s=None, post_observation_s=None, clock_tolerance_s=None,
                          contract_ready=False, common_invalid_evidence=None, blockers=blockers))
    return dict(version='COMMON-QA07-CASE-PROPOSAL-6.0', definition='QA07-REQUEST-RESPONSE-v2',
                population_contract='QA07-REQUEST-RESPONSE-POP-v2', audience='EVALUATOR_ONLY',
                status='PROPOSED_NOT_FROZEN', planned_case_families=len(cases), frozen_denominator=None,
                repeat_count=None, target=None, score_bands=None, cases=cases,
                coverage_gaps=['모호한 입력의 실제 명확화 질문과 사용자 답 연결이 없음',
                               'Agent 실패/거부/승인 사건을 실행하는 fixture가 없음',
                               '화면 전용과 수신 범위 변경은 Voice ON 원전의 커버리지 밖',
                               '일부 선행 요청 원문과 최초 확정 및 실제 중단 이력이 없음'])


def evaluate_case(case, record):
    """Aggregate evaluator adjudications, never infer correctness from timings/oracles.

    Multiple judgments retain a proven earlier failure even after recovery.
    Common fixture invalidation requires its own evidence; it is not candidate exclusion.
    """
    if case.get('common_invalid_evidence'):
        return 'INVALID'
    required = {o['id'] for o in case['obligations']}
    judgments = record.get('judgments', [])
    if any(j['obligation_id'] not in required or j['verdict'] not in ['PASS', 'FAIL', 'UNKNOWN'] for j in judgments):
        raise ValueError('Unknown obligation or verdict')
    evidenced = [j for j in judgments if j.get('evidence_ref') and j.get('evaluator_ref')]
    if any(j['verdict'] == 'FAIL' for j in evidenced):
        return 'FAIL'
    latest = {j['obligation_id']: j for j in judgments}
    proved = {oid for oid,j in latest.items() if j['verdict']=='PASS' and j.get('evidence_ref') and j.get('evaluator_ref')}
    if (case['contract_ready'] and required == proved
            and record.get('source_execution_key') and record.get('repeat_id')
            and record.get('observation_closed_evidence')):
        return 'PASS'
    return 'UNKNOWN'


def aggregate(statuses, frozen=False):
    counts = {k: statuses.count(k) for k in ['PASS', 'FAIL', 'UNKNOWN', 'INVALID']}
    n = sum(counts[k] for k in ['PASS', 'FAIL', 'UNKNOWN'])
    q = 100 * counts['PASS'] / n if frozen and n and not counts['UNKNOWN'] else None
    bounds = [100*counts['PASS']/n, 100*(counts['PASS']+counts['UNKNOWN'])/n] if frozen and n else None
    return dict(**counts, draft_slots=len(statuses), N=n if frozen else None, Q=q, bounds=bounds, score=None)


def evaluate_records(cases, records, frozen=False):
    """One configuration's records. Duplicate execution evidence blocks the denominator."""
    by_id={c['case_id']:c for c in cases}
    keys=[r['source_execution_key'] for r in records if r.get('source_execution_key')]
    duplicates={key for key in keys if keys.count(key)>1}
    statuses=[]
    for r in records:
        status=evaluate_case(by_id[r['case_id']],r)
        if status=='PASS' and r.get('source_execution_key') in duplicates:
            status='UNKNOWN'
        statuses.append(status)
    return statuses,aggregate(statuses,frozen and not duplicates)


def build(output):
    base = read(BASE / 'artifacts/ledger.json')
    manifest = make_manifest(base)
    records = []
    for cfg in base['configs']:
        for case in manifest['cases']:
            rec = dict(config_id=cfg['id'], case_id=case['case_id'], repeat_id=None,
                       source_execution_key=None, observation_closed_evidence=None,
                       judgments=[], error_causes=[])
            rec['status'] = evaluate_case(case, rec)
            records.append(rec)
    qa7 = dict(manifest=manifest, records=records,
               summary={cfg['id']:evaluate_records(manifest['cases'],[r for r in records if r['config_id']==cfg['id']])[1] for cfg in base['configs']})
    data = copy.deepcopy(base)
    data.update(version=VERSION, qa07=qa7, generator_sha256=digest(Path(__file__)),
                parent_ledger_sha256=digest(BASE/'artifacts/ledger.json'))
    data['contracts']['qa07'] = manifest['definition']
    data['policy']['qa07_v6'] = '해석/구성/실제 전달 의무와 UNKNOWN 기록. 24개는 사례 구성 제안이며 동결 분모가 아님.'
    write(output/'qa07-case-manifest.json', manifest)
    write(output/'artifacts/ledger.json', data)
    write(output/'artifacts/qa07-results.json', dict(version=VERSION, records=records, summary=qa7['summary']))
    write(output/'artifacts/summary.json', data['summary'])
    write(output/'preserved-v5-sha256.json', {str(p.relative_to(ROOT)):digest(p) for p in sorted(BASE.rglob('*')) if p.is_file()})
    write(output/'definition-sources.json', {str(p.relative_to(ROOT)):digest(p) for p in [PARENT/'qa/qa07-request-meaning-accuracy.md', PARENT/'qa/qa07-evaluation-plan.md']})
    (output/'artifacts/comparison.csv').write_bytes((BASE/'artifacts/comparison.csv').read_bytes())
    with (output/'artifacts/qa07-cases.csv').open('w') as f:
        writer=csv.writer(f,lineterminator='\n');writer.writerow(['config_id','case_id','status','repeat_id','source_execution_key','Q','score'])
        writer.writerows([r['config_id'],r['case_id'],r['status'],'','','',''] for r in records)
    lines=['# QA07 사례와 응답 의무 매핑', '', '[시작](README.md) / [전체 명세](qa07-case-manifest.json)', '',
           '24입력을 24개의 검토용 사례 묶음으로 매핑했다. 같은 ID의 다른 입력은 연속 실행이 아니라 독립 fixture다. 선행 요청이 필요한 묶음은 시작 원문과 최초 해석을 추가해야 하며, 24를 확정 분모로 사용하지 않는다. 7구성에 같은 의무를 적용한다. 실제 실행과 반복 수는 0/미정이다.', '',
           '| 입력 | QA07 사례 | 구성에서 지킬 내용 | 선행 요청 복원 | 의무 수 |', '| --- | --- | --- | --- | ---: |']
    for i,c in enumerate(manifest['cases']):
        lines.append(f"| {c['source_case_id']} | {c['case_id']} | {RESPONSES[i]} | {'필요' if c['proposed_root_input'] is None else '독립 시작 요청'} | {len(c['obligations'])} |")
    lines += ['', '## 전달과 종료', '', '화면과 음성은 별도의 필수 의무다. Agent 결과가 필요한 사례는 결과의 구성과 실제 전달도 각각 확인한다. U18은 정상 폐기와 금지 재전달, U19/U20은 실제 중단 위치와 유효 잔여를 확인한다. 모든 사례에 후속 관측 종료 의무가 있다. QA03의 준비 수용 노드와 QA01의 분석 onset은 실제 전달 성공의 증거가 아니다.', '',
              'U08의 구형 meaning에 있던 revise_T1은 v5에서 명시한 VIA 설명 정정으로 한정했다. 제품 Task 수정으로 해석하지 않는다. 구형 정답 파일은 보존한다. U22/U23의 복합 요청은 한 입력 묶음으로 제안하되 기존 질문의 원래 요청 증거와 사례 연결을 확인하기 전 전체 성공을 판정하지 않는다.', '',
              '## 아직 없는 커버리지', '', *['- '+x for x in manifest['coverage_gaps']], '']
    (output/'02-qa07-case-mapping.md').write_text('\n'.join(lines))
    print(json.dumps(dict(version=VERSION,case_families=len(manifest['cases']),
                         obligations_per_config=sum(len(c['obligations']) for c in manifest['cases']),
                         draft_records=len(records),actual_executions=0),ensure_ascii=False))


if __name__ == '__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,default=OUT)
    build(parser.parse_args().output)
