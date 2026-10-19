import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {Presentation, PresentationFile} from '@oai/artifact-tool';

// DP3 four-slide editable deck using the approved DP1 reference layout. Run a copy from a private build
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
  return s.shapes.add({name:name||'DP3 object '+(++serial),geometry:geom,position:{left:x,top:y,width:w,height:h},fill,line:line(stroke,lw)});
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
  txt(s,'DP3. 화면과 발화의 지칭 결합 방안'+suffix,60,29,1200,68,48,{bold:true});
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
 return '발표 DP3는 저장소04/DP-47 화면과 발화 지칭 결합에 대응한다. 현재 문서 번호03 처리 단계 연결과 다르다. A는 Request Interpreter 내부 Joint Grounding Interpreter가 원본에서 target extent와 binding을 함께 생산한다. B는 Speech Reference Producer와 Screen Target Producer가 각각 후보 revision을 게시하고 Multimodal Binding Resolver가 이를 결합한다. 없는 범위는 원 생산자가 원본에서 확장하며 Resolver의 raw-to-final bypass는 기본 B가 아니다. 모듈은 논리 책임이며 독립 프로세스나 모델 복제를 뜻하지 않는다. 입력 수집과 Request Controller의 현재성/채택은 공통이다. 명백한 native 선택은 양쪽 모두 code 경로를 허용한다. B의 초기 생산은 기본 순차 dispatch이며 별도 B4의 병행과 구별한다. 설계의 OmniParser 참고와 이번 PoC의 실제 Qwen vision 생산을 구분한다. 이번 수치는 OmniParser 실행 결과가 아니다.\n주 비교는 2026-10-19 발표 캠페인 중 별도로 동결한 공통 정규화 출력 A/B24회다. 안별4작성사례×3회=12episode. 0~1000 bbox를 frame pixel로 축별 ties-even 변환, 범위 밖 clamp 금지. raw cue는 원본 pixel. A2/A5/B4는 넣지 않았다. QA01 actual callback proxy15입력, QA02 최초 채택 대상집합24판정, QA03 host 종료12목표의 분모를 보존한다. B dense3회 미완료이므로 성공 평균을 전체 속도 우위로 쓰지 않는다. QA04 새 판본 미측정, 이전2.50은 과거 네 Rust 변경 과제의 값일 뿐이다. QA05 안별12episode 합계와 고정 입력$0.40/M·출력$1.60/M 단가. QA06 Python+Rust 최고 동시 RSS이며 UI/cloud weights/KV 제외. 기존 core QA ID를 이 발표 QA01~06으로 바꾸지 않는다.\n현재 권고는 정규화 기본 A이며 추가 tactic 없음. 선택장과4장의 A2+A5는 별도 pixel 계약의 복구 대안이다. 같은 pixel 실험창의 기본 A→A2+A5만 전후로 비교한다. QA02 19/24→24/24, 첫 응답12/15→15/15, 완료10/12→12/12. A2 영향표현 patch와 A5 좌표 view 조합의 개별 인과는 분리하지 않았다. 정규화 A+A2+A5는 미실행. 기본 A 비용에는 usage 미확정1회와 예약상한$0.0729404가 남으며 실제 청구액/0비용으로 바꾸지 않는다. 새로운 common9 준비/부분 실측은 이 자료에 합치지 않는다.\n출처:\n'+ROOT+'/docs/architecture/12-decisions/decision-packages/seven-problem-review/04-screen-speech-grounding.md §4~6\n'+ROOT+'/docs/architecture/12-decisions/decision-packages/seven-problem-review/17-presentation-results.md §2,5\n'+ROOT+'/results/architecture-evaluation/current/four-dp-presentation-20261019/dp04/normalized-comparison.md\n'+ROOT+'/results/architecture-evaluation/current/four-dp-presentation-20261019/dp04/comparison.md\n스타일: DP2 v2, 사용자 레퍼런스18-DP02채택,23-DP04방안,24-DP04채택.\n'+extra;
}

