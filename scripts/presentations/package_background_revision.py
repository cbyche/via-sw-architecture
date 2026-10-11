"""Replace four backgrounds and move the unchanged originals to the appendix.

Every v2 slide/notes/media part remains byte-identical in the v3 package. Only
the four main positions point to newly authored slides; all other ids/order stay.
"""
from pathlib import Path
import copy
import json
import sys
import uuid
import xml.etree.ElementTree as E

import full_presentation_package as F


def build(source_path, authored_path, previous_manifest, plan_path, output, report):
    source_path, authored_path, previous_manifest, plan_path, output, report = map(
        Path, (source_path, authored_path, previous_manifest, plan_path, output, report))
    source, authored = F.read(source_path), F.read(authored_path)
    previous, plan = json.loads(previous_manifest.read_text()), json.loads(plan_path.read_text())
    assert F.digest(source_path.read_bytes()) == previous['output_sha256'] == plan['source_sha256']
    order = F.ordered(source)
    assert len(order) == len(previous['slides']) == 57
    assert {r['source_page'] for r in plan['changes']} == {15, 17, 19, 21}
    rows = copy.deepcopy(previous['slides'])
    for r, (ident, part) in zip(rows, order):
        assert r['slide_id'] == int(ident) and r['part'] == part
        r['source_page'] = r['page']
        r['source_key'] = r['key']
        r['origin_kind'] = r['kind']
        r['kind'] = 'preserved'
    parts = dict(source)
    importer = F.Importer(parts)
    new_order = F.ordered(authored)
    max_part = max(int(p.rsplit('slide', 1)[1].split('.')[0]) for _, p in order)
    selected = [new_order[c['authored_page'] - 1][1] for c in plan['changes']]
    fixed = [f'ppt/slides/slide{max_part+i}.xml' for i in range(1, 5)]
    mapping, _, media = importer.import_slides(authored, selected, 'V3Background', fixed)
    max_id = max(int(ident) for ident, _ in order)
    for index, change in enumerate(plan['changes'], 1):
        old = rows[change['source_page'] - 1]
        saved = copy.deepcopy(old)
        saved.update(page=change['preserved_page'], section='Appendix / v2 DP 배경 보존', kind='preserved-v2-background')
        rows.append(saved)
        part = mapping[new_order[change['authored_page'] - 1][1]]
        parts[part] = F.name_slide(parts[part], change['key'])
        rows[change['source_page'] - 1] = {
            'page':change['source_page'], 'key':change['key'], 'section':old['section'],
            'kind':'authored', 'part':part, 'slide_id':max_id+index,
            'source_page':change['source_page'], 'source_key':old['key'],
            'preserved_source_page':change['preserved_page'], 'native_tables':len(E.fromstring(parts[part]).findall('.//a:tbl', F.NS))}
    assert [r['page'] for r in rows] == list(range(1, 62))
    assert len({r['key'] for r in rows}) == len({r['slide_id'] for r in rows}) == 61
    F.add_registry(parts, importer)
    root, rels = E.fromstring(parts['ppt/presentation.xml']), E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
    for rel in list(rels):
        if rel.get('Type').endswith('/slide'):
            rels.remove(rel)
    lst = root.find('p:sldIdLst', F.NS)
    lst.clear()
    for row in rows:
        rid = f'BackgroundRevisionSlide{row["page"]}'
        E.SubElement(lst, '{'+F.P+'}sldId', {'id':str(row['slide_id']), '{'+F.R+'}id':rid})
        E.SubElement(rels, '{'+F.PR+'}Relationship', {'Id':rid, 'Type':F.R+'/slide', 'Target':'/'+row['part']})
    for child in list(root):
        if child.tag.split('}')[-1] == 'extLst':
            root.remove(child)
    extlst = E.SubElement(root, '{'+F.P+'}extLst')
    ext = E.SubElement(extlst, '{'+F.P+'}ext', {'uri':'{521415D9-36F7-43E2-AB2F-B90AF26B5E84}'})
    sections = E.SubElement(ext, '{'+F.P14+'}sectionLst')
    for section_name in dict.fromkeys(r['section'] for r in rows):
        section = E.SubElement(sections, '{'+F.P14+'}section', {'name':section_name, 'id':'{'+str(uuid.uuid5(uuid.NAMESPACE_URL,'via-v3:'+section_name)).upper()+'}'})
        ids = E.SubElement(section, '{'+F.P14+'}sldIdLst')
        for row in rows:
            if row['section'] == section_name:
                E.SubElement(ids, '{'+F.P14+'}sldId', {'id':str(row['slide_id'])})
    parts['ppt/presentation.xml'], parts['ppt/_rels/presentation.xml.rels'] = F.xml(root), F.xml(rels)
    app = E.fromstring(parts['docProps/app.xml'])
    for node in app:
        if node.tag.split('}')[-1] in ('Slides', 'Notes'):
            node.text = '61'
    parts['docProps/app.xml'] = F.xml(app)
    preserved = []
    for page, (ident, part) in enumerate(order, 1):
        assert parts[part] == source[part]
        assert parts[F.rels_path(part)] == source[F.rels_path(part)]
        note = F.related(source, part, 'notesSlide')
        assert parts[note] == source[note] and parts[F.rels_path(note)] == source[F.rels_path(note)]
        dest = next(r['page'] for r in rows if r['part'] == part)
        preserved.append({'source_page':page,'page':dest,'slide_sha256':F.digest(source[part]),'notes_sha256':F.digest(source[note])})
    for name, data in source.items():
        if name.startswith('ppt/media/'):
            assert parts[name] == data
    F.write(output, parts)
    manifest = {'version':3, 'source_path':'docs/presentations_files/VIA-architecture-presentation_v2.pptx',
        'source_sha256':plan['source_sha256'], 'output_path':'docs/presentations_files/VIA-architecture-presentation_v3.pptx',
        'output_sha256':F.digest(output.read_bytes()), 'main_slides':24, 'appendix_divider_page':25,
        'revised_main_pages':[15,17,19,21], 'preserved_v2_background_pages':[58,59,60,61],
        'slides':rows, 'preserved_source_slides':preserved,
        'source_slides_notes_media_byte_identical':True, 'imported_media':media}
    report.write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'slides':61,'preserved_source_slides':57,'moved_original_backgrounds':4}))


if __name__ == '__main__':
    build(*sys.argv[1:])
