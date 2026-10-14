"""Implementation-level DP47 contracts and flows, not candidate runtime code."""
from __future__ import annotations
import heapq
from dp47_detail_scene import GroundingScene, INK, BLUE, RED, GRAY


def connect(p, source, target, sp='R', tp='L', kind='post', sd=0, td=0):
    """Orthogonal route through the actual diagram's free space; nodes never move."""
    # Independent request / completion wires have separate, bound ports.
    used=getattr(p,'used_ports',{});p.used_ports=used
    def offset(key,side,explicit):
        count=used.get((key,side),0);used[(key,side)]=count+1
        return explicit + (0 if count==0 else ((count+1)//2)*8*(1 if count%2 else -1))
    sd=offset(source,sp,sd);td=offset(target,tp,td)
    start=p.port(source,sp,sd); end=p.port(target,tp,td)
    exempt=p.ancestors(source)|p.ancestors(target)
    blocks=[n for key,n in p.nodes.items() if key not in exempt and n['kind'] not in ('component','group')]
    def clear(a,b):
        for n in blocks:
            x,y,w,h=(n[k] for k in ('x','y','w','h'))
            pad=0 if n['id'] in (source,target) else 8
            if a[0]==b[0]: hit=x-pad<a[0]<x+w+pad and max(min(a[1],b[1]),y-pad)<min(max(a[1],b[1]),y+h+pad)
            else: hit=y-pad<a[1]<y+h+pad and max(min(a[0],b[0]),x-pad)<min(max(a[0],b[0]),x+w+pad)
            if hit:return False
        return True
    xs=sorted({start[0],end[0],35,900,2940,3805}|{v for n in p.nodes.values() for v in (n['x']-24,n['x']+n['w']+24) if 20<=v<=3820})
    ys=sorted({start[1],end[1],210,1270,1290,1915,2025}|{v for n in p.nodes.values() for v in (n['y']-24,n['y']+n['h']+24) if 205<=v<=2030})
    initial=(xs.index(start[0]),ys.index(start[1]),'');goal=(xs.index(end[0]),ys.index(end[1]))
    cost={initial:0};previous={};q=[(0,initial)];finish=None
    while q:
        value,state=heapq.heappop(q)
        if value!=cost[state]:continue
        xi,yi,direction=state
        if (xi,yi)==goal:finish=state;break
        for dx,dy,newdir in ((1,0,'h'),(-1,0,'h'),(0,1,'v'),(0,-1,'v')):
            xx,yy=xi+dx,yi+dy
            if not(0<=xx<len(xs) and 0<=yy<len(ys)):continue
            a,b=(xs[xi],ys[yi]),(xs[xx],ys[yy])
            outward={'R':(1,0),'L':(-1,0),'T':(0,-1),'B':(0,1)}
            if a==start and (dx,dy)!=outward[sp]:continue
            if b==end and (-dx,-dy)!=outward[tp]:continue
            if not clear(a,b):continue
            # Separating parallel wires makes independent contracts traceable.
            overlap=0
            for wire in p.edges:
                for c,d in zip(wire['points'],wire['points'][1:]):
                    if a[0]==b[0]==c[0]==d[0] and max(min(a[1],b[1]),min(c[1],d[1]))<min(max(a[1],b[1]),max(c[1],d[1])):overlap+=300
                    if a[1]==b[1]==c[1]==d[1] and max(min(a[0],b[0]),min(c[0],d[0]))<min(max(a[0],b[0]),max(c[0],d[0])):overlap+=300
            nv=value+abs(a[0]-b[0])+abs(a[1]-b[1])+(90 if direction and direction!=newdir else 0)+overlap
            ns=(xx,yy,newdir)
            if nv<cost.get(ns,float('inf')):cost[ns]=nv;previous[ns]=state;heapq.heappush(q,(nv,ns))
    assert finish,(source,target,'no route')
    points=[];s=finish
    while s!=initial:points.append((xs[s[0]],ys[s[1]]));s=previous[s]
    points.append(start);points.reverse();simple=[]
    for pt in points:
        if len(simple)>1 and ((simple[-2][0]==simple[-1][0]==pt[0]) or (simple[-2][1]==simple[-1][1]==pt[1])):simple.pop()
        simple.append(pt)
    p.edge(source,target,sp,tp,via=simple[1:-1],kind=kind,sd=sd,td=td)


def common(side):
    title = ('A / 하나의 공동 해석 workflow가 원본에서 binding을 직접 갱신한다' if side == 'A' else
             'B / 두 생산자의 후보를 저장하고, coordinator가 준비된 후보의 결합을 지시한다')
    p = GroundingScene('dp47-implementation-' + side.lower(), title,
        '1 입력 등록 → 2 작업 전달 → 3 원본 조회 → 4 의미 처리 → 5 조건부 게시 → 6 다음 실행 → 7 제안 → 8 채택')
    p.text(60, 225, ('STYLE  Layered application + asynchronous workflow / Ports & Adapters / 단일 binding aggregate' if side == 'A' else
                    'STYLE  Orchestrated dataflow + Fork–Join / Producer–Consumer / 생산자별 versioned repository'), 29, BLUE, True)
    p.node('im',60,285,690,925,'Interaction Manager',kind='component')
    p.node('audio',90,360,295,140,'Channel I/O',[
        'WASAPI audio chunks', '로컬 VAD는 계속 실행'],owner='im')
    p.node('capture',415,360,305,140,'Evidence Capture',[
        'WGC / Raw Input / UIA', 'frame + path + 선택'],owner='im')
    p.node('observation_queue',100,545,610,115,'Observation Queue',[
        'bounded RAM FIFO / refs + source clock / TryWrite'],kind='queue',owner='im')
    p.node('timeline',100,715,610,95,'Timeline & Buffer',[
        '단일 recorder: append + 시간 index + gap'],owner='im')
    p.node('raw',100,865,610,155,'Raw Evidence Window',[
        'RAM: 유한 chunk ring + timestamp index', 'audio/frame bytes + 관측 record / TTL', '원본 refs만 전달 / pin도 전체 한도 적용'],kind='memory',owner='im')
    p.node('evidence_port',100,1040,610,125,'Evidence Read Port',[
        'ReadEvidence: lease/policy 확인 → receipt', 'refs + coverage + sourceVector + gap'],owner='im')
    p.node('rc',60,1330,690,460,'Request Controller',kind='component')
    p.node('input_control',100,1410,610,135,'Input Revision Control',[
        '입력 r2 등록 + hold / Input Revision State', 'BeginRevision(r2) → GroundingInput 전달'],owner='rc')
    p.node('adoption',100,1640,610,110,'Adoption Control',[
        '8 current r / base / source / policy 재검사'],owner='rc')
    p.node('db_owner',60,1850,690,210,'State Store',kind='component')
    p.node('adopted',100,1920,610,115,'Adopted Binding Records',[
        '영속 DB / repository transaction + artifact refs'],kind='db',owner='db_owner')
    p.node('ri',850,285,2110,1775,'Request Interpreter / local in-process async tasks',kind='component')
    p.node('entry',900,375,500,150,'Grounding Session Port',[
        '1 BeginRevision: gate 아래 fence 갱신', 'GroundingInput = refs + prior + budget', '실행 중 job cancel은 best effort'],owner='ri')
    p.node('inbox',900,610,500,140,'Control Inbox',[
        '2 bounded FIFO / revision별 입력 DTO', '단일 consumer / media bytes 없음'],kind='queue',owner='ri')
    p.node('reader',1530,375,600,150,'R / Grounding Evidence Reader',[
        '3 async read port / snapshot + bounded crop', '원본과 허용 Context를 조회하는 adapter', 'request별 cache / 의미 후보를 만들지 않음'],owner='ri')
    p.node('model_port',2260,375,610,150,'M / Grounding Model Port',[
        '4 async request/reply / role + jobId + r', 'media materialize → await result DTO', '호출한 worker로 직접 반환 / message bus 아님'],owner='ri')
    p.node('completion',900,1120,500,140,'Completion Inbox',[
        '5 bounded FIFO / jobId + role + outcome', '게시 완료, 실패, 취소를 해당 제어로'],kind='queue',owner='ri')
    p.node('jobs',900,1410,500,200,'Grounding Attempt Registry',[
        'RAM Map<requestId, Attempt>', 'expected r / generation / job / budget', 'RevisionGate: 요청별 짧은 lock', 'BeginRevision와 TryPublish가 같은 gate'],kind='memory',owner='ri')
    p.node('proposal',900,1860,1970,160,'GroundingProposal / immutable result DTO',[
        '7 requestId + inputRev + baseBindingVersion + sourceVector + policyEpoch + binding delta + unresolved',
        '예: KEEP 매출 범위, PDF 표 / REPLACE graph 집합 → 아래 graph / 기호는 해석 후 생산된 결과',
        '채택 뒤 전체 요청 해석과 admission → Response Manager 또는 Task Manager / grounding만으로 업무 실행하지 않음'],kind='data',owner='ri')
    p.node('ma',3060,285,720,890,'Model Access / shared adapter',kind='component')
    p.node('voice',3110,365,620,100,'Voice API Client',[
        'audio 전송 / transcript revision 반환'],owner='ma')
    p.node('semantic',3110,555,620,130,'Semantic API Client',[
        'async Task / role별 요청과 결과 correlation', '동일 model capability / bounded call slots'],owner='ma')
    p.node('export_gate',3110,785,620,110,'Model Use Gate',[
        '외부 전송 직전 lease / policyEpoch 확인'],owner='ma')
    p.node('calls',3110,995,620,120,'Model Call Registry',[
        'RAM Map<jobId, role / r / cancel / usage>'],kind='memory',owner='ma')
    p.node('cloud',3110,1250,620,100,'클라우드 모델 API',[
        'voice / semantic / 실제 동시성은 provider 조건'],kind='external')
    p.node('policy',3060,1450,720,215,'Policy Manager',kind='component')
    p.node('policy_port',3110,1525,620,105,'Evidence Use Authorization',[
        'read / 전송 / reuse / 채택 사용권 확인'],owner='policy')
    p.node('context',3060,1740,720,290,'Context Manager',kind='component')
    p.node('context_port',3110,1825,620,130,'Context Read Port',[
        '같은 관련 이력 조회 / async receipt', '현재 화면 원본과 구별'],owner='context')
    p.text(60,1282,'1 입력/전사 revision 등록 / capture는 의미 처리와 병행',23,BLUE)
    p.text(900,2104,'사각형 = 실행 코드   칸이 있는 상자 = RAM queue   겹친 카드 = RAM data model   원통 = 영속 DB   접힌 종이 = immutable DTO',24)
    p.text(60,2145,'검정 양방향 = async 호출/반환   파랑 → = P2P enqueue/dispatch   파랑 점선 = 완료   빨강 = 재판단/정정   회색 = 상태 read/write / 별도 bus와 pub/sub 없음',23,GRAY)
    return p


def common_edges(p):
    for src,dst,sp,tp,kind in [
        ('audio','observation_queue','B','T','post'),('capture','observation_queue','B','T','post'),
        ('observation_queue','timeline','B','T','post'),
        ('timeline','raw','B','T','state'),('evidence_port','raw','T','B','state'),
        ('audio','voice','T','T','post'),('voice','observation_queue','L','R','done'),
        ('timeline','input_control','L','L','post'),('input_control','entry','R','L','rpc'),
        ('entry','jobs','L','L','cancel'),('entry','inbox','B','T','post'),('inbox','control','B','T','post'),
        ('completion','control','T','B','done'),('control','jobs','L','L','state'),
        ('reader','evidence_port','L','R','rpc'),('reader','context_port','R','L','rpc'),
        ('model_port','semantic','R','L','rpc'),('semantic','export_gate','B','T','post'),
        ('voice','export_gate','R','R','post'),('semantic','calls','R','R','state'),
        ('export_gate','cloud','R','R','rpc'),('export_gate','policy_port','R','R','rpc'),
        ('proposal','adoption','L','R','done'),('adoption','adopted','B','T','state'),
        ('adoption','policy_port','R','L','rpc')]:
        connect(p,src,dst,sp,tp,kind)


def a():
    p=common('A')
    p.node('control',900,845,500,190,'Grounding Attempt Control',[
        'CODE: 두 inbox select / 상태 머신',
        'START → WAIT_RESULT → DONE', 'NeedEvidence → read 범위 확대', '긴 I/O를 시작하고 loop에 즉시 복귀'],owner='ri',different=True)
    p.node('joint_queue',1530,650,1340,140,'Joint Work Queue / bounded Channel<JointJob>',[
        '2 refs + prior bindings + correction + expected r / backpressure / worker는 async task',
        'attempt당 유효 job 관리 / 입력 r2는 기존 job 완료를 기다리지 않는다'],kind='queue',owner='ri',different=True)
    p.node('joint',1530,935,1340,185,'Joint Grounding Interpreter',[
        '4 ASYNC WORKER: Reader로 원본 확보 → Model Port로 공동 의미 판단 → 결과 검증',
        'speech + 당시 frame/path/native + prior를 함께 사용 / 새 target extent와 binding을 직접 생산',
        '5 TryApply(expected r, base version, delta) / 성공 또는 stale 결과를 Completion Inbox에 반환'],owner='ri',different=True)
    p.node('state',1530,1260,1340,230,'Joint Grounding State / versioned RAM aggregate',[
        'Map<bindingId, BindingRevision> + raw dependencies + coverage / TTL = 현재 요청 시도',
        'writer: Joint Grounding Interpreter / TryApply는 RevisionGate 안에서 검사와 교체를 함께 수행',
        'r1: b3 = 두 graph → r2: b3 = 아래 graph / b1 매출 범위와 b2 PDF 표의 유지도 의미 판단',
        'cache / crop / 부분 patch 허용 / 원본과 binding 사이의 필수 target 후보 저장 단계는 없음'],kind='memory',owner='ri',different=True)
    p.node('decision',1530,1620,1340,140,'InterpretGrounding result / typed union',[
        '6 Ready(delta) → proposal / NeedEvidence(scope) → 같은 workflow 재실행',
        'Ambiguous(question) / Failed / Cancelled / stale done는 저장 전에 거절'],kind='data',owner='ri')
    p.node('joint_r',1448,975,46,46,'R',kind='port',owner='ri')
    p.node('joint_m',2890,1040,46,46,'M',kind='port',owner='ri')
    common_edges(p)
    for src,dst,sp,tp,kind in [
        ('control','joint_queue','R','L','post'),('joint_queue','joint','B','T','post'),
        ('joint','state','B','T','state'),('decision','completion','L','R','done'),
        ('joint','decision','R','R','done'),('decision','proposal','B','T','done'),
        ('joint','joint_r','L','R','rpc'),('joint','joint_m','R','L','rpc')]:connect(p,src,dst,sp,tp,kind)
    p.text(1560,585,'R 태그 = 이 adapter의 readAsync 호출/반환',25,BLUE)
    p.text(2290,585,'M 태그 = 이 adapter의 invokeAsync 호출/반환',25,BLUE)
    p.text(1550,1195,'원본 → 의미 모델의 직접 binding → 하나의 binding aggregate',28,BLUE,True)
    p.text(930,1700,'상태 lock 안에서는\nawait / 모델 호출을 하지 않는다\n추가 판단도 같은 workflow로 복귀',23)
    return p


def b():
    p=common('B')
    p.node('control',900,845,500,190,'Grounding Coordinator',[
        'CODE: 두 inbox select / orchestration', '게시 통지 → 호환된 snapshot 확보', '양쪽 응답 또는 deadline → 결합 지시', 'Rejudge → 해당 producer queue'],owner='ri',different=True)
    p.node('speech_queue',1530,650,600,140,'Speech Work Queue',[
        '2 bounded Channel<SpeechJob>', 'refs + peer snapshot + generation'],kind='queue',owner='ri',different=True)
    p.node('screen_queue',2260,650,610,140,'Screen Work Queue',[
        '2 bounded Channel<ScreenJob>', 'refs + peer snapshot + generation'],kind='queue',owner='ri',different=True)
    p.node('speech',1530,935,600,170,'Speech Reference Producer',[
        '4 ASYNC WORKER / 원음/전사',
        'Reader → Model Port → mention 후보', '5 TryPublishSpeech(expected, candidates)'],owner='ri',different=True)
    p.node('screen',2260,935,610,170,'Screen Target Producer',[
        '4 ASYNC WORKER / frame/path/native', 'Reader → 필요 시 Model Port → target', '5 TryPublishScreen(expected, candidates)'],owner='ri',different=True)
    p.node('mentions',1530,1200,600,185,'Speech Candidate Revisions',[
        'RAM Map<mentionId, immutable revision>', 'm3@v1: count=2 / m4@v2: lower only', 'writer = Speech Reference Producer', 'source refs / peer versions / UNKNOWN'],kind='memory',owner='ri',different=True)
    p.node('targets',2260,1200,610,185,'Screen Candidate Revisions',[
        'RAM Map<targetId, immutable revision>', 't0@v1: figure → t7,t8@v2: 두 extent', 'writer = Screen Target Producer', 'source refs / coverage / peer versions'],kind='memory',owner='ri',different=True)
    p.node('binding_queue',900,1680,500,140,'Binding Work Queue',[
        '6 bounded Channel<BindingJob>', 'candidate read-set + prior + deadline'],kind='queue',owner='ri',different=True)
    p.node('resolver',1530,1520,600,175,'Multimodal Binding Resolver',[
        '6 ASYNC WORKER / snapshot + excerpt', 'Model Port → binding 또는 Rejudge', '후보 ID를 연결 / 새 target은 생산자에게', '7 TryApplyBinding(expected read-set)'],owner='ri',different=True)
    p.node('bindings',2260,1520,610,175,'Binding Revision State',[
        'RAM Map<bindingId, BindingRevision>', 'b3: m3@v1 → {t7,t8}@v2', 'writer = Multimodal Binding Resolver', 'read-set + raw deps / base version'],kind='memory',owner='ri',different=True)
    for key,x,y,tag in [('speech_r',1448,973,'R'),('speech_m',2160,1042,'M'),('screen_r',2188,973,'R'),('screen_m',2890,1042,'M'),('resolver_m',2160,1582,'M')]:
        p.node(key,x,y,46,46,tag,kind='port',owner='ri')
    common_edges(p)
    for src,dst,sp,tp,kind in [
        ('control','speech_queue','R','L','post'),('control','screen_queue','R','T','post'),
        ('speech_queue','speech','B','T','post'),('screen_queue','screen','B','T','post'),
        ('speech','speech_r','L','R','rpc'),('screen','screen_r','L','R','rpc'),
        ('speech','speech_m','R','L','rpc'),('screen','screen_m','R','L','rpc'),
        ('speech','mentions','B','T','state'),('screen','targets','B','T','state'),
        ('speech','completion','L','R','done'),('screen','completion','R','R','done'),
        ('control','binding_queue','L','L','post'),('binding_queue','resolver','R','L','post'),
        ('mentions','resolver','B','T','state'),('targets','resolver','B','R','state'),
        ('resolver','resolver_m','R','L','rpc'),('resolver','bindings','R','L','state'),
        ('resolver','completion','L','R','done'),('resolver','proposal','B','T','done')]:connect(p,src,dst,sp,tp,kind)
    p.text(1530,585,'R 태그 = 이 adapter의 readAsync 호출/반환',25,BLUE)
    p.text(2260,585,'M 태그 = 이 adapter의 invokeAsync 호출/반환',25,BLUE)
    p.text(1530,1440,'Candidate Exchange State = 위 두 RAM repository / RevisionGate 아래 조건부 게시 + 일관된 read-set',23,BLUE)
    p.text(1530,1477,'6 JOIN: 호환된 read-set을 Binding Work Queue로 전달',22,BLUE)
    p.text(1530,1770,'7 후보 부족: RejudgeRequest → Completion Inbox → Coordinator → 원 producer queue',25,RED)
    p.text(1530,1818,'t0만 있으면 원본을 다시 읽고 t7,t8을 생산한 뒤 재결합 / coordinator는 의미 값을 채우지 않는다',23,RED)
    return p


SHARED = ['im','audio','capture','observation_queue','timeline','raw','evidence_port','rc','input_control','adoption',
          'db_owner','adopted','ri','entry','inbox','reader','model_port','completion','jobs','proposal',
          'ma','voice','semantic','export_gate','calls','cloud','policy','policy_port','context','context_port']

def sequence(side):
    if side=='A':
        names=[('rc','Request Controller'),('ctl','Grounding Attempt Control\nControl / Completion Inbox'),
               ('worker','Joint Grounding Interpreter\nJoint Work Queue + async worker'),('im','Interaction Manager\nEvidence Read Port'),
               ('ma','Model Access\nSemantic API Client'),('state','Request Interpreter\nJoint Grounding State')]
        subtitle='같은 O17/P3에서 graph 집합을 만들고 r2의 아래 graph만으로 patch / 새 target은 공동 해석 책임이 생산한다'
    else:
        names=[('rc','Request Controller'),('ctl','Grounding Coordinator\nControl / Completion Inbox'),('speech','Speech Reference Producer\nSpeech Work Queue + async worker'),
               ('screen','Screen Target Producer\nScreen Work Queue + async worker'),('resolver','Multimodal Binding Resolver\nBinding Work Queue + async worker'),('state','Request Interpreter\nCandidate Exchange State')]
        subtitle='처음 figure 하나뿐인 후보를 둘로 확장한 뒤 r2로 일부 정정 / 후보 누락과 늦은 결과까지 같은 사건을 추적한다'
    p=GroundingScene('dp47-revision-'+side.lower(),side+' / 정상 생성, 부분 정정과 늦은 결과의 실제 반환 경로',subtitle)
    p.text(60,215,'D 번호는 이 사건도의 메시지 순서 / 세로 간격은 시간 측정값이 아님 / target 이름과 후보 key는 생산 이후의 설명 예시',24)
    xs=[340,980,1620,2260,2900,3540]
    for (key,title),x in zip(names,xs):
        p.node(key,x-290,270,580,110,title.split('\n')[0],[title.split('\n')[1]] if '\n' in title else [],kind='trace')
        # Lifeline is an editable node, rather than an unbound decorative arrow.
        p.node(key+'_life',x-1,400,2,1470,'',kind='life')
    def msg(no,source,target,y,text,kind='post'):
        a=names.index(next(n for n in names if n[0]==source));b=names.index(next(n for n in names if n[0]==target));x1,x2=xs[a],xs[b]
        # Sequence messages intentionally cross other lifelines.
        e=dict(id='D'+str(no),source=source+'_life',target=target+'_life',points=[(x1,y),(x2,y)],kind=kind)
        p.edges.append(e);p.text(min(x1,x2)+18,y-16,f'D{no}  '+text,24,color=RED if kind=='cancel' else INK)
    if side=='A':
        rows=[
            ('rc','ctl','BeginRevision(r1) → Control Inbox / raw refs + prior','post'),
            ('ctl','worker','Joint Work Queue → JointJob(j1,r1) / loop는 복귀','post'),
            ('worker','im','R.readAsync(scope,lease) / 원본 owner에 bounded 조회','call'),
            ('im','worker','immutable receipt / O17,P3 + coverage + sourceVector','done'),
            ('worker','ma','M.invokeAsync(j1): raw + “두 graph” + prior','post'),
            ('ma','worker','j1 done: 직접 생산한 두 extent와 b3 delta','done'),
            ('worker','state','RevisionGate → TryApply(r1,base,delta) → unlock','state'),
            ('worker','ctl','Completion Inbox: Ready(j1) / 진행 중 job 회수','done'),
            ('ctl','worker','Joint Work Queue: j1b / 모호한 관계의 추가 판단','post'),
            ('worker','ma','M.invokeAsync(j1b): 추가 판단 진행 중','post'),
            ('rc','ctl','BeginRevision(r2) fence → Control Inbox / j1b cancel 요청','cancel'),
            ('ctl','worker','Joint Work Queue: j2 / 정정 + prior + 유효 raw refs','post'),
            ('worker','ma','M.invokeAsync(j2): 아래 graph만 / 부분 재판단','post'),
            ('ma','worker','j1b late done: gate에서 r1 불일치 → 상태 갱신 없음','cancel'),
            ('ma','worker','j2 done: KEEP b1,b2 / REPLACE b3 → lower extent','done'),
            ('worker','state','RevisionGate → TryApply(r2,base,delta) → unlock','state'),
            ('worker','ctl','Completion Inbox: Ready(j2) / raw deps + binding version','done'),
            ('worker','rc','GroundingProposal r2 → 현재성 검사 후 State Store 채택','done'),
        ]
        foot='A: 새 lower extent와 keep/replace 의미를 같은 해석 책임이 직접 생산한다. crop/cache/부분 patch를 허용한다.'
    else:
        rows=[
            ('rc','ctl','BeginRevision(r1) → Control Inbox / 원본 refs','post'),
            ('ctl','speech','Speech Work Queue: r1 / R read → M 의미 호출','post'),
            ('ctl','screen','Screen Work Queue: r1 / O17,P3 / R read → 필요 시 M','post'),
            ('speech','state','gate → TryPublishSpeech(m3@v1: count=2) → unlock','state'),
            ('speech','ctl','Completion Inbox: CandidatePublished(m3@v1)','done'),
            ('screen','state','gate → TryPublishScreen(t0@v1: figure 하나) → unlock','state'),
            ('screen','ctl','Completion Inbox: CandidatePublished(t0@v1)','done'),
            ('ctl','resolver','ReadSnapshot → Binding Work Queue: m3@v1,t0@v1','post'),
            ('resolver','ctl','Completion Inbox: Rejudge(Screen, peer=m3@v1)','cancel'),
            ('ctl','screen','Screen Work Queue: 두 extent 필요 / 원본 확대 read','post'),
            ('screen','state','gate → t7,t8@v2 게시 / source + m3@v1 의존','state'),
            ('screen','ctl','Completion Inbox: CandidatePublished(t7,t8@v2)','done'),
            ('ctl','resolver','Binding Work Queue: 새 read-set → M 재결합','post'),
            ('resolver','ctl','Completion Inbox: Ready(binding r1) / job 회수','done'),
            ('resolver','rc','TryApplyBinding 성공 → r1 proposal / read-set 동봉','done'),
            ('ctl','screen','Screen Work Queue: r1 추가 coverage job g1b 진행','post'),
            ('rc','ctl','BeginRevision(r2) fence + hold → Control Inbox','cancel'),
            ('ctl','speech','Speech Work Queue: r2 “아래 것만” / prior + peer','post'),
            ('speech','state','gate → m4@v2: lower / 앞의 두 자료 유지 후보','state'),
            ('speech','ctl','Completion Inbox: CandidatePublished(m4@v2)','done'),
            ('ctl','screen','Screen Work Queue: g2 / m4@v2 의존 재검토','post'),
            ('screen','ctl','g1b late done: gate에서 거절 → Completion Inbox stale','cancel'),
            ('screen','state','gate → g2 t8@v3 / extent 재사용 + 새 peer 의존','state'),
            ('screen','ctl','Completion Inbox: CandidatePublished(t8@v3)','done'),
            ('ctl','resolver','Binding Work Queue: m4@v2,t8@v3 + prior → M','post'),
            ('resolver','ctl','Completion Inbox: Ready(binding r2) / job 회수','done'),
            ('resolver','rc','TryApplyBinding 성공 → r2 proposal / 최신 read-set','done'),
        ]
        foot='B: D9–D13은 후보 확장 왕복이다. r2는 D18–D25의 후보 갱신과 새 read-set 결합을 거친다.'
    for i,row in enumerate(rows,1):
        msg(i,row[0],row[1],430+(i-1)*(1400/(len(rows)-1)),row[2],row[3])
    p.text(60,1910,foot,26)
    p.text(60,1965,'공통: 결과 대기 중 capture / 전사 / local stop은 계속된다. Request Controller가 최신 revision, 사용권, 근거와 base version을 검사한다.',24)
    p.text(60,2015,'채택 뒤에는 전체 요청 해석과 admission을 거쳐 응답 / 업무로 연결한다. 일부 지칭 실패는 해당 범위를 질문하며 앞의 의미를 임의 확정하지 않는다.',24)
    p.text(60,2065,'R/M = 구조도의 같은 read/model adapter / gate 안에는 await 없음 / B의 각 worker 내부 R/M 왕복은 축약 / 실행 trace와 성능 측정 아님',23,GRAY)
    p.text(60,2120,'실선: 요청 / 파랑 점선: 결과 / 빨강 점선: 정정과 무효화 / 회색 점선: 상태 게시 / 실행 trace가 아닌 같은 사건의 설계 예시',23,GRAY)
    return p


def scenes():
    first,second=a(),b()
    for key in SHARED:assert tuple(first.nodes[key][k] for k in ('x','y','w','h'))==tuple(second.nodes[key][k] for k in ('x','y','w','h')),key
    return [first,second,sequence('A'),sequence('B')]
