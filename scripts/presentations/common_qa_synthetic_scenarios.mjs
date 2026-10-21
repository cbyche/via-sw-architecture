// Authored fictional scenario fixtures for the shared DP2/DP5 illustration.
// These are not captured conversations, observed logs, or model test results.
// Performance assumptions and authored A/B outcomes belong to the caller.

const PROFILES = [
  {project:'오로라 고객센터 전환',topic:'고객지원 시스템',audience:'서비스 운영위원회',internal:'상담팀 실무회의',deliverable:'전환 검토 발표자료',count:5,short:3,limit:'월 운영비 240만원 이하',omit:'초기 설치비',period:'첫 12개월',focus:'야간 상담 배정',metric:'월 운영비',unit:'만원',values:[260,220,245],updated:235,positive:1,order:[0,1,2],newFact:'야간 상담 1,280건 중 자동 분류 864건',question:'다른 질문이야. TCP와 UDP는 연결 방식이 어떻게 달라? 기존 자료 작업은 계속해줘.',answer:'TCP는 연결을 설정한 뒤 순서와 전달을 관리하고, UDP는 연결 설정 없이 데이터그램을 보냅니다.'},
  {project:'봄길 지역전시 준비',topic:'지역전시 운영',audience:'참가업체 설명회',internal:'전시 운영회의',deliverable:'운영 제안 발표자료',count:4,short:2,limit:'주말 운영 인력 12명 이하',omit:'일회성 장식비',period:'전시 4주',focus:'혼잡 시간 입장 동선',metric:'주간 운영비',unit:'만원',values:[320,355,340],updated:345,positive:0,order:[2,0,1],newFact:'토요일 방문객 2,400명 중 오전 입장 960명',question:'별도 질문이야. SVG와 PNG는 확대할 때 어떤 차이가 나? 진행 중인 자료는 유지해줘.',answer:'SVG는 벡터 도형을 확대해 그리므로 선이 선명하고, PNG는 픽셀 이미지여서 크게 확대하면 픽셀이 드러날 수 있습니다.'},
  {project:'해온 신입교육 개편',topic:'신입직원 교육',audience:'교육 승인위원회',internal:'강사 준비회의',deliverable:'교육 개편 발표자료',count:6,short:4,limit:'참가자당 교육시간 18시간 이하',omit:'숙박비',period:'입사 후 3주',focus:'실습과 피드백 순서',metric:'인당 운영비',unit:'만원',values:[42,37,45],updated:40,positive:2,order:[1,2,0],newFact:'신입 36명 중 사전 실습 경험자는 9명',question:'이건 새 질문이야. 중앙값과 평균의 차이를 한 문장으로 알려줘. 교육 자료는 계속 만들어줘.',answer:'평균은 모든 값을 더해 개수로 나눈 값이고, 중앙값은 값을 크기순으로 정렬했을 때 가운데 위치한 값입니다.'},
  {project:'라임 배송거점 개선',topic:'배송거점 운영',audience:'협력사 운영설명회',internal:'배차 담당자 회의',deliverable:'거점 개선 발표자료',count:7,short:3,limit:'일일 추가 운행 8회 이하',omit:'차량 구매비',period:'평일 20일',focus:'오후 출고 대기',metric:'일 운영비',unit:'만원',values:[79,88,83],updated:84,positive:1,order:[1,0,2],newFact:'하루 평균 출고 870건 중 오후 집중 548건',question:'따로 궁금한데 CSV와 JSON은 구조를 표현하는 방식이 어떻게 달라? 배송 자료 작업은 계속해줘.',answer:'CSV는 행과 열의 표 형태를 표현하고, JSON은 이름과 값, 배열과 중첩 객체로 구조를 표현합니다.'},
  {project:'가온 사무실 조명교체',topic:'사무실 조명',audience:'입주사 설명회',internal:'시설 담당회의',deliverable:'조명 교체 검토자료',count:5,short:2,limit:'층별 정전 30분 이하',omit:'철거비',period:'교체 후 6개월',focus:'회의실 사용 중 공사',metric:'월 전력비',unit:'만원',values:[94,89,102],updated:93,positive:0,order:[2,1,0],newFact:'회의실 14개 중 일평균 6시간 이상 사용 공간은 8개',question:'독립 질문이야. 직렬 연결과 병렬 연결에서 전류의 경로는 어떻게 달라? 조명 자료는 계속해줘.',answer:'직렬 연결은 전류가 흐르는 경로가 하나이고, 병렬 연결은 여러 갈래의 경로로 전류가 나뉩니다.'},
  {project:'별빛 채용행사 준비',topic:'채용행사 운영',audience:'참여기업 안내회',internal:'채용 담당자 회의',deliverable:'행사 운영 발표자료',count:8,short:4,limit:'현장 대기 20분 이하',omit:'홍보물 인쇄비',period:'행사 2일',focus:'면접 슬롯 중복',metric:'일 운영비',unit:'만원',values:[168,156,162],updated:160,positive:2,order:[0,2,1],newFact:'사전 신청 420명 중 오후 면접 희망자는 273명',question:'작업과 별개로 HTTP에서 GET과 POST가 통상 어떤 용도로 쓰이는지 알려줘. 행사 자료는 유지해줘.',answer:'GET은 주로 자원 조회에, POST는 주로 데이터를 제출해 처리를 요청하는 데 사용됩니다.'},
  {project:'청솔 부품검사 개선',topic:'출하 전 부품검사',audience:'납품처 품질설명회',internal:'검사 공정회의',deliverable:'검사 개선 발표자료',count:6,short:3,limit:'샘플 검사 45분 이하',omit:'검사장비 구입비',period:'다음 8주',focus:'검사 누락 재확인',metric:'주간 운영비',unit:'만원',values:[115,121,108],updated:112,positive:1,order:[0,1,2],newFact:'지난 800개 표본 중 재확인 대상 24개',question:'새 질문인데 체크섬과 암호화는 목적이 어떻게 달라? 검사 자료 작업은 계속해줘.',answer:'체크섬은 데이터 변경이나 손상을 확인하는 데 쓰이고, 암호화는 권한 없는 사람이 내용을 읽기 어렵게 만드는 데 쓰입니다.'},
  {project:'노을 서비스 캠페인',topic:'서비스 안내 캠페인',audience:'지역 파트너 설명회',internal:'콘텐츠 편집회의',deliverable:'캠페인 계획 발표자료',count:4,short:2,limit:'안내문 500자 이하',omit:'일회성 촬영비',period:'운영 5주',focus:'첫 방문자의 신청 절차',metric:'주간 운영비',unit:'만원',values:[64,59,71],updated:63,positive:0,order:[1,0,2],newFact:'설문 240명 중 신청 단계를 어려워한 응답은 78명',question:'별도 질문으로 래스터 이미지와 벡터 이미지의 차이를 알려줘. 캠페인 작업은 계속해줘.',answer:'래스터 이미지는 픽셀로 이루어지고, 벡터 이미지는 점과 선, 도형의 수학적 정보로 이루어집니다.'},
  {project:'파도 창고배치 변경',topic:'창고 피킹 동선',audience:'배송 협력사 설명회',internal:'창고 운영회의',deliverable:'배치 변경 발표자료',count:7,short:4,limit:'주문당 이동 180m 이하',omit:'선반 구매비',period:'적용 후 4주',focus:'동시 피킹 교차 구간',metric:'일 운영비',unit:'만원',values:[137,149,142],updated:145,positive:2,order:[2,0,1],newFact:'측정된 것으로 설정한 300개 주문 중 교차 대기 주문은 96개',question:'작업은 계속하고 질문 하나만 답해줘. 스택과 큐에서 먼저 꺼내는 항목은 어떻게 달라?',answer:'스택은 가장 나중에 넣은 항목을 먼저 꺼내고, 큐는 가장 먼저 넣은 항목을 먼저 꺼냅니다.'},
  {project:'온유 사내포털 개편',topic:'사내포털 이용',audience:'부서대표 설명회',internal:'포털 운영회의',deliverable:'포털 개선 발표자료',count:5,short:3,limit:'첫 화면 메뉴 9개 이하',omit:'초기 이전비',period:'출시 후 2개월',focus:'반복 문의의 메뉴 접근',metric:'월 운영비',unit:'만원',values:[182,174,191],updated:178,positive:1,order:[1,2,0],newFact:'문의 560건 중 메뉴 위치 문의는 196건',question:'포털 작업과 무관한 질문이야. UTF-8에서 모든 문자가 같은 바이트 수를 쓰는지 알려줘. 기존 작업은 계속해줘.',answer:'UTF-8은 가변 길이 인코딩이므로 모든 문자가 같은 바이트 수를 쓰지 않으며, 한 문자를 1~4바이트로 표현합니다.'}
];

