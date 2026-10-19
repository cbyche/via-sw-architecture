"""Contract20 balanced441 controller. Preparation never authorizes model calls.

Uses the unchanged method-v3 harnesses, one fresh process per episode, one
top-level paid-controller lock, and the existing reservation-aware global ledger.
"""
import argparse
import fcntl
import json
import math
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
COMMON = HERE.parent / 'common9_poc_20261019'
REPO = HERE.parents[2]
sys.path.append(str(COMMON))
import campaign as common

RESULT = REPO / 'results/architecture-evaluation/current/common-supplement6-20261019'
CONTRACT20 = REPO / 'docs/architecture/12-decisions/decision-packages/seven-problem-review/20-common-supplement-evaluation-contract.md'
CONFIGS = common.CONFIGS
SUPPLEMENT_ORDER = ['S80', 'S83', 'S81', 'S84', 'S82', 'S85']
GLOBAL_CONTROLLER_ROOT = common.RESULT / 'shared-paid-controller'


def identity(job):
    return {key: job[key] for key in ('configuration', 'episode', 'rep')}


def jobs():
    output = []
    primary = common.jobs()
    for start in range(0, len(primary), 7):
        block = primary[start:start + 7]
        rep, episode = block[0]['rep'], block[0]['episode']
        block_id = f'PRIMARY-r{rep}-{episode}'
        output.extend({**job, 'population': 'COMMON9', 'block_id': block_id} for job in block)
        if rep <= 3 and episode in common.EPISODES[:6]:
            offset = 2 * (rep - 1)
            order = SUPPLEMENT_ORDER[offset:] + SUPPLEMENT_ORDER[:offset]
            supplemental = order[common.EPISODES.index(episode)]
            shift = ((rep - 1) + int(supplemental[1:]) - 80) % 7
            configs = CONFIGS[shift:] + CONFIGS[:shift]
            output.extend({'configuration': cfg, 'episode': supplemental, 'rep': rep, 'population': 'COMMON_SUPPLEMENT6', 'block_id': f'SUPPLEMENT-r{rep}-{supplemental}'} for cfg in configs)
    assert [identity(job) for job in output if job['population'] == 'COMMON9'] == primary
    assert len(output) == 441 and len({tuple(identity(job).values()) for job in output}) == 441
    return output


def budget_snapshot(path=None):
    path = path or common.RESULT / 'budget.json'
    with path.open() as handle:
        fcntl.flock(handle, fcntl.LOCK_SH)
        ledger = json.load(handle)
    if ledger['limit_usd'] != 40:
        raise RuntimeError('This plan requires the existing explicit USD40 cap; no cap mutation is performed')
    known = sum(row['amount_usd'] for row in ledger['calls'].values() if row['state'] not in ('RESERVED', 'UNKNOWN'))
    reserved = sum(row['amount_usd'] for row in ledger['calls'].values() if row['state'] == 'RESERVED')
    unknown = sum(row['amount_usd'] for row in ledger['calls'].values() if row['state'] == 'UNKNOWN')
    return {'limit_usd': ledger['limit_usd'], 'known_usd': known, 'inflight_reservation_usd': reserved, 'unknown_reservation_usd': unknown, 'remaining_reservation_aware_usd': ledger['limit_usd'] - known - reserved - unknown}


def block_requirement(manifest, block_id):
    block = [job for job in manifest['jobs'] if job['block_id'] == block_id]
    if len(block) != 7 or {job['configuration'] for job in block} != set(CONFIGS):
        raise RuntimeError('Not a complete seven-configuration block')
    amounts = manifest['block_planning']['per_episode_conservative_usd']
    values = [amounts[job['population']][job['configuration']][job['episode']] for job in block]
    if any(not isinstance(value, (float, int)) or not math.isfinite(value) or value < 0 for value in values):
        raise ValueError('Nonfinite or negative planning amount')
    return sum(values)


def source_files():
    files = common.source_files() + [CONTRACT20]
    files += [p for p in HERE.rglob('*') if p.is_file() and '__pycache__' not in p.parts]
    return sorted(set(files))