// 1. Editable event schematic: the referent belongs to an earlier observation.
{
 const s=pres.slides.add();header(s,' (배경)',1);
 txt(s,'말이 끝났을 때, 가리킨 화면은 이미 바뀌어 있다',66,145,1785,62,43,{bold:true});
 txt(s,'VIA는 발화, 당시 화면, 포인터와 선택 범위를 연결해 실제 대상을 정해야 한다.',68,213,1760,40,28);
 const sx=[85,665,1245];
 const names=['① 표 범위 선택','② 그래프 두 개 지정','③ 화면 이동과 부분 정정'];
 names.forEach((v,i)=>{
  txt(s,v,sx[i],287,490,42,30,{bold:true});
  shape(s,'rect',sx[i],341,490,270,C.white,C.black,2);
  shape(s,'rect',sx[i],341,490,38,'#E9EBED',C.black,1.1);
  txt(s,['매출 현황.xlsx','실적 분석.pdf','메일'][i],sx[i]+16,341,455,38,21,{bold:true});
 });
 const tab=nativeTable(s,[['구분','상반기','하반기'],['제품 A','120','150'],['제품 B','90','110'],['제품 C','80','95']],106,394,[150,145,145],[39,47,47,47],22);
 highlightHeader(tab,3);
 shape(s,'rect',255,433,290,94,'none',C.blue,4);
 txt(s,'실제 선택 범위',278,578,245,25,19,{color:C.blue,bold:true});
 // Two graph objects and an open spatial designation are schematic, not measured data.
 [[686,398,'매출'],[918,398,'이익']].forEach(([x,y,title],i)=>{
  shape(s,'rect',x,y,216,176,C.white,C.grid,1.4);txt(s,title,x+10,y+8,196,27,23,{bold:true});
  poly(s,[[x+25,y+137],[x+25,y+50]],C.black,1.3);
  poly(s,[[x+25,y+137],[x+192,y+137]],C.black,1.3);
  poly(s,[[x+33,y+123],[x+73,y+(i?90:108)],[x+115,y+85],[x+175,y+51]],C.blue,3);
 });
 const region=shape(s,'ellipse',680,386,465,187,'none',C.red,3);region.line=line(C.red,3,'dashed');
 txt(s,'포인터로 둘러싼 영역',745,578,350,25,19,{color:C.red,bold:true,align:'center'});
 txt(s,'받은 편지함',1270,397,425,35,26,{bold:true});
 [0,1,2].forEach(i=>{poly(s,[[1270,478+i*50],[1707,478+i*50]],C.grid,1);txt(s,['회의 자료 확인','다음 주 일정','검토 요청'][i],1271,442+i*50,420,35,23);});
 poly(s,[[582,475],[642,475]],C.black,2.3,false,true);
 poly(s,[[1162,475],[1222,475]],C.black,2.3,false,true);
 box(s,'“이 매출과…”',85,642,490,70,{fill:C.cream,size:31,bold:true});
 box(s,'“이 두 그래프를 비교해줘.”',665,642,490,70,{fill:C.cream,size:29,bold:true});
 box(s,'“아니, 오른쪽 것만.”',1245,642,490,70,{fill:C.cream,size:30,bold:true});
 poly(s,[[1490,712],[1490,756],[1194,756],[1194,509],[1155,509]],C.red,2.7,true,true);
 txt(s,'정정은 현재 메일이 아니라 앞선 그래프를 가리킨다',280,793,1370,47,34,{bold:true,color:C.red,align:'center'});
 poly(s,[[65,881],[1830,881]],C.blue,1.5);
 txt(s,'이 DP의 질문',66,918,280,56,34,{bold:true,color:C.red});
 txt(s,'원본을 함께 해석할까, 화면·발화 후보를 먼저 만들어 결합할까?',365,913,1475,62,35,{bold:true});
 txt(s,'대상의 범위와 관계를 생산하는 SW 책임과 중간 상태를 선택한다.',365,986,1460,42,28);
 s.speakerNotes.textFrame.setText(notes('배경은 설계 설명용 편집 가능한 개념도이며 실측 screenshot이 아니다. 표 숫자와 그래프 선은 설명용 예시로 QA 데이터가 아니다. 당시 화면/selection/path 보존과 시간 대응은 공통 기능이고, 시간 일치만으로 의도가 확정되지는 않는다.'));
}

