"""Replace the v2 introduction only, preserving every other slide part.

The existing integrated deck has eleven slides. Replace its overview with two
user-experience pages, or replace the existing two v2 intro pages on rerun.
Generated images are imported with new relationship IDs and package paths.
"""
from __future__ import annotations

import hashlib
import json
import re
import sys
import zipfile
from pathlib import Path
import xml.etree.ElementTree as E

from dp_pptx_package import P, R, NS, related, rels_path, target_part

PR = 'http://schemas.openxmlformats.org/package/2006/relationships'
SCENES = ('via-v2-responsibilities', 'via-v2-overlapping-events')


def read(p):
    with zipfile.ZipFile(p) as z:
        return {i.filename: z.read(i.filename) for i in z.infolist()}


def slides(parts):
    relations = {r.get('Id'): target_part('ppt/presentation.xml', r.get('Target'))
                 for r in E.fromstring(parts['ppt/_rels/presentation.xml.rels'])}
    result = []
    for node in E.fromstring(parts['ppt/presentation.xml']).find('p:sldIdLst', NS):
        part = relations[node.get(f'{{{R}}}id')]
        names = [n.get('name', '') for n in E.fromstring(parts[part]).findall('.//p:cNvPr', NS)]
        result.append(dict(part=part, id=node.get('id'), rid=node.get(f'{{{R}}}id'), names=names))
    return result


def intro_index(items):
    count = sum(any(n.startswith('via-v2-') for n in s['names']) for s in items)
    assert count in (0, 2), ('Unexpected v2 intro count', count)
    if count:
        assert all(any(n.startswith(slug+' / ') for n in items[i]['names']) for i, slug in enumerate(SCENES))
        return 2
    assert len(items) == 11 and any(n.startswith('dp-background-overview / ') for n in items[0]['names'])
    return 1


def xml_bytes(root):
    if root.tag == '{http://schemas.openxmlformats.org/package/2006/content-types}Types':
        E.register_namespace('', 'http://schemas.openxmlformats.org/package/2006/content-types')
    E.register_namespace('p', P)
    E.register_namespace('r', R)
    E.register_namespace('p14', 'http://schemas.microsoft.com/office/powerpoint/2010/main')
    return E.tostring(root, encoding='utf-8', xml_declaration=True)


