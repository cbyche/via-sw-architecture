"""Copy v3 to v4 with native QA summaries and four inserted detail pages."""
from pathlib import Path
import hashlib
import html
import json
import re
import sys
import xml.etree.ElementTree as ET
import zipfile

from dp_v2_intro_package import read, slides
from dp_pptx_package import A, P, R, NS, related, rels_path

PR='http://schemas.openxmlformats.org/package/2006/relationships'
CT='http://schemas.openxmlformats.org/package/2006/content-types'
SHAPE=re.compile(r'<p:sp(?:\s[^>]*)?>.*?</p:sp>',re.S)


def find(items,slug):
    matches=[s for s in items if any(n.startswith(slug+' / ') for n in s['names'])]
    assert len(matches)==1,(slug,matches)
    return matches[0]


def graft_table(original,generated,dp):
    text=original.decode()
    nodes=ET.fromstring(original).findall('./p:cSld/p:spTree/p:sp',NS)
    chunks=list(SHAPE.finditer(text))
    assert len(nodes)==len(chunks)
    selected=[];retained=[]
    for node,chunk in zip(nodes,chunks):
        offset=node.find('./p:spPr/a:xfrm/a:off',NS)
        if offset is not None and (934 if dp==41 else 871)<=int(offset.get('y'))/9525<1080:
            selected.append(chunk)
        else:retained.append((node,chunk))
    assert selected
    nextid=max(int(n.find('./p:nvSpPr/p:cNvPr',NS).get('id')) for n,_ in retained)+1
    newchunks=list(SHAPE.finditer(generated.decode()))
    assert len(newchunks)>24
    replacements=[]
    for i,chunk in enumerate(newchunks):
        replacements.append(re.sub(r'(<p:cNvPr id=")\d+',lambda m:m[1]+str(nextid+i),chunk[0],count=1))
    locations={c.start() for c in selected};first=min(locations)
    result=SHAPE.sub(lambda m:(''.join(replacements) if m.start()==first else '')
                     if m.start() in locations else m[0],text)
    after=[m[0] for m in SHAPE.finditer(result) if f'name="dp{dp}-v4-summary / ' not in m[0]]
    assert after==[c[0] for _,c in retained], 'Non-QA native object changed'
    ids=[n.get('id') for n in ET.fromstring(result).findall('.//p:cNvPr',NS)]
    assert len(ids)==len(set(ids))
    return result.encode(),len(retained)


def append_notes(raw,paragraphs):
    text=raw.decode()
    for chunk in SHAPE.finditer(text):
        if 'type="body"' not in chunk[0]:continue
        extra=''.join(f'<a:p xmlns:a="{A}"><a:r><a:t>{html.escape(line)}</a:t></a:r></a:p>' for line in paragraphs.split('\n'))
        body=chunk[0].replace('</p:txBody>',extra+'</p:txBody>')
        return (text[:chunk.start()]+body+text[chunk.end():]).encode()
    raise AssertionError('Notes body placeholder missing')


def relationship_copy(raw,kind,target):
    root=ET.fromstring(raw)
    for rel in root:
        if rel.get('Type','').endswith('/'+kind):rel.set('Target','/'+target)
    ET.register_namespace('',PR)
    return ET.tostring(root,encoding='utf-8',xml_declaration=True)


