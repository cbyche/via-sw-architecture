"""Reorder a versioned presentation without losing the original native slides.

The 32 source slides and their notes/media remain byte-identical in the appendix.
Main copies can carry presentation-only overlays. Architecture inputs are read-only.
"""
from pathlib import Path
import copy
import json
import re
import sys
import xml.etree.ElementTree as E

import full_presentation_package as F


def build(source_path, authored_path, plan_path, output, report):
    source_path, authored_path, plan_path, output, report = map(Path, (source_path, authored_path, plan_path, output, report))
    original = F.read(source_path)
    authored = F.read(authored_path)
    old = [part for _, part in F.ordered(original)]
    new = [part for _, part in F.ordered(authored)]
    assert len(old) == 32
    plan = json.loads(plan_path.read_text())
    assert F.digest(source_path.read_bytes()) == plan['source_sha256']
    assert sorted(row['original_page'] for row in plan['slides'] if row['kind'] == 'original') == list(range(1, 33))
    parts = dict(original)
    importer = F.Importer(parts)
    clones = [row for row in plan['slides'] if row['kind'] == 'clone']
    selected = [old[row['original_page'] - 1] for row in clones]
    fixed = [f'ppt/slides/slide{33+i}.xml' for i in range(len(clones))]
    mapping, _, clone_media = importer.import_slides(original, selected, 'V2Clone', fixed)
    for row in clones:
        row['part'] = mapping[old[row['original_page'] - 1]]
        root = E.fromstring(parts[row['part']])
        # Navigation in the main deck reflects its actual current sections.
        swaps = {'과제 소개': '배경과 소개', '요구사항 분석': '요구사항', '설계': '설계 비교',
                 '구현 및 검증': '검토 요약', '결론': 'Appendix'}
        for t in root.findall('.//a:t', F.NS):
            if t.text in swaps:
                t.text = swaps[t.text]
        # Only main copies lose obsolete bottom page numbers. The appendix
        # keeps the original slide XML, including its historical navigation.
        tree = root.find('p:cSld/p:spTree', F.NS)
        for child in list(tree):
            off = child.find('.//a:xfrm/a:off', F.NS)
            text = ''.join(node.text or '' for node in child.findall('.//a:t', F.NS)).strip()
            if child.tag.endswith('sp') and text.isdigit() and off is not None:
                if int(off.get('y')) >= 1000 * 9525 and int(off.get('x')) >= 1700 * 9525:
                    tree.remove(child)
        if row.get('overlay'):
            overlay = E.fromstring(authored[new[row['overlay'] - 1]])
            tree = root.find('p:cSld/p:spTree', F.NS)
            # Main copies replace the old bottom explanation; originals remain intact.
            if row['key'].startswith('dp') and '-background-' in row['key']:
                for child in list(tree):
                    off = child.find('.//a:xfrm/a:off', F.NS)
                    if child.tag.endswith('sp') and off is not None and int(off.get('y')) >= 940 * 9525:
                        tree.remove(child)
            for child in overlay.find('p:cSld/p:spTree', F.NS):
                if child.tag.split('}')[-1] in ('nvGrpSpPr', 'grpSpPr'):
                    continue
                assert not any(k.startswith('{'+F.R+'}') for node in child.iter() for k in node.attrib), 'Overlay must be native text/shapes'
                tree.append(copy.deepcopy(child))
        # Numeric ids are unique after overlaying editable native shapes.
        for i, node in enumerate(root.findall('.//p:cNvPr', F.NS), 1):
            node.set('id', str(i))
        root.find('p:cSld', F.NS).set('name', row['key'])
        parts[row['part']] = F.xml(root)
    native = [row for row in plan['slides'] if row['kind'] == 'authored']
    selected = [new[row['authored_page'] - 1] for row in native]
    fixed = [f'ppt/slides/slide{33+len(clones)+i}.xml' for i in range(len(native))]
    mapping, _, native_media = importer.import_slides(authored, selected, 'V2Native', fixed)
    for row in native:
        row['part'] = mapping[new[row['authored_page'] - 1]]
        parts[row['part']] = F.name_slide(parts[row['part']], row['key'])
    for row in plan['slides']:
        if row['kind'] == 'original':
            row['part'] = old[row['original_page'] - 1]
            row['key'] = E.fromstring(original[row['part']]).find('p:cSld', F.NS).get('name')
    # Keep all original masters/notes masters and register only new imported ones.
    F.add_registry(parts, importer)
    root = E.fromstring(parts['ppt/presentation.xml'])
    rels = E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
    for rel in list(rels):
        if rel.get('Type').endswith('/slide'):
            rels.remove(rel)
    slides = root.find('p:sldIdLst', F.NS)
    slides.clear()
    for i, row in enumerate(plan['slides'], 1):
        row['page'] = i
        row['slide_id'] = 255 + i
        rid = f'V2Slide{i}'
        E.SubElement(slides, '{'+F.P+'}sldId', {'id': str(row['slide_id']), '{'+F.R+'}id': rid})
        E.SubElement(rels, '{'+F.PR+'}Relationship', {'Id': rid, 'Type': F.R+'/slide', 'Target': '/'+row['part']})
        row['native_tables'] = len(E.fromstring(parts[row['part']]).findall('.//a:tbl', F.NS))
    for ext in list(root):
        if ext.tag.split('}')[-1] == 'extLst':
            root.remove(ext)
    extlst = E.SubElement(root, '{'+F.P+'}extLst')
    ext = E.SubElement(extlst, '{'+F.P+'}ext', {'uri':'{521415D9-36F7-43E2-AB2F-B90AF26B5E84}'})
    sections = E.SubElement(ext, '{'+F.P14+'}sectionLst')
    import uuid
    for section_name in dict.fromkeys(row['section'] for row in plan['slides']):
        section = E.SubElement(sections, '{'+F.P14+'}section', {'name': section_name, 'id':'{'+str(uuid.uuid5(uuid.NAMESPACE_URL, 'via-v2:'+section_name)).upper()+'}'})
        ids = E.SubElement(section, '{'+F.P14+'}sldIdLst')
        for row in plan['slides']:
            if row['section'] == section_name:
                E.SubElement(ids, '{'+F.P14+'}sldId', {'id': str(row['slide_id'])})
    parts['ppt/presentation.xml'] = F.xml(root)
    parts['ppt/_rels/presentation.xml.rels'] = F.xml(rels)
    app = E.fromstring(parts['docProps/app.xml'])
    for node in app:
        if node.tag.split('}')[-1] in ('Slides', 'Notes'):
            node.text = str(len(plan['slides']))
    parts['docProps/app.xml'] = F.xml(app)
    # Reuse byte-identical original media in clones instead of doubling deck size.
    originals = {F.digest(data): name for name, data in original.items() if name.startswith('ppt/media/')}
    dedup = {name: originals[F.digest(data)] for name, data in parts.items()
             if name.startswith('ppt/media/') and name not in original and F.digest(data) in originals}
    for name in list(parts):
        if not name.endswith('.rels'):
            continue
        relroot = E.fromstring(parts[name])
        owner = name.replace('/_rels/', '/').removesuffix('.rels') if name != '_rels/.rels' else ''
        changed = False
        for rel in relroot:
            if rel.get('TargetMode') != 'External':
                referenced = F.target(owner, rel.get('Target'))
                if referenced in dedup:
                    rel.set('Target', '/' + dedup[referenced])
                    changed = True
        if changed:
            parts[name] = F.xml(relroot)
    for name in dedup:
        del parts[name]
    types = E.fromstring(parts['[Content_Types].xml'])
    for item in list(types):
        if item.get('PartName', '').lstrip('/') in dedup:
            types.remove(item)
    parts['[Content_Types].xml'] = F.xml(types)
    for row in clone_media + native_media:
        if row['imported'] in dedup:
            row['reused_original'] = dedup[row['imported']]
            row['imported'] = dedup[row['imported']]
    assert len(plan['slides']) == 57
    # The unedited appendix is the lossless preservation contract.
    preserved = []
    for i, part in enumerate(old, 1):
        assert parts[part] == original[part]
        assert parts[F.rels_path(part)] == original[F.rels_path(part)]
        note = F.related(original, part, 'notesSlide')
        assert parts[note] == original[note]
        assert parts[F.rels_path(note)] == original[F.rels_path(note)]
        preserved.append({'original_page': i, 'page': next(row['page'] for row in plan['slides'] if row['kind']=='original' and row['original_page']==i),
                          'slide_sha256': F.digest(original[part]), 'notes_sha256': F.digest(original[note])})
    for name, data in original.items():
        if name.startswith('ppt/media/'):
            assert parts[name] == data
    F.write(output, parts)
    plan.update({'output_sha256': F.digest(output.read_bytes()), 'preserved_original_slides': preserved,
                 'original_media_count': sum(n.startswith('ppt/media/') for n in original),
                 'source_preserved_byte_identical': True, 'imported_media': clone_media+native_media})
    report.write_text(json.dumps(plan, ensure_ascii=False, indent=2)+'\n')
    print(json.dumps({'slides':len(plan['slides']), 'original_slides_preserved':32, 'output':str(output)}))


if __name__ == '__main__':
    build(*sys.argv[1:])
