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
    title=('A / Joint Grounding Interpreter가 원본에서 binding과 부분 교체를 직접 생산' if side=='A' else
           'B / 두 생산자의 후보가 준비된 뒤 결합하고, 후보가 없으면 원 생산자로 돌아간다')
    p=GroundingScene('dp47-implementation-'+side.lower(),title,
        '동일 배치의 공통 Component / input r1 → graph 집합 → r2 “아래 것만, 앞의 두 자료는 그대로” / 기호는 생산 후 예시')
    p.node('im',60,240,760,1010,'Interaction Manager',kind='component')
    p.node('ri',960,240,1940,1660,'Request Interpreter',kind='component')
    p.node('ma',3000,240,780,680,'Model Access',kind='component')
    p.node('rc',60,1320,760,580,'Request Controller',kind='component')
    p.node('policy',3000,1260,780,200,'Policy Manager',kind='component')
    p.node('context',3000,1540,780,205,'Context Manager',kind='component')
    p.node('response',1060,1935,720,85,'Response Manager',kind='component')
    p.node('task',2110,1935,720,85,'Task Manager',kind='component')
    p.node('audio',100,330,295,70,'Channel I/O',owner='im')
    p.node('vad',445,330,325,70,'Turn-Taking Control',owner='im')
    p.node('capture',100,475,670,70,'Evidence Capture',owner='im')
    p.node('timeline',100,645,670,75,'Timeline & Buffer',owner='im')
    p.node('raw',100,825,670,150,'Raw Evidence Window',[
        'audioRef / transcriptRev / frameRef / pointerPath',
        'window / viewport / sourceVector / gap / expiresAt'],kind='store',owner='im')
    p.node('receipt',100,1080,670,125,'EvidenceReadReceipt',[
        'source refs + coverage + clock error + lease',
        '없는 span / UIA / 과거 화면은 UNKNOWN'],kind='data',owner='im')
    p.node('input_control',100,1400,670,70,'Input Revision Control',owner='rc')
    p.node('revision',100,1520,300,100,'Input Revision State',['r2 / hold / policyEpoch'],kind='store',owner='rc')
    p.node('adoption',450,1660,320,75,'Adoption Control',owner='rc')
    p.node('adopted',100,1780,670,85,'Adopted Binding Records',['binding version / artifact refs'],kind='store',owner='rc')
    p.node('voice',3050,340,680,70,'Voice API Client',owner='ma')
    p.node('export_gate',3050,500,680,65,'Model Use Gate',owner='ma')
    p.node('semantic',3050,595,680,90,'Semantic API Client',owner='ma')
    p.node('calls',3050,770,680,110,'Model Call Registry',['jobId / role / inputRev / cancel / usage'],kind='store',owner='ma')
    p.node('cloud',3050,1010,680,120,'클라우드 모델 API',['voice / semantic: 동일 capability와 예산'],kind='external')
    p.node('policy_port',3050,1340,680,65,'Evidence Use Authorization',owner='policy')
    p.node('context_port',3050,1620,680,65,'Context Read Port',owner='context')
    p.node('input',1050,330,720,155,'GroundingInput',[
        'R47 / inputRev r1 / observationWindow W47',
        'audio + transcript refs / prior bindings / policy',
        '현재 시점까지 확보한 자료, 미래 정답 없음'],kind='data',owner='ri')
    p.node('example',1960,330,865,155,'동일 원본의 검토 예시',[
        'S0: 실제 노출된 선택 / O11: 이동 전 PDF 표',
        'O17 + P3: page drag 뒤 frame + 버튼 없는 circle',
        'r2: “아래 것만” / mail 화면은 과거 근거 아님'],kind='data',owner='ri')
    p.text(100,305,'원음 / sample clock',21)
    p.text(100,445,'Windows frame / pointer / button / native selection',21)
    p.text(100,602,'기록 + 시각 정규화 / 의미 binding은 생성하지 않음',21)
    p.text(100,785,'실제 취득 범위만 유한 보존 / 시작 관측 공백도 기록',21)
    p.text(100,1030,'ReadEvidence(scope, sourceVector, useLease)',21)
    p.text(3060,440,'audio 전송 → / 전사 revision ←\n단어 span은 제공된 범위만 보존',23)
    p.text(3060,734,'비동기 start / correlated done / best-effort cancel',21)
    p.text(3010,1208,'외부 제공 전과 실제 사용 시 policyEpoch 재확인',22)
    p.text(3060,1510,'같은 관련 Context / 후보 정답을 미리 만들지 않음',21)
    p.text(100,1288,'1 입력 / 전사 revision / 관측 참조를 등록',23,BLUE)
    p.text(1060,2058,'정상 1–8 = 본문 계약 표 / 정정 순서는 다음 사건도에서 확대 / 8 이후 전체 요청 해석과 admission을 거쳐 응답 또는 업무',23)
    p.text(60,2110,'검정 실선: 직접 호출   파랑 실선: 요청/자료   파랑 점선: 결과   빨강 점선: 정정/재판단/취소   회색 점선: 상태 접근',24)
    p.text(60,2146,'살구색: 안별 Module/임시 상태 / 원통: owner 상태 / 접힌 종이: 자료 / 사각형: 이름만 / 별도 process, thread 수, SDK는 미선정',23,GRAY)
    return p


