"""04-41 landscape main comparison; unchanged alternatives and event diagrams.

Each alternative is complete inside its own column, including common contracts.
Every connector is also an editable edge in the companion draw.io scene.
"""
from stage4_diagram_design import Plate, INK, MUTED, BLUE, GREEN, LINE

TITLE = '요청 의미 확정 — 모델 중심 ReAct와 모델 틀/코드 완성'


class Slide(Plate):
    def __init__(self):
        super().__init__('choice41-structure', '04-41', TITLE, '', height=1440)
        self.items = []
        self.serial = 0

    @staticmethod
    def text_geometry(i):
        size = 26
        leading = size * 1.25
        return i['y'] + (i['h']-len(i['name'])*leading)/2-2+(7 if i['type']=='store' else 0), size, leading

    def validate(self):
        super().validate()
        # A and B must be self-contained, including repeated common Components.
        limits={'A': (64,1248), 'B': (1312,2496)}
        for key,(x,y,w,h) in self.nodes.items():
            lo,hi=limits[key[0]]
            assert lo<=x and x+w<=hi, (key,'outside comparison column')
        for item in self.items:
            if item['kind']=='line' and 'source' in item:
                prefix=item['source'][0]
                assert prefix==item['target'][0], (item['id'],'cross-option edge')
                lo,hi=limits[prefix]
                assert all(lo<=x<=hi for x,y in item['points']), (item['id'],'edge leaves column')

    def svg(self):
        return super().svg().replace('font-size="23"', 'font-size="26"')

    def drawio(self):
        return super().drawio().replace('fontSize=23;', 'fontSize=26;')

    def arrow(self, a, b, label='', at=None, sp='B', tp='T', via=(), color=INK, ret=False, sd=0, td=0, size=24):
        self.edge(a, b, sp=sp, tp=tp, via=via, color=color, ret=ret, sd=sd, td=td)
        if label:
            self.label(*at, label, color, size=size)


