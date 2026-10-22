// Current-guide replacements for the LEFT mechanism figures only on p4, p8, p19.
// Source QA tables, values, tactic headings, footnotes and recommendation status stay in the base package.
const BLACK='#111111', BLUE='#4676CB';
const edge=(s,id,from,to,points,text,label,back=false,color=BLACK,size=17)=>s.edge(id,from,to,points,{...(text?{text,label,size}:{}),dashed:back,color});
function paint(s){for(const n of Object.values(s.nodes)) if(!n.stroke)n.stroke=BLACK;return s;}

export function dp1TacticDetail(Scene){
 const s=new Scene('dp1-v10-tactic-detail-left',4,'DP1. 현재 요청의 근거 공급 방안 (보완 효과 상세)','기본 A와 변경 조각 갱신 A2의 책임 확대도');
 s.comp('a-cm','Context Manager',50,330,250,205,20);
 s.mod('a-query','근거 조회 및 문맥 갱신기',65,380,220,36,18,'a-cm');
 s.store('a-view','유효 작업 문맥\n출처 / 의존 판본\n(대화 수명)',65,449,220,75,18,'a-cm');
 s.comp('a-rc','Request Controller',350,330,250,88,20);
 s.mod('a-adopt','허가 및 의미 채택기',365,377,220,34,18,'a-rc');
 s.comp('a-ri','Request Interpreter',350,455,250,84,20);
 s.mod('a-interpret','요청 의미 해석기',365,501,220,32,18,'a-ri');
 edge(s,'a-view-permit','a-adopt','a-query',[[365,389],[285,389]],'조회 허가\n채택 후 갱신',[291,336,61,42],false,BLACK,15);
 edge(s,'a-view-return','a-query','a-adopt',[[285,406],[365,406]],'근거',[297,416,54,20],true,BLACK,16);
 edge(s,'a-view-read','a-view','a-query',[[120,449],[120,416]],'조회',[67,423,49,20],true,BLACK,16);
 edge(s,'a-view-write','a-query','a-view',[[240,416],[240,449]],'갱신',[244,423,45,20],false,BLACK,16);
 edge(s,'a-current-input','a-adopt','a-interpret',[[415,411],[415,439],[335,439],[335,515],[365,515]],'현재 입력 + 유효 근거',[371,418,223,20],false,BLACK,17);
 edge(s,'a-meaning-proposal','a-interpret','a-adopt',[[585,518],[620,518],[620,397],[585,397]],null,null,true);
 // An independent helper is not introduced. A2 is another internal path of the same Context Manager.
 s.comp('a2-cm','Context Manager',275,670,325,192,20);
 s.mod('a2-delta','변경 조각 갱신기',290,719,295,38,19,'a2-cm');
 s.mod('a2-check','전체 근거 판본 검증기',290,811,295,36,19,'a2-cm');
 s.ext('a2-source','원본 소유자 (VIA 내부)\n변경 판본 v1 → v2',50,680,185,85,18);
 edge(s,'a2-read','a2-delta','a2-source',[[290,733],[235,733]],'읽기',[237,706,50,20],false,BLUE,16);
 edge(s,'a2-raw','a2-source','a2-delta',[[235,753],[290,753]],'원본',[237,755,50,20],true,BLUE,16);
 edge(s,'a2-validate','a2-delta','a2-check',[[435,757],[435,811]],'전체 현재성 재검사',[446,772,145,23],false,BLUE,17);
 edge(s,'a2-view-update','a2-check','a-query',[[290,831],[32,831],[32,400],[65,400]],'검증 후 문맥 갱신',[50,790,220,30],false,BLUE,18);
 for(const id of ['a2-delta','a2-check'])s.nodes[id].stroke=BLUE;
 s.fragment={page:4,removeShapeIds:[...Array.from({length:15},(_,i)=>21+i),...Array.from({length:12},(_,i)=>39+i)],replaceBounds:[{x:24,y:320,w:610,h:219},{x:24,y:665,w:610,h:203}],preserveShapeIds:[16,17,18,19,20,36,37,38,51,52],preserve:['all right-side tables, QA values, first-response tactic comparison','A heading and red selection check','A2 heading','uncertain-impact full-reconstruction condition','A2 not-adopted status']};
 s.note('p4 좌측 기본 구조와 A2 그림만 갱신한다. Context Manager의 유효 작업 문맥이 최초 현재 입력에도 공급되며 Request Controller가 읽기 허가와 최종 의미를 채택한다. 현재성 확인을 통과한 문맥만 다음 요청에 재사용한다. 위아래 Context Manager는 동일 Component 책임을 기본 경로와 A2 확대 경로로 반복 표시한 것이며 두 인스턴스나 새 영속 저장소가 아니다. A2는 변경 구간만 읽고 전체 근거 판본을 다시 검사해 기존 문맥 갱신기에 전달한다. 영향 범위가 불명확하면 기존 전체 재구성으로 복귀한다. 파랑은 기존 A2 보완이다. 실제 근거의 A2 미채택 상태와 수치는 변경하지 않는다.');
 return paint(s);
}

