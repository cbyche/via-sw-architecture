"""Immutable QA04 formatter-control addendum; never edits actual patched copies.

Formatting-only exclusion requires byte identity with the separately formatted
baseline. Existing narrowly audited test/comment/whitespace exclusions remain
separately labelled. Operational punctuation is never globally normalized.
"""
import argparse
import datetime
import difflib
import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
RESULT = REPO / 'results/architecture-evaluation/current/common9-poc-20261019'
BASELINE = RESULT / 'qa04-method-v3-baseline-snapshot/freeze.json'
BASELINE_SHA = '69934a2190d52ad616542005abbb3353bcb39472dfe36e9129f2098ba246a476'
RAW = RESULT / 'qa04-method-v3'
CONFIGS = ['AAAA', 'BAAA', 'CAAA', 'ABAA', 'AABA', 'AAAB', 'AAAC']
TASKS = ['C01', 'C02', 'C03', 'C04', 'C05', 'C06']


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def dump(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('x') as handle:
        handle.write(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def files(crate):
    return {str(p.relative_to(crate)): sha(p) for p in sorted(crate.rglob('*')) if p.is_file() and not any(x in p.relative_to(crate).parts for x in ['target', '__pycache__'])}


def command(args):
    start = time.time()
    result = subprocess.run(args, cwd=REPO, capture_output=True, text=True)
    row = {'args': args, 'cwd': str(REPO), 'env_override': None, 'started_unix': start, 'finished_unix': time.time(), 'returncode': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr}
    if result.returncode:
        raise RuntimeError(json.dumps(row))
    return row


def load_inventory(path):
    spec = importlib.util.spec_from_file_location('immutable_qa04_module_inventory', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def classify(original, control, actual, operational_tokens):
    if actual == original:
        return False, 'UNCHANGED_ORIGINAL_BYTES'
    if actual == control:
        return False, 'FORMATTER_CONTROL_EXACT_BYTES'
    if operational_tokens(actual.decode()) == operational_tokens(control.decode()):
        return False, 'EXPLICIT_EXISTING_TEST_COMMENT_WHITESPACE_ONLY_VS_FORMATTER_CONTROL'
    return True, 'ACTUAL_OPERATIONAL_TOKENS_DIFFER_FROM_FORMATTER_CONTROL'


def finite(module):
    def run(original, control, actual):
        return classify(original.encode(), control.encode(), actual.encode(), module.operational_tokens)
    baseline = 'fn f(){g(a,b);}\n'
    control = 'fn f() {\n    g(a, b);\n}\n'
    cases = [
        ('unchanged_original', baseline, control, baseline, False, 'UNCHANGED_ORIGINAL_BYTES'),
        ('formatter_control_bytes', baseline, control, control, False, 'FORMATTER_CONTROL_EXACT_BYTES'),
        ('comments_separate_rule', baseline, control, control + '// actual comment\n', False, 'EXPLICIT_EXISTING_TEST_COMMENT_WHITESPACE_ONLY_VS_FORMATTER_CONTROL'),
        ('test_module_separate_rule', baseline, control, control + '#[cfg(test)] mod tests { fn proof(){ assert!(true); } }', False, 'EXPLICIT_EXISTING_TEST_COMMENT_WHITESPACE_ONLY_VS_FORMATTER_CONTROL'),
        ('operator_not_equal_vs_equal', 'fn f(){a!=b;}', 'fn f() { a != b; }', 'fn f() { a == b; }', True, 'ACTUAL_OPERATIONAL_TOKENS_DIFFER_FROM_FORMATTER_CONTROL'),
        ('literal_bool', 'fn f(){true;}', 'fn f() { true; }', 'fn f() { false; }', True, 'ACTUAL_OPERATIONAL_TOKENS_DIFFER_FROM_FORMATTER_CONTROL'),
        ('string_literal_internal_whitespace', 'fn f(){"a b";}', 'fn f() { "a b"; }', 'fn f() { "ab"; }', True, 'ACTUAL_OPERATIONAL_TOKENS_DIFFER_FROM_FORMATTER_CONTROL'),
        ('argument_removed', baseline, control, 'fn f() { g(a); }', True, 'ACTUAL_OPERATIONAL_TOKENS_DIFFER_FROM_FORMATTER_CONTROL'),
        ('tuple_comma_semantic_difference', 'fn f(){let x=(a);}', 'fn f() { let x = (a); }', 'fn f() { let x = (a,); }', True, 'ACTUAL_OPERATIONAL_TOKENS_DIFFER_FROM_FORMATTER_CONTROL'),
        ('branch_scope_braces', 'fn f(){if a {b();c();}}', 'fn f() { if a { b(); c(); } }', 'fn f() { if a { b(); } c(); }', True, 'ACTUAL_OPERATIONAL_TOKENS_DIFFER_FROM_FORMATTER_CONTROL'),
        ('parent_module_registration', 'mod old;', 'mod old;\n', 'mod new;\n', True, 'ACTUAL_OPERATIONAL_TOKENS_DIFFER_FROM_FORMATTER_CONTROL'),
        ('comma_not_globally_stripped', 'fn f(){g((a));}', 'fn f() { g((a)); }', 'fn f() { g((a,)); }', True, 'ACTUAL_OPERATIONAL_TOKENS_DIFFER_FROM_FORMATTER_CONTROL'),
    ]
    rows = []
    for name, original, formatted, actual, expected, reason in cases:
        counted, classification = run(original, formatted, actual)
        assert counted == expected and classification == reason, name
        rows.append({'case': name, 'counted': counted, 'classification': classification, 'status': 'PASS'})
    return rows


def config_discovery(crate):
    candidates = set()
    for folder in [crate, *(crate / 'src').rglob('*')]:
        if not folder.is_dir():
            continue
        for ancestor in [folder, *folder.parents]:
            for name in ['rustfmt.toml', '.rustfmt.toml', 'rust-toolchain', 'rust-toolchain.toml']:
                candidates.add(ancestor / name)
    for folder in [Path.home(), Path(os.environ.get('XDG_CONFIG_HOME', str(Path.home() / '.config'))) / 'rustfmt']:
        for name in ['rustfmt.toml', '.rustfmt.toml']:
            candidates.add(folder / name)
    return [{'path': str(p), 'sha256': sha(p)} for p in sorted(candidates) if p.is_file()]


def create(root):
    root.mkdir(parents=True, exist_ok=False)
    assert sha(BASELINE) == BASELINE_SHA
    baseline_manifest = json.loads(BASELINE.read_text())
    original = Path(next(s['frozen_path'] for s in baseline_manifest['sources'] if s['relativePath'] == 'prototypes/candidates/common9_poc_20261019/Cargo.toml')).parent
    baseline_inventory = files(original)
    for row in baseline_manifest['sources']:
        if row['relativePath'].startswith('prototypes/candidates/common9_poc_20261019/'):
            assert sha(row['frozen_path']) == row['sha256']
    control = root / 'formatter-control/candidate'
    shutil.copytree(original, control, ignore=shutil.ignore_patterns('target', '__pycache__'))
    assert files(control) == baseline_inventory
    version_records = [command(['cargo', '--version']), command(['cargo', 'fmt', '--version']), command(['rustfmt', '--version']), command(['rustup', 'show', 'active-toolchain']), command(['rustup', 'which', 'rustfmt']), command(['rustup', 'which', 'cargo'])]
    executable_records = []
    for item in version_records[-2:]:
        path = Path(item['stdout'].strip())
        executable_records.append({'path': str(path), 'sha256': sha(path), 'bytes': path.stat().st_size, 'mtime_ns': path.stat().st_mtime_ns})
    fmt = command(['cargo', 'fmt', '--manifest-path', str(control / 'Cargo.toml')])
    control_inventory = files(control)
    configs = config_discovery(control)
    actual_config_paths = []
    for cfg in CONFIGS:
        for task in TASKS:
            latest = json.loads((RAW / cfg / task / 'latest-attempt.json').read_text())
            actual_config_paths.extend(config_discovery(Path(latest['path']) / 'candidate'))
    # No hidden per-copy configuration may be ignored when transferring the
    # baseline control. Current campaign has none; fail for explicit review if so.
    assert not configs and not actual_config_paths, (configs, actual_config_paths)
    dump(root / 'formatter-control/command.json', fmt)
    dump(root / 'formatter-control/toolchain.json', {'records': version_records, 'executables': executable_records, 'config_discovery': {'control': configs, 'actual42': actual_config_paths, 'rule': 'Applicable source-directory/ancestor and user config paths checked; none present'}, 'original_commands_recorded_version': False, 'version_scope': 'Current installed formatter/toolchain captured now. Historical42commands record args/cwd but not versions; exact byte-equality against this control is the independent exclusion evidence, not an invented prior version log.'})
    control_record = {'baseline_freeze_path': str(BASELINE), 'baseline_freeze_sha256': BASELINE_SHA, 'original_source': str(original), 'original_inventory': baseline_inventory, 'formatter_control': str(control), 'formatted_inventory': control_inventory, 'format_only_changed_files': sorted(p for p in baseline_inventory if baseline_inventory[p] != control_inventory[p]), 'command_path': str(root / 'formatter-control/command.json'), 'toolchain_path': str(root / 'formatter-control/toolchain.json'), 'candidate_build_or_execution': False, 'provider_API_calls': 0}
    dump(root / 'formatter-control/manifest.json', control_record)
    inventory_source = Path(json.loads((RAW / 'AAAA/C01/latest-attempt.json').read_text())['path']) / 'tools/module_inventory.py'
    tools = root / 'tools'
    tools.mkdir()
    shutil.copy2(inventory_source, tools / 'module_inventory.py')
    shutil.copy2(Path(__file__), tools / Path(__file__).name)
    module = load_inventory(tools / 'module_inventory.py')
    finite_rows = finite(module)
    dump(root / 'finite-acceptance.json', {'status': 'PASS', 'cases': finite_rows, 'count': len(finite_rows), 'module_inventory_sha256': sha(tools / 'module_inventory.py'), 'actual_API_calls': 0})
    original_mapping = module.inventory(original)
    control_mapping = module.inventory(control)
    assert original_mapping == control_mapping
    rows = []
    for cfg in CONFIGS:
        for task in TASKS:
            latest_path = RAW / cfg / task / 'latest-attempt.json'
            attempt = Path(json.loads(latest_path.read_text())['path'])
            actual = attempt / 'candidate'
            manifest_path, review_path, patch_path = [attempt / name for name in ['manifest.json', 'module-review.json', 'actual.patch']]
            raw = json.loads(manifest_path.read_text())
            assert raw['baseline_freeze_sha256'] == BASELINE_SHA
            assert raw['candidate_source_sha256'] == files(actual)
            assert raw['qa04_tool_sha256']['module_inventory.py'] == sha(tools / 'module_inventory.py')
            mapping = module.inventory(actual)
            by_file = {entry['file']: entry['operational_module_path'] for entry in original_mapping + mapping}
            file_rows = []
            for relative in sorted(set(baseline_inventory) | set(files(actual))):
                if not relative.endswith('.rs'):
                    continue
                before = (original / relative).read_bytes() if (original / relative).exists() else b''
                formatted = (control / relative).read_bytes() if (control / relative).exists() else b''
                after = (actual / relative).read_bytes() if (actual / relative).exists() else b''
                counted, reason = classify(before, formatted, after, module.operational_tokens)
                if before != after:
                    file_rows.append({'file': relative, 'operational_module_path': by_file[relative], 'original_sha256': hashlib.sha256(before).hexdigest(), 'formatted_control_sha256': hashlib.sha256(formatted).hexdigest(), 'actual_sha256': hashlib.sha256(after).hexdigest(), 'counted': counted, 'classification': reason, 'control_byte_identical': after == formatted, 'raw_original_counted': relative in raw['actual_changed_operational_modules']})
            corrected = [entry['file'] for entry in file_rows if entry['counted']]
            assert all(e['control_byte_identical'] for e in file_rows if e['classification'] == 'FORMATTER_CONTROL_EXACT_BYTES')
            command_rows = [json.loads(line) for line in (attempt / 'commands.jsonl').read_text().splitlines()]
            matching = [c for c in command_rows if c['args'] == ['cargo', 'fmt', '--manifest-path', str(actual / 'Cargo.toml')]]
            assert len(matching) == 1 and matching[0]['returncode'] == 0
            # Save the full control-to-actual Rust diff without modifying either.
            difference = ''.join(''.join(difflib.unified_diff((control / p).read_text().splitlines(True) if (control / p).exists() else [], (actual / p).read_text().splitlines(True) if (actual / p).exists() else [], fromfile='formatter-control/' + p, tofile='actual/' + p)) for p in sorted({e['file'] for e in file_rows}))
            folder = root / 'classifications' / cfg / task
            folder.mkdir(parents=True)
            (folder / 'control-to-actual.patch').write_text(difference)
            row = {'configuration': cfg, 'task': task, 'version': 'QA04_FORMATTER_CONTROL_CLASSIFICATION_v2', 'original_raw_count': raw['count'], 'original_raw_modules': raw['actual_changed_operational_modules'], 'corrected_count': len(corrected), 'corrected_modules': corrected, 'corrected_operational_module_paths': sorted({by_file[p] for p in corrected}), 'actual_module_inventory': mapping, 'selected_live_path': json.loads(review_path.read_text())['selected_live_path'], 'file_classifications': file_rows, 'sources': [{'path': str(p), 'sha256': sha(p)} for p in [manifest_path, review_path, patch_path, attempt / 'commands.jsonl']], 'actual_candidate_source_inventory': raw['candidate_source_sha256'], 'formatter_control_manifest_sha256': sha(root / 'formatter-control/manifest.json'), 'original_formatter_command': matching[0], 'control_to_actual_patch_sha256': sha(folder / 'control-to-actual.patch'), 'verification_results_inherited': False, 'meaning': 'Count reclassification only; actual same42candidate execution/630 acceptance separately required and retained.'}
            dump(folder / 'classification.json', row)
            rows.append(row)
    assert files(control) == control_inventory
    assert files(original) == baseline_inventory
    counts = {cfg: [next(r['corrected_count'] for r in rows if r['configuration'] == cfg and r['task'] == task) for task in TASKS] for cfg in CONFIGS}
    dump(root / 'CLASSIFICATION-RESULTS.json', {'status': 'COMPLETE42_CLASSIFICATION_PENDING_INDEPENDENT_REVIEW_NOT_ACCEPTANCE', 'version': 'QA04_FORMATTER_CONTROL_CLASSIFICATION_v2', 'reason': 'Whole-crate cargo fmt introduced punctuation/layout collateral in original module token ledger; exact formatter-only control isolates it without changing candidates or stripping meaningful punctuation.', 'baseline_freeze_sha256': BASELINE_SHA, 'formatter_control_manifest': str(root / 'formatter-control/manifest.json'), 'formatter_control_manifest_sha256': sha(root / 'formatter-control/manifest.json'), 'finite_acceptance': {'path': str(root / 'finite-acceptance.json'), 'sha256': sha(root / 'finite-acceptance.json')}, 'all_counts_by_configuration': counts, 'means': {cfg: sum(values) / 6 for cfg, values in counts.items()}, 'raw_counts_retained': {cfg: [next(r['original_raw_count'] for r in rows if r['configuration'] == cfg and r['task'] == task) for task in TASKS] for cfg in CONFIGS}, 'rows': [{'configuration': row['configuration'], 'task': row['task'], 'path': str(root / 'classifications' / row['configuration'] / row['task'] / 'classification.json'), 'sha256': sha(root / 'classifications' / row['configuration'] / row['task'] / 'classification.json')} for row in rows], 'actual_candidates_binaries_tools_ledgers_modified': False, 'candidate_model_or_regression_reruns': 0, 'provider_API_calls': 0, 'tool_sha256': sha(Path(__file__)), 'created_utc': datetime.datetime.now(datetime.timezone.utc).isoformat()})
    print(json.dumps({'root': str(root), 'counts': counts, 'classification_only': True, 'API_calls': 0}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path, required=True)
    args = parser.parse_args()
    create(args.root.resolve())
