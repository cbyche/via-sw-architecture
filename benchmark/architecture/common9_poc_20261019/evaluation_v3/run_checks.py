"""Persist exact finite-check identities and current source hashes."""
import argparse
import hashlib
import io
import json
import unittest
from pathlib import Path
from . import checks


def flatten(suite):
    for item in suite:
        if isinstance(item, unittest.TestSuite):
            yield from flatten(item)
        else:
            yield item.id()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    suite = unittest.defaultTestLoader.loadTestsFromModule(checks)
    identities = list(flatten(suite))
    approved = [name for name in identities if '.Endpoints.test_' in name and name.rsplit('test_', 1)[1][:2].isdigit() and 1 <= int(name.rsplit('test_', 1)[1][:2]) <= 14]
    log = io.StringIO()
    result = unittest.TextTestRunner(stream=log, verbosity=2).run(suite)
    here = Path(__file__).resolve().parent
    args.output.mkdir(parents=True, exist_ok=False)
    (args.output / 'checks.log').write_text(log.getvalue())
    manifest = {'status': 'PASS' if result.wasSuccessful() else 'FAIL', 'test_count': result.testsRun,
        'approved_addendum_cases': {'count': len(approved), 'test_ids': approved},
        'additional_cases': {'count': len(identities) - len(approved), 'test_ids': [i for i in identities if i not in approved]},
        'case_groups': {group: [name for name in identities if name.split('.')[-2] == group]
            for group in sorted({name.split('.')[-2] for name in identities})},
        'failures': [(t.id(), detail) for t, detail in result.failures],
        'errors': [(t.id(), detail) for t, detail in result.errors], 'api_calls': 0,
        'sources': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(here.iterdir()) if p.suffix in ('.py', '.md')},
        'preserved_dependencies': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in (here.parent / 'evaluate.py', here.parent / 'causal_timing.py', here.parent / 'analyze.py')},
        'scope': 'finite Python evaluator checks only; no candidate execution, model accuracy or performance measurement'}
    (args.output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(json.dumps({'status': manifest['status'], 'tests': result.testsRun, 'approved': len(approved), 'additional': len(identities)-len(approved), 'output': str(args.output)}))
    raise SystemExit(0 if result.wasSuccessful() else 1)


if __name__ == '__main__':
    main()
