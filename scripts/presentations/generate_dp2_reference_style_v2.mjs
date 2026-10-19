import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {Presentation, PresentationFile} from '@oai/artifact-tool';

// DP2 four-slide editable deck using the approved DP1 reference layout. Run a copy from a private build
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
  return s.shapes.add({name:name||'DP2 object '+(++serial),geometry:geom,position:{left:x,top:y,width:w,height:h},fill,line:line(stroke,lw)});
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
  txt(s,'DP2. 후속 입력의 의미 판단 실행 방안'+suffix,60,29,1200,68,48,{bold:true});
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

function notes(extra=''){
 return 'DP2는 후속 입력의 의미 판단 실행 구조다. A는 현재 모델/읽기 step 경계에서 새 입력을 같은 대화 실행에 반영한다. 앞선 요청 전체나 외부 Agent 업무 완료까지 기다리는 구조가 아니다. B는 Request Interpreter의 공통 대화 해석과 요청별 논리 실행을 분리한다. 짧은 답은 공통 경로에서 직접 가능하며 모든 입력에 router+worker 두 호출을 강제하지 않는다. 논리 실행은 Component, OS process, 모델 복제를 뜻하지 않는다. 두 안은 동일 Context Manager 근거 정책과 Model Access 호출 한도를 쓴다. Request Controller가 input/revision/실행 대응, 현재성 검증과 채택을 맡는다. Response Manager가 응답 준비와 게시 원장, Interaction Manager가 실제 표시, Task Manager가 외부 Task 사실을 소유한다.\n기본값은 2026-10-19 발표 캠페인의 기존 실제 원격 모델 PoC다. 안별 4개 작성 사례×3회=12episode, QA01 30개 예정 입력/통지와 QA03 24개 예정 목표는 다르다. 시간 괄호는 관측/예정이며 A QA03 23/24의 누락을 삭제하거나 벌점으로 대체하지 않는다. QA02는 지칭 집합으로 이 비교에서 N/A이며 N-R 요청 관계 진단은 별도다. QA04는 별도 네 Rust 변경 과제 평균. QA05는 12episode의 전체 usage에 입력$0.40/M·출력$1.60/M 고정 단가를 적용한 비용 합. QA06은 같은 시각 Python harness+Rust RSS 최고이며 제품 UI·원격 모델 weights/KV를 제외한다. N-R은 A27/27, B26/27이고 B는 불필요한 메일 Task 메타데이터를 추가했다. 실제 잘못된 메일 발송은 없었다. 전체 제품/on-device Omni 결과나 승인된 ADR로 확대하지 않는다. 새로운 공통9 평가 준비와 부분 조건 진단을 이 결과에 합치지 않는다.\nB1은 별도 순서 제어 응답 재생이다. N=1/2/4마다 안별12회, 전체72회. 새 API 0회, 기존 응답686개 재생. source 약7.5KB, padding 없음. 불변 input/source와 wire payload 공유/빌린 직렬화만 바꾸고 요청별 mutable 상태, revision, 대화 snapshot과 복구를 유지한다. 결과와 callback72/72, 실제 model 입력/instruction/max_tokens168/168 session 동일. 재생 순서를 보정한 strict-control-v2 정본을 사용한다. QA06 최고와 trial peak 평균, 요청 할당 byte를 구분한다. B1의 새 QA01/03나 모델 비용 개선은 미측정이며 N-R 오류도 그대로 보존했다. 실제 기본46.170MB와 별도 재생31.867MB를 전후로 연결하지 않는다.\n내용 출처:\n'+ROOT+'/docs/architecture/12-decisions/decision-packages/seven-problem-review/02-interpretation-execution.md\n'+ROOT+'/docs/architecture/12-decisions/decision-packages/seven-problem-review/15-corrected-poc-results.md §DP02\n'+ROOT+'/docs/architecture/12-decisions/decision-packages/seven-problem-review/17-presentation-results.md §2~4\n'+ROOT+'/results/architecture-evaluation/current/four-dp-presentation-20261019/dp02/strict-control-v2/report.md\n스타일 출처: DP1 v3 및 사용자 첨부 23-DP04방안, 24-DP04채택, 18-DP02채택.\n'+extra;
}
function label(s,t,x,y,w,size=22,color=C.black){return txt(s,t,x,y,w,32,size,{color,bold:true,align:'center'});}
function cellStyle(cell,size,color=C.black,bold=false,align='center'){
 cell.text.style={typeface:FONT,fontSize:size,color,bold,alignment:align,verticalAlignment:'middle',lineSpacing:1};
}
function highlightHeader(table,n){table.cells.block({row:0,column:0,rowCount:1,columnCount:n}).assign({fill:C.green,textStyle:{typeface:FONT,fontSize:25,bold:true,color:C.black,alignment:'center'}});}
function sw(s,name,body,x,y,w,h,{blue=false,size=23}={}){
 shape(s,'rect',x,y,w,h,C.cream,C.black,2.7,name+' Component');
 txt(s,name,x+7,y+2,w-14,30,25,{bold:true,align:'center'});
 poly(s,[[x,y+34],[x+w,y+34]],C.black,1.5);
 txt(s,body,x+10,y+36,w-20,h-39,size,{align:'center',color:blue?C.blue:C.black});
}