def prepare(root, projection):
    root = root.resolve()
    if (root / 'freeze.json').exists():
        raise FileExistsError('Preserve previous freeze; choose a fresh root')
    plan = json.loads(projection.read_text())
    if plan['status'] != 'PLANNING_NOT_PROVIDER_TOKEN_MEASUREMENT':
        raise ValueError('Unexpected planning artifact')
    sources = []
    for path in source_files():
        relative = path.relative_to(REPO)
        frozen = root / 'frozen/source' / relative
        frozen.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, frozen)
        sources.append({'path': str(path), 'relativePath': str(relative), 'sha256': common.sha(path), 'frozen_path': str(frozen)})
    binary = root / 'frozen/bin/common9_poc'
    binary.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(common.CRATE / 'target/debug/common9_poc', binary)
    font = json.loads((HERE / 'assets/render-manifest.json').read_text())
    font_path = Path(font['font_path'])
    if common.sha(font_path) != font['font_sha256']:
        raise RuntimeError('Pinned rendering font changed')
    dependency = {'path': str(font_path), 'sha256': common.sha(font_path), 'bytes': font_path.stat().st_size, 'mtime_ns': font_path.stat().st_mtime_ns, 'large_weight': False}
    projection_copy = root / 'planning/combined-cost-projection.json'
    projection_copy.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(projection, projection_copy)
    manifest = {'version': 'COMMON9_PLUS_SUPPLEMENT6_JOINT441_v1', 'purpose': 'FORMAL_REVIEW_CANDIDATE', 'authorization': 'No API permission. Separate human scope, parent freeze review and exact MODEL-CALLS-AUTHORIZED.json required.', 'created_utc': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()), 'sources': sources, 'binary': {'path': str(binary), 'sha256': common.sha(binary)}, 'external_dependencies': common.external_dependencies() + [dependency], 'jobs': jobs(), 'total_episode_runs': 441, 'unique_configurations': CONFIGS, 'common_k': 2, 'per_episode_max_cloud_calls': 100, 'populations': {'COMMON9': {'episodes': 315, 'repetitions': 5, 'per_rep': {'responses': 43, 'goals': 23, 'bindings': 15}}, 'COMMON_SUPPLEMENT6': {'episodes': 126, 'repetitions': 3, 'per_rep': {'responses': 12, 'goals': 8, 'bindings': 6}}}, 'ledger_path': str(common.RESULT / 'budget.json'), 'global_controller_lock_root': str(GLOBAL_CONTROLLER_ROOT), 'budget_snapshot_at_preparation': budget_snapshot(), 'block_planning': {'policy': 'Before every previously unstarted complete7 block, require the sum of frozen text-byte/outputcap/imagebyte MAX3 planning amounts for all7 configurations. Never choose cheaper members. This is not a provider cost bound; individual reservations still stop unexpected midblock exhaustion.', 'projection_path': str(projection_copy), 'projection_sha256': common.sha(projection_copy), 'per_episode_conservative_usd': plan['per_episode_block_planning_usd']}}
    common.dump(root / 'freeze.json', manifest)
    common.dump(root / 'planned-jobs.json', manifest['jobs'])
    return manifest


def validate(root, large=False):
    manifest = common.validate(root, verify_large=large)
    if manifest['version'] != 'COMMON9_PLUS_SUPPLEMENT6_JOINT441_v1' or manifest['jobs'] != jobs():
        raise RuntimeError('Joint441 plan differs from preregistered order')
    if common.sha(manifest['block_planning']['projection_path']) != manifest['block_planning']['projection_sha256']:
        raise RuntimeError('Frozen planning artifact changed')
    for job in manifest['jobs'][::7]:
        if block_requirement(manifest, job['block_id']) < 0:
            raise RuntimeError('Invalid negative planning amount')
    return manifest


def append(path, row):
    with path.open('a') as handle:
        handle.write(json.dumps(row, ensure_ascii=False) + '\n')
        handle.flush()
        os.fsync(handle.fileno())


