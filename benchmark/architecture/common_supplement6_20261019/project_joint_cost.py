"""Joint315+126 byte/call planning; not token evidence or a completion guarantee."""
import argparse
import collections
import hashlib
import json
from pathlib import Path

from joint_campaign import CONFIGS, budget_snapshot, common


def load(root, count):
    paths = sorted(root.rglob('trace.json'))
    rows = []
    for path in paths:
        row = json.loads(path.read_text())
        row['_path'] = str(path.resolve())
        row['_sha256'] = hashlib.sha256(path.read_bytes()).hexdigest()
        rows.append(row)
    assert len(rows) == 7 * count, (root, len(rows), count)
    assert len({(r['configuration'], r['episode']) for r in rows}) == len(rows)
    assert all(r['actual_API_calls'] == 0 for r in rows)
    return rows


def episode_plan(trace):
    cloud = [c for c in trace['calls'] if c['stage'] != 'grounding.local_screen_parser']
    local = [c for c in trace['calls'] if c['stage'] == 'grounding.local_screen_parser']
    text = sum(c['input_text_bytes'] + c['instruction_bytes'] for c in cloud)
    outputs = sum(c['max_output_tokens'] for c in cloud)
    output_bytes = sum(c['offline_response_utf8_bytes'] for c in cloud)
    images = sum(c['image_bytes'] for c in cloud)
    groups = collections.defaultdict(list)
    for call in cloud:
        if call['stage'] == 'meaning_interpret_and_respond':
            goals = call['causal_trace']['goal_ids']
            assert len(goals) == 1 and goals[0].startswith('input:')
            groups[goals[0]].append(call)
    extra_calls = extra_input = extra_output = extra_images = 0
    for calls in groups.values():
        assert 1 <= len(calls) <= 3
        extra = 3 - len(calls)
        extra_calls += extra
        extra_input += extra * (max(c['input_text_bytes'] + c['instruction_bytes'] for c in calls) + 4096)
        extra_output += extra * max(c['max_output_tokens'] for c in calls)
        extra_images += extra * max(c['image_bytes'] for c in calls)
    image_proxy = images * .4 / 1e6
    ordinary = (text * .4 / 3 + outputs * 1.6) / 1e6 + image_proxy
    full = ordinary + (extra_input * .4 / 3 + extra_output * 1.6 + extra_images * .4) / 1e6
    # Same reservation rule as bridge: complete input/image bytes plus4096 per
    # admitted cloud call and configured output cap. MAX3 repair slot context has
    # its separate4096 error-context allowance above. Neither proxy is a tokenizer.
    conservative = ((text + extra_input + images + extra_images + 4096 * (len(cloud) + extra_calls)) * .4 + (outputs + extra_output) * 1.6) / 1e6
    return {'observed_cloud_stage_calls': len(cloud), 'observed_local_parser_stage_calls': len(local), 'input_instruction_utf8_bytes': text, 'configured_output_cap_tokens': outputs, 'double_output_utf8_bytes': output_bytes, 'image_binary_bytes': images, 'meaning_workflows': len(groups), 'additional_calls_to_MAX3': extra_calls, 'additional_meaning_context_bytes': extra_input, 'additional_meaning_output_cap_tokens': extra_output, 'additional_meaning_image_bytes': extra_images, 'illustrative_input_output_3bytes_plus_imagebyte_usd': (text * .4 + output_bytes * 1.6) / 3e6 + image_proxy, 'ordinary_input3_outputcap_imagebyte_usd': ordinary, 'full_MAX3_input3_outputcap_imagebyte_usd': full, 'conservative_full_MAX3_reservation_byte_outputcap_imagebyte_usd': conservative, 'stage_breakdown': {stage: {'calls': sum(c['stage'] == stage for c in cloud), 'input_bytes': sum(c['input_text_bytes'] + c['instruction_bytes'] for c in cloud if c['stage'] == stage), 'output_cap_tokens': sum(c['max_output_tokens'] for c in cloud if c['stage'] == stage)} for stage in sorted({c['stage'] for c in cloud})}}


