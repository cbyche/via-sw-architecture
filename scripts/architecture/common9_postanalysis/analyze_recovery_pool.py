"""Audit the disjoint 257+184 joint evaluation without changing either generation.

The failed parent trace and its spend remain evidence, but its identity is
represented in the primary measurement by the separately frozen rerun only.
"""
import argparse
import hashlib
import json
import statistics
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
SUPPLEMENT = REPO / 'benchmark/architecture/common_supplement6_20261019'
sys.path.insert(0, str(SUPPLEMENT.parent / 'common9_poc_20261019'))
sys.path.insert(0, str(SUPPLEMENT))
import evaluate as supplement  # noqa: E402
from evaluation_v3 import evaluate as evaluate_common9  # noqa: E402
from evaluation_v3.evaluate import summarize as summarize_common9  # noqa: E402
import joint_campaign as joint  # noqa: E402
import recovery_campaign as recovery  # noqa: E402

PARENT = recovery.PARENT
PARENT_ANALYSIS = PARENT / 'postanalysis/stopped258-input-assembly-v1/joint-analysis.json'
PAIRS = {'DP1': (('AAAA', 'BAAA'), ('AAAA', 'CAAA'), ('BAAA', 'CAAA')),
         'DP2': (('AAAA', 'ABAA'),), 'DP3': (('AAAA', 'AABA'),),
         'DP4': (('AAAA', 'AAAB'), ('AAAA', 'AAAC'), ('AAAB', 'AAAC'))}


def identity(row):
    return (row['configuration'], row['episode'], row['rep'])


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def require(value, message):
    if not value:
        raise ValueError(message)


def descriptive_generation_stratum(results, plan):
    """Expose batch effects without treating a partial batch as a balanced trial."""
    output = {}
    for population in ('COMMON9', 'COMMON_SUPPLEMENT6'):
        output[population] = {}
        for configuration in supplement.CONFIGURATIONS:
            rows = [r for r in results if plan[identity(r)]['population'] == population
                    and r['configuration'] == configuration]
            qa01 = [entry['latency_ns'] / 1e6 for row in rows for entry in row['qa01']
                    if entry.get('latency_ns') is not None]
            qa03 = [entry['via_ns'] / 1e6 for row in rows for entry in row['qa03']
                    if entry.get('via_ns') is not None]
            judged = [entry for row in rows for entry in row['qa02']
                      if entry['status'] != 'UNRESOLVED_OR_LATE_ADOPTION']
            peaks = [row['qa06']['peak_concurrent_sum_bytes'] for row in rows
                     if row['qa06']['peak_concurrent_sum_bytes'] is not None]
            output[population][configuration] = {
                'certified_episodes': len(rows),
                'qa01_observed': len(qa01),
                'qa01_observed_mean_ms': statistics.mean(qa01) if qa01 else None,
                'qa02_judged': len(judged),
                'qa02_correct': sum(entry['correct'] for entry in judged),
                'qa03_observed': len(qa03),
                'qa03_observed_mean_ms': statistics.mean(qa03) if qa03 else None,
                'qa05_known_usd': sum(row['qa05']['known_api_cost_usd'] for row in rows),
                'qa06_same_sample_peak_bytes': max(peaks) if peaks else None,
                'qa06_incomplete_episode_count': sum(row['qa06']['status'] != 'OBSERVED' for row in rows),
            }
    return output


