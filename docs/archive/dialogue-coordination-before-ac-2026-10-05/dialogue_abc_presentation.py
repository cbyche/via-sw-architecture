"""04-33: three semantic responsibility arrangements, not three model replicas.

One scene is rendered as SVG and editable draw.io by the existing comparison
generator. The main plate is deliberately a single 16:9 presentation slide.
"""
from stage4_diagram_design import Plate, INK, MUTED, BLUE, GREEN, LINE

PURPLE = '#7446A4'
TITLE = '사용자 의도를 정확히 이해하려면, 판단 책임을 어떻게 나눌까?'


class Slide(Plate):
    def __init__(self, slug, title, height=1440):
        super().__init__(slug, '04-33', title, '', height=height)
        self.items = []
        self.serial = 0
        self.box(0, 0, 2560, 9, INK, 'none', 0)
        self.text(64, 25, 'VIA / 33 / 판단 책임의 분해', 24, MUTED, bold=True)
        self.text(2496, 25, '설계 탐색 / 미선정 / 미측정', 24, MUTED, 'right')
        self.text(64, 73, title, 43, bold=True)

    @staticmethod
    def text_geometry(i):
        size = 27 if i['w'] >= 290 else 24
        leading = size * 1.25
        return i['y']+(i['h']-len(i['name'])*leading)/2-2+(7 if i['type']=='store' else 0),size,leading

    def svg(self):
        svg = super().svg().replace('font-size="23"', 'font-size="27"')
        marker = f'<marker id="a{PURPLE[1:]}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M1 1L9 5L1 9" fill="none" stroke="{PURPLE}" stroke-width="1.6"/></marker>'
        return svg.replace('</defs>', marker+'</defs>')

    def drawio(self):
        return super().drawio().replace('fontSize=23;', 'fontSize=27;')

    def arrow(self,a,b,label='',at=None,sp='B',tp='T',via=(),color=INK,ret=False,sd=0,td=0,size=24):
        self.edge(a,b,sp=sp,tp=tp,via=via,color=color,ret=ret,sd=sd,td=td)
        if label:
            self.label(*at,label,color,size=size)


