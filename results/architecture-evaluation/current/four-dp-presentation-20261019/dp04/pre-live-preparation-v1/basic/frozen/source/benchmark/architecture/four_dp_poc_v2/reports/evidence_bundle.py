"""Lossless, split evidence archive. Stdlib only; no network or model execution.

Original campaign files are read only. Safe internal source symlinks are recorded
and materialized as regular target bytes, so a restored tree is self contained.
"""
import argparse
import collections
import gzip
import hashlib
import io
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import re
import runpy
import shutil
import stat
import sys
import tarfile

ROOTS = (
    'results/architecture-evaluation/current/four-dp-qwen37-20261018',
    'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018',
)
PART_BYTES = 20 * 1024 * 1024
VERSION = 'FOUR_DP_LOSSLESS_EVIDENCE_V1_20261019'
TOKEN_PATTERNS = {
    'openai_live_key': re.compile(rb'\bsk-(?:proj-|svcacct-)?[A-Za-z0-9_-]{24,}'),
    'github_token': re.compile(rb'\b(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{35,})'),
    'aws_access_id': re.compile(rb'\b(?:AKIA|ASIA)[A-Z0-9]{16}\b'),
    'private_key': re.compile(rb'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----'),
    'literal_bearer_token': re.compile(rb'(?i)authorization[\s\\"\x27:]{1,12}bearer\s+[A-Za-z0-9._~-]{24,}'),
}


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def save_new(path, value):
    path = Path(path)
    if path.exists() or path.is_symlink():
        raise ValueError('Refuse replacing package file: ' + str(path))
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + '\n')


def safe_relative(name):
    if not isinstance(name, str) or not name or name == '.' or '\\' in name or '\x00' in name or ':' in name:
        raise ValueError('Unsafe path name')
    p = PurePosixPath(name)
    if p.is_absolute() or any(x in ('', '.', '..') for x in p.parts) or str(p) != name:
        raise ValueError('Unsafe relative path: ' + repr(name))
    return p


def in_roots(name):
    return any(name.startswith(root + '/') for root in ROOTS)


def cache_reason(relative):
    p = PurePosixPath(relative)
    if 'target' in p.parts:
        return 'regenerable Rust target build/cache (measured frozen/bin retained)'
    if '__pycache__' in p.parts or p.suffix in ('.pyc', '.pyo'):
        return 'regenerable Python bytecode cache'
    if p.name == '.DS_Store':
        return 'Finder metadata cache'
    return None


def inventory(repo):
    entries, excluded = [], []
    for root in ROOTS:
        base = repo / root
        if not base.is_dir() or base.is_symlink():
            raise ValueError('Missing/nonregular source campaign: ' + root)
        for directory, dirs, files in os.walk(base, followlinks=False):
            # Directory symlinks could silently hide unarchived paths: refuse.
            for name in dirs:
                if (Path(directory) / name).is_symlink():
                    raise ValueError('Directory symlink is not supported')
            for name in sorted(files):
                p = Path(directory) / name
                relative = p.relative_to(repo).as_posix()
                safe_relative(relative)
                reason = cache_reason(relative)
                if reason:
                    excluded.append({'path': relative, 'bytes': p.lstat().st_size, 'reason': reason})
                    continue
                original_type = 'regular'
                target = p
                link = None
                if p.is_symlink():
                    original_type = 'materialized_internal_symlink'
                    link = os.readlink(p)
                    target = p.resolve(strict=True)
                    if not any(target.is_relative_to((repo / r).resolve()) for r in ROOTS):
                        raise ValueError('Outside-scope symlink: ' + relative)
                if not target.is_file() or not stat.S_ISREG(target.stat().st_mode):
                    raise ValueError('Special source file: ' + relative)
                entry = {'path': relative, 'bytes': target.stat().st_size,
                         'sha256': sha(target), 'mode': stat.S_IMODE(target.stat().st_mode),
                         'original_type': original_type}
                if link is not None:
                    entry.update(original_link_target=link,
                                 resolved_target=target.relative_to(repo).as_posix(),
                                 restore_type='regular_materialized_bytes')
                if relative.endswith('/IMPLEMENTATION-dp01-dp05.md') and '/frozen/source/' in relative:
                    entry['markdown_link_context'] = relative.split('/frozen/source/', 1)[1]
                history = ROOTS[1] + '/qa04/history-before-union-final-review/final-effective-cross-review.md'
                if relative == history:
                    entry['markdown_link_context'] = ROOTS[1] + '/qa04/final-effective-cross-review.md'
                entries.append(entry)
    return sorted(entries, key=lambda x: x['path']), sorted(excluded, key=lambda x: x['path'])


