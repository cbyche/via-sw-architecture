"""Package the scoped Artifact Tool edit without re-exporting preserved assets.

Six native intros and five metadata slides change. Every other slide body,
speaker note, original media, layout, master and theme is preserved. Page fields
and the two moved slides' stage navigation are the only global slide edits.
"""
from pathlib import Path
import copy
import html
import json
import re
import sys
import uuid
import xml.etree.ElementTree as E
import full_presentation_package as F
import presentation_windows_policy as W


def tree(data):
    return E.fromstring(data).find('p:cSld/p:spTree', F.NS)


def named(data, name):
    hits = [n for n in tree(data) if (n.find('.//p:cNvPr', F.NS) is not None
            and n.find('.//p:cNvPr', F.NS).get('name') == name)]
    assert len(hits) == 1, (name, len(hits))
    return hits[0]


def patch_node(data, name, node=None):
    text = data.decode('utf-8-sig')
    found = 0
    def edit(m):
        nonlocal found
        prop = re.search(r'<p:cNvPr\b[^>]*\bname="([^"]*)"', m[0])
        if prop is None or html.unescape(prop[1]) != name:
            return m[0]
        found += 1
        return '' if node is None else E.tostring(node, encoding='unicode')
    # These scoped objects are direct native shapes or graphic frames, not groups.
    text = re.sub(r'<p:(sp|graphicFrame|cxnSp)\b[^>]*>.*?</p:\1>', edit, text, flags=re.S)
    assert found == 1, (name, found)
    return text.encode()


def normalized_node(node):
    root = E.Element('{'+F.P+'}sld')
    root.append(copy.deepcopy(node))
    return E.fromstring(W.normalize_xml('scoped.xml', F.xml(root)))[0]


def notes_body(data):
    root = E.fromstring(data)
    return next(n for n in root.findall('.//p:sp', F.NS)
                if n.find('p:nvSpPr/p:nvPr/p:ph', F.NS) is not None
                and n.find('p:nvSpPr/p:nvPr/p:ph', F.NS).get('type') == 'body')


def revise_notes(template, authored):
    old, new = notes_body(template), notes_body(authored)
    root = E.fromstring(template)
    dest = notes_body(template)
    # Locate the body on the actual root and preserve the note scaffolding.
    dest = next(n for n in root.findall('.//p:sp', F.NS)
                if n.find('p:nvSpPr/p:nvPr/p:ph', F.NS) is not None
                and n.find('p:nvSpPr/p:nvPr/p:ph', F.NS).get('type') == 'body')
    dest.remove(dest.find('p:txBody', F.NS))
    dest.append(copy.deepcopy(new.find('p:txBody', F.NS)))
    return W.normalize_xml('notes.xml', F.xml(root))


def body_signature(data):
    root = E.fromstring(data)
    st = root.find('p:cSld/p:spTree', F.NS)
    for n in list(st):
        prop = n.find('.//p:cNvPr', F.NS)
        if prop is not None and (prop.get('name') == 'v6-header-page'
                or prop.get('name', '').startswith('v6-header-stage-')):
            st.remove(n)
    return E.tostring(root)


