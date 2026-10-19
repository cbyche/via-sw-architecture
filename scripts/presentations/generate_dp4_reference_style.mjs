import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {Presentation, PresentationFile} from '@oai/artifact-tool';

// DP4 four-slide editable deck using the approved DP1 reference layout. Run a copy from a private build
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
  return s.shapes.add({name:name||'DP4 object '+(++serial),geometry:geom,position:{left:x,top:y,width:w,height:h},fill,line:line(stroke,lw)});
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
  txt(s,'DP4. Agent 간 업무 인계 방안'+suffix,60,29,1200,68,48,{bold:true});
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

function label(s,t,x,y,w,size=22,color=C.black){return txt(s,t,x,y,w,32,size,{color,bold:true,align:'center'});}
function cellStyle(cell,size,color=C.black,bold=false,align='center'){
 cell.text.style={typeface:FONT,fontSize:size,color,bold,alignment:align,verticalAlignment:'middle',lineSpacing:1};
}
function highlightHeader(table,n){table.cells.block({row:0,column:0,rowCount:1,columnCount:n}).assign({fill:C.green,textStyle:{typeface:FONT,fontSize:25,bold:true,color:C.black,alignment:'center'}});}
function component(s,name,x,y,w,h){
 shape(s,'rect',x,y,w,h,C.cream,C.black,2.7,name+' Component');
 txt(s,name,x+8,y+2,w-16,32,27,{bold:true,align:'center'});
 poly(s,[[x,y+36],[x+w,y+36]],C.black,1.4);
}
function notes(extra=''){
 return '발표 DP4는 저장소05 Agent 간 업무 인계다. 17-presentation-results.md §2·6의 완료된 발표 캠페인을 사용한다. 05 설계 본문의 초기 A 권고보다 최신 조건부 C 권고를 따른다. 새 common9의 준비·부분 측정/QA04를 이 완료 캠페인에 섞지 않는다.\n구조: A는 실제 위임 목적별 자료 구성, B는 공개 사건에서 source-backed 업무 설명/관계를 생산하여 지속 record로 재사용, C는 필수 목표/조건/제외/수행 범위는 VIA가 명시하고 상세 범위 선택을 실제 READ 지원 수신자에게 맡긴다. A/B에도 유효 artifact 참조와 기확보 원문 재사용을 허용한다. B+C는 별도 구조 조합으로 가능하다. Agent 비공개 추론/memory 이동을 가정하지 않는다. Context Manager는 자료 후보·파생 기록·자료 제공을 담당한다. Request Controller는 현재 목표/조건과 전송 admission, Task Manager는 Agent Command, Agent Gateway는 실제 전송/capability/수신 사실을 소유한다. 원본 권위는 입력/정정 Request Controller, 실제 전달 Response Manager, 현재 업무/공개 결과 Task Manager에 유지한다. 내부 작은 상자는 한국어 기능 책임이며 새 정식 Component/프로세스/모델이 아니다. 필요한 의미 가공만 Model Access를 사용한다.\n기본 A/B/C는 각4작성사례×3회=12episode, 전체36회. QA01 actual callback proxy의 관측/예정, QA03 host 종료의 관측/예정을 표시한다. QA02 지칭 집합 N/A. QA03은 수신자 Qwen HTTP 구간 합집합을 제외한 VIA 귀속 시간이며 전체경과를 따로 기록한다. QA04는 당시 네 Rust 변경 과제 평균이다. QA05는12episode 모델 고정단가 합계로 수신자 비용과 B 준비51호출을 포함한다. QA06은 동시 Python+Rust 최고 RSS이며 cloud weights/KV/제품 전체RAM은 제외한다. QA01 공통20callback A4.307/B4.409/C4.327초, 입력 경로가 달라 구조적 동점도 아니고 작은 차이의 우열도 미확정이다. 같은16완료목표 QA03 A10.565/B12.299/C4.213초, 전체경과31.372/31.406/23.532초. 원래21분모의 완료20/19/19를 대체하지 않는다.\nC1은 C의 상대적 QA 열세가 입증돼서 추가한 것이 아니라 필수 조건 본문 누락 위험의 보완이다. 양쪽 같은 condition_source_refs가 있고 C1만 다음 위임 목적을 알기 전에 기채택 조건의 provenance 원문1~2행을 읽어 최초 인계에 붙인다. 조건 출처를 새로 만드는 upstream 비용은 미측정이다. 첫 해석 호출에는 C1 원문을 넣지 않는다. 별도 E3×3회 각9인계의 전후이며 기본전체 C값과 연결하지 않는다. 본문 저자대조4/9→9/9와 자동literal4/9→7/9, N-H9/9→9/9, QA02 N/A는 서로 다른 관측이다. 독립 검토는 실제 메일6개에 한정됐다. C의 미표시1건은 fact15개가 프롬프트에 안내되지 않은 host상한14를 넘긴 수량계약 불일치다. 원문 부족이나 의미오답으로 재분류하지 않는다. 첫callback9/9는 올바른내용9/9를 뜻하지 않으며 C 첫안내 조건오류1건을 보존한다.\nC1 비용 감소는 C복구1호출을 포함한 관측이다. 그것을 제외하면 .0365256→.0376028로 증가하며, 같은8완료목표 전체경과30.532→31.519초로 늘었다. RSS감소를 인과적으로 C1 메모리최적화라 주장하지 않는다. 원문읽기20→21회, 수신자조건읽기2→0과 사전읽기3, 첨부상태323byte. C2cache는 전체serving이7.602→8.784ms로 늘어 기본보류이고 C1+C2미실행이다.\n출처:\n'+ROOT+'/docs/architecture/12-decisions/decision-packages/seven-problem-review/05-agent-handoff.md §3~6\n'+ROOT+'/docs/architecture/12-decisions/decision-packages/seven-problem-review/17-presentation-results.md §2,6\n'+ROOT+'/results/architecture-evaluation/current/four-dp-presentation-20261019/dp05/fresh-summary.json\n'+ROOT+'/results/architecture-evaluation/current/four-dp-presentation-20261019/dp05/mechanism-breakdown.json\n'+ROOT+'/results/architecture-evaluation/current/four-dp-presentation-20261019/dp05/content-review.json\n'+ROOT+'/results/architecture-evaluation/current/four-dp-presentation-20261019/dp05/prefix-review.json\n스타일: DP2 v2·DP3 네 장, 사용자 레퍼런스18-DP02채택,23-DP04방안,24-DP04채택.\n'+extra;
}

