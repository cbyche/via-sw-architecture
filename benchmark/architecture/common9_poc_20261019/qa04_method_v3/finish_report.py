"""Aggregate only newly accepted QA04 exercises; never executes model calls."""
import argparse
import datetime
import json
from pathlib import Path

import orchestrate as o


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--freeze', type=Path, required=True)
    parser.add_argument('--digest', required=True)
    parser.add_argument('--supplement-freeze', type=Path, required=True)
    parser.add_argument('--supplement-digest', required=True)
    parser.add_argument('--execution-review', type=Path)
    parser.add_argument('--grounding-review', type=Path)
    args = parser.parse_args()
    source, frozen = o.baseline(args.freeze, args.digest)
    from supplement_regression import frozen_benchmark
    frozen_benchmark(args.supplement_freeze, args.supplement_digest, args.digest)
    for entry in frozen['sources']:
        if o.sha(Path(entry['path'])) != entry['sha256']:
            raise RuntimeError('Current dependency differs from exact freeze: ' + entry['relativePath'])
    rows = []
    failed = []
    totals = {population: {'episodes': 0, 'responses': 0, 'goals': 0, 'bindings': 0} for population in ['COMMON9', 'COMMON_SUPPLEMENT6']}
    versions = set()
    for cfg in o.CONFIGS:
        for task in o.TASKS:
            taskroot = o.RESULT / cfg / task
            pointer = o.read(taskroot / 'latest-attempt.json')
            if pointer['status'] != 'PASS':
                raise RuntimeError('Fresh exercise not accepted: ' + cfg + '/' + task)
            attempt = Path(pointer['path'])
            verification = Path(pointer['verification'])
            if o.read(verification / 'status.json')['status'] != 'PASS':
                raise RuntimeError('Latest verification not PASS')
            row = o.read(verification / 'summary.json')
            manifest = o.read(attempt / 'manifest.json')
            if row['baseline_freeze_sha256'] != args.digest or manifest['baseline_freeze_sha256'] != args.digest:
                raise RuntimeError('Prior baseline evidence cannot be imported')
            if row.get('supplement_freeze_sha256') != args.supplement_digest or manifest.get('supplement_freeze_sha256') != args.supplement_digest:
                raise RuntimeError('Different supplement dependency freeze cannot be imported')
            for name, digest in manifest['candidate_source_sha256'].items():
                if o.sha(attempt / 'candidate' / name) != digest:
                    raise RuntimeError('Candidate changed after acceptance: ' + name)
            for name, digest in manifest['qa04_tool_sha256'].items():
                if o.sha(attempt / 'tools' / name) != digest:
                    raise RuntimeError('Attempt tool changed after acceptance: ' + name)
            versions.add(json.dumps(manifest['qa04_tool_sha256'], sort_keys=True))
            actual, _ = o.counts(source, attempt / 'candidate')
            if actual != row['modules'] or len(actual) != row['count']:
                raise RuntimeError('Actual operational diff count differs from accepted ledger')
            acceptance_path = Path(row['acceptance'])
            acceptance = o.read(acceptance_path)
            if acceptance['provider_calls'] != 0 or acceptance['task_acceptance'] != 'PASS' or acceptance['common9_regression'] != 'PASS':
                raise RuntimeError('Incomplete acceptance or unauthorized provider use')
            if (acceptance.get('baseline_freeze_sha256') != args.digest or
                acceptance.get('evaluator_version') != 'COMMON9-EVALUATOR-v3' or acceptance.get('evaluation_mode') != 'strict'):
                raise RuntimeError('Acceptance did not use this exact freeze and strict v3 evaluator')
            if acceptance.get('supplement6_regression') != 'PASS' or acceptance.get('supplement_freeze_sha256') != args.supplement_digest:
                raise RuntimeError('Supplement6 regression missing or from a different exact source')
            for population, field, count in [('COMMON9', 'common9_rows', 9), ('COMMON_SUPPLEMENT6', 'supplement6_rows', 6)]:
                population_rows = acceptance[field]
                if len(population_rows) != count or len({r['episode'] for r in population_rows}) != count:
                    raise RuntimeError('Regression population changed or duplicated: ' + population)
                for episode in population_rows:
                    checks = episode['checks']
                    if checks['status'] != 'PASS' or episode['provider_calls'] != 0:
                        raise RuntimeError('A retained regression row is not PASS or used a provider')
                    evaluation = checks.get('evaluation', {})
                    if (evaluation.get('version') != 'COMMON9-EVALUATOR-v3' or evaluation.get('evaluation_mode') != 'strict'
                        or evaluation.get('primary_measurement_eligibility') != 'ELIGIBLE'):
                        raise RuntimeError('A regression lacks strict v3 technical certification: ' + population)
                    c = checks['counts']
                    totals[population]['episodes'] += 1
                    totals[population]['responses'] += c['responses_observed']
                    totals[population]['goals'] += c['goals_observed']
                    totals[population]['bindings'] += c['bindings_correct']
            row.update(verification_summary=str(verification / 'summary.json'),
                       candidate_manifest=str(attempt / 'manifest.json'),
                       source_patch=str(verification / 'final.patch'),
                       feature_trace=str(acceptance_path.parent / 'feature-trace.json'),
                       common9_regression_directory=str(acceptance_path.parent / 'regression'),
                       supplement6_regression_directory=str(acceptance_path.parent / 'supplement6'),
                       commands=str(verification / 'commands.jsonl'))
            rows.append(row)
            for status in sorted((taskroot / 'attempts').glob('attempt-*/verifications/verify-*/status.json')):
                prior = o.read(status)
                if prior['status'] == 'VERIFY_FAILED':
                    command_path = status.parent / 'commands.jsonl'
                    command_text = command_path.read_text() if command_path.exists() else ''
                    ps_failure = 'MEMORY_SAMPLER_FAILED' in command_text and "Operation not permitted: 'ps'" in command_text
                    same_attempt = status.parent.parent.parent == attempt
                    failed.append({'status_file': str(status), 'commands_file': str(status.parent / 'commands.jsonl'),
                                   'reason': prior.get('error'), 'superseded_by_success_without_erasure': str(verification / 'summary.json'),
                                   'source_candidate_unchanged_on_retry': same_attempt,
                                   'classification': 'Sandbox ps instrumentation failure; same source candidate accepted on approved-access retry' if ps_failure and same_attempt else 'Preserved prior verification failure; see exact commands and attempt identity'})
    if totals != {'COMMON9': {'episodes': 378, 'responses': 1806, 'goals': 966, 'bindings': 630},
                  'COMMON_SUPPLEMENT6': {'episodes': 252, 'responses': 504, 'goals': 336, 'bindings': 252}}:
        raise RuntimeError('Full fresh acceptance totals differ: ' + str(totals))
    reviews = {'status': 'PENDING_FINAL_INDEPENDENT_REVIEWS'}
    if args.execution_review and args.grounding_review:
        for role, path in [('execution', args.execution_review), ('grounding', args.grounding_review)]:
            review = o.read(path)
            if not str(review.get('status', '')).startswith('PASS'):
                raise RuntimeError('Independent review is not PASS: ' + str(path))
            reviews[role] = {'audit': str(path.resolve()), 'sha256': o.sha(path)}
        reviews['status'] = 'PASS_NO_UNRESOLVED_P1_P2'
    counts = {cfg: [r['count'] for r in rows if r['configuration'] == cfg] for cfg in o.CONFIGS}
    result = {'status': 'COMPLETE_QA04_CODE_CHANGE_EXERCISES', 'baseline_freeze': str(args.freeze.resolve()),
              'baseline_freeze_sha256': args.digest, 'configurations': list(o.CONFIGS), 'independent_tasks_each': 6,
              'supplement_dependency_freeze': str(args.supplement_freeze.resolve()), 'supplement_dependency_freeze_sha256': args.supplement_digest,
              'accepted_independent_changes': 42, 'all_counts_by_configuration': counts,
              'mean_changed_operational_modules': {cfg: sum(values) / 6 for cfg, values in counts.items()},
              'regression_totals_by_population': totals, 'combined_regression_episodes': 630, 'actual_provider_api_calls': 0, 'tool_snapshot_versions': len(versions),
              'preserved_failed_verifications': failed,
              'excluded_prior_generations': [
                  {'path': str(o.RESULT.parent / name), 'accepted_rows_imported': 0,
                   'reason': 'Different candidate baseline; no counts or traces imported'}
                  for name in ['qa04', 'qa04-protocol-v2']],
              'independent_review': reviews,
              'scope': 'Same42 actual independent changes for six common adapter/source/publication tasks; not a general intrinsic architecture modifiability ranking. Common9 and supplement6 regressions remain separate populations on every copy, plus finite task acceptance. Deterministic semantic/vision doubles; pending audio is outside VoiceOFF scope.',
              'synthetic_usage_note': 'C01 17/11-token accounting uses isolated in-memory transport fixtures, not provider/API use or QA05 cost.',
              'report_generator_sha256': o.sha(Path(__file__)), 'rows': rows}
    o.dump(o.RESULT / 'QA04-RESULTS.json', result)
    lines = ['# Fresh QA04 results after deadline and protocol-metadata correction', '',
             '**42/42 independent changes accepted; actual provider API calls: 0.**', '',
             'The prior generations are preserved in `../qa04/` and `../qa04-protocol-v2/`; no prior count or regression is pooled here. This generation uses the corrected candidate baseline, exact freeze SHA `' + args.digest + '`.', '',
             '| Configuration | C01 | C02 | C03 | C04 | C05 | C06 | Mean |', '| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
    for cfg, values in counts.items():
        lines.append('| ' + cfg + ' | ' + ' | '.join(map(str, values)) + ' | ' + str(sum(values)) + '/6 = ' + format(sum(values) / 6, '.3f') + ' |')
    equal_counts = len({tuple(values) for values in counts.values()}) == 1
    lines += ['', 'Count sets are ' + ('observed equal' if equal_counts else 'observed different') + ' across configurations. These six common wire, ingestion, publication and source-authority tasks do not establish a general architecture ranking.', '',
              'Operational modules are explicitly mapped for this crate. Test-only, comment and whitespace changes are excluded; parent module registration and necessary baseline-contract preservation are counted. Actual module sets are recorded per exercise below and in the accepted module-review ledgers.', '',
              'The same feature purposes were independently implemented, built and exercised on this final baseline. Any source-anchor rebase is evidenced by the actual new patch and acceptance; no old diff/count is inherited. Existing deadline/cancellation and authority obligations remain required.', '',
              'All42 Rust test/build checks, finite feature acceptances and both regression populations passed. Common9 retains378 episodes,1,806 callbacks,966 goals and630 exact sets. Supplement6 separately retains252 episodes,504 callbacks,336 goals and252 exact sets. These630 regression episodes support the same42 change exercises; they are deterministic code checks, not primary model accuracy or timing observations.', '',
              '## Preserved failures', '',
              'No earlier generation failure is imported into this generation. Actual failures of this generation, if any, remain preserved and are classified from their own commands and attempt identity. Instrumentation failures remain distinct from candidate failures and actual formal-model proposal failures.', '']
    if failed:
        for failure in failed:
            lines += ['- [' + failure['classification'] + '](' + failure['status_file'] + ')']
    else:
        lines += ['No failed verification was recorded in this generation.']
    lines += ['', '## Verification evidence', '', '| Configuration/task | Accepted verification | Actual source patch | Actual operating modules |', '| --- | --- | --- | --- |']
    for row in rows:
        verification = Path(row['verification_summary']).parent.relative_to(o.RESULT)
        lines.append('| ' + row['configuration'] + '/' + row['task'] + ' | [summary](' + str(verification / 'summary.json') + ') | [diff](' + str(verification / 'final.patch') + ') | ' + ', '.join(row['operational_module_paths']) + ' |')
    base = str(args.freeze)
    supplement_flags = ' --supplement-freeze ' + str(args.supplement_freeze) + ' --supplement-digest ' + args.supplement_digest
    cmd = 'python3 benchmark/architecture/common9_poc_20261019/qa04_method_v3/'
    lines += ['', '## Reproduce', '',
              'Use a shell with read-only process/RSS access. Each failed attempt remains preserved; prepare creates a new independent source attempt and verify creates a new verification directory.', '', '```sh',
              cmd + 'orchestrate.py prepare --freeze ' + base + ' --digest ' + args.digest + supplement_flags,
              cmd + 'run_matrix.py --freeze ' + base + ' --digest ' + args.digest + supplement_flags + ' --workers 2',
              cmd + 'finish_report.py --freeze ' + base + ' --digest ' + args.digest + supplement_flags, '```', '',
              '## Independent review', '', reviews['status'], '']
    for role in ['execution', 'grounding']:
        if role in reviews:
            lines += ['- ' + role + ': [audit](' + reviews[role]['audit'] + ')']
    (o.RESULT / 'QA04-RESULTS.md').write_text('\n'.join(lines) + '\n')
    o.dump(o.RESULT / 'report-checkpoint.json', {'status': 'STABLE_ACCEPTED_42_PENDING_REVIEWS' if 'PENDING' in reviews['status'] else 'STABLE_QA04_WRITES_CLOSED',
        'at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'baseline_freeze_sha256': args.digest,
        'accepted': 42, 'provider_calls': 0, 'independent_review': reviews['status'],
        'report_sha256': o.sha(o.RESULT / 'QA04-RESULTS.json')})
    print(json.dumps({'accepted': 42, 'counts': counts, 'regressions': totals, 'independent_review': reviews['status']}))


if __name__ == '__main__':
    main()
