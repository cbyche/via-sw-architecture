"""Bind unchanged QA04 executions to independently reviewed counts and raw audit.

This post-execution tool never builds, runs candidates, or edits retained ledgers.
The immutable report precedes final Astra acceptance; the closure checkpoint
subsequently binds both without a report/review hash cycle.
"""
import argparse
from collections import Counter
import datetime
import hashlib
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(HERE))
import orchestrate as o

R = o.RESULT
CORE = '69934a2190d52ad616542005abbb3353bcb39472dfe36e9129f2098ba246a476'
SUPP = '0a53c454e50b5c6166e7f44bb8845527af947605f28aa4b2188c150658fdf002'
CLASS = '55f87e66f4b511ed74686f6a5e66e0f42f3287febddb28cadbb3ecd5f5841130'
REBASE = '6411be4f23abccb9930ec5259af31d68d40e832a8996ba04b38f1443c96f4833'
SOURCE_REVIEW = '5ba78f0b01892725a6de4553173326800531dc21daa331d3c45adb3867901c82'


def require(ok, message):
    if not ok:
        raise RuntimeError(message)


def ref(path):
    path = Path(path).resolve()
    return {'path': str(path), 'sha256': o.sha(path)}


def immutable(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    data = value if isinstance(value, bytes) else (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode()
    with path.open('xb') as handle:
        handle.write(data)


def audit_inputs(audit):
    for item in audit['input_evidence']:
        require(o.sha(item['path']) == item['sha256'], 'Raw audit input changed: ' + item['path'])


def make_report(args):
    baseline_root = R.parent / 'qa04-method-v3-baseline-snapshot'
    freeze = baseline_root / 'freeze.json'
    supplement = baseline_root / 'supplement-freeze.json'
    source, manifest = o.baseline(freeze, CORE)
    o.require_final_rebase(freeze, CORE, supplement, SUPP)
    require(o.sha(HERE / 'FINAL-REBASE.json') == REBASE, 'Different execution rebase')
    for item in manifest['sources']:
        require(o.sha(item['path']) == item['sha256'], 'Core dependency changed')
    classification_path = R.parent / 'qa04-method-v3-count-classification/CLASSIFICATION-RESULTS.json'
    require(o.sha(classification_path) == CLASS, 'Unreviewed count classification')
    classification = o.read(classification_path)
    scoped_path = R / 'independent-review/source-count-scoped.json'
    require(o.sha(scoped_path) == SOURCE_REVIEW, 'Scoped source review changed')
    scoped = o.read(scoped_path)
    require(scoped['input_sha256']['count_classification_v2'] == CLASS, 'Count review mismatch')
    raw_path = args.raw_audit.resolve()
    raw = o.read(raw_path)
    qualified_path = raw_path.parent.parent / 'final-qualified-manifest.json'
    qualified = o.read(qualified_path)
    require(qualified['status'] == 'PASS' and qualified['functional_runtime_status'] == 'PASS_630_OF_630'
            and qualified['copies'] == 42 and qualified['episodes'] == 630, 'Independent qualification incomplete')
    require(any(Path(item['path']).resolve() == raw_path and item['sha256'] == o.sha(raw_path)
                for item in qualified['input_evidence']), 'Qualification does not bind exact raw audit')
    audit_inputs(qualified)
    require(raw['status'] in ('PASS', 'QUALIFIED_PASS_RSS_INCOMPLETE'), 'Independent raw audit not qualified')
    require(raw['copies_audited'] == 42 and not raw['pending_copies'], 'Raw audit incomplete')
    require(raw['runtime_acceptance_status'] == 'PASS_FOR_AUDITED_TERMINAL_COPIES', 'Raw functional audit failed')
    require(raw['dependencies']['core_freeze_sha256'] == CORE and raw['dependencies']['supplement_freeze_sha256'] == SUPP,
            'Raw audit used a different dependency generation')
    audit_inputs(raw)
    classes = {}
    for item in classification['rows']:
        require(o.sha(item['path']) == item['sha256'], 'Classification row changed')
        key = (item['configuration'], item['task'])
        require(key not in classes, 'Duplicate classification row')
        classes[key] = (item, o.read(item['path']))
    expected = {(cfg, task) for cfg in o.CONFIGS for task in o.TASKS}
    require(set(classes) == expected, 'Count classification population changed')
    totals = {p: dict(episodes=0, responses=0, goals=0, bindings=0) for p in ('COMMON9', 'COMMON_SUPPLEMENT6')}
    rss_counts = {p: Counter() for p in totals}
    diagnostics, rows, seen_runs, versions = [], [], set(), set()
    for cfg in o.CONFIGS:
        for task in o.TASKS:
            pointer = o.read(R / cfg / task / 'latest-attempt.json')
            require(pointer['status'] == 'PASS', 'Nonterminal exercise')
            attempt, verification = Path(pointer['path']), Path(pointer['verification'])
            require(o.read(verification / 'status.json')['status'] == 'PASS', 'Verification not PASS')
            row = o.read(verification / 'summary.json')
            m = o.read(attempt / 'manifest.json')
            require(row['status'] == 'PASS' and m['baseline_freeze_sha256'] == CORE and m['supplement_freeze_sha256'] == SUPP, 'Exercise identity changed')
            for name, digest in m['candidate_source_sha256'].items():
                require(o.sha(attempt / 'candidate' / name) == digest, 'Accepted source changed: ' + name)
            for name, digest in m['qa04_tool_sha256'].items():
                require(o.sha(attempt / 'tools' / name) == digest, 'Execution tool snapshot changed')
            versions.add(json.dumps(m['qa04_tool_sha256'], sort_keys=True))
            original, _ = o.counts(source, attempt / 'candidate')
            require(original == row['modules'] == m['actual_changed_operational_modules'] and len(original) == row['count'] == m['count'], 'Raw ledger changed')
            item, corrected = classes[(cfg, task)]
            require(corrected['actual_candidate_source_inventory'] == m['candidate_source_sha256'], 'Classification is not this accepted copy')
            require(corrected['original_raw_count'] == row['count'] and corrected['original_raw_modules'] == original, 'Count correction has different raw input')
            for linked in corrected['sources']:
                require(o.sha(linked['path']) == linked['sha256'], 'Count input provenance changed')
            for entry in corrected['file_classifications']:
                require(o.sha(attempt / 'candidate' / entry['file']) == entry['actual_sha256'], 'Counted file changed')
                if entry['classification'] == 'FORMATTER_CONTROL_EXACT_BYTES':
                    require(entry['actual_sha256'] == entry['formatted_control_sha256'] and entry['control_byte_identical'], 'Unproven formatting exclusion')
            commands = [json.loads(line) for line in (verification / 'commands.jsonl').read_text().splitlines()]
            require(len(commands) == 3 and all(c['returncode'] == 0 for c in commands), 'Test/build/acceptance command failure')
            require(commands[0]['args'][:3] == ['cargo', 'test', '--offline'] and commands[1]['args'][:3] == ['cargo', 'build', '--offline'], 'Wrong verification commands')
            acceptance_path = Path(row['acceptance'])
            acceptance = o.read(acceptance_path)
            require(acceptance['task_acceptance'] == acceptance['common9_regression'] == acceptance['supplement6_regression'] == 'PASS' and acceptance['provider_calls'] == 0, 'Incomplete functional acceptance')
            for population, field, count in [('COMMON9', 'common9_rows', 9), ('COMMON_SUPPLEMENT6', 'supplement6_rows', 6)]:
                episodes = acceptance[field]
                require(len(episodes) == len({e['episode'] for e in episodes}) == count, 'Regression population duplicated or missing')
                for episode in episodes:
                    check = episode['checks']
                    evaluation = check['evaluation']
                    run_id = episode['run_id']
                    require(run_id not in seen_runs, 'Run ID reused across independent copies')
                    seen_runs.add(run_id)
                    require(check['status'] == 'PASS' and episode['provider_calls'] == 0 and evaluation['evaluation_mode'] == 'strict'
                            and evaluation['version'] == 'COMMON9-EVALUATOR-v3' and evaluation['primary_measurement_eligibility'] == 'ELIGIBLE'
                            and evaluation['deadline_audit']['status'] == 'CONSISTENT', 'Regression lacks strict technical/functional qualification')
                    c = check['counts']
                    t = totals[population]
                    t['episodes'] += 1
                    for target, source_key in [('responses', 'responses_observed'), ('goals', 'goals_observed'), ('bindings', 'bindings_correct')]:
                        t[target] += c[source_key]
                    q = evaluation['qa06']
                    rss_counts[population][q['status']] += 1
                    if q['status'] != 'OBSERVED':
                        base = acceptance_path.parent
                        trace = (base / 'regression/raw/runs' / run_id / 'trace.json') if population == 'COMMON9' else (base / 'supplement6/episodes' / (cfg + '-' + episode['episode']) / 'raw/runs' / run_id / 'trace.json')
                        diagnostics.append({'configuration': cfg, 'task': task, 'population': population, 'episode': episode['episode'], 'run_id': run_id,
                            'trace': ref(trace), 'status': q['status'], 'invalid_sample_count': q['invalid_sample_count'],
                            'sampler_failures': q['sampler_failures'], 'lower_bound_only': q['peak_is_observed_lower_bound_if_incomplete'],
                            'reason': 'Registered process not observed in retained sample; inspect original lifecycle/exit bracket. No relabel or exclusion.',
                            'invalid_samples': q['invalid_samples']})
            row.update(raw_count=row['count'], raw_modules=row['modules'], raw_operational_module_paths=row['operational_module_paths'],
                       count=corrected['corrected_count'], modules=corrected['corrected_modules'], operational_module_paths=corrected['corrected_operational_module_paths'],
                       count_classification=item, candidate_manifest=ref(attempt / 'manifest.json'), verification_summary=ref(verification / 'summary.json'),
                       acceptance_summary=ref(acceptance_path), commands=ref(verification / 'commands.jsonl'), source_patch=ref(verification / 'final.patch'))
            rows.append(row)
    expected_totals = {'COMMON9': dict(episodes=378, responses=1806, goals=966, bindings=630),
                       'COMMON_SUPPLEMENT6': dict(episodes=252, responses=504, goals=336, bindings=252)}
    require(totals == expected_totals, 'Separated full630 denominators differ')
    for pop, t in totals.items():
        audited = raw['populations'][pop]
        require(audited['episodes_audited'] == t['episodes'], 'Independent episode count differs')
        for key, audit_key in [('responses', 'responses_observed'), ('goals', 'goals_observed'), ('bindings', 'bindings_correct')]:
            require(audited['fixed_counts'][audit_key] == t[key], 'Independent obligation count differs')
    require({d['run_id'] for d in diagnostics} == {f['run_id'] for f in raw['findings'] if f['severity'] == 'RSS_INCOMPLETE_DIAGNOSTIC'}, 'Independent RSS diagnostic identities differ')
    require(not [f for f in raw['findings'] if f['severity'] != 'RSS_INCOMPLETE_DIAGNOSTIC'], 'Independent audit has unresolved functional findings')
    counts = {cfg: [r['count'] for r in rows if r['configuration'] == cfg] for cfg in o.CONFIGS}
    require(counts == classification['all_counts_by_configuration'], 'Recount aggregate differs')
    gate = baseline_root / 'frozen/source/benchmark/architecture/common9_poc_20261019/evaluation_v3/evaluate.py'
    preserved = [R / 'launch-attempt-01.json', HERE / 'preparation/supplement-freeze-attempt-01.json',
                 R / 'finite-c04/attempt-01/normal-feature/feature-failed-trace.json',
                 R / 'finite-c04/attempt-01/negative-v1/revocation_payloads/failure.json',
                 R / 'private-common15-integration/supplement-generation-addendum.json']
    report = {'status': 'COMPLETE42_FUNCTIONAL_AND_REVIEWED_COUNT_EVIDENCE_FINAL_ACCEPTANCE_SEPARATE',
        'created_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'baseline_freeze': ref(freeze), 'supplement_dependency_freeze': ref(supplement),
        'final_tool_rebase': ref(HERE / 'FINAL-REBASE.json'), 'accepted_independent_changes': 42, 'independent_tasks_each': 6,
        'combined_regression_episodes': 630, 'regression_totals_by_population': totals, 'actual_provider_api_calls': 0,
        'all_counts_by_configuration': counts, 'mean_changed_operational_modules': {cfg: sum(v) / 6 for cfg, v in counts.items()},
        'raw_counts_preserved': classification['raw_counts_retained'], 'raw_matrix_summary': ref(R / 'summary.json'),
        'count_classification': ref(classification_path), 'qa06_diagnostics': {'status_counts_by_population': {p: dict(c) for p, c in rss_counts.items()},
            'non_observed': diagnostics, 'functional_gate_source': ref(gate), 'functional_gate_lines': [76, 88],
            'disposition': 'Frozen preregistered functional predicate excludes RSS completeness. Retain INCOMPLETE and observed-lower-bound labels; no relabel, rerun, or population exclusion. QA04 API-free functional regressions are not formal QA06 memory measurements.'},
        'independent_reviews': {'source_count_scoped': {'artifact': ref(scoped_path), 'scope': scoped['scope'], 'status': scoped['status'], 'reviewer_model': scoped['reviewer_model'],
            'immutable_prose': ref(R / 'independent-review/source-count-scoped-prose.md')},
            'raw630': {'artifact': ref(qualified_path), 'detailed_audit': ref(raw_path), 'scope': qualified['scope'], 'status': qualified['status'], 'reviewer': qualified['reviewer']},
            'final_qa04_acceptance': 'Separate subsequent sidecar binds this immutable report; final-checkpoint.json records closure.'},
        'execution_tool_snapshot_versions': len(versions), 'report_generator': ref(Path(__file__)), 'rows': rows,
        'preserved_preparation_failure_and_intermediate_records': [ref(p) for p in preserved if p.exists()],
        'final_matrix_verification_failures': [ref(p) for p in R.glob('*/C*/attempts/attempt-*/verifications/verify-*/status.json') if o.read(p)['status'] == 'VERIFY_FAILED'],
        'limitations': ['Six common adapter/source/publication change purposes, not a general intrinsic architecture modifiability ranking.',
            'Observed equal count vectors are not forced ties; no previous generation count is imported.',
            'Deterministic semantic/vision doubles; no model accuracy, formal response timing, production memory, winner or paid authorization claim.',
            'C01 synthetic17/11 accounting tests isolated transport bookkeeping, not actual provider spend.',
            'Historical formatter version was not recorded; formatting exclusions require actual byte equality with the recorded formatter control.',
            'Original top-level tools, FINAL-REBASE, raw ledgers and all intermediate/failure records remain unchanged.']}
    lines = ['# QA04 final execution evidence', '', '42 independent changes; 630 strict functional regressions PASS; provider API calls: 0.', '',
             '| Configuration | C01 | C02 | C03 | C04 | C05 | C06 | Mean |', '| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
    for cfg, values in counts.items():
        lines.append('| ' + cfg + ' | ' + ' | '.join(map(str, values)) + ' | ' + f'{sum(values)}/6 = {sum(values)/6:.3f}' + ' |')
    lines += ['', 'Counts use the independently reviewed exact formatter-only control. The original raw7/7/8/6/6/6 ledgers remain preserved. C04 counts evidence, runtime and sources. Equality is observed for these six shared change purposes and establishes no general architecture ranking.', '',
        'Common9 separately retains378 episodes,1,806 responses,966 goals and630 exact bindings. Supplement6 retains252 episodes,504 responses,336 goals and252 exact bindings. These630 episodes support the same42 changes.', '',
        '## Memory diagnostics', '', 'The frozen functional gate in evaluation_v3/evaluate.py76–88 excludes RSS completeness. These API-free QA04 regressions are not formal QA06 memory measurements. All original INCOMPLETE/lower-bound observations remain included.', '']
    for pop, values in rss_counts.items():
        lines.append('- ' + pop + ': ' + json.dumps(dict(values)))
    for d in diagnostics:
        lines.append('- ' + '/'.join([d['configuration'], d['task'], d['episode']]) + ' `' + d['run_id'] + '`: ' + d['status'] + '; invalid samples=' + str(d['invalid_sample_count']) + '; ' + d['reason'])
    lines += ['', '## Independent reviews', '', '- Source/count and initial90 raw scope: [scoped review](independent-review/source-count-scoped.json).',
        '- Full630 independent retained-raw audit: [' + qualified['reviewer']['model'] + '](' + str(qualified_path) + ').',
        '- Final QA04 acceptance is a subsequent immutable sidecar. Closure binds this report and that sidecar in final-checkpoint.json; the report is not rewritten.', '',
        '## Provenance and limits', '', '[Machine-readable report](QA04-RESULTS.json) links every accepted copy, commands, source patch, corrected ledger and retained failure/intermediate record. No old result/count is inherited. Deterministic model doubles establish code regression behavior only. Historical formatter version was not captured; exclusions require actual control-byte equality.', '']
    immutable(R / 'QA04-RESULTS.json', report)
    immutable(R / 'QA04-RESULTS.md', ('\n'.join(lines) + '\n').encode())
    immutable(R / 'postanalysis-tools/finish_report_v2.py', Path(__file__).read_bytes())
    checkpoint = {'status': 'STABLE_EXECUTION_REPORT_PENDING_FINAL_QA04_ACCEPTANCE', 'accepted_independent_changes': 42,
        'combined_regression_episodes': 630, 'actual_provider_api_calls': 0, 'report_json': ref(R / 'QA04-RESULTS.json'),
        'report_md': ref(R / 'QA04-RESULTS.md'), 'independent_reviews': {'source_count_scoped': ref(scoped_path), 'raw630': ref(qualified_path), 'raw630_detail': ref(raw_path)},
        'count_classification': ref(classification_path), 'report_generator': ref(Path(__file__))}
    immutable(R / 'execution-report-checkpoint.json', checkpoint)
    print(json.dumps(checkpoint))


def close(args):
    checkpoint_path = R / 'execution-report-checkpoint.json'
    checkpoint = o.read(checkpoint_path)
    for key in ('report_json', 'report_md', 'count_classification', 'report_generator'):
        item = checkpoint[key]
        require(o.sha(item['path']) == item['sha256'], 'Report evidence changed before acceptance')
    for item in checkpoint['independent_reviews'].values():
        require(o.sha(item['path']) == item['sha256'], 'Independent audit changed')
    acceptance = o.read(args.final_acceptance)
    require(acceptance['status'].startswith('PASS'), 'Final QA04 review not PASS')
    inputs = acceptance['input_sha256']
    require(inputs['report_json'] == checkpoint['report_json']['sha256'] and inputs['report_md'] == checkpoint['report_md']['sha256'], 'Final review does not bind immutable report')
    require(inputs['raw630_audit'] == checkpoint['independent_reviews']['raw630']['sha256'] and inputs['count_classification_v2'] == CLASS, 'Final review evidence mismatch')
    checkpoint.update(status='STABLE_QA04_WRITES_CLOSED', writes_closed=True, closed_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                      execution_report_checkpoint=ref(checkpoint_path))
    checkpoint['independent_reviews']['final_qa04_acceptance'] = ref(args.final_acceptance)
    immutable(R / 'final-checkpoint.json', checkpoint)
    print(json.dumps(ref(R / 'final-checkpoint.json')))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--raw-audit', type=Path)
    parser.add_argument('--final-acceptance', type=Path)
    args = parser.parse_args()
    if args.final_acceptance:
        close(args)
    else:
        require(args.raw_audit is not None, 'Provide completed independent raw630 audit')
        make_report(args)