def scan_tokens(repo, entries):
    hits = []
    for e in entries:
        p = repo / e['path']
        # Tokens may straddle chunks. Do not print matching bytes under any circumstance.
        seen = collections.Counter()
        with p.open('rb') as f:
            tail = b''
            while True:
                block = f.read(1024 * 1024)
                if not block:
                    break
                data = tail + block
                for name, pattern in TOKEN_PATTERNS.items():
                    for m in pattern.finditer(data):
                        if m.end() > len(tail):
                            seen[name] += 1
                tail = data[-512:]
        for name, count in seen.items():
            hits.append({'path': e['path'], 'pattern': name, 'matches': count})
    return {'files_scanned': len(entries), 'pattern_names': sorted(TOKEN_PATTERNS),
            'finding_count': sum(x['matches'] for x in hits), 'findings': hits,
            'scope': 'Content token-pattern scan, no credential lookup; values never emitted. Not a proof against every possible secret format.'}


class SplitWriter:
    def __init__(self, folder):
        self.folder, self.handle, self.part_hash = folder, None, None
        self.parts, self.total, self.current = [], 0, 0
        self.full_hash = hashlib.sha256()

    def writable(self): return True

    def flush(self):
        if self.handle:
            self.handle.flush()

    def finish_part(self):
        if self.handle:
            self.handle.close()
            self.parts[-1].update(bytes=self.current, sha256=self.part_hash.hexdigest())
            self.handle = None

    def write(self, data):
        n = len(data)
        offset = 0
        while offset < n:
            if self.handle is None:
                name = f'evidence.tar.gz.part{len(self.parts) + 1:04d}'
                self.handle = (self.folder / name).open('xb')
                self.parts.append({'path': name})
                self.part_hash, self.current = hashlib.sha256(), 0
            block = data[offset:offset + min(n - offset, PART_BYTES - self.current)]
            self.handle.write(block)
            self.part_hash.update(block)
            self.full_hash.update(block)
            self.total += len(block)
            self.current += len(block)
            offset += len(block)
            if self.current == PART_BYTES:
                self.finish_part()
        return n


class PartsReader(io.RawIOBase):
    def __init__(self, package, parts):
        self.paths = iter(package / p['path'] for p in parts)
        self.handle = None

    def readable(self): return True

    def readinto(self, buf):
        if not len(buf): return 0
        while True:
            if self.handle is None:
                path = next(self.paths, None)
                if path is None: return 0
                self.handle = path.open('rb')
            n = self.handle.readinto(buf)
            if n: return n
            self.handle.close()
            self.handle = None

    def close(self):
        if self.handle: self.handle.close()
        super().close()


def load_checked(package):
    meta = read(package / 'package.json')
    if meta['version'] != VERSION or sha(package / 'manifest.json') != meta['manifest_sha256']:
        raise ValueError('Invalid manifest version/hash')
    if sha(package / 'exclusions.json') != meta['exclusions_sha256']:
        raise ValueError('Invalid exclusions hash')
    if sha(package / 'secret-scan.json') != meta['secret_scan_sha256']:
        raise ValueError('Invalid token-scan hash')
    manifest = read(package / 'manifest.json')
    entries, seen = manifest['files'], set()
    for e in entries:
        name = e['path']; safe_relative(name)
        if name in seen or not in_roots(name): raise ValueError('Duplicate/out-of-scope manifest path')
        if type(e['bytes']) is not int or e['bytes'] < 0 or not re.fullmatch('[0-9a-f]{64}', e['sha256']):
            raise ValueError('Invalid file metadata')
        if type(e['mode']) is not int or not 0 <= e['mode'] <= 0o777:
            raise ValueError('Unsafe file mode')
        if 'markdown_link_context' in e:
            safe_relative(e['markdown_link_context'])
        seen.add(name)
    for name in seen:
        if any(str(p) in seen for p in PurePosixPath(name).parents if str(p) != '.'):
            raise ValueError('File is ancestor of another file')
    if len(entries) != meta['files'] or sum(e['bytes'] for e in entries) != meta['uncompressed_file_bytes']:
        raise ValueError('Manifest aggregate mismatch')
    total, full = 0, hashlib.sha256()
    for i, part in enumerate(meta['parts'], 1):
        if part['path'] != f'evidence.tar.gz.part{i:04d}' or not 0 < part['bytes'] <= PART_BYTES:
            raise ValueError('Unsafe/noncontiguous part list')
        p = package / part['path']
        if p.is_symlink() or not p.is_file() or p.stat().st_size != part['bytes'] or sha(p) != part['sha256']:
            raise ValueError('Part hash/size mismatch: ' + part['path'])
        with p.open('rb') as f:
            for block in iter(lambda: f.read(1024 * 1024), b''): full.update(block)
        total += part['bytes']
    if total != meta['compressed_bytes'] or full.hexdigest() != meta['compressed_sha256']:
        raise ValueError('Compressed aggregate hash mismatch')
    return meta, entries


