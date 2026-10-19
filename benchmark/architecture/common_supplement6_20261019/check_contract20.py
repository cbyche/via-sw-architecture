"""Finite supplemental acceptance. Injected waits are technical fixtures only.

No API, learned parser or performance/accuracy claim. New result roots required.
"""
import argparse
import copy
import hashlib
import json
import subprocess
import sys
import threading
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
import evaluate
import joint_campaign
from trigger_rules import final_anchor, FinalAnchorRegistry


def pure_checks():
    checks = []
    def check(name, condition):
        if not condition:
            raise AssertionError(name)
        checks.append(name)
    check('before30_no_final_no_anchor', final_anchor(100, None, 399, 300) is None)
    check('at30_missing_boundary', final_anchor(100, None, 400, 300)['at_ns'] == 400)
    check('timely_queued_final_remains_actual', final_anchor(100, 399, 500, 300) == {'kind': 'ACTUAL_FINAL', 'at_ns': 399, 'anchor_deadline_ns': 400})
    check('exact30_final_inclusive', final_anchor(100, 400, 500, 300)['kind'] == 'ACTUAL_FINAL')
    check('late_final_before_next_poll_stays_missing', final_anchor(100, 401, 500, 300)['kind'] == 'MISSING_AT_30_SECONDS')
    # Deterministically pause between receipt timestamp and dictionary publication.
    # The choice thread must block on the same mutex, then preserve the timely
    # stamped fact+parent even though the fake current clock has crossed30.
    observed, release, chooser_started, chosen = threading.Event(), threading.Event(), threading.Event(), []
    clock = [399]
    registry = FinalAnchorRegistry(clock=lambda: clock[0])
    class PausedPublish(dict):
        def setdefault(self, key, value):
            observed.set()
            assert release.wait(5)
            return super().setdefault(key, value)
    registry.records = PausedPublish()
    reader = threading.Thread(target=registry.observe, args=('U1', 'actual-parent'))
    reader.start()
    assert observed.wait(5)
    clock[0] = 500
    def select():
        chooser_started.set()
        chosen.append(registry.choose('U1', 100, 300))
    chooser = threading.Thread(target=select)
    chooser.start()
    assert chooser_started.wait(5)
    check('unpublished_timely_receipt_owns_anchor_mutex', registry.lock.locked())
    release.set()
    reader.join(5)
    chooser.join(5)
    check('atomic_timely_stamp_parent_beats_delayed_fallback_poll', chosen == [{'kind': 'ACTUAL_FINAL', 'at_ns': 399, 'anchor_deadline_ns': 400, 'parent_node': 'actual-parent'}])
    check('boundary_choice_recorded_once', registry.choose('U1', 100, 300) is None)
    plan = joint_campaign.jobs()
    check('441_unique', len(plan) == len({tuple(joint_campaign.identity(j).values()) for j in plan}) == 441)
    check('primary315_relative_order_exact', [joint_campaign.identity(j) for j in plan if j['population'] == 'COMMON9'] == joint_campaign.common.jobs())
    check('supplement126_and3reps', len([j for j in plan if j['population'] == 'COMMON_SUPPLEMENT6']) == 126 and {j['rep'] for j in plan if j['population'] == 'COMMON_SUPPLEMENT6'} == {1, 2, 3})
    check('63_full_config_blocks', len({j['block_id'] for j in plan}) == 63 and all({j['configuration'] for j in plan[i:i+7]} == set(joint_campaign.CONFIGS) for i in range(0, 441, 7)))
    for rep, expected in [(1, ['S80', 'S83', 'S81', 'S84', 'S82', 'S85']), (2, ['S81', 'S84', 'S82', 'S85', 'S80', 'S83']), (3, ['S82', 'S85', 'S80', 'S83', 'S81', 'S84'])]:
        actual = [j['episode'] for j in plan[::7] if j['rep'] == rep and j['population'] == 'COMMON_SUPPLEMENT6']
        check('supplement_condition_rotation_r' + str(rep), actual == expected)
    check('own12_8_6_denominators', evaluate.COUNTS == {'responses': 12, 'goals': 8, 'bindings': 6})
    empty = evaluate.summarize([], 3)['FORMAL:AAAA']
    check('empty_keeps36_24_18', [empty[k]['planned'] for k in ['qa01', 'qa03', 'qa02']] == [36, 24, 18])
    check('empty_no_cost_or_memory_mean', empty['qa05']['mean_six_episode_bundle_usd'] is None and empty['qa06']['maximum_same_sample_rss_bytes'] is None)
    common_empty = evaluate.method_v3.summarize([], 5)
    check('own_adapter_does_not_replace_common9_global_denominators', all(s['qa01']['planned'] == 215 and s['qa03']['planned'] == 115 and s['qa02']['planned'] == 75 for s in common_empty.values()))
    fixtures = json.loads((HERE / 'fixtures.json').read_text())
    check('text0_screen6', all(s['authored_utterance_seconds'] == (0 if e['id'] in ['S80', 'S81', 'S82'] else 6) for e in fixtures for s in e['schedule']))
    check('actualprovided_trigger_only_S80_S82', all(e['schedule'][1]['trigger']['kind'] == ('after_actual_provided' if e['id'] in ['S80', 'S82'] else 'after_actual_final_or_missing_anchor') for e in fixtures))
    check('no_agent_specs', all(s['event']['kind'] == 'user' for e in fixtures for s in e['schedule']))
    return checks


