// Compact implementation views for the reference-format presentation.
// Every numeric value is read from the committed QA forecast JSON.
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
const [repo,out,tieRule='score',sourceRef='HEAD']=process.argv.slice(2);
if(!repo||!out)throw Error('Usage: reference_dp_comparison_content.mjs REPO OUTPUT.json [score|raw] [SOURCE_REF]');
if(!['score','raw'].includes(tieRule))throw Error('tie rule must be score or raw');
const base='docs/architecture/12-decisions/decision-packages/';
const files={44:base+'04-44-continuous-interaction.md',47:base+'04-47-screen-voice-grounding.md',45:base+'04-45-memory-and-context.md',42:base+'04-42-lifecycle-ownership.md'};
const qaPath=base+'qa-candidate-package/thought-experiment/predictions.json';
function git(...args){return execFileSync('git',args,{cwd:repo,encoding:'utf8',maxBuffer:64*1024*1024}).trim();}
const snapshot=git('rev-parse',sourceRef);
const pred=JSON.parse(git('show',snapshot+':'+qaPath));
const sources={snapshot,qaVersion:pred.version,qaDocument:base+'qa-candidate-package/qa-dp-thought-experiment.md',qaCommit:git('log','-1','--format=%H',snapshot,'--',qaPath),documents:{}};
for(const [dp,file] of Object.entries(files))sources.documents[dp]={path:file,commit:git('log','-1','--format=%H',snapshot,'--',file)};
const cream='#FFF4D7',green='#C6E0B4',red='#C62525';
let g;
function node(id,x,y,w,h,text,opt={}){g.nodes.push({id,x,y,w,h,text,pt:12,...opt});}
function text(id,x,y,w,h,label,pt=12,opt={}){node(id,x,y,w,h,label,{geometry:'textbox',stroke:'none',fill:'none',pt,align:'left',...opt});}
function group(id,x,y,w,h,label,opt={}){node(id,x,y,w,h,label,{layer:'back',bold:true,pt:13,align:'left',valign:'top',insets:{left:8,right:2,top:4,bottom:0},...opt});}
function mod(id,x,y,w,h,label,diff=false,opt={}){node(id,x,y,w,h,label,{geometry:'roundRect',fill:diff?cream:'#FFFFFF',...opt});}
function store(id,x,y,w,h,label,opt={}){node(id,x,y,w,h,label,{geometry:'can',pt:11,...opt});}
function edge(id,points,opt={}){g.edges.push({id,points,arrow:true,...opt});}
function step(id,n,x,y,label,w=150,opt={}){
  node(id+'-num',x,y,19,20,String(n),{pt:11,bold:true,geometry:'roundRect',fill:green,stroke:'#72AD49'});
  text(id+'-label',x+22,y-1,w,23,label,11,{fill:'#FFFFFF',...opt});
}
function start(){g={nodes:[],edges:[]};}
function common47(x,s){
  text(s+'scene',x+10,207,875,25,'같은 입력: 두 그래프 지칭 후 “아래 것만, 처음 범위는 그대로”',13);
  group(s+'im',x+10,239,270,66,'Interaction Manager');
  mod(s+'capture',x+20,271,250,27,'R: Timeline & Buffer / Evidence Read Port',false,{pt:10});
  group(s+'rc',x+320,239,260,66,'Request Controller');
  mod(s+'rev',x+330,271,240,27,'Input Revision Control',false,{pt:11});
  group(s+'ma',x+620,239,260,66,'Model Access');
  mod(s+'api',x+630,271,240,27,'M: Semantic API Client',false,{pt:11});
  group(s+'ri',x+10,332,870,323,'Request Interpreter');
  mod(s+'port',x+22,365,260,36,'Grounding Session Port',false,{pt:11});
  store(s+'registry',x+314,366,255,35,'Grounding Attempt Registry\nRevisionGate / RAM', {pt:10});
  mod(s+'adopt',x+25,710,825,27,'Request Controller / Adoption Control → State Store',false,{pt:11});
  text(s+'ports',x+20,683,545,23,s==='A'?'공동 해석기가 R / M 사용. 유효 부분 재사용과 원본 확대 지원':'결합부는 게시 후보와 M 사용. 원본 재확장은 원 생산자가 수행',10);
  edge(s+'input',[[x+450,305],[x+450,330],[x+150,330],[x+150,365]]);step(s+'s1',1,x+22,309,'입력 revision 먼저 등록',240);
  text(s+'raw',x+326,308,550,22,'원본: 음성 / 화면 / 포인터 경로 / native 선택 범위',10);
  mod(s+'reader-port',x+24,660,425,20,'R: Grounding Evidence Reader → Interaction Manager',false,{pt:10});
  mod(s+'model-port',x+460,660,405,20,'M: Grounding Model Port → Model Access',false,{pt:10});
  step(s+'s6',6,x+585,685,'현재성 검사 후 채택',220);
}
function dp47(){start();for(const [s,x] of [['A',108],['B',1008]]){
  common47(x,s);
  if(s==='A'){
    mod(s+'control',x+28,430,235,49,'Grounding Attempt\nControl',true,{pt:12});
    store(s+'inbox',x+600,366,255,35,'Control / Completion Inbox',{pt:10});
    mod(s+'queue',x+300,433,225,41,'Joint Work Queue',true,{pt:12});
    mod(s+'worker',x+580,425,275,57,'Joint Grounding\nInterpreter',true,{pt:13});
    store(s+'state',x+580,542,275,56,'Joint Grounding State\nRAM: binding + 원본 dependency',{pt:11,fill:cream});
    text(s+'patch',x+35,599,500,39,'원본에서 대상과 binding 직접 생산\n유효 범위 재사용, 일부만 patch',12);
    edge(s+'port-inbox',[[x+282,383],[x+297,383],[x+297,412],[x+730,412],[x+730,401]]);
    edge(s+'inbox-control',[[x+600,383],[x+590,383],[x+590,414],[x+145,414],[x+145,430]]);
    edge(s+'job',[[x+263,454],[x+300,454]]);step(s+'s2',2,x+290,405,'JointJob',130);
    edge(s+'pop',[[x+525,454],[x+580,454]]);
    step(s+'s3',3,x+600,495,'R / M 원본 공동 해석',230);
    edge(s+'apply',[[x+718,482],[x+718,542]]);step(s+'s4',4,x+600,518,'조건부 patch',180);
    edge(s+'done',[[x+850,425],[x+865,425],[x+865,385],[x+855,385]],{dash:true});
    step(s+'s5',5,x+285,492,'완료 회수, 필요 시 원본 확대',270);
    edge(s+'return',[[x+600,397],[x+574,397],[x+574,522],[x+145,522],[x+145,479]],{dash:true});
    edge(s+'proposal',[[x+145,479],[x+145,573],[x+565,573],[x+565,706],[x+435,706],[x+435,710]]);
  } else {
    mod(s+'coord',x+28,432,230,46,'Grounding Coordinator',true,{pt:11});
    store(s+'inbox',x+600,366,255,35,'Control / Completion Inbox',{pt:10});
    group(s+'speech',x+292,422,250,141,'Speech Reference Producer',{fill:cream,pt:11});
    mod(s+'sq',x+307,458,220,29,'Speech Work Queue',true,{pt:11});
    mod(s+'sw',x+307,515,220,30,'Speech Worker',true,{pt:11});
    group(s+'screen',x+589,422,266,141,'Screen Target Producer',{fill:cream,pt:12});
    mod(s+'cq',x+604,458,236,29,'Screen Work Queue',true,{pt:11});
    mod(s+'cw',x+604,515,236,30,'Screen Worker',true,{pt:11});
    store(s+'speech-rev',x+292,594,250,42,'Speech Candidate Revisions\nRAM',{pt:10,fill:cream});
    store(s+'screen-rev',x+589,594,266,42,'Screen Candidate Revisions\nRAM',{pt:10,fill:cream});
    mod(s+'bq',x+28,517,230,31,'Binding Work Queue',true,{pt:11});
    mod(s+'binding',x+28,563,230,42,'Multimodal Binding\nResolver',true,{pt:11});
    store(s+'binding-state',x+28,617,230,25,'Binding Revision State / RAM',{pt:10,fill:cream});
    edge(s+'port-inbox',[[x+282,383],[x+297,383],[x+297,408],[x+730,408],[x+730,401]]);
    edge(s+'control',[[x+600,389],[x+581,389],[x+581,417],[x+150,417],[x+150,432]]);
    edge(s+'fork',[[x+258,454],[x+275,454],[x+275,412],[x+720,412],[x+720,458]]);edge(s+'fork2',[[x+417,412],[x+417,458]]);
    step(s+'s2',2,x+290,402,'두 생산 경로에 job 배정',250);
    edge(s+'sjob',[[x+417,487],[x+417,515]]);edge(s+'cjob',[[x+722,487],[x+722,515]]);
    text(s+'reads-speech',x+315,491,200,20,'R / M 원본 읽기와 해석',10);
    text(s+'reads-screen',x+612,491,225,20,'R / M 원본 읽기와 해석',10);
    edge(s+'publish1',[[x+417,545],[x+417,594]]);edge(s+'publish2',[[x+722,545],[x+722,594]]);
    step(s+'s3',3,x+294,568,'생산자가 자기 후보 게시',255);
    edge(s+'speech-done',[[x+527,530],[x+558,530],[x+558,408],[x+730,408],[x+730,401]],{dash:true});
    edge(s+'screen-done',[[x+840,530],[x+867,530],[x+867,383],[x+855,383]],{dash:true});
    edge(s+'snap',[[x+292,615],[x+274,615],[x+274,490],[x+245,490],[x+245,478]],{dash:true});
    edge(s+'read2',[[x+589,615],[x+570,615],[x+570,640],[x+274,640],[x+274,615]],{dash:true});
    step(s+'s4',4,x+27,484,'후보 snapshot 확인',215);
    edge(s+'binding-job',[[x+142,478],[x+142,517]]);
    edge(s+'bind',[[x+142,548],[x+142,563]]);
    edge(s+'binding-store',[[x+142,605],[x+142,617]]);
    edge(s+'rejudge',[[x+28,585],[x+18,585],[x+18,410],[x+730,410],[x+730,401]],{color:red,dash:true});
    step(s+'s5',5,x+589,568,'부족 후보는 원 생산자로',240,{color:red});
    edge(s+'proposal',[[x+143,642],[x+143,706],[x+435,706],[x+435,710]]);
  }
}return g;}

