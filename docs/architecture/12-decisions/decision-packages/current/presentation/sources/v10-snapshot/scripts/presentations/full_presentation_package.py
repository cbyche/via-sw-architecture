"""Import native slide dependency graphs and replace scoped slides in one deck.

PPTX contents stay editable. Binary media, notes and custom geometry are copied;
only canvas lengths, relationship targets and slide names are normalized.
"""
from pathlib import Path
from fractions import Fraction
import argparse
import hashlib
import html
import json
import posixpath
import re
import uuid
import xml.etree.ElementTree as E
import zipfile

P='http://schemas.openxmlformats.org/presentationml/2006/main'
A='http://schemas.openxmlformats.org/drawingml/2006/main'
R='http://schemas.openxmlformats.org/officeDocument/2006/relationships'
PR='http://schemas.openxmlformats.org/package/2006/relationships'
CT='http://schemas.openxmlformats.org/package/2006/content-types'
P14='http://schemas.microsoft.com/office/powerpoint/2010/main'
NS={'p':P,'a':A}
SIZE=(18288000,10287000)
OUTPUT='docs/presentations_files/VIA-architecture-presentation.pptx'
SOURCES=[
 ('VIA_과제배경_필요성_검토반영_v7.pptx','배경과 필요성',[
  ('project-background','관련 동향'),('via-role','VIA의 역할과 범위'),('project-necessity','과제 필요성')]),
 ('VIA-representative-scenario.pptx','대표 시나리오',[('representative-scenario','대표 시나리오')]),
 ('functional-requirements/VIA-functional-requirements-and-constraints.pptx','기능 요구사항',[
  ('functional-requirements','기능 요구사항과 제약사항')]),
 ('quality-attributes/VIA-quality-attributes.pptx','품질 요구사항',[
  ('quality-01-03','품질 요구사항 01~03'),('quality-04-06','품질 요구사항 04~06'),
  ('quality-workload','P21 공통 평가묶음'),('quality-cases-01-07','사례 대응 1~7'),
  ('quality-cases-08-14','사례 대응 8~14'),('quality-cases-15-21','사례 대응 15~21')]),
 ('quality-metrics/VIA-quality-metrics.pptx','QA 목표 근거',[
  (f'qa{i:02d}-metric',title) for i,title in enumerate([
   '요청 처리 정확성','상호작용 응답시간','VIA 요청 처리시간',
   'VIA 모델 사용 비용','VIA 변경 용이성','로컬 VIA 메모리 사용량'],1)]),
 ('design-overview/VIA-design-overview-DP41-45.pptx','설계 Overview',[('design-overview','설계 Overview')]),
 ('VIA-DP-background-and-comparison-41-45_v5.pptx','DP 배경과 비교',[
  ('experience-responsibilities','사용자 경험: VIA의 역할'),('experience-overlap','사용자 경험: 겹친 사건')]+
  [(f'dp{dp}-{kind}',f'DP{dp} {label}') for dp in (41,44,45,42)
   for kind,label in [('background','배경'),('comparison','설계 비교'),('qa','QA 상세')]]),
]


def digest(data):return hashlib.sha256(data).hexdigest()
def read(path):
 with zipfile.ZipFile(path) as z:
  assert z.testzip() is None
  return {i.filename:z.read(i.filename) for i in z.infolist() if not i.is_dir()}
def rels_path(part):
 folder,name=posixpath.split(part)
 return f'{folder}/_rels/{name}.rels'
def target(part,value):
 return posixpath.normpath(value.lstrip('/') if value.startswith('/') else posixpath.join(posixpath.dirname(part),value))
def relations(parts,part):
 return list(E.fromstring(parts[rels_path(part)])) if rels_path(part) in parts else []
def related(parts,part,kind):
 return next(target(part,r.get('Target')) for r in relations(parts,part) if r.get('Type').endswith('/'+kind))
def ordered(parts):
 rs={r.get('Id'):target('ppt/presentation.xml',r.get('Target')) for r in relations(parts,'ppt/presentation.xml')}
 return [(s.get('id'),rs[s.get('{'+R+'}id')]) for s in E.fromstring(parts['ppt/presentation.xml']).find('p:sldIdLst',NS)]
