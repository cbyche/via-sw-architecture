"""Check preserved v8 content and the native DP44 v9 scene/evidence contract."""
from pathlib import Path
import sys,json,hashlib,xml.etree.ElementTree as E
import full_presentation_package as F
import presentation_windows_policy as W
from package_dp3_cpu_v6 import replace_footer_bytes

def check(repo,source_path,final_path,scene_path,preview_dir=None,source_preview_dir=None):
 source=F.read(source_path);final=F.read(final_path);before=F.ordered(source);after=F.ordered(final)
 assert len(before)==29 and len(after)==36
 cross_old='Appendix25~29쪽으로이동했다. 페이지번호만현행29장에맞춘다.'
 cross_new='Appendix32~36쪽으로이동했다. 페이지번호만현행36장에맞춘다.'
 rows=[]
 for oldpage,(_,part) in enumerate(before,1):
  newpage=oldpage if oldpage<=23 else oldpage+7
  assert after[newpage-1][1]==part
  assert replace_footer_bytes(final[part],f'{newpage} / 36',f'{oldpage} / 29')==source[part]
  note=F.related(source,part,'notesSlide')
  for n in (F.rels_path(part),note,F.rels_path(note)):
   actual=final[n]
   if oldpage==24 and n==note:actual=actual.replace(cross_new.encode(),cross_old.encode())
   assert actual==source[n],n
  rows.append({'source_page':oldpage,'final_page':newpage,'body_notes_relations_preserved_with_allowed_edits':True})
 scenes=json.loads(scene_path.read_text());assert len(scenes)==7
 qatext=[];native=[]
 for scene,(_,part) in zip(scenes,after[23:30]):
  root=E.fromstring(final[part]);pics=root.findall('.//p:pic',F.NS);assert not pics
  shapes=root.findall('.//p:sp',F.NS)
  named={s.find('p:nvSpPr/p:cNvPr',F.NS).get('name'):s for s in shapes}
  for nid,node in scene['nodes'].items():
   sp=named['node:'+nid];actual=''.join(t.text or '' for t in sp.findall('.//a:t',F.NS))
   assert actual==node['text'].replace('\n',''),(scene['page'],nid,actual)
  if scene['page']==29:qatext=[''.join(t.text or '' for t in tc.findall('.//a:t',F.NS)) for tc in root.findall('.//a:tc',F.NS)]
  note=F.related(final,part,'notesSlide');assert b'SYNTHETIC_AUTHORED_NOT_MEASURED' in final[note]
  native.append({'page':scene['page'],'native_shapes':len(shapes),'native_tables':len(root.findall('.//a:tbl',F.NS)),'pictures':len(pics),'shape_owned_node_labels':len(scene['nodes'])})
 summary=json.loads((repo/'docs/presentations_files/dp44-synthetic-backdata-20261022/summary.json').read_text())['summary']
 for option in ['A','B']:
  for key in ['qa01_display_s','qa01_voice_s','qa03_s','qa04_modules','qa05_usd','qa06_mb']:
   fmt=f"{summary[option][key]:.{6 if key=='qa05_usd' else 3}f}"
   assert any(fmt in t for t in qatext),(option,key,fmt)
 for t in ['120/120','80/80']:assert qatext.count(t)==2
 pixels=[]
 if preview_dir and source_preview_dir:
  from PIL import Image,ImageChops,ImageDraw
  for row in rows:
   a=Image.open(source_preview_dir/f"slide-{row['source_page']:02d}.png").convert('RGB')
   b=Image.open(preview_dir/f"slide-{row['final_page']:02d}.png").convert('RGB')
   assert a.size==b.size==(1920,1080)
   # Only the bottom-right footer rectangle is masked.
   for im in [a,b]:ImageDraw.Draw(im).rectangle((1770,1040,1919,1079),fill='white')
   bbox=ImageChops.difference(a,b).getbbox();assert bbox is None,(row,bbox)
   pixels.append({'source_page':row['source_page'],'final_page':row['final_page'],'exact_pixels_outside_footer':True})
 policy=W.policy_report(final)
 return {'status':'PASS','final_sha256':F.digest(final_path.read_bytes()),'source_sha256':F.digest(source_path.read_bytes()),'preserved_pages':rows,'native_new_pages':native,'summary_values_verified':12,'accuracy_count_cells_verified':4,'pixel_preservation':pixels,'windows_typography_policy':policy,'limit':'Package/Artifact Tool rendering verification only; no native PowerPoint app or VIA performance validation.'}
if __name__=='__main__':
 import argparse
 ap=argparse.ArgumentParser();ap.add_argument('repo',type=Path);ap.add_argument('source',type=Path);ap.add_argument('pptx',type=Path);ap.add_argument('scene',type=Path);ap.add_argument('--previews',type=Path);ap.add_argument('--source-previews',type=Path);ap.add_argument('--report',type=Path);a=ap.parse_args()
 result=check(a.repo,a.source,a.pptx,a.scene,a.previews,a.source_previews)
 if a.report:a.report.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
 print(json.dumps({'status':result['status'],'preserved':len(result['preserved_pages']),'new':len(result['native_new_pages']),'exact_pixel_comparisons':len(result['pixel_preservation'])}))
