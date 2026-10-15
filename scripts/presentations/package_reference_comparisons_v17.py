"""Add two historical necessity slides and four reference DP comparisons to v16."""
from __future__ import annotations
import copy
import json
import posixpath
import sys
import xml.etree.ElementTree as E
from pathlib import Path
import full_presentation_package as F
import presentation_windows_policy as W
from package_problem_intros_v8 import named, patch_node

BASE = Path('docs/presentations_files/VIA-architecture-presentation_v16.pptx')
SPECS = [
    ('necessity10', 'VIA-project-introduction_v10.pptx', [2], 'v6-header-page', ['necessity-v10-retained-v17']),
    ('necessity9', 'VIA-project-introduction_v9.pptx', [2], 'v6-header-page', ['necessity-v9-retained-v17']),
    ('reference', 'VIA-DP-comparisons-reference_v1.pptx', [1, 2, 3, 4], 'page',
     ['dp44-reference-comparison-v17', 'dp47-reference-comparison-v17', 'dp45-reference-comparison-v17', 'dp42-reference-comparison-v17']),
]

def field(data, name='v6-header-page'):
    return ''.join(t.text or '' for t in named(data, name).findall('.//a:t', F.NS))

def replace_page(data, value):
    node = named(data, 'v6-header-page')
    spans = node.findall('.//a:t', F.NS)
    assert len(spans) == 1
    spans[0].text = value
    return patch_node(data, 'v6-header-page', node)

