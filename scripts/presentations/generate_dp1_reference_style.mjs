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

// Slide 1: concrete interaction, actual evidence, and resource/currentness tension.
{
 const s=pres.slides.add();header(s,' (배경)',1);
 const xx=[64,690,1308];
 [655,1260].forEach(x=>poly(s,[[x,157],[x,865]],C.blue,1));
 txt(s,'■ 이어지는 질문과 정정',xx[0],155,560,52,33,{bold:true});
 txt(s,'■ 여러 원본에 흩어진 근거',xx[1],155,540,52,33,{bold:true});
 txt(s,'■ 반복 구성과 유지 비용',xx[2],155,550,52,33,{bold:true});
 const steps=[['1','최초 질문','두 제품의 가격을 비교해줘'],['2','후속 질문','납기와 유지보수까지 보면?'],['3','원문 정정','견적의 납기 조건이 변경됨'],['4','재질문','수정된 조건으로 다시 비교해줘']];
 steps.forEach(([id,title,body],i)=>{
   const yy=245+i*142;n(s,id,xx[0]+2,yy+15);
   txt(s,title,xx[0]+48,yy,500,39,27,{bold:true});
   box(s,body,xx[0]+48,yy+46,485,62,{fill:i===2?C.cream:C.white,size:26,stroke:i===2?C.greenD:C.black});
   if(i<3)poly(s,[[xx[0]+18,yy+54],[xx[0]+18,yy+139]],C.black,1.8,false,true);
 });
 txt(s,'같은 자료가 다시 필요하지만\n정정 전 조건을 그대로 사용할 수 없음',xx[0]+48,827,500,58,24,{bold:true});
 doc(s,'VIA가 실제로 전달한 이전 설명','가격이 같으면\n유지보수 기간을 우선',xx[1]+5,246,505,154);
 doc(s,'제품 비교 결과','제품별 가격과 조건\n읽은 범위와 출처',xx[1]+5,442,505,154);
 doc(s,'현재 견적','원문 판본 v1 → v2\n납기 조건의 변경 위치',xx[1]+5,638,505,154);
 txt(s,'원문 참조 + 실제 전달 범위 + 변경 판본',xx[1]+5,826,530,46,24,{bold:true});
 const reasons=[
  ['매 요청마다 다시 구성','같은 자료를 다시 찾고 읽으며\n근거를 조합하는 대기가 반복됨'],
  ['이전에 읽은 근거를 유지','주제가 바뀌면 불필요한 입력이 남고\n원문 정정 뒤 갱신이 필요함'],
  ['관계를 미리 생산','반복 조회에는 활용할 수 있지만\n사용하지 않는 생산 비용도 발생함']
 ];
 reasons.forEach(([title,body],i)=>{
   const yy=249+i*194;
   shape(s,'rect',xx[2]+4,yy+29,490,19,C.green,'none',0);
   txt(s,title,xx[2]+4,yy,510,50,29,{bold:true});
   txt(s,body,xx[2]+4,yy+70,520,80,25);
 });
 txt(s,'현재 판본과 권한 확인은 모든 안의 공통 조건',xx[2]+4,826,535,46,23,{bold:true});
 poly(s,[[62,914],[1859,914]],C.blue,1.5);
 txt(s,'Challenge:',65,947,260,52,37,{bold:true,color:C.red});
 txt(s,'1) 후속 요청에 필요한 근거의 반복 탐색·구성을 줄일 방안 필요',340,940,1500,49,31,{bold:true});
 txt(s,'2) 원문 변경을 반영하며 불필요한 유지·사전 생산 비용을 제한할 방안 필요',340,995,1500,49,31,{bold:true});
 s.speakerNotes.textFrame.setText('배경은 설계 문제를 설명하는 예시이며 새 실험 결과가 아니다. 연속 질문, 주제 전환, 원문 정정, 다른 대화의 근거 재소비를 함께 고려한다. 실제 전달 기록을 준비한 전체 답변과 혼동하지 않는다. 원본의 소유권과 현재 권한은 모든 구조에 공통이다.\n내용 출처: '+ROOT+'/docs/architecture/12-decisions/decision-packages/seven-problem-review/01-evidence-supply.md §1~4\n스타일 참조: 첨부 22-DP04배경.png 및 19-DP03배경.png. 원본 그림을 붙이지 않고 설명 도형과 텍스트를 새로 만들었다.');
}

