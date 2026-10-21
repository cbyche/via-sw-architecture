import {Scene,C} from './dp44_v9_scene.mjs';
const ROOT='docs/architecture/12-decisions/decision-packages/seven-problem-review/dp44-local-voice-execution-20261021';
const commonNotes=`DP44-DELIVERY-SYN-1.0. SYNTHETIC_AUTHORED_NOT_MEASURED. Source: ${ROOT}/synthetic-comparison-20261022/README.md; docs/presentations_files/dp44-synthetic-backdata-20261022/synthetic-data.json. Component names are comparison-local logical responsibilities, not renamings of the v8 reference. Request Interpretation Service (DP2) is the fixed interpretation/adoption boundary. Response Registry does not own business Task completion. Delivery History records presentation evidence, never user understanding or approval. Model integration hexagons labelled VIA are references to detailed common Components on page 26, not extra remote services. No separate OS process or replicated model is implied.\n`;
function page(name,n,title,sub){const d=new Scene('dp44-v9-'+name,n,title,sub);d.note(commonNotes);return d;}
function color(d,ids,c){for(const id of ids)d.nodes[id].stroke=c;}
function e(d,id,a,b,pts,t,lab,c=C.black,size=21.333333){d.edge(id,a,b,pts,{text:t,label:lab,color:c,size});}
function base(d){for(const n of Object.values(d.nodes))n.stroke??=C.black;return d;}
function evidence(d){d.text('가정 기반 합성 수치 / 실측 아님',1350,1009,510,28,21.333333,{align:'right',color:C.muted});}

function background(){const d=page('01-background',24,'DP44. 음성 준비와 재생 실행 방안 (배경)','이미 완성된 질문과 결과가 겹칠 때, 언제 음성을 준비하고 누가 재생을 제어할 것인가?');
 d.band('겹친 사건이 같은 대화로 들어온다',60,209,650);
 d.ext('user','사용자',65,323,250,100,28);
 d.ext('mail','Downstream Agent\n메일',65,493,250,100,25.333333);
 d.ext('report','Downstream Agent\n보고서',65,663,250,100,25.333333);
 d.comp('registry','Response Registry',805,355,510,365,28);
 d.mod('admit','Revision Admission',845,430,430,65,26.666667,'registry');
 d.store('pending','응답 원문과 판본\nTask 연결 / 내구 기록',845,550,430,115,25.333333,'registry');
 d.ext('dp2','Request Interpretation\nService (DP2)',805,210,510,95,26.666667);
 d.ext('voice','S2S Runtime\n음성 생성',1525,386,320,95,26.666667);
 d.ext('device','Audio Playout\n실제 장치 출력',1525,620,320,95,26.666667);
 e(d,'user-dp2','user','dp2',[[315,350],[650,350],[650,257],[805,257]],'1 “6월 기준으로 고쳐줘.\n메일 질문부터 읽어줘.”',[345,259,430,82]);
 e(d,'dp2-reg','dp2','admit',[[1060,305],[1060,325],[780,325],[780,445],[845,445]],'2 정정 범위와 순서 채택',[1075,315,395,38]);
 e(d,'q1','mail','admit',[[315,540],[720,540],[720,450],[845,450]],'Q1: “수신자는 김 대리인가요?”',[340,484,430,50]);
 e(d,'r1','report','admit',[[315,710],[740,710],[740,480],[845,480]],'R1-v1: 보고서 결과 텍스트',[340,650,430,50]);
 e(d,'register','admit','pending',[[1060,495],[1060,550]],'3 현재 판본 저장',[1080,503,270,35]);
 e(d,'prepare','pending','voice',[[1275,585],[1420,585],[1420,430],[1525,430]],'음성 준비 시점?', [1332,490,225,60]);
 e(d,'play','voice','device',[[1685,481],[1685,620]],'생성과 재생의\n수명 분리?', [1525,520,320,65]);
 d.text('배경: R0를 말하는 중에 Q1과 R1이 도착한다. 사용자는 R0를 끊고 보고서 정정과 읽기 순서를 요청한다.',60,795,1800,50,26.666667,{bold:true});
 d.table([
 ['공통으로 지키는 것','선택에 따라 달라지는 것'],
 ['질문과 결과의 업무 내용은 Downstream Agent가 작성\n화면 표시, 로컬 발화 감지와 DP2 의미 채택은 동일','A: 응답별 생성 실행과 보관을 재생과 분리\nB: 현재 발화 실행이 생성과 버퍼를 함께 관리'],
 ['구판 재생 차단, 유효 음성 재사용, 실제 출력 이력','A의 준비 적중 이익 ↔ 미전달 음성의 생성 비용']
 ],60,865,[900,900],[42,80,52],24);
 d.note('This is a broad design scene including R0 playback. S03-05 numerical timeline on page 28 does not include R0 playback. S05 is the authored interruption/resumption case. Downstream response text is already complete; no response-authoring LLM is added.');return base(d);}