// 1. One visible conflict: the new input arrives during an ongoing VIA step.
{
 const s=pres.slides.add();header(s,' (배경)',1);
 txt(s,'앞선 판단이 끝나기 전에 새 질문이 들어온다',66,145,1785,62,43,{bold:true});
 txt(s,'VIA는 입력을 계속 받으면서 원래 목표와 새 요청을 함께 이어가야 한다.',68,211,1760,40,27);
 poly(s,[[1350,292],[1350,850]],C.blue,1.3);

 box(s,'“문서 D1과 D2를 비교해줘.”',92,304,490,91,{fill:C.white,size:31,bold:true});
 shape(s,'roundRect',664,303,616,127,C.pale,C.blue,2.3);
 txt(s,'새 독립 질문',687,315,558,30,23,{bold:true,color:C.blue});
 txt(s,'“그건 계속하고,\nTCP와 UDP 차이도 알려줘.”',687,350,558,65,30,{bold:true,align:'center'});
 poly(s,[[336,395],[336,490]],C.black,2.2,false,true);
 poly(s,[[770,430],[770,681]],C.blue,2.1,true,true);

 txt(s,'VIA의 처리',90,449,245,37,25,{bold:true});
 shape(s,'rect',212,490,1070,96,C.cream,C.black,2);
 txt(s,'문서 읽기와 진행 중 의미 판단',235,507,1025,42,31,{bold:true,align:'center'});
 txt(s,'현재 모델 / 읽기 step',235,549,1025,27,22,{align:'center',color:C.muted});
 poly(s,[[1282,490],[1282,749]],C.gray,1.7,true);
 txt(s,'현재 step 경계',1090,600,225,31,23,{bold:true,align:'center'});
 txt(s,'입력 수신',90,669,220,38,26,{bold:true});
 box(s,'새 입력은 즉시 보존',642,682,275,61,{fill:C.pale,stroke:C.blue,size:26,bold:true,color:C.blue});
 poly(s,[[917,712],[1282,712]],C.red,3,true,true);
 txt(s,'이 대기를 줄일 수 있는가?',935,665,345,34,25,{bold:true,color:C.red,align:'center'});
 poly(s,[[213,801],[1290,801]],C.black,1.7,false,true);
 txt(s,'시간',1234,813,75,29,22,{align:'right'});
 txt(s,'수신과 의미 판단 시작은 다른 사건',216,823,935,35,26,{bold:true,color:C.blue});

 txt(s,'무엇이 문제인가?',1398,310,440,50,31,{bold:true});
 n(s,1,1404,406);txt(s,'같은 실행에서 기다리면',1452,399,383,42,27,{bold:true});
 txt(s,'새 질문의 응답이\n현재 판단에 묶인다',1406,467,427,91,32,{bold:true,color:C.red});
 poly(s,[[1406,585],[1834,585]],C.grid,1.5);
 n(s,2,1404,631);txt(s,'실행을 나누면',1452,624,383,42,27,{bold:true});
 txt(s,'요청별 문맥과\n반환 상태가 늘어난다',1406,693,427,91,32,{bold:true,color:C.red});
 poly(s,[[63,895],[1795,895]],C.blue,1.5);
 txt(s,'이 DP의 질문',66,930,285,55,33,{bold:true,color:C.red});
 txt(s,'새 입력을 같은 해석 실행에 넣을까, 요청별로 나눌까?',376,924,1430,62,37,{bold:true});
 txt(s,'원래 목표를 보존하며, 새 입력의 판단을 시작하는 실행 구조를 선택한다.',376,992,1450,43,27);
 s.speakerNotes.textFrame.setText(notes('배경은 정량 시간축이 아닌 사건 순서의 개념도다. 긴 막대는 VIA 모델/읽기 step이며 외부 Agent 업무 대기가 아니다. 새 독립 질문이 현재 step 중 도착하는 S2/E2 상황을 단순화했다. 입력이 도착한다고 즉시 의미 관계가 확정되는 것은 아니다.'));
}