def xml(root):
 for prefix,uri in [('p',P),('a',A),('r',R),('p14',P14)]:E.register_namespace(prefix,uri)
 E.register_namespace('',PR if root.tag=='{'+PR+'}Relationships' else CT if root.tag=='{'+CT+'}Types' else 'unused')
 return E.tostring(root,encoding='utf-8',xml_declaration=True)
def content_types(parts):
 root=E.fromstring(parts['[Content_Types].xml'])
 defaults={n.get('Extension'):n.get('ContentType') for n in root if n.tag.endswith('Default')}
 overrides={n.get('PartName').lstrip('/'):n.get('ContentType') for n in root if n.tag.endswith('Override')}
 return lambda n:overrides.get(n) or defaults[n.rsplit('.',1)[-1]]
def closure(parts,starts):
 seen=set();todo=list(starts)
 while todo:
  n=todo.pop()
  if n in seen:continue
  assert n in parts,n
  seen.add(n)
  todo.extend(target(n,r.get('Target')) for r in relations(parts,n) if r.get('TargetMode')!='External')
 return seen


LENGTHS={
 'a:off':('x','y'),'a:ext':('cx','cy'),'a:chOff':('x','y'),'a:chExt':('cx','cy'),
 'a:gridCol':('w',),'a:tr':('h',),'a:ln':('w',),'a:tab':('pos',),
 'a:bodyPr':('lIns','rIns','tIns','bIns','spcCol'),
 'a:tcPr':('marL','marR','marT','marB'),
 'a:rPr':('sz','kern','spc'),'a:defRPr':('sz','kern','spc'),'a:endParaRPr':('sz','kern','spc'),
 'a:spcPts':('val',),'a:buSzPts':('val',),'a:pPr':('marL','marR','indent','defTabSz'),
 'a:outerShdw':('blurRad','dist'),'a:innerShdw':('blurRad','dist'),
 'a:glow':('rad',),'a:softEdge':('rad',),'a:reflection':('blurRad','dist'),
 **{f'a:lvl{i}pPr':('marL','marR','indent','defTabSz') for i in range(1,10)},
}


def scale_native(data,factor):
 if factor==1:return data
 def element(m):
  tag,attrs=m.group(1),m.group(2)
  for name in LENGTHS.get(tag,()):
   attrs=re.sub(r'(?<![\w:])'+name+r'="(-?\d+)"',
    lambda a:name+'="'+str(round(int(a[1])*factor))+'"',attrs)
  return '<'+tag+attrs+'>'
 return re.sub(r'<([\w:]+)([^<>]*)>',element,data.decode('utf-8-sig')).encode()


def name_slide(data,key):
 def replace(m):
  attrs=re.sub(r'\sname="[^"]*"','',m[1])
  return '<p:cSld'+attrs+' name="'+html.escape(key,quote=True)+'">'
 result,count=re.subn(r'<p:cSld\b([^>]*)>',replace,data.decode('utf-8-sig'),count=1)
 assert count==1
 return result.encode()


class Importer:
 def __init__(self,parts):self.parts=parts;self.types={};self.masters=[];self.note_masters=[]
 def import_slides(self,source,selected,prefix,fixed):
  assert len(selected)==len(fixed)
  graph=closure(source,selected);mapping=dict(zip(selected,fixed))
  for old,new in zip(selected,fixed):
   mapping[related(source,old,'notesSlide')]='ppt/notesSlides/notesSlide'+re.search(r'slide(\d+)\.xml',new)[1]+'.xml'
  for n in sorted(graph):
   if n not in mapping:
    folder,filename=posixpath.split(n)
    mapping[n]=f'{folder}/{prefix}_{filename}'
   assert mapping[n] not in self.parts or n in selected or n in {related(source,s,'notesSlide') for s in selected},mapping[n]
  size=E.fromstring(source['ppt/presentation.xml']).find('p:sldSz',NS)
  ratio=Fraction(SIZE[0],int(size.get('cx')))
  assert ratio==Fraction(SIZE[1],int(size.get('cy'))),'Aspect ratio changed'
  ctype=content_types(source)
  media=[]
  for n in sorted(graph):
   dest=mapping[n];data=source[n];kind=ctype(n)
   if any(k in kind for k in ('presentationml.slide+xml','presentationml.slideLayout+xml','presentationml.slideMaster+xml','drawingml.theme+xml')):
    data=scale_native(data,ratio)
   self.parts[dest]=data;self.types[dest]=kind
   if kind.endswith('presentationml.slideMaster+xml'):self.masters.append(dest)
   if kind.endswith('presentationml.notesMaster+xml'):self.note_masters.append(dest)
   if n.startswith('ppt/media/'):
    assert data==source[n]
    media.append(dict(source=n,imported=dest,sha256=digest(data)))
   rs=relations(source,n)
   if rs:
    root=E.Element('{'+PR+'}Relationships')
    for rel in rs:
     attrs=dict(rel.attrib)
     if attrs.get('TargetMode')!='External':attrs['Target']='/'+mapping[target(n,attrs['Target'])]
     E.SubElement(root,'{'+PR+'}Relationship',attrs)
    self.parts[rels_path(dest)]=xml(root)
  return mapping,ratio,media