// 2 and 3 share the whole native comparison table and all base geometry.
function comparison(selected){
 const s=pres.slides.add();header(s,selected?' (선택 및 보완)':' (설계 방안 비교)',selected?3:2);
 txt(s,selected?'정규화 계약의 기본 A 선택 / A2+A5는 pixel 계약에서 확인한 별도 복구 대안':'A는 원본에서 공동 생산 / B는 생산자별 후보를 게시한 뒤 결합',28,124,1835,39,28,{bold:true,color:selected?C.blue:C.black});
 const vals=[
 ['설계안','(A) 원본 공동 해석','(B) 발화·화면 후보 생산 후 결합'],
 ['구조','',''],
 ['장점','별도 후보 게시를 기다리지 않고 대상·관계를 공동 생산\n발화에 맞는 자유로운 범위를 원본에서 직접 판단','유효 후보를 같은 판단 안에서 반복 결합\n화면·발화의 해석 근거와 오류 지점을 분리'],
 ['단점','좌표·대상 혼동 시 공동 판단을 다시 수행\n큰 원본과 복합 관계를 함께 처리하는 부담','생산·게시·확장과 결합 의존 및 상태 추가\n후보의 범위 오류가 최종 결합을 막을 수 있음'],
 ['QA01 첫 응답 (초)','7.716 (15/15)','18.774 (12/15)'],
 ['QA02 지칭 집합','24/24 (100%)','18/24 (75.0%)'],
 ['QA03 처리 (초)','9.646 (12/12)','25.033 (9/12)'],
 ['QA04 변경 (개)','미측정 (이전 판본 2.50)','미측정 (이전 판본 2.50)'],
 ['QA05 모델 비용 ($)','0.0254572','0.0650212'],
 ['QA06 RSS (MB)','40.141','41.894']];
 const t=nativeTable(s,vals,24,174,[172,850,850],[46,426,58,90,32,56,32,32,32,32],20);
 highlightHeader(t,3);
 t.cells.block({row:1,column:0,rowCount:1,columnCount:1}).assign({textStyle:{typeface:FONT,fontSize:29,bold:true},anchor:'top',margins:{top:16,left:12,right:12,bottom:0}});
 t.cells.block({row:2,column:0,rowCount:2,columnCount:1}).assign({textStyle:{typeface:FONT,fontSize:25,bold:true}});
 t.cells.block({row:3,column:1,rowCount:1,columnCount:2}).assign({textStyle:{typeface:FONT,fontSize:20,color:C.red,alignment:'center'}});
 t.cells.block({row:4,column:0,rowCount:6,columnCount:1}).assign({fill:'#F7F8FA',textStyle:{typeface:FONT,fontSize:19,bold:true,alignment:'left'}});
 shape(s,'rect',40,292,139,69,C.cream,C.black,2.7);txt(s,'Component',44,294,130,27,20,{bold:true,align:'center'});
 shape(s,'rect',49,326,121,26,C.white,C.black,1.3);txt(s,'내부 Module',51,326,117,26,18,{align:'center'});
 storeShape(s,'보존 상태',42,398,135,60,{size:20});
 txt(s,'실선: 처리·자료\n점선: 조건부 경로',33,482,157,56,18,{align:'center'});
 txt(s,'모듈별 모델 복제나\n독립 process 아님',33,555,157,49,17,{align:'center',color:C.muted});
 if(selected)txt(s,'파란 점선:\n별도 복구 대안',33,609,157,34,16,{align:'center',color:C.blue});

 // A: two raw evidence paths meet in a single semantic writer and one state.
 label(s,'대상 범위와 지칭 관계를 한 책임에서 생산',207,230,828,25);
 storeShape(s,'발화 원문\n시각·정정',214,309,189,77,{size:23});
 storeShape(s,'당시 화면\n선택·pointer',214,420,189,85,{size:23});
 component(s,'Request Interpreter',425,272,596,316);
 box(s,'Grounding Attempt Control / Joint Work Queue',445,315,556,36,{fill:C.white,size:20});
 box(s,'Joint Grounding\nInterpreter',445,371,340,88,{fill:C.white,size:27,bold:true});
 storeShape(s,'Joint Grounding\nState',815,420,186,104,{size:21});
 poly(s,[[403,346],[420,346],[420,395],[445,395]],C.black,2.2,false,true);
 poly(s,[[403,461],[420,461],[420,438],[445,438]],C.black,2.2,false,true);
 poly(s,[[615,351],[615,371]],C.black,2,false,true);
 poly(s,[[615,459],[615,484]],C.black,2.2,false,true);
 n(s,1,447,357);
 box(s,'TryApply / 현재성·기하 검증',445,484,340,40,{fill:C.white,size:22});
 n(s,2,447,470);
 poly(s,[[785,504],[800,504],[800,472],[815,472]],C.black,2.2,false,true);
 n(s,3,793,406);
 if(selected){
  box(s,'pixel 복구 대안 (미적용): A2 패치 + A5 좌표 view',445,545,556,33,{fill:C.pale,stroke:C.blue,size:19,color:C.blue,dash:true});
  poly(s,[[722,524],[722,545]],C.blue,2,true,true);
  poly(s,[[445,562],[433,562],[433,453],[445,453]],C.blue,2,true,true);
 }else{
  txt(s,'추가 근거가 필요하면 같은 책임에서 재판단',446,548,555,30,21,{align:'center'});
  poly(s,[[445,504],[433,504],[433,453],[445,453]],C.black,1.8,true,true);
 }
 box(s,'Request Controller: 현재성 확인과 최종 채택',425,606,596,32,{fill:C.white,size:23,bold:true});
 poly(s,[[1001,498],[1008,498],[1008,606]],C.black,2,false,true);

 // B: candidate producer ownership and staged binding are explicit.
 label(s,'원본별 생산자와 후보 저장, 후속 결합을 분리',1056,230,830,25);
 txt(s,'발화 원문·시각',1290,269,226,29,21,{align:'center'});
 txt(s,'당시 frame·선택·pointer',1550,269,302,29,21,{align:'center'});
 component(s,'Request Interpreter',1074,304,794,284);
 box(s,'Grounding\nCoordinator',1088,351,167,65,{fill:C.white,size:22,bold:true});
 txt(s,'역할 queue\n예산·완료\n확장 요청',1092,432,159,96,23,{align:'center'});
 box(s,'Speech Reference\nProducer',1280,351,241,65,{fill:C.white,size:23,bold:true});
 box(s,'Screen Target\nProducer',1547,351,307,65,{fill:C.white,size:23,bold:true});
 poly(s,[[1400,298],[1400,351]],C.black,2,false,true);
 poly(s,[[1700,298],[1700,351]],C.black,2,false,true);
 poly(s,[[1255,382],[1280,382]],C.black,1.8,true,true);
 poly(s,[[1171,351],[1171,345],[1700,345],[1700,351]],C.black,1.5,true,true);
 storeShape(s,'Speech Candidate\nRevisions',1280,434,241,48,{size:19});
 storeShape(s,'Screen Candidate Revisions\n범위·관계·coverage',1547,434,307,48,{size:18});
 poly(s,[[1400,416],[1400,434]],C.black,2,false,true);
 poly(s,[[1700,416],[1700,434]],C.black,2,false,true);
 box(s,'Multimodal Binding Resolver',1280,504,574,38,{fill:C.white,size:24,bold:true});
 poly(s,[[1400,482],[1400,504]],C.black,2,false,true);
 poly(s,[[1700,482],[1700,504]],C.black,2,false,true);
 storeShape(s,'검증 후 Binding Revision State / 후보 read-set',1280,557,574,25,{size:17});
 poly(s,[[1556,542],[1556,557]],C.black,2,false,true);
 poly(s,[[1854,524],[1879,524],[1879,383],[1854,383]],C.black,1.8,true,true);
 txt(s,'공통 현재성 검사 후\n후보 게시',1088,536,167,45,17,{align:'center',color:C.muted});
 n(s,1,1260,354);n(s,2,1251,438);n(s,3,1251,507);
 box(s,'Request Controller: 현재성 확인과 최종 채택',1074,606,794,32,{fill:C.white,size:23,bold:true});
 poly(s,[[1556,582],[1556,606]],C.black,2,false,true);

 if(selected){
  shape(s,'rect',196,174,850,836,'none',C.blue,4.2,'Selected A column');
  txt(s,'✓',204,148,67,79,65,{color:C.red,bold:true});
  const con=t.getCell(3,1);con.value+='\n[복구 대안] A2+A5 / 정규화 A에는 추가 적용하지 않음';cellStyle(con,20,C.red);
  con.text.get('[복구 대안] A2+A5 / 정규화 A에는 추가 적용하지 않음').fill=C.blue;
  const qa=t.getCell(5,1);qa.value='24/24 (정규화 기본)\n[별도 pixel 복구] 19/24 → 24/24';cellStyle(qa,20);
  qa.text.get('[별도 pixel 복구] 19/24 → 24/24').fill=C.blue;
  txt(s,'선택 A의 이번 QA 열세는 확인되지 않음. 파란 복구 경로와 전후 수치는 별도 pixel 실험이며, 정규화 A와의 조합은 미실행.',28,1016,1810,27,20,{color:C.blue});
 }else{
  txt(s,'공통 계약: bbox 0~1000 출력 후 pixel 변환. B 기본은 순차 생산, 충분한 후보는 코드 결합 가능. PoC는 Qwen vision 생산.',28,1016,1840,27,20,{bold:true});
 }
 txt(s,'정규화 24회: 안별 4사례×3 / 시간 괄호: 관측·예정 / 비용: 12회 합계 / RSS: 로컬 실행기 / B의 미완료 3회 유지',28,1047,1805,25,19,{color:C.muted});
 s.speakerNotes.textFrame.setText(notes('기본 구조와 선택 구조의 공통 표 및 B의 모든 도형은 동일하다. 파란 점선은 pixel 검증 거절 시 별도 선택 가능한 복구로 현재 정규화 기본 A에 적용했다고 뜻하지 않는다. QA02의 두 줄은 서로 다른 계약이다. 정규화 A24/24에 pixel19/24를 연결하지 않는다. 기하 검증과 원본 보존은 공통 계약이며 그림에서 생략한 B의 TryApply도 유지한다. B 후보의 확장은 원 소유자에게 돌아가며 도식은 화면 확장 한 경로를 대표로 표시했다. Request Controller의 입력 등록은 상단 원본 공급과 함께 생략한 공통 흐름이다.'));
}
comparison(false);comparison(true);

