"""Read-only evidence arrangement for one explicitly named closed seven-run block.

No evaluator import, semantic judgment, metric calculation, network or subprocess.
All newly created review dimensions remain UNREVIEWED.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re

CONFIGS = ('AAAA', 'BAAA', 'CAAA', 'ABAA', 'AABA', 'AAAB', 'AAAC')
VERSION = 'COMMON9_CLOSED_BLOCK_EVIDENCE_v2'
GENERATION = 'COMMON9_PLUS_SUPPLEMENT6_JOINT441_v1'
DIMENSIONS = ('body', 'task_revision', 'approval_authority', 'handoff')
MAX_TRACE = 16 * 1024 * 1024
MAX_TOTAL = 128 * 1024 * 1024
MAX_TEXT = 65536
CLOSURE_ABI = {
    'FIRST_FIXED7_CONFIGURATION_BLOCK_CLOSED_NOT_CAMPAIGN_COMPLETE':
        ('COMMON9', 'S02', 1, 'FIRST-BLOCK-S02-SUMMARY-v1.json'),
}
ORACLES = {
    'COMMON9': 'benchmark/architecture/common9_poc_20261019/oracles.json',
    'COMMON_SUPPLEMENT6': 'benchmark/architecture/common_supplement6_20261019/oracles.json',
}
REFERENCE_SOURCES = (
    'benchmark/architecture/common9_poc_20261019/evaluation_v3/evaluate.py',
    'benchmark/architecture/common_supplement6_20261019/analyze_joint.py',
    'docs/architecture/12-decisions/decision-packages/seven-problem-review/19-common9-evaluation-contract.md',
)


class EvidenceError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise EvidenceError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def pointer(parts):
    return ''.join('/' + str(p).replace('~', '~0').replace('/', '~1') for p in parts)


def inside(path, root):
    path = Path(path).resolve()
    require(path.is_relative_to(root), 'Reference outside exact root: ' + str(path))
    return path


class Reader:
    def __init__(self, root, max_trace=MAX_TRACE, max_total=MAX_TOTAL):
        self.root = root
        self.max_trace = max_trace
        self.max_total = max_total
        self.total_bytes = 0
        self.files = {}
        self.omissions = []

    def read(self, path, limit=4 * 1024 * 1024, parse=True):
        path = inside(path, self.root)
        stat = path.stat()
        require(stat.st_size <= limit, 'Input exceeds byte limit: ' + str(path))
        require(self.total_bytes + stat.st_size <= self.max_total, 'Block input byte limit exceeded')
        with path.open('rb') as handle:
            data = handle.read(limit + 1)
        require(len(data) <= limit, 'Input grew beyond byte limit')
        after = path.stat()
        require((stat.st_size, stat.st_mtime_ns) == (after.st_size, after.st_mtime_ns), 'Input changed while reading')
        item = {'path': str(path), 'sha256': digest(data), 'bytes': len(data)}
        if str(path) in self.files:
            require(self.files[str(path)] == item, 'Input changed between references')
        self.files[str(path)] = item
        self.total_bytes += len(data)
        return (json.loads(data) if parse else data), item

    def project(self, value, source, parts=()):
        """Keep plain evidence; every omission retains the unchanged raw pointer."""
        raw = {'file': source, 'json_pointer': pointer(parts)}
        if isinstance(value, str):
            reason = None
            if re.match(r'^data:[^,]*;base64,', value, re.I):
                reason = 'BASE64_DATA_URL_REFERENCE_ONLY'
            elif len(value) > 256 and re.fullmatch(r'[A-Za-z0-9+/\r\n]+={0,2}', value):
                reason = 'BASE64_LIKE_PAYLOAD_REFERENCE_ONLY'
            elif len(value) > MAX_TEXT:
                reason = 'OVERSIZED_TEXT_REFERENCE_ONLY'
            if reason:
                item = {'omitted': reason, 'raw_ref': raw, 'characters': len(value)}
                self.omissions.append(item)
                return item
            return value
        if isinstance(value, dict):
            result = {}
            for key, child in value.items():
                low = key.lower()
                if any(token in low for token in ('base64', 'image_url', 'screenshot', 'pixel', 'embedding')) or low in ('image', 'images', 'raw_bytes', 'binary'):
                    item = {'omitted': 'BINARY_OR_IMAGE_FIELD_REFERENCE_ONLY', 'raw_ref': {'file': source, 'json_pointer': pointer((*parts, key))}}
                    self.omissions.append(item)
                    result[key] = item
                else:
                    result[key] = self.project(child, source, (*parts, key))
            return result
        if isinstance(value, list):
            if len(value) > 256:
                item = {'omitted': 'OVERSIZED_ARRAY_REFERENCE_ONLY', 'raw_ref': raw, 'items': len(value)}
                self.omissions.append(item)
                return item
            return [self.project(child, source, (*parts, i)) for i, child in enumerate(value)]
        return value


def identity(value):
    return {key: value.get(key) for key in ('population', 'configuration', 'episode', 'rep')}


def unreviewed():
    return {dimension: {'status': 'UNREVIEWED', 'reason': 'Independent Astra review not yet supplied', 'reviewer': None, 'evidence_refs': []} for dimension in DIMENSIONS}


def event_ref(source, event, index):
    return {'file': source, 'json_pointer': pointer(('events', index)), 'event_index': index, 'event_id': event.get('event_id')}


def stimulus_links(event):
    """Only explicitly recorded, known typed fields; not inferred causality."""
    data = event.get('data', {})
    if not isinstance(data, dict):
        return []
    request = data.get('request', {})
    candidates = [(('data', 'stimulus_id'), data.get('stimulus_id')),
                  (('data', 'origin_event_id'), data.get('origin_event_id'))]
    if isinstance(request, dict):
        candidates.append((('data', 'request', 'stimulus_id'), request.get('stimulus_id')))
    if event.get('kind') == 'WORKFLOW_FAILED':
        candidates.append((('data', 'event_id'), data.get('event_id')))
    nested = data.get('data', {})
    if event.get('kind') == 'STATE_EFFECT_COMMIT' and isinstance(nested, dict):
        candidates.append((('data', 'data', 'stimulus_id'), nested.get('stimulus_id')))
    return [{'value': value, 'event_relative_pointer': pointer(parts), 'link_type': 'EXPLICIT_RECORDED_STIMULUS_FIELD'}
            for parts, value in candidates if isinstance(value, str)]


def stimulus_ids(event):
    return {link['value'] for link in stimulus_links(event)}


def typed_fact_refs(events, trace_ref):
    facts = []
    for index, event in enumerate(events):
        kind = event.get('kind', '')
        tags = []
        if kind.startswith('STATE_EFFECT_'):
            tags.append('RECORDED_STATE_EFFECT_LEDGER')
        if kind == 'WORKFLOW_FAILED' or kind.endswith('_FAILED'):
            tags.append('RECORDED_FAILURE_KIND')
        if 'REJECT' in kind:
            tags.append('RECORDED_REJECTION_KIND')
        if 'CANCEL' in kind:
            tags.append('RECORDED_CANCELLATION_KIND')
        if kind.startswith('PUBLICATION_'):
            tags.append('RECORDED_PUBLICATION_KIND')
        if 'AUTHORITY' in kind or 'PERMISSION' in kind:
            tags.append('RECORDED_AUTHORITY_OR_PERMISSION_KIND')
        if tags:
            facts.append({'raw_ref': event_ref(trace_ref, event, index), 'recorded_kind': kind,
                          'fact_types': tags, 'explicit_stimulus_links': stimulus_links(event),
                          'interpretation': 'Raw kind/field classification only; not causal, timely, authorized or successful effect proof.'})
    return facts


def build_bundle(root, closure, population, episode, rep, freeze_sha256, saved_index=None,
                 max_trace=MAX_TRACE, max_total=MAX_TOTAL):
    require(saved_index is None, 'UNSUPPORTED_SAVED_EVALUATION_PROVENANCE: this version does not accept saved evaluations, including new envelopes')
    root = Path(root).resolve()
    reader = Reader(root, max_trace, max_total)
    freeze, freeze_ref = reader.read(root / 'freeze.json')
    require(freeze_ref['sha256'] == freeze_sha256, 'Wrong frozen root/hash')
    require(freeze['version'] == GENERATION, 'Wrong generation')
    require(population in ORACLES and isinstance(rep, int) and rep > 0, 'Invalid explicit block identity')
    closed, closure_ref = reader.read(closure)
    supported = CLOSURE_ABI.get(closed.get('status'))
    require(supported is not None and supported[:3] == (population, episode, rep)
            and Path(closure).resolve() == root / supported[3]
            and closed.get('population') == population and closed.get('episode') == episode and closed.get('rep') == rep
            and closed.get('closed_episodes') == 7 and closed.get('planned_joint_episodes') == 441 and freeze.get('total_episode_runs') == 441
            and isinstance(closed.get('rows'), list), 'Unsupported closure status/path/shape; no closure proof')
    planned = [j for j in freeze['jobs'] if j.get('population') == population and j.get('episode') == episode and j.get('rep') == rep]
    require(len(planned) == 7 and Counter(j['configuration'] for j in planned) == Counter(CONFIGS), 'Planned block must contain all seven configurations exactly once')
    listed = {}
    run_ids = set()
    for row in closed.get('rows', []):
        cfg, run_id = row.get('configuration'), row.get('run_id')
        require(cfg in CONFIGS and cfg not in listed, 'Duplicate/unplanned configuration in closed block')
        require(isinstance(run_id, str) and re.fullmatch(r'[A-Za-z0-9_.-]+', run_id) and run_id not in run_ids, 'Duplicate/invalid run ID')
        for key, expected in (('population', population), ('episode', episode), ('rep', rep)):
            require(key not in row or row[key] == expected, 'Mixed block identity')
        listed[cfg] = row
        run_ids.add(run_id)
    source_entries = {e['relativePath']: e for e in freeze['sources']}
    source_refs = {}
    oracle = None
    extra_functional = None
    functional_relative = 'benchmark/architecture/common_supplement6_20261019/functional-oracles.json'
    references = (ORACLES[population], *REFERENCE_SOURCES, *((functional_relative,) if population == 'COMMON_SUPPLEMENT6' else ()))
    for relative in references:
        require(relative in source_entries, 'Frozen source reference missing: ' + relative)
        entry = source_entries[relative]
        path = root / 'frozen/source' / relative
        require(path.resolve() == Path(entry['frozen_path']).resolve(), 'Wrong-root frozen source reference')
        value, record = reader.read(path, parse=relative.endswith('.json'))
        require(record['sha256'] == entry['sha256'], 'Frozen source hash mismatch')
        source_refs[relative] = record
        if relative == ORACLES[population]:
            oracle = value[episode]
        if relative == functional_relative:
            extra_functional = {'file': record, 'status': 'DIRECT_EPISODE_RECORD' if episode in value else 'DIRECT_EPISODE_KEY_NOT_FOUND_NO_NORMALIZATION',
                                'value': reader.project(value[episode], record, (episode,)) if episode in value else None,
                                'json_pointer': pointer((episode,)) if episode in value else None}
    require(len({r['response_id'] for r in oracle['responses']}) == len(oracle['responses']), 'Duplicate frozen response obligations')
    require(len({r['id'] for r in oracle['goals']}) == len(oracle['goals']), 'Duplicate frozen goal obligations')
    worklist, evidence, executions, issues = [], [], [], []
    for cfg in CONFIGS:
        listed_row = listed.get(cfg)
        run_id = listed_row['run_id'] if listed_row else None
        trace, trace_ref, saved, saved_ref = {}, None, {}, None
        saved_state = 'MISSING_SAVED_EVALUATION'
        execution_state = 'MISSING_RUN_RECORD'
        if run_id:
            trace_path = root / 'raw/runs' / run_id / 'trace.json'
            if trace_path.is_file():
                trace, trace_ref = reader.read(trace_path, limit=max_trace)
                for key, expected in (('run_id', run_id), ('configuration', cfg), ('episode', episode), ('rep', rep), ('phase', 'FORMAL')):
                    require(trace.get(key) == expected, 'Mixed phase/configuration/identity in trace')
                require(trace.get('binary_sha256') == freeze['binary']['sha256'], 'Mixed binary generation')
                candidate = inside(trace['candidate_input_file'], root)
                require(candidate.parent == trace_path.parent.resolve(), 'Wrong-root candidate input reference')
                require(isinstance(trace.get('returncode'), int) and isinstance(trace.get('end_ns'), int), 'Trace is not closed')
                execution_state = 'CLOSED_TRACE_PRESENT'
                events = trace.get('events', [])
                evidence.append({'identity': dict(population=population, configuration=cfg, episode=episode, rep=rep, run_id=run_id),
                    'trace': trace_ref, 'events': [{'raw_ref': event_ref(trace_ref, e, i), 'record': reader.project(e, trace_ref, ('events', i))} for i, e in enumerate(events)],
                    'host_display_receipts': reader.project(trace.get('host_display_receipts', []), trace_ref, ('host_display_receipts',)),
                    'provided_sources': reader.project(trace.get('provided_sources', []), trace_ref, ('provided_sources',)),
                    'source_nodes': reader.project(trace.get('source_nodes', []), trace_ref, ('source_nodes',)),
                    'candidate_input': None, 'raw_model_files': [],
                    'typed_failure_effect_control_refs': typed_fact_refs(events, trace_ref),
                    'candidate_input_scope': 'SINGLE_INITIAL_JSON_OBJECT_ONLY; later streamed ingress/control remains separately in trace/raw evidence',
                    'prompt_context_state': 'RECORDED_RAW_REQUEST_FILE_REFERENCES_ONLY_NO_INFERRED_PROMPT'})
                if candidate.is_file():
                    initial, initial_ref = reader.read(candidate, limit=max_trace)
                    evidence[-1]['candidate_input'] = {'file': initial_ref, 'plain_evidence': reader.project(initial, initial_ref)}
                    del initial
                else:
                    issues.append({'configuration': cfg, 'run_id': run_id, 'reason': 'MISSING_RECORDED_CANDIDATE_INPUT'})
                for i, call in enumerate(trace.get('calls', [])):
                    for key, value in call.items():
                        if isinstance(value, str) and ('request' in key or 'response' in key) and key.endswith(('_file', '_path')):
                            path = inside(value, root)
                            record = {'recorded_by': {'file': trace_ref, 'json_pointer': pointer(('calls', i, key))}, 'path': str(path), 'status': 'MISSING_RECORDED_FILE'}
                            if path.is_file():
                                _, file_ref = reader.read(path, limit=max_trace, parse=False)
                                record.update(status='HASHED_REFERENCE_ONLY', file=file_ref)
                            evidence[-1]['raw_model_files'].append(record)
            else:
                execution_state = 'MISSING_TRACE'
        if execution_state != 'CLOSED_TRACE_PRESENT':
            issues.append({'configuration': cfg, 'run_id': run_id, 'reason': execution_state})
        events = trace.get('events', [])
        event_ids = Counter(e.get('event_id') for e in events if e.get('event_id'))
        facts = typed_fact_refs(events, trace_ref)
        executions.append({'identity': dict(population=population, configuration=cfg, episode=episode, rep=rep, run_id=run_id),
            'execution_state': execution_state, 'saved_evaluation_state': saved_state, 'trace': trace_ref, 'saved_evaluation': saved_ref,
            'recorded_technical_state': {k: reader.project(trace[k], trace_ref, (k,)) for k in ('technical_certification', 'technical_failures', 'technical_abort_reason', 'returncode', 'unprovided_mandatory_events') if k in trace},
            'saved_primary_measurement_eligibility': saved.get('primary_measurement_eligibility', 'MISSING_SAVED_EVALUATION'),
            'saved_deadline_audit_status': 'MISSING_SAVED_EVALUATION',
            'typed_failure_effect_control_refs': facts,
            'association_limit': 'Only known explicit stimulus fields are linked. Other run-level facts remain context, not inferred causes. Absence from a row timeline cannot establish absence from the trace.'})
        declarations = [('response', 'responses', r, r['response_id']) for r in oracle['responses']]
        declarations += [('goal_relation', 'goals', r, r['id']) for r in oracle['goals']]
        observed = [('observed_handoff_control', None, e, e.get('event_id') or 'raw-index-' + str(i)) for i, e in enumerate(events)
                    if e.get('kind') in ('AGENT_INGRESS', 'CONTROL_INGRESS')]
        for kind, group, expected, obligation_id in declarations + observed:
            index = oracle[group].index(expected) if group else None
            members = [expected['event_id']] if kind == 'response' else expected.get('member_events', []) if group else list(stimulus_ids(expected))
            linked = [(i, e) for i, e in enumerate(events) if stimulus_ids(e).intersection(members)]
            displays = [(i, e) for i, e in linked if e.get('kind') == 'DISPLAY_PROXY']
            adoptions = [(i, e) for i, e in linked if e.get('kind') == 'SEMANTIC_ADOPTION']
            usable_clocks = all(type(e.get('observed_ns')) is int for _, e in displays)
            ordered = sorted(displays, key=lambda pair: (pair[1].get('observed_ns') if type(pair[1].get('observed_ns')) is int else float('inf'), pair[0]))
            first = None
            first_status = 'NOT_OBSERVED' if execution_state == 'CLOSED_TRACE_PRESENT' else 'MISSING_RUN_EVIDENCE'
            if ordered:
                at = ordered[0][1].get('observed_ns')
                ambiguous = not usable_clocks or sum(e.get('observed_ns') == at for _, e in ordered) > 1 or event_ids[ordered[0][1].get('event_id')] > 1
                first_status = 'AMBIGUOUS_RAW_CALLBACK' if ambiguous else 'FIRST_RAW_MATCHING_DISPLAY_NOT_A_QA01_SELECTION'
                if not ambiguous:
                    first = event_ref(trace_ref, ordered[0][1], ordered[0][0])
            row = {'key': dict(population=population, configuration=cfg, episode=episode, rep=rep, run_id=run_id, obligation_kind=kind, obligation_id=obligation_id),
                'origin': 'DECLARED_FROZEN_ORACLE' if group else 'OBSERVED_EVENT_ONLY_NOT_A_NEW_PLANNED_DENOMINATOR',
                'frozen_expectation': {'value': reader.project(expected, source_refs[ORACLES[population]], (episode, group, index)), 'raw_ref': {'file': source_refs[ORACLES[population]], 'json_pointer': pointer((episode, group, index))}} if group else None,
                'execution_state': execution_state, 'saved_evaluation_state': saved_state,
                'raw_callback_state': first_status, 'first_raw_callback': first,
                'all_matching_display_refs': [event_ref(trace_ref, e, i) for i, e in displays],
                'actual_adopted_plan_Task_goal_revision_refs': [event_ref(trace_ref, e, i) for i, e in adoptions],
                'explicit_stimulus_linked_timeline_refs': [event_ref(trace_ref, e, i) for i, e in linked],
                'matched_typed_fact_refs': [f for f in facts if any(link['value'] in members for link in f['explicit_stimulus_links'])],
                'other_run_typed_fact_event_indexes': [f['raw_ref']['event_index'] for f in facts if not any(link['value'] in members for link in f['explicit_stimulus_links'])],
                'association_limit': 'Unmatched facts remain visible run context. No inferred causal relation or absence claim.',
                'full_trace_context_ref': trace_ref, 'review': unreviewed()}
            if kind == 'response':
                row['saved_QA01_selection'] = {'status': 'MISSING_SAVED_EVALUATION_NO_CERTIFIED_ENDPOINT_SUPPLIED', 'selected': None}
            if kind == 'goal_relation':
                row['saved_QA03_rows'] = [r for r in saved.get('qa03', []) if r.get('id') == obligation_id]
            if not group:
                row['observed_event_ref'] = event_ref(trace_ref, expected, events.index(expected))
            worklist.append(row)
        linked = displays = adoptions = ordered = declarations = observed = []
        del trace, events, saved
    manifest = {'version': VERSION, 'status': 'EVIDENCE_READY_UNREVIEWED' if not issues else 'INCOMPLETE_EVIDENCE_UNREVIEWED',
        'block': dict(population=population, episode=episode, rep=rep), 'freeze': freeze_ref, 'closure': closure_ref,
        'source_references': source_refs, 'planned_configurations': list(CONFIGS), 'planned_run_count': 7,
        'additional_supplement_functional_expectation': extra_functional,
        'declared_obligations': {'response': len(oracle['responses']) * 7, 'goal_relation': len(oracle['goals']) * 7},
        'observed_event_review_rows': sum(r['origin'] != 'DECLARED_FROZEN_ORACLE' for r in worklist),
        'executions': executions, 'issues': issues, 'omissions': reader.omissions, 'inputs': list(reader.files.values()),
        'bytes_read': reader.total_bytes, 'limits': {'trace_bytes': max_trace, 'total_input_bytes': max_total, 'inline_text_characters': MAX_TEXT},
        'saved_evaluation_support': 'UNSUPPORTED_IN_V2; identity/hash match alone is not trace/evaluator provenance. No certified selected endpoint or eligibility is supplied.',
        'interpretation': 'No semantic PASS/FAIL, replacement QA01 filter, metric/paired aggregate, inferred command/source/prompt, evaluator or episode execution. Actual user input, recorded prompt references and frozen oracle conditions remain distinct. Missing output does not establish technically complete failure. Populations remain separate.'}
    return manifest, evidence, worklist


def write_bundle(output, bundle):
    output = Path(output)
    require(not output.exists(), 'Immutable output already exists')
    output.mkdir(parents=True, exist_ok=False)
    manifest, evidence, worklist = bundle
    for name, value in [('evidence.json', evidence), ('worklist.json', worklist)]:
        data = (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode()
        with(output / name).open('xb') as handle:
            handle.write(data)
        manifest.setdefault('outputs', {})[name] = {'sha256': digest(data), 'bytes': len(data)}
    text = readable_review(manifest, evidence, worklist)
    text_bytes = text.encode('utf-8')
    with(output / 'review.md').open('xb') as handle:
        handle.write(text_bytes)
    manifest['outputs']['review.md'] = {'sha256': digest(text_bytes), 'bytes': len(text_bytes)}
    manifest['tool'] = {'path': str(Path(__file__).resolve()), 'sha256': digest(Path(__file__).read_bytes())}
    schema = Path(__file__).with_name('bundle-schema-v2.json')
    manifest['schema'] = {'path': str(schema.resolve()), 'sha256': digest(schema.read_bytes()), 'bytes': schema.stat().st_size}
    checkpoint = Path(__file__).with_name('source-checkpoint-v2.json')
    manifest['tool_checkpoint'] = {'path': str(checkpoint.resolve()), 'sha256': digest(checkpoint.read_bytes()), 'bytes': checkpoint.stat().st_size} if checkpoint.exists() else {'status': 'NOT_PRESENT_PRECHECKPOINT_SYNTHETIC_RUN'}
    with(output / 'manifest.json').open('x') as handle:
        json.dump(manifest, handle, ensure_ascii=False, indent=2)
        handle.write('\n')


def readable_review(manifest, evidence, worklist):
    """Readable projection of existing facts, never a new endpoint/metric."""
    by_run = {e['identity']['run_id']: e for e in evidence}
    lines = ['# Closed-block evidence: UNREVIEWED', '',
        'No callback, adoption, receipt or literal equality implies semantic PASS. All new dimensions remain UNREVIEWED.', '',
        'This v2 accepts no saved evaluator input: no certified selected endpoint or primary eligibility is supplied. Raw first display is not a QA01 selection.', '',
        'Actual initial/streamed input, recorded raw prompt references and frozen oracle conditions are distinct. A literal oracle difference does not automatically establish failure against the actual request.', '']
    for execution in manifest['executions']:
        key = execution['identity']
        lines += ['## ' + key['configuration'] + ' / ' + str(key['run_id']), '',
                  'Execution: ' + execution['execution_state'] + '; saved evaluation: ' + execution['saved_evaluation_state'], '']
        source = by_run.get(key['run_id'])
        if source:
            lines += ['### Recorded source/input context', '', 'Image/binary fields are reference-only. This context is not an inferred source association.', '',
                      '```json', json.dumps(source['candidate_input'], ensure_ascii=False, indent=2), '```', '']
            lines += ['The candidate input file is only the initial JSON object. Later streamed input/control is in the retained events. Recorded model request/response references (no replacement prompt is inferred):', '',
                      '```json', json.dumps(source['raw_model_files'], ensure_ascii=False, indent=2), '```', '',
                      '### Run-level failure/effect/control context', '',
                      'These raw kinds and typed fields do not establish causality, authority or timeliness. No absence claim follows from a shorter row timeline.', '']
            for fact in source['typed_failure_effect_control_refs']:
                lines += ['```json', json.dumps({'typed_reference': fact, 'record': source['events'][fact['raw_ref']['event_index']]['record']}, ensure_ascii=False, indent=2), '```', '']
        for row in worklist:
            if row['key']['configuration'] != key['configuration']:
                continue
            lines += ['### ' + row['key']['obligation_kind'] + ' / ' + row['key']['obligation_id'], '',
                      'Review: **UNREVIEWED**', '', 'Frozen expectation:', '', '```json',
                      json.dumps(row['frozen_expectation'], ensure_ascii=False, indent=2), '```', '',
                      'Raw callback state: ' + row['raw_callback_state'], '', 'Saved endpoint fact:', '', '```json',
                      json.dumps(row.get('saved_QA01_selection', row.get('saved_QA03_rows')), ensure_ascii=False, indent=2), '```', '']
            if source:
                refs = [r for r in [row['first_raw_callback'], *row['all_matching_display_refs'], *row['actual_adopted_plan_Task_goal_revision_refs']] if r]
                selected = row.get('saved_QA01_selection', {}).get('selected')
                if selected:
                    refs.append(selected)
                if row.get('observed_event_ref'):
                    refs.append(row['observed_event_ref'])
                seen = set()
                for record in refs:
                    if record['event_index'] in seen:
                        continue
                    seen.add(record['event_index'])
                    event = source['events'][record['event_index']]
                    lines += ['Actual event index ' + str(record['event_index']) + ' / ' + str(record['event_id']) + ':', '',
                              '```json', json.dumps(event, ensure_ascii=False, indent=2), '```', '']
                lines += ['Timeline with explicit matching stimulus IDs (all preceding/following trace context stays in evidence.json):', '',
                          '| Index | Event ID | Kind | Host-observed ns |', '| ---: | --- | --- | ---: |']
                for record in row['explicit_stimulus_linked_timeline_refs']:
                    event = source['events'][record['event_index']]['record']
                    lines.append('| ' + ' | '.join(str(v) for v in [record['event_index'], record['event_id'], event.get('kind'), event.get('observed_ns')]) + ' |')
                lines += ['']
    return '\n'.join(lines) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--closure', type=Path, required=True)
    parser.add_argument('--population', choices=tuple(ORACLES), required=True)
    parser.add_argument('--episode', required=True)
    parser.add_argument('--rep', type=int, required=True)
    parser.add_argument('--freeze-sha256', required=True)
    parser.add_argument('--saved-evaluations', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    checkpoint = Path(__file__).with_name('source-checkpoint-v2.json')
    require(checkpoint.exists(), 'Reviewed-source checkpoint v2 is required before actual extraction')
    recorded = json.loads(checkpoint.read_bytes())
    require(any(Path(entry['path']).resolve() == Path(__file__).resolve() and entry['sha256'] == digest(Path(__file__).read_bytes())
                for entry in recorded['files']), 'Helper source changed after v2 checkpoint')
    require(not args.output.exists(), 'Immutable output already exists')
    require(not args.output.resolve().is_relative_to(args.root.resolve() / 'frozen'), 'Output may not enter frozen sources')
    bundle = build_bundle(args.root, args.closure, args.population, args.episode, args.rep,
                          args.freeze_sha256, args.saved_evaluations)
    write_bundle(args.output, bundle)


if __name__ == '__main__':
    main()
