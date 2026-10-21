#!/usr/bin/env python3
"""Read-only cross-artifact verification for authored common QA v3.

Uses serialized ledgers and exported XLSX XML, without importing either authoring
script. It does not run a model, product, browser, or native Excel calculation.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
import subprocess
import xml.etree.ElementTree as ET
import zipfile
from collections import defaultdict
from pathlib import Path
from statistics import mean

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/presentations_files/qa-common-synthetic-v3-20261022'
NS = {'s': 'http://schemas.openxmlformats.org/spreadsheetml/2006/main'}


def read(name):
    return json.loads((OUT / name).read_text())


def digest(data):
    return hashlib.sha256(data).hexdigest()


def same(a, b, label, tol=1e-7):
    assert math.isfinite(a) and math.isfinite(b) and abs(a - b) <= tol, (label, a, b)


def canon(value):
    if isinstance(value, list):
        return sorted((canon(v) for v in value), key=lambda v: json.dumps(v, sort_keys=True))
    if isinstance(value, dict):
        return {k: canon(v) for k, v in sorted(value.items())}
    return value


def verify():
    data, summary, manifest = read('normalized-data.json'), read('summary.json'), read('manifest.json')
    assert data['contract_id'] == manifest['contract_id'] == 'QA-COMMON-SYN-CONTRACT-v3-STRATIFIED'
    assert manifest['corpus_id'] == 'COMMON-QA-CORPUS-3.0-STRATIFIED'
    assert data['actual_model_calls'] == data['actual_product_runs'] == 0
    assert data['aggregation_policy'] == 'WITHIN_PROFILE_ONLY_NO_POOLED_HEADLINE'
    assert data['summary'] == summary['configs']
    for name, sha in manifest['outputs'].items():
        assert digest((OUT / name).read_bytes()) == sha, name
    generator = manifest['generator']
    assert digest((ROOT / generator['path']).read_bytes()) == generator['sha256']
    for parent in manifest['parent_sources']:
        snapshot = (OUT / parent['snapshot_path']).read_bytes()
        assert digest(snapshot) == parent['sha256']
        published = subprocess.check_output(['git', 'show', parent['source_commit'] + ':' + parent['source_path']], cwd=ROOT)
        assert snapshot == published == (ROOT / parent['source_path']).read_bytes(), parent['stratum']

    config = {c['id']: c for c in data['configs']}
    source_ids = {(s['config_id'], s['source_id']) for s in data['sources']}
    for scenario in data['scenarios']:
        if scenario['stratum'] == 'DELIVERY':
            for cid in ['D-A', 'D-B']:
                assert all((cid, sid) in source_ids for sid in scenario['source_ids']), scenario['case_id']
    stages = {(s['config_id'], s['id']): s for s in data['stages']}
    assert len(stages) == len(data['stages'])
    for stage in stages.values():
        same(stage['end_s'] - stage['start_s'], stage['duration_s'], stage['id'], 3e-6)
        for parent_id in stage['parents']:
            parent = stages[(stage['config_id'], parent_id)]
            assert parent['case_id'] == stage['case_id']
            assert parent['end_s'] <= stage['start_s'] + 3e-6

    # Reconstruct every goal's projected causal duration, retaining dependencies
    # through excluded external/playback stages and clipping only at goal start.
    for goal in data['goals']:
        memo, visiting = {}, set()

        def project(nid):
            if nid in memo:
                return memo[nid]
            assert nid not in visiting, ('cycle', nid)
            visiting.add(nid)
            node = stages[(goal['config_id'], nid)]
            assert node['case_id'] == goal['case_id']
            work = max(0, node['end_s'] - max(node['start_s'], goal['start_s'])) if node['included'] else 0
            memo[nid] = work + max([0] + [project(p) for p in node['parents']])
            visiting.remove(nid)
            return memo[nid]

        same(max(project(n) for n in goal['terminal_nodes']), goal['via_s'], goal['goal_id'], 3e-6)

    for response in data['responses']:
        same(response['first_display_s'] - response['start_s'], response['qa01_s'], response['input_id'])
        if response['stratum'] == 'DELIVERY':
            assert response['expected'] == response['adopted'] and response['qa07_success'] == 1
            same(response['first_voice_s'] - response['start_s'], response['qa01_voice_s'], response['input_id'])
        else:
            assert response['qa01_voice_s'] is None and response['qa01_voice_status'] == 'NOT_MODELED'
            assert response['qa07_success'] == int(canon(response['expected']) == canon(response['adopted']))
    for ref in data['referents']:
        assert ref['correct'] == int(set(ref['expected']) == set(ref['adopted']))
    for usage in data['usage']:
        assert (usage['config_id'], usage['node_id']) in stages
        same(usage['amount'] * usage['rate_usd'], usage['usd'], usage['id'], 1e-10)
    for memory in data['memory']:
        same(sum(memory['components_mb'].values()), memory['total_mb'], memory['id'])

    old = read('parents/v2_summary.json')['configs']
    groups = {}
    for collection in ['responses', 'referents', 'goals', 'usage', 'memory', 'changes']:
        groups[collection] = defaultdict(list)
        for item in data[collection]:
            groups[collection][item['config_id']].append(item)
    for cid, result in summary['configs'].items():
        rows = {k: v[cid] for k, v in groups.items()}
        values = [mean(r['qa01_s'] for r in rows['responses']),
                  mean(r['correct'] for r in rows['referents']),
                  mean(r['via_s'] for r in rows['goals']),
                  mean(len(set(r['modules'])) for r in rows['changes']),
                  sum(r['usd'] for r in rows['usage'] if r['role'] == 'VIA'),
                  max(r['total_mb'] for r in rows['memory']),
                  mean(r['qa07_success'] for r in rows['responses'])]
        for i, value in enumerate(values, 1):
            key = f'qa{i:02}'
            same(value, result[key], (cid, key))
            if cid in old:
                same(value, old[cid][key], ('v2', cid, key), 1e-10)
        if config[cid]['stratum'] == 'DELIVERY':
            same(mean(r['qa01_voice_s'] for r in rows['responses']), result['qa01_voice'], (cid, 'voice'))
            assert result['receiver_cost'] is None and result['total_cost'] is None
            assert result['semantic_evidence'] == 'FIXED_CORRECT_SEMANTIC_FIXTURE'
            assert len(rows['responses']) == 80 and len(rows['referents']) == 120 and len(rows['goals']) == 90
            originals = [g for g in rows['goals'] if g['original_goal']]
            assert len(originals) == 10 and all(g['start_s'] == -6 for g in originals)
        else:
            assert result['qa01_voice'] is None and config[cid]['dp44'] == 'FIXED_UNMAPPED'

    xlsx = OUT / 'outputs/01a15dfe/VIA-QA-stratified-v3.xlsx'
    report = read('workbook-validation.json')
    assert report['source_sha256'] == digest((OUT / 'normalized-data.json').read_bytes())
    assert len(report['summary_cells']) == 63 and len(report['voice_cells']) == 2
    assert {(c['config'], c['qa']) for c in report['summary_cells']} == {(cid, f'qa{i:02}') for cid in config for i in range(1, 8)}
    assert {(c['config'], c['qa']) for c in report['voice_cells']} == {('D-A', 'qa01_voice'), ('D-B', 'qa01_voice')}
    with zipfile.ZipFile(xlsx) as z:
        strings = ET.fromstring(z.read('xl/sharedStrings.xml'))
        shared = [''.join(si.itertext()) for si in strings]
        sheets = [name for name in z.namelist() if re.fullmatch(r'xl/worksheets/sheet\d+\.xml', name)]
        assert len(sheets) == 9
        errors, formulas = [], 0
        for name in sheets:
            xml = ET.fromstring(z.read(name))
            for cell in xml.findall('.//s:c', NS):
                if cell.get('t') == 'e':
                    errors.append((name, cell.get('r'), cell.findtext('s:v', namespaces=NS)))
                if cell.find('s:f', NS) is not None:
                    formulas += 1
        assert not errors, errors
        cells = {c.get('r'): c for c in ET.fromstring(z.read('xl/worksheets/sheet1.xml')).findall('.//s:c', NS)}
        for metric in report['summary_cells'] + report['voice_cells']:
            cell = cells[metric['address']]
            assert cell.find('s:f', NS) is not None
            same(float(cell.findtext('s:v', namespaces=NS)), summary['configs'][metric['config']][metric['qa']], metric['address'])
        for address in ['F53', 'G53', 'F54', 'G54']:
            assert address not in cells or (cells[address].find('s:f', NS) is None and cells[address].find('s:v', NS) is None), address
        for i, cid in enumerate(config, 46):
            for col, key in zip('BCDEFG', ['scored_usd', 'preparation_usd', 'maintenance_usd', 'qa05', 'receiver_cost', 'total_cost']):
                expected = summary['configs'][cid][key]
                if expected is None:
                    continue
                cell = cells[f'{col}{i}']
                assert cell.find('s:f', NS) is not None
                same(float(cell.findtext('s:v', namespaces=NS)), expected, (cid, key))
        # Read semantic JSON strings from the exported workbook. DELIVERY array
        # order must match the ledger, even though referent set order is irrelevant.
        meaning = {c.get('r'): c for c in ET.fromstring(z.read('xl/worksheets/sheet5.xml')).findall('.//s:c', NS)}
        for i, response in enumerate(data['responses'], 6):
            if response['stratum'] != 'DELIVERY':
                continue
            for col, key in [('D', 'expected'), ('E', 'adopted')]:
                cell = meaning[f'{col}{i}']
                value = cell.findtext('s:v', namespaces=NS)
                if cell.get('t') == 's':
                    value = shared[int(value)]
                elif cell.get('t') == 'inlineStr':
                    value = ''.join(cell.find('s:is', NS).itertext())
                else:
                    assert cell.get('t') == 'str', cell.get('t')
                assert json.loads(value) == response[key]

    html = (OUT / 'outputs/01a15dfe/guide.html').read_text()
    match = re.search(r'const D=(.*?);\nconst \$=', html, re.S)
    assert match, 'embedded HTML ledger missing'
    embedded = json.loads(match.group(1))
    assert embedded['summary'] == summary['configs']
    assert len(embedded['responses']) == len(data['responses'])
    assert len(embedded['stages']) == len(data['stages'])
    for stage in embedded['stages']:
        assert stage['parents'] == stages[(stage['config_id'], stage['id'])]['parents'], stage['id']
    assert embedded['sources'] == data['sources']
    assert 'QA-COMMON-SYN-CONTRACT-v3-STRATIFIED' in html
    assert 'SYNTHETIC_AUTHORED_NOT_MEASURED' in html

    return {'status': 'PASS', 'dataset_id': data['dataset_id'],
            'evidence_kind': data['evidence_kind'],
            'source_sha256': digest((OUT / 'normalized-data.json').read_bytes()),
            'xlsx_sha256': digest(xlsx.read_bytes()),
            'html_sha256': digest((OUT / 'outputs/01a15dfe/guide.html').read_bytes()),
            'checks': ['published parent Git objects, working files and frozen snapshots are byte-identical',
                       'manifest hashes and generator identity', 'all 1200 causal goal projections',
                       '63 QA and 2 separate voice values reaggregated from serialized ledgers',
                       '49 v2 values unchanged', 'ordered DELIVERY meaning and null semantics',
                       'all 9 XLSX sheets scanned for cached errors',
                       '65 XLSX QA and 50 cost formula caches match JSON; 4 DELIVERY cost blanks preserved',
                       'exported DELIVERY semantic order preserved',
                       'embedded HTML summary, stage dependencies and source rows match JSON'],
            'xlsx_formula_count': formulas, 'xlsx_error_count': len(errors),
            'limitations': ['No native Excel application or browser rendering verification.',
                            'Synthetic consistency only; no model or product performance measurement.']}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    result = verify()
    text = json.dumps(result, ensure_ascii=False, indent=2) + '\n'
    if args.report:
        args.report.write_text(text)
    print(text, end='')