def common_edges(p,controller):
    for a,b,sp,tp,kind in [('audio','vad','R','L','call'),('capture','timeline','B','T','post'),('vad','timeline','B','T','post'),
                          ('audio','raw','L','L','post'),('timeline','raw','B','T','state'),('raw','receipt','B','T','done'),
                          ('timeline','input_control','L','L','post'),('input_control','revision','B','T','state'),
                          ('adoption','adopted','B','T','state'),('voice','export_gate','B','T','post'),
                          ('cloud','voice','L','R','done'),('semantic','export_gate','T','B','post'),('export_gate','cloud','R','R','post'),
                          ('export_gate','policy_port','R','R','call'),('policy_port','export_gate','R','R','done'),('cloud','semantic','L','L','done'),
                          ('semantic','calls','B','T','state'),('audio','voice','T','T','post'),('voice','timeline','L','R','done'),
                          ('input_control','input','R','L','post'),('input','%s'%controller,'B','T','post'),
                          ('revision',controller,'R','L','cancel'),('adoption','policy_port','R','L','call'),
                          ('policy_port','adoption','L','R','done'),('revision','adoption','R','T','state'),
                          ('adoption','response','R','L','post'),('adoption','task','B','L','post')]:
        connect(p,a,b,sp,tp,kind)
    p.text(850,1435,'2 GroundingInput',21,BLUE)
    p.text(850,1535,'r2 / stale fence',21,RED)
    p.text(455,1632,'8 현재 조건 확인',21)
    p.text(115,1748,'채택 = 의미 정답의 증명 아님',21)