function comparison(){const d=page('02-comparison',25,'DP44. 음성 준비와 재생 실행 방안 (설계 방안 비교)','생성의 소유 단위가 다르다: 응답별 준비 실행 A / 현재 발화 실행 B');
 for(const [p,x,c,title] of [['a',60,C.blue,'A. 응답별 준비 실행 + 별도 재생 제어'],['b',1030,C.b,'B. 현재 발화가 생성과 버퍼 소유']]){
 d.band(title,x,197,800,42);
 d.comp(p+'reg','Response Registry',x,264,800,120,25.333333);
 d.mod(p+'admit','Revision Admission',x+20,312,300,52,24,p+'reg');
 d.store(p+'text','응답 원문 / 판본\n내구 기록',x+460,305,320,65,22.666667,p+'reg');
 e(d,p+'write',p+'admit',p+'text',[[x+320,337],[x+460,337]],'1 등록',[x+322,305,135,29]);
 d.comp(p+'adapter','S2S Runtime Adapter',x,700,360,130,25.333333);
 d.mod(p+'client','Output Session Client',x+20,751,320,56,22.666667,p+'adapter');
 d.ext(p+'model','원격 S2S Runtime',x,931,360,70,25.333333);
 e(d,p+'modelreq',p+'client',p+'model',[[x+105,807],[x+105,931]],'text →',[x+7,843,95,35]);
 e(d,p+'modelret',p+'model',p+'client',[[x+265,931],[x+265,807]],'← audio',[x+270,843,90,35]);
 d.ext(p+'out','Audio Playout\n공통 VIA 출력 경계',x+440,931,360,70,24);
 }
 d.comp('aprep','Speech Preparation Coordinator',60,461,360,145,22.666667);
 d.mod('agen','Preparation Jobs',80,518,320,60,24,'aprep');
 d.comp('aplay','Speech Playback Coordinator',500,461,360,145,22.666667);
 d.mod('aselect','Playback Selection',520,518,320,60,24,'aplay');
 d.comp('astore','Prepared Audio Store',500,700,360,165,25.333333);
 d.mod('aaccess','Range\nAccess',520,756,140,70,22.666667,'astore');
 d.store('abuffer','준비 음성\nRAM',685,749,150,90,22.666667,'astore');
 e(d,'a-admit-gen','aadmit','agen',[[180,364],[180,425],[40,425],[40,548],[80,548]],'2 유효 응답마다',[75,407,250,36],C.blue);
 e(d,'a-admit-select','aadmit','aselect',[[380,337],[450,337],[450,420],[885,420],[885,533],[840,533]],'3 발화 허가 후 선택',[480,407,340,35]);
 e(d,'a-call','agen','aclient',[[150,578],[150,640],[40,640],[40,770],[80,770]],'4 생성 요청',[65,634,155,35],C.blue);
 e(d,'a-return','aclient','agen',[[400,790],[440,790],[440,590],[330,590],[330,578]],'5 chunk 반환',[300,612,180,34],C.blue);
 e(d,'a-save','agen','aaccess',[[400,548],[460,548],[460,790],[520,790]],'6 저장',[420,726,100,32],C.blue);
 e(d,'a-storewrite','aaccess','abuffer',[[660,776],[685,776]],'',null,C.blue);
 e(d,'a-storeread','abuffer','aaccess',[[685,806],[660,806]],'',null,C.blue);
 e(d,'a-read','aselect','aaccess',[[570,578],[570,650],[480,650],[480,775],[520,775]],'7 판본 확인\n구간 요청',[495,624,185,60],C.blue);
 e(d,'a-read-return','aaccess','aselect',[[630,826],[630,880],[890,880],[890,565],[840,565]],'',null,C.blue);
 e(d,'a-enqueue','aselect','aout',[[840,548],[910,548],[910,901],[710,901],[710,931]],'8 유효 audio 재생',[655,875,250,33],C.blue);
 d.comp('bturn','Speech Turn Controller',1030,461,800,180,25.333333);
 d.mod('bexec','Turn Execution',1050,518,300,60,25.333333,'bturn');
 d.store('bbuffer','Turn Audio Buffer\n유효 구간 / RAM',1490,510,310,95,24,'bturn');
 e(d,'b-admit-turn','badmit','bexec',[[1180,364],[1180,420],[1010,420],[1010,548],[1050,548]],'2 발화 허가 후 선택',[1040,407,280,36],C.b);
 e(d,'b-call','bexec','bclient',[[1120,578],[1120,675],[1010,675],[1010,770],[1050,770]],'3 선택된 항목 생성',[1033,652,260,35],C.b);
 e(d,'b-return','bclient','bexec',[[1370,790],[1410,790],[1410,660],[1300,660],[1300,578]],'4 chunk 반환',[1170,610,180,30],C.b);
 e(d,'b-write','bexec','bbuffer',[[1350,538],[1490,538]],'5 보관',[1360,494,120,32],C.b);
 e(d,'b-read','bbuffer','bexec',[[1490,568],[1350,568]],'6 재사용',[1357,589,125,32],C.b);
 e(d,'b-enqueue','bexec','bout',[[1350,553],[1425,553],[1425,890],[1680,890],[1680,931]],'7 유효 audio 재생',[1520,852,280,35],C.b);
 d.text('미선택 항목은 텍스트로 대기한다.\n복귀하면 유효 audio를 먼저 쓰고,\n미생성 후반만 추가 생성한다.',1490,690,340,135,25.333333);
 color(d,['aprep','agen','aplay','aselect','astore','aaccess','abuffer'],C.blue);color(d,['bturn','bexec','bbuffer'],C.b);
 d.text('검정 공통 / 파랑 A / 초록 B (A/B 배타적). 실선: 요청과 전달 / 점선: 결과와 복원. 입력과 실제 이력은 26쪽.',60,1012,1780,30,21.333333);
 d.note('Flow A 1 register, 2 launch response preparation, 3 select at eligibility, 4 text request, 5 audio return, 6 store, 7 read current revision, 8 enqueue. Flow B 1 register, 2 select, 3 request selected generation, 4 audio return, 5 buffer, 6 reuse, 7 enqueue. The two S2S symbols depict the same model configuration in exclusive alternatives, not simultaneous replicas. Device output receipts and input/echo path are expanded on page 26.');return base(d);}