const values=[
 ['설계안','(A) 유효 작업 문맥 유지','(B) 요청마다 원본 구성','(C) 별도 근거 생산·게시'],
 ['구조','','',''],
 ['장점','후속 질문에서 같은 근거의\n발견과 조합 반복 감소','현재 목적에 필요한 범위로 구성\n이전 해석 문맥의 유지 부담 감소','대화가 바뀌어도 게시한 근거와\n관계를 반복 활용'],
 ['단점','주제 전환 후 불필요한 입력\n원문 정정에 따른 문맥 갱신','같은 자료를 이어 물으면\n근거의 재조회와 재조합','사전 생산과 갱신 비용\n미사용 생산 및 원본 보완 비용'],
 ['QA01 첫 응답 (초)','6.609 (36/36)','6.267 (36/36)','11.995 (36/36)'],
 ['QA02 지칭 집합','36/36','36/36','34/36'],
 ['QA03 처리시간 (초)','13.617 (36/36)','14.242 (35/36)','12.364 (36/36)'],
 ['QA04 변경 모듈 (개)','1.75','1.75','2.00'],
 ['QA05 모델 비용 ($)','0.06750','0.06992','0.16308'],
 ['QA06 최고 RSS (MB)','44.253','45.318','50.364']
];
const widths=[216,552,552,552],heights=[50,354,62,64,36,36,36,36,36,36];
const colX=[240,792,1344];

