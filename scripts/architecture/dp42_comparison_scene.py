"""Focused DP42: shared lifetimes, different state authorities and handoffs.

This scene omits unrelated common interpretation/model internals. The full MAIN
retains those collaborators. Examples describe local authority, never external
Agent acceptance or actual delivery based only on a database commit.
"""
from dp_comparison_structures import ResolutionGraph, COMMON, DIFFERENCE_STROKE, BOUNDARY_WIDTH
from dp41_comparison_scene import FeedbackGraph, document, step
from dp44_comparison_scene import v3_legend as execution_legend


def legend(s):
    execution_legend(s)
    # This additional key is an execution/commit boundary, never a Component.
    for xx,dashed,label in [(54,False,'A'),(93,True,'B')]:
        s.rect(xx,286,29,18,'white',DIFFERENCE_STROKE,dashed=dashed)
        s.items[-1]['line_width']=1.5
        s.text(xx+14.5,288,25,[label],10,COMMON,align='center')
    s.label(54,311,76,'실행 / 저장\n확정 경계',10,COMMON)


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
    g=FeedbackGraph(s,x)
    if side==0:legend(s)
    g.box('interaction',35,253,270,32,'Interaction Manager',size=18)
    document(g,'input',340,253,490,32,
             ['Input1: “응, 상반기로 해줘”'],15)
    g.edge('interaction','input','R','L')
    g.text(35,289,790,'공통 장면: 보고서 기간 질문 Question1을 Publication1로 실제 제시',size=13)
    # Layer I: logical lifetimes are common, independent of either process cut.
    for yy,title,end,detail in [
        (309,'Conversation1',824,'대화 연결 유지'),
        (327,'Request1',610,'이번 답변의 외부 접수 확인 후 처리 종료'),
        (345,'Task1',824,'보고서 Task는 계속 / 대화와 별도 수명'),
        (363,'Execution1',824,'Downstream Agent의 실제 실행은 별도 수명')]:
        g.text(35,yy,145,title,size=12)
        g.raw([(186,yy+13),(end,yy+13)],arrow=False)
        g.text(195,yy-1,625,detail,size=11)

    # Layer II: same owners; A joins local changes, B gives each service its
    # own state authority, transaction and execution lifetime.
    if side==0:
        g.group(25,382,810,254,'VIA Core 실행 경계',execution=True,side=side)
        g.text(390,393,422,'모듈형 Core / 관련 변경의 한 번 확정',size=14)
    else:
        g.group(25,382,340,254,'대화 서비스 실행 경계',execution=True,side=side)
        g.group(470,382,365,254,'Task 서비스 실행 경계',execution=True,side=side)

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
        g.text(491,627,327,'Task 원본 / 독립 local transaction',size=10)
    g.edge('gate','gateway','R','T',via=[(814,473),(814,547),(652.5,547)])

    # Layer III: concrete validated changes and successive durable facts.
    # These folded notes are data examples, outside Architecture owner boxes.
    if side==0:
        document(g,'request_changes',35,650,340,68,[
            '공동 변경 1 / Request Controller의 관계',
            'Request1 → Question1 → Task1',
            '사용자 답변: 상반기',
            'Command1과 연결 / 관련 변경과 함께 확정'],12)
        document(g,'task_changes',455,650,370,68,[
            '공동 변경 2 / Task Manager의 상태',
            'Question1: OPEN → ANSWERED',
            'Command1: PENDING / payload.answer:상반기',
            '성공: 모두 반영 / 충돌과 실패: 모두 미반영'],12)
    else:
        document(g,'intent',35,650,240,75,[
            '① 대화 저장 / 접수 대기',
            'command_id:Command1 / request_id:Request1',
            'task_id:Task1 / question_id:Question1',
            'payload.answer:상반기',
            'expected_question_revision:7'],10.5)
        document(g,'acceptance',320,650,240,75,[
            '② Task 상태 확정 / 내부 접수',
            'Question1: ANSWERED',
            'Command1: PENDING',
            'acceptance_status:ACCEPTED',
            '확정 접수 결과도 함께 저장'],11)
        document(g,'receipt',595,650,230,75,[
            '③ 대화에 접수 결과 반영',
            'AcceptanceReceipt (필드 축약)',
            'command_id:Command1',
            'acceptance_status:ACCEPTED',
            '같은 Command1의 내부 접수 확인'],10.5)
        g.edge('intent','acceptance','R','L',color=DIFFERENCE_STROKE)
        g.edge('acceptance','receipt','R','L',color=DIFFERENCE_STROKE)
    trace(g,side)
    validate(g,side)
    g.finish()
    return g


def trace(g,side):
    g.raw([(25,727),(835,727)],arrow=False)
    g.text(35,731,790,'실행 순서도 / 같은 상반기 답변의 로컬 확정',size=14)
    labels=(['RC / TM: 관련 변경 준비','트랜잭션 관리자: 공동 확정','Gateway: 확정 명령 전송'] if side==0 else
            ['대화: 의도 저장','Task: 접수 상태 확정','대화: 접수 receipt 반영'])
    for i,(xx,label) in enumerate(zip([35,315,595],labels)):
        step(g,'trace'+str(i),xx,755,235 if i<2 else 230,27,label)
        if i:g.edge('trace'+str(i-1),'trace'+str(i),'R','L',color=DIFFERENCE_STROKE)
    g.text(35,783,790,('충돌과 실패: 함께 미반영 / 모델과 외부 통신은 transaction 밖' if side==0 else
                            'Task 접수 후 외부 전송 가능 / 대화 반영은 별도 / 접수 대기는 같은 Command1로 확인'),size=10)


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
    assert all(k in text for k in ('Conversation1','Request1','Task1','Command1','Question1','Publication1','Execution1'))
    assert 'Omni' not in text
