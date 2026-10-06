"""Presentation scenes preserving the mechanisms of the 40s MAIN figures.

This is an editable comparison view, not a new Architecture or a sequence spec.
All coordinates are local to one option; no operational edge crosses options.
"""
from generate_dp_background_slides import INK, MUTED, BLUE, TEAL, RED, LINE
import unicodedata

PURPLE = '#7155A4'


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


def graph41(s,x,side):
    g=Graph(s,x,BLUE if side==0 else TEAL)
    semantic_start(g)
    # Containers precede all internal shapes and edges.
    if side==0:
        g.group(25,358,584,259,'Request Interpreter')
        g.box('producer',45,405,243,48,'ReAct 해석 제어기',g.c,size=20)
        g.box('tools',340,405,249,48,'읽기 도구 실행기',g.c,size=20)
        g.box('check',45,538,243,46,'의미 제안 검증기',g.c,size=20)
        g.store('temp',340,534,249,52,'임시 해석 상태',size=18)
        g.edge('start','producer',via=[(452,345),(167,345)])
        g.text(46,324,500,'1 모델의 읽기 선택과 의미 제안',g.c,size=18)
        g.edge('producer','tools','R','L',sd=-10,td=-10)
        g.edge('tools','producer','L','R',sd=11,td=11,ret=True)
        g.text(45,468,535,'2 허용 읽기 → 정보 반환 → 모델 재판단',g.c,size=17)
        g.edge('producer','check')
        g.text(57,502,280,'의미 제안 / 부분 수정',size=17)
        g.edge('tools','temp')
        g.edge('temp','producer','L','R',via=[(312,560),(312,449)],ret=True)
        g.text(343,594,245,'읽기 영수증과 원본 버전',size=16)
        g.edge('check','producer','L','L',via=[(34,561),(34,429)],ret=True)
        g.edge('producer','model','T','L',via=[(167,393),(622,393),(622,300)],ret=True,color=MUTED)
        g.text(390,369,210,'모델 호출과 반환',size=16)
        output='check'
    else:
        g.group(25,448,584,169,'Request Resolution Engine')
        g.box('producer',45,358,544,45,'Request Interpreter',g.c,size=20)
        g.box('tools',45,487,243,44,'미해결 항목 처리기',g.c,size=19)
        g.box('check',340,487,249,44,'관계 결합기',g.c,size=19)
        g.store('temp',153,560,328,43,'요청 해석 상태',size=17)
        g.edge('start','producer',via=[(452,345),(317,345)])
        g.text(48,324,510,'1 모델의 유한 요청 틀 생성',g.c,size=18)
        g.edge('producer','tools',via=[(317,437),(166,437)])
        g.text(45,414,529,'목표 / Referent 조건 / 관계 / 금지 / 미해결 항목',size=17)
        g.edge('tools','check','R','L')
        g.text(299,543,288,'2 코드의 조회 일정과 관계 결합',g.c,size=16)
        g.edge('tools','temp',via=[(166,549),(216,549)],td=-101)
        g.edge('check','temp',via=[(464,549),(418,549)],td=101)
        g.edge('tools','producer','L','L',via=[(34,509),(34,381)],ret=True)
        g.text(46,624,525,'필요한 부분의 모델 재해석 / 틀 밖 관계의 보류',size=17)
        g.edge('producer','model','R','L',via=[(623,381),(623,300)],ret=True,color=MUTED)
        output='check'
    g.box('context',637,448,208,41,'Context Manager',size=18)
    g.box('tasks',637,547,208,41,'Task Manager',size=18)
    g.text(638,494,208,'화면과 과거 정보\n동일 권한과 조회 범위',size=16)
    g.text(638,597,208,'Task 후보와 버전',size=16)
    g.edge('tasks','context',sp='T',tp='B',ret=True,color=MUTED)
    if side==0:
        g.edge('tools','context','R','L',via=[(620,429),(620,463)],td=-6)
    else:
        g.edge('tools','context','B','L',via=[(166,539),(620,539),(620,463)],td=-6)
    g.edge('context','tools' if side==0 else 'check','L','R',via=[(628,478),(628,527 if side else 441)],sd=10,td=20 if side else 12,ret=True)
    g.text(640,408,205,'허용 읽기 / 버전 반환',size=16)
    semantic_finish(g,output,via=[(166 if side==0 else 464,644),(155,644)])