def destination_path(destination, name):
    p = destination / safe_relative(name)
    q = p
    while q != destination:
        if q.is_symlink(): raise ValueError('Refuse existing destination symlink: ' + str(q))
        if q != p and q.exists() and not q.is_dir(): raise ValueError('Destination parent is not a directory')
        q = q.parent
    return p


def verify_restored(package, destination, live_original=False):
    meta, entries = load_checked(package)
    for e in entries:
        p = destination / e['path'] if live_original else destination_path(destination, e['path'])
        if not p.is_file() or p.stat().st_size != e['bytes'] or sha(p) != e['sha256']:
            raise ValueError('Restored/live file differs: ' + e['path'])
        if live_original and e['original_type'] == 'materialized_internal_symlink':
            if not p.is_symlink() or os.readlink(p) != e['original_link_target']:
                raise ValueError('Original symlink topology changed')
        elif p.is_symlink(): raise ValueError('Unexpected restored symlink')
    return {'status': 'PASS', 'files': len(entries), 'bytes': meta['uncompressed_file_bytes'],
            'mode': 'live-original-source-unchanged' if live_original else 'every-restored-file-byte-hash',
            'destination': str(destination)}


def archive(package, destination=None):
    meta, entries = load_checked(package)
    expected = {e['path']: e for e in entries}
    if destination is not None:
        if destination.is_symlink(): raise ValueError('Destination root is a symlink')
        # Validate all conflicts before making any extraction directories/writes.
        for e in entries:
            p = destination_path(destination, e['path'])
            if p.exists() and (not p.is_file() or p.stat().st_size != e['bytes'] or sha(p) != e['sha256']):
                raise ValueError('Refuse overwriting different existing file: ' + e['path'])
    seen = set()
    with io.BufferedReader(PartsReader(package, meta['parts'])) as stream:
        with tarfile.open(fileobj=stream, mode='r|gz') as tar:
            for member in tar:
                name = member.name; safe_relative(name)
                e = expected.get(name)
                if e is None or name in seen or not member.isfile() or member.size != e['bytes'] or member.mode != e['mode']:
                    raise ValueError('Unexpected/unsafe archive member')
                seen.add(name)
                p = destination_path(destination, name) if destination is not None else None
                temporary = None
                if p is not None and not p.exists():
                    p.parent.mkdir(parents=True, exist_ok=True)
                    temporary = p.with_name('.' + p.name + '.evidence-restore-tmp')
                    if temporary.exists() or temporary.is_symlink(): raise ValueError('Temporary destination already exists')
                out = temporary.open('xb') if temporary is not None else None
                h = hashlib.sha256()
                try:
                    with tar.extractfile(member) as source:
                        for block in iter(lambda: source.read(1024 * 1024), b''):
                            h.update(block)
                            if out: out.write(block)
                    if out: out.close(); out = None
                    if h.hexdigest() != e['sha256']: raise ValueError('Archive content hash mismatch: ' + name)
                    if temporary is not None:
                        os.chmod(temporary, e['mode']); temporary.rename(p)
                finally:
                    if out: out.close()
                    if temporary is not None and temporary.exists(): temporary.unlink()
    if seen != set(expected): raise ValueError('Missing archive members')
    return {'status': 'PASS', 'files': len(seen), 'compressed_sha256': meta['compressed_sha256'],
            'destination': str(destination) if destination is not None else None}


