"""One approved <=1MiB metadata prefix; no directory/corpus discovery or jobs."""
import argparse
import datetime
import json
from pathlib import Path
import stat

from extract_closed_block_v3 import Reader, digest
from schema_v3 import (INDEX_VERSION, MAX_FILE, MAX_PREFIX, MAX_TOTAL,
                       EvidenceError, require, block_jobs, validate_cursor,
                       terminal_run_id, safe_path, checkpoint_guard)


def write_new(path, data):
    with path.open('xb') as handle:
        handle.write(data)
    return {'path': str(path.resolve()), 'sha256': digest(data), 'bytes': len(data)}


def select_prefix(prefix, block, jobs):
    end = prefix.rfind(b'\n') + 1
    selected = {}
    offset = 0
    for line in prefix[:end].splitlines(keepends=True):
        record = json.loads(line)
        if record.get('block_id') == block['block_id'] and record.get('state') == 'TERMINATED':
            cfg = record.get('configuration')
            require(cfg in [j['configuration'] for j in jobs] and cfg not in selected,
                    'Duplicate/conflicting requested terminal')
            for key in ('population', 'episode', 'rep'):
                require(record.get(key) == block[key], 'Mixed requested terminal identity')
            selected[cfg] = (offset, line, record)
        offset += len(line)
    return selected, end