// 2 and 3. Same A/B table; only B's selected improvement changes.
function comparison(selected){
 const s=pres.slides.add();header(s,selected?' (선택 및 보완)':' (설계 방안 비교)',selected?3:2);
 txt(s,selected?'독립 입력이 앞선 VIA 판단과 겹치고 호출 여유가 있는 조건에서 B 선택':'A: 하나의 대화 해석 실행   /   B: 공통 대화 해석과 요청별 실행',28,124,1835,39,28,{bold:true,color:selected?C.blue:C.black});
 const vals=[
 ['설계안','(A) 하나의 대화 해석 실행','(B) 공통 대화 해석 + 요청별 실행'],
 ['구조','',''],
 ['장점','같은 문맥의 근거와 잠정 판단을 연속 사용\n실행 대응과 반환 상태가 단순','호출 여유가 있으면 긴 판단 중 새 입력 해석\n독립 요청의 읽기·판단을 겹쳐 진행'],
 ['단점','후속 의미 판단이 현재 step 경계를 기다림\n앞선 요청 전체 완료를 기다리는 것은 아님','실행별 문맥·상태와 반환 관리가 추가됨\n호출 한도가 차면 병행 이익이 줄어듦'],
 ['QA01 첫 응답 (초)','8.677 (30/30)','6.698 (30/30)'],
 ['QA02 지칭 집합','N/A','N/A'],
 ['QA03 처리 (초)','24.092 (23/24)','20.982 (24/24)'],
 ['QA04 변경 (개)','2.00','2.00'],
 ['QA05 모델 비용 ($)','0.1224560','0.1125156'],
 ['QA06 RSS (MB)','45.351','46.170']];
 const t=nativeTable(s,vals,24,174,[172,850,850],[46,426,58,78,32,32,32,32,32,56],20);
 highlightHeader(t,3);
 t.cells.block({row:1,column:0,rowCount:1,columnCount:1}).assign({textStyle:{typeface:FONT,fontSize:29,bold:true},anchor:'top',margins:{top:16,left:12,right:12,bottom:0}});
 t.cells.block({row:2,column:0,rowCount:2,columnCount:1}).assign({textStyle:{typeface:FONT,fontSize:25,bold:true}});
 t.cells.block({row:3,column:1,rowCount:1,columnCount:2}).assign({textStyle:{typeface:FONT,fontSize:21,color:C.red,alignment:'center'}});
 t.cells.block({row:4,column:0,rowCount:6,columnCount:1}).assign({fill:'#F7F8FA',textStyle:{typeface:FONT,fontSize:19,bold:true,alignment:'left'}});
 // Legend distinguishes software ownership, execution instances and state.
 shape(s,'rect',40,291,139,68,C.cream,C.black,2.7);txt(s,'Component',44,293,130,27,20,{bold:true,align:'center'});
 shape(s,'rect',49,325,121,25,C.white,C.black,1.3);txt(s,'내부 처리',51,325,117,25,18,{align:'center'});
 const eg=shape(s,'rect',40,383,139,44,'none',C.black,1.8);eg.line=line(C.black,1.8,'dashed');
 txt(s,'논리 실행',43,386,133,35,20,{align:'center'});
 storeShape(s,'보존 데이터',42,449,135,55,{size:19});
 txt(s,'R1·R2는 논리 실행\n모델 복제 아님',35,515,151,43,17,{align:'center',color:C.muted});
 txt(s,'실선: 요청·반환\n원통: 상태 / 입력',35,567,151,42,17,{align:'center',color:C.muted});
 if(selected){shape(s,'rect',42,621,22,18,C.white,C.blue,2.7);txt(s,'보완 B1',73,617,113,27,19,{bold:true,color:C.blue});}

 const firstDiagramShape=s.shapes.items.length;
 // A: a single execution owns one evolving interpretation context.
 label(s,'대화 실행 1개와 문맥 1개를 이어 사용',207,230,828,25);
 shape(s,'rect',224,275,794,92,C.cream,C.black,2.7,'A Request Controller Component');
 txt(s,'Request Controller',236,278,770,30,27,{bold:true,align:'center'});
 poly(s,[[224,310],[1018,310]],C.black,1.4);
 box(s,'입력 보존 / 경계에 해석 요청',237,318,253,41,{fill:C.white,size:20});
 storeShape(s,'채택 목표 / revision',512,317,226,42,{size:19});
 box(s,'현재성 확인·채택',762,318,243,41,{fill:C.white,size:21});
 poly(s,[[400,367],[400,391]],C.black,2.3,false,true);
 poly(s,[[917,391],[917,367]],C.black,2.3,false,true);
 txt(s,'해석 요청',410,367,150,25,18);txt(s,'결과 반환',768,367,139,25,18,{align:'right'});
 shape(s,'rect',224,391,794,253,C.cream,C.black,2.7,'A Request Interpreter Component');
 txt(s,'Request Interpreter',237,394,768,30,27,{bold:true,align:'center'});
 poly(s,[[224,427],[1018,427]],C.black,1.4);
 const aRun=shape(s,'rect',244,438,754,187,'none',C.black,1.8,'A single conversation execution');aRun.line=line(C.black,1.8,'dashed');
 txt(s,'대화 해석 실행 1개',260,444,723,34,26,{bold:true,align:'center'});
 box(s,'현재 step 종료 뒤\n새 입력의 관계·내용 판단',281,491,280,70,{fill:C.white,size:23});
 storeShape(s,'한 대화의 근거와\nR1·R2 잠정 판단',646,484,320,88,{size:23});
 poly(s,[[646,509],[561,509]],C.black,2.1,false,true);
 poly(s,[[561,548],[646,548]],C.black,2.1,false,true);
 txt(s,'조회',569,486,68,22,17,{align:'center'});txt(s,'갱신',569,549,68,22,17,{align:'center'});
 poly(s,[[420,561],[420,603],[264,603],[264,478],[420,478],[420,491]],C.black,2,false,true);
 txt(s,'남은 목표와 새 입력을 같은 실행에서 계속',443,587,534,31,22,{bold:true,align:'center'});

 // B: control keeps execution correspondence; Interpreter separates contexts.
 label(s,'공통 대화 + 요청별 실행과 문맥 + 반환 관리',1056,230,830,25);
 shape(s,'rect',1074,275,244,369,C.cream,C.black,2.7,'B Request Controller Component');
 txt(s,'Request\nController',1085,283,222,60,27,{bold:true,align:'center'});
 poly(s,[[1074,343],[1318,343]],C.black,1.4);
 box(s,'입력 보존 / 실행 요청',1087,353,218,42,{fill:C.white,size:20});
 storeShape(s,'입력 / 실행 ID 대응\n목표·revision\n변경·완료 상태',1087,415,218,98,{size:21});
 box(s,'개별 반환 검증·채택\n필요한 재결합 요청',1087,548,218,81,{fill:C.white,size:21});
 poly(s,[[1196,395],[1196,415]],C.black,1.9,false,true);
 poly(s,[[1196,513],[1196,548]],C.black,1.9,false,true);

 shape(s,'rect',1360,275,508,369,C.cream,C.black,2.7,'B Request Interpreter Component');
 txt(s,'Request Interpreter',1371,278,486,30,27,{bold:true,align:'center'});
 poly(s,[[1360,310],[1868,310]],C.black,1.4);
 box(s,'공통 대화 해석\n관계 판단·확인 질문\n짧은 답은 직접 반환',1374,324,278,83,{fill:C.white,size:21});
 storeShape(s,'최근 문맥\n요청 참조',1674,324,180,83,{size:22});
 poly(s,[[1674,366],[1652,366]],C.black,1.7,false,true);
 poly(s,[[1318,374],[1374,374]],C.black,2.1,false,true);
 // The common path and each request return separately, without a join barrier.
 poly(s,[[1374,394],[1332,394],[1332,561],[1318,561]],C.black,2.1,false,true);
 poly(s,[[1513,407],[1513,426],[1488,426],[1488,446]],C.black,2.1,false,true);
 poly(s,[[1513,426],[1740,426],[1740,446]],C.black,2.1,false,true);
 const r1=shape(s,'rect',1374,446,228,142,'none',C.black,1.8,'B R1 logical execution');r1.line=line(C.black,1.8,'dashed');
 const r2=shape(s,'rect',1626,446,228,142,'none',C.black,1.8,'B R2 logical execution');r2.line=line(C.black,1.8,'dashed');
 txt(s,'R1 해석 실행',1384,451,208,29,23,{bold:true,align:'center'});
 txt(s,'R2 해석 실행',1636,451,208,29,23,{bold:true,align:'center'});
 box(s,'근거 읽기·긴 판단',1386,485,204,34,{fill:C.white,size:20});
 box(s,'근거 읽기·긴 판단',1638,485,204,34,{fill:C.white,size:20});
 storeShape(s,'R1 문맥·잠정 상태\n대기 / revision',1386,531,204,51,{size:18});
 storeShape(s,'R2 문맥·잠정 상태\n대기 / revision',1638,531,204,51,{size:18});
 poly(s,[[1374,502],[1346,502],[1346,589],[1318,589]],C.black,2.1,false,true);
 poly(s,[[1854,502],[1880,502],[1880,651],[1328,651],[1328,615],[1318,615]],C.black,2.1,false,true);
 if(selected){
  storeShape(s,'B1 공유 불변 입력·원본',1410,601,420,35,{size:19,stroke:C.blue,color:C.blue});
  poly(s,[[1488,601],[1488,588]],C.blue,2.3,false,true);
  poly(s,[[1740,601],[1740,588]],C.blue,2.3,false,true);
 }else{
  storeShape(s,'불변 입력 복사①',1386,601,204,35,{size:18,fill:C.white});
  storeShape(s,'불변 입력 복사②',1638,601,204,35,{size:18,fill:C.white});
  poly(s,[[1488,601],[1488,588]],C.black,2.1,false,true);
  poly(s,[[1740,601],[1740,588]],C.black,2.1,false,true);
 }


 for(const object of s.shapes.items.slice(firstDiagramShape)){
  const p=object.position;if(p.top>=275)object.position={left:p.left,top:p.top-12,width:p.width,height:p.height};
 }
 if(selected){
  shape(s,'rect',1046,174,850,824,'none',C.blue,4.2,'Selected B column');
  txt(s,'✓',1054,150,67,79,65,{color:C.red,bold:true});
  const con=t.getCell(3,2);con.value+='\n[QA06 보완] B1 불변 입력 공유 / 개별 변경 상태는 유지';cellStyle(con,21,C.red);
  con.text.get('[QA06 보완] B1 불변 입력 공유 / 개별 변경 상태는 유지').fill=C.blue;
  const qa=t.getCell(9,2);qa.value='46.170 (기본)\n[별도 재생 N=4] 32.326 → 31.867 MB';cellStyle(qa,20);
  qa.text.get('46.170 (기본)').fill=C.red;qa.text.get('[별도 재생 N=4] 32.326 → 31.867 MB').fill=C.blue;
  txt(s,'B1은 작은 로컬 복제 부담 보완. N=1 최고 RSS는 증가. 전체 A/B 재비교와 새 QA01·QA03 측정은 미실시.',28,1007,1810,27,21,{color:C.blue});
  txt(s,'기본값과 재생값은 다른 실행기. 시간 괄호: 관측/예정. N별 전체 전후 수치와 남은 관계 오류는 4장에 표시.',28,1040,1805,27,20,{color:C.muted});
 }else{
  txt(s,'모든 해석 실행은 동일 Model Access 호출 한도 K를 공유. 입력 수신은 계속되며 외부 Agent 업무 대기는 비교 대상이 아님.',28,1007,1830,27,21,{bold:true});
  txt(s,'기존 발표 PoC: 안별 12episode / 시간 괄호: 관측/예정 / QA04 별도 4과제 / 비용은 합계 / RSS는 로컬 실행기',28,1040,1805,27,20,{color:C.muted});
 }
 t.cells.block({row:0,column:0,rowCount:10,columnCount:3}).textStyle.lineSpacing=1;
 s.speakerNotes.textFrame.setText(notes('B 기본 도식의 입력 복사는 측정 후보의 실제 불변 input/source/wire 복제 경로를 나타낸다. B 구조 자체가 전체 대화를 무조건 복제해야 한다는 주장은 아니다. 그림은 의미 실행 구간의 확대도이며 공통 Context Manager와 Model Access, 응답 전달 Components는 생략한다. A는 하나의 대화 실행과 유지 문맥을 순환 구조로 표현했다. B는 입력/실행 대응과 개별 반환 검증을 Request Controller에, 공통 대화와 R1/R2의 논리 실행/잠정 문맥을 Request Interpreter에 배치했다. 각 실행의 반환은 개별 경로이며 모든 실행 완료를 기다리는 join 장벽이 아니다.'));
}
comparison(false);comparison(true);