def structure():
    p = Slide()
    p.box(0, 0, 2560, 8, INK, 'none', 0)
    p.text(64, 25, 'VIA / 04-41 / 요청 해석의 제어와 생산 책임', 23, MUTED, bold=True)
    p.text(2496, 25, '설계 비교 · 미선정 · 미측정', 23, MUTED, 'right')
    p.text(64, 65, '요청의 뜻을 완성하는 다음 단계, 모델과 코드 중 누가 결정하는가?', 43, bold=True)
    p.text(64, 126, '같은 사례  “이 표를 아까 보고서에 넣고, 메일은 보내지 말고 초안만 만들어.”', 29, bold=True)
    p.text(64, 164, '보고서 후보 2개 → “어느 보고서인가요?” → “예산 보고서” → 표 연결 + 발송 금지 유지 후 인계', 25, MUTED)

    # Each column includes its own complete path; there are no A/B connectors.
    for prefix, dx, color, title in [
        ('A', 0, BLUE, 'A  모델이 조회와 최종 관계를 결정'),
        ('B', 1248, GREEN, 'B  모델은 틀 제공, 코드는 조회와 관계 결합')]:
        p.box(64+dx, 211, 1184, 1148, 'white', LINE, 0)
        p.box(64+dx, 211, 1184, 48, '#EEF4FF' if prefix=='A' else '#EDF9F2', 'none', 0)
        p.text(80+dx, 217, title, 29, color, bold=True)
        p.component(prefix+'input', 80+dx, 290, 310, 62, 'Interaction Manager')
        p.component(prefix+'start', 500+dx, 290, 335, 62, 'Request Controller')
        p.arrow(prefix+'input', prefix+'start', '1·6  원문 / 시점 / 입력 버전',
                (424+dx, 261), sp='R', tp='L', size=23)
        p.text(80+dx, 356, '같은 최근 대화·게시 질문·허용 자료 / 대상 Task는 조회 후 판단', 22, MUTED)

    p.component('Ari', 80, 407, 755, 405, 'Request Interpreter', BLUE)
    p.node('Aloop', 110, 502, 305, 66, 'ReAct 해석 제어기', owner='Ari', color=BLUE)
    p.node('Atools', 500, 606, 300, 66, '읽기 도구 실행기', owner='Ari', color=BLUE)
    p.node('Acheck', 110, 724, 305, 66, '의미 제안 검증기', owner='Ari', color=BLUE)
    p.node('Atemp', 500, 730, 300, 64, '임시 해석 상태', kind='store', owner='Ari', color=BLUE)
    p.component('Ama', 925, 407, 290, 62, 'Model Access')
    p.node('Aomni', 925, 550, 290, 62, '공유 Omni', kind='model')
    p.component('Acm', 925, 694, 290, 62, 'Context Manager')
    p.component('Atm', 925, 800, 290, 62, 'Task Manager')

    p.arrow('Astart', 'Aloop', '2  원문 + 읽기 도구 계약', (100, 385), sp='B', tp='T',
            via=[(667.5, 389), (435, 389), (435, 478), (262.5, 478)], color=BLUE)
    p.arrow('Aloop', 'Atools', '3  모델이 선택한 조회', (480, 573), sp='R', tp='L', via=[(455, 535), (455, 639)], color=BLUE)
    p.arrow('Atools', 'Aloop', '4  조회 결과로 재판단', (116, 576), sp='T', tp='B',
            sd=-100, td=100, via=[(550, 602), (445, 602), (445, 612), (362.5, 612)], color=BLUE, ret=True)
    p.arrow('Atools', 'Atemp', '근거·조회 기록', (585, 688), sd=70, td=70, color=BLUE)
    p.arrow('Atemp', 'Aloop', '유효 기록 회수', (340, 675), sp='L', tp='B', td=50,
            via=[(435, 762), (435, 704), (312.5, 704)], color=BLUE, ret=True)
    p.arrow('Aloop', 'Acheck', '5  최종 관계 / 질문', (120, 632), color=BLUE, sd=-100, td=-100)
    p.arrow('Acheck', 'Aloop', '오류: 제한 재생성', (116, 680), sp='T', tp='B', sd=-15, td=-15, ret=True, color=BLUE)
    p.arrow('Aloop', 'Ama', '추론 요청 / 조회·질문·완성안 반환', (495, 379), sp='T', tp='T', sd=135,
            via=[(397.5, 396), (1070, 396)], color=BLUE)
    p.arrow('Ama', 'Aloop', sp='L', tp='T', td=80, sd=15,
            via=[(885, 453), (885, 491), (342.5, 491)], color=BLUE, ret=True)

    p.component('Bri', 1330, 407, 470, 62, 'Request Interpreter', GREEN)
    p.component('Bengine', 1330, 540, 755, 322, 'Request Resolution Engine', GREEN)
    p.node('Bplanner', 1760, 652, 295, 66, '미해결 항목 처리기', owner='Bengine', color=GREEN)
    p.node('Bbind', 1360, 652, 310, 66, '관계 결합기', owner='Bengine', color=GREEN)
    p.node('Bstate', 1760, 774, 295, 72, '요청 해석 상태', kind='store', owner='Bengine', color=GREEN)
    p.component('Bma', 2173, 407, 290, 62, 'Model Access')
    p.node('Bomni', 2173, 550, 290, 62, '공유 Omni', kind='model')
    p.component('Bcm', 2173, 694, 290, 62, 'Context Manager')
    p.component('Btm', 2173, 800, 290, 62, 'Task Manager')
    p.arrow('Bstart', 'Bri', '2  원문 + 요청 틀 형식', sp='B', tp='T',
            at=(1350, 383), via=[(1915.5, 393), (1565, 393)], color=GREEN)
    p.arrow('Bri', 'Bma', '틀 / 부분 해석 요청', (1840, 403), sp='R', tp='L', sd=-10, td=-10, color=GREEN)
    p.arrow('Bma', 'Bri', '틀 / 후보 / 미해석 반환', (1830, 450), sp='L', tp='R', sd=15, td=15, color=GREEN, ret=True)
    p.arrow('Bri', 'Bplanner', '3  대상·Task·조건이 미결정인 틀', (1385, 490), sd=-50, via=[(1515, 527), (1907.5, 527)], color=GREEN)
    p.arrow('Bplanner', 'Bri', '4  필요 시\n범위 지정 해석', (1820, 482), sp='T', tp='B', sd=90, td=180,
            via=[(1997.5, 510), (1745, 510)], color=GREEN)
    p.arrow('Bplanner', 'Bstate', '후보·근거 버전', (1765, 736), color=GREEN)
    p.arrow('Bstate', 'Bbind', '허용 관계로 결합', (1395, 757), sp='L', tp='B',
            via=[(1515, 810)], color=GREEN)
    p.arrow('Bbind', 'Bplanner', '부족·충돌: 추가 조회 / 질문 결정', (1385, 615), sp='T', tp='T',
            via=[(1515, 645), (1907.5, 645)], color=GREEN, ret=True)

    # Equal data/Task contracts. Semantic producers remain inside VIA, model weights
    # are an on-device dependency outside VIA's implementation responsibility.
    for s, c, read in [('A', BLUE, 'Atools'), ('B', GREEN, 'Bplanner')]:
        dx = 0 if s == 'A' else 1248
        p.arrow(s+'ma', s+'omni', '추론 / 반환', (945+dx, 496), sd=-30, td=-30)
        p.arrow(s+'omni', s+'ma', sp='T', tp='B', sd=30, td=30, ret=True)
        p.text(928+dx, 620, 'PC의 모델 의존성 · 가중치 한 벌', 22, MUTED)
        if s == 'A':
            for target, yy, sy, lane in [('cm', 710, -16, 854), ('tm', 816, 16, 878)]:
                p.arrow(read, s+target, sp='R', tp='L', sd=sy, td=-15,
                        via=[(lane, 639+sy), (lane, yy)], color=c)
                p.arrow(s+target, read, sp='L', tp='R', sd=15, td=sy+8,
                        via=[(lane+12, yy+30), (lane+12, 639+sy+8)], ret=True, color=c)
            p.label(870, 660, '4  조회 ↔ 근거/버전', c, size=23)
        else:
            for target, yy, sy, lane in [('cm', 710, -16, 2110), ('tm', 816, 16, 2137)]:
                p.arrow(read, s+target, sp='R', tp='L', sd=sy, td=-15,
                        via=[(lane, 685+sy), (lane, yy)], color=c)
                p.arrow(s+target, read, sp='L', tp='R', sd=15, td=sy+8,
                        via=[(lane+12, yy+30), (lane+12, 685+sy+8)], color=c, ret=True)
            p.label(2115, 660, '4  조회 ↔ 근거/버전', c, size=23)
    p.text(86, 829, '구조화 출력 · 조건 보존 · 부분 수정 · cache 가능', 23, BLUE)
    p.text(1380, 822, '미지원 관계는 보류', 23, GREEN)

    # Repeat the equal adoption, storage and delivery contracts inside each panel.
    for prefix, dx, color in [('A', 0, BLUE), ('B', 1248, GREEN)]:
        def component(key, x, y, w, name):
            p.component(prefix+key, x+dx, y, w, 62, name)
        def arrow(a, b, label='', at=None, **kw):
            if at:
                at=(at[0]+dx, at[1])
            if 'via' in kw:
                kw['via']=[(x+dx, y) for x,y in kw['via']]
            p.arrow(prefix+a, prefix+b, label, at, **kw)
        component('finish', 485, 980, 350, 'Request Controller')
        component('policy', 935, 980, 280, 'Policy Manager')
        p.node(prefix+'saved', 80+dx, 973, 300, 76, 'State Store', kind='store')
        component('response', 80, 1115, 310, 'Response Manager')
        component('task', 485, 1115, 350, 'Task Manager')
        component('gateway', 935, 1115, 280, 'Agent Gateway')
        component('output', 80, 1260, 310, 'Interaction Manager')
        p.node(prefix+'agent', 930+dx, 1260, 290, 62, 'Downstream Agent', kind='external')
        if prefix=='A':
            arrow('check','finish','5·7  완성 의미 / 확인 질문',
                  (485,912), sp='B', tp='T', via=[(262.5,900),(660,900)], color=color,ret=True)
        else:
            arrow('bind','finish','5·7  결합 결과 / 질문 / 지원 한계',
                  (75,910), sp='L',tp='T',td=-80,
                  via=[(68,685),(68,953),(580,953)], color=color,ret=True)
            arrow('finish','engine','7  채택 통지 / 질문 게시 연결',
                  (455,870), sp='T',tp='B',sd=100,td=100,
                  via=[(760,947),(559.5,947)], color=color)
        arrow('finish','saved','채택·질문 기록', (92,945),sp='L',tp='R',sd=-12,td=-12)
        arrow('saved','finish',sp='R',tp='L',sd=14,td=14,ret=True)
        arrow('finish','policy','7  현재 권한 검사', (918,944),sp='R',tp='L',sd=-12,td=-12)
        arrow('policy','finish','허용 / 거절', (895,1044),sp='L',tp='R',sd=14,td=14,ret=True,size=22)
        arrow('finish','task','8  Task 생성/변경', (493,1068),sd=-90,td=-90,size=23)
        arrow('task','finish','현재 사실 /\n접수·상태', (731,1060),sp='T',tp='B',sd=80,td=80,ret=True,size=22)
        arrow('task','gateway','요청·조건', (836,1085),sp='R',tp='L',sd=-12,td=-12,size=22)
        arrow('gateway','task','접수·결과', (836,1182),sp='L',tp='R',sd=14,td=14,ret=True,size=22)
        arrow('gateway','agent','업무 위임', (932,1212),sd=-60,td=-60,size=22)
        arrow('agent','gateway','접수·결과', (1124,1212),sp='T',tp='B',sd=60,td=60,ret=True,size=22)
        arrow('finish','response','5·8  질문 / 응답 / 확인된 상태', (85,1078),sp='L',tp='R',sd=6,
              via=[(414,1017),(414,1146)],size=23)
        arrow('response','finish','실제 게시 기록', (397,1185),sp='R',tp='L',sd=20,td=25,
              via=[(447,1166),(447,1036)],ret=True,size=22)
        arrow('response','output','Text / 음성', (85,1212),sd=-60,td=-60,size=22)
        arrow('output','response','실제 전달', (254,1212),sp='T',tp='B',sd=60,td=60,ret=True,size=22)
        arrow('output','input','6  답변은 새 입력', (86,1330),sp='L',tp='L',
              via=[(70,1291),(70,321)],size=22)
        p.text(490+dx, 1226, ['7  입력·자료·Task·질문 버전 확인',
                             '변경/철회는 재평가, 저장 실패는 인계 보류',
                             '응답 추론도 Model Access 경유'],20,MUTED,leading=30)
        p.text(932+dx,1330,'외부 업무 계획·실행 책임',21,MUTED)

    p.text(64,1379,'검정: 공통 / 파랑: A / 초록: B · 네모: Component/Module · 원통: 상태 · 실선: 요청/전달 · 점선: 반환',21,MUTED)
    p.text(2496,1409,'각 칸은 독립 대안 · 같은 이름은 칸 안의 동일 인스턴스 · VIA 논리 책임, 별도 process 가정 없음 · Omni 한 벌/역할별 세션 · 음성 입력 유지',20,MUTED,'right')
    return p
