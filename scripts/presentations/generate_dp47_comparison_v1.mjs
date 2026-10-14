import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {Presentation, PresentationFile, FileBlob} from '@oai/artifact-tool';

// Copy into a private build directory linked to the bundled node_modules.
const [repoArg,buildArg]=process.argv.slice(2);
const repo=await fs.realpath(repoArg),build=await fs.realpath(buildArg);
const skill=process.env.VIA_PRESENTATION_SKILL_DIR,python=process.env.VIA_RUNTIME_PYTHON;
if(![skill,python].every(v=>path.isAbsolute(v??'')))throw Error('Bundled skill and Python paths required');
const src=path.join(repo,'docs/architecture/12-decisions/decision-packages/diagrams/dp47-implementation-comparison-v1.json');
const scene=JSON.parse(await fs.readFile(src,'utf8'));
const render=JSON.parse(await fs.readFile(src.replace('.json','-render.json'),'utf8'));
const presentation=Presentation.create({slideSize:{width:1920,height:1080}});
const slide=presentation.slides.add();slide.background.fill='#FFFFFF';
const scale=.5;
function poly(name,points,color,width=1,arrow=false,dashed=false){
  const pts=points.map(([x,y])=>[x*scale,y*scale]);
  const x=Math.min(...pts.map(p=>p[0])),y=Math.min(...pts.map(p=>p[1]));
  const w=Math.max(1,Math.max(...pts.map(p=>p[0]))-x),h=Math.max(1,Math.max(...pts.map(p=>p[1]))-y);
  slide.shapes.add({name,geometry:'custom',position:{left:x,top:y,width:w,height:h},fill:'none',
    line:{fill:color,width,style:dashed?'dashed':'solid'},
    customPaths:[{width:w,height:h,commands:pts.map(([px,py],i)=>({[i?'lineTo':'moveTo']:{x:px-x,y:py-y}}))}]});
  if(arrow){const [a,b]=points.slice(-2),theta=Math.atan2(b[1]-a[1],b[0]-a[0]),n=15;
    poly(name+'-arrow',[[b[0]-n*Math.cos(theta-.5),b[1]-n*Math.sin(theta-.5)],b,[b[0]-n*Math.cos(theta+.5),b[1]-n*Math.sin(theta+.5)]],color,width);}
}
for(const n of scene.nodes){
  const container=['component','group'].includes(n.kind);
  const shape=slide.shapes.add({name:n.id,geometry:n.kind==='db'?'can':n.kind==='data'?'foldedCorner':n.kind==='code'?'roundRect':'rect',
    position:{left:n.x*scale,top:n.y*scale,width:n.w*scale,height:n.h*scale},
    fill:n.different?'#F0F6F5':'#FFFFFF',line:{fill:n.different?'#238176':'#111111',width:1},...(n.kind==='code'?{borderRadius:4}:{})});
  shape.text.style={typeface:'Calibri',fontSize:16,color:'#000000',alignment:'left',verticalAlignment:'top',
    wrap:'none',autoFit:'none',lineSpacing:1.05,insets:{left:11,right:11,top:n.kind==='db'?14:6,bottom:0}};
  const titles=n.title.split('\n').map(run=>({runs:[{run,style:{bold:true,fontSize:'14pt',typeface:'Calibri',color:'#000000'}}],paragraphStyle:{alignment:container?'left':'center'},spaceAfter:0}));
  const body=n.lines.map(run=>({runs:[{run,style:{fontSize:'12pt',typeface:'Calibri',color:'#000000'}}],spaceAfter:0}));
  shape.text.set([...titles,...body]);
  if(n.kind==='queue')for(let j=0;j<4;j++)slide.shapes.add({name:n.id+'-slot-'+j,geometry:'rect',
    position:{left:(n.x+n.w-190+j*38)*scale,top:(n.y+n.h-38)*scale,width:15,height:15},
    fill:'#FFFFFF',line:{fill:n.different?'#238176':'#777777',width:.5}});
  if(n.kind==='memory')poly(n.id+'-stack',[[n.x+8,n.y+n.h+6],[n.x+n.w+6,n.y+n.h+6],[n.x+n.w+6,n.y+8]],n.different?'#238176':'#777777');
}
for(const e of scene.edges){const color={post:'#195FBD',done:'#195FBD',cancel:'#B63732',state:'#777777',call:'#111111',rpc:'#111111'}[e.kind];
  poly(e.id,e.points,color,1,true,['done','cancel','state'].includes(e.kind));}
