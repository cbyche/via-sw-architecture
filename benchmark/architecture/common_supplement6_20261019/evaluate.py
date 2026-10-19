"""Supplemental host-only oracle adapter and fixed6case/3rep aggregation.

Per-trace QA/certification algorithms are the unchanged method-v3 evaluator.
No common9 module globals or source files are rewritten.
"""
import argparse
import json
import statistics
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
COMMON = HERE.parent / 'common9_poc_20261019'
sys.path.append(str(COMMON))
from evaluation_v3 import evaluate as method_v3

ORACLES = json.loads((HERE / 'oracles.json').read_text())
CONFIGURATIONS = ('AAAA', 'BAAA', 'CAAA', 'ABAA', 'AABA', 'AAAB', 'AAAC')
COUNTS = {key: sum(len(o[key]) for o in ORACLES.values()) for key in ['responses', 'goals', 'bindings']}
assert COUNTS == {'responses': 12, 'goals': 8, 'bindings': 6}


def evaluate(trace, oracle=None, mode='strict'):
    value = method_v3.evaluate(trace, oracle if oracle is not None else ORACLES[trace['episode']], mode)
    value['population'] = 'COMMON_SUPPLEMENT6'
    functional = json.loads((HERE / 'functional-oracles.json').read_text())[trace['episode']]
    value['semantic_diagnostics']['supplement_required_facts'] = {'status': 'MANUAL_REVIEW_REQUIRED_NOT_QA01_FILTER', 'expected': functional, 'actual_displays': [e['data'] for e in trace.get('events', []) if e.get('kind') == 'DISPLAY_PROXY']}
    return value


def regression(trace, oracle=None):
    return method_v3.regression(trace, oracle if oracle is not None else ORACLES[trace['episode']])


def summarize(results, repetitions=3):
    output = {}
    phases = sorted({r.get('phase') for r in results}, key=str) or ['FORMAL']
    for phase in phases:
        for configuration in CONFIGURATIONS:
            rows = [r for r in results if r.get('phase') == phase and r.get('configuration') == configuration]
            identities = [(r['episode'], r['rep']) for r in rows]
            expected = {(episode, rep) for episode in ORACLES for rep in range(1, repetitions + 1)}
            duplicate = len(identities) != len(set(identities))
            unexpected = sorted(set(identities) - expected)
            valid = not duplicate and not unexpected
            complete = valid and set(identities) == expected
            certified = [r for r in rows if r['primary_measurement_eligibility'] == 'ELIGIBLE']
            all_certified = complete and len(certified) == len(rows)
            summary = {'population': 'COMMON_SUPPLEMENT6', 'episodes_observed': len(rows), 'episodes_planned': 6 * repetitions, 'population_complete': complete, 'duplicate_episode_repetitions': duplicate, 'unexpected_episode_repetitions': unexpected, 'missing_episode_repetitions': sorted(expected - set(identities)), 'primary_measurement_eligible_episodes': len(certified), 'deadline_audit': {status: sum(r['deadline_audit']['status'] == status for r in rows) for status in ['CONSISTENT', 'UNRESOLVED', 'VIOLATION']}}
            for name, metric, count in [('qa01', 'latency_ns', COUNTS['responses']), ('qa03', 'via_ns', COUNTS['goals'])]:
                values = [row[metric] for record in certified for row in record[name] if row.get(metric) is not None]
                raw = [row[metric] for record in rows for row in record[name] if row.get(metric) is not None]
                summary[name] = {'planned': count * repetitions, 'observed': len(values), 'observed_partial_mean_ms': statistics.mean(values) / 1e6 if values and valid else None, 'status': 'COMPLETE_CERTIFIED_ENDPOINTS' if valid and len(values) == count * repetitions else 'INCOMPLETE_ENDPOINTS_OR_POPULATION', 'unresolved': sum(row['status'].startswith('UNRESOLVED') for record in rows for row in record[name]), 'technically_uncertified_obligations': sum(len(r[name]) for r in rows if r not in certified), 'raw_observed_count': len(raw), 'raw_partial_mean_ms': statistics.mean(raw) / 1e6 if raw and valid else None}
            bindings = [row for record in certified for row in record['qa02'] if row['status'] != 'UNRESOLVED_OR_LATE_ADOPTION']
            correct = sum(row['correct'] for row in bindings)
            planned = COUNTS['bindings'] * repetitions
            summary['qa02'] = {'planned': planned, 'correct': correct, 'judgments_observed': len(bindings), 'technically_unjudgeable': sum(len(r['qa02']) for r in rows) - len(bindings), 'semantic_incorrect_judgments': sum(not r['correct'] for r in bindings), 'fixed_denominator_accuracy': correct / planned if valid and len(bindings) == planned else None, 'fixed_denominator_observed_success_lower_bound': correct / planned if valid else None, 'status': 'COMPLETE_CERTIFIED_JUDGMENTS' if valid and len(bindings) == planned else 'INCOMPLETE_JUDGMENTS_OR_POPULATION'}
            known = sum(r['qa05']['known_api_cost_usd'] for r in rows)
            unknown = sum(r['qa05']['unknown_call_count'] for r in rows)
            summary['qa05'] = {'known_total_usd': known, 'unknown_calls': unknown, 'unknown_reservation_usd': sum(r['qa05']['unknown_reservation_usd'] for r in rows), 'mean_six_episode_bundle_usd': known / repetitions if all_certified and not unknown else None, 'status': 'COMPLETE_API_USAGE' if complete and not unknown else 'INCOMPLETE_COST_OR_POPULATION', 'comparability': 'COMPLETE_CERTIFIED_EXECUTION' if all_certified else 'INCOMPLETE_TECHNICAL_CERTIFICATION_OR_POPULATION'}
            peaks = [r['qa06']['peak_concurrent_sum_bytes'] for r in rows if r['qa06']['peak_concurrent_sum_bytes'] is not None]
            summary['qa06'] = {'maximum_same_sample_rss_bytes': max(peaks) if peaks else None, 'status': 'COMPLETE_POPULATION' if all_certified and all(r['qa06']['status'] == 'OBSERVED' for r in rows) else 'INCOMPLETE_OBSERVATION_OR_POPULATION_LOWER_BOUND_ONLY', 'invalid_samples': sum(r['qa06']['invalid_sample_count'] for r in rows), 'confirmed_exit_boundary_samples': sum(r['qa06']['confirmed_exit_boundary_count'] for r in rows), 'startup_zero_rss_samples': sum(r['qa06']['startup_zero_rss_count'] for r in rows), 'sampler_failures': sum(len(r['qa06']['sampler_failures']) for r in rows), 'incomplete_samples': sum(r['qa06']['incomplete_registered_process_samples'] for r in rows)}
            summary['qa04'] = {'status': 'SAME42_EXPERIMENT_REFERENCE_ONLY_NOT_EXTRA_MEASUREMENTS', 'required_change_tasks': 6, 'supplemental_regression_required_per_copy': 6}
            output[str(phase) + ':' + configuration] = summary
    return output


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('runs', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--repetitions', type=int, default=3)
    args = parser.parse_args()
    traces = [json.loads(line) for line in args.runs.read_text().splitlines() if line.strip()]
    results = [evaluate(trace) for trace in traces]
    if args.output.exists():
        raise FileExistsError('Preserve existing analysis; choose a fresh output file')
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps({'population': 'COMMON_SUPPLEMENT6', 'results': results, 'summary': summarize(results, args.repetitions)}, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'output': str(args.output), 'traces': len(traces)}))


if __name__ == '__main__':
    main()