// 4. Only same-contract pixel measurements form the before/after comparison.
{
 const s=pres.slides.add();header(s,' (보완 효과 상세)',4);
 txt(s,'pixel 좌표 계약의 A2+A5: 거절된 지칭을 복구하되 지연과 메모리를 더 사용',28,126,1840,44,31,{bold:true,color:C.blue});
 txt(s,'현재 선택은 정규화 기본 A. 아래는 별도 pixel 실험의 A와 A2+A5 전후 비교다.',28,178,1830,35,26);
 shape(s,'rect',24,232,612,742,'none',C.blue,3);
 shape(s,'rect',24,232,612,50,C.green,'none',0);
 txt(s,'거절 시 실행하는 복구 경로',44,237,572,40,29,{bold:true,align:'center'});
 box(s,'Joint Grounding Interpreter\n원본에서 대상과 관계 판단',79,308,502,81,{fill:C.white,size:27,bold:true});
 poly(s,[[330,389],[330,422]],C.black,2.4,false,true);
 box(s,'source·revision·기하 검증',79,422,502,52,{fill:C.white,size:27});
 txt(s,'검증 거절',348,484,220,29,23,{color:C.red,bold:true});
 poly(s,[[330,474],[330,529]],C.red,2.5,true,true);
 box(s,'A5  영향 범위의 좌표 view\nraw 지시점 crop 또는 전체 격자',79,529,502,91,{fill:C.pale,stroke:C.blue,color:C.blue,size:25,bold:true});
 poly(s,[[330,620],[330,657]],C.blue,2.6,false,true);
 box(s,'A2  영향 표현만 다시 판단·패치\n비영향 표현의 binding은 유지',79,657,502,91,{fill:C.pale,stroke:C.blue,color:C.blue,size:25,bold:true});
 poly(s,[[330,748],[330,786]],C.blue,2.6,false,true);
 box(s,'전체 현재성 재검증 후 채택',79,786,502,57,{fill:C.white,size:26,bold:true});
 txt(s,'동일 Joint 책임에서 수행\n원본 PNG와 overlay 유지',52,872,558,67,25,{align:'center'});
 txt(s,'QA02  19/24 (79.2%) → 24/24 (100%)',671,232,1216,52,37,{bold:true,color:C.blue});
 const v=[['지표','pixel A','pixel A2+A5','해석'],
  ['QA01 첫 응답 (초)','9.268 (12/15)','11.036 (15/15)','미관측 3건 회복'],
  ['같은 관측 12건의 QA01','9.268','10.875','+1.607초 (+17.33%)'],
  ['QA02 지칭 집합','19/24','24/24','5건 추가 정답'],
  ['QA03 처리 (초)','12.559 (10/12)','13.795 (12/12)','미완료 2건 회복'],
  ['QA04 변경 (개)','미측정','미측정','이전 값을 전용하지 않음'],
  ['QA05 확정 비용 ($)','0.0339156\n+ 사용량 미확정 1회','0.0413400','총비용 증감 미확정'],
  ['QA06 최고 RSS (MB)','41.452','50.037','+8.585 MB'],
  ['실제 모델 요청 (회)','22','23','거절·복구 포함']];
 const t=nativeTable(s,v,671,306,[337,280,280,327],[43,56,56,56,56,56,77,56,51],23);highlightHeader(t,4);
 [1,3,4].forEach(r=>cellStyle(t.getCell(r,3),23,C.blue,true));
 [2,7].forEach(r=>cellStyle(t.getCell(r,3),23,C.red,true));
 txt(s,'비용 미확정 1회는 예약 상한 $0.0729404를 별도 보존. 실제 청구액이 아님.',674,837,1210,37,22,{color:C.red});
 txt(s,'네 사례×3회씩 비교. A2와 A5 각각의 효과를 분리한 실험은 아님.',674,883,1210,34,24);
 txt(s,'정답 bbox 미공급. 좌표 오류·공급자 오류를 포함한 기본 실패도 보존.',674,930,1210,34,24);
 poly(s,[[28,997],[1894,997]],C.blue,1.4);
 txt(s,'정규화 A + A2+A5는 미실행. 정규화 계약의 24/24와 pixel 복구의 24/24를 합산하거나 같은 조합의 성과로 해석하지 않는다.',28,1010,1810,31,21,{bold:true});
 txt(s,'기존 완료 발표 캠페인의 별도 실험. 실제 UI·음성·전체 VIA와 원격 모델 가중치는 측정 범위 밖.',28,1044,1800,26,19,{color:C.muted});
 s.speakerNotes.textFrame.setText(notes('4장 수치는 comparison.md의 pixel BASIC와 A2A5 구성 각12회에서만 가져왔다. QA01 같은 관측12건의9.268→10.875는 모든15입력의 평균 비교가 아니다. QA06 차이50.036736-41.451520=8.585216MB. QA02 증가5/24=20.833percentage points. 사용량 미확정1회는 HTTP400 provider 오류 관련이며 실제총비용 증감으로 단정하지 않는다. 원본과 coordinate view를 함께 주고 gate 거절 후 한정 복구하며 정답 bbox를 넣지 않았다.'));
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
const out=path.join(BUILD,'output','VIA-DP3-reference-style.pptx');
const result=await finalizePresentation({workspaceDir:BUILD,candidatePath:draft,finalPath:out,pythonExecutable:PY,integrityValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_layout_geometry.py'),explicitTotalSlideCount:4,requiredNativeTableOwnerSlides:[1,2,3,4],fontPolicy:{basis:'design',families:[FONT]},verifyArtifactToolImport:true,layoutArgs:['--expected-slide-size-emu',String(W*9525)+','+String(H*9525),'--validate-bullet-geometry','--validate-heading-fit','--require-native-table-slide','1','--require-native-table-slide','2','--require-native-table-slide','3','--require-native-table-slide','4'],receiptPath:path.join(BUILD,'validation.json')});
console.log(JSON.stringify({output:out,result},null,2));
