"""Reporting-only adapter. No evaluator imports, subprocesses or runtime control.

Consumes explicit quiescent artifacts and independently recorded associations.
It cannot certify an identity-only saved result by hashing it after the fact.
"""
import argparse
from collections import Counter, defaultdict
from decimal import Decimal, InvalidOperation
import hashlib
import json
import math
from pathlib import Path
import re

VERSION = 'JOINT441_QUANTITATIVE_REPORT_v1'
SPEC_SHA = 'b4bb8735e67ec84dbef1cb193ac3a36f6ea70411094515a7f5d616b0ee82a09e'
FREEZE_SHA = '0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd'
APPROVAL_SHA = 'd3f78606f7a8bad161ee58a0c0fb5e7f7bf83e287140dbc9a4bbe2c028ce93bd'
CONFIGS = ('AAAA', 'BAAA', 'CAAA', 'ABAA', 'AABA', 'AAAB', 'AAAC')
POPULATIONS = {'COMMON9': {'cases': ('S02', 'S08', 'S11', 'S12', 'S15', 'S18', 'S19', 'S20', 'S26'), 'reps': 5, 'episodes': 45, 'qa01': 215, 'qa02': 75, 'qa03': 115},
               'COMMON_SUPPLEMENT6': {'cases': tuple('S' + str(i) for i in range(80, 86)), 'reps': 3, 'episodes': 18, 'qa01': 36, 'qa02': 18, 'qa03': 24}}
QA_FIELDS = {'qa01': ('responses', 'response_id', 'latency_ns'), 'qa02': ('bindings', 'id', None), 'qa03': ('goals', 'id', 'via_ns')}
PAIRS = {'DP1': (('AAAA', 'BAAA'), ('AAAA', 'CAAA'), ('BAAA', 'CAAA')), 'DP2': (('AAAA', 'ABAA'),),
         'DP3': (('AAAA', 'AABA'),), 'DP4': (('AAAA', 'AAAB'), ('AAAA', 'AAAC'), ('AAAB', 'AAAC'))}
CONDITIONS = dict(zip(POPULATIONS['COMMON_SUPPLEMENT6']['cases'], ('independent_overlap', 'sequential_countercondition', 'dependent_correction', 'small_dense', 'larger_sparse', 'native_on_dense')))
QA04_HASHES = {'report': 'f9dee15d44e02f4ea95826cd16d73812755e756cebd00b5afed5f103cbd27f0e',
              'classification': '55f87e66f4b511ed74686f6a5e66e0f42f3287febddb28cadbb3ecd5f5841130',
              'final_review': '43bdd5ef5f37378ae4f26f78e0038d21f40d85091a4f7b9f412866bfd20cea41',
              'raw630': '26cd58d883958a7e1b77e612c90acc299fb40d809de22cf6f506873bc74287ab',
              'checkpoint': '27a11b365fbcf194a4e93d97e577571dcee78c67aa85d043a893e6f6b7e281d0'}
LIMITATIONS = ['Observed timing means are micro means over certified observed obligations, not whole-population successful speed.',
    'Semantic review never replaces or filters a frozen QA01 callback or its denominator.',
    'No provider-internal concurrency or exact free-K2 proof; activation is diagnostic, not hypothesis acceptance.',
    'RSS is nominal50ms observed same-sample process-tree RSS, not continuous-time/full-product/remote-model memory; shared-page deduplication unverified.',
    'Local compute monetary cost is NOT_MEASURED. Recorded model comparison prices are not an invoice.',
    'Prior campaigns and probes remain separate global spending. AAAA observations are reused, not multiplied.',
    'Every prediction judgment is UNREVIEWED; numeric signs and missing evidence never create a winner.']


class InputError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise InputError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def identity(row):
    rep = row.get('rep')
    # Python bool==1 must never match a planned repetition.
    return (row.get('population'), row.get('configuration'), row.get('episode'),
            rep if type(rep) is int else ('INVALID_REP_TYPE', repr(rep)))


def key_object(key):
    return dict(zip(('population', 'configuration', 'episode', 'rep'), key))


def finite(value):
    return type(value) in (int, float) and math.isfinite(value) and value >= 0


def exact_ns(value):
    return type(value) is int and value >= 0


def decimal(value):
    if value is None or isinstance(value, bool):
        return None
    try:
        result = Decimal(str(value))
        return result if result.is_finite() and result >= 0 else None
    except (InvalidOperation, ValueError):
        return None


def decimal_text(value):
    return str(value) if value is not None else None


def validate_plan(jobs, oracles):
    keys = [identity(j) for j in jobs]
    expected = {(pop, cfg, case, rep) for pop, rule in POPULATIONS.items() for cfg in CONFIGS for case in rule['cases'] for rep in range(1, rule['reps'] + 1)}
    require(len(keys) == len(set(keys)) == 441 and set(keys) == expected, 'Invalid/duplicate441 planned identities')
    for pop, rule in POPULATIONS.items():
        require(set(oracles[pop]) == set(rule['cases']), 'Oracle episode membership differs')
        for qa, (group, id_field, _) in QA_FIELDS.items():
            count = 0
            for case in rule['cases']:
                rows = oracles[pop][case][group]
                require(len({r[id_field] for r in rows}) == len(rows), 'Duplicate frozen obligation ID')
                count += len(rows)
            require(count * rule['reps'] == rule[qa], 'Fixed oracle denominator differs: ' + pop + '/' + qa)
    return keys


def progress_lifecycle(progress, has_record, progress_complete):
    """Reconcile recorded controller order, not execution or evaluator semantics."""
    expected = ['STARTED', 'DISPATCHED', 'TERMINATED']
    if any(state not in expected for state in progress) or len(set(progress)) != len(progress):
        return {'status': 'CONFLICT', 'reason': 'UNSUPPORTED_OR_DUPLICATE_LIFECYCLE_STATE'}
    positions = [expected.index(state) for state in progress]
    if positions != sorted(positions):
        return {'status': 'CONFLICT', 'reason': 'OUT_OF_ORDER_LIFECYCLE'}
    if progress == expected:
        return {'status': 'EXACT_TERMINAL_LIFECYCLE', 'reason': None}
    is_prefix = progress == expected[:len(progress)]
    if progress_complete and (not is_prefix or has_record):
        return {'status': 'CONFLICT', 'reason': 'COMPLETE_PROGRESS_CONTRADICTS_RESULT_OR_SUPPORTED_LIFECYCLE'}
    if not progress_complete:
        return {'status': 'RECOVERABLE_INCOMPLETE_PROGRESS', 'reason': 'NOT_ENOUGH_PROGRESS_EVIDENCE_FOR_CERTIFICATION'}
    return {'status': 'SUPPORTED_UNSTARTED' if not progress else 'SUPPORTED_STARTED_UNCLOSED', 'reason': None}


def episode_state(records, progress, quiescence):
    if len(records) > 1:
        return 'IDENTITY_OR_SOURCE_CONFLICT'
    lifecycle = progress_lifecycle(progress, bool(records), quiescence.get('progress_complete') is True)
    if lifecycle['status'] == 'CONFLICT':
        return 'IDENTITY_OR_SOURCE_CONFLICT'
    states = set(progress)
    if not records:
        if 'TERMINATED' in states:
            return 'TERMINATED_TRACE_MISSING'
        if states.intersection(('STARTED', 'DISPATCHED')):
            return 'STARTED_UNCLOSED'
        if quiescence.get('progress_complete') and quiescence.get('controller_stopped') and quiescence.get('no_active_episode_processes'):
            return 'UNEXECUTED_VERIFIED'
        return 'CLOSED_RESULT_UNVERIFIED'
    record = records[0]
    if record.get('conflicts'):
        return 'IDENTITY_OR_SOURCE_CONFLICT'
    unavailable = record.get('unavailable_evidence', [])
    if any(e['artifact'] in ('trace', 'result') for e in unavailable):
        return 'TERMINATED_TRACE_MISSING' if 'TERMINATED' in states else 'CLOSED_RESULT_UNVERIFIED'
    if lifecycle['status'] != 'EXACT_TERMINAL_LIFECYCLE' or not record.get('provenance_valid'):
        return 'CLOSED_RESULT_UNVERIFIED'
    result = record['result']
    eligibility = result.get('primary_measurement_eligibility')
    audit = result.get('deadline_audit', {}).get('status')
    if eligibility == 'ELIGIBLE' and audit == 'CONSISTENT':
        return 'CLOSED_CERTIFIED'
    if eligibility == 'NOT_CERTIFIED' and audit in ('UNRESOLVED', 'VIOLATION'):
        return 'CLOSED_TECHNICALLY_UNCERTIFIED'
    return 'IDENTITY_OR_SOURCE_CONFLICT'


def obligation_state(qa, state, row):
    if state == 'UNEXECUTED_VERIFIED':
        return 'UNEXECUTED_VERIFIED', None
    if state in ('STARTED_UNCLOSED', 'TERMINATED_TRACE_MISSING', 'CLOSED_RESULT_UNVERIFIED', 'IDENTITY_OR_SOURCE_CONFLICT') or row is None:
        return 'EVIDENCE_UNAVAILABLE', 'ABI_CONTRADICTION' if state == 'IDENTITY_OR_SOURCE_CONFLICT' else 'MISSING_OR_UNVERIFIED_EVIDENCE'
    if state == 'CLOSED_TECHNICALLY_UNCERTIFIED':
        return 'TECHNICALLY_UNCERTIFIED', None
    status = row.get('status')
    if qa == 'qa02':
        if status == 'UNRESOLVED_OR_LATE_ADOPTION':
            return 'TECHNICALLY_UNJUDGEABLE_ADOPTION', None
        if status == 'ADOPTED' and type(row.get('correct')) is bool:
            return ('KNOWN_CORRECT' if row['correct'] else 'ADOPTED_INCORRECT'), None
        if status == 'NOT_ADOPTED' and row.get('correct') is False:
            return 'NOT_ADOPTED', None
        return 'EVIDENCE_UNAVAILABLE', 'ABI_CONTRADICTION'
    value = row.get(QA_FIELDS[qa][2])
    if status == {'qa01': 'UNRESOLVED_EARLIEST_BOUNDARY', 'qa03': 'UNRESOLVED'}[qa] and value is None:
        return 'CERTIFIED_UNRESOLVED', None
    if status == 'NOT_OBSERVED' and value is None:
        return 'CERTIFIED_NOT_OBSERVED', None
    if status == 'OBSERVED' and exact_ns(value):
        return 'CERTIFIED_OBSERVED', None
    return 'EVIDENCE_UNAVAILABLE', 'ABI_CONTRADICTION'


