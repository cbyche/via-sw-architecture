"""No API: finite joint order/budget/resume and actual OS-lock acceptance."""
import argparse
import json
import subprocess
import sys
import tempfile
import time
from pathlib import Path

import joint_campaign as joint
import mechanism


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--lock-child', choices=['hold', 'fail', 'once'])
    args = parser.parse_args()
    if args.lock_child:
        with joint.common.controller_lock(args.root):
            if args.lock_child == 'hold':
                (args.root / 'HELD').write_text('actual OS flock held\n')
                while not (args.root / 'RELEASE').exists():
                    time.sleep(.02)
            elif args.lock_child == 'fail':
                raise ValueError('Intentional finite-test failure inside lock')
        return
    args.root.mkdir(parents=True, exist_ok=False)
    checks = []
    def check(name, condition):
        assert condition, name
        checks.append(name)
    plan = joint.jobs()
    check('primary_relative_order_unchanged', [joint.identity(j) for j in plan if j['population'] == 'COMMON9'] == joint.common.jobs())
    check('all441_unique', len(plan) == len({tuple(joint.identity(j).values()) for j in plan}) == 441)
    amounts = {population: {cfg: {episode: .5 for episode in joint.common.EPISODES + ['S80', 'S81', 'S82', 'S83', 'S84', 'S85']} for cfg in joint.CONFIGS} for population in ['COMMON9', 'COMMON_SUPPLEMENT6']}
    manifest = {'jobs': plan, 'block_planning': {'per_episode_conservative_usd': amounts}}
    check('block_requires_all7_not_cheapest', joint.block_requirement(manifest, plan[0]['block_id']) == 3.5)
    amounts['COMMON9']['AAAA']['S02'] = float('nan')
    try:
        joint.block_requirement(manifest, plan[0]['block_id'])
        raise AssertionError('Accepted NaN')
    except ValueError:
        checks.append('nonfinite_block_planning_rejected')
    ledger = args.root / 'synthetic-budget.json'
    ledger.write_text(json.dumps({'limit_usd': 40, 'calls': {'old': {'state': 'USAGE_RECORDED', 'amount_usd': 1}, 'unknown': {'state': 'UNKNOWN', 'amount_usd': 2}, 'inflight': {'state': 'RESERVED', 'amount_usd': 3}}}))
    check('known_unknown_inflight_all_reduce_remaining', joint.budget_snapshot(ledger)['remaining_reservation_aware_usd'] == 34)
    orphan = args.root / 'orphan-case'
    orphan.mkdir()
    joint.append(orphan / 'campaign-progress.jsonl', {**plan[0], 'state': 'STARTED'})
    try:
        joint.prior_progress(orphan)
        raise AssertionError('Orphan accepted')
    except RuntimeError:
        check('orphan_preserved_no_automatic_rerun', (orphan / 'ORPHANED-STARTED-JOBS.json').exists())
    joint.append(orphan / 'campaign-progress.jsonl', {**plan[0], 'state': 'TERMINATED', 'returncode': 1})
    rows, done = joint.prior_progress(orphan)
    check('failed_terminal_preserved_not_reexecuted_identity', len(done) == 1 and rows[-1]['returncode'] == 1)
    joint.append(orphan / 'campaign-progress.jsonl', {**plan[0], 'state': 'TERMINATED'})
    try:
        joint.prior_progress(orphan)
        raise AssertionError('Duplicate terminal accepted')
    except RuntimeError:
        checks.append('duplicate_termination_rejected')
    lockroot = args.root / 'actual-lock'
    command = [sys.executable, str(Path(__file__).resolve()), '--root', str(lockroot), '--lock-child']
    holder = subprocess.Popen(command + ['hold'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        deadline = time.monotonic() + 10
        while not (lockroot / 'HELD').exists() and time.monotonic() < deadline:
            time.sleep(.02)
        assert (lockroot / 'HELD').exists()
        duplicate = subprocess.run(command + ['once'], capture_output=True, text=True)
        check('simultaneous_controller_nonblocking_rejected', duplicate.returncode != 0 and 'already active' in duplicate.stderr)
        (args.root / 'duplicate-stderr.txt').write_text(duplicate.stderr)
    finally:
        (lockroot / 'RELEASE').write_text('release fixture holder\n')
        out, err = holder.communicate(timeout=10)
        (args.root / 'holder-output.txt').write_text(out + err)
    check('holder_success_released', holder.returncode == 0)
    failure = subprocess.run(command + ['fail'], capture_output=True, text=True)
    (args.root / 'intentional-failure-stderr.txt').write_text(failure.stderr)
    check('intentional_exception_preserved', failure.returncode != 0 and 'Intentional finite-test' in failure.stderr)
    reacquired = subprocess.run(command + ['once'], capture_output=True, text=True)
    check('exception_release_reacquires', reacquired.returncode == 0)
    # Synthetic trace demonstrates extraction logic, never asserted model work.
    synthetic = {'run_id': 'FINITE', 'episode': 'S80', 'configuration': 'ABAA', 'rep': 1, 'phase': 'FORMAL', 'end_ns': 100, 'events': [{'kind': 'INPUT_CAPTURED', 'observed_ns': 15, 'data': {'stimulus_id': 'S80-U02'}}, {'kind': 'OPERATION_START', 'observed_ns': 1, 'goal_ids': ['input:S80-U01'], 'data': {'node_id': 'a', 'stage': 'meaning', 'role': 'VIA'}}, {'kind': 'OPERATION_END', 'observed_ns': 25, 'goal_ids': ['input:S80-U01'], 'data': {'node_id': 'a', 'stage': 'meaning', 'role': 'VIA'}}], 'calls': [{'call_uuid': 'x', 'stage': 'meaning', 'role': 'VIA', 'actual_api_sent': True, 'status': 'COMPLETED', 'http_dispatch_registered_ns': 5, 'terminal_ns': 25, 'resource_slot': 0, 'causal_trace': {'goal_ids': ['input:S80-U01']}}, {'call_uuid': 'y', 'stage': 'meaning', 'role': 'VIA', 'actual_api_sent': True, 'status': 'COMPLETED', 'http_dispatch_registered_ns': 18, 'terminal_ns': 30, 'resource_slot': 1, 'causal_trace': {'goal_ids': ['input:S80-U02']}}]}
    extracted = mechanism.extract(synthetic)
    check('activation_uses_actual_owned_model_overlap', extracted['dp2']['positive_actual_cloud_intervals'][0]['overlap_ns'] == 7 and extracted['dp2']['activation'] == 'PROVEN_ACTUAL_MODEL_OVERLAP')
    check('occupancy_at_capture_actual1_not_assumed2', extracted['dp2']['active_dispatched_cloud_calls_at_capture_lower_bound'] == 1 and extracted['dp2']['exact_free_K2_capacity'].startswith('UNRESOLVED'))
    check('raw_capture_clock_explicit_proxy', extracted['dp2']['u2_capture_clock'].startswith('RAW_INPUT_CAPTURED'))
    synthetic['calls'].append({'call_uuid': 'cancelled', 'stage': 'meaning', 'actual_api_sent': False, 'status': 'CANCELLED_NO_API', 'slot_acquired_ns': 10, 'terminal_ns': 40, 'finite_fixture_actual_release_ns': 12})
    cancelled_extract = mechanism.extract(synthetic)
    check('cancelled_outer_terminal_never_proves_slot_ownership', cancelled_extract['dp2']['known_acquired_slots_at_capture_lower_bound'] == 0 and 'cancelled' in cancelled_extract['dp2']['noncompleted_acquired_records_unresolved'])
    synthetic['events'].append({'kind': 'GROUNDING_NATIVE_CANDIDATES', 'observed_ns': 1, 'data': {}})
    synthetic['calls'].append({'call_uuid': 'parser', 'stage': 'grounding.local_screen_parser', 'local_spawn_admitted_ns': 3, 'local_parse_dispatch_admitted_ns': None})
    check('native_helper_spawn_prevents_skip_claim', mechanism.extract(synthetic)['dp3']['native_skip_observed'] is False)
    synthetic['calls'][-1]['local_spawn_admitted_ns'] = None
    check('native_no_spawn_no_parse_skip_evidence', mechanism.extract(synthetic)['dp3']['native_skip_observed'] is True)
    synthetic['phase'] = 'OFFLINE'
    check('offline_never_claims_real_activation', mechanism.extract(synthetic)['dp2']['activation'] == 'NOT_MEASURED_OFFLINE')
    joint.common.dump(args.root / 'summary.json', {'status': 'PASS', 'checks': checks, 'finite_count': len(checks), 'actual_API_calls': 0, 'actual_lock_processes': 4})
    print(json.dumps({'status': 'PASS', 'checks': len(checks), 'API_calls': 0}))


if __name__ == '__main__':
    main()
