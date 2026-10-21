"""Insert seven native DP44 slides after v8 page23, preserving original slide content.

Only existing footer text, presentation registries, content types and app slide
count are changed. Original notes, relationships, masters and media are retained.
"""
from pathlib import Path
import sys, json, re, uuid
import xml.etree.ElementTree as E
import full_presentation_package as F
import presentation_windows_policy as W
from package_dp3_cpu_v8 import normalize_master_registry, element_signature
from package_dp3_cpu_v6 import replace_footer_bytes


def main(repo, build):
 source_path=build/'source-v8.pptx'
 source=F.read(source_path); authored=F.read(build/'dp44-v9-authored.pptx')
 assert F.digest(source_path.read_bytes())=='0b2a87104b740714a9ced05e35b36407ffca723e172e40c2afc5dc00118ace10'
 old=F.ordered(source); incoming=F.ordered(authored)
 assert len(old)==29 and len(incoming)==7
 authored={n:W.normalize_xml(n,d) if n.endswith('.xml') else d for n,d in authored.items()}
 parts=dict(source);imp=F.Importer(parts)
 fixed=[f'ppt/slides/slide{n}.xml' for n in range(624,631)]
 assert not any(n in parts for n in fixed)
 imp.import_slides(authored,[p for _,p in incoming],'dp44v9',fixed)
 F.add_registry(parts,imp);registry=normalize_master_registry(parts)
 root=E.fromstring(parts['ppt/presentation.xml']);rels=E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
 lst=root.find('p:sldIdLst',F.NS)
 sid=max(int(n.get('id')) for n in lst)+1
 added=[]
 for i,part in enumerate(fixed):
  rid=f'rIdDP44v9_{i+1}'
  assert rid not in [r.get('Id') for r in rels]
  E.SubElement(rels,'{'+F.PR+'}Relationship',{'Id':rid,'Type':F.R+'/slide','Target':'/'+part})
  n=E.Element('{'+F.P+'}sldId',{'id':str(sid+i),'{'+F.R+'}id':rid})
  lst.insert(23+i,n);added.append((str(sid+i),part))
 # Add section if v8 has section metadata. Existing slide memberships are untouched.
 sections=next((n for n in root.iter() if n.tag=='{'+F.P14+'}sectionLst'),None)
 if sections is not None:
  sec=E.Element('{'+F.P14+'}section',{'name':'DP44 음성 준비와 재생','id':'{'+str(uuid.uuid5(uuid.NAMESPACE_URL,'via-dp44-v9')).upper()+'}'})
  ids=E.SubElement(sec,'{'+F.P14+'}sldIdLst')
  for value,_ in added:E.SubElement(ids,'{'+F.P14+'}sldId',{'id':value})
  appendix_sid=old[23][0]
  at=next((i for i,n in enumerate(sections) if any(x.get('id')==appendix_sid for x in n.iter())),len(sections))
  sections.insert(at,sec)
 parts['ppt/presentation.xml']=F.xml(root);parts['ppt/_rels/presentation.xml.rels']=F.xml(rels)
 footers=[]
 for i,(_,part) in enumerate(old,1):
  new=i if i<=23 else i+7
  before=f'{i} / 29';after=f'{new} / 36'
  parts[part]=replace_footer_bytes(source[part],before,after)
  assert replace_footer_bytes(parts[part],after,before)==source[part]
  footers.append({'source_page':i,'output_page':new,'part':part,'from':before,'to':after,'outside_footer_byte_preserved':True})
 # The appendix divider's current-location note is an explicit cross-reference,
 # unlike the historical notes inside the preserved appendix.
 divider_note=F.related(source,old[23][1],'notesSlide')
 old_ref='Appendix25~29쪽으로이동했다. 페이지번호만현행29장에맞춘다.'
 new_ref='Appendix32~36쪽으로이동했다. 페이지번호만현행36장에맞춘다.'
 assert old_ref.encode() in parts[divider_note]
 parts[divider_note]=parts[divider_note].replace(old_ref.encode(),new_ref.encode())
 # App metadata counts are allowed to change. No old slide contents are normalized.
 if 'docProps/app.xml' in parts:
  parts['docProps/app.xml']=re.sub(rb'(<(?:\w+:)?Slides>)29(</(?:\w+:)?Slides>)',rb'\g<1>36\g<2>',parts['docProps/app.xml'])
 expected=old[:23]+added+old[23:]
 assert F.ordered(parts)==expected
 allowed={'ppt/presentation.xml','ppt/_rels/presentation.xml.rels','[Content_Types].xml','docProps/app.xml'}|{p for _,p in old}|{divider_note}
 changed={n for n in source if source[n]!=parts[n]}
 assert changed<=allowed,changed-allowed
 for _,p in old:
  note=F.related(source,p,'notesSlide')
  for n in (F.rels_path(p),note,F.rels_path(note)):
   assert source[n]==(parts[n].replace(new_ref.encode(),old_ref.encode()) if n==divider_note else parts[n]),n
 for page,(_,part) in enumerate(expected,1):
  texts=[''.join(t.text or '' for t in s.findall('.//a:t',F.NS)) for s in E.fromstring(parts[part]).findall('.//p:sp',F.NS)]
  assert texts.count(f'{page} / 36')==1,(page,'footer',texts[-2:])
 F.verify(parts);policy=W.policy_report(parts)
 F.write(build/'candidate.pptx',parts)
 report={'version':'DP44-v9','source_sha256':F.digest(source_path.read_bytes()),'output_sha256':F.digest((build/'candidate.pptx').read_bytes()),'slides':36,'inserted_pages':list(range(24,31)),'existing_slide_footer_changes':footers,'existing_notes_relationships_masters_media_preserved_except_explicit_crossref':True,'all_source_parts_retained':set(source)<=set(parts),'changed_existing_parts':sorted(changed),'registry':registry,'policy':policy,'cross_reference_changes':[{'part':divider_note,'from':old_ref,'to':new_ref}],'cross_reference_review':'The appendix divider note now names pages32–36 and total36. Earlier historical source-specific references are preserved.'}
 (build/'preservation.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
 print(json.dumps({'pages':36,'old_preserved_except_footer':29,'inserted':7,'policy':policy},ensure_ascii=False))
if __name__=='__main__':main(Path(sys.argv[1]).resolve(),Path(sys.argv[2]).resolve())
