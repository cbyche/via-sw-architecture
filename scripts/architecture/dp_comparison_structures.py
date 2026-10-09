"""Presentation scenes preserving the mechanisms of the 40s MAIN figures.

This is an editable comparison view, not a new Architecture or a sequence spec.
All coordinates are local to one option; no operational edge crosses options.
"""
from generate_dp_background_slides import INK, MUTED, BLUE, TEAL, RED, LINE
import unicodedata

PURPLE = '#7155A4'
COMMON = '#000000'
APRICOT = '#FFF0DD'
DIFFERENCE_STROKE = '#B8753F'
DIFFERENCE_WIDTH = 1.8
BOUNDARY_WIDTH = 2.6
BOUNDARY_A = DIFFERENCE_STROKE
BOUNDARY_B = DIFFERENCE_STROKE


class Graph:
    def __init__(self, slide, x, color):
        self.s, self.x, self.c, self.nodes = slide, x, color, {}
        self.annotations = []

    def box(self, key, x, y, w, h, name, color=None, size=18):
        self.nodes[key] = (x, y, w, h)
        self.s.box(self.x+x, y, w, h, name, color or INK, size=size)

    def store(self, key, x, y, w, h, name, size=17):
        self.nodes[key] = (x, y, w, h)
        self.s.store(self.x+x, y, w, h, name, size=size)

    def group(self, x, y, w, h, name):
        self.s.group(self.x+x, y, w, h, name, self.c)

    def text(self, x, y, w, text, color=MUTED, size=17):
        # Native label backgrounds keep annotation letters clear at a bus crossing.
        # Their widths follow the text rather than covering the whole drawing row.
        before=len(self.s.items)
        lines=text.split('\n')
        width=min(w,max(sum(size*(1 if unicodedata.east_asian_width(ch) in 'WF' else .58)
                            for ch in line) for line in lines)+4)
        self.s.rect(self.x+x-2,y-1,width+4,min(len(lines)*size*1.2+2,794-(y-1)),'white','none')
        self.s.label(self.x+x, y, w, text, size, color)
        self.annotations.extend(self.s.items[before:])

    def finish(self):
        ids={i['id'] for i in self.annotations}
        self.s.items=[i for i in self.s.items if i['id'] not in ids]+self.annotations

    def port(self, key, side, offset=0):
        x,y,w,h = self.nodes[key]
        return {'T':(x+w/2+offset,y), 'B':(x+w/2+offset,y+h),
                'L':(x,y+h/2+offset), 'R':(x+w,y+h/2+offset)}[side]

    def edge(self, a, b, sp='B', tp='T', via=(), ret=False, color=None, sd=0, td=0):
        pts = [self.port(a,sp,sd), *via, self.port(b,tp,td)]
        assert all(8 <= xx <= 858 and 243 <= yy <= 792 for xx,yy in pts), (a,b,pts)
        self.s.line([(self.x+xx,yy) for xx,yy in pts],color=color or self.c,dashed=ret)

    def raw(self, pts, ret=False, color=None, arrow=True):
        self.s.line([(self.x+xx,yy) for xx,yy in pts],color=color or self.c,dashed=ret,arrow=arrow)


def semantic_start(g):
    g.box('input',25,281,245,38,'Interaction Manager')
    g.box('start',337,281,230,38,'Request Controller')
    g.box('model',639,281,206,38,'Model Access')
    g.text(28,253,280,'확정 User Turn / 새 Request',size=17)
    g.text(638,324,210,'공유 Omni 1벌 / 역할별 KV',size=16)
    g.edge('input','start','R','L')
    g.text(277,257,130,'원문',size=16)