function dp45(){start();for(const [s,x] of [['A',108],['B',1008]]){
  text(s+'scene',x+10,207,875,25,'같은 요청: “지난번 설명한 기준에 제품 비교와 견적을 맞춰줘”',13);
  group(s+'ri',x+12,240,250,63,'Request Interpreter');mod(s+'tool',x+24,271,226,26,'Read Tool Runner',false,{pt:11});
  group(s+'rm',x+310,240,270,63,'Response Manager');mod(s+'delivery',x+322,271,246,26,'원문 / 실제 전달 read port',false,{pt:11});
  group(s+'tm',x+630,240,248,63,'Task Manager');mod(s+'task',x+642,271,224,26,'Task / 결과 read port',false,{pt:11});
  group(s+'cm',x+12,330,866,349,'Context Manager');
  text(s+'common',x+18,689,860,43,'공통: Model Access / Policy Manager / Request Controller의 입력 이력과 채택\n긴 읽기는 비동기 실행. 원본의 실제 전달 범위, 버전과 현재 사용권을 확인',11);
  if(s==='A'){
    group(s+'composer',x+29,364,832,190,'Context Composer',{fill:cream});
    mod(s+'planner',x+44,405,225,46,'Read Pump / Planner',true,{pt:12});
    mod(s+'broker',x+328,405,225,46,'Owner Read Broker',true,{pt:12});
    mod(s+'builder',x+610,405,235,46,'Bundle Builder / Use Guard',true,{pt:11});
    store(s+'evidence',x+328,503,225,39,'Request Evidence Set / RAM',{pt:10,fill:cream});
    store(s+'jobs',x+610,503,235,39,'Read Job Index / RAM',{pt:11,fill:cream});
    store(s+'cache',x+44,604,225,50,'Source View Cache\n검증한 발췌와 관계',{pt:11});
    mod(s+'invalid',x+328,604,225,50,'Invalidation Handler',true,{pt:11});
    store(s+'change',x+610,604,235,50,'Change Inbox',{pt:12});
    edge(s+'query',[[x+136,303],[x+136,405]]);step(s+'s1',1,x+30,307,'현재 Query',160);
    edge(s+'read',[[x+269,428],[x+328,428]]);step(s+'s2',2,x+287,374,'원본 읽기',150);
    edge(s+'read-rm',[[x+410,405],[x+410,303]]);edge(s+'rm-return',[[x+470,303],[x+470,405]],{dash:true});
    edge(s+'read-tm',[[x+520,405],[x+520,317],[x+730,317],[x+730,303]]);edge(s+'tm-return',[[x+800,303],[x+800,322],[x+550,322],[x+550,405]],{dash:true});
    edge(s+'read-state',[[x+440,451],[x+440,503]]);step(s+'s3',3,x+328,471,'완료를 read에 반영',210);
    edge(s+'bundle',[[x+553,522],[x+580,522],[x+580,428],[x+610,428]]);step(s+'s4',4,x+607,471,'범위와 버전 조합',215);
    edge(s+'job-state',[[x+730,451],[x+730,503]],{dash:true});
    edge(s+'result',[[x+845,428],[x+867,428],[x+867,350],[x+280,350],[x+280,285],[x+262,285]],{dash:true});
    step(s+'s5',5,x+575,307,'Bundle + 읽은 범위 반환',280);
    edge(s+'cache-read',[[x+130,451],[x+130,604]]);edge(s+'cache-return',[[x+184,604],[x+184,451]],{dash:true});
    text(s+'cachelabel',x+40,568,260,25,'유효 cache 재사용 / 부분 갱신',11);
    edge(s+'invalidate',[[x+610,628],[x+553,628]],{color:red,dash:true});edge(s+'invalidate-cache',[[x+328,628],[x+269,628]],{color:red,dash:true});
    edge(s+'invalidate-job',[[x+440,604],[x+440,579],[x+730,579],[x+730,542]],{color:red,dash:true});
    text(s+'state-note',x+595,562,250,26,'변경 / 철회 시 사용 차단',11,{color:red});
  } else {
    group(s+'publisher',x+28,365,833,154,'Memory Publisher',{fill:cream});
    mod(s+'admit',x+43,405,205,35,'Change Admission',true,{pt:11});
    mod(s+'read-worker',x+321,405,240,35,'Source Read Worker',true,{pt:11});
    mod(s+'derive',x+620,405,225,35,'Relation Builder',true,{pt:11});
    store(s+'journal',x+43,472,205,35,'Production Journal / 내구',{pt:10,fill:cream});
    store(s+'batch',x+321,472,240,35,'Candidate Batch / RAM',{pt:11,fill:cream});
    mod(s+'guard',x+620,472,225,35,'Publish Guard',true,{pt:11});
    store(s+'repo',x+321,548,524,39,'Episodic Memory Repository / 관계 + Coverage / 내구',{pt:11,fill:cream});
    store(s+'wait',x+43,548,205,39,'Read Wait Registry / RAM',{pt:10});
    group(s+'reader',x+28,609,833,57,'Evidence Reader',{pt:12,fill:cream});
    mod(s+'read-pump',x+43,634,246,25,'Read Pump / Bundle Builder',true,{pt:10});
    mod(s+'coverage',x+346,634,246,25,'Index / Coverage Resolver',true,{pt:10});
    mod(s+'verify',x+648,634,197,25,'Origin Verifier',true,{pt:11});
    edge(s+'changed',[[x+750,303],[x+750,317],[x+145,317],[x+145,405]]);step(s+'s1',1,x+305,307,'확정 변경 / 지원 범위 생산',270);
    edge(s+'claim',[[x+248,490],[x+276,490],[x+276,423],[x+321,423]]);edge(s+'durable-job',[[x+145,440],[x+145,472]]);
    edge(s+'read-rm',[[x+410,405],[x+410,303]]);edge(s+'read-tm',[[x+490,405],[x+490,352],[x+798,352],[x+798,303]]);
    edge(s+'source-return',[[x+470,303],[x+470,405]],{dash:true});
    edge(s+'produce',[[x+561,423],[x+620,423]]);step(s+'s2',2,x+588,377,'関係 생산'.replace('関係','관계'),170);
    edge(s+'batch-result',[[x+730,440],[x+730,456],[x+440,456],[x+440,472]],{dash:true});
    edge(s+'validate',[[x+561,490],[x+620,490]]);step(s+'s3',3,x+588,449,'검증',80);
    edge(s+'publish',[[x+730,507],[x+730,548]]);step(s+'s4',4,x+585,521,'관계와 Coverage 게시',230);
    edge(s+'query',[[x+135,303],[x+8,303],[x+8,646],[x+43,646]]);step(s+'s5',5,x+32,588,'현재 Query',160);
    edge(s+'lookup',[[x+289,646],[x+346,646]]);edge(s+'repo-read',[[x+470,634],[x+470,587]]);
    edge(s+'read-verify',[[x+592,646],[x+648,646]]);
    edge(s+'verify-source',[[x+845,634],[x+864,634],[x+864,307],[x+820,307],[x+820,303]]);
    edge(s+'result',[[x+845,646],[x+873,646],[x+873,320],[x+280,320],[x+280,283],[x+262,283]],{dash:true});
    step(s+'s6',6,x+605,588,'원본 확인 후 Bundle 반환',245);
    edge(s+'backfill',[[x+346,646],[x+308,646],[x+308,568],[x+248,568]],{color:red,dash:true});
    edge(s+'backfill-request',[[x+145,548],[x+145,530],[x+20,530],[x+20,421],[x+43,421]],{color:red,dash:true});
    text(s+'backfill-label',x+35,521,270,24,'미생산: 생산 대기 후 재조회',10,{color:red});
    text(s+'read-note',x+35,674,850,18,'미지원 관계는 재시도로 해결하지 않음. 원본 직접 조합 우회는 별도 혼합안',10);
  }
}return g;}

