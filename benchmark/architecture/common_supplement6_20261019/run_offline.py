"""Fresh subprocess supplement6 regression. Doubles only; no learned/API execution."""
import argparse
import concurrent.futures
import hashlib
import json
import subprocess
import sys
from pathlib import Path
import evaluate

HERE = Path(__file__).resolve().parent
CONFIGS = ['AAAA', 'BAAA', 'CAAA', 'ABAA', 'AABA', 'AAAB', 'AAAC']


def dump(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--crate', type=Path, default=HERE.parents[2] / 'prototypes/candidates/common9_poc_20261019')
    parser.add_argument('--configuration', choices=CONFIGS)
    parser.add_argument('--input-transform-module', type=Path, help='Offline QA04 only; module exports migrate_c03')
    parser.add_argument('--workers', type=int, default=2)
    parser.add_argument('--time-scale', type=float, default=.05)
    args = parser.parse_args()
    args.root.mkdir(parents=True, exist_ok=False)
    fixtures = json.loads((HERE / 'fixtures.json').read_text())
    binary = args.crate / 'target/debug/common9_poc'
    binary_sha = hashlib.sha256(binary.read_bytes()).hexdigest()
    jobs = [{'configuration': cfg, 'episode': e['id'], 'rep': 1} for e in fixtures for cfg in ([args.configuration] if args.configuration else CONFIGS)]
    dump(args.root / 'planned-jobs.json', jobs)
    def run(job):
        child = args.root / 'episodes' / (job['configuration'] + '-' + job['episode'])
        command = [sys.executable, str(HERE / 'harness.py'), '--root', str(child), '--configuration', job['configuration'], '--episode', job['episode'], '--rep', '1', '--offline', '--time-scale', str(args.time_scale), '--crate', str(args.crate)]
        if args.input_transform_module:
            command += ['--input-transform-module', str(args.input_transform_module.resolve())]
        child.mkdir(parents=True)
        dump(child / 'command.json', {'command': command, 'binary_sha256': binary_sha})
        result = subprocess.run(command, capture_output=True, text=True)
        (child / 'stdout.txt').write_text(result.stdout)
        (child / 'stderr.txt').write_text(result.stderr)
        paths = list((child / 'raw/runs').glob('*/trace.json'))
        trace = json.loads(paths[0].read_text()) if len(paths) == 1 else None
        row = {**job, 'returncode': result.returncode, 'trace': str(paths[0]) if trace else None}
        if trace:
            assert trace['binary_sha256'] == binary_sha
            regression = evaluate.regression(trace)
            dump(child / 'regression.json', regression)
            row.update(regression_status=regression['status'], counts=regression['counts'], technical=regression['evaluation']['deadline_audit']['status'])
        return row, trace
    rows = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as executor:
        for future in concurrent.futures.as_completed([executor.submit(run, job) for job in jobs]):
            row, trace = future.result()
            rows.append(row)
            if trace:
                with (args.root / 'runs.jsonl').open('a') as handle:
                    handle.write(json.dumps(trace, ensure_ascii=False) + '\n')
            dump(args.root / 'progress.json', {'completed': len(rows), 'planned': len(jobs), 'rows': rows, 'actual_API_calls': 0})
            print(json.dumps({'completed': len(rows), 'planned': len(jobs), **row}), flush=True)
    passed = all(row.get('regression_status') == 'PASS' and row['returncode'] == 0 for row in rows)
    dump(args.root / 'summary.json', {'status': 'PASS' if passed else 'FAIL', 'completed': len(rows), 'planned': len(jobs), 'rows': rows, 'binary_sha256': binary_sha, 'actual_API_calls': 0, 'scope': 'Scaled deterministic integration only; no model accuracy, latency, token or parser measurement claim'})
    if not passed:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
