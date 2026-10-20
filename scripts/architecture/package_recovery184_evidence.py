#!/usr/bin/env python3
"""Preserve the exact recovery generation without repackaging the parent.

No model or candidate is launched. Compiled binaries, environments and secrets
remain local; frozen source, raw calls, failures and reports are lossless.
"""
import argparse
import hashlib
import json
import os
import stat
import tarfile
from pathlib import Path, PurePosixPath

import package_common9_evidence as base

REPO = Path(__file__).resolve().parents[2]
ROOT = REPO / 'results/architecture-evaluation/current/common-supplement6-20261019/joint-recovery184-v1'


def require(value, message):
    if not value:
        raise ValueError(message)


def inventory():
    require(ROOT.is_dir() and (ROOT / 'freeze.json').is_file(), 'Recovery root/freeze absent')
    require((ROOT / 'status.json').is_file() and not (ROOT / 'TECHNICAL-STOP.json').exists()
            and not (ROOT / 'BUDGET-STOP.json').exists(), 'Recovery has not completed cleanly')
    status = json.loads((ROOT / 'status.json').read_text())
    require(status.get('completed') == status.get('planned') == 184, 'Incomplete recovery')
    rows = []
    for folder, dirs, files in os.walk(ROOT, followlinks=False):
        dirs[:] = sorted(d for d in dirs if d not in ('__pycache__', 'target', '.venv', 'venv', 'cache'))
        for name in sorted(files):
            path = Path(folder) / name
            relative = path.relative_to(REPO)
            if path.is_symlink() or base.excluded_reason(relative) or '/frozen/bin/' in '/' + str(relative):
                continue
            require(path.is_file(), 'Nonregular evidence file: ' + str(relative))
            base.check_no_secret(path, str(relative))
            before = base.stamp(path)
            row = {'path': str(relative), 'sha256': base.digest(path), **before}
            require(before == base.stamp(path), 'Changing source during inventory: ' + str(relative))
            rows.append(row)
    require(len(rows) > 100 and len({r['path'] for r in rows}) == len(rows), 'Missing or duplicate recovery evidence')
    return {'version': 'RECOVERY184_EVIDENCE_INVENTORY_v1', 'root': str(ROOT.relative_to(REPO)),
            'files': rows, 'logical_bytes': sum(r['size'] for r in rows),
            'scope': 'Exact new recovery generation only; original stopped258 evidence remains separately preserved',
            'excluded': ['frozen/bin/common9_poc', 'model weights', 'environments', 'secrets']}


def publish(out):
    require(not out.exists(), 'Choose an absent evidence output directory')
    info = inventory()
    out.mkdir(parents=True)
    base.dump(out / 'inventory.json', info)
    writer = base.SplitWriter(out, base.LIMIT)
    with tarfile.open(fileobj=writer, mode='w|gz', format=tarfile.PAX_FORMAT) as archive:
        archive.add(out / 'inventory.json', arcname='inventory.json', recursive=False)
        for row in info['files']:
            path = REPO / row['path']
            require(base.stamp(path) == {k: row[k] for k in ('size', 'mtime_ns', 'mode')}, 'Source changed before archive')
            with path.open('rb') as source:
                member = archive.gettarinfo(str(path), arcname=row['path'])
                archive.addfile(member, source)
            require(base.stamp(path) == {k: row[k] for k in ('size', 'mtime_ns', 'mode')}, 'Source changed during archive')
    parts = writer.finish()
    parts.update({'inventory_sha256': base.digest(out / 'inventory.json'), 'file_count': len(info['files'])})
    base.dump(out / 'chunk-manifest.json', parts)
    return verify(out)


def verify(out):
    manifest = json.loads((out / 'chunk-manifest.json').read_text())
    info = json.loads((out / 'inventory.json').read_text())
    require(base.digest(out / 'inventory.json') == manifest['inventory_sha256'], 'Inventory hash mismatch')
    expected = {row['path']: row for row in info['files']}
    require(len(expected) == manifest['file_count'], 'Inventory count mismatch')
    whole = hashlib.sha256()
    paths = []
    for part in manifest['parts']:
        path = out / part['name']
        require(path.is_file() and not path.is_symlink() and 0 < path.stat().st_size == part['bytes'] <= base.LIMIT,
                'Part size/type mismatch')
        require(base.digest(path) == part['sha256'], 'Part digest mismatch')
        paths.append(path)
        with path.open('rb') as source:
            for chunk in iter(lambda: source.read(1024 * 1024), b''):
                whole.update(chunk)
    require(whole.hexdigest() == manifest['archive_sha256'], 'Whole archive digest mismatch')
    reader = base.ConcatReader(paths)
    seen = set()
    try:
        with tarfile.open(fileobj=reader, mode='r|gz') as archive:
            for member in archive:
                require(member.name not in seen and base.safe_relative(member.name) and member.isfile(), 'Unsafe/duplicate archive member')
                seen.add(member.name)
                if member.name == 'inventory.json':
                    require(archive.extractfile(member).read() == (out / 'inventory.json').read_bytes(), 'Inventory payload mismatch')
                    continue
                require(member.name in expected, 'Unlisted archive member')
                row = expected[member.name]
                require(member.size == row['size'] and member.mode == row['mode'], 'Member size/mode mismatch')
                digest = hashlib.sha256()
                source = archive.extractfile(member)
                for chunk in iter(lambda: source.read(1024 * 1024), b''):
                    digest.update(chunk)
                require(digest.hexdigest() == row['sha256'], 'Member content digest mismatch')
    finally:
        reader.close()
    require(seen == set(expected) | {'inventory.json'}, 'Archive membership mismatch')
    return {'verified_files': len(expected), 'archive_sha256': manifest['archive_sha256'],
            'archive_bytes': manifest['archive_bytes'], 'parts': len(paths), 'actual_api_calls': 0}


def restore(bundle, out):
    checked = verify(bundle)
    require(not out.exists(), 'Restore destination must be absent')
    out.mkdir(parents=True)
    manifest = json.loads((bundle / 'chunk-manifest.json').read_text())
    info = json.loads((bundle / 'inventory.json').read_text())
    expected = {row['path']: row for row in info['files']}
    reader = base.ConcatReader([bundle / part['name'] for part in manifest['parts']])
    try:
        with tarfile.open(fileobj=reader, mode='r|gz') as archive:
            for member in archive:
                if member.name == 'inventory.json':
                    continue
                require(member.name in expected and member.isfile(), 'Unexpected restore member')
                destination = out / member.name
                destination.parent.mkdir(parents=True, exist_ok=True)
                with destination.open('xb') as target:
                    source = archive.extractfile(member)
                    for chunk in iter(lambda: source.read(1024 * 1024), b''):
                        target.write(chunk)
                os.chmod(destination, member.mode & 0o777)
    finally:
        reader.close()
    for name, row in expected.items():
        path = out / name
        require(path.is_file() and path.stat().st_size == row['size'] and base.digest(path) == row['sha256']
                and stat.S_IMODE(path.stat().st_mode) == row['mode'], 'Restored file mismatch: ' + name)
    return {**checked, 'restored_files': len(expected), 'destination': str(out)}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=['publish', 'verify', 'restore'])
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--out', type=Path)
    args = parser.parse_args()
    if args.action == 'publish':
        result = publish(args.bundle.resolve())
    elif args.action == 'verify':
        result = verify(args.bundle.resolve())
    else:
        if not args.out:
            parser.error('--out is required for restore')
        result = restore(args.bundle.resolve(), args.out.resolve())
    print(json.dumps(result, ensure_ascii=False))


if __name__ == '__main__':
    main()