function dp42(){start();for(const [s,x] of [['A',108],['B',1008]]){
  const a=s==='A';
  text(s+'life',x+10,206,875,42,'C1 대화 / R1 위임 → Q1 질문 → P1 실제 제시 → R2 답변 / T1 업무와 X1 실행은 지속\nR3 정정 → X2 후속 실행. 같은 Q1 답변 명령 K1의 저장과 전송을 비교',11);
  group(s+'im',x+10,254,870,57,'Interaction Manager',{pt:12});
  mod(s+'input',x+22,280,200,25,'입력 수집기',false,{pt:11});mod(s+'stop',x+303,280,200,25,'로컬 출력 제어기',false,{pt:11});mod(s+'out',x+614,280,250,25,'출력 실행 / 실제 전달 확인',false,{pt:11});
  ['Model Access','Context Manager','Policy Manager'].forEach((v,i)=>node(s+'dep'+i,x+12+i*150,321,142,25,v,{pt:10}));
  node(s+'agent',x+534,321,342,25,'Downstream Agent',{pt:12});
  if(a)group(s+'process',x+8,355,874,381,'VIA Core process',{stroke:'#9C7136',pt:11});
  else {group(s+'dialogue-process',x+8,355,464,381,'대화 서비스 process',{stroke:'#9C7136',dash:true,pt:11});group(s+'work-process',x+490,355,392,381,'업무 서비스 process',{stroke:'#9C7136',dash:true,pt:11});}
  group(s+'ri',x+19,384,180,125,'Request Interpreter',{pt:11});mod(s+'interpret',x+29,417,160,30,'요청 해석기',false,{pt:11});store(s+'scratch',x+29,472,160,27,'해석 작업 / RAM',{pt:10});
  group(s+'rc',x+218,384,238,242,'Request Controller',{pt:12});
  const rcNames=['요청 채택기',a?'관계 변경 준비기':'명령 의도 기록기','현재 입력 제어기','진행 사실 반영기',a?'재개 조정기':'재연결 조정기'];
  rcNames.forEach((v,i)=>mod(s+'rc'+i,x+232,416+i*41,210,29,v,i===1||!a,{pt:11}));
  group(s+'rm',x+19,540,180,86,'Response Manager',{pt:11});mod(s+'publish',x+29,566,160,23,'게시 제어기',false,{pt:10});mod(s+'ledger',x+29,597,160,23,'실제 전달 기록기',false,{pt:10});
  group(s+'tm',x+510,384,354,142,'Task Manager',{pt:12});
  mod(s+'query',x+521,416,151,29,'업무 조회기',false,{pt:11});mod(s+'accept',x+701,416,151,29,'답변 접수기',!a,{pt:11});
  mod(s+'gate',x+521,459,331,28,'업무 접수 및 전송 게이트',!a,{pt:11});mod(s+'apply',x+521,497,331,23,'업무 사건 적용기',false,{pt:10});
  group(s+'ag',x+510,550,354,76,'Agent Gateway',{pt:12});
  mod(s+'send',x+521,577,156,24,'명령 전송 및 확인',false,{pt:10});mod(s+'ingress',x+700,577,152,24,'외부 사건 수신기',false,{pt:10});
  mod(s+'recovery',x+521,605,156,17,'전송 복구기 / 상태 조회',false,{pt:9});
  if(a){
    group(s+'ss',x+19,650,845,77,'State Store',{pt:12});
    mod(s+'tx',x+30,676,822,22,'트랜잭션 관리자',true,{pt:11});
    store(s+'dialogue-db',x+30,706,250,18,'요청 C1/R1~R3 / 실제 전달 P1',{pt:9});store(s+'task-db',x+310,706,250,18,'업무 T1 / 실행 X1,X2 / 질문 Q1',{pt:9});store(s+'commands-db',x+590,706,262,18,'명령 K1 / gate / 외부 inbox',{pt:9});
    edge(s+'commit',[[x+337,626],[x+337,676]]);step(s+'s4',4,x+380,632,'A1 관련 변경 공동 확정',260);
  }else{
    group(s+'dss',x+19,650,437,77,'State Store / 대화',{pt:12});group(s+'wss',x+510,650,354,77,'State Store / 업무',{pt:12});
    mod(s+'dtx',x+30,676,412,22,'트랜잭션 관리자',false,{pt:11});mod(s+'wtx',x+521,676,331,22,'트랜잭션 관리자',false,{pt:11});
    store(s+'dd',x+30,706,200,18,'C1 / R1~R3 / P1',{pt:9});store(s+'do',x+245,706,197,18,'K1 의도 outbox / cursor',{pt:9});
    store(s+'wd',x+521,706,158,18,'T1 / X1,X2 / Q1',{pt:9});store(s+'wo',x+695,706,157,18,'K1 / gate / inbox',{pt:9});
    edge(s+'dcommit',[[x+337,626],[x+337,676]]);edge(s+'wcommit',[[x+521,473],[x+500,473],[x+500,644],[x+685,644],[x+685,676]]);
    step(s+'s4',4,x+377,630,'W1 접수 → D2 반영 / 로컬 확정',450);
  }
  edge(s+'input-route',[[x+122,305],[x+122,316],[x+480,316],[x+480,373],[x+336,373],[x+336,416]]);step(s+'s1',1,x+280,357,'입력 / hold',150);
  edge(s+'sem',[[x+232,428],[x+189,428]]);edge(s+'sem-return',[[x+189,440],[x+232,440]],{dash:true});step(s+'s2',2,x+19,515,'의미 제안 / 채택',185);
  edge(s+'task-read',[[x+442,428],[x+521,428]]);edge(s+'task-return',[[x+521,440],[x+442,440]],{dash:true});
  edge(s+'prepare',[[x+442,471],[x+483,471],[x+483,451],[x+775,451],[x+775,445]]);step(s+'s3',3,x+30,630,a?'변경 준비 K1':'D1 의도 기록 후 K1 전달',290);
  edge(s+'receipt',[[x+701,437],[x+688,437],[x+688,534],[x+469,534],[x+469,552],[x+442,552]],{dash:true});
  edge(s+'claim',[[x+598,580],[x+598,487]]);step(s+'s5',5,x+528,526,'gate / CAS 후 전송',240);
  edge(s+'send-agent',[[x+677,589],[x+880,589],[x+880,334],[x+876,334]]);
  edge(s+'event',[[x+858,346],[x+887,346],[x+887,615],[x+850,615],[x+850,601]],{dash:true});
  edge(s+'unknown',[[x+600,601],[x+600,605]],{dash:true});
  edge(s+'fact',[[x+775,580],[x+775,520]],{dash:true});step(s+'s6',6,x+703,526,'외부 사실',150);
  edge(s+'publish-permit',[[x+232,552],[x+210,552],[x+210,578],[x+189,578]]);step(s+'s7',7,x+210,608,'게시 허용',115);
  edge(s+'display',[[x+29,578],[x+14,578],[x+14,313],[x+739,313],[x+739,305]]);
  edge(s+'actual',[[x+864,294],[x+893,294],[x+893,642],[x+109,642],[x+109,620]],{dash:true});step(s+'s8',8,x+707,611,'실제 전달 기록',170);
}return g;}