function inputHistory(){const d=page('03-input-history',26,'DP44. 발화 감지, 중단과 실제 전달 이력','A/B 모두 로컬에서 즉시 재생을 끊고, 실제 제시 기록을 DP2의 입력 근거로 제공한다');
 d.comp('input','Voice Input Controller',60,220,480,200,26.666667);
 d.mod('finalizer','Input Finalizer',90,355,420,50,25.333333,'input');
 d.mod('capture','Audio Capture',90,280,200,60,25.333333,'input');d.mod('vad','Local VAD',320,280,190,60,25.333333,'input');
 d.comp('adapter','S2S Runtime Adapter',700,220,480,130,26.666667);d.mod('inputclient','Input Session Client',730,270,420,55,25.333333,'adapter');
 d.ext('model','원격 S2S Runtime\n입력 인식 세션',1370,240,440,85,26.666667);
 d.comp('ac','Speech Playback Coordinator',60,440,480,135,25.333333);d.mod('as','Playback Selection',90,495,420,55,25.333333,'ac');
 d.comp('bc','Speech Turn Controller',60,650,480,135,26.666667);d.mod('bs','Turn Execution',90,705,420,55,25.333333,'bc');
 d.comp('play','Audio Playout',700,440,480,345,26.666667);d.mod('writer','Device Writer',730,495,210,65,25.333333,'play');d.store('queue','장치 큐\nRAM',1000,495,150,85,24,'play');d.mod('receipt','Output Receipt',730,675,210,60,24,'play');
 d.ext('dp2','Request Interpretation\nService (DP2)',1370,440,440,85,26.666667);
 d.comp('history','Delivery History',1370,650,440,330,26.666667);d.mod('record','Presentation Recorder',1400,705,380,55,24,'history');d.store('facts','표시 / 출력 구간\n내구 기록',1400,820,180,100,22.666667,'history');d.mod('snapshot','Input\nSnapshot',1620,825,160,70,24,'history');
 d.comp('ui','UI Renderer',60,865,480,145,26.666667);d.mod('publish','Text Publisher',90,920,420,55,25.333333,'ui');
 d.comp('reg','Response Registry',700,865,480,145,26.666667);d.mod('apply','Revision Admission',730,920,420,55,25.333333,'reg');
 e(d,'samples','capture','vad',[[290,310],[320,310]],'',null);
 e(d,'inputstream','capture','inputclient',[[90,300],[40,300],[40,202],[680,202],[680,300],[730,300]],'1 입력 audio는 계속 전달',[320,175,415,32]);
 e(d,'vad-a','vad','as',[[510,290],[560,290],[560,610],[40,610],[40,520],[90,520]],'2 발화 시작 A',[60,590,280,34],C.blue);
 e(d,'vad-b','vad','bs',[[510,310],[580,310],[580,735],[510,735]],'2 발화\n시작 B',[425,590,135,55],C.b);
 e(d,'stop-a','as','writer',[[510,521],[730,521]],'3 즉시 중단',[529,478,190,34],C.blue);
 e(d,'stop-b','bs','writer',[[510,735],[650,735],[650,548],[730,548]],'3 동일 중단',[550,797,180,32],C.b);
 e(d,'enqueue','writer','queue',[[940,515],[1000,515]],'',null);
 e(d,'position','queue','writer',[[1000,548],[940,548]],'',null);
 e(d,'receiptrange','writer','receipt',[[835,560],[835,675]],'4 실제 출력 위치',[720,600,280,40]);
 e(d,'halt-ack-a','receipt','as',[[730,690],[620,690],[620,540],[510,540]],'',null,C.blue);
 e(d,'halt-ack-b','receipt','bs',[[730,720],[690,720],[690,778],[300,778],[300,760]],'',null,C.b);
 e(d,'recordaudio','receipt','record',[[940,705],[1400,705]],'5 재생 시도별 실제 구간',[965,663,380,36]);
 e(d,'sendinput','inputclient','model',[[1150,285],[1370,285]],'입력 audio',[1170,245,180,32]);
 e(d,'inputack','model','inputclient',[[1370,313],[1150,313]],'인식 결과 / 상태',[1163,329,200,32]);
 e(d,'transcript','inputclient','finalizer',[[730,315],[650,315],[650,365],[510,365]],'',null);
 e(d,'speech-end','vad','finalizer',[[415,340],[415,355]],'',null);
 e(d,'meaninginput','finalizer','dp2',[[510,387],[600,387],[600,420],[1320,420],[1320,480],[1370,480]],'6 입력 판본 확정',[830,368,350,36]);
 e(d,'query','dp2','snapshot',[[1810,460],[1870,460],[1870,842],[1780,842]],'',null);
 e(d,'snapshotreturn','snapshot','dp2',[[1780,873],[1840,873],[1840,492],[1810,492]],'',null);
 d.text('7 제시 이력 조회 / 반환',1450,599,360,35,22.666667);
 e(d,'factswrite','record','facts',[[1490,760],[1490,820]],'',null);
 e(d,'factsread','facts','snapshot',[[1580,866],[1620,866]],'',null);
 e(d,'adopt','dp2','apply',[[1590,525],[1590,595],[1260,595],[1260,940],[1150,940]],'8 정정 / 읽기 순서',[1230,535,340,36]);
 e(d,'display','apply','publish',[[730,942],[510,942]],'9 즉시 표시',[535,899,180,32]);
 e(d,'uireceipt','publish','record',[[300,975],[300,1025],[1320,1025],[1320,735],[1400,735]],'10 표시 확인',[590,990,225,32]);
 color(d,['ac','as'],C.blue);color(d,['bc','bs'],C.b);
 d.note('VAD confirms local speech onset; A/B controllers issue the same halt command. Actual device receipt, not generation callbacks, feeds Delivery History. No model tool calling is granted to S2S. Capture includes common echo cancellation using device output reference; omitted internal AEC wire is not an alternative. Queries include facts before and during the utterance with timestamps, not only onset snapshot. Two controller boxes are exclusive alternatives; only the selected one runs. Diagram intentionally does not route S2S generation completion into presentation history.');d.text('검정 공통 / 파랑 A / 초록 B (A/B 배타적). 실선: 요청과 전달 / 점선: 결과와 복원',60,1034,1650,25,20);return base(d);}