def semantic_finish(g, proposal, via=()):
    """Current adoption is distinct from temporary producer state."""
    g.box('adopt',30,685,250,39,'Request Controller')
    g.store('adopted',345,683,230,43,'State Store',size=17)
    g.box('policy',635,685,208,39,'Policy Manager')
    g.edge(proposal,'adopt',via=via)
    g.edge('policy','adopt','L','R',via=[(614,704),(614,677),(299,677),(299,704)],color=MUTED,ret=True)
    g.edge('adopt','adopted','R','L')
    g.text(305,652,528,'현재 검사: 입력 / 정보 / Task / 질문 / 권한',size=16)
    g.text(355,734,462,'채택 의미와 질문 / 임시 해석 상태와 별도',size=16)
    g.text(30,757,808,'채택 후: Response Manager → Interaction Manager / Task Manager → Agent Gateway',size=17)
    g.text(30,777,808,'확인 질문의 실제 전달과 새 답변 / Downstream Agent 접수와 결과의 별도 확인',size=16)
    g.finish()


class ResolutionGraph(Graph):
    """41 notation: logical Components, contained Modules and named state."""
    def __init__(self, slide, x):
        super().__init__(slide,x,'#000000')
        self.first=len(slide.items)

    def box(self,key,x,y,w,h,name,different=False,module=False,size=18):
        self.nodes[key]=(x,y,w,h)
        before=len(self.s.items)
        self.s.box(self.x+x,y,w,h,name,self.c,fill=APRICOT if different else 'white',size=size)
        self.s.items[before].update(stroke=DIFFERENCE_STROKE if different else COMMON,
                                   line_width=DIFFERENCE_WIDTH if different else 1.0,rounded=module)

    def group(self,x,y,w,h,name,execution=False,side=0):
        before=len(self.s.items)
        color=(BOUNDARY_A if side==0 else BOUNDARY_B) if execution else COMMON
        self.s.group(self.x+x,y,w,h,name,color)
        self.s.items[before].update(fill='white',stroke=color,line_width=BOUNDARY_WIDTH if execution else 1.0,
                                   dashed=execution and side==1)
        self.s.items[before+1]['color']=COMMON

    def store(self,key,x,y,w,h,name,size=17,different=False):
        self.nodes[key]=(x,y,w,h)
        before=len(self.s.items)
        self.s.store(self.x+x,y,w,h,name,size=size)
        self.s.items[before].update(fill='white',stroke=self.c,line_width=1.0)
        if h<36: self.s.items[before+1]['y']-=5

    def text(self,x,y,w,text,color=None,size=16):
        super().text(x,y,w,text,color or self.c,size)

    def finish(self):
        for item in self.s.items[self.first:]:
            if item['kind']=='line': item['width']=1.1
        super().finish()


