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
import posixpath
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


def canon(value, field_path=()):
    if isinstance(value, list):
        values = [canon(v, field_path + ('[]',)) for v in value]
        if field_path in [('targets',), ('conditions',), ('exclusions',)]:
            return ('set', tuple(sorted(set(values))))
        assert field_path == ('conditions', 'order'), ('unregistered meaning list', field_path)
        return ('sequence', tuple(values))
    if isinstance(value, dict):
        return ('object', tuple((k, canon(v, field_path + (k,))) for k, v in sorted(value.items())))
    if value is None:
        return ('null',)
    if isinstance(value, bool):
        return ('boolean', value)
    if isinstance(value, (int, float)):
        assert math.isfinite(value)
        return ('number', value)
    assert isinstance(value, str), type(value)
    return ('string', value)


def verify():
    data, summary, manifest = read('normalized-data.json'), read('summary.json'), read('manifest.json')
    ranges = read('range-review.json')
    assert data['review_revision'] == summary['review_revision'] == ranges['review_revision'] == 'COMMON-QA-SYN-3.0-REVIEW-2'
    assert data['range_review'] == ranges
    for key in ['metric_catalog', 'display_policy', 'semantic_comparison_policy']:
        assert data[key] == summary[key], key
    assert data['contract_id'] == manifest['contract_id'] == 'QA-COMMON-SYN-CONTRACT-v3-STRATIFIED'
    assert manifest['corpus_id'] == 'COMMON-QA-CORPUS-3.0-STRATIFIED'
    assert data['actual_model_calls'] == data['actual_product_runs'] == 0
    assert data['aggregation_policy'] == 'WITHIN_PROFILE_ONLY_NO_POOLED_HEADLINE'
    assert data['summary'] == summary['configs']
    assert canon({'targets': ['A', 'B']}) == canon({'targets': ['B', 'A']})
    assert canon({'conditions': {'order': ['A', 'B']}}) != canon({'conditions': {'order': ['B', 'A']}})
    assert canon({'targets': ['A']}) != canon({'targets': ['a']})
    assert canon({'keep': False}) != canon({})
    assert canon({'keep': None}) != canon({})
    assert canon({'keep': None}) != canon({'keep': False})
    assert canon({'keep': False}) != canon({'keep': 0})
    assert canon({'keep': True}) != canon({'keep': 1})
    assert canon({'keep': 1}) == canon({'keep': 1.0})
    assert canon({'targets': ['A', 'A']}) == canon({'targets': ['A']})
    try:
        canon({'unknown': ['A']})
    except AssertionError:
        pass
    else:
        raise AssertionError('unregistered list accepted')
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
        display = stages[(response['config_id'], response['first_display_node_id'])]
        adoption = stages[(response['config_id'], response['full_meaning_adoption_node_id'])]
        assert display['case_id'] == adoption['case_id'] == response['case_id']
        same(display['end_s'], response['first_display_s'], ('display boundary', response['input_id']))
        assert response['qa01_s'] >= 0
        if response['adopted_at_s'] is None:
            assert response['stratum'] == 'VOICE'
            assert response['adoption_interval'] == [adoption['start_s'], adoption['end_s']]
        else:
            assert response['adoption_interval'] is None
            same(adoption['end_s'], response['adopted_at_s'], ('adoption boundary', response['input_id']))
        if response['first_is_partial']:
            assert response['config_id'] == 'T-EXEC'
            assert response['first_display_s'] < response['adopted_at_s']
        if response['stratum'] == 'DELIVERY':
            assert canon(response['expected']) == canon(response['adopted']) and response['qa07_success'] == 1
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
        assert all(math.isfinite(v) and v >= 0 for v in memory['components_mb'].values())

    assert sum(r['first_is_partial'] for r in data['responses']) == 70
    for usage in data['usage']:
        assert all(math.isfinite(usage[k]) and usage[k] >= 0 for k in ['amount', 'rate_usd', 'usd'])
    for goal in data['goals']:
        assert 0 <= goal['via_s'] <= goal['wall_s'] + 3e-6

    old = read('parents/v2_summary.json')['configs']
    published_v3 = json.loads(subprocess.check_output([
        'git', 'show', '8b2f555f8b89eaa5a6a7085ca6323cd44a1adbd7:docs/presentations_files/qa-common-synthetic-v3-20261022/summary.json'
    ], cwd=ROOT))['configs']
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
            assert result[key] == published_v3[cid][key], ('original v3', cid, key)
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
        assert result['qa01_voice'] == published_v3[cid]['qa01_voice']

        # Recompute ranges independently from serialized rows, not builder helpers.
        observed = ranges['configs'][cid]
        for key, values in [
            ('qa01', [r['qa01_s'] for r in rows['responses']]),
            ('qa03', [r['via_s'] for r in rows['goals']]),
            ('qa04', [len(set(r['modules'])) for r in rows['changes']]),
        ]:
            assert observed[key]['n'] == len(values)
            for field, value in [('min', min(values)), ('max', max(values)), ('mean', mean(values))]:
                same(observed[key][field], value, ('range', cid, key, field))
        for key, collection, verdict in [('qa02', 'referents', 'correct'), ('qa07', 'responses', 'qa07_success')]:
            assert all(r[verdict] in [0, 1] for r in rows[collection])
            assert observed[key]['correct'] == sum(r[verdict] for r in rows[collection])
            assert observed[key]['denominator'] == len(rows[collection])
            same(observed[key]['ratio'], result[key], (cid, key, 'ratio'))
        case_cost, case_peak = defaultdict(float), defaultdict(float)
        for row in rows['usage']:
            if row['role'] == 'VIA':
                case_cost[row['case_id']] += row['usd']
        for row in rows['memory']:
            case_peak[row['case_id']] = max(case_peak[row['case_id']], row['total_mb'])
        for key, values in [('qa05', list(case_cost.values())), ('qa06', list(case_peak.values()))]:
            assert observed[key]['n'] == len(values)
            same(observed[key]['min'], min(values), (cid, key, 'min'))
            same(observed[key]['max'], max(values), (cid, key, 'max'))
        same(observed['qa05']['total_usd'], result['qa05'], (cid, 'range cost total'))
        peak = observed['qa06']['peak_witness']
        assert peak in rows['memory'] and peak['total_mb'] == result['qa06']
        budget = observed['memory_budget_reference']
        assert budget['reference_mb'] == 512 and budget['real_qualification'] is False and budget['score_assigned'] is None
        same(budget['signed_headroom_mb'], 512 - result['qa06'], (cid, 'headroom'))
        same(budget['excess_mb'], max(0, result['qa06'] - 512), (cid, 'excess'))
        same(budget['headroom_mb'], max(0, 512 - result['qa06']), (cid, 'positive headroom'))
        assert budget['model_base_mb'] == (238 if config[cid]['stratum'] == 'TEXT' else 560)
        same(budget['model_base_excess_mb'], max(0, budget['model_base_mb'] - 512), (cid, 'base excess'))
        assert budget['status'] == ('MODEL_EXCEEDS' if result['qa06'] > 512 else 'BELOW_MODEL_ONLY')
        assert observed['first_partial_display_n'] == sum(r['first_is_partial'] for r in rows['responses'])
        voice = [r['qa01_voice_s'] for r in rows['responses'] if r['qa01_voice_s'] is not None]
        assert observed['first_voice_latency']['n'] == len(voice)
        for field, fn in [('min', min), ('max', max), ('mean', mean)]:
            if voice:
                same(observed['first_voice_latency'][field], fn(voice), (cid, 'voice range', field))
            else:
                assert observed['first_voice_latency'][field] is None
        families = observed['family_statistics']
        for key, collection, field in [('qa01', 'responses', 'qa01_s'), ('qa02', 'referents', 'correct'),
                                       ('qa03', 'goals', 'via_s'), ('qa07', 'responses', 'qa07_success')]:
            for family in families:
                values = [r[field] for r in rows[collection] if r['family_id'] == family['family_id']]
                same(family[key], mean(values), (cid, family['family_id'], key))
            band = observed['family_mean_ranges'][key]
            assert band['family_n'] == len(families)
            same(band['min'], min(r[key] for r in families), (cid, key, 'family min'))
            same(band['max'], max(r[key] for r in families), (cid, key, 'family max'))

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
        assert len(sheets) == 10
        relationships = ET.fromstring(z.read('xl/_rels/workbook.xml.rels'))
        targets = {r.get('Id'): r.get('Target') for r in relationships}
        book = ET.fromstring(z.read('xl/workbook.xml'))
        sheet_paths = {}
        for sheet in book.findall('.//s:sheet', NS):
            target = targets[sheet.get('{http://schemas.openxmlformats.org/officeDocument/2006/relationships}id')]
            sheet_paths[sheet.get('name')] = target.lstrip('/') if target.startswith('/') else posixpath.normpath('xl/' + target)
        assert '범위검토' in sheet_paths
        errors, formulas = [], 0
        for name in sheets:
            xml = ET.fromstring(z.read(name))
            for cell in xml.findall('.//s:c', NS):
                if cell.get('t') == 'e':
                    errors.append((name, cell.get('r'), cell.findtext('s:v', namespaces=NS)))
                if cell.find('s:f', NS) is not None:
                    formulas += 1
                    assert cell.find('s:v', NS) is not None, ('missing formula cache', name, cell.get('r'))
        assert not errors, errors
        cells = {c.get('r'): c for c in ET.fromstring(z.read(sheet_paths['비교요약'])).findall('.//s:c', NS)}
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
        # Read canonical semantic JSON for every stratum. Set order is irrelevant;
        # the explicit conditions.order sequence must remain unchanged.
        meaning = {c.get('r'): c for c in ET.fromstring(z.read(sheet_paths['의미판정'])).findall('.//s:c', NS)}
        referents = {c.get('r'): c for c in ET.fromstring(z.read(sheet_paths['지칭판정'])).findall('.//s:c', NS)}
        for i in range(6, 6 + len(data['referents'])):
            assert 'EXACT(' in referents[f'G{i}'].findtext('s:f', namespaces=NS)
        for i, response in enumerate(data['responses'], 6):
            formula = meaning[f'F{i}'].findtext('s:f', namespaces=NS)
            assert 'EXACT(' in formula, formula
            for col, key in [('D', 'expected'), ('E', 'adopted')]:
                cell = meaning[f'{col}{i}']
                value = cell.findtext('s:v', namespaces=NS)
                if cell.get('t') == 's':
                    value = shared[int(value)]
                elif cell.get('t') == 'inlineStr':
                    value = ''.join(cell.find('s:is', NS).itertext())
                else:
                    assert cell.get('t') == 'str', cell.get('t')
                assert canon(json.loads(value)) == canon(response[key])
                if isinstance(response[key].get('conditions'), dict) and 'order' in response[key]['conditions']:
                    assert json.loads(value)['conditions']['order'] == response[key]['conditions']['order']

    html = (OUT / 'outputs/01a15dfe/guide.html').read_text()
    match = re.search(r'const D=(.*?);\nconst \$=', html, re.S)
    assert match, 'embedded HTML ledger missing'
    embedded = json.loads(match.group(1))
    assert embedded['summary'] == summary['configs']
    assert embedded['range_review'] == ranges
    assert len(embedded['responses']) == len(data['responses'])
    responses = {(r['config_id'], r['input_id']): r for r in data['responses']}
    for row in embedded['responses']:
        original = responses[(row['config_id'], row['input_id'])]
        for key in ['first_is_partial', 'first_display_s', 'adopted_at_s', 'adoption_interval',
                    'first_display_node_id', 'full_meaning_adoption_node_id']:
            assert row[key] == original[key], (row['input_id'], key)
    assert len(embedded['stages']) == len(data['stages'])
    for stage in embedded['stages']:
        assert stage['parents'] == stages[(stage['config_id'], stage['id'])]['parents'], stage['id']
    assert embedded['sources'] == data['sources']
    assert 'QA-COMMON-SYN-CONTRACT-v3-STRATIFIED' in html
    assert 'SYNTHETIC_AUTHORED_NOT_MEASURED' in html

    return {'status': 'PASS', 'dataset_id': data['dataset_id'],
            'review_revision': data['review_revision'],
            'evidence_kind': data['evidence_kind'],
            'source_sha256': digest((OUT / 'normalized-data.json').read_bytes()),
            'xlsx_sha256': digest(xlsx.read_bytes()),
            'html_sha256': digest((OUT / 'outputs/01a15dfe/guide.html').read_bytes()),
            'checks': ['published parent Git objects, working files and frozen snapshots are byte-identical',
                       'manifest hashes and generator identity', 'all 1200 causal goal projections',
                       '63 QA and 2 separate voice values reaggregated from serialized ledgers',
                       '49 v2 and all 65 initial v3 values unchanged',
                       'field-aware set/sequence semantics, case and JSON scalar distinctions',
                       '1360 display/adoption boundary joins and 70 partial displays',
                       '9 configuration ranges, denominators, case costs, RAM witnesses and 512 MB status',
                       'all 10 XLSX sheets scanned for cached errors',
                       '65 XLSX QA and 50 cost formula caches match JSON; 4 DELIVERY cost blanks preserved',
                       '1360 exported semantic strings and EXACT formulas; ordered sequences preserved',
                       'embedded HTML summary, ranges, response boundaries, stage dependencies and source rows match JSON'],
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
