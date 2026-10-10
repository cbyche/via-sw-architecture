import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {Presentation, PresentationFile, FileBlob} from '@oai/artifact-tool';

// Run a copy in a fresh private build directory linked to bundled node_modules.
// Select DP(s) explicitly; the scoped package graft preserves every other part.
const [repoArg,buildArg,selection='41,44,45,42']=process.argv.slice(2);
if(!repoArg||!buildArg) throw new Error('Usage: generate_dp_v2_backgrounds.mjs REPO BUILD [41,44,45,42]');
const repo=await fs.realpath(repoArg),build=await fs.realpath(buildArg);
const selected=selection.split(',').map(Number);
if(!selected.length||new Set(selected).size!==selected.length||selected.some(n=>![41,44,45,42].includes(n))) throw new Error('Invalid DP selection');
const skill=process.env.VIA_PRESENTATION_SKILL_DIR,python=process.env.VIA_RUNTIME_PYTHON;
if(![skill,python].every(p=>path.isAbsolute(p??''))) throw new Error('Set skill and bundled Python paths');
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const presentation=Presentation.create({slideSize:{width:1920,height:1080}});
const font='Apple SD Gothic Neo';
const C={ink:'#202B30',muted:'#66777D',teal:'#397C80',apricot:'#A56237',guide:'#CDD7DA'};
const manifest=[];
// Jury-approved background endings describe the SW challenge, not A/B options.
const challenges={
  41:'발화만으로 알 수 없는 정보를 찾아 확인하고,\n조회 결과에 따라 추가 조회와 해석을 이어가며 Request를 완성하는 구조',
  44:'사용자 입력과 Task 알림, 내부 처리 완료가 서로 다른 시점에 도착해도,\n입력을 계속 받으며 후속 처리와 대기, 응답 전달을 이어가는 실행 구조',
  45:'과거 대화와 여러 Task에 흩어진 정보 중 현재 Request에 필요한 내용을 찾아,\n정정 이력과 실제 전달 내용을 반영해 제공하는 맥락 관리 구조',
  42:'대화 연결과 Request 처리가 끝나도 Task의 진행 상태와 결과 기록을 유지하고,\n서로 다른 수명을 가진 상태와 연결 관계를 일관되게 관리하는 구조',
};
function visibleCopy(content) {
  return content.replaceAll('업무','Task')
    .replaceAll('요청을','Request를').replaceAll('요청은','Request는')
    .replaceAll('요청이','Request가').replaceAll('요청과','Request와')
    .replaceAll('요청으로','Request로').replaceAll('요청','Request')
    .replaceAll('근거','정보').replace(/(?<!지칭 )대상/g,'지칭 대상')
    .replace(/\s*·\s*/g,', ');
}
function text(s,name,content,x,y,w,size=30,color=C.ink,bold=false,align='left') {
  for(const [i,t] of visibleCopy(content).split('\n').entries()) {
    const shape=s.shapes.add({name:`${s._scene} / ${name} / ${i+1}`,geometry:'textbox',
      position:{left:x,top:y+i*size*1.25,width:w,height:size*1.35},fill:'none',line:{fill:'none',width:0}});
    shape.text=t;shape.text.style={typeface:font,fontSize:size,bold,color,alignment:align,
      verticalAlignment:'top',wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
  }
}
function line(s,name,points,color=C.guide,width=1.6,arrow=false,dashed=false) {
  const xs=points.map(p=>p[0]),ys=points.map(p=>p[1]),left=Math.min(...xs),top=Math.min(...ys);
  const w=Math.max(.1,Math.max(...xs)-left),h=Math.max(.1,Math.max(...ys)-top);
  s.shapes.add({name:`${s._scene} / ${name}`,geometry:'custom',position:{left,top,width:w,height:h},fill:'none',
    line:{fill:color,width,style:dashed?'dashed':'solid'},customPaths:[{width:w,height:h,
      commands:points.map(([x,y],i)=>({[i?'lineTo':'moveTo']:{x:x-left,y:y-top}}))}]});
  if(arrow) {
    const [x,y]=points.at(-1),[px,py]=points.at(-2),len=Math.hypot(x-px,y-py),u=(x-px)/len,v=(y-py)/len;
    line(s,name+' / arrow',[[x-u*14-v*7,y-v*14+u*7],[x,y],[x-u*14+v*7,y-v*14-u*7]],color,width);
  }
}
function scene(dp,title,quote,quoteSize=35) {
  const s=presentation.slides.add();s._scene=`dp${dp}-background`;s.background.fill='#FFFFFF';
  text(s,'title',title,70,52,1780,50,C.ink,true);
  text(s,'request',quote,95,206,1740,quoteSize,C.teal,true);
  return s;
}
async function art(s,dp,crops,widths,y=312,h=272) {
  const bytes=await fs.readFile(path.join(repo,`docs/presentations_files/dp-background-v2/assets/dp${dp}-background-illustration.png`));
  for(const [i,center] of [330,965,1590].entries()) {
    const [left,top,right,bottom]=crops[i].map(n=>n/100000);
    s.images.add({name:`${s._scene} / illustration ${i+1}`,blob:bytes,contentType:'image/png',
      alt:`DP${dp} 설명용 AI 삽화 — 장면 ${i+1}`,fit:'cover',crop:{left,top,right,bottom},
      position:{left:center-widths[i]/2,top:y,width:widths[i],height:h}});
  }
  manifest.push({dp,slug:s._scene,crops});
}
function headings(s,labels,y=602) {
  for(const [i,c] of [330,965,1590].entries())text(s,'participant '+i,labels[i],c-275,y,550,34,i===1?C.teal:C.ink,true,'center');
}
function column(s,name,i,content,y=692,size=29) {
  text(s,name,content,[330,965,1590][i]-275,y,550,size,C.ink,false,'center');
}
function handoff(s) {
  line(s,'evidence-to-via',[[590,665],[705,665]],C.teal,2,true);
  line(s,'via-to-agent',[[1225,665],[1340,665]],C.teal,2,true);
}
function close(s,benefit,challenge,benefitY=868,challengeY=961) {
  text(s,'user-benefit',benefit,95,benefitY,1730,29,C.ink,true,'center');
  line(s,'challenge-divider',[[95,challengeY-22],[1825,challengeY-22]],C.guide,1.3);
  text(s,'design-challenge',challenge,95,challengeY,1730,32,C.teal,true,'center');
}
const commonNotes=`삽화는 builtin imagegen 생성 설명용 이미지이며 실제 제품 UI나 측정 결과가 아니다. 사용자/VIA/Agent 또는 사건/시간 구간은 설명용 참여자·데이터이며 Component·Module·process 경계를 선택하지 않는다. 제목·문구·데이터 흐름·타임라인은 native editable objects다. A/B는 미선정·미측정이며 이 배경은 양안의 공통 요구를 설명한다. Architecture 문서의 SVG/draw.io와 기존 원본 PPTX는 수정하지 않는다. 근거: 사용자 2026-10-10 발표자료 리뷰, docs/architecture/01-system-mission-and-boundary.md 및 docs/architecture/05-representative-use-cases.md. `;

for(const dp of selected) {
  if(dp===41) {
    const s=scene(dp,'말 속의 대상과 조건을 찾아,\n위임할 요청을 완성하는 VIA',
      '“이 표를 아까 보고서에 넣고,\n메일은 보내지 말고 보고서 결과로 메일 초안만 만들어줘.”',35);
    await art(s,dp,[[0,13000,65000,15000],[37500,16000,37500,16000],[65500,8000,0,15000]],[400,300,370]);
    headings(s,['발화 + 선택한 화면','VIA의 요청 이해','위임할 업무']);handoff(s);
    column(s,'input',0,'화면에서 선택한 표\n대화에 남아 있는 보고서');
    column(s,'resolve',1,'대상: 어느 표 · 어느 보고서?\n순서: 보고서 수정 → 메일 초안\n조건: 메일 발송 금지');
    column(s,'delegate',2,'보고서에 표 추가\n그 결과로 메일 초안 작성\n발송하지 않는 조건 유지');
    text(s,'clarify','지칭 대상이 모호하면 VIA가 사용자에게 확인합니다',635,807,660,25,C.muted,false,'center');
    close(s,'사용자는 파일명이나 Task 번호를 외우지 않아도, 지칭 대상과 순서, 조건을 함께 말할 수 있습니다',challenges[dp]);
    s.speakerNotes.textFrame.setText(commonNotes+`\nDP41 근거: docs/architecture/12-decisions/decision-packages/04-41-request-resolution-control.md. 선택 표, 기존 보고서의 후보와 연결 업무는 조회/의미 판단의 결과다. 입력에 정답 Task를 사전 지정하지 않는다. 보고서를 수정한 결과로 메일 초안을 쓰는 연속 업무이며 별도 독립 메일이나 발송 요청이 아니다. 대상·순서·조건은 설명용 개념이지 B의 실제 고정 key field를 정의한 schema가 아니다. A는 모델이 제한된 조회와 다음 해석을 반복하고 최종 의미를 제안한다. B는 유한한 typed 표현에 따라 코드가 조회/조건부 의미 호출과 결합을 완성한다. B도 여러 모델 호출이 가능하다. 메일 초안 작성·보고서 편집은 업무 수행 Agent 책임이다. 이 페이지는 의미 생산의 기능 분할(43), 실행 연결(44), 과거 정보 공급(45), 대화/업무 상태 권한(42)을 다시 선택하지 않는다.`);
  }
  if(dp===44) {
    const s=scene(dp,'VIA가 처리하는 동안에도,\n새 발화와 업무 결과는 계속 들어옵니다',
      '“보고서 내용을 설명해주고,\n내일 팀 회의 안내 메일 초안도 만들어줘.”',36);
    await art(s,dp,[[0,10000,70000,18000],[43000,18000,35000,28000],[67000,13000,0,21000]],[340,330,400]);
    headings(s,['새 사용자 발화','VIA의 준비 완료','Agent의 질문']);
    column(s,'new-speech',0,'“잠깐, 표부터 설명해줘.”',672,29);
    column(s,'late-preparation',1,'이전 보고서 설명이\n늦게 준비됨',657,29);
    column(s,'agent-question',2,'“회의 안내 메일의\n수신자는 누구인가요?”',657,29);
    for(const [i,x] of [330,965,1590].entries())line(s,'arrival '+i,[[x,750],[x,793]],i===2?C.apricot:C.teal,2,true);
    line(s,'event-join',[[330,803],[1590,803]],C.guide,1.8);
    line(s,'continuation',[[965,803],[965,832]],C.teal,2,true);
    text(s,'via-continuation','새 입력 처리 · 유효한 응답과 질문 연결 · 실제 전달 뒤 후속 처리',180,842,1560,28,C.ink,true,'center');
    close(s,'사용자는 말을 이어가고, 업무 질문과 결과도 같은 대화에서 받습니다',challenges[dp],899);
    s.speakerNotes.textFrame.setText(commonNotes+`\nDP44 근거: docs/architecture/12-decisions/decision-packages/04-44-continuous-interaction.md 배경 및 직접 다루는 네 종류의 사건. VIA는 보고서 설명을 직접 준비하고 Agent는 독립적인 회의 안내 메일 초안 업무를 수행한다. 그동안 새 표 설명 요청, 메일 수신자 질문, 이전 보고서 설명의 준비 완료가 겹친다. 중앙 삽화는 VIA라는 참여자이며 응답 준비가 별도 Component임을 뜻하지 않는다. 합류선은 사건들을 다뤄야 한다는 공통 요구이지 중앙 Dispatcher나 B의 join Module을 미리 선택한 구조가 아니다. 네 번째 사건인 실제 표시/재생/중단 receipt는 하단 후속 처리에 포함한다. 발화 시작 시 음성 중단과 지속 입력은 공통 기능이다. 발화 시작이 Task 취소는 아니며 응답 후보 준비가 사용자 전달 완료도 아니다. 기존 응답의 현재 유효성·권한·출력 차례를 확인하고 필요한 대기를 이어간다. 도착 순서가 고정 전달 우선순위를 정하지 않는다. A도 비동기이고 B도 협력이 필요하다. 비교는 중앙의 실행 명령·완료 회수·대기 관리와 연결된 단계의 입력/대기 상태·조건 충족 활성화다.`);
  }
  if(dp===45) {
    const s=scene(dp,'이전 설명과 여러 Agent의 결과를,\n새 요청에 함께 활용하는 VIA',
      '“지난번 네가 설명한 평가 기준으로,\n제품 비교와 견적 결과를 함께 써서 제안서를 만들어줘.”',35);
    await art(s,dp,[[0,11000,66000,13000],[37000,11000,35000,13000],[65500,10000,0,12000]],[360,310,355]);
    headings(s,['서로 다른 과거 기록','VIA의 맥락 연결','새 업무를 수행하는 Agent']);handoff(s);
    column(s,'past-records',0,'VIA가 설명한 평가 기준\n제품 조사 Agent의 비교 결과\n견적 Agent의 견적 결과');
    column(s,'context',1,'관련 기록·실제 전달 범위 확인\n필요한 발췌·결과 참조 연결\n이번 요청에 필요한 맥락 제공',692,27);
    column(s,'domain-work',2,'평가 기준을 적용해 자료 분석\n새 제안서 작성',692,28);
    close(s,'사용자는 이전 설명과 여러 업무의 결과를 다시 모아 전달할 필요가 없습니다',challenges[dp]);
    s.speakerNotes.textFrame.setText(commonNotes+`\nDP45 근거: docs/architecture/12-decisions/decision-packages/04-45-memory-and-context.md §1, §3. 평가 기준은 VIA의 직접 응답이고 제품 비교·견적은 서로 다른 Agent의 확인된 업무 결과다. 같은 Agent에게 예전 보고서 스타일을 다시 위임하는 예시는 여기서 근거로 쓰지 않는다. VIA는 수신한 기록, 허용된 발췌·결과/버전 참조와 실제 전달 기록을 연결한다. 결과 참조가 전체 본문 보관을 뜻하지 않는다. 모든 Agent 내부 reasoning/tool 이력이나 전달받지 않은 자료를 알고 있다고 가정하지 않는다. 기준 적용과 새 제안서의 업무 판단·작성은 Agent 책임이다. 이 페이지의 세 기록은 관련 근거의 예시이지 특정 저장 구조를 선택하지 않는다. A도 summary/index/cache/부분 갱신을 허용한다. 비교는 요청 소비자가 원본 소유자 읽기를 조합하는 책임과 생산자가 공통 과거 관계를 생산·게시·폐기하는 지속 읽기 계약이다. B의 파생 기록이 현재 요청의 정답 의미나 위임 Task를 미리 확정하지 않는다. 원본·현재성·권한·누락 확인은 공통이다. 과거 기록을 장기 사용자 선호로 자동 승격하지 않는다.`);
  }
  if(dp===42) {
    const s=scene(dp,'대화와 업무는\n서로 다른 시점에 시작하고 끝납니다',
      '보고서를 맡기고 → 음성 연결을 잠시 끊고 → 다시 돌아와 결론을 수정',34);
    await art(s,dp,[[0,9000,67000,12000],[34500,13000,34500,16000],[67000,9000,0,12000]],[325,340,325],272,250);
    headings(s,['보고서 작성 요청','음성 연결을 잠시 종료','완료된 보고서 수정 요청'],542);
    column(s,'first-request',0,'“이 자료로 보고서를 만들어줘.”',595,27);
    column(s,'connection-ended',1,'보고서 업무는 계속 진행',595,27);
    column(s,'revision',2,'“결론 부분만 짧게 바꿔줘.”',595,27);
    const start=465,end=1790;
    const rows=[['음성 연결',706],['개별 요청 처리',763],['보고서 업무 이력',820],['Agent의 실제 실행',877]];
    for(const [label,y] of rows) {
      text(s,'row '+label,label,85,y-17,335,28,C.ink,true);
      line(s,'baseline '+label,[[start,y],[end,y]],C.guide,1.1,false,true);
    }
    line(s,'voice-first',[[start,706],[715,706]],C.teal,4);
    line(s,'voice-return',[[1365,706],[end,706]],C.teal,4);
    text(s,'voice-off','연결 없음',860,679,325,24,C.muted,false,'center');
    line(s,'request-one',[[start,763],[650,763]],C.teal,4);
    line(s,'request-two',[[1365,763],[1650,763]],C.teal,4);
    text(s,'request-one-label','위임 요청 처리',460,735,200,23,C.teal,false,'center');
    text(s,'request-two-label','수정 요청 처리',1365,735,285,23,C.teal,false,'center');
    line(s,'task-history',[[start,820],[end,820]],C.teal,4);
    text(s,'task-record','같은 보고서의 업무·결과·정정 이력을 유지',730,792,940,24,C.teal,false,'center');
    line(s,'write-execution',[[555,877],[1125,877]],C.apricot,4);
    line(s,'revise-execution',[[1480,877],[end,877]],C.apricot,4);
    text(s,'write-label','보고서 작성 실행',650,849,400,24,C.apricot,false,'center');
    text(s,'revise-label','보고서 수정 실행',1480,849,310,24,C.apricot,false,'center');
    close(s,'사용자는 다시 돌아와, 이미 맡긴 보고서의 결과를 확인하고 수정을 이어갑니다',challenges[dp],913,975);
    s.speakerNotes.textFrame.setText(commonNotes+`\nDP42 근거: docs/architecture/12-decisions/decision-packages/04-42-lifecycle-ownership.md §1~3 및 main-figure review. 타임라인 길이는 설명용이며 시간·성능 측정 값이 아니다. 음성 연결 종료는 Conversation·Task 취소/삭제가 아니다. Conversation/Request/Task/Execution의 서로 다른 논리 수명은 양안 공통이다. 첫 Request 처리가 끝나는 것은 확인된 위임 접수 등 요청 처리의 종료이지 보고서 실행 완료가 아니다. 외부 Agent는 요청 처리와 음성 연결에 독립적으로 보고서 작성을 계속하고 결과를 VIA에 통지한다. 돌아온 사용자 정정은 새로운 Request이며 같은 보고서 업무에 연결되어 새 수정 실행을 만들 수 있다. 업무 이력의 연속 실선은 처음 작성 실행이 계속 살아있거나 완료 Task가 영원히 실행중이라는 뜻이 아니다. A는 모듈형 통합 Core의 대화/업무 상태 소유와 관련 로컬 변경의 공동 확정, B는 독립 대화/업무 서비스의 상태 권한과 commands/receipts/events 협력을 비교한다. 양안 모두 이 장면을 수행한다. 정상 장면의 정확성·응답성·호출비용에 B의 자동 이익을 주장하지 않으며 장애·업데이트를 대표 이익으로 쓰지 않는다. 조건부 독립 수명/장애 격리와 cross-service coordination 비용은 다음 비교에서 설명한다.`);
  }
}

const draft=path.join(build,'background-candidate.pptx'),manifestPath=path.join(build,'manifest.json');
await fs.writeFile(manifestPath,JSON.stringify(manifest,null,2)+'\n');
await (await PresentationFile.exportPptx(presentation)).save(draft);
const dest=path.join(repo,'docs/presentations_files/VIA-DP-background-and-comparison-41-45_v2.pptx');
execFileSync(python,[path.join(repo,'scripts/presentations/dp_v2_background_package.py'),draft,dest,manifestPath,path.join(build,'preservation.json')]);
await fs.mkdir(path.join(build,'final'),{recursive:true});
const final=path.join(build,'final','VIA-DP-background-and-comparison-41-45_v2.pptx');
await finalizePresentation({workspaceDir:build,candidatePath:draft,finalPath:final,explicitTotalSlideCount:12,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-heading-fit'],
  requiredNativeTableOwnerSlides:[],fontPolicy:{basis:'design',families:[font]},verifyArtifactToolImport:true,
  receiptPath:path.join(build,'validation.json')});
const imported=await PresentationFile.importPptx(await FileBlob.load(final));
const positions={41:2,44:4,45:6,42:8};
for(const dp of selected) {
  const png=await imported.export({slide:imported.slides.items[positions[dp]],format:'png',scale:1});
  await fs.writeFile(path.join(build,`dp${dp}-background.png`),new Uint8Array(await png.arrayBuffer()));
}
await fs.copyFile(final,dest);
console.log(JSON.stringify({deck:dest,pages:12,selected,previews:selected.map(dp=>path.join(build,`dp${dp}-background.png`))}));