def paired_observed(results, plan):
    """Same-obligation timing intersections only; missing endpoints stay missing."""
    by = {(plan[identity(r)]['population'], r['configuration'], r['episode'], r['rep']): r for r in results}
    output = []
    for population in ('COMMON9', 'COMMON_SUPPLEMENT6'):
        cases = joint.common.EPISODES if population == 'COMMON9' else tuple('S' + str(i) for i in range(80, 86))
        reps = 5 if population == 'COMMON9' else 3
        for dp, pairs in PAIRS.items():
            for left, right in pairs:
                for qa, key, metric in (('QA01', 'response_id', 'latency_ns'), ('QA03', 'id', 'via_ns')):
                    differences = []
                    planned = 0
                    for case in cases:
                        for rep in range(1, reps + 1):
                            a = by[(population, left, case, rep)]
                            b = by[(population, right, case, rep)]
                            left_rows = {row[key]: row for row in a[qa.lower()]}
                            right_rows = {row[key]: row for row in b[qa.lower()]}
                            require(set(left_rows) == set(right_rows), 'Paired obligation IDs differ')
                            planned += len(left_rows)
                            for obligation in left_rows:
                                lvalue = left_rows[obligation].get(metric)
                                rvalue = right_rows[obligation].get(metric)
                                if lvalue is not None and rvalue is not None:
                                    differences.append((rvalue - lvalue) / 1e6)
                    output.append({'population': population, 'dp': dp, 'left': left, 'right': right, 'qa': qa,
                                   'paired_observed': len(differences), 'planned': planned,
                                   'right_minus_left_mean_ms': sum(differences) / len(differences) if differences else None,
                                   'incomplete_endpoint_comparison': len(differences) != planned})
    return output


