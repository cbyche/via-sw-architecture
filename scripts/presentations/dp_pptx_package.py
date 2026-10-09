"""Preserve native DP slides when updating or reordering the presentation.

Lookup uses authored scene names and presentation relationships, never physical
slide numbers. Only overview/notes/order metadata change for --intro-flow-only.
"""
from __future__ import annotations

import argparse
import html
import json
import posixpath
import re
import zipfile
from pathlib import Path
import xml.etree.ElementTree as E

P = 'http://schemas.openxmlformats.org/presentationml/2006/main'
R = 'http://schemas.openxmlformats.org/officeDocument/2006/relationships'
A = 'http://schemas.openxmlformats.org/drawingml/2006/main'
NS = {'p': P, 'a': A}
SECTION_URI = '{521415D9-36F7-43E2-AB2F-B90AF26B5E84}'


def rels_path(part):
    folder, name = posixpath.split(part)
    return f'{folder}/_rels/{name}.rels'


def target_part(source, target):
    return target.lstrip('/') if target.startswith('/') else posixpath.normpath(posixpath.join(posixpath.dirname(source), target))


def related(parts, source, kind):
    for rel in E.fromstring(parts[rels_path(source)]):
        if rel.get('Type', '').endswith('/'+kind):
            return target_part(source, rel.get('Target'))
    raise AssertionError((source, kind, 'relationship missing'))


def ordered_slides(parts):
    rels = {r.get('Id'): target_part('ppt/presentation.xml', r.get('Target'))
            for r in E.fromstring(parts['ppt/_rels/presentation.xml.rels'])}
    result = []
    for node in E.fromstring(parts['ppt/presentation.xml']).find('p:sldIdLst', NS):
        part = rels[node.get(f'{{{R}}}id')]
        names = [n.get('name', '') for n in E.fromstring(parts[part]).findall('.//p:cNvPr', NS)]
        slugs = {name.split(' / ')[0] for name in names if re.match(r'(dp-background-overview|dp4[1-5]-(?:background|comparison)) / ', name)}
        assert len(slugs) == 1, (part, slugs)
        result.append(dict(slug=slugs.pop(), part=part, id=node.get('id'), rid=node.get(f'{{{R}}}id')))
    assert len({s['slug'] for s in result}) == len(result)
    return result


def read_package(path):
    with zipfile.ZipFile(path) as z:
        return {i.filename: z.read(i.filename) for i in z.infolist()}


def replace_related(parts, candidate, old_part, new_part):
    """Native slide and notes only; keep original layout/master relationships."""
    parts[old_part] = candidate[new_part]
    old_notes = related(parts, old_part, 'notesSlide')
    new_notes = related(candidate, new_part, 'notesSlide')
    parts[old_notes] = candidate[new_notes]


def add_overview(parts, candidate):
    scene = next(s for s in ordered_slides(candidate) if s['slug'] == 'dp-background-overview')
    old_slide = scene['part']
    old_note = related(candidate, old_slide, 'notesSlide')
    slide_no = 1+max(int(re.search(r'slide(\d+)\.xml$', p)[1]) for p in parts if re.fullmatch(r'ppt/slides/slide\d+\.xml', p))
    note_no = 1+max(int(re.search(r'notesSlide(\d+)\.xml$', p)[1]) for p in parts if re.fullmatch(r'ppt/notesSlides/notesSlide\d+\.xml', p))
    new_slide, new_note = f'ppt/slides/slide{slide_no}.xml', f'ppt/notesSlides/notesSlide{note_no}.xml'
    parts[new_slide], parts[new_note] = candidate[old_slide], candidate[old_note]
    parts[rels_path(new_slide)] = candidate[rels_path(old_slide)].replace(('/'+old_note).encode(), ('/'+new_note).encode())
    parts[rels_path(new_note)] = candidate[rels_path(old_note)].replace(('/'+old_slide).encode(), ('/'+new_slide).encode())
    # Native overview uses the same existing layout and notes master, no media.
    for source in (new_slide, new_note):
        for rel in E.fromstring(parts[rels_path(source)]):
            assert target_part(source, rel.get('Target')) in parts
    assert related(parts, new_slide, 'notesSlide') == new_note
    assert related(parts, new_note, 'slide') == new_slide
    slides = ordered_slides(parts)
    rid = 'RviaOverview'
    assert rid not in {r.get('Id') for r in E.fromstring(parts['ppt/_rels/presentation.xml.rels'])}
    extra = f'<Relationship Type="{R}/slide" Target="/{new_slide}" Id="{rid}" />'
    parts['ppt/_rels/presentation.xml.rels'] = parts['ppt/_rels/presentation.xml.rels'].replace(b'</Relationships>', extra.encode()+b'</Relationships>')
    extra = ''.join(f'<Override PartName="/{part}" ContentType="application/vnd.openxmlformats-officedocument.presentationml.{kind}+xml" />' for part, kind in ((new_slide, 'slide'), (new_note, 'notesSlide')))
    parts['[Content_Types].xml'] = parts['[Content_Types].xml'].replace(b'</Types>', extra.encode()+b'</Types>')
    return dict(slug='dp-background-overview', part=new_slide, id=str(1+max(int(s['id']) for s in slides)), rid=rid)