def timing_summary(rows, planned, frozen_ref=None):
    partition = Counter(r['reporting_state'] for r in rows)
    observed = [r['raw_value'][QA_FIELDS[r['qa']][2]] for r in rows if r['reporting_state'] == 'CERTIFIED_OBSERVED']
    valid = not any(r.get('reason') == 'ABI_CONTRADICTION' for r in rows)
    total = sum(observed) if observed else None
    require(sum(partition.values()) == planned, 'Timing partition does not equal fixed planned denominator')
    return {'planned': planned, 'partition': dict(partition), 'observed_count': len(observed), 'sum_ns': total,
            'mean_ms': total / len(observed) / 1e6 if observed and valid else None, 'aggregate_valid': valid,
            'observed_partial_mean_ms_diagnostic': total / len(observed) / 1e6 if observed else None,
            'frozen_summary_ref': frozen_ref, 'raw_diagnostic_ref': None}


def binding_summary(rows, planned):
    partition = Counter(r['reporting_state'] for r in rows)
    require(sum(partition.values()) == planned, 'Binding partition does not equal fixed denominator')
    known = sum(partition[s] for s in ('KNOWN_CORRECT', 'ADOPTED_INCORRECT', 'NOT_ADOPTED'))
    return {'planned': planned, 'partition': dict(partition), 'known_correct': partition['KNOWN_CORRECT'], 'known_judgments': known,
            'known_correct_over_planned_lower_bound': partition['KNOWN_CORRECT'] / planned if planned else None,
            'fixed_denominator_accuracy': partition['KNOWN_CORRECT'] / planned if planned and known == planned else None,
            'coverage_incomplete': known != planned, 'first_frozen_judgment_preserved': True}


def cost_summary(episodes, planned, reps):
    observed = [e for e in episodes if e.get('qa05')]
    values = [decimal(e['qa05'].get('known_api_cost_usd')) for e in observed]
    known = sum((v for v in values if v is not None), Decimal(0))
    reservations = [decimal(e['qa05'].get('unknown_reservation_usd')) for e in observed]
    unknown = [e['qa05'].get('unknown_call_count') for e in observed]
    complete = len(observed) == planned and all(e['state'] == 'CLOSED_CERTIFIED' for e in observed) and all(v is not None for v in values) and all(type(u) is int and u == 0 for u in unknown)
    return {'known_expenditure_usd': decimal_text(known) if any(v is not None for v in values) else None, 'unknown_call_count_observed': sum(u for u in unknown if type(u) is int) if any(type(u) is int for u in unknown) else None,
            'unknown_reservation_usd_observed': decimal_text(sum((v for v in reservations if v is not None), Decimal(0))) if any(v is not None for v in reservations) else None,
            'unknown_usage_records_missing': planned - len(observed) + sum(type(u) is not int for u in unknown), 'comparison_complete': complete,
            'representative_fixed_bundle_mean_usd': decimal_text(known / reps) if complete else None,
            'planned_episode_count': planned, 'observed_cost_records': len(observed), 'local_compute_monetized': 'NOT_MEASURED',
            'scope': 'All known expenditure, including failed/unused/discarded work where actually recorded; no timing/semantic inclusion filter.'}


def recorded_peak_consistency(qa06):
    """Check saved peak arithmetic only. Never reclassify or select samples."""
    value = qa06.get('peak_concurrent_sum_bytes')
    sample = qa06.get('peak_same_sample')
    if not finite(value) or value <= 0:
        return 'ABSENT_OR_INVALID_RECORDED_PEAK'
    if not isinstance(sample, dict) or not finite(sample.get('concurrent_sum_bytes')) or sample['concurrent_sum_bytes'] != value:
        return 'ABSENT_OR_MISMATCHED_RECORDED_SAME_SAMPLE'
    processes = sample.get('processes')
    if not isinstance(processes, list) or not processes or any(not isinstance(p, dict) or not finite(p.get('rss_bytes')) for p in processes):
        return 'ABSENT_OR_INVALID_RECORDED_PROCESS_CONTRIBUTIONS'
    if sum(p['rss_bytes'] for p in processes) != value:
        return 'RECORDED_PROCESS_CONTRIBUTIONS_DO_NOT_SUM_TO_PEAK'
    return None


def rss_summary(episodes, planned):
    values = [(e, e.get('qa06', {}).get('peak_concurrent_sum_bytes')) for e in episodes]
    valid = [(e, value) for e, value in values if recorded_peak_consistency(e.get('qa06', {})) is None]
    peak = max((v for _, v in valid), default=None)
    tied = [{'identity': e['identity'], 'run_id': e.get('run_id'), 'result_ref': e.get('result_ref'),
             'peak_same_sample': e['qa06'].get('peak_same_sample')} for e, value in valid if value == peak]
    complete = len(episodes) == planned and all(e['state'] == 'CLOSED_CERTIFIED' and e.get('qa06', {}).get('status') == 'OBSERVED'
        and recorded_peak_consistency(e.get('qa06', {})) is None for e in episodes)
    counters = ('sample_count', 'valid_sample_count', 'invalid_sample_count', 'confirmed_exit_boundary_count', 'startup_zero_rss_count', 'incomplete_registered_process_samples')
    return {'peak_concurrent_sum_bytes': peak, 'decimal_MB': peak / 1e6 if peak is not None else None,
            'MiB': peak / 1048576 if peak is not None else None, 'tied_same_sample_refs': tied, 'comparison_complete': complete,
            'maximum_status': 'COMPLETE_OBSERVED_MAXIMUM' if complete else 'OBSERVED_LOWER_BOUND_DIAGNOSTIC' if peak is not None else 'NOT_OBSERVED',
            'status_counts': dict(Counter(e.get('qa06', {}).get('status', 'MISSING') for e in episodes)),
            'counters': {k: sum(e.get('qa06', {}).get(k, 0) for e in episodes if type(e.get('qa06', {}).get(k)) is int) for k in counters},
            'max_sample_gap_ns': max((e.get('qa06', {}).get('max_sample_gap_ns') for e in episodes if exact_ns(e.get('qa06', {}).get('max_sample_gap_ns'))), default=None),
            'sampler_failure_records': [{'identity': e['identity'], 'records': e.get('qa06', {}).get('sampler_failures')} for e in episodes if e.get('qa06', {}).get('sampler_failures')],
            'unverified_peak_records': [{'identity': e['identity'], 'run_id': e.get('run_id'), 'result_ref': e.get('result_ref'),
                'recorded_peak_concurrent_sum_bytes': value, 'recorded_peak_same_sample': e.get('qa06', {}).get('peak_same_sample'),
                'reason': recorded_peak_consistency(e.get('qa06', {}))}
                for e, value in values if recorded_peak_consistency(e.get('qa06', {})) is not None and e.get('qa06')],
            'scope': LIMITATIONS[3], 'raw_values_and_incomplete_states_preserved': True}


def semantic_coverage(obligations):
    result = {}
    for qa, dimension in [('qa01', 'body'), ('qa03', 'task_revision')]:
        rows = [r for r in obligations if r['qa'] == qa]
        counts = Counter(r.get('semantic_annotation', {}).get(dimension, {}).get('status', 'UNREVIEWED') for r in rows)
        result[dimension] = {'planned': len(rows), 'review_status_counts': dict(counts), 'changes_QA_metric': False}
    result['approval_authority_and_handoff'] = {'planned_applicable': None, 'reason': 'Actual event applicability requires separate bound review; NOT_APPLICABLE is not PASS.'}
    return result


