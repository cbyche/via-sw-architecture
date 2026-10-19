"""Actual harness lifecycle acceptances with explicit offline fault injection.

No network client, parser or paid model is used. Artificial receiver delay is
an injected boundary stimulus, never a model-time/performance observation.
Run only after the runtime lead announces a stable built candidate.
"""
import argparse
import copy
import hashlib
import json
import shutil
import time
from pathlib import Path

import harness
from evaluation_v3.evaluate import evaluate

HERE = Path(__file__).resolve().parent
CASES = ('all_meaning_invalid_full60', 'clock_mismatch', 'clock_missing',
         'initial_deadline_rejected', 'initial_deadline_missing_ack',
         'late_agent_before_cutoff', 'late_agent_after_cutoff',
         'extension_deadline_rejected', 'extension_deadline_missing_ack')


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def run_case(out, name, scale):
    scenario = 'S08' if name.startswith(('late_agent_', 'extension_')) else 'S02'
    configuration = 'CAAA' if name in ('clock_mismatch', 'clock_missing') else 'AAAA'
    episode = copy.deepcopy(next(e for e in json.loads((HERE / 'fixtures.json').read_text()) if e['id'] == scenario))
    old_client = harness.OfflineClient
    binary = harness.CRATE / 'target/debug/common9_poc'
    before = sha(binary)
    injected = []
    delay = (76 if name == 'late_agent_after_cutoff' else 40) * scale if scenario == 'S08' else 0

    class InjectedClient(old_client):
        def call(self, payload, meta):
            value, record = super().call(payload, meta)
            if name == 'all_meaning_invalid_full60' and meta['stage'] == 'meaning_interpret_and_respond':
                value['targets'] = ['UNREGISTERED-HARNESS-BOUNDARY-TEST-SOURCE']
                record['test_only_injection'] = 'Force every actual meaning proposal invalid; no adoption/RESPONSE_FINAL.'
            mandatory = payload.get('input', {}).get('contract', {}).get('mandatory', {})
            if delay and meta['stage'] == 'receiver_select_or_complete' and mandatory.get('event_id') == 'S08-U02':
                begin = time.monotonic_ns()
                time.sleep(delay)
                end = time.monotonic_ns()
                record['terminal_ns'] = end
                record['test_only_receiver_delay'] = {'begin_ns': begin, 'end_ns': end,
                    'requested_seconds': delay, 'purpose': 'Boundary stimulus only, not actual model performance.'}
            return value, record

    def transform(value):
        value = copy.deepcopy(value)
        kind = value.get('type')
        if kind == 'clock_probe' and name == 'clock_mismatch':
            value['host_probe_id'] = 'INJECTED-MISMATCH'
            injected.append({'kind': 'CLOCK_PROBE_ID_MISMATCH'})
        if kind == 'clock_probe' and name == 'clock_missing':
            value['type'] = 'SUPPRESSED_CLOCK_PROBE_FOR_OFFLINE_TEST'
            injected.append({'kind': 'CLOCK_PROBE_SUPPRESSED'})
        if kind == 'observation_deadline':
            initial = value.get('previous_deadline_ns') is None
            target = initial and name.startswith('initial_deadline_') or not initial and name.startswith('extension_deadline_')
            if target:
                original = copy.deepcopy(value)
                if name.endswith('rejected'):
                    value['deadline_ns'] = time.monotonic_ns() - 1
                else:
                    value['type'] = 'SUPPRESSED_DEADLINE_CONTROL_FOR_OFFLINE_TEST'
                injected.append({'kind': 'DEADLINE_CONTROL_REJECT_OR_SUPPRESS', 'original': original, 'sent': copy.deepcopy(value)})
        return value

    harness.OfflineClient = InjectedClient
    try:
        trace = harness.execute(out / 'attempt-01', episode, configuration, 1, offline=True,
                                time_scale=scale, input_transform=transform)
    finally:
        harness.OfflineClient = old_client
    result = evaluate(trace)
    provided = trace.get('provided_sources', {})
    updates = trace.get('observation_deadline_updates', [])
    stop = trace.get('effective_admission_stop_ns')
    settlement = trace.get('settlement_deadline_ns')
    cancel = trace.get('cancellation_requested_ns')
    contract = trace.get('contract_observation_deadline_ns')
    checks = {'no_paid_calls': trace.get('actual_API_calls') == 0 and not any(c.get('actual_api_sent') is True for c in trace.get('calls', [])),
        'same_binary_through_case': trace.get('binary_sha256') == before == sha(binary),
        'settlement_anchored_effective_stop_plus120': isinstance(stop, int) and settlement == stop + int(120 * max(scale, .01) * 1e9),
        'settlement_anchor_preserved': trace.get('settlement_anchor_ns') == stop,
        'contract_deadline_never_replaced_by_abort': contract == trace.get('observation_deadline_ns')}
    if name == 'all_meaning_invalid_full60':
        last = max(s['provided_at_ns'] for s in provided.values())
        checks.update(all_six_actual_sources=len(provided) == 6, no_actual_response_final=not trace.get('actual_response_final_ns'),
            six_missing_response_anchors=len(trace.get('anchor_fallback_ns', {})) == 6,
            full60_from_actual_provision=contract == last + int(60 * scale * 1e9),
            cancellation_not_anchor30=isinstance(cancel, int) and cancel >= contract,
            process_not_ended_at_anchor30=trace.get('end_ns', 0) >= contract,
            ordinary_model_failure_not_technical_abort=not trace.get('technical_abort_reason'))
    if name in ('clock_mismatch', 'clock_missing'):
        checks.update(not_certified=result['primary_measurement_eligibility'] == 'NOT_CERTIFIED',
            technical_stop_recorded=bool(trace.get('technical_abort_reason')),
            no_actual_semantic_double_dispatch=not any(c.get('status') == 'OFFLINE_DOUBLE' for c in trace.get('calls', [])),
            no_semantic_adoption=not any(e.get('kind') == 'SEMANTIC_ADOPTION' for e in trace.get('events', [])))
    if name.startswith(('initial_deadline_', 'extension_deadline_')):
        matching = [u for u in updates if (u.get('previous_deadline_ns') is None) == name.startswith('initial_deadline_')]
        checks.update(target_control_actually_injected=bool(injected) and bool(matching),
            not_certified=result['primary_measurement_eligibility'] == 'NOT_CERTIFIED',
            technical_stop_recorded=bool(trace.get('technical_abort_reason')),
            actual_abort_clock_retained=trace.get('technical_abort_requested_ns') == cancel,
            actual_proposed_clock_retained=trace.get('proposed_observation_deadline_ns') == updates[-1]['deadline_ns'])
        if name.startswith('initial_'):
            checks['no_old_deadline_fabricated'] = matching[0].get('previous_deadline_ns') is None and trace.get('last_accepted_observation_deadline_ns') is None
        else:
            previous = matching[0].get('previous_deadline_ns')
            checks['old_accepted_deadline_retained'] = isinstance(previous, int) and trace.get('last_accepted_observation_deadline_ns') == previous
            checks['effective_stop_no_later_than_old_deadline'] = isinstance(stop, int) and stop <= previous
    if name == 'late_agent_before_cutoff':
        late = [u for u in updates if u.get('reason') == 'LATE_ACTUAL_MANDATORY_SOURCE']
        checks.update(agent_actually_provided='S08-A01' in provided, actual_extension_observed=bool(late))
        if late:
            supplied = provided.get('S08-A01', {}).get('provided_at_ns')
            checks.update(provided_before_old_cutoff=isinstance(supplied, int) and supplied < late[0]['previous_deadline_ns'],
                extension_uses_actual_provision=late[0]['deadline_ns'] == supplied + int(60 * scale * 1e9),
                missing_anchor_record_preserved='S08-A01' in trace.get('anchor_fallback_ns', {}),
                no_technical_abort=not trace.get('technical_abort_reason'))
    if name == 'late_agent_after_cutoff':
        receiver = [e for e in trace.get('events', []) if e.get('kind') == 'RECEIVER_RESULT_AVAILABLE' and harness.wire_data(e.get('data', {}), 'receipt').get('stimulus_id') == 'S08-U02']
        checks.update(no_agent_source_fabricated='S08-A01' not in provided,
            no_deadline_extension=not any(u.get('reason') == 'LATE_ACTUAL_MANDATORY_SOURCE' for u in updates),
            receipt_only_after_cutoff=bool(receiver) and all(e.get('observed_ns', 0) > contract for e in receiver))
    # Scope the no-resumption audit to actual state/model admissions; raw late
    # receipt, failure, usage and process bookkeeping remain allowed evidence.
    state_after = [e for e in trace.get('events', []) if e.get('kind') == 'STATE_EFFECT_COMMIT' and isinstance(stop, int) and e.get('data', {}).get('linearization_ns', 0) >= stop]
    checks['no_semantic_commit_at_or_after_effective_stop'] = not state_after
    report = {'case': name, 'configuration': configuration, 'status': 'PASS' if all(checks.values()) else 'FAIL', 'checks': checks,
        'run_id': trace['run_id'], 'trace_path': str(out / 'attempt-01/raw/runs' / trace['run_id'] / 'trace.json'),
        'binary_sha256': before, 'injections': injected, 'artificial_receiver_delay_seconds': delay,
        'scale': scale, 'api_calls': 0, 'deadline_fields': {k: trace.get(k) for k in
            ('contract_observation_deadline_ns', 'proposed_observation_deadline_ns', 'last_accepted_observation_deadline_ns',
             'technical_abort_requested_ns', 'effective_admission_stop_ns', 'settlement_anchor_ns', 'settlement_deadline_ns', 'pending_deadline_update_cutoffs_at_cancel')},
        'actual_sources': provided, 'deadline_updates': updates,
        'technical_abort_reason': trace.get('technical_abort_reason'), 'technical_failures': trace.get('technical_failures'),
        'evaluation_audit_status': result['deadline_audit']['status'], 'scope': 'Actual offline harness acceptance; delay/injections are test stimuli, never model output or performance.'}
    save(out / 'check.json', report)
    return report


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path)
    parser.add_argument('--case', action='append', choices=CASES)
    parser.add_argument('--scale', type=float, default=.05)
    args = parser.parse_args()
    out = args.root or harness.DEFAULT / 'development' / ('harness-lifecycle-v3-' + str(time.monotonic_ns()))
    out.mkdir(parents=True, exist_ok=False)
    sources = [Path(__file__).resolve(), HERE / 'harness.py', HERE / 'lifecycle.py', HERE / 'offline.py',
               HERE / 'fixtures.json', HERE / 'oracles.json'] + sorted((HERE / 'evaluation_v3').glob('*.py'))
    candidate_sources = sorted((harness.CRATE / 'src').rglob('*.rs')) + [harness.CRATE / 'Cargo.toml', harness.CRATE / 'Cargo.lock']
    snapshot = out / 'checked-source'; snapshot.mkdir()
    inventory = []
    for source in sources + candidate_sources:
        relative = Path('candidate-source') / source.relative_to(harness.CRATE) if source in candidate_sources else Path('benchmark-source') / source.relative_to(HERE)
        dest = snapshot / relative; dest.parent.mkdir(parents=True, exist_ok=True); shutil.copyfile(source, dest)
        inventory.append({'source': str(source), 'sha256': sha(source), 'snapshot': str(dest)})
    save(out / 'source-manifest.json', {'sources': inventory, 'binary': {'path': str(harness.CRATE / 'target/debug/common9_poc'), 'sha256': sha(harness.CRATE / 'target/debug/common9_poc')}})
    results = []
    for name in args.case or CASES:
        try:
            result = run_case(out / name, name, args.scale)
        except Exception as error:
            result = {'case': name, 'status': 'FAIL', 'exception': type(error).__name__ + ': ' + str(error)}
            save(out / name / 'exception.json', result)
        results.append(result)
        save(out / 'progress.json', {'results': results, 'api_calls': 0})
        print(json.dumps({'case': name, 'status': result['status'], 'failed_checks': [k for k, v in result.get('checks', {}).items() if not v], 'exception': result.get('exception')}, ensure_ascii=False), flush=True)
    changed = [item['source'] for item in inventory if sha(item['source']) != item['sha256']]
    summary = {'status': 'PASS' if all(r['status'] == 'PASS' for r in results) and not changed else 'FAIL',
        'cases': len(results), 'results': results, 'sources_changed_during_check': changed, 'api_calls': 0,
        'scope': 'Offline actual-harness boundary acceptance; no paid calls or real parser; no performance comparison.'}
    save(out / 'summary.json', summary)
    print(json.dumps({'status': summary['status'], 'root': str(out), 'cases': len(results), 'api_calls': 0}), flush=True)
    raise SystemExit(0 if summary['status'] == 'PASS' else 1)


if __name__ == '__main__':
    main()
