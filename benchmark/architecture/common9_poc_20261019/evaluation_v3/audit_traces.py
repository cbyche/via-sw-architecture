"""Read retained raw traces once and persist explicit regression/audit evidence."""
import argparse
import hashlib
import json
from pathlib import Path
from .evaluate import evaluate, regression, summarize


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('input', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--repetitions', type=int, default=5)
    parser.add_argument('--mode', choices=('strict', 'legacy_diagnostic'), default='strict')
    args = parser.parse_args()
    raw = args.input.read_bytes()
    traces = [json.loads(line) for line in raw.decode().splitlines() if line.strip()] if args.input.suffix == '.jsonl' else [json.loads(raw)]
    evaluations = [evaluate(t, mode=args.mode) for t in traces]
    rows = []
    for trace, result in zip(traces, evaluations):
        rows.append({'run_id': trace.get('run_id'), 'configuration': trace.get('configuration'), 'episode': trace.get('episode'),
            'rep': trace.get('rep'), 'returncode': trace.get('returncode'), 'deadline_audit': result['deadline_audit']['status'],
            'measurement_eligibility': result['primary_measurement_eligibility'],
            'responses_observed': sum(r['status'] == 'OBSERVED' for r in result['qa01']), 'responses_planned': len(result['qa01']),
            'goals_observed': sum(r['status'] == 'OBSERVED' for r in result['qa03']), 'goals_planned': len(result['qa03']),
            'bindings_correct': sum(r['correct'] for r in result['qa02']), 'bindings_planned': len(result['qa02']),
            'known_cost_usd': result['qa05']['known_api_cost_usd'], 'unknown_calls': result['qa05']['unknown_call_count'],
            'rss_status': result['qa06']['status'], 'host_failures': result['host_failures'],
            'functional_failures': result['functional_failures']})
    args.output.mkdir(parents=True, exist_ok=False)
    (args.output / 'audit.json').write_text(json.dumps({'input': str(args.input.resolve()), 'input_sha256': hashlib.sha256(raw).hexdigest(),
        'mode': args.mode, 'api_calls_by_auditor': 0, 'rows': rows, 'summary': summarize(evaluations, args.repetitions),
        'results': evaluations}, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'traces': len(traces), 'audit_statuses': {s: sum(r['deadline_audit'] == s for r in rows) for s in ('CONSISTENT', 'UNRESOLVED', 'VIOLATION')},
        'output': str(args.output)}))


if __name__ == '__main__':
    main()
