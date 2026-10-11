#!/usr/bin/env python3
"""Check ASR-QA source, exact case projection, native PPTX and archived generation."""
from pathlib import Path
from zipfile import ZipFile
from fractions import Fraction
from urllib.parse import unquote
import hashlib
import json
import re
import struct
import subprocess
import sys
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT/'docs/presentations_files'
NS = {'a':'http://schemas.openxmlformats.org/drawingml/2006/main',
      'p':'http://schemas.openxmlformats.org/presentationml/2006/main'}
SYMBOLS = ['○○○','◐○○','●○○','●◐○','●●○','●●◐','●●●']

def text(node):
    return ''.join(t.text or '' for t in node.findall('.//a:t',NS))

def compact(value):
    return re.sub(r'\s+','',value)

def check_links(path):
    body=re.sub(r'```.*?```','',path.read_text(),flags=re.S)
    for target in re.findall(r'!?\[[^\]]*\]\(([^)]+)\)',body):
        target=unquote(target.strip('<>')).split('#',1)[0]
        if not target or re.match(r'^\w+:',target):continue
        assert (path.parent/target).exists(),(path,target)

def archive_check():
    for name,count in [('qa-presentations-before-asr-20261011',32),('qa-common-workload-before-v5-20261011',33),('qa-common-workload-before-v6-20261011',38),('qa-model-reference-before-gemini-20261011',37),('qa-voice-reference-before-realtime-20261011',40)]:
        archive=ROOT/'docs/archive'/name
        manifest=json.loads((archive/'manifest.json').read_text())
        assert len(manifest)==count
        for record in manifest:
            p=archive/record['original_path']
            assert p.stat().st_size==record['bytes'],p
            assert hashlib.sha256(p.read_bytes()).hexdigest()==record['sha256'],p
        for html in archive.rglob('index.html'):
            for image in re.findall(r'<img[^>]+src="([^"]+)"',html.read_text()):
                if image.endswith('.png'):assert (html.parent/image).exists(), (html,image)


