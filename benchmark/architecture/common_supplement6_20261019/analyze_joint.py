"""Separate common9/supplement6 analysis of one frozen joint generation only."""
import argparse
import hashlib
import json
from pathlib import Path

import evaluate as supplement
import joint_campaign
import mechanism


def analyze(root, offline=False):
    freeze = json.loads((root / 'freeze.json').read_text())
    if freeze['version'] != 'COMMON9_PLUS_SUPPLEMENT6_JOINT441_v1':
        raise ValueError('Expected one joint441 generation')
    traces = [json.loads(line) for line in (root / 'runs.jsonl').read_text().splitlines() if line.strip()]
    planned = {tuple(joint_campaign.identity(j).values()): j for j in freeze['jobs']}
    identities = [tuple(joint_campaign.identity(t).values()) for t in traces]
    if len(identities) != len(set(identities)) or any(key not in planned for key in identities):
        raise ValueError('Duplicate/unplanned identities are preserved but cannot be pooled into a primary aggregate')
    primary, secondary, activations, provenance = [], [], [], []
    for trace, key in zip(traces, identities):
        if trace['binary_sha256'] != freeze['binary']['sha256']:
            raise ValueError('Mixed candidate generation')
        if trace.get('phase') != ('OFFLINE' if offline else 'FORMAL'):
            raise ValueError('Mixed development/formal phase')
        source = root / 'raw/runs' / trace['run_id'] / 'trace.json'
        candidate_input = Path(trace['candidate_input_file']).resolve()
        if candidate_input.parent != source.parent.resolve():
            raise ValueError('Trace from another root/generation; do not pool probes or prior attempts')
        if json.loads(source.read_text()) != trace:
            raise ValueError('Combined raw row differs from preserved per-run trace')
        provenance.append({'run_id': trace['run_id'], 'trace_path': str(source.resolve()), 'sha256': hashlib.sha256(source.read_bytes()).hexdigest()})
        if planned[key]['population'] == 'COMMON9':
            primary.append(supplement.method_v3.evaluate(trace))
        else:
            secondary.append(supplement.evaluate(trace))
            activations.append(mechanism.extract(trace))
    return {'version': 'JOINT441_SEPARATE_POPULATION_ANALYSIS_v1', 'freeze_sha256': joint_campaign.common.sha(root / 'freeze.json'), 'primary_common9': {'planned_episodes': 315, 'observed_episodes': len(primary), 'results': primary, 'summary': supplement.method_v3.summarize(primary, 5)}, 'supplement6': {'planned_episodes': 126, 'observed_episodes': len(secondary), 'results': secondary, 'summary': supplement.summarize(secondary, 3), 'mechanism_activation': activations}, 'shared_control': 'One AAAA trace per episode/repetition reused in fourDP comparisons; correlated paired comparisons, never four independent samples', 'comparison_limits': ['Observed timing means retain fixed coverage and semantic diagnostics. Fast/cheap partial work is not evidence of a structural advantage.', 'Supplement conditions remain in fixed denominators whether mechanisms activate, fail or are unobserved. Activation is diagnostic, never a population filter.', 'QA04 references the same42 change experiments with630 required regressions; this file creates no new modifiability measurements.', 'Prior generations and development probes are excluded from this single-root analysis; their actual spend remains in the global ledger.'], 'trace_provenance': provenance, 'total_actual_recorded_API_cost_usd': sum(r['qa05']['known_api_cost_usd'] for r in primary + secondary), 'unknown_call_count': sum(r['qa05']['unknown_call_count'] for r in primary + secondary), 'unexecuted_planned_identities': [joint_campaign.identity(j) for j in freeze['jobs'] if tuple(joint_campaign.identity(j).values()) not in identities]}


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--offline', action='store_true')
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError('Preserve previous analysis')
    joint_campaign.common.dump(args.output, analyze(args.root.resolve(), args.offline))