def build(repo, build_dir):
    repo, b = Path(repo), Path(build_dir)
    src_path = repo/'docs/presentations_files/VIA-architecture-presentation_v7.pptx'
    src, native, meta = map(F.read, [src_path, b/'intros-authored.pptx', b/'metadata-authored.pptx'])
    plan = json.loads((b/'intro-plan.json').read_text())
    previous = json.loads((repo/'docs/presentations_files/full-presentation/manifest-v7.json').read_text())
    assert F.digest(src_path.read_bytes()) == previous['output_sha256'] == plan['source_sha256']
    order, native_order, meta_order = F.ordered(src), F.ordered(native), F.ordered(meta)
    assert len(order) == 61 and len(native_order) == 6
    parts = dict(src)
    for edit in plan['edits']:
        part = order[edit['page']-1][1]
        old = named(parts[part], edit['name'])
        new = named(meta[meta_order[edit['page']-1][1]], edit['name'])
        assert ''.join(n.text or '' for n in old.findall('.//a:t', F.NS)) == edit['oldText'].replace('\n', '')
        assert ''.join(n.text or '' for n in new.findall('.//a:t', F.NS)) == edit['newText'].replace('\n', '')
        result = copy.deepcopy(old)
        result.remove(result.find('p:txBody', F.NS))
        result.append(copy.deepcopy(new.find('p:txBody', F.NS)))
        parts[part] = patch_node(parts[part], edit['name'], normalized_node(result))
    for edit in plan['tableEdits']:
        part = order[edit['page']-1][1]
        tables = [n for n in tree(parts[part]) if n.find('.//a:tbl', F.NS) is not None]
        assert len(tables) == 1
        old = tables[0]
        table_name = old.find('.//p:cNvPr', F.NS).get('name')
        new_tables = [n for n in tree(meta[meta_order[edit['page']-1][1]]) if n.find('.//a:tbl', F.NS) is not None]
        assert len(new_tables) == 1
        new = new_tables[0]
        old_cell = old.findall('.//a:tr', F.NS)[edit['row']].findall('a:tc', F.NS)[edit['column']]
        new_cell = new.findall('.//a:tr', F.NS)[edit['row']].findall('a:tc', F.NS)[edit['column']]
        assert ''.join(n.text or '' for n in new_cell.findall('.//a:t', F.NS)) == edit['newText']
        old_cell.remove(old_cell.find('a:txBody', F.NS))
        old_cell.insert(0, copy.deepcopy(new_cell.find('a:txBody', F.NS)))
        parts[part] = patch_node(parts[part], table_name, normalized_node(old))
    for name in plan['removeOverviewNames']:
        parts[order[12][1]] = patch_node(parts[order[12][1]], name)
    for page in plan['metadataPages']:
        part, authored_part = order[page-1][1], meta_order[page-1][1]
        note = F.related(src, part, 'notesSlide')
        parts[note] = revise_notes(src[note], meta[F.related(meta, authored_part, 'notesSlide')])

    old_rows = copy.deepcopy(previous['slides'])
    for row in old_rows:
        row['source_page'] = row['page']
        row['source_key'] = row['key']
        row['kind'] = 'preserved-body'
    # Original pages 15/16 are moved to the appendix; all other order changes
    # merely follow the added basic-structures and unnumbered problem pages.
    source_pages = list(range(1, 15)) + [17, 18, 19, None, 20, 21, 22, None] + list(range(23, 62)) + [15, 16]
    assert len(source_pages) == 63
    rows = []
    max_part = max(int(re.search(r'slide(\d+)\.xml$', part)[1]) for _, part in order)
    max_id = max(int(i) for i, _ in order)
    new_index = 0
    for page, source_page in enumerate(source_pages, 1):
        if source_page:
            row = copy.deepcopy(old_rows[source_page-1])
        else:
            new_index += 1
            row = {'part':f'ppt/slides/slide{max_part+new_index}.xml', 'slide_id':max_id+new_index,
                   'source_page':None, 'source_key':None, 'section':'설계 비교', 'kind':'authored-problem-intro'}
        row['page'] = page
        rows.append(row)
    configs = [
        (14, 1, 'via-common-request-background-v8'),
        (15, 2, 'dp44-background-v3'),
        (17, 3, 'dp45-background-v3'),
        (18, 4, 'dp45-basic-v8'),
        (20, 5, 'dp42-background-v3'),
        (22, 6, 'screen-grounding-background-v8'),
    ]
    source_header = order[16][1]
    headers = [copy.deepcopy(named(src[source_header], 'v6-header-stage-'+str(n))) for n in range(5)]
    headers.append(copy.deepcopy(named(src[source_header], 'v6-header-page')))
    inactive = headers[0].find('p:spPr/a:solidFill/a:srgbClr', F.NS).get('val')
    active = headers[2].find('p:spPr/a:solidFill/a:srgbClr', F.NS).get('val')
    new_types = {}
    for page, authored_page, key in configs:
        row = rows[page-1]
        part, new_part = row['part'], native_order[authored_page-1][1]
        root = E.fromstring(native[new_part]);st = root.find('p:cSld/p:spTree', F.NS)
        for header in headers:
            st.append(copy.deepcopy(header))
        for n, props in enumerate(st.findall('.//p:cNvPr', F.NS), 2):
            props.set('id', str(n))
        root.find('p:cSld', F.NS).set('name', key)
        parts[part] = W.normalize_xml(part, F.xml(root))
        authored_note = F.related(native, new_part, 'notesSlide')
        if row['source_page']:
            note = F.related(src, part, 'notesSlide')
            parts[note] = revise_notes(src[note], native[authored_note])
            # Retain the original layout and notes relations, no new masters.
        else:
            note = f'ppt/notesSlides/notesSlideV8{page}.xml'
            template_note = F.related(src, source_header, 'notesSlide')
            parts[note] = revise_notes(src[template_note], native[authored_note])
            rs = E.Element('{'+F.PR+'}Relationships')
            E.SubElement(rs, '{'+F.PR+'}Relationship', Id='rId1', Type=F.R+'/slideLayout', Target='/'+F.related(src, source_header, 'slideLayout'))
            E.SubElement(rs, '{'+F.PR+'}Relationship', Id='rId2', Type=F.R+'/notesSlide', Target='/'+note)
            parts[F.rels_path(part)] = F.xml(rs)
            note_rs = E.Element('{'+F.PR+'}Relationships')
            E.SubElement(note_rs, '{'+F.PR+'}Relationship', Id='rId1', Type=F.R+'/slide', Target='/'+part)
            E.SubElement(note_rs, '{'+F.PR+'}Relationship', Id='rId2', Type=F.R+'/notesMaster', Target='/'+F.related(src, template_note, 'notesMaster'))
            parts[F.rels_path(note)] = F.xml(note_rs)
            new_types[part] = 'application/vnd.openxmlformats-officedocument.presentationml.slide+xml'
            new_types[note] = 'application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml'
        row.update(key=key, title=plan['scenes'][authored_page-1]['title'], kind='authored-problem-intro', section='설계 비교')
    for row in rows:
        part = row['part']
        header = named(parts[part], 'v6-header-page')
        ts = header.findall('.//a:t', F.NS)
        assert len(ts) == 1
        ts[0].text = f'{row["page"]} / 63'
        parts[part] = patch_node(parts[part], 'v6-header-page', header)
        if row['page'] in (62, 63):
            row['section'] = 'Appendix / 제외 주제 41 보존'
            row['kind'] = 'moved-dp41-body-preserved'
            for i in range(5):
                stage = named(parts[part], 'v6-header-stage-'+str(i))
                fill = stage.find('p:spPr/a:solidFill/a:srgbClr', F.NS)
                assert fill is not None
                fill.set('val', active if i == 4 else inactive)
                parts[part] = patch_node(parts[part], 'v6-header-stage-'+str(i), stage)
        row['native_tables'] = len(E.fromstring(parts[part]).findall('.//a:tbl', F.NS))
        row['slide_sha256'] = F.digest(parts[part])
        row['notes_sha256'] = F.digest(parts[F.related(parts, part, 'notesSlide')])
    root = E.fromstring(parts['ppt/presentation.xml'])
    rs = E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
    for rel in list(rs):
        if rel.get('Type').endswith('/slide'):
            rs.remove(rel)
    lst = root.find('p:sldIdLst', F.NS);lst.clear()
    for row in rows:
        rid = 'V8Slide'+str(row['page'])
        E.SubElement(lst, '{'+F.P+'}sldId', {'id':str(row['slide_id']), '{'+F.R+'}id':rid})
        E.SubElement(rs, '{'+F.PR+'}Relationship', Id=rid, Type=F.R+'/slide', Target='/'+row['part'])
    # Rebuild section ordering while preserving registered masters/defaults.
    section_list = root.find('.//{'+F.P14+'}sectionLst')
    assert section_list is not None
    section_list.clear()
    for name in dict.fromkeys(r['section'] for r in rows):
        section = E.SubElement(section_list, '{'+F.P14+'}section', name=name,
                               id='{'+str(uuid.uuid5(uuid.NAMESPACE_URL,'via-v8:'+name)).upper()+'}')
        ids = E.SubElement(section, '{'+F.P14+'}sldIdLst')
        for row in rows:
            if row['section'] == name:
                E.SubElement(ids, '{'+F.P14+'}sldId', id=str(row['slide_id']))
    parts['ppt/presentation.xml'], parts['ppt/_rels/presentation.xml.rels'] = F.xml(root), F.xml(rs)
    ct = E.fromstring(parts['[Content_Types].xml'])
    for part, kind in new_types.items():
        E.SubElement(ct, '{'+F.CT+'}Override', PartName='/'+part, ContentType=kind)
    parts['[Content_Types].xml'] = F.xml(ct)
    app = E.fromstring(parts['docProps/app.xml'])
    for node in app:
        if node.tag.split('}')[-1] in ('Slides', 'Notes'):
            node.text = '63'
    parts['docProps/app.xml'] = F.xml(app)
    changed_source_pages = {12,13,14,17,19,21,23,24,25}
    preserved = []
    for page, (_, part) in enumerate(order, 1):
        if page not in changed_source_pages:
            assert body_signature(parts[part]) == body_signature(src[part]), page
            note = F.related(src, part, 'notesSlide')
            assert parts[note] == src[note], page
            preserved.append(page)
    media = [n for n in src if n.startswith('ppt/media/')]
    assets = [n for n in src if n.startswith(('ppt/slideMasters/','ppt/slideLayouts/','ppt/notesMasters/','ppt/theme/'))]
    for name in media+assets:
        assert src[name] == parts[name], name
    report = W.policy_report(parts)
    candidate = b/'candidate-v8.pptx'
    F.write(candidate, parts)
    manifest = {'version':8, 'published_date':'2026-10-13',
        'source_path':str(src_path.relative_to(repo)), 'source_sha256':plan['source_sha256'],
        'output_path':'docs/presentations_files/VIA-architecture-presentation_v8.pptx', 'output_sha256':F.digest(candidate.read_bytes()),
        'main_slides':24,'appendix_divider_page':25,'total_slides':63,
        'revised_main_pages':[12,13,14,15,17,18,20,22,23,24], 'moved_dp41_pages':[62,63],
        'added_pages':[18,22], 'changed_source_pages':sorted(changed_source_pages),
        'preserved_source_bodies_and_notes':preserved, 'all_original_media_byte_identical':True,
        'original_media_parts':len(media), 'masters_layouts_themes_byte_identical':True,
        'page_fields_updated':'1 / 63 through 63 / 63', 'fixed_qa_definitions_and_targets_preserved':True,
        'existing_44_45_42_comparison_bodies_preserved':True,'authoring_policy':report,
        'slides':rows}
    (b/'manifest-v8.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2)+'\n')
    print(json.dumps({'slides':63, 'preservedBodiesAndNotes':len(preserved), 'policyViolations':report['violations']}))


if __name__ == '__main__':
    build(*sys.argv[1:])
