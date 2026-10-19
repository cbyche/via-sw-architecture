"""Pure v3 closure/index invariants; no experiment or evaluator imports."""
import hashlib
import json
from pathlib import Path
import re
import stat

INDEX_VERSION = 'JOINT441_CLOSED_BLOCK_INDEX_v1'
CURSOR_VERSION = 'JOINT441_REVIEW_CURSOR_v1'
CONFIGS = ('AAAA', 'BAAA', 'CAAA', 'ABAA', 'AABA', 'AAAB', 'AAAC')
MAX_FILE = 16 * 1024 * 1024
MAX_TOTAL = 128 * 1024 * 1024
MAX_PREFIX = 1048576

class EvidenceError(ValueError):
    pass

def require(value, reason):
    if not value:
        raise EvidenceError(reason)

def digest(data):
    return hashlib.sha256(data).hexdigest()

def safe_path(value, root):
    require(isinstance(value, (str, Path)) and bool(str(value)) and '://' not in str(value), 'Unsupported reference scheme/path')
    raw = Path(value)
    require(not any(part == '..' for part in raw.parts), 'Traversal reference rejected')
    path = (raw if raw.is_absolute() else root / raw).resolve()
    require(path.is_relative_to(root), 'Reference outside exact root')
    if path.exists():
        require(stat.S_ISREG(path.stat().st_mode), 'Unexpected input file type')
    return path

def integer(value):
    return type(value) is int

def ref(reader, item, parse=True):
    require(isinstance(item, dict), 'Missing immutable reference')
    path = safe_path(item.get('path', item.get('copied_path', '')), reader.root)
    value, actual = reader.read(path, limit=MAX_FILE, parse=parse)
    require(actual['sha256'] == item.get('sha256') and actual['bytes'] == item.get('bytes'), 'Immutable reference hash/size mismatch')
    return value, actual

def block_jobs(freeze, block):
    require(freeze.get('version') == 'COMMON9_PLUS_SUPPLEMENT6_JOINT441_v1' and freeze.get('total_episode_runs') == 441 and len(freeze.get('jobs', [])) == 441, 'Wrong exact frozen generation/population')
    ordinal = block.get('ordinal_in_frozen_block_order')
    require(integer(ordinal) and 0 <= ordinal < 63, 'Invalid block ordinal')
    indexes = list(range(ordinal * 7, ordinal * 7 + 7))
    require(block.get('job_indexes') == indexes and all(integer(x) for x in block['job_indexes']), 'Wrong frozen job indexes')
    jobs = [freeze['jobs'][i] for i in indexes]
    for key in ('block_id', 'population', 'episode', 'rep'):
        require(all(j.get(key) == block.get(key) for j in jobs), 'Mixed/unplanned frozen block identity')
    require(type(block.get('rep')) is int and block['rep'] > 0 and block['population'] in ('COMMON9', 'COMMON_SUPPLEMENT6'), 'Invalid population/repetition')
    require(len({j['configuration'] for j in jobs}) == 7 and set(j['configuration'] for j in jobs) == set(CONFIGS), 'Seven configurations required')
    return jobs