def a():
    p=common('A')
    p.node('control',2320,590,505,90,'Grounding Attempt Control',owner='ri',different=True)
    p.node('reader',1050,590,720,90,'Grounding Evidence Reader',owner='ri')
    p.node('joint',1050,900,1140,100,'Joint Grounding Interpreter',owner='ri',different=True)
    p.node('state',1050,1190,1140,170,'Joint Grounding State',[
        'binding b1 / b2 / b3 + raw refs / dependency / revision',
        'r1: graph set → r2: lower graph / b1,b2 유지 제안',
        '요청 시도 RAM / 유효 crop, cache, 부분 재판단 허용'],kind='store',owner='ri',different=True)
    p.node('jobs',2320,1190,505,170,'Grounding Attempt Registry',[
        'attempt a2 / sourceVector / policyEpoch',
        'jobId / expected r2 / cancel handle',
        'old done는 상태 갱신 전에 거절'],kind='store',owner='ri',different=True)
    p.node('proposal',1050,1585,1140,210,'GroundingProposal',[
        'r2 / base binding version / sourceVector / policyEpoch',
        'KEEP b1,b2 / REPLACE b3 → O17의 lower extent',
        '근거, unresolved, omitted coverage / 의미 모델의 제안',
        '새 target extent를 Joint Grounding Interpreter가 직접 생산'],kind='data',owner='ri')
    p.node('outcome',2320,1560,505,205,'InterpretGrounding outcome',[
        'proposal delta | NeedEvidence',
        'Ambiguous | Failed | Cancelled',
        '읽기 확대 또는 같은 판단 재실행',
        '한 호출 / 전체 replay 강제 없음'],kind='data',owner='ri')
    p.text(1050,550,'3 bounded 원본 조회 / crop와 실제 coverage 반환',23,BLUE)
    p.text(2320,550,'2 새 시도 등록 / 이전 시도 무효화',22,BLUE)
    p.text(1050,760,'4 같은 해석 책임에 원음/전사 + 당시 pixels/path/native\n+ 이전 binding + 정정 원문을 제공한다',26,BLUE)
    p.text(1050,1115,'5 직접 binding / keep / replace / 추가 근거 요구 ←',25,BLUE)
    p.text(2320,1115,'onDone: 현재 시도만 상태 갱신',22)
    p.text(1050,1450,'6 원본 의존과 잠정 patch 기록 / 영향 불명확하면 넓혀 재판단',24)
    p.text(1050,1845,'7 유효한 제안 또는 해당 지칭의 확인 질문 → Request Controller',23,BLUE)
    p.text(2320,1430,'추가 read와 모델 재호출은\n같은 소유자의 재판단이다',22)
    common_edges(p,'control')
    for a1,b,sp,tp,kind in [('control','reader','L','R','post'),('reader','timeline','L','R','call'),
        ('receipt','reader','R','L','done'),('reader','joint','B','T','post'),('control','joint','L','R','post'),
        ('joint','semantic','R','L','post'),('semantic','joint','L','R','done'),
        ('control','jobs','B','T','state'),('joint','state','B','T','state'),('state','joint','R','B','state'),
        ('joint','outcome','R','T','done'),('outcome','control','R','R','done'),
        ('joint','proposal','L','T','done'),('proposal','adoption','L','R','done'),
        ('control','semantic','R','L','cancel'),('reader','context_port','R','L','call'),
        ('context_port','reader','L','R','done'),('jobs','joint','L','R','state')]:connect(p,a1,b,sp,tp,kind)
    return p


def b():
    p=common('B')
    p.node('control',1050,590,720,90,'Grounding Coordinator',owner='ri',different=True)
    p.node('jobs',2320,580,505,155,'Grounding Attempt Registry',[
        'inputRev / producer generation / jobId',
        'read-set / expected candidate version',
        '코드는 경로와 현재성만 조정'],kind='store',owner='ri',different=True)
    p.node('speech',1050,860,720,90,'Speech Reference Producer',owner='ri',different=True)
    p.node('screen',1990,860,835,90,'Screen Target Producer',owner='ri',different=True)
    p.node('candidates',1010,1100,1855,285,'Candidate Exchange State',kind='group',owner='ri',different=True)
    p.node('mentions',1050,1190,720,155,'Speech Candidate Revisions',[
        'm3@v1: “두 graph” / count=2 / scope=possible',
        '원음/전사 refs + peer refs / competing / UNKNOWN',
        'Speech Reference Producer만 의미 값을 갱신'],kind='store',owner='candidates',different=True)
    p.node('targets',1990,1190,835,155,'Screen Candidate Revisions',[
        't0@v1: figure → t7,t8@v2: 두 visual extent',
        'frame/path refs + peer m3@v1 / coverage / alternatives',
        'Screen Target Producer만 target와 extent를 갱신'],kind='store',owner='candidates',different=True)
    p.node('resolver',1050,1490,1140,90,'Multimodal Binding Resolver',owner='ri',different=True)
    p.node('bindings',2320,1490,505,140,'Binding Revision State',[
        'b3: m3@v1 → {t7,t8}@v2',
        'read-set / dependency / base version'],kind='store',owner='ri',different=True)
    p.node('proposal',1050,1730,1140,145,'GroundingProposal',[
        'r2 / KEEP b1,b2 / REPLACE b3 / candidate read-set',
        '생산된 target 후보 참조 / 미해결이면 해당 지칭 확인'],kind='data',owner='ri')
    p.node('rejudge',2320,1730,505,145,'RejudgeRequest',[
        'owner / missing relation / peer refs',
        '추가 raw scope / expected versions'],kind='data',owner='ri')
    p.text(1050,550,'3 두 생산 요청 / peer 참조 전달 / bounded raw read',23,BLUE)
    p.text(1050,805,'4 표현 / 수 / 정정 역할 후보',24,BLUE)
    p.text(1990,805,'4 native + visual / navigation과 designation 후보',24,BLUE)
    p.text(1050,1038,'5 producer별 결과 검증 후 조건부 교체 / 원본과 peer version 보존',24)
    p.text(1050,1432,'6 호환된 candidate read-set + 허용 excerpt로 결합 / 의미는 모델 판단',24,BLUE)
    p.text(1050,1677,'7 후보 참조 binding 또는 RejudgeRequest / 새로운 target 직접 생성 금지',23,BLUE)
    common_edges(p,'control')
    for a1,b1,sp,tp,kind in [('control','jobs','R','L','state'),('control','speech','B','T','post'),('control','screen','R','T','post'),
        ('control','timeline','L','R','call'),('receipt','control','R','L','done'),('control','context_port','R','L','call'),
        ('speech','mentions','B','T','state'),('screen','targets','B','T','state'),
        ('mentions','resolver','B','T','state'),('targets','resolver','B','R','state'),('control','resolver','L','L','post'),
        ('speech','semantic','R','L','post'),('semantic','speech','L','R','done'),
        ('screen','semantic','R','L','post'),('semantic','screen','L','R','done'),
        ('resolver','semantic','R','L','post'),('semantic','resolver','L','R','done'),
        ('mentions','control','L','B','done'),('targets','control','R','R','done'),
        ('context_port','control','L','R','done'),('bindings','resolver','L','R','state'),
        ('resolver','bindings','R','L','state'),('resolver','proposal','B','T','done'),('proposal','adoption','L','R','done'),
        ('resolver','rejudge','R','T','cancel'),('rejudge','control','R','R','cancel'),('control','semantic','R','L','cancel')]:connect(p,a1,b1,sp,tp,kind)
    return p