def graft(candidate_path, source_path, report_path):
    generated, before = read(candidate_path), read(source_path)
    parts = dict(before)
    current, new = slides(parts), slides(generated)
    assert len(new) == 2
    old_count = intro_index(current)
    retained = current[old_count:]
    assert len(retained) == 10
    # Each unselected slide and all its existing layout/notes/media remain intact.
    intro = current[:old_count]
    if old_count == 1:
        old = current[0]
        old_note = related(parts, old['part'], 'notesSlide')
        sn = 1 + max(int(re.search(r'slide(\d+)\.xml$', p)[1]) for p in parts if re.fullmatch(r'ppt/slides/slide\d+\.xml', p))
        nn = 1 + max(int(re.search(r'notesSlide(\d+)\.xml$', p)[1]) for p in parts if re.fullmatch(r'ppt/notesSlides/notesSlide\d+\.xml', p))
        added = dict(part=f'ppt/slides/slide{sn}.xml', id=str(1+max(int(s['id']) for s in current)), rid='RviaV2Experience')
        note = f'ppt/notesSlides/notesSlide{nn}.xml'
        parts[added['part']] = parts[old['part']]
        parts[note] = parts[old_note]
        sr = E.fromstring(parts[rels_path(old['part'])])
        for r in sr:
            if r.get('Type', '').endswith('/notesSlide'):
                r.set('Target', '/'+note)
        parts[rels_path(added['part'])] = xml_bytes(sr)
        nr = E.fromstring(parts[rels_path(old_note)])
        for r in nr:
            if r.get('Type', '').endswith('/slide'):
                r.set('Target', '/'+added['part'])
        parts[rels_path(note)] = xml_bytes(nr)
        pr = E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
        assert added['rid'] not in {r.get('Id') for r in pr}
        E.SubElement(pr, f'{{{PR}}}Relationship', {'Id':added['rid'], 'Type':R+'/slide', 'Target':'/'+added['part']})
        parts['ppt/_rels/presentation.xml.rels'] = xml_bytes(pr)
        ct = E.fromstring(parts['[Content_Types].xml'])
        ct_ns = 'http://schemas.openxmlformats.org/package/2006/content-types'
        for part, kind in ((added['part'],'slide'),(note,'notesSlide')):
            E.SubElement(ct,f'{{{ct_ns}}}Override',{'PartName':'/'+part,'ContentType':f'application/vnd.openxmlformats-officedocument.presentationml.{kind}+xml'})
        parts['[Content_Types].xml'] = xml_bytes(ct)
        intro.append(added)

    for index, (dst, src) in enumerate(zip(intro, new), 1):
        old_note = related(parts, dst['part'], 'notesSlide')
        new_note = related(generated, src['part'], 'notesSlide')
        rels = E.fromstring(parts[rels_path(dst['part'])])
        for r in list(rels):
            if r.get('Id','').startswith('RviaIntroImage'):
                rels.remove(r)
        slide_xml = generated[src['part']]
        source_rels = E.fromstring(generated[rels_path(src['part'])])
        image_count = 0
        for r in source_rels:
            if not r.get('Type','').endswith('/image'):
                continue
            image_count += 1
            image = target_part(src['part'],r.get('Target'))
            suffix = Path(image).suffix
            media = f'ppt/media/via-v2-intro-{index}-{image_count}{suffix}'
            parts[media] = generated[image]
            rid = f'RviaIntroImage{image_count}'
            assert rid not in {r.get('Id') for r in rels}
            slide_xml = slide_xml.replace(('r:embed="'+r.get('Id')+'"').encode(), ('r:embed="'+rid+'"').encode())
            E.SubElement(rels,f'{{{PR}}}Relationship',{'Id':rid,'Type':r.get('Type'),'Target':'/'+media})
        assert 1 <= image_count <= 3
        # The artifact exporter derives a symmetric "cover" crop and currently
        # omits the authored asymmetric image.crop. Preserve the PNG pixels and
        # set the native OOXML framing so the user's face remains inside frame.
        slide_root = E.fromstring(slide_xml)
        crops = ([(0,15000,70000,19000),(35000,25000,34000,17000),(73500,15000,500,17000)]
                 if index == 1 else
                 [(0,15000,67000,22000),(36000,25000,36000,24000),(65000,15000,1000,27000)])
        pictures = slide_root.findall('.//p:pic', NS)
        assert len(pictures) == 3
        for picture, (left, top, right, bottom) in zip(pictures, crops):
            crop = picture.find('.//a:srcRect', NS)
            assert crop is not None
            crop.attrib.update(l=str(left), t=str(top), r=str(right), b=str(bottom))
        E.register_namespace('a', 'http://schemas.openxmlformats.org/drawingml/2006/main')
        slide_xml = xml_bytes(slide_root)
        parts[dst['part']] = slide_xml
        parts[rels_path(dst['part'])] = xml_bytes(rels)
        parts[old_note] = generated[new_note]

    ct = E.fromstring(parts['[Content_Types].xml'])
    if not any(r.get('Extension') == 'png' for r in ct):
        E.SubElement(ct,'{http://schemas.openxmlformats.org/package/2006/content-types}Default',{'Extension':'png','ContentType':'image/png'})
    parts['[Content_Types].xml'] = xml_bytes(ct)
    order = intro + retained
    root = E.fromstring(parts['ppt/presentation.xml'])
    ids = root.find('p:sldIdLst',NS)
    ids.clear()
    for s in order:
        E.SubElement(ids,f'{{{P}}}sldId',{'id':s['id'],f'{{{R}}}id':s['rid']})
    p14 = 'http://schemas.microsoft.com/office/powerpoint/2010/main'
    section = root.find(f'.//{{{p14}}}sectionLst')
    if section is not None:
        for node, members in zip(list(section),(order[:10],order[10:])):
            sids=node.find(f'{{{p14}}}sldIdLst');sids.clear()
            for s in members:E.SubElement(sids,f'{{{p14}}}sldId',{'id':s['id']})
    parts['ppt/presentation.xml'] = xml_bytes(root)
    parts['docProps/app.xml'] = re.sub(rb'<ap:(Slides|Notes)>\d+</ap:\1>', lambda m:b'<ap:'+m[1]+b'>12</ap:'+m[1]+b'>',parts['docProps/app.xml'])

    mutable = {'[Content_Types].xml','ppt/presentation.xml','ppt/_rels/presentation.xml.rels','docProps/app.xml'}
    for s in intro:
        mutable.update((s['part'],rels_path(s['part']),related(parts,s['part'],'notesSlide')))
    mutable.update(p for p in parts if p.startswith('ppt/media/via-v2-intro-'))
    changed = [p for p in before if before[p] != parts[p]]
    assert set(changed) <= mutable, sorted(set(changed)-mutable)
    for s in retained:
        assert parts[s['part']] == before[s['part']]
        assert parts[related(parts,s['part'],'notesSlide')] == before[related(before,s['part'],'notesSlide')]
    assert len(slides(parts)) == 12
    with zipfile.ZipFile(candidate_path) as candidate, zipfile.ZipFile(str(candidate_path)+'.scoped','w',zipfile.ZIP_DEFLATED) as out:
        for p, data in parts.items():out.writestr(p,data)
    Path(str(candidate_path)+'.scoped').replace(candidate_path)
    Path(report_path).write_text(json.dumps({'source':str(source_path),'source_sha256':hashlib.sha256(Path(source_path).read_bytes()).hexdigest(),'unselected_slides_preserved':10,'intro_pages':2,'changed_existing_parts':changed,'new_parts':sorted(set(parts)-set(before))},indent=2)+'\n')


if __name__ == '__main__':
    assert len(sys.argv) == 4
    graft(*map(Path,sys.argv[1:]))