function lifetime(){const d=page('04-lifetimes',27,'DP44. 정정 반영, 생성 취소와 재개','같은 구판 차단을 수행하되, A는 응답별 job을 관리하고 B는 현재 turn의 생성 범위를 관리한다');
 d.band('A. 생성 실행과 재생 실행의 수명이 독립',60,200,800,42);d.band('B. 현재 발화에 생성 실행이 귀속',1030,200,800,42);
 d.comp('revision','Response Registry',500,435,360,105,25.333333);
 d.mod('revisionadmit','Revision Admission',520,480,320,45,22.666667,'revision');
 d.comp('prep','Speech Preparation Coordinator',60,280,800,140,25.333333);d.mod('jobs','Generation Lifecycle',80,335,300,60,24,'prep');d.store('state','응답별 job / 판본\nRAM',520,328,300,80,24,'prep');
 d.comp('store','Prepared Audio Store',60,555,360,180,25.333333);d.mod('access','Range\nAccess',80,610,140,65,22.666667,'store');d.store('buffer','유효 audio\nRAM',240,600,150,95,22.666667,'store');
 d.comp('play','Speech Playback Coordinator',500,555,360,140,22.666667);d.mod('select','Playback Selection',520,610,320,60,24,'play');
 d.comp('turn','Speech Turn Controller',1030,280,800,415,26.666667);d.mod('exec','Turn Execution',1050,335,310,60,25.333333,'turn');d.store('turnbuffer','Turn Audio Buffer\n구간 / 판본 / RAM',1490,325,300,100,24,'turn');d.store('pending','미선택 텍스트 참조\nRAM',1490,550,300,100,24,'turn');
 d.comp('adapter','S2S Runtime Adapter',600,855,520,155,26.666667);d.mod('client','Output Session Client',625,910,470,60,25.333333,'adapter');d.ext('model','원격 S2S Runtime',1290,900,350,85,26.666667);
 e(d,'revision-a','revisionadmit','jobs',[[520,500],[480,500],[480,365],[380,365]],'',null);
 e(d,'revision-b','revisionadmit','exec',[[840,500],[955,500],[955,350],[1050,350]],'채택 판본 /\n읽기 순서',[885,465,130,70]);
 e(d,'order-a','revisionadmit','select',[[680,525],[680,540],[900,540],[900,640],[840,640]],'',null);
 e(d,'ajob-state','jobs','state',[[380,355],[520,355]],'1 判본 반영'.replace('判','판'),[385,325,130,29],C.blue);
 e(d,'ajob-state-return','state','jobs',[[520,382],[380,382]],'상태 반환',[390,402,125,30],C.blue);
 e(d,'a-commit','jobs','access',[[150,395],[150,530],[40,530],[40,635],[80,635]],'2 판본 확인\n유효 chunk만 보관',[60,458,365,70],C.blue);
 e(d,'a-reject','access','jobs',[[80,660],[25,660],[25,410],[100,410],[100,395]],'구판 / 철회 tail: 거절 반환',[60,435,400,32],C.blue);
 e(d,'a-buffer-write','access','buffer',[[220,630],[240,630]],'',null,C.blue);
 e(d,'a-buffer-return','buffer','access',[[240,660],[220,660]],'',null,C.blue);
 e(d,'a-resume-read','select','access',[[570,670],[570,760],[180,760],[180,675]],'5 복귀 시 유효 구간 요청',[240,729,350,32],C.blue);
 e(d,'a-resume-return','access','select',[[200,675],[200,782],[470,782],[470,640],[520,640]],'',null,C.blue);
 e(d,'a-api','jobs','client',[[80,367],[40,367],[40,1025],[560,1025],[560,930],[625,930]],'',null,C.blue);
 e(d,'a-api-return','client','jobs',[[625,955],[580,955],[580,800],[450,800],[450,367],[380,367]],'',null,C.blue);
 d.text('3 구판 job 취소 / 4 늦은 chunk 반환\n유효 R0 job은 재생 중단 중에도 유지',60,808,530,60,22.666667,{color:C.blue});
 e(d,'b-reject','exec','exec',[[1050,375],[1015,375],[1015,465],[1150,465],[1150,395]],'',null,C.b);
 d.text('4 구판 chunk는 거절 후 종료',1040,480,380,32,21.333333);
 e(d,'b-write','exec','turnbuffer',[[1360,353],[1490,353]],'1 구간 보관',[1360,322,130,30],C.b);
 e(d,'b-reuse','turnbuffer','exec',[[1490,387],[1360,387]],'5 유효분 재사용',[1348,434,165,55],C.b);
 e(d,'b-text-read','pending','exec',[[1490,600],[1415,600],[1415,395],[1320,395]],'미선택은\n생성하지 않음',[1120,530,290,70],C.b);
 e(d,'b-api','exec','client',[[1050,365],[990,365],[990,810],[590,810],[590,940],[625,940]],'2 발화 전환 때 취소\n6 미생성 후반 요청',[865,707,310,75],C.b);
 e(d,'b-api-return','client','exec',[[1095,947],[1210,947],[1210,785],[1400,785],[1400,395],[1270,395]],'3 늦은 반환\n4 판본 확인',[1330,707,255,70],C.b);
 e(d,'api-model','client','model',[[1095,924],[1290,924]],'요청 / 취소',[1110,873,175,32]);
 e(d,'model-api','model','client',[[1290,960],[1095,960]],'audio / 상태',[1110,980,175,30]);
 color(d,['prep','jobs','state','store','access','buffer','play','select'],C.blue);color(d,['turn','exec','turnbuffer','pending'],C.b);
 d.note('Response Registry is the same instance repeated from page 26. Revision Admission explicitly sends the adopted revision to Generation Lifecycle and Turn Execution and the read order to Playback Selection. A revision key is (responseId, revision, range, generationId). Chunk admission rechecks revision and revocation tombstone. A superseded/revoked revision rejects late chunks. Cancellation for a speech switch does not revoke valid content: valid tail can remain buffered until generation terminal, as in S05. Cancel is best-effort and all tail usage remains billed. Playback Selection revalidates the current key before enqueue; Audio Playout rejects a stale epoch across the check/enqueue race. B Turn Execution rejects late invalid chunks before buffer admission and performs the same validation and keeps already generated valid audio on interruption. Under the basic B design current generation is retained through meaning adoption, cancelled on actual switch to Q1, and ungenerated suffix is generated at return, with additional input context. Short unconfirmed VAD may pause playback without cancelling. Diagram covers generation control; local halt/actual receipts on page 26.');d.text('검정 공통 / 파랑 A / 초록 B (A/B 배타적). 실선: 요청과 전달 / 점선: 결과와 복원',60,1034,1650,25,20);return base(d);}