def main(repo, build):
    previous = json.loads((repo/'docs/presentations_files/full-presentation/manifest-v16.json').read_text())
    assert F.digest((repo/BASE).read_bytes()) == previous['output_sha256']
    output = build/'candidate.pptx'
    assert not output.exists()
    base = F.read(repo/BASE)
    parts = dict(base)
    old_order = F.ordered(base)
    assert len(old_order) == 43
    next_id = max(int(n) for n, _ in old_order)
    ct = E.fromstring(parts['[Content_Types].xml'])
    inserted = {}
    provenance = []
    source_checks = []
    for tag, filename, pages, marker_name, keys in SPECS:
        source_path = Path('docs/presentations_files')/filename
        original = F.read(repo/source_path)
        source = dict(original)
        edited = F.read(build/f'edited-{tag}.pptx')
        order = F.ordered(source)
        edited_order = F.ordered(edited)
        selected = [order[n-1][1] for n in pages]
        for source_page, part in zip(pages, selected):
            marker = named(edited[edited_order[source_page-1][1]], marker_name)
            marker.find('p:nvSpPr/p:cNvPr', F.NS).set('id', named(original[part], marker_name).find('p:nvSpPr/p:cNvPr', F.NS).get('id'))
            marker = E.fromstring(W.normalize_xml(part, F.xml(marker)))
            source[part] = patch_node(source[part], marker_name, marker)
        graph = F.closure(source, selected)
        mapping = {name: posixpath.join(posixpath.dirname(name), f'v17_{tag}_'+posixpath.basename(name)) for name in graph}
        types = F.content_types(source)
        for old in sorted(graph):
            new = mapping[old]
            assert new not in parts
            data = source[old]
            if new.endswith('.xml'):
                data = W.normalize_xml(new, data)
            parts[new] = data
            E.SubElement(ct, '{'+F.CT+'}Override', PartName='/'+new, ContentType=types(old))
            relations = F.relations(source, old)
            if relations:
                root = E.Element('{'+F.PR+'}Relationships')
                for rel in relations:
                    rel = copy.deepcopy(rel)
                    if rel.get('TargetMode') != 'External':
                        rel.set('Target', '/'+mapping[F.target(old, rel.get('Target'))])
                    root.append(rel)
                parts[F.rels_path(new)] = F.xml(root)
        added = []
        for source_page, old, key in zip(pages, selected, keys):
            new = mapping[old]
            root = E.fromstring(parts[new])
            root.find('p:cSld', F.NS).set('name', key)
            parts[new] = F.xml(root)
            next_id += 1
            title_name = 'title' if tag == 'reference' else 'v6-header-title'
            added.append(dict(key=key, title=field(original[old], title_name), section='본 발표',
                kind='inserted-'+tag, part=new, slide_id=next_id, source_path=str(source_path), source_page=source_page,
                source_sha256=F.digest((repo/source_path).read_bytes()), native_tables=len(root.findall('.//a:tbl', F.NS))))
            source_checks.append((original, old, new, marker_name))
        inserted[tag] = added
        provenance.append(dict(path=str(source_path), sha256=F.digest((repo/source_path).read_bytes()), source_pages=pages))
    parts['[Content_Types].xml'] = F.xml(ct)
    old_rows = copy.deepcopy(previous['slides'])
    for old_page, row in enumerate(old_rows, 1):
        row.update(source_path=str(BASE), source_page=old_page, kind='retained-v16')
    rows = old_rows[:1]+inserted['necessity10']+inserted['necessity9']+old_rows[1:18]+inserted['reference']+old_rows[18:]
    assert len(rows) == 49
    for page, row in enumerate(rows, 1):
        row['page'] = page
        part = row['part']
        if row['kind'] == 'retained-v16':
            old_page = row['source_page']
            prefix = f'{old_page} / 43'
            value = field(base[part])
            assert value.startswith(prefix)
            parts[part] = replace_page(parts[part], f'{page} / 49'+value[len(prefix):])
        else:
            name = 'page' if row['kind'] == 'inserted-reference' else 'v6-header-page'
            assert field(parts[part], name) == f'{page} / 49'
        row['slide_sha256'] = F.digest(parts[part])
        row['notes_sha256'] = F.digest(parts[F.related(parts, part, 'notesSlide')])
    root = E.fromstring(parts['ppt/presentation.xml'])
    relations = E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
    for rel in list(relations):
        if rel.get('Type').endswith('/slide'):
            relations.remove(rel)
    listing = root.find('p:sldIdLst', F.NS)
    listing.clear()
    for row in rows:
        rid = 'V17Slide'+str(row['page'])
        E.SubElement(listing, '{'+F.P+'}sldId', {'id':str(row['slide_id']), '{'+F.R+'}id':rid})
        E.SubElement(relations, '{'+F.PR+'}Relationship', Id=rid, Type=F.R+'/slide', Target='/'+row['part'])
    for section in root.findall('.//{'+F.P14+'}section'):
        ids = section.find('{'+F.P14+'}sldIdLst')
        ids.clear()
        for row in (rows[:40] if section.get('name') == '본 발표' else rows[40:]):
            E.SubElement(ids, '{'+F.P14+'}sldId', id=str(row['slide_id']))
    parts['ppt/presentation.xml'] = F.xml(root)
    parts['ppt/_rels/presentation.xml.rels'] = F.xml(relations)
    app = E.fromstring(parts['docProps/app.xml'])
    for node in app:
        if node.tag.split('}')[-1] in ('Slides', 'Notes'):
            node.text = '49'
    parts['docProps/app.xml'] = F.xml(app)
    for _, part in old_order:
        restored = replace_page(parts[part], field(base[part]))
        assert F.xml(E.fromstring(restored)) == F.xml(E.fromstring(base[part])), part
        note = F.related(base, part, 'notesSlide')
        assert parts[note] == base[note], note
    for original, old, new, marker in source_checks:
        original_root = E.fromstring(original[old])
        root = E.fromstring(parts[new])
        root.find('p:cSld', F.NS).attrib = original_root.find('p:cSld', F.NS).attrib
        restored = patch_node(F.xml(root), marker, named(original[old], marker))
        assert F.xml(E.fromstring(restored)) == F.xml(original_root), old
        assert parts[F.related(parts, new, 'notesSlide')] == original[F.related(original, old, 'notesSlide')]
    for name in base:
        if name.startswith(('ppt/media/', 'ppt/embeddings/', 'ppt/charts/', 'ppt/slideMasters/', 'ppt/slideLayouts/', 'ppt/notesMasters/', 'ppt/theme/')):
            assert parts[name] == base[name], name
    F.verify(parts)
    policy = W.policy_report(parts)
    assert len(F.ordered(parts)) == 49 and not policy['violations']
    F.write(output, parts)
    manifest = dict(version=17, published_date='2026-10-15', source_path=str(BASE), source_sha256=previous['output_sha256'],
        added_sources=provenance, output_path='docs/presentations_files/VIA-architecture-presentation_v17.pptx',
        output_sha256=F.digest(output.read_bytes()), total_slides=49, main_slides=40, appendix_first_page=41, appendix_last_page=49,
        restored_necessity_pages=[2,3], dp_summary_page=20, reference_comparison_pages=[21,22,23,24],
        existing_content_and_notes_preserved=True, source_content_and_notes_preserved=True,
        preserved_existing_media_and_layouts=True, authoring_policy=policy, slides=rows)
    (build/'manifest-v17.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2)+'\n')
    print('PASS: v17, 49 slides; necessity 2-3; reference comparisons 21-24; all content preserved')

if __name__ == '__main__':
    main(Path(sys.argv[1]), Path(sys.argv[2]))