def verify_baseline(root):
    rows = [json.loads(line) for line in (root / 'runs.jsonl').read_text().splitlines()]
    checks = []
    for trace in rows:
        episode = trace['episode']
        source1 = trace['provided_sources'][episode + '-U01']
        source2 = trace['provided_sources'][episode + '-U02']
        trigger = next(a for a in trace['trigger_audit'] if a['event_id'] == episode + '-U02')
        if episode in ['S80', 'S82']:
            assert trigger['anchor_ns'] == source1['provided_at_ns']
            assert trigger['anchor_is_actual_final'] is False
        else:
            assert trigger['anchor_ns'] == trace['actual_response_final_ns'][episode + '-U01']
            assert trigger['anchor_is_actual_final'] is True
        scale = trace['authored_time_scale']
        assert trigger['scheduled_input_begin_ns'] == trigger['anchor_ns'] + int(2 * scale * 1e9)
        assert source2['source_ns'] == trigger['scheduled_input_begin_ns'] + int((0 if episode in ['S80', 'S81', 'S82'] else 6) * scale * 1e9)
        assert source1['source_ns'] == trace['initial_source_provided_ns'] + int((5 + (0 if episode in ['S80', 'S81', 'S82'] else 6)) * scale * 1e9)
        assert source1['inputbegin_ns'] <= source1['source_ns'] <= source1['provided_at_ns']
        assert source2['inputbegin_ns'] <= source2['source_ns'] <= source2['provided_at_ns']
        assert trace['contract_observation_deadline_ns'] == source2['provided_at_ns'] + int(60 * scale * 1e9)
        assert evaluate.regression(trace)['status'] == 'PASS'
        checks.append(trace['run_id'] + ':actual_source_trigger_clock_strict_regression')
    evaluations = [evaluate.evaluate(t) for t in rows]
    summary = evaluate.summarize(evaluations, 1)
    assert all(s['qa01']['observed'] == 12 and s['qa03']['observed'] == 8 and s['qa02']['correct'] == 6 and s['population_complete'] for s in summary.values())
    checks.append('all7_fixed_bundle12_8_6')
    one = [r for r in evaluations if r['configuration'] == 'AAAA']
    repeated = [dict(copy.deepcopy(row), rep=rep) for rep in [1, 2, 3] for row in one]
    for row in repeated:
        row['qa05']['known_api_cost_usd'] = .1
    full3 = evaluate.summarize(repeated, 3)['OFFLINE:AAAA']
    assert full3['population_complete'] and [full3[k]['planned'] for k in ['qa01', 'qa03', 'qa02']] == [36, 24, 18]
    assert abs(full3['qa05']['known_total_usd'] - 1.8) < 1e-9 and abs(full3['qa05']['mean_six_episode_bundle_usd'] - .6) < 1e-9
    checks.append('synthetic3repeat36_24_18_and_QA05_six_bundle_total_div3')
    missing = evaluate.summarize(one[:-1], 1)['OFFLINE:AAAA']
    assert not missing['population_complete'] and missing['qa01']['planned'] == 12 and missing['qa05']['mean_six_episode_bundle_usd'] is None
    checks.append('missing_episode_keeps_fixed_denominator_and_no_complete_cost')
    duplicate = evaluate.summarize(one + [one[0]], 1)['OFFLINE:AAAA']
    assert duplicate['duplicate_episode_repetitions'] and duplicate['qa01']['observed_partial_mean_ms'] is None and duplicate['qa02']['fixed_denominator_accuracy'] is None
    checks.append('duplicate_never_inflates_primary_mean_or_accuracy')
    bad = copy.deepcopy(one)
    bad[0]['primary_measurement_eligibility'] = 'NOT_ELIGIBLE'
    bad[0]['deadline_audit']['status'] = 'UNRESOLVED'
    partial = evaluate.summarize(bad, 1)['OFFLINE:AAAA']
    assert partial['primary_measurement_eligible_episodes'] == 5 and partial['qa01']['planned'] == 12 and partial['qa05']['mean_six_episode_bundle_usd'] is None
    checks.append('technical_uncertified_excluded_primary_timing_not_spend_or_denominator')
    unknown = copy.deepcopy(one)
    unknown[0]['qa05']['unknown_call_count'] = 1
    unknown[0]['qa05']['unknown_reservation_usd'] = .1
    summary_unknown = evaluate.summarize(unknown, 1)['OFFLINE:AAAA']
    assert summary_unknown['qa05']['mean_six_episode_bundle_usd'] is None and summary_unknown['qa05']['unknown_reservation_usd'] == .1
    checks.append('unknown_reservation_retained_no_complete_cost')
    return checks