function scenario(){const d=page('05-scenario',28,'DP44. 동일 시나리오의 처리와 비용','S03-05: Q1은 보존하고 R1-v1은 무효화한다. 정정 뒤 메일 질문을 먼저 읽는다');
 d.table([
 ['공통 사건 / 초','입력과 처리'],
 ['-4.400 ~ 0.000','“보고서는 6월 기준으로 다시 해줘. 메일 질문부터 들려줘.”'],
 ['Q1 -1.400 / R1-v1 -1.000','메일 질문과 구판 결과 텍스트 도착. DP2 의미 확정 0.867'],
 ['0.907 / 0.947','정정과 읽기 순서 반영 / “6월 기준 재요청, 메일 질문부터” 화면 표시'],
 ['3.427 / 3.467','새 보고서 R1-v2 도착 / 새 결과 화면 표시. 양안 동일']
 ],60,207,[390,1410],[44,57,57,57,57],24);
 d.band('같은 장면 시각: 생성 시작~종료와 실제 첫 음성 / 전체 생성 전에도 재생 가능',60,500,1800,42);
 d.table([
 ['음성 항목','A 생성 구간 / 실제 첫 음성','B 생성 구간 / 실제 첫 음성'],
 ['Q1 “수신자는 김 대리인가요?”','-1.400 ~ -0.351 / 0.947','0.907 ~ 1.956 / 1.348'],
 ['R1-v1 구판 보고서','-1.000 시작, 1.007 생성 중단\n6.587초 분량 폐기, 재생 없음','아직 미선택이므로 생성 없음'],
 ['R1-v2 새 보고서','3.427 ~ 5.784 / 3.876','4.142 ~ 6.499 / 4.591'],
 ['Q1 / 새 보고서 실제 재생 끝','3.741 / 11.870','4.142 / 12.585'],
 ['이 사례의 전체 VIA 모델 비용','$0.043073','$0.029720']
 ],60,563,[570,615,615],[44,62,82,62,55,55],24);
 d.text('첫 화면 차이 0초 / 첫 음성 차이 0.401초 / A의 구판 폐기 비용 $0.013353',60,944,1800,38,26.666667,{bold:true});
 evidence(d);
 d.note('S03-05 exact times in synthetic-data.json: interpretation .866666667; adoption .906666667; first UI .946666667; A Q1 start -1.4, full -.3505, onset .946666667, playback end3.740666667; B .906666667,full1.956166667,onset1.347666667,end4.141666667. Old A generation stopped1.006666667 after .1s tail, actual6.586666668s not requested7.994s. New report source3.426666667, first UI3.466666667. A full5.784166667, onset3.875666667, end11.869666667; B full6.499166667,onset4.590666667,end12.584666667. These are wall-clock scene timestamps, not QA03. QA03 original report A3.924166667/B4.245166667, Q1 read goal A.946666667/B1.956166667. S03 has no R0 playback; interruption is S05. S03 costs include common .000664+.002+.00078+.0044; Q1 .005714; v2 .016162; A waste .013353333.');return base(d);}

