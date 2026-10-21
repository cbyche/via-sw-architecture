#!/usr/bin/env python3
"""Reconcile authored QA ledgers in separate, immutable profile strata.

No models, product candidates, measurements, or parent generators are run.
First use pins the two parent JSONs. Later use reads only those copies.
"""
from __future__ import annotations

import csv
import copy
import hashlib
import json
import math
from collections import defaultdict
from pathlib import Path
from statistics import mean

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/presentations_files/qa-common-synthetic-v2-20261021'
DOC = ROOT / 'docs/architecture/12-decisions/decision-packages/qa-common-synthetic-v2-20261021'
VERSION = 'COMMON-QA-SYN-2.0'
EVIDENCE = 'SYNTHETIC_AUTHORED_NOT_MEASURED'
PARENTS = {
    'TEXT': 'docs/presentations_files/qa-common-synthetic-20261021/synthetic-data.json',
    'VOICE': 'docs/architecture/12-decisions/decision-packages/seven-problem-review/dp3-grounding-architecture-20261021/synthetic-comparison/synthetic-data.json',
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def canon(value):
    if isinstance(value, dict):
        return {k: canon(v) for k, v in sorted(value.items())}
    if isinstance(value, list):
        return sorted((canon(v) for v in value), key=lambda x: json.dumps(x, ensure_ascii=False, sort_keys=True))
    return value


def close(a, b, tolerance=0.000003):
    assert math.isclose(a, b, abs_tol=tolerance), (a, b)


def render_report(summary, comparisons, families, records):
    lines = ['# 층별 재산출 수치와 설계 판단', '',
             f'`{VERSION}` / **{EVIDENCE}**', '',
             '아래 49개 대표값은 고정한 부모 원장의 입력, 정답, 단계, 사용량과 소유 상태에서 다시 확인했다. 원본 숫자를 보존하며 통합에 따른 성능 개선을 주장하지 않는다. 두 층의 절대값은 서로 다른 workload와 profile이다.', '']
    for layer, title in [('TEXT', 'Text 이력: DP2와 DP5'), ('VOICE', 'Voice/Screen: DP3')]:
        lines += [f'## {title}', '',
                  '| 구성 | QA01 초 | QA02 일치/표현 | QA03 초 | QA04 모듈/과제 | QA05 VIA USD | QA06 MB | QA07 일치/입력 |',
                  '| --- | ---: | --- | ---: | ---: | ---: | ---: | --- |']
        for cid, r in summary.items():
            if r['stratum'] != layer: continue
            lines.append(f"| {cid} | {r['qa01']:.6f} | {r['referent_correct']}/{r['referent_n']} ({r['qa02']:.2%}) | {r['qa03']:.6f} | {r['qa04']:.6f} | {r['qa05']:.6f} | {r['qa06']:.3f} | {r['meaning_correct']}/{r['input_n']} ({r['qa07']:.2%}) |")
        lines += ['', 'Text는 240입력, 250표현, 180목표와 120에피소드의 활동 비용이다. Voice는 120평가입력, 120표현, 120목표이며 비용과 RAM에는 별도 준비 60활동을 포함한다.' if layer == 'VOICE' else 'T-BASE는 두 DP가 공유하는 동일 원장이다. T-EXEC는 DP2만, T-REL은 DP5만 변경했다.', '']
    lines += ['## 같은 조건에서 한 축을 바꾼 차이', '',
              '차이는 **오른쪽 대안−왼쪽 기준**이다. 시간, 변경, 비용과 RAM은 음수가 감소이고 정확도는 양수가 증가다. 정확도 차이 단위는 percentage point다. 서로 다른 비교쌍의 차이를 더하지 않는다.', '',
              '| 비교 | 기준 → 대안 | QA01 초 | QA02 %p | QA03 초 | QA04 모듈 | QA05 USD | QA06 MB | QA07 %p |',
              '| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
    for r in comparisons:
        d = r['delta']
        lines.append(f"| {r['comparison']} | {r['baseline']} → {r['alternative']} | {d['qa01']:+.6f} | {100*d['qa02']:+.3f} | {d['qa03']:+.6f} | {d['qa04']:+.6f} | {d['qa05']:+.6f} | {d['qa06']:+.3f} | {100*d['qa07']:+.3f} |")
    lines += ['', '## 준비와 수신 Agent를 포함한 비용 구분', '',
              '| 구성 | 준비 VIA | 종료 후 갱신 VIA | 평가 처리 VIA | QA05 합계 | 수신 Agent | 역할 합계 |',
              '| --- | ---: | ---: | ---: | ---: | ---: | ---: |']
    for cid, r in summary.items():
        lines.append('| ' + cid + ' | ' + ' | '.join(f"${r[k]:.6f}" for k in ['preparation_usd', 'maintenance_usd', 'scored_usd', 'qa05', 'receiver_cost', 'total_cost']) + ' |')
    lines += ['', 'Voice의 준비는 동일한 이전 사용자 활동 60개다. Text 준비는 T-REL의 사전 관계 생산이며 둘을 같은 활동 수로 환산하지 않는다. Local 시각 생산의 cloud 청구액이 0이어도 시간과 RAM 부담은 남는다.', '',
              '## RAM 최고값과 처리 종료', '',
              '| 구성 | 최고값의 사례 | phase | 시각 초 | 동시 합 MB | terminal의 의미 불일치 목표 |',
              '| --- | --- | --- | ---: | ---: | ---: |']
    for cid, r in summary.items():
        p = r['peak_witness']
        lines.append(f"| {cid} | {p['case_id']} | {p['phase']} | {p['time_s']:.3f} | {p['total_mb']:.3f} | {r['terminal_content_incorrect_n']}/{r['goal_n']} |")
    lines += ['', '모든 구성에 첫 표시와 terminal이 작성돼 있지만 마지막 열처럼 의미 불일치가 남는다. 이를 제품 성공률 100%로 읽지 않는다. 준비 중 peak도 QA06에 포함하며, 각 stage의 전체 snapshot을 서로 더하지 않는다.', '',
              '## 공통 변경 과제', '',
              '| 구성 | C01 | C02 | C03 | C04 | C05 | C06 | 합/6 |',
              '| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
    for cid, r in summary.items():
        counts = [c['module_count'] for c in sorted(records['changes'], key=lambda x: x['change_id']) if c['config_id'] == cid]
        lines.append('| ' + cid + ' | ' + ' | '.join(str(n) for n in counts) + f" | {sum(counts)}/6 = {r['qa04']:.3f} |")
    lines += ['', 'C06의 추가 1모듈은 파생 관계 또는 시각 후보의 구간별 의존 무효화 가정이다. 공통 guard로 충분하면 동점이 될 수 있다. 두 층의 과제를 12개로 합산하지 않으며 작은 모듈 수 차이를 핵심 선택 근거로 부풀리지 않는다.', '',
              '## 사례군별 첫 응답과 정확도', '',
              '각 행의 모든 구성은 같은 층과 같은 사례군을 사용한다. Voice의 균등 비중은 실제 사용 빈도 조사값이 아니다.', '',
              '| 사례군 | 구성 | QA01 초 | QA02 | QA03 초 | QA07 | 입력/표현/목표 |',
              '| --- | --- | ---: | ---: | ---: | ---: | --- |']
    for r in families:
        lines.append(f"| {r['family_id']} | {r['config_id']} | {r['qa01']:.3f} | {r['qa02']:.2%} | {r['qa03']:.3f} | {r['qa07']:.2%} | {r['input_n']}/{r['referent_n']}/{r['goal_n']} |")
    lines += ['', '## 같은 대상 오류가 두 정확도에 나타나는 방식', '',
              '| 구성 | 대상 오류가 있는 입력 | 대상은 모두 맞고 의미가 틀린 입력 | 의미 오류 입력 합계 |',
              '| --- | ---: | ---: | ---: |']
    for cid, s in summary.items():
        target_errors = {r['input_id'] for r in records['referents'] if r['config_id'] == cid and not r['correct']}
        meaning_errors = {r['input_id'] for r in records['responses'] if r['config_id'] == cid and not r['qa07_success']}
        assert target_errors <= meaning_errors
        lines.append(f'| {cid} | {len(target_errors)} | {len(meaning_errors - target_errors)} | {len(meaning_errors)} |')
    lines += ['', 'DP3 A의 대상 오류 13개 중 B가 9개를 해결하고, 공통 실패 4개와 B 신규 오류 1개를 남긴다. B의 순개선은 8입력이다. 의미 전용 오류 5개는 동일하므로 QA02와 QA07의 +6.667%p는 같은 대상 오류 개선이다. DP5는 지칭 정확도가 높아져도 과거 목적을 현재 요청에 잘못 적용하면 전체 의미는 나빠질 수 있다.', '',
              '## 조건부 개별 선택', '',
              '| 비교 | 이 가정과 기존 QA01 우선순위에서의 권고 | 받아들이는 손실 | 다시 볼 조건 |',
              '| --- | --- | --- | --- |',
              '| DP2 | B. 평균 첫 응답 0.326초와 목표 처리 0.845초 감소 | 대상 −1.60%p, 전체 의미 −4.583%p, 비용 +$0.121911, RAM +27.292 MB | 진행 중 후속 입력이 적거나 기존 문맥 의존이 큰 실제 분포. 완료 후 후속 입력에서는 추가 실행이 느릴 수 있음 |',
              '| DP3 | A. R2보다 첫 응답과 목표 처리 1.817초 빠르고 RAM 1,782 MB 감소 | 대상과 전체 의미 각각 −6.667%p인 동일 오류 손실, 비용 +$0.486400 | 정확도가 우선인 별도 요구, 더 느린 원본 해석, 재사용이 압도적인 분포 |',
              '| DP5 | B. 평균 첫 응답 0.469초와 목표 처리 0.775초 감소, 지칭 +2.00%p | 전체 의미 −2.917%p, 비용 +$0.904190, RAM +50.892 MB, C06 +1모듈 가정 | 새 피드백이 많아 관계가 미생산/무효이거나 현재 목적을 과거 선호와 분리하기 어려운 경우 |', '',
              '이는 QA01→QA02→QA03→QA04→QA05→QA06 순서에서 해당 비교의 명확한 가상 첫 응답 차이를 우선한 판단이다. QA07을 임의로 마지막 순위나 최우선으로 넣지 않았으며 손실을 별도로 공개했다. 사용자 채택이나 실제 제품 승리를 선언하지 않는다. 개별 B/A/B 권고를 조합한 성능은 미작성이다.', '',
              'B 계열의 조건부 기본은 R2다. R1 대비 첫 응답 0.500초 감소와 RAM 420 MB 증가를 교환한다. R3는 R2보다 정상 요청 평균 0.040초와 RAM 288 MB가 추가되며 정확도와 cloud 비용은 같다. 독립 worker 복구나 서로 다른 runtime 운영 요구가 있을 때 R3의 선택 이유가 생긴다. 이 복구 가치를 현 QA04나 정상 요청 정확도로 바꾸지 않는다.', '',
              '## 민감도와 경계', '',
              'V-S5에서는 R2가 A보다 0.20초 빠르다. 다른 다섯 군의 상대 비중을 같게 유지한 채 S5 비중만 바꾸면 차이는 `(1−w)×2.22−w×0.20`초다. 역전점은 `w=2.22/2.42=91.7355%`다. 결과를 뒤집기 위한 모집단 변경이 아니라 이미 작성한 family 값의 민감도다. 원본 직접 해석만 현재보다 평균 1.8167초 더 오래 걸려도 나머지 조건이 같으면 R2와 첫 응답이 같아진다.', '',
              'R3의 보조 병렬 계산 2.62/6.62초는 시각 stage의 응답이며 전체 요청 QA01/03이 아니다. R2도 같은 제한 병렬 admission을 가질 수 있어 process 분리만의 이익으로 합치지 않는다. S3의 같은 객체라는 설명은 실제 이미지로 검증한 결과가 아니고, Voice 120개에는 후속 정정이 없다. 이러한 한계와 교차 조합 미작성은 [구성과 시나리오](02-configurations-and-scenarios.md)에 남겼다.', '',
              '계산 원값, 단계와 출처는 [공통 원장](../../../../presentations_files/qa-common-synthetic-v2-20261021/normalized-data.json), [검증](../../../../presentations_files/qa-common-synthetic-v2-20261021/validation.json), [이관 기록](04-migration-and-review.md)에서 확인한다.', '']
    DOC.mkdir(parents=True, exist_ok=True)
    (DOC / '03-results-and-decisions.md').write_text('\n'.join(lines))


def build():
    OUT.mkdir(parents=True, exist_ok=True)
    parent_records, data = [], {}
    for layer, relative in PARENTS.items():
        live = ROOT / relative
        frozen = OUT / 'parents' / (layer.lower() + '.json')
        if not frozen.exists():
            frozen.parent.mkdir(parents=True, exist_ok=True)
            frozen.write_bytes(live.read_bytes())
        payload = frozen.read_bytes()
        data[layer] = json.loads(payload)
        parent_records.append({'stratum': layer, 'source_path': relative,
                               'snapshot_path': str(frozen.relative_to(OUT)),
                               'sha256': sha(payload), 'bytes': len(payload)})
    text, voice = data['TEXT'], data['VOICE']
    assert text['version'] == 'COMMON-QA-SYN-1.0'
    assert voice['dataset_id'] == 'DP3-GROUNDING-CPU-SYN-1.0'
    assert all(d['evidence_kind'] == EVIDENCE for d in data.values())
    profiles = {
        'TEXT': {'id': 'TEXT-HISTORY-SYN-1.0', 'parent_dataset': text['version'],
                 'start_event': 'TEXT_SUBMITTED', 'assumptions': text['assumptions'],
                 'episodes': 120, 'inputs': 240, 'referents': 250, 'goals': 180,
                 'prior_user_activities': 0, 'background_preparation_in_cost_and_memory': True},
        'VOICE': {'id': voice['profile']['id'], 'parent_dataset': voice['dataset_id'],
                  'start_event': 'ACOUSTIC_UTTERANCE_END', 'assumptions': voice['profile'],
                  'episodes': 120, 'inputs': 120, 'referents': 120, 'goals': 120,
                  'prior_user_activities': 60, 'background_preparation_in_cost_and_memory': True},
    }
    configs = [
        {'id': 'T-BASE', 'stratum': 'TEXT', 'parent_config': 'BASE', 'dp2': 'A', 'dp3': 'NOT_EXERCISED', 'dp5': 'A'},
        {'id': 'T-EXEC', 'stratum': 'TEXT', 'parent_config': 'EXEC', 'dp2': 'B', 'dp3': 'NOT_EXERCISED', 'dp5': 'A'},
        {'id': 'T-REL', 'stratum': 'TEXT', 'parent_config': 'REL', 'dp2': 'A', 'dp3': 'NOT_EXERCISED', 'dp5': 'B'},
    ] + [{'id': f'V-{suffix}', 'stratum': 'VOICE', 'parent_config': parent,
          'dp2': 'FIXED_UNMAPPED', 'dp3': parent, 'dp5': 'FIXED_UNMAPPED'}
         for suffix, parent in [('A', 'A'), ('R1', 'B-R1'), ('R2', 'B-R2'), ('R3', 'B-R3')]]
    for c in configs:
        c['profile_id'] = profiles[c['stratum']]['id']
        c['source_execution_key'] = profiles[c['stratum']]['parent_dataset'] + ':' + c['parent_config']
        c['evidence_kind'] = EVIDENCE
    config_lookup = {(c['stratum'], c['parent_config']): c for c in configs}
    records = {k: [] for k in ['scenarios', 'responses', 'referents', 'goals', 'stages', 'usage', 'memory', 'changes']}

    def base(layer, config, case):
        c = config_lookup[(layer, config)]
        return {'config_id': c['id'], 'stratum': layer, 'profile_id': c['profile_id'],
                'case_id': layer + ':' + case, 'source_execution_key': c['source_execution_key'] + ':' + case}

    for ep in text['episodes']:
        inputs = copy.deepcopy(ep['inputs'])
        for item in inputs:
            item['source_input_id'], item['id'] = item['id'], 'TEXT:' + item['id']
            item['source_goal_id'], item['goal_id'] = item['goal_id'], 'TEXT:' + item['goal_id']
            for expression in item['referents']:
                expression['source_expression_id'], expression['id'] = expression['id'], 'TEXT:' + expression['id']
        records['scenarios'].append({'case_id': 'TEXT:' + ep['id'], 'stratum': 'TEXT',
                                    'family_id': 'T-' + ep['family_id'], 'inputs': inputs,
                                    'source_ids': ep['source_ids'], 'source_locator': 'parents/text.json#/episodes/' + str(text['episodes'].index(ep))})
    for case in voice['cases']:
        records['scenarios'].append({'case_id': 'VOICE:' + case['id'], 'stratum': 'VOICE',
                                    'family_id': 'V-' + case['family'], 'inputs': [{'id': 'VOICE:' + case['input_id'],
                                        'source_input_id': case['input_id'], 'goal_id': 'VOICE:' + case['goal_id'],
                                        'source_goal_id': case['goal_id'], 'revision': case['revision'],
                                        'referents': [{'id': 'VOICE:' + case['expression_id'], 'source_expression_id': case['expression_id'], 'expected': case['oracle']['targets']}],
                                        'text': case['utterance'], 'oracle': case['oracle']}],
                                    'prior_user_activity': case['setup_request'], 'source_ids': [case['source_id']],
                                    'source_locator': 'parents/voice.json#/cases/' + str(voice['cases'].index(case))})
    times = {(r['config'], r['input_id']): r for r in text['times']}
    for r in text['responses']:
        t = times[(r['config'], r['input_id'])]
        correct = int(canon(r['expected_meaning']) == canon(r['adopted_meaning']))
        assert correct == r['qa07']
        close(t['first_s'] - t['start_s'], t['qa01_s'])
        records['responses'].append({**base('TEXT', r['config'], r['episode_id']),
            'family_id': 'T-' + r['family_id'], 'input_id': 'TEXT:' + r['input_id'], 'revision': r['revision'],
            'goal_id': 'TEXT:' + r['goal_id'], 'start_s': t['start_s'], 'first_display_s': t['first_s'],
            'qa01_s': t['qa01_s'], 'expected': r['expected_meaning'], 'adopted': r['adopted_meaning'],
            'qa07_success': correct, 'first_content': r['first_content'], 'error_reason': r['error_reason'],
            'adopted_at_s': r['full_meaning_adopted_s'], 'adoption_interval': None})
    for r in text['referents']:
        correct = int(set(r['expected']) == set(r['adopted']))
        assert correct == r['correct']
        records['referents'].append({**base('TEXT', r['config'], r['episode_id']),
            'family_id': 'T-' + r['family_id'], 'input_id': 'TEXT:' + r['input_id'],
            'expression_id': 'TEXT:' + r['expression_id'], 'revision': r['revision'],
            'expected': r['expected'], 'adopted': r['adopted'], 'correct': correct})
    groups = defaultdict(list)
    for n in text['nodes']:
        groups[(n['config'], n['episode_id'])].append(n)
        records['stages'].append({**base('TEXT', n['config'], n['episode_id']),
            'id': 'TEXT:' + n['id'], 'start_s': n['start_s'], 'end_s': n['finish_s'],
            'duration_s': n['duration_s'], 'kind': n['label'], 'role': n['role'],
            'included': not(n['role'].startswith('EXTERNAL') or n['role'] == 'USER_SOURCE'),
            'parents': ['TEXT:' + p for p in n['parents']], 'source_scope': 'episode'})
    for g in text['goals']:
        projected = {}
        for n in groups[(g['config'], g['episode_id'])]:
            assert all(p in projected for p in n['parents'])
            work = 0 if n['role'].startswith('EXTERNAL') or n['role'] == 'USER_SOURCE' else max(0, n['finish_s'] - max(n['start_s'], g['start_s']))
            projected[n['id']] = work + max([0] + [projected[p] for p in n['parents']])
        close(projected[g['terminal_node']], g['via_s'])
        records['goals'].append({**base('TEXT', g['config'], g['episode_id']),
            'family_id': 'T-' + g['family_id'], 'goal_id': 'TEXT:' + g['goal_id'],
            'input_ids': ['TEXT:' + x for x in g['members']], 'via_s': g['via_s'], 'wall_s': g['wall_s'],
            'start_s': g['start_s'], 'terminal_node': 'TEXT:' + g['terminal_node'],
            'terminal_content_correct': g['terminal_content_correct'], 'status': g['status']})
    for row in text['costs']:
        phase = 'preparation' if '사전 생산' in row['stage'] else 'maintenance' if '갱신' in row['stage'] else 'scored'
        values = [('text_input', row['input_tokens'], 1e-6), ('text_output', row['output_tokens'], 4e-6), ('audio_output_seconds', row['audio_seconds'], .002)]
        close(sum(amount * rate for _, amount, rate in values), row['cost_usd'], 1e-10)
        for bucket, amount, rate in values:
            if amount:
                records['usage'].append({**base('TEXT', row['config'], row['episode_id']),
                    'id': 'TEXT:' + row['id'] + ':' + bucket, 'node_id': 'TEXT:' + row['node_id'],
                    'phase': phase, 'role': row['role'], 'bucket': bucket, 'amount': amount,
                    'rate_usd': rate, 'usd': amount * rate, 'reason': row['stage']})
    for i, row in enumerate(text['memory']):
        components = {k: v for k, v in row.items() if k.endswith('_mb') and k != 'total_mb'}
        close(sum(components.values()), row['total_mb'])
        records['memory'].append({**base('TEXT', row['config'], row['episode_id']),
            'id': f'TEXT:M{i}', 'time_s': row['time_s'], 'end_s': None, 'phase': row['phase'],
            'components_mb': components, 'total_mb': row['total_mb'], 'observation': 'AUTHORED_POINT_STATE'})
    for ch in text['changes']:
        for c, row in ch['configs'].items():
            records['changes'].append({**base('TEXT', c, ch['id']), 'change_id': ch['id'],
                'title': ch['title'], 'acceptance': ch['acceptance'], 'modules': row['modules'],
                'module_count': len(set(row['modules'])), 'reason': row['reason'], 'evidence_kind': 'SYNTHETIC_DESIGN_CHANGE_REVIEW'})

    vgroups = defaultdict(list)
    for n in voice['stages']:
        vgroups[(n['config'], n['case'], n['scope'])].append(n)
        records['stages'].append({**base('VOICE', n['config'], n['case']),
            'id': 'VOICE:' + n['id'], 'start_s': n['start'], 'end_s': n['end'],
            'duration_s': n['duration'], 'kind': n['kind'],
            'role': 'VIA' if n['qa03_included'] else 'EXTERNAL_AGENT', 'included': n['qa03_included'],
            'parents': ['VOICE:' + p for p in n['parents']], 'source_scope': n['scope']})
    for r in voice['responses']:
        common = base('VOICE', r['config'], r['case'])
        nodes = vgroups[(r['config'], r['case'], 'scored')]
        projected = {}
        for n in nodes:
            assert all(p in projected for p in n['parents'])
            projected[n['id']] = (n['duration'] if n['qa03_included'] else 0) + max([0] + [projected[p] for p in n['parents']])
        terminal = next(n for n in nodes if n['kind'] == 'final_ui')
        first = next(n for n in nodes if n['kind'] == 'adopt_display')
        close(projected[terminal['id']], r['qa03_seconds'])
        close(first['end'], r['first_seconds'])
        qa02 = int(set(r['oracle']['targets']) == set(r['first_adopted']['targets']))
        qa07 = int(canon(r['oracle']) == canon(r['first_adopted']))
        assert qa02 == r['qa02_success'] and qa07 == r['qa07_success']
        records['responses'].append({**common, 'family_id': 'V-' + r['family'], 'input_id': 'VOICE:' + r['input_id'],
            'revision': 1, 'goal_id': 'VOICE:' + r['goal_id'], 'start_s': 0, 'first_display_s': first['end'],
            'qa01_s': first['end'], 'expected': r['oracle'], 'adopted': r['first_adopted'],
            'qa07_success': qa07, 'first_content': r['first_response'], 'error_reason': r['error_reason'],
            'adopted_at_s': None, 'adoption_interval': [first['start'], first['end']]})
        records['referents'].append({**common, 'family_id': 'V-' + r['family'], 'input_id': 'VOICE:' + r['input_id'],
            'expression_id': 'VOICE:' + r['expression_id'], 'revision': 1, 'expected': r['oracle']['targets'],
            'adopted': r['first_adopted']['targets'], 'correct': qa02})
        records['goals'].append({**common, 'family_id': 'V-' + r['family'], 'goal_id': 'VOICE:' + r['goal_id'],
            'input_ids': ['VOICE:' + r['input_id']], 'via_s': projected[terminal['id']], 'wall_s': r['wall_seconds'],
            'start_s': 0, 'terminal_node': 'VOICE:' + terminal['id'], 'terminal_content_correct': int(r['semantic_success']), 'status': r['terminal']})
    for i, row in enumerate(voice['usage']):
        close(row['amount'] * row['unit_rate_usd'], row['usd'], 1e-10)
        records['usage'].append({**base('VOICE', row['config'], row['case']),
            'id': f'VOICE:U{i}', 'node_id': 'VOICE:' + row['node_id'],
            'phase': 'preparation' if row['scope'] == 'prior_setup' else 'scored',
            'role': 'VIA' if row['role'] == 'VIA' else 'RECEIVER', 'bucket': row['bucket'],
            'amount': row['amount'], 'rate_usd': row['unit_rate_usd'], 'usd': row['usd'], 'reason': row['scope']})
    for i, row in enumerate(voice['memory']):
        close(sum(row['components_mb'].values()), row['total_mb'])
        records['memory'].append({**base('VOICE', row['config'], row['case']),
            'id': f'VOICE:M{i}', 'time_s': row['start'], 'end_s': row['end'],
            'phase': 'preparation' if row['scope'] == 'prior_setup' else 'scored',
            'components_mb': row['components_mb'], 'total_mb': row['total_mb'],
            'observation': 'AUTHORED_STAGE_STATE', 'node_id': 'VOICE:' + row['node_id']})
    for ch in voice['changes']:
        spec = next(x for x in text['changes'] if x['id'] == ch['change_id'])
        records['changes'].append({**base('VOICE', ch['config'], ch['change_id']),
            'change_id': ch['change_id'], 'title': spec['title'], 'acceptance': spec['acceptance'],
            'modules': ch['modules'], 'module_count': len(set(ch['modules'])), 'reason': ch['reason'],
            'evidence_kind': 'SYNTHETIC_DESIGN_CHANGE_REVIEW'})

    summary, families = {}, []
    for c in configs:
        cid = c['id']
        rows = {k: [r for r in records[k] if r['config_id'] == cid] for k in records if k != 'scenarios'}
        q, ref, goals, usage, memory, changes = (rows[k] for k in ['responses', 'referents', 'goals', 'usage', 'memory', 'changes'])
        peak = max(memory, key=lambda r: r['total_mb'])
        value = {'config_id': cid, 'stratum': c['stratum'], 'profile_id': c['profile_id'],
                 'qa01': mean(r['qa01_s'] for r in q), 'qa02': mean(r['correct'] for r in ref),
                 'qa03': mean(r['via_s'] for r in goals), 'qa04': mean(r['module_count'] for r in changes),
                 'qa05': sum(r['usd'] for r in usage if r['role'] == 'VIA'), 'qa06': peak['total_mb'],
                 'qa07': mean(r['qa07_success'] for r in q), 'input_n': len(q), 'referent_n': len(ref),
                 'goal_n': len(goals), 'change_n': len(changes), 'referent_correct': sum(r['correct'] for r in ref),
                 'meaning_correct': sum(r['qa07_success'] for r in q), 'first_missing_n': 0, 'terminal_missing_n': 0,
                 'terminal_content_incorrect_n': sum(not r['terminal_content_correct'] for r in goals),
                 'preparation_usd': sum(r['usd'] for r in usage if r['role'] == 'VIA' and r['phase'] == 'preparation'),
                 'maintenance_usd': sum(r['usd'] for r in usage if r['role'] == 'VIA' and r['phase'] == 'maintenance'),
                 'scored_usd': sum(r['usd'] for r in usage if r['role'] == 'VIA' and r['phase'] == 'scored'),
                 'receiver_cost': sum(r['usd'] for r in usage if r['role'] == 'RECEIVER'), 'peak_witness': peak}
        value['total_cost'] = value['qa05'] + value['receiver_cost']
        expected = profiles[c['stratum']]
        assert [len(q), len(ref), len(goals), len(changes)] == [expected['inputs'], expected['referents'], expected['goals'], 6]
        old = data[c['stratum']]['summary'][c['parent_config']]
        old_keys = {1: 'qa01_seconds', 3: 'qa03_seconds', 4: 'qa04_modules', 5: 'qa05_usd', 6: 'qa06_mb'}
        for n in range(1, 8):
            if c['stratum'] == 'TEXT': previous = old[f'qa{n:02}']
            elif n in (2, 7): previous = old[f'qa{n:02}_correct'] / old[f'qa{n:02}_denominator']
            else: previous = old[old_keys[n]]
            close(value[f'qa{n:02}'], previous)
        summary[cid] = value
        for family in sorted({r['family_id'] for r in q}):
            fq, fr, fg = ([r for r in source if r['family_id'] == family] for source in [q, ref, goals])
            families.append({'config_id': cid, 'stratum': c['stratum'], 'family_id': family,
                             'qa01': mean(r['qa01_s'] for r in fq), 'qa02': mean(r['correct'] for r in fr),
                             'qa03': mean(r['via_s'] for r in fg), 'qa07': mean(r['qa07_success'] for r in fq),
                             'input_n': len(fq), 'referent_n': len(fr), 'goal_n': len(fg)})
    pairs = [('DP2', 'T-BASE', 'T-EXEC'), ('DP5', 'T-BASE', 'T-REL'),
             ('DP3 A vs R1', 'V-A', 'V-R1'), ('DP3 A vs R2', 'V-A', 'V-R2'),
             ('DP3 A vs R3', 'V-A', 'V-R3'), ('DP3 residency', 'V-R1', 'V-R2'), ('DP3 isolation', 'V-R2', 'V-R3')]
    comparisons = []
    for name, a, b in pairs:
        assert summary[a]['profile_id'] == summary[b]['profile_id']
        comparisons.append({'comparison': name, 'baseline': a, 'alternative': b,
                            'stratum': summary[a]['stratum'], 'difference_direction': 'alternative minus baseline',
                            'delta': {f'qa{n:02}': summary[b][f'qa{n:02}'] - summary[a][f'qa{n:02}'] for n in range(1, 8)}})
    result = {'dataset_id': VERSION, 'contract_id': 'QA-COMMON-SYN-CONTRACT-v2-STRATIFIED',
              'evidence_kind': EVIDENCE, 'actual_model_calls': 0, 'actual_product_runs': 0,
              'aggregation_policy': 'WITHIN_PROFILE_ONLY_NO_POOLED_HEADLINE',
              'profiles': profiles, 'configs': configs, 'parent_sources': parent_records, **records,
              'summary': summary, 'family_summary': families, 'comparisons': comparisons}
    validation = {'dataset_id': VERSION, 'evidence_kind': EVIDENCE, 'status': 'PASS',
                  'checks': ['parent evidence labels', 'unique IDs by configuration', 'oracle exact equality',
                             'first display timestamps', 'causal DAG including external-duration removal',
                             'usage amount times tariff', 'memory simultaneous component sums',
                             'QA04 unique module counts', 'fixed denominators', 'all 49 parent QA values reproduced',
                             'all comparison pairs use one profile'],
                  'counts': {k: len(v) for k, v in records.items()},
                  'parent_qa_values_verified': 49, 'model_calls': 0, 'product_runs': 0,
                  'limitations': ['Authored assumptions and errors; no performance validation.',
                                  'No cross-profile aggregate or DP2/DP3/DP5 combined execution.',
                                  'Voice preparation activity first-response accuracy remains outside scored population.']}
    for kind, key in [('responses', 'input_id'), ('referents', 'expression_id'), ('goals', 'goal_id')]:
        keys = [(r['config_id'], r[key]) for r in records[kind]]
        assert len(keys) == len(set(keys))
    stage_ids = {r['id'] for r in records['stages']}
    assert len(stage_ids) == len(records['stages'])
    assert all(r['node_id'] in stage_ids for r in records['usage'])
    input_registry = {i['id']: i for s in records['scenarios'] for i in s['inputs']}
    expression_registry = {e['id']: e for i in input_registry.values() for e in i['referents']}
    goal_registry = {i['goal_id'] for i in input_registry.values()}
    assert len(input_registry) == 360 and len(expression_registry) == 370 and len(goal_registry) == 300
    assert all(r['input_id'] in input_registry and r['goal_id'] in goal_registry for r in records['responses'])
    assert all(r['input_id'] in input_registry and r['expression_id'] in expression_registry for r in records['referents'])
    assert all(g['goal_id'] in goal_registry and all(i in input_registry for i in g['input_ids']) for g in records['goals'])
    validation['checks'].append('all normalized scenario input/goal/expression IDs join to verdict ledgers')
    write_json(OUT / 'normalized-data.json', result)
    write_json(OUT / 'summary.json', {'dataset_id': VERSION, 'evidence_kind': EVIDENCE, 'aggregation_policy': result['aggregation_policy'], 'configs': summary, 'comparisons': comparisons, 'families': families})
    write_json(OUT / 'validation.json', validation)
    with (OUT / 'comparison.csv').open('w', newline='') as f:
        fields = ['config_id', 'stratum', 'profile_id'] + [f'qa{n:02}' for n in range(1, 8)] + ['input_n', 'referent_n', 'goal_n', 'change_n', 'preparation_usd', 'maintenance_usd', 'scored_usd', 'receiver_cost', 'terminal_content_incorrect_n']
        writer = csv.DictWriter(f, fieldnames=fields, extrasaction='ignore', lineterminator='\n'); writer.writeheader(); writer.writerows(summary.values())
    hashes = {p.name: sha(p.read_bytes()) for p in [OUT / 'normalized-data.json', OUT / 'summary.json', OUT / 'validation.json', OUT / 'comparison.csv']}
    write_json(OUT / 'manifest.json', {'dataset_id': VERSION, 'evidence_kind': EVIDENCE,
              'schema_version': 'STRATIFIED-LEDGER-1', 'corpus_id': 'COMMON-QA-CORPUS-2.0-STRATIFIED',
              'contract_id': result['contract_id'], 'registry_id': 'COMMON-QA-CONFIG-2.0',
              'parent_sources': parent_records, 'outputs': hashes,
              'generator': {'path': str(Path(__file__).resolve().relative_to(ROOT)), 'sha256': sha(Path(__file__).read_bytes())},
              'policy': 'Parent snapshots are immutable on rerun. New source revisions require a new dataset/output directory.'})
    render_report(summary, comparisons, families, records)
    print(json.dumps({'status': 'PASS', 'dataset_id': VERSION, 'counts': validation['counts'], 'parent_qa_values_verified': 49}))


if __name__ == '__main__':
    build()