def child(root, case):
    import importlib.util
    spec = importlib.util.spec_from_file_location("supplement_contract20_harness", HERE / "harness.py")
    harness = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(harness)
    import supplement_offline
    scale = .05
    original = supplement_offline.respond
    def injected(payload, meta):
        event = payload.get('input', {}).get('event', {})
        if meta['stage'] == 'meaning_interpret_and_respond' and event.get('id') == 'S81-U01':
            if case == 'missing':
                return {}
            if case == 'late':
                time.sleep(31 * scale)
            if case == 'wrong_delegate':
                value = original(payload, meta)
                value.update(route='DELEGATE', recipient='synthetic-capable-agent', artifact_type='text', completion_condition='Return an actual artifact')
                return value
        return original(payload, meta)
    supplement_offline.respond = injected
    if case == 'timely_queued':
        original_queue = harness.queue.Queue
        class DelayedFinalQueue(original_queue):
            delayed = False
            def get(self, *args, **kwargs):
                value = super().get(*args, **kwargs)
                if not self.delayed and value.get('kind') == 'RESPONSE_FINAL' and value.get('data', {}).get('stimulus_id') == 'S81-U01':
                    self.delayed = True
                    time.sleep(31 * scale)
                return value
        harness.queue.Queue = DelayedFinalQueue
    episode = next(e for e in json.loads((HERE / 'fixtures.json').read_text()) if e['id'] == 'S81')
    trace = harness.execute(root, episode, 'AAAA', 1, offline=True, time_scale=scale)
    sid = 'S81-U01'
    choice = trace['final_anchor_choices'][sid]
    trigger = next(t for t in trace['trigger_audit'] if t['event_id'] == 'S81-U02')
    if case in ['missing', 'late']:
        assert choice['kind'] == 'MISSING_AT_30_SECONDS'
        assert trigger['anchor_ns'] == trace['provided_sources'][sid]['provided_at_ns'] + int(30 * scale * 1e9)
        assert trigger['anchor_is_actual_final'] is False and trigger['missing_anchor_failure_retained'] is True
        assert any(f['kind'] == 'RESPONSE_ANCHOR_TIMEOUT' and f['event_id'] == sid for f in trace['host_failures'])
    elif case == 'timely_queued':
        assert choice['kind'] == 'ACTUAL_FINAL'
        assert choice['at_ns'] <= choice['anchor_deadline_ns']
        assert trigger['anchor_is_actual_final'] is True and not trigger['missing_anchor_failure_retained']
        assert not any(f['kind'] == 'RESPONSE_ANCHOR_TIMEOUT' and f['event_id'] == sid for f in trace['host_failures'])
    else:
        assert not any(e['kind'] == 'INPUT_CAPTURED' and e['data'].get('input_class') == 'agent' for e in trace['events'])
        assert not any('-A' in sid for sid in trace['provided_sources'])
    assert trace['actual_API_calls'] == 0
    result = {'case': case, 'status': 'PASS', 'trace': str(next((root / 'raw/runs').glob('*/trace.json'))), 'binary_sha256': trace['binary_sha256'], 'actual_API_calls': 0, 'injected_wait_scope': 'TECHNICAL_TEST_ONLY_NOT_REAL_MODEL_WORK', 'chosen_anchor': choice, 'trigger': trigger}
    (root / 'check.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(result, ensure_ascii=False))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--baseline', type=Path)
    parser.add_argument('--child', choices=['missing', 'late', 'timely_queued', 'wrong_delegate'])
    args = parser.parse_args()
    if args.child:
        args.root.mkdir(parents=True, exist_ok=False)
        child(args.root, args.child)
        return
    args.root.mkdir(parents=True, exist_ok=False)
    sources = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in HERE.glob('*.py')}
    checks = pure_checks()
    if args.baseline:
        checks += verify_baseline(args.baseline)
    cases = []
    for case in ['missing', 'late', 'timely_queued', 'wrong_delegate']:
        command = [sys.executable, str(Path(__file__).resolve()), '--root', str(args.root / case), '--child', case]
        result = subprocess.run(command, capture_output=True, text=True)
        (args.root / (case + '-stdout.txt')).write_text(result.stdout)
        (args.root / (case + '-stderr.txt')).write_text(result.stderr)
        assert result.returncode == 0, (case, result.stderr)
        cases.append(json.loads((args.root / case / 'check.json').read_text()))
    assert all(hashlib.sha256(Path(p).read_bytes()).hexdigest() == digest for p, digest in sources.items())
    (args.root / 'summary.json').write_text(json.dumps({'status': 'PASS', 'finite_checks': checks, 'finite_count': len(checks), 'actual_harness_cases': cases, 'source_sha256': sources, 'actual_API_calls': 0}, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'status': 'PASS', 'finite_checks': len(checks), 'actual_harness_cases': len(cases), 'API_calls': 0}))


if __name__ == '__main__':
    main()
