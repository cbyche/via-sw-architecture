// DP5 p21: the existing request/source and producer responsibilities, with closed read loops.
// Compatible with scripts/presentations/dp3_v8_scene.mjs. No new process, model, or QA claim.
export function dp5GroundingLoopsPage(Scene) {
  const d = new Scene('dp5-v10-query-and-fallback-loops', 21,
    'DP5. 위임을 위한 메모리 관리 (설계 구조 비교)',
    '차이의 중심: 요청이 관계를 해석하는가, 사건 갱신 경로가 파생 관계를 유지하는가');
  const blue='#4676CB', green='#32824A', black='#111111';
  d.band('A  요청 경로에서 원본 관계 해석',40,184,900,48);
  d.band('B  사건 갱신 경로 + 관계 조회 경로',980,184,900,48);
  d.rule([[960,184],[960,930]],{color:'#B7B9B9',width:1});

  // A: permit, source read, interpretation and additional-read request are a closed loop.
  d.comp('a-rc','Request Controller',60,255,400,130);
  d.mod('a-permit','허가 및 의미 채택기',80,308,360,55,24,'a-rc');
  d.comp('a-cm','Context Manager',610,255,310,205);
  d.mod('a-read','원본 조회기',630,308,270,55,24,'a-cm');
  d.store('a-read-state','읽은 판본과 허용 범위\n(RAM)',630,390,270,57,21,'a-cm');
  d.ext('a-source','공통 원본과 확정 사실\n기존 소유자 권위 유지',610,535,310,100,24);
  d.comp('a-ri','Request Interpreter',60,650,400,190);
  d.mod('a-interpret','원본 관계 해석기',80,704,210,55,24,'a-ri');
  d.store('a-state','해석 후보\n근거와 불확실성\n(요청 RAM)',310,700,130,112,19,'a-ri');
  d.edge('a-permitted-read','a-permit','a-read',[[440,325],[630,325]],
    {text:'1 조회 허가\n추가 범위 재허가',label:[466,268,150,49],size:20});
  d.edge('a-read-result','a-read','a-permit',[[630,349],[550,349],[550,418],[410,418],[410,363]],
    {text:'2 원본 또는 조회 불가',label:[325,425,245,29],size:20,dashed:true});
  d.edge('a-revision','a-read','a-read-state',[[765,363],[765,390]],{});
  d.edge('a-source-query','a-read','a-source',[[900,326],[938,326],[938,570],[920,570]],
    {text:'허용 범위 읽기',label:[730,481,195,29],size:20});
  d.edge('a-source-return','a-source','a-read',[[610,605],[585,605],[585,350],[630,350]],
    {text:'원본과 판본',label:[615,641,210,24],size:20,dashed:true});
  d.edge('a-resume','a-permit','a-interpret',[[130,363],[130,605],[40,605],[40,731],[80,731]],
    {text:'3 원본 전달 / 보완 후 재개',label:[60,555,355,32],size:22});
  d.edge('a-proposal-or-read','a-interpret','a-permit',[[180,759],[180,854],[490,854],[490,344],[440,344]],
    {text:'4 의미 제안 / 추가 읽기',label:[195,818,280,28],size:20,dashed:true});
  d.edge('a-work-state','a-interpret','a-state',[[290,747],[310,747]],{});
  d.text('미확정 관계는 현재 요청에서 해석\n추가 영속 관계 상태는 없음\n확정 사실과 유효 근거는 재사용',610,697,325,120,23,{align:'left'});
  d.text('추가 읽기: 4 → 1 → 2 → 3\n최종 제안: 현재 판본과 권한 확인 후 채택',60,860,860,62,22);

  // B: Context Manager owns background production and persisted derived relations.
  // A query has one allowed scope. A wider source need returns to Controller for reauthorization.
  d.comp('b-rc','Request Controller',1000,255,340,130);
  d.mod('b-permit','허가 및 의미 채택기',1020,308,300,55,24,'b-rc');
  d.comp('b-cm','Context Manager',1450,255,430,500);
  d.mod('b-query','관계 조회기',1470,308,180,55,23,'b-cm');
  d.mod('b-read','원본 조회기',1680,410,180,55,23,'b-cm');
  d.ext('b-source','같은 공통 원본과 확정 사실\n기존 소유자 권위 유지',1000,445,340,100,23);
  d.mod('b-track','변경 추적기',1470,550,180,55,23,'b-cm');
  d.mod('b-produce','관계 생산 및 게시기',1670,550,190,55,21,'b-cm');
  d.store('b-meta','의존 판본과 처리 진척\n미확인 / 무효 상태\n(영속)',1470,650,180,78,19,'b-cm');
  d.store('b-relations','파생 관계와 근거\n대상 / 측면 / 상황\n(영속)',1670,650,190,78,19,'b-cm');
  d.comp('b-ri','Request Interpreter',1000,690,340,160);
  d.mod('b-interpret','관계 적용 및\n원본 보완 해석기',1020,741,200,66,22,'b-ri');
  d.store('b-state','해석 상태\n(요청 RAM)',1240,740,80,88,18,'b-ri');
  d.edge('b-permitted-query','b-permit','b-query',[[1320,325],[1470,325]],
    {text:'1 관계 / 원본\n조회 허가',label:[1337,267,116,50],size:20});
  d.edge('b-query-result','b-query','b-permit',[[1470,349],[1385,349],[1385,409],[1290,409],[1290,363]],
    {text:'3 관계 / 원본 / 더 넓은 범위 필요',label:[1350,370,313,26],size:19,dashed:true});
  d.edge('b-source-fallback','b-query','b-read',[[1650,340],[1665,340],[1665,429],[1680,429]],
    {text:'2 미생산 / 무효 / 부족\n허가 범위 원본 보완',label:[1668,365,210,42],size:18});
  d.edge('b-source-back','b-read','b-query',[[1680,452],[1660,452],[1660,357],[1650,357]],
    {text:'원본 / 조회 불가',label:[1690,471,180,25],size:19,dashed:true});
  d.edge('b-read-source','b-read','b-source',[[1680,425],[1432,425],[1432,478],[1340,478]],
    {text:'읽기',label:[1348,448,70,24],size:19});
  d.edge('b-source-result','b-source','b-read',[[1340,518],[1413,518],[1413,454],[1680,454]],
    {text:'원본과 판본',label:[1340,520,110,24],size:19,dashed:true});
  d.edge('b-interpret-resume','b-permit','b-interpret',[[1020,350],[980,350],[980,772],[1020,772]],
    {text:'4 근거 전달 / 보완 후 재개',label:[1000,627,330,29],size:21});
  d.edge('b-proposal-or-read','b-interpret','b-permit',[[1170,807],[1170,845],[1360,845],[1360,344],[1320,344]],
    {text:'5 의미 제안 / 추가 읽기',label:[1375,780,430,28],size:20,dashed:true});
  d.edge('b-interpret-state','b-interpret','b-state',[[1220,794],[1240,794]],{});

  // Background updates are event-triggered, not an obligatory step after a user query.
  d.edge('b-source-event','b-source','b-track',[[1340,535],[1375,535],[1375,577],[1470,577]],
    {text:'사건: 공개 결과 / 피드백 / 정정',label:[1000,558,340,27],size:19});
  d.edge('b-produce-trigger','b-track','b-produce',[[1650,575],[1670,575]],{});
  d.edge('b-producer-source-read','b-produce','b-read',[[1860,566],[1875,566],[1875,443],[1860,443]],
    {text:'생산도 같은 원본 조회',label:[1668,508,210,26],size:18});
  d.edge('b-producer-source-result','b-read','b-produce',[[1860,458],[1883,458],[1883,590],[1860,590]],{dashed:true});
  d.edge('b-track-state','b-track','b-meta',[[1550,605],[1550,650]],
    {text:'진척 / 무효화',label:[1460,611,192,28],size:19});
  d.edge('b-publish','b-produce','b-relations',[[1760,605],[1760,650]],
    {text:'판본과 권한 재검사 후 게시',label:[1660,611,218,28],size:18});
  d.edge('b-state-check','b-meta','b-query',[[1470,690],[1442,690],[1442,349],[1470,349]],{});
  d.edge('b-relation-read','b-relations','b-query',[[1860,690],[1893,690],[1893,330],[1650,330]],
    {text:'관계 / 현재성 / 확인 범위',label:[1665,300,210,26],size:18,dashed:true});
  d.text('생산은 요청과 별도. 필요한 의미 해석만 동일 모델 사용',1452,815,425,35,19);
  d.text('의미 불일치 / 추가 범위: 5 → 1 → 2 → 3 → 4\n허가 범위 안의 원본 보완은 2에서 수행',1000,860,860,62,21);

  // State ownership and previous selection/QA status stay unchanged.
  d.text('검정: 공통 / 파랑: A 고유 / 초록: B 고유    실선: 요청 / 점선: 결과    제목 상자: Component / 둥근 상자: Module / 원통: 상태',40,925,1840,26,20);
  d.fragment={page:21,replaceBounds:{x:24,y:180,w:1872,h:773},preserve:['title','subtitle','native cost table at y959','footer','existing notes and QA status']};

  for (const id of ['a-interpret']) d.nodes[id].stroke=blue;
  for (const id of ['b-query','b-track','b-produce','b-meta','b-relations','b-interpret']) d.nodes[id].stroke=green;
  for (const n of Object.values(d.nodes)) if (!n.stroke) n.stroke=black;
  for (const e of d.items.filter(x=>x.kind==='edge')) {
    if (['a-work-state'].includes(e.id)) e.color=blue;
    if (['b-source-event','b-produce-trigger','b-track-state','b-publish','b-state-check','b-relation-read','b-source-fallback','b-source-back'].includes(e.id)) e.color=green;
  }
  d.note('기존 DP5 설계와 QA, 원본 권위 및 미선택 상태를 보존한다. Request Controller의 허가 및 의미 채택기는 현재 요청의 허용 범위를 검증하고 최종 제안을 채택하는 내부 책임이다. Context Manager의 조회는 그 범위에서 원본 권한과 현재 판본을 다시 확인한다.');
  d.note('A 추가 읽기는 4→1→2→3으로 반복한다. B 조회는 유효한 관계가 없거나 coverage가 부족하면 현재 허가 범위에서 원본을 읽는다. 더 넓은 범위는 3으로 반환하고 Request Controller에서 재허가한 후 1을 반복한다. 의미 적용 중 불일치 또는 추가 필요가 나오면 5→1→2→3→4로 보완한다. 조회 불가나 원본 모호성은 동일 불확실성 및 확인 절차로 반환하며 성공으로 채택하지 않는다.');
  d.note('B의 사건 생산은 사용자 요청 처리 번호에 포함하지 않는 별도 흐름이다. 원본 기록 후 변경 추적, 조건부 의미 생산, 원본 판본 및 권한 재검사, 파생 관계 게시를 수행한다. producer도 같은 원본 조회 계약을 사용한다. 모델 호출 횟수, process 분리, 모델 복제를 추가하지 않는다.');
  return d;
}