def add_registry(parts,importer,slides=None,sections=None):
 root=E.fromstring(parts['ppt/presentation.xml']);rs=E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
 if slides is not None:
  for n in list(root):
   if n.tag.split('}')[-1] in ('sldMasterIdLst','notesMasterIdLst','sldIdLst','extLst'):root.remove(n)
  for r in list(rs):
   if r.get('Type').split('/')[-1] in ('slideMaster','notesMaster','slide'):rs.remove(r)
 masters=root.find('p:sldMasterIdLst',NS)
 if masters is None:masters=E.Element('{'+P+'}sldMasterIdLst');root.insert(0,masters)
 notes=root.find('p:notesMasterIdLst',NS)
 if notes is None:notes=E.Element('{'+P+'}notesMasterIdLst');root.insert(1,notes)
 existing={r.get('Id') for r in rs};master_start=max([int(n.get('id')) for n in masters]+[2147483647])+1
 for kind,nodes,items in [('slideMaster',masters,importer.masters),('notesMaster',notes,importer.note_masters)]:
  for i,part in enumerate(items):
   rid='Via_'+part.replace('/','_').replace('.','_');assert rid not in existing
   E.SubElement(rs,'{'+PR+'}Relationship',dict(Id=rid,Type=R+'/'+kind,Target='/'+part))
   attrs={'{'+R+'}id':rid}
   if kind=='slideMaster':attrs['id']=str(master_start+i)
   E.SubElement(nodes,'{'+P+'}'+kind+'Id',attrs)
 if slides is not None:
  lst=E.Element('{'+P+'}sldIdLst');root.insert(2,lst)
  for row in slides:
   rid='ViaSlide'+str(row['page'])
   E.SubElement(lst,'{'+P+'}sldId',{'id':str(row['slide_id']),'{'+R+'}id':rid})
   E.SubElement(rs,'{'+PR+'}Relationship',dict(Id=rid,Type=R+'/slide',Target='/'+row['part']))
  ex=E.SubElement(root,'{'+P+'}extLst')
  ex=E.SubElement(ex,'{'+P+'}ext',{'uri':'{521415D9-36F7-43E2-AB2F-B90AF26B5E84}'})
  sl=E.SubElement(ex,'{'+P14+'}sectionLst')
  for name,members in sections:
   section=E.SubElement(sl,'{'+P14+'}section',{'name':name,'id':'{'+str(uuid.uuid5(uuid.NAMESPACE_URL,'via-presentation:'+name)).upper()+'}'})
   ids=E.SubElement(section,'{'+P14+'}sldIdLst')
   for r in members:E.SubElement(ids,'{'+P14+'}sldId',{'id':str(r['slide_id'])})
 parts['ppt/presentation.xml']=xml(root);parts['ppt/_rels/presentation.xml.rels']=xml(rs)
 ct=E.fromstring(parts['[Content_Types].xml'])
 lookup={n.get('PartName'):n for n in ct if n.tag.endswith('Override')}
 for part,kind in importer.types.items():
  node=lookup.get('/'+part)
  if node is None:node=E.SubElement(ct,'{'+CT+'}Override',{'PartName':'/'+part})
  node.set('ContentType',kind)
 parts['[Content_Types].xml']=xml(ct)