def build(repo, package):
    package.mkdir(parents=True, exist_ok=True)
    if any((package / n).exists() for n in ['manifest.json', 'package.json', 'exclusions.json']) or list(package.glob('evidence.tar.gz.part*')):
        raise ValueError('Use a new package directory; existing bundle is immutable')
    entries, excluded = inventory(repo)
    scan = scan_tokens(repo, entries)
    save_new(package / 'secret-scan.json', scan)
    if scan['finding_count']:
        raise ValueError('Token patterns found; names/counts only are in secret-scan.json. No archive created.')
    save_new(package / 'manifest.json', {'version': VERSION, 'original_repo_root': str(repo),
             'campaign_roots': list(ROOTS), 'files': entries,
             'symlink_policy': 'Safe internal source symlinks materialized as regular target bytes; original link target and resolved target are provenance, not restored symlink topology.'})
    save_new(package / 'exclusions.json', {'version': VERSION, 'files': excluded,
             'scope': 'Only Rust target build/cache, Python bytecode/__pycache__, Finder .DS_Store. All other failed/calibration/stopped/raw/frozen/demo/source-snapshot files retained.'})
    writer = SplitWriter(package)
    with gzip.GzipFile(filename='', mode='wb', compresslevel=6, fileobj=writer, mtime=0) as zipped:
        with tarfile.open(fileobj=zipped, mode='w|', format=tarfile.PAX_FORMAT) as tar:
            for e in entries:
                info = tarfile.TarInfo(e['path']); info.size = e['bytes']; info.mode = e['mode']
                info.mtime = 0; info.uid = info.gid = 0; info.uname = info.gname = ''
                with (repo / e['path']).open('rb') as source: tar.addfile(info, source)
    writer.finish_part()
    groups = {}
    for root in ROOTS:
        group = [e for e in entries if e['path'].startswith(root + '/')]
        groups[root] = {'files': len(group), 'bytes': sum(e['bytes'] for e in group),
                        'materialized_symlinks': sum(e['original_type'] != 'regular' for e in group)}
    meta = {'version': VERSION, 'files': len(entries), 'uncompressed_file_bytes': sum(e['bytes'] for e in entries),
            'compressed_bytes': writer.total, 'compressed_sha256': writer.full_hash.hexdigest(),
            'max_part_bytes': PART_BYTES, 'parts': writer.parts, 'campaigns': groups,
            'manifest_sha256': sha(package / 'manifest.json'), 'exclusions_sha256': sha(package / 'exclusions.json'),
            'secret_scan_sha256': sha(package / 'secret-scan.json'), 'tool_sha256': sha(__file__),
            'excluded_files': len(excluded), 'excluded_bytes': sum(e['bytes'] for e in excluded),
            'deterministic_encoding': 'sorted paths, regular files only, original mode, tar uid/gid/mtime0, blank owner names, gzip mtime0/blank filename, level6, contiguous20MiB parts'}
    save_new(package / 'package.json', meta)
    unchanged = verify_restored(package, repo, live_original=True)
    after_entries, after_excluded = inventory(repo)
    if entries != after_entries or excluded != after_excluded:
        raise ValueError('Source inventory changed while packaging')
    unchanged['pre_post_inventory_and_hashes_equal'] = True
    save_new(package / 'live-source-unchanged.json', unchanged)
    return meta


