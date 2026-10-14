// Run a copy in a private build directory with bundled node_modules linked there.
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {Presentation,PresentationFile,FileBlob} from '@oai/artifact-tool';
const [repoArg,buildArg]=process.argv.slice(2);
const repo=await fs.realpath(repoArg),build=await fs.realpath(buildArg);
const py=process.env.VIA_RUNTIME_PYTHON,skill=process.env.VIA_PRESENTATION_SKILL_DIR;
const scene=JSON.parse(await fs.readFile(path.join(build,'scene.json'),'utf8'));
const p=Presentation.create({slideSize:{width:1920,height:1080}}),slide=p.slides.add();
slide.background.fill='#FFFFFF';
const scale=.5;
function line(name,pts,color,width=1,arrow=false,dash=false){
  const ps=pts.map(([x,y])=>[x*scale,y*scale]);
  const x=Math.min(...ps.map(v=>v[0])),y=Math.min(...ps.map(v=>v[1]));
  const w=Math.max(1,Math.max(...ps.map(v=>v[0]))-x),h=Math.max(1,Math.max(...ps.map(v=>v[1]))-y);
  slide.shapes.add({name,geometry:'custom',position:{left:x,top:y,width:w,height:h},fill:'none',line:{fill:color,width,style:dash?'dashed':'solid'},customPaths:[{width:w,height:h,commands:ps.map(([a,b],i)=>({[i?'lineTo':'moveTo']:{x:a-x,y:b-y}}))}]});
  if(arrow){const [a,b]=pts.slice(-2),t=Math.atan2(b[1]-a[1],b[0]-a[0]),n=16;line(name+'-arrow',[[b[0]-n*Math.cos(t-.5),b[1]-n*Math.sin(t-.5)],b,[b[0]-n*Math.cos(t+.5),b[1]-n*Math.sin(t+.5)]],color,width);}
}
for(const n of scene.nodes){
  const group=['component','group'].includes(n.kind),store=n.kind==='store';
  const shape=slide.shapes.add({name:n.id,geometry:store?'can':n.kind==='code'?'roundRect':'rect',position:{left:n.x/2,top:n.y/2,width:n.w/2,height:n.h/2},fill:n.different?'#FFF3E5':'#FFFFFF',line:{fill:n.different?'#B8753F':'#263445',width:1},...(store?{adjustmentList:[{name:'adj',formula:'val 6500'}]}:{})});
  shape.text=[{runs:[{run:n.title,style:{bold:true,fontSize:group?'14pt':'12pt'}}],spaceAfter:group?0:8},...n.lines.map(t=>({runs:[{run:t,style:{fontSize:'12pt'}}],spaceAfter:2}))];
  shape.text.style={typeface:'Calibri',fontSize:16,color:'#000000',alignment:'left',verticalAlignment:'top',wrap:'none',autoFit:'none',insets:{left:9,right:7,top:group?1:store?11:8,bottom:0},lineSpacing:1.1};
  if(group)line(n.id+'-header',[[n.x,n.y+48],[n.x+n.w,n.y+48]],n.different?'#B8753F':'#263445');
  if(n.kind==='queue')for(let j=0;j<4;j++)slide.shapes.add({name:n.id+'-slot'+j,geometry:'rect',position:{left:(n.x+n.w-190+j*38)/2,top:(n.y+n.h-38)/2,width:15,height:15},fill:'#FFFFFF',line:{fill:'#B8753F',width:.7}});
}
for(const e of scene.edges)line(e.id,e.points,{post:'#195FBD',done:'#195FBD',call:'#263445',state:'#758294',cancel:'#B63732'}[e.kind],1,true,['done','state','cancel'].includes(e.kind));
for(const [i,n] of scene.notes.entries()){
  const fontPt=Math.round(n.size*scale*.75);
  const width=Math.min(3740,Math.max(...n.lines.map(t=>[...t].reduce((a,c)=>a+(c.charCodeAt(0)>255?1:.62),0)))*n.size+20);
  const x=n.align==='right'?n.x-width:n.align==='center'?n.x-width/2:n.x;
  const shape=slide.shapes.add({name:'annotation-'+i,geometry:'textbox',position:{left:x/2,top:(n.y-n.size)/2,width:width/2,height:(n.size*1.35*n.lines.length)/2},fill:n.mask?'#FFFFFF':'none',line:{fill:'none',width:0}});
  shape.text=n.lines.join('\n');shape.text.style={typeface:'Calibri',fontSize:fontPt*4/3,color:n.color==='#263445'?'#000000':n.color,bold:n.bold,alignment:n.align,verticalAlignment:'top',autoFit:'none',wrap:'none',insets:0};
}
slide.speakerNotes.textFrame.setText('DP-45 구현 구조 비교. A는 현재 read가 원본과 유효 cache를 조합한다. B는 독립 생산 job의 후보를 검증하여 관계와 Coverage를 게시하고 현재 read가 이를 조회한다. 미생산은 지원 범위 backfill, 게시 통지, 저장소 재조회를 거친다. S는 Request Controller의 입력 이력 read port이며 B의 Source Read Worker와 Origin Verifier도 같은 port를 사용한다. 원본의 실제 전달 범위, source 개정과 권한 검사는 양안 공통이다. M은 Model Access의 cloud 의미 모델 연동이며 B 관계 생산은 필요할 때만 호출한다. U는 현재 권한 검사와 사용 등록이다. 현재 의미 판단과 Request Controller의 채택은 근거 반환 뒤 공통으로 수행한다. 긴 작업은 bounded async executor를 사용하고 상자 수가 process 수는 아니다. A의 선행 index/summary와 관계 cache, 병렬 읽기 및 부분 갱신을 허용한다. B의 게시/갱신, 미사용 생산 및 복원 비용을 함께 비교한다. 미선정 설계안이며 제품 구현과 품질 측정 결과가 아니다.\n출처: docs/architecture/12-decisions/decision-packages/04-45-memory-and-context.md §3.10. 이전 상세 설계도와 비교 페이지는 보존한다.');
const draft=path.join(build,'draft.pptx');await(await PresentationFile.exportPptx(p)).save(draft);
execFileSync(py,['-c',`import sys;from pathlib import Path;sys.path.insert(0,sys.argv[1]+'/scripts/presentations');import full_presentation_package as F;import presentation_windows_policy as W;p=F.read(Path(sys.argv[2]));p={k:W.normalize_xml(k,v) if k.endswith('.xml') else v for k,v in p.items()};F.write(Path(sys.argv[3]),p);print(W.policy_report(p))`,repo,draft,path.join(build,'candidate.pptx')],{stdio:'inherit'});
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const final=path.join(build,'final','VIA-DP45-implementation-comparison.pptx');
await finalizePresentation({workspaceDir:build,candidatePath:path.join(build,'candidate.pptx'),finalPath:final,explicitTotalSlideCount:1,pythonExecutable:py,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit'],fontPolicy:{basis:'user_request',families:['Calibri']},verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
const again=await PresentationFile.importPptx(await FileBlob.load(final));
const png=await again.export({slide:again.slides.items[0],format:'png',scale:1.5});await fs.writeFile(path.join(build,'pptx-preview.png'),new Uint8Array(await png.arrayBuffer()));
await fs.writeFile(path.join(build,'layout.json'),await (await again.slides.items[0].export({format:'layout'})).text());
console.log(final);