def assess(root):
    parent = json.loads((PARENT / 'freeze.json').read_text())
    child = recovery.validate(root, large=True)
    original = json.loads(PARENT_ANALYSIS.read_text())
    require(original['freeze_sha256'] == digest(PARENT / 'freeze.json'), 'Parent analysis/freeze mismatch')
    require(child['parent']['freeze_sha256'] == original['freeze_sha256'], 'Recovery parent binding mismatch')
    require(parent['binary']['sha256'] == child['binary']['sha256'], 'Candidate binary differs')
    old = {s['relativePath']: s['sha256'] for s in parent['sources']}
    new = {s['relativePath']: s['sha256'] for s in child['sources']}
    changed = sorted(k for k in old.keys() & new.keys() if old[k] != new[k])
    expected_changes = sorted(('benchmark/architecture/common9_poc_20261019/harness.py',
                               'benchmark/architecture/common_supplement6_20261019/harness.py'))
    require(changed == expected_changes and set(new) - set(old) == {'benchmark/architecture/common_supplement6_20261019/recovery_campaign.py'}
            and not set(old) - set(new), 'Source change is not limited to two shutdown handlers plus recovery driver')
    for suffix in ('fixtures.json', 'oracles.json', 'evaluation_v3.py'):
        common = 'benchmark/architecture/common9_poc_20261019/' + suffix
        if common in old:
            require(old[common] == new[common], 'Common method source changed: ' + suffix)
    require([(d['path'], d['sha256']) for d in parent['external_dependencies']] ==
            [(d['path'], d['sha256']) for d in child['external_dependencies']], 'External dependency changed')

    plan = {identity(j): j for j in joint.jobs()}
    failed = identity(recovery.EXPECTED_FAILED)
    parent_results = original['primary_common9']['results'] + original['supplement6']['results']
    require(len(parent_results) == 258 and len({identity(r) for r in parent_results}) == 258, 'Parent result count/identity mismatch')
    parent_by_key = {identity(r): r for r in parent_results}
    require(parent_by_key[failed]['primary_measurement_eligibility'] == 'NOT_CERTIFIED', 'Original failure was reclassified')
    require(all(r['primary_measurement_eligibility'] == 'ELIGIBLE' for key, r in parent_by_key.items() if key != failed),
            'Parent 257 certified assumption differs')
    require(len(original['trace_provenance']) == 258 and len({p['run_id'] for p in original['trace_provenance']}) == 258,
            'Parent raw provenance incomplete')
    for row in original['trace_provenance']:
        source = Path(row['trace_path'])
        require(source.is_file() and digest(source) == row['sha256'], 'Parent raw trace changed: ' + row['run_id'])

    trace_path = root / 'runs.jsonl'
    progress, done = joint.prior_progress(root)
    require(len(done) == 184 and not any(row['state'] == 'TERMINATED' and
            (row.get('returncode') or row.get('technical_failure') or row.get('budget_stop')) for row in progress),
            'Recovery controller has a technical/cost stop')
    require(not any((root / name).exists() for name in ('TECHNICAL-STOP.json', 'BUDGET-STOP.json')),
            'Recovery stop marker exists')
    results, raw, observed = [], [], set()
    with trace_path.open() as source:
        for line in source:
            if not line.strip():
                continue
            index = len(results)
            require(index < len(child['jobs']), 'Too many recovery traces')
            trace = json.loads(line)
            key = identity(trace)
            require(key == identity(child['jobs'][index]) and key not in observed,
                    'Recovery execution order, membership or uniqueness differs')
            observed.add(key)
            require(key in plan and trace['binary_sha256'] == child['binary']['sha256'] and trace['phase'] == 'FORMAL',
                    'Recovery trace candidate or phase differs')
            path = root / 'raw/runs' / trace['run_id'] / 'trace.json'
            candidate_input = Path(trace['candidate_input_file'])
            require(path.is_file() and candidate_input.is_file() and candidate_input.resolve().parent == path.parent.resolve(),
                    'Recovery trace/initial path differs')
            require(json.loads(path.read_text()) == trace, 'Recovery raw trace differs: ' + trace['run_id'])
            require([send['type'] for send in trace['control_sends']] == ['cancel_work'], 'Unexpected recovery shutdown controls')
            require(sum(e.get('kind') == 'WORK_CANCELLATION_ACK' for e in trace['events']) == 1
                    and sum(e.get('kind') == 'EPISODE_RESULT' for e in trace['events']) == 1,
                    'Recovery control ACK/result missing')
            value = evaluate_common9.evaluate(trace) if plan[key]['population'] == 'COMMON9' else supplement.evaluate(trace)
            require(value['primary_measurement_eligibility'] == 'ELIGIBLE', 'Recovery trace is not technically certified: ' + trace['run_id'])
            results.append(value)
            raw.append({'run_id': trace['run_id'], 'identity': dict(zip(('configuration', 'episode', 'rep'), key)),
                        'trace_path': str(path.resolve()), 'sha256': digest(path)})
    require(len(results) == len(observed) == 184, 'Recovery trace count/identity mismatch')
    require({identity(r) for r in results} == {identity(j) for j in child['jobs']}, 'Recovery evaluator coverage differs')
    require(set(parent_by_key) & {identity(r) for r in results} == {failed}, 'Unexpected generation overlap')
    combined = [r for key, r in parent_by_key.items() if key != failed] + results
    require(len(combined) == len(plan) == 441 and {identity(r) for r in combined} == set(plan), 'Combined denominator differs')
    require(original['unknown_call_count'] == 0 and all(r['qa05']['unknown_call_count'] == 0 for r in results),
            'Unknown API usage remains in a measurement generation')
    counts = Counter(plan[identity(r)]['population'] for r in combined)
    require(counts == {'COMMON9': 315, 'COMMON_SUPPLEMENT6': 126}, 'Population count differs')
    primary = [r for r in combined if plan[identity(r)]['population'] == 'COMMON9']
    secondary = [r for r in combined if plan[identity(r)]['population'] == 'COMMON_SUPPLEMENT6']
    budget = joint.budget_snapshot()
    require(budget['limit_usd'] == 40 and budget['remaining_reservation_aware_usd'] >= 0,
            'Global budget cap or ledger differs')
    require(budget['inflight_reservation_usd'] == budget['unknown_reservation_usd'] == 0,
            'Unsettled global budget entries remain')
    parent_formal_cost = original['total_actual_recorded_API_cost_usd']
    recovery_formal_cost = sum(row['qa05']['known_api_cost_usd'] for row in results)
    prior_nonformal_cost = budget['known_usd'] - parent_formal_cost - recovery_formal_cost
    require(prior_nonformal_cost >= -1e-8 and abs(prior_nonformal_cost - 0.7150616) < 1e-6,
            'Global ledger and per-generation cost do not reconcile')
    return {'version': 'JOINT441_RECOVERY_POOL_AUDIT_v1', 'parent_freeze_sha256': original['freeze_sha256'],
            'recovery_freeze_sha256': digest(root / 'freeze.json'), 'parent_analysis_sha256': digest(PARENT_ANALYSIS),
            'source_delta': changed, 'binary_sha256': child['binary']['sha256'],
            'excluded_parent_failed_identity': recovery.EXPECTED_FAILED,
            'original_failed_cost_preserved_in_global_ledger': True,
            'certified_parent': 257, 'certified_recovery': 184, 'primary_population': counts,
            'generation_strata': {
                'stopped_parent_certified_only': descriptive_generation_stratum(
                    [r for key, r in parent_by_key.items() if key != failed], plan),
                'recovery_certified_only': descriptive_generation_stratum(results, plan),
            },
            'budget': budget,
            'cost_reconciliation': {'prior_nonformal_usd': prior_nonformal_cost,
                                    'stopped_parent_formal_usd_including_failed': parent_formal_cost,
                                    'recovery_formal_usd': recovery_formal_cost,
                                    'global_known_usd': budget['known_usd']},
            'primary_common9': {'results': primary, 'summary': summarize_common9(primary, 5)},
            'supplement6': {'results': secondary, 'summary': supplement.summarize(secondary, 3)},
            'paired_observed_intersections': paired_observed(combined, plan),
            'recovery_trace_provenance': raw,
            'pooling_decision': 'CONDITIONALLY_POOL_DISJOINT_CERTIFIED_257_PLUS_184_UNDER_IDENTICAL_FROZEN_METHOD; REPORT_GENERATION_STRATA_AND_DO_NOT_INFER_A_BATCH_INVARIANT_TIMING_OR_RSS_WINNER',
            'limits': ['This is a reference harness Qwen evaluation, not product or physical output latency.',
                       'Timing observations and semantic content failures stay in fixed denominators; no success-only structural winner.',
                       'The older and recovery runs occurred in different wall-clock periods. Provider/network and host load drift are not experimentally controlled.',
                       'QA04 is the earlier separate 42-change/630-regression experiment; no new code-change samples.',
                       'Observed RSS is same-sample process-tree RSS and may vary with host load.']}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--summary-output', type=Path)
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError('Preserve previous result; choose a new output')
    if args.summary_output and args.summary_output.exists():
        raise FileExistsError('Preserve previous compact summary; choose a new output')
    output = assess(args.root.resolve())
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, ensure_ascii=False, indent=2) + '\n')
    if args.summary_output:
        compact = {key: output[key] for key in (
            'version', 'parent_freeze_sha256', 'recovery_freeze_sha256', 'parent_analysis_sha256',
            'source_delta', 'binary_sha256', 'excluded_parent_failed_identity',
            'original_failed_cost_preserved_in_global_ledger', 'certified_parent',
            'certified_recovery', 'primary_population', 'generation_strata', 'budget',
            'cost_reconciliation',
            'paired_observed_intersections', 'pooling_decision', 'limits')}
        compact['primary_common9_summary'] = output['primary_common9']['summary']
        compact['supplement6_summary'] = output['supplement6']['summary']
        compact['full_analysis_sha256'] = digest(args.output)
        args.summary_output.parent.mkdir(parents=True, exist_ok=True)
        args.summary_output.write_text(json.dumps(compact, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'combined': sum(output['primary_population'].values()), 'known_global_usd': output['budget']['known_usd'],
                      'output': str(args.output), 'summary_output': str(args.summary_output) if args.summary_output else None}, ensure_ascii=False))


if __name__ == '__main__':
    main()