def verify(parts):
 assert len({i for i,_ in ordered(parts)})==len(ordered(parts))
 for n in parts:
  if n.endswith('.xml'):E.fromstring(parts[n])
  if n.endswith('.rels'):
   src=n.replace('/_rels/','/').removesuffix('.rels') if n!='_rels/.rels' else ''
   for r in E.fromstring(parts[n]):
    if r.get('TargetMode')!='External':assert target(src,r.get('Target')) in parts,(n,r.attrib)


def write(path,parts):
 assert not path.exists(),'Refusing to overwrite an existing candidate'
 verify(parts)
 with zipfile.ZipFile(path,'w',compression=zipfile.ZIP_DEFLATED) as z:
  for n,data in parts.items():z.writestr(n,data)


def inventory(repo,plan_path):
 b=plan_path.parent;folder=b/'sources';folder.mkdir()
 plan=[]
 for i,(relative,section,items) in enumerate(SOURCES,1):
  original=repo/'docs/presentations_files'/relative;snapshot=folder/f'{i:02d}.pptx'
  data=original.read_bytes();snapshot.write_bytes(data);parts=read(snapshot)
  assert len(ordered(parts))==len(items)
  plan.append(dict(index=i,source=str(original.relative_to(repo)),snapshot=str(snapshot),sha256=digest(data),
   size=E.fromstring(parts['ppt/presentation.xml']).find('p:sldSz',NS).attrib,section=section,
   slides=[dict(key=k,title=t) for k,t in items]))
 plan_path.write_text(json.dumps(plan,ensure_ascii=False,indent=2)+'\n')


def merge(plan_path,scaffold,candidate,manifest_path):
 plan=json.loads(plan_path.read_text());parts=read(scaffold)
 # Keep the Artifact Tool-authored root and common presentation defaults.
 for n in list(parts):
  if n.startswith(('ppt/slides/','ppt/notesSlides/','ppt/slideLayouts/','ppt/slideMasters/','ppt/notesMasters/')):del parts[n]
 ct=E.fromstring(parts['[Content_Types].xml'])
 for n in list(ct):
  if n.get('PartName','').lstrip('/').startswith(('ppt/slides/','ppt/notesSlides/','ppt/slideLayouts/','ppt/slideMasters/','ppt/notesMasters/')):ct.remove(n)
 parts['[Content_Types].xml']=xml(ct)
 importer=Importer(parts);rows=[];media=[];sections=[]
 for src in plan:
  source=read(Path(src['snapshot']));source_order=ordered(source)
  first=len(rows)+1;dest=[f'ppt/slides/slide{first+i}.xml' for i in range(len(source_order))]
  mapping,ratio,assets=importer.import_slides(source,[p for _,p in source_order],f'initial{src["index"]:02d}',dest)
  media.extend(assets)
  for local,(old_id,old) in enumerate(source_order,1):
   info=src['slides'][local-1];page=len(rows)+1;part=mapping[old]
   parts[part]=name_slide(parts[part],info['key'])
   if info['key']=='representative-scenario':
    node=E.fromstring(parts[part]);pages=[s for s in node.findall('.//p:sp',NS)
     if (s.find('./p:spPr/a:xfrm/a:off',NS) is not None
      and int(s.find('./p:spPr/a:xfrm/a:off',NS).get('x'))>SIZE[0]*.94
      and int(s.find('./p:spPr/a:xfrm/a:off',NS).get('y'))>SIZE[1]*.94)]
    assert len(pages)==1
    original_text=''.join(t.text or '' for t in pages[0].findall('.//a:t',NS));assert original_text=='3'
    text=parts[part].decode();name=pages[0].find('./p:nvSpPr/p:cNvPr',NS).get('name')
    def page_change(m):return m[0].replace('<a:t>3</a:t>',f'<a:t>{page}</a:t>') if f'name="{name}"' in m[0] else m[0]
    parts[part]=re.sub(r'<p:sp\b.*?</p:sp>',page_change,text,flags=re.S).encode()
   note=related(parts,part,'notesSlide');assert parts[note]==source[related(source,old,'notesSlide')]
   rows.append(dict(page=page,key=info['key'],title=info['title'],section=src['section'],slide_id=255+page,
    part=part,initial_source=src['source'],initial_source_slide=local,initial_source_part=old,scale=str(ratio),
    native_tables=len(E.fromstring(parts[part]).findall('.//a:tbl',NS))))
  sections.append((src['section'],rows[first-1:]))
 # Split the final discussion into useful native sections without extra slides.
 sections=sections[:-1]+[('사용자 경험',rows[18:20])]+[(f'DP{dp}',rows[start:start+3])for dp,start in [(41,20),(44,23),(45,26),(42,29)]]
 assert len(rows)==32 and len({r['key'] for r in rows})==32
 add_registry(parts,importer,rows,sections)
 manifest=dict(schema_version='via-full-presentation.v1',presentation=OUTPUT,current_edit_target=True,
  slide_size_emu=list(SIZE),slide_count=32,sections=[dict(name=n,first_page=s[0]['page'],last_page=s[-1]['page'])for n,s in sections],
  slides=rows,initial_sources=[{k:v for k,v in s.items()if k!='snapshot'}for s in plan],preserved_media=media,
  maintenance='Edit the canonical PPTX; initial sources are provenance, not automatic rebuild inputs')
 write(candidate,parts);manifest_path.write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')


