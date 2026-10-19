"""V3 correction wrapper; old evaluator source and all raw events stay intact."""
import copy
import hashlib
import importlib.util
import sys
import statistics
from pathlib import Path
from . import deadlines, endpoints

HERE = Path(__file__).resolve().parent
LEGACY_PATH = HERE.parent / 'evaluate.py'
sys.path.insert(0, str(HERE.parent))
spec = importlib.util.spec_from_file_location('_common9_preserved_evaluator_v1', LEGACY_PATH)
legacy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(legacy)
ORACLES, CONFIGURATIONS = legacy.ORACLES, legacy.CONFIGURATIONS


def evaluate(trace, oracle=None, mode='strict'):
    oracle = oracle if oracle is not None else ORACLES[trace['episode']]
    original = legacy.evaluate(trace, oracle)
    result = copy.deepcopy(original)
    audit = deadlines.audit(trace, mode)
    cutoff = audit['cutoff_ns']
    publication_proofs = {r['display_event_id']: r['admission_proved'] for r in audit['publication_audit']['rows']} if mode == 'strict' else None
    result['qa01'] = [endpoints.select(trace, expected, legacy.substantive, cutoff, strict=mode == 'strict', publication_proofs=publication_proofs) for expected in oracle['responses']]
    # The original trace supplies the unchanged full DAG/calls/RSS. Only the
    # terminal eligibility list is bounded; no wall interval is subtracted.
    bounded = dict(trace)
    goal_proofs = {r['event_id']: r['proved'] for r in audit['publication_audit']['terminal_checks']}
    bounded['events'] = [e for e in trace.get('events', []) if e.get('kind') != 'GOAL_COMPLETE' or
                         cutoff is not None and endpoints.integer(e.get('observed_ns')) and e['observed_ns'] <= cutoff and
                         (mode != 'strict' or goal_proofs.get(e.get('event_id')))]
    result['qa03'] = legacy.evaluate(bounded, oracle)['qa03']
    # QA02 concerns actual adopted state, not stdout delivery time. Its only
    # deadline proof is the producer's unique gate ledger for this stimulus.
    adoption_proofs = {}
    for event in trace.get('events', []):
        data = event.get('data', {})
        if event.get('kind') == 'STATE_EFFECT_COMMIT' and data.get('effect_kind') == 'SEMANTIC_ADOPTION':
            adoption_proofs.setdefault(data.get('data', {}).get('stimulus_id'), []).append(data)
    for row in result['qa02']:
        proofs = adoption_proofs.get(row['event_id'], [])
        adopted = next((e for e in trace.get('events', []) if e.get('event_id') == row['adoption_event_id']), None)
        row['pre_deadline_correct'] = row['correct']
        row['adoption_deadline_proofs'] = proofs
        valid = len(proofs) == 1 and any(e['effect_kind'] == 'SEMANTIC_ADOPTION' and
            e['data'].get('stimulus_id') == row['event_id'] and e['status'] in ('WITHIN_GATE', 'PRE_ARM_WITHIN_FINAL_CUTOFF')
            for e in audit['state_effect_checks'])
        valid = valid and adopted is not None and endpoints.integer(adopted.get('host_monotonic_ns')) and proofs[0]['linearization_ns'] <= adopted['host_monotonic_ns']
        if valid:
            for field in ('goal_id', 'revision'):
                if field in proofs[0].get('data', {}) and proofs[0]['data'][field] != adopted.get('data', {}).get(field):
                    valid = False
        if row['status'] == 'ADOPTED' and not valid and mode == 'strict':
            row.update(correct=False, status='UNRESOLVED_OR_LATE_ADOPTION')
    for row, old in zip(result['qa03'], original['qa03']):
        row['deadline_terminal_audit'] = [{'event_id': e.get('event_id'), 'reason': 'UNRESOLVED_OBSERVATION_DEADLINE' if cutoff is None else 'AFTER_OBSERVATION_DEADLINE'}
            for e in trace.get('events', []) if e.get('kind') == 'GOAL_COMPLETE' and e.get('data', {}).get('origin_event_id') == row['start_event'] and
            (cutoff is None or not endpoints.integer(e.get('observed_ns')) or e['observed_ns'] > cutoff)]
        if cutoff is None:
            row.update(status='UNRESOLVED', via_ns=None)
    result.update(version='COMMON9-EVALUATOR-v3', evaluation_mode=mode, deadline_audit=audit,
                  legacy_evaluator_sha256=hashlib.sha256(LEGACY_PATH.read_bytes()).hexdigest())
    result['original_vs_corrected'] = {}
    for key, identity, fields in (('qa01', 'response_id', ('status', 'latency_ns', 'accepted_callback_id')),
                                   ('qa02', 'id', ('status', 'correct', 'adoption_event_id')),
                                   ('qa03', 'id', ('status', 'via_ns', 'accepted_terminal_event_id'))):
        result['original_vs_corrected'][key] = [{identity: new[identity], 'changed': any(old.get(f) != new.get(f) for f in fields),
            'original': {f: old.get(f) for f in fields}, 'corrected': {f: new.get(f) for f in fields}}
            for old, new in zip(original[key], result[key])]
    result['primary_measurement_eligibility'] = 'ELIGIBLE' if mode == 'strict' and audit['status'] == 'CONSISTENT' else 'NOT_CERTIFIED'
    return result