def check(kind):
    out=BASE/f'quality-{kind}'
    data=json.loads((out/f'quality-{kind}.json').read_text())
    source=ROOT/data['source']
    digest=hashlib.sha256(source.read_bytes()).hexdigest()
    assert digest==data['source_sha256']
    assert data['contract_version']=='ASR-QA-v6'
    rows=data['rows']
    assert len(rows)==6
    assert [r['id'] for r in rows]==[f'ASR-QA-{n:02d}' for n in range(1,7)]
    assert [r['priority'] for r in rows]==list(range(1,7))
    for row in rows:
        for key in ['name','description','method','target','targetBasis','detail','guards']:
            assert row[key].strip(),(row['id'],key)
        assert not any(s in ' '.join(row[k] for k in ['description','method','target','targetBasis']) for s in ['미정','검토 예정'])
        assert len(row['calculationRows'])==6
        assert row['sources'] and all(k in data['references'] for k in row['sources'])
    assert data==json.loads((BASE/f'quality-{"metrics" if kind=="attributes" else "attributes"}'/f'quality-{"metrics" if kind=="attributes" else "attributes"}.json').read_text())
    validation=json.loads((out/'validation-summary.json').read_text())
    assert validation['source_sha256']==digest and validation['common_workload_sha256']==data['common_workload_sha256']
    assert validation['final_pptx_sha256']==hashlib.sha256((out/f'VIA-quality-{kind}.pptx').read_bytes()).hexdigest()
    assert validation['slide_count']==6 and validation['native_tables']==(6 if kind=='attributes' else 18)
    assert validation['package_integrity_passed'] and validation['font_policy_passed'] and validation['artifact_tool_import_passed']
    assert validation['layout_findings']==validation['layout_warnings']==0
    assert not validation['native_powerpoint_execution_verified'] and not validation['candidate_measurements_executed']
    count=6
    with ZipFile(out/f'VIA-quality-{kind}.pptx') as z:
        assert z.testzip() is None
        slides=[n for n in z.namelist() if re.fullmatch(r'ppt/slides/slide\d+\.xml',n)]
        assert len(slides)==count
        assert not any(n.startswith('ppt/media/') for n in z.namelist()),'All slides must be editable text/tables'
        presentation=ET.fromstring(z.read('ppt/presentation.xml'))
        size=presentation.find('p:sldSz',NS)
        assert (int(size.get('cx')),int(size.get('cy')))==(18288000,10287000)
        for i in range(1,count+1):
            node=ET.fromstring(z.read(f'ppt/slides/slide{i}.xml'))
            tables=node.findall('.//a:tbl',NS)
            assert len(tables)==(1 if kind=='attributes' else 3)
            note=text(ET.fromstring(z.read(f'ppt/notesSlides/notesSlide{i}.xml')))
            assert digest in note and data['contract_version'] in note
            assert data['common_workload_sha256'] in note
            assert compact(data['common_workload_detail']) in compact(note)
            group=(rows[(i-1)*3:i*3] if i<=2 else []) if kind=='attributes' else [rows[i-1]]
            if kind=='attributes' and i>=4:
                body=compact(text(node))
                for c in data['case_mapping'][(i-4)*7:(i-3)*7]:
                    assert c['case_id'] in body and compact(c['name']) in body
                    assert c['case_id'] in note and c['selection_reason'] in note
            for row in group:
                body=compact(text(node))
                assert compact(row['name']) in body
                assert compact(row['description']) in body
                assert compact(row['displayTarget']) in body
                assert row['id'] in body
                for key in ['description','method','target','targetBasis','detail','guards']:
                    assert compact(row[key]) in compact(note),(row['id'],key)
                for key in row['sources']:
                    assert data['references'][key]['url'] in note
            if kind=='metrics':
                grading=tables[-1].findall('a:tr',NS)
                assert len(grading)==8
                for j,tr in enumerate(grading[1:]):
                    cells=tr.findall('a:tc',NS)
                    assert text(cells[0])==SYMBOLS[6-j]
                    assert text(cells[2])==('충족' if j<6 else '미달')
    expected=[f'quality-attributes-{n:02d}.png' for n in range(1,7)] if kind=='attributes' else [f'qa-metric-asr-qa-{n:02d}.png' for n in range(1,7)]
    actual=sorted(p.name for p in out.glob('*.png'))
    assert actual==expected,(actual,expected)
    for name in expected:
        blob=(out/name).read_bytes()
        assert blob[:8]==b'\x89PNG\r\n\x1a\n'
        assert struct.unpack('>II',blob[16:24])==(1920,1080)
    html=(out/'index.html').read_text()
    assert re.findall(r'<img src="([^"]+)"',html)==expected
    for filename in re.findall(r'(?:src|href)="([^"]+)"',html):
        assert (out/filename).exists(),filename
    for path in [BASE/f'quality-{kind}.md',out/'README.md',out/'verification.md']:
        check_links(path)
    print(f'PASS: {count} editable {kind} slides, exact definitions/targets/notes/citations, {count} final PNGs and links')


