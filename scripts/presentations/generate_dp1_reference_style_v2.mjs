import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {Presentation, PresentationFile} from '@oai/artifact-tool';

// Dedicated, editable three-slide DP1 deck. Run a copy from a private build
// directory linked to the bundled runtime node_modules.
const BUILD=process.argv[2];
if (!BUILD || !path.isAbsolute(BUILD)) throw new Error('Pass absolute build directory');
const SKILL='/Users/changbae/.codex/plugins/cache/openai-primary-runtime/presentations/26.1015.12040/skills/presentations';
const PY='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3';
process.env.RUNTIME_NODE_MODULES='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules';
const ROOT='/Users/changbae/Workspace/via-sw-architecture';
const FONT='Apple SD Gothic Neo';
const W=1920,H=1080;
const C={black:'#111111', gray:'#98999B', grid:'#B7B9B9', green:'#C9E1B3', greenD:'#70A843', cream:'#FFF6DD', blue:'#4676CB', cyan:'#35A8CD', pale:'#F1F6FC', red:'#C00000', muted:'#50555B', white:'#FFFFFF'};
const pres=Presentation.create({slideSize:{width:W,height:H}});
const line=(fill=C.black,width=1.7,style='solid')=>({fill,width,style});
let serial=0;
function shape(s,geom,x,y,w,h,fill='none',stroke='none',lw=0,name=''){
  return s.shapes.add({name:name||'DP1 object '+(++serial),geometry:geom,position:{left:x,top:y,width:w,height:h},fill,line:line(stroke,lw)});
}
function txt(s,t,x,y,w,h,size=23,opt={}){
  const q=shape(s,'textbox',x,y,w,h,opt.fill||'none','none',0,opt.name);
  q.text=t;q.text.style={typeface:FONT,fontSize:size,color:opt.color||C.black,bold:opt.bold||false,alignment:opt.align||'left',verticalAlignment:opt.valign||'middle',wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
  return q;
}
function poly(s,pts,color=C.black,width=2,dashed=false,arrow=false,name=''){
  const xs=pts.map(p=>p[0]),ys=pts.map(p=>p[1]);
  const x=Math.min(...xs),y=Math.min(...ys),w=Math.max(0.1,Math.max(...xs)-x),h=Math.max(0.1,Math.max(...ys)-y);
  s.shapes.add({name:name||'Editable path '+(++serial),geometry:'custom',position:{left:x,top:y,width:w,height:h},fill:'none',line:line(color,width,dashed?'dashed':'solid'),customPaths:[{width:w,height:h,commands:pts.map(([a,b],i)=>({[i?'lineTo':'moveTo']:{x:a-x,y:b-y}}))}]});
  if(arrow){
    const [ex,ey]=pts.at(-1),[px,py]=pts.at(-2),len=Math.hypot(ex-px,ey-py),ux=(ex-px)/len,uy=(ey-py)/len;
    const d=10,b=5;
    const p=[[ex,ey],[ex-d*ux+b*uy,ey-d*uy-b*ux],[ex-d*ux-b*uy,ey-d*uy+b*ux]];
    const ax=Math.min(...p.map(t=>t[0])),ay=Math.min(...p.map(t=>t[1])),aw=Math.max(0.1,Math.max(...p.map(t=>t[0]))-ax),ah=Math.max(0.1,Math.max(...p.map(t=>t[1]))-ay);
    s.shapes.add({name:'Editable arrowhead '+(++serial),geometry:'custom',position:{left:ax,top:ay,width:aw,height:ah},fill:color,line:line(color,0.6),customPaths:[{width:aw,height:ah,commands:[...p.map(([a,b],i)=>({[i?'lineTo':'moveTo']:{x:a-ax,y:b-ay}})),{close:{}}]}]});
  }
}
function box(s,t,x,y,w,h,{fill=C.cream,color=C.black,stroke=C.black,size=23,bold=false,dash=false}={}){
  const q=shape(s,'rect',x,y,w,h,fill,stroke,stroke===C.blue?3:1.8,t);
  if(dash)q.line=line(stroke,1.8,'dashed');
  txt(s,t,x+7,y+4,w-14,h-8,size,{color,bold,align:'center'});return q;
}
function storeShape(s,t,x,y,w,h,{fill=C.cream,stroke=C.black,size=22,color=C.black}={}){
  const q=s.shapes.add({name:t,geometry:'can',position:{left:x,top:y,width:w,height:h},fill,line:line(stroke,1.8),adjustmentList:[{name:'adj',formula:'val 16000'}]});
  txt(s,t,x+5,y+11,w-10,h-17,size,{align:'center',color});return q;
}
function n(s,v,x,y,color=C.greenD){
  shape(s,'roundRect',x,y,28,29,C.green,color,1.8);txt(s,String(v),x,y,28,29,22,{align:'center'});
}
function doc(s,title,body,x,y,w,h){
  const f=16;
  s.shapes.add({name:title,geometry:'custom',position:{left:x,top:y,width:w,height:h},fill:C.white,line:line(C.black,1.7),customPaths:[{width:w,height:h,commands:[{moveTo:{x:0,y:0}},{lineTo:{x:w-f,y:0}},{lineTo:{x:w,y:f}},{lineTo:{x:w,y:h}},{lineTo:{x:0,y:h}},{close:{}}]}]});
  poly(s,[[x+w-f,y],[x+w-f,y+f],[x+w,y+f]],C.black,1.4);
  txt(s,title,x+17,y+15,w-38,36,26,{bold:true});
  poly(s,[[x+17,y+57],[x+w-17,y+57]],C.grid,1);
  txt(s,body,x+17,y+68,w-34,h-80,24);
}
function nativeTable(s,values,x,y,widths,heights,font=22){
  const t=s.tables.add({rows:values.length,columns:widths.length,left:x,top:y,width:widths.reduce((a,b)=>a+b),height:heights.reduce((a,b)=>a+b),columnWidths:widths,values});
  t.styleOptions={headerRow:false,bandedRows:false,bandedColumns:false};
  t.borders.assign({fill:C.grid,width:1.2,style:'solid'});
  t.cells.block({row:0,column:0,rowCount:values.length,columnCount:widths.length}).assign({fill:C.white,textStyle:{typeface:FONT,fontSize:font,color:C.black,alignment:'center',verticalAlignment:'middle',lineSpacing:1},margins:{left:10,right:10,top:1,bottom:1},anchor:'center'});
  heights.forEach((h,i)=>t.rows[i].height=h);
  return t;
}
function header(s,suffix,page){
  s.background.fill=C.white;
  txt(s,'DP1. 현재 요청의 근거 공급 방안'+suffix,60,29,1200,68,48,{bold:true});
  const labels=['과제 소개','요구사항 분석','설계','구현 및 검증','결론'];
  labels.forEach((v,i)=>{
    const x=1210+i*134;
    shape(s,'chevron',x,33,143,44,i===2?C.cyan:C.gray,C.white,2);
    txt(s,v,x+14,33,111,44,19,{bold:true,color:C.white,align:'center'});
  });
  poly(s,[[58,108],[1860,108]],'#A7A7A7',3);
  txt(s,String(page),1835,1049,38,20,17,{align:'right',color:C.muted});
}
function chip(s,label,x,y,w,fill=C.blue){
  shape(s,'roundRect',x,y,w,29,fill,'none',0);txt(s,label,x+8,y,w-16,29,19,{color:C.white,align:'center'});
}

function commonNotes(){
 return '구조와 소유권은 01-evidence-supply.md를 따른다. 작업 문맥과 게시 근거는 데이터이며 새 최상위 Component를 뜻하지 않는다. Context Manager는 허용 근거 구성과 파생 자료 사용, Request Interpreter는 현재 의미 제안, Request Controller는 현재 revision에서의 채택을 맡는다. 그림은 근거 공급 구간의 확대도이며 공통 원본 owner와 Model Access, Policy Manager, State Store, Response Manager 등은 생략했다. 요청 채택은 실제 상세 응답 완료와 다르다. 모든 대안에 원본 보존, 현재 판본/권한 검사, 부족분 원문 보완이 필요하다. C 생산자는 확정 원본 변경에서 관계를 생산하며 자유문 해석 때 조건부 모델을 호출한다. 게시 근거는 파생값이고 미래 요청의 정답을 미리 알지 못한다.\nA/B는 작업 문맥 수명 축, B/C는 별도 생산 책임 축이다. A+C도 조합 가능하나 미실험이다.\n기본안은 각 4개 작성 사례×3회=12episode, 36입력/목표다. 시간은 실제 관측된 endpoint 평균이며 B QA03은 35/36이다. A/B 둘 다 종료된 같은 35목표에서는 QA03 A13.730/B14.242초다. A/B QA01 차이 0.342초는 우열 미확정이며 구조적 동점으로 취급하지 않는다. 첫 호출의 입력과 완료/조회 분기가 다르다. 본문 조건 충족은 A30/36, B30/36, C34/36의 별도 저자 진단으로 QA02와 다르다. C 사전 생산 33호출 비용 $0.0729348이 전체 비용에 포함된다. QA04는 별도 네 Rust 변경 과제의 평균이다. QA06은 Python harness와 Rust의 동시 로컬 RSS이며 원격 모델 weights/KV나 제품 UI를 포함하지 않는다. 원격 qwen3.7-plus-2026-05-26을 사용한 작성 사례의 제한된 PoC이고 실제 제품/on-device Omni 성능으로 일반화하지 않는다.\n내용 출처:\n'+ROOT+'/docs/architecture/12-decisions/decision-packages/seven-problem-review/01-evidence-supply.md\n'+ROOT+'/docs/architecture/12-decisions/decision-packages/seven-problem-review/17-presentation-results.md §3\n'+ROOT+'/results/architecture-evaluation/current/four-dp-presentation-20261019/dp01/presentation-report.md\n스타일 참조: 첨부 23-DP04방안.png, 24-DP04채택.png, 18-DP02채택.png. 대안 3개를 보존하기 위해 두 열을 세 열로 확장했다.';
}
// Revision 2: one visual problem scene, visibly different execution topologies,
// and paired tactic evidence separated from the basic A/B/C population.
function label(s,t,x,y,w,size=22,color=C.black){return txt(s,t,x,y,w,32,size,{color,bold:true,align:'center'});}
function badge(s,t,x,y,w,fill=C.green,color=C.black){shape(s,'rect',x,y,w,34,fill,'none',0);txt(s,t,x+8,y,w-16,34,22,{bold:true,color,align:'center'});}
function cellStyle(cell,size,color=C.black,bold=false,align='center'){
 cell.text.style={typeface:FONT,fontSize:size,color,bold,alignment:align,verticalAlignment:'middle',lineSpacing:1};
}
function highlightHeader(table,n){
 table.cells.block({row:0,column:0,rowCount:1,columnCount:n}).assign({fill:C.green,textStyle:{typeface:FONT,fontSize:25,bold:true,color:C.black,alignment:'center'}});
}

// 1. A single scene: a follow-up must combine actual prior delivery, a result,
// and a changed source. Text describes the conflict, not the implementation.
{
 const s=pres.slides.add();header(s,' (배경)',1);
 txt(s,'같은 자료를 다시 묻는데, 원문은 바뀐다',66,145,1785,62,43,{bold:true});
 txt(s,'VIA는 연속된 대화에서 이전 설명과 업무 결과를 현재 조건에 맞게 다시 사용해야 한다.',68,211,1760,40,27);

 // Current user request dominates the scene.
 shape(s,'roundRect',105,302,1188,133,C.pale,C.blue,2.5);
 txt(s,'현재 요청',128,316,165,34,24,{bold:true,color:C.blue});
 txt(s,'“지난번 설명한 기준과 제품 비교 결과에\n수정된 견적을 반영해줘.”',128,357,1120,64,34,{bold:true,align:'center'});
 poly(s,[[1218,435],[1265,467],[1248,435]],C.blue,2);

 // Three source artifacts are editable document shapes.
 doc(s,'이전에 전달한 설명','',103,583,350,198);
 txt(s,'가격이 같으면',120,650,312,37,25);
 shape(s,'rect',118,710,317,35,C.green,'none',0);
 txt(s,'유지보수 기간 우선',120,708,312,37,25,{bold:true});
 label(s,'실제로 전달한 범위만',103,800,350,24);

 doc(s,'앞선 제품 비교 결과','제품 A  /  제품 B\n가격 · 보증 · 설치 조건',523,583,350,198);
 label(s,'다른 업무의 결과도 필요',523,800,350,24);

 doc(s,'현재 견적  v2','',943,583,350,198);
 txt(s,'기존: 납기 4주',961,650,311,40,25);
 poly(s,[[962,672],[1146,672]],C.red,2.7);
 txt(s,'변경: 납기 6주',961,712,311,40,27,{bold:true,color:C.red});
 badge(s,'v1 → v2',1132,547,160,C.green);
 label(s,'정정 전 조건을 쓰면 안 됨',943,800,350,24,C.red);

 [278,698,1118].forEach((x,i)=>{
   poly(s,[[x,583],[x,499],[x,435]],C.black,2.3,false,true);
   n(s,i+1,x-15,513);
 });


 // The two costs, not a wall of text, define the design tension.
 poly(s,[[1350,293],[1350,849]],C.blue,1.3);
 txt(s,'무엇이 문제인가?',1400,306,440,48,31,{bold:true});
 shape(s,'ellipse',1403,389,48,48,C.green,'none',0);
 txt(s,'1',1403,389,48,48,29,{bold:true,align:'center'});
 txt(s,'매번 다시 찾으면',1473,389,370,48,29,{bold:true});
 txt(s,'반복 탐색·구성의\n대기가 쌓임',1405,456,437,91,32,{color:C.red,bold:true});
 poly(s,[[1406,574],[1834,574]],C.grid,1.4);
 shape(s,'ellipse',1403,615,48,48,C.green,'none',0);
 txt(s,'2',1403,615,48,48,29,{bold:true,align:'center'});
 txt(s,'이전 근거를 남기면',1473,615,370,48,29,{bold:true});
 txt(s,'불필요한 유지·입력과\n정정 갱신 비용이 생김',1405,684,437,91,30,{color:C.red,bold:true});

 poly(s,[[62,895],[1859,895]],C.blue,1.5);
 txt(s,'이 DP의 질문',66,927,310,54,34,{bold:true,color:C.red});
 txt(s,'앞서 읽은 근거를, 다음 요청에 어떻게 공급할 것인가?',375,916,1480,64,37,{bold:true});
 txt(s,'요청 사이의 근거 보존 방식과 구성 시점을 선택한다.',375,986,1480,45,28);
 s.speakerNotes.textFrame.setText('큰 장면은 설계 문제를 설명하는 예시이며 측정 사례의 재현이나 수치 결과가 아니다. 가격/납기 예시는 읽기와 정정 문제를 명확히 하기 위한 시각화다. 원본과 실제 전달 기록은 세 안 모두 보존한다. 과거에 설명하도록 준비했으나 전달하지 않은 내용은 실제 설명 근거가 아니다. 현재 판본과 권한 확인은 공통 필수 조건이다. 이 DP는 지칭의 화면/음성 정렬이나 Downstream Agent 내부 업무 계획을 선택하지 않는다.\n내용 출처: '+ROOT+'/docs/architecture/12-decisions/decision-packages/seven-problem-review/01-evidence-supply.md §1~6\n스타일 참조: 사용자 제공 DP3/DP4 배경 이미지. 모든 설명 도형과 문서는 편집 가능한 native shape다.');
}

// 2. The real architectural difference is encoded in topology and state lifetime.
{
 const s=pres.slides.add();header(s,' (설계 방안 비교)',2);
 txt(s,'무엇을 남기는가?  A: 대화 작업 문맥   /   B: 원본과 공통 기록   /   C: 별도 생산한 게시 근거',28,124,1835,39,27,{bold:true});
 const data=[
 ['설계안','(A) 유효 작업 문맥 유지','(B) 요청마다 원본 구성','(C) 별도 근거 생산·게시'],
 ['구조','','',''],
 ['장점','후속 질문에서 이미 읽은\n근거의 발견·조합을 줄임','새 목적에 필요한 범위만 구성\n이전 해석 문맥의 유지 부담 감소','대화가 바뀌어도 게시한\n근거와 관계를 반복 활용'],
 ['단점','주제 전환 후에도 남는 입력\n원문 정정에 따른 문맥 갱신','같은 자료를 이어 물으면\n원본 재조회와 재조합 반복','사전 생산·갱신·미사용 비용\n부족한 게시 근거는 원본 보완'],
 ['QA01 첫 응답 (초)','6.609 (36/36)','6.267 (36/36)','11.995 (36/36)'],
 ['QA02 지칭 집합','36/36','36/36','34/36'],
 ['QA03 처리시간','13.617 (36/36)','14.242 (35/36)','12.364 (36/36)'],
 ['QA04 변경모듈','1.75','1.75','2.00'],
 ['QA05 모델 비용 ($)','0.06750','0.06992','0.16308'],
 ['QA06 RSS (MB)','44.253','45.318','50.364']
 ];
 const table=nativeTable(s,data,24,184,[172,566,566,568],[50,374,58,58,36,36,36,36,36,36],22);
 highlightHeader(table,4);
 table.cells.block({row:1,column:0,rowCount:1,columnCount:1}).assign({textStyle:{typeface:FONT,fontSize:29,bold:true},anchor:'top',margins:{top:16,left:12,right:12,bottom:0}});
 table.cells.block({row:2,column:0,rowCount:2,columnCount:1}).assign({textStyle:{typeface:FONT,fontSize:25,bold:true}});
 table.cells.block({row:3,column:1,rowCount:1,columnCount:3}).assign({textStyle:{typeface:FONT,fontSize:22,color:C.red,alignment:'center'}});
 table.cells.block({row:4,column:0,rowCount:6,columnCount:1}).assign({fill:'#F7F8FA',textStyle:{typeface:FONT,fontSize:19,bold:true,alignment:'left'}});
 box(s,'Component',41,351,136,43,{fill:C.white,size:18});
 storeShape(s,'근거 상태',41,424,136,56,{size:19});
 shape(s,'rect',43,514,25,23,C.cream,C.black,1.3);txt(s,'비교 대상',78,507,112,38,19);
 txt(s,'순서와 수명\n중심의 확대도',41,553,143,46,19,{color:C.muted,align:'center'});

 // A: persistent state creates a large circulation path across requests.
 let x=196;
 label(s,'같은 대화에서 다음 요청으로 이어짐',x+14,241,538,25);
 shape(s,'roundRect',x+16,281,535,220,'none',C.greenD,1.6);
 txt(s,'대화의 근거 재사용',x+31,285,240,29,20,{bold:true,color:C.greenD});
 storeShape(s,'유효 작업 문맥\n이전 근거 · 대화 수명',x+32,341,213,91,{size:22});
 box(s,'Request Interpreter\n현재 의미 판단',x+289,323,241,76,{size:22,bold:true});
 txt(s,'현재 요청',x+302,286,215,28,22,{bold:true,align:'center'});
 poly(s,[[x+409,315],[x+409,323]],C.black,2,false,true);
 poly(s,[[x+245,365],[x+289,365]],C.black,2.6,false,true);
 n(s,1,x+251,327);
 box(s,'Context Manager\n유효 문맥 갱신',x+289,432,241,59,{size:22});
 poly(s,[[x+410,399],[x+410,432]],C.black,2.6,false,true);
 poly(s,[[x+289,461],[x+138,461],[x+138,432]],C.black,2.6,false,true);
 label(s,'채택 후 갱신 → 다음 요청에서 재사용',x+28,501,509,23,C.greenD);
 storeShape(s,'허용 원본',x+31,545,182,48,{fill:C.white,size:22});
 poly(s,[[x+213,568],[x+410,568],[x+410,491]],C.black,1.8,true,true);
 txt(s,'부족·변경 시 읽기',x+228,538,286,28,21,{align:'center'});

 // B: all extra evidence is formed inside one request, then ends.
 x=762;
 label(s,'요청 안에서 구성하고 추가 묶음은 해제',x+10,241,545,25);
 const bound=shape(s,'rect',x+220,280,329,317,'none',C.gray,1.4);bound.line=line(C.gray,1.4,'dashed');
 box(s,'Request Interpreter\n현재 입력에서 범위 판단',x+244,288,281,61,{fill:C.white,size:21,bold:true});
 poly(s,[[x+386,349],[x+386,356]],C.black,2.2,false,true);
 box(s,'Context Manager\n원본 조회·구성',x+244,356,281,63,{size:23});
 storeShape(s,'허용 원본\n공통 기록',x+23,354,171,74,{fill:C.white,size:23});
 poly(s,[[x+194,389],[x+244,389]],C.black,2.4,false,true);
 n(s,1,x+204,348);
 poly(s,[[x+386,419],[x+386,438]],C.black,2.2,false,true);
 storeShape(s,'임시 근거 묶음 (요청 수명)',x+244,438,281,54,{size:20});
 poly(s,[[x+386,492],[x+386,510]],C.black,2.2,false,true);
 box(s,'Request Interpreter\n현재 의미 판단',x+244,510,281,60,{size:23,bold:true});
 txt(s,'요청 종료 → 추가 묶음 해제',x+235,574,302,25,20,{bold:true,color:C.red,align:'center'});
 poly(s,[[x+225,581],[x+208,581],[x+208,322],[x+244,322]],C.gray,1.8,true,true);
 txt(s,'다음 요청도\n새로 구성',x+28,471,166,66,24,{bold:true,align:'center'});

 // C: producer and consumer lanes are genuinely separate lifetimes.
 x=1328;
 label(s,'요청 밖의 생산 경로 + 요청 안의 조회',x+10,241,546,25);
 shape(s,'rect',x+16,281,536,145,C.cream,C.black,1.4);
 txt(s,'생산 경로 · 요청과 별도로 동작',x+29,286,500,29,22,{bold:true});
 box(s,'허용 원본\n변경 발생',x+29,331,126,65,{fill:C.white,size:21});
 box(s,'근거 생산자\n관계 생산·검증',x+182,326,176,75,{fill:C.white,size:21});
 storeShape(s,'게시 근거',x+391,332,146,62,{fill:C.white,size:22});
 poly(s,[[x+155,362],[x+182,362]],C.black,2.3,false,true);
 poly(s,[[x+358,362],[x+391,362]],C.black,2.3,false,true);
 shape(s,'rect',x+16,459,536,138,'none',C.black,1.4);
 txt(s,'요청 처리 경로',x+29,465,290,27,22,{bold:true});
 box(s,'Context Manager\n게시 근거 조회',x+29,505,225,59,{size:21});
 box(s,'Request Interpreter\n현재 의미 판단',x+300,505,237,59,{size:21,bold:true});
 poly(s,[[x+254,535],[x+300,535]],C.black,2.3,false,true);
 poly(s,[[x+464,394],[x+464,444],[x+142,444],[x+142,505]],C.black,2.3,false,true);
 txt(s,'현재 요청',x+320,465,195,27,22,{bold:true,align:'center'});
 poly(s,[[x+418,492],[x+418,505]],C.black,1.8,false,true);
 poly(s,[[x+29,536],[x+7,536],[x+7,362],[x+29,362]],C.black,1.6,true,true);
 txt(s,'미생산·오류·원문 필수 구간은 원본 보완',x+28,567,510,25,19,{align:'center',color:C.muted});

 txt(s,'공통: 최근 대화·채택 상태와 원본 보존, 현재 판본·권한 확인, 부족분 원문 보완, Request Controller의 채택',28,950,1850,31,23,{bold:true});
 txt(s,'각 안 12episode / QA01·03: 초, 괄호는 관측·예정 수 / QA04: 4과제 평균 모듈 수 / 비용: 사전 생산 포함 / RSS: 로컬 실행기',28,989,1840,27,20,{color:C.muted});
 txt(s,'A/B QA01 우열 미확정. 본문 조건 충족은 별도 진단 A/B 30/36, C 34/36. A+C 조합 가능하나 미실험.',28,1024,1810,27,20,{color:C.muted});
 table.cells.block({row:0,column:0,rowCount:10,columnCount:4}).textStyle.lineSpacing=1;
 s.speakerNotes.textFrame.setText(commonNotes()+'\n수정본 구조도는 실제 수명/책임을 표현한다. A는 대화 수명의 유효 작업 문맥 순환과 부족분 읽기 경로, B는 요청 수명 안의 구성/소비와 추가 묶음 해제, C는 별도 생산/게시와 요청 소비 두 경로다. 화살표는 논리적 근거 전달과 처리 순서이며 새로운 별도 process 배치를 뜻하지 않는다. B는 동일 Request Interpreter의 현재 입력 범위 판단과 근거 획득 후 의미 판단을 처리 순서상 두 위치에 표시한다. 서로 다른 Component 또는 모델 복제본을 뜻하지 않는다. 부족하면 추가 원본 조회를 반복할 수 있다. 기본 A 문맥 갱신은 채택 후 Context Manager가 관리하며 Request Controller 채택은 공통 하단에 명시했다.');
}

// 3. Selected A, its observed weak axes, and two independently paired repairs.
{
 const s=pres.slides.add();header(s,' (선택 및 보완)',3);
 txt(s,'같은 자료를 이어 묻는 사용에서 A 선택',28,124,1310,43,32,{bold:true,color:C.blue});
 txt(s,'B 대비 동일 자료 답변 호출 6회 감소  /  C의 사전 생산 포함 총 모델 비용은 A의 2.42배',28,170,1840,31,24);

 // Selected architecture is large and visibly highlights the patch path.
 shape(s,'rect',24,219,610,759,'none',C.blue,3.8);
 shape(s,'rect',24,219,610,52,C.green,'none',0);
 txt(s,'✓',36,207,60,66,54,{bold:true,color:C.red});
 txt(s,'A. 유효 작업 문맥 유지',107,222,500,43,30,{bold:true});
 label(s,'기본 구조: 읽은 근거를 다음 요청에 유지',44,284,570,25);
 storeShape(s,'유효 작업 문맥\n출처와 의존 판본',57,358,238,101,{size:27});
 box(s,'Request Interpreter\n현재 의미 판단',337,338,257,85,{size:24,bold:true});
 txt(s,'현재 요청',357,309,210,27,22,{bold:true,align:'center'});
 poly(s,[[465,336],[465,338]],C.black,2,false,true);
 poly(s,[[295,389],[337,389]],C.black,2.5,false,true);
 box(s,'Context Manager\n유효 문맥 갱신',337,471,257,68,{size:24});
 poly(s,[[465,423],[465,471]],C.black,2.5,false,true);
 poly(s,[[337,505],[175,505],[175,459]],C.black,2.5,false,true);
 txt(s,'채택 후 갱신 → 다음 요청',68,543,526,35,24,{bold:true,align:'center'});

 poly(s,[[47,595],[612,595]],C.blue,1.7);
 txt(s,'A2  정정 시 변경 조각 갱신',53,611,550,43,29,{bold:true,color:C.blue});
 storeShape(s,'원문 변경\nv1 → v2',58,697,171,86,{fill:C.white,size:25});
 box(s,'영향 근거 식별\n변경 부분만 읽기',294,679,300,76,{stroke:C.blue,size:25,color:C.blue});
 box(s,'전체 현재성 재검증',294,799,300,57,{stroke:C.blue,size:25,color:C.blue});
 poly(s,[[229,737],[294,717]],C.blue,2.5,false,true);
 poly(s,[[444,755],[444,799]],C.blue,2.5,false,true);
 poly(s,[[594,827],[615,827],[615,505],[594,505]],C.blue,2.2,false,true);
 txt(s,'영향 범위가 불명확하면 전체 재구성',54,878,545,34,23,{bold:true,color:C.blue,align:'center'});
 txt(s,'파란 경로: 보완안 A2\n대규모 성능 개선으로는 채택하지 않음',56,923,541,44,22,{color:C.muted,align:'center'});

 // The table makes the weakness visible before discussing improvements.
 txt(s,'기본 PoC에서 관측한 A의 QA 약점',673,217,1200,41,29,{bold:true});
 const weak=nativeTable(s,[
   ['QA','A','비교안','해석'],
   ['QA03 처리시간','13.617초','C 12.364초','관측: A가 1.253초 더 걸림'],
   ['QA01 첫 응답','6.609초','B 6.267초','관측 차이 0.342초 / 우열 미확정']
 ],671,267,[263,215,260,488],[43,51,51],25);
 highlightHeader(weak,4);
 cellStyle(weak.getCell(1,1),27,C.red,true);
 cellStyle(weak.getCell(1,2),27,C.black,true);
 cellStyle(weak.getCell(2,3),23,C.muted);

 txt(s,'보완 ①  QA03 — 변경 조각만 읽고 갱신',674,437,1210,41,30,{bold:true,color:C.blue});
 txt(s,'정정 사례 3회에서 기본 A와 A+A2를 비교',674,481,1210,30,23,{color:C.muted});
 const patch=nativeTable(s,[
   ['항목','기본 A','A + A2'],
   ['QA03 처리시간','21.822초','21.433초'],
   ['원본 읽기 / 모델 호출','12회 / 12회','9회 / 12회']
 ],671,520,[444,391,391],[40,57,43],25);
 highlightHeader(patch,3);
 patch.cells.block({row:0,column:0,rowCount:1,columnCount:3}).assign({fill:C.pale});
 cellStyle(patch.getCell(1,1),34,C.black,true);cellStyle(patch.getCell(1,2),34,C.blue,true);
 txt(s,'정정 사례 관측: 0.389초 감소. 재조회 감소만 확인했으며 전체 A/C 재비교는 미실시.',674,667,1210,31,23,{color:C.muted});

 txt(s,'보완 ②  QA01 — 채택한 근거·처리 의도를 먼저 안내',674,723,1210,41,29,{bold:true,color:C.blue});
 txt(s,'A/B/C 공통 보완을 A에 적용한 별도 E3 비교',674,765,1210,30,23,{color:C.muted});
 txt(s,'첫 응답',674,812,245,53,28,{bold:true});
 txt(s,'7.092초',924,801,294,66,45,{bold:true});
 txt(s,'→',1231,801,95,66,47,{color:C.blue,align:'center'});
 txt(s,'1.905초',1350,801,437,66,48,{bold:true,color:C.blue});
 txt(s,'대가  QA03 21.508 → 23.118초   /   모델 호출 12 → 18회',674,881,1210,35,25,{color:C.red,bold:true});
 txt(s,'모델 비용 $0.01466 → $0.01574. 본문 조건 충족 3/6은 그대로.',674,923,1210,33,24,{color:C.muted});

 poly(s,[[27,997],[1894,997]],C.blue,1.5);
 txt(s,'보완 전후는 각각 동일 조건의 별도 비교값이며 기본 비교표의 A 수치를 교체하지 않는다. 두 보완의 결합 효과는 미측정.',28,1010,1815,31,22,{bold:true});
 txt(s,'원격 모델 PoC의 제한된 관측값. 첫 안내는 callback proxy 기준이며 제품 UI 끝단 성능을 뜻하지 않는다.',28,1046,1780,25,20,{color:C.muted});
 s.speakerNotes.textFrame.setText(commonNotes()+'\n선택은 후속 근거 재사용 조건의 발표상 권고이며 accepted ADR을 변경하지 않는다. QA01의 A/B 구조 우열은 미확정이다. A2는 정정 사례에서 재조회 감소만 확인했으며 주요 성능 보완으로 채택하지 않았다.\nA2 비교는 정정 E3×3회이며 기본A 대비 A2의 QA03은21.822→21.433초, 원문읽기12→9, 모델호출12→12, 비용0.0148424→0.0145860USD, 본문3/6동일이다. 전체 A대C 비교를 다시 수행하지 않았으므로 C와의 격차 해소로 확대하지 않는다. 필수14KB hard-cap 수정은 선택적 tactic의 효과가 아니다. A3 작은 보존 예산은 기본 적용 보류다.\n공통 첫 채택 안내는 E3 ABC×2정책×3회=18episode의 별도 실험이다. 여기서는 A만 확대해 표시한다. A QA01은7.092→1.905초, QA03은21.508→23.118초, 호출12→18, 비용0.0146552→0.0157364USD. B QA01은6.724→1.829, C는17.730→2.322초다. 이 정책은 A 독점의 장점이 아니며 A/B 순위 역전을 입증하지 않는다. 각 안/정책의 첫 응답·host종료·QA02는6/6, 본문은A/B3/6,C6/6이다. 유지 view가 없거나 정정으로 무효화되어 A재사용을 시험하지 않는다. A/B의 첫 입력은12개 짝비교에서 같다. C사전생산은정책별독립실행이라 순수정책효과와구분한다. 첫안내는실제source와처리의도채택이며완료선언이아니다. 실제내부intent라벨이노출된callback proxy이며다듬은제품UI문구의실측이아니다. A2와공통첫안내는별도실험이고결합효과는미측정이다.\n수치 출처: '+ROOT+'/docs/architecture/12-decisions/decision-packages/seven-problem-review/17-presentation-results.md §3\n'+ROOT+'/results/architecture-evaluation/current/four-dp-presentation-20261019/dp01/presentation-report.md');
}
await fs.mkdir(path.join(BUILD,'previews'),{recursive:true});
await fs.mkdir(path.join(BUILD,'output'),{recursive:true});
for(let i=0;i<pres.slides.items.length;i++){
 const slide=pres.slides.items[i];
 const png=await pres.export({slide,format:'png',scale:1});
 await fs.writeFile(path.join(BUILD,'previews','slide-'+(i+1)+'.png'),new Uint8Array(await png.arrayBuffer()));
 const layout=await slide.export({format:'layout'});
 await fs.writeFile(path.join(BUILD,'previews','slide-'+(i+1)+'.layout.json'),await layout.text());
}
const draft=path.join(BUILD,'candidate.pptx');
await(await PresentationFile.exportPptx(pres)).save(draft);
const {finalizePresentation}=await import(pathToFileURL(path.join(SKILL,'container_tools/artifact_tool_utils.mjs')).href);
const out=path.join(BUILD,'output','VIA-DP1-reference-style-v2.pptx');
const result=await finalizePresentation({workspaceDir:BUILD,candidatePath:draft,finalPath:out,pythonExecutable:PY,integrityValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_layout_geometry.py'),explicitTotalSlideCount:3,requiredNativeTableOwnerSlides:[2,3],fontPolicy:{basis:'design',families:[FONT]},verifyArtifactToolImport:true,layoutArgs:['--expected-slide-size-emu',String(W*9525)+','+String(H*9525),'--validate-bullet-geometry','--validate-heading-fit','--require-native-table-slide','2','--require-native-table-slide','3'],receiptPath:path.join(BUILD,'validation.json')});
console.log(JSON.stringify({output:out,result},null,2));