// 1. The same accepted condition must survive a change of agent and work purpose.
{
 const s=pres.slides.add();header(s,' (배경)',1);
 txt(s,'Agent가 바뀌어도, 사용자가 정한 조건은 이어져야 한다',66,145,1790,62,43,{bold:true});
 txt(s,'같은 비교 결과로 발표자료와 메일을 만들 때, 다음 Agent가 받을 자료를 VIA가 연결한다.',68,215,1770,40,28);
 const xs=[90,690,1290];
 ['① 조사 결과 확보','② 발표자료로 재위임','③ 메일 초안으로 재위임'].forEach((v,i)=>txt(s,v,xs[i],292,500,41,31,{bold:true}));
 const agents=['조사 Agent','발표자료 Agent','메일 Agent'];
 const titles=['제품 비교 결과','발표자료 작성','메일 초안 작성'];
 const bodies=['가격·유지보수 비교\n공개 결과의 원본과 판본','같은 결과를 새 목적에 사용\n표·설명·제외사항 전달','받는 사람과 문체는 변경\n채택 조건은 계속 유지'];
 xs.forEach((x,i)=>{
  box(s,agents[i],x,351,500,56,{fill:C.cream,size:29,bold:true});
  doc(s,titles[i],bodies[i],x+20,428,460,172);
 });
 poly(s,[[590,379],[690,379]],C.black,2.5,false,true);poly(s,[[1190,379],[1290,379]],C.black,2.5,false,true);
 txt(s,'VIA',610,336,65,32,24,{bold:true,align:'center'});txt(s,'VIA',1210,336,65,32,24,{bold:true,align:'center'});
 shape(s,'rect',109,631,1660,53,C.green,'none',0);
 txt(s,'채택한 조건: “가격이 같을 때 유지보수 기간이 긴 제품 우선”',130,635,1620,45,30,{bold:true,align:'center'});
 [340,940,1540].forEach(x=>poly(s,[[x,631],[x,600]],C.greenD,2.4,false,true));
 const t=nativeTable(s,[['인계에서 잃기 쉬운 것','문제가 되는 전달 예'],['조건의 적용 전제','“가격이 같을 때”가 빠지고 유지보수를 항상 우선'],['자료의 유효성','이전 판본·철회된 자료를 다음 Agent가 사용']],109,724,[470,1190],[43,54,54],27);
 highlightHeader(t,2);cellStyle(t.getCell(1,1),27,C.red,true);cellStyle(t.getCell(2,1),27,C.red);
 poly(s,[[65,913],[1830,913]],C.blue,1.5);
 txt(s,'이 DP의 질문',66,949,275,52,34,{bold:true,color:C.red});
 txt(s,'인계 자료를 언제 준비하고, 누가 상세 범위를 선택할 것인가?',355,945,1475,56,37,{bold:true});
 txt(s,'위임할 때 구성 / 지속 업무 기록에서 추출 / 수신자가 원문을 선택해 읽기',355,1010,1460,37,27);
 s.speakerNotes.textFrame.setText(notes('배경은 설계 설명용 도식으로 실제 외부 상용 Agent 수행 화면이 아니다. 조건 예시는 C1 근거의 가격 동률 전제를 설명하며 빨간 전달 예는 의미 손실을 보여주는 설명 문구다. 실제 모든 출력이 이 문구와 동일했다고 주장하지 않는다. Agent의 비공개 상태가 아닌 공개 자료와 채택 조건을 인계한다.'));
}

