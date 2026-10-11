import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {Presentation, PresentationFile, FileBlob} from '@oai/artifact-tool';

// This is a scoped revision of v2. It never reassembles the seven legacy decks.
const [repoArg, buildArg] = process.argv.slice(2);
const repo = await fs.realpath(repoArg), build = await fs.realpath(buildArg);
const skill = process.env.VIA_PRESENTATION_SKILL_DIR, python = process.env.VIA_RUNTIME_PYTHON;
if (![skill, python].every(v => path.isAbsolute(v ?? ''))) throw new Error('Set bundled runtime paths');
const source = path.join(repo, 'docs/presentations_files/VIA-architecture-presentation_v2.pptx');
const sourceHash = createHash('sha256').update(await fs.readFile(source)).digest('hex');
const previous = JSON.parse(await fs.readFile(path.join(repo,'docs/presentations_files/full-presentation/manifest-v2.json'),'utf8'));
if (sourceHash !== previous.output_sha256) throw new Error('Source changed; rebase the background revision');
const p = Presentation.create({slideSize:{width:1920,height:1080}});
const C = {ink:'#202B35',muted:'#576773',teal:'#00889A',navy:'#354354',line:'#BCCAD2',light:'#F5F7F8',blue:'#EAF3F7',orange:'#A55D28',red:'#A54B43',white:'#FFFFFF'};
let s, serial=0;
function shape(name,geometry,x,y,w,h,fill='none',stroke='none',width=0){
  return s.shapes.add({name:`bg-${serial++}-${name}`,geometry,position:{left:x,top:y,width:w,height:h},fill,line:{fill:stroke,width}});
}
function txt(value,x,y,w,h,size=30,color=C.ink,bold=false,align='left'){
  const o=shape('text','textbox',x,y,w,h);o.text=value;
  o.text.style={typeface:'Apple SD Gothic Neo',fontSize:size,color,bold,alignment:align,verticalAlignment:'middle',wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
  return o;
}
function poly(name,pts,color=C.muted,width=2.5,arrow=false,dashed=false){
  const xs=pts.map(v=>v[0]),ys=pts.map(v=>v[1]),x=Math.min(...xs),y=Math.min(...ys),w=Math.max(1,Math.max(...xs)-x),h=Math.max(1,Math.max(...ys)-y);
  s.shapes.add({name:`bg-${serial++}-${name}`,geometry:'custom',position:{left:x,top:y,width:w,height:h},fill:'none',line:{fill:color,width,style:dashed?'dashed':'solid'},
    customPaths:[{width:w,height:h,commands:pts.map(([px,py],i)=>({[i?'lineTo':'moveTo']:{x:px-x,y:py-y}}))}]});
  if(arrow){const [a,b]=pts.slice(-2),ang=Math.atan2(b[1]-a[1],b[0]-a[0]),len=13;
    poly(name+'-head',[[b[0]-len*Math.cos(ang-.5),b[1]-len*Math.sin(ang-.5)],b,[b[0]-len*Math.cos(ang+.5),b[1]-len*Math.sin(ang+.5)]],color,width);}
}
function base(dp,page,title){
  s=p.slides.add();s.background.fill=C.white;
  txt(`VIA SOFTWARE ARCHITECTURE / DP${dp} 배경`,52,20,1500,34,24,C.teal,true);
  txt(`v3 / ${page}`,1740,20,130,34,23,C.muted,false,'right');
  txt(`${dp}. ${title}`,52,63,1816,67,51,C.ink,true);
  poly('title-rule',[[52,140],[1868,140]],C.line,2);
}
function footer(question,cost){
  poly('footer-rule',[[52,890],[1868,890]],C.line,2);
  txt('설계 질문',52,912,215,52,34,C.teal,true);
  txt(question,280,912,1588,52,31,C.teal,true);
  txt(cost,52,994,1816,47,27,C.muted);
}
function note(dp,body){s.speakerNotes.textFrame.setText(body+`\n\n출처: docs/architecture/12-decisions/decision-packages/04-${dp}-${({41:'request-resolution-control',44:'continuous-interaction',45:'memory-and-context',42:'lifecycle-ownership'})[dp]}.md; 04-40-common-execution-contract.md. A/B 미선정, 미구현, 미측정 상태의 공통 문제를 설명한다.`);}
function circle(label,x,y,d,fill=C.blue,size=31){
  shape(label,'ellipse',x,y,d,d,fill,C.teal,2.5);txt(label,x,y,d,d,size,C.ink,true,'center');
}

// 41: the sentence is decomposed into evidence and meaning, not actor roles.
base(41,15,'이번 지시의 대상과 적용 범위');
txt('“이 표를 아까 보고서에 넣고,\n메일은 보내지 말고 보고서 결과로 초안만 만들어줘.”',52,164,1816,116,38,C.teal,true);
const vals=[
  ['말 속 표현','읽어서 확인할 근거','VIA가 완성할 요청 의미'],
  ['이 표','당시 선택 화면과 시점','실제 표와 근거 연결'],
  ['아까 보고서','이전 요청과 현재 업무 상태','수정할 보고서 업무 연결'],
  ['보고서 결과로','두 요청의 순서와 데이터 관계','보고서 결과와 메일 초안의 의존 관계'],
  ['보내지 말고, 초안만','이번 지시와 유지해야 할 조건','메일 발송 제외 조건 유지']
];
const t=s.tables.add({name:'dp41-meaning-evidence',rows:5,columns:3,left:52,top:322,width:1816,height:425,columnWidths:[412,628,776],values:vals});
for(let r=0;r<5;r++){t.rows[r].height=r?91.25:60;
  for(let c=0;c<3;c++)t.cells.block({row:r,column:c,rowCount:1,columnCount:1}).assign({fill:r?(r%2?C.white:C.light):C.navy,
    margins:{left:16,right:12,top:5,bottom:5},anchor:'center',textStyle:{typeface:'Apple SD Gothic Neo',fontSize:30,color:r?(c===0?C.teal:C.ink):C.white,bold:r===0||c===0,alignment:'left'}});}
t.borders.assign({style:'solid',fill:C.line,width:1});
txt('조회 결과에 따라 뜻이 달라지고, 바뀐 뜻에 따라 다음에 읽을 자료도 달라집니다.',52,780,1816,50,34,C.ink,true);
txt('대상이 모호하면 확인합니다. VIA는 요청 의미를 연결하고, 실제 작성과 수정은 Downstream Agent가 실행합니다.',52,838,1816,35,25,C.muted);
footer('다음 조회와 전체 의미를 모델이 결정할까, 코드가 항목 해결 상태로 진행할까?',
  '모델의 유연한 재판단과 반복 비용 / 코드의 추적 가능성, 유한 표현의 지원 범위와 추가 해석 비용');
note(41,'이번 입력의 지칭, Task 연결, 요청 간 순서와 유지 조건을 함께 완성해야 하는 문제다. 표의 오른쪽은 확인 후 완성해야 할 내용이며 최초 입력에 정답 Task를 제공한다는 뜻이 아니다. 추가 조회와 뜻 해석은 서로 영향을 준다. 최종 의미의 검증/채택과 실제 업무 실행을 구별한다. 보고서 수정과 메일 초안 작성은 Downstream Agent가 담당한다. 45의 과거 근거 공급 방식은 여기서 선택하지 않는다.');

// 44: overlapping arrivals and actual delivery use an event timeline.
base(44,17,'겹쳐 도착하는 사건과 응답 시점');
txt('보고서 설명을 들으며 메일 초안 작성도 진행 중입니다.\n새 발화, 업무 질문과 이전 응답의 준비 완료가 겹쳐 들어옵니다.',52,169,1816,107,35,C.ink);
poly('event-order',[[354,320],[1850,320]],C.line,2.5,true);
for(const [x,label,w] of [[365,'설명 중',220],[785,'새 발화 시작',310],[1260,'발화 종료',260],[1650,'실제 전달',218]]){
  poly('tick',[[x,310],[x,332]],C.line,2);txt(label,x-5,279,w,33,27,C.muted,true);
}
for(const [y,label] of [[406,'사용자 입력'],[515,'VIA 음성'],[636,'메일 업무 질문'],[751,'이전 응답 준비']]){
  txt(label,52,y-25,275,52,31,C.navy,true);poly('lane',[[354,y],[1850,y]],C.line,1.6,false,true);
}
shape('speech-interval','rect',785,397,475,18,C.teal);
txt('“잠깐, 표부터 설명해줘.”',803,427,618,38,30,C.teal,true);
poly('old-audio',[[365,515],[785,515]],C.teal,8);
poly('stop',[[785,495],[785,534]],C.red,4);txt('즉시 재생 중단',692,463,353,37,29,C.red,true);
txt('보고서 설명',385,542,276,37,27,C.muted);
poly('new-audio',[[1340,515],[1520,515]],C.teal,8);txt('표 설명',1340,542,225,37,27,C.teal);
circle('',940,623,26,C.white);txt('수신자 질문 도착',910,584,348,39,29,C.orange,true);
poly('question-wait',[[966,636],[1620,636]],C.orange,3,false,true);
txt('질문을 해당 업무에 연결하고 전달 대기',975,660,752,41,28,C.muted);
poly('question-delivery',[[1620,636],[1824,636]],C.orange,7,true);txt('메일 질문 전달',1585,585,283,40,28,C.orange);
circle('',1054,737,28,C.white);txt('옛 후보가 늦게 준비됨',889,778,513,35,28,C.muted);
poly('stale-result',[[1082,751],[1176,751]],C.red,3,true);
poly('discard-x1',[[1193,738],[1220,765]],C.red,3);poly('discard-x2',[[1220,738],[1193,765]],C.red,3);
txt('현재 요청에 맞지 않는 후보 폐기',1260,727,608,49,29,C.red,true);
txt('설명용 사건 순서입니다. 선의 길이는 측정 시간이 아니며, 전달 순서는 장면의 예입니다.',52,842,1816,36,25,C.muted);
footer('완료와 대기를 중앙이 연결할까, 단계별 조건으로 후속 처리를 이어갈까?',
  '양안 모두 비동기 실행과 지속 입력 가능. 중앙 회수/지시 비용과 단계별 Window/Join 상태 비용을 비교');
note(44,'새 발화 때 로컬 재생을 즉시 중단하며 의미 해석과 외부 Task 취소 완료를 기다리지 않는다. 메일 업무 자체는 별도로 유지된다. 지금 그림은 공통 사건의 예이며 A/B의 Component 배치를 나타내지 않는다. 질문은 input 해결 상태에 맞춰 보류하거나 적절한 차례에 전달할 수 있다. 표 설명 후 질문 전달은 예시이지 모든 상황의 고정 우선순위가 아니다. 늦은 후보의 현재성을 확인하고 실제 표시/재생 결과로 후속 처리를 이어가야 한다. 선의 길이나 간격은 실측 시간이 아니다.');

// 45: a history ledger distinguishes source evidence from current Context.
base(45,19,'흩어진 과거 근거의 연결과 재사용');
txt('“지난번 네가 설명한 평가 기준으로,\n제품 비교와 견적 결과를 함께 써서 제안서를 만들어줘.”',52,164,1816,116,38,C.teal,true);
txt('서로 다른 시점과 원본의 기록',52,311,1030,49,36,C.navy,true);
txt('이번 요청에 필요한 Context',1232,311,636,49,36,C.teal,true);
poly('history-order',[[84,390],[84,782]],C.line,3,true);
const history=[
  [403,'VIA가 직접 설명한 평가 기준','P20: 실제 전달 범위와 당시 원문','평가 기준의 필요한 부분'],
  [556,'제품 조사 업무의 확인된 결과','T21 / D21@v2: 결과 참조와 허용 발췌','제품 비교 결과의 근거'],
  [709,'견적 업무의 확인된 결과','T22 / D22@v1: 결과 참조와 원본 버전','견적과 적용 가능한 버전']
];
for(const [y,title,details,context] of history){
  circle('',72,y+12,24,C.white);txt(title,145,y-5,914,50,33,C.ink,true);
  txt(details,145,y+50,914,42,28,C.muted);
  poly('history-reference',[[1084,y+28],[1200,y+28]],C.teal,3,true,true);
  txt(context,1232,y+1,636,56,32,C.teal,true);
}
txt('기록은 축적되고, 같은 과거 관계를 다른 요청에서도 다시 사용할 수 있습니다.',145,796,1700,43,30,C.ink,true);
txt('원본의 수정/삭제와 실제 전달 범위도 확인합니다. 결과 참조는 본문 전체의 보관을 뜻하지 않습니다.',52,845,1816,33,25,C.muted);
footer('요청마다 원본 읽기를 조합할까, 공통 과거 관계를 생산하고 유지해 조회할까?',
  'A도 유효 cache와 부분 갱신 가능. B는 초기/미사용 생산, 삭제/철회와 게시 범위 관리 비용을 부담');
note(45,'과거 평가 기준은 VIA가 실제 전달한 설명이고 제품 조사와 견적은 서로 다른 Task/Agent의 확인된 결과다. 조회 화살표는 공통 근거 제공 요구를 나타내며 별도 공통 저장소나 사전 생산을 선택하지 않는다. ID/버전은 기존 설명 사례의 참조이고 측정 fixture를 새로 정한 것이 아니다. 현재 Task 사실과 권한은 원본 owner에게 확인한다. 과거 근거를 연결하고 필요한 발췌/참조를 제공하는 일은 VIA, 자료 분석과 제안서 작성은 Downstream Agent의 책임이다. 재사용은 가능한 필요를 보여주며 전부 미리 생산해야 한다는 결론이 아니다. 이번 요청의 최종 대상/의미를 정하는 41과 과거 근거 공급을 구별한다.');

// 42: many requests and executions connect to one persistent Task identity.
base(42,21,'서로 다른 요청과 실행을 잇는 업무 상태');
txt('보고서 작성이 끝나도 같은 업무를 확인하고 수정할 수 있습니다.',52,170,1816,53,37,C.ink,true);
txt('Conversation C1: 음성 연결이 꺼져도 대화 기록과 업무 연결은 유지합니다.',52,235,1816,43,31,C.teal);
txt('개별 요청의 처리',100,308,735,43,34,C.navy,true);
txt('계속 추적하는 업무',863,308,458,43,34,C.teal,true);
txt('Downstream Agent 실행',1458,308,410,43,31,C.navy,true);
for(const [label,y,body] of [
  ['R1',365,'보고서 작성 위임\n외부 접수 확인 뒤 요청 처리 종료'],
  ['R2',515,'진행 확인\n확인된 상태로 답변, 새 실행 없음'],
  ['R3',665,'결론 수정\n같은 보고서 업무에 수정 요청 연결']
]){circle(label,100,y,82,C.blue,30);txt(body,215,y-1,607,84,29,C.ink);}
circle('Task T1',920,480,190,C.blue,37);
txt('보고서 업무',906,684,264,43,30,C.teal,true,'center');
poly('r1-task',[[141,447],[141,461],[870,461],[870,529],[920,529]],C.teal,2.8,true);
poly('r2-task',[[141,597],[141,611],[865,611],[920,611]],C.muted,2.8,true,true);
poly('r3-task',[[141,747],[141,767],[875,767],[875,642],[920,642]],C.teal,2.8,true);
circle('X1',1500,369,108,C.light,34);txt('보고서 작성\n완료',1640,371,228,99,29,C.ink);
circle('X2',1500,646,108,C.light,34);txt('결론 수정\n진행 중',1640,647,228,101,29,C.ink);
poly('task-x1',[[1110,538],[1180,538],[1180,423],[1500,423]],C.muted,2.8,true,true);
poly('task-x2',[[1110,619],[1180,619],[1180,700],[1500,700]],C.teal,2.8,true);
txt('최초 작성',1240,367,240,39,28,C.muted);
txt('이후 수정',1240,647,240,39,28,C.teal);
txt('대화의 R3→T1 연결과 업무의 수정 명령/버전이 서로 맞아야 합니다.',52,817,1816,56,34,C.ink,true);
footer('관련 대화/업무 상태를 함께 확정할까, 독립 상태의 접수 결과로 맞출까?',
  'A도 논리 수명 분리와 모듈화 가능. B의 독립 운영 조건과 인계/재확인, 추가 process와 상태 비용을 비교');
note(42,'R1은 최초 위임, R2는 기존 보고서의 확인된 진행 조회, R3는 같은 보고서 결론 수정의 개념 사례다. 원본 배경처럼 보고서 Task identity는 유지되며 완료한 첫 작성 X1과 후속 수정 X2를 구별한다. R1의 요청 처리 종료는 외부 Agent의 접수 확인 후이며 업무 실행 완료를 뜻하지 않는다. 음성 연결이 꺼져도 Conversation은 유지한다. 이 선은 데이터/참조의 개념 관계이며 process 배치나 A/B 선택 그림이 아니다. 이번 초점은 해석 후 관련 대화/업무 상태를 어느 권위가 유지하고 확정하는가다. 자연어 해석은 41, 사건 후속 실행은 44에서 다룬다. 의미 모델을 transaction 안에서 실행하거나 서비스 분리를 정확성/변경 우위로 주장하지 않는다.');

const draft=path.join(build,'authored.pptx');await(await PresentationFile.exportPptx(p)).save(draft);
const planPath=path.join(build,'background-plan.json');
await fs.writeFile(planPath,JSON.stringify({version:3,source_sha256:sourceHash,changes:[
  {dp:41,source_page:15,authored_page:1,key:'dp41-background-v3',preserved_page:58},
  {dp:44,source_page:17,authored_page:2,key:'dp44-background-v3',preserved_page:59},
  {dp:45,source_page:19,authored_page:3,key:'dp45-background-v3',preserved_page:60},
  {dp:42,source_page:21,authored_page:4,key:'dp42-background-v3',preserved_page:61}
]},null,2)+'\n');
const candidate=path.join(build,'candidate.pptx'),manifestPath=path.join(build,'manifest-v3.json');
execFileSync(python,[path.join(repo,'scripts/presentations/package_background_revision.py'),source,draft,
  path.join(repo,'docs/presentations_files/full-presentation/manifest-v2.json'),planPath,candidate,manifestPath],{stdio:'inherit'});
const manifest=JSON.parse(await fs.readFile(manifestPath,'utf8'));
const owners=manifest.slides.filter(r=>r.native_tables>0).map(r=>r.page);
const final=path.join(build,'final','VIA-architecture-presentation_v3.pptx');await fs.mkdir(path.dirname(final),{recursive:true});
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:final,explicitTotalSlideCount:61,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit',...owners.flatMap(n=>['--require-native-table-slide',String(n)])],
  requiredNativeTableOwnerSlides:owners,fontPolicy:{basis:'reference',families:['Apple SD Gothic Neo','Arial Unicode MS'],referencePath:source,referenceSha256:sourceHash},
  verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
manifest.output_sha256=createHash('sha256').update(await fs.readFile(final)).digest('hex');
await fs.writeFile(manifestPath,JSON.stringify(manifest,null,2)+'\n');
const rendered=await PresentationFile.importPptx(await FileBlob.load(final));
await fs.writeFile(path.join(build,'inspection.ndjson'),(await rendered.inspect({kind:'slide,layout,table,image',maxChars:200000})).ndjson);
await fs.mkdir(path.join(build,'renders'),{recursive:true});
for(const [i,slide] of rendered.slides.items.entries()){
  const png=await rendered.export({slide,format:'png',scale:1});
  await fs.writeFile(path.join(build,'renders',`${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await png.arrayBuffer()));
  console.log(`Rendered ${i+1}/61`);
}
if(sourceHash!==createHash('sha256').update(await fs.readFile(source)).digest('hex'))throw new Error('Source changed during edit');
console.log(JSON.stringify({final,manifestPath,sourceUnchanged:true}));