def portable_replay(package, destination, output):
    """Run archived replay builder with scoped in-memory original-root relocation.

    No raw/frozen JSON is rewritten. This is not a promise that the historical
    model runner or full audit is relocatable: /private/tmp source paths remain.
    """
    if output.is_relative_to(destination): raise ValueError('Replay output must be outside restored evidence tree')
    verify_restored(package, destination)
    original = Path(read(package / 'manifest.json')['original_repo_root'])
    root = destination / ROOTS[1]
    builder = root / 'demo.build-sources/build_replay.py'
    # Discovery marker only; all actual replay evidence comes from restored files.
    (destination / 'benchmark/architecture/four_dp_poc_v2').mkdir(parents=True, exist_ok=True)
    old_init = Path.__init__
    routed = set()
    prefix, replacement = str(original), str(destination)
    def relocated(path, *args, **kwargs):
        old_init(path, *args, **kwargs)
        raw = str(path)
        if raw == prefix or raw.startswith(prefix + '/'):
            routed.add(raw)
            old_init(path, replacement + raw[len(prefix):])
    old_argv = sys.argv
    old_bytecode = sys.dont_write_bytecode
    try:
        sys.dont_write_bytecode = True
        Path.__init__ = relocated
        sys.argv = [str(builder), str(root), '--output', str(output), '--include-calibration']
        runpy.run_path(str(builder), run_name='__main__')
    finally:
        Path.__init__ = old_init; sys.argv = old_argv; sys.dont_write_bytecode = old_bytecode
    metadata = read(output.with_suffix('.build.json'))
    if metadata['runs'] != 229 or metadata['provider_responses'] != 1274 or metadata['assets'] != 8 or metadata['warnings']:
        raise ValueError('Portable replay population/assets/warnings differ')
    def embedded(path):
        text = path.read_text()
        match = re.search(r'<script[^>]*type="application/json"[^>]*>(.*?)</script>', text, re.S)
        if not match: raise ValueError('Replay embedded evidence absent')
        return json.loads(match.group(1))
    before, after = embedded(root / 'demo.html'), embedded(output)
    if {k: v['data_uri'] for k, v in before['assets'].items()} != {k: v['data_uri'] for k, v in after['assets'].items()}:
        raise ValueError('Replay original pixel bytes differ')
    expected_runs = {r['record']['run_id']: r for r in before['runs']}
    for r in after['runs']:
        previous = expected_runs[r['record']['run_id']]
        for key in ['record', 'actual_input']:
            if r[key] != previous[key]: raise ValueError('Replay actual record/input differs')
        for key, payload in [('model_requests', 'actual_request_with_image_references'), ('provider_responses', 'actual_provider_response')]:
            if [x[payload] for x in r[key]] != [x[payload] for x in previous[key]]:
                raise ValueError('Replay actual model request/response differs')
    verify_restored(package, destination)
    return {'status': 'PASS', 'runs': 229, 'requests_and_responses': 1274, 'png_assets': 8,
            'routed_original_root_paths': len(routed), 'output': str(output), 'output_sha256': sha(output),
            'no_original_raw_or_frozen_json_rewrite': True, 'original_repo_read_fallback': False,
            'scope': 'Archived exact replay builder, original-root read paths relocated in memory to verified restored tree. Restored evidence hashes rechecked after replay. No model/candidate execution. Full historical finalizer/audit relocation is not asserted.'}