const TW=[168,568,568,568], RH=[46,420,55,68,32,32,45,32,45,32,45];
function compactComponent(s,t,x,y,w,h,size=21){box(s,t,x,y,w,h,{fill:C.cream,size,bold:true});}
function commonSend(s,x){
 compactComponent(s,'Task Manager: 명령 확정',x+18,516,532,32,22);
 compactComponent(s,'Agent Gateway: 실제 전송',x+18,559,532,32,22);
 box(s,'외부 Downstream Agent',x+18,605,532,28,{fill:C.white,size:22});
 poly(s,[[x+284,504],[x+284,516]],C.black,2,false,true);
 poly(s,[[x+284,548],[x+284,559]],C.black,2,false,true);
 poly(s,[[x+284,591],[x+284,605]],C.black,2,false,true);
}
function diagramA(s,x){
 label(s,'위임 요청마다 목적별 구성',x+10,226,548,26);
 compactComponent(s,'Request Controller: 채택·허가',x+18,264,532,34,22);
 component(s,'Context Manager',x+18,313,532,191);
 box(s,'목적별 자료 선택\n필수 조건·발췌·참조 구성',x+33,361,275,57,{fill:C.white,size:23,bold:true});
 storeShape(s,'공통 원본\n기확보 근거',x+339,358,194,65,{size:22});
 poly(s,[[x+339,390],[x+308,390]],C.black,2,false,true);
 poly(s,[[x+170,298],[x+170,361]],C.black,2,false,true);
 storeShape(s,'임시 인계 묶음 (capsule)',x+74,445,420,49,{size:24});
 poly(s,[[x+170,418],[x+170,445]],C.black,2,false,true);
 n(s,1,x+25,349);n(s,2,x+54,438);
 commonSend(s,x);
}
function diagramB(s,x){
 label(s,'공개 사건을 선생산하고 반복 소비',x+10,226,548,25);
 compactComponent(s,'Request Controller: 채택·허가',x+18,264,532,34,22);
 component(s,'Context Manager',x+18,313,532,191);
 txt(s,'공개 결과·정정 사건',x+35,350,234,27,21,{align:'center'});
 box(s,'업무 설명·관계\n생산 / 갱신',x+35,386,234,52,{fill:C.white,size:23,bold:true});
 storeShape(s,'지속 업무 기록\n판본·의존성·coverage',x+304,359,230,78,{size:21});
 poly(s,[[x+151,377],[x+151,386]],C.black,2,false,true);
 poly(s,[[x+269,411],[x+304,411]],C.black,2,false,true);
 box(s,'목적별 추출 / 부족한 원본 보완',x+74,457,420,37,{fill:C.white,size:23,bold:true});
 poly(s,[[x+419,437],[x+419,448],[x+284,448],[x+284,457]],C.black,2,false,true);
 poly(s,[[x+83,298],[x+8,298],[x+8,476],[x+74,476]],C.black,1.8,true,true);
 n(s,1,x+25,375);n(s,2,x+284,351);n(s,3,x+54,447);
 commonSend(s,x);
}
function diagramC(s,x,selected){
 label(s,'필수 명세 전달 + 수신자 READ 왕복',x+10,226,548,25);
 compactComponent(s,'Request Controller: 채택·허가',x+18,264,532,34,22);
 component(s,'Context Manager',x+18,313,532,191);
 box(s,'필수 명세·자료 목록 구성',x+31,359,241,38,{fill:C.white,size:21,bold:true});
 poly(s,[[x+150,298],[x+150,359]],C.black,2,false,true);
 if(selected){
  box(s,'C1 사전 확보한 조건 원문 첨부',x+31,408,241,37,{fill:C.pale,stroke:C.blue,size:18,bold:true,color:C.blue});
  poly(s,[[x+151,397],[x+151,408]],C.blue,2,false,true);
  poly(s,[[x+151,445],[x+151,457]],C.blue,2,false,true);
 }else{
  txt(s,'필수 조건은 VIA가 명시',x+31,409,241,32,21,{align:'center'});
  poly(s,[[x+151,397],[x+151,407]],C.black,2,false,true);
  poly(s,[[x+151,442],[x+151,457]],C.black,2,false,true);
 }
 storeShape(s,'manifest / 필수 조건·참조',x+31,457,241,37,{size:19});
 box(s,'요청 범위 읽기 제공\n권한·판본·철회 검사',x+303,359,229,62,{fill:C.white,size:22,bold:true});
 storeShape(s,'lease / read receipt',x+303,451,229,43,{size:20});
 poly(s,[[x+418,421],[x+418,451]],C.black,1.9,false,true);
 compactComponent(s,'Task Manager\n명령 확정',x+31,518,241,55,23);
 compactComponent(s,'Agent Gateway\n전송 / READ 연결',x+303,518,229,55,22);
 poly(s,[[x+151,494],[x+151,518]],C.black,2,false,true);
 poly(s,[[x+272,546],[x+303,546]],C.black,2.1,false,true);
 box(s,'외부 Downstream Agent\n상세 범위를 선택·요청',x+303,594,229,40,{fill:C.white,size:20,bold:true});
 poly(s,[[x+377,573],[x+377,594]],C.black,2,false,true);
 poly(s,[[x+457,594],[x+457,573]],C.black,2,false,true);
 poly(s,[[x+532,534],[x+544,534],[x+544,384],[x+532,384]],C.black,1.8,true,true);
 poly(s,[[x+532,409],[x+556,409],[x+556,556],[x+532,556]],C.black,1.8,false,true);
 txt(s,'세부 선택은 수신자',x+31,600,241,31,21,{align:'center',bold:true});
 n(s,1,x+22,349);n(s,2,x+285,350);n(s,3,x+283,582);
}
function comparison(selected){
 const s=pres.slides.add();header(s,selected?' (선택 및 보완)':' (설계 방안 비교)',selected?3:2);
 txt(s,selected?'읽기 지원 수신자에 조건부 C 선택 / C1은 이미 채택된 조건의 원문 제공 보완':'A↔B는 준비 수명 / A↔C는 상세 자료 선택 책임이 다르다',28,124,1835,39,28,{bold:true,color:selected?C.blue:C.black});
 const vals=[
 ['설계안','(A) 위임 시 인계 묶음','(B) 지속 업무 기록','(C) 필수 명세 + 수신자 읽기'],
 ['구조','','',''],
 ['장점','필요한 위임에만 자료를 구성\n기확보 원문·유효 참조를 재사용','여러 인계에서 업무 설명을 재사용\n공개 결과와 조건 관계를 유지','VIA의 선행 세부 선택을 줄임\n업무 중 필요한 범위만 수신자가 선택'],
 ['단점','새 목적마다 선택·구성 부담\n반복 인계의 준비 작업 증가','인계가 없어도 생산·갱신 비용\n새 목적에는 원본 보완이 필요','수신자의 읽기 지원·접근 관리 필요\n유효 참조만으로 본문 조건 보존은 불충분'],
 ['QA01 첫 응답 (초)','4.307 (20/21)','4.421 (21/21)','4.321 (21/21)'],
 ['QA02 지칭 집합','N/A','N/A','N/A'],
 ['QA03 처리 (초)','11.019 (20/21)','12.439 (19/21)','4.329 (19/21)'],
 ['QA04 변경 (개)','1.50','1.75','1.50'],
 ['QA05 모델 비용 ($)','0.1018408','0.2886828','0.0757928'],
 ['QA06 RSS (MB)','49.119','57.229','48.153'],
 ['본문 충족 (진단)','8/18','15/18','12/18']];
 const t=nativeTable(s,vals,24,174,TW,RH,19);highlightHeader(t,4);
 t.cells.block({row:1,column:0,rowCount:1,columnCount:1}).assign({textStyle:{typeface:FONT,fontSize:28,bold:true},anchor:'top',margins:{top:16,left:12,right:12,bottom:0}});
 t.cells.block({row:2,column:0,rowCount:2,columnCount:1}).assign({textStyle:{typeface:FONT,fontSize:24,bold:true}});
 t.cells.block({row:3,column:1,rowCount:1,columnCount:3}).assign({textStyle:{typeface:FONT,fontSize:19,color:C.red,alignment:'center'}});
 t.cells.block({row:4,column:0,rowCount:7,columnCount:1}).assign({fill:'#F7F8FA',textStyle:{typeface:FONT,fontSize:18,bold:true,alignment:'left'}});
 shape(s,'rect',38,286,140,72,C.cream,C.black,2.7);txt(s,'Component',42,289,132,28,20,{bold:true,align:'center'});
 shape(s,'rect',46,324,124,27,C.white,C.black,1.4);txt(s,'내부 처리',49,324,118,27,19,{align:'center'});
 storeShape(s,'상태 / 자료',40,388,137,61,{size:20});
 txt(s,'실선: 자료·명령\n점선: 요청·조건',32,480,154,61,18,{align:'center'});
 txt(s,'Task·원본 권위는\n기존 owner 유지',32,555,154,49,18,{align:'center',color:C.muted});
 diagramA(s,192);diagramB(s,760);diagramC(s,1328,selected);
 if(selected){
  shape(s,'rect',1328,174,568,852,'none',C.blue,4.2,'Selected C column');
  txt(s,'✓',1336,147,67,79,65,{color:C.red,bold:true});
  const con=t.getCell(3,3);con.value+='\n[C1] 조건 출처가 있을 때 원문 첨부';cellStyle(con,18,C.red);con.text.get('[C1] 조건 출처가 있을 때 원문 첨부').fill=C.blue;
  const q3=t.getCell(6,3);q3.value+='\n[별도 E3] 4.302(8/9) → 4.301(9/9)';cellStyle(q3,18);q3.text.get('[별도 E3] 4.302(8/9) → 4.301(9/9)').fill=C.blue;
  const q5=t.getCell(8,3);q5.value+='\n[별도 E3] 0.0407024 → 0.0376028';cellStyle(q5,18);q5.text.get('[별도 E3] 0.0407024 → 0.0376028').fill=C.blue;
  const qd=t.getCell(10,3);qd.value+=' (기본)\n[별도 E3 저자 대조] 4/9 → 9/9';cellStyle(qd,18);qd.text.get('[별도 E3 저자 대조] 4/9 → 9/9').fill=C.blue;
  txt(s,'C1은 조건 누락 위험의 보완. 적용 QA의 상대적 열세는 입증되지 않음. 별도 E3 수치는 전체 기본값과 직접 연결하지 않음.',28,1032,1840,22,18,{color:C.blue});
 }else{
  txt(s,'QA01 우열 미확정. C는 실제 범위 읽기·권한·판본 검사를 지원하는 수신자 조건. 본문 충족은 QA02와 별도 진단.',28,1032,1840,22,18,{bold:true});
 }
 txt(s,'기존 완료 PoC: 안별 4사례×3 / 시간 괄호: 관측·예정 / 비용: 12회 합계(수신자 포함) / RSS: 로컬 실행기 / QA03은 수신자 HTTP 시간 제외',28,1057,1805,19,16,{color:C.muted});
 s.speakerNotes.textFrame.setText(notes('선택 전후는 같은 표와 같은 A/B구조다. C의 보완은 상세선택을 VIA로 되돌리지 않고 조건 원문만 인계에 붙인다. C1 상자는 논리적 추가 기능을 표시한다. 실제 사전읽기는 다음 위임 목적을 알기 전에 일어나며 현재 명세구성 뒤에 새 동기읽기를 강제한다는 뜻이 아니다. 정확한 시점은4장에 표시했다. Request Controller의 admission은 Task Manager/Agent Gateway 전송 시 재검증하며 topbox가 자료검증 전에 최종전송을 허가한다는 뜻이 아니다. B 공개사건은 원본owner에서 오는 갱신 신호이며 위임요청과 별개의 트리거다.'));
}
comparison(false);comparison(true);