function diagram(s,idx,selected){
 const x=colX[idx], stateName=idx===0?'유효 작업 문맥':idx===1?'요청 근거 묶음':'게시 근거';
 txt(s,'현재 입력 + 최근 대화·채택 상태',x+19,237,285,31,20,{bold:true});
 poly(s,[[x+137,266],[x+137,290]],C.black,1.7,false,true);
 const ry=290;
 box(s,'',x+22,ry,244,184,{fill:C.cream});
 txt(s,'Request Interpreter',x+29,ry+10,230,35,23,{bold:true,align:'center'});
 poly(s,[[x+34,ry+52],[x+254,ry+52]],C.black,1.2);
 txt(s,idx===0?'유효 근거에서\n현재 의미 판단':idx===1?'이번 요청에 필요한\n근거 범위 판단':'게시 근거에서\n현재 의미 판단',x+34,ry+64,220,65,23,{align:'center'});
 txt(s,'부족하면 추가 조회',x+34,ry+137,220,32,20,{align:'center'});
 const stY=idx===2?304:286;
 storeShape(s,stateName+(idx===1?'\n요청 수명':idx===2?'\n별도 게시 판본':'\n대화 수명'),x+329,stY,198,72,{size:22});
 if(idx===1){
   shape(s,'rect',x+319,277,218,89,'none',C.gray,1.2).line=line(C.gray,1.2,'dashed');
 }
 if(idx===2){
   box(s,'근거 생산자',x+329,240,198,35,{size:21,fill:C.cream});
   txt(s,'원문 변경 시 생산·검증',x+329,277,198,23,18,{align:'center'});
   poly(s,[[x+428,299],[x+428,304]],C.black,1.5,false,true);
   poly(s,[[x+527,535],[x+540,535],[x+540,257],[x+527,257]],C.black,1.5,true,true);
 }
 poly(s,[[x+329,stY+39],[x+266,stY+39]],C.black,1.9,false,true);
 n(s,1,x+285,stY+3);
 txt(s,idx===0?'재사용':idx===1?'반환':'조회',x+270,stY+46,58,43,17,{align:'center'});
 const blue=idx===0&&selected;
 if(blue){
   box(s,'',x+329,405,198,73,{stroke:C.blue});
   txt(s,'Context Manager',x+333,410,190,25,22,{bold:true,align:'center'});
   txt(s,'변경 조각 조회\n전체 현재성 재검증',x+333,436,190,38,17,{align:'center',color:C.blue});
 }else box(s,'Context Manager',x+329,412,198,62,{size:22,bold:true});
 if(blue){
   txt(s,'A2 변경 조각 갱신',x+324,377,210,29,20,{bold:true,color:C.blue,align:'center'});
 }else txt(s,idx===2?'미생산·오류·부족분':idx===0?'부족·변경 시 보완':'현재 요청에 맞춰 구성',x+318,378,218,27,19,{align:'center'});
 poly(s,[[x+266,431],[x+329,431]],C.black,1.8,false,true);
 poly(s,[[x+329,455],[x+266,455]],C.black,1.8,false,true);
 n(s,2,x+285,397);
 if(idx===2)poly(s,[[x+428,stY+72],[x+428,412]],C.black,1.8,false,true);
 else poly(s,[[x+428,blue?405:412],[x+428,stY+72]],blue?C.blue:C.black,1.8,false,true);
 storeShape(s,'허용 원본',x+329,508,198,52,{fill:C.white,size:21});
 poly(s,[[x+428,508],[x+428,474]],blue?C.blue:C.black,1.8,false,true);
 poly(s,[[x+144,474],[x+144,512]],C.black,1.8,false,true);n(s,3,x+158,480);
 box(s,'Request Controller\n현재 요청에서 채택',x+22,512,244,50,{fill:C.white,size:21});
 txt(s,idx===0?'다음 요청에 유효 문맥 유지':idx===1?'요청 종료 후 추가 묶음 해제':'게시 근거는 원본 권위와 구분',x+22,565,505,19,18,{align:'center',color:C.muted});
}
function commonNotes(){
 return '구조와 소유권은 01-evidence-supply.md를 따른다. 작업 문맥과 게시 근거는 데이터이며 새 최상위 Component를 뜻하지 않는다. Context Manager는 허용 근거 구성과 파생 자료 사용, Request Interpreter는 현재 의미 제안, Request Controller는 현재 revision에서의 채택을 맡는다. 그림은 근거 공급 구간의 확대도이며 공통 원본 owner와 Model Access, Policy Manager, State Store, Response Manager 등은 생략했다. 요청 채택은 실제 상세 응답 완료와 다르다. 모든 대안에 원본 보존, 현재 판본/권한 검사, 부족분 원문 보완이 필요하다. C 생산자는 확정 원본 변경에서 관계를 생산하며 자유문 해석 때 조건부 모델을 호출한다. 게시 근거는 파생값이고 미래 요청의 정답을 미리 알지 못한다.\nA/B는 작업 문맥 수명 축, B/C는 별도 생산 책임 축이다. A+C도 조합 가능하나 미실험이다.\n기본안은 각 4개 작성 사례×3회=12episode, 36입력/목표다. 시간은 실제 관측된 endpoint 평균이며 B QA03은 35/36이다. A/B 둘 다 종료된 같은 35목표에서는 QA03 A13.730/B14.242초다. A/B QA01 차이 0.342초는 우열 미확정이며 구조적 동점으로 취급하지 않는다. 첫 호출의 입력과 완료/조회 분기가 다르다. 본문 조건 충족은 A30/36, B30/36, C34/36의 별도 저자 진단으로 QA02와 다르다. C 사전 생산 33호출 비용 $0.0729348이 전체 비용에 포함된다. QA04는 별도 네 Rust 변경 과제의 평균이다. QA06은 Python harness와 Rust의 동시 로컬 RSS이며 원격 모델 weights/KV나 제품 UI를 포함하지 않는다. 원격 qwen3.7-plus-2026-05-26을 사용한 작성 사례의 제한된 PoC이고 실제 제품/on-device Omni 성능으로 일반화하지 않는다.\n내용 출처:\n'+ROOT+'/docs/architecture/12-decisions/decision-packages/seven-problem-review/01-evidence-supply.md\n'+ROOT+'/docs/architecture/12-decisions/decision-packages/seven-problem-review/17-presentation-results.md §3\n'+ROOT+'/results/architecture-evaluation/current/four-dp-presentation-20261019/dp01/presentation-report.md\n스타일 참조: 첨부 23-DP04방안.png, 24-DP04채택.png, 18-DP02채택.png. 대안 3개를 보존하기 위해 두 열을 세 열로 확장했다.';
}
for(const selected of [false,true]){
 const s=pres.slides.add();header(s,selected?' (선택 및 보완)':'',selected?3:2);
 if(!selected){
   txt(s,'A/B: 작업 문맥의 수명    B/C: 별도 근거의 생산 시점과 책임',28,122,1140,38,24,{bold:true});
   chip(s,'현재 요청의 유효 근거 공급',1450,121,444,C.greenD);
 }else{
   txt(s,'조건부 권고: 같은 자료를 이어 묻는 사용에서는 A',28,119,1160,39,28,{bold:true,color:C.blue});
   txt(s,'동일 자료 답변 호출 B보다 6회 감소 / QA01 전체 우열 미확정',28,156,1250,25,20);
 }
 ['QA01 첫 응답','QA02 지칭 집합','QA05 모델 비용'].forEach((v,i)=>chip(s,v,1210+i*229,151,222));
 const table=nativeTable(s,values,24,184,widths,heights,22);
 table.cells.block({row:0,column:0,rowCount:1,columnCount:4}).assign({fill:C.green,textStyle:{typeface:FONT,fontSize:26,bold:true,color:C.black,alignment:'center'}});
 table.cells.block({row:1,column:0,rowCount:1,columnCount:1}).assign({textStyle:{typeface:FONT,fontSize:30,bold:true,color:C.black},anchor:'top',margins:{left:12,right:12,top:17,bottom:0}});
 table.cells.block({row:2,column:0,rowCount:2,columnCount:1}).assign({textStyle:{typeface:FONT,fontSize:27,bold:true,color:C.black}});
 table.cells.block({row:4,column:0,rowCount:6,columnCount:1}).assign({fill:'#F7F8FA',textStyle:{typeface:FONT,fontSize:21,bold:true,color:C.black,alignment:'left'}});
 table.cells.block({row:3,column:1,rowCount:1,columnCount:3}).assign({textStyle:{typeface:FONT,fontSize:22,color:C.red,alignment:'center'}});
 // Native shape legend lives in the empty structure-label cell.
 box(s,'Component',46,336,169,48,{fill:C.white,size:21});
 storeShape(s,'보존 상태',46,412,169,62,{size:21});
 shape(s,'rect',48,501,30,23,C.cream,C.black,1.2);txt(s,'비교 대상',88,496,128,33,20);
 if(selected){shape(s,'rect',48,542,30,23,C.white,C.blue,2.5);txt(s,'보완 부분',88,537,128,33,20,{color:C.blue});}
 else txt(s,'번호: 처리 순서',46,541,170,28,19,{color:C.muted});
 [0,1,2].forEach(i=>diagram(s,i,selected));
 if(selected){
   // Blue overlay is added after all base content so the original panel remains.
   shape(s,'rect',240,184,552,746,'none',C.blue,4);
   txt(s,'✓',245,171,55,64,53,{bold:true,color:C.red});
   const v=table.getCell(3,1);v.value='주제 전환 후 남는 입력과 정정 갱신 부담\n[A2 별도 3회] 원문 읽기 12→9 / 호출 12→12';
   v.text.style={typeface:FONT,fontSize:21,color:C.red,alignment:'center',bold:false};
   v.text.get('[A2 별도 3회] 원문 읽기 12→9 / 호출 12→12').fill=C.blue;
   // Shared policy is outside the selected A panel, across all three alternatives.
   shape(s,'rect',24,972,1872,76,C.pale,C.blue,1.7);
   txt(s,'공통 보완',43,977,170,31,25,{bold:true,color:C.blue});
   txt(s,'(A/B/C)',43,1011,170,26,21,{color:C.blue});
   txt(s,'채택한 근거·처리 의도를 먼저 안내한 뒤 상세 처리',235,977,1030,32,24,{bold:true,color:C.blue});
   txt(s,'E3의 A: 첫 응답 7.092→1.905초 / 처리시간 21.508→23.118초 / 호출 12→18회 / 비용 $0.01466→$0.01574',235,1013,1600,29,21);
   txt(s,'A2와 첫 안내는 별도 실험이며 결합 효과는 미측정. A2 본문 3/6 동일. A3 기본 적용 보류, A+C 미실험.',28,1051,1775,24,18,{color:C.muted});
   // Compact selection conditions sit above the unchanged QA table.
   table.getCell(2,1).value='같은 자료의 답변 호출 6회 감소\n후속 근거 재사용 조건에서 권고';
   table.getCell(2,1).text.style={typeface:FONT,fontSize:22,color:C.blue,alignment:'center',bold:true};
   table.getCell(2,2).value='주제 전환이 많고\n근거 재사용이 적을 때 검토';
   table.getCell(2,3).value='본문 조건 충족 34/36 관측\n생산비를 반복 소비로 회수할 때 검토';
   txt(s,'표는 기본안별 12episode / QA04 별도 4과제 / RSS 로컬 실행기',1210,121,685,26,18,{align:'right',color:C.muted});
   txt(s,'A/B QA01 우열 미확정. 본문 충족 A/B 30/36, C 34/36. C 총비용은 A의 2.42배.',28,938,1750,27,21);
   s.speakerNotes.textFrame.setText(commonNotes()+'\n선택은 조건부 저자 권고이며 accepted ADR 또는 사용자 채택 선언이 아니다. QA01의 전체 승자는 입증하지 못했다. 구조의 선택과 공통 응답 정책을 분리한다.\nA2 변경 조각 갱신은 정정 E3×3회, 원문 읽기12→9, 모델 호출12→12, QA03 21.822→21.433초, 비용0.0148424→0.0145860 USD, 본문3/6동일이다. 큰 지연/비용 향상 또는 본문 누락 해결로 확대하지 않는다. 필수14KB hard-cap 구현 수정은 보완 성과에 포함하지 않는다.\n공통 첫 안내는 E3 ABC×2정책×3회=18episode의 별도 실험이다. A/B의 첫 입력은 정책/반복/질문으로 짝지은12쌍에서 같았다. 유지 view가 없거나 정정으로 무효화되므로 A의 유효 문맥 재사용을 시험하지 않는다. 첫 안내는 실제 source와 처리 의도의 채택이며 완료 답변이 아니다. 내부 intent 라벨이 노출된 callback proxy라 자연스러운 제품 UI의 끝단 지연으로 부르지 않는다. B QA01 6.724→1.829, QA03 22.172→23.422, 호출12→18, 비용0.0150744→0.0159652. C QA01 17.730→2.322, QA03 17.731→20.350, 호출15→21, 비용0.0348104→0.0426752. C 생산도 정책마다 독립 실행해 순수 정책효과와 구별한다. 각 안·정책의 첫 응답/종료/QA02는6/6이다. 본문은 A/B3/6, C6/6으로 유지됐다.\nA2와 첫 안내 정책 조합은 실행하지 않았다. A3 작은 보존 예산은 상태 감소와 재읽기 증가의 교환으로 기본 적용 보류.');
 }else{
   txt(s,'공통: 원본 보존, 현재 판본·권한 확인, 부족분 재조회, 현재 요청 revision에서 채택',28,940,1850,30,23,{bold:true});
   txt(s,'기본안별 12episode. 시간 괄호는 관측/예정, 비용은 합계. QA04는 별도 4과제 평균, RSS는 로컬 실행기 범위.',28,976,1850,27,21,{color:C.muted});
   txt(s,'A/B QA01 우열 미확정. 별도 본문 충족은 A/B 30/36, C 34/36이며 QA02 및 종료 여부와 구분.',28,1008,1850,27,21,{color:C.muted});
   txt(s,'작성 사례와 원격 모델을 사용한 PoC 관측값. A+C 조합 가능, 미실험.',28,1040,1770,26,20,{color:C.muted});
   s.speakerNotes.textFrame.setText(commonNotes());
 }
 // Explicit native table leading keeps rows fixed across slide editors.
 table.cells.block({row:0,column:0,rowCount:10,columnCount:4}).textStyle.lineSpacing=1;
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
const out=path.join(BUILD,'output','VIA-DP1-reference-style.pptx');
const result=await finalizePresentation({workspaceDir:BUILD,candidatePath:draft,finalPath:out,pythonExecutable:PY,integrityValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_layout_geometry.py'),explicitTotalSlideCount:3,requiredNativeTableOwnerSlides:[2,3],fontPolicy:{basis:'design',families:[FONT]},verifyArtifactToolImport:true,layoutArgs:['--expected-slide-size-emu',String(W*9525)+','+String(H*9525),'--validate-bullet-geometry','--validate-heading-fit','--require-native-table-slide','2','--require-native-table-slide','3'],receiptPath:path.join(BUILD,'validation.json')});
console.log(JSON.stringify({output:out,result},null,2));