def validate_cursor(reader, cursor_ref, freeze, freeze_sha, block):
    cursor, actual = ref(reader, cursor_ref)
    require(cursor.get('version') == CURSOR_VERSION and cursor.get('freeze_sha256') == freeze_sha, 'Wrong cursor version/freeze')
    ordinal = block['ordinal_in_frozen_block_order']
    require(cursor.get('next_block_ordinal') == ordinal and integer(cursor.get('next_block_ordinal')), 'Cursor must select exact next block')
    previous = cursor.get('previous_block', {})
    require(previous.get('ordinal_in_frozen_block_order', previous.get('ordinal')) == ordinal - 1, 'Wrong preceding block')
    prev = dict(previous, ordinal_in_frozen_block_order=ordinal - 1)
    prev.pop('ordinal', None)
    block_jobs(freeze, prev)
    previous_bundle, _ = ref(reader, cursor.get('previous_bundle'))
    ref(reader, cursor.get('previous_review'))
    authority, _ = ref(reader, cursor.get('root_authorization'))
    if cursor.get('bootstrap_kind') == 'ACCEPTED_PRE_V3_S02_BUNDLE':
        require(ordinal == 1 and previous.get('population') == 'COMMON9' and previous.get('episode') == 'S02' and previous.get('rep') == 1, 'Bootstrap only for exact S02 to ordinal1')
        require(cursor.get('previous_index') == {'status': 'NOT_AVAILABLE_PRE_V3', 'ref': None}, 'No synthetic bootstrap index')
        ref(reader, cursor.get('historical_closure'))
        require(authority.get('scope') == 'ACCEPTED_PRE_V3_S02_BUNDLE_ONLY' and authority.get('next_block_ordinal') == 1, 'Wrong bootstrap authorization scope')
        require(authority.get('allowed_bootstrap_sha256') == {k: cursor[k]['sha256'] for k in ('historical_closure', 'previous_bundle', 'previous_review')}, 'Bootstrap references differ from explicit authority')
    else:
        require(cursor.get('bootstrap_kind') is None, 'Unsupported bootstrap kind')
        prior, _ = ref(reader, cursor.get('previous_index'))
        require(prior.get('version') == INDEX_VERSION and prior.get('block') == prev, 'Cursor previous v3 index mismatch')
        require(previous_bundle.get('version') == 'JOINT441_CLOSED_BLOCK_EVIDENCE_v3' and previous_bundle.get('block') == prev and previous_bundle.get('closure', {}).get('sha256') == cursor['previous_index']['sha256'], 'Cursor previous v3 bundle/index mismatch')
        require(authority.get('scope') == 'ADVANCE_EXACT_PREVIOUS_V3_REVIEW' and authority.get('freeze_sha256') == freeze_sha and authority.get('next_block_ordinal') == ordinal and authority.get('previous_block') == prev, 'Ordinary cursor lacks exact parent advancement authority')
        require(authority.get('allowed_previous_sha256') == {k: cursor[k]['sha256'] for k in ('previous_index', 'previous_bundle', 'previous_review')}, 'Ordinary cursor previous review/index/bundle association mismatch')
    return actual

def terminal_run_id(record):
    require(record.get('state') == 'TERMINATED', 'Exact TERMINATED required')
    require(type(record.get('stdout')) is str, 'Unsupported recorded stdout')
    lines = [x for x in record['stdout'].splitlines() if x.strip()]
    require(bool(lines), 'Missing recorded stdout run-ID evidence')
    try:
        summary = json.loads(lines[-1])
    except (ValueError, TypeError) as error:
        raise EvidenceError('Unsupported stdout final JSON') from error
    run_id = summary.get('run_id') if isinstance(summary, dict) else None
    require(isinstance(run_id, str) and re.fullmatch(r'[A-Za-z0-9_.-]+', run_id), 'Invalid recorded run ID')
    return run_id