def structure():
    p=Slide('choice33-structure',TITLE)
    p.text(64,139,'같은 입력  “보고서는 PDF로 하고, 메일에는 그 보고서의 결론을 넣어줘.”',30,bold=True)
    p.text(64,184,'다음 정정  “메일에는 결론 대신 표만 넣어줘.”   →   세 안 모두 PDF 유지, 메일의 내용 조건만 교체',28)
    p.text(64,231,'공통 근거: Interaction Manager의 발화/화면 + Context Manager의 관련 자료 + Task Manager의 확인된 질문/결과',25,MUTED)
    p.line([(64,278),(2496,278)],LINE,arrow=False)
    for x,c,name,sub in [(64,BLUE,'A  통합 판단','관련 맥락을 모아 한 주체가 의미를 생산'),
                         (888,GREEN,'B  Task별 지속 판단','공통 대화 + 생성된 Task마다 판단 담당'),
                         (1712,PURPLE,'C  기능별 전문 판단','대상 / 업무 연결 / 관계를 나누어 생산')]:
        p.text(x,295,name,35,c,bold=True)
        p.text(x,346,sub,25,c)
    for x in (864,1688):
        p.line([(x,294),(x,980)],LINE,arrow=False)

    # A: one semantic producer, with strong retrieval and preserved conditions.
    for x in (110,940,1770):
        p.text(x,377,'코드 조정',20,MUTED)
    p.component('Arc',100,402,710,64,'Request Controller')
    p.component('Ajudge',130,555,650,66,'통합 요청 판단자',BLUE)
    p.arrow('Arc','Ajudge','1  원문 + 관련 맥락',(145,480),color=BLUE,sd=-200,td=-200)
    p.arrow('Ajudge','Arc','2  PDF + 결론 참조\n두 요청의 관계',
            (470,480),sp='T',tp='B',sd=200,td=200,color=BLUE,ret=True)
    p.node('Astate',170,747,570,82,'대화 / 요청 / 조건 기록',kind='store',color=BLUE)
    p.arrow('Arc','Astate','관련 기록 조회',(140,675),sp='L',tp='L',via=[(90,419),(90,773)],sd=-15,td=-15,color=BLUE)
    p.arrow('Astate','Arc','조건/질문 반환',(140,712),sp='L',tp='L',via=[(70,803),(70,449)],sd=15,td=15,color=BLUE,ret=True)
    p.text(175,850,'기록 소유: Request Controller',25,MUTED)
    p.text(108,886,['정정: 메일 조건만 바꾼 의미 제안','Task별 검색 / 조건 보존 / 부분 수정 가능'],26,BLUE,leading=34)
    p.text(115,963,'아래 채택도 같은 Request Controller',22,MUTED)

    # B: coordinator routes; Task actors write local proposals; common judge
    # handles no-Task and pre-Task semantics. No actor is a downstream executor.
    p.component('Bcoord',930,402,710,64,'Dialogue Coordinator',GREEN)
    p.component('Br',962,590,290,64,'보고서 대화 처리자',GREEN)
    p.component('Bm',1310,590,290,64,'메일 대화 처리자',GREEN)
    p.arrow('Bcoord','Br','1  원문/보고서 근거',(968,488),sd=-198,td=-20,color=GREEN)
    p.arrow('Bcoord','Bm','1  원문/메일 근거\n3  보고서의 결론 참조',(1325,480),sd=150,td=-20,color=GREEN)
    p.arrow('Br','Bcoord','2  PDF / 결론 참조',(968,549),sp='T',tp='B',sd=20,td=-158,color=GREEN,ret=True)
    p.arrow('Bm','Bcoord','2  필요 참조 / 메일 제안',(1320,549),sp='T',tp='B',sd=20,td=190,color=GREEN,ret=True)
    p.node('Brs',967,753,280,75,'보고서 조건/질문',kind='store',color=GREEN)
    p.node('Bms',1315,753,280,75,'메일 조건/질문',kind='store',color=GREEN)
    for k in ('r','m'):
        p.arrow('B'+k,'B'+k+'s','',sd=-25,td=-25,color=GREEN)
        p.arrow('B'+k+'s','B'+k,'',sp='T',tp='B',sd=25,td=25,color=GREEN,ret=True)
    p.label(985,666,'Task 수명 동안 각 담당이 조건/질문 보존',GREEN,size=24)
    p.label(985,707,'4  채택 후 저장 ↕ 이전 조건/질문 읽기',GREEN,size=24)
    p.component('Bcommon',960,890,640,64,'Common Dialogue Interpreter',GREEN)
    p.arrow('Bcoord','Bcommon','Task 없음 / 생성 전 / 교차 관계',(967,849),sp='L',tp='L',via=[(910,434),(910,922)],color=GREEN)
    p.arrow('Bcommon','Bcoord','',sp='R',tp='R',via=[(1665,922),(1665,434)],color=GREEN,ret=True)
    p.text(972,963,'정정: 메일 담당이 내용 교체 / PDF 조건 유지',24,GREEN)

    # C: separate functional producers; relation producer combines by reference,
    # never silently rewrites the other two producers' decisions.
    p.component('Ccoord',1760,402,680,64,'Semantic Coordinator',PURPLE)
    p.component('Cref',1775,590,295,64,'대상 해석 담당',PURPLE)
    p.component('Ctask',2130,590,295,64,'업무 연결 담당',PURPLE)
    p.arrow('Ccoord','Cref','1  원문/자료 후보',(1780,488),sd=-197.5,td=-20,color=PURPLE)
    p.arrow('Ccoord','Ctask','1  원문/Task 후보',(2140,488),sd=157.5,td=-20,color=PURPLE)
    p.arrow('Cref','Ccoord','2  결론의 대상/범위',(1780,549),sp='T',tp='B',sd=20,td=-157.5,color=PURPLE,ret=True)
    p.arrow('Ctask','Ccoord','2  보고서/메일 연결',(2140,549),sp='T',tp='B',sd=20,td=197.5,color=PURPLE,ret=True)
    p.component('Crel',1780,746,645,64,'관계 해석 담당',PURPLE)
    p.arrow('Ccoord','Crel','3  원문 + 대상/업무 제안',(1790,688),sp='L',tp='L',via=[(1738,434),(1738,778)],color=PURPLE)
    p.arrow('Crel','Ccoord','4  PDF / 결론→메일 의존 / 교체 범위',(1780,826),sp='R',tp='R',via=[(2470,778),(2470,434)],color=PURPLE,ret=True)
    p.node('Cstate',1840,896,525,65,'요청별 부분 제안 / 의존 원장',kind='store',color=PURPLE)
    p.arrow('Ccoord','Cstate','',sp='R',tp='R',via=[(2490,419),(2490,913.5)],sd=-15,td=-15,color=PURPLE)
    p.arrow('Cstate','Ccoord','',sp='R',tp='R',via=[(2510,943.5),(2510,449)],sd=15,td=15,color=PURPLE,ret=True)
    p.text(1790,866,'조정자가 기록/조회, 의미 값은 생산자만 변경',23,PURPLE)
    p.text(1790,965,'정정: 표 대상 + 메일 연결 + 교체 관계를 결합',24,PURPLE)

    # A single shared adoption boundary; the three columns are alternatives,
    # not three simultaneously deployed pipelines.
    p.component('guard',100,1028,2330,64,'Request Controller',INK)
    for key,x in [('Arc',80),('Bcoord',876),('Ccoord',1700)]:
        sx,sy=p.port(key,'L')
        p.arrow(key,'guard','',sp='L',tp='T',td=x+150-1265,via=[(x,sy),(x,1008),(x+150,1008)],color=INK)
    p.label(165,996,'각 안의 의미 제안 → 현재 입력/근거/질문/권한 검사 → 채택 또는 관련 담당 재판단/사용자 확인',INK,size=24)
    p.text(115,1105,'채택: State Store에 의미/상태/명령 의도 저장 → 담당 통지.   미채택 제안은 실행·게시 금지.   Task 사실의 원본은 Task Manager.',24)
    p.text(115,1140,'위임: Task Manager → Agent Gateway ↔ 외부 Downstream Agent   |   직접 답변/질문: Response Manager ↔ Interaction Manager',24)
    p.line([(64,1190),(2496,1190)],LINE,arrow=False)
    for x,c,lines in [
        (64,BLUE,['강점 후보: 교차 맥락 이해 / 짧은 처리 경로','대가: 맥락 누락·혼입 / 통합 판단 부담','V-01~03: 전체 관계   V-04~06: 조정 절약']),
        (888,GREEN,['강점 후보: 긴 Task의 조건·질문 이어가기','대가: 참가 누락 / 참조 왕복 / 지속 상태','V-01~03: 지역 연속성   V-08: Task 규칙']),
        (1712,PURPLE,['강점 후보: 기능별 집중 판단 / 오류 구분','대가: 부분 판단 불일치 / 추가 추론·결합','V-01~03: 전문화   V-08/09: 변경·분석'])]:
        p.text(x,1208,lines,25,c,leading=36)
    p.text(64,1325,'모든 판단자: Model Access ↔ 공유 on-device Omni 1벌. 역할별 Context/KV와 호출 비용은 별도. 입력 수신·ASR은 계속.',25,bold=True)
    p.text(64,1371,'네모 = Component 이름 / 원통 = 상태 이름. 색 = A/B/C 구분, 우열 아님. 실선 = 요청/전달, 점선 화살표 = 반환. 논리 분해이며 process 분리가 아님.',22,MUTED)
    p.text(64,1406,'B: Task는 외부에 위임한 사용자 목표. 앱별 담당 아님. C: 충돌은 원 생산자가 재판단. 같은 Omni의 역할 분리가 정확성 향상을 보장하지 않음.',21,MUTED)
    return p


