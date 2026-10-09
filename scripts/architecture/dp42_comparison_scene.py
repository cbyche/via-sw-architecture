"""Focused DP42: shared lifetimes, different state authorities and handoffs.

This scene omits unrelated common interpretation/model internals. The full MAIN
retains those collaborators. Examples describe local authority, never external
Agent acceptance or actual delivery based only on a database commit.
"""
from dp_comparison_structures import ResolutionGraph, COMMON, DIFFERENCE_STROKE, BOUNDARY_WIDTH
from dp41_comparison_scene import document
from dp44_comparison_scene import legend as execution_legend


def legend(s):
    before=len(s.items)
    execution_legend(s)
    # Keep the same Component/Module/data/state/arrow key; replace the last
    # caption with the process/commit boundaries that are DP42's comparison.
    s.items=s.items[:before]+[i for i in s.items[before:]
                            if not(i.get('x',200)<140 and i.get('y',0)>=725)]
    for xx,dashed,label in [(54,False,'A'),(93,True,'B')]:
        s.rect(xx,727,29,18,'white',DIFFERENCE_STROKE,dashed=dashed)
        s.items[-1]['line_width']=BOUNDARY_WIDTH
        s.text(xx+14.5,729,25,[label],10,COMMON,align='center')
    s.label(54,751,76,'실행 / 저장\n확정 경계',10,COMMON)
    for index,item in enumerate(s.items[before:]):
        item['id']=f'legend42_{index}'
        if item.get('lines')==['State']:item['y']=571


def _state(g,key,x,y,w,h,name,size=12):
    before=len(g.s.items)
    g.store(key,x,y,w,h,name,size=size)
    g.s.items[before]['store_radius']=4
    # Short owner-state cylinders require centered text rather than the
    # legacy store caption's fixed +17px inset.
    g.s.items[-1]['y']=y+(h-size*1.2)/2


def _owner(g,key,x,y,w,h,name):
    g.group(x,y,w,h,name)
    # Edges enter the owner's heading/boundary; the state remains inside it.
    g.nodes[key]=(x,y,w,32)