def prepare(root, freeze_sha, block, cursor_reference, output, after_capture=None, initial_bytes=0, source_inputs=None):
    root = Path(root).resolve()
    output = Path(output).resolve()
    require(output.is_relative_to(root) and not output.is_relative_to(root / 'frozen') and not output.exists(), 'Fresh nonfrozen output inside exact root required')
    reader = Reader(root, MAX_FILE, MAX_TOTAL)
    reader.total_bytes = initial_bytes
    freeze, freeze_ref = reader.read(root / 'freeze.json')
    require(freeze_ref['sha256'] == freeze_sha, 'Wrong freeze hash')
    jobs = block_jobs(freeze, block)
    validate_cursor(reader, cursor_reference, freeze, freeze_sha, block)
    progress = safe_path(root / 'campaign-progress.jsonl', root)
    require(stat.S_ISREG(progress.stat().st_mode), 'Expected regular progress file')
    before = progress.stat()
    with progress.open('rb') as handle:
        prefix = handle.read(MAX_PREFIX)  # Exactly one prefix read; never another window.
    captured = datetime.datetime.now(datetime.timezone.utc).isoformat()
    reader.total_bytes += len(prefix)
    require(reader.total_bytes <= MAX_TOTAL, 'Total input byte limit exceeded')
    output.mkdir(parents=True, exist_ok=False)
    prefix_ref = write_new(output / 'progress-prefix.bin', prefix)
    selected, complete = select_prefix(prefix, block, jobs)
    progress_ref = {'original_path': str(progress), 'captured_at_utc': captured,
                    'scope': 'ONE_BOUNDED_PREFIX_COMPLETE_LINE_RANGES_ONLY_FILE_REMAINS_APPENDABLE',
                    'method': 'ONE_READ_FROM_BYTE0_MAX1048576_NO_EOF_LOOP',
                    'prefix': prefix_ref, 'complete_line_bytes': complete,
                    'trailing_partial_bytes': len(prefix) - complete,
                    'file_identity': {'device': before.st_dev, 'inode': before.st_ino},
                    'limitation': 'Cannot prove absence of contradictions appended after this prefix; final reconciliation required.'}
    if len(selected) != 7:
        diagnostic = {'version': INDEX_VERSION, 'status': 'REQUESTED_BLOCK_NOT_FULLY_IN_CAPTURED_PREFIX',
                      'formal_root': str(root), 'freeze': freeze_ref, 'block': block,
                      'source_progress': progress_ref, 'planned_jobs': jobs,
                      'present_configurations': list(selected), 'selection': 'NO_ALTERNATIVE_BLOCK_SELECTED',
                      'planned_obligations': 'UNCHANGED_REFER_TO_EXACT_FROZEN_ORACLE',
                      'bytes_read': reader.total_bytes}
        write_new(output / 'index-diagnostic.json', (json.dumps(diagnostic, indent=2) + '\n').encode())
        return diagnostic
    if after_capture:
        after_capture(progress)  # Synthetic acceptance hook; no production wait/IO hook.
    after = progress.stat()
    require((before.st_dev, before.st_ino) == (after.st_dev, after.st_ino), 'Progress file replaced')
    rows = []
    seen = set()
    for position, job in enumerate(jobs):
        offset, line, record = selected[job['configuration']]
        with progress.open('rb') as handle:
            handle.seek(offset)
            reread = handle.read(len(line))
        reader.total_bytes += len(reread)
        require(reader.total_bytes <= MAX_TOTAL and reread == line, 'Selected terminal range changed/truncated')
        current = progress.stat()
        require((before.st_dev, before.st_ino) == (current.st_dev, current.st_ino), 'Progress file replaced')
        rid = terminal_run_id(record)
        require(rid not in seen, 'Duplicate recorded run ID')
        seen.add(rid)
        term = write_new(output / ('terminal-' + str(position) + '.jsonl'), line)
        term.update(copied_path=term.pop('path'), original_byte_offset=offset,
                    original_byte_length=len(line), state='TERMINATED')
        trace_path = root / 'raw/runs' / rid / 'trace.json'
        if trace_path.exists():
            _, trace_ref = reader.read(trace_path, limit=MAX_FILE)
        else:
            trace_ref = {'path': str(trace_path), 'status': 'MISSING_TRACE'}
        rows.append(dict(job_index=block['job_indexes'][position], configuration=job['configuration'],
                         episode=job['episode'], rep=job['rep'], population=job['population'], run_id=rid,
                         termination_record=term, run_id_origin={'method': 'RECORDED_STDOUT_LAST_JSON', 'field': 'run_id'},
                         trace=trace_ref))
    index = {'version': INDEX_VERSION, 'kind': 'POSTLAUNCH_TERMINATION_INDEX_NOT_MEASUREMENT_CERTIFICATION',
             'formal_root': str(root), 'freeze': freeze_ref, 'binary_sha256': freeze['binary']['sha256'],
             'block': block, 'source_progress': progress_ref, 'rows': rows,
             'review_cursor': cursor_reference, 'construction': {'tool_or_reviewer': 'Sol High prepare_block_index_v3',
             'method': 'Explicit fixed block/cursor, one bounded prefix and seven exact range rereads',
             'runtime_modified': False, 'jobs_executed': False, 'source_inputs': source_inputs}, 'bytes_read': reader.total_bytes}
    write_new(output / 'index.json', (json.dumps(index, ensure_ascii=False, indent=2) + '\n').encode())
    return index


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ('root', 'output', 'cursor'):
        parser.add_argument('--' + key, type=Path, required=True)
    for key in ('freeze-sha256', 'block-id', 'population', 'episode', 'cursor-sha256'):
        parser.add_argument('--' + key, required=True)
    parser.add_argument('--rep', type=int, required=True)
    parser.add_argument('--ordinal', type=int, required=True)
    args = parser.parse_args()
    block = dict(block_id=args.block_id, population=args.population, episode=args.episode,
                 rep=args.rep, ordinal_in_frozen_block_order=args.ordinal,
                 job_indexes=list(range(args.ordinal * 7, args.ordinal * 7 + 7)))
    checkpoint = Path(__file__).with_name('source-checkpoint-v3.json')
    require(checkpoint.exists(), 'Reviewed-source checkpoint v3 required before actual preparation')
    helper_refs, helper_bytes = checkpoint_guard(checkpoint, Path(__file__))
    cursor_ref = {'path': str(args.cursor.resolve()), 'sha256': args.cursor_sha256, 'bytes': args.cursor.stat().st_size}
    require(not args.output.exists(), 'Immutable preparation output already exists')
    require(args.output.resolve().is_relative_to(args.root.resolve()) and not args.output.resolve().is_relative_to(args.root.resolve() / 'frozen'), 'Unsafe preparation output')
    try:
        result = prepare(args.root, args.freeze_sha256, block, cursor_ref, args.output, initial_bytes=helper_bytes, source_inputs=helper_refs)
    except (EvidenceError, ValueError, OSError) as error:
        args.output.mkdir(parents=True, exist_ok=True)
        write_new(args.output / 'preparation-error.json', (json.dumps({'status': 'INCOMPLETE_INDEX_EVIDENCE', 'block': block, 'planned_run_count': 7, 'error': str(error), 'runtime_modified': False, 'jobs_executed': False}) + '\n').encode())
        raise
    print(json.dumps({'status': result.get('status', 'TERMINATION_INDEX_CREATED_UNREVIEWED'), 'block': block, 'bytes_read': result['bytes_read']}))


if __name__ == '__main__':
    main()
