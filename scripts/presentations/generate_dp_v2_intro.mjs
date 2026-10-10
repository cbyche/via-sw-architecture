import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {Presentation, PresentationFile, FileBlob} from '@oai/artifact-tool';

// Run a copy from a private build directory linked to the bundled node_modules.
// Only the two v2 user-experience pages are authored. Other slides are grafted
// from the current v2 package byte-for-byte by dp_v2_intro_package.py.
const [repoArg, buildArg] = process.argv.slice(2);
if (!repoArg || !buildArg) throw new Error('Usage: generate_dp_v2_intro.mjs REPO BUILD');
const repo = await fs.realpath(repoArg), build = await fs.realpath(buildArg);
const skill = process.env.VIA_PRESENTATION_SKILL_DIR;
const python = process.env.VIA_RUNTIME_PYTHON;
if (![skill, python].every(p => path.isAbsolute(p ?? ''))) throw new Error('Set skill and bundled Python paths');
const {finalizePresentation} = await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const presentation = Presentation.create({slideSize:{width:1920,height:1080}});
const font = 'Apple SD Gothic Neo';
const C = {ink:'#202B30', muted:'#66777D', teal:'#397C80', apricot:'#A56237', guide:'#CDD7DA'};
const assetDir = path.join(repo,'docs/presentations_files/dp-background-v2/assets');
const sceneNames = ['via-v2-responsibilities','via-v2-overlapping-events'];