def replace(base,edited,mapping_file,expected_sha,candidate,report):
 raw=base.read_bytes();assert digest(raw)==expected_sha,'Current presentation changed; rebase the edit'
 before=read(base);parts=dict(before);source=read(edited);source_slides=ordered(source)
 current={E.fromstring(before[p]).find('p:cSld',NS).get('name'):p for _,p in ordered(before)}
 assert all(current) and len(current)==len(ordered(before)),'Duplicate or missing slide keys'
 mapping=json.loads(mapping_file.read_text());assert mapping and all(k in current for k in mapping)
 assert all(type(i) is int and 1<=i<=len(source_slides) for i in mapping.values()),'Invalid source slide number'
 assert len(set(mapping.values()))==len(mapping),'One source slide assigned more than once'
 selected=[source_slides[int(i)-1][1] for i in mapping.values()]
 destinations=[current[k] for k in mapping]
 importer=Importer(parts);prefix='edit_'+digest(edited.read_bytes())[:12]+'_'+expected_sha[:8]
 mapped,_,_=importer.import_slides(source,selected,prefix,destinations)
 allowed=set(destinations)|{related(before,p,'notesSlide')for p in destinations}
 allowed|={rels_path(p) for p in allowed}
 for key,p in zip(mapping,destinations):parts[p]=name_slide(parts[p],key)
 add_registry(parts,importer)
 metadata={'ppt/presentation.xml','ppt/_rels/presentation.xml.rels','[Content_Types].xml'}
 changed={n for n in before if parts[n]!=before[n]}
 assert changed<=allowed|metadata,changed-allowed-metadata
 assert ordered(before)==ordered(parts),'Slide order or IDs changed'
 for _,p in ordered(before):
  if p not in destinations:assert before[p]==parts[p] and before[related(before,p,'notesSlide')]==parts[related(parts,p,'notesSlide')]
 write(candidate,parts)
 report.write_text(json.dumps(dict(base_sha256=expected_sha,selected_keys=list(mapping),changed_existing_parts=sorted(changed),
  unselected_slides_and_notes_byte_preserved=True,all_existing_media_and_shared_resources_byte_preserved=True,
  added_parts=sorted(set(parts)-set(before))),ensure_ascii=False,indent=2)+'\n')


if __name__=='__main__':
 parser=argparse.ArgumentParser();sub=parser.add_subparsers(dest='mode',required=True)
 inv=sub.add_parser('inventory');inv.add_argument('repo',type=Path);inv.add_argument('plan',type=Path)
 mer=sub.add_parser('merge')
 for key in ['plan','scaffold','candidate','manifest']:mer.add_argument(key,type=Path)
 rep=sub.add_parser('replace')
 for key in ['base','edited','mapping']:rep.add_argument(key,type=Path)
 rep.add_argument('expected_sha');rep.add_argument('candidate',type=Path);rep.add_argument('report',type=Path)
 args=vars(parser.parse_args());mode=args.pop('mode')
 if mode=='inventory':inventory(args['repo'],args['plan'])
 elif mode=='merge':merge(*(args[k] for k in ['plan','scaffold','candidate','manifest']))
 else:replace(*(args[k] for k in ['base','edited','mapping','expected_sha','candidate','report']))