function qa(){const d=page('06-qa',29,'DP44. QA01~07 비교','같은 80개 평가 입력을 사용한다. 준비 80활동, 지칭 표현 120개와 원래 목표 90개를 구별한다');
 d.table([
 ['QA / 관측 경계','A','B','손익과 근거 상태'],
 ['QA01 첫 구체적 화면 표시','0.923초','0.923초','동점. UI는 음성과 독립'],
 ['QA02 지칭 대상 집합 정확도','120/120','120/120','고정 정답 조건. 모델 정확도 증거 아님'],
 ['QA03 VIA 처리시간','1.965초','2.710초','A 0.745초 감소. 원래 목표 90개의 인과 경로'],
 ['QA04 예상 변경 모듈 수','2.333개','2.000개','B 유리 가정. 실제 Rust diff 아님'],
 ['QA05 전체 VIA 모델 비용','$2.479873','$2.224660','B $0.255213 절감. A가 11.47% 높음'],
 ['QA06 최대 동시 로컬 RAM','561.659 MB','561.507 MB','0.152 MB 차이. 큰 선택 근거 아님'],
 ['QA07 요청 의미 해석 정확도','80/80','80/80','고정 정답 조건. 별도 우선순위 미합의'],
 ['첫 실제 음성 / 별도 종료점','1.069초','1.276초','A 0.207초 감소. QA01 화면값 대체 안 함']
 ],60,213,[560,275,275,690],[50,64,69,69,69,69,69,69,74],24);
 d.text('A의 폐기 준비 비용 $0.263333 − B의 추가 재개 문맥 비용 $0.008120 = 순차이 $0.255213',60,876,1800,40,25.333333,{bold:true});
 d.text('QA03은 외부 업무, 사용자 대기와 순수 재생 duration을 제외한 인과 계산이다. 실제 경과시간 단축과 다르다.\n8유형 × 10작성변형은 독립 실험이나 실사용 빈도 분포가 아니다. 수신 Agent 내부 비용과 원격 RAM은 제외한다.',60,931,1780,66,22.666667);
 evidence(d);d.note('Canonical UI endpoint retained. QA04 concept modules could tie2/2 under a common invalidation interface. QA05 includes setup80 activities and evaluated80 inputs, all used/discarded generation, cancellation tail and suffix resume context. Self-hosted unbilled deployment removes this USD differential. Local RAM excludes remote weights/KV. QA03 is a causally projected DAG, not elapsed-minus-overlap arithmetic; S08 may have50ms physical voice gain but larger projected QA03 gain. No cross-DP sum or new score bands.');return base(d);}

