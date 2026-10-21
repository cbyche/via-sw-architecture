"""Append DP5 to the exact user-specified v4, preserving all 17 original bodies."""
from pathlib import Path
import copy
import json
import sys
import xml.etree.ElementTree as E
import full_presentation_package as F
import presentation_windows_policy as W


def main(repo, build):
    base = repo / 'docs/presentations_files/dp1-dp4-reference-style-20261019/VIA-DP1-DP4-reference-style-17slides-v4-windows.pptx'
    original = F.read(base)
    order = F.ordered(original)
    assert len(order) == 17
    authored = F.read(build / 'dp5-authored.pptx')
    authored = {k: W.normalize_xml(k, v) if k.endswith('.xml') else v for k, v in authored.items()}
    new_order = [p for _, p in F.ordered(authored)]
    assert len(new_order) == 4
    parts = dict(original)
    importer = F.Importer(parts)
    fixed = [f'ppt/slides/slide{218+i}.xml' for i in range(4)]
    assert not any(p in parts for p in fixed)
    importer.import_slides(authored, new_order, 'dp5v5', fixed)
    preserved = []
    for page, (_, part) in enumerate(order, 1):
        root = E.fromstring(parts[part])
        before = copy.deepcopy(root)
        candidates = []
        for shape in root.findall('.//p:sp', F.NS):
            spans = shape.findall('.//a:t', F.NS)
            if ''.join(s.text or '' for s in spans) == f'{page} / 17':
                candidates.append(shape)
        assert len(candidates) == 1, (page, len(candidates))
        spans = candidates[0].findall('.//a:t', F.NS)
        assert len(spans) == 1
        spans[0].text = f'{page} / 21'
        parts[part] = F.xml(root)
        spans[0].text = f'{page} / 17'
        assert F.xml(root) == F.xml(before), page
        note = F.related(original, part, 'notesSlide')
        assert original[note] == parts[note], page
        assert original[F.rels_path(part)] == parts[F.rels_path(part)]
        preserved.append(page)
    for page, part in enumerate(fixed, 18):
        root = E.fromstring(parts[part])
        candidates = []
        for shape in root.findall('.//p:sp', F.NS):
            spans = shape.findall('.//a:t', F.NS)
            off = shape.find('p:spPr/a:xfrm/a:off', F.NS)
            if ''.join(s.text or '' for s in spans) == str(page) and off is not None and int(off.get('y')) > 9900000:
                candidates.append(shape)
        assert len(candidates) == 1, (page, len(candidates))
        shape = candidates[0]
        shape.findall('.//a:t', F.NS)[0].text = f'{page} / 21'
        shape.find('p:nvSpPr/p:cNvPr', F.NS).set('name', 'Continuous slide number')
        pos = shape.find('p:spPr/a:xfrm', F.NS)
        pos.find('a:off', F.NS).attrib.update(x=str(1836*9525), y=str(1052*9525))
        pos.find('a:ext', F.NS).attrib.update(cx=str(62*9525), cy=str(21*9525))
        parts[part] = F.xml(root)
    F.add_registry(parts, importer)
    root = E.fromstring(parts['ppt/presentation.xml'])
    rels = E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
    listing = root.find('p:sldIdLst', F.NS)
    max_id = max(int(n.get('id')) for n in listing)
    for i, part in enumerate(fixed, 1):
        rid = f'Dp5V5Append{i}'
        assert not any(r.get('Id') == rid for r in rels)
        E.SubElement(listing, '{'+F.P+'}sldId', {'id': str(max_id+i), '{'+F.R+'}id': rid})
        E.SubElement(rels, '{'+F.PR+'}Relationship', Id=rid, Type=F.R+'/slide', Target='/'+part)
    parts['ppt/presentation.xml'] = F.xml(root)
    parts['ppt/_rels/presentation.xml.rels'] = F.xml(rels)
    app = E.fromstring(parts['docProps/app.xml'])
    for n in app:
        if n.tag.split('}')[-1] in ('Slides', 'Notes'):
            n.text = '21'
    parts['docProps/app.xml'] = F.xml(app)
    # Original visual dependencies, media and all notes remain byte-identical.
    for key, data in original.items():
        if key.startswith(('ppt/slideMasters/', 'ppt/slideLayouts/', 'ppt/theme/', 'ppt/media/', 'ppt/notesSlides/')):
            assert parts[key] == data, key
    assert len(F.ordered(parts)) == 21
    policy = W.policy_report(parts)
    F.write(build / 'candidate.pptx', parts)
    (build / 'preservation.json').write_text(json.dumps({
        'source': str(base), 'source_sha256': F.digest(base.read_bytes()),
        'slides': 21, 'new_dp5_pages': [18, 19, 20, 21],
        'preserved_body_notes_relationships_pages': preserved,
        'allowed_original_change': 'Footer denominator 17 to 21 only; original source file unchanged',
        'original_master_layout_theme_media_notes_bytes_preserved': True,
        'new_model_calls': 0, 'dp5_status': 'DESIGN_HYPOTHESES_UNMEASURED_UNSELECTED',
        'windows_policy': policy
    }, ensure_ascii=False, indent=2)+'\n')
    print('Exact v4 base: preserved all 17 slide bodies and notes; appended four DP5 slides.')


if __name__ == '__main__':
    main(Path(sys.argv[1]), Path(sys.argv[2]))