export function dp2TacticDetail(Scene){
 const s=new Scene('dp2-v10-tactic-detail-left',8,'DP2. 후속 입력의 의미 판단 실행 방안 (보완 효과 상세)','B1의 불변 입력 공유와 개별 가변 상태');
 s.comp('base-ri','Request Interpreter',50,333,560,162,21);
 s.mod('base-r1','요청 해석 실행기 (R1)',70,380,225,35,18,'base-ri');
 s.mod('base-r2','요청 해석 실행기 (R2)',365,380,225,35,18,'base-ri');
 s.store('base-input1','불변 입력 / 원본 사본 1\n(RAM)',70,434,225,50,17,'base-ri');
 s.store('base-input2','불변 입력 / 원본 사본 2\n(RAM)',365,434,225,50,17,'base-ri');
 edge(s,'base-use1','base-input1','base-r1',[[180,434],[180,415]],null,null,true);
 edge(s,'base-use2','base-input2','base-r2',[[475,434],[475,415]],null,null,true);
 s.text('B1 적용',365,515,220,33,21,{color:BLUE,bold:true,align:'center'});
 s.comp('b1-ri','Request Interpreter',50,575,560,287,21);
 s.store('b1-shared','B1 공유 불변 입력 / 원본\n(RAM)',170,620,320,55,19,'b1-ri');
 s.mod('b1-r1','요청 해석 실행기 (R1)',70,735,225,38,18,'b1-ri');
 s.mod('b1-r2','요청 해석 실행기 (R2)',365,735,225,38,18,'b1-ri');
 s.store('b1-state1','R1 가변 상태\nrevision / snapshot (RAM)',70,795,225,55,17,'b1-ri');
 s.store('b1-state2','R2 가변 상태\nrevision / snapshot (RAM)',365,795,225,55,17,'b1-ri');
 edge(s,'b1-input-r1','b1-shared','b1-r1',[[170,650],[145,650],[145,735]],'참조',[80,690,60,22],true,BLUE,18);
 edge(s,'b1-input-r2','b1-shared','b1-r2',[[490,650],[510,650],[510,735]],'참조',[520,690,60,22],true,BLUE,18);
 edge(s,'b1-save1','b1-r1','b1-state1',[[150,773],[150,795]],null,null);
 edge(s,'b1-read1','b1-state1','b1-r1',[[225,795],[225,773]],null,null,true);
 edge(s,'b1-save2','b1-r2','b1-state2',[[445,773],[445,795]],null,null);
 edge(s,'b1-read2','b1-state2','b1-r2',[[520,795],[520,773]],null,null,true);
 s.nodes['b1-shared'].stroke=BLUE;
 s.fragment={page:8,removeShapeIds:[...Array.from({length:10},(_,i)=>20+i),...Array.from({length:12},(_,i)=>31+i)],replaceBounds:[{x:24,y:330,w:612,h:532}],preserveShapeIds:[16,17,18,19,30,43,44],preserve:['all right-side tables and numeric diagnostics','basic B label','B1 title','mutable-state/recovery and payload-serialization captions']};
 s.note('p8 좌측 불변 입력 복사와 B1 공유 구조만 갱신한다. 위는 기본 B, 아래는 B1을 적용한 같은 Request Interpreter 책임의 비교이며 두 Component 인스턴스를 동시 실행하는 뜻이 아니다. R1/R2는 논리 실행이고 별도 process 또는 모델 복제가 아니다. 공통 Model Access 호출 한도는 p6/7과 같다. 기본 B는 실행별 입력 사본을 보유하고 B1은 불변 입력만 공유한다. R1/R2 가변 상태와 복구 규칙은 그대로 분리된다. 파랑은 기존 B1 보완이며 QA 원값과 재생 비교의 적용 범위를 변경하지 않는다.');
 return paint(s);
}