def validate_index(reader, index_path, index_sha, freeze, freeze_ref):
    index, index_ref = reader.read(safe_path(index_path, reader.root), limit=MAX_FILE)
    require(index_ref['sha256'] == index_sha and index.get('version') == INDEX_VERSION, 'Wrong index hash/version')
    require(index.get('kind') == 'POSTLAUNCH_TERMINATION_INDEX_NOT_MEASUREMENT_CERTIFICATION' and Path(index.get('formal_root', '')).resolve() == reader.root, 'Wrong formal root/index kind')
    require(index.get('freeze') == freeze_ref and index.get('binary_sha256') == freeze['binary']['sha256'], 'Wrong index freeze/binary')
    jobs = block_jobs(freeze, index['block'])
    validate_cursor(reader, index['review_cursor'], freeze, freeze_ref['sha256'], index['block'])
    prefix, prefix_ref = ref(reader, index['source_progress']['prefix'], parse=False)
    require(len(prefix) <= MAX_PREFIX, 'Prefix cap exceeded')
    complete = prefix.rfind(b'\n') + 1
    require(index['source_progress']['complete_line_bytes'] == complete and index['source_progress']['trailing_partial_bytes'] == len(prefix) - complete, 'Prefix boundary mismatch')
    require(safe_path(index['source_progress']['original_path'], reader.root) == reader.root / 'campaign-progress.jsonl', 'Wrong original progress path')
    requested = [json.loads(line) for line in prefix[:complete].splitlines() if line]
    requested = [r for r in requested if r.get('block_id') == index['block']['block_id'] and r.get('state') == 'TERMINATED']
    require(len(requested) == 7 and len({r.get('configuration') for r in requested}) == 7, 'Duplicate/missing requested terminal in captured prefix')
    rows = index.get('rows', [])
    require(len(rows) == 7, 'Exactly seven index rows required')
    seen = set()
    for row, job, job_index in zip(rows, jobs, index['block']['job_indexes']):
        require(row.get('job_index') == job_index and integer(row.get('job_index')), 'Index job position mismatch')
        for key in ('configuration', 'episode', 'rep', 'population'):
            require(row.get(key) == job[key], 'Mixed index row identity')
        raw, term_ref = ref(reader, row['termination_record'], parse=False)
        meta = row['termination_record']; offset = meta.get('original_byte_offset'); length = meta.get('original_byte_length')
        require(integer(offset) and integer(length) and offset >= 0 and length > 0 and raw.endswith(b'\n') and raw.count(b'\n') == 1, 'Invalid complete-line range')
        require(offset + length <= complete and prefix[offset:offset+length] == raw and length == len(raw), 'Terminal range/prefix mismatch')
        record = json.loads(raw)
        for key in ('configuration', 'episode', 'rep', 'population', 'block_id'):
            require(record.get(key) == job[key], 'Mixed terminal identity')
        require(integer(record.get('returncode')) and integer(record.get('at_ns')), 'Invalid terminal clock/returncode')
        require(all(k in record and (record[k] is None or type(record[k]) is bool) for k in ('technical_failure', 'budget_stop')) and isinstance(record.get('stderr'), str), 'Incomplete terminal evidence; flags must be present bool-or-null')
        require(integer(row.get('rep')) and integer(record.get('rep')), 'Bool repetition rejected')
        run_id = terminal_run_id(record)
        require(row.get('run_id_origin') == {'method': 'RECORDED_STDOUT_LAST_JSON', 'field': 'run_id'} and row.get('run_id') == run_id and run_id not in seen, 'Duplicate/mismatched run ID origin')
        seen.add(run_id)
        expected = reader.root / 'raw/runs' / run_id / 'trace.json'
        require(safe_path(row['trace']['path'], reader.root) == expected, 'Wrong expected trace path')
        if row['trace'].get('status') != 'MISSING_TRACE':
            trace, _ = ref(reader, row['trace'])
            for key, value in {'run_id': run_id, 'configuration': job['configuration'], 'episode': job['episode'], 'rep': job['rep'], 'phase': 'FORMAL', 'binary_sha256': freeze['binary']['sha256']}.items():
                require(trace.get(key) == value, 'Trace identity/binary mismatch')
            require(integer(trace.get('rep')), 'Bool trace repetition rejected')
            require(integer(trace.get('end_ns')) and integer(trace.get('returncode')), 'Open/bool closed trace rejected')
            candidate = safe_path(trace.get('candidate_input_file', ''), reader.root)
            require(candidate.parent == expected.parent, 'Wrong candidate initial parent')
    return index, index_ref


def checkpoint_guard(checkpoint_path, own_path):
    """Bound helper-source reads too; only reviewed sibling source files allowed."""
    checkpoint_path, own_path = Path(checkpoint_path).resolve(), Path(own_path).resolve()
    total, attempts = 0, []
    def read_bounded(path):
        nonlocal total
        before = path.stat()
        require(stat.S_ISREG(before.st_mode) and before.st_size <= MAX_FILE and total + before.st_size <= MAX_TOTAL, 'Helper source byte/type cap before read')
        attempt = {'path': str(path), 'requested_bytes': before.st_size, 'returned_bytes': None}
        attempts.append(attempt)
        with path.open('rb') as handle:
            raw = handle.read(before.st_size)
        attempt['returned_bytes'] = len(raw); total += len(raw)
        after = path.stat()
        require(len(raw) == before.st_size and (before.st_size, before.st_mtime_ns) == (after.st_size, after.st_mtime_ns), 'Helper source changed during bounded read')
        return raw
    try:
        data = read_bounded(checkpoint_path)
        saved = json.loads(data)
        entries = saved.get('files', [])
        require(bool(entries) and any(Path(e['path']).resolve() == own_path for e in entries), 'Current helper absent from checkpoint')
        refs = [{'path': str(checkpoint_path), 'sha256': digest(data), 'bytes': len(data)}]
        for item in entries:
            path = Path(item['path']).resolve()
            require(path.is_relative_to(checkpoint_path.parent), 'Unsafe helper source checkpoint reference')
            raw = read_bounded(path)
            require(digest(raw) == item['sha256'], 'Helper dependency changed')
            refs.append({'path': str(path), 'sha256': digest(raw), 'bytes': len(raw)})
        return refs, total
    except Exception as error:
        error.evidence_bytes_read = total
        error.evidence_read_attempts = attempts
        raise