def build_report(jobs, oracles, records, progress, quiescence, *, calls=(), ledger=None,
                 qa04=None, summaries=None, raw_pairs=None, reviews=None, mechanisms=None,
                 registrations=None, provenance=None, external_issues=()):
    """Pure projection of already-proven input; I/O certification lives below."""
    keys = validate_plan(jobs, oracles)
    planned = set(keys)
    issues = list(external_issues)
    by_key, by_progress, run_ids = defaultdict(list), defaultdict(list), defaultdict(list)
    for record in records:
        key = identity(record['identity'])
        if key not in planned:
            issues.append({'kind': 'UNPLANNED_RESULT', 'identity': record['identity'], 'result_ref': record.get('result_ref')})
            continue
        by_key[key].append(record)
        run_id = record.get('run_id')
        if isinstance(run_id, str) and run_id:
            run_ids[run_id].append(key)
        elif record.get('result') or record.get('trace_ref'):
            record.setdefault('conflicts', []).append({'detail': 'MISSING_RUN_ID_FOR_OBSERVED_ARTIFACT'})
    duplicate_runs = {run for run, values in run_ids.items() if len(values) > 1}
    for row in progress:
        if row.get('state') in ('BLOCK_ADMITTED', 'BLOCK_BUDGET_STOP'):
            if any(field in row for field in ('population', 'configuration', 'episode', 'rep')):
                issues.append({'kind': 'BLOCK_CONTROL_WITH_EPISODE_IDENTITY', 'record': row})
                if identity(row) in planned:
                    by_progress[identity(row)].append(row['state'])
            continue
        key = identity(row)
        if key not in planned:
            issues.append({'kind': 'UNPLANNED_PROGRESS', 'record': row})
        else:
            by_progress[key].append(row.get('state'))
    progress_conflicts = set()
    lifecycles = {}
    for key in keys:
        lifecycle = progress_lifecycle(by_progress[key], bool(by_key[key]), quiescence.get('progress_complete') is True)
        lifecycles[key] = lifecycle
        if lifecycle['status'] == 'CONFLICT':
            progress_conflicts.add(key)
            issues.append({'kind': 'PROGRESS_IDENTITY_CONFLICT', 'identity': key_object(key),
                           'states': by_progress[key], 'reason': lifecycle['reason']})
    episode_rows, obligations = [], []
    for key in keys:
        pop, cfg, case, rep = key
        found = by_key[key]
        state = episode_state(found, by_progress[key], quiescence)
        if key in progress_conflicts:
            state = 'IDENTITY_OR_SOURCE_CONFLICT'
        if any(r.get('run_id') in duplicate_runs for r in found):
            state = 'IDENTITY_OR_SOURCE_CONFLICT'
        record = found[0] if len(found) == 1 else None
        result = record.get('result', {}) if record else {}
        if record and result and (result.get('phase') != 'FORMAL' or result.get('run_id') != record.get('run_id')
                or 'population' in result and result['population'] != pop
                or type(result.get('rep')) is not int
                or any(result.get(k) != v for k, v in [('configuration', cfg), ('episode', case), ('rep', rep)])):
            state = 'IDENTITY_OR_SOURCE_CONFLICT'
        if record and record.get('provenance_valid') and (result.get('version') != 'COMMON9-EVALUATOR-v3' or result.get('evaluation_mode') != 'strict'):
            state = 'IDENTITY_OR_SOURCE_CONFLICT'
        for qa, (group, id_field, _) in QA_FIELDS.items():
            declarations = {row[id_field]: row for row in oracles[pop][case][group]}
            actual_ids = set()
            for row in result.get(qa, []):
                oid = row.get(id_field)
                if oid not in declarations or oid in actual_ids or any(field not in row or row[field] != value for field, value in declarations.get(oid, {}).items()):
                    state = 'IDENTITY_OR_SOURCE_CONFLICT'
                    issues.append({'kind': 'OBLIGATION_DECLARATION_CONFLICT', 'identity': key_object(key), 'qa': qa, 'obligation_id': oid,
                                   'expected': declarations.get(oid), 'actual': row, 'result_ref': record.get('result_ref') if record else None})
                actual_ids.add(oid)
        if state == 'IDENTITY_OR_SOURCE_CONFLICT':
            issues.append({'kind': state, 'identity': key_object(key), 'all_result_refs': [r.get('result_ref') for r in found]})
        ep = {'identity': key_object(key), 'state': state, 'run_id': record.get('run_id') if record else None,
              'trace_ref': record.get('trace_ref') if record else None, 'result_ref': record.get('result_ref') if record else None,
              'progress_states': by_progress[key], 'progress_lifecycle': lifecycles[key],
              'unavailable_evidence': record.get('unavailable_evidence', []) if record else [], 'functional_failures': result.get('functional_failures'),
              'raw_trace_facts': record.get('raw_trace_facts') if record else None,
              'host_failures': result.get('host_failures'), 'returncode': result.get('returncode'),
              'qa05': result.get('qa05', {}), 'qa06': result.get('qa06', {}),
              'mechanism_annotation': (mechanisms or {}).get(key), 'source_conflicts': record.get('conflicts', []) if record else []}
        if ep['qa06'].get('status') == 'OBSERVED' and recorded_peak_consistency(ep['qa06']) is not None:
            issues.append({'kind': 'RSS_SAVED_PEAK_ABI_CONTRADICTION', 'identity': key_object(key),
                           'reason': recorded_peak_consistency(ep['qa06']), 'result_ref': ep['result_ref']})
        episode_rows.append(ep)
        for qa, (group, id_field, _) in QA_FIELDS.items():
            expected = oracles[pop][case][group]
            actual = result.get(qa, [])
            actual_map = defaultdict(list)
            for row in actual:
                actual_map[row.get(id_field)].append(row)
            expected_ids = {r[id_field] for r in expected}
            unknown = set(actual_map) - expected_ids
            if unknown:
                issues.append({'kind': 'UNKNOWN_OBLIGATION', 'identity': key_object(key), 'qa': qa, 'ids': sorted(map(str, unknown))})
            for declaration in expected:
                obligation_id = declaration[id_field]
                rows = actual_map.get(obligation_id, [])
                raw = rows[0] if len(rows) == 1 else None
                report_state, reason = obligation_state(qa, state, raw)
                if len(rows) > 1 or unknown:
                    report_state, reason = 'EVIDENCE_UNAVAILABLE', 'ABI_CONTRADICTION'
                if reason == 'ABI_CONTRADICTION':
                    issues.append({'kind': reason, 'identity': key_object(key), 'qa': qa, 'obligation_id': obligation_id, 'raw_rows': rows})
                review_key = (*key, qa, obligation_id)
                obligations.append({'identity': key_object(key), 'qa': qa, 'obligation_id': obligation_id,
                    'run_id': ep['run_id'], 'trace_ref': ep['trace_ref'], 'result_ref': ep['result_ref'],
                    'episode_state': state, 'reporting_state': report_state, 'reason': reason,
                    'frozen_expectation': declaration, 'raw_value': raw,
                    'raw_result_row_pointer': '/' + qa + '/' + str(actual.index(raw)) if raw is not None else None,
                    'semantic_annotation': (reviews or {}).get(review_key, {}), 'semantic_annotation_changes_metric': False})
    episode_groups, obligation_groups = defaultdict(list), defaultdict(list)
    obligation_index = {}
    for episode in episode_rows:
        k = identity(episode['identity'])
        episode_groups[k[:2]].append(episode)
    for row in obligations:
        k = identity(row['identity'])
        obligation_groups[k[:2]].append(row)
        obligation_index[(*k, row['qa'], row['obligation_id'])] = row
    populations, case_rows = [], []
    for pop, rule in POPULATIONS.items():
        configurations = []
        for cfg in CONFIGS:
            eps, obs = episode_groups[(pop, cfg)], obligation_groups[(pop, cfg)]
            item = {'configuration': cfg, 'execution_partition': dict(Counter(e['state'] for e in eps)),
                    'qa01': timing_summary([r for r in obs if r['qa'] == 'qa01'], rule['qa01']),
                    'qa02': binding_summary([r for r in obs if r['qa'] == 'qa02'], rule['qa02']),
                    'qa03': timing_summary([r for r in obs if r['qa'] == 'qa03'], rule['qa03']),
                    'qa04_ref': qa04, 'qa05': cost_summary(eps, rule['episodes'], rule['reps']),
                    'qa06': rss_summary(eps, rule['episodes']), 'semantic_coverage': semantic_coverage(obs)}
            frozen = (summaries or {}).get((pop, cfg))
            if frozen:
                for qa, count_field in [('qa01', 'observed'), ('qa03', 'observed')]:
                    supplied = frozen.get(qa, {})
                    item[qa]['frozen_summary_ref'] = frozen.get('source_ref')
                    if count_field in supplied and supplied[count_field] != item[qa]['observed_count']:
                        issues.append({'kind': 'FROZEN_SUMMARY_COUNT_MISMATCH', 'population': pop, 'configuration': cfg, 'qa': qa})
                    if 'observed_partial_mean_ms' in supplied and supplied['observed_partial_mean_ms'] != item[qa]['mean_ms']:
                        issues.append({'kind': 'FROZEN_SUMMARY_MEAN_MISMATCH', 'population': pop, 'configuration': cfg, 'qa': qa, 'supplied': supplied['observed_partial_mean_ms'], 'adapter': item[qa]['mean_ms']})
                mapping = [('qa02', 'correct', 'known_correct'), ('qa02', 'judgments_observed', 'known_judgments'),
                           ('qa02', 'fixed_denominator_accuracy', 'fixed_denominator_accuracy'),
                           ('qa06', 'maximum_same_sample_rss_bytes', 'peak_concurrent_sum_bytes')]
                for qa, supplied_field, computed_field in mapping:
                    if supplied_field in frozen.get(qa, {}) and frozen[qa][supplied_field] != item[qa][computed_field]:
                        issues.append({'kind': 'FROZEN_SUMMARY_FIELD_MISMATCH', 'population': pop, 'configuration': cfg,
                                       'qa': qa, 'field': supplied_field, 'supplied': frozen[qa][supplied_field], 'adapter': item[qa][computed_field]})
                if frozen.get('source_ref'):
                    item['frozen_summary_ref'] = frozen['source_ref']
                known = decimal(frozen.get('qa05', {}).get('known_total_usd'))
                own_known = decimal(item['qa05']['known_expenditure_usd'])
                item['qa05']['frozen_known_cost_residual_usd'] = str(own_known - known) if known is not None and own_known is not None else None
                if 'unknown_calls' in frozen.get('qa05', {}) and frozen['qa05']['unknown_calls'] != item['qa05']['unknown_call_count_observed']:
                    issues.append({'kind': 'FROZEN_SUMMARY_UNKNOWN_USAGE_MISMATCH', 'population': pop, 'configuration': cfg})
            configurations.append(item)
            for rep in range(1, rule['reps'] + 1):
                fixed_bundle = [e for e in eps if e['identity']['rep'] == rep]
                bundle_obligations = [o for o in obs if o['identity']['rep'] == rep]
                case_rows.append({'kind': 'FIXED_REPETITION_BUNDLE', 'population': pop, 'configuration': cfg, 'rep': rep,
                                  'planned_cases': list(rule['cases']),
                                  'qa01': timing_summary([o for o in bundle_obligations if o['qa'] == 'qa01'], rule['qa01'] // rule['reps']),
                                  'qa02': binding_summary([o for o in bundle_obligations if o['qa'] == 'qa02'], rule['qa02'] // rule['reps']),
                                  'qa03': timing_summary([o for o in bundle_obligations if o['qa'] == 'qa03'], rule['qa03'] // rule['reps']),
                                  'qa05': cost_summary(fixed_bundle, len(rule['cases']), 1),
                                  'qa06': rss_summary(fixed_bundle, len(rule['cases']))})
                for case in rule['cases']:
                    selected = [r for r in obs if r['identity']['rep'] == rep and r['identity']['episode'] == case]
                    episode = next(e for e in eps if e['identity']['rep'] == rep and e['identity']['episode'] == case)
                    case_rows.append({'kind': 'CASE_REPETITION', **key_object((pop, cfg, case, rep)), 'episode_state': episode['state'],
                        'condition': CONDITIONS.get(case), 'qa01': timing_summary([r for r in selected if r['qa'] == 'qa01'], len(oracles[pop][case]['responses'])),
                        'qa02': binding_summary([r for r in selected if r['qa'] == 'qa02'], len(oracles[pop][case]['bindings'])),
                        'qa03': timing_summary([r for r in selected if r['qa'] == 'qa03'], len(oracles[pop][case]['goals'])),
                        'qa05': cost_summary([episode], 1, 1), 'qa06': rss_summary([episode], 1), 'mechanism_annotation': episode['mechanism_annotation']})
        populations.append({'population': pop, 'planned': rule, 'configurations': configurations,
                            'case_repetition_refs': {'file': 'case-repetition.jsonl', 'population': pop}})
    pairs = make_pairs(jobs, oracles, obligation_index, episode_rows)
    costs = reconcile_calls(calls, ledger or {}, (provenance or {}).get('formal_root'))
    analysis_totals = (provenance or {}).get('frozen_analysis_totals')
    if analysis_totals is not None:
        recorded_known = decimal(analysis_totals.get('total_actual_recorded_API_cost_usd'))
        projected_known = sum((decimal(e['qa05'].get('known_api_cost_usd')) or Decimal(0) for e in episode_rows), Decimal(0))
        recorded_unknown = analysis_totals.get('unknown_call_count')
        projected_unknown = sum(e['qa05'].get('unknown_call_count', 0) for e in episode_rows if type(e['qa05'].get('unknown_call_count', 0)) is int)
        costs['frozen_analysis_projection'] = {'recorded': analysis_totals, 'per_result_known_usd_decimal': str(projected_known),
            'known_residual_usd': str(projected_known - recorded_known) if recorded_known is not None else None,
            'per_result_unknown_count': projected_unknown, 'unknown_count_matches': recorded_unknown == projected_unknown,
            'scope': 'Decimal residual preserves floating-point representation differences; never reprices/reallocates.'}
        if recorded_known is None or recorded_unknown != projected_unknown:
            issues.append({'kind': 'FROZEN_ANALYSIS_COST_ABI_OR_UNKNOWN_COUNT_MISMATCH'})
    projected_costs = [decimal(e['qa05'].get('known_api_cost_usd')) for e in episode_rows if e.get('qa05')]
    costs['per_result_known_usd_decimal'] = str(sum((v for v in projected_costs if v is not None), Decimal(0))) if projected_costs else None
    cost_residual = decimal(costs['per_result_known_usd_decimal'])
    costs['per_result_minus_unique_formal_calls_usd'] = str(cost_residual - Decimal(costs['unique_formal_call_known_usd'])) if cost_residual is not None else None
    if cost_residual is not None and cost_residual != Decimal(costs['unique_formal_call_known_usd']):
        costs['discrepancies'].append({'kind': 'PER_RESULT_VS_UNIQUE_FORMAL_CALL_COST_RESIDUAL', 'residual_usd': costs['per_result_minus_unique_formal_calls_usd']})
        costs['reconciliation_complete'] = False
    if costs['conflicts']:
        issues.extend({'kind': 'COST_RECONCILIATION_CONFLICT', 'detail': c} for c in costs['conflicts'])
    predictions = prediction_cells(registrations or {}, provenance or {})
    complete_states = {'UNEXECUTED_VERIFIED', 'CLOSED_CERTIFIED', 'CLOSED_TECHNICALLY_UNCERTIFIED'}
    complete = all(e['state'] in complete_states for e in episode_rows) and all(r['reporting_state'] != 'EVIDENCE_UNAVAILABLE' for r in obligations) and qa04 is not None and bool(quiescence.get('progress_complete'))
    report = {'version': VERSION, 'status': 'INVALID_REPORT_INPUT' if issues else 'COMPLETE_INPUT_REPORT' if complete else 'INCOMPLETE_EVIDENCE_REPORT',
        'provenance': provenance or {}, 'populations': populations, 'qa04_reference': qa04 or {'status': 'MISSING_QA04_BINDING'},
        'pairs': {'planned_rows': len(pairs), 'rows_file': 'paired.jsonl', 'certified_timing_summaries': summarize_pairs(pairs),
                  'raw_analyzer_paired_diagnostics': raw_pairs, 'shared_AAAA_reuse': 'Same exact records reused, never independent copies or repeated expenditure'},
        'cost_reconciliation': costs, 'prediction_worklist': predictions,
        'input_issues': issues, 'limitations': LIMITATIONS, 'completion_scope': 'Input/evidence coverage is not model/semantic success or a completed campaign claim.',
        'planned_totals': {'episodes': 441, 'responses': 1757, 'bindings': 651, 'goals': 973},
        'execution_coverage': dict(Counter(e['state'] for e in episode_rows))}
    return report, {'planned-episodes.jsonl': episode_rows, 'obligations.jsonl': obligations,
                    'case-repetition.jsonl': case_rows, 'paired.jsonl': pairs, 'calls.jsonl': costs['calls']}


def get_pointer(value, pointer):
    require(pointer == '' or isinstance(pointer, str) and pointer.startswith('/'), 'Invalid JSON pointer')
    if not pointer:
        return value
    for part in pointer[1:].split('/'):
        require(not any(part[i] == '~' and (i + 1 == len(part) or part[i + 1] not in '01') for i in range(len(part))), 'Invalid JSON pointer escape')
        part = part.replace('~1', '/').replace('~0', '~')
        if isinstance(value, list):
            require(re.fullmatch(r'0|[1-9][0-9]*', part) is not None and int(part) < len(value), 'Noncanonical or out-of-range JSON array index')
            value = value[int(part)]
        else:
            require(isinstance(value, dict) and part in value, 'JSON pointer does not name an existing object member')
            value = value[part]
    return value


class ArtifactReader:
    """Explicit references only; never search a directory or import a source."""
    def __init__(self, root, file_limit=16 * 1024 * 1024, total_limit=8 * 1024 * 1024 * 1024):
        self.root = Path(root).resolve()
        require(0 < file_limit <= 64 * 1024 * 1024 and 0 < total_limit <= 16 * 1024 * 1024 * 1024, 'Unsupported read limits')
        self.file_limit, self.total_limit = file_limit, total_limit
        self.bytes_read, self.inputs = 0, {}

    def read(self, ref, *, jsonl=False, raw=False):
        require(isinstance(ref, dict), 'File reference must be an object')
        require(type(ref.get('bytes')) is int and ref['bytes'] >= 0 and isinstance(ref.get('sha256'), str) and re.fullmatch('[0-9a-f]{64}', ref['sha256']), 'Invalid file hash/byte reference ABI')
        name = ref['path']
        require(isinstance(name, str) and Path(name).is_absolute() and '..' not in Path(name).parts, 'File reference must be explicit absolute path without traversal')
        path = Path(name)
        require(path.resolve().is_relative_to(self.root), 'File reference outside exact formal root; restore mapping is unsupported')
        require(not any(p.is_symlink() for p in (path, *path.parents)), 'Symlink file/ancestor unsupported')
        before = path.stat()
        require(path.is_file(), 'Artifact is not a regular file: ' + name)
        require(before.st_size <= self.file_limit and self.bytes_read + before.st_size <= self.total_limit, 'Oversized input: ' + name)
        data = path.read_bytes()
        require(path.stat().st_mtime_ns == before.st_mtime_ns and len(data) == before.st_size, 'Input changed: ' + name)
        require(len(data) == ref['bytes'] and sha(data) == ref['sha256'], 'Input hash/bytes mismatch: ' + name)
        item = {'path': name, 'sha256': sha(data), 'bytes': len(data)}
        require(name not in self.inputs or self.inputs[name] == item, 'Conflicting copies of file reference')
        self.inputs[name] = item
        self.bytes_read += len(data)
        if raw:
            return data
        if jsonl:
            require(not ref.get('json_pointer'), 'JSONL pointer unsupported; use one explicit bounded record file')
            return [json.loads(line, parse_constant=lambda value: (_ for _ in ()).throw(InputError('Nonfinite JSON constant: ' + value))) for line in data.splitlines() if line.strip()]
        return get_pointer(json.loads(data, parse_constant=lambda value: (_ for _ in ()).throw(InputError('Nonfinite JSON constant: ' + value))), ref.get('json_pointer', ''))


def same_ref(a, b):
    return all(a.get(k) == b.get(k) for k in ('path', 'sha256', 'bytes', 'json_pointer'))


def validate_association(association, record, freeze, freeze_ref, oracle_ref, root):
    require(association.get('version') == 'JOINT441_RESULT_ASSOCIATION_v1', 'Unsupported association ABI; identity-only/partial envelopes are not certified')
    require(association.get('phase') == 'FORMAL' and association.get('mode') == 'strict' and association.get('evaluator_version') == 'COMMON9-EVALUATOR-v3', 'Wrong phase/mode/evaluator association')
    require(association.get('formal_root') == str(root) and association.get('formal_freeze_sha256') == freeze_ref['sha256'], 'Mixed root/freeze association')
    require(association.get('binary_sha256') == freeze['binary']['sha256'], 'Mixed binary association')
    require(identity(association['identity']) == identity(record['identity']) and association.get('run_id') == record['run_id'], 'Association identity differs')
    result = record['result']
    expected = record['identity']
    require(result.get('run_id') == record['run_id'] and result.get('phase') == 'FORMAL'
            and type(result.get('rep')) is int
            and all(result.get(k) == expected[k] for k in ('configuration', 'episode', 'rep'))
            and ('population' not in result or result['population'] == expected['population']), 'Result body identity differs from association')
    require(same_ref(association['result_ref'], record['result_ref']) and same_ref(association['trace_ref'], record['trace_ref']), 'Association trace/result mismatch')
    require(same_ref(association['oracle_ref'], oracle_ref), 'Association missing exact population oracle')
    require(association.get('source_sha256') == {s['relativePath']: s['sha256'] for s in freeze['sources']}, 'Incomplete evaluator/dependency source association')
    require(isinstance(association.get('recorded_analysis_argv'), list) and association['recorded_analysis_argv'] and all(isinstance(v, str) for v in association['recorded_analysis_argv']), 'Recorded analysis command missing; never synthesized or executed')
    require(association.get('recorded_by') and association.get('recorded_at'), 'Independent recorded association metadata missing')
    return True


def approved_qa04(reader, supplied, approval):
    values, refs = {}, {}
    for key, expected in QA04_HASHES.items():
        item = supplied[key]
        matches = [a for a in approval['artifacts'] if a.get('role') == item['approval_role']]
        require(len(matches) == 1, 'Missing/ambiguous exact QA04 approval role')
        bound = matches[0]
        reference = item['ref']
        require(reference['path'] in (bound['original_path'], bound['copied_path']) and reference['sha256'] == bound['sha256'] == expected and reference['bytes'] == bound['bytes'], 'QA04 role path/hash/size not bound by approval')
        values[key], refs[key] = reader.read(reference), reference
    report = values['report']
    require(report['accepted_independent_changes'] == 42 and report['combined_regression_episodes'] == 630 and report['actual_provider_api_calls'] == 0, 'Wrong QA04 dataset')
    require(len(report['rows']) == 42 and {(r['configuration'], r['task']) for r in report['rows']} == {(cfg, 'C0' + str(i)) for cfg in CONFIGS for i in range(1, 7)}, 'Wrong QA04 change population')
    require(all(report['all_counts_by_configuration'][cfg] == [1, 1, 3, 3, 1, 2] and report['mean_changed_operational_modules'][cfg] == 11 / 6 for cfg in CONFIGS), 'Historical/raw QA04 count cannot substitute')
    review = values['final_review']
    require(review['status'] == 'PASS_QA04_FUNCTIONAL_AND_MODULE_ACCEPTANCE_WITH_RETAINED_RSS_DIAGNOSTICS'
            and all(review['input_sha256'][field] == refs[key]['sha256'] for field, key in
                    [('report_json', 'report'), ('count_classification_v2', 'classification'), ('raw630_audit', 'raw630')]), 'QA04 final acceptance missing')
    checkpoint = values['checkpoint']
    require(checkpoint.get('status') == 'STABLE_QA04_WRITES_CLOSED' and checkpoint.get('writes_closed') is True
            and checkpoint['report_json']['sha256'] == refs['report']['sha256']
            and checkpoint['count_classification']['sha256'] == refs['classification']['sha256']
            and checkpoint['independent_reviews']['raw630']['sha256'] == refs['raw630']['sha256']
            and checkpoint['independent_reviews']['final_qa04_acceptance']['sha256'] == refs['final_review']['sha256'], 'QA04 final checkpoint links differ')
    return {'status': 'FINAL_ACCEPTED42_CHANGES630_REGRESSIONS', 'artifacts': refs, 'counts': report['all_counts_by_configuration'],
            'numerator': 11, 'denominator': 6, 'mean': 11 / 6, 'observed_tie_scope': 'Six specified common changes only; not intrinsic production architecture equality.',
            'raw_counts_preserved': report.get('raw_counts_preserved'), 'regression_RSS_diagnostics_separate_from_formal_QA06': report.get('qa06_diagnostics')}


def make_pairs(jobs, oracles, obligations, episodes):
    episode_index = {identity(e['identity']): e for e in episodes}
    seen, rows = set(), []
    for job in jobs:
        pop, _, case, rep = identity(job)
        if (pop, case, rep) in seen:
            continue
        seen.add((pop, case, rep))
        for dp, comparisons in PAIRS.items():
            for left, right in comparisons:
                for qa, (group, field, metric) in QA_FIELDS.items():
                    for expected in oracles[pop][case][group]:
                        sides = [obligations[(pop, cfg, case, rep, qa, expected[field])] for cfg in (left, right)]
                        states = [r['reporting_state'] for r in sides]
                        certified = qa != 'qa02' and states == ['CERTIFIED_OBSERVED', 'CERTIFIED_OBSERVED'] and all(r['episode_state'] == 'CLOSED_CERTIFIED' for r in sides)
                        delta = sides[1]['raw_value'][metric] - sides[0]['raw_value'][metric] if certified else None
                        rows.append({'population': pop, 'dp': dp, 'left': left, 'right': right, 'episode': case, 'rep': rep,
                            'qa': qa, 'obligation_id': expected[field], 'side_states': states,
                            'side_refs': [{'run_id': r['run_id'], 'trace_ref': r['trace_ref'], 'result_ref': r['result_ref'], 'episode_state': r['episode_state'], 'raw_value': r['raw_value']} for r in sides],
                            'certified_intersection': certified, 'delta_ns_right_minus_left': delta,
                            'categorical_pair': states if qa == 'qa02' else None,
                            'exclusion_reason': None if certified else 'CATEGORICAL_PAIR_NOT_TIMING' if qa == 'qa02' else 'BOTH_SIDES_MUST_BE_CLOSED_CERTIFIED_AND_SAME_OBLIGATION_OBSERVED',
                            'semantic_annotation_is_not_a_pair_filter': True})
                for qa in ('qa05', 'qa06'):
                    sides = [episode_index[(pop, cfg, case, rep)] for cfg in (left, right)]
                    complete = all(e['state'] == 'CLOSED_CERTIFIED' for e in sides)
                    if qa == 'qa05':
                        values = [decimal(e['qa05'].get('known_api_cost_usd')) for e in sides]
                        comparable = complete and all(v is not None for v in values) and all(type(e['qa05'].get('unknown_call_count')) is int and e['qa05']['unknown_call_count'] == 0 for e in sides)
                    else:
                        values = [e['qa06'].get('peak_concurrent_sum_bytes') for e in sides]
                        comparable = complete and all(e['qa06'].get('status') == 'OBSERVED' for e in sides) and all(recorded_peak_consistency(e['qa06']) is None for e in sides)
                    rows.append({'population': pop, 'dp': dp, 'left': left, 'right': right, 'episode': case, 'rep': rep, 'qa': qa,
                        'obligation_id': None, 'side_states': [e['state'] for e in sides],
                        'recorded_values': [decimal_text(v) for v in values] if qa == 'qa05' else values,
                        'side_statuses': [e[qa].get('status') for e in sides], 'comparison_complete': comparable,
                        'comparable_delta_right_minus_left': decimal_text(values[1] - values[0]) if comparable and qa == 'qa05' else values[1] - values[0] if comparable else None,
                        'exclusion_reason': None if comparable else 'INCOMPLETE_EXECUTION_USAGE_OR_RSS_OBSERVATION',
                        'scope': 'Supplementary episode pair; representative cost is fixed-bundle mean and memory is fixed-population maximum.'})
    return rows


def summarize_pairs(rows):
    groups = defaultdict(list)
    for row in rows:
        if row['qa'] in ('qa01', 'qa03'):
            groups[(row['population'], row['dp'], row['left'], row['right'], row['qa'])].append(row)
    result = []
    for key, selected in groups.items():
        values = [r['delta_ns_right_minus_left'] for r in selected if r['certified_intersection']]
        result.append(dict(zip(('population', 'dp', 'left', 'right', 'qa'), key), planned_pair_count=len(selected),
            paired_count=len(values), sum_delta_ns=sum(values) if values else None,
            mean_delta_ms=sum(values) / len(values) / 1e6 if values else None,
            scope='Same-obligation certified intersection only; original full-population coverage remains primary.'))
    return result


def project_call(raw, identity_value, run_id, source_ref):
    actual = raw.get('actual_api_sent')
    kind = 'ACTUAL_CLOUD_DISPATCH' if actual is True else 'LOCAL_MODEL' if raw.get('evidence_kind') == 'LOCAL_MODEL' else 'ADMINISTRATIVE_OR_UNSENT'
    fields = ('call_uuid', 'stage', 'role', 'evidence_kind', 'status', 'actual_api_sent', 'input_tokens', 'output_tokens',
              'cost_usd', 'cost_state', 'reservation_usd', 'price', 'scope_root', 'initialization_ns', 'inference_ns', 'model')
    result = {k: raw.get(k) for k in fields}
    result.update(identity=identity_value, run_id=run_id, record_kind=kind, trace_pointer=source_ref,
                  raw_usage_ref={'source': source_ref, 'field': 'raw_usage', 'present': 'raw_usage' in raw},
                  raw_usage_metadata=raw.get('raw_usage'),
                  recorded_work_disposition={k: raw[k] for k in ('discarded', 'discarded_after_candidate_end', 'unused', 'retried', 'retry_of', 'cancelled') if k in raw},
                  request_response_file_refs={k: {'recorded_path': raw[k], 'content_read_by_adapter': False} for k in ('request_file', 'response_file') if k in raw},
                  local_compute_monetized='NOT_MEASURED',
                  token_interpretation='Recorded provider usage fields' if kind == 'ACTUAL_CLOUD_DISPATCH' and raw.get('raw_usage') is not None else 'Token zeros/fields are not assumed measured provider usage')
    return result


def reconcile_calls(calls, ledger, formal_root):
    unique, anonymous, conflicts, discrepancies, annotation_disagreements = {}, [], [], [], []
    for index, call in enumerate(calls):
        value = dict(call)
        uuid = value.get('call_uuid')
        signature = {k: v for k, v in value.items() if k not in ('trace_pointer', 'raw_usage_ref', 'occurrences', 'recorded_work_disposition', 'work_disposition_occurrences')}
        if uuid is None:
            value['anonymous_record_index'] = index
            anonymous.append(value)
            if value.get('actual_api_sent') is True:
                conflicts.append({'kind': 'ACTUAL_CLOUD_CALL_WITHOUT_UUID', 'source': value.get('trace_pointer')})
            continue
        if uuid in unique:
            previous = unique[uuid]
            if previous['signature'] != signature:
                conflicts.append({'kind': 'CONFLICTING_UUID_COPIES', 'call_uuid': uuid, 'first': previous['value'], 'other': value})
            previous['value']['occurrences'].append(value.get('trace_pointer'))
            previous['value']['work_disposition_occurrences'].append({'source': value.get('trace_pointer'), 'recorded_fields': value.get('recorded_work_disposition', {})})
            merged = previous['value'].setdefault('recorded_work_disposition', {})
            for field, recorded in value.get('recorded_work_disposition', {}).items():
                if field in merged and merged[field] != recorded:
                    annotation_disagreements.append({'call_uuid': uuid, 'field': field, 'first': merged[field], 'other': recorded,
                                                    'source': value.get('trace_pointer'), 'changes_financial_identity': False})
                else:
                    merged[field] = recorded
        else:
            value['recorded_work_disposition'] = dict(value.get('recorded_work_disposition', {}))
            value['occurrences'] = [value.get('trace_pointer')]
            value['work_disposition_occurrences'] = [{'source': value.get('trace_pointer'), 'recorded_fields': dict(value.get('recorded_work_disposition', {}))}]
            unique[uuid] = {'signature': signature, 'value': value}
    rows = [entry['value'] for entry in unique.values()] + anonymous
    ledger_calls = ledger.get('calls', {})
    groups, ledger_scopes, joins = {}, {}, []
    for call in rows:
        price = call.get('price') or {}
        price_identity = {'model': call.get('model'), **{k: price.get(k) for k in ('currency', 'input_per_million', 'output_per_million', 'region', 'input_bracket', 'source', 'date')}}
        rate_class = json.dumps(price_identity, sort_keys=True, ensure_ascii=False)
        group_key = (call.get('identity', {}).get('population'), call.get('identity', {}).get('configuration'), call.get('role'), call.get('stage'), call.get('record_kind'), rate_class)
        if group_key not in groups:
            groups[group_key] = {'records': 0, 'known': Decimal(0), 'known_records': 0, 'unknown_records': 0, 'price': price_identity}
        group = groups[group_key]
        group['records'] += 1
        amount = decimal(call.get('cost_usd'))
        if amount is None:
            group['unknown_records'] += 1
        else:
            group['known'] += amount
            group['known_records'] += 1
        uuid = call.get('call_uuid')
        entry = ledger_calls.get(uuid) if uuid is not None else None
        join = {'call_uuid': uuid, 'call': call, 'ledger': entry, 'status': 'JOINED' if entry is not None else 'NO_LEDGER_RECORD'}
        if entry is None:
            if call.get('record_kind') == 'LOCAL_MODEL' or call.get('actual_api_sent') is False and call.get('cost_state') == 'NO_REQUEST_SENT_NO_API_CHARGE':
                join['reason'] = 'RECORDED_NON_RESERVING_OR_UNSENT_CLASS'
            else:
                discrepancies.append({'kind': 'UNMATCHED_CALL_UUID', 'call_uuid': uuid, 'record_kind': call.get('record_kind')})
        else:
            state, ledger_amount = entry.get('state'), decimal(entry.get('amount_usd'))
            if state in ('RESERVED', 'UNKNOWN') and amount is not None:
                discrepancies.append({'kind': 'TRACE_KNOWN_LEDGER_HELD_STATE_DISAGREEMENT', 'call_uuid': uuid, 'trace_amount': call.get('cost_usd'), 'ledger': entry})
            if state not in ('RESERVED', 'UNKNOWN') and amount is None:
                discrepancies.append({'kind': 'LATE_LEDGER_SETTLEMENT_ABSENT_FROM_TRACE', 'call_uuid': uuid, 'trace': call.get('cost_state'), 'ledger': entry})
            elif state not in ('RESERVED', 'UNKNOWN') and ledger_amount != amount:
                discrepancies.append({'kind': 'AMOUNT_DISAGREEMENT', 'call_uuid': uuid, 'trace_amount': call.get('cost_usd'), 'ledger': entry})
            if entry.get('scope_root') != call.get('scope_root'):
                discrepancies.append({'kind': 'SCOPE_DISAGREEMENT', 'call_uuid': uuid, 'call_scope': call.get('scope_root'), 'ledger_scope': entry.get('scope_root')})
        joins.append(join)
    for uuid, entry in ledger_calls.items():
        scope = entry.get('scope_root')
        if not scope:
            discrepancies.append({'kind': 'LEDGER_SCOPE_MISSING', 'call_uuid': uuid})
        group = ledger_scopes.setdefault(scope, {'known': Decimal(0), 'reserved': Decimal(0), 'unknown': Decimal(0), 'invalid_amount_records': [], 'states': Counter()})
        group['states'][entry.get('state')] += 1
        amount = decimal(entry.get('amount_usd'))
        if amount is None:
            group['invalid_amount_records'].append(uuid)
        else:
            bucket = 'reserved' if entry.get('state') == 'RESERVED' else 'unknown' if entry.get('state') == 'UNKNOWN' else 'known'
            group[bucket] += amount
        if scope == formal_root and uuid not in unique:
            discrepancies.append({'kind': 'UNMATCHED_FORMAL_LEDGER_UUID', 'call_uuid': uuid, 'ledger': entry})
    totals = {k: sum((g[k] for g in ledger_scopes.values()), Decimal(0)) for k in ('known', 'reserved', 'unknown')}
    cap = decimal(ledger.get('limit_usd'))
    if cap != Decimal('40'):
        discrepancies.append({'kind': 'MISSING_OR_CHANGED40USD_CAP', 'recorded': ledger.get('limit_usd')})
    held = sum(totals.values())
    groups_output = []
    for key, group in groups.items():
        pop, cfg, role, stage, kind, _ = key
        price = group['price']
        groups_output.append({'population': pop, 'configuration': cfg, 'role': role, 'stage': stage, 'record_kind': kind,
            'price_identity': price, 'record_count': group['records'], 'known_cost_usd': str(group['known']) if group['known_records'] else None,
            'unknown_record_count': group['unknown_records'], 'different_or_missing_fixed_comparison_rate': kind == 'ACTUAL_CLOUD_DISPATCH' and (decimal(price['input_per_million']) != Decimal('0.4') or decimal(price['output_per_million']) != Decimal('1.6'))})
    scope_output = {str(k): {**{field + '_usd': str(v[field]) for field in ('known', 'reserved', 'unknown')}, 'states': dict(v['states']), 'invalid_amount_records': v['invalid_amount_records'], 'scope_class': 'UNRESOLVED_MISSING_SCOPE' if not k else 'CURRENT_FORMAL' if k == formal_root else 'OTHER_PRIOR_OR_EXTERNAL_SCOPE'} for k, v in ledger_scopes.items()}
    formal_rows = [c for c in rows if c.get('scope_root') == formal_root]
    formal_known = sum((decimal(c.get('cost_usd')) or Decimal(0) for c in formal_rows), Decimal(0))
    formal_ledger = ledger_scopes.get(formal_root)
    return {'unique_formal_call_known_usd': str(formal_known),
        'formal_call_minus_ledger_known_residual_usd': str(formal_known - formal_ledger['known']) if formal_ledger else None,
        'calls': rows, 'uuid_joins': joins, 'groups': groups_output, 'ledger_scopes': scope_output,
        'actual_cloud_dispatch_count': sum(c.get('actual_api_sent') is True for c in rows), 'local_record_count': sum(c.get('record_kind') == 'LOCAL_MODEL' for c in rows),
        'input_record_count': len(calls), 'deduplicated_record_count': len(rows), 'disposition_annotation_disagreements': annotation_disagreements, 'conflicts': conflicts, 'discrepancies': discrepancies,
        'global': {**{k + '_usd': str(v) for k, v in totals.items()}, 'known_plus_RESERVED_plus_UNKNOWN_usd': str(held),
                   'cap_usd': decimal_text(cap), 'remaining_reservation_aware_usd': str(cap - held) if cap is not None else None},
        'reconciliation_complete': not conflicts and not discrepancies and cap == Decimal('40') and totals['reserved'] == 0 and totals['unknown'] == 0 and not any(g['invalid_amount_records'] for g in ledger_scopes.values()),
        'accounting': 'Decimal arithmetic over recorded decimal representations; UNKNOWN/RESERVED are held amounts, not settled spend. No repricing or guessed episode allocation.'}


def prediction_cells(registrations, provenance):
    cells = []
    for dp, alternatives in PAIRS.items():
        for qa in ('qa01', 'qa02', 'qa03', 'qa04', 'qa05', 'qa06'):
            supplied = registrations.get((dp, qa), {})
            cells.append({'dp': dp, 'qa': qa, 'alternatives': alternatives,
                'preregistered_statement_ref': supplied.get('preregistered_statement_ref', provenance.get('preregistered_source_refs')),
                'prior_direction_or_NO_PRIOR_RANKING': supplied.get('prior_direction', 'UNRESOLVED_PREREGISTRATION_REVIEW'),
                'favorable_conditions': supplied.get('favorable_conditions'), 'counterconditions': supplied.get('counterconditions'),
                'fixed_population_summary_refs': [{'population': pop, 'file': 'report.json', 'dp': dp, 'qa': qa} for pop in POPULATIONS],
                'case_rep_refs': {'file': 'case-repetition.jsonl'}, 'certified_pair_refs': {'file': 'paired.jsonl', 'dp': dp, 'qa': qa},
                'raw_pair_refs': {'file': 'report.json', 'field': 'pairs.raw_analyzer_paired_diagnostics'},
                'actual_mechanism_refs': provenance.get('mechanism_refs'), 'semantic_review_coverage_refs': provenance.get('semantic_review_refs'),
                'limitations': LIMITATIONS, 'judgment_status': 'UNREVIEWED', 'judgment': None, 'reviewer': None,
                'reason': 'Independent Astra interpretation pending; absent registration/activation evidence is explicit, never inferred from numeric sign.'})
    return cells


def read_record_artifact(reader, item, artifact, record):
    """Unavailable evidence is not a hash/identity contradiction."""
    reference = item.get(artifact + '_ref')
    if not reference:
        record['unavailable_evidence'].append({'artifact': artifact, 'reason': 'NO_DECLARED_REFERENCE', 'ref': reference})
        return None
    try:
        value = reader.read(reference)
        require(isinstance(value, dict), 'Expected object artifact: ' + artifact)
        return value
    except OSError as error:
        record['unavailable_evidence'].append({'artifact': artifact, 'reason': 'REFERENCED_EVIDENCE_UNAVAILABLE',
                                              'error_type': type(error).__name__, 'detail': str(error), 'ref': reference})
    except (InputError, KeyError, ValueError, TypeError) as error:
        record['conflicts'].append({'artifact': artifact, 'detail': str(error), 'ref': reference})
    return None


def load_inputs(manifest_path, manifest_sha256, *, expected_freeze=FREEZE_SHA, expected_approval=APPROVAL_SHA, context=None):
    """Supported final-input ABI is explicit; no actual collection/discovery."""
    manifest_path = Path(manifest_path)
    require(manifest_path.stat().st_size <= 2 * 1024 * 1024, 'Oversized input manifest')
    data = manifest_path.read_bytes()
    require(sha(data) == manifest_sha256, 'Input manifest hash mismatch')
    supplied = json.loads(data)
    require(isinstance(supplied, dict), 'Input manifest must be an object')
    require(supplied.get('version') == 'JOINT441_QUANTITATIVE_INPUT_v1', 'Unsupported input manifest ABI')
    declared_root = Path(supplied['formal_root'])
    require(declared_root.is_absolute() and '..' not in declared_root.parts and not any(p.is_symlink() for p in (declared_root, *declared_root.parents)), 'Unsafe formal root')
    root = declared_root.resolve()
    require(manifest_path.is_absolute() and manifest_path.resolve().is_relative_to(root) and not any(p.is_symlink() for p in (manifest_path, *manifest_path.parents)), 'Input manifest outside exact root or symlinked')
    reader = ArtifactReader(root, **supplied.get('read_limits', {}))
    refs = supplied['artifacts']
    require(refs['freeze']['path'] == str(root / 'freeze.json') and refs['freeze']['sha256'] == expected_freeze, 'Wrong exact formal freeze')
    require(refs['approval']['path'] == str(root / 'approval/APPROVAL-BINDING.json') and refs['approval']['sha256'] == expected_approval, 'Wrong exact approval binding')
    freeze = reader.read(refs['freeze'])
    require(freeze.get('version') == 'COMMON9_PLUS_SUPPLEMENT6_JOINT441_v1', 'Wrong freeze version')
    approval = reader.read(refs['approval'])
    require(approval.get('formal_freeze_sha256') == refs['freeze']['sha256'], 'Approval bound another freeze')
    q = reader.read(refs['quiescence'])
    require(q.get('version') == 'JOINT441_QUIESCENT_SNAPSHOT_v1' and q.get('controller_state') in ('STOPPED', 'COMPLETED')
            and q.get('no_active_episode_processes') is True and q.get('evidence_refs'), 'Quiescence not independently established; no corpus read allowed')
    for name, original in [('progress', str(root / 'campaign-progress.jsonl')), ('status', str(root / 'status.json')), ('ledger', freeze['ledger_path'])]:
        require(refs[name].get('snapshot_of') == original and refs[name]['path'] != original, 'Explicit immutable snapshot required: ' + name)
        require(q.get(name + '_snapshot_sha256') == refs[name]['sha256'], 'Quiescence snapshot association differs: ' + name)
    for reference in q['evidence_refs']:
        reader.read(reference, raw=True)
    progress = reader.read(refs['progress'], jsonl=True)
    terminal = reader.read(refs['status'])
    ledger = reader.read(refs['ledger'])
    require(terminal.get('planned') == 441 and type(q.get('progress_complete')) is bool, 'Unsupported terminal/progress snapshot ABI')
    q = dict(q, controller_stopped=True)
    require(len(freeze.get('sources', [])) == 112 and len({e['relativePath'] for e in freeze['sources']}) == 112, 'Frozen112 source inventory differs')
    source_map = {entry['relativePath']: entry for entry in freeze['sources']}
    oracles, oracle_refs = {}, {}
    for pop, relative in [('COMMON9', 'benchmark/architecture/common9_poc_20261019/oracles.json'),
                          ('COMMON_SUPPLEMENT6', 'benchmark/architecture/common_supplement6_20261019/oracles.json')]:
        reference = supplied['oracles'][pop]
        expected = source_map[relative]
        require(reference['path'] == expected['frozen_path'] and reference['sha256'] == expected['sha256'], 'Wrong population oracle path/hash')
        oracles[pop], oracle_refs[pop] = reader.read(reference), reference
    jobs = freeze['jobs']
    planned_keys = set(validate_plan(jobs, oracles))
    if context is not None:
        context.update(jobs=jobs, oracles=oracles, inputs=list(reader.inputs.values()))
    # Hash the explicit frozen source list only; no directory or weight scan.
    for entry in freeze['sources']:
        path = Path(entry['frozen_path'])
        require(path == root / 'frozen/source' / entry['relativePath'], 'Frozen source root association mismatch')
        reference = {'path': str(path), 'sha256': entry['sha256'], 'bytes': path.stat().st_size}
        reader.read(reference, raw=True)
    index = reader.read(refs['result_index'], jsonl=True)
    require(len(index) <= 882, 'Result index exceeds bounded441 plan plus conflict diagnostics')
    records, calls, issues = [], [], []
    for item in index:
        require(isinstance(item, dict) and isinstance(item.get('identity'), dict), 'Result index item/identity must be objects')
        require(all(item.get(field) is None or isinstance(item[field], dict) for field in ('result_ref','trace_ref','association_ref')), 'Result index references must be objects or null')
        record = {'identity': item['identity'], 'run_id': item.get('run_id'), 'result_ref': item.get('result_ref'),
                  'trace_ref': item.get('trace_ref'), 'result': {}, 'provenance_valid': False, 'conflicts': [], 'unavailable_evidence': []}
        if identity(item['identity']) not in planned_keys:
            records.append(record)
            continue
        run_id = item.get('run_id')
        if item.get('trace_ref') and (not isinstance(run_id, str) or not run_id or run_id in ('.','..') or '/' in run_id
                or item['trace_ref'].get('path') != str(root / 'raw/runs' / run_id / 'trace.json')):
            record['conflicts'].append({'artifact': 'trace', 'detail': 'Trace path/run association mismatch before availability check', 'ref': item.get('trace_ref')})
        result = read_record_artifact(reader, item, 'result', record)
        if result is not None:
            record['result'] = result
        trace = read_record_artifact(reader, item, 'trace', record)
        if trace is not None:
            try:
                key = identity(item['identity'])
                require(type(trace.get('rep')) is int and all(trace.get(k) == v for k, v in [('configuration', key[1]), ('episode', key[2]), ('rep', key[3]), ('run_id', item['run_id']), ('phase', 'FORMAL')]), 'Mixed trace phase/identity')
                require(trace.get('binary_sha256') == freeze['binary']['sha256'], 'Mixed trace binary generation')
                expected_path = root / 'raw/runs' / item['run_id'] / 'trace.json'
                require(item['trace_ref']['path'] == str(expected_path) and Path(trace['candidate_input_file']).parent == expected_path.parent, 'Trace path/root association mismatch')
                require(type(trace.get('returncode')) is int and type(trace.get('end_ns')) is int, 'Unclosed trace cannot be an observed final result')
                record['raw_trace_facts'] = {k: trace.get(k) for k in ('returncode', 'host_failures', 'technical_failures', 'technical_abort_reason', 'unprovided_mandatory_events', 'cancellation_requested_ns', 'effective_admission_stop_ns')}
                for i, call in enumerate(trace.get('calls', [])):
                    calls.append(project_call(call, item['identity'], item['run_id'], {'file': item['trace_ref'], 'json_pointer': '/calls/' + str(i)}))
            except (InputError, KeyError, ValueError, TypeError) as error:
                record['conflicts'].append({'artifact': 'trace', 'detail': str(error), 'ref': item.get('trace_ref')})
            del trace
        association = read_record_artifact(reader, item, 'association', record)
        if association is not None and result is not None and not record['conflicts'] and not any(e['artifact'] in ('trace', 'result') for e in record['unavailable_evidence']):
            try:
                record['provenance_valid'] = validate_association(association, record, freeze, refs['freeze'], oracle_refs[item['identity']['population']], root)
                record['association_ref'] = item['association_ref']
            except (InputError, KeyError, ValueError, TypeError) as error:
                record['conflicts'].append({'artifact': 'association', 'detail': str(error), 'ref': item.get('association_ref')})
        records.append(record)
    terminated = {identity(row) for row in progress if row.get('state') == 'TERMINATED'}
    if terminal.get('completed') != len(terminated):
        issues.append({'kind': 'TERMINAL_PROGRESS_COUNT_MISMATCH', 'terminal': terminal.get('completed'), 'unique_terminated': len(terminated)})
    if refs.get('global_call_snapshot'):
        require(refs['global_call_snapshot']['path'] != str(root / 'model-calls.jsonl'), 'Use immutable global-call snapshot, not the live log')
        for i, call in enumerate(reader.read(refs['global_call_snapshot'], jsonl=True)):
            raw_identity = {k: call.get(k) for k in ('population', 'configuration', 'episode', 'rep')}
            if raw_identity['population'] is None:
                matches = [k for k in planned_keys if k[1:] == (call.get('configuration'), call.get('episode'), call.get('rep'))]
                require(len(matches) == 1, 'Cannot associate global call with exact frozen identity')
                raw_identity = key_object(matches[0])
            calls.append(project_call(call, raw_identity, call.get('run_id'), {'file': refs['global_call_snapshot'], 'jsonl_line': i + 1}))
    summaries, mechanisms, reviews, registrations = {}, {}, {}, {}
    for item in reader.read(refs['summary_index'], jsonl=True) if refs.get('summary_index') else []:
        key = (item['population'], item['configuration'])
        require(key not in summaries, 'Duplicate frozen summary identity')
        summaries[key] = dict(reader.read(item['ref']), source_ref=item['ref'])
    for item in reader.read(refs['mechanism_index'], jsonl=True) if refs.get('mechanism_index') else []:
        key = identity(item['identity'])
        require(key in planned_keys and key not in mechanisms, 'Duplicate/unplanned mechanism annotation')
        mechanisms[key] = {'value': reader.read(item['ref']), 'source_ref': item['ref'], 'interpretation': 'Frozen diagnostic only; not an automatic prediction judgment'}
    for item in reader.read(refs['semantic_review_index'], jsonl=True) if refs.get('semantic_review_index') else []:
        key = (*identity(item['identity']), item['qa'], item['obligation_id'])
        pop, _, case, _ = identity(item['identity'])
        require(identity(item['identity']) in planned_keys and item['qa'] in QA_FIELDS and key not in reviews, 'Duplicate/unplanned semantic annotation')
        group, field, _ = QA_FIELDS[item['qa']]
        require(item['obligation_id'] in {r[field] for r in oracles[pop][case][group]}, 'Unplanned semantic obligation')
        dimensions = {}
        for dimension, reference in item['dimension_refs'].items():
            value = reader.read(reference)
            require(value.get('status') in ('PASS', 'FAIL', 'UNREVIEWED', 'NOT_APPLICABLE'), 'Unsupported semantic review state')
            dimensions[dimension] = dict(value, source_ref=reference)
        reviews[key] = dimensions
    for item in reader.read(refs['registrations']) if refs.get('registrations') else []:
        key = (item['dp'], item['qa'])
        require(key[0] in PAIRS and key[1] in ('qa01', 'qa02', 'qa03', 'qa04', 'qa05', 'qa06') and key not in registrations, 'Duplicate/unplanned preregistration cell')
        registrations[key] = item
    raw_pairs = reader.read(refs['raw_pairs']) if refs.get('raw_pairs') else None
    qa04 = approved_qa04(reader, supplied['qa04'], approval)
    analysis_totals = dict(reader.read(refs['frozen_analysis_totals']), source_ref=refs['frozen_analysis_totals']) if refs.get('frozen_analysis_totals') else None
    provenance = {'formal_root': str(root), 'input_manifest': {'path': str(manifest_path.resolve()), 'sha256': manifest_sha256, 'bytes': len(data)},
        'freeze': refs['freeze'], 'approval': refs['approval'], 'quiescence': refs['quiescence'], 'ledger': refs['ledger'],
        'mechanism_refs': refs.get('mechanism_index'), 'semantic_review_refs': refs.get('semantic_review_index'),
        'preregistered_source_refs': [entry for relative, entry in source_map.items() if relative.endswith(('19-common9-evaluation-contract.md', '20-common-supplement-evaluation-contract.md'))],
        'inputs': list(reader.inputs.values()), 'bytes_read': reader.bytes_read,
        'read_limits': {'file_bytes': reader.file_limit, 'total_bytes': reader.total_limit},
        'spec_sha256': SPEC_SHA, 'block_control_records': [r for r in progress if r.get('state') in ('BLOCK_ADMITTED', 'BLOCK_BUDGET_STOP')],
        'frozen_analysis_totals': analysis_totals, 'missing_optional_summary_index': not bool(refs.get('summary_index'))}
    return dict(jobs=jobs, oracles=oracles, records=records, progress=progress, quiescence=q, calls=calls, ledger=ledger,
                qa04=qa04, summaries=summaries, raw_pairs=raw_pairs, reviews=reviews, mechanisms=mechanisms,
                registrations=registrations, provenance=provenance, external_issues=issues)


def display(value, digits=3):
    if value is None:
        return '—'
    if type(value) in (int, float):
        return str(value) if type(value) is int else format(value, '.' + str(digits) + 'f')
    return str(value).replace('|', '\\|').replace('\n', ' ')


def render_markdown(report):
    lines = ['# Joint441 quantitative evidence report', '', '**' + report['status'] + '**', '',
             'Reporting-only projection of bound frozen results. Input coverage is separate from semantic correctness. All24 prediction judgments remain UNREVIEWED.', '']
    for population in report['populations']:
        lines += ['## ' + population['population'], '',
                  '| Configuration | QA01 ms (observed/planned) | QA02 correct/planned; complete accuracy | QA03 ms (observed/planned) | QA04 modules | QA05 complete bundle USD; known spent | QA06 MB; status |',
                  '| --- | --- | --- | --- | --- | --- | --- |']
        for row in population['configurations']:
            t1, b, t3, cost, rss = (row[q] for q in ('qa01', 'qa02', 'qa03', 'qa05', 'qa06'))
            qa04 = report['qa04_reference']
            lines.append('| ' + ' | '.join([row['configuration'],
                f"{display(t1['mean_ms'])} ({t1['observed_count']}/{t1['planned']})",
                f"{b['known_correct']}/{b['planned']}; {display(b['fixed_denominator_accuracy'])}",
                f"{display(t3['mean_ms'])} ({t3['observed_count']}/{t3['planned']})",
                '1.833 (11/6)' if qa04.get('status') == 'FINAL_ACCEPTED42_CHANGES630_REGRESSIONS' else 'UNAVAILABLE',
                f"{display(cost['representative_fixed_bundle_mean_usd'])}; {display(cost['known_expenditure_usd'])}",
                f"{display(rss['decimal_MB'])}; {rss['maximum_status']}"]) + ' |')
        lines += ['', '### Coverage and failures', '', '| Configuration | Episode states | QA01 partition | QA02 partition | QA03 partition | RSS states |', '| --- | --- | --- | --- | --- | --- |']
        for row in population['configurations']:
            lines.append('| ' + ' | '.join([row['configuration'], *[display(json.dumps(v, sort_keys=True)) for v in [row['execution_partition'], row['qa01']['partition'], row['qa02']['partition'], row['qa03']['partition'], row['qa06']['status_counts']]]]) + ' |')
        lines += ['', 'QA01: first substantive response host-callback latency. QA03: frozen causal VIA processing time. Observed-subset means are not whole-population success speed. QA02 incomplete correct/planned is a lower bound, not complete accuracy.', '',
                  'QA05 compares a complete fixed9-case or6-case bundle separately; partial known expenditure is retained. QA06 is an observed same-sample local process-tree maximum; incomplete coverage is a lower bound. Decimal MB and MiB differ.', '']
    lines += ['## Paired and condition evidence', '', 'All8 alternative pairs retain all planned same-obligation rows. Only both-certified observed timing intersections carry a delta. B/C is computed directly; AAAA is reused without duplicated spending. Raw analyzer pairs remain separate diagnostics.', '',
              'All15 cases, all repetitions, all7 configurations, and all6 supplemental conditions remain in case-repetition.jsonl, including missing/failed activation. No receipt proxy becomes semantic adoption or a free-worker proof.', '',
              '## Financial reconciliation', '',
              '| Global known USD | RESERVED USD | UNKNOWN USD | Held total USD | Remaining USD | Complete |', '| --- | --- | --- | --- | --- | --- |']
    cost = report['cost_reconciliation']
    g = cost['global']
    lines.append('| ' + ' | '.join(display(g.get(k)) for k in ['known_usd', 'reserved_usd', 'unknown_usd', 'known_plus_RESERVED_plus_UNKNOWN_usd', 'remaining_reservation_aware_usd']) + ' | ' + str(cost['reconciliation_complete']) + ' |')
    lines += ['', f"Recorded call rows: {cost['input_record_count']}; deduplicated rows: {cost['deduplicated_record_count']}; actual cloud dispatches: {cost['actual_cloud_dispatch_count']}; local records: {cost['local_record_count']}.", '',
              'Full UUID joins, rate classes, other scopes, late settlements and residuals are in report.json/calls.jsonl. UNKNOWN/RESERVED are held amounts, not settled spend. Local API0 is not zero compute cost.', '',
              '## Input issues', '', f"{len(report['input_issues'])} input issues; see the exact retained structured records in report.json.", '',
              '## Interpretation and limitations', '']
    lines += ['- ' + text for text in report['limitations']]
    lines += ['', 'Semantic annotations, including actual-request/oracle disagreements and NOT_APPLICABLE authority cases, never change a frozen metric or pairing denominator. All24 prediction judgments require independent review.', '']
    return '\n'.join(lines)


def json_bytes(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2, allow_nan=False) + '\n').encode()


def validate_report_shape(report, rows):
    require(report.get('version') == VERSION and report.get('status') in ('COMPLETE_INPUT_REPORT', 'INCOMPLETE_EVIDENCE_REPORT', 'INVALID_REPORT_INPUT'), 'Unsupported report ABI')
    require([p['population'] for p in report['populations']] == list(POPULATIONS), 'Report populations differ')
    for pop in report['populations']:
        require([c['configuration'] for c in pop['configurations']] == list(CONFIGS), 'Report configuration order/membership differs')
        require(pop['case_repetition_refs'] == {'file': 'case-repetition.jsonl', 'population': pop['population']}, 'Population file reference differs')
        for cfg in pop['configurations']:
            for qa in QA_FIELDS:
                require(cfg[qa]['planned'] == POPULATIONS[pop['population']][qa] and sum(cfg[qa]['partition'].values()) == cfg[qa]['planned'], 'Report denominator/partition differs')
    require(len(rows['planned-episodes.jsonl']) == 441 and len({identity(r['identity']) for r in rows['planned-episodes.jsonl']}) == 441, 'Report plan incomplete/duplicate')
    require(len(rows['obligations.jsonl']) == 3381 and len({(*identity(r['identity']), r['qa'], r['obligation_id']) for r in rows['obligations.jsonl']}) == 3381, 'Report obligations incomplete/duplicate')
    predictions = report['prediction_worklist']
    require(len(predictions) == 24 and {(p['dp'], p['qa']) for p in predictions} == {(dp, qa) for dp in PAIRS for qa in ('qa01','qa02','qa03','qa04','qa05','qa06')}, 'Prediction worklist differs')
    require(all(p['judgment_status'] == 'UNREVIEWED' and p['judgment'] is None and p['reviewer'] is None for p in predictions), 'Automatic prediction judgment forbidden')
    paired = rows['paired.jsonl']
    require(len(paired) == 4872 and report['pairs']['planned_rows'] == 4872 and len({(p['population'],p['dp'],p['left'],p['right'],p['episode'],p['rep'],p['qa'],p['obligation_id']) for p in paired}) == 4872, 'Planned pair rows differ')
    bundles = [r for r in rows['case-repetition.jsonl'] if r['kind'] == 'FIXED_REPETITION_BUNDLE']
    require(len(bundles) == 56 and len({(b['population'],b['configuration'],b['rep']) for b in bundles}) == 56, 'Fixed repetition bundles differ')
    for bundle in bundles:
        rule = POPULATIONS[bundle['population']]
        require(all(bundle[q]['planned'] == rule[q] // rule['reps'] and sum(bundle[q]['partition'].values()) == bundle[q]['planned'] for q in QA_FIELDS), 'Repetition bundle obligation partitions differ')


def write_outputs(output, report, rows, command):
    """Exclusive new directory, all bytes prepared first, manifest written last."""
    output = Path(output)
    require(output.is_absolute() and '..' not in output.parts and not any(p.is_symlink() for p in (output, *output.parents)), 'Unsafe output path')
    require(not output.exists(), 'Immutable output already exists')
    validate_report_shape(report, rows)
    payloads = {'report.json': json_bytes(report), 'prediction-worklist.json': json_bytes(report['prediction_worklist']),
                'report.md': render_markdown(report).encode()}
    for name, values in rows.items():
        require(name in ('planned-episodes.jsonl', 'obligations.jsonl', 'case-repetition.jsonl', 'paired.jsonl', 'calls.jsonl'), 'Unsupported output filename')
        payloads[name] = b''.join((json.dumps(value, ensure_ascii=False, sort_keys=True, allow_nan=False) + '\n').encode() for value in values)
    source_paths = [Path(__file__), *[Path(__file__).with_name(name) for name in ('report_adapter.schema.json', 'report_adapter.input.schema.json')]]
    sources = [{'path': str(path.resolve()), 'sha256': sha(path.read_bytes()), 'bytes': path.stat().st_size} for path in source_paths]
    outputs = {name: {'sha256': sha(data), 'bytes': len(data)} for name, data in payloads.items()}
    manifest = {'version': 'JOINT441_QUANTITATIVE_MANIFEST_v1', 'report_version': VERSION, 'status': report['status'],
                'approved_spec_sha256': SPEC_SHA, 'command': command, 'sources': sources, 'inputs': report['provenance'].get('inputs', []),
                'input_manifest': report['provenance'].get('input_manifest'), 'outputs': outputs, 'manifest_written_last': True,
                'scope': 'Quiescent reporting only; no evaluator/model/candidate/parser/network execution; judgments UNREVIEWED',
                'warnings': report['limitations'], 'input_issue_count': len(report['input_issues'])}
    output.mkdir(parents=True, exist_ok=False)
    for name, data in payloads.items():
        with (output / name).open('xb') as handle:
            handle.write(data)
    with (output / 'manifest.json').open('xb') as handle:
        handle.write(json_bytes(manifest))
    return manifest


def main():
    import sys
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input-manifest', type=Path, required=True)
    parser.add_argument('--input-sha256', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    context = {}
    try:
        report, rows = build_report(**load_inputs(args.input_manifest, args.input_sha256, context=context))
    except (InputError, OSError, ValueError, KeyError, TypeError, AttributeError) as error:
        if 'jobs' in context and 'oracles' in context:
            report, rows = build_report(context['jobs'], context['oracles'], [], [], {},
                external_issues=[{'kind': 'INPUT_REJECTED', 'message': str(error)}],
                provenance={'input_manifest': {'path': str(args.input_manifest), 'sha256': args.input_sha256},
                            'inputs': context.get('inputs', []), 'planned_rows_preserved_without_execution_inference': True})
        else:
            # Before a verified plan is available, do not invent jobs or oracle fields.
            require(not args.output.exists(), 'Immutable output already exists')
            require(args.output.is_absolute() and '..' not in args.output.parts and not any(p.is_symlink() for p in (args.output, *args.output.parents)), 'Unsafe diagnostic output path')
            args.output.mkdir(parents=True, exist_ok=False)
            diagnostic = {'status': 'INVALID_REPORT_INPUT', 'reason': str(error), 'planned_rows': 'UNAVAILABLE_BEFORE_VERIFIED_PLAN',
                          'input_manifest': str(args.input_manifest), 'input_sha256': args.input_sha256,
                          'command': sys.argv, 'tool_sha256': sha(Path(__file__).read_bytes()), 'spec_sha256': SPEC_SHA}
            data = json_bytes(diagnostic)
            (args.output / 'input-rejection.json').write_bytes(data)
            with (args.output / 'manifest.json').open('xb') as handle:
                handle.write(json_bytes({'status': 'INVALID_REPORT_INPUT', 'outputs': {'input-rejection.json': {'sha256': sha(data), 'bytes': len(data)}}, 'manifest_written_last': True}))
            print(json.dumps({'status': 'INVALID_REPORT_INPUT', 'output': str(args.output)}))
            return 2
    write_outputs(args.output, report, rows, sys.argv)
    print(json.dumps({'status': report['status'], 'output': str(args.output)}))
    return 2 if report['status'] == 'INVALID_REPORT_INPUT' else 0


if __name__ == '__main__':
    raise SystemExit(main())
