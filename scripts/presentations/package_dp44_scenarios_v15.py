"""Insert four focused DP44 scenes before the retained combined diagram."""
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

BASE=Path('docs/presentations_files/VIA-architecture-presentation_v14.pptx')
SOURCE=Path('docs/presentations_files/dp-comparison/VIA-DP44-scenario-comparisons_v1.pptx')
OUTPUT=Path('docs/presentations_files/VIA-architecture-presentation_v15.pptx')
MANIFEST=Path('docs/presentations_files/full-presentation/manifest-v15.json')
NAMES=['input','agent','onset','correction']
TITLES=['일반 사용자 발화','Agent 질문','새 발화 시작과 hold','정정 채택과 늦은 완료']


def field(data):
    return ''.join(t.text or '' for t in named(data,'v6-header-page').findall('.//a:t',F.NS))


def replace_page(data,value):
    node=named(data,'v6-header-page')
    spans=node.findall('.//a:t',F.NS)
    assert len(spans)==1
    spans[0].text=value
    return patch_node(data,'v6-header-page',node)


def main(repo):
    prev=json.loads((repo/'docs/presentations_files/full-presentation/manifest-v14.json').read_text())
    assert F.digest((repo/BASE).read_bytes())==prev['output_sha256']
    assert not (repo/OUTPUT).exists()
    base,source=F.read(repo/BASE),F.read(repo/SOURCE)
    parts=dict(base)
    old_order,source_order=F.ordered(base),F.ordered(source)
    assert len(old_order)==38 and len(source_order)==4
    slides=[p for _,p in source_order]
    graph=F.closure(source,slides)
    mapping={n:posixpath.join(posixpath.dirname(n),'dp44scenario_v15_'+posixpath.basename(n)) for n in graph}
    for i,part in enumerate(slides,1):
        mapping[part]=f'ppt/slides/dp44ScenarioV15_{i}.xml'
    types=F.content_types(source)
    ct=E.fromstring(parts['[Content_Types].xml'])
    for old in sorted(graph):
        new=mapping[old]
        assert new not in parts
        data=source[old]
        if new.endswith('.xml'):
            data=W.normalize_xml(new,data)
        parts[new]=data
        E.SubElement(ct,'{'+F.CT+'}Override',PartName='/'+new,ContentType=types(old))
        rels=F.relations(source,old)
        if rels:
            root=E.Element('{'+F.PR+'}Relationships')
            for rel in rels:
                rel=copy.deepcopy(rel)
                if rel.get('TargetMode')!='External':
                    rel.set('Target','/'+mapping[F.target(old,rel.get('Target'))])
                root.append(rel)
            parts[F.rels_path(new)]=F.xml(root)
    parts['[Content_Types].xml']=F.xml(ct)

    marker=copy.deepcopy(named(base[old_order[19][1]],'v6-header-page'))
    new_rows=[]
    for i,old in enumerate(slides,1):
        dest=mapping[old]
        root=E.fromstring(parts[dest])
        page_marker=copy.deepcopy(marker)
        page_marker.find('./p:nvSpPr/p:cNvPr',F.NS).set('id',str(10000+i))
        page_marker.findall('.//a:t',F.NS)[0].text=f'{20+i} / 42'
        root.find('p:cSld/p:spTree',F.NS).append(page_marker)
        root.find('p:cSld',F.NS).set('name',f'dp44-scenario-{NAMES[i-1]}')
        parts[dest]=F.xml(root)
        new_rows.append(dict(page=20+i,key=f'dp44-scenario-{NAMES[i-1]}',title=f'DP44. {TITLES[i-1]}',
             section='본 발표',kind='inserted-dp44-scenario',part=dest,
             slide_id=max(int(n) for n,_ in old_order)+i,source_path=str(SOURCE),source_page=i,
             source_sha256=F.digest((repo/SOURCE).read_bytes()),native_tables=0))

    rows=[]
    for old_page,row in enumerate(prev['slides'],1):
        if old_page==21:
            rows.extend(copy.deepcopy(new_rows))
        page=old_page if old_page<=20 else old_page+4
        row=copy.deepcopy(row)
        row.update(page=page,source_page=old_page,source_path=str(BASE),kind='retained-v14')
        part=row['part']
        before=field(base[part]); prefix=f'{old_page} / 38'
        assert before.startswith(prefix),(old_page,before)
        parts[part]=replace_page(parts[part],f'{page} / 42'+before[len(prefix):])
        rows.append(row)
    assert len(rows)==42 and rows[20:24]==new_rows
    root=E.fromstring(parts['ppt/presentation.xml'])
    rels=E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
    for rel in list(rels):
        if rel.get('Type').endswith('/slide'):
            rels.remove(rel)
    listing=root.find('p:sldIdLst',F.NS)
    listing.clear()
    for row in rows:
        rid='V15Slide'+str(row['page'])
        E.SubElement(listing,'{'+F.P+'}sldId',{'id':str(row['slide_id']),'{'+F.R+'}id':rid})
        E.SubElement(rels,'{'+F.PR+'}Relationship',Id=rid,Type=F.R+'/slide',Target='/'+row['part'])
    for section in root.findall('.//{'+F.P14+'}section'):
        ids=section.find('{'+F.P14+'}sldIdLst');ids.clear()
        selected=rows[:33] if section.get('name')=='본 발표' else rows[33:]
        for row in selected:
            E.SubElement(ids,'{'+F.P14+'}sldId',id=str(row['slide_id']))
    parts['ppt/presentation.xml']=F.xml(root)
    parts['ppt/_rels/presentation.xml.rels']=F.xml(rels)
    app=E.fromstring(parts['docProps/app.xml'])
    for node in app:
        if node.tag.split('}')[-1] in ('Slides','Notes'):
            node.text='42'
    parts['docProps/app.xml']=F.xml(app)

    for row in rows:
        part=row['part']
        row['slide_sha256']=F.digest(parts[part])
        row['notes_sha256']=F.digest(parts[F.related(parts,part,'notesSlide')])
    assert len(F.ordered(parts))==42
    for n in base:
        if n.startswith(('ppt/media/','ppt/embeddings/','ppt/charts/','ppt/slideMasters/','ppt/slideLayouts/','ppt/notesMasters/','ppt/theme/')):
            assert parts[n]==base[n],n
    F.verify(parts)
    policy=W.policy_report(parts)
    assert not policy['violations'],policy['violations'][:10]
    F.write(repo/OUTPUT,parts)
    manifest=dict(version=15,published_date='2026-10-15',source_path=str(BASE),source_sha256=prev['output_sha256'],
        new_slide_source_path=str(SOURCE),new_slide_source_sha256=F.digest((repo/SOURCE).read_bytes()),
        output_path=str(OUTPUT),output_sha256=F.digest((repo/OUTPUT).read_bytes()),total_slides=42,
        main_slides=33,appendix_first_page=34,appendix_last_page=42,
        scenario_pages=[21,22,23,24],retained_combined_page=25,
        old_dp44_comparison_page=20,preserved_existing_media_and_layouts=True,
        authoring_policy=policy,slides=rows)
    (repo/MANIFEST).write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
    print('PASS: v15, 42 slides; scenarios 21-24, retained combined diagram 25')


if __name__=='__main__':
    main(Path(sys.argv[1]))