def common_check():
    subprocess.run([sys.executable,str(ROOT/'scripts/architecture/qa_common_workload.py'),'--check'],check=True)
    subprocess.run([sys.executable,str(ROOT/'scripts/architecture/qa_poc_reference.py'),'--check'],check=True)
    subprocess.run([sys.executable,str(ROOT/'scripts/presentations/build_asr_qa_sources.py'),'--check'],check=True)
    archive_check()
    population=json.loads((BASE/'quality-attributes/functional-coverage.json').read_text())
    assert population['trials']==105 and population['binary_conditions']==630
    cases=population['cases'];assert len(cases)==21
    assert {r['id'] for r in cases}=={f'S{s}-{c:02d}' for s,n in enumerate([2,5,3,3,4,4],1) for c in range(1,n+1)}
    for case in cases:
        assert case['required_conditions']==[f'C{i}' for i in range(1,7)]
        assert all(f'| C{i} ' in case['specification'] for i in range(1,7)),case['id']
        assert all(s in case['specification'] for s in ['**초기 상황:**','**입력:**','**사건 순서:**']),case['id']
    plan=json.loads((BASE/'quality-attributes/evaluation-plan.json').read_text())
    assert list(plan['targets'].values())==[98,1,10,0.50,3,4096]
    rows=json.loads((BASE/'quality-attributes/quality-attributes.json').read_text())['rows']
    parsed=[Fraction(re.search(r'[0-9,]+(?:\.[0-9]+)?',r['target'])[0].replace(',','')) for r in rows]
    assert parsed==[Fraction(str(v)) for v in plan['targets'].values()], 'Front table and numerical contract goals diverged'
    assert plan['populations']['accuracy']==dict(cases=21,repeats=5,trials=105,denominator=630)
    assert plan['populations']['reaction']==dict(events=27,repeats=5,trials=135)
    assert plan['populations']['cycle']==dict(cycles=19,repeats=5,trials=95)
    assert plan['populations']['cost_memory']==dict(workload='P21',user_scenes=19,agent_scenes=2,internal_agent_events=4,independent_runs=5)
    assert plan['populations']['change']==dict(fixtures=6,responsibilities=18)
    sys.path.insert(0,str(ROOT/'scripts/architecture'))
    from qa_poc_reference import score,self_check
    inputs=json.loads((ROOT/'docs/architecture/12-decisions/decision-packages/03-02-poc-inputs.json').read_text())
    self_check(inputs)
    for row in inputs['score_bands']:
        bounds=[Fraction(str(v)) for v in row['bounds']]
        assert bounds[-1]==Fraction(str(plan['targets'][row['id']])), 'Target and 1-point score boundary diverged'
        for i,value in enumerate(bounds):assert score(row,value)==6-i
        # Full valid domain near every boundary, without rounding to display digits.
        for value in bounds:
            for d in [-Fraction(1,10**6),Fraction(1,10**6)]:
                x=value+d
                if x>=0 and (row['direction']!='high' or x<=100):
                    assert 0<=score(row,x)<=6
    d=ROOT/'docs/architecture/12-decisions/decision-packages'
    workload=json.loads((d/'03-02-common-workload.json').read_text())
    manifest=json.loads((d/'03-02-workload-manifest.json').read_text())
    projection=json.loads((BASE/'quality-attributes/quality-attributes.json').read_text())
    assert projection['common_workload']==workload
    assert projection['workload_summary']==manifest['summary']
    assert projection['common_workload_sha256']==manifest['input_sha256']
    assert projection['workload_manifest_sha256']==hashlib.sha256((d/'03-02-workload-manifest.json').read_bytes()).hexdigest()
    archived_source=(ROOT/'docs/archive/qa-common-workload-before-v5-20261011'/projection['source']).read_text()
    specs=lambda t:{cid:body.strip().replace("§3.6.3", "§3.5") for cid,body in re.findall(r'##### (S[1-6]-0[1-3]) [^\n]+\n(.*?)(?=\n<a id=|\n#### 3\.2\.3)',t,re.S)}
    before=specs(archived_source);after=specs((d/'03-02-quality-attribute-definitions.md').read_text())
    assert {k:v for k,v in before.items() if k!='S1-03'}=={k:v for k,v in after.items() if k in before},'Preserved 17 checkpoint oracles changed'
    main=(d/'03-02-quality-attribute-definitions.md').read_text()
    moved=re.search(r'#### M-06 [^\n]+\n(.*?)(?=\n### |\n## )',main,re.S)
    assert moved and before['S1-03'] in moved[1], 'Preference registration regression omitted'
    assert projection['case_mapping']==workload['cases']
    assert len(projection['case_mapping'])==21
    for r in rows:
        visible=' '.join(str(r[k]) for k in ['description','displayTarget','shortMetric','shortBasis','fact','choice','calculationRows','slideMethod','guards'])
        assert not re.search(r'8시간|8h|3session|시간 전용 파생|1\.1325|0\.3675',visible)
    assert 'USD/P21' in inputs['score_bands'][3]['unit']
    assert '50초' in str(rows[3]['calculationRows']) and '179초' in str(rows[3]['calculationRows'])
    assert 'P21' in rows[3]['shortMetric']
    assert 'P21' in rows[5]['shortMetric']
    check_links(BASE/'README.md')
    check_links(BASE/'quality-attributes/measurement-design.md')
    print('PASS: six complete ASR definitions, 21 full case mappings/630 conditions, one P21 5-repeat population, all band boundaries and 180 archive hashes')

if __name__=='__main__':
    common_check()
    for kind in sys.argv[1:] or ['attributes','metrics']:check(kind)