function dp44(){start();for(const [s,x] of [['A',108],['B',1008]]){
  const a=s==='A';
  text(s+'scene',x+10,207,875,25,'같은 사건: 응답 준비 중 사용자 정정 u2와 Agent 질문 Q1이 도착',13);
  group(s+'im',x+12,239,250,71,'Interaction Manager',{pt:12});mod(s+'local',x+24,269,226,33,'계속 입력 / 즉시 출력 stop',false,{pt:11});
  node(s+'agent',x+708,239,172,35,'Downstream Agent',{pt:11});node(s+'ag',x+494,239,173,35,'Agent Gateway',{pt:12});node(s+'tm',x+286,239,170,35,'Task Manager',{pt:12});
  edge(s+'agent-notice',[[x+708,256],[x+667,256]]);edge(s+'task-fact',[[x+494,256],[x+456,256]]);
  text(s+'facts',x+286,281,594,27,'업무 사실 확인과 저장 후 대화에 연결 (양안 공통)',11);
  group(s+'rc',x+12,348,396,297,'Request Controller',{pt:13});group(s+'rm',x+451,348,429,297,'Response Manager',{pt:13});
  group(s+'ri',x+12,679,396,52,'Request Interpreter',{pt:12});mod(s+'interpret',x+24,704,372,23,'비동기 의미 처리',false,{pt:11});
  text(s+'model',x+455,689,425,42,'공통 Model Access: 긴 처리는 비동기\n입력과 제어는 계속 수용, 취소 후 늦은 완료도 검사',11);
  if(a){
    mod(s+'control-q',x+28,390,164,39,'Control Lane',true,{pt:12});mod(s+'event-q',x+218,390,174,39,'Event Mailbox',true,{pt:12});
    mod(s+'dispatcher',x+90,472,245,47,'Dialogue Dispatcher',true,{pt:13});
    store(s+'progress',x+28,578,174,45,'Dialogue Progress\nState / RAM',{pt:11,fill:cream});store(s+'jobs',x+220,578,172,45,'Job Index / RAM',{pt:11,fill:cream});
    mod(s+'composer',x+473,390,384,43,'Response Composer',false,{pt:13});store(s+'candidate',x+473,488,180,48,'Candidate / RAM',{pt:12});mod(s+'publish',x+676,488,181,48,'Publication\nControl',false,{pt:12});
    store(s+'outbox',x+473,579,180,44,'Publication Outbox',{pt:11});store(s+'receipt',x+676,579,181,44,'ActualReceipt Ledger',{pt:10});
    edge(s+'input',[[x+135,310],[x+135,330],[x+305,330],[x+305,390]]);step(s+'s1',1,x+18,316,'입력 / 확인된 업무 사건',270);
    edge(s+'fact-input',[[x+372,274],[x+372,317],[x+305,317],[x+305,390]]);
    edge(s+'control',[[x+110,429],[x+110,449],[x+211,449],[x+211,472]]);edge(s+'event',[[x+305,429],[x+305,449],[x+211,449]]);
    edge(s+'job',[[x+211,519],[x+211,663],[x+211,679]]);step(s+'s2',2,x+28,651,'해석 job 시작',210);
    edge(s+'interpret-done',[[x+24,715],[x+5,715],[x+5,444],[x+392,444],[x+392,410]],{dash:true});
    edge(s+'prepare',[[x+335,495],[x+427,495],[x+427,411],[x+473,411]]);step(s+'s3',3,x+447,446,'준비 요청',150);
    edge(s+'prepare-done',[[x+857,411],[x+890,411],[x+890,326],[x+305,326],[x+305,390]],{dash:true});step(s+'s4',4,x+605,309,'완료는 Mailbox로 회수',250);
    edge(s+'candidate-write',[[x+563,433],[x+563,488]]);
    edge(s+'state1',[[x+157,519],[x+157,578]],{dash:true});edge(s+'state2',[[x+282,519],[x+282,578]],{dash:true});
    edge(s+'permit',[[x+335,507],[x+434,507],[x+434,551],[x+767,551],[x+767,536]]);step(s+'s5',5,x+474,550,'현재성 / 권한 검사 후 게시',270);
    edge(s+'to-pub',[[x+653,512],[x+676,512]]);
    edge(s+'to-outbox',[[x+767,536],[x+767,568],[x+563,568],[x+563,579]]);
    edge(s+'output',[[x+563,623],[x+563,638],[x+885,638],[x+885,310],[x+250,310],[x+250,302]]);
    edge(s+'receipt-flow',[[x+250,302],[x+250,342],[x+894,342],[x+894,600],[x+857,600]],{dash:true});step(s+'s6',6,x+475,650,'실제 전달 receipt → 진행 반영',295);
    edge(s+'receipt-back',[[x+767,623],[x+767,636],[x+416,636],[x+416,460],[x+305,460],[x+305,429]],{dash:true});
    edge(s+'hold',[[x+110,302],[x+110,390]],{color:red,dash:true});
  }else{
    group(s+'input-stage',x+28,391,173,225,'Input Resolution\nStage',{fill:cream,pt:11});group(s+'notice-stage',x+218,391,174,225,'Task Notice\nStage',{fill:cream,pt:11});
    [x+28,x+218].forEach((xx,i)=>{mod(s+'queue'+i,xx+7,449,159,37,'입력 / 완료 / 제어 큐',true,{pt:10});mod(s+'pump'+i,xx+7,523,159,33,'Pump / Done',true,{pt:11});store(s+'window'+i,xx+7,579,159,29,i?'Notice Window':'Input Window',{pt:10,fill:cream});});
    group(s+'composer',x+473,391,384,102,'Response Composer',{pt:13});mod(s+'composer-queue',x+485,432,175,43,'입력 / 완료 / 제어 큐',false,{pt:10});mod(s+'composer-pump',x+680,432,165,43,'Pump / Done',false,{pt:11});
    mod(s+'join',x+473,532,180,42,'Publication Join',true,{pt:11});mod(s+'pub',x+676,532,181,42,'Publication Control',false,{pt:11});
    store(s+'publication-window',x+473,600,180,30,'Publication Window',{pt:10,fill:cream});store(s+'actual',x+676,600,181,30,'실제 전달 원장',{pt:11});
    edge(s+'input',[[x+135,310],[x+115,310],[x+115,449]]);edge(s+'notice',[[x+372,274],[x+372,320],[x+304,320],[x+304,449]]);step(s+'s1',1,x+28,318,'입력 / 업무 알림을 각 Stage로',300);
    edge(s+'input-pump',[[x+115,486],[x+115,523]]);edge(s+'notice-pump',[[x+305,486],[x+305,523]]);
    edge(s+'interpret-job',[[x+115,556],[x+18,556],[x+18,663],[x+210,663],[x+210,679]]);step(s+'s2',2,x+28,651,'각 Stage가 작업과 완료 처리',285);
    edge(s+'meaning-return',[[x+24,715],[x+5,715],[x+5,469],[x+35,469]],{dash:true});
    edge(s+'adopted',[[x+194,540],[x+210,540],[x+210,504],[x+421,504],[x+421,454],[x+485,454]]);
    edge(s+'notice-output',[[x+385,540],[x+421,540],[x+421,454]]);step(s+'s3',3,x+447,499,'채택 의미 / Notice 직접 전달',310);
    edge(s+'compose-work',[[x+660,454],[x+680,454]]);edge(s+'candidate',[[x+762,475],[x+762,516],[x+563,516],[x+563,532]]);
    edge(s+'conditions',[[x+115,608],[x+115,636],[x+445,636],[x+445,552],[x+473,552]],{dash:true});step(s+'s4',4,x+210,619,'후보 + admission + 유효성',260);
    edge(s+'eligible',[[x+653,552],[x+676,552]]);step(s+'s5',5,x+670,505,'게시 검사',145);
    edge(s+'join-state',[[x+563,574],[x+563,600]],{dash:true});edge(s+'pub-actual',[[x+767,574],[x+767,600]],{dash:true});
    edge(s+'output',[[x+857,552],[x+886,552],[x+886,310],[x+250,310],[x+250,302]]);
    edge(s+'receipt-flow',[[x+250,302],[x+250,342],[x+894,342],[x+894,615],[x+857,615]],{dash:true});step(s+'s6',6,x+475,650,'실제 전달과 다음 조건 갱신',280);
    edge(s+'cancel',[[x+135,302],[x+270,302],[x+270,335],[x+405,335],[x+405,379],[x+650,379],[x+650,432]],{color:red,dash:true});
    text(s+'cancel-label',x+520,316,350,24,'정정 / 제어는 Stage별 큐로 전달',10,{color:red});
  }
}return g;}