function text(s,name,content,x,y,w,size=30,color=C.ink,bold=false,align='left') {
  const lines=content.split('\n');
  for(const [i,line] of lines.entries()) {
    const shape=s.shapes.add({name:`${s._scene} / ${name} / ${i+1}`,geometry:'textbox',
      position:{left:x,top:y+i*size*1.25,width:w,height:size*1.35},fill:'none',line:{fill:'none',width:0}});
    shape.text=line;
    shape.text.style={typeface:font,fontSize:size,bold,color,alignment:align,
      verticalAlignment:'top',wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
  }
}
function line(s,name,points,color=C.guide,width=1.6,arrow=false,dashed=false) {
  const xs=points.map(p=>p[0]),ys=points.map(p=>p[1]);
  const left=Math.min(...xs),top=Math.min(...ys),w=Math.max(.1,Math.max(...xs)-left),h=Math.max(.1,Math.max(...ys)-top);
  s.shapes.add({name:`${s._scene} / ${name}`,geometry:'custom',position:{left,top,width:w,height:h},fill:'none',
    line:{fill:color,width,style:dashed?'dashed':'solid'},customPaths:[{width:w,height:h,
      commands:points.map(([x,y],i)=>({[i?'lineTo':'moveTo']:{x:x-left,y:y-top}}))}]});
  if(arrow) {
    const [x,y]=points.at(-1),[px,py]=points.at(-2),len=Math.hypot(x-px,y-py),u=(x-px)/len,v=(y-py)/len;
    line(s,name+' / arrow',[[x-u*14-v*7,y-v*14+u*7],[x,y],[x-u*14+v*7,y-v*14-u*7]],color,width);
  }
}
function message(s,name,from,to,y,label,color=C.teal,size=29) {
  line(s,name,[[from,y],[to,y]],color,2.1,true);
  text(s,name+' label',label,Math.min(from,to)+25,y-47,Math.abs(to-from)-50,size,color,false,'center');
}
async function art(s,file,y,h) {
  const second=s._scene===sceneNames[1];
  const widths=second?[470,495,520]:[425,500,355];
  const crops=second?[[0,.15,.67,.22],[.36,.25,.36,.24],[.65,.15,.01,.27]]:
    [[0,.15,.70,.19],[.35,.25,.34,.17],[.735,.15,.005,.17]];
  const bytes=await fs.readFile(path.join(assetDir,file));
  for(const [i,center] of [330,965,1590].entries()) {
    const [left,top,right,bottom]=crops[i];
    // Unique subjects from an imagegen illustration sheet, each framed as a
    // native image. No raster pixels are edited or decorative shapes drawn.
    s.images.add({name:`${s._scene} / illustration ${i+1}`,blob:bytes,contentType:'image/png',
      alt:`AI 생성 설명용 삽화의 ${['사용자','VIA','업무 수행 Agent'][i]} 부분`,
      fit:'cover',crop:{left,top,right,bottom},position:{left:center-widths[i]/2,top:y,width:widths[i],height:h}});
  }
}
function scene(slug) {const s=presentation.slides.add();s._scene=slug;s.background.fill='#FFFFFF';return s;}
function headers(s,y) {
  text(s,'user','사용자',85,y,490,35,C.ink,true,'center');
  text(s,'via','VIA',720,y,490,35,C.teal,true,'center');
  text(s,'agent','업무 수행 Agent',1345,y,490,35,C.ink,true,'center');
}
const s1=scene(sceneNames[0]);
text(s1,'title','사용자의 요청을 이해해 업무를 위임하고,\n입력부터 응답까지 사용자와의 대화를 관리하는 VIA',70,52,1790,49,C.ink,true);
text(s1,'request','“이 표와 지난달 매출보고서를 바탕으로\n이번 달 매출보고서를 만들어줘.”',95,206,1740,37,C.teal,true);
await art(s1,'via-responsibilities-illustration.png',308,310);
headers(s1,628);
for(const x of [330,965,1590]) line(s1,'participant '+x,[[x,680],[x,980]],C.guide,1.4,false,true);
message(s1,'initial-input',330,965,725,'음성 요청 + 선택한 표');
message(s1,'delegate',965,1590,725,'목표·자료·조건을 이해해 위임',C.teal,27);
message(s1,'agent-question',1590,965,805,'강조할 항목은 무엇인가요?',C.apricot,28);
message(s1,'ask-user',965,330,805,'질문을 사용자에게 전달',C.apricot,28);
message(s1,'user-answer',330,965,885,'“국내 매출을 강조해줘.”',C.teal,28);
message(s1,'relay-condition',965,1590,885,'답변을 해당 업무에 연결',C.teal,28);
message(s1,'report-result',1590,965,965,'완성된 보고서',C.apricot,28);
message(s1,'present-result',965,330,965,'화면에 결과, 음성으로 핵심 안내',C.apricot,26);
text(s1,'benefit','사용자는 VIA에서 대화를 이어가고, Agent는 실제 보고서 작성을 수행합니다',95,1004,1730,30,C.ink,true,'center');
s1.speakerNotes.textFrame.setText(`사용자 경험 1: VIA의 책임\n화면에서 선택한 이번 달 매출 표와 접근이 허용된 지난달 보고서를 사용한다. VIA는 요청·대상·조건을 이해하여 업무 수행 Agent에 위임한다. Agent가 업무 수행 중 확인 질문을 보내면 VIA가 사용자에게 전달하고 답변을 원래 업무에 연결한다. 추가 조건 접수가 실제 Agent 적용 완료를 뜻하지 않으며 이 그림은 지연·실패 없는 설명용 장면이다. 보고서 작성·도메인 분석은 Downstream Agent 책임이다. 모든 사용자 응답은 Text로 남고 Voice 활성화 시 핵심을 음성으로 전달한다. 실선은 설명용 메시지, 세로 점선은 참여자의 지속 역할이며 Component/process 경계가 아니다.\n근거: docs/architecture/01-system-mission-and-boundary.md §1.1~1.2, docs/architecture/05-representative-use-cases.md UC-03/05/06/08 및 §5.2. 사용자 2026-10-10 발표 리뷰.\n삽화: builtin imagegen 생성 설명용 이미지이며 실제 제품 화면·측정 결과가 아니다.`);

const s2=scene(sceneNames[1]);
text(s2,'title','업무를 맡긴 뒤에도,\n새 발화와 업무 결과가 같은 대화로 들어옵니다',70,52,1790,50,C.ink,true);
text(s2,'request','“이 표와 지난달 보고서로 이번 달 매출보고서를 만들고,\n내일 오후 3시 팀 회의 안내 메일 초안도 준비해줘.”',95,206,1740,35,C.teal,true);
await art(s2,'via-overlapping-events-illustration.png',308,310);
headers(s2,628);
for(const x of [330,965,1590]) line(s2,'participant '+x,[[x,680],[x,980]],C.guide,1.4,false,true);
message(s2,'compound-input',330,965,725,'한 발화에 서로 독립적인 두 업무',C.teal,27);
message(s2,'two-delegations',965,1590,725,'보고서 작성 / 회의 안내 메일 초안',C.teal,26);
text(s2,'ongoing-tasks','보고서·메일 처리 중',1282,744,570,27,C.muted,false,'center');
message(s2,'interruption',330,965,835,'“잠깐, 보고서엔 국내 매출만 넣어줘.”',C.teal,26);
message(s2,'old-result',1590,965,835,'이전에 맡긴 공급업체 견적 도착',C.apricot,26);
text(s2,'coincidence','새 발화와 결과가 겹쳐 도착',742,861,445,25,C.muted,true,'center');
message(s2,'report-correction',965,1590,965,'정정은 보고서 업무에 연결',C.teal,27);
message(s2,'quote-delivery',965,330,965,'말을 마친 뒤, 견적 도착을 안내',C.apricot,26);
text(s2,'benefit','사용자는 Agent별 창을 오가거나, 진행 중인 업무를 다시 설명할 필요가 없습니다',95,1004,1730,30,C.ink,true,'center');
s2.speakerNotes.textFrame.setText(`사용자 경험 2: 계속되는 입력과 비동기 업무 결과\n보고서 작성과 내일 오후 3시 팀 회의 안내 메일 초안은 서로 독립적인 두 업무이며 보고서 결과에 의존하는 메일이 아니다. 메일 초안만 요청하며 발송 권한을 주지 않는다. 공급업체 견적 업무는 이 발화 전에 이미 맡긴 별도 업무다.\nVIA가 두 업무의 위임을 진행하는 동안 사용자가 끼어들어 보고서만 국내 매출로 정정하고, 견적 Agent의 결과도 겹쳐 들어오는 설명용 장면이다. 발화 시작 시 음성 재생은 즉시 중단하고 계속 듣는다. 발화 중 견적 음성 알림을 재생하지 않는다. 견적 결과 수신을 결과 전달 완료로 간주하지 않는다. 국내 매출 정정은 보고서 업무에만 연결하고 회의 메일과 견적 업무에는 적용하지 않는다. 마지막 두 화살표는 해야 하는 연결을 표현하며 실제 병렬 수행·반환 순서 또는 Agent 적용 완료를 확정하지 않는다. 이 페이지는 구조 대안·우선순위 정책·성능 결과를 선택하지 않는다.\n이 장면은 요청과 관계의 이해, 계속되는 실행 연결, 대화·업무 맥락의 유지, 서로 다른 업무 상태 추적이 필요한 이유를 소개한다. DP 번호와 세부 상태 schema는 다음 비교 페이지에서 다룬다.\n근거: docs/architecture/05-representative-use-cases.md UC-09/10/11/13/14 및 §5.2. 사용자 2026-10-10 발표 리뷰.\n삽화: builtin imagegen 생성 설명용 이미지이며 실제 제품 화면·측정 결과가 아니다.`);

await fs.mkdir(path.join(build,'final'),{recursive:true});
const draft=path.join(build,'intro-candidate.pptx');
await (await PresentationFile.exportPptx(presentation)).save(draft);
const dest=path.join(repo,'docs/presentations_files/VIA-DP-background-and-comparison-41-45_v2.pptx');
const original=path.join(repo,'docs/presentations_files/VIA-DP-background-and-comparison-41-45.pptx');
let source=dest;try{await fs.access(source)}catch{source=original}
execFileSync(python,[path.join(repo,'scripts/presentations/dp_v2_intro_package.py'),draft,source,path.join(build,'preservation.json')]);
const final=path.join(build,'final','VIA-DP-background-and-comparison-41-45_v2.pptx');
await finalizePresentation({workspaceDir:build,candidatePath:draft,finalPath:final,explicitTotalSlideCount:12,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-heading-fit'],
  requiredNativeTableOwnerSlides:[],fontPolicy:{basis:'design',families:[font]},verifyArtifactToolImport:true,
  receiptPath:path.join(build,'validation.json')});
const imported=await PresentationFile.importPptx(await FileBlob.load(final));
await fs.writeFile(path.join(build,'inspection.ndjson'),(await imported.inspect({kind:'slide,textbox,image,layout',maxChars:12000})).ndjson);
for(let i=0;i<2;i++) {
  const png=await imported.export({slide:imported.slides.items[i],format:'png',scale:1});
  await fs.writeFile(path.join(build,`intro-${i+1}.png`),new Uint8Array(await png.arrayBuffer()));
}
await fs.copyFile(final,dest);
console.log(JSON.stringify({deck:dest,pages:12,preview:[path.join(build,'intro-1.png'),path.join(build,'intro-2.png')]}));