def graph42(s,x,side):
    g=Graph(s,x,BLUE if side==0 else TEAL)
    g.text(24,253,820,'C1 최초 R1 보고서와 R2 메일 → 후속 R3 “보고서만 상반기로”',size=17,color=INK)
    # Logical lifetimes repeat in each option, distinct from process lifetimes below.
    rows=[('Conversation C1',285,827,'대화 기록과 연결 유지'),
          ('Request R1 / R2 / R3',310,410,'각 입력의 해석과 접수 완료'),
          ('Task T1 / T2',335,827,'보고서 T1 수정 / 메일 T2 계속'),
          ('Agent Execution',360,660,'T1: X1 종료 → X2 / T2: Y1'),
          ('질문 Q1',385,496,'제시 → 답변 연결 → 해소')]
    for name,y,end,desc in rows:
        g.text(24,y,220,name,size=17)
        start=260
        g.raw([(start,y+20),(end,y+20)],arrow=False,color=g.c)
        if name.startswith('Request'):
            g.text(280,y-2,175,'R1 / R2 접수 완료',size=16,color=g.c)
            g.raw([(465,y+20),(615,y+20)],arrow=False,color=g.c)
            g.text(627,y,217,'R3 후속 수정',size=16)
        else:g.text(280,y-2,end-start-24,desc,size=16,color=g.c)
    if side==0:
        g.group(24,427,819,293,'VIA Core')
        g.text(238,438,585,'동일 실행 경계 / 모듈별 처리 책임',size=17)
        g.box('dialogue',44,477,260,42,'대화 처리기',g.c,size=20)
        g.box('work',528,477,292,42,'업무 관리기',g.c,size=20)
        g.box('commit',342,570,225,42,'트랜잭션 관리자',g.c,size=19)
        g.store('state',302,652,307,48,'통합 상태 저장소',size=18)
        g.text(47,529,300,'C1 / R1~R3 / Q1 연결',size=17)
        g.text(532,529,285,'T1 / T2 / Q1 / 명령',size=17)
        g.edge('dialogue','work','R','L')
        g.text(330,467,181,'1 검증 변경 제안',size=17)
        g.edge('dialogue','commit',via=[(174,559),(454,559)])
        g.edge('work','commit',via=[(674,629),(454,629)],sp='B',tp='B')
        g.edge('commit','state')
        g.text(44,589,279,'2 답변 연결과 명령의\n하나의 로컬 확정',g.c,size=18)
        g.text(634,582,192,'모델과 네트워크\n대기 밖 트랜잭션',size=16)
        g.edge('state','work',sp='R',tp='B',via=[(678,676),(678,519)],ret=True)
        g.text(639,642,190,'3 저장 후 전송',g.c,size=17)
        sender='work';route=[(851,498),(851,722)]
    else:
        g.group(24,427,354,293,'대화 서비스')
        g.group(440,427,403,293,'업무 서비스')
        g.text(203,440,172,'독립 실행 경계',size=16)
        g.text(642,440,199,'독립 실행 경계',size=16)
        g.box('dialogue',44,477,314,42,'대화 처리기',g.c,size=20)
        g.box('work',459,477,364,42,'업무 관리기',g.c,size=20)
        g.store('dialogue-state',44,642,314,57,'대화 상태 저장소',size=18)
        g.store('work-state',459,642,364,57,'업무 상태 저장소',size=18)
        g.text(48,529,308,'C1 / R1~R3 / Q1 연결',size=17)
        g.text(464,529,345,'T1 / T2 / Q1 / command ID',size=17)
        g.edge('dialogue','dialogue-state')
        g.edge('work','work-state')
        g.edge('dialogue','work','R','L')
        g.text(329,468,122,'1 명령',size=17)
        g.edge('work-state','dialogue','L','B',via=[(413,671),(413,574),(201,574)],ret=True)
        g.text(467,566,342,'2 업무의 로컬 접수와 확정',g.c,size=18)
        g.text(47,589,300,'3 내부 접수의 대화 반영\n같은 command ID의 재시도',g.c,size=17)
        g.text(465,609,356,'외부 Agent 접수와 별도 사실',size=16)
        sender='work';route=[(851,498),(851,722)]
    g.box('gateway',42,732,210,39,'Agent Gateway',size=18)
    g.box('agent',349,732,247,39,'Downstream Agent',size=18)
    g.box('model',668,732,175,39,'Model Access',size=17)
    g.edge(sender,'gateway','R','R',via=route+[(266,722),(266,752)])
    g.edge('gateway','work','T','R',via=[(147,724),(838,724),(838,510)],td=12,ret=True,color=MUTED)
    g.edge('gateway','agent','R','L',sd=-8,td=-8)
    g.edge('agent','gateway','L','R',sd=10,td=10,ret=True,color=MUTED)
    g.text(268,725,92,'명령',size=16)
    g.text(264,771,380,'외부 접수 / X1 질문 / X2 결과',size=16)
    g.text(669,772,175,'공유 Omni 1벌',size=16)
    g.text(43,715,230,'Core 소속' if side==0 else '업무 서비스 소속',size=15)
    g.finish()


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
    g=Graph(s,x,BLUE if side==0 else TEAL)
    g.box('capture',24,272,242,37,'Interaction Manager',size=18)
    g.box('gate',331,272,242,37,'Input Control Gate',size=18)
    g.box('task',638,272,205,37,'Task Manager',size=18)
    g.edge('capture','gate','R','L')
    g.edge('gate','task','R','L')
    g.text(29,315,579,'새 발화 u2: 즉시 output fence / 미전송 효과 hold',size=17,color=INK)
    g.text(641,315,202,'확인된 Q1 / r1',size=17)
    g.text(29,251,809,'P1 설명 중 u2 “잠깐, 표부터” + 메일 Task의 Q1 질문',size=17)
    if side==0:
        g.group(277,364,320,259,'Interaction Orchestrator')
        g.group(638,364,205,305,'Response Manager')
        g.box('dispatch',296,421,282,47,'Dialogue Dispatcher',g.c,size=19)
        g.store('progress',296,543,282,56,'Dialogue Progress State',size=17)
        g.box('rc',24,421,242,47,'Request Controller',size=18)
        g.box('voice',24,572,242,39,'Voice Runtime',size=18)
        g.box('compose',655,421,171,47,'Response\nComposer',size=17)
        g.box('publish',655,543,171,47,'Publication\nControl',size=17)
        g.store('outbox',655,616,171,39,'Publication Outbox',size=15)
        g.edge('gate','dispatch',via=[(452,350),(437,350)])
        g.edge('task','dispatch',via=[(742,345),(611,345),(611,403),(487,403)],td=50)
        g.edge('dispatch','rc','L','R',sd=-10,td=-10)
        g.edge('rc','dispatch','R','L',sd=11,td=11,ret=True)
        g.edge('dispatch','compose','R','L',sd=-10,td=-10)
        g.edge('compose','dispatch','L','R',sd=11,td=11,ret=True)
        g.text(29,480,565,'1 의미와 후보의 반환 회수 / 다음 처리의 재배정',size=17,color=g.c)
        g.edge('dispatch','progress',sd=-55,td=-55)
        g.edge('progress','dispatch','T','B',sd=55,td=55,ret=True)
        g.text(300,513,273,'대기 ID / revision / 후보',size=16)
        g.edge('dispatch','publish','R','L',via=[(616,445),(616,566)],sd=18)
        g.text(638,492,201,'2 현재 검사와 게시',size=17)
        g.edge('publish','outbox')
        g.edge('voice','rc','T','B')
        g.text(27,543,241,'승인 Direct 후보 통과',size=16)
        g.text(299,609,294,'후속 처리의 중앙 상태 유지',size=16)
        g.box('turn',28,693,230,37,'Turn-Taking Control',size=17)
        g.box('channel',332,693,209,37,'Channel I/O',size=18)
        g.store('playback',619,693,208,44,'Playback State',size=17)
        g.edge('publish','turn','B','R',via=[(740,680),(272,680),(272,711)])
        g.edge('playback','dispatch','T','L',via=[(722,675),(284,675),(284,458)],td=13,ret=True)
        g.text(331,754,493,'실제 전달 후 receipt → 게시 상태와 대기 연결',size=17,color=g.c)
    else:
        g.group(24,354,819,304,'Reactive Interaction Runtime')
        g.box('input',43,403,245,42,'Input Resolution Stage',g.c,size=17)
        g.store('iw',43,462,245,39,'Input Window',size=16)
        g.box('notice',581,403,242,42,'Task Notice Stage',g.c,size=18)
        g.store('nw',581,462,242,39,'Notice Window',size=16)
        g.box('rc',43,536,245,44,'Request Controller',size=18)
        g.box('voice',43,605,245,39,'Voice Runtime',size=18)
        g.box('compose',581,536,242,44,'Response Composer',size=18)
        g.box('join',326,536,221,44,'Publication Join',g.c,size=18)
        g.store('pw',326,601,221,40,'Publication Window',size=16)
        g.edge('gate','input',via=[(452,343),(165,343)])
        g.edge('task','notice',via=[(741,344),(702,344)])
        g.edge('input','iw')
        g.edge('iw','rc')
        g.edge('notice','nw')
        g.edge('nw','compose')
        g.edge('rc','join','R','L',sd=-9,td=-9)
        g.edge('rc','compose','B','L',sd=100,via=[(265,591),(567,591),(567,558)])
        g.edge('voice','rc','T','B')
        g.text(586,585,237,'승인 Direct 후보의 재생성 생략',size=15)
        g.edge('compose','join','L','R',sd=-9,td=-9)
        g.edge('join','pw')
        g.text(37,510,510,'1 typed 사건의 직접 활성화',size=17,color=g.c)
        g.text(583,510,239,'Response Manager',size=17)
        g.edge('join','input','T','R',via=[(436,386),(310,386),(310,434)],td=10,ret=True)
        g.text(342,406,216,'credit / cancel',size=17,color=g.c)
        g.text(330,452,225,'현재 admission\n원본 버전 / 과거 receipt',size=16)
        g.box('publish',581,680,242,45,'Publication Control',size=18)
        g.store('outbox',581,735,242,39,'Publication Outbox',size=16)
        g.box('turn',24,680,244,40,'Turn-Taking Control',size=17)
        g.box('channel',330,680,218,40,'Channel I/O',size=18)
        g.store('playback',330,735,218,39,'Playback State',size=16)
        g.edge('join','publish','R','T',via=[(559,558),(559,667),(702,667)])
        g.text(581,651,258,'2 Response Manager의 현재 검사',size=16)
        g.edge('publish','outbox')
        g.edge('publish','turn','L','R',via=[(560,702),(560,669),(282,669),(282,700)])
        g.edge('channel','playback')
        g.edge('playback','join','T','L',via=[(315,763),(315,566)],td=8,ret=True)
        g.text(25,735,267,'실제 전달 후 snapshot\n자기 receipt의 선행 대기 금지',size=16)
    g.edge('turn','channel','R','L')
    if side==0:g.edge('channel','playback','R','L')
    if side==0:
        g.edge('channel','publish','T','R',via=[(437,681),(852,681),(852,577)],td=11,ret=True,color=MUTED)
    else:
        g.edge('channel','publish','T','R',via=[(439,676),(852,676),(852,714)],td=12,ret=True,color=MUTED)
    g.text(29,663 if side==0 else 651,514,'Interaction Manager / 표시와 재생의 실제 경계',size=16)
    g.text(30,775,804,'실제 Q1 제시 확인 후 Publication Control → Request Controller의 focus 갱신',size=16)
    g.finish()