def sequence(side):
    if side=='A':
        names=[('im','Interaction Manager\nTimeline & Buffer'),('rc','Request Controller'),('ctl','Request Interpreter\nGrounding Attempt Control'),
               ('worker','Request Interpreter\nJoint Grounding Interpreter'),('ma','Model Access'),('state','Request Interpreter\nJoint Grounding State')]
        subtitle='같은 O17/P3에서 graph 집합을 만들고 r2의 아래 graph만으로 patch / 새 target은 공동 해석 책임이 생산한다'
    else:
        names=[('rc','Request Controller'),('ctl','Request Interpreter\nGrounding Coordinator'),('speech','Request Interpreter\nSpeech Reference Producer'),
               ('screen','Request Interpreter\nScreen Target Producer'),('resolver','Request Interpreter\nMultimodal Binding Resolver'),('state','Request Interpreter\nCandidate Exchange State')]
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
            ('rc','ctl','GroundingInput r1 / W47 / raw refs / prior bindings','post'),
            ('ctl','im','Evidence Reader를 통해 ReadEvidence(scope, lease)','post'),
            ('im','worker','Reader를 거친 raw / coverage / sourceVector 반환','done'),
            ('worker','ma','job j1: InterpretGrounding(raw, “두 graph”, prior)','post'),
            ('ma','worker','j1 done: 두 extent의 b3 / 근거와 불확실성','done'),
            ('worker','state','현재 attempt 확인 후 b1,b2,b3와 dependency 기록','state'),
            ('worker','ma','j1b: 아직 모호한 관계의 추가 판단 시작','post'),
            ('rc','ctl','r2 “아래 것만” / hold / a1 fence / j1b 취소 요청','cancel'),
            ('ctl','worker','a2: 정정 원문 + 이전 binding + 유효 O17/P3','post'),
            ('worker','ma','j2: 부분 재판단 / 필요한 추가 raw read 허용','post'),
            ('ma','worker','j1b late done: a1 != a2 → 상태 갱신 없이 폐기','cancel'),
            ('ma','worker','j2 done: KEEP b1,b2 / REPLACE b3 → lower extent','done'),
            ('worker','state','a2 / r2 / source 유효성 확인 후 patch 기록','state'),
            ('worker','rc','GroundingProposal r2 / base version / raw dependency','done'),
        ]
        foot='A: 새 lower extent와 keep/replace 의미를 같은 해석 책임이 직접 생산한다. crop/cache/부분 patch를 허용한다.'
    else:
        rows=[
            ('rc','ctl','GroundingInput r1 / 확보한 raw refs / coverage','post'),
            ('ctl','speech','음성/전사 read → ProduceSpeechReferences → Model Access','post'),
            ('ctl','screen','O17/P3 read → ProduceScreenTargets → Model Access','post'),
            ('speech','state','현재 generation의 m3@v1: “두 graph”, count=2 게시','state'),
            ('screen','state','현재 generation의 t0@v1: figure 하나, 분할 불명확','state'),
            ('ctl','resolver','완료 통지 후 m3@v1 + t0@v1 / 허용 excerpt → 의미 호출','post'),
            ('resolver','ctl','RejudgeRequest(Screen): 두 extent 필요 / peer m3@v1','cancel'),
            ('ctl','screen','원본 O17/P3 확대 read / peer m3 전달 → 의미 호출','post'),
            ('screen','state','t7,t8@v2 게시 / source refs / m3@v1 의존 / t0 대체','state'),
            ('ctl','resolver','완료 통지 후 새 read-set(m3@v1,t7,t8@v2) → 재결합','post'),
            ('resolver','rc','b3={t7,t8} 잠정 제안 / candidate versions와 raw refs','done'),
            ('ctl','screen','r1의 추가 coverage 확인 job g1b 시작, 결과 대기','post'),
            ('rc','ctl','r2 “아래 것만” / hold / 옛 generation fence + cancel','cancel'),
            ('ctl','speech','r2 원음/전사 + prior / 정정 후보 생산 → Model Access','post'),
            ('speech','state','m4@v2: lower / KEEP 앞의 두 자료 / raw와 peer refs','state'),
            ('ctl','screen','새 g2: m4@v2에 따른 target 검토 / 유효 원본 재사용','post'),
            ('screen','ctl','g1b late done: expected generation 불일치 → 미게시','cancel'),
            ('screen','state','g2: lower=t8@v3 / extent 재사용 / peer m4@v2 갱신','state'),
            ('ctl','resolver','m4@v2,t8@v3와 유지 binding read-set → 부분 재결합','post'),
            ('resolver','rc','r2 제안: KEEP b1,b2 / REPLACE b3 / stale read-set 거절','done'),
        ]
        foot='B: 후보가 없으면 D7–D10을 거친다. r2는 D14–D19의 생산자별 갱신과 새 read-set 결합을 거친다.'
    for i,row in enumerate(rows,1):
        msg(i,row[0],row[1],430+(i-1)*(104 if side=='A' else 74),row[2],row[3])
    p.text(60,1910,foot,26)
    p.text(60,1965,'공통: 결과 대기 중 capture / 전사 / local stop은 계속된다. Request Controller가 최신 revision, 사용권, 근거와 base version을 검사한다.',24)
    p.text(60,2015,'채택 뒤에는 전체 요청 해석과 admission을 거쳐 응답 / 업무로 연결한다. 일부 지칭 실패는 해당 범위를 질문하며 앞의 의미를 임의 확정하지 않는다.',24)
    p.text(60,2065,'B 모델 왕복과 candidate 완료 통지는 메시지에 축약했다. 구조도에서 실제 호출과 반환을 표시한다. D 간격과 병행 정도는 성능 주장이 아니다.',23,GRAY)
    p.text(60,2120,'실선: 요청 / 파랑 점선: 결과 / 빨강 점선: 정정과 무효화 / 회색 점선: 상태 게시 / 실행 trace가 아닌 같은 사건의 설계 예시',23,GRAY)
    return p


def scenes():
    first,second=a(),b()
    shared=['im','ri','ma','rc','policy','context','response','task','audio','vad','capture','timeline','raw','receipt','input_control','revision','adoption','adopted','voice','export_gate','semantic','calls','cloud','policy_port','context_port','input','example']
    for key in shared:assert tuple(first.nodes[key][k] for k in ('x','y','w','h'))==tuple(second.nodes[key][k] for k in ('x','y','w','h')),key
    return [first,second,sequence('A'),sequence('B')]