export function dp4TacticDetail(Scene){
 const s=new Scene('dp4-v10-tactic-detail-left',19,'DP4. Agent 간 업무 인계 방안 (보완 효과 상세)','C1의 기존 조건 원문 사전 읽기와 공통 인계 승인');
 s.text('검정 공통 / 파랑 C1   실선 요청 / 점선 반환',45,292,552,20,16);
 s.comp('c1-cm','Context Manager',50,315,250,214,20);
 s.mod('c1-read','원본 및 범위 조회기',65,364,220,34,18,'c1-cm');
 s.store('c1-attachment','조건 원문 첨부 상태\n(위임 수명)',65,415,220,50,17,'c1-cm');
 s.mod('c1-manifest','필수 명세 구성기',65,489,220,34,18,'c1-cm');
 s.comp('c1-rc','Request Controller',350,315,250,100,20);
 s.mod('c1-permit','인계 검증 및 허가기',365,364,220,34,18,'c1-rc');
 s.comp('c1-tm','Task Manager',350,480,250,100,20);
 s.mod('c1-command','Agent 명령 확정기',365,529,220,34,18,'c1-tm');
 s.comp('c1-ag','Agent Gateway',350,645,250,100,20);
 s.mod('c1-send','전송기',365,695,95,33,18,'c1-ag');
 s.mod('c1-read-adapter','READ 연결기',475,695,110,33,18,'c1-ag');
 s.ext('c1-source','채택 조건 원문\nsource refs\n“가격이 같으면\n유지보수 기간 우선”',50,590,250,100,18);
 s.ext('c1-agent','외부 Downstream Agent',350,785,250,42,17);
 s.ext('c1-detail-source','허용된 상세 자료 원본\n기존 소유자 권위 유지',50,775,250,50,16);
 edge(s,'c1-authorize-read','c1-permit','c1-read',[[365,375],[285,375]],'1 허가',[301,337,60,22],false,BLUE,16);
 edge(s,'c1-source-read','c1-read','c1-source',[[65,380],[35,380],[35,620],[50,620]],'원문 1~2행 읽기',[60,548,230,24],false,BLUE,18);
 edge(s,'c1-source-return','c1-source','c1-read',[[50,660],[25,660],[25,390],[65,390]],null,null,true,BLUE);
 edge(s,'c1-attachment-save','c1-read','c1-attachment',[[175,398],[175,415]],null,null,false,BLUE);
 edge(s,'c1-attach-to-manifest','c1-attachment','c1-manifest',[[175,465],[175,489]],'명세 첨부',[183,466,105,20],false,BLUE,16);
 edge(s,'c1-proposal','c1-manifest','c1-permit',[[285,499],[325,499],[325,389],[365,389]],'2 명세',[301,418,60,22],true,BLACK,16);
 edge(s,'c1-command-authority','c1-permit','c1-command',[[585,379],[610,379],[610,545],[585,545]],'3 현재 조건 / 전송 권한 확인',[356,438,244,24],false,BLACK,17);
 edge(s,'c1-command-send','c1-command','c1-send',[[365,548],[337,548],[337,711],[365,711]],'4 명령 전달',[357,600,233,23],false,BLACK,17);
 edge(s,'c1-send-contract','c1-send','c1-agent',[[412,728],[412,785]],'5 계약 전송',[355,746,112,18],false,BLACK,16);
 edge(s,'c1-agent-read','c1-agent','c1-read-adapter',[[600,806],[612,806],[612,711],[585,711]],null,null,false,BLACK);
 edge(s,'c1-serving-request','c1-read-adapter','c1-read',[[475,704],[468,704],[468,765],[312,765],[312,383],[285,383]],'상세 READ 요청\n기존 범위 / 권한 검사',[58,712,242,48],false,BLACK,18);
 edge(s,'c1-serving-result','c1-read','c1-read-adapter',[[285,393],[306,393],[306,770],[467,770],[467,719],[475,719]],null,null,true,BLACK);
 edge(s,'c1-detail-read','c1-read','c1-detail-source',[[65,396],[42,396],[42,790],[50,790]],null,null,false,BLACK);
 edge(s,'c1-detail-return','c1-detail-source','c1-read',[[50,811],[46,811],[46,395],[65,395]],null,null,true,BLACK);
 edge(s,'c1-serving-delivery','c1-read-adapter','c1-agent',[[530,728],[530,785]],'원본 / 거절',[470,750,130,22],true,BLACK,16);
 for(const id of ['c1-read','c1-attachment'])s.nodes[id].stroke=BLUE;
 s.fragment={page:19,removeShapeIds:Array.from({length:18},(_,i)=>19+i),replaceBounds:[{x:24,y:290,w:592,h:537}],preserveShapeIds:[16,17,18,37,38],preserve:['all right-side QA comparison and numbers','conditional C1 heading','original read counts, prefetch counts and maximum attachment bytes','unmeasured source-creation caveat']};
 s.note('p19 좌측 C1 실행만 확대한다. 채택 조건의 출처가 이미 있는 경우에 한해 Context Manager가 원문 1~2행을 미리 읽고 필수 명세에 첨부한다. 조건 출처를 새로 만드는 기능과 비용을 발명하지 않는다. Request Controller가 구성된 명세를 검증하고 전송을 허가한 뒤 Task Manager가 명령을 확정하고 Agent Gateway가 보낸다. 이는 최종 의미를 새로 해석하는 LLM 호출 추가가 아니다. 조건 원문과 상세 자료 원본을 두 원본 범위로 구분해 표시하며 새로운 저장소를 추가하지 않는다. 상세 자료는 기존 C의 수신자 READ 경로를 유지하며 Context Manager의 조회기는 현재 허용 범위, 권한과 원본 판본을 검사하고 불가하면 거절한다. 원본과 조건의 기존 owner 권위를 바꾸지 않는다. 파랑은 C1의 사전 읽기와 원문 첨부이며 공통 명령/전송/READ는 검정이다. 원래 별도 실험 수치와 미확인 조건을 보존한다.');
 return paint(s);
}
export function tacticDetailScenes(Scene){return [dp1TacticDetail(Scene),dp2TacticDetail(Scene),dp4TacticDetail(Scene)];}
