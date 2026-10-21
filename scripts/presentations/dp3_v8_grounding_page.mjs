export function groundingPage(Scene){
 const d=new Scene('dp3-v8-ab-responsibilities',10,'DP3. 화면 지칭 해석 구조 (1/2)','공통 입력: 같은 전사와 단어 시각, 당시 화면과 조작 이력 / 화면 근거가 필요한 요청만 진입');
 d.band('A  원본 공동 해석',40,184,900,48); d.band('B-R1  후보 생산과 모델 교대 로딩',980,184,900,48);
 d.rule([[960,184],[960,1035]],{color:'#B7B9B9',width:1});
 for(const [p,x] of [['a',60],['b',1000]]){
  d.comp(p+'rc','Request Controller',x,255,420,120);
  d.mod(p+'guard','요청 검증 및 채택기',x+20,301,380,52,25,p+'rc');
  d.comp(p+'ctx','Context Manager',x+580,255,280,145,24);
  d.mod(p+'read','근거 조회기',x+598,301,244,43,24,p+'ctx');
  d.store(p+'raw','원본 참조 / 읽은 판본',x+598,354,244,35,19,p+'ctx');
  d.comp(p+'ri','Request Interpreter',x,535,420,157);
  d.mod(p+'interpret',p==='a'?'원본 해석기':'후보 결합기',x+20,582,205,48,25,p+'ri');
  d.store(p+'session','해석 상태\n(RAM)',x+250,580,150,90,22,p+'ri');
  d.edge(p+'read-allow',p+'guard',p+'read',[[x+400,313],[x+598,313]],{text:'1 조회 허가\n새 범위 재허가',label:[x+428,265,145,45],size:20});
  d.edge(p+'read-return',p+'read',p+'guard',[[x+598,337],[x+540,337],[x+540,388],[x+400,388],[x+400,353]],{text:'2 원본 또는\n조회 불가',label:[x+430,401,145,47],dashed:true,size:20});
  d.edge(p+'read-cache',p+'read',p+'raw',[[x+720,344],[x+720,354]],{});
  d.edge(p+'interpret-start',p+'guard',p+'interpret',[[x+70,353],[x+70,430],[x-20,430],[x-20,604],[x+20,604]],{text:p==='a'?'3 해석 / 보완 후 재개':'5 후보 전달 / 해석 재개',label:[x+7,452,250,31],size:21});
  d.edge(p+'interpret-need',p+'interpret',p+'guard',(p==='a'?[[x+190,630],[x+190,710],[x+450,710],[x+450,345],[x+400,345]]:[[x+190,630],[x+190,700],[x-10,700],[x-10,480],[x+140,480],[x+140,353]]),{text:p==='a'?'6 의미 제안\n또는 원본 부족':'8 의미 제안 또는\n후보 확장 요청',label:(p==='a'?[x+460,500,200,55]:[x+150,400,260,45]),dashed:true,size:20});
  d.edge(p+'state-write',p+'interpret',p+'session',[[x+225,613],[x+250,613]],{});

 }
 // A owns direct visual interpretation in its semantic session.
 d.comp('ama','Model Access',60,812,420,120);
 d.mod('acloud','클라우드 호출기',80,859,380,49,25,'ama');
 d.ext('allm','Cloud 의미 LLM',640,812,280,120,27);
 d.edge('a-infer','ainterpret','acloud',[[130,630],[130,750],[40,750],[40,884],[80,884]],{text:'4 전사 + 당시 원본',label:[72,715,250,28],size:23});
 d.edge('a-model-return','acloud','ainterpret',[[460,893],[520,893],[520,780],[240,780],[240,630]],{text:'5 해석 / 추가 원본 필요',label:[240,750,300,27],dashed:true,size:21});
 d.edge('a-cloud','acloud','allm',[[460,876],[640,876]],{text:'추론 요청',label:[490,837,135,30]});
 d.edge('a-cloud-return','allm','acloud',[[640,905],[460,905]],{text:'결과',label:[495,917,110,28],dashed:true});
 d.text('검정: 공통 / 파랑: A 고유 / 초록: B 고유\n제목 상자: Component / 둥근 상자: Module / 원통: 상태\n회색 점선: process / 실선: 요청 / 점선: 반환',65,965,850,76,20);
 // B's Controller authorizes original reads and candidate expansion separately.
 d.comp('bp','Screen Target Producer',1580,535,280,157,23);
 d.mod('bassemble','후보 구성기',1598,582,244,43,24,'bp');
 d.store('bcandidates','후보 기록과 판본 (RAM)',1598,639,244,41,20,'bp');
 d.edge('b-produce-allow','bguard','bassemble',[[1370,353],[1370,462],[1900,462],[1900,586],[1842,586]],{text:'3 최초 생산 / 확장 허가 + 원본',label:[1520,464,350,28],size:21});
 d.edge('b-produce-return','bassemble','bguard',[[1598,600],[1503,600],[1503,495],[1400,495],[1400,353]],{text:'4 후보 / 추가 원본 필요',label:[1460,509,390,24],dashed:true,size:20});
 d.edge('b-candidate-save','bassemble','bcandidates',[[1720,625],[1720,639]],{});
 d.comp('bma','Model Access (host 배치)',1000,810,420,135,24);
 d.mod('bcloud','클라우드 호출기',1018,861,195,52,23,'bma');
 d.mod('bcontrol','로컬 실행 제어기',1237,861,165,52,21,'bma');
 d.edge('b-infer','binterpret','bcloud',[[1020,615],[970,615],[970,890],[1018,890]],{text:'6 전사 + 후보 + 원본 crop',label:[1000,715,310,30],size:21});
 d.edge('b-model-return','bcloud','binterpret',[[1018,903],[985,903],[985,650],[1205,650],[1205,630]],{text:'7 해석 / 후보 부족',label:[1000,752,310,30],dashed:true,size:20});
 d.edge('b-visual-job','bassemble','bcontrol',[[1598,614],[1480,614],[1480,735],[1435,735],[1435,870],[1402,870]],{text:'시각 필요 시 UI / 정밀 경계 필요 시 SAM',label:[1435,700,435,29],size:21});
 d.edge('b-visual-result','bcontrol','bassemble',[[1402,900],[1450,900],[1450,780],[1880,780],[1880,603],[1842,603]],{text:'검출 / 경계 / 추적 결과',label:[1530,751,330,28],dashed:true,size:21});
 d.ext('bllm','Cloud 의미 LLM',1000,985,240,55,23);
 d.edge('b-cloud','bcloud','bllm',[[1060,913],[1060,985]],{text:'요청',label:[1000,950,80,27],size:20});
 d.edge('b-cloud-return','bllm','bcloud',[[1180,985],[1180,913]],{text:'결과',label:[1140,950,90,27],dashed:true,size:20});
 d.proc('bw','R1 Vision Worker process',1500,807,370,237);
 d.comp('bwma','Model Access (worker 배치)',1515,850,340,179,21,'bw');
 d.mod('bipc','IPC 수신기',1530,890,145,40,22,'bwma');
 d.mod('bexec','CPU 시각 실행기',1690,890,150,40,21,'bwma');
 d.store('bmodel','현재 모델\n(RAM)',1530,965,145,52,20,'bwma');
 d.store('bepisode','추적 상태\n(RAM)',1690,965,150,52,20,'bwma');
 d.edge('b-ipc-job','bcontrol','bipc',[[1270,913],[1270,972],[1485,972],[1485,900],[1530,900]],{text:'IPC 요청',label:[1295,942,160,27],size:19});
 d.edge('b-ipc-result','bipc','bcontrol',[[1530,918],[1467,918],[1467,1005],[1390,1005],[1390,913]],{text:'완료 / 오류',label:[1265,1010,215,27],dashed:true,size:19});
 d.edge('b-local-exec','bipc','bexec',[[1675,900],[1690,900]],{});
 d.edge('b-local-result','bexec','bipc',[[1690,920],[1675,920]],{dashed:true});
 d.edge('b-model-use','bexec','bmodel',[[1725,930],[1725,948],[1600,948],[1600,965]],{});
 d.edge('b-episode-use','bexec','bepisode',[[1800,930],[1800,965]],{});

 d.note('A의 최초 흐름은1조회허가,2원본반환,3해석,4모델요청,5모델결과,6의미제안또는원본부족이다. 원본부족이면6→1→2→3으로반복한다. B는1조회,2원본,3생산허가,4후보,5결합해석,6모델요청,7모델결과,8의미제안또는확장요청이다. 후보확장은8→3→4→5이고 새원본이필요하면8→1→2→3→4→5이다. 생산기자체가원본부족을반환하면4→1→2→3이다. 최초B가후보를만들기위해불필요한의미LLM호출을먼저수행하지않는다. 최종의미는Request Controller가현재입력과권한을검사하여채택한다.');
 d.note('CPU 시각 실행기는 11~12쪽 UI/SAM 어댑터와 CPU 추론 런타임의 축약 내부 Module 표기이다. 별도 Component나 새 process가 아니다. R1의 현재 모델과 추적 상태는 별개 RAM 상태이며 상세 load/release/close 경로는 12쪽이다.');
 return d;
}