def append_transition(parts, slide, transition):
    note = related(parts, slide['part'], 'notesSlide')
    if transition.encode() in parts[note]:
        return
    extra = ''.join(f'<a:p xmlns:a="{A}"><a:r><a:t>{html.escape(t)}</a:t></a:r></a:p>' for t in ('다음 주제로 연결', transition))
    assert b'</p:txBody>' in parts[note]
    parts[note] = parts[note].replace(b'</p:txBody>', extra.encode()+b'</p:txBody>', 1)


def set_presentation_order(parts, slides):
    ids = '<p:sldIdLst>'+''.join(f'<p:sldId id="{s["id"]}" r:id="{s["rid"]}" xmlns:r="{R}" />' for s in slides)+'</p:sldIdLst>'
    text = parts['ppt/presentation.xml'].decode()
    text = re.sub(r'<p:sldIdLst>.*?</p:sldIdLst>', lambda _: ids, text)
    # PowerPoint native sections, per MS-PPTX §3.3. Content of 43 is untouched.
    section = f'<p:ext uri="{SECTION_URI}"><p14:sectionLst xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main">'
    for name, section_id, members in [('본 발표', '{82000001-0000-4000-8000-000000000001}', slides[:9]), ('부록 · 보충 자료', '{82000001-0000-4000-8000-000000000002}', slides[9:])]:
        section += f'<p14:section name="{name}" id="{section_id}"><p14:sldIdLst>'+''.join(f'<p14:sldId id="{s["id"]}" />' for s in members)+'</p14:sldIdLst></p14:section>'
    section += '</p14:sectionLst></p:ext>'
    if SECTION_URI in text:
        text = re.sub(r'<p:ext uri="'+re.escape(SECTION_URI)+r'">.*?</p:ext>', lambda _: section, text)
    elif '</p:extLst>' in text:
        text = text.replace('</p:extLst>', section+'</p:extLst>')
    else:
        text = text.replace('</p:presentation>', '<p:extLst>'+section+'</p:extLst></p:presentation>')
    parts['ppt/presentation.xml'] = text.encode()
    parts['docProps/app.xml'] = re.sub(rb'<ap:(Slides|Notes)>\d+</ap:\1>', lambda m: b'<ap:'+m[1]+b'>'+str(len(slides)).encode()+b'</ap:'+m[1]+b'>', parts['docProps/app.xml'])


def update_package(candidate_path, original_path, selected, intro=False):
    candidate, parts = read_package(candidate_path), read_package(original_path)
    originals = {s['slug']: s for s in ordered_slides(parts)}
    generated = {s['slug']: s for s in ordered_slides(candidate)}
    for slug in selected:
        assert slug in originals and slug in generated, (slug, 'missing scene')
        replace_related(parts, candidate, originals[slug]['part'], generated[slug]['part'])
    if intro:
        import sys
        sys.path.insert(0, str(Path(__file__).resolve().parents[1]/'architecture'))
        from generate_dp_background_slides import PRESENTATION_ORDER, TRANSITIONS
        if 'dp-background-overview' in originals:
            replace_related(parts, candidate, originals['dp-background-overview']['part'], generated['dp-background-overview']['part'])
        else:
            originals['dp-background-overview'] = add_overview(parts, candidate)
        order = ['dp-background-overview']+[f'dp{n}-{kind}' for n in (*PRESENTATION_ORDER, 43) for kind in ('background', 'comparison')]
        assert set(order) == set(originals)
        for n, transition in TRANSITIONS.items():
            append_transition(parts, originals[f'dp{n}-comparison'], transition)
        set_presentation_order(parts, [originals[slug] for slug in order])
    with zipfile.ZipFile(original_path) as old, zipfile.ZipFile(str(candidate_path)+'.scoped', 'w') as out:
        for info in old.infolist():
            out.writestr(info, parts.pop(info.filename))
        for name, data in parts.items():
            out.writestr(name, data)
    Path(str(candidate_path)+'.scoped').replace(candidate_path)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('candidate', type=Path)
    parser.add_argument('original', type=Path)
    parser.add_argument('selected', help='JSON scene slugs')
    parser.add_argument('--intro', action='store_true')
    parser.add_argument('--sections-only', action='store_true')
    args = parser.parse_args()
    if args.sections_only:
        parts = read_package(args.candidate)
        slides = ordered_slides(parts)
        assert len(slides) == 11 and slides[0]['slug'] == 'dp-background-overview'
        set_presentation_order(parts, slides)
        with zipfile.ZipFile(args.candidate) as original, zipfile.ZipFile(str(args.candidate)+'.sections', 'w') as out:
            for info in original.infolist():
                out.writestr(info, parts[info.filename])
        Path(str(args.candidate)+'.sections').replace(args.candidate)
    else:
        update_package(args.candidate, args.original, json.loads(args.selected), args.intro)
