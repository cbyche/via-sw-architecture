// Native editable replacement regions for DP1, DP2 and DP4.
// Uses scripts/presentations/dp3_v8_scene.mjs without modifying repository files.
const BLACK='#111111',BLUE='#4676CB',GREEN='#32824A',GRAY='#98999B';
function comp(s,id,name,x,y,w,h,fs=21){const n=s.comp(id,name,x,y,w,h,fs);n.stroke=BLACK;return n;}
function mod(s,id,name,x,y,w,h,p,fs=19,color=BLACK){const n=s.mod(id,name,x,y,w,h,fs,p);n.stroke=color;return n;}
function store(s,id,name,x,y,w,h,p,fs=18,color=BLACK){const n=s.store(id,name,x,y,w,h,fs,p);n.stroke=color;return n;}
function ext(s,id,name,x,y,w,h,fs=18){const n=s.ext(id,name,x,y,w,h,fs);n.stroke=BLACK;return n;}
function edge(s,id,a,b,p,text,label,{back=false,color=BLACK,size=17}={}){s.edge(id,a,b,p,{color,dashed:back,...(text?{text,label,size}:{})});}
function caption(s,text,x,y,w,size=21,color=BLACK){s.text(text,x,y,w,27,size,{bold:true,align:'center',color});}
function commonEvidenceHead(s,k,x){
 comp(s,k+'rc','Request Controller',x+10,268,235,78);
 mod(s,k+'control','요청 검증 및 채택기',x+22,311,211,29,k+'rc');
 comp(s,k+'ri','Request Interpreter',x+310,268,245,78);
 mod(s,k+'interpret','요청 의미 해석기',x+323,311,219,29,k+'ri');
 edge(s,k+'interpret-call',k+'control',k+'interpret',[[x+233,324],[x+323,324]],'입력 +\n근거',[x+248,279,60,42],{size:16});
 edge(s,k+'proposal',k+'interpret',k+'control',[[x+433,340],[x+433,367],[x+127,367],[x+127,340]],'의미 제안 / 근거 보완 요구',[x+174,345,253,21],{back:true,size:17});
}
export function dp1Structure(Scene,selected=false){
 const s=new Scene('dp1-v10-'+(selected?'selection':'comparison'),selected?3:2,'DP1 구조','');
 for(const [alt,x] of [['A',196],['B',762],['C',1328]]){
  const k='d1'+alt; commonEvidenceHead(s,k,x);
  caption(s,alt==='A'?'대화 수명 근거 재사용':alt==='B'?'요청마다 근거 구성':'변경 시 생산 + 요청 시 조회',x+10,236,545,21);
  comp(s,k+'cm','Context Manager',x+10,417,545,124);
  if(alt!=='C'){
   mod(s,k+'query',alt==='A'?(selected?'A2 변경 조각 갱신기':'근거 조회 및 문맥 갱신기'):'원본 조회 및 묶음 구성기',x+25,462,220,34,k+'cm',18,alt==='A'?BLUE:GREEN);
   store(s,k+'view',alt==='A'?'유효 작업 문맥\nRAM / 대화 수명':'임시 근거 묶음\nRAM / 요청 수명',x+310,460,230,65,k+'cm',18,alt==='A'?BLUE:GREEN);
   ext(s,k+'source','공통 원본 owner (VIA 내부)',x+10,555,235,33,17);
   edge(s,k+'allow',k+'control',k+'query',[[x+62,340],[x+62,396],[x+115,396],[x+115,462]],alt==='A'?'초기 문맥 / 추가 범위 조회 허가':'시작 근거 / 추가 범위 조회 허가',[x+75,376,235,20]);
   edge(s,k+'bundle',k+'query',k+'control',[[x+25,477],[x+1,477],[x+1,353],[x+40,353],[x+40,340]],alt==='A'?'문맥 / 근거 / 불가':'근거 / 불가',[x+5,398,190,19],{back:true});
   edge(s,k+'read',k+'query',k+'source',[[x+80,496],[x+80,555]],'부족 시\n읽기',[x+12,502,64,34],{size:16});
   edge(s,k+'raw',k+'source',k+'query',[[x+190,555],[x+190,496]],'원본',[x+193,516,64,20],{back:true});
   edge(s,k+'write',k+'query',k+'view',[[x+245,474],[x+310,474]],'갱신',[x+248,452,59,20],{color:alt==='A'?BLUE:GREEN,size:16});
   edge(s,k+'reuse',k+'view',k+'query',[[x+310,494],[x+245,494]],'조회',[x+248,499,59,20],{back:true,color:alt==='A'?BLUE:GREEN,size:16});
   s.text(alt==='A'?(selected?'A2: 변경 부분 읽기\n전체 현재성 재검증 유지':'최초 해석 전 유효 문맥 공급\n채택 뒤 갱신 / 부족분 보완'):'최소 시작 근거 + 부족분 보완\n요청 종료 후 임시 묶음 해제',x+265,550,286,41,17,{align:'center',color:alt==='A'?BLUE:BLACK});
  }else{
   mod(s,k+'query','게시 근거 조회 및 보완기',x+25,462,228,30,k+'cm',17,GREEN);
   mod(s,k+'producer','관계 생산 및 게시기',x+310,462,230,30,k+'cm',18,GREEN);
   store(s,k+'published','게시 근거 (내구)',x+230,505,310,34,k+'cm',18,GREEN);
   ext(s,k+'source','공통 원본 owner (VIA 내부)',x+10,555,235,33,17);
   edge(s,k+'allow',k+'control',k+'query',[[x+62,340],[x+62,396],[x+115,396],[x+115,462]],alt==='A'?'초기 문맥 / 추가 범위 조회 허가':'시작 근거 / 추가 범위 조회 허가',[x+75,376,235,20]);
   edge(s,k+'bundle',k+'query',k+'control',[[x+25,477],[x+1,477],[x+1,353],[x+40,353],[x+40,340]],alt==='A'?'문맥 / 근거 / 불가':'근거 / 불가',[x+5,398,190,19],{back:true});
   edge(s,k+'read',k+'query',k+'source',[[x+80,492],[x+80,555]],'부족 시\n읽기',[x+12,499,64,34],{size:16});
   edge(s,k+'raw',k+'source',k+'query',[[x+190,555],[x+190,492]],'원본',[x+193,516,64,20],{back:true});
   edge(s,k+'published-read',k+'published',k+'query',[[x+230,519],[x+216,519],[x+216,492]],'조회',[x+249,490,51,18],{back:true,color:GREEN,size:16});
   edge(s,k+'publish',k+'producer',k+'published',[[x+420,492],[x+420,505]],'게시',[x+430,490,54,18],{color:GREEN,size:16});
   edge(s,k+'background',k+'source',k+'producer',[[x+245,572],[x+562,572],[x+562,477],[x+540,477]],'확정 원본 변경 → 별도 생산',[x+268,548,283,21],{color:GREEN,size:17});
  }
 }
 s.note('구조 칸만 교체. 모든 대안의 Request Controller가 읽기 허가, 새 범위 재허가 및 현재 revision의 의미 채택을 수행한다. A는 첫 해석 전에 RC→CM 초기 문맥 조회→RC→RI의 경로로 유효 작업 문맥을 공급한다. 부족/변경 시에는 RI→RC→CM→원본→CM→RC→RI로 보완한다. 이 구조도는 실행 순서를 강제하는 숫자를 사용하지 않는다. 원본 owner는 VIA 내부의 기존 책임자이며 Context Manager로 권위가 이전되지 않는다. A의 문맥 갱신은 의미 채택 후, B의 임시 묶음은 요청 수명, C의 게시 근거는 별도 생산 수명이다. 공통 Model Access 모델 연동 구간은 이 근거 공급 확대도 밖에 있다. C의 관계 생산기는 자유문 해석이 필요할 때만 공통 Model Access를 사용한다. 색: 검정 공통, 파랑 A 고유, 초록 B/C 교체 구조. A2는 변경 조각만 읽되 전체 현재성 검증은 유지한다.');
 return s;
}
export function dp2Structure(Scene,selected=false){
 const s=new Scene('dp2-v10-'+(selected?'selection':'comparison'),selected?7:6,'DP2 구조','');
 for(const [alt,x] of [['A',196],['B',1046]]){
  const k='d2'+alt;
  caption(s,alt==='A'?'대화 해석 실행 1개 / 현재 step 경계에서 후속 입력 수용':'공통 대화 해석 + 요청별 논리 실행 / 개별 반환',x+10,230,830,21);
  comp(s,k+'rc','Request Controller',x+10,270,240,279);
  mod(s,k+'input','입력 및 실행 요청기',x+22,315,216,34,k+'rc',19);
  store(s,k+'control-state',alt==='A'?'입력 / 채택 목표 / 판본\nRAM':'입력과 실행 ID 대응\n목표 / 판본 / 변경 상태\nRAM',x+22,373,216,75,k+'rc',18);
  mod(s,k+'adopt','반환 검증 및 채택기',x+22,474,216,46,k+'rc',18);
  edge(s,k+'input-save',k+'input',k+'control-state',[[x+125,349],[x+125,373]],null,null);
  edge(s,k+'state-check',k+'control-state',k+'adopt',[[x+125,448],[x+125,474]],'판본 확인',[x+138,449,92,21],{back:true,size:17});
  comp(s,k+'ri','Request Interpreter',x+300,270,530,279);
  comp(s,k+'ma','Model Access',x+300,558,320,81);
  mod(s,k+'call','모델 호출 및 한도 관리기',x+315,602,290,29,k+'ma',18);
  ext(s,k+'cloud','공통 cloud\n모델',x+710,588,120,45,16);
  edge(s,k+'cloud-call',k+'call',k+'cloud',[[x+605,609],[x+710,609]],'호출',[x+663,582,45,22],{size:16});
  edge(s,k+'cloud-result',k+'cloud',k+'call',[[x+710,624],[x+605,624]],'반환',[x+663,624,45,22],{back:true,size:16});
  if(alt==='A'){
   mod(s,k+'conversation','대화 해석 실행기',x+315,315,240,57,k+'ri',21,BLUE);
   store(s,k+'context','대화 문맥 / 근거\nR1, R2 잠정 판단\nRAM / 대화 수명',x+590,315,225,101,k+'ri',19,BLUE);
   edge(s,k+'start',k+'input',k+'conversation',[[x+238,330],[x+315,330]],'step\n경계 후',[x+252,282,46,42],{size:15,color:BLUE});
   edge(s,k+'proposal',k+'conversation',k+'adopt',[[x+315,356],[x+271,356],[x+271,489],[x+238,489]],'의미 제안',[x+251,395,92,22],{back:true,size:17});
   edge(s,k+'resume',k+'adopt',k+'conversation',[[x+238,507],[x+285,507],[x+285,368],[x+315,368]],'재판단 요청',[x+80,525,150,21],{size:16});
   edge(s,k+'context-read',k+'context',k+'conversation',[[x+590,338],[x+555,338]],null,null,{back:true,color:BLUE});
   edge(s,k+'context-save',k+'conversation',k+'context',[[x+555,359],[x+590,359]],null,null,{color:BLUE});
   edge(s,k+'infer',k+'conversation',k+'call',[[x+435,372],[x+435,461],[x+310,461],[x+310,616],[x+315,616]],'모델 판단',[x+327,435,111,23],{color:BLUE,size:17});
   edge(s,k+'infer-result',k+'call',k+'conversation',[[x+605,624],[x+640,624],[x+640,554],[x+840,554],[x+840,440],[x+573,440],[x+573,382],[x+542,382],[x+542,372]],'판단 결과',[x+662,530,145,21],{back:true,color:BLUE,size:17});
   s.text('남은 목표와 새 입력을 같은 실행에서 계속\n앞선 요청 전체 완료까지 기다리는 구조는 아님',x+335,479,475,48,19,{align:'center'});
  }else{
   mod(s,k+'conversation','공통 대화 해석기',x+315,315,240,45,k+'ri',20);
   store(s,k+'recent','최근 문맥 / 요청 참조\nRAM',x+590,315,225,58,k+'ri',18);
   mod(s,k+'r1','요청 해석 실행기 (R1)',x+315,410,240,36,k+'ri',18,GREEN);
   mod(s,k+'r2','요청 해석 실행기 (R2)',x+590,410,225,36,k+'ri',18,GREEN);
   store(s,k+'r1-state','R1 가변 상태 (RAM)',x+315,456,240,30,k+'ri',17,GREEN);
   store(s,k+'r2-state','R2 가변 상태 (RAM)',x+590,456,225,30,k+'ri',17,GREEN);
   if(selected){
    store(s,k+'shared','B1 공유 불변 입력\n원본 (RAM)',x+315,489,240,49,k+'ri',17,BLUE);
    edge(s,k+'shared-r1',k+'shared',k+'r1',[[x+315,516],[x+304,516],[x+304,440],[x+315,440]],null,null,{back:true,color:BLUE});
    edge(s,k+'shared-r2',k+'shared',k+'r2',[[x+555,516],[x+578,516],[x+578,440],[x+590,440]],null,null,{back:true,color:BLUE});
   }else{
    store(s,k+'r1-input','불변 입력 사본 1 (RAM)',x+315,501,240,29,k+'ri',16,GREEN);
    store(s,k+'r2-input','불변 입력 사본 2 (RAM)',x+590,501,225,29,k+'ri',16,GREEN);
    edge(s,k+'input-r1',k+'r1-input',k+'r1',[[x+315,516],[x+304,516],[x+304,440],[x+315,440]],null,null,{back:true,color:GREEN});
    edge(s,k+'input-r2',k+'r2-input',k+'r2',[[x+815,516],[x+824,516],[x+824,440],[x+815,440]],null,null,{back:true,color:GREEN});
   }
   edge(s,k+'start',k+'input',k+'conversation',[[x+238,330],[x+315,330]],'새\n입력',[x+252,282,46,42],{size:16});
   edge(s,k+'short',k+'conversation',k+'adopt',[[x+315,352],[x+265,352],[x+265,481],[x+238,481]],'짧은 답',[x+245,370,103,21],{back:true,size:16});
   edge(s,k+'resume',k+'adopt',k+'conversation',[[x+238,513],[x+278,513],[x+278,360],[x+315,360]],'재결합 요청',[x+115,528,125,21],{size:16});
   edge(s,k+'recent-read',k+'recent',k+'conversation',[[x+590,337],[x+555,337]],null,null,{back:true});
   edge(s,k+'r1-start',k+'conversation',k+'r1',[[x+435,360],[x+435,410]],'긴 판단',[x+317,381,115,22],{color:GREEN,size:17});
   edge(s,k+'r2-start',k+'conversation',k+'r2',[[x+505,360],[x+505,389],[x+701,389],[x+701,410]],'독립 실행',[x+597,366,130,22],{color:GREEN,size:17});
   edge(s,k+'r1-state-write',k+'r1',k+'r1-state',[[x+435,446],[x+435,456]],null,null,{color:GREEN});
   edge(s,k+'r2-state-write',k+'r2',k+'r2-state',[[x+702,446],[x+702,456]],null,null,{color:GREEN});
   edge(s,k+'r1-return',k+'r1',k+'adopt',[[x+315,426],[x+254,426],[x+254,495],[x+238,495]],'반환',[x+264,465,45,20],{back:true,color:GREEN,size:15});
   edge(s,k+'r2-return',k+'r2',k+'adopt',[[x+815,426],[x+840,426],[x+840,552],[x+256,552],[x+256,507],[x+238,507]],'개별 반환',[x+670,533,132,20],{back:true,color:GREEN,size:16});
   edge(s,k+'common-infer',k+'conversation',k+'call',[[x+555,346],[x+572,346],[x+572,541],[x+640,541],[x+640,609],[x+605,609]],null,null);
   edge(s,k+'r1-infer',k+'r1',k+'call',[[x+315,417],[x+292,417],[x+292,614],[x+315,614]],null,null,{color:GREEN});
   edge(s,k+'r2-infer',k+'r2',k+'call',[[x+590,432],[x+565,432],[x+565,546],[x+629,546],[x+629,616],[x+605,616]],null,null,{color:GREEN});
   // Returns fan out from the same software gate; no weight/runtime replica.
   edge(s,k+'common-result',k+'call',k+'conversation',[[x+605,625],[x+649,625],[x+649,538],[x+580,538],[x+580,357],[x+555,357]],null,null,{back:true});
   edge(s,k+'r1-result',k+'call',k+'r1',[[x+315,624],[x+286,624],[x+286,435],[x+315,435]],null,null,{back:true,color:GREEN});
   edge(s,k+'r2-result',k+'call',k+'r2',[[x+605,630],[x+636,630],[x+636,550],[x+584,550],[x+584,443],[x+590,443]],null,null,{back:true,color:GREEN});
  }
 }
 s.note('같은 Model Access 호출 한도 K와 cloud 모델을 두 대안 각각에 동일하게 표시한다. 모델 가중치를 논리 실행별로 복제하지 않는다. 공통 대화와 요청 실행기의 모델 호출 및 반환은 각각 연결되어 있으며 모든 완료를 기다리는 join은 없다. A는 현재 모델/읽기 step 경계에서 새 입력을 반영하고 앞선 목표 전체 종료를 기다리지 않는다. 공통 Context Manager 근거 정책, 실제 표시와 외부 Agent 업무는 이 의미 실행 확대도 밖에 있다. R1/R2는 논리 실행 인스턴스이며 OS process가 아니다. B1은 불변 자료만 공유하고 R1/R2 가변 상태를 보존한다. 색: 검정 공통, 파랑 A 고유, 초록 B 고유, B1 파랑은 보완 강조.');
 return s;
}
export function dp4Structure(Scene,selected=false){
 const s=new Scene('dp4-v10-'+(selected?'selection':'comparison'),selected?18:17,'DP4 구조','');
 for(const [alt,x] of [['A',192],['B',760],['C',1328]]){
  const k='d4'+alt;
  ext(s,k+'source','공통 원본 owner (VIA 내부)',x+15,227,535,29,18);
  comp(s,k+'cm','Context Manager',x+15,266,535,162);
  const composer=alt==='A'?'인계 묶음 구성기':alt==='B'?'업무 기록 생산기':'필수 명세 구성기';
  mod(s,k+'compose',composer,x+28,310,248,34,k+'cm',19,alt==='A'?BLUE:GREEN);
  if(alt==='A'){
   store(s,k+'capsule','인계 묶음\nRAM / 위임 수명',x+310,310,225,59,k+'cm',18,BLUE);
   edge(s,k+'capsule-save',k+'compose',k+'capsule',[[x+276,325],[x+310,325]],null,null,{color:BLUE});
   edge(s,k+'capsule-read',k+'capsule',k+'compose',[[x+310,340],[x+276,340]],null,null,{back:true,color:BLUE});
  }else if(alt==='B'){
   store(s,k+'record','업무 기록 / 의존 판본\n내구 기록',x+310,310,225,59,k+'cm',17,GREEN);
   mod(s,k+'extract','목적별 추출 및 원본 보완기',x+28,382,507,34,k+'cm',19,GREEN);
   edge(s,k+'record-publish',k+'compose',k+'record',[[x+276,325],[x+310,325]],null,null,{color:GREEN});
   edge(s,k+'record-read',k+'record',k+'extract',[[x+425,369],[x+425,382]],null,null,{back:true,color:GREEN});
   edge(s,k+'extract-raw-read',k+'extract',k+'source',[[x+535,393],[x+548,393],[x+548,256]],'부족분 읽기',[x+448,272,94,21],{color:GREEN,size:16});
   edge(s,k+'extract-raw-result',k+'source',k+'extract',[[x+542,256],[x+542,407],[x+535,407]],null,null,{back:true,color:GREEN});
  }else{
   mod(s,k+'reader','자료 읽기 제공기',x+310,310,225,34,k+'cm',19,GREEN);
   store(s,k+'manifest','필수 명세 / 자료 목록\nRAM / 위임 수명',x+28,370,248,50,k+'cm',17,GREEN);
   store(s,k+'lease','접근 원장 / 읽기 영수증\n접근 수명',x+310,370,225,50,k+'cm',17,GREEN);
   edge(s,k+'manifest-save',k+'compose',k+'manifest',[[x+145,344],[x+145,370]],selected?'C1 조건 원문 첨부':'필수 조건',[x+28,347,243,22],{color:selected?BLUE:BLACK,size:17});
   edge(s,k+'lease-save',k+'reader',k+'lease',[[x+425,344],[x+425,370]],'권한 / 판본 검사',[x+310,347,225,22],{color:GREEN,size:17});
   edge(s,k+'source-read',k+'reader',k+'source',[[x+468,310],[x+468,256]],'읽기',[x+471,273,52,20],{color:GREEN,size:16});
   edge(s,k+'source-return',k+'source',k+'reader',[[x+524,256],[x+524,310]],null,null,{back:true,color:GREEN});
  }
  // Existing source authority stays outside Context Manager in every option.
  edge(s,k+'raw-read',k+'compose',k+'source',[[x+65,310],[x+65,256]],'읽기',[x+23,273,40,20],{size:16});
  edge(s,k+'raw-result',k+'source',k+'compose',[[x+111,256],[x+111,310]],alt==='B'?'공개 변경\n원본':'원본',[x+114,269,79,33],{back:alt!=='B',size:16});
  comp(s,k+'rc','Request Controller',x+15,457,245,76);
  mod(s,k+'permit','전송 검증 및 허가기',x+28,501,219,29,k+'rc',18);
  comp(s,k+'tm','Task Manager',x+305,457,245,76);
  mod(s,k+'command','Agent 명령 확정기',x+318,501,219,29,k+'tm',18);
  comp(s,k+'ag','Agent Gateway',x+15,566,245,77);
  mod(s,k+'send','전송 및 수신 관리기',x+28,610,219,29,k+'ag',18);
  ext(s,k+'agent','외부 Downstream Agent',x+305,591,245,45,17);
  const target=alt==='B'?k+'extract':k+'compose';
  const targetX=alt==='B'?x+280:x+200, targetBottom=alt==='B'?416:344;
  edge(s,k+'prepare',k+'permit',target,alt==='C'?[[x+247,510],[x+285,510],[x+285,337],[x+276,337]]:[[x+247,510],[x+285,510],[x+285,442],[targetX,442],[targetX,targetBottom]],'1 현재 목적 / 구성 요청',[x+150,432,190,22],{size:16});
  const returnY=alt==='B'?399:337;
  edge(s,k+'prepared',target,k+'permit',[[x+28,returnY],[x+5,returnY],[x+5,518],[x+28,518]],'2 인계 후보',[x+10,434,135,21],{back:true});
  edge(s,k+'admission',k+'permit',k+'command',[[x+247,518],[x+318,518]],'3 허가',[x+256,485,57,25],{size:16});
  edge(s,k+'command-send',k+'command',k+'send',[[x+427,530],[x+427,546],[x+280,546],[x+280,623],[x+247,623]],'4 명령',[x+292,535,105,21],{size:17});
  edge(s,k+'dispatch',k+'send',k+'agent',[[x+247,614],[x+305,614]],'5 전송',[x+252,589,51,20],{size:15});
  edge(s,k+'receipt',k+'agent',k+'send',[[x+305,632],[x+247,632]],'확인',[x+259,627,42,18],{back:true,size:15});
  if(alt==='C'){
   edge(s,k+'read-command',k+'agent',k+'send',[[x+460,591],[x+460,575],[x+243,575],[x+243,610]],'6 READ 요청',[x+295,574,152,18],{color:GREEN,size:16});
   edge(s,k+'read-adapter',k+'send',k+'reader',[[x+247,635],[x+289,635],[x+289,451],[x+562,451],[x+562,326],[x+535,326]],'7 범위 / 현재 권한',[x+342,431,211,20],{color:GREEN,size:16});
   edge(s,k+'read-result',k+'reader',k+'send',[[x+535,340],[x+556,340],[x+556,555],[x+270,555],[x+270,628],[x+247,628]],'8 원본 / 거절',[x+410,552,138,20],{back:true,color:GREEN,size:16});
   edge(s,k+'read-delivery',k+'send',k+'agent',[[x+247,638],[x+295,638],[x+295,623],[x+305,623]],null,null,{back:true,color:GREEN});
   // Receipt is drawn after the READ paths so its caption is not crossed by later lines.
   const receiptIndex=s.items.findIndex(o=>o.kind==='edge'&&o.id===k+'receipt');
   s.items.push(s.items.splice(receiptIndex,1)[0]);
  }
 }
 s.note('인계 구조 칸만 교체. 원본은 기존 VIA owner의 권위이며 Context Manager의 자체 DB로 바꾸지 않는다. Request Controller가 현재 목적에 대한 준비를 요청하고 구성된 후보를 반환받은 뒤 현재 조건과 전송 권한을 확인한다. 그 허가를 Task Manager가 Agent Command로 확정하고 Agent Gateway가 실제 전송과 수신 사실을 관리한다. A는 위임 수명 인계 묶음, B는 공개 사건에서 생산한 내구 업무 기록과 목적별 추출, C는 필수 명세와 수신자 READ 왕복이다. C의 read 제공기는 범위/권한/판본/철회를 매번 검사한다. 접근 원장과 읽기 영수증의 물리 저장 방식은 이 그림에서 확정하지 않으며 재시작 시 접근 원장 복원 및 유효성 재검사를 유지한다. B의 목적별 추출기는 게시 근거가 부족하면 생산을 기다리지 않고 현재 원본을 직접 읽어 보완한다. C1 선택 장에서는 이미 채택한 조건 원문을 manifest에 추가하는 보완만 파랑으로 강조한다. A/B에도 기확보 원문과 유효 참조 재사용이 있다. 공통 모델 가공은 필요 시 Model Access를 사용하며 이 인계 확대도 밖에 있다. B의 공개 원본 변경 흐름은 현재 위임의 1~5 요청 흐름과 별도다. 색: 검정 공통, 파랑 A 고유, 초록 B/C 교체, 선택 장 C1 파랑은 보완 강조.');
 return s;
}
export function legacyStructureScenes(Scene){return [dp1Structure(Scene,false),dp1Structure(Scene,true),dp2Structure(Scene,false),dp2Structure(Scene,true),dp4Structure(Scene,false),dp4Structure(Scene,true)];}
