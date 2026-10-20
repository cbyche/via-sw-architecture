#!/usr/bin/env python3
"""Offline, source-bound QA02 reanalysis. Never calls a candidate or a model."""
from __future__ import annotations

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
EVIDENCE = REPO / 'results/architecture-evaluation/current/common-supplement6-20261019'
OUT = EVIDENCE / 'qa02-regrade-v1'
POOL = EVIDENCE / 'joint-recovery184-v1/postanalysis/recovery-pool-v1/analysis.json'
PARENT = EVIDENCE / 'joint-formal-v1/postanalysis/stopped258-input-assembly-v1/joint-analysis.json'


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def save(path, value):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def user_payload(call):
    request = read(call['request_file'])
    messages = [m for m in request['messages'] if m['role'] == 'user']
    assert len(messages) == 1 and isinstance(messages[0]['content'], str)
    return json.loads(messages[0]['content'])


def extract():
    compact = read(EVIDENCE / 'preservation-recovery184-v1/summary.json')
    assert digest(POOL) == compact['full_analysis_sha256']
    pool = read(POOL)
    assert digest(PARENT) == pool['parent_analysis_sha256']
    refs = {x['run_id']: x for x in read(PARENT)['trace_provenance']}
    refs.update({x['run_id']: x for x in pool['recovery_trace_provenance']})
    predictions, provenance = [], []
    for population, key in [('COMMON9', 'primary_common9'), ('COMMON_SUPPLEMENT6', 'supplement6')]:
        for result in pool[key]['results']:
            if not result['qa02']:
                continue
            ref = refs[result['run_id']]
            trace_path = Path(ref['trace_path'])
            assert digest(trace_path) == ref['sha256']
            trace = read(trace_path)
            calls = []
            for call in trace['calls']:
                if call.get('stage') == 'meaning_interpret_and_respond':
                    calls.append((call, user_payload(call)))
            for old in result['qa02']:
                rowkey = hashlib.sha256((population + ':' + result['run_id'] + ':' + old['id']).encode()).hexdigest()[:20]
                adoption = next((e for e in trace['events'] if old.get('adoption_event_id') is not None
                                 and e.get('event_id') == old['adoption_event_id']), None)
                assert bool(adoption) == (old['status'] == 'ADOPTED')
                data = adoption['data'] if adoption else {}
                if adoption:
                    choices = [(c, p) for c, p in calls if p.get('event', {}).get('id') == old['event_id']
                               and c.get('terminal_ns', 0) <= adoption['observed_ns']]
                    assert choices, (rowkey, 'No actual meaning input before adoption')
                    call, payload = max(choices, key=lambda x: x[0]['terminal_ns'])
                else:
                    call, payload = {}, {}
                captured = next((e['data']['text'] for e in trace['events']
                                 if e['kind'] == 'INPUT_CAPTURED' and e.get('data', {}).get('stimulus_id') == old['event_id']), None)
                public = payload.get('public_state', {})
                row = {'key': rowkey, 'binding_id': old['id'], 'input_id': old['event_id'],
                       'reference_expression': old['expression'], 'actual_input': captured,
                       'adoption_event_id': old.get('adoption_event_id'), 'adopted_revision': old.get('adopted_revision'),
                       'plan': data.get('plan'), 'source_catalog': payload.get('source_catalog', []),
                       'delivered': public.get('delivered', []), 'requests': public.get('requests', []),
                       'tasks': public.get('tasks', []),
                       'actual_originals': payload.get('actually_supplied_originals', [])}
                predictions.append(row)
                provenance.append({'key': rowkey, 'population': population, 'configuration': result['configuration'],
                                   'episode': result['episode'], 'rep': result['rep'], 'run_id': result['run_id'],
                                   'trace_path': str(trace_path.relative_to(REPO)), 'trace_sha256': ref['sha256'],
                                   'meaning_request_path': str(Path(call['request_file']).relative_to(REPO)) if call else None,
                                   'meaning_request_sha256': digest(call['request_file']) if call else None,
                                   'old': old})
    assert len(predictions) == len({x['key'] for x in predictions}) == 651
    predictions.sort(key=lambda x: x['key'])
    provenance.sort(key=lambda x: x['key'])
    save(OUT / 'review-inputs.json', predictions)
    save(OUT / 'provenance.json', provenance)
    save(OUT / 'extraction.json', {'method_sha256': digest(HERE / 'METHOD.md'),
        'pool_sha256': digest(POOL), 'rows': len(predictions),
        'review_input_sha256': digest(OUT / 'review-inputs.json'),
        'provenance_sha256': digest(OUT / 'provenance.json'),
        'expected_targets_in_review_inputs': False, 'actual_api_calls': 0})
    print('Extracted 651 anonymous adopted records with source hashes; no target answers or aggregate scores.')


