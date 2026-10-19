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
              'raw630': '26cd58d883958a7e1b77e612c90acc299fb40d809de22cf6f506873bc74287ab'}
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
    return tuple(row.get(k) for k in ('population', 'configuration', 'episode', 'rep'))


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


def episode_state(records, progress, quiescence):
    if len(records) > 1:
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
    if not record.get('provenance_valid'):
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
    if isinstance(status, str) and status.startswith('UNRESOLVED') and value is None:
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
    unknown = [e['qa05'].get('unknown_call_count') for e in observed]
    complete = len(observed) == planned and all(e['state'] == 'CLOSED_CERTIFIED' for e in observed) and all(v is not None for v in values) and all(type(u) is int and u == 0 for u in unknown)
    return {'known_expenditure_usd': decimal_text(known) if any(v is not None for v in values) else None, 'unknown_call_count_observed': sum(u for u in unknown if type(u) is int) if any(type(u) is int for u in unknown) else None,
            'unknown_usage_records_missing': planned - len(observed) + sum(type(u) is not int for u in unknown), 'comparison_complete': complete,
            'representative_fixed_bundle_mean_usd': decimal_text(known / reps) if complete else None,
            'planned_episode_count': planned, 'observed_cost_records': len(observed), 'local_compute_monetized': 'NOT_MEASURED',
            'scope': 'All known expenditure, including failed/unused/discarded work where actually recorded; no timing/semantic inclusion filter.'}