def regression(record, oracle=None):
    result = evaluate(record, oracle)
    checks = {'all_required_substantive_callbacks': all(r['status'] == 'OBSERVED' for r in result['qa01']),
        'all_final_goals_observed': all(r['status'] == 'OBSERVED' for r in result['qa03']),
        'all_exact_referent_sets': all(r['correct'] for r in result['qa02']),
        'deadline_and_effect_audit': result['deadline_audit']['status'] == 'CONSISTENT',
        'no_recorded_functional_failures': not result['functional_failures'],
        'no_host_failures': not result['host_failures'], 'process_exit_ok': record.get('returncode') == 0,
        'no_automatic_handoff_field_violations': all(h['preservation_status'] != 'AUTO_VIOLATION' for h in result['semantic_diagnostics']['handoff_conditions'])}
    return {'status': 'PASS' if all(checks.values()) else 'FAIL', 'checks': checks,
        'counts': {'responses_planned': len(result['qa01']), 'responses_observed': sum(r['status'] == 'OBSERVED' for r in result['qa01']),
            'goals_planned': len(result['qa03']), 'goals_observed': sum(r['status'] == 'OBSERVED' for r in result['qa03']),
            'bindings_planned': len(result['qa02']), 'bindings_correct': sum(r['correct'] for r in result['qa02'])}, 'evaluation': result}


def summarize(results, repetitions=5):
    summary = legacy.summarize(results, repetitions)
    for key, row in summary.items():
        selected = [r for r in results if str(r.get('phase')) + ':' + r.get('configuration', '') == key]
        certified = [r for r in selected if r.get('primary_measurement_eligibility') == 'ELIGIBLE']
        certified_identities = {id(r) for r in certified}
        raw = copy.deepcopy({k: row[k] for k in ('qa01', 'qa02', 'qa03', 'qa05', 'qa06')})
        row['raw_observation_diagnostics'] = raw
        row['deadline_audit'] = {status: sum(r.get('deadline_audit', {}).get('status') == status for r in selected) for status in ('CONSISTENT', 'UNRESOLVED', 'VIOLATION')}
        row['primary_measurement_eligible_episodes'] = sum(r.get('primary_measurement_eligibility') == 'ELIGIBLE' for r in selected)
        row['qa01']['unresolved'] = sum(r['status'].startswith('UNRESOLVED') for e in selected for r in e['qa01'])
        valid_ids = not row['duplicate_episode_repetitions'] and not row['unexpected_episode_repetitions']
        for key_name, metric in (('qa01', 'latency_ns'), ('qa03', 'via_ns')):
            values = [r[metric] for episode in certified for r in episode[key_name] if r.get(metric) is not None]
            row[key_name]['observed'] = len(values)
            row[key_name]['observed_partial_mean_ms'] = statistics.mean(values)/1e6 if values and valid_ids else None
            row[key_name]['technically_uncertified_obligations'] = sum(len(r[key_name]) for r in selected if id(r) not in certified_identities)
            row[key_name]['status'] = 'COMPLETE_CERTIFIED_ENDPOINTS' if len(values) == row[key_name]['planned'] and valid_ids else 'INCOMPLETE_ENDPOINTS_OR_POPULATION'
        judged = [r for episode in certified for r in episode['qa02'] if r['status'] != 'UNRESOLVED_OR_LATE_ADOPTION']
        row['qa02'].update(correct=sum(r['correct'] for r in judged), judgments_observed=len(judged),
            technically_unjudgeable=sum(len(r['qa02']) for r in selected)-len(judged),
            semantic_incorrect_judgments=sum(not r['correct'] for r in judged),
            status='COMPLETE_CERTIFIED_JUDGMENTS' if len(judged) == row['qa02']['planned'] and valid_ids else 'INCOMPLETE_JUDGMENTS_OR_POPULATION')
        row['qa02']['fixed_denominator_accuracy'] = sum(r['correct'] for r in judged)/row['qa02']['planned'] if len(judged) == row['qa02']['planned'] and valid_ids else None
        row['qa02']['fixed_denominator_observed_success_lower_bound'] = sum(r['correct'] for r in judged)/row['qa02']['planned'] if valid_ids else None
        all_certified = len(certified) == len(selected) and row['population_complete']
        row['qa05']['comparability'] = 'COMPLETE_CERTIFIED_EXECUTION' if all_certified else 'INCOMPLETE_TECHNICAL_CERTIFICATION_OR_POPULATION'
        if not all_certified:
            row['qa05']['mean_nine_episode_bundle_usd'] = None
            row['qa06']['status'] = 'INCOMPLETE_TECHNICAL_CERTIFICATION_OR_POPULATION'
    return summary