// Only the two Context Manager fragments change on p22. Existing decision table,
// tactic headings, advantages/costs, and unselected status are kept by the packager.
export function dp5TacticFragmentsPage(Scene) {
  const d=new Scene('dp5-v10-tactic-state-fragments',22,
    'DP5. 위임을 위한 메모리 관리 (선택 기준 및 보완)',
    '기본안 미선택 / 과거 관계의 재참조 빈도와 첫 응답의 실제 대기로 판단');
  const blue='#4676CB';
  d.comp('a1-cm','Context Manager',75,606,810,168);
  d.mod('a1-update','색인 갱신기',92,650,200,50,23,'a1-cm');
  d.store('a1-index','관계 재조회 색인\n(요청 간 보존)',430,650,190,70,21,'a1-cm');
  d.mod('a1-check','권한 및 판본 검증기',650,650,215,50,21,'a1-cm');
  d.edge('a1-record','a1-update','a1-index',[[292,675],[430,675]],
    {text:'확정 관계 반영',label:[299,646,124,26],size:18,color:blue});
  d.edge('a1-invalidate','a1-check','a1-update',[[650,680],[625,680],[625,748],[215,748],[215,700]],
    {text:'원본 변경 시 갱신',label:[320,722,260,22],size:19,color:blue});
  d.text('기존 권한과 현재성 검사 유지',650,710,220,43,18,{color:'#111111',align:'center'});

  d.comp('b1-cm','Context Manager',1025,606,810,168);
  d.mod('b1-batch','사건 묶음\n조정기',1042,650,180,45,21,'b1-cm');
  d.store('b1-pending','대기 사건\n(RAM)',1240,650,150,80,20,'b1-cm');
  d.mod('b1-produce','관계 생산기',1410,650,180,45,21,'b1-cm');
  d.mod('b1-query','관계 조회기',1610,650,205,40,21,'b1-cm');
  d.mod('b1-source','원본 조회기',1610,726,205,36,21,'b1-cm');
  d.edge('b1-add','b1-batch','b1-pending',[[1222,670],[1240,670]],{});
  d.edge('b1-start','b1-pending','b1-produce',[[1390,670],[1410,670]],{});
  d.edge('b1-pending-read','b1-pending','b1-query',[[1390,708],[1600,708],[1600,670],[1610,670]],
    {text:'미처리 범위 확인',label:[1410,712,185,23],size:18});
  d.edge('b1-fallback','b1-query','b1-source',[[1715,690],[1715,726]],
    {text:'미게시 범위\n원본 보완',label:[1718,692,105,33],size:18});
  d.edge('b1-fallback-result','b1-source','b1-query',[[1815,744],[1828,744],[1828,665],[1815,665]],{dashed:true});
  d.text('호환 사건만 합류',1040,710,190,26,19,{align:'center'});
  d.text('묶음 확정 후 생산',1405,742,195,23,18,{align:'center'});
  for(const n of Object.values(d.nodes))n.stroke=['a1-update','a1-index','b1-batch','b1-pending'].includes(n.id)?blue:'#111111';
  d.fragment={page:22,replaceBounds:[{x:75,y:606,w:810,h:168},{x:1025,y:606,w:810,h:168}],preserve:['all text and objects outside the two Context Manager rectangles']};
  d.note('A1과 B1은 여전히 미평가 보완 후보다. A1 색인은 요청 후 재조회에 쓰도록 요청 사이에 보존하는 상태이며, 디스크 형식이나 재시작 복원 계약을 새로 확정하지 않는다. B1의 대기 사건은 생산 전 임시 RAM 상태이며 게시된 파생 관계 저장소와 구별한다. B1이 게시를 기다리지 못하는 요청은 기본안 B의 원본 보완을 계속 사용한다.');
  return d;
}
