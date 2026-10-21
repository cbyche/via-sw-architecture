#!/usr/bin/env python3
"""Reconcile authored QA ledgers in separate, immutable profile strata.

No models, product candidates, measurements, or parent generators are run.
First use pins v2 parent snapshots and the DP44 authored source. Later use reads only those copies.
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
OUT = ROOT / 'docs/presentations_files/qa-common-synthetic-v3-20261022'
DOC = ROOT / 'docs/architecture/12-decisions/decision-packages/qa-common-synthetic-v3-20261022'
VERSION = 'COMMON-QA-SYN-3.0'
EVIDENCE = 'SYNTHETIC_AUTHORED_NOT_MEASURED'
REVIEW_REVISION = 'COMMON-QA-SYN-3.0-REVIEW-2'
FIRST_V3_COMMIT = '8b2f555f8b89eaa5a6a7085ca6323cd44a1adbd7'
FIRST_V3_NUMERIC_SHA256 = 'bb4804e1e0ff5b55546b239ad2e54db2f04e1fc9e6e1e6a9f0770df512876409'
METRIC_CATALOG = {
    'qa01': {'label': '첫 응답 지연시간 (화면)', 'unit': '초', 'display_decimals': 3, 'aggregation': 'ARITHMETIC_MEAN_PER_INPUT', 'direction': 'LOWER_IS_BETTER'},
    'qa02': {'label': '지칭 대상 식별 정확도', 'unit': '%', 'storage_unit': 'ratio', 'display_scale': 100, 'display_decimals': 2, 'aggregation': 'EXACT_SET_SUCCESS_PER_EXPRESSION_REVISION', 'direction': 'HIGHER_IS_BETTER'},
    'qa03': {'label': 'VIA 처리시간', 'unit': '초', 'display_decimals': 3, 'aggregation': 'ARITHMETIC_MEAN_PER_ORIGINAL_GOAL_FINAL_REVISION', 'direction': 'LOWER_IS_BETTER'},
    'qa04': {'label': '변경 범위 (예상 운영 모듈 수)', 'unit': '모듈/과제', 'display_decimals': 3, 'aggregation': 'MEAN_UNIQUE_CONCEPTUAL_MODULES_PER_CHANGE', 'direction': 'LOWER_IS_BETTER'},
    'qa05': {'label': '모델 사용 비용 (VIA)', 'unit': 'USD/활동 묶음', 'display_decimals': 6, 'aggregation': 'VIA_COST_SUM_ALL_PHASES', 'direction': 'LOWER_IS_BETTER'},
    'qa06': {'label': '최대 로컬 RAM 사용량', 'unit': 'MB', 'display_decimals': 3, 'aggregation': 'MAX_SIMULTANEOUS_OWNED_STATE_DECIMAL_MB', 'direction': 'LOWER_IS_BETTER'},
    'qa07': {'label': '요청 의미 해석 정확도', 'unit': '%', 'storage_unit': 'ratio', 'display_scale': 100, 'display_decimals': 2, 'aggregation': 'ALL_REQUIRED_MEANING_SUCCESS_PER_INPUT_REVISION', 'direction': 'HIGHER_IS_BETTER'},
    'first_voice_latency': {'label': '첫 음성 지연 (별도 진단)', 'unit': '초', 'display_decimals': 3, 'aggregation': 'ARITHMETIC_MEAN_PER_MODELED_DELIVERY_INPUT', 'direction': 'LOWER_IS_BETTER'},
}
SEMANTIC_COMPARISON_POLICY = {
    'id': 'QA07-FIELD-SEMANTICS-1', 'same_rule_for_all_strata': True,
    'unordered_set_paths': ['targets', 'conditions', 'exclusions'],
    'ordered_sequence_paths': ['conditions.order'],
    'dictionary_key_order': 'IGNORED', 'strings': 'CASE_SENSITIVE',
    'missing_fields': 'DISTINCT_FROM_FALSE_OR_NULL',
    'scalar_types': 'JSON_NULL_BOOLEAN_NUMBER_STRING; BOOLEAN_DISTINCT_FROM_NUMBER; 1_EQUALS_1.0',
    'unknown_list_path': 'REJECT_REQUIRE_EXPLICIT_POLICY_EXTENSION',
    'duplicates_in_set_fields': 'DEDUPLICATED',
}
PARENTS = {
    'TEXT': 'docs/presentations_files/qa-common-synthetic-v2-20261021/parents/text.json',
    'VOICE': 'docs/presentations_files/qa-common-synthetic-v2-20261021/parents/voice.json',
    'DELIVERY': 'docs/presentations_files/dp44-synthetic-backdata-20261022/synthetic-data.json',
    'V2_SUMMARY': 'docs/presentations_files/qa-common-synthetic-v2-20261021/summary.json',
    'V2_MANIFEST': 'docs/presentations_files/qa-common-synthetic-v2-20261021/manifest.json',
}
SOURCE_COMMITS = {'DELIVERY': 'c8da10b1ffe63603bcf59ba8e433e7e452970c35',
                  'TEXT': '886e93db78937253ecaac37515de93f90ac0a99b',
                  'VOICE': '886e93db78937253ecaac37515de93f90ac0a99b',
                  'V2_SUMMARY': '886e93db78937253ecaac37515de93f90ac0a99b',
                  'V2_MANIFEST': '886e93db78937253ecaac37515de93f90ac0a99b'}
# Verified against the published Git objects at integration time. A modified
# snapshot is a new source revision, even if a stale manifest has been removed.
SOURCE_SHA256 = {
    'TEXT': 'e7058f7e9ea4710a549ccf754ecd62f4d5659002a4b98642ecb8835699b57a18',
    'VOICE': 'fb12a76dec39d7a6215ddd18c24284660a71eaa9d53645af9b7770615251c2f2',
    'DELIVERY': '95e85719a807c9bce2185662da7fe8bc39106b334b633ad9ddfbb4210f7672ee',
    'V2_SUMMARY': '9557bfd3331eae2a4e7da677764a4795e1fd51b4a897ecf622e4d48220699900',
    'V2_MANIFEST': '84d6f9be71996bd08c5511296953e19ef04bda46a207d942967725623ce1ac6e',
}



def sha(data):
    return hashlib.sha256(data).hexdigest()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def canonical_meaning(value, path=()):
    """Compare required meaning by field semantics, not by input modality."""
    if isinstance(value, dict):
        return ('object', tuple((k, canonical_meaning(v, path + (k,))) for k, v in sorted(value.items())))
    if isinstance(value, list):
        location = '.'.join(path)
        items = [canonical_meaning(v, path + ('*',)) for v in value]
        if location in SEMANTIC_COMPARISON_POLICY['unordered_set_paths']:
            return ('set', tuple(sorted(set(items), key=lambda x: json.dumps(x, ensure_ascii=False))))
        assert location in SEMANTIC_COMPARISON_POLICY['ordered_sequence_paths'], ('unknown meaning list path', location)
        return ('sequence', tuple(items))
    if value is None:
        return ('null', None)
    if isinstance(value, bool):
        return ('boolean', value)
    if isinstance(value, (int, float)):
        assert math.isfinite(value), ('nonfinite meaning number', path)
        return ('number', value)
    assert isinstance(value, str), ('unsupported meaning scalar', path, type(value).__name__)
    return ('string', value)


def verify_semantic_policy():
    examples = [
        {'targets': ['A', 'B'], 'conditions': ['x', 'y'], 'keep': False},
        {'targets': ['A', 'B'], 'exclusions': ['x', 'y'], 'keep': False},
        {'targets': ['A', 'B'], 'conditions': {'order': ['A', 'B']}, 'keep': False},
    ]
    for expected in examples:
        adopted = copy.deepcopy(expected)
        adopted['targets'].reverse()
        assert canonical_meaning(expected) == canonical_meaning(adopted)
        adopted['targets'].append(adopted['targets'][0])
        assert canonical_meaning(expected) == canonical_meaning(adopted)
        adopted['targets'][0] = adopted['targets'][0].lower()
        assert canonical_meaning(expected) != canonical_meaning(adopted)
        missing = copy.deepcopy(expected); missing.pop('keep')
        assert canonical_meaning(expected) != canonical_meaning(missing)
        null = copy.deepcopy(expected); null['keep'] = None
        assert canonical_meaning(expected) != canonical_meaning(null)
    reordered = copy.deepcopy(examples[-1]); reordered['conditions']['order'].reverse()
    assert canonical_meaning(examples[-1]) != canonical_meaning(reordered)
    assert canonical_meaning({'value': False}) != canonical_meaning({'value': 0})
    assert canonical_meaning({'value': 1}) == canonical_meaning({'value': 1.0})
    try:
        canonical_meaning({'unregistered_list': ['A', 'B']})
    except AssertionError as error:
        assert 'unknown meaning list path' in str(error)
    else:
        raise AssertionError('unknown meaning lists must be rejected')


def close(a, b, tolerance=0.000003):
    assert math.isclose(a, b, abs_tol=tolerance), (a, b)


def review_ranges(configs, profiles, records, summary, families):
    """Audit mathematical domains of authored values, not product feasibility."""
    def span(values):
        assert values and all(math.isfinite(v) for v in values)
        return {'min': min(values), 'max': max(values), 'n': len(values), 'mean': mean(values)}

    reviewed = {}
    for c in configs:
        cid, layer = c['id'], c['stratum']
        rows = {name: [r for r in records[name] if r.get('config_id') == cid]
                for name in ['responses', 'referents', 'goals', 'changes', 'usage', 'memory', 'stages']}
        q, ref, goals, changes, usage, memory, stages = (rows[name] for name in
                ['responses', 'referents', 'goals', 'changes', 'usage', 'memory', 'stages'])
        nonnegative = ([r['qa01_s'] for r in q] + [r['via_s'] for r in goals] +
                       [r['wall_s'] for r in goals] + [r['duration_s'] for r in stages] +
                       [r[k] for r in usage for k in ['amount', 'rate_usd', 'usd']] +
                       [r['total_mb'] for r in memory] +
                       [v for r in memory for v in r['components_mb'].values()])
        assert all(math.isfinite(v) and v >= 0 for v in nonnegative), cid
        assert all(r['via_s'] <= r['wall_s'] + 3e-6 for r in goals), cid
        assert all(r['correct'] in (0, 1) for r in ref), cid
        assert all(r['qa07_success'] in (0, 1) for r in q), cid
        assert all(isinstance(r['module_count'], int) and r['module_count'] >= 0
                   and r['module_count'] == len(r['modules']) == len(set(r['modules'])) for r in changes), cid
        s = summary[cid]
        assert 0 <= s['qa02'] <= 1 and 0 <= s['qa07'] <= 1
        case_costs, case_peaks = defaultdict(float), {}
        for u in usage:
            if u['role'] == 'VIA': case_costs[u['case_id']] += u['usd']
        for m in memory:
            case_peaks[m['case_id']] = max(case_peaks.get(m['case_id'], 0), m['total_mb'])
        assert len(case_costs) == len(case_peaks) == profiles[layer]['episodes'], cid
        close(sum(case_costs.values()), s['qa05'], 1e-8)
        base_mb = 238 if layer == 'TEXT' else 560
        assert all(m['total_mb'] >= base_mb for m in memory), cid
        offset_values = [r[k] for r in stages for k in ['start_s', 'end_s']]
        offset_values += [r['time_s'] for r in memory] + [r['start_s'] for r in goals]
        assert all(math.isfinite(t) for t in offset_values), cid
        voice = [r['qa01_voice_s'] for r in q if r['qa01_voice_s'] is not None]
        assert all(math.isfinite(v) and v >= 0 for v in voice), cid
        if voice:
            assert all(r['qa01_voice_s'] >= r['qa01_s'] - 1e-7 for r in q)
        peak_span = span(list(case_peaks.values()))
        peak_span.pop('mean')  # The mean of episode peaks is not QA06.
        peak_span['peak_witness'] = s['peak_witness']
        cost_span = span(list(case_costs.values()))
        cost_span.pop('mean')  # The mean cost per case is not QA05.
        cost_span.update(total_usd=s['qa05'], basis='VIA_COST_PER_CASE_ALL_PHASES')
        margin = 512 - s['qa06']
        fs = [r for r in families if r['config_id'] == cid]
        reviewed[cid] = {
            'config_id': cid, 'stratum': layer, 'profile_id': c['profile_id'],
            'qa01': span([r['qa01_s'] for r in q]),
            'qa02': {'correct': s['referent_correct'], 'denominator': s['referent_n'], 'ratio': s['qa02']},
            'qa03': span([r['via_s'] for r in goals]),
            'qa04': span([r['module_count'] for r in changes]), 'qa05': cost_span, 'qa06': peak_span,
            'qa07': {'correct': s['meaning_correct'], 'denominator': s['input_n'], 'ratio': s['qa07']},
            'first_partial_display_n': sum(r['first_is_partial'] for r in q),
            'first_voice_latency': {**span(voice), 'status': 'AUTHORED_AVAILABLE'} if voice else
                {'min': None, 'max': None, 'mean': None, 'n': 0, 'status': 'NOT_MODELED'},
            'goal_wall_seconds': span([r['wall_s'] for r in goals]),
            'family_mean_ranges': {key: {'min': min(r[key] for r in fs), 'max': max(r[key] for r in fs),
                                         'family_n': len(fs)} for key in ['qa01', 'qa02', 'qa03', 'qa07']},
            'family_statistics': fs,
            'timestamp_offsets': {'min': min(offset_values), 'max': max(offset_values),
                'negative_count': sum(t < 0 for t in offset_values), 'negative_allowed': True,
                'reason': 'Episode-relative preparation and original-goal timestamps may precede evaluation t=0; latency and duration cannot be negative.'},
            'memory_budget_reference': {'reference_mb': 512, 'model_peak_mb': s['qa06'],
                'model_base_mb': base_mb, 'model_base_excess_mb': max(0, base_mb - 512),
                'excess_mb': max(0, -margin), 'headroom_mb': max(0, margin), 'signed_headroom_mb': margin,
                'status': 'MODEL_EXCEEDS' if margin < 0 else 'BELOW_MODEL_ONLY',
                'real_qualification': False, 'score_assigned': None,
                'source': 'docs/architecture/12-decisions/decision-packages/qa-candidate-package/targets-source-qa06-platform.md'},
            'domain_status': 'PASS_AUTHORED_DOMAIN_ONLY',
            'domain_checks': ['all durations, latencies, quantities, rates, cost and RAM are finite/nonnegative',
                'exact-set and all-meaning verdicts are binary; aggregate ratios are within [0,1]',
                'change counts equal unique conceptual module integers',
                'goal projected VIA duration does not exceed its authored wall duration',
                'all RAM states meet the explicit profile baseline floor',
                'negative timestamp offsets are retained separately from nonnegative durations'],
        }
    # Audio source_offset_s is a position within audio, unlike episode-relative
    # source arrival timestamps: it must not be negative.
    assert all(math.isfinite(p['source_offset_s']) and p['source_offset_s'] >= 0 for p in records['playback'])
    numbers = {cid: {key: row[key] for key in [f'qa{i:02}' for i in range(1, 8)] +
                    (['qa01_voice'] if row['qa01_voice'] is not None else [])} for cid, row in summary.items()}
    fingerprint = sha(json.dumps(numbers, sort_keys=True, separators=(',', ':')).encode())
    assert fingerprint == FIRST_V3_NUMERIC_SHA256, 'Review metadata must preserve the first published v3 63+2 values'
    return {'dataset_id': VERSION, 'review_revision': REVIEW_REVISION, 'evidence_kind': EVIDENCE,
        'status': 'PASS_AUTHORED_DOMAIN_ONLY', 'configs': reviewed,
        'numeric_preservation': {'reference_commit': FIRST_V3_COMMIT, 'primary_values': 63,
            'separate_voice_values': 2, 'canonical_numeric_sha256': fingerprint, 'unchanged': True},
        'metric_catalog': METRIC_CATALOG,
        'interpretation': ['Min/max describe authored cases, not confidence intervals or empirical model limits.',
            'Family QA03 means are not averaged again: the original goal denominator weights the headline.',
            'QA05 ranges are per-case VIA costs including all phases; headline QA05 remains the whole-stratum activity sum.',
            'QA06 ranges contain one simultaneous peak per case; the headline is their maximum and alternatives may peak at different cases/times.',
            '512 MB is a preserved system-budget reference, not a new scoring boundary or a qualification result.',
            'VOICE and DELIVERY base 560 MB already exceeds the 512 MB reference; conditional comparisons do not establish a deployable product.',
            'No score bands or thresholds from another population are applied; no empirical realism validation is claimed.'],
        'audio_source_offset_domain': 'NONNEGATIVE_AUDIO_POSITION; distinct from possibly negative source-arrival timestamps'}


def build():
    verify_semantic_policy()
    OUT.mkdir(parents=True, exist_ok=True)
    parent_records, data = [], {}
    existing_manifest = json.loads((OUT / 'manifest.json').read_text()) if (OUT / 'manifest.json').exists() else None
    pinned_hashes = {p['stratum']: p['sha256'] for p in existing_manifest['parent_sources']} if existing_manifest else {}
    for layer, relative in PARENTS.items():
        live = ROOT / relative
        frozen = OUT / 'parents' / (layer.lower() + '.json')
        if not frozen.exists():
            frozen.parent.mkdir(parents=True, exist_ok=True)
            frozen.write_bytes(live.read_bytes())
        payload = frozen.read_bytes()
        if layer in SOURCE_SHA256:
            assert sha(payload) == SOURCE_SHA256[layer], ('published parent changed', layer)
        if layer in pinned_hashes:
            assert sha(payload) == pinned_hashes[layer], ('frozen parent changed', layer)
        data[layer] = json.loads(payload)
        parent_records.append({'stratum': layer, 'source_path': relative,
                               'snapshot_path': str(frozen.relative_to(OUT)),
                               'sha256': sha(payload), 'bytes': len(payload),
                               'source_commit': SOURCE_COMMITS[layer],
                               'parent_dataset_id': data[layer].get('dataset_id', data[layer].get('version'))})
    text, voice = data['TEXT'], data['VOICE']
    assert text['version'] == 'COMMON-QA-SYN-1.0'
    assert voice['dataset_id'] == 'DP3-GROUNDING-CPU-SYN-1.0'
    assert all(d['evidence_kind'] == EVIDENCE for d in data.values())
    delivery = data['DELIVERY']
    assert delivery['version'] == 'DP44-DELIVERY-SYN-1.0'
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
    profiles['DELIVERY'] = {'id': delivery['assumptions']['profile'], 'parent_dataset': delivery['version'],
        'start_event': 'ACOUSTIC_UTTERANCE_END', 'assumptions': delivery['assumptions'],
        'episodes': 80, 'inputs': 80, 'referents': 120, 'goals': 90, 'prior_user_activities': 80,
        'background_preparation_in_cost_and_memory': True,
        'semantic_evidence': 'FIXED_CORRECT_SEMANTIC_FIXTURE',
        'receiver_cost_status': 'NOT_MODELED'}
    for c in configs: c['dp44'] = 'FIXED_UNMAPPED'
    configs.extend({'id': 'D-' + option, 'stratum': 'DELIVERY', 'parent_config': option,
        'dp2': 'FIXED_UNMAPPED', 'dp3': 'FIXED_UNMAPPED', 'dp5': 'FIXED_UNMAPPED', 'dp44': option}
        for option in ['A', 'B'])
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
        correct = int(canonical_meaning(r['expected_meaning']) == canonical_meaning(r['adopted_meaning']))
        assert correct == r['qa07']
        close(t['first_s'] - t['start_s'], t['qa01_s'])
        records['responses'].append({**base('TEXT', r['config'], r['episode_id']),
            'family_id': 'T-' + r['family_id'], 'input_id': 'TEXT:' + r['input_id'], 'revision': r['revision'],
            'goal_id': 'TEXT:' + r['goal_id'], 'start_s': t['start_s'], 'first_display_s': t['first_s'],
            'qa01_s': t['qa01_s'], 'expected': r['expected_meaning'], 'adopted': r['adopted_meaning'],
            'qa07_success': correct, 'first_content': r['first_content'], 'error_reason': r['error_reason'],
            'adopted_at_s': r['full_meaning_adopted_s'], 'adoption_interval': None,
            'first_is_partial': r['first_is_partial'],
            'first_display_node_id': 'TEXT:' + t['first_node'],
            'full_meaning_adoption_node_id': 'TEXT:' + t['adopt_node']})
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
        qa07 = int(canonical_meaning(r['oracle']) == canonical_meaning(r['first_adopted']))
        assert qa02 == r['qa02_success'] and qa07 == r['qa07_success']
        records['responses'].append({**common, 'family_id': 'V-' + r['family'], 'input_id': 'VOICE:' + r['input_id'],
            'revision': 1, 'goal_id': 'VOICE:' + r['goal_id'], 'start_s': 0, 'first_display_s': first['end'],
            'qa01_s': first['end'], 'expected': r['oracle'], 'adopted': r['first_adopted'],
            'qa07_success': qa07, 'first_content': r['first_response'], 'error_reason': r['error_reason'],
            'adopted_at_s': None, 'adoption_interval': [first['start'], first['end']],
            'first_is_partial': False, 'first_display_node_id': 'VOICE:' + first['id'],
            'full_meaning_adoption_node_id': 'VOICE:' + first['id']})
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

    # DELIVERY is an additional profile, never a DP2/3/5 cross combination.
    for row in records['responses']:
        row.update(qa01_voice_s=None, first_voice_s=None, qa01_voice_status='NOT_MODELED')
    for row in records['goals']:
        row['terminal_nodes'] = [row['terminal_node']]
    for key in ['sources', 'jobs', 'playback', 'history', 'delivery_diagnostics', 'preparation_activities']:
        records[key] = []
    dnodes = {n['id']: n for n in delivery['nodes']}
    djobs = {j['id']: j for j in delivery['jobs']}
    dinputs = {i['id']: i for i in delivery['inputs']}
    assert len(dnodes) == len(delivery['nodes']) and len(djobs) == len(delivery['jobs'])
    def did(value): return 'DELIVERY:' + value
    def dbase(row): return base('DELIVERY', row['option'], row['episode'])
    for i, inp in enumerate(delivery['inputs']):
        goal_ids = sorted({g['id'] for g in delivery['goals'] if g['episode'] == inp['episode']})
        current_goal = next(g for g in goal_ids if not g.endswith(':report_original'))
        expressions = [{'id': did(e['id']), 'source_expression_id': e['id'],
                        'text': e['expression'], 'expected': e['oracle']} for e in inp['expressions']]
        records['scenarios'].append({'case_id': did(inp['episode']), 'stratum': 'DELIVERY',
            'family_id': 'D-' + inp['family'], 'goal_ids': list(map(did, goal_ids)),
            'inputs': [{'id': did(inp['id']), 'source_input_id': inp['id'], 'goal_id': did(current_goal),
                'source_goal_id': current_goal, 'related_goal_ids': list(map(did, goal_ids)),
                'revision': 1, 'text': inp['text'], 'oracle': inp['oracle_meaning'],
                'referents': expressions, 'acoustic_start_s': inp['acoustic_start_s'],
                'acoustic_end_s': inp['acoustic_end_s'], 'fixture_variant': inp['fixture_variant']}],
            'prior_user_activity': inp['setup_request'],
            'source_ids': sorted({did(s['id']) for s in delivery['sources'] if s['episode'] == inp['episode']}),
            'parent_source_ids': sorted({s['id'] for s in delivery['sources'] if s['episode'] == inp['episode']}),
            'source_locator': f'parents/delivery.json#/inputs/{i}',
            'semantic_evidence': 'FIXED_CORRECT_SEMANTIC_FIXTURE'})
        records['preparation_activities'].append({'stratum': 'DELIVERY', 'case_id': did(inp['episode']),
            'id': did(inp['episode'] + ':setup'), 'start_s': inp['setup_start_s'],
            'request': inp['setup_request'], 'accuracy_denominator_member': False,
            'cost_and_memory_included': True, 'original_goal_id': next((did(g) for g in goal_ids if g.endswith(':report_original')), None)})
    for n in delivery['nodes']:
        close(n['start_s'] + n['duration_s'], n['end_s'], 2e-8)
        if n['parents']:
            assert all(dnodes[p]['episode'] == n['episode'] and dnodes[p]['option'] == n['option'] for p in n['parents'])
            close(n['start_s'], max(dnodes[p]['end_s'] for p in n['parents']), 2e-8)
        records['stages'].append({**dbase(n), 'id': did(n['id']), 'start_s': n['start_s'],
            'end_s': n['end_s'], 'duration_s': n['duration_s'], 'kind': n['name'],
            'role': 'VIA' if n['via'] else 'EXCLUDED_EXTERNAL_OR_PLAYBACK_WAIT',
            'included': n['via'], 'parents': list(map(did, n['parents'])),
            'source_scope': 'preparation' if n['end_s'] <= 0 else 'evaluation',
            'source_locator': 'parents/delivery.json#/nodes/' + str(delivery['nodes'].index(n))})
    for r in delivery['responses']:
        inp = dinputs[r['input_id']]
        # Order is meaningful in DP44 (calendar, mail, report). Keep full meaning equality.
        qa07 = int(canonical_meaning(inp['oracle_meaning']) == canonical_meaning(r['adopted_meaning']))
        assert qa07 == r['qa07_correct'] == 1
        ui = dnodes[r['episode'] + ':' + r['option'] + ':evaluation_first_display']
        close(ui['end_s'], r['first_ui_s'])
        job = djobs[r['voice_job']]
        plays = [p for p in delivery['playback'] if p['job_id'] == job['id'] and p['start_s'] >= 0]
        close(min(p['start_s'] for p in plays), r['first_voice_s'])
        scenario = next(s for s in records['scenarios'] if s['case_id'] == did(r['episode']))
        records['responses'].append({**dbase(r), 'family_id': 'D-' + r['family'],
            'input_id': did(r['input_id']), 'goal_id': scenario['inputs'][0]['goal_id'], 'revision': 1,
            'start_s': inp['acoustic_end_s'], 'first_display_s': r['first_ui_s'],
            'qa01_s': r['first_ui_s'] - inp['acoustic_end_s'],
            'first_voice_s': r['first_voice_s'], 'qa01_voice_s': r['first_voice_s'] - inp['acoustic_end_s'],
            'qa01_voice_status': 'AUTHORED_AVAILABLE', 'first_voice_content': r['first_voice_text'],
            'voice_job_id': did(r['voice_job']), 'expected': inp['oracle_meaning'], 'adopted': r['adopted_meaning'],
            'qa07_success': qa07, 'first_content': r['first_ui_text'],
            'error_reason': 'FIXED_CORRECT_SEMANTIC_FIXTURE; not model accuracy evidence',
            'semantic_evidence': 'FIXED_CORRECT_SEMANTIC_FIXTURE',
            'adopted_at_s': r['first_meaning_s'], 'adoption_interval': None,
            'first_is_partial': False, 'first_display_node_id': did(ui['id']),
            'full_meaning_adoption_node_id': did(r['episode'] + ':' + r['option'] + ':meaning_adoption')})
        for expression in inp['expressions']:
            correct = int(set(expression['oracle']) == set(expression['adopted']))
            assert correct == 1
            records['referents'].append({**dbase(r), 'family_id': 'D-' + r['family'],
                'input_id': did(inp['id']), 'expression_id': did(expression['id']), 'revision': 1,
                'expected': expression['oracle'], 'adopted': expression['adopted'], 'correct': correct,
                'semantic_evidence': 'FIXED_CORRECT_SEMANTIC_FIXTURE'})
    for g in delivery['goals']:
        memo, visiting = {}, set()
        def project(nid):
            if nid in memo: return memo[nid]
            assert nid not in visiting, ('cycle', nid)
            visiting.add(nid)
            n = dnodes[nid]
            assert (n['episode'], n['option']) == (g['episode'], g['option'])
            value = 0 if n['end_s'] <= g['start_s'] else max([0] + [project(p) for p in n['parents']]) + (max(0, n['end_s'] - max(n['start_s'], g['start_s'])) if n['via'] else 0)
            visiting.remove(nid); memo[nid] = value
            return value
        terminal = max(g['terminals'], key=project)
        close(project(terminal), g['qa03_s'], 2e-8)
        inp = next(i for i in delivery['inputs'] if i['episode'] == g['episode'])
        records['goals'].append({**dbase(g), 'family_id': 'D-' + inp['family'], 'goal_id': did(g['id']),
            'input_ids': [did(inp['id'])], 'via_s': project(terminal),
            'wall_s': max(dnodes[n]['end_s'] for n in g['terminals']) - g['start_s'],
            'wall_boundary': 'includes authored playback and waits; not QA03',
            'start_s': g['start_s'], 'terminal_node': did(terminal), 'terminal_nodes': list(map(did, g['terminals'])),
            'terminal_content_correct': int(g['terminal_content_correct']), 'status': 'AUTHORED_TERMINAL',
            'description': g['description'], 'original_goal': g['id'].endswith(':report_original'),
            'preparation_activity_id': did(inp['episode'] + ':setup') if g['id'].endswith(':report_original') else None})
    job_by_node = {j['full_node']: j for j in delivery['jobs']}
    for u in delivery['usage']:
        activity_start = job_by_node[u['node_id']]['start_s'] if u['node_id'] in job_by_node else dnodes[u['node_id']]['start_s']
        # Whole activity classification preserves work crossing t=0; not time-prorated billing.
        phase = 'preparation' if u['phase'] == 'setup' or activity_start < 0 else 'scored'
        buckets = [('text_input', 'input_rate'), ('text_output', 'output_rate'),
                   ('audio_input_s', 'audio_input_rate'), ('audio_output_s', 'audio_output_rate')]
        close(sum(u[k] * u[rate] for k, rate in buckets), u['amount_usd'], 2e-8)
        for bucket, rate in buckets:
            if not u[bucket]: continue
            name = bucket.replace('_s', '_seconds') if bucket.startswith('audio_') else bucket
            records['usage'].append({**dbase(u), 'id': did(u['id'] + ':' + name), 'node_id': did(u['node_id']),
                'phase': phase, 'role': 'VIA', 'source_role': u['role'], 'source_phase': u['phase'],
                'activity_start_s': activity_start, 'bucket': name, 'amount': u[bucket],
                'rate_usd': u[rate], 'usd': u[bucket] * u[rate], 'reason': u['phase'],
                'discarded': u['phase'] == 'discarded'})
    for i, m in enumerate(delivery['memory']):
        # Independently reconstruct owned live audio/metadata from jobs and playback.
        t = m['at_s']; payload, live, active = 0., 0, 0
        for j in delivery['jobs']:
            if (j['episode'], j['option']) != (m['episode'], m['option']): continue
            if t < j['start_s'] or (j['release_s'] is not None and t > j['release_s']): continue
            live += 1; active += int(t < j['end_s'])
            gen = 0 if t < j['first_s'] else min(j['generated_audio_s'], .2 + max(0, t - j['first_s']) * delivery['assumptions']['generation_rate'])
            consumed = max([0] + [p['source_offset_s'] + min(max(0, t - p['start_s']), p['audio_seconds']) for p in delivery['playback'] if p['job_id'] == j['id'] and t >= p['start_s']])
            payload += max(0, gen - consumed) * delivery['assumptions']['pcm_bytes_per_second'] / 1e6
        close(payload, m['audio_mb'], 2e-7)
        close(live * delivery['assumptions']['job_metadata_bytes'] / 1e6, m['metadata_mb'], 2e-7)
        assert (live, active) == (m['live_jobs'], m['active_generation'])
        assert active <= delivery['assumptions']['output_generation_slots'] and payload < delivery['assumptions']['audio_ram_cap_mb']
        components = {k: m[k] for k in ['base_mb', 'audio_mb', 'metadata_mb']}
        close(sum(components.values()), m['total_mb'], 2e-8)
        records['memory'].append({**dbase(m), 'id': f'DELIVERY:M{i}', 'time_s': t, 'end_s': None,
            'phase': 'preparation' if t < 0 else 'scored', 'components_mb': components, 'total_mb': m['total_mb'],
            'observation': 'AUTHORED_POINT_STATE_RECONSTRUCTED_FROM_JOBS', 'active_generation': active, 'live_jobs': live})
    for ch in delivery['changes']:
        spec = next(x for x in text['changes'] if x['id'] == ch['id'])
        assert ch['title'] == spec['title'] and ch['acceptance'] == spec['acceptance']
        records['changes'].append({**base('DELIVERY', ch['option'], ch['id']), 'change_id': ch['id'],
            'title': ch['title'], 'acceptance': ch['acceptance'], 'modules': ch['modules'],
            'module_count': len(set(ch['modules'])), 'reason': ch['reason'],
            'evidence_kind': 'SYNTHETIC_DESIGN_CHANGE_REVIEW', 'status': 'DESIGN_CHANGE_REVIEW_NOT_RUST_DIFF'})
    for s in delivery['sources']:
        records['sources'].append({**dbase(s), **copy.deepcopy(s), 'source_id': did(s['id']),
            'id': did(s['id']) + ':' + s['option'], 'node_id': did(s['node_id']), 'display_node': did(s['display_node'])})
    for j in delivery['jobs']:
        records['jobs'].append({**dbase(j), **copy.deepcopy(j), 'id': did(j['id']), 'source_id': did(j['source_id']),
            'first_node': did(j['first_node']), 'full_node': did(j['full_node'])})
    for i, p in enumerate(delivery['playback']):
        j = djobs[p['job_id']]
        assert not j['discarded']
        assert p['start_s'] >= j['first_s'] - 2e-7
        assert p['source_offset_s'] + p['audio_seconds'] <= j['generated_audio_s'] + 2e-7
        inp = next(x for x in delivery['inputs'] if x['episode'] == p['episode'])
        if p['start_s'] < 0: assert p['end_s'] <= inp['acoustic_start_s'] + .0800001
        else:
            resp = next(x for x in delivery['responses'] if (x['episode'], x['option']) == (p['episode'], p['option']))
            assert p['start_s'] >= resp['first_meaning_s']
        records['playback'].append({**dbase(p), **copy.deepcopy(p), 'id': f'DELIVERY:P{i}',
            'job_id': did(p['job_id']), 'source_id': did(p['source_id'])})
    for inp in delivery['inputs']:
        for opt in ['A', 'B']:
            ps = sorted([p for p in delivery['playback'] if (p['episode'], p['option']) == (inp['episode'], opt)], key=lambda p:p['start_s'])
            assert all(a['end_s'] <= b['start_s'] + 2e-7 for a,b in zip(ps, ps[1:]))
    for i, h in enumerate(delivery['history']):
        assert not h['claims_heard']
        records['history'].append({**dbase(h), **copy.deepcopy(h), 'id': f'DELIVERY:H{i}',
            'response_id': did(h['response_id']) if h['response_id'] else None})
    for i, d in enumerate(delivery['deliveries']):
        records['delivery_diagnostics'].append({**dbase(d), **copy.deepcopy(d), 'id': f'DELIVERY:D{i}', 'source_id': did(d['source_id'])})
    # Link the delivery diagnostics to actual authored source, generation,
    # playback and usage rows. Source availability is not input QA01.
    dsources = {(s['option'], s['id']): s for s in delivery['sources']}
    for j in delivery['jobs']:
        source = dsources[(j['option'], j['source_id'])]
        assert source['revision'] == j['revision']
        assert j['start_s'] >= source['available_s'] - 2e-8
        first, full = dnodes[j['first_node']], dnodes[j['full_node']]
        close(first['start_s'], j['start_s'], 2e-8)
        close(first['end_s'], j['first_s'], 2e-8)
        close(full['end_s'], j['end_s'], 2e-8)
        usage = next(u for u in delivery['usage'] if u['node_id'] == j['full_node'])
        close(usage['audio_output_s'], j['generated_audio_s'], 2e-8)
        assert usage['text_input'] == j['input_tokens']
        if j['id'].endswith(':resume_suffix'):
            cached = next(old for old in delivery['jobs'] if old['option'] == j['option']
                          and old['source_id'] == j['source_id'] and old['id'] != j['id'])
            close(j['requested_audio_s'], source['spoken_seconds'] - cached['generated_audio_s'], 2e-8)
    for diagnostic in delivery['deliveries']:
        source = dsources[(diagnostic['option'], diagnostic['source_id'])]
        close(source['available_s'], diagnostic['source_available_s'], 2e-8)
        close(dnodes[source['display_node']]['end_s'], diagnostic['display_s'], 2e-8)
        plays = [p for p in delivery['playback'] if p['option'] == diagnostic['option']
                 and p['source_id'] == diagnostic['source_id']]
        if plays:
            first = min(p['start_s'] for p in plays)
            close(first, diagnostic['first_audio_s'], 2e-8)
            close(first - source['available_s'], diagnostic['source_to_audio_s'], 2e-8)
        else:
            assert diagnostic['first_audio_s'] is None and diagnostic['source_to_audio_s'] is None
    summary, families = {}, []
    for c in configs:
        cid, layer = c['id'], c['stratum']
        rows = {k: [r for r in records[k] if r.get('config_id') == cid] for k in ['responses','referents','goals','usage','memory','changes']}
        q, ref, goals, usage, memory, changes = (rows[k] for k in ['responses','referents','goals','usage','memory','changes'])
        peak = max(memory, key=lambda r:r['total_mb'])
        value = {'config_id': cid, 'stratum': layer, 'profile_id': c['profile_id'],
            'qa01': mean(r['qa01_s'] for r in q), 'qa02': mean(r['correct'] for r in ref),
            'qa03': mean(r['via_s'] for r in goals), 'qa04': mean(r['module_count'] for r in changes),
            'qa05': sum(r['usd'] for r in usage if r['role'] == 'VIA'), 'qa06': peak['total_mb'],
            'qa07': mean(r['qa07_success'] for r in q),
            'qa01_voice': mean(r['qa01_voice_s'] for r in q) if layer == 'DELIVERY' else None,
            'qa01_voice_status': 'AUTHORED_AVAILABLE' if layer == 'DELIVERY' else 'NOT_MODELED',
            'semantic_evidence': 'FIXED_CORRECT_SEMANTIC_FIXTURE' if layer == 'DELIVERY' else 'AUTHORED_SEMANTIC_OUTCOMES',
            'input_n': len(q), 'referent_n': len(ref), 'goal_n': len(goals), 'change_n': len(changes),
            'referent_correct': sum(r['correct'] for r in ref), 'meaning_correct': sum(r['qa07_success'] for r in q),
            'first_missing_n': sum(r['qa01_s'] is None for r in q), 'terminal_missing_n': sum(r['via_s'] is None for r in goals),
            'terminal_content_incorrect_n': sum(not r['terminal_content_correct'] for r in goals),
            'preparation_usd': sum(r['usd'] for r in usage if r['role'] == 'VIA' and r['phase'] == 'preparation'),
            'maintenance_usd': sum(r['usd'] for r in usage if r['role'] == 'VIA' and r['phase'] == 'maintenance'),
            'scored_usd': sum(r['usd'] for r in usage if r['role'] == 'VIA' and r['phase'] == 'scored'),
            'discarded_usd': sum(r['usd'] for r in usage if r.get('discarded')) if layer == 'DELIVERY' else None,
            'receiver_cost': None if layer == 'DELIVERY' else sum(r['usd'] for r in usage if r['role'] == 'RECEIVER'),
            'receiver_cost_status': 'NOT_MODELED' if layer == 'DELIVERY' else 'AUTHORED_AVAILABLE', 'peak_witness': peak}
        value['total_cost'] = None if value['receiver_cost'] is None else value['qa05'] + value['receiver_cost']
        value['total_cost_status'] = value['receiver_cost_status']
        close(value['preparation_usd'] + value['maintenance_usd'] + value['scored_usd'], value['qa05'], 1e-9)
        expected = profiles[layer]
        assert [len(q), len(ref), len(goals), len(changes)] == [expected['inputs'], expected['referents'], expected['goals'], 6]
        if layer == 'DELIVERY':
            old = delivery['summary'][c['parent_config']]
            keys = ['qa01_display_s','qa02','qa03_s','qa04_modules','qa05_usd','qa06_mb','qa07']
            for n, key in enumerate(keys,1): close(value[f'qa{n:02}'], old[key], 2e-8)
            close(value['qa01_voice'], old['qa01_voice_s'], 2e-8)
            close(value['discarded_usd'], old['discarded_cost_usd'], 2e-8)
            value['qa04_sensitivity'] = {'condition': 'common section/extent and permission guard absorb C04/C06 additions',
                'qa04_if_shared_interfaces': 2.0, 'status': 'DESIGN_ASSUMPTION_NOT_MEASURED', 'not_a_tactic_id': True}
        else:
            # All old values are recomputed from immutable v2 parent inputs, never copied.
            old = data['V2_SUMMARY']['configs'][cid]
            for n in range(1,8): close(value[f'qa{n:02}'], old[f'qa{n:02}'], 1e-10)
        summary[cid] = value
        for family in sorted({r['family_id'] for r in q}):
            fq, fr, fg = ([r for r in source if r['family_id'] == family] for source in [q,ref,goals])
            families.append({'config_id': cid, 'stratum': layer, 'family_id': family,
                'qa01': mean(r['qa01_s'] for r in fq), 'qa02': mean(r['correct'] for r in fr),
                'qa03': mean(r['via_s'] for r in fg), 'qa07': mean(r['qa07_success'] for r in fq),
                'qa01_voice': mean(r['qa01_voice_s'] for r in fq) if layer == 'DELIVERY' else None,
                'qa01_voice_status': value['qa01_voice_status'], 'semantic_evidence': value['semantic_evidence'],
                'input_n': len(fq), 'referent_n': len(fr), 'goal_n': len(fg)})
    pairs = [('DP2','T-BASE','T-EXEC'),('DP5','T-BASE','T-REL'),('DP3 A vs R1','V-A','V-R1'),
        ('DP3 A vs R2','V-A','V-R2'),('DP3 A vs R3','V-A','V-R3'),
        ('DP3 residency','V-R1','V-R2'),('DP3 isolation','V-R2','V-R3'),('DP44','D-A','D-B')]
    comparisons = []
    for name, a, b in pairs:
        assert summary[a]['profile_id'] == summary[b]['profile_id']
        comparisons.append({'comparison': name, 'baseline': a, 'alternative': b, 'stratum': summary[a]['stratum'],
            'difference_direction': 'alternative minus baseline',
            'delta': {f'qa{n:02}': summary[b][f'qa{n:02}'] - summary[a][f'qa{n:02}'] for n in range(1,8)},
            'qa01_voice_delta': summary[b]['qa01_voice'] - summary[a]['qa01_voice'] if name == 'DP44' else None})
    # Joins are configuration-aware: no other option/profile can satisfy a reference.
    stages = {(n['config_id'],n['id']):n for n in records['stages']}
    assert len(stages) == len(records['stages'])
    for stage in records['stages']:
        assert all((stage['config_id'],p) in stages for p in stage['parents'])
        close(stage['end_s'] - stage['start_s'], stage['duration_s'])
    for usage in records['usage']:
        assert (usage['config_id'],usage['node_id']) in stages
        close(usage['amount'] * usage['rate_usd'], usage['usd'],1e-10)
    for memory in records['memory']:
        close(sum(memory['components_mb'].values()),memory['total_mb'])
        if 'node_id' in memory: assert (memory['config_id'],memory['node_id']) in stages
    input_registry = {i['id']:i for s in records['scenarios'] for i in s['inputs']}
    expression_registry = {e['id']:e for i in input_registry.values() for e in i['referents']}
    goal_registry = {i['goal_id'] for i in input_registry.values()} | {g for s in records['scenarios'] for g in s.get('goal_ids',[])}
    assert (len(input_registry),len(expression_registry),len(goal_registry)) == (440,490,390)
    for kind,key in [('responses','input_id'),('referents','expression_id'),('goals','goal_id')]:
        ids = [(r['config_id'],r[key]) for r in records[kind]]
        assert len(ids) == len(set(ids)),kind
    for r in records['responses']:
        assert r['input_id'] in input_registry and r['goal_id'] in goal_registry
        inp = input_registry[r['input_id']]
        expected = inp.get('oracle',inp.get('expected_meaning'))
        if expected is not None: assert r['expected'] == expected
        close(r['first_display_s']-r['start_s'],r['qa01_s'])
        display_node = stages[(r['config_id'], r['first_display_node_id'])]
        adoption_node = stages[(r['config_id'], r['full_meaning_adoption_node_id'])]
        assert display_node['case_id'] == adoption_node['case_id'] == r['case_id']
        close(display_node['end_s'], r['first_display_s'])
        if r['adoption_interval'] is not None:
            assert r['adopted_at_s'] is None
            assert r['adoption_interval'] == [adoption_node['start_s'], adoption_node['end_s']]
        else:
            close(adoption_node['end_s'], r['adopted_at_s'])
        assert r['qa07_success'] == int(canonical_meaning(r['expected']) == canonical_meaning(r['adopted']))
        if r['first_is_partial']:
            assert r['first_display_s'] < r['adopted_at_s']
    assert sum(r['first_is_partial'] for r in records['responses']) == 70
    for r in records['referents']:
        assert r['input_id'] in input_registry and r['expression_id'] in expression_registry
        assert set(r['expected']) == set(expression_registry[r['expression_id']]['expected'])
        assert r['correct'] == int(set(r['expected']) == set(r['adopted']))
    for g in records['goals']:
        assert g['goal_id'] in goal_registry and all(i in input_registry for i in g['input_ids'])
        assert all((g['config_id'],t) in stages for t in g['terminal_nodes'])
    source_registry = {(s['config_id'],s['source_id']):s for s in records['sources']}
    assert len(source_registry) == len(records['sources'])
    for scenario in records['scenarios']:
        if scenario['stratum'] != 'DELIVERY': continue
        assert scenario['source_ids'] == list(map(did, scenario['parent_source_ids']))
        for cid in ['D-A','D-B']:
            for sid in scenario['source_ids']:
                source = source_registry[(cid,sid)]
                assert source['case_id'] == scenario['case_id'] and source['stratum'] == 'DELIVERY'
                assert (cid,source['node_id']) in stages and (cid,source['display_node']) in stages
    for cid in ['D-A','D-B']:
        originals=[g for g in records['goals'] if g['config_id']==cid and g.get('original_goal')]
        assert len(originals)==10 and all(g['start_s']==-6 for g in originals)
        assert len([g for g in records['goals'] if g['config_id']==cid and not g.get('original_goal')])==80
        # Same correct control fixture and UI endpoint for both structures.
    dq = {r['input_id']:r for r in records['responses'] if r['config_id']=='D-A'}
    for r in records['responses']:
        if r['config_id']=='D-B':
            assert r['expected']==dq[r['input_id']]['expected']==r['adopted']==dq[r['input_id']]['adopted']
            close(r['qa01_s'],dq[r['input_id']]['qa01_s'],1e-10)
    v2manifest=data['V2_MANIFEST']
    for p in parent_records:
        if p['stratum'] in ('TEXT','VOICE'):
            prior=next(x for x in v2manifest['parent_sources'] if x['stratum']==p['stratum'])
            assert p['sha256']==prior['sha256']
            p['original_source_path']=prior['source_path']
            p['upstream_snapshot_path']='docs/presentations_files/qa-common-synthetic-v2-20261021/'+prior['snapshot_path']
    assert sha((OUT/'parents/v2_summary.json').read_bytes())==v2manifest['outputs']['summary.json']
    family_ids=sorted({s['family_id'] for s in records['scenarios']})
    assert len(family_ids)==26 and len(configs)==9 and len(summary)*7==63
    result={'dataset_id':VERSION,'contract_id':'QA-COMMON-SYN-CONTRACT-v3-STRATIFIED',
        'evidence_kind':EVIDENCE,'actual_model_calls':0,'actual_product_runs':0,
        'aggregation_policy':'WITHIN_PROFILE_ONLY_NO_POOLED_HEADLINE',
        'qa01_primary_endpoint':'FIRST_CONCRETE_ADOPTED_CONTENT_DISPLAY',
        'qa01_voice_role':'SEPARATE_DELIVERY_DIAGNOSTIC_NOT_REPLACEMENT_OR_AVERAGE',
        'profiles':profiles,'configs':configs,'parent_sources':parent_records,**records,
        'summary':summary,'family_summary':families,'comparisons':comparisons,
        'family_registry': [{'family_id':fid,'stratum':next(s['stratum'] for s in records['scenarios'] if s['family_id']==fid),
            'case_n':sum(s['family_id']==fid for s in records['scenarios'])} for fid in family_ids],
        'phase_policy': 'DELIVERY usage: whole activity is preparation if it begins before evaluation t=0 or source phase is setup; crossing t=0 is not prorated. Discarded is a separate overlapping subset, not an extra cost.',
        'scope_limitations':['No combined DP2/DP3/DP5/DP44 configuration.',
            'DELIVERY semantic accuracy is fixed-correct fixture, not estimated or measured model accuracy.',
            'DELIVERY receiver Agent cost and role-total cost are NOT_MODELED/null.',
            'DELIVERY original report goal starts at -6; 80 setup activities remain outside input accuracy denominators.',
            'Voice endpoint remains NOT_MODELED/null for legacy strata.',
            'QA04 is conceptual module review; no actual Rust change in this dataset.',
            'First voice and QA03 differences are distinct endpoints; S08 50ms voice difference is not the larger projected QA03 difference.']}
    range_review = review_ranges(configs, profiles, records, summary, families)
    display_policy = {'source_values': 'PRESERVE_UNROUNDED_JSON_AND_CSV',
        'metric_format_source': 'metric_catalog', 'accuracy_storage': 'RATIO_0_TO_1',
        'accuracy_display': 'PERCENT_WITH_NUMERATOR_AND_DENOMINATOR',
        'accuracy_delta_display': 'RAW_RATIO_DELTA_TIMES_100_PERCENTAGE_POINTS',
        'relative_change_display_decimals': 2,
        'precision_meaning': 'Consistent display rounding; not empirical measurement precision.'}
    result.update(review_revision=REVIEW_REVISION, metric_catalog=METRIC_CATALOG,
                  display_policy=display_policy, range_review=range_review,
                  semantic_comparison_policy=SEMANTIC_COMPARISON_POLICY,
                  first_display_disclosure={'partial_input_count': 70,
                      'rule': 'first_content and first_display_node_id describe the first screen content. adopted describes full meaning at adopted_at_s or adoption_interval; never backfill its targets/data_source into an earlier partial display.',
                      'false_partial_flag': 'Not flagged as a partial handling intention; does not imply identical display and meaning-adoption timestamps.'})
    validation={'dataset_id':VERSION,'evidence_kind':EVIDENCE,'status':'PASS',
        'checks':['immutable v2 parent hashes and v2 summary hash','unique configuration-scoped IDs',
            'same-input and expression oracle joins','first display and independent voice endpoint',
            'one field-based semantic policy across all strata; ordered meaning preserved and unknown list fields rejected',
            '70 partial first displays retain separate full-meaning adoption nodes/timestamps without backfill',
            'DELIVERY same fixed-correct semantic fixture A/B','causal DAG with external/playback duration removed',
            'DELIVERY 10 original goals retain -6 start and 80 additional goals',
            '80 setup activities outside 80 evaluation-input and 120-expression denominators',
            'usage amount times tariff, including setup/discarded/cancel tail/resume context',
            'DELIVERY simultaneous RAM independently reconstructed from jobs and playback',
            'usage, memory, goal and parent-stage links stay within configuration',
            'all DELIVERY scenario source IDs join to same-case/configuration source and stage rows',
            'no overlapping playback, discarded audio, or premature source use',
            'QA04 common C01-C06 task identity and unique conceptual modules',
            '49 v2 and 14 DP44 primary values plus 2 voice diagnostics reproduced',
            'first published v3 63+2 raw values unchanged under REVIEW-2 metadata',
            'authored domain/range and model-only memory-budget reference audit',
            'all comparison pairs are within one profile','missing values remain null/NOT_MODELED'],
        'counts':{k:len(v) for k,v in records.items()}, 'family_definitions':26,'configurations':9,
        'primary_qa_values_verified':63,'legacy_qa_values_unchanged':49,'separate_voice_values_verified':2,
        'input_registry_n':440,'expression_registry_n':490,'goal_registry_n':390,
        'model_calls':0,'product_runs':0,'limitations':result['scope_limitations'],
        'review_revision':REVIEW_REVISION,'range_review_status':range_review['status'],
        'first_published_v3_numeric_preservation':range_review['numeric_preservation']}
    write_json(OUT/'normalized-data.json',result)
    write_json(OUT/'summary.json',{'dataset_id':VERSION,'evidence_kind':EVIDENCE,
        'aggregation_policy':result['aggregation_policy'],'configs':summary,'comparisons':comparisons,'families':families,
        'review_revision':REVIEW_REVISION,'metric_catalog':METRIC_CATALOG,'display_policy':display_policy,
        'semantic_comparison_policy':SEMANTIC_COMPARISON_POLICY,
        'first_display_disclosure':result['first_display_disclosure']})
    write_json(OUT/'validation.json',validation)
    write_json(OUT/'range-review.json',range_review)
    with (OUT/'comparison.csv').open('w',newline='') as f:
        fields=['config_id','stratum','profile_id']+[f'qa{n:02}' for n in range(1,8)]+['qa01_voice','qa01_voice_status',
            'semantic_evidence','input_n','referent_n','goal_n','change_n','preparation_usd','maintenance_usd',
            'scored_usd','discarded_usd','receiver_cost','receiver_cost_status','total_cost','total_cost_status','terminal_content_incorrect_n']
        writer=csv.DictWriter(f,fieldnames=fields,extrasaction='ignore',lineterminator='\n');writer.writeheader();writer.writerows(summary.values())
    outputs={p.name:sha(p.read_bytes()) for p in [OUT/'normalized-data.json',OUT/'summary.json',OUT/'validation.json',OUT/'comparison.csv',OUT/'range-review.json']}
    write_json(OUT/'manifest.json',{'dataset_id':VERSION,'evidence_kind':EVIDENCE,'schema_version':'STRATIFIED-LEDGER-2',
        'review_revision':REVIEW_REVISION,
        'corpus_id':'COMMON-QA-CORPUS-3.0-STRATIFIED','contract_id':result['contract_id'],
        'registry_id':'COMMON-QA-CONFIG-3.0','parent_sources':parent_records,'outputs':outputs,
        'generator':{'path':str(Path(__file__).resolve().relative_to(ROOT)),'sha256':sha(Path(__file__).read_bytes())},
        'policy':'Parent snapshots are immutable on rerun. New source/scenario/profile revisions require a new dataset and output directory.'})
    print(json.dumps({'status':'PASS','dataset_id':VERSION,'counts':validation['counts'],
        'primary_qa_values_verified':63,'separate_voice_values_verified':2}))

if __name__=='__main__':
    build()