def canonical(reference, catalog):
    """Resolve an actual reference to source, version and identical physical extent."""
    if not isinstance(reference, str):
        raise ValueError('NON_STRING_TARGET')
    if ':v' in reference:
        source_id, rest = reference.rsplit(':v', 1)
        version, extent = rest.split(':', 1)
        versions = [s for s in catalog if s['id'] == source_id and s['revision'] == int(version)]
        if len(versions) != 1:
            raise ValueError('UNKNOWN_SOURCE_VERSION:' + reference)
        span = versions[0].get('extents', {}).get(extent)
        if not span or len(span) != 2:
            raise ValueError('UNKNOWN_EXTENT:' + reference)
        return (source_id, int(version), int(span[0]), int(span[1]))
    current = [s for s in catalog if s['id'] == reference and s.get('is_current')]
    if len(current) != 1:
        raise ValueError('UNKNOWN_CURRENT_SOURCE:' + reference)
    source = current[0]
    return (reference, source['revision'], 1, source['line_count'])


def norm(value):
    return ' '.join(str(value).split()).rstrip('.。')


def associate(row, expressions, reference_counts):
    if row['plan'] is None:
        return [], 'NO_ADOPTION', []
    allowed = {norm(x) for x in expressions[row['binding_id']]}
    bindings = row['plan'].get('bindings', [])
    matched = [i for i, b in enumerate(bindings) if norm(b.get('expression', '')) in allowed]
    if len(matched) == 1:
        return bindings[matched[0]].get('targets', []), 'EXPLICIT_SEMANTIC_BINDING', matched
    if len(matched) > 1:
        sets = {tuple(sorted(b.get('targets', []))) for b in (bindings[i] for i in matched)}
        if len(sets) == 1:
            return list(next(iter(sets))), 'DUPLICATE_IDENTICAL_BINDING', matched
        return [], 'CONFLICTING_BINDINGS', matched
    targets = row['plan'].get('targets') or []
    if reference_counts[row['input_id']] == 1 and len(targets) == 1:
        return targets, 'ADOPTED_PLAN_TARGETS', []
    return [], 'ATTRIBUTION_REVIEW', []


def predict():
    rows = read(OUT / 'review-inputs.json')
    expressions = read(HERE / 'associations.json')['expressions']
    # Unique fixed reference IDs per input, never candidate-created counts.
    reference_ids = {}
    for row in rows:
        reference_ids.setdefault(row['input_id'], set()).add(row['binding_id'])
    counts = {k: len(v) for k, v in reference_ids.items()}
    annotations_path = HERE / 'manual-associations.json'
    manual = read(annotations_path)['rows'] if annotations_path.exists() else {}
    output = []
    for row in rows:
        raw, method, indices = associate(row, expressions, counts)
        annotation = manual.get(row['key'])
        if annotation:
            assert method == 'ATTRIBUTION_REVIEW', (row['key'], 'Manual override of automatic attribution forbidden')
            raw, method = annotation['targets'], 'AUTHOR_SOURCE_ATTRIBUTION'
        canonical_targets, error = [], None
        try:
            canonical_targets = sorted(set(canonical(t, row['source_catalog']) for t in raw))
        except (ValueError, KeyError) as exc:
            error = str(exc)
        output.append({'key': row['key'], 'binding_id': row['binding_id'],
                       'association': method, 'binding_indices': indices,
                       'raw_targets': raw, 'canonical_targets': canonical_targets,
                       'canonical_error': error, 'manual_evidence': annotation})
    save(OUT / 'predicted-targets.json', output)
    save(OUT / 'prediction-freeze.json', {'method_sha256': digest(HERE / 'METHOD.md'),
        'associations_sha256': digest(HERE / 'associations.json'),
        'manual_associations_sha256': digest(annotations_path) if annotations_path.exists() else None,
        'source_roles_sha256': digest(HERE / 'source-roles.json') if (HERE / 'source-roles.json').exists() else None,
        'review_input_sha256': digest(OUT / 'review-inputs.json'),
        'predictions_sha256': digest(OUT / 'predicted-targets.json'),
        'rows': len(output), 'stage': 'PREDICTION_EXTRACTION_BEFORE_CORRECTED_AGGREGATION',
        'association_counts': dict(Counter(x['association'] for x in output)),
        'actual_api_calls': 0})
    print(json.dumps({'predicted_rows': len(output), 'association_counts': dict(Counter(x['association'] for x in output))}))