def project(primary_root, supplement_root):
    populations = {}
    per_episode = {}
    provenance = []
    for name, root, count, reps in [('COMMON9', primary_root, 9, 5), ('COMMON_SUPPLEMENT6', supplement_root, 6, 3)]:
        traces = load(root, count)
        plans = {cfg: {} for cfg in CONFIGS}
        for trace in traces:
            plans[trace['configuration']][trace['episode']] = episode_plan(trace)
            provenance.append({'population': name, 'path': trace['_path'], 'sha256': trace['_sha256'], 'binary_sha256': trace['binary_sha256'], 'run_id': trace['run_id']})
        cost_keys = [key for key in next(iter(plans['AAAA'].values())) if key.endswith('_usd')]
        totals = {key: reps * sum(p[key] for cfg in plans.values() for p in cfg.values()) for key in cost_keys}
        populations[name] = {'episodes_per_configuration': count, 'repetitions': reps, 'episode_runs': count * reps * 7, 'per_configuration_per_episode': plans, 'projections_usd': totals, 'cloud_calls_ordinary_extrapolated': reps * sum(p['observed_cloud_stage_calls'] for cfg in plans.values() for p in cfg.values()), 'cloud_calls_full_MAX3_extrapolated': reps * sum(p['observed_cloud_stage_calls'] + p['additional_calls_to_MAX3'] for cfg in plans.values() for p in cfg.values())}
        per_episode[name] = {cfg: {episode: p['conservative_full_MAX3_reservation_byte_outputcap_imagebyte_usd'] for episode, p in rows.items()} for cfg, rows in plans.items()}
    budget = budget_snapshot()
    prior = budget['known_usd'] + budget['inflight_reservation_usd'] + budget['unknown_reservation_usd']
    combined = {key: prior + sum(p['projections_usd'][key] for p in populations.values()) for key in cost_keys}
    return {'status': 'PLANNING_NOT_PROVIDER_TOKEN_MEASUREMENT', 'actual_API_calls': 0, 'planned_episode_runs': 441, 'population_reduction': False, 'price_usd_per_million': {'input': .4, 'output': 1.6}, 'budget': budget, 'populations': populations, 'combined_including_all_prior_usage_and_unknown_usd': combined, 'per_episode_block_planning_usd': per_episode, 'complete_within40_guaranteed': False, 'requires_root_budget_feasibility_judgment_before_paid_GO': True, 'assumptions': ['One fresh offline trace per case/configuration supplies actual serialized text/instruction/image byte counts and configured output caps. Doubles do not predict real model choices or latency.', '3UTF8bytes/token is illustrative, not verified Qwen tokenization. Imagebyte-as-input-token is a disclosed planning proxy, not the provider visual-token formula or universal upper bound.', 'MAX3 prices every unused meaning slot from the largest observed workflow input+instruction plus4096 error-context bytes. All observed evidence/producer/grounding/receiver calls remain included.', 'Conservative block amount additionally follows bridge full-byte reservation plus4096 overhead per cloud call. It can exceed actual provider cost and still cannot bound unobserved model/preparation/repair paths.', 'Per-episode100 call cap and global40 ledger with unknown/inflight reservations remain controlling. No cap, policy, condition, repetition, model, or output limit is reduced.', 'Local parser API charge is zero; monetized local compute cost is unmeasured. Its process RSS and observed time remain measured during real runs.', 'Primary315 and supplement126 are separate populations. Their expected workload and costs must not be ranked as interchangeable bundles.', 'Prior generations and fixed development probes remain actual spend; no old/new outcome pooling or hidden retry. A projection above40 requires explicit budget/common-policy judgment, not silent incompletion presented as a full campaign.'], 'trace_provenance': provenance}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--primary', type=Path, required=True)
    parser.add_argument('--supplement', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError('Preserve previous projection')
    value = project(args.primary, args.supplement)
    common.dump(args.output, value)
    print(json.dumps({'combined_including_prior_usd': value['combined_including_all_prior_usage_and_unknown_usd'], 'budget': value['budget'], 'planned': 441}))


if __name__ == '__main__':
    main()
