"""Check presentation order, notes and scoped updates after physical part remapping."""
from __future__ import annotations

import argparse
import json
import re
import tempfile
import zipfile
from pathlib import Path
import sys
import xml.etree.ElementTree as E

from dp_pptx_package import NS, ordered_slides, read_package, related, update_package

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'scripts/architecture'))
from generate_dp_background_slides import OVERVIEW_SCRIPT, TRANSITIONS

EXPECTED = ['dp-background-overview']+[f'dp{n}-{k}' for n in (41, 44, 45, 42, 43) for k in ('background', 'comparison')]
DECK = ROOT/'docs/presentations_files/VIA-DP-background-and-comparison-41-45.pptx'


def note_text(parts, slide):
    return '\n'.join(n.text or '' for n in E.fromstring(parts[related(parts, slide['part'], 'notesSlide')]).findall('.//a:t', NS))


def permuted_candidate(parts):
    """A valid candidate with different physical slide/notes numbering."""
    names = {name: re.sub(r'(notesSlide|slide)(\d+)(\.xml)', lambda m: m[1]+str(int(m[2])+20)+m[3], name)
             for name in parts if re.search(r'(?:notesSlide|slide)\d+\.xml', name)}
    result = {}
    for name, data in parts.items():
        if name.endswith('.rels') or name == '[Content_Types].xml':
            for old, new in names.items():
                if old.endswith('.xml'):
                    data = data.replace(('/'+old).encode(), ('/'+new).encode())
        result[names.get(name, name)] = data
    for slide in ordered_slides(result):
        if slide['slug'] == 'dp-background-overview':
            continue
        result[slide['part']] += f'<!-- selected {slide["slug"]} -->'.encode()
        result[related(result, slide['part'], 'notesSlide')] += f'<!-- notes {slide["slug"]} -->'.encode()
    return result


def write_package(path, parts):
    with zipfile.ZipFile(path, 'w', zipfile.ZIP_DEFLATED) as z:
        for name, data in parts.items():
            z.writestr(name, data)


def check_scopes(selection_json, comparison_subset=False):
    plans = json.loads(selection_json.read_text())
    integrated = next(p for p in plans if p['file'] == str(DECK.relative_to(ROOT)))
    expected_positions = {'--intro-flow-only': [1], '--dp41-42-only': [3, 9], '--dp44-only': [5], '--dp45-only': [6, 7], '--comparison-only': [3, 5, 7, 9, 11]}
    if comparison_subset:
        expected_positions['--dp45-only'] = [7]
    for scope, positions in expected_positions.items():
        assert [s['position'] for s in integrated['selections'][scope]] == positions, scope
    count = 0
    with tempfile.TemporaryDirectory(prefix='via-pptx-scope-') as tmp:
        for plan in plans:
            original = read_package(ROOT/plan['file'])
            permuted = permuted_candidate(original)
            for scope, selection in plan['selections'].items():
                if scope == '--intro-flow-only':
                    if selection:
                        candidate, baseline = Path(tmp)/'intro.pptx', Path(tmp)/'intro-before.pptx'
                        write_package(candidate, permuted)
                        write_package(baseline, original)
                        update_package(candidate, baseline, [], intro=True)
                        assert read_package(candidate) == original, 'intro rerun duplicated notes or altered DP parts'
                        count += 1
                    continue
                if not selection:
                    continue
                selected = [s['slug'] for s in selection]
                candidate, baseline = Path(tmp)/'candidate.pptx', Path(tmp)/'original.pptx'
                write_package(candidate, permuted)
                write_package(baseline, original)
                update_package(candidate, baseline, selected)
                patched = read_package(candidate)
                changed = {n for n in original if original[n] != patched[n]}
                expected_parts = {n for s in ordered_slides(original) if s['slug'] in selected
                                  for n in (s['part'], related(original, s['part'], 'notesSlide'))}
                assert changed == expected_parts, (plan['file'], scope, changed, expected_parts)
                assert set(original) == set(patched)
                count += 1
    print(f'PASS: {count} scoped package updates with permuted physical part numbers')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--selection-json', type=Path)
    parser.add_argument('--comparison-subset-json', type=Path)
    parser.add_argument('--before', type=Path, help='Optional original 10/11-slide package for preservation check')
    args = parser.parse_args()
    parts = read_package(DECK)
    slides = ordered_slides(parts)
    assert [s['slug'] for s in slides] == EXPECTED
    sections = E.fromstring(parts['ppt/presentation.xml']).findall('.//{http://schemas.microsoft.com/office/powerpoint/2010/main}section')
    assert [(s.get('name'), len(s[0])) for s in sections] == [('본 발표', 9), ('부록 · 보충 자료', 2)]
    assert [[s.get('id') for s in section[0]] for section in sections] == [[s['id'] for s in slides[:9]], [s['id'] for s in slides[9:]]]
    assert OVERVIEW_SCRIPT in note_text(parts, slides[0])
    for n, text in TRANSITIONS.items():
        slide = next(s for s in slides if s['slug'] == f'dp{n}-comparison')
        assert note_text(parts, slide).count(text) == 1
    for slide in slides:
        root = E.fromstring(parts[slide['part']])
        assert not root.findall('.//p:pic', NS)
        assert len(root.findall('.//p:sp', NS)) > 50
    if args.before:
        original = read_package(args.before)
        by_slug = {s['slug']: s for s in slides}
        for old in ordered_slides(original):
            new = by_slug[old['slug']]
            if old['slug'] == 'dp-background-overview':
                continue
            assert original[old['part']] == parts[new['part']], old['slug']+' native slide changed'
            if old['slug'] not in {f'dp{n}-comparison' for n in TRANSITIONS}:
                assert note_text(original, old) == note_text(parts, new), old['slug']+' notes changed'
        allowed = {'ppt/presentation.xml', 'ppt/_rels/presentation.xml.rels', '[Content_Types].xml', 'docProps/app.xml'}
        allowed.update(related(original, s['part'], 'notesSlide') for s in ordered_slides(original) if s['slug'] in {f'dp{n}-comparison' for n in TRANSITIONS})
        if 'dp-background-overview' in {s['slug'] for s in ordered_slides(original)}:
            overview = next(s for s in ordered_slides(original) if s['slug'] == 'dp-background-overview')
            allowed.update([overview['part'], related(original, overview['part'], 'notesSlide')])
        assert {n for n in original if original[n] != parts.get(n)} <= allowed
        print('PASS: original DP slides, unrelated notes and common package parts preserved')
    if args.selection_json:
        check_scopes(args.selection_json)
    if args.comparison_subset_json:
        check_scopes(args.comparison_subset_json, comparison_subset=True)
    print('PASS: 11 native slides, main 9 + appendix 2, script and transitions')


if __name__ == '__main__':
    main()