def graph45(s,x,side):
    g=Graph(s,x,BLUE if side==0 else TEAL)
    g.text(25,253,819,'동일 이력: D2 보고서 → R7 “짧은 문장, 결론 먼저” → D3 수정 → P4 전달',size=17,color=INK)
    for key,xx,name,record in [('source-request',25,'Request Controller','R7 원문과 정정 관계'),
                             ('source-task',310,'Task Manager','D2 → D3 확정 결과'),
                             ('source-response',595,'Response Manager','P4 실제 전달 기록')]:
        g.box(key,xx,284,250,39,name,size=18)
        g.text(xx+3,329,248,record,size=17)
    g.group(25,392,819,242,'Context Manager')
    g.text(26,359,808,'원본 owner 유지 / 수정과 삭제 및 권한 철회',size=17)
    if side==0:
        g.box('composer',311,454,248,43,'Context Composer',g.c,size=19)
        g.store('cache',46,565,260,47,'Source View Cache',size=18)
        g.store('current',584,565,240,47,'Request Evidence Set',size=17)
        for key in ['source-request','source-task','source-response']:
            xx=g.port(key,'B')[0]
            g.edge('composer',key,'T','B',via=[(421,381),(xx-9,381)],sd=-14,td=-9)
            g.edge(key,'composer','B','T',via=[(xx+9,441),(449,441)],sd=9,td=14,ret=True)
        g.text(43,431,795,'1 R9의 범위 읽기 / 원문과 버전 및 누락 범위 반환',size=17,color=g.c)
        g.edge('composer','cache','L','T',via=[(176,475)])
        g.edge('cache','composer','R','B',via=[(435,589)],ret=True)
        g.edge('composer','current','R','T',via=[(704,475)])
        g.text(47,515,275,'원본별 요약과 색인 재사용',size=17)
        g.text(585,515,238,'2 이번 Request에 결합',size=17,color=g.c)
        g.text(48,615,770,'원본 변경의 부분 무효화 / cache 적중 시 재조합 축소',size=16)
        g.edge('source-request','cache','L','L',via=[(12,304),(12,589)],color=MUTED)
        output='current'
    else:
        g.box('publisher',45,455,252,42,'Memory Publisher',g.c,size=19)
        g.store('repo',339,451,280,55,'Episodic Memory\nRepository',size=17)
        g.box('reader',646,455,177,42,'Evidence Reader',g.c,size=17)
        for key in ['source-request','source-task','source-response']:
            xx=g.port(key,'B')[0]
            g.edge(key,'publisher','B','T',sd=10,td=10,via=[(xx+10,388),(181,388)])
            g.edge('publisher',key,'T','B',sd=-10,td=-10,via=[(161,379),(xx-10,379)],ret=True)
        g.text(46,431,755,'1 변경 알림 → 범위 읽기 → 원문과 버전 검사 → 관계 게시',size=17,color=g.c)
        g.edge('publisher','repo','R','L')
        g.edge('reader','repo','L','R',sd=-9,td=-9)
        g.edge('repo','reader','R','L',sd=11,td=11,ret=True)
        g.text(342,513,284,'정정 / 결과 개정 / 실제 전달\n출처와 coverage / 지속 상태',size=16,color=g.c)
        g.text(646,517,177,'2 유효한 관계 조회',size=16,color=g.c)
        g.edge('reader','publisher','B','B',via=[(734,568),(171,568)],ret=True)
        g.text(45,577,778,'미게시 범위의 생산 요청 / 충돌 재검증 / 표현 밖 관계의 미지원',size=17)
        for key in ['source-request','source-task','source-response']:
            xx=g.port(key,'T')[0]
            g.edge('reader',key,'R','T',via=[(855,476),(855,276),(xx,276)],color=RED)
            g.edge(key,'reader','T','R',sd=8,td=8,via=[(xx+8,280),(849,280),(849,484)],color=RED,ret=True)
        g.text(45,607,778,'모든 owner의 버전과 권한 재확인 / 수정과 삭제 즉시 사용 차단',size=16,color=RED)
        output='reader'
    g.box('current-request',25,667,260,40,'Request Controller',size=18)
    g.box('interpreter',349,667,274,40,'Request Interpreter',size=18)
    g.store('user-memory',668,664,176,47,'User Memory',size=17)
    g.edge('current-request','composer' if side==0 else 'reader','T','B',via=[(155,636),(435 if side==0 else 734,636)])
    g.edge(output,'current-request','B','T',via=[(704 if side==0 else 754,663),(176,663)],sd=0 if side==0 else 20,td=21,ret=True)
    g.text(31,641,650,'R9 조회 / Context + 출처 + 버전 + 누락 범위 반환',size=17,color=g.c)
    g.edge('current-request','interpreter','R','L')
    g.text(349,719,291,'현재 Referent와 Task의 판단',size=17)
    g.text(670,718,174,'Context Manager 소유\n명시 저장과 삭제',size=14)
    g.edge('current-request','user-memory','R','L',via=[(304,687),(304,738),(647,738),(647,687)],ret=True,color=MUTED)
    g.text(27,756,804,'사용과 채택: 현재 권한과 버전 검사 → State Store / Response 또는 Delegation',size=17)
    g.text(27,778,804,'공통: Model Access와 공유 Omni 1벌 / B의 관계 생산 session과 KV 추가',size=16)
    g.finish()