def intended_targets(row, old):
    expected = old['expected_set']
    if expected == ['TA:revised-result']:
        # Resolve the placeholder from actual earlier U02 Task, result source and delivery.
        requests = [r for r in row['requests'] if r.get('origin_event_id') == 'S18-U02']
        aliases = []
        for request in requests:
            task_id = request.get('task_id')
            for task in row['tasks']:
                if task_id and task.get('task_id') == task_id and task.get('result_source_id'):
                    deliveries = [d for d in row['delivered'] if d.get('stimulus_id') == 'S18-A01'
                                  and d.get('task_id') == task_id and d.get('goal_id') == request.get('goal_id')
                                  and d.get('phase') == 'AGENT_RESULT']
                    if deliveries:
                        aliases.append(task['result_source_id'])
        aliases = sorted(set(aliases))
        if len(aliases) != 1:
            raise ValueError('INTENDED_REVISED_RESULT_NOT_AVAILABLE')
        return sorted(set(canonical(t, row['source_catalog']) for t in aliases)), {
            'placeholder': 'TA:revised-result', 'actual_source_ids': aliases,
            'lineage': 'Earlier S18-U02 request Task -> result_source_id -> actual S18-A01 delivery'}
    return sorted(set(canonical(t, row['source_catalog']) for t in expected)), None


