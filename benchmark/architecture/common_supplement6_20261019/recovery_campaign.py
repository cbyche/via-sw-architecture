"""One-generation recovery of the failed slot and the 183 unstarted joint jobs.

The stopped parent remains immutable. Only a freshly prepared root may run.
"""
import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

import joint_campaign as joint

common = joint.common
PARENT = joint.RESULT / 'joint-formal-v1'
EXPECTED_FAILED = {'configuration': 'ABAA', 'episode': 'S12', 'rep': 3}


def pending(parent=PARENT):
    frozen = json.loads((parent / 'planned-jobs.json').read_text())
    if frozen != joint.jobs():
        raise RuntimeError('Parent planned order changed')
    rows, ended = joint.prior_progress(parent)
    if len(ended) != 258:
        raise RuntimeError('Expected exactly 258 ended parent identities')
    terminal = json.loads((parent / 'TECHNICAL-STOP.json').read_text())
    if joint.identity(terminal) != EXPECTED_FAILED or not terminal['technical_failure']:
        raise RuntimeError('Unexpected parent stop identity or status')
    failed = tuple(EXPECTED_FAILED.values())
    if failed not in ended:
        raise RuntimeError('Failed identity is not ended')
    selected = [job for job in frozen if tuple(joint.identity(job).values()) not in ended or joint.identity(job) == EXPECTED_FAILED]
    if len(selected) != 184 or sum(tuple(joint.identity(j).values()) == failed for j in selected) != 1:
        raise RuntimeError('Recovery must be the one failed slot plus 183 unstarted slots')
    return selected


def prepare(root, projection, parent=PARENT):
    selected = pending(parent)
    manifest = joint.prepare(root, projection)
    manifest['version'] = 'JOINT441_RECOVERY184_v1'
    manifest['parent'] = {
        'root': str(parent.resolve()),
        'freeze_sha256': common.sha(parent / 'freeze.json'),
        'planned_jobs_sha256': common.sha(parent / 'planned-jobs.json'),
        'campaign_progress_sha256': common.sha(parent / 'campaign-progress.jsonl'),
        'technical_stop_sha256': common.sha(parent / 'TECHNICAL-STOP.json'),
        'preserved_failed_identity': EXPECTED_FAILED,
        'certified_parent_rows_to_consider_for_pooling': 257,
    }
    manifest['jobs'] = selected
    manifest['total_episode_runs'] = 184
    manifest['recovery_scope'] = 'One uncertified slot rerun and 183 previously unstarted jobs; parent failure and its spend remain preserved'
    manifest['authorization'] = 'Human authorized exactly these 184 recovery jobs, synthetic DashScope input and total USD40 cap on 2026-10-20; exact freeze-bound MODEL-CALLS-AUTHORIZED.json still required'
    common.dump(root / 'freeze.json', manifest)
    common.dump(root / 'planned-jobs.json', selected)
    return manifest


def validate(root, parent=PARENT, large=False):
    manifest = common.validate(root, verify_large=large)
    if manifest['version'] != 'JOINT441_RECOVERY184_v1' or manifest['jobs'] != pending(parent):
        raise RuntimeError('Recovery jobs or version changed')
    references = {'freeze_sha256': 'freeze.json', 'planned_jobs_sha256': 'planned-jobs.json',
                  'campaign_progress_sha256': 'campaign-progress.jsonl', 'technical_stop_sha256': 'TECHNICAL-STOP.json'}
    if manifest['parent']['root'] != str(parent.resolve()):
        raise RuntimeError('Parent root changed')
    for field, filename in references.items():
        if manifest['parent'][field] != common.sha(parent / filename):
            raise RuntimeError('Parent evidence changed: ' + filename)
    if common.sha(manifest['block_planning']['projection_path']) != manifest['block_planning']['projection_sha256']:
        raise RuntimeError('Projection changed')
    if joint.budget_snapshot()['limit_usd'] != 40:
        raise RuntimeError('Budget cap changed')
    return manifest


