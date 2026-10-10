#!/usr/bin/env python3
"""Verify QA goal preservation, exhaustive score boundaries and editable exports."""
from fractions import Fraction
from pathlib import Path
from zipfile import ZipFile
import json
import re
import struct
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/presentations_files/quality-metrics'
NS = {'a': 'http://schemas.openxmlformats.org/drawingml/2006/main'}


def compact(s):
    return re.sub(r'\s+', '', s)


def texts(node):
    return ''.join(t.text or '' for t in node.findall('.//a:t', NS))


def intervals(r):
    b = list(map(Fraction, r['bounds']))
    if r['kind'] == 'high':
        return [(6, b[0], b[0], True, True)] + [
            (5-i, b[i+1], b[i], True, False) for i in range(5)
        ] + [(0, Fraction(0), b[5], True, False)]
    if r['kind'] == 'low':
        return [(6, Fraction(0), b[0], True, True)] + [
            (5-i, b[i], b[i+1], False, True) for i in range(5)
        ] + [(0, b[5], None, False, False)]
    return [(6-i, b[i], b[i], True, True) for i in range(6)] + [
        (0, Fraction(6), None, True, False)
    ]


def score(r, value):
    x = Fraction(value)
    if x < 0 or (r['kind'] == 'high' and x > 100):
        raise ValueError('Invalid measurement domain')
    if r['kind'] in ('missing', 'events') and x.denominator != 1:
        raise ValueError('Counts must be integers')
    if r['id'] == 'V-03' and x > 94:
        raise ValueError('Missing variants exceed the 94-variant contract')
    matches = []
    for s, lo, hi, include_lo, include_hi in intervals(r):
        if (x > lo or include_lo and x == lo) and (
            hi is None or x < hi or include_hi and x == hi
        ):
            matches.append(s)
    assert len(matches) == 1, (r['id'], x, matches)
    return matches[0]


def rendered_ranges(r):
    s, b = r['symbol'], r['bounds']
    if r['kind'] == 'high':
        return [f'{s} = {b[0]}%'] + [f'{b[i+1]}% ≤ {s} < {b[i]}%' for i in range(5)] + [f'0% ≤ {s} < {b[5]}%']
    if r['kind'] == 'low':
        return [f'0 ≤ {s} ≤ {b[0]}'] + [f'{b[i]} < {s} ≤ {b[i+1]}' for i in range(5)] + [f'{s} > {b[5]}']
    v = s if r['kind'] == 'events' else 'm'
    return [f'{v} = {n}{r["unit"]}' for n in range(6)] + [f'{v} ≥ 6{r["unit"]}']


