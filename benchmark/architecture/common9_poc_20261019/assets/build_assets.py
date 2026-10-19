"""Render authored common9 source; native metadata exposes ONLY actual current selection.
No evaluator expected target or binding ID is read or emitted.
"""
from pathlib import Path
import json,html,hashlib,subprocess
BASE=Path(__file__).resolve().parent
NODE='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node';SHARP='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp'
ROWS=[('T1','비용 표',['Cedar 월 96달러','Birch 월 64달러']),('T2','일정 표',['시작 10월 20일','완료 10월 30일']),('T3','자원 표',['Cedar RAM 640MB','Birch RAM 384MB']),('P1','핵심 설명',['준비된 파일은 10개이며','결론을 먼저 제시한다.']),('P2','수정 대상',['이 문단은 두 문장으로','명확하게 수정해야 한다.'])]
BOXES={'T1':[45,95,465,340],'T2':[510,95,930,340],'T3':[975,95,1395,340],'P1':[45,390,1395,565],'P2':[45,610,1395,800]}
def text(x,y,s,size=23):return f'<text x="{x}" y="{y}" font-size="{size}" fill="#14243c">{html.escape(s)}</text>'
def render(name,selected,rev=1):
 b=text(45,49,f'DOC-A / revision {rev} / current document',28)
 for item,title,lines in ROWS:
  x1,y1,x2,y2=BOXES[item];fill='#e2efff'if item in selected else'#fff';stroke='#1672cf'if item in selected else'#8794a4';width=4 if item in selected else 2
  b+=f'<rect x="{x1}" y="{y1}" width="{x2-x1}" height="{y2-y1}" rx="6" fill="{fill}" stroke="{stroke}" stroke-width="{width}"/>'+text(x1+18,y1+37,item+' / '+title,24)
  for i,line in enumerate(lines):
   if rev==2 and item=='P1':line=line.replace('10개','12개')
   b+=text(x1+18,y1+85+i*39,line,21)
  if item in selected:b+=text(x2-96,y2-18,'선택됨',17)
 p=BASE/(name+'.svg');p.write_text(f'<svg xmlns="http://www.w3.org/2000/svg" width="2880" height="1800" viewBox="0 0 1440 900"><rect width="1440" height="900" fill="#edf2f7"/><g font-family="Apple SD Gothic Neo,Noto Sans CJK KR,Arial,sans-serif">{b}</g></svg>');subprocess.run([NODE,'-e','require(process.argv[1])(process.argv[2]).png().toFile(process.argv[3])',SHARP,str(p),str(p.with_suffix('.png'))],check=True)
 frame={'frame_id':name,'source_id':'DOC-A','source_revision':rev,'width':2880,'height':1800,'coordinate_space':'frame_pixels','image_path':str(p.with_suffix('.png')),'sha256':hashlib.sha256(p.with_suffix('.png').read_bytes()).hexdigest(),'evidence_kind':'AUTHORED_RENDERED_SCREEN_NOT_CAPTURED_WINDOWS_APP','native_selection':{'status':'available','capability':'selected_object_ranges_only','provider':'AUTHORED_SELECTION_ADAPTER_OF_RENDERED_SOURCE','complete_scene_objects':False,'items':[{'item_id':i,'source_id':'DOC-A','source_revision':rev,'bbox':[2*x for x in BOXES[i]],'coordinate_space':'frame_pixels','source_line_range':[1+next(k for k,r in enumerate(ROWS)if r[0]==i)]*2}for i in selected]}}
 return frame
s08=render('s08-doc-a-v1',['T1','T2']);s15=render('s15-doc-a-v1',['P1']);render('doc-a-v2-unselected',[],2)
def point(id,frame,x,y,t):return {'event_id':id,'type':'pointer_move','frame_id':frame,'x':2*x,'y':2*y,'t_ms':t,'button':'none','provenance':'AUTHORED_RAW_POINTER_EVENT_NOT_INFERRED_TARGET'}
selection08={'event_id':'native-selection','type':'native_selection','frame_id':s08['frame_id'],'t_ms':0,'item_ids':['T1','T2'],'observed_in':'frame.native_selection'}
selection15={'event_id':'native-selection','type':'native_selection','frame_id':s15['frame_id'],'t_ms':0,'item_ids':['P1'],'observed_in':'frame.native_selection'}
manifest={'scope':'Public source observations only; canonical QA binding oracle remains outside','events':{'S08-U01':{'frames':[s08],'cues':[selection08,point('point-early',s08['frame_id'],240,475,4500),point('point-later',s08['frame_id'],1040,700,7600)]},'S08-U02':{'frames':[s08],'cues':[selection08,point('point-current',s08['frame_id'],1185,225,3500)]},'S15-U01':{'frames':[s15],'cues':[selection15]}}}
(BASE/'observations.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'public_events':list(manifest['events']),'asset_pngs':3,'no_oracle_read':True}))
