"""Replace only the v5 QA areas and their notes; preserve every structural shape."""
from pathlib import Path
import hashlib
import json
import re
import sys
import xml.etree.ElementTree as ET
import zipfile

from dp_v2_intro_package import read, slides
from dp_pptx_package import NS, related
from dp_v4_qa_package import SHAPE


def find(items, prefix):
    matches=[s for s in items if any(n.startswith(prefix+' / ') for n in s['names'])]
    assert len(matches)==1, (prefix, len(matches))
    return matches[0]


def graft_qa(original, generated, dp):
    text=original.decode()
    nodes=ET.fromstring(original).findall('./p:cSld/p:spTree/p:sp',NS)
    chunks=list(SHAPE.finditer(text))
    assert len(nodes)==len(chunks)
    selected=[];retained=[]
    for node,chunk in zip(nodes,chunks):
        pos=node.find('./p:spPr/a:xfrm/a:off',NS)
        if pos is not None and (934 if dp==41 else 871)<=int(pos.get('y'))/9525<1080:
            selected.append(chunk)
        else:retained.append((node,chunk))
    assert selected
    nextid=max(int(n.find('./p:nvSpPr/p:cNvPr',NS).get('id')) for n,_ in retained)+1
    newchunks=list(SHAPE.finditer(generated.decode()))
    assert len(newchunks)>=25
    replacements=[re.sub(r'(<p:cNvPr id=")\d+',lambda m:m[1]+str(nextid+i),chunk[0],count=1)
                  for i,chunk in enumerate(newchunks)]
    locations={c.start() for c in selected};first=min(locations)
    result=SHAPE.sub(lambda m:(''.join(replacements) if m.start()==first else '')
                     if m.start() in locations else m[0],text)
    after=[m[0] for m in SHAPE.finditer(result) if f'name="dp{dp}-v5-summary / ' not in m[0]]
    assert after==[c[0] for _,c in retained], 'Non-QA native shape changed'
    ids=[n.get('id') for n in ET.fromstring(result).findall('.//p:cNvPr',NS)]
    assert len(ids)==len(set(ids))
    return result.encode(),len(retained)


QA_NOTES_BOUNDARY = '최신 QA 기준과 계산'


def structural_paragraphs(note):
    body=next(m[0] for m in SHAPE.finditer(note.decode()) if 'type="body"' in m[0])
    paragraphs=list(re.finditer(r'<a:p\b[^>]*>.*?</a:p>',body,re.S))
    for i,p in enumerate(paragraphs):
        value=''.join(ET.fromstring('<root xmlns:a="'+NS['a']+'">'+p[0]+'</root>').itertext())
        if value in (QA_NOTES_BOUNDARY, '사고 실험 가정값 / 측정 결과와 최종 대안 선정 아님'):
            return ''.join(re.sub(r'<a:p\b([^>]*)>',
                lambda m:m[0] if 'xmlns:a=' in m[1] else '<a:p xmlns:a="'+NS['a']+'"'+m[1]+'>',
                p[0],count=1) for p in paragraphs[:i])
    return ''


def native_structure(slide, dp):
    nodes=ET.fromstring(slide).findall('./p:cSld/p:spTree/p:sp',NS)
    chunks=list(SHAPE.finditer(slide.decode()))
    assert len(nodes)==len(chunks)
    return [c[0] for n,c in zip(nodes,chunks)
            if (pos:=n.find('./p:spPr/a:xfrm/a:off',NS)) is None
            or int(pos.get('y'))/9525<(934 if dp==41 else 871)]


def replace_notes(original, generated, prefix=''):
    def body(text):
        matches=[m for m in SHAPE.finditer(text) if 'type="body"' in m[0]]
        assert len(matches)==1
        return matches[0]
    old,new=body(original.decode()),body(generated.decode())
    tx=re.search(r'<p:txBody>.*?</p:txBody>',new[0],re.S)[0]
    if prefix:
        boundary='<a:p xmlns:a="'+NS['a']+'"><a:r><a:t>'+QA_NOTES_BOUNDARY+'</a:t></a:r></a:p>'
        tx=re.sub(r'<a:p\b',lambda m:prefix+boundary+m[0],tx,count=1)
    replaced=re.sub(r'<p:txBody>.*?</p:txBody>',lambda _:tx,old[0],flags=re.S)
    return (original.decode()[:old.start()]+replaced+original.decode()[old.end():]).encode()


def make(candidate,source,destination,report):
    before,generated=read(source),read(candidate)
    originals,newslides=slides(before),slides(generated)
    assert len(originals)==14 and len(newslides)==8
    parts=dict(before);allowed=set();counts={}
    for dp in (41,44,45,42):
        comparison=find(originals,f'dp{dp}-comparison')
        # Supports both the earlier v5 and a regenerated current v5.
        olddetails=[s for s in originals if any(re.match(fr'dp{dp}-v[45]-qa-detail / ',n) for n in s['names'])]
        assert len(olddetails)==1
        detail=olddetails[0]
        summary=find(newslides,f'dp{dp}-v5-summary')
        newdetail=find(newslides,f'dp{dp}-v5-qa-detail')
        parts[comparison['part']],counts[str(dp)]=graft_qa(before[comparison['part']],generated[summary['part']],dp)
        parts[detail['part']]=generated[newdetail['part']]
        for old,new in ((comparison,summary),(detail,newdetail)):
            note=related(before,old['part'],'notesSlide')
            prefix=structural_paragraphs(before[note]) if old is comparison else ''
            if old is comparison and not prefix:
                # Recover the unchanged structural explanations from v4 after an
                # earlier QA-only refresh. Geometry must still match exactly.
                legacy=read(source.with_name('VIA-DP-background-and-comparison-41-45_v4.pptx'))
                legacy_slide=find(slides(legacy),f'dp{dp}-comparison')
                assert native_structure(before[comparison['part']],dp)==native_structure(legacy[legacy_slide['part']],dp)
                prefix=structural_paragraphs(legacy[related(legacy,legacy_slide['part'],'notesSlide')])
            if old is comparison:assert prefix,'Structural notes missing'
            parts[note]=replace_notes(before[note],generated[related(generated,new['part'],'notesSlide')],prefix)
            allowed.update((old['part'],note))
    changed={n for n in parts if parts[n]!=before[n]}
    assert changed<=allowed
    assert len(slides(parts))==14
    # Current visible text and notes cannot retain old populations/results.
    current=' '.join(' '.join(ET.fromstring(v).itertext()) for n,v in parts.items()
                     if re.fullmatch(r'ppt/(?:slides/slide|notesSlides/notesSlide)\d+\.xml',n))
    for stale in ('ASR-QA-v5','W100','53800/540','1.1867','04-43'):
        assert stale not in current,stale
    with zipfile.ZipFile(source) as original,zipfile.ZipFile(destination,'w') as out:
        for info in original.infolist():out.writestr(info,parts[info.filename])
    report.write_text(json.dumps(dict(source=str(source),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
        output_slides=14,changed_parts=sorted(changed),retained_non_qa_native_shapes=counts,
        all_other_package_parts_byte_preserved=True,contract='ASR-QA-v6',candidate_values_and_grades='NOT_EVALUATED'),indent=2)+'\n')


if __name__=='__main__':
    assert len(sys.argv)==5
    make(*map(Path,sys.argv[1:]))
