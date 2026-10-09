// Native DP46 deck from its editable source scenes; no 41–45 mutation.
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {Presentation,PresentationFile,FileBlob} from '@oai/artifact-tool';
const [repoArg,buildArg]=process.argv.slice(2);
if(!repoArg||!buildArg)throw new Error('Usage: generate_dp46_editable_pptx.mjs REPO BUILD_DIR');
const repo=await fs.realpath(repoArg),build=await fs.realpath(buildArg);
const skill=process.env.VIA_PRESENTATION_SKILL_DIR,python=process.env.VIA_RUNTIME_PYTHON;
if(![skill,python].every(p=>path.isAbsolute(p??'')))throw new Error('Set absolute presentation skill and Python runtime paths');
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const font='Apple SD Gothic Neo';
const extraction=String.raw`
import sys,json,hashlib
from pathlib import Path
r=Path(sys.argv[1]);sys.path.insert(0,str(r/'scripts/architecture'))
import generate_evidence_context_diagrams as g
sources=[g.background(),g.structure(),g.quality(),g.timeline(),g.lifecycle()]
result=[]
for s in sources:
 p=r/'docs/architecture/12-decisions/decision-packages/diagrams'/f'{s.slug}.svg'
 assert p.read_text()==s.render()['svg'],f'{s.slug} source out of sync'
 containers={n['owner'] for n in s.nodes if n['owner']};items=[]
 for n in s.nodes:
  x,y,w,h=(n[k] for k in ('x','y','w','h'));kind=n['kind']
  fill='#FCE4D6' if n['different'] else '#FFFFFF';stroke='#9B502B' if n['different'] else '#222222'
  sw=2.4 if n['different'] else 1.6
  top=n['id'] in containers or kind in ('component','boundary')
  size=25 if kind=='component' else 21
  items.append(dict(kind='store' if kind=='state' else 'note' if kind=='data' else 'rect',id=n['id'],x=x,y=y,w=w,h=85 if kind=='lifeline' else h,fill=fill,stroke=stroke,line_width=sw,rounded=kind=='module',external=kind=='external',dashed=kind=='boundary'))
  lines=n['name'].split('\n')
  if kind=='lifeline':
   items.append(dict(kind='line',id=n['id']+'-lifeline',points=[(x+w/2,y+85),(x+w/2,y+h)],color='#777777',width=1.6,dashed=True,arrow=False))
   tx,ty,tw,align,bold,sz=x+w/2,y+12,w-24,'center',True,25
  elif top:tx,ty,tw,align,bold,sz=x+15,y+8,w-30,'left',kind=='component',size
  else:tx,ty,tw,align,bold,sz=x+w/2,y+h/2-len(lines)*size*.65+(8 if kind=='state' else 0),w-24,'center',False,size
  items.append(dict(kind='text',id=n['id']+'-label',x=tx,y=ty,w=tw,lines=lines,size=sz,color='#111111',bold=bold,align=align,leading=sz*1.3))
 for i,e in enumerate(s.edges):
  items.append(dict(kind='line',id=f'edge-{i}',points=e['points'],color='#333333',width=1.6,dashed=e['dashed'],arrow=True))
  if e['both']:items.append(dict(kind='line',id=f'edge-{i}-return-head',points=list(reversed(e['points'])),color='#333333',width=1.6,dashed=e['dashed'],arrow=True))
 for i,t in enumerate(s.texts):
  if t.get('erase',True):items.append(dict(kind='rect',id=f'text-mask-{i}',x=t['x']-3,y=t['y'],w=t['w']+6,h=len(t['lines'])*t['size']*1.3,fill='#FFFFFF',stroke='none'))
  items.append(dict(kind='text',id=f'text-{i}',x=t['x'],y=t['y'],w=t['w'],lines=t['lines'],size=t['size'],color='#111111',bold=t['bold'],align='left',leading=t['size']*1.3))
 notes=s.title+'\nSource: '+str(p.relative_to(r))+'\n04-46-input-and-context-evidence.md: 설계 검토용/미선정/미구현/미측정. 원본과 Component 위치는 공통. A는 필요한 시간/과거 원본을 요청별 구성, B는 owner별 생산/게시된 관계를 조회한다. 시간 후보는 최종 지칭 정답이 아니다. 45 과거 선택과 46 temporal 선택의 교차 조합은 별도 명시한다. 로컬 VIA/VAD/권위, cloud 음성·의미 의존성. 모델 비용은 생산/갱신/미사용/실패/폐기도 포함. 입력final은 temporal게시완료를기다리는새경계가아니다.'
 result.append(dict(slug=s.slug,items=items,notes=notes,source=str(p.relative_to(r)),sha256=hashlib.sha256(p.read_bytes()).hexdigest()))
print(json.dumps(result,ensure_ascii=False))
`;
const scenes=JSON.parse(execFileSync(python,['-c',extraction,repo],{encoding:'utf8',maxBuffer:16*1024*1024}));
await fs.writeFile(path.join(build,'dp46-source-scenes.json'),JSON.stringify(scenes,null,2));
const stroke=(color,width=1.7,dashed=false)=>({fill:color==='none'?'none':color,width:color==='none'?0:width,style:dashed?'dashed':'solid'});
function polyline(slide,name,pts,color,width,dashed=false) {
  const xs=pts.map(p=>p[0]),ys=pts.map(p=>p[1]);
  const left=Math.min(...xs),top=Math.min(...ys),w=Math.max(.1,Math.max(...xs)-left),h=Math.max(.1,Math.max(...ys)-top);
  return slide.shapes.add({name,geometry:'custom',position:{left,top,width:w,height:h},fill:'none',line:stroke(color,width,dashed),
    customPaths:[{width:w,height:h,commands:pts.map(([x,y],i)=>({[i?'lineTo':'moveTo']:{x:x-left,y:y-top}}))}]});
}
function arrowhead(slide,name,pts,color,width) {
  const [ex,ey]=pts.at(-1);
  let previous=pts.length-2;
  while(previous>=0 && pts[previous][0]===ex && pts[previous][1]===ey) previous--;
  if(previous<0)return;
  const [px,py]=pts[previous],length=Math.hypot(ex-px,ey-py),u=(ex-px)/length,v=(ey-py)/length;
  const back=width*6.4,side=width*3.2;
  polyline(slide,name,[[ex-u*back-v*side,ey-v*back+u*side],[ex,ey],[ex-u*back+v*side,ey-v*back-u*side]],color,width*1.28);
}
function nativeScene(presentation,scene) {
  const slide=presentation.slides.add();slide.background.fill='#FFFFFF';
  for(const item of scene.items) {
    const name=`${scene.slug} / ${item.id}`;
    if(item.kind==='line') {
      polyline(slide,name+' / path',item.points,item.color,item.width,item.dashed);
      if(item.arrow)arrowhead(slide,name+' / arrow',item.points,item.color,item.width);
      continue;
    }
    if(item.kind==='text') {
      const left=item.x-(item.align==='center'?item.w/2:item.align==='right'?item.w:0);
      // Separate lines keep the exact authored line breaks and baseline spacing.
      for(const [index,content] of item.lines.entries()) {
        const shape=slide.shapes.add({name:name+` / text ${index+1}`,geometry:'textbox',
          position:{left,top:item.y+index*item.leading,width:item.w,height:item.size*1.45},fill:'none',line:stroke('none',0)});
        shape.text=content;
        shape.text.style={typeface:font,fontSize:item.size,bold:item.bold,color:item.color,
          alignment:item.align,verticalAlignment:'top',wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
      }
      continue;
    }
    const position={left:item.x,top:item.y,width:item.w,height:item.h};
    if(item.kind==='note') {
      const w=item.w,h=item.h;
      slide.shapes.add({name,geometry:'custom',position,fill:item.fill,line:stroke(item.stroke,item.line_width??1.7),
        customPaths:[{width:w,height:h,commands:[{moveTo:{x:0,y:0}},{lineTo:{x:w-18,y:0}},
          {lineTo:{x:w,y:18}},{lineTo:{x:w,y:h}},{lineTo:{x:0,y:h}},{close:{}}]}]});
      polyline(slide,name+' / fold',[[item.x+item.w-18,item.y],[item.x+item.w-18,item.y+18],[item.x+item.w,item.y+18]],item.stroke,item.line_width??1.7);
    } else if(item.kind==='store') {
      slide.shapes.add({name,geometry:'can',position,fill:item.fill,line:stroke(item.stroke,item.line_width??1.7),
        adjustmentList:[{name:'adj',formula:'val 20000'}]});
    } else if(item.kind==='rect') {
      slide.shapes.add({name,geometry:item.flow_decision?'diamond':item.external?'hexagon':item.rounded?'roundRect':'rect',position,fill:item.fill,line:stroke(item.stroke,item.line_width??1.7,item.dashed)});
    } else throw new Error(`Unsupported scene object: ${item.kind}`);
  }
  slide.speakerNotes.textFrame.setText(scene.notes);
  return slide;
}
const p=Presentation.create({slideSize:{width:2560,height:1440}});
for(const s of scenes)nativeScene(p,s);
const slug='VIA-DP46-evidence',candidate=path.join(build,slug+'.candidate.pptx'),final=path.join(build,'final',slug+'.pptx');
await fs.mkdir(path.dirname(final),{recursive:true});await(await PresentationFile.exportPptx(p)).save(candidate);
await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:final,explicitTotalSlideCount:scenes.length,
 pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
 layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),layoutArgs:['--expected-slide-size-emu','24384000,13716000','--validate-heading-fit'],
 requiredNativeTableOwnerSlides:[],fontPolicy:{basis:'design',families:[font]},verifyArtifactToolImport:true,receiptPath:path.join(build,'dp46-validation.json')});
const imported=await PresentationFile.importPptx(await FileBlob.load(final));
for(const [i,s] of imported.slides.items.entries()){
 const png=await imported.export({slide:s,format:'png',scale:.75});await fs.writeFile(path.join(build,`${i+1}-${scenes[i].slug}.png`),new Uint8Array(await png.arrayBuffer()));
}
const out=path.join(repo,'docs/presentations_files/dp46-evidence');await fs.mkdir(out,{recursive:true});
await fs.copyFile(final,path.join(out,slug+'.pptx'));
await fs.writeFile(path.join(out,'native-source-manifest.json'),JSON.stringify({status:'DOCUMENTATION_NOT_IMPLEMENTED_NOT_MEASURED',slides:scenes.map(s=>({slug:s.slug,source:s.source,svg_sha256:s.sha256})),format:'5 editable slides: background, MAIN, qualitative QA, timeline, lifecycle'},null,2)+'\n');
console.log('PASS: 46 five native slides, imported renders and source manifest');