const FAMILY_SPECS = [
  ['F01','명시 ID와 조건 정정',['S02','S17'],'explicit','correction',false,'명시한 ID는 공통 사실 조회로 해결한다. 순차 정정은 DP2의 기다림 이익이 작고 관계 사전 생산도 필수 경로가 아니다.'],
  ['F02','과거 평가의 첫 재참조',['S11','S15'],'prepared','addition',true,'과거 표시와 피드백을 처음 연결한다. 관계 생산의 재사용 가능성과 진행 중 별도 요약의 실행 분리를 함께 살핀다.'],
  ['F03','비슷한 이름의 다른 업무 자료',['S13','S14'],'prepared','addition',true,'같은 제목의 다른 Task 자료를 구분해야 한다. 대상 연결 실패와 문맥 재사용을 별도 원인으로 남긴다.'],
  ['F04','실제로 본 판본과 최신 판본',['S12','S13'],'prepared','addition',false,'과거 평가가 붙은 v1과 아직 보지 않은 v2를 분리한다. 이후 최신 판본 조회는 새 목표다.'],
  ['F05','정정된 피드백과 무효 관계',['S08','S11'],'dirty','correction',true,'이전 평가를 바꾼 원문이 존재한다. B도 무효 관계를 버리고 원문을 보완해야 하며 A도 정정 이력을 읽는다.'],
  ['F06','과거 대상 유지와 현재 목적 변경',['S15','S17'],'prepared','correction',true,'자료 대상의 재사용과 현재 audience 및 설명 순서 변경을 구별한다. 대상이 맞아도 전체 요청 의미가 틀릴 수 있다.'],
  ['F07','평가 측면과 선호 적용 범위',['S27','S17'],'prepared','correction',false,'구성만 좋다는 평가와 이번 업무 한정 Agent 사용을 전역 선호로 확대하지 않는다.'],
  ['F08','공통으로 확인한 관계 재사용',['S11'],'confirmed','addition',false,'이미 사용자에게 확인된 대상은 A와 B에 같은 사실로 제공한다. A의 구조 고유 재사용을 제거하지 않는 동점 대조다.'],
  ['F09','방금 도착한 피드백의 미생산 관계',['S08','S17'],'unproduced','correction',true,'U01 후 도착한 새 피드백이 B의 생산 범위 밖에 있다. B도 현재 원문을 읽어야 하므로 준비된 관계의 이익을 전제하지 않는다.'],
  ['F10','형식과 내용 출처를 나눠 사용',['S11','S13','S15'],'prepared','addition',true,'과거 자료의 구성과 새 조사 내용의 출처를 각각 지칭한다. 과거 수치까지 복사하면 QA07의 조건 오류다.'],
  ['F11','조회 결과에 의존하는 대상 정정',['S02','S08','S17'],'explicit','correction',true,'후속 의도는 먼저 이해할 수 있지만 어느 견적을 채택할지는 진행 중인 비용 조회 결과가 필요하다.'],
  ['F12','자료 작업 중 독립 질문',['S01','S18'],'prepared','independent',true,'후속 질문은 자료 지칭이 없는 일반 지식이다. 원래 자료 작업을 보존하면서 별도 목표를 처리한다.']
];