def resolution_legend(s, horizontal=False):
    """One notation key for 41/42, independent of Architecture choice or rank."""
    def symbol(x,y,w,h,name,kind='component',different=False):
        if kind=='state':
            s.store(x,y,w,h,name,size=13)
            s.items[-2].update(fill='white',stroke=COMMON,line_width=1.0)
            s.items[-1]['y']=y+9
        else:
            s.box(x,y,w,h,name,COMMON,fill=APRICOT if different else 'white',size=10)
            s.items[-2].update(stroke=DIFFERENCE_STROKE if different else COMMON,
                               line_width=DIFFERENCE_WIDTH if different else 1.0,rounded=kind=='module')
    def boundary(x,y,w,h):
        for xx,color,dashed,name in [(x,BOUNDARY_A,False,'A'),(x+w/2+3,BOUNDARY_B,True,'B')]:
            s.rect(xx,y,w/2-3,h,'white',color,dashed=dashed)
            s.items[-1]['line_width']=BOUNDARY_WIDTH
            s.text(xx+(w/2-3)/2,y+5,w/2-7,[name],12,COMMON,align='center')
    if horizontal:
        # 2560px MAIN footer, same symbols and meanings as the vertical key.
        s.text(64,1376,110,['Legend'],20,COMMON,True)
        for x,w,name,label,kind,diff in [
            (200,155,'Component','논리 책임 경계','component',False),
            (400,140,'Module','내부 구현','module',False),
            (925,130,'State','저장 상태','state',False),
            (1620,165,'Component','공통: 흰색 / 검정','component',False),
            (1890,165,'Module','설계 차이: 살구색','module',True)]:
            symbol(x,1373,w,28,name,kind,diff)
            s.text(x,1408,260,[label],17,COMMON)
        boundary(600,1373,160,28)
        s.text(578,1408,335,['실행 / 서비스: A 실선, B 점선'],17,COMMON)
        for x,dashed,label in [(1120,False,'요청 / 전달'),(1360,True,'응답 / 반환')]:
            s.line([(x,1387),(x+155,1387)],color=COMMON,width=1.1,dashed=dashed)
            s.text(x,1408,220,[label],17,COMMON)
        boundary(2210,1373,160,28)
        s.text(2160,1408,330,['설계 차이: 강조 경계 / 채움 없음'],17,COMMON)
        return
    s.rect(47,277,86,512,'white',COMMON);s.items[-1]['line_width']=1.0
    s.text(90,292,82,['Legend'],15,COMMON,True,'center')
    for y,name,label,kind,diff in [
        (313,'Component','논리 책임','component',False),
        (370,'Module','내부 구현','module',False),
        (477,'State','저장 상태','state',False),
        (631,'Component','흰색 / 검정','component',False),
        (687,'Module','차이: 살구색','module',True)]:
        symbol(53,y,74,27,name,kind,diff)
        s.label(54,y+33,76,label,12,COMMON)
    boundary(53,426,74,27)
    s.label(54,459,76,'실행 / 서비스',11,COMMON)
    for y,dashed,label in [(548,False,'요청 / 전달'),(590,True,'응답 / 반환')]:
        s.line([(54,y),(123,y)],color=COMMON,width=1.1,dashed=dashed)
        s.label(54,y+9,76,label,12,COMMON)
    s.label(54,743,76,'경계 A: 실선\n경계 B: 점선',11,COMMON)
    s.label(54,774,76,'차이: 경계선',11,COMMON)


def append_plate_legend(p):
    """Use the same key on Plate-based MAIN and supplementary diagrams."""
    from generate_dp_comparison_slides import Comparison
    legend=Comparison(41);legend.items=[]
    resolution_legend(legend,horizontal=True)
    for item in legend.items:
        data={k:v for k,v in item.items() if k not in ('id','kind')}
        if item['kind']=='rect':
            p.box(data['x'],data['y'],data['w'],data['h'],data['fill'],data['stroke'],
                  7 if data.get('rounded') else 0,data.get('dashed',False),thick=data.get('line_width',1.0)*4/3)
        elif item['kind']=='store':
            p._add('node',x=data['x'],y=data['y'],w=data['w'],h=data['h'],name=[],
                   type='store',color=COMMON,parent=None)
        elif item['kind']=='text':
            p.text(data['x'],data['y'],data['lines'],data['size'],data['color'],
                   data['align'],data['bold'],width=data['w'],leading=data['leading'])
        elif item['kind']=='line':
            p.line(data['points'],data['color'],data['dashed'],data['arrow'],1.3)