def event():
    p=Slide('choice33-event','같은 입력과 정정, 세 구조에서 누가 어떤 의미를 만드는가?',height=1960)
    p.text(64,145,'공통 사례: 보고서 Task R / 메일 Task M / 확인된 보고서 v4. 결론이 아직 없으면 의존 대기하며 내용을 만들지 않는다.',26)
    groups=[
        ('A',BLUE,['Request\nController','통합 요청\n판단자','State\nStore'],[
            (0,1,'1  원문 + R/M 조건·질문·자료 근거',False),
            (1,0,'PDF / M에 R@4 결론 / 의존 관계',True),
            (0,2,'2  현재 검사 후 의미·변경·명령 의도 저장',False),
            (2,0,'채택 결과 → 공통 인계/게시',True),
            (0,1,'3  “메일에는 결론 대신 표만” + 이전 조건',False),
            (1,0,'M 내용만 표로 교체 / R PDF 유지',True),
            (0,2,'4  옛 미전송 의도 무효화 / 새 변경 채택',False),
            (0,1,'5  새 근거 충돌: 의존한 의미만 재판단',False),
            (1,0,'새 제안 또는 필요한 확인 질문',True),
            (0,0,'6  모호한 “그거”: 후보 유지 / 확인',False)]),
        ('B',GREEN,['Dialogue\nCoordinator','보고서\n대화 처리자','메일\n대화 처리자'],[
            (0,1,'1  원문/버전 + 보고서 맥락',False),
            (0,2,'같은 원문/버전 + 메일 맥락',False),
            (1,0,'2  PDF 변경 / 확인된 R@4 결론 참조',True),
            (2,0,'메일 변경 / 필요한 결론 참조',True),
            (0,2,'3  참조 전달 → 메일 담당의 재제안',False),
            (0,0,'4  공통 검사/저장 → 각 담당에 채택 통지',False),
            (0,2,'5  같은 정정 → 표 참조로 내용 교체',False),
            (0,1,'필요한 표 조회 / R PDF는 유지',False),
            (1,0,'6  새 결과/질문 → 지역 제안과 변경 통지',True),
            (0,0,'“그거” 경쟁: 참가 확대/관계 재판단/확인',False)]),
        ('C',PURPLE,['Semantic\nCoordinator','대상 해석\n담당','업무 연결\n담당','관계 해석\n담당'],[
            (0,1,'1  원문 + 자료/화면 근거',False),
            (0,2,'원문 + Task 후보/조건',False),
            (1,0,'2  결론의 대상/범위 제안',True),
            (2,0,'R/M 연결의 별도 제안',True),
            (0,3,'3  원문 + 두 제안 + 이전 조건',False),
            (3,0,'R PDF / 결론→M 의존 관계',True),
            (0,0,'4  제안 참조를 결합 / 공통 검사·저장',False),
            (0,1,'5  같은 정정 → 표 대상 확인',False),
            (1,0,'표 참조 / 기존 M 연결은 유효하면 재사용',True),
            (0,3,'유효 부분 제안 + 교체할 내용 조건',False),
            (3,0,'결론 의존 제거 / 표 의존 / PDF 유지',True),
            (0,1,'6  근거 충돌 → 해당 생산자의 재판단',False),
            (0,0,'미해결 관계는 질문 / 중앙의 임의 덮기 금지',False)])]
    for j,(letter,color,names,steps) in enumerate(groups):
        x=64+j*824
        p.text(x,215,letter+'  같은 사건의 생산 책임',31,color,bold=True)
        xs=[x+126,x+382,x+638] if len(names)==3 else [x+88,x+272,x+456,x+640]
        w=226 if len(names)==3 else 166
        for xx,name in zip(xs,names):
            p.box(xx-w/2,276,w,90,'white',color,4)
            p.text(xx,291,name,25 if len(names)==3 else 23,color,'center',True,leading=31)
            p.line([(xx,366),(xx,1510)],LINE,dashed=True,arrow=False)
        for i,(a,b,label,ret) in enumerate(steps):
            y=446+i*(109 if len(steps)==10 else 82)
            p.text(x+6,y-47,label,23,color)
            pts=[(xs[a],y),(xs[b],y)] if a!=b else [(xs[a],y),(xs[a]+60,y),(xs[a]+60,y+23),(xs[a],y+23)]
            p.line(pts,color,dashed=ret,width=2.5)
    p.text(64,1570,'공통 예외',30,bold=True)
    p.text(64,1620,[
        '새 입력: 이전 미전송 요청을 보류. 취소/정정의 대상과 영향을 확인한 뒤 유효한 변경만 채택한다.',
        '전송 후: 실제 Agent 접수/실행을 확인한다. 지역 채택이나 재시작만으로 취소 완료·외부 rollback을 주장하지 않는다.',
        '철회: 관련 근거, 부분 제안, Task 담당 맥락과 모델 세션의 신규 사용을 차단한다. 이미 제공한 정보는 소급 회수하지 않는다.',
        '실패/재시작: 내구 채택 상태·질문·전달 기록과 외부 사실을 복원. 미채택 제안은 재검증/재계산하고 중복 실행하지 않는다.',
        'B의 Task 없는 직접 대화와 생성 전 요청은 Common Dialogue Interpreter가 담당. Task 상태는 Task Manager가 원본.',
        '모든 의미 판단은 Model Access를 통해 같은 Omni를 공유한다. 번호는 각 열의 실행 설명이며 고정 호출 횟수가 아니다.'
    ],25,leading=42)
    p.text(64,1900,'실선: 요청/변경   점선 화살표: 반환   /   보충 사건도. 발표용 메인 비교는 choice33-structure.svg',23,MUTED)
    return p
