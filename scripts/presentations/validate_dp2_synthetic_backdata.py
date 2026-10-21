"""Check authored consistency and saved workbook caches; never performance validation."""
import json
import math
import pathlib
import xml.etree.ElementTree as ET
import zipfile
from collections import Counter

ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/presentations_files/dp2-synthetic-backdata-20261021'
d = json.loads((OUT / 'synthetic-data.json').read_text())
assert d['evidence_kind'] == 'SYNTHETIC_AUTHORED_NOT_MEASURED'
assert d['actual_model_calls'] == 0
assert Counter(c['group'] for c in d['cases']) == dict.fromkeys(['G1', 'G2', 'G3', 'G4'], 50)
assert len({c['case_id'] for c in d['cases']}) == 200
source_ids = {s['id'] for s in d['sources']}
for c in d['cases']:
    assert set(c['source_ids']) <= source_ids
for r in d['responses']:
    assert r['qa07'] == int(r['oracle'] == r['adopted'])
    for target in r['adopted']['targets']:
        assert target in source_ids
for r in d['referents']:
    assert r['correct'] == int(set(r['expected']) == set(r['adopted']))
    assert set(r['expected']) | set(r['adopted']) <= source_ids
events = {}
for e in d['events']:
    assert e['event_id'] not in events
    parents = [events[p] for p in e['parents']]
    assert all(p['case_id'] == e['case_id'] and p['option'] == e['option'] for p in parents)
    expected_start = max((p['finish_s'] for p in parents), default=0)
    projected = max((p['projected_finish_s'] for p in parents), default=0)
    projected += e['duration_s'] if e['count_in_qa03'] else 0
    assert math.isclose(e['start_s'], expected_start, abs_tol=1e-8)
    assert math.isclose(e['projected_finish_s'], projected, abs_tol=1e-8)
    events[e['event_id']] = e
for t in d['times']:
    prefix = f"{t['case_id']}-{t['option']}"
    assert math.isclose(events[prefix+'-VOICE']['projected_finish_s'], t['via'], abs_tol=1e-8)
    assert math.isclose(events[prefix+'-DISPLAY']['finish_s']+t['goal_start_s'], t['first'], abs_tol=1e-8)
    assert t['full_meaning_at'] >= 0
for c in d['companions']:
    if c['status_adopted'] == 'CANCEL':
        assert c['synthetic_via_completion_s'] is None
        assert c['terminal_kind'] == 'WRONGLY_CANCELLED_NOT_COMPLETED'
for c in d['costRows']:
    assert c['scope'] == 'VIA_ONLY'
    assert math.isclose(c['quantity']*c['unit_price'], c['cost'], abs_tol=1e-9)
    if c['bucket'].endswith('token'):
        assert int(c['quantity']) == c['quantity']
for m in d['memory']:
    assert m['total'] == sum(m[k] for k in ['base','input_buffer','context','result_buffer','execution'])

ns = {'s': 'http://schemas.openxmlformats.org/spreadsheetml/2006/main'}
file = OUT / 'outputs/01a154e2/DP2-synthetic-backdata.xlsx'
with zipfile.ZipFile(file) as z:
    sheets = [n for n in z.namelist() if n.startswith('xl/worksheets/sheet') and n.endswith('.xml')]
    assert len(sheets) == 14
    errors = []
    for name in sheets:
        root = ET.fromstring(z.read(name))
        errors.extend((name,c.attrib['r'],c.findtext('s:v',namespaces=ns)) for c in root.findall('.//s:c',ns) if c.attrib.get('t') == 'e')
    assert not errors, errors
    root = ET.fromstring(z.read('xl/worksheets/sheet1.xml'))
    cells = {c.attrib['r']:c for c in root.findall('.//s:c',ns)}
    for option,column in [('A','C'),('B','D')]:
        for row in range(6,13):
            cell = cells[f'{column}{row}']
            assert cell.find('s:f',ns) is not None
            value = float(cell.findtext('s:v',namespaces=ns))
            assert math.isclose(value,d['summary'][option][f'qa0{row-5}'],abs_tol=1e-8)

report = {'status':'AUTHORED_CONSISTENCY_CHECKED_NOT_MEASURED','saved_workbook_sheets':14,'saved_formula_errors':0,'summary_cached_values_match':True,'source_links_and_oracles_checked':True,'dag_paths_checked':True,'cancelled_companion_not_counted_complete':True,'cost_quantities_and_units_checked':True,'real_excel_application_checked':False}
(OUT / 'saved-workbook-check.json').write_text(json.dumps(report,ensure_ascii=False,indent=2))
print(json.dumps(report,ensure_ascii=False))