def graph41(s,x,side):
    g=ResolutionGraph(s,x)
    if side==0: resolution_legend(s)
    g.box('input',30,278,245,35,'Interaction Manager')
    g.box('model',650,278,195,35,'Model Access')
    g.text(30,252,530,'확정 User Turn / 새 Request',size=16)
    g.text(650,320,200,'공유 Omni 1벌 / 역할별 KV',size=14)
    g.box('adopt',30,667,265,39,'Request Controller')
    g.store('adopted',345,667,250,39,'State Store')
    g.box('policy',650,667,195,39,'Policy Manager',size=17)
    g.box('context',650,431,195,40,'Context Manager',size=17)
    g.box('tasks',650,549,195,40,'Task Manager',size=17)
    g.text(652,480,193,'화면 / 과거 정보 / 버전',size=14)
    g.text(652,598,193,'Task 후보 / 상태 / 버전',size=14)
    # One controller instance owns receipt, interpretation invocation and adoption.
    g.edge('input','adopt','L','L',via=[(15,295),(15,686)])
    g.text(30,319,300,'1 입력 접수 → 해석 요청',size=17)
    if side==0:
        g.group(30,394,565,211,'Request Interpreter')
        g.box('producer',50,438,230,42,'ReAct 해석 제어기',True,True,size=18)
        g.box('tools',340,438,235,42,'읽기 도구 실행기',True,True,size=18)
        g.box('check',50,553,230,38,'의미 제안 검증기',True,True,size=18)
        g.store('temp',340,549,235,43,'임시 해석 상태',different=True)
        g.edge('adopt','producer','R','T',td=110,via=[(318,686),(318,632),(608,632),(608,382),(275,382)])
        g.edge('producer','tools','R','L',sd=-9,td=-9)
        g.edge('tools','producer','L','R',sd=10,td=10,ret=True)
        g.text(291,431,49,'읽기',size=13)
        g.text(291,469,49,'정보',size=13)
        g.edge('producer','check','R','T',sd=16,via=[(300,475),(300,540),(165,540)])
        g.text(53,496,233,'2 모델의 의미 제안 / 부분 수정',size=14)
        g.edge('check','producer','L','L',via=[(40,572),(40,459)],ret=True)
        g.text(55,524,230,'검증 결과 반환 / 실패 시 재판단',size=14)
        g.edge('tools','temp',sd=30,td=30)
        g.edge('temp','tools',sd=-30,td=-30,ret=True)
        g.edge('producer','temp','R','T',sd=4,td=-60,via=[(325,463),(325,520),(397,520)])
        g.edge('temp','producer','T','B',sd=-90,via=[(367,543),(32,543),(32,488),(165,488)],ret=True)
        g.text(344,502,236,'해석과 조회 상태 / 보존 / 갱신',size=14)
        producer='producer';output='check'
    else:
        g.box('producer',30,394,565,40,'Request Interpreter',True,size=20)
        g.group(30,479,565,146,'Request Resolution Engine')
        g.box('check',50,520,230,38,'관계 결합기',True,True,size=18)
        g.box('tools',340,520,235,38,'미해결 항목 처리기',True,True,size=18)
        g.store('temp',340,582,235,32,'요청 해석 상태',different=True,size=16)
        g.edge('adopt','producer','R','T',via=[(318,686),(318,632),(608,632),(608,382),(312,382)])
        g.edge('producer','tools',sd=145,td=20,via=[(457,470),(477,470)])
        g.text(35,442,310,'2 유한 요청 틀: 목표 / 조건 / 관계 등',size=14)
        g.edge('tools','producer','R','R',via=[(584,539),(584,414)],ret=False,td=0)
        g.text(350,445,100,'부분 재해석\n요청 / 반환',size=12)
        g.edge('producer','tools','B','T',sd=245,td=90,via=[(557,448),(548,448)],ret=True)
        g.edge('tools','check','L','R',sd=-8,td=-8)
        g.edge('check','tools','R','L',sd=9,td=9,ret=True)
        g.text(54,566,275,'확인 후보 / 정보 / 버전 → 코드 결합',size=14)
        g.text(53,602,275,'미해결 / 충돌 → 추가 조회',size=14)
        g.edge('tools','temp',sd=25,td=25)
        g.edge('temp','tools',sd=-25,td=-25,ret=True)
        g.edge('check','temp','R','L',sd=16,td=-5,via=[(299,555),(299,593)])
        g.edge('temp','check','L','R',sd=6,td=18,via=[(309,604),(309,557)],ret=True)
        producer='producer';output='check'
    # Both requests and model results are explicit, rather than a return-only edge.
    call_offset=95 if side==0 else -35
    return_offset=105 if side==0 else 35
    px,py=g.port(producer,'T',call_offset)
    g.edge(producer,'model','T','L',sd=call_offset,td=-8,via=[(px,352),(629,352),(629,290)])
    g.edge('model',producer,'L','T',sd=9,td=return_offset,via=[(639,304),(639,372),(g.port(producer,'T',return_offset)[0],372)],ret=True)
    g.text(292,340,324,'모델 호출 → / ← 해석 결과',size=15)
    # Each executor talks directly to both owners. Providers return to that executor.
    ty=g.port('tools','R',-9)[1]
    g.edge('tools','context','R','L',sd=-9,td=-8,via=[(618,ty),(618,443)])
    g.edge('context','tools','L','R',sd=9,td=8,via=[(628,460),(628,g.port('tools','R',8)[1])],ret=True)
    g.edge('tools','tasks','R','L',sd=-2,td=-8,via=[(618,g.port('tools','R',-2)[1]),(618,561)])
    g.edge('tasks','tools','L','R',sd=9,td=14,via=[(628,578),(628,g.port('tools','R',14)[1])],ret=True)
    g.text(653,398,188,'허용 조회 → / ← 정보',size=14)
    # Adoption checks use separate outer paths, not the interpreter's read bus.
    g.edge('adopt','context','B','R',sd=90,td=-7,via=[(252,739),(852,739),(852,444)])
    g.raw([(852,562),(845,562)])
    g.edge('context','adopt','R','B',sd=8,td=103,via=[(858,459),(858,746),(265,746)],ret=True)
    g.raw([(845,577),(858,577)],ret=True,arrow=False)
    g.text(351,721,490,'현재 정보 / Task / 질문 연결 재확인 / 반환',size=14)
    # Proposal and failure routes return to the same controller.
    g.edge(output,'adopt')
    g.text(32,630,280,'3 의미 제안 / 확인 질문\n틀 밖 관계는 보류',size=14)
    g.edge('adopt','adopted','R','L',sd=-8,td=-8)
    g.edge('adopted','adopt','L','R',sd=9,td=9,ret=True)
    g.text(350,648,288,'채택 저장 → / ← 성공 / 실패',size=14)
    g.edge('adopt','policy','B','B',sd=-42,td=-42,via=[(120,714),(705,714)])
    g.edge('policy','adopt','B','B',sd=42,td=42,via=[(790,730),(205,730)],ret=True)
    g.text(33,718,257,'권한 검사 → / ← 허용 / 거부',size=14)
    g.text(31,759,810,'채택 후: Response Manager → Interaction Manager / Task Manager → Agent Gateway',size=16)
    g.text(31,780,810,'Component는 논리적 책임 경계 / 포함된 Module은 내부 구현 / 별도 프로세스 의미 없음',size=14)
    g.finish()