def score():
    extraction = read(OUT / 'extraction.json')
    frozen = read(OUT / 'prediction-freeze.json')
    assert frozen['method_sha256'] == digest(HERE / 'METHOD.md')
    assert frozen['associations_sha256'] == digest(HERE / 'associations.json')
    assert frozen['manual_associations_sha256'] == digest(HERE / 'manual-associations.json')
    assert frozen['source_roles_sha256'] == digest(HERE / 'source-roles.json')
    assert frozen['predictions_sha256'] == digest(OUT / 'predicted-targets.json')
    assert extraction['review_input_sha256'] == frozen['review_input_sha256'] == digest(OUT / 'review-inputs.json')
    assert extraction['provenance_sha256'] == digest(OUT / 'provenance.json')
    inputs = {x['key']: x for x in read(OUT / 'review-inputs.json')}
    provenance = {x['key']: x for x in read(OUT / 'provenance.json')}
    roles = read(HERE / 'source-roles.json')['rows']
    decisions = []
    for prediction in read(OUT / 'predicted-targets.json'):
        key = prediction['key']
        row, original = inputs[key], provenance[key]
        reason, expected, alias = 'MATCH', None, None
        try:
            if row['plan'] is None:
                reason = 'NO_ADOPTION'
            elif not prediction['raw_targets']:
                reason = 'NO_ATTRIBUTABLE_TARGET'
            elif prediction['canonical_error']:
                reason = 'INVALID_TARGET'
            else:
                expected, alias = intended_targets(row, original['old'])
                if 'S26:summary:body-only' in {t[0] for t in prediction['canonical_targets']}:
                    role = roles.get(key)
                    assert role and role['role'] == 'actual_delivered_summary', (key, 'Unreviewed summary role')
                    index = int(role['pointer'].rsplit('/', 1)[1])
                    assert row['delivered'][index]['text'] == role['actual_text']
                    assert row['delivered'][index]['delivery_source_id'] == role['source_id']
                if prediction['canonical_targets'] != [list(t) for t in expected]:
                    reason = 'WRONG_TARGET_SET'
        except ValueError as exc:
            reason = str(exc)
        old = original['old']
        decisions.append({**{k: original[k] for k in ['key', 'population', 'configuration', 'episode', 'rep', 'run_id', 'trace_path', 'trace_sha256']},
            'binding_id': row['binding_id'], 'input_id': row['input_id'], 'expression': row['reference_expression'],
            'actual_input': row['actual_input'], 'adoption_event_id': row['adoption_event_id'],
            'old_correct': old['correct'], 'old_status': old['status'], 'old_expression_match_count': old['matching_binding_count'],
            'old_actual_set': old['actual_set'], 'old_expected_set': old['expected_set'],
            'correct': reason == 'MATCH', 'reason': reason, 'changed': old['correct'] != (reason == 'MATCH'),
            'association': prediction['association'], 'actual_bindings': (row['plan'] or {}).get('bindings', []),
            'plan_targets': (row['plan'] or {}).get('targets', []),
            'predicted_raw_set': prediction['raw_targets'], 'predicted_canonical_set': prediction['canonical_targets'],
            'expected_canonical_set': expected, 'dynamic_alias_proof': alias,
            'manual_evidence': prediction['manual_evidence'], 'source_role_review': roles.get(key)})
    configs = ['AAAA', 'BAAA', 'CAAA', 'ABAA', 'AABA', 'AAAB', 'AAAC']
    summary = {}
    for population, denominator in [('COMMON9', 75), ('COMMON_SUPPLEMENT6', 18)]:
        summary[population] = {}
        for config in configs:
            records = [d for d in decisions if d['population'] == population and d['configuration'] == config]
            assert len(records) == denominator
            assert len({(d['binding_id'], d['rep']) for d in records}) == denominator
            summary[population][config] = {'correct': sum(d['correct'] for d in records), 'planned': denominator,
                'accuracy_percent': 100 * sum(d['correct'] for d in records) / denominator,
                'original_correct': sum(d['old_correct'] for d in records),
                'changed_to_correct': sum(d['changed'] and d['correct'] for d in records),
                'changed_to_incorrect': sum(d['changed'] and not d['correct'] for d in records),
                'reasons': dict(Counter(d['reason'] for d in records)),
                'scenarios': {sid: {'correct': sum(d['correct'] for d in records if d['episode'] == sid),
                                     'planned': sum(d['episode'] == sid for d in records)}
                              for sid in sorted({d['episode'] for d in records})}}
    pairs = []
    for population in summary:
        left = {(d['binding_id'], d['rep']): d for d in decisions
                if d['population'] == population and d['configuration'] == 'AAAA'}
        for config in configs[1:]:
            right = {(d['binding_id'], d['rep']): d for d in decisions
                     if d['population'] == population and d['configuration'] == config}
            assert left.keys() == right.keys()
            counts = Counter((left[k]['correct'], right[k]['correct']) for k in left)
            pairs.append({'population': population, 'left': 'AAAA', 'right': config,
                'both_correct': counts[True, True], 'only_left_correct': counts[True, False],
                'only_right_correct': counts[False, True], 'both_incorrect': counts[False, False],
                'different_correctness_rows': counts[True, False] + counts[False, True]})
    save(OUT / 'decisions.json', decisions)
    save(OUT / 'summary.json', {'version': 'QA02-ADOPTED-REFERENT-REGRADE-v1', 'actual_api_calls': 0,
        'independent_second_review': False, 'population': summary,
        'all_rows': len(decisions), 'changed': sum(d['changed'] for d in decisions),
        'paired_judgments': pairs,
        'corrected_score_is_posthoc': True, 'predictions_frozen_before_corrected_aggregation': True,
        'files': {name: digest(OUT / name) for name in ['extraction.json', 'prediction-freeze.json', 'predicted-targets.json', 'review-inputs.json', 'provenance.json', 'decisions.json']},
        'source': {'pool_sha256': digest(POOL), 'method_sha256': digest(HERE / 'METHOD.md'),
                   'script_sha256': digest(Path(__file__)), 'associations_sha256': digest(HERE / 'associations.json'),
                   'manual_associations_sha256': digest(HERE / 'manual-associations.json'),
                   'source_roles_sha256': digest(HERE / 'source-roles.json')}})
    print(json.dumps({p: {c: f"{s['correct']}/{s['planned']} = {s['accuracy_percent']:.2f}%" for c, s in v.items()} for p, v in summary.items()}))


