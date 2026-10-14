"""Append the detailed DP44 runtime comparison after the retained DP44 page."""
from __future__ import annotations

import copy
import json
import posixpath
import re
import sys
import xml.etree.ElementTree as E
from pathlib import Path

import full_presentation_package as F
import presentation_windows_policy as W
from package_problem_intros_v8 import named, patch_node


BASE = Path('docs/presentations_files/VIA-architecture-presentation_v13.pptx')
SOURCE = Path('docs/presentations_files/dp-comparison/VIA-DP44-implementation-comparison.pptx')
OUTPUT = Path('docs/presentations_files/VIA-architecture-presentation_v14.pptx')
MANIFEST = Path('docs/presentations_files/full-presentation/manifest-v14.json')


def field(data):
    return ''.join(t.text or '' for t in named(data, 'v6-header-page').findall('.//a:t', F.NS))


def replace_page(data, text):
    node = named(data, 'v6-header-page')
    spans = node.findall('.//a:t', F.NS)
    assert len(spans) == 1
    spans[0].text = text
    return patch_node(data, 'v6-header-page', node)


def main(repo):
    previous = json.loads((repo/'docs/presentations_files/full-presentation/manifest-v13.json').read_text())
    assert F.digest((repo/BASE).read_bytes()) == previous['output_sha256']
    assert not (repo/OUTPUT).exists()
    base, source = F.read(repo/BASE), F.read(repo/SOURCE)
    parts = dict(base)
    old_order, source_order = F.ordered(base), F.ordered(source)
    assert len(old_order) == 37 and len(source_order) == 1
    source_slide = source_order[0][1]
    new_slide = 'ppt/slides/dp44ImplementationV14.xml'
    graph = F.closure(source, [source_slide])
    mapping = {n: posixpath.join(posixpath.dirname(n), 'dp44v14_' + posixpath.basename(n)) for n in graph}
    mapping[source_slide] = new_slide
    types = F.content_types(source)
    ct = E.fromstring(parts['[Content_Types].xml'])
    for old in sorted(graph):
        new = mapping[old]
        assert new not in parts
        data = source[old]
        if new.endswith('.xml'):
            data = W.normalize_xml(new, data)
        parts[new] = data
        E.SubElement(ct, '{'+F.CT+'}Override', PartName='/'+new, ContentType=types(old))
        rels = F.relations(source, old)
        if rels:
            root = E.Element('{'+F.PR+'}Relationships')
            for rel in rels:
                rel = copy.deepcopy(rel)
                if rel.get('TargetMode') != 'External':
                    rel.set('Target', '/'+mapping[F.target(old, rel.get('Target'))])
                root.append(rel)
            parts[F.rels_path(new)] = F.xml(root)
    parts['[Content_Types].xml'] = F.xml(ct)

    # The new diagram has its own title and rule. Add the familiar page marker only.
    marker = copy.deepcopy(named(base[old_order[19][1]], 'v6-header-page'))
    marker.find('./p:nvSpPr/p:cNvPr', F.NS).set('id', '10000')
    marker.findall('.//a:t', F.NS)[0].text = '21 / 38'
    root = E.fromstring(parts[new_slide])
    root.find('p:cSld/p:spTree', F.NS).append(marker)
    root.find('p:cSld', F.NS).set('name', 'dp44-implementation-comparison')
    parts[new_slide] = F.xml(root)

    rows = []
    for old_page, row in enumerate(previous['slides'], 1):
        page = old_page if old_page <= 20 else old_page + 1
        row = copy.deepcopy(row)
        row.update(page=page, source_page=old_page, source_path=str(BASE), kind='retained-v13')
        part = row['part']
        before = field(base[part])
        assert before.startswith(f'{old_page} / 37')
        parts[part] = replace_page(parts[part], f'{page} / 38'+before[len(f'{old_page} / 37'):])
        row['slide_sha256'] = F.digest(parts[part])
        rows.append(row)
        if old_page == 20:
            rows.append(dict(page=21, key='dp44-implementation-comparison',
                title='DP44. 중앙 회수와 단계별 완료 처리의 구현 구조', section='본 발표',
                kind='inserted-dp44-implementation', part=new_slide,
                slide_id=max(int(i) for i,_ in old_order)+1,
                source_path=str(SOURCE), source_page=1, source_sha256=F.digest((repo/SOURCE).read_bytes()),
                native_tables=0))
    root = E.fromstring(parts['ppt/presentation.xml'])
    rels = E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
    for rel in list(rels):
        if rel.get('Type').endswith('/slide'):
            rels.remove(rel)
    listing = root.find('p:sldIdLst', F.NS)
    listing.clear()
    for row in rows:
        rid = 'V14Slide'+str(row['page'])
        E.SubElement(listing, '{'+F.P+'}sldId', {'id':str(row['slide_id']), '{'+F.R+'}id':rid})
        E.SubElement(rels, '{'+F.PR+'}Relationship', Id=rid, Type=F.R+'/slide', Target='/'+row['part'])
    for section in root.findall('.//{'+F.P14+'}section'):
        ids = section.find('{'+F.P14+'}sldIdLst')
        ids.clear()
        selected = rows[:29] if section.get('name') == '본 발표' else rows[29:]
        for row in selected:
            E.SubElement(ids, '{'+F.P14+'}sldId', id=str(row['slide_id']))
    parts['ppt/presentation.xml'] = F.xml(root)
    parts['ppt/_rels/presentation.xml.rels'] = F.xml(rels)
    app = E.fromstring(parts['docProps/app.xml'])
    for node in app:
        if node.tag.split('}')[-1] in ('Slides', 'Notes'):
            node.text = '38'
    parts['docProps/app.xml'] = F.xml(app)

    for row in rows:
        part = row['part']
        row['slide_sha256'] = F.digest(parts[part])
        row['notes_sha256'] = F.digest(parts[F.related(parts, part, 'notesSlide')])
    assert len(F.ordered(parts)) == 38
    for n in base:
        if n.startswith(('ppt/media/','ppt/embeddings/','ppt/charts/','ppt/slideMasters/','ppt/slideLayouts/','ppt/notesMasters/','ppt/theme/')):
            assert parts[n] == base[n], n
    F.verify(parts)
    policy = W.policy_report(parts)
    assert not policy['violations'], policy['violations'][:10]
    F.write(repo/OUTPUT, parts)
    manifest = dict(version=14, published_date='2026-10-15', source_path=str(BASE),
        source_sha256=previous['output_sha256'], new_slide_source_path=str(SOURCE),
        new_slide_source_sha256=F.digest((repo/SOURCE).read_bytes()), output_path=str(OUTPUT),
        output_sha256=F.digest((repo/OUTPUT).read_bytes()), total_slides=38,
        main_slides=29, appendix_first_page=30, appendix_last_page=38,
        old_dp44_comparison_page=20, new_dp44_implementation_page=21,
        preserved_existing_media_and_layouts=True, authoring_policy=policy, slides=rows)
    (repo/MANIFEST).write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
    print('PASS: v14, 38 slides; old DP44 page retained at 20, new implementation page at 21')


if __name__ == '__main__':
    main(Path(sys.argv[1]))