def rss_summary(episodes, planned):
    values = [(e, e.get('qa06', {}).get('peak_concurrent_sum_bytes')) for e in episodes]
    valid = [(e, value) for e, value in values if finite(value)]
    peak = max((v for _, v in valid), default=None)
    tied = [{'identity': e['identity'], 'run_id': e.get('run_id'), 'result_ref': e.get('result_ref'),
             'peak_same_sample': e['qa06'].get('peak_same_sample')} for e, value in valid if value == peak]
    complete = len(episodes) == planned and all(e['state'] == 'CLOSED_CERTIFIED' and e.get('qa06', {}).get('status') == 'OBSERVED' for e in episodes)
    counters = ('sample_count', 'valid_sample_count', 'invalid_sample_count', 'confirmed_exit_boundary_count', 'startup_zero_rss_count', 'incomplete_registered_process_samples')
    return {'peak_concurrent_sum_bytes': peak, 'decimal_MB': peak / 1e6 if peak is not None else None,
            'MiB': peak / 1048576 if peak is not None else None, 'tied_same_sample_refs': tied, 'comparison_complete': complete,
            'maximum_status': 'COMPLETE_OBSERVED_MAXIMUM' if complete else 'OBSERVED_LOWER_BOUND_DIAGNOSTIC' if peak is not None else 'NOT_OBSERVED',
            'status_counts': dict(Counter(e.get('qa06', {}).get('status', 'MISSING') for e in episodes)),
            'counters': {k: sum(e.get('qa06', {}).get(k, 0) for e in episodes if type(e.get('qa06', {}).get(k)) is int) for k in counters},
            'max_sample_gap_ns': max((e.get('qa06', {}).get('max_sample_gap_ns') for e in episodes if exact_ns(e.get('qa06', {}).get('max_sample_gap_ns'))), default=None),
            'sampler_failure_records': [{'identity': e['identity'], 'records': e.get('qa06', {}).get('sampler_failures')} for e in episodes if e.get('qa06', {}).get('sampler_failures')],
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
        run_ids[record.get('run_id')].append(key)
    duplicate_runs = {run for run, values in run_ids.items() if run is None or len(values) > 1}
    for row in progress:
        if row.get('state') in ('BLOCK_ADMITTED', 'BLOCK_BUDGET_STOP'):
            continue
        key = identity(row)
        if key not in planned:
            issues.append({'kind': 'UNPLANNED_PROGRESS', 'record': row})
        else:
            by_progress[key].append(row.get('state'))
    for key, states in by_progress.items():
        if states.count('STARTED') > 1 or states.count('TERMINATED') > 1 or ('TERMINATED' in states and 'STARTED' not in states):
            issues.append({'kind': 'PROGRESS_IDENTITY_CONFLICT', 'identity': key_object(key), 'states': states})
    episode_rows, obligations = [], []
    for key in keys:
        pop, cfg, case, rep = key
        found = by_key[key]
        state = episode_state(found, by_progress[key], quiescence)
        if any(r.get('run_id') in duplicate_runs for r in found):
            state = 'IDENTITY_OR_SOURCE_CONFLICT'
        record = found[0] if len(found) == 1 else None
        result = record.get('result', {}) if record else {}
        if record and result and (result.get('phase') != 'FORMAL' or any(result.get(k) != v for k, v in [('configuration', cfg), ('episode', case), ('rep', rep)])):
            state = 'IDENTITY_OR_SOURCE_CONFLICT'
        if record and record.get('provenance_valid') and (result.get('version') != 'COMMON9-EVALUATOR-v3' or result.get('evaluation_mode') != 'strict'):
            state = 'IDENTITY_OR_SOURCE_CONFLICT'
        if state == 'IDENTITY_OR_SOURCE_CONFLICT':
            issues.append({'kind': state, 'identity': key_object(key), 'all_result_refs': [r.get('result_ref') for r in found]})
        ep = {'identity': key_object(key), 'state': state, 'run_id': record.get('run_id') if record else None,
              'trace_ref': record.get('trace_ref') if record else None, 'result_ref': record.get('result_ref') if record else None,
              'progress_states': by_progress[key], 'functional_failures': result.get('functional_failures'),
              'raw_trace_facts': record.get('raw_trace_facts') if record else None,
              'host_failures': result.get('host_failures'), 'returncode': result.get('returncode'),
              'qa05': result.get('qa05', {}), 'qa06': result.get('qa06', {}),
              'mechanism_annotation': (mechanisms or {}).get(key), 'source_conflicts': record.get('conflicts', []) if record else []}
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
            configurations.append(item)
            for rep in range(1, rule['reps'] + 1):
                fixed_bundle = [e for e in eps if e['identity']['rep'] == rep]
                case_rows.append({'kind': 'FIXED_REPETITION_BUNDLE', 'population': pop, 'configuration': cfg, 'rep': rep,
                                  'planned_cases': list(rule['cases']), 'qa05': cost_summary(fixed_bundle, len(rule['cases']), 1),
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
        value = value[int(part)] if isinstance(value, list) else value[part]
    return value


class ArtifactReader:
    """Explicit references only; never search a directory or import a source."""
    def __init__(self, root, file_limit=16 * 1024 * 1024, total_limit=8 * 1024 * 1024 * 1024):
        self.root = Path(root).resolve()
        require(0 < file_limit <= 64 * 1024 * 1024 and 0 < total_limit <= 16 * 1024 * 1024 * 1024, 'Unsupported read limits')
        self.file_limit, self.total_limit = file_limit, total_limit
        self.bytes_read, self.inputs = 0, {}

    def read(self, ref, *, jsonl=False, raw=False):
        name = ref['path']
        require(isinstance(name, str) and Path(name).is_absolute() and '..' not in Path(name).parts, 'File reference must be explicit absolute path without traversal')
        path = Path(name)
        require(path.resolve().is_relative_to(self.root), 'File reference outside exact formal root; restore mapping is unsupported')
        require(not any(p.is_symlink() for p in (path, *path.parents)), 'Symlink file/ancestor unsupported')
        before = path.stat()
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
            return [json.loads(line) for line in data.splitlines() if line.strip()]
        return get_pointer(json.loads(data), ref.get('json_pointer', ''))


def same_ref(a, b):
    return all(a.get(k) == b.get(k) for k in ('path', 'sha256', 'bytes', 'json_pointer'))


def validate_association(association, record, freeze, freeze_ref, oracle_ref, root):
    require(association.get('version') == 'JOINT441_RESULT_ASSOCIATION_v1', 'Unsupported association ABI; identity-only/partial envelopes are not certified')
    require(association.get('phase') == 'FORMAL' and association.get('mode') == 'strict' and association.get('evaluator_version') == 'COMMON9-EVALUATOR-v3', 'Wrong phase/mode/evaluator association')
    require(association.get('formal_root') == str(root) and association.get('formal_freeze_sha256') == freeze_ref['sha256'], 'Mixed root/freeze association')
    require(association.get('binary_sha256') == freeze['binary']['sha256'], 'Mixed binary association')
    require(identity(association['identity']) == identity(record['identity']) and association.get('run_id') == record['run_id'], 'Association identity differs')
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
    require(values['final_review']['status'].startswith('PASS') and values['final_review']['input_sha256']['report_json'] == refs['report']['sha256'], 'QA04 final acceptance missing')
    return {'status': 'FINAL_ACCEPTED42_CHANGES630_REGRESSIONS', 'artifacts': refs, 'counts': report['all_counts_by_configuration'],
            'numerator': 11, 'denominator': 6, 'mean': 11 / 6, 'observed_tie_scope': 'Six specified common changes only; not intrinsic production architecture equality.',
            'raw_counts_preserved': report.get('raw_counts_preserved'), 'regression_RSS_diagnostics_separate_from_formal_QA06': report.get('qa06_diagnostics')}