def verify():
    summary = read(OUT / 'summary.json')
    records = read(OUT / 'decisions.json')
    inputs = {x['key']: x for x in read(OUT / 'review-inputs.json')}
    predictions = {x['key']: x for x in read(OUT / 'predicted-targets.json')}
    frozen = read(OUT / 'prediction-freeze.json')
    for name, sha in summary['files'].items():
        assert digest(OUT / name) == sha, name
    assert digest(Path(__file__)) == summary['source']['script_sha256']
    assert digest(HERE / 'METHOD.md') == summary['source']['method_sha256'] == frozen['method_sha256']
    original = read(EVIDENCE / 'preservation-recovery184-v1/summary.json')
    original_keys = {'COMMON9': 'primary_common9_summary', 'COMMON_SUPPLEMENT6': 'supplement6_summary'}
    checked = set()
    for record in records:
        row, prediction = inputs[record['key']], predictions[record['key']]
        trace = REPO / record['trace_path']
        if trace not in checked:
            assert digest(trace) == record['trace_sha256']
            checked.add(trace)
        if record['correct']:
            assert row['plan'] is not None and prediction['canonical_targets']
            expected, _ = intended_targets(row, {'expected_set': record['old_expected_set']})
            assert prediction['canonical_targets'] == [list(t) for t in expected]
        if record['manual_evidence']:
            value = row
            for component in record['manual_evidence']['pointer'].split('/')[1:]:
                value = value[int(component)] if isinstance(value, list) else value[component]
            assert value is not None, record['key']
    for population, configs in summary['population'].items():
        for config, item in configs.items():
            group = [r for r in records if r['population'] == population and r['configuration'] == config]
            assert len(group) == item['planned']
            assert sum(r['correct'] for r in group) == item['correct']
            assert sum(item['reasons'].values()) == item['planned']
            assert item['correct'] == item['original_correct'] + item['changed_to_correct'] - item['changed_to_incorrect']
            assert item['original_correct'] == original[original_keys[population]]['FORMAL:' + config]['qa02']['correct']
    assert len(records) == len({r['key'] for r in records}) == 651
    result = {'version': 'QA02-REGRADE-VERIFICATION-v1', 'status': 'PASS',
        'checked_rows': len(records), 'source_trace_hashes_checked': len(checked),
        'population_denominators': {'COMMON9': 525, 'COMMON_SUPPLEMENT6': 126},
        'changed_rows': sum(r['changed'] for r in records), 'actual_api_calls': 0,
        'checks': ['full row coverage and identity uniqueness', 'original scores reproduce published summary',
                   'all success sets match canonical predictions frozen before corrected aggregation',
                   'source trace hashes unchanged', 'manual attribution pointers exist in adopted records',
                   'all score changes reconcile with fixed denominators', 'report data hashes match']}
    save(OUT / 'verification.json', result)
    print(json.dumps(result))