def run(root):
    root = root.resolve()
    with common.controller_lock(joint.GLOBAL_CONTROLLER_ROOT), common.controller_lock(root):
        manifest = validate(root, large=True)
        auth = json.loads((root / 'MODEL-CALLS-AUTHORIZED.json').read_text())
        if auth['freeze_sha256'] != common.sha(root / 'freeze.json') or auth['allowed_jobs'] != [joint.identity(j) for j in manifest['jobs']]:
            raise RuntimeError('Exact recovery authorization missing or different')
        rows, done = joint.prior_progress(root)
        if any(r['state'] == 'TERMINATED' and (r.get('returncode') or r.get('technical_failure') or r.get('budget_stop')) for r in rows):
            raise RuntimeError('Recovery stop requires separate review')
        admitted = {r['block_id'] for r in rows if r['state'] == 'BLOCK_ADMITTED'}
        progress = root / 'campaign-progress.jsonl'
        for job in manifest['jobs']:
            key = tuple(joint.identity(job).values())
            if key in done:
                continue
            if any((root / name).exists() for name in ('TECHNICAL-STOP.json', 'BUDGET-STOP.json', 'METHOD-REVIEW-HOLD')):
                break
            validate(root)
            if job['block_id'] not in admitted:
                block = [j for j in manifest['jobs'] if j['block_id'] == job['block_id']]
                amounts = manifest['block_planning']['per_episode_conservative_usd']
                required = sum(amounts[j['population']][j['configuration']][j['episode']] for j in block)
                budget = joint.budget_snapshot()
                row = {'state': 'BLOCK_ADMITTED' if required <= budget['remaining_reservation_aware_usd'] else 'BLOCK_BUDGET_STOP',
                       'block_id': job['block_id'], 'remaining_jobs_in_block': len(block), 'conservative_planning_usd': required,
                       'budget': budget, 'at_ns': time.monotonic_ns()}
                joint.append(progress, row)
                if row['state'] == 'BLOCK_BUDGET_STOP':
                    common.dump(root / 'BUDGET-STOP.json', row)
                    break
                admitted.add(job['block_id'])
            harness = joint.COMMON / 'harness.py' if job['population'] == 'COMMON9' else joint.HERE / 'harness.py'
            command = [sys.executable, str(harness), '--root', str(root), '--configuration', job['configuration'],
                       '--episode', job['episode'], '--rep', str(job['rep'])]
            joint.append(progress, {**job, 'state': 'STARTED', 'at_ns': time.monotonic_ns(), 'command': command})
            start = time.monotonic_ns()
            process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            joint.append(progress, {**job, 'state': 'DISPATCHED', 'harness_pid': process.pid, 'at_ns': time.monotonic_ns()})
            stdout, stderr = process.communicate()
            summary = json.loads(stdout.strip().splitlines()[-1]) if process.returncode == 0 and stdout.strip() else {}
            row = {**job, 'state': 'TERMINATED', 'harness_pid': process.pid, 'returncode': process.returncode,
                   'stdout': stdout, 'stderr': stderr, 'technical_failure': summary.get('technical_failure'),
                   'budget_stop': summary.get('budget_stop'), 'budget': joint.budget_snapshot(),
                   'episode_elapsed_seconds': (time.monotonic_ns() - start) / 1e9, 'at_ns': time.monotonic_ns()}
            joint.append(progress, row)
            done.add(key)
            common.dump(root / 'status.json', {'controller_pid': os.getpid(), 'completed': len(done), 'planned': 184,
                                               'latest': row, 'budget': row['budget']})
            print(json.dumps(row, ensure_ascii=False), flush=True)
            if process.returncode or row['technical_failure']:
                common.dump(root / 'TECHNICAL-STOP.json', row)
                break
            if row['budget_stop']:
                common.dump(root / 'BUDGET-STOP.json', row)
                break


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=['prepare', 'validate', 'run', 'pending'])
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--projection', type=Path)
    args = parser.parse_args()
    if args.action == 'pending':
        print(json.dumps({'jobs': len(pending()), 'first': joint.identity(pending()[0])}))
    elif args.action == 'prepare':
        if not args.projection:
            parser.error('--projection required')
        print(json.dumps({'prepared': len(prepare(args.root, args.projection)['jobs']), 'authorized': False}))
    elif args.action == 'validate':
        print(json.dumps({'valid': bool(validate(args.root))}))
    else:
        run(args.root)


if __name__ == '__main__':
    main()