const ordinal = n => ['첫 번째','두 번째','세 번째'][n];
const clone = value => JSON.parse(JSON.stringify(value));
const KNOWLEDGE_GOALS = ['TCP와 UDP의 연결 방식 비교','SVG와 PNG의 확대 특성 비교','중앙값과 평균의 차이 설명','CSV와 JSON의 구조 표현 비교','직렬 연결과 병렬 연결의 전류 경로 비교','HTTP GET과 POST의 통상 용도 설명','체크섬과 암호화의 목적 비교','래스터 이미지와 벡터 이미지 비교','스택과 큐의 항목 인출 순서 비교','UTF-8의 문자별 바이트 수 설명'];

/** Return authored scenarios and raw fictional sources, without A/B scores. */
export function buildScenarios() {
  const families = FAMILY_SPECS.map(([id,name,common_mapping,relation_state,second_kind,overlap,mechanism_note]) => ({id,name,common_mapping,relation_state,second_kind,overlap,mechanism_note}));
  const episodes = [], sources = [];

  for (const family of families) for (const [variant,p] of PROFILES.entries()) {
    const id = `SYN-${family.id}-${String(variant+1).padStart(2,'0')}`;
    const sid = suffix => `${id}-SRC-${suffix}`;
    const canonical = (n,version=1,extent='') => `${id}:D${n+1}@v${version}${extent ? `#${extent}` : ''}`;
    const good = p.positive, other = (good+1)%3, third = (good+2)%3;
    const selected = family.id==='F05' ? other : good;
    const selectedId=canonical(selected), selectedAlt=canonical(selected===good?other:good);
    const fresh=`${id}:N@v1`, freshAlt=`${id}:N@v0`;
    const sourceIds=[];
    const addSource = (suffix,kind,content,available_event=null) => {
      const source={id:sid(suffix),episode_id:id,kind,content:`[작성한 가상 원자료] ${content}`,...(available_event?{available_event}:{})};
      sources.push(source); sourceIds.push(source.id); return source.id;
    };
    const docIds=[];
    for (let n=0;n<3;n++) {
      const title=family.id==='F03' ? `${p.topic} 검토자료` : `${p.topic} ${['기본안','운영안','대안'][n]}${family.id==='F11'?' 견적서':''}`;
      docIds.push(addSource(`D${n+1}V1`,'document',`원본 ID ${canonical(n)}. 제목: ${title}. 소유 업무: ${p.project}의 ${n===other?'내부 검토':'고객 설명'} 업무 T${n+1}. 구성: ${n===good?'현황, 핵심 문제, 대안 비교, 실행 순서':n===other?'위험, 예산, 상세 조건, 참고표':'요약, 세부 기능, 일정, 부록'}. ${p.metric}: ${p.values[n]}${p.unit}, 별도 항목인 ${p.omit}는 이 금액에 포함되지 않는다. ${p.omit}: ${18+variant*3+n*4}만원. 적용 기간: ${p.period}. 일정 표 범위 ID ${canonical(n,1,'schedule')}: 준비 ${3+variant+n}일, 검토 ${2+(variant+n)%3}일, 적용 ${5+variant+n}일. 초점 항목 범위 ID ${canonical(n,1,'focus')}: ${p.focus}. 공통 제한: ${p.limit}.`));
    }
    addSource('LATEST','document',`원본 ID ${canonical(good,2)}. ${canonical(good)}의 수정 판본. 제목은 동일하며 ${p.metric}가 ${p.values[good]}${p.unit}에서 ${p.updated}${p.unit}로 변경됐다. ${p.focus}의 책임자 확인 항목이 추가됐다. 생성 순서는 모든 v1 표시와 과거 피드백 이후다. 최신 판본은 아직 사용자에게 표시되지 않았다.`);
    addSource('NEW','document',`원본 ID ${fresh}. 제목: ${p.project} 신규 조사. 조사 범위: ${p.period}. 본문: ${p.newFact}. ${p.focus}에 대한 후속 확인이 필요하다. 이전 조사 ${freshAlt}에는 같은 표본과 수치가 없으며 현재 자료는 이 판본으로 교체됐다. 구성표: 현황 수치, 세부 분포, 확인 질문. 이 수치는 시나리오를 위해 작성했으며 실제 조사 결과가 아니다.`);
    addSource('DISPLAY','presentation_record',`표시 순서는 ${p.order.map((n,k)=>`${ordinal(k)} ${canonical(n)}의 v1 전체 보기`).join(', ')}다. 각 표시는 서로 다른 완료 업무 카드 T${p.order[0]+1}, T${p.order[1]+1}, T${p.order[2]+1}에서 이루어졌다. ${canonical(good,2)}는 생성만 되었고 표시 이력이 없다. U01 이전에 다른 공개 응답이 이 순서를 바꾸지 않았다.`);
    const feedback = family.id==='F07'
      ? `${ordinal(p.order.indexOf(good))}로 보여준 자료는 ${p.audience}에 설명할 구성만 좋아. 가격이나 문체까지 좋다는 뜻은 아니야. 문서 Agent P${variant+1}는 이번 ${p.project} 작업에만 써줘.`
      : `${ordinal(p.order.indexOf(good))}로 보여준 자료는 ${p.audience}에서 설명하기 좋은 구성이야. ${ordinal(p.order.indexOf(other))} 자료는 ${p.internal} 검토에 두자.`;
    addSource('FEEDBACK1','user_feedback',`세 자료를 위 순서로 표시한 다음 사용자 발화: “${feedback}”`);
    if(family.id==='F05') addSource('FEEDBACK2','historical_user_correction',`과거 피드백을 정정한 원문: “정정할게. ${p.audience}에 쓰기 좋은 구성은 ${ordinal(p.order.indexOf(other))}로 보여준 쪽이야. 처음 칭찬했던 쪽을 쓰자는 말은 취소할게.” 초기 관계 생산은 t=-8초부터 t=0초 전에 끝났으며, 이 정정 원문은 생산이 끝난 뒤 U01 직전인 t=-0.1초에 허용된 대화 이력에서 동기화된다. 현재 평가 입력 U01/U02에 새 사용자 발화를 추가한 것은 아니다. 생산된 기존 관계는 이 정정을 반영하지 못한 dirty 상태다.`, 'U01_MINUS_0.1S_AFTER_INITIAL_RELATION_PRODUCTION');
    if(family.id==='F08') addSource('CONFIRM','user_confirmation',`과거 대화에서 VIA가 “구성이 좋다고 한 자료는 ${canonical(good)}인가요?”라고 물었고 사용자가 “맞아. ${canonical(good)}의 구성을 말한 거야.”라고 답했다. 이 확인 발화와 자료 ID가 공통 대화 기록에 보존돼 있다.`);
    addSource('TASK','task_snapshot',`U01 직전 ${p.project}의 새 ${p.deliverable}는 아직 시작하지 않았다. 다른 조사 업무 T-BG는 ${fresh}의 후속 자료를 정리 중이다. 문서 Agent P${variant+1}는 접속 가능하고 발표자료 생성 capability와 지정 자료 읽기 권한이 있다. 사용자에게 실제 전달된 과거 자료는 DISPLAY 기록을 따른다. 외부 발송이나 파일 공유는 이번 요청에 포함되지 않는다.`);

    const initialConditions=[`${p.audience}용`,`${p.count}장 이내`,`${p.limit}`,`대상 기간 ${p.period}`,'외부 발송 없음'];
    const commonGoal=`${p.project} ${p.deliverable} 작성 위임`;
    const mkRef=(inputNo,refNo,phrase,expected,alternative)=>({id:`${id}-U0${inputNo}-R0${refNo}`,phrase,expected:[].concat(expected),alternative:[].concat(alternative)});
    const mkInput=(n,{text,refs=[],goal=commonGoal,targets,conditions=initialConditions,data_source=fresh,first_response})=>({
      id:`${id}-U0${n}`,text,revision:n===2&&family.second_kind==='correction'?2:1,
      goal_id:`${id}-G0${n===2&&family.second_kind!=='correction'?2:1}`,
      kind:n===1?'initial':family.second_kind,referents:refs,
      handling:n===2&&family.second_kind!=='correction'?'VIA_DIRECT':'AGENT_DELEGATED',
      meaning:{goal,targets:[].concat(targets),conditions:[...conditions],prior_task_action:n===1?'new':family.second_kind==='correction'?'revise':'keep',data_source},
      first_response,source_ids:[]
    });
    const firstTask=(target=selectedId,extra='')=>`${target}의 구성을 기준으로 ${p.audience}용 ${p.deliverable}를 ${p.count}장 이내로 작성하도록 준비하겠습니다.${extra}`;
    let u1,u2;

    if(family.id==='F01') {
      u1=mkInput(1,{text:`${selectedId}를 기준으로 ${p.audience}용 ${p.deliverable}를 ${p.count}장 이내로 만들어줘. ${p.limit}, ${p.period} 기준이야. 외부 발송은 하지 마.`,refs:[mkRef(1,1,selectedId,selectedId,selectedAlt)],targets:[selectedId],data_source:selectedId,first_response:firstTask()});
      const conditions=[...initialConditions.filter(c=>c!==`${p.count}장 이내`),`${p.short}장 이내`,`${p.omit} 제외`];
      u2=mkInput(2,{text:`방금 요청은 같은 자료로 ${p.short}장 이내로 바꿔줘. ${p.omit}는 제외하고 나머지 조건은 유지해.`,refs:[mkRef(2,1,'같은 자료',selectedId,selectedAlt)],targets:[selectedId],conditions,data_source:selectedId,first_response:`${selectedId} 기준의 기존 요청을 ${p.short}장 이내와 ${p.omit} 제외 조건으로 수정하겠습니다.`});
    } else if(family.id==='F02') {
      u1=mkInput(1,{text:`예전에 ${p.audience}에서 설명하기 좋은 구성이라고 했던 자료로 ${p.deliverable}를 ${p.count}장 이내로 만들어줘. ${p.limit}, ${p.period} 기준으로 하고 외부 발송은 하지 마.`,refs:[mkRef(1,1,'설명하기 좋은 구성이라고 했던 자료',selectedId,selectedAlt)],targets:[selectedId],data_source:selectedId,first_response:firstTask()});
      u2=mkInput(2,{text:`발표자료 작성은 계속하고, 그 자료의 일정 표만 따로 ${p.short}문장으로 요약해줘. 준비와 적용 날짜를 구분해줘.`,refs:[mkRef(2,1,'그 자료의 일정 표',`${selectedId}#schedule`,`${selectedAlt}#schedule`)],goal:'원래 자료의 일정 표 별도 요약',targets:[`${selectedId}#schedule`],conditions:[`${p.short}문장`,'준비와 적용 일정 구분','기존 발표자료 작성 유지'],data_source:selectedId,first_response:`발표자료 작성은 유지하고 ${selectedId}의 일정 표를 준비와 적용 단계로 나누어 별도로 요약하겠습니다.`});
    } else if(family.id==='F03') {
      u1=mkInput(1,{text:`같은 제목인 자료가 여럿 있는데, 내가 ${p.audience}용 구성을 좋다고 했던 쪽으로 ${p.deliverable} ${p.count}장 이내를 만들어줘. ${p.limit}, ${p.period} 기준이야. 외부 발송은 하지 마.`,refs:[mkRef(1,1,'내가 '+p.audience+'용 구성을 좋다고 했던 쪽',selectedId,selectedAlt)],targets:[selectedId],data_source:selectedId,first_response:firstTask()});
      u2=mkInput(2,{text:`작성은 계속해. 그 고객 설명용 자료와 내부 검토에 두자고 했던 자료의 일정 차이만 ${p.short}문장으로 따로 알려줘.`,refs:[mkRef(2,1,'그 고객 설명용 자료',selectedId,selectedAlt),mkRef(2,2,'내부 검토에 두자고 했던 자료',canonical(other),canonical(third))],goal:'서로 다른 업무 자료의 일정 차이 설명',targets:[selectedId,canonical(other)],conditions:['일정 차이만',`${p.short}문장`,'기존 발표자료 작성 유지'],data_source:`${selectedId}; ${canonical(other)}`,first_response:`기존 작성은 유지하고 ${selectedId}와 ${canonical(other)}의 일정 차이를 별도로 정리하겠습니다.`});
    } else if(family.id==='F04') {
      u1=mkInput(1,{text:`내가 실제로 보고 ${p.audience}용 구성이 좋다고 했던 당시 판본으로 ${p.deliverable}를 ${p.count}장 이내로 만들어줘. 최신본으로 바꾸지 말고 ${p.limit}, ${p.period} 조건을 지켜줘. 외부 발송은 하지 마.`,refs:[mkRef(1,1,'실제로 보고 '+p.audience+'용 구성이 좋다고 했던 당시 판본',selectedId,canonical(good,2))],targets:[selectedId],conditions:[...initialConditions,'평가 당시 v1 유지'],data_source:selectedId,first_response:firstTask(selectedId,' 평가 당시 v1을 유지하겠습니다.')});
      u2=mkInput(2,{text:`기존 발표자료는 당시 판본으로 유지하고, 지금 최신 판본에서 달라진 ${p.metric}와 책임자 확인 항목만 따로 알려줘.`,refs:[mkRef(2,1,'지금 최신 판본',canonical(good,2),selectedId)],goal:'동일 자료 최신 판본의 변경사항 별도 조회',targets:[canonical(good,2)],conditions:[`${p.metric} 변경 확인`,'책임자 확인 항목 확인','기존 발표자료는 v1 유지'],data_source:canonical(good,2),first_response:`기존 발표자료는 ${selectedId}로 유지하고 최신 ${canonical(good,2)}의 비용 및 책임자 항목 변경을 별도로 확인하겠습니다.`});
    } else if(family.id==='F05') {
      u1=mkInput(1,{text:`나중에 정정해서 ${p.audience}용 구성이 좋다고 했던 자료로 ${p.deliverable}를 ${p.count}장 이내로 만들어줘. ${p.limit}, ${p.period} 조건이고 외부 발송은 하지 마.`,refs:[mkRef(1,1,'나중에 정정해서 '+p.audience+'용 구성이 좋다고 했던 자료',selectedId,selectedAlt)],targets:[selectedId],data_source:selectedId,first_response:firstTask()});
      u2=mkInput(2,{text:`그 자료 선택은 유지하고, ${p.focus} 항목을 첫 장에 놓아줘. ${p.omit}는 빼고 ${p.short}장 이내로 줄여줘.`,refs:[mkRef(2,1,'그 자료',selectedId,selectedAlt)],targets:[selectedId],conditions:[...initialConditions.filter(c=>c!==`${p.count}장 이내`),`${p.short}장 이내`,`${p.focus} 항목을 첫 장에 제시`,`${p.omit} 제외`],data_source:selectedId,first_response:`정정 후 선택한 ${selectedId}는 유지하고 ${p.focus} 항목을 먼저 보여주는 ${p.short}장 구성으로 바꾸겠습니다.`});
    } else if(family.id==='F06') {
      u1=mkInput(1,{text:`전에 ${p.audience}용으로 구성이 좋았던 자료를 써서 ${p.deliverable}를 ${p.count}장 이내로 만들어줘. ${p.limit}, ${p.period} 조건이고 외부 발송은 하지 마.`,refs:[mkRef(1,1,'전에 '+p.audience+'용으로 구성이 좋았던 자료',selectedId,selectedAlt)],targets:[selectedId],data_source:selectedId,first_response:firstTask()});
      u2=mkInput(2,{text:`같은 자료는 쓰되 이번에는 ${p.internal}용으로 바꿔. 장점보다 위험과 ${p.focus}의 미확인 사항을 먼저 보여줘. 나머지 조건은 유지해.`,refs:[mkRef(2,1,'같은 자료',selectedId,selectedAlt)],targets:[selectedId],conditions:[...initialConditions.filter(c=>c!==`${p.audience}용`),`${p.internal}용`,'위험 우선',`${p.focus}의 미확인 사항 우선`],data_source:selectedId,first_response:`${selectedId}는 유지하고 ${p.internal}용으로 목적을 바꾸어 위험과 미확인 사항을 먼저 정리하겠습니다.`});
    } else if(family.id==='F07') {
      u1=mkInput(1,{text:`내가 구성만 좋다고 했던 자료를 바탕으로 이번 ${p.deliverable}를 문서 Agent P${variant+1}에게 맡겨줘. ${p.audience}용 ${p.count}장 이내로, ${p.limit}, ${p.period} 기준이야. 외부 발송은 하지 마.`,refs:[mkRef(1,1,'구성만 좋다고 했던 자료',selectedId,selectedAlt)],targets:[selectedId],conditions:[...initialConditions,'구성만 재사용',`문서 Agent P${variant+1}는 이번 작업만`,'가격과 문체 선호를 계승하지 않음'],data_source:selectedId,first_response:`${selectedId}의 구성만 사용해 이번 ${p.deliverable}를 문서 Agent P${variant+1}에게 맡기겠습니다.`});
      u2=mkInput(2,{text:`그 자료의 가격이나 문체는 복사하지 마. 이번 답은 표를 ${p.short}개 이내로 쓰고, 장기 선호는 저장하지 말아줘. 발표 목적과 나머지 조건은 유지해.`,refs:[mkRef(2,1,'그 자료',selectedId,selectedAlt)],targets:[selectedId],conditions:[...u1.meaning.conditions,`표 ${p.short}개 이내`,'가격과 문체 복사 금지','장기 선호 저장 금지'],data_source:selectedId,first_response:`${selectedId}의 구성만 유지하고 가격과 문체는 복사하지 않으며, 표 ${p.short}개 이내의 이번 요청으로만 적용하겠습니다.`});
    } else if(family.id==='F08') {
      u1=mkInput(1,{text:`앞서 맞다고 확인한 자료의 구성으로 ${p.deliverable}를 ${p.count}장 이내로 만들어줘. ${p.audience}용이고 ${p.limit}, ${p.period} 기준이야. 외부 발송은 하지 마.`,refs:[mkRef(1,1,'앞서 맞다고 확인한 자료',selectedId,selectedAlt)],targets:[selectedId],data_source:selectedId,first_response:firstTask()});
      u2=mkInput(2,{text:`기존 작성은 유지하고 그 자료의 ${p.focus} 항목만 별도로 ${p.short}문장으로 설명해줘. 새 파일은 만들지 마.`,refs:[mkRef(2,1,'그 자료의 '+p.focus+' 항목',`${selectedId}#focus`,`${selectedAlt}#focus`)],goal:'확인한 자료의 초점 항목 별도 설명',targets:[`${selectedId}#focus`],conditions:[`${p.short}문장`,'새 파일 생성 없음','기존 발표자료 작성 유지'],data_source:selectedId,first_response:`기존 발표자료 작성은 유지하고 ${selectedId}의 ${p.focus} 항목을 별도 답변으로 설명하겠습니다.`});
    } else if(family.id==='F09') {
      const baseline=canonical(third), chosen=canonical(other);
      u1=mkInput(1,{text:`${baseline}를 기준으로 ${p.deliverable}를 ${p.count}장 이내로 만들어줘. ${p.audience}용이고 ${p.limit}, ${p.period} 기준이야. 외부 발송은 하지 마.`,refs:[mkRef(1,1,baseline,baseline,chosen)],targets:[baseline],data_source:baseline,first_response:firstTask(baseline)});
      addSource('FRESH-FEEDBACK','external_feedback_note_after_U01',`U01의 첫 구체적 안내 이후, U02 직전에 접근이 허용된 업무 T-BG의 검토 메모가 추가된다. 메모 원문: “공동 검토 의견: ${ordinal(p.order.indexOf(other))}로 보여준 쪽의 구성이 이번 ${p.audience}에 더 맞음.” 이 외부 원자료 갱신은 별도 사용자 발화나 VIA 표시 의무가 있는 Agent 통지가 아니다. 기존 관계 생산이 새 메모를 처리하기 전에 U02가 들어오는 가상 순서다.`, 'AFTER_U01_FIRST_CONTENT_BEFORE_U02');
      u2=mkInput(2,{text:`방금 추가된 검토 메모에서 이번 설명회에 더 맞는다고 한 쪽으로 자료를 바꿔줘. ${p.omit}는 빼고 나머지 작성 조건은 유지해.`,refs:[mkRef(2,1,'방금 추가된 검토 메모에서 이번 설명회에 더 맞는다고 한 쪽',chosen,baseline)],targets:[chosen],conditions:[...initialConditions,`${p.omit} 제외`],data_source:chosen,first_response:`방금 추가된 검토 메모가 평가한 ${chosen}로 기존 작성 대상을 바꾸고 ${p.omit} 제외 조건을 적용하겠습니다.`});
    } else if(family.id==='F10') {
      u1=mkInput(1,{text:`구성이 좋다고 했던 자료의 형식만 빌리고 신규 조사 ${fresh}로 내용을 바꿔 ${p.deliverable}를 만들어줘. ${p.audience}용 ${p.count}장 이내, ${p.limit}, ${p.period} 기준이야. 과거 수치와 ${p.omit}는 넣지 말고 외부 발송은 하지 마.`,refs:[mkRef(1,1,'구성이 좋다고 했던 자료',selectedId,selectedAlt),mkRef(1,2,`신규 조사 ${fresh}`,fresh,freshAlt)],targets:[selectedId,fresh],conditions:[...initialConditions,'과거 자료 형식만 재사용','과거 수치 복사 금지',`${p.omit} 제외`],data_source:fresh,first_response:`${selectedId}의 구성만 활용하고 내용은 ${fresh}로 교체한 ${p.count}장 이내 자료를 준비하겠습니다.`});
      u2=mkInput(2,{text:`발표자료는 계속 만들고, 그 신규 조사의 현황 수치와 확인 질문만 ${p.short}문장으로 따로 알려줘. 옛 자료 내용과 섞지 마.`,refs:[mkRef(2,1,'그 신규 조사',fresh,selectedId)],goal:'신규 조사의 수치와 확인 질문 별도 요약',targets:[fresh],conditions:[`${p.short}문장`,'현황 수치와 확인 질문만','과거 자료 내용 혼합 금지','기존 발표자료 작성 유지'],data_source:fresh,first_response:`기존 작성은 유지하고 ${fresh}의 현황 수치와 확인 질문만 별도로 요약하겠습니다.`});
    } else if(family.id==='F11') {
      const left=variant%3,right=(left+1)%3,win=p.values[left]<p.values[right]?left:right,lose=win===left?right:left;
      u1=mkInput(1,{text:`${canonical(left)}와 ${canonical(right)} 두 견적의 ${p.metric}와 ${p.omit}를 조회해서 비교 발표자료를 만들어줘. ${p.audience}용 ${p.count}장 이내, ${p.limit}, ${p.period} 기준이고 외부 발송은 하지 마.`,refs:[mkRef(1,1,'두 견적',[canonical(left),canonical(right)],[canonical(left),canonical((left+2)%3)])],targets:[canonical(left),canonical(right)],data_source:`${canonical(left)}; ${canonical(right)}`,first_response:`${canonical(left)}와 ${canonical(right)}의 비용 내역을 조회해 ${p.audience}용 비교자료를 준비하겠습니다.`});
      u2=mkInput(2,{text:`두 개를 모두 쓰지 말고 조회 결과에서 ${p.omit}를 제외한 ${p.metric}가 더 낮은 쪽 하나만 써줘. ${p.short}장 이내로 하고 나머지 조건은 유지해.`,refs:[mkRef(2,1,p.metric+'가 더 낮은 쪽',canonical(win),canonical(lose))],targets:[canonical(win)],conditions:[...initialConditions.filter(c=>c!==`${p.count}장 이내`),`${p.short}장 이내`,`${p.omit} 제외`,`${p.metric}가 낮은 하나만 선택`],data_source:canonical(win),first_response:`두 견적의 조회 결과에서 ${p.omit}를 제외한 ${p.metric}가 낮은 하나를 선택해 ${p.short}장 이내로 수정하겠습니다.`});
    } else {
      u1=mkInput(1,{text:`${p.audience}에서 설명하기 좋은 구성이라고 했던 자료로 ${p.deliverable}를 ${p.count}장 이내로 만들어줘. ${p.limit}, ${p.period} 기준이고 외부 발송은 하지 마.`,refs:[mkRef(1,1,p.audience+'에서 설명하기 좋은 구성이라고 했던 자료',selectedId,selectedAlt)],targets:[selectedId],data_source:selectedId,first_response:firstTask()});
      u2=mkInput(2,{text:p.question,goal:KNOWLEDGE_GOALS[variant],targets:[],conditions:['일반 지식 직접 답변',...(p.question.includes('한 문장')?['한 문장']:[]),'기존 발표자료 작성 유지'],data_source:'공통 일반 지식',first_response:p.answer});
      addSource('KNOWLEDGE','authored_common_knowledge_note',`이 가상 묶음에서 제공하는 공통 일반 지식 참고문: ${p.answer}`);
    }

    const rawInitialIds=sourceIds.filter(x=>x!==sid('FRESH-FEEDBACK')&&x!==sid('KNOWLEDGE'));
    const input1Source=addSource('INPUT-U01','user_input',u1.text,'U01_SUBMIT');
    const input2Source=addSource('INPUT-U02','user_input_after_U01',u2.text,'U02_SUBMIT');
    u1.source_ids=[...rawInitialIds,input1Source];
    u2.source_ids=[...sourceIds];
    const timing=family.overlap
      ? 'U02는 U01의 첫 구체적 안내 표시 뒤 0.35초에 들어오는 가상 순서다. U01은 자료 ID와 업무 의도를 채택했지만 지정 원문의 본문 읽기와 세부 준비가 진행 중이다. 원문이 데이터셋에 존재한다는 사실은 후보가 이미 읽었다는 뜻이 아니다.'
      : 'U02는 U01의 목표 완료 뒤 1.5초에 들어오는 가상 순서다. 정정이면 같은 목표의 새 revision으로 재개하고, 추가 요청이면 기존 완료 결과를 유지한 새 목표다.';
    const special=family.id==='F11'
      ? ' F11의 U01은 견적 두 개의 identity와 조회 의도만 먼저 채택한다. metric 값은 진행 중인 원문 조회 반환에서 읽는다. U02의 첫 안내는 비교 규칙을 채택한 의도이며 더 낮은 견적 ID의 최초 확정은 조회 반환 이후다. oracle의 win ID를 사전 입력이나 저장 관계로 공급하지 않는다.'
      : family.id==='F09'
      ? ' FRESH-FEEDBACK은 접근이 허용된 외부 업무 자료의 새 검토 메모다. U01에는 보이지 않고 U02에만 보이며, 사용자 입력 두 개 외의 미채점 발화를 넣지 않는다. 새 메모의 관계는 아직 사전 생산되지 않았으며 이전 선호 대상과 다르다.'
      : family.id==='F08'
      ? ' CONFIRM 원문과 확인된 identity는 양안에 공통 제공한다. 이 사례를 위해 다른 episode의 A/B 오답을 정답으로 덮어쓰지 않는다.'
      : '';
    const history=`${p.project}의 서로 다른 자료 세 개를 ${p.order.map(n=>`D${n+1}`).join('→')} 순서로 표시하고 피드백을 받은 가상 이력이다. ${family.mechanism_note}`;
    episodes.push({id,family_id:family.id,variant,project:p.project,history,source_ids:[...sourceIds],inputs:[u1,u2],evidence_notes:`모든 이력, 원자료와 입력은 작성한 가상 사례다. COMMON 매핑은 주제 대응이며 기존 COMMON-28 실행이나 기능 coverage를 뜻하지 않는다. ${timing}${special}`});
  }
  return clone({families,episodes,sources});
}