const meta={
44:{title:'연속 대화와 업무 사건의 실행 연결',functionalTag:'기능 요구: 사용자 입력과 업무 사건의 연속 처리',options:['(방안 A) 중앙 비동기 조정','(방안 B) 단계별 반응형 실행 연결'],
 pros:[['▪ 실행과 완료를 한곳에서 연결해 정정 원인과 대기를 추적','▪ 긴 작업은 비동기 처리하며 입력과 즉시 중단을 계속 수용'],['▪ 입력과 업무 알림의 큐 및 대기 상태를 단계별로 관리','▪ 후속 소비자가 준비 조건에 따라 실행을 이어감 (조건부 이점)']],
 cons:[['▪ 생산자와 정정 조건이 늘면 중앙의 전이 및 배정 규칙도 증가'],['▪ 단계 간 계약, Publication Join과 각 Window를 함께 유지','▪ 늦은 완료와 제어 신호의 유효성을 각 연결에서 검사']]},
47:{title:'화면과 음성의 지칭 해석',functionalTag:'기능 요구: 화면 조작과 발화의 지칭 대상 연결',options:['(방안 A) 원본 공동 해석','(방안 B) 생산자별 후보 교환과 결합'],
 pros:[['▪ 원본의 교차 단서를 함께 판단해 순차 의존을 줄임 (QA01↑)','▪ 후보 생산 계약이 적어 변경 파급이 작음 (QA04↑)'],['▪ 생산자별 후보와 원본 근거를 보존해 부분 재판단 범위를 명시','▪ 후보 재사용과 오류 추적에 유리한 조건이 있음 (미측정)']],
 cons:[['▪ 복잡한 원본을 함께 읽어야 하며 공동 해석의 재판단 부담이 남음'],['▪ 후보 생산 후 결합하며 부족한 후보는 재생산 (QA01↓)','▪ 후보화 누락 위험과 생산자 / 결합부의 동반 변경 (QA02↓, QA04↓)']]},
45:{title:'과거 이력의 근거 공급',functionalTag:'기능 요구: 과거 이력을 현재 요청의 근거로 제공',options:['(방안 A) 요청별 원본 조회와 조합','(방안 B) 관계 사전 생산과 게시본 조회'],
 pros:[['▪ 미사용 관계의 선행 생산 비용을 피함 (QA05↑)','▪ 추가 게시 저장과 생산 버퍼가 적음 (QA04↑, QA06↑)'],['▪ 준비된 과거 관계를 읽어 첫 응답을 앞당김 (QA01↑)','▪ 같은 관계가 반복되면 생산 결과를 여러 요청에서 재사용']],
 cons:[['▪ 새로운 의미 관계는 첫 조회에서 원본과 함께 해석 (QA01↓)'],['▪ 사용하지 않는 관계도 생산 / 갱신 비용 발생 (QA05↓)','▪ 생산, 저장, 조회 계약과 추가 RAM을 유지 (QA04↓, QA06↓)']]},
42:{title:'대화와 업무의 상태 관리 경계',functionalTag:'기능 요구: 수명이 다른 대화와 업무의 연속성 유지',options:['(방안 A) 모듈화된 통합 Core','(방안 B) 독립 대화 서비스와 업무 서비스'],
 pros:[['▪ 관련 로컬 변경을 함께 확정해 계약 변경 지점 축소 (QA04↑)','▪ 별도 업무 프로세스와 저장소 cache가 없어 RAM 절감 (QA06↑)'],['▪ 업무의 상태와 전송을 대화 서비스의 실행 수명과 독립 관리','▪ 독립 운영 요구가 있는 경우의 이점이며 정상 사용 우위는 미확인']],
 cons:[['▪ 독립 배포와 운영이 필요하면 Core 전체의 조정 부담이 남음'],['▪ 접수 결과와 cursor로 두 상태를 다시 연결해야 함 (QA04↓)','▪ 별도 heap, 저장소 cache와 전달 버퍼가 필요 (QA06↓)']]}
};
const builders={44:dp44,47:dp47,45:dp45,42:dp42};
const metricNotes={
  47:{QA01:['원본 직접 해석','생산 후 결합'],QA02:['교차 단서 유지','후보 누락 위험'],QA04:['공동 해석부 중심','생산자와 결합부']},
  45:{QA01:['첫 조회에서 관계 해석','준비된 관계 조회'],QA04:['원본 조회 계약','생산 / 저장 / 조회'],QA05:['선행 생산 없음','사전 생산 포함'],QA06:['조회 cache','게시 관계와 생산 버퍼']},
  42:{QA04:['공동 확정 계약','서비스 간 계약'],QA06:['통합 Core','별도 heap과 buffer']}
};
const slides=[];
for(const dp of [44,47,45,42]){
 const qas=[],excluded=[];
 for(let i=1;i<=6;i++){
  const id='QA'+String(i).padStart(2,'0'),a=pred.summary.find(r=>r.dp===dp&&r.option==='A'&&r.qa===id),b=pred.summary.find(r=>r.dp===dp&&r.option==='B'&&r.qa===id);
  if(!a||!b)throw Error('Missing forecast '+dp+id);
  const tied=tieRule==='raw'?a.point_estimate.decimal===b.point_estimate.decimal:a.score_0_to_6===b.score_0_to_6;
  if(tied){excluded.push({id,A:a,B:b});continue;}
  const item={id,name:a.name,reason:a.structural_reason};
  for(const [opt,r] of [['A',a],['B',b]]){
   const v=r.point_estimate.decimal;
   let display=i===5?`$${v.toFixed(4)} / 공통 28개 묶음`:i===4?`${Number(v.toFixed(4))} 모듈 / 변경`:i===6?`${v} MB`:i===2?`${Number(v.toFixed(4))}%`:`${v.toFixed(4)} s`;
   item[opt]={score:r.score_0_to_6,value:v,display,annotation:metricNotes[dp]?.[id]?.[opt==='A'?0:1]??''};
  }
  qas.push(item);
 }
 const notes=[`DP-${dp} 구현 수준 비교를 레퍼런스의 5행 비교표에 맞춰 재배치한 발표 페이지.`,
  '아키텍처와 예상치의 원천 문서는 커밋된 버전이다. 세부 구현 전체를 한 장에 복제하지 않고 비교의 실행 주체, 큐, 상태와 핵심 왕복을 펼쳤다. 생략한 공통 API와 예외 계약은 원문 그대로 유지한다.',
  '번호는 이 발표 그림의 요약 처리 순서이며 원문의 상세 호출 번호를 재번호한 계약이 아니다. 원본과 모델의 접근 조건, 비동기 작업과 최종 채택의 책임은 변하지 않는다.',
  'QA 값은 QA-DP-THOUGHT-v2-20261015 작성자 사고 실험 예상치다. 실측, 최종 QA 승인 또는 A/B 선택 결과가 아니다. QA04는 실제 코드 변경 실험 전의 예정 Rust 모듈 영향 분석이다.',
  'QA01/03은 관련 사례의 산술평균이며 오답도 실제 시간을 포함한다. QA05는 COMMON-28 전체 총비용 단위다. 원값과 0~6점 구간을 바꾸지 않았다.',
  `표시한 QA: ${qas.map(q=>q.id).join(', ')||'없음'}. 동점 제외: ${excluded.map(q=>q.id).join(', ')}.`,
  ...qas.map(q=>q.id+': '+q.reason),
  '장점과 단점의 조건부 설명은 숫자로 확인한 이익과 구별한다. 특히 B44의 국소 실행, B47의 후보 관리, B42의 독립 운영은 해당 QA 점수 우세를 주장하지 않는다.',
  '출처 원문: '+files[dp],
 ].join('\n');
 slides.push({dp,...meta[dp],qas,excluded,diagram:builders[dp](),notes});
}
await fs.writeFile(out,JSON.stringify({version:'reference-comparison-v1',tieRule,sources,slides},null,2)+'\n');
console.log(JSON.stringify({snapshot,qaCommit:sources.qaCommit,selected:slides.map(s=>({dp:s.dp,qas:s.qas.map(q=>q.id)}))}));
