"""04-41 landscape main comparison; unchanged alternatives and event diagrams.

A/B share the input and adoption/delivery rails, not their resolution mechanism.
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

    # Shared input rail. The branches are alternative paths, never a fan-out job.
    p.component('input', 64, 220, 355, 62, 'Interaction Manager')
    p.component('start', 825, 220, 405, 62, 'Request Controller')
    p.arrow('input', 'start', '1·6  원문 / 시점 / 입력 버전', (446, 217), sp='R', tp='L')
    p.text(1330, 222, '아래는 A 또는 B를 선택한 구조 비교', 28, bold=True)
    p.text(1330, 261, '같은 최근 대화·게시 질문·허용 자료 / 정답 Task를 미리 주지 않음', 24, MUTED)
    p.line([(64, 315), (2496, 315)], LINE, arrow=False)

    # A/B columns are aligned only at the external contracts.
    for x, c, title in [(64, BLUE, 'A  모델이 조회와 최종 관계를 결정'),
                         (1312, GREEN, 'B  모델은 틀 제공, 코드는 조회와 관계 결합')]:
        p.box(x, 332, 1184, 48, '#EEF4FF' if c == BLUE else '#EDF9F2', 'none', 0)
        p.text(x+16, 338, title, 29, c, bold=True)
    p.line([(1280, 332), (1280, 866)], LINE, arrow=False)

    p.component('Ari', 80, 407, 755, 405, 'Request Interpreter', BLUE)
    p.node('Aloop', 110, 502, 305, 66, 'ReAct 해석 제어기', owner='Ari', color=BLUE)
    p.node('Atools', 500, 606, 300, 66, '읽기 도구 실행기', owner='Ari', color=BLUE)
    p.node('Acheck', 110, 724, 305, 66, '의미 제안 검증기', owner='Ari', color=BLUE)
    p.node('Atemp', 500, 730, 300, 64, '임시 해석 상태', kind='store', owner='Ari', color=BLUE)
    p.component('Ama', 925, 407, 290, 62, 'Model Access')
    p.node('Aomni', 925, 550, 290, 62, '공유 Omni', kind='model')
    p.component('Acm', 925, 694, 290, 62, 'Context Manager')
    p.component('Atm', 925, 800, 290, 62, 'Task Manager')

    p.arrow('start', 'Aloop', '2  원문 + 읽기 도구 계약', (100, 385), sp='B', tp='T', sd=-120,
            via=[(907.5, 301), (54, 301), (54, 478), (262.5, 478)], color=BLUE)
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
    p.arrow('start', 'Bri', '2  원문 + 요청 틀 형식', sp='B', tp='T', sd=120,
            at=(1450, 383), via=[(1147.5, 301), (1300, 301), (1300, 397), (1565, 397)], color=GREEN)
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

    # Common adoption/output contract expanded once; repeated names denote the
    # same logical instance. Both proposals return, neither is an execution permit.
    p.text(64, 886, '공통 채택·위임·사용자 전달  —  A/B의 완성 의미·질문·지원 한계를 같은 Controller가 받음', 25, bold=True)
    p.component('finish', 840, 967, 405, 62, 'Request Controller')
    p.component('task', 1390, 967, 310, 62, 'Task Manager')
    p.component('gateway', 1825, 967, 280, 62, 'Agent Gateway')
    p.node('agent', 2220, 967, 300, 62, 'Downstream Agent', kind='external')
    p.component('response', 440, 1070, 335, 62, 'Response Manager')
    p.component('output', 64, 1070, 300, 62, 'Interaction Manager')
    p.component('policy', 64, 967, 310, 62, 'Policy Manager')
    p.node('saved', 447, 958, 310, 76, 'State Store', kind='store')
    p.arrow('Acheck', 'finish', '5·7  의미 / 질문 제안', (432, 920), sp='B', tp='T', td=-100,
            via=[(262.5, 867), (792, 867), (792, 953), (942.5, 953)], color=BLUE, ret=True)
    p.arrow('Bbind', 'finish', '5·7  결합 결과 / 질문\n지원 한계', (1325, 867), sp='L', tp='T', td=100,
            via=[(1300, 685), (1300, 909), (1142.5, 909)], color=GREEN, ret=True)
    p.arrow('finish', 'Bengine', '7  채택 통지 / 질문 게시 연결', (1818, 885), sp='T', tp='B', sd=160, td=100,
            via=[(1202.5, 916), (1807.5, 916)], color=GREEN)
    p.arrow('finish', 'policy', '7  현재 권한 ↔ 검사 결과', (75, 931), sp='L', tp='T', sd=-10,
            via=[(808, 988), (808, 955), (219, 955)])
    p.arrow('policy', 'finish', sp='R', tp='L', sd=15, td=20,
            via=[(395, 1013), (395, 1048), (816, 1048), (816, 1018)], ret=True)
    p.arrow('finish', 'saved', '채택·질문 기록', (456, 1041), sp='L', tp='R', sd=-2)
    p.arrow('saved', 'finish', sp='R', tp='L', sd=18, td=16, ret=True)
    p.arrow('finish', 'task', '8  채택한 요청', (1255, 936), sp='R', tp='L', sd=-13, td=-13)
    p.arrow('task', 'finish', '현재 사실 / 접수·상태', (1252, 1037), sp='L', tp='R', sd=15, td=15, ret=True, size=22)
    p.arrow('task', 'gateway', 'command ID / 조건', (1696, 936), sp='R', tp='L', sd=-13, td=-13, size=22)
    p.arrow('gateway', 'task', '접수·결과', (1712, 1037), sp='L', tp='R', sd=15, td=15, ret=True, size=22)
    p.arrow('gateway', 'agent', '업무 위임', (2117, 936), sp='R', tp='L', sd=-13, td=-13, size=22)
    p.arrow('agent', 'gateway', '접수·결과', (2120, 1037), sp='L', tp='R', sd=15, td=15, ret=True, size=22)
    p.arrow('finish', 'response', '5·8  질문 / 직접 응답 / 확인된 상태', (842, 1060), sp='B', tp='R', sd=-80,
            via=[(962.5, 1101)])
    p.arrow('response', 'finish', '실제 게시 기록', (842, 1110), sp='R', tp='B', sd=20, td=110,
            via=[(1152.5, 1121)], ret=True)
    p.arrow('response', 'output', 'Text / 음성', (279, 1138), sp='L', tp='R', sd=-10, td=-10)
    p.label(70, 1140, '반환: 실제 전달 기록', MUTED, size=22)
    p.arrow('output', 'response', sp='R', tp='L', sd=15, td=15, ret=True)
    p.text(70, 1042, '6  답변은 맨 위의 새 입력', 22, MUTED)
    p.text(1387, 1090, '7  입력·자료·Task·질문 버전 확인 후 채택', 24, bold=True)
    p.text(1387, 1124, '철회·변경은 재평가 / 저장 실패는 인계 보류 / 응답도 Model Access 경유', 22, MUTED)
    p.text(2223, 1065, '외부 업무 계획·실행 책임', 22, MUTED)

    # Short comparison rows, not invented quality scores.
    p.line([(64, 1180), (2496, 1180)], LINE, arrow=False)
    p.line([(1280, 1180), (1280, 1370)], LINE, arrow=False)
    for x,c,lines in [(80,BLUE,[('장점', '열린 표현을 근거와 공동 해석; 새 맥락에 유연하게 조회'),
                                 ('대가', '반복 추론·모델 경합, 잘못된 연결과 조건 누락의 검증'),
                                 ('선택 조건', '작은 검증/슬롯 처리로 충분하고 열린 관계가 중요한 경우')]),
                       (1330,GREEN,[('장점', '명시 관계의 결합·수정과 미해결 원인을 코드로 추적'),
                                    ('대가', '표현 범위·엔진 확장·후보 상태 비용; 추가 해석/질문 가능'),
                                    ('선택 조건', '한정된 표현이 주요 요청을 포괄하고 반복 결합 통제가 중요한 경우')])]:
        for j,(label,value) in enumerate(lines):
            yy=1196+j*58
            p.text(x,yy,label,26,c,bold=True)
            p.text(x+148,yy,value,25)
    p.line([(64, 1377), (2496, 1377)], LINE, arrow=False)
    p.text(64,1391,'검정: 공통 / 파랑: A / 초록: B · 네모: Component/Module · 원통: 상태 · 실선: 요청/전달 · 점선: 반환',21,MUTED)
    p.text(2496,1410,'같은 이름은 같은 인스턴스 · VIA 논리 책임(별도 process 가정 없음) · Omni 한 벌, 역할별 세션 분리 · 음성 입력/인식 유지',20,MUTED,'right')
    return p
