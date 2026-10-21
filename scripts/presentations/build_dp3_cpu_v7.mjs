// DP3 v7: the same editable geometry drives PPTX, SVG and draw.io.
import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {Presentation,PresentationFile,FileBlob} from '@oai/artifact-tool';
const [ROOT,BUILD]=process.argv.slice(2);
const SKILL='/Users/changbae/.codex/plugins/cache/openai-primary-runtime/presentations/26.1017.22135/skills/presentations';
const PY='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3';
process.env.RUNTIME_NODE_MODULES='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules';
const FONT='Calibri',W=1920,H=1080;
const C={black:'#111111',gray:'#98999B',grid:'#B7B9B9',green:'#C9E1B3',greenD:'#70A843',cream:'#FFF6DD',blue:'#4676CB',cyan:'#35A8CD',pale:'#F1F6FC',red:'#C00000',muted:'#50555B',white:'#FFFFFF'};
const src=path.join(BUILD,'source-v6.pptx');
const reference=await PresentationFile.importPptx(await FileBlob.load(src));
await fs.writeFile(path.join(BUILD,'reference-inspection.ndjson'),(await reference.inspect({select:'$.slides[1,5,16]',include:['id','index','layout','elements.text','elements.position','elements.style']})).ndjson);
await fs.writeFile(path.join(BUILD,'reference-masters.json'),JSON.stringify({masters:reference.masters.items.map(x=>({id:x.id,name:x.name})),layouts:reference.layouts.items.map(x=>({id:x.id,name:x.name,placeholders:x.placeholders.summary()}))},null,2));
const original=await fs.readFile(path.join(ROOT,'scripts/presentations/generate_dp3_reference_style.mjs'),'utf8');
const hh=original.slice(original.indexOf('const line='),original.indexOf('function notes('));
const hp=path.join(BUILD,'helpers-v7.mjs');await fs.writeFile(hp,`export function helpers(ctx){const {FONT,W,H,C}=ctx;\n${hh}\nreturn {shape,txt,poly,storeShape,nativeTable,highlightHeader};}`);
const h=(await import(pathToFileURL(hp))).helpers({FONT,W,H,C});
const pres=Presentation.create({slideSize:{width:W,height:H}});let scene=[],scenes=[];
function rect(s,x,y,w,hh,fill=C.cream,dash=false,geom='rect',stroke=C.black){const q=h.shape(s,geom,x,y,w,hh,fill,stroke,2);if(dash)q.line={fill:stroke,width:2,style:'dashed'};scene.push({kind:'shape',geom,x,y,w,h:hh,fill,stroke,dash});return q;}
function text(s,t,x,y,w,hh,size=23,opt={}){h.txt(s,t,x,y,w,hh,size,opt);scene.push({kind:'text',t,x,y,w,h:hh,size,bold:!!opt.bold,color:opt.color||C.black,align:opt.align||'left'});}
function line(s,pts,{dash=false,arrow=false,color=C.black,width=2}={}){h.poly(s,pts,color,width,dash,arrow);scene.push({kind:'line',pts,dash,arrow,color,width});}
function label(s,t,x,y,w,size=21){rect(s,x,y,w,29,C.white,false,'rect',C.white);text(s,t,x,y,w,29,size,{align:'center'});}
function comp(s,t,x,y,w,hh,size=25){rect(s,x,y,w,hh);text(s,t,x+7,y+2,w-14,33,size,{bold:true,align:'center'});line(s,[[x,y+37],[x+w,y+37]],{width:1.5});}
function mod(s,t,x,y,w,hh,size=23){rect(s,x,y,w,hh,C.white,false,'roundRect');text(s,t,x+6,y+2,w-12,hh-4,size,{align:'center'});}
function store(s,t,x,y,w,hh,size=21){h.storeShape(s,t,x,y,w,hh,{fill:C.white,size});scene.push({kind:'store',t,x,y,w,h:hh,size});}
function external(s,t,x,y,w,hh,size=23){rect(s,x,y,w,hh,C.pale,false,'hexagon');text(s,t,x+20,y+3,w-40,hh-6,size,{bold:true,align:'center'});}
function proc(s,t,x,y,w,hh){rect(s,x,y,w,hh,'none',true,'rect',C.blue);text(s,t,x+12,y+3,w-24,29,24,{bold:true,color:C.blue});}
function header(s,title,page,sub){s.background.fill=C.white;scene=[];text(s,title,60,29,1140,68,44,{bold:true});['과제 소개','요구사항 분석','설계','구현 및 검증','결론'].forEach((v,i)=>{const x=1210+i*134;rect(s,x,33,143,44,i===2?C.cyan:C.gray,false,'chevron',C.white);text(s,v,x+14,33,111,44,19,{bold:true,color:C.white,align:'center'});});line(s,[[58,108],[1860,108]],{color:'#A7A7A7',width:3});text(s,sub,60,129,1800,47,28,{bold:true});text(s,String(page),1835,1049,38,20,17,{align:'right',color:C.muted});}
function grid(s,a,b){rect(s,24,188,172,54,C.green,false,'rect',C.grid);text(s,'방안',24,188,172,54,27,{bold:true,align:'center'});for(const [x,t] of [[196,a],[1046,b]]){rect(s,x,188,850,54,C.green,false,'rect',C.grid);text(s,t,x+8,188,834,54,28,{bold:true,align:'center'});}line(s,[[24,242],[1896,242]],{color:C.grid,width:1.2});line(s,[[196,188],[196,1018]],{color:C.grid,width:1.2});line(s,[[1046,188],[1046,1018]],{color:C.grid,width:1.2});text(s,'구조',42,270,135,40,29,{bold:true,align:'center'});text(s,'노란 경계\nComponent\n\n내부 둥근 상자\nModule\n\n원통\n소유 상태\n\n파란 점선\nOS Process\n\n육각형\n외부 모델\n\n실선 요청\n점선 반환',36,350,145,555,20,{align:'center'});}
function note(s,t){s.speakerNotes.textFrame.setText(`DP3 architecture v7. Representation and ownership correction only. All numeric assumptions remain DP3-GROUNDING-CPU-SYN-1.0 / SYNTHETIC_AUTHORED_NOT_MEASURED. Source contract: ${ROOT}/docs/architecture/12-decisions/decision-packages/seven-problem-review/dp3-grounding-architecture-20261021/presentation-architecture-contract.md. Models and queues are design proposals, not implemented components. Same ASR/timestamps and cloud S2S/meaning LLM.\n${t}`);}
function foot(s,t){text(s,t,210,1020,1630,27,20,{color:C.muted});}
// p10: logical responsibility, candidate authority and final adoption.
{
 const s=pres.slides.add();header(s,'DP3. 화면 지칭 해석 구조 (1/2)',10,'공통 입력: Interaction Manager의 화면·동작 이력 + local ASR의 전사·단어 시각 / 화면 근거가 필요한 요청만 진입');grid(s,'A  원본 공동 해석','B-R1  후보 생산 + 단일 worker 교대 로딩');
 comp(s,'Request Controller',220,268,430,100);mod(s,'Request Revision Guard',239,314,392,42);text(s,'최종 의미 채택',240,377,390,27,21,{color:C.blue,align:'center'});
 comp(s,'Context Manager',790,268,220,392,23);store(s,'허가된 원본 참조\nframe / pointer\nselection / native\n읽기 cache (RAM)',804,320,192,171,21);store(s,'채택 binding 복사\n읽기 receipt',804,523,192,93,21);
 comp(s,'Request Interpreter',220,440,430,220);mod(s,'Joint Grounding Interpreter',240,489,390,48,24);store(s,'해석 세션 (RAM)\n원본 참조 / 후보 해석 / 이전 binding',240,563,390,77,21);
 line(s,[[280,368],[280,440]],{arrow:true});label(s,'1  해석 요청',223,406,153);line(s,[[610,440],[610,368]],{dash:true,arrow:true});label(s,'6  proposal',465,406,160);
 line(s,[[650,320],[790,320]],{arrow:true});label(s,'조회 허가',659,281,123,20);line(s,[[650,505],[790,505]],{arrow:true});label(s,'2 Read / Crop',655,471,130,20);line(s,[[790,604],[650,604]],{dash:true,arrow:true});label(s,'3  원본 + 판본',655,568,130,20);
 comp(s,'Model Access',220,737,430,100);mod(s,'Cloud Model Client',240,783,390,42);external(s,'Cloud 의미 LLM',790,737,220,100,24);
 line(s,[[280,660],[280,737]],{arrow:true});label(s,'4  원본 + 전사',223,685,176);line(s,[[610,737],[610,660]],{dash:true,arrow:true});label(s,'5  해석 / 추가 읽기',427,685,205,20);
 line(s,[[650,775],[790,775]],{arrow:true});label(s,'Infer',666,740,111);line(s,[[790,812],[650,812]],{dash:true,arrow:true});label(s,'결과',666,817,111);
 text(s,'원본 범위와 지칭 관계를 같은 해석 세션에서 판단\n부족한 crop·과거 관측은 2–5 경로로 재조회',230,886,770,78,25,{bold:true});
 // B hosts same control authority but separately owns candidates.
 comp(s,'Request Controller',1070,268,360,100,24);mod(s,'Request Revision Guard',1087,314,326,42,22);text(s,'최종 의미 채택',1080,377,340,27,21,{color:C.blue,align:'center'});
 comp(s,'Context Manager',1530,268,330,100,24);store(s,'허가된 원본 참조·cache (RAM)',1548,313,294,43,20);
 comp(s,'Request Interpreter',1070,440,360,220,24);mod(s,'Multimodal Binding Resolver',1086,488,328,48,22);store(s,'해석 세션 (RAM)\n전사·시각 / candidate revision',1086,563,328,77,21);
 comp(s,'Screen Target Producer',1530,440,330,220,23);mod(s,'Candidate Assembler',1546,489,298,48,22);store(s,'대상·동작 기록 (RAM)\nbbox / mask·track 참조\n출처·판본·불확실성',1546,551,298,91,20);
 line(s,[[1120,368],[1120,440]],{arrow:true});label(s,'1  해석 요청',1071,406,150);line(s,[[1398,440],[1398,368]],{dash:true,arrow:true});label(s,'7  proposal',1248,406,170);
 line(s,[[1430,300],[1530,300]],{arrow:true});label(s,'조회 허가',1434,263,92,19);line(s,[[1430,340],[1480,340],[1480,440],[1530,440]],{arrow:true});label(s,'생산 허가',1434,375,92,19);line(s,[[1430,503],[1530,503]],{arrow:true});label(s,'2 Produce',1434,467,92,19);line(s,[[1530,607],[1430,607]],{dash:true,arrow:true});label(s,'5 후보 rev',1434,570,92,19);
 line(s,[[1600,440],[1600,368]],{arrow:true});label(s,'Read / Crop',1540,404,130,19);line(s,[[1815,368],[1815,440]],{dash:true,arrow:true});label(s,'원본 반환',1720,404,130,19);
 comp(s,'Model Access  (VIA host 배치)',1070,737,790,100,25);mod(s,'Cloud Model Client',1087,782,326,43,22);mod(s,'Local Model Execution Controller',1527,782,316,43,20);
 line(s,[[1130,660],[1130,737]],{arrow:true});label(s,'6 후보 + crop + 전사',1075,685,238,21);line(s,[[1385,737],[1385,660]],{dash:true,arrow:true});
 line(s,[[1590,660],[1590,737]],{arrow:true});label(s,'3 RunUI / 필요 시 SAM',1490,685,254,20);line(s,[[1815,737],[1815,660]],{dash:true,arrow:true});label(s,'4 결과',1765,685,90,20);
 external(s,'Cloud 의미 LLM',1090,910,320,79,24);proc(s,'R1 Vision Worker process',1515,892,350,108);text(s,'Model Access의 worker 배치\nUI / SAM adapter + CPU runtime\n현재 model session + episode',1531,927,318,65,19,{align:'center'});
 line(s,[[1160,837],[1160,910]],{arrow:true});label(s,'Infer',1092,861,101);line(s,[[1360,910],[1360,837]],{dash:true,arrow:true});label(s,'결과',1307,861,90);
 line(s,[[1590,837],[1590,892]],{arrow:true});label(s,'IPC 명령',1523,856,130,20);line(s,[[1815,892],[1815,837]],{dash:true,arrow:true});label(s,'완료',1764,856,90,20);
 foot(s,'Read·Produce는 기허가 범위. B의 부족한 후보는 2로 Expand 재요청. 새 범위는 Request Controller가 허가. native 선택이면 worker 생략, bbox로 충분하면 SAM 생략.');
 note(s,'A 원본 읽기와 추가 해석을 허용. B는 Screen Target Producer가 소유하는 후보 revision을 Request Interpreter가 읽으며, 후보 부족 시 같은 생산기에 Expand를 요청한다. 6의 Infer 반환은 같은 호스트 모듈로 돌아온다. 모든 proposal은 Request Controller가 input/source revision, 권한, 현행성을 검증해 채택한다. Context Manager 원본은 Interaction Manager 관측과 동일 전사/시각에서 온다. 읽기 그림은 이미 허가된 범위이며 새 범위는 Request Controller가 별도 승인한다. Model Access의 host/worker는 같은 논리 Component의 분산 배치이다. R1 worker 내부 상세 및 episode와 model session의 서로 다른 수명은 12쪽. 번호는 각 대안 안에서 실행 순서를 나타낸다. 2/3 원본 재조회는 반복 가능하며 A의 호출횟수를1회로제한하지않는다.');
 scenes.push({name:'dp3-v7-ab-responsibilities',items:scene});
}
// p11: explicit process allocation. UI result MUST return to host before SAM admission.
{
 const s=pres.slides.add();header(s,'DP3. 화면 지칭 해석 구조 (2/2)',11,'B의 후보·결합 LLM은 동일 / 시각 CPU 총 4 thread·heavy stage 순차 / UI seed가 필요한 SAM은 UI 결과 반환 후 실행');grid(s,'B-R2  한 process에 사용한 모델 유지','B-R3  UI / SAM process 별도 유지');
 for(const base of [220,1070]){
  proc(s,'VIA Host process',base,267,790,252);
  comp(s,'Screen Target Producer',base+18,309,310,191,22);mod(s,'Candidate Assembler',base+34,354,278,40,21);store(s,'대상·동작 기록 (RAM)\n후보 rev / 출처 / mask 참조',base+34,412,278,71,20);
  comp(s,'Model Access',base+449,309,323,191,25);mod(s,'Local Model\nExecution Controller',base+465,354,291,67,23);text(s,'CPU 허가 / queue·generation\n모델 수명 / 재시작 책임',base+466,438,289,48,20,{align:'center'});
  line(s,[[base+328,375],[base+449,375]],{arrow:true});label(s,'생산 요청',base+333,338,111,20);line(s,[[base+449,455],[base+328,455]],{dash:true,arrow:true});label(s,'후보 재료',base+333,465,111,20);
 }
 proc(s,'Vision Worker process  (단일 주소 공간)',220,639,790,350);
 comp(s,'Model Access  (worker 배치)',238,680,754,289,25);mod(s,'IPC Endpoint / Job Queue  (한 stage씩)',254,727,722,44,24);
 mod(s,'UI Adapter',254,801,338,45,23);mod(s,'SAM Adapter',638,801,338,45,23);
 store(s,'UI model session\n검출 + OCR weights',254,867,238,65,20);store(s,'SAM model session\nTiny weights',508,867,225,65,20);store(s,'SAM episode\nframe features / tensor',749,867,227,65,19);
 text(s,'공유 Python / PyTorch CPU runtime',255,937,720,27,22,{bold:true,align:'center'});
 line(s,[[710,519],[710,639]],{arrow:true});label(s,'1 RunUI / 3 RunSegment·Track',430,556,330,21);line(s,[[940,639],[940,519]],{dash:true,arrow:true});label(s,'2 UI 결과 / 4 mask·track',763,594,240,20);
 line(s,[[400,771],[400,801]],{arrow:true});line(s,[[807,771],[807,801]],{arrow:true});
 // Two worker processes. Their calls are sequential via the host, not a fork/join.
 for(const [x,title,adapter,model,episode] of [[1070,'UI Worker process','UI Adapter','UI Model Session\n검출 / OCR','UI job\nworkspace'],[1490,'SAM Worker process','SAM Adapter','SAM Model\nSession / Tiny','SAM episode\n+ workspace']]){
  proc(s,title,x,639,370,350);comp(s,'Model Access (worker 배치)',x+15,680,340,289,21);mod(s,'IPC Endpoint / Job Queue',x+28,727,314,44,21);mod(s,adapter,x+28,801,314,45,23);store(s,model,x+28,867,167,65,18);store(s,episode,x+207,867,135,65,17);text(s,'전용 Python / PyTorch runtime',x+24,938,322,25,20,{bold:true,align:'center'});line(s,[[x+185,771],[x+185,801]],{arrow:true});
 }
 line(s,[[1550,519],[1550,550],[1165,550],[1165,639]],{arrow:true});label(s,'1 RunUI',1090,566,144,21);
 line(s,[[1380,639],[1380,585],[1620,585],[1620,519]],{dash:true,arrow:true});label(s,'2 UI seed 반환',1290,602,180,20);
 line(s,[[1730,519],[1730,639]],{arrow:true});label(s,'3 필요 시 SAM',1597,553,176,21);
 line(s,[[1835,639],[1835,519]],{dash:true,arrow:true});label(s,'4 mask·track',1745,600,142,20);
 text(s,'Model Access는 동일 Component의 분산 배치. process 경계의 호출선은 각 worker의 IPC Endpoint에 연결.',210,991,1630,27,20,{color:C.muted});foot(s,'R2: worker crash 시 두 session 함께 재시작. R3: SAM crash에도 UI session 유지. host 후보 기록은 두 안 모두 유지하며 낡은 generation 결과는 거절.');
 note(s,'p10의 Request Controller, Request Interpreter, cloud Model Access client는 공통이므로 이 배치 상세에서는 생략했다. 각 안의 Screen Target Producer는 p10과 같은 인스턴스다. Model Access는 host scheduler와 worker adapter/runtime를 포함하는 하나의 논리 Component이며 process마다 복제된 component권위가 아니다. UI/SAM session은 최초사용후상주. 기본heavy stage1개. 1UI명령→2UI결과host반환→seed검토→3필요시SAM→4mask결과host반환이다. R3의 fanout/join 없음. 한runtime의모델상주R2와별도runtimeR3차이는모델동시실행과별개. 작은candidate기록host, 실제SAMtensorworker. R2 crash는공유주소공간의두session영향,R3SAMcrash는UI주소공간보존. stalegeneration결과는host거절하고권한있는원본과seed로재생산한다.');scenes.push({name:'dp3-v7-resident-processes',items:scene});
}
// p12: real runtime composition and model/episode lifecycle, with unchanged RAM ledger.
{
 const s=pres.slides.add();header(s,'DP3. CPU 실행과 상태 수명',12,'R1과 R2는 같은 코드·process 배치 / 모델 session의 유지 정책만 변경 / R3는 worker 주소 공간과 runtime을 분리');
 proc(s,'VIA Host process',60,205,520,356);comp(s,'Model Access (host 배치)',78,247,484,293,25);mod(s,'Local Model Execution Controller',94,294,452,49,23);store(s,'Job / Lease State (RAM)\njob·generation / shared-buffer handle',94,381,452,101,22);text(s,'기동·CPU 허가·load·release·cancel',96,496,448,29,22,{align:'center'});
 proc(s,'Vision Worker process  (R1 / R2)',875,205,945,356);comp(s,'Model Access (worker 배치)',894,247,907,293,26);mod(s,'IPC Endpoint / Job Queue',911,294,873,43,24);mod(s,'UI Adapter / SAM Adapter',911,365,374,48,23);mod(s,'Python / PyTorch CPU Runtime',1398,365,386,48,23);
 store(s,'Model Session (RAM)\nweights / model objects',1398,440,386,78,22);store(s,'SAM Episode (RAM)\nframe features / track tensors',911,440,374,78,22);
 line(s,[[1150,337],[1150,365]],{arrow:true});line(s,[[1285,388],[1398,388]],{arrow:true});label(s,'추론 호출',1291,351,100,20);line(s,[[1590,413],[1590,440]],{arrow:true});text(s,'load / 사용 / release',1607,413,175,25,18);line(s,[[1050,413],[1050,440]],{arrow:true});line(s,[[1230,440],[1230,413]],{dash:true,arrow:true});text(s,'읽기·갱신',1090,414,130,24,18,{align:'center'});
 line(s,[[562,317],[911,317]],{arrow:true});label(s,'1 EnsureModel / RunStage',586,275,283,22);line(s,[[911,328],[858,328],[858,393],[562,393]],{dash:true,arrow:true});label(s,'2 revision·generation 결과',586,351,260,21);text(s,'named pipe\n명령·완료·읽기 ACK\n\nframe 본문은 공유 buffer\nhandle로 참조',592,421,263,128,21,{align:'center'});store(s,'Model Files + Manifest (local disk)',1398,570,386,37,18);line(s,[[1530,570],[1530,518]],{arrow:true});label(s,'EnsureModel: local load',1550,542,250,18);
 // These three rows are state transitions, not pipeline components.
 const rows=[['수명 / 동시 RAM · MB','A','B-R1','B-R2','B-R3'],['시각 model session','없음','stage 후 회수·교대','사용 후 유지','worker별 유지'],['episode / 후보 기록','확정 이력','모델 회수와 별도 보존','같은 보존 정책','같은 보존 정책'],['추가 시각 runtime + model 객체','0','340 + 170','340 + 590','620 + 590'],['공통 / workspace·episode / 입력 buffer','560 / 0 / 32','560 / 860 / 24','560 / 860 / 24','560 / 860 / 32'],['QA06 동시 peak 합계','592','1,954','2,374','2,662']];
 const t=h.nativeTable(s,rows,60,619,[610,270,310,310,300],[53,63,63,59,63,64],25);h.highlightHeader(t,5);t.cells.block({row:5,column:0,rowCount:1,columnCount:5}).assign({fill:C.pale,textStyle:{typeface:FONT,fontSize:27,bold:true,color:C.blue}});
 // record the native table as equivalent editable cells for SVG/draw.io.
 let ty=619;const ws=[610,270,310,310,300],hs=[53,63,63,59,63,64];rows.forEach((row,r)=>{let tx=60;row.forEach((v,c)=>{scene.push({kind:'shape',geom:'rect',x:tx,y:ty,w:ws[c],h:hs[r],fill:r===0?C.green:r===5?C.pale:C.white,stroke:C.grid});scene.push({kind:'text',t:v,x:tx+8,y:ty+2,w:ws[c]-16,h:hs[r]-4,size:r===0?25:25,bold:r===0||r===5,color:r===5?C.blue:C.black,align:'center'});tx+=ws[c];});ty+=hs[r];});
 text(s,'R1: ReleaseModel → EnsureModel → RunStage  /  CloseEpisode는 별도 명령  /  cold 시작은 모든 B에 로딩 발생',65,1000,1780,30,23,{bold:true});text(s,'가정 RAM, 실측 아님. 입력 buffer는 A의 cloud 입력 / B의 IPC. R1→R2: UI 객체 420 MB 유지. R3: runtime private 280 MB + IPC 8 MB 추가.',65,1040,1750,24,19,{color:C.muted});
 note(s,'상단은R1/R2동일배치의확대도. ModelSession은실행단계가아닌Runtime소유RAM상태. adapter가runtime을호출한다. UI/SAMweights는diskmanifest로검증하고EnsureModel에서load한다. 디스크파일은이그림의RAM표에가산하지않는다. bundledPython+PyTorchCPUworker는VIA가spawn하고namedpipecommand/result와sharedbufferhandles를사용한다. 1명령에는job/request/input_revision/source_revision/capability/frame_refs/coordinate_transforms/seed_or_prompt/model_generation. 2결과는동일식별자와candidate_or_mask_refs/uncertainty/model_version/error. ReadACK는bufferlease회수일뿐candidate채택아님. ReleaseModel과CloseEpisode는다른명령. R1runtime유지,한모델session교대. R2usedsessions상주.R3두process/runtimes. 이확대도는생산기를생략한Model Access의분산배치를보인다. 요청선은host실행제어기에서workerIPC Endpoint로,완료선은같은host제어상태로돌아온다. frame본문을namedpipe에복제하지않으며공유bufferhandle과읽기ACK를사용한다. QA06수치기존가정동일:560공통,UI420,SAM170,runtime60shared+280private,workspace680+episode180,IPC24/32. SAMpeak이며UIworkspace동시합산안함.');scenes.push({name:'dp3-v7-runtime-lifetimes',items:scene});
}
const esc=t=>String(t).replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;');
function svgFor(items){let p=['<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080" viewBox="0 0 1920 1080"><defs><marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0L10,5L0,10Z" fill="context-stroke"/></marker></defs><rect width="1920" height="1080" fill="white"/>'];for(const o of items){const {x,y,w,h}=o;if(o.kind==='shape'){const common=`fill="${o.fill}" stroke="${o.stroke}" stroke-width="2" ${o.dash?'stroke-dasharray="9 6"':''}`;if(o.geom==='hexagon')p.push(`<polygon points="${x+20},${y} ${x+w-20},${y} ${x+w},${y+h/2} ${x+w-20},${y+h} ${x+20},${y+h} ${x},${y+h/2}" ${common}/>`);else if(o.geom==='chevron')p.push(`<polygon points="${x},${y} ${x+w-15},${y} ${x+w},${y+h/2} ${x+w-15},${y+h} ${x},${y+h} ${x+15},${y+h/2}" ${common}/>`);else p.push(`<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${o.geom==='roundRect'?7:0}" ${common}/>`);}else if(o.kind==='line')p.push(`<polyline points="${o.pts.map(a=>a.join(',')).join(' ')}" fill="none" stroke="${o.color}" stroke-width="${o.width}" ${o.dash?'stroke-dasharray="8 5"':''} ${o.arrow?'marker-end="url(#arrow)"':''}/>`);else if(o.kind==='store'){p.push(`<path d="M${x},${y+10} a${w/2},10 0 0 1 ${w},0 v${h-20} a${w/2},10 0 0 1 -${w},0 Z" fill="white" stroke="#111" stroke-width="2"/><ellipse cx="${x+w/2}" cy="${y+10}" rx="${w/2}" ry="10" fill="white" stroke="#111" stroke-width="2"/>`);p.push(svgText({...o,kind:'text',y:y+10,h:h-13,align:'center',color:C.black}));}else if(o.kind==='text')p.push(svgText(o));}return p.join('')+'</svg>';}
function svgText(o){const lines=o.t.split('\n'),lh=o.size*1.16,xx=o.align==='center'?o.x+o.w/2:o.align==='right'?o.x+o.w:o.x,yy=o.y+(o.h-lh*lines.length)/2+o.size*.90;return `<text fill="${o.color}" font-family="Calibri,Arial,sans-serif" font-size="${o.size}" font-weight="${o.bold?'700':'400'}" text-anchor="${o.align==='center'?'middle':o.align==='right'?'end':'start'}">${lines.map((v,i)=>`<tspan x="${xx}" y="${yy+i*lh}">${esc(v)}</tspan>`).join('')}</text>`;}
function drawioFor(items,name){const cells=['<mxCell id="0"/><mxCell id="1" parent="0"/>'];let id=2;for(const o of items){if(o.kind==='line'){cells.push(`<mxCell id="${id++}" edge="1" parent="1" style="endArrow=${o.arrow?'block':'none'};strokeColor=${o.color};strokeWidth=${o.width};dashed=${o.dash?1:0};rounded=0;"><mxGeometry relative="1" as="geometry"><mxPoint x="${o.pts[0][0]}" y="${o.pts[0][1]}" as="sourcePoint"/><mxPoint x="${o.pts.at(-1)[0]}" y="${o.pts.at(-1)[1]}" as="targetPoint"/><Array as="points">${o.pts.slice(1,-1).map(([x,y])=>`<mxPoint x="${x}" y="${y}"/>`).join('')}</Array></mxGeometry></mxCell>`);}else{const isText=o.kind==='text',style=isText?`text;html=0;strokeColor=none;fillColor=none;align=${o.align};verticalAlign=middle;fontFamily=Calibri;fontSize=${o.size};fontStyle=${o.bold?1:0};fontColor=${o.color};whiteSpace=nowrap;`:`shape=${o.kind==='store'?'cylinder':o.geom==='hexagon'?'hexagon':o.geom==='chevron'?'chevron':'rectangle'};rounded=${o.geom==='roundRect'?1:0};fillColor=${o.kind==='store'?'#FFFFFF':o.fill};strokeColor=${o.stroke||C.black};strokeWidth=2;dashed=${o.dash?1:0};fontFamily=Calibri;fontSize=${o.size||22};whiteSpace=wrap;`;cells.push(`<mxCell id="${id++}" value="${esc(o.t||'').replaceAll('\n','&#xa;')}" vertex="1" parent="1" style="${style}"><mxGeometry x="${o.x}" y="${o.y}" width="${o.w}" height="${o.h}" as="geometry"/></mxCell>`);}}return `<mxfile host="app.diagrams.net"><diagram name="${name}" id="${name}"><mxGraphModel dx="1920" dy="1080" grid="1" gridSize="10" page="1" pageWidth="1920" pageHeight="1080"><root>${cells.join('')}</root></mxGraphModel></diagram></mxfile>`;}
const diagrams=path.join(BUILD,'diagrams');await fs.mkdir(diagrams,{recursive:true});for(const sc of scenes){await fs.writeFile(path.join(diagrams,sc.name+'.svg'),svgFor(sc.items));await fs.writeFile(path.join(diagrams,sc.name+'.drawio'),drawioFor(sc.items,sc.name));}await fs.writeFile(path.join(diagrams,'scene-model.json'),JSON.stringify(scenes,null,2));
await(await PresentationFile.exportPptx(pres)).save(path.join(BUILD,'dp3-cpu-v7-authored.pptx'));
execFileSync(PY,[path.join(ROOT,'scripts/presentations/package_dp3_cpu_v7.py'),ROOT,BUILD],{stdio:'inherit'});
const {finalizePresentation}=await import(pathToFileURL(path.join(SKILL,'container_tools/artifact_tool_utils.mjs')));
const final=path.join(BUILD,'output','VIA-DP1-DP5-reference-style-29slides-v7-dp3-architecture-windows.pptx');
await finalizePresentation({workspaceDir:BUILD,candidatePath:path.join(BUILD,'candidate.pptx'),finalPath:final,explicitTotalSlideCount:29,pythonExecutable:PY,integrityValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_layout_geometry.py'),layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit',...[12,13,14,15].flatMap(n=>['--require-native-table-slide',String(n)])],requiredNativeTableOwnerSlides:[12,13,14,15],fontPolicy:{basis:'reference',families:['Calibri'],referencePath:src,referenceSha256:createHash('sha256').update(await fs.readFile(src)).digest('hex')},verifyArtifactToolImport:true,receiptPath:path.join(BUILD,'validation.json')});
const result=await PresentationFile.importPptx(await FileBlob.load(final));
for(let i=0;i<29;i++){const sl=result.slides.items[i];await fs.writeFile(path.join(BUILD,'previews',`slide-${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await(await result.export({slide:sl,format:'png',scale:1})).arrayBuffer()));if(i>=9&&i<=11)await fs.writeFile(path.join(BUILD,'previews',`slide-${i+1}.layout.json`),await(await sl.export({format:'layout'})).text());console.log('Rendered '+(i+1));}
console.log(final);