def main():
    data = json.loads((OUT/'quality-metrics.json').read_text())
    baseline = json.loads((OUT/data['targetSource']).read_text())
    rows = data['rows']
    assert len(rows) == 10 and data['scoreScale'] == list(range(7))
    assert data['status'] == 'PROPOSED_PRESENTATION_RUBRIC_NOT_FROZEN_NOT_MEASURED'
    assert [r['id'] for r in rows] == [r['id'] for r in baseline['rows']]
    for r, old in zip(rows, baseline['rows']):
        for key in ('id','name','priority','description','target','metric'):
            assert r[key] == old[key], (r['id'], key)
        assert len(r['bounds']) == 6
        assert r['sources'] and all(s in data['references'] for s in r['sources'])
        if r['kind'] in ('high','low'):
            assert Fraction(r['bounds'][-1]) == Fraction(r['metric']['value'])
            differences = [Fraction(r['bounds'][i+1])-Fraction(r['bounds'][i]) for i in range(5)]
            assert len(set(differences)) == 1 and differences[0] != 0
            top = 100 if r['kind'] == 'high' else max(Fraction(r['bounds'][-1])*2, 2)
            points = {Fraction(i,100)*top for i in range(101)}
            for boundary in map(Fraction,r['bounds']):
                points.update((boundary,boundary-Fraction(1,10**8),boundary+Fraction(1,10**8)))
            points = sorted(x for x in points if x >= 0 and (r['kind'] != 'high' or x <= 100))
            scores = [score(r,x) for x in points]
            assert set(scores) == set(range(7))
            assert scores == sorted(scores,reverse=r['kind']=='low')
        else:
            assert r['bounds'] == ['0','1','2','3','4','5']
            assert [score(r,n) for n in range(8)] == [6,5,4,3,2,1,0,0]

    byid = {r['id']:r for r in rows}
    # Actual finite-population thresholds and misleading rounded values.
    assert score(byid['V-01'], Fraction(1786*100,1880)) == 1
    assert score(byid['V-01'], Fraction(1785*100,1880)) == 0
    assert score(byid['V-01'], Fraction(1805*100,1880)) == 2
    assert score(byid['V-02'], '0.6004') == 4
    assert score(byid['V-02'], '0.6') == 5
    assert score(byid['V-08'], Fraction(17,6)) == 1
    assert score(byid['V-08'], Fraction(16,6)) == 2
    assert score(byid['V-06'], Fraction(16_000_000_000,10**9)) == 1
    assert score(byid['V-06'], Fraction(16_000_000_001,10**9)) == 0

    md_path = OUT.parent/'quality-metrics.md'
    md = md_path.read_text()
    assert len(re.findall(r'^## V-\d\d ',md,re.M)) == 10
    assert 'UNMEASURED' in md and '95% 구간' in md and '등급 차이 불확실' in md
    assert 'QA 사이에서 점수 크기를 비교' in md
    assert '로그 유실, 오답과 판단 불가 및 5분 내 미제출' in md
    assert '미전달도 실패로 분모에 남긴다' in md
    assert '이미 관측한 위반' in md
    for r in rows:
        assert r['target'] in md and r['detail'] in md and r['scoringBasis'] in md
        for band in rendered_ranges(r):
            assert band in md
        name = f'qa-metric-{r["id"].lower().replace("-", "")}.png'
        png = (OUT/name).read_bytes()
        assert png[:8] == b'\x89PNG\r\n\x1a\n'
        assert struct.unpack('>II',png[16:24]) == (1920,1080)

    with ZipFile(OUT/'VIA-quality-metrics.pptx') as z:
        slides = [n for n in z.namelist() if re.fullmatch(r'ppt/slides/slide\d+\.xml',n)]
        assert len(slides) == 10
        for i,r in enumerate(rows,1):
            slide = ET.fromstring(z.read(f'ppt/slides/slide{i}.xml'))
            native = slide.findall('.//a:tbl',NS)
            assert len(native) == 3, (r['id'], len(native))
            slide_text = texts(slide)
            assert r['target'] in slide_text and r['description'] in slide_text
            for field in ('fact','choice','slideMethod','slideNote','calculation'):
                assert compact(r[field]) in compact(slide_text), (r['id'],field)
            actual = [[texts(c) for c in tr.findall('a:tc',NS)] for tr in native[2].findall('a:tr',NS)][1:]
            assert len(actual) == 7
            for j,(s,actual_band,goal) in enumerate(actual):
                expected_score=6-j
                assert s == str(expected_score)
                assert actual_band == rendered_ranges(r)[j], (r['id'], actual_band)
                passed = expected_score > 0 if r['kind'] in ('high','low') else expected_score == 6
                assert goal == ('충족' if passed else '미달')
            notes = texts(ET.fromstring(z.read(f'ppt/notesSlides/notesSlide{i}.xml')))
            for field in ('detail','scoringBasis'):
                assert compact(r[field]) in compact(notes), (r['id'],field)
            for s in r['sources']:
                assert data['references'][s][1] in notes

    for p in [md_path,*OUT.glob('*.md'),*OUT.glob('*.json'),OUT/'index.html']:
        content=p.read_text()
        assert not re.search(r'[\u00b7\u2027\u2219\u30fb]',content), p
        if p.suffix == '.md':
            for target in re.findall(r'!?\[[^\]]*\]\(([^)]+)\)',content):
                local=target.split('#',1)[0]
                if local and not local.startswith(('https:','http:')):
                    assert (p.parent/local).exists(), (p,local)
    print('PASS: 10 preserved goals, 70 exact score ranges, exhaustive boundaries, 30 native tables, notes and 10 PNGs')


if __name__ == '__main__':
    main()