def graph42(s,x,side):
    g=ResolutionGraph(s,x)
    if side==0: resolution_legend(s)
    g.text(24,251,820,'Q1 “상반기로 할까요?”를 P1로 제시 → u2 “응, 상반기로”',size=16)
    for y,title,desc,end in [(278,'Conversation C1','입력과 실제 응답의 연결 유지',833),
                             (297,'Request R1 / R2 / R3','최초 위임 / Q1 답변 / 결과 수정',740),
                             (316,'Task T1','보고서 업무와 결과 유지',833),
                             (335,'Execution / Question','X1 완료 → X2 / Q1 제시 → 답변',815)]:
        g.text(24,y,230,title,size=15)
        g.raw([(268,y+16),(end,y+16)],arrow=False)
        g.text(281,y-1,555,desc,size=14)
    g.text(281,353,555,'강조 경계: Core/서비스 실행 영역 / 박스: 논리 Component',size=12)
    if side==0:
        g.group(20,367,825,346,'VIA Core',execution=True,side=side)
        g.text(292,376,540,'같은 실행 경계 / 관련 변경의 공동 확정',size=15)
    else:
        g.group(20,367,420,346,'대화 서비스',execution=True,side=side)
        g.group(490,367,355,346,'업무 서비스',execution=True,side=side)
        g.text(190,377,230,'독립 상태 / 실행 수명',size=14)
        g.text(650,377,190,'독립 상태 / 실행 수명',size=14)
    g.box('controller',40,411,235,36,'Request Controller',size=18)
    g.box('tasks',555,411,270,36,'Task Manager',size=18)
    g.box('interpreter',40,485,235,34,'Request Interpreter',size=17)
    g.box('response',40,559,235,36,'Response Manager',size=18)
    g.box('gateway',555,559,270,36,'Agent Gateway',size=18)
    g.text(43,450,300,'C1 / R2 / u2 → Q1 → T1 / P1 참조',size=14)
    g.text(558,452,265,'조회: T1 / X1 / Q1=OPEN,v7',size=14)
    # Same state owners and bounded semantic judgment in both options.
    g.edge('controller','tasks','R','T',sd=-9,td=-100,via=[(295,420),(295,401),(590,401)])
    g.edge('tasks','controller','T','R',sd=-85,td=-2,via=[(605,407),(285,407),(285,427)],ret=True)
    g.text(301,400,238,'질문 조회 → / ← Q1,v7',size=13)
    g.edge('controller','tasks','R','L',sd=10,td=10)
    g.text(302,428,240,'K1: Q1 답변=상반기 / v7',size=13)
    g.edge('tasks','controller','L','R',sd=-9,td=-9,via=[(533,420),(533,472),(292,472),(292,420)],ret=True)
    g.text(302,454,240,'외부 접수 / 질문 / 결과',size=14)
    g.edge('controller','interpreter',sd=-45,td=-45)
    g.edge('interpreter','controller',sd=45,td=45,ret=True)
    g.text(43,522,270,'u2 + 실제 제시 P1 → 답변 의미 제안',size=13)
    # Output content/admission and delivery/focus are distinct contracts.
    g.edge('controller','response','L','T',td=-60,via=[(30,429),(30,550),(97.5,550)])
    g.edge('response','controller','R','R',sd=-10,td=2,via=[(315,567),(315,431)],ret=True)
    g.text(321,506,218,'P2: 상반기 전달 확인\n게시 허용 / 실제 제시 반환',size=12)
    g.text(44,600,290,'P1→Q1 실제 제시 / P2 전달 원장',size=14)
    g.edge('tasks','gateway')
    g.edge('gateway','tasks','L','L',via=[(542,577),(542,429)],ret=True)
    g.text(558,490,265,'접수: Q1=ANSWERED,v8\nK1=전송 대기',size=14)
    g.text(558,600,271,'K1 외부 접수 / X1 관측 상태',size=14)
    if side==0:
        g.box('commit',325,621,220,31,'트랜잭션 관리자',True,True,size=17)
        g.store('saved',345,672,200,30,'State Store',size=16)
        g.edge('controller','commit','R','T',sd=16,td=-55,via=[(305,445),(305,613),(380,613)])
        g.edge('tasks','commit','L','T',sd=16,td=55,via=[(550,445),(550,613),(490,613)])
        g.edge('commit','saved',via=[(435,661),(445,661)])
        g.edge('saved','commit',sd=55,td=55,via=[(500,663),(490,663)],ret=True)
        g.text(43,631,275,'R2↔Q1↔T1 연결과 K1을\n관련 owner 변경으로 함께 확정',size=14)
        g.text(558,631,265,'성공: 둘 다 반영\n충돌 / 실패: 둘 다 미반영',size=14)
        g.edge('commit','controller','L','R',td=15,via=[(299,636.5),(299,444)],ret=True)
        g.edge('commit','tasks','R','L',td=15,via=[(552,636.5),(552,444)],ret=True)
    else:
        g.store('dsaved',40,672,380,30,'대화 상태 저장소',size=16)
        g.store('tsaved',510,672,315,30,'업무 상태 저장소',size=16)
        g.edge('controller','dsaved','R','T',sd=16,td=50,via=[(303,445),(303,661),(280,661)])
        g.edge('tasks','tsaved','L','T',sd=16,td=-105,via=[(501,445),(501,661),(562.5,661)])
        g.edge('tasks','controller','B','B',sd=-60,td=60,via=[(630,471),(460,471),(460,477),(217.5,477)],ret=True)
        # Explicit intermediate state, followed by reflection of the receipt.
        g.text(43,631,371,'선저장: R2=접수 대기 / K1\n후반영: K1 내부 접수 확인',size=14)
        g.text(515,631,312,'Q1 변경 + K1 + 내부 접수 결과\n업무의 로컬 transaction',size=14)
        g.text(322,486,215,'K1 내부 접수 결과 반환',size=13)
    # Response Manager owns the publication journal, not the Controller log.
    target='saved' if side==0 else 'dsaved'
    g.edge('response',target,'L','T',td=-100,via=[(34,577),(34,657),(g.port(target,'T',-100)[0],657)])
    g.box('interaction',40,737,235,32,'Interaction Manager',size=17)
    g.box('model',345,737,230,32,'Model Access',size=17)
    g.box('agent',600,737,245,32,'Downstream Agent',size=17)
    g.edge('interaction','controller','L','L',via=[(10,753),(10,429)])
    g.edge('response','interaction','L','T',sd=10,td=-30,via=[(24,587),(24,724),(127.5,724)])
    if side==0:
        g.edge('interaction','response','R','R',sd=7,td=10,via=[(284,760),(284,587)],ret=True)
    else:
        g.edge('interaction','response','R','L',sd=7,td=-10,via=[(286,760),(286,730),(28,730),(28,567)],ret=True)
    g.text(302,705,534,'release(P2,epoch) / receipt(P2,실제 전달 범위)',size=13)
    c1,c2,c3,c4=(318,323,310,312) if side==0 else (450,456,462,468)
    g.edge('interpreter','model','R','T',td=-70,via=[(c1,502),(c1,725),(390,725)])
    g.edge('model','interpreter','T','R',sd=-50,td=10,via=[(410,729),(c2,729),(c2,512)],ret=True)
    g.edge('response','model','R','T',sd=4,td=30,via=[(c3,581),(c3,719),(490,719)])
    g.edge('model','response','T','R',sd=50,td=10,via=[(510,723),(c4,723),(c4,587)],ret=True)
    g.edge('gateway','agent','R','T',td=-25,via=[(851,577),(851,719),(697.5,719)])
    g.edge('agent','gateway','T','R',sd=25,td=10,via=[(747.5,726),(858,726),(858,587)],ret=True)
    g.text(43,716,231,'u2 입력 / Text / Voice 실제 전달',size=13)
    g.text(347,772,230,'공유 Omni 1벌 / 같은 해석',size=13)
    g.text(603,772,245,'K1 전송 / 외부 접수 확인',size=13)
    g.text(43,772,235,'Voice Runtime: 음성 I/O 구현',size=12)
    # Check straight routes and unrelated actor interiors before export.
    for item in s.items[g.first:]:
        if item['kind']!='line': continue
        pts=[(xx-x,yy) for xx,yy in item['points']]
        for start,end in zip(pts,pts[1:]):
            assert start[0]==end[0] or start[1]==end[1],(side,item['id'],'non-orthogonal')
            for key,(nx,ny,nw,nh) in g.nodes.items():
                if any(nx<=px<=nx+nw and ny<=py<=ny+nh for px,py in (pts[0],pts[-1])): continue
                vertical=start[0]==end[0] and nx+1<start[0]<nx+nw-1 and max(min(start[1],end[1]),ny+1)<min(max(start[1],end[1]),ny+nh-1)
                horizontal=start[1]==end[1] and ny+1<start[1]<ny+nh-1 and max(min(start[0],end[0]),nx+1)<min(max(start[0],end[0]),nx+nw-1)
                assert not(vertical or horizontal),(side,item['id'],'crosses',key)
    g.finish()
    if not hasattr(s,'lifecycle_graphs'): s.lifecycle_graphs=[]
    s.lifecycle_graphs.append(g)


