"""Replace four DP3 slides with five; preserve every other slide body."""
from pathlib import Path
import copy
import json
import re
import sys
import xml.etree.ElementTree as E
import full_presentation_package as F
import presentation_windows_policy as W


def main(repo, build):
    source_path = repo/'docs/presentations_files/dp1-dp4-reference-style-20261019/VIA-DP1-DP4-reference-style-16slides-v2-windows.pptx'
    original = F.read(source_path)
    source = F.read(build/'dp3-authored.pptx')
    source = {k: W.normalize_xml(k,v) if k.endswith('.xml') else v for k,v in source.items()}
    parts = dict(original)
    order = F.ordered(original)
    selected = [part for _,part in F.ordered(source)]
    assert len(order)==16 and len(selected)==5
    importer = F.Importer(parts)
    fixed = [f'ppt/slides/slide{109+i}.xml' for i in range(5)]
    importer.import_slides(source,selected,'dp3parallelv3',fixed)
    rows=[]
    preserved=[]
    for old_page,(_,part) in enumerate(order[:8],1):
        rows.append(dict(page=len(rows)+1,part=part,slide_id=255+len(rows)+1,old_page=old_page))
    for part in fixed:
        rows.append(dict(page=len(rows)+1,part=part,slide_id=255+len(rows)+1,old_page=None))
    for old_page,(_,part) in enumerate(order[12:],13):
        rows.append(dict(page=len(rows)+1,part=part,slide_id=255+len(rows)+1,old_page=old_page))
    for row in rows:
        part=row['part'];root=E.fromstring(parts[part])
        if row['old_page']:
            # Change only the existing native footer; exact body preservation below.
            names=['Continuous slide number']
        else:
            names=[]
        candidates=[]
        for shape in root.findall('.//p:sp',F.NS):
            props=shape.find('p:nvSpPr/p:cNvPr',F.NS)
            text=''.join(t.text or '' for t in shape.findall('.//a:t',F.NS))
            if (props is not None and props.get('name') in names) or (not row['old_page'] and re.fullmatch(r'[1-5]',text)):
                off=shape.find('p:spPr/a:xfrm/a:off',F.NS)
                if row['old_page'] or off is not None and int(off.get('y'))>9900000:
                    candidates.append(shape)
        assert len(candidates)==1,(row,len(candidates))
        spans=candidates[0].findall('.//a:t',F.NS);assert len(spans)==1
        spans[0].text=f"{row['page']} / 17"
        # Use the established combined-deck footer geometry.
        pos=candidates[0].find('p:spPr/a:xfrm',F.NS)
        pos.find('a:off',F.NS).attrib.update(x=str(1836*9525),y=str(1052*9525))
        pos.find('a:ext',F.NS).attrib.update(cx=str(62*9525),cy=str(21*9525))
        parts[part]=F.xml(root)
        if row['old_page']:
            old_root=E.fromstring(original[part]);check=copy.deepcopy(root)
            old_footer=next(s for s in old_root.findall('.//p:sp',F.NS) if s.find('p:nvSpPr/p:cNvPr',F.NS).get('name')=='Continuous slide number')
            new_footer=next(s for s in check.findall('.//p:sp',F.NS) if s.find('p:nvSpPr/p:cNvPr',F.NS).get('name')=='Continuous slide number')
            parent=check.find('p:cSld/p:spTree',F.NS);idx=list(parent).index(new_footer);parent.remove(new_footer);parent.insert(idx,copy.deepcopy(old_footer))
            assert F.xml(check)==F.xml(old_root),part
            assert parts[F.related(parts,part,'notesSlide')]==original[F.related(original,part,'notesSlide')]
            if row['old_page']>=13:
                note=F.related(parts,part,'notesSlide')
                parts[note]=parts[note].replace('16쪽'.encode(),'17쪽'.encode())
            preserved.append(row['old_page'])
    # Keep original masters/layouts/notes registries and add imported dependencies.
    # add_registry's full rebuild discards those registries, so update slide list directly.
    F.add_registry(parts,importer)
    root=E.fromstring(parts['ppt/presentation.xml']);rels=E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
    listing=root.find('p:sldIdLst',F.NS);listing.clear()
    for rel in list(rels):
        if rel.get('Type').endswith('/slide'):rels.remove(rel)
    for row in rows:
        rid='Dp3ParallelV3Slide'+str(row['page'])
        E.SubElement(listing,'{'+F.P+'}sldId',{'id':str(row['slide_id']),'{'+F.R+'}id':rid})
        E.SubElement(rels,'{'+F.PR+'}Relationship',Id=rid,Type=F.R+'/slide',Target='/'+row['part'])
    # Remove old section lists that reference replaced slide IDs.
    for ex in root.findall('p:extLst',F.NS):root.remove(ex)
    parts['ppt/presentation.xml']=F.xml(root);parts['ppt/_rels/presentation.xml.rels']=F.xml(rels)
    app=E.fromstring(parts['docProps/app.xml'])
    for node in app:
        if node.tag.split('}')[-1] in ('Slides','Notes'):node.text='17'
    parts['docProps/app.xml']=F.xml(app)
    F.verify(parts);assert len(F.ordered(parts))==17
    policy=W.policy_report(parts)
    F.write(build/'candidate.pptx',parts)
    (build/'preservation.json').write_text(json.dumps(dict(source=str(source_path),source_sha256=F.digest(source_path.read_bytes()),slides=17,
        replaced_source_pages=[9,10,11,12],new_dp3_pages=[9,10,11,12,13],preserved_body_and_notes_source_pages=preserved,
        allowed_changes_to_other_slides='native page footer and DP4 notes cross-reference 16쪽 to 17쪽 only',no_new_model_calls=True,new_parallel_normalized_B='NOT_RUN',policy=policy),ensure_ascii=False,indent=2)+'\n')
    print('Preserved 12 slide bodies and notes; authored 5 DP3 slides; no benchmark execution.')


if __name__=='__main__':main(Path(sys.argv[1]),Path(sys.argv[2]))
