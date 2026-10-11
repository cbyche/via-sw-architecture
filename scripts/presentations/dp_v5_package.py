"""Remove withdrawn DP43 from the integrated v4 while preserving native slides.

The current presentation is maintained as one integrated file. The comparison
v4 is a reference input to verify content inclusion, never a v5 output.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET
import zipfile

from dp_v2_intro_package import read, slides
from dp_pptx_package import A, NS, P, R, related, rels_path, target_part

P14 = 'http://schemas.microsoft.com/office/powerpoint/2010/main'


def authored_slug(parts, slide):
    names = {name.split(' / ')[0] for name in slide['names'] if ' / ' in name}
    # Summary objects share a comparison slide with its original scene.
    primary = {name for name in names if not name.endswith('-v4-summary')}
    if len(primary) != 1:
        raise ValueError(f'Expected one authored scene: {primary}')
    return primary.pop()


def semantic_slide(parts, slide):
    """Compare native content/geometry, ignoring generated object identities."""
    root = ET.fromstring(parts[slide['part']])
    for node in root.iter():
        if node.tag.split('}')[-1] in ('cNvPr', 'creationId'):
            node.attrib.pop('id', None)
        if node.tag.split('}')[-1] == 'creationId':
            node.attrib.pop('val', None)
    return ET.tostring(root.find('p:cSld', NS))


def verify_subset(integrated, comparison):
    sources = {authored_slug(integrated, s): s for s in slides(integrated)}
    report = []
    for slide in slides(comparison):
        slug = authored_slug(comparison, slide)
        match = sources[slug]
        if semantic_slide(comparison, slide) != semantic_slide(integrated, match):
            raise ValueError(f'Comparison content/geometry differs: {slug}')
        report.append(dict(scene=slug, comparison_part=slide['part'], integrated_part=match['part'],
                           native_content_and_geometry_equal=True))
    return report


def serialize(root):
    ET.register_namespace('p', P)
    ET.register_namespace('a', A)
    ET.register_namespace('r', R)
    ET.register_namespace('p14', P14)
    if root.tag == '{http://schemas.openxmlformats.org/package/2006/content-types}Types':
        ET.register_namespace('', 'http://schemas.openxmlformats.org/package/2006/content-types')
    elif root.tag == '{http://schemas.openxmlformats.org/package/2006/relationships}Relationships':
        ET.register_namespace('', 'http://schemas.openxmlformats.org/package/2006/relationships')
    return ET.tostring(root, encoding='utf-8', xml_declaration=True)


def build(source: Path, comparison: Path, destination: Path, report_path: Path):
    if destination.exists():
        raise FileExistsError(destination)
    before = read(source)
    subset = verify_subset(before, read(comparison))
    parts = dict(before)
    original_slides = slides(before)
    withdrawn = [s for s in original_slides if authored_slug(before, s).startswith('dp43-')]
    retained = [s for s in original_slides if s not in withdrawn]
    if len(original_slides) != 16 or len(withdrawn) != 2 or len(retained) != 14:
        raise ValueError('Expected integrated v4 with 16 slides, including exactly two DP43 slides.')

    dropped_parts = set()
    for slide in withdrawn:
        notes = related(before, slide['part'], 'notesSlide')
        dropped_parts.update((slide['part'], rels_path(slide['part']), notes, rels_path(notes)))
    for name in dropped_parts:
        parts.pop(name)

    ids = {s['id'] for s in withdrawn}
    rids = {s['rid'] for s in withdrawn}
    presentation = ET.fromstring(before['ppt/presentation.xml'])
    ordered = presentation.find('p:sldIdLst', NS)
    for node in list(ordered):
        if node.get('id') in ids:
            ordered.remove(node)
    # Remove references to withdrawn slides and then empty appendix sections.
    for parent in presentation.iter():
        for child in list(parent):
            if child.tag == f'{{{P14}}}sldId' and child.get('id') in ids:
                parent.remove(child)
            elif child.tag == f'{{{P}}}sld' and child.get(f'{{{R}}}id') in rids:
                parent.remove(child)
    for parent in presentation.iter():
        for child in list(parent):
            if child.tag == f'{{{P14}}}section' and not child.findall(f'.//{{{P14}}}sldId'):
                parent.remove(child)
    parts['ppt/presentation.xml'] = serialize(presentation)

    relationships = ET.fromstring(before['ppt/_rels/presentation.xml.rels'])
    for node in list(relationships):
        if node.get('Id') in rids:
            relationships.remove(node)
    parts['ppt/_rels/presentation.xml.rels'] = serialize(relationships)
    types = ET.fromstring(before['[Content_Types].xml'])
    for node in list(types):
        if node.get('PartName', '').lstrip('/') in dropped_parts:
            types.remove(node)
    parts['[Content_Types].xml'] = serialize(types)
    parts['docProps/app.xml'] = re.sub(
        rb'<ap:(Slides|Notes)>\d+</ap:\1>',
        lambda match: b'<ap:' + match[1] + b'>14</ap:' + match[1] + b'>',
        before['docProps/app.xml'],
    )

    # A retained DP41 background note still described DP43 as a comparison.
    edited_notes = set()
    for slide in retained:
        note = related(before, slide['part'], 'notesSlide')
        raw = parts[note]
        changed = raw.replace('의미 생산의 기능 분할(43), '.encode(), b'')
        if changed != raw:
            parts[note] = changed
            edited_notes.add(note)

    allowed = {'ppt/presentation.xml', 'ppt/_rels/presentation.xml.rels',
               '[Content_Types].xml', 'docProps/app.xml'} | edited_notes
    changed = {name for name in parts if parts[name] != before[name]}
    if not changed <= allowed:
        raise AssertionError(changed - allowed)
    for slide in retained:
        if parts[slide['part']] != before[slide['part']]:
            raise AssertionError(f'Retained native slide changed: {slide["part"]}')
    for name, content in parts.items():
        if name.startswith(('ppt/slides/', 'ppt/notesSlides/')) and name.endswith('.xml'):
            text = '\n'.join(node.text or '' for node in ET.fromstring(content).findall('.//a:t', NS))
            if re.search(r'(?:04-|DP|\()43\b|공유 Omni', text):
                raise AssertionError(f'Withdrawn comparison reference remains: {name}')
        if name.endswith('.rels'):
            origin = name.replace('/_rels/', '/').removesuffix('.rels') if name != '_rels/.rels' else ''
            for rel in ET.fromstring(content):
                if rel.get('TargetMode') != 'External':
                    target = target_part(origin, rel.get('Target'))
                    if target not in parts:
                        raise AssertionError(f'Dangling relationship: {name} -> {target}')

    destination.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(source) as original, zipfile.ZipFile(destination, 'w') as output:
        for info in original.infolist():
            if info.filename in parts:
                output.writestr(info, parts[info.filename])
    report = dict(
        source=str(source), source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
        source_slides=16, output_slides=14, removed_scenes=[authored_slug(before, s) for s in withdrawn],
        removed_parts=sorted(dropped_parts), changed_parts=sorted(changed),
        retained_slide_xml_byte_identical=True, comparison_is_content_subset=subset,
        qa_snapshot='ASR-QA-v5 thought experiment preserved; current P21 values require separate recalculation',
        output_sha256=hashlib.sha256(destination.read_bytes()).hexdigest(),
    )
    report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print('PASS: 14 retained native slides; DP43 slides/notes/section references removed.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('comparison_reference', type=Path)
    parser.add_argument('destination', type=Path)
    parser.add_argument('report', type=Path)
    args = parser.parse_args()
    build(args.source, args.comparison_reference, args.destination, args.report)