def prior_progress(root):
    path = root / 'campaign-progress.jsonl'
    rows = [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []
    started = {tuple(identity(row).values()) for row in rows if row['state'] == 'STARTED'}
    done = {tuple(identity(row).values()) for row in rows if row['state'] == 'TERMINATED'}
    if len(started) != sum(row['state'] == 'STARTED' for row in rows) or len(done) != sum(row['state'] == 'TERMINATED' for row in rows) or not done <= started:
        raise RuntimeError('Duplicate or unstarted terminated identity; preserve progress for explicit review')
    if started - done:
        common.dump(root / 'ORPHANED-STARTED-JOBS.json', {'identities': sorted(started - done), 'reason': 'No automatic rerun of started identity; original raw/spend retained and explicit recovery required'})
        raise RuntimeError('Orphaned started identity')
    return rows, done


def run(root):
    root = root.resolve()
    # Lock acquisition does not authorize dispatch. Both locks are released on
    # success, host failure, cap/technical stop, validation exception or interrupt.
    with common.controller_lock(GLOBAL_CONTROLLER_ROOT), common.controller_lock(root):
        manifest = validate(root, True)
        auth = json.loads((root / 'MODEL-CALLS-AUTHORIZED.json').read_text())
        if auth['freeze_sha256'] != common.sha(root / 'freeze.json') or auth['allowed_jobs'] != [identity(job) for job in manifest['jobs']]:
            raise RuntimeError('Exact441 authorization missing or different')
        rows, done = prior_progress(root)
        if any(row['state'] == 'TERMINATED' and (row.get('returncode') or row.get('technical_failure') or row.get('budget_stop')) for row in rows):
            raise RuntimeError('Previous failed/technical/cap stop requires explicit recovery; no automatic continuation')
        checked_blocks = {row['block_id'] for row in rows if row['state'] == 'BLOCK_ADMITTED'}
        progress = root / 'campaign-progress.jsonl'
        for job in manifest['jobs']:
            key = tuple(identity(job).values())
            if key in done:
                continue
            if (root / 'METHOD-REVIEW-HOLD').exists() or (root / 'TECHNICAL-STOP.json').exists() or (root / 'BUDGET-STOP.json').exists():
                break
            validate(root)
            if job['block_id'] not in checked_blocks:
                snapshot = budget_snapshot()
                required = block_requirement(manifest, job['block_id'])
                row = {'state': 'BLOCK_ADMITTED' if required <= snapshot['remaining_reservation_aware_usd'] else 'BLOCK_BUDGET_STOP', 'block_id': job['block_id'], 'conservative_planning_usd': required, 'budget': snapshot, 'at_ns': time.monotonic_ns()}
                append(progress, row)
                if row['state'] == 'BLOCK_BUDGET_STOP':
                    common.dump(root / 'BUDGET-STOP.json', row)
                    break
                checked_blocks.add(job['block_id'])
            harness = COMMON / 'harness.py' if job['population'] == 'COMMON9' else HERE / 'harness.py'
            command = [sys.executable, str(harness), '--root', str(root), '--configuration', job['configuration'], '--episode', job['episode'], '--rep', str(job['rep'])]
            append(progress, {**job, 'state': 'STARTED', 'at_ns': time.monotonic_ns(), 'command': command})
            launch_ns = time.monotonic_ns()
            process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            append(progress, {**job, 'state': 'DISPATCHED', 'harness_pid': process.pid, 'at_ns': time.monotonic_ns()})
            stdout, stderr = process.communicate()
            summary = json.loads(stdout.strip().splitlines()[-1]) if process.returncode == 0 and stdout.strip() else {}
            row = {**job, 'state': 'TERMINATED', 'harness_pid': process.pid, 'returncode': process.returncode, 'stdout': stdout, 'stderr': stderr, 'technical_failure': summary.get('technical_failure'), 'budget_stop': summary.get('budget_stop'), 'budget': budget_snapshot(), 'episode_elapsed_seconds': (time.monotonic_ns() - launch_ns) / 1e9, 'at_ns': time.monotonic_ns()}
            append(progress, row)
            done.add(key)
            common.dump(root / 'status.json', {'controller_pid': os.getpid(), 'completed': len(done), 'planned': 441, 'latest': row, 'budget': row['budget'], 'planned_counts_unchanged': True})
            print(json.dumps(row, ensure_ascii=False), flush=True)
            if process.returncode or row['technical_failure']:
                common.dump(root / 'TECHNICAL-STOP.json', row)
                break
            if row['budget_stop']:
                common.dump(root / 'BUDGET-STOP.json', row)
                break


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=['plan', 'prepare', 'validate', 'run'])
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--projection', type=Path)
    args = parser.parse_args()
    if args.action == 'plan':
        args.root.mkdir(parents=True, exist_ok=True)
        if (args.root / 'planned-jobs.json').exists():
            raise FileExistsError('Do not overwrite previous plan')
        common.dump(args.root / 'planned-jobs.json', jobs())
        print(json.dumps({'planned': 441, 'authorization': False}))
    elif args.action == 'prepare':
        if not args.projection:
            parser.error('--projection required')
        print(json.dumps({'prepared': len(prepare(args.root, args.projection)['jobs']), 'authorization': False}))
    elif args.action == 'validate':
        print(json.dumps({'valid': bool(validate(args.root))}))
    else:
        run(args.root)


if __name__ == '__main__':
    main()