def draw(s,x,side):
    g=ResolutionGraph(s,x)
    if side==0:legend(s)
    g.box('interaction',35,253,270,32,'Interaction Manager',size=18)
    document(g,'input',340,253,490,32,
             ['Input1: “응, 상반기로 해줘”'],15)
    g.edge('interaction','input','R','L')
    g.text(35,293,790,'공통 장면: 보고서 기간 질문 Question1을 Publication1로 실제 제시',size=14)
    # Layer I: logical lifetimes are common, independent of either process cut.
    for yy,title,end,detail in [
        (318,'Conversation1',824,'대화 연결 유지'),
        (337,'Request1',610,'Agent 답변 접수 확인 뒤 요청 처리 종료'),
        (356,'Task1',824,'보고서 업무 / 외부 실행은 계속')]:
        g.text(35,yy,145,title,size=13)
        g.raw([(186,yy+14),(end,yy+14)],arrow=False)
        g.text(195,yy-1,625,detail,size=12)

    # Layer II: same owners; A joins local changes, B gives each service its
    # own state authority, transaction and execution lifetime.
    if side==0:
        g.group(25,382,810,254,'VIA Core',execution=True,side=side)
        g.text(390,393,422,'모듈형 Core / 관련 변경 공동 확정',size=14)
    else:
        g.group(25,382,340,254,'대화 서비스',execution=True,side=side)
        g.group(470,382,365,254,'업무 서비스',execution=True,side=side)

    _owner(g,'controller',40,425,310,90,'Request Controller')
    _state(g,'request_state',50,479,290,25,'Request Graph State',12)
    _owner(g,'task',485,425,335,117,'Task Manager')
    g.box('gate',500,462,305,22,'업무 접수·전송 게이트',side==1,True,size=14)
    _state(g,'task_state',495,498,315,27,'Task Command State',12)
    _owner(g,'response',40,529,310,60,'Response Manager')
    _state(g,'publication',50,561,290,26,'Publication Ledger',10)
    g.box('gateway',485,553,335,30,'Agent Gateway',size=18)
    g.text(501,529,310,'Task1: 보고서 / Command1: 상반기 답변',size=11)
    # Input triggers the owner, not a pre-resolved Task. The left route keeps
    # the common lifespan strip free of operational crossings.
    g.edge('interaction','controller','L','L',via=[(25,269),(25,441)])
    if side==0:
        g.group(385,591,425,37,'State Store')
        # The Unit-of-Work is visibly inside its technical owner.
        g.box('commit',605,596,190,25,'트랜잭션 관리자',True,True,size=14)
        g.edge('controller','commit','R','T',sd=18,td=-40,
               via=[(370,459),(370,587),(660,587)])
        g.edge('task','commit','L','T',sd=18,
               via=[(474,459),(474,584),(700,584)])
        g.edge('commit','controller','L','R',td=12,
               via=[(596,608.5),(596,632),(374,632),(374,453)],ret=True)
        g.edge('commit','task','R','R',td=12,
               via=[(826,608.5),(826,453)],ret=True)
        g.text(370,461,108,'검증한 변경',size=10)
        g.text(370,485,108,'같은 commit',size=10)
    else:
        g.box('dialogue_store',40,595,310,31,'State Store',size=18)
        g.box('work_store',485,595,335,31,'State Store',size=18)
        g.edge('controller','task','R','L')
        g.text(368,421,110,'Command1 →',size=10)
        g.edge('task','controller','L','R',sd=18,td=18,
               via=[(474,459),(474,519),(363,519),(363,459)],ret=True)
        g.text(369,482,102,'← 접수 결과',size=10)
        g.edge('controller','dialogue_store','R','T',sd=10,
               via=[(358,451),(358,589),(195,589)])
        g.edge('task','work_store','R','T',sd=10,
               via=[(826,451),(826,589),(652.5,589)])
        g.text(46,627,299,'대화 원본 / 독립 local transaction',size=10)
        g.text(491,627,327,'업무 원본 / 독립 local transaction',size=10)
    g.edge('gate','gateway','R','T',via=[(814,473),(814,547),(652.5,547)])

    # Layer III: actual exchanged/committed data. Folded notes are explicitly
    # data, rather than new Component/Module boxes or transaction actions.
    document(g,'exchange',35,650,790,80,
             (['공동 변경 집합: Request1→Question1→Task1 답변 연결 + 질문 변경 + Command1 준비',
               'Question1: OPEN → ANSWERED / Command1: PENDING',
               'commit 성공: 관련 변경 모두 반영 / 충돌 / 실패: 모두 미반영',
               '그 뒤 Agent 전송 / 외부 접수 확인 / 실제 응답 전달은 별도 사건'] if side==0 else
              ['대화 tx: Request1→Question1→Task1 / 접수 대기 / Command1 의도 저장',
               '업무 tx: Question1=ANSWERED / Command1=PENDING / 접수 결과 저장',
               'AcceptanceReceipt{command_id:Command1, acceptance_status:ACCEPTED} → 대화 반영',
               'ACCEPTED는 VIA 내부 접수 / 외부 Agent 접수 / 실제 전달은 별도 확인']),12)
    g.raw([(25,739),(835,739)],arrow=False)
    g.text(35,745,790,('보조 제어: hold와 전송 CAS의 선후를 같은 로컬 확정 순서로 판정' if side==0 else
                           '보조 제어: hold 요청 → 업무 gate 적용 ACK → 현재 의미로 reconciliation'),size=13)
    g.text(35,768,790,'DISPATCHING 뒤 정정: 실제 외부 상태 조회 / 제어 / 발화 시작만으로 취소 완료 아님',size=12)
    validate(g,side)
    g.finish()
    return g


def validate(g,side):
    for item in g.s.items[g.first:]:
        if item['kind']!='line':continue
        points=[(px-g.x,py) for px,py in item['points']]
        for a,b in zip(points,points[1:]):
            assert a[0]==b[0] or a[1]==b[1],('Non-orthogonal',side,a,b)
            for key,(nx,ny,nw,nh) in g.nodes.items():
                if any(nx<=px<=nx+nw and ny<=py<=ny+nh for px,py in (points[0],points[-1])):continue
                vertical=a[0]==b[0] and nx+1<a[0]<nx+nw-1 and max(min(a[1],b[1]),ny+1)<min(max(a[1],b[1]),ny+nh-1)
                horizontal=a[1]==b[1] and ny+1<a[1]<ny+nh-1 and max(min(a[0],b[0]),nx+1)<min(max(a[0],b[0]),nx+nw-1)
                assert not(vertical or horizontal),('Unrelated node crossing',side,key,a,b)
    for key,(ox,oy,ow,oh) in {
        'request_state':(40,425,310,90),'task_state':(485,425,335,117),
        'gate':(485,425,335,117),'publication':(40,529,310,60),
        **({'commit':(385,591,425,37)} if side==0 else {})}.items():
        nx,ny,nw,nh=g.nodes[key]
        assert ox<=nx and oy<=ny and nx+nw<=ox+ow and ny+nh<=oy+oh,('Owner containment',side,key)
    text=' '.join(' '.join(i.get('lines',[])) for i in g.s.items[g.first:])
    assert all(k in text for k in ('Conversation1','Request1','Task1','Command1','Question1','Publication1'))
    assert 'Omni' not in text