for(const [i,n] of scene.notes.entries()){
  const sizePt=Math.round(n.size*scale*3/4),fontSize=sizePt*4/3;
  const shape=slide.shapes.add({name:'annotation-'+i,geometry:'textbox',
    position:{left:n.x*scale,top:(n.y-n.size)*scale,width:Math.min((3840-n.x-20)*scale,render.note_bounds[i].width*scale+14),height:fontSize*1.3*n.lines.length},
    fill:'#FFFFFF',line:{fill:'none',width:0}});
  shape.text=n.lines.join('\n');shape.text.style={typeface:'Calibri',fontSize,color:n.color,bold:n.bold,
    alignment:n.align,verticalAlignment:'top',wrap:'none',autoFit:'none',insets:0};
}
slide.speakerNotes.textFrame.setText(
  'DP-47의 기존 MAIN과 상세도는 보존하고 별도로 만든 구현 구조 비교 페이지다. '+
  'A는 raw evidence에서 target과 binding을 직접 생산하는 async workflow다. '+
  'B는 producer가 게시한 candidate revision을 필수 입력으로 결합하는 orchestrated dataflow와 Fork–Join/feedback이다. '+
  'R/M은 같은 adapter를 호출하는 표기이며 원본이나 모델을 역할별로 복제하지 않는다. '+
  '모든 queue는 local bounded RAM FIFO이고 모듈은 같은 process의 async task라는 구현 가정이다. '+
  'B의 결합 준비 조건은 호환 snapshot, 필요한 생산 경로의 응답 또는 deadline, 남은 예산이다. 충분한 의미인지는 Resolver가 판단한다. '+
  '양안의 게시 API는 BeginRevision과 같은 RevisionGate에서 현재성을 확인한다. 최종 채택은 Request Controller가 수행한다. '+
  '작업 queue 포화, 사용권과 원본 수명은 상세 문서 계약을 따른다. 이 페이지는 구현 결과나 측정상 우위를 보고하지 않는다.\n\n'+
  '출처: docs/architecture/12-decisions/decision-packages/04-47-screen-voice-grounding.md §8.5; dp47-structure-references.md §2.3; dp47-design-review.md §9.\n'+
  'Reference 연결은 VIA 설계 추론이다. QuickSet https://aclanthology.org/P97-1036.pdf ; Gemini Live https://github.com/google-gemini/live-api-web-console ; Microsoft OmniParser https://github.com/microsoft/OmniParser ; Channels https://learn.microsoft.com/en-us/dotnet/core/extensions/channels'
);
await(await PresentationFile.exportPptx(presentation)).save(path.join(build,'authored.pptx'));
// Apply the repository's existing language/font policy to the native package.
execFileSync(python,['-c',
  'import sys,zipfile; from pathlib import Path; sys.path.insert(0,sys.argv[1]); from presentation_windows_policy import normalize_xml; '+
  'a=zipfile.ZipFile(sys.argv[2]); b=zipfile.ZipFile(sys.argv[3],"w",zipfile.ZIP_DEFLATED); '+
  '[b.writestr(n,normalize_xml(n,a.read(n)) if n.endswith(".xml") else a.read(n)) for n in a.namelist()]; b.close(); a.close()',
  path.join(repo,'scripts/presentations'),path.join(build,'authored.pptx'),path.join(build,'candidate.pptx')],{stdio:'inherit'});
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const final=path.join(build,'final','VIA-DP47-implementation-comparison_v1.pptx');await fs.mkdir(path.dirname(final),{recursive:true});
await finalizePresentation({workspaceDir:build,candidatePath:path.join(build,'candidate.pptx'),finalPath:final,explicitTotalSlideCount:1,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit'],
  fontPolicy:{basis:'user_request',families:['Calibri']},verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
const imported=await PresentationFile.importPptx(await FileBlob.load(final));
const png=await imported.export({slide:imported.slides.items[0],format:'png',scale:2});
await fs.writeFile(path.join(build,'final-slide.png'),new Uint8Array(await png.arrayBuffer()));
await fs.writeFile(path.join(build,'final-layout.json'),await(await imported.slides.items[0].export({format:'layout'})).text());
console.log(final);