function choice(){const d=page('07-choice',30,'DP44. 선택과 보완','화면 첫 응답은 동점이다. 현 합성 조건에서는 QA03의 이익을 우선하여 A를 권고한다');
 d.table([
 ['선택 판단','A. 응답별 준비 실행','B. 현재 발화 실행'],
 ['얻는 이익','QA03 평균 0.745초 감소\n별도 첫 음성 평균 0.207초 감소','미선택 구판의 생성 비용 회피\n예상 변경 범위가 작음'],
 ['감수하는 비용','전체 VIA 모델 비용 11.47% 증가\n준비 job, 구간과 판본의 관리 부담','발화 선택 뒤 생성 대기\n긴 응답 재개 시 후반 문맥 재입력'],
 ['유리한 사용 조건','유효한 질문과 결과가 자주 대기\n정정 뒤에도 준비 내용이 유지','전달 전 철회와 대체가 잦음\n선행 준비할 시간 여유가 적음'],
 ['조건부 판단','A 권고: QA01 UI, QA02 동점 다음 QA03\nQA07을 임의 우선순위에 넣지 않음','B 보존: 비용 제약 또는 준비 적중 저하\n무과금 모델이면 QA05 차이 재검토']
 ],60,214,[360,720,720],[48,92,92,92,96],25.333333);
 d.rule([[420,214],[1140,214],[1140,634],[420,634],[420,214]],{color:C.blue,width:4});d.text('✓',1080,223,55,46,38.666667,{color:C.red,bold:true,align:'center'});
 d.band('보완 후보 A1: 다음 응답 하나만 준비 / 미적용, 미평가',60,673,1800,45);
 d.comp('limit','Speech Preparation Coordinator',60,750,900,175,26.666667);d.mod('admission','Next Response Admission',90,810,380,65,25.333333,'limit');d.mod('jobs','Preparation Jobs',570,810,350,65,25.333333,'limit');
 e(d,'onejob','admission','jobs',[[470,842],[570,842]],'최대 1개',[470,810,100,30],C.blue);
 d.text('버려질 준비를 줄이는 대신, 순서가 바뀌면 준비되지 않은 항목을 기다린다.\n기본 A의 값에 A1의 효과를 섞지 않는다. 같은 80개 입력으로 별도 평가한다.',1010,770,820,120,25.333333);
 d.text('반대 사례: S01/S06 동점 / S03/S04 폐기 / S05 유효분 재사용과 후반 재개 / S08 준비 여유 50ms',60,953,1800,40,22.666667);
 evidence(d);color(d,['limit','admission','jobs'],C.blue);
 d.note('Author recommendation only, not adopted ADR or measured superiority. QA01 UI tie then QA02 fixed tie permits QA03 consideration in existing priority order; separate voice endpoint is additional value. QA04 hypothetical and RAM difference small. A1 changes preparation admission only; the underlying response-owned generation/storage authority remains A. No A1 before/after numeric claim is supplied. Revisit when useful preparation rate, generation onset cost, cancellation usage, or runtime pause/resume changes.');return base(d);}
export const pages=()=>{const result=[background(),comparison(),inputHistory(),lifetime(),scenario(),qa(),choice()];const returns=new Set(['amodelret','bmodelret','a-return','b-return','a-storeread','a-read-return','b-read','inputack','transcript','position','receiptrange','halt-ack-a','halt-ack-b','recordaudio','snapshotreturn','factsread','uireceipt','ajob-state-return','a-buffer-return','a-resume-return','a-api-return','a-reject','b-reuse','b-api-return','model-api']);for(const d of result)for(const e of d.items.filter(e=>e.kind==='edge'))if(returns.has(e.id))e.dashed=true;return result;};
