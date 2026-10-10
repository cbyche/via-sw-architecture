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
    archive=ROOT/'docs/archive/qa-presentations-before-asr-20261011'
    manifest=json.loads((archive/'manifest.json').read_text())
    assert len(manifest)==32
    for record in manifest:
        p=archive/record['original_path']
        assert p.stat().st_size==record['bytes'],p
        assert hashlib.sha256(p.read_bytes()).hexdigest()==record['sha256'],p


def check(kind):
    out=BASE/f'quality-{kind}'
    data=json.loads((out/f'quality-{kind}.json').read_text())
    source=ROOT/data['source']
    digest=hashlib.sha256(source.read_bytes()).hexdigest()
    assert digest==data['source_sha256']
    assert data['contract_version']=='ASR-QA-v4'
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
    count=2 if kind=='attributes' else 6
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
            group=rows[(i-1)*3:i*3] if kind=='attributes' else [rows[i-1]]
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
    expected=[f'quality-attributes-{n:02d}.png' for n in range(1,3)] if kind=='attributes' else [f'qa-metric-asr-qa-{n:02d}.png' for n in range(1,7)]
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
    subprocess.run([sys.executable,str(ROOT/'scripts/presentations/build_asr_qa_sources.py'),'--check'],check=True)
    archive_check()
    population=json.loads((BASE/'quality-attributes/functional-coverage.json').read_text())
    assert population['trials']==90 and population['binary_conditions']==540
    cases=population['cases'];assert len(cases)==18
    assert {r['id'] for r in cases}=={f'S{s}-0{c}' for s in range(1,7) for c in range(1,4)}
    for case in cases:
        assert case['required_conditions']==[f'C{i}' for i in range(1,7)]
        assert all(f'| C{i} ' in case['specification'] for i in range(1,7)),case['id']
        assert all(s in case['specification'] for s in ['**초기 상황:**','**입력:**','**사건 순서:**']),case['id']
    plan=json.loads((BASE/'quality-attributes/evaluation-plan.json').read_text())
    assert list(plan['targets'].values())==[98,1,10,1.5,3,4096]
    rows=json.loads((BASE/'quality-attributes/quality-attributes.json').read_text())['rows']
    parsed=[Fraction(re.search(r'[0-9,]+(?:\.[0-9]+)?',r['target'])[0].replace(',','')) for r in rows]
    assert parsed==[Fraction(str(v)) for v in plan['targets'].values()], 'Front table and numerical contract goals diverged'
    assert plan['populations']['accuracy']==dict(cases=18,repeats=5,trials=90,denominator=540)
    assert plan['populations']['reaction']['trials']==plan['populations']['cycle']['trials']==30
    assert plan['populations']['cost_memory']==dict(hours=8,sessions=3,goals=100)
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
    check_links(BASE/'README.md')
    check_links(BASE/'quality-attributes/measurement-design.md')
    print('PASS: six complete ASR definitions, 18 exact cases/540 conditions, all band boundaries and 32 archive hashes')

if __name__=='__main__':
    common_check()
    for kind in sys.argv[1:] or ['attributes','metrics']:check(kind)
