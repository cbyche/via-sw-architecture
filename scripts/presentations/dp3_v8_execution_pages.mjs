// DP3 deployment and lifetime pages. The same scene is used by all renderers.
export function executionPages(Scene) {
  const d = new Scene('dp3-v8-resident-processes', 11, 'DP3. 화면 지칭 해석 구조 (2/2)',
    '같은 후보 생산과 의미 결합 / 시각 CPU 총 4 thread / 한 번에 한 단계 / Model Access는 같은 Component의 분산 배치');
  d.band('B-R2  한 worker에 사용한 모델 유지', 60, 200, 870);
  d.band('B-R3  UI와 SAM worker에 각각 유지', 990, 200, 870);

  // Host responsibilities are identical. These are the same logical Components
  // as page 10; Model Access continues inside each worker below.
  for (const [p, x] of [['r2',60],['r3',990]]) {
    d.proc(p+'host','VIA Host process',x,265,870,235);
    d.comp(p+'producer','Screen Target Producer',x+18,307,340,173,23,p+'host');
    d.mod(p+'assembler','후보 구성기',x+34,352,308,48,24,p+'producer');
    d.store(p+'candidates','후보 기록 (RAM)',x+34,430,308,40,21,p+'producer');
    d.comp(p+'hostma','Model Access',x+470,307,382,173,25,p+'host');
    d.mod(p+'control','로컬 실행 제어기',x+486,352,350,48,24,p+'hostma');
    d.store(p+'jobs','작업과 실행 세대 (RAM)',x+530,430,260,40,20,p+'hostma');
    d.edge(p+'submit',p+'assembler',p+'control',[[x+342,365],[x+486,365]],
      {text:'1 UI / 4 SAM',label:[x+350,332,128,28],size:19});
    d.edge(p+'observations',p+'control',p+'assembler',[[x+486,390],[x+342,390]],
      {text:'3 UI / 6 SAM',label:[x+348,398,136,28],dashed:true,size:18});
    d.edge(p+'writecandidate',p+'assembler',p+'candidates',[[x+188,400],[x+188,430]],{});
    d.edge(p+'writejobs',p+'control',p+'jobs',[[x+660,400],[x+660,430]],{});
  }

  // R2: one queue, one runtime, both adapters and retained sessions in one process.
  d.proc('r2worker','Vision Worker process',60,620,870,398);
  d.comp('r2workerma','Model Access',78,661,834,338,25,'r2worker');
  d.mod('r2ipc','IPC 수신기',110,706,220,45,23,'r2workerma');
  d.store('r2queue','실행 대기열 (RAM)',420,701,260,55,21,'r2workerma');
  d.mod('r2ui','UI 어댑터',170,802,250,45,23,'r2workerma');
  d.mod('r2sam','SAM 어댑터',620,802,250,45,23,'r2workerma');
  d.mod('r2runtime','CPU 추론 런타임',395,884,250,45,23,'r2workerma');
  d.store('r2models','UI + SAM 모델 세션 (RAM)',110,950,380,39,20,'r2workerma');
  d.store('r2episode','영상 작업 상태 (RAM)',600,950,270,39,20,'r2workerma');
  d.edge('r2ipcqueue','r2ipc','r2queue',[[330,728],[420,728]],{text:'접수',label:[338,698,74,25],size:19});
  d.edge('r2queueui','r2queue','r2ui',[[480,756],[480,778],[295,778],[295,802]],
    {text:'UI 차례',label:[325,754,132,24],size:19});
  d.edge('r2queuesam','r2queue','r2sam',[[620,756],[620,778],[745,778],[745,802]],
    {text:'SAM 차례',label:[637,754,125,24],size:19});
  d.edge('r2uiruntime','r2ui','r2runtime',[[295,847],[295,863],[450,863],[450,884]],{});
  d.edge('r2samruntime','r2sam','r2runtime',[[745,847],[745,863],[590,863],[590,884]],{});
  d.edge('r2runtimeweights','r2runtime','r2models',[[430,929],[430,940],[300,940],[300,950]],{});
  d.edge('r2samstate','r2sam','r2episode',[[870,825],[890,825],[890,935],[735,935],[735,950]],{});
  d.edge('r2uiinferreturn','r2runtime','r2ui',[[420,884],[420,873],[200,873],[200,847]],{dashed:true});
  d.edge('r2saminferreturn','r2runtime','r2sam',[[620,884],[620,873],[840,873],[840,847]],{dashed:true});
  d.edge('r2uicompletion','r2ui','r2ipc',[[170,825],[142,825],[142,775],[125,775],[125,751]],{dashed:true});
  d.edge('r2samcompletion','r2sam','r2ipc',[[870,835],[898,835],[898,786],[315,786],[315,751]],{dashed:true});
  d.edge('r2request','r2control','r2ipc',[[560,400],[560,540],[68,540],[68,718],[110,718]],
    {text:'2 UI 실행 → 반환 후 5 필요한 SAM 실행',label:[80,510,460,28],size:21});
  d.edge('r2response','r2ipc','r2control',[[110,742],[48,742],[48,580],[875,580],[875,400]],
    {text:'UI 결과 / SAM 결과를 host로 반환',label:[350,584,490,28],dashed:true,size:21});

  // R3: process-local queues, adapters, runtimes and heavy states. Only the host
  // owns global CPU admission and the execution generation of every job.
  for (const [p,x,title,adapter,model,episode] of [
    ['ui',990,'UI Worker process','UI 어댑터','UI 세션 RAM','UI 상태 RAM'],
    ['sam',1440,'SAM Worker process','SAM 어댑터','SAM 세션 RAM','영상 상태 RAM']
  ]) {
    d.proc(p+'worker',title,x,620,420,398);
    d.comp(p+'ma','Model Access',x+18,661,384,338,24,p+'worker');
    d.mod(p+'ipc','IPC 수신기',x+34,706,165,45,21,p+'ma');
    d.store(p+'queue','대기열 (RAM)',x+239,701,145,55,19,p+'ma');
    d.mod(p+'adapter',adapter,x+100,802,230,45,23,p+'ma');
    d.mod(p+'runtime','CPU 추론 런타임',x+100,884,230,45,22,p+'ma');
    d.store(p+'models',model,x+34,950,165,39,18,p+'ma');
    d.store(p+'state',episode,x+239,950,145,39,18,p+'ma');
    d.edge(p+'enqueue',p+'ipc',p+'queue',[[x+199,728],[x+239,728]],{});
    d.edge(p+'dequeue',p+'queue',p+'adapter',[[x+310,756],[x+310,778],[x+215,778],[x+215,802]],{});
    d.edge(p+'run',p+'adapter',p+'runtime',[[x+215,847],[x+215,884]],{});
    d.edge(p+'weights',p+'runtime',p+'models',[[x+140,929],[x+140,950]],{});
    d.edge(p+'stateflow',p+'adapter',p+'state',[[x+330,825],[x+390,825],[x+390,940],[x+310,940],[x+310,950]],{});
    d.edge(p+'inferreturn',p+'runtime',p+'adapter',[[x+295,884],[x+295,847]],{dashed:true});
    d.edge(p+'completion',p+'adapter',p+'ipc',[[x+100,825],[x+25,825],[x+25,729],[x+34,729]],{dashed:true});
  }
  d.edge('r3uirequest','r3control','uiipc',[[1490,400],[1490,530],[998,530],[998,718],[1024,718]],
    {text:'2 UI 실행',label:[1010,500,190,28],size:21});
  d.edge('r3uiresponse','uiipc','r3control',[[1024,742],[978,742],[978,570],[1805,570],[1805,400]],
    {text:'UI 결과 host 반환',label:[1200,574,250,28],dashed:true,size:21});
  d.edge('r3samrequest','r3control','samipc',[[1826,376],[1846,376],[1846,610],[1448,610],[1448,718],[1474,718]],
    {text:'5 UI 결과로 seed 확정 후 필요한 SAM 실행',label:[1465,580,370,28],size:19});
  d.edge('r3samresponse','samipc','r3control',[[1474,742],[1428,742],[1428,550],[1790,550],[1790,400]],
    {text:'SAM 결과',label:[1598,518,163,28],dashed:true,size:21});
  d.text('제목칸 상자: Component    둥근 상자: Module    원통: 저장소    회색 점선: process    실선: 요청    점선: 반환',60,1024,1800,26,20,{color:'#50555B'});
  d.note('화면 생산의 요청과 반환은 1 생산기 UI 요청, 2 worker UI 실행, 3 UI 결과 생산기 반환, 4 정밀 경계가 필요할 때 생산기 SAM 요청, 5 worker SAM 실행, 6 SAM 결과 생산기 반환 순서다. 후보 구성기가 UI 결과에서 seed를 결정한 뒤에만 SAM 작업을 제출한다. native 후보가 충분하거나 UI bbox가 충분하면 해당 시각 추론은 생략한다. 단계 번호는 이 페이지의 host-worker 흐름이며 p10의 요청 의미 해석 번호와 별개다.');
  d.note('각 대안의 로컬 실행 제어기는 모든 시각 worker에 걸쳐 총 CPU 4 thread와 heavy stage 1개를 허가한다. 두 worker를 자동 병렬 실행하는 구조가 아니다. IPC 수신기는 Module이고 대기열은 별도의 RAM 저장이다. 모든 호출과 결과선은 실제 Module에 연결된다. UI/SAM 모델 세션은 최초 필요 시 로딩한 이후 유지하며, 영상 작업 상태의 종료는 모델 수명과 별도다. runtime와 adapter 내부 전후처리는 p12에서 확대한다.');
  d.note('R2 crash는 같은 주소 공간의 UI/SAM 모델 세션과 영상 tensor에 영향을 준다. R3 SAM crash는 UI worker 주소 공간을 직접 제거하지 않는다. host 후보 기록은 두 안 모두 남는다. 실행 세대 확인으로 이전 worker 결과를 거절하고 권한 있는 원본과 seed에서 필요한 tensor를 재생산한다.');

  const e = new Scene('dp3-v8-runtime-lifetimes',12,'DP3. CPU 실행과 상태 수명',
    '모델 회수와 영상 상태 종료는 별도 명령 / R1과 R2의 코드 배치는 같고 세션 유지 정책이 다름');
  e.proc('lifehost','VIA Host process',60,215,440,380);
  e.comp('lifema','Model Access',78,258,404,316,25,'lifehost');
  e.mod('lifecontrol','로컬 실행 제어기',100,311,360,60,25,'lifema');
  e.store('lifejobs','작업과 실행 세대 (RAM)',100,465,360,70,23,'lifema');
  e.edge('lifejobwrite','lifecontrol','lifejobs',[[120,371],[120,465]],
    {text:'작업 / 취소 / 버전 확인',label:[140,402,310,28],size:22});
  e.proc('lifeworker','Vision Worker process (R1 / R2)',735,215,1125,380);
  e.comp('lifeworkerma','Model Access',753,258,1089,316,25,'lifeworker');
  e.mod('lifeipc','IPC 수신기',775,311,190,60,24,'lifeworkerma');
  e.mod('lifeadapter','UI / SAM 어댑터',1060,311,240,60,24,'lifeworkerma');
  e.mod('liferuntime','CPU 추론 런타임',1395,311,240,60,24,'lifeworkerma');
  e.store('lifefiles','모델 파일\n(local disk)',1685,302,135,83,21,'lifeworkerma');
  e.store('lifeepisode','영상 작업 상태 (RAM)',1060,466,240,70,22,'lifeworkerma');
  e.store('lifemodels','모델 세션 (RAM)',1395,466,240,70,23,'lifeworkerma');
  e.edge('liferequest','lifecontrol','lifeipc',[[460,330],[775,330]],
    {text:'1 작업 / 수명 명령',label:[510,295,222,30],size:21});
  e.edge('liferesult','lifeipc','lifecontrol',[[775,355],[460,355]],
    {text:'완료 / 오류 / 실행 세대',label:[500,370,248,51],dashed:true,size:21});
  e.text('named pipe: 명령과 결과\n공유 buffer: frame 참조',508,222,220,58,19,{align:'center'});
  e.text('제목칸 상자: Component\n둥근 상자: Module\n원통: 저장소\n회색 점선: process\n실선: 요청 / 점선: 반환',508,437,220,135,19,{align:'center',color:'#50555B'});
  e.edge('lifedispatch','lifeipc','lifeadapter',[[965,340],[1060,340]],
    {text:'2 분배',label:[972,300,83,28],size:20});
  e.edge('lifeinfer','lifeadapter','liferuntime',[[1300,340],[1395,340]],
    {text:'3 호출',label:[1307,300,83,28],size:20});
  e.edge('lifeload','lifefiles','liferuntime',[[1685,343],[1635,343]],
    {text:'load',label:[1638,309,43,24],size:18});
  e.edge('lifemodelsession','liferuntime','lifemodels',[[1615,371],[1615,466]],
    {text:'EnsureModel: 적재\nReleaseModel: 회수',label:[1398,391,201,58],size:20});
  e.edge('lifeepisodestore','lifeadapter','lifeepisode',[[1280,371],[1280,466]],
    {text:'RunStage: 사용과 갱신\nCloseEpisode: 종료',label:[1063,391,201,58],size:20});
  e.edge('lifeinferreturn','liferuntime','lifeadapter',[[1395,355],[1300,355]],
    {text:'결과',label:[1312,376,76,26],dashed:true,size:19});
  e.edge('lifecompletion','lifeadapter','lifeipc',[[1060,355],[965,355]],
    {text:'4 결과',label:[974,376,77,26],dashed:true,size:19});
  e.text('ReleaseModel은 모델 세션만 회수하고 CloseEpisode는 영상 상태만 종료한다. 사용 중 참조가 없을 때 수행한다.',60,605,1800,29,23,{bold:true});
  const rows = [
    ['수명 / 동시 RAM (MB)','A','B-R1','B-R2','B-R3'],
    ['시각 모델 세션','없음','단계 후 회수와 교체','사용 후 유지','worker별 유지'],
    ['영상 상태 / 후보 기록','확정 이력','모델 회수와 별도 보존','같은 보존 정책','같은 보존 정책'],
    ['추가 시각 runtime + 모델 객체','0','340 + 170','340 + 590','620 + 590'],
    ['공통 / 추론과 영상 상태 / 입력 buffer','560 / 0 / 32','560 / 860 / 24','560 / 860 / 24','560 / 860 / 32'],
    ['QA06 동시 peak 합계','592','1,954','2,374','2,662']
  ];
  e.table(rows,60,651,[610,270,310,310,300],[49,57,57,57,57,60],24);
  e.text('가정 RAM, 실측 아님. R1→R2: UI 모델 객체 420 MB 유지. R3: runtime private 280 MB + IPC 8 MB 추가.',60,1004,1800,31,22,{color:'#50555B'});
  e.note('실행 제어기는 worker의 기동, CPU 실행 허가, 작업과 실행 세대, 취소 및 결과 유효성 확인을 책임진다. Model Access의 동일 논리 Component를 host와 worker에 나누어 배치한다. IPC 수신기에서 UI/SAM 어댑터로 명령을 분배하며 adapter가 CPU runtime을 호출한다. EnsureModel은 이미 설치된 disk 파일을 runtime으로 읽어 모델 객체와 weights를 구성한다. disk 파일은 RAM 합계에 넣지 않는다.');
  e.note('RunStage, EnsureModel, ReleaseModel, CloseEpisode는 named pipe의 별도 명령이다. IPC와 adapter를 거쳐 ReleaseModel은 runtime의 모델 세션 회수로, CloseEpisode는 adapter가 소유한 영상 tensor 종료로 이어진다. 각 path는 그림에 실제로 연결되어 있다. 추론 결과는 runtime에서 adapter로 돌아온 뒤 전후처리와 판본을 갖춰 IPC에 반환한다. runtime을 호출하지 않는 CloseEpisode도 adapter에서 완료 또는 오류를 IPC로 반환하는 같은 경로를 갖는다. ReleaseModel이 사용 중이거나 tensor가 참조하는 model을 강제 삭제한다는 의미가 아니며 참조 확인 후 회수하고 실제 RAM을 따로 확인한다. 공통 원본과 host 후보 기록은 둘 중 어느 명령으로도 삭제하지 않는다.');
  e.note('CPU runtime은 bundled Python + PyTorch CPU 실행기다. R1은 runtime을 유지하면서 필요한 모델 session을 교체한다. R2는 최초 필요 시 load한 모델 session을 유지한다. R3는 p11의 두 process에 같은 실행 구조가 각각 배치된다. 모델 상주만으로 영상 상태 보존이나 정확도 차이를 만들지 않는다.');
  e.note('RAM 값은 v7의 합성 가정을 그대로 유지한다. 공통 560 MB, UI 객체 420 MB, SAM 객체 170 MB, shared runtime 60 MB와 process private 280 MB, 추론 workspace 680 MB와 영상 상태 180 MB, 입력 buffer A32/R1R2 24/R3 32 MB. SAM peak이며 UI workspace를 동시에 더하지 않는다. UI와 SAM의 실제 추론을 수행해 측정한 값이 아니다.');
  return [d,e];
}