def graph43(s,x,side):
    g=Graph(s,x,BLUE if side==0 else PURPLE)
    semantic_start(g)
    g.text(25,326,560,'공통 읽기: Context Manager 정보 / Task Manager의 Task 후보',size=16)
    g.group(24,358,821,266,'Request Interpreter')
    if side==0:
        g.box('producer',156,411,554,48,'Integrated Semantic Interpreter',g.c,size=21)
        g.store('temp',268,535,328,53,'Interpretation Attempt State',size=18)
        g.edge('start','producer',sd=100,via=[(552,347),(433,347)])
        g.text(49,470,751,'F1 의도 / F2 Referent / F3 Task Association',size=19,color=g.c)
        g.text(49,497,751,'F4 Request 관계 / F5 정정 범위 / F6 처리 방향',size=19,color=g.c)
        g.edge('producer','temp')
        g.edge('temp','producer','L','L',via=[(135,562),(135,435)],ret=True)
        g.text(45,597,778,'이전 조건 유지와 부분 갱신 / 검색과 전문 보조 허용',size=17)
        g.edge('producer','model','R','B',via=[(827,435),(827,347),(742,347)],ret=True,color=MUTED)
        output='temp'
    else:
        names=[('intent','Request Intent\nInterpreter','F1 목표와 완료 조건'),
               ('referent','Referent Resolver','F2 표와 결론의 범위'),
               ('association','Request Association\nResolver','F3 Conversation / Task 연결'),
               ('relation','Request Relation\nInterpreter','F4 조건과 의존 관계'),
               ('revision','Request Revision\nInterpreter','F5 PDF 유지 / 결론→표'),
               ('handling','Request Handling\nInterpreter','F6 Delegation / 질문 / 대기')]
        g.box('producer',316,406,240,149,'Interpretation\nCoordinator',INK,size=20)
        g.store('temp',316,567,240,43,'Partial Interpretation Ledger',size=15)
        for i,(key,name,result) in enumerate(names):
            col=i//3;row=i%3;xx=44 if col==0 else 590;yy=406+row*69
            g.box(key,xx,yy,236,43,name,g.c,size=17)
            g.text(xx,yy+45,242,result,size=15,color=g.c)
            g.edge(key,'producer','R' if col==0 else 'L','L' if col==0 else 'R',sd=-9,td=(row-1)*51-9)
            g.edge('producer',key,'L' if col==0 else 'R','R' if col==0 else 'L',sd=(row-1)*51+9,td=9,ret=True)
        g.edge('start','producer',sd=100,via=[(552,347),(567,347),(567,401),(436,401)])
        g.edge('producer','temp')
        # The six producers, never the code Coordinator, own model-produced values.
        g.raw([(34,397),(852,397),(852,347),(742,347),(742,319)],ret=True,color=MUTED)
        for key in ['intent','referent','association']:
            xx,yy=g.port(key,'L')
            g.raw([(xx,yy),(34,yy),(34,397)],ret=True,color=MUTED,arrow=False)
        for key in ['relation','revision','handling']:
            xx,yy=g.port(key,'R')
            g.raw([(xx,yy),(852,yy),(852,397)],ret=True,color=MUTED,arrow=False)
        g.text(311,370,521,'각 F1~F6 생산자의 모델 호출 / 반환',size=15)
        g.text(44,611,788,'실선 제안 / 점선 원생산자 재판단 / Coordinator의 의미 덮어쓰기 금지',size=16)
        output='temp'
    g.text(31,631,810,'정정: F2와 F5 재판단 → F4 갱신 / Task Association은 입력 전제가 아닌 결과',size=17,color=g.c)
    semantic_finish(g,output,via=[(432 if side==0 else 436,678),(155,678)])


def graph44(s,x,side):
    # MAIN and presentation share the same ten owners and execution graph.
    from continuous_interaction_scene import draw_option, compact_legend
    if side==0:compact_legend(s)
    draw_option(s,x+15,276,.67,'A' if side==0 else 'B')


def graph45(s,x,side):
    from memory_context_scene import draw_option
    draw_option(s,x,260,.73,'A' if side==0 else 'B')