def portable_numeric(package, destination):
    """Recompute final core metrics with archived analyzer, frozen oracles and raw runs."""
    verify_restored(package, destination)
    original = Path(read(package / 'manifest.json')['original_repo_root'])
    root = destination / ROOTS[1]
    pins = read(root / 'analysis/final-analysis-source-manifest.json')['files']
    key = 'benchmark/architecture/four_dp_poc_v2/analyze.py'
    archived_path = Path(pins[key]['snapshot'])
    analyzer_path = destination / archived_path.relative_to(original)
    analyzer_base = analyzer_path.parent
    for name in ['analyze.py', 'timing.py', 'evidence.py', 'score_dp01.py', 'score_dp02.py', 'score_dp04.py', 'score_dp05.py']:
        source_key = 'benchmark/architecture/four_dp_poc_v2/' + name
        if sha(analyzer_base / name) != pins[source_key]['sha256']:
            raise ValueError('Archived numeric analyzer SHA mismatch')
    old_init, old_path, old_bytecode = Path.__init__, list(sys.path), sys.dont_write_bytecode
    routed = set()
    prefix, replacement = str(original), str(destination)
    def relocated(path, *args, **kwargs):
        old_init(path, *args, **kwargs)
        raw = str(path)
        if raw == prefix or raw.startswith(prefix + '/'):
            routed.add(raw)
            old_init(path, replacement + raw[len(prefix):])
    comparisons = []
    selection = {x['run_id'] for x in read(root / 'effective-main-selection.json')['selection']}
    groups = collections.defaultdict(list)
    totals = collections.Counter()
    phase_checks = []
    try:
        sys.dont_write_bytecode = True
        Path.__init__ = relocated
        sys.path.insert(0, str(analyzer_base))
        spec = importlib.util.spec_from_file_location('archived_numeric_analyzer', str(analyzer_path))
        analyzer = importlib.util.module_from_spec(spec); spec.loader.exec_module(analyzer)
        import evidence
        evidence.REPO = destination
        from evidence import read_evaluator
        for marker in sorted(root.glob('*/MODEL-CALLS-AUTHORIZED.json')):
            phase = marker.parent; phase_counts = collections.Counter()
            for line in (phase / 'runs.jsonl').read_text().splitlines():
                if not line: continue
                record = json.loads(line)
                phase_counts['episodes'] += 1; phase_counts['calls'] += len(record['calls'])
                phase_counts['goal_failed_events'] += sum(e.get('kind') == 'GOAL_FAILED' for e in record['events'])
                phase_counts['episode_error_events'] += sum(e.get('kind') == 'EPISODE_ERROR' for e in record['events'])
                phase_counts['nonzero_process_exit_episodes'] += record['returncode'] != 0
                phase_counts['cost_usd'] += sum(c.get('cost_usd') or 0 for c in record['calls'])
                phase_counts['unknown_usage_calls'] += sum(c.get('cost_usd') is None for c in record['calls'])
                if record['run_id'] in selection or phase.name.startswith('tactic-'):
                    oracle = read_evaluator(record, Path('oracles') / ('dp' + record['dp'] + '.json'))
                    oracle = {x['id']: x for x in oracle} if isinstance(oracle, list) else oracle
                    row = analyzer.row(record, oracle.get(record['episode'], {}))
                    groups[(phase.name, row['dp'], row['variant'])].append(row)
            phase_checks.append({'phase': phase.name, **phase_counts})
            totals.update(phase_counts)
    finally:
        Path.__init__ = old_init; sys.path[:] = old_path; sys.dont_write_bytecode = old_bytecode
    master = read(root / 'final-measurement-summary.json')
    tolerances = {'time_ms': 0.000001, 'usd': 0.000000001}
    def same(actual, expected, label):
        if actual is None or expected is None:
            ok = actual == expected
        elif isinstance(actual, (int, float)) and isinstance(expected, (int, float)):
            ok = abs(actual - expected) <= (tolerances['usd'] if 'usd' in label else tolerances['time_ms'])
        else: ok = actual == expected
        if not ok: raise ValueError('Recomputed numeric difference: ' + label)
        return {'metric': label, 'actual': actual, 'archived': expected, 'status': 'PASS'}
    for (folder, dp, variant), rows in sorted(groups.items()):
        saved = next(x for x in read(root / folder / 'summary.json')['rows'] if x['dp'] == dp and x['variant'] == variant)
        aggregate = {k: sum(r[k] for r in rows) for k in ['qa01_n', 'qa01_expected', 'qa03_n', 'qa03_expected', 'qa02_correct', 'qa02_total', 'nr_correct', 'nr_total', 'nh_correct', 'nh_total', 'content_correct', 'content_total', 'model_calls', 'unknown_usage_calls']}
        n1, n3 = aggregate['qa01_n'], aggregate['qa03_n']
        aggregate.update(qa01_ms=sum(r['qa01_sum_ms'] for r in rows) / n1 if n1 else None,
                         qa03_ms=sum(r['qa03_sum_ms'] for r in rows) / n3 if n3 else None,
                         qa03_full_elapsed_ms=sum(r['qa03_full_elapsed_sum_ms'] for r in rows) / n3 if n3 else None,
                         receiver_shim_excluded_ms=sum(r['receiver_shim_excluded_sum_ms'] for r in rows) / n3 if n3 else None,
                         qa05_via_usd=sum(r['via_cost_usd'] for r in rows),
                         qa05_receiver_usd=sum(r['receiver_cost_usd'] for r in rows),
                         qa06_peak_rss_mb=max(r['rss_concurrent_peak_mb'] for r in rows),
                         errors=sum(r['functional_error'] for r in rows))
        checks = [same(value, saved[key], key) for key, value in aggregate.items()]
        comparisons.append({'phase_folder': folder, 'dp': dp, 'variant': variant, 'trials': len(rows), 'checks': checks})
    campaign = read(root / 'campaign-totals.json')
    for phase in phase_checks:
        saved = next(p for p in campaign['phases'] if Path(p['directory']).name == phase['phase'])
        for a, b in [('episodes', 'recorded_episode_trials'), ('calls', 'actual_call_count'), ('cost_usd', 'known_usage_total_usd'), ('goal_failed_events', 'goal_failed_events'), ('episode_error_events', 'episode_error_events'), ('nonzero_process_exit_episodes', 'nonzero_process_exit_episodes'), ('unknown_usage_calls', 'unknown_usage_calls')]:
            same(phase[a], saved[b], 'phase/' + a)
    same(totals['episodes'], 229, 'total/episodes'); same(totals['calls'], 1274, 'total/calls')
    same(totals['cost_usd'], campaign['usage_confirmed_usd'], 'total/cost_usd')
    effective_rows = [row for (folder, dp, variant), rows in groups.items() for row in rows if row['run_id'] in selection]
    same(len(effective_rows), 120, 'effective/trials')
    for metric, key in [('qa01_expected', 'qa01_expected'), ('qa01_n', 'qa01_observed'), ('qa03_expected', 'qa03_expected'), ('qa03_n', 'qa03_observed')]:
        same(sum(r[metric] for r in effective_rows), master[key], 'master/' + metric)
    for option in master['options']:
        rows = [r for r in effective_rows if (r['dp'], r['variant']) == (option['dp'], option['variant'])]
        saved = next(c for c in comparisons if c['dp'] == option['dp'] and c['variant'] == option['variant'] and not c['phase_folder'].startswith('tactic-'))
        by_metric = {c['metric']: c['actual'] for c in saved['checks']}
        for metric, expected in [('qa01_ms', option['qa01']['conditional_mean_ms']), ('qa03_ms', option['qa03']['conditional_via_mean_ms']), ('qa05_via_usd', option['qa05']['via_usd']), ('qa05_receiver_usd', option['qa05']['receiver_usd']), ('qa06_peak_rss_mb', option['qa06_same_peak']['concurrent_peak_mb'])]:
            same(by_metric[metric], expected, 'master/' + metric)
        tasks = option['qa04']['tasks']
        same(sum(x['count'] for x in tasks) / len(tasks), option['qa04']['mean_changed_operational_modules'], 'qa04/archived-task-count-mean')
    verify_restored(package, destination)
    return {'status': 'PASS', 'effective_trials': len(effective_rows), 'effective_options': 10,
            'tactic_or_parallel_diagnostic_trials': sum(len(v) for k, v in groups.items() if k[0].startswith('tactic-')),
            'all_actual_trials': totals['episodes'], 'all_actual_calls': totals['calls'],
            'all_actual_usage_usd': totals['cost_usd'], 'all_phase_counts': phase_checks,
            'routed_original_root_paths': len(routed), 'comparisons': comparisons,
            'qa04_scope': 'Archived actual task counts averaged and compared; temporary QA04 code changes/acceptance executables are not rerun.',
            'scope': 'Archived exact numeric analyzer row(), phase-frozen oracles/goal manifests, raw records. No raw edits, model/candidate execution, or network. All restored file hashes rechecked.'}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('action', choices=['build', 'verify', 'restore', 'verify-restored', 'verify-live', 'portable-replay', 'portable-numeric'])
    p.add_argument('--package', type=Path, required=True); p.add_argument('--repo', type=Path)
    default_repo = next(q for q in Path(__file__).resolve().parents if (q / 'benchmark/architecture/four_dp_poc_v2').is_dir())
    p.add_argument('--destination', type=Path, default=default_repo); p.add_argument('--output', type=Path)
    a = p.parse_args(); package = a.package.resolve()
    if a.action == 'build': result = build(a.repo.resolve(), package)
    elif a.action == 'verify': result = archive(package)
    elif a.action == 'restore': result = archive(package, a.destination.absolute())
    elif a.action == 'verify-restored': result = verify_restored(package, a.destination.absolute())
    elif a.action == 'verify-live': result = verify_restored(package, a.destination.absolute(), live_original=True)
    elif a.action == 'portable-replay': result = portable_replay(package, a.destination.absolute(), a.output.absolute())
    else: result = portable_numeric(package, a.destination.absolute())
    if a.output and a.action != 'portable-replay': save_new(a.output, result)
    print(json.dumps(result, ensure_ascii=False, sort_keys=True))


if __name__ == '__main__': main()