def make(candidate,source,destination,scene_file,report):
    before,generated=read(source),read(candidate)
    parts=dict(before);originals=slides(before);newslides=slides(generated)
    metadata=json.loads(scene_file.read_text())['scenes']
    byslug={s['slug']:s for s in metadata}
    sn=1+max(int(re.search(r'slide(\d+)\.xml$',n)[1]) for n in parts if re.fullmatch(r'ppt/slides/slide\d+\.xml',n))
    nn=1+max(int(re.search(r'notesSlide(\d+)\.xml$',n)[1]) for n in parts if re.fullmatch(r'ppt/notesSlides/notesSlide\d+\.xml',n))
    nextid=1+max(int(s['id']) for s in originals)
    added={};selected=set();counts={}
    for index,dp in enumerate((41,44,45,42)):
        old=find(originals,f'dp{dp}-comparison')
        summary=find(newslides,f'dp{dp}-v4-summary')
        detail=find(newslides,f'dp{dp}-v4-qa-detail')
        parts[old['part']],counts[str(dp)]=graft_table(before[old['part']],generated[summary['part']],dp)
        note=related(before,old['part'],'notesSlide')
        parts[note]=append_notes(before[note],byslug[f'dp{dp}-v4-summary']['notes'])
        selected.update((old['part'],note))
        sp=f'ppt/slides/slide{sn+index}.xml';np=f'ppt/notesSlides/notesSlide{nn+index}.xml'
        parts[sp]=generated[detail['part']]
        parts[np]=generated[related(generated,detail['part'],'notesSlide')]
        parts[rels_path(sp)]=relationship_copy(before[rels_path(old['part'])],'notesSlide',np)
        parts[rels_path(np)]=relationship_copy(before[rels_path(note)],'slide',sp)
        added[dp]=dict(part=sp,id=str(nextid+index),rid=f'RviaV4QA{dp}')
    order=[]
    # Retain integrated storytelling and place 43 in the existing appendix.
    if len(originals)==12:
        for old in originals:
            order.append(old)
            for dp in added:
                if any(n.startswith(f'dp{dp}-comparison / ') for n in old['names']):order.append(added[dp])
    else:
        assert len(originals)==5
        for dp in (41,44,45,42):order.extend([find(originals,f'dp{dp}-comparison'),added[dp]])
        order.append(find(originals,'dp43-comparison'))
    relations=ET.fromstring(before['ppt/_rels/presentation.xml.rels'])
    for s in added.values():
        ET.SubElement(relations,f'{{{PR}}}Relationship',Type=f'{R}/slide',Target='/'+s['part'],Id=s['rid'])
    ET.register_namespace('',PR)
    parts['ppt/_rels/presentation.xml.rels']=ET.tostring(relations,encoding='utf-8',xml_declaration=True)
    types=ET.fromstring(before['[Content_Types].xml'])
    for s in added.values():
        for part,kind in ((s['part'],'slide'),(related(parts,s['part'],'notesSlide'),'notesSlide')):
            ET.SubElement(types,f'{{{CT}}}Override',PartName='/'+part,ContentType=f'application/vnd.openxmlformats-officedocument.presentationml.{kind}+xml')
    ET.register_namespace('',CT)
    parts['[Content_Types].xml']=ET.tostring(types,encoding='utf-8',xml_declaration=True)
    ptext=before['ppt/presentation.xml'].decode()
    ids='<p:sldIdLst>'+''.join(f'<p:sldId id="{s["id"]}" r:id="{s["rid"]}" xmlns:r="{R}" />' for s in order)+'</p:sldIdLst>'
    ptext=re.sub(r'<p:sldIdLst>.*?</p:sldIdLst>',lambda _:ids,ptext,flags=re.S)
    # Extend the existing main section membership while preserving its name/ID.
    def section(match):
        value=match[0]
        if '본 발표' not in value:return value
        main=order[:-2] if len(originals)==12 else order[:-1]
        ids='<p14:sldIdLst>'+''.join(f'<p14:sldId id="{s["id"]}" />' for s in main)+'</p14:sldIdLst>'
        return re.sub(r'<p14:sldIdLst>.*?</p14:sldIdLst>',lambda _:ids,value,flags=re.S)
    ptext=re.sub(r'<p14:section\b.*?</p14:section>',section,ptext,flags=re.S)
    parts['ppt/presentation.xml']=ptext.encode()
    parts['docProps/app.xml']=re.sub(rb'<ap:(Slides|Notes)>\d+</ap:\1>',lambda m:b'<ap:'+m[1]+b'>'+str(len(order)).encode()+b'</ap:'+m[1]+b'>',before['docProps/app.xml'])
    allowed=selected|{'ppt/presentation.xml','ppt/_rels/presentation.xml.rels','[Content_Types].xml','docProps/app.xml'}
    changed={n for n in before if parts[n]!=before[n]}
    assert changed<=allowed,changed-allowed
    assert len(slides(parts))==len(originals)+4
    with zipfile.ZipFile(source) as original,zipfile.ZipFile(destination,'w') as out:
        for info in original.infolist():out.writestr(info,parts.pop(info.filename))
        for name,value in parts.items():out.writestr(name,value)
    report.write_text(json.dumps(dict(source=str(source),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
        source_slides=len(originals),v4_slides=len(order),changed_parts=sorted(changed),
        retained_non_qa_native_objects=counts,all_other_existing_parts_preserved=True),indent=2)+'\n')


if __name__=='__main__':
    assert len(sys.argv)==6
    make(*map(Path,sys.argv[1:]))