// 4. Conditions for selection and the complete B1 replay evidence.
{
 const s=pres.slides.add();header(s,' (보완 효과 상세)',4);
 txt(s,'병행 이익은 호출 여유에서, B1의 효과는 입력 복제 감소에서 나온다',28,126,1840,44,30,{bold:true,color:C.blue});
 txt(s,'기본 구조 선택의 실제 모델 관측과 보완의 응답 재생을 구분한다.',28,175,1820,32,24);

 // Left: editable before/after mechanism, not new software Components.
 shape(s,'rect',24,218,612,762,'none',C.blue,3);
 shape(s,'rect',24,218,612,51,C.green,'none',0);
 txt(s,'B1. 불변 input/source 공유',44,222,572,42,30,{bold:true,align:'center'});
 txt(s,'기본 B: 실행마다 불변 입력을 복사',44,285,572,38,27,{bold:true,align:'center'});
 shape(s,'rect',52,341,252,142,C.white,C.black,2,'R1 logical execution');
 shape(s,'rect',354,341,252,142,C.white,C.black,2,'R2 logical execution');
 txt(s,'R1 해석 실행',61,347,234,32,25,{bold:true,align:'center'});txt(s,'R2 해석 실행',363,347,234,32,25,{bold:true,align:'center'});
 storeShape(s,'입력·원본 복사①',65,397,226,70,{size:23});storeShape(s,'입력·원본 복사②',367,397,226,70,{size:23});
 poly(s,[[330,493],[330,565]],C.blue,3,false,true);
 txt(s,'불변 자료를 한 번 보존',57,513,249,35,25,{bold:true,color:C.blue});
 storeShape(s,'B1 공유 불변 입력·원본',134,578,392,81,{size:27,color:C.blue,stroke:C.blue});
 poly(s,[[240,659],[240,701],[178,701],[178,737]],C.blue,2.6,false,true);
 poly(s,[[420,659],[420,701],[480,701],[480,737]],C.blue,2.6,false,true);
 txt(s,'참조',92,680,85,32,22,{color:C.blue,align:'center'});txt(s,'참조',493,680,85,32,22,{color:C.blue,align:'center'});
 box(s,'R1 개별 상태\nrevision·snapshot',52,737,252,111,{fill:C.white,size:26});
 box(s,'R2 개별 상태\nrevision·snapshot',354,737,252,111,{fill:C.white,size:26});
 txt(s,'변경 가능한 상태와 복구 규칙은 유지',43,881,575,37,26,{bold:true,align:'center'});
 txt(s,'공유 입력을 빌려 payload 직렬화',43,931,575,32,23,{color:C.blue,align:'center'});

 txt(s,'B 선택이 성립하는 조건',671,218,1216,42,30,{bold:true});
 const cond=nativeTable(s,[['조건과 지표','A','B','확인한 실행 경로'],['K=2 독립 질문 / 첫 응답','13.388초','6.270초','실제 API 구간 6.261초 중첩'],['K=1 독립 질문 / 후속 완료','13.429초','13.087초','호출 중첩 0 / 병행 이익 없음']],671,269,[400,190,190,444],[43,54,54],23);
 highlightHeader(cond,4);
 txt(s,'QA06 보완: 별도 순서 제어 재생의 최고 동시 RSS',671,440,1216,45,29,{bold:true,color:C.blue});
 const mem=nativeTable(s,[['동시 문맥 N','기본 B (MB)','B + B1 (MB)','변화 (MB)'],['1','30.097','30.294','+0.197'],['2','30.802','30.507','−0.295'],['4','32.326','31.867','−0.459']],671,498,[230,320,320,354],[44,57,57,57],26);
 highlightHeader(mem,4);
 cellStyle(mem.getCell(1,3),26,C.red,true);
 cellStyle(mem.getCell(2,3),26,C.blue,true);cellStyle(mem.getCell(3,3),26,C.blue,true);
 txt(s,'N=4 요청 할당 peak 평균: 573,372 → 365,413 byte',674,730,1210,39,27,{bold:true,color:C.blue});
 txt(s,'결과·callback 72/72 동일 / 모델 입력 snapshot 168/168 session 동일',674,784,1210,37,25,{bold:true});
 txt(s,'N별·안별 12회, 전체 72회. 새 API 0회, 기존 응답 686개 재생.',674,831,1210,34,24);
 txt(s,'N=1 최고 RSS는 증가. 새 QA01·QA03와 모델 비용 개선은 미측정.',674,879,1210,34,24,{color:C.red});
 txt(s,'별도 요청 관계 진단 N-R: A 27/27, B 26/27. B1에서도 B의 오류는 그대로.',674,927,1210,34,23,{color:C.muted});
 poly(s,[[28,997],[1894,997]],C.blue,1.4);
 txt(s,'기본 실측 46.170MB와 재생 31.867MB는 전후값이 아님. 제품 전체 RAM, 원격 모델 weights/KV와 새 지연을 측정한 결과가 아님.',28,1010,1810,31,21,{bold:true});
 txt(s,'기존 발표 캠페인과 strict-control-v2 재생 근거. 새로운 공통 대표 평가의 정식 결과와 구분한다.',28,1044,1800,26,19,{color:C.muted});
 s.speakerNotes.textFrame.setText(notes('K=2/K=1 조건표는 각 같은 사례의 3회 평균이며 K=2는 첫 응답, K=1은 후속 목표 완료로 지표가 다르므로 행 간 직접 속도 비교를 하지 않는다. 각 N별 최고 RSS는 해당 안12회 중 최고 동시 Python+Rust 값이다. N=4 실제값32.325632→31.866880MB, N=1 30.097408→30.294016MB. 감소/증가는 반올림 전 값으로 계산해0.459/0.197MB로 표시한다. N=2 감소0.294912MB. 요청 할당 peak 평균은573372.3→365412.7byte이며 RSS와 같은 지표가 아니다. 전송 payload snapshot동등성을 확인했지만 새로운 모델 정확도나 원격 latency를 측정한 것은 아니다.'));
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
const out=path.join(BUILD,'output','VIA-DP2-reference-style-v2.pptx');
const result=await finalizePresentation({workspaceDir:BUILD,candidatePath:draft,finalPath:out,pythonExecutable:PY,integrityValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_layout_geometry.py'),explicitTotalSlideCount:4,requiredNativeTableOwnerSlides:[2,3,4],fontPolicy:{basis:'design',families:[FONT]},verifyArtifactToolImport:true,layoutArgs:['--expected-slide-size-emu',String(W*9525)+','+String(H*9525),'--validate-bullet-geometry','--validate-heading-fit','--require-native-table-slide','2','--require-native-table-slide','3','--require-native-table-slide','4'],receiptPath:path.join(BUILD,'validation.json')});
console.log(JSON.stringify({output:out,result},null,2));
