"""Read-only authored-fixture egress inventory; never reads credentials or endpoint routing."""
import argparse
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
COMMON = HERE.parent / 'common9_poc_20261019'
PROBE = REPO / 'results/architecture-evaluation/current/common9-poc-20261019/compatibility-probe-v3-r2'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def walk(value):
    if isinstance(value, dict):
        yield value
        for child in value.values():
            yield from walk(child)
    elif isinstance(value, list):
        for child in value:
            yield from walk(child)


def proof():
    consent = PROBE / 'HUMAN-EGRESS-CONSENT-VERBATIM.json'
    addendum = PROBE / 'HUMAN-EGRESS-CONSENT-ADDENDUM.json'
    assert consent.exists() and addendum.exists()
    allowed_asset_roots = [(COMMON / 'assets').resolve(), (HERE / 'assets').resolve()]
    populations, file_rows = {}, []
    for name, folder in [('COMMON9', COMMON), ('COMMON_SUPPLEMENT6', HERE)]:
        fixtures = json.loads((folder / 'fixtures.json').read_text())
        rows = []
        for episode in fixtures:
            initial = episode['initial']
            frames = {}
            for node in walk(episode):
                if 'image_path' in node:
                    path = Path(node['image_path']).resolve()
                    assert any(path.is_relative_to(root) for root in allowed_asset_roots), str(path)
                    frames[str(path)] = {'path': str(path.relative_to(REPO)), 'sha256': sha(path), 'bytes': path.stat().st_size, 'evidence_class': 'AUTHORED_RENDERED_SYNTHETIC_SCREEN'}
                # SourceRegistry supports file-backed sources, but none of the
                # authored fixed fixtures may acquire arbitrary files that way.
                if 'local_allowed' in node and 'text' in node:
                    assert 'path' not in node
                    assert isinstance(node['text'], str)
            rows.append({'episode': episode['id'], 'synthetic_raw_history_turns': len(initial['raw_history']), 'synthetic_tasks_total': len(initial['tasks']), 'synthetic_completed_tasks': sum(t.get('state') == 'COMPLETED' for t in initial['tasks']), 'other_known_task_records': sum(t.get('state') != 'COMPLETED' for t in initial['tasks']), 'actual_task_state_counts': {state: sum(t.get('state') == state for t in initial['tasks']) for state in sorted({t.get('state') for t in initial['tasks']})}, 'literal_original_source_ids': [s['id'] for s in initial['sources']], 'scheduled_authored_events': len(episode['schedule']), 'source_file_paths': [], 'frames': list(frames.values())})
        populations[name] = rows
        for filename in ['fixtures.json', 'build_fixtures.py', 'oracles.json']:
            path = folder / filename
            file_rows.append({'path': str(path.relative_to(REPO)), 'sha256': sha(path), 'role': 'EVALUATOR_ONLY_NOT_MODEL_INPUT' if filename == 'oracles.json' else 'FIXTURE_OR_AUTHORING_SOURCE_NOT_CODE_TRANSMISSION'})
    return {'version': 'JOINT441_AUTHORED_EGRESS_PROOF_v1', 'scope': {'fixed_development_probe': 2, 'primary': 315, 'supplement': 126, 'total_cap_usd': 40}, 'permission_is_existing_human_consent_not_new_paid_GO': True, 'consent_records': [{'path': str(p.relative_to(REPO)), 'sha256': sha(p)} for p in [consent, addendum]], 'destination': {'provider': 'Alibaba Cloud DashScope / Model Studio', 'region': 'Singapore', 'hostname_template': '<configured-workspace>.ap-southeast-1.maas.aliyuncs.com', 'endpoint_path': '/compatible-mode/v1/chat/completions', 'model': 'qwen3.7-plus-2026-05-26'}, 'data_classes': populations, 'actual_body_fields': ['Authored synthetic current user/external event and authorized synthetic raw document spans', 'Authored synthetic200turn dialogue and20completed Task records plus actual authored known Task slots', 'Actual rendered fixture PNGs and raw/native observation metadata', 'Experiment system/semantic instructions and structured catalog/association state', 'Prior actual model-derived interpretations, actual displayed synthetic responses, synthetic receiver outputs and error-only bounded rejudge reasons'], 'excluded_from_model_input': ['Actual personal files, email/calendar contents, operating-system documents', 'Repository implementation files and Git data as source text', 'Credentials, API keys, environment values, keychain records, routing workspace identifiers', 'Future event schedules, evaluator expected goals/sets/content, full renderer geometry maps'], 'absence_evidence': ['All15 fixed initial/event source records are inline authored literal text; file-backed source.path is rejected by this inventory.', 'All candidate frame paths are inside the two authored fixture asset roots, and actual PNG digests are listed.', 'Builder code constructs synthetic history/Tasks from authored constants/common-scenario inputs. Code is executed locally; its file contents are not serialized as model input.', 'Frozen bridge builds request messages only from payload.input/instruction and explicitly listed images. Credentials are provider Authorization headers, never prompt/request-body evidence.', 'Runtime external-result shim uses actually admitted synthetic contract/receiver output and synthetic control actions; it does not read actual private files.'], 'source_evidence': file_rows + [{'path': str((COMMON / 'bridge.py').relative_to(REPO)), 'sha256': sha(COMMON / 'bridge.py')}, {'path': str((HERE / 'harness.py').relative_to(REPO)), 'sha256': sha(HERE / 'harness.py')}], 'credentials_or_routing_read_by_this_check': False, 'actual_API_calls': 0}


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError('Preserve previous proof')
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(proof(), ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'proof': str(args.output), 'sha256': sha(args.output), 'actual_API_calls': 0}))