// 4. All before/after numbers use the separate E3 C/C1 experiment.
{
 const s=pres.slides.add();header(s,' (보완 효과 상세)',4);
 txt(s,'C1: 채택 조건의 원문을 먼저 제공해 인계 본문에서 전제가 빠지는 위험을 보완',28,125,1850,45,31,{bold:true,color:C.blue});
 txt(s,'별도 E3 실험: 목적을 바꾼 세 번 인계 × 3회 / C와 C1 각각 9개 인계',28,177,1830,36,26);
 shape(s,'rect',24,232,592,733,'none',C.blue,3);
 shape(s,'rect',24,232,592,49,C.green,'none',0);
 txt(s,'기존 조건 출처가 있는 경우에 한정',42,236,556,41,28,{bold:true,align:'center'});
 storeShape(s,'채택 조건의 원문·source refs\n“가격이 같으면 유지보수 기간 우선”',60,309,520,88,{size:25});
 poly(s,[[320,397],[320,425]],C.blue,2.4,false,true);
 box(s,'Context Manager / C1\n다음 위임 전, 원문 1~2행 사전 읽기',60,425,520,81,{fill:C.pale,stroke:C.blue,size:26,bold:true,color:C.blue});
 poly(s,[[320,506],[320,541]],C.blue,2.4,false,true);
 box(s,'필수 명세에 조건 원문 첨부\n상세 자료는 기존 수신자 READ 유지',60,541,520,78,{fill:C.pale,stroke:C.blue,size:26,bold:true,color:C.blue});
 poly(s,[[320,619],[320,651]],C.black,2.4,false,true);
 box(s,'Task Manager / Agent Gateway\n현재 조건·권한 재검사 후 전송',60,651,520,72,{fill:C.cream,size:26,bold:true});
 poly(s,[[320,723],[320,754]],C.black,2.4,false,true);
 box(s,'수신 Agent: 조건을 유지하며 본문 작성',60,754,520,55,{fill:C.white,size:26,bold:true});
 txt(s,'전체 원문 읽기 20 → 21회\n사전 읽기 3회 추가 / 첨부 상태 최대 323byte',53,839,535,68,25,{align:'center'});
 txt(s,'조건 출처를 새로 만드는 비용은 미측정',52,920,536,30,22,{align:'center',color:C.red});
 txt(s,'본문 조건 충족 4/9 → 9/9',650,232,1225,49,37,{bold:true,color:C.blue});
 txt(s,'저자 본문 대조: C 충족4·오류2·모호2·미표시1 / C1 충족9  (QA02 아님)',650,285,1235,32,23);
 const vals=[['지표','C','C1','해석'],
 ['QA01 첫 응답 (초)','4.256 (9/9)','4.271 (9/9)','첫 안내 개선 근거 없음'],
 ['QA02 지칭 집합','N/A','N/A','본문 진단과 구분'],
 ['QA03 VIA 처리 (초)','4.302 (8/9)','4.301 (9/9)','host 종료와 본문 정답은 별개'],
 ['QA04 변경 (개)','미측정','미측정','새 변경 과제 실행 없음'],
 ['QA05 모델 비용 ($)','0.0407024','0.0376028','복구 1호출 차이 포함'],
 ['QA06 최고 RSS (MB)','45.269','43.254','관측값 / 개선 인과 미확정'],
 ['같은 8목표 전체 시간 (초)','30.532','31.519','+0.987초'],
 ['N-H 구조 계약','9/9','9/9','기존 구조 계약 유지']];
 const t=nativeTable(s,vals,650,332,[326,266,266,338],[42,52,40,52,40,52,52,52,40],23);highlightHeader(t,4);
 cellStyle(t.getCell(7,3),23,C.red,true);
 txt(s,'복구 1호출 제외 비용: $0.0365256 → $0.0376028 (증가)',653,831,1227,33,25,{color:C.red,bold:true});
 txt(s,'자동 literal 판정은 4/9 → 7/9. 위 9/9는 별도 저자 내용 대조다.',653,877,1227,32,23);
 txt(s,'C 미표시 1건: 안내되지 않은 host 수량 상한과 불일치. 원문 부족으로 분류하지 않음.',653,918,1227,32,22);
 poly(s,[[28,990],[1894,990]],C.blue,1.4);
 txt(s,'C1은 첫 안내 입력을 바꾸지 않음. C의 잘못된 첫 안내 1건도 보존. 본문 개선 전체를 사전 읽기만의 인과 효과로 단정하지 않음.',28,1000,1830,30,21,{bold:true});
 txt(s,'C2 cache는 전체 제공 시간이 7.602→8.784ms로 늘어 보류. C1+C2 조합 미실행. 기존 완료 캠페인 범위의 보완이다.',28,1043,1807,27,20,{color:C.muted});
 s.speakerNotes.textFrame.setText(notes('4장의 before/after는 PREFETCH_TACTIC E3 각3episode만 사용한다. QA03 모든관측 평균은8/9와9/9이며 공통8의 VIA시간4.302→4.284초를 별도로 보존한다. 공통8전체경과30.532→31.519초와 전체모델비용을 함께 보고한다. C1 whole9전체경과31.614초를공통8값으로 혼용하지 않는다. 수신자조건읽기2→0이나 전체읽기20→21로 증가한다. source-read 요청1848byte, 로컬구간합계.201ms, 첨부상태최대직렬화323byte. C의 미표시1은fact상한계약문제이며 C1의원문만으로실패가원인적으로해결됐다고하지않는다.'));
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
const out=path.join(BUILD,'output','VIA-DP4-reference-style.pptx');
const result=await finalizePresentation({workspaceDir:BUILD,candidatePath:draft,finalPath:out,pythonExecutable:PY,integrityValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_layout_geometry.py'),explicitTotalSlideCount:4,requiredNativeTableOwnerSlides:[1,2,3,4],fontPolicy:{basis:'design',families:[FONT]},verifyArtifactToolImport:true,layoutArgs:['--expected-slide-size-emu',String(W*9525)+','+String(H*9525),'--validate-bullet-geometry','--validate-heading-fit','--require-native-table-slide','1','--require-native-table-slide','2','--require-native-table-slide','3','--require-native-table-slide','4'],receiptPath:path.join(BUILD,'validation.json')});
console.log(JSON.stringify({output:out,result},null,2));