def report():
    summary = read(OUT / 'summary.json')
    labels = {'AAAA': '공통 A', 'BAAA': 'DP1 B', 'CAAA': 'DP1 C', 'ABAA': 'DP2 B',
              'AABA': 'DP3 B', 'AAAB': 'DP4 B', 'AAAC': 'DP4 C'}
    lines = ['# QA02 원시 기록 재채점 결과', '', '2026-10-20 / QA02-ADOPTED-REFERENT-REGRADE-v1', '',
        '동일한 첫 의미 채택 기록 651건을 재채점했다. 대표 525건 중 202건이 정정됐고, 별도 보완 126건은 원래 판정과 같았다. 모델 API 추가 호출은 0회다.', '',
        '## 구성별 정확도', '', '| 방안 | 구성 | 대표 정정 정확도 | 대표 원래 엄격 판정 | 별도 보완 정정 정확도 |',
        '| --- | --- | ---: | ---: | ---: |']
    for cfg, label in labels.items():
        primary = summary['population']['COMMON9'][cfg]
        supplement = summary['population']['COMMON_SUPPLEMENT6'][cfg]
        lines.append(f"| {label} | {cfg} | **{primary['accuracy_percent']:.2f}% ({primary['correct']}/75)** | {primary['original_correct']}/75 | **{supplement['accuracy_percent']:.2f}% ({supplement['correct']}/18)** |")
    lines += ['', '## 대표 사례별 정답 수', '', '| 사례 | 예정 | ' + ' | '.join(labels.values()) + ' |',
              '| --- | ---: | ' + ' | '.join(['---:'] * len(labels)) + ' |']
    scenarios = summary['population']['COMMON9']['AAAA']['scenarios']
    for sid, item in scenarios.items():
        values = [str(summary['population']['COMMON9'][cfg]['scenarios'][sid]['correct']) for cfg in labels]
        lines.append(f"| {sid} | {item['planned']} | " + ' | '.join(values) + ' |')
    lines += ['', '## 판정 방식과 검증', '',
        '문자열이 같은지를 먼저 검사하던 방식을 수정해, 실제 채택한 표현 또는 처리 목적과 제약에서 지칭 대상을 추출하고 원본, 판본, 범위를 정규화했다. 표현이 다른 동일 대상과 전체 원본/전체 named extent 별칭을 허용한다. 실제 전달한 설명을 원본 문단과 합치지 않으며, 미채택과 정정 전 결과 및 근거 없는 대상은 실패로 남긴다.', '',
        '202건의 정정은 명시적 binding의 표현/ID 정규화 129건, 단일 지칭의 실제 채택 targets 확인 59건, 채택한 목적/제약의 명시적 대상 배정 확인 14건이다. 수동 검토한 배정 20건 중 6건은 여전히 실패이며, 실제 요약 객체 여부 12건도 원문을 확인했다. 독립된 두 번째 평가자 검토를 했다고 주장하지 않는다.', '',
        '공통 A는 정답42, 미채택20, 잘못된 대상9, 의도한 수정 결과가 당시 생성/전달되지 않아 대상을 확인할 수 없는4건이다. 마지막4건도 분모75에 실패로 남는다. S12는 원래0/10에서10/10으로 정정됐지만, S15의 이전 설명을 원본 문단으로 잘못 고른5건은 유지됐다.', '',
        '보완의 S83/S84는 모든 구성에서0/12다. 작은/큰 버튼 화면에서 최종 의미 채택이 없었기 때문에, 표현 정규화만으로 성공을 만들 수 없다. 보완의 성공은 S85 native 선택 사례에서만 나왔다.', '',
        '대표와 보완은 별도 모집단이다. 같은 공통 A를 네 DP에서 재사용하므로 네 독립 표본으로 합산하지 않는다. 관측 순위는 이 합성 PoC의 결과이며 새로운 재실행이나 일반 제품 정확도가 아니다.', '',
        '- [방법 계약](../../../../../benchmark/architecture/common9_poc_20261019/qa02_regrade_v1/METHOD.md)',
        '- [651건 판정 변경 원장](decisions.json)', '- [정답을 넣지 않은 채택 기록](review-inputs.json)',
        '- [구성 및 원시 해시 연결](provenance.json)', '- [정답 비교 전 대상 추출 동결](prediction-freeze.json)',
        '- [구성별 요약과 짝 비교](summary.json)', '- [검증 결과](verification.json)', '',
        '## 재현', '', '저장소 root에서 기존 두 증거 패키지를 원래 결과 경로로 복원한 뒤 다음 명령을 실행한다. 원시 실행기나 모델은 호출하지 않는다.', '',
        '```sh', '.venv/bin/python benchmark/architecture/common9_poc_20261019/qa02_regrade_v1/regrade.py extract',
        '.venv/bin/python benchmark/architecture/common9_poc_20261019/qa02_regrade_v1/regrade.py predict',
        '.venv/bin/python benchmark/architecture/common9_poc_20261019/qa02_regrade_v1/regrade.py score',
        '.venv/bin/python benchmark/architecture/common9_poc_20261019/qa02_regrade_v1/regrade.py verify',
        '.venv/bin/python benchmark/architecture/common9_poc_20261019/qa02_regrade_v1/regrade.py report', '```', '']
    (OUT / 'README.md').write_text('\n'.join(lines))
    print('Wrote QA02 report.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('command', choices=['extract', 'predict', 'score', 'verify', 'report'])
    args = parser.parse_args()
    if args.command == 'extract':
        extract()
    elif args.command == 'predict':
        predict()
    elif args.command == 'score':
        score()
    elif args.command == 'verify':
        verify()
    elif args.command == 'report':
        report()
