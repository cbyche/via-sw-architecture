"""04-45: request-composed evidence versus a maintained episodic read model."""
import argparse
from pathlib import Path
import xml.etree.ElementTree as ET

from stage4_diagram_design import Plate, INK, MUTED, BLUE, GREEN, LINE

OUT = Path(__file__).resolve().parents[2] / 'docs/architecture/12-decisions/decision-packages/diagrams'


class Slide(Plate):
    def __init__(self, slug, title, height=1440):
        super().__init__(slug, '04-45', title, '', height=height)
        self.items, self.serial = [], 0
        self.box(0, 0, 2560, 8, INK, 'none', 0)
        self.text(64, 24, 'VIA / 45 / MEMORY AND CONTEXT', 23, MUTED, bold=True)
        self.text(2496, 24, '설계 후보 / 미선정 / 미측정', 23, MUTED, 'right')
        self.text(64, 70, title, 40, bold=True)

    @staticmethod
    def text_geometry(i):
        size = 25 if i['w'] >= 290 else 23
        leading = size * 1.22
        return i['y'] + (i['h'] - len(i['name']) * leading) / 2 - 2 + (7 if i['type'] == 'store' else 0), size, leading

    def arrow(self, a, b, label='', at=None, sp='B', tp='T', via=(), color=INK, ret=False, sd=0, td=0, size=24):
        self.edge(a, b, sp=sp, tp=tp, via=via, color=color, ret=ret, sd=sd, td=td)
        if label:
            self.label(*at, label, color, size=size)

    def duplex(self, a, b, **kwargs):
        self.arrow(a, b, **kwargs)
        edge = next(i for i in reversed(self.items) if i['kind'] == 'line')
        edge['duplex'] = True

    def svg(self):
        data = super().svg()
        for i in self.items:
            if i.get('duplex'):
                data = data.replace(f'<polyline id="{i["id"]}"', f'<polyline id="{i["id"]}" marker-start="url(#a{i["color"][1:]})"')
        return data

    def drawio(self):
        root = ET.fromstring(super().drawio())
        duplex = {i['id'] for i in self.items if i.get('duplex')}
        for cell in root.findall('.//mxCell'):
            if cell.get('id') in duplex:
                cell.set('style', cell.get('style') + 'startArrow=open;startFill=0;')
        return ET.tostring(root, encoding='unicode') + '\n'


def structure():
    p = Slide('choice45-structure', '과거 근거를 요청마다 조합할 것인가, 공통 기억으로 유지할 것인가?')
    p.text(64, 131, '같은 현재 요청 R9   “지난번에 내가 고친 표현 방식으로 이번 보고서도 정리해줘.”', 30, bold=True)
    p.text(64, 178, '같은 이력: D2 보고서 → R7 “짧은 문장, 결론 먼저” → 수정 결과 D3 → 실제 제시 P4. 이 사례의 재사용은 선호 저장이 아니다.', 25, MUTED)
    p.line([(64, 225), (2496, 225)], LINE, arrow=False)
    p.text(64, 242, 'A  원본 서비스 조합', 35, BLUE, bold=True)
    p.text(1340, 242, 'B  공유 파생 기억 저장소', 35, GREEN, bold=True)
    p.text(64, 291, '요청의 질문 → 원본별 읽기 → 이번 요청의 근거 묶음', 25, BLUE)
    p.text(1340, 291, '확정된 변경 → 출처 있는 과거 관계 게시 → 여러 요청이 읽기', 25, GREEN)

    for prefix, x in [('A', 84), ('B', 1360)]:
        for j, (key, name, data) in enumerate([
            ('request', 'Request Controller', 'R7 원문 / 정정 관계'),
            ('response', 'Response Manager', 'P4 실제 전달 기록'),
            ('task', 'Task Manager', 'D2 → D3 확정 결과'),
        ]):
            xx = x + 370*j
            p.node(prefix+key, xx, 350, 330, 76, name, kind='external')
            p.label(xx+16, 438, data, MUTED, 23)
    p.component('Acm', 64, 508, 1160, 585, 'Context Manager')
    p.component('Bcm', 1340, 508, 1156, 585, 'Context Manager')

    p.node('composer', 416, 755, 456, 90, 'Context Composer', owner='Acm', color=BLUE)
    p.node('cache', 116, 945, 350, 90, 'Source View Cache', kind='store', owner='Acm', color=BLUE)
    p.node('attempt', 814, 945, 354, 90, 'Request Evidence Set', kind='data', owner='Acm', color=BLUE)
    # The two buses retain explicit owner endpoints: neither implies a global source DB.
    for j, key in enumerate(['request', 'response', 'task']):
        sx = 249 + 370*j
        p.arrow('composer', 'A'+key, sp='T', tp='B', sd=-30, td=-14,
                via=[(614, 625), (sx-14, 625)], color=BLUE)
        p.arrow('A'+key, 'composer', sp='B', tp='T', sd=14, td=30,
                via=[(sx+14, 684), (674, 684)], color=BLUE, ret=True)
    p.label(90, 584, '읽기 범위, 질문, 권한 →', BLUE, 24)
    p.label(730, 686, '← 원문 / 버전 / 누락 범위', BLUE, 24)
    p.arrow('composer', 'cache', '읽기 / 오류 무효화', (138, 873), sp='L', tp='T',
            via=[(291, 800)], color=BLUE)
    p.arrow('cache', 'composer', sp='R', tp='B', sd=-13, td=-80,
            via=[(564, 977)], color=BLUE, ret=True)
    p.arrow('composer', 'attempt', '이번 요청에 결합', (815, 873), sp='R', tp='T',
            via=[(991, 800)], color=BLUE)
    p.text(116, 1044, '원본별 요약, 색인, cache / 배경 갱신 가능', 22, BLUE)
    p.text(814, 1044, 'R9의 근거 / 요청 범위', 22, BLUE)
    for j, key in enumerate(['request', 'response', 'task']):
        sx = 249 + 370*j
        p.arrow('A'+key, 'cache', sp='B', tp='L',
                via=[(sx, 482), (80, 482), (80, 990)], color=INK)
    p.label(88, 706, '수정 / 삭제 → 무효화', INK, 24)

    p.node('publisher', 1460, 650, 420, 90, 'Memory Publisher', owner='Bcm', color=GREEN)
    p.node('repository', 2050, 650, 400, 115, 'Episodic Memory\nRepository', kind='store', owner='Bcm', color=GREEN)
    p.node('reader', 1740, 950, 440, 90, 'Evidence Reader', owner='Bcm', color=GREEN)
    for j, key in enumerate(['request', 'response', 'task']):
        sx = 1525 + 370*j
        p.arrow('B'+key, 'publisher', sp='B', tp='T', td=-30,
                via=[(sx, 578), (1640, 578)], color=GREEN)
        p.arrow('publisher', 'B'+key, sp='T', tp='B', sd=30, td=16,
                via=[(1700, 608), (sx+16, 608)], color=GREEN)
        p.arrow('B'+key, 'publisher', sp='B', tp='T', sd=-16,
                via=[(sx-16, 632), (1670, 632)], color=GREEN, ret=True)
    p.label(1400, 580, '변경 알림', GREEN, 22)
    p.label(2025, 582, '↑ 범위 읽기 / ↓ 원문 + 버전', GREEN, 22)
    p.arrow('publisher', 'repository', '검사 후 게시', (1885, 666), sp='R', tp='L',
            sd=12.5, color=GREEN, size=22)
    p.text(2000, 789, ['R7→D2 정정 / D3→D2 개정', 'P4→D3 실제 제시 / 출처와 범위', '여러 요청에 남는 공통 읽기 상태'], 23, GREEN, leading=34)
    p.arrow('reader', 'repository', '범위 조회', (2210, 927), sp='R', tp='B', sd=-15,
            via=[(2240, 980)], td=-10, color=GREEN, size=23)
    p.arrow('repository', 'reader', '유효한 관계와 원문 참조', (1725, 901), sp='B', tp='T', sd=-70,
            via=[(2180, 880), (1960, 880)], color=GREEN, ret=True, size=23)
    for j, key in enumerate(['request', 'response', 'task']):
        sx = 1525 + 370*j
        p.arrow('reader', 'B'+key, sp='R', tp='T',
                via=[(2480, 995), (2480, 328), (sx, 328)], color=INK)
        p.arrow('B'+key, 'reader', sp='T', tp='R', sd=14, td=25,
                via=[(sx+14, 338), (2460, 338), (2460, 1020)], color=INK, ret=True)
    p.label(2130, 1058, '인용 / 충돌 원문 재확인', INK, 22)
    p.arrow('reader', 'publisher', sp='T', tp='B', sd=-180, td=110, color=GREEN)
    p.arrow('publisher', 'reader', sp='B', tp='T', sd=124, td=-166, color=GREEN, ret=True)
    p.label(1805, 786, '미게시 생산\n오류 재검증\n해당 버전 차단', GREEN, 21)
    p.text(1430, 785, ['과거 관계 후보 생성', '출처와 버전 검증', '영향 범위 재게시'], 23, GREEN, leading=32)
    p.text(1415, 932, ['R9 / 이어지는 다른 요청 R10', '“수정 결과를 메일에도”'], 21, MUTED, leading=28)
    p.text(1415, 1047, '표현 불가 → 미지원 반환', 22, GREEN)
    for j, key in enumerate(['request', 'response', 'task']):
        sx = 1525 + 370*j
        p.arrow('B'+key, 'reader', sp='B', tp='L',
                via=[(sx, 482), (1360, 482), (1360, 892), (1710, 892), (1710, 995)], color=INK)
    p.label(1390, 882, '변경 즉시 기존 근거 차단', INK, 22)

    # One current interpretation/adoption path is fixed across both memory alternatives.
    p.node('rc', 1020, 1170, 520, 76, 'Request Controller', kind='external')
    p.node('ri', 120, 1170, 440, 76, 'Request Interpreter', kind='external')
    p.arrow('rc', 'composer', '조회 / 근거 오류 신고', (440, 1104), sp='T', tp='B', sd=-190, td=-20,
            via=[(1090, 1140), (624, 1140)], color=BLUE)
    p.arrow('composer', 'rc', '근거 + 출처 + 버전 + 누락', (682, 1143), sp='B', tp='T', sd=20, td=-80,
            via=[(664, 1155), (1200, 1155)], color=BLUE, ret=True, size=22)
    p.arrow('rc', 'reader', '조회 / 근거 오류 신고', (1440, 1118), sp='T', tp='B', sd=120, td=30,
            via=[(1400, 1140), (1800, 1140), (1800, 1100), (1990, 1100)], color=GREEN, size=22)
    p.arrow('reader', 'rc', '출처 / 버전 / 누락 반환', (1570, 1144), sp='B', tp='T', sd=-30, td=210,
            via=[(1930, 1102), (1818, 1102), (1818, 1155), (1490, 1155)], color=GREEN, ret=True, size=21)
    p.arrow('rc', 'ri', '현재 요청 + 근거', (602, 1168), sp='L', tp='R', sd=-14, td=-14)
    p.arrow('ri', 'rc', '추가 조회 / 최종 의미 제안', (600, 1220), sp='R', tp='L', sd=14, td=14, ret=True, size=22)
    p.text(1020, 1262, '사용 / 채택: 권한과 버전 검사 + 허용 직렬화', 22, bold=True)
    p.text(410, 1262, '이번 대상과 Task 판단', 23, bold=True)
    p.component('common-cm', 1836, 1110, 660, 183, 'Context Manager')
    p.node('user-memory', 1870, 1180, 580, 90, 'User Memory', kind='store', owner='common-cm')
    p.arrow('rc', 'user-memory', '명시 저장 / 삭제 / 조회', (1575, 1170), sp='R', tp='L', sd=-14, td=-31, size=21)
    p.arrow('user-memory', 'rc', '허용 선호 / 삭제 표식', (1575, 1231), sp='L', tp='R', sd=3, td=20, ret=True, size=21)

    p.node('access', 1840, 1305, 260, 68, 'Model Access', kind='external')
    p.node('omni', 2200, 1305, 295, 68, 'Shared Omni', kind='model')
    p.duplex('ri', 'access', label='현재 의미 해석', at=(610, 1320), sp='B', tp='L',
            via=[(340, 1339)], color=INK, size=23)
    p.duplex('publisher', 'access', label='필요한 과거 관계 추론', at=(1440, 1094), sp='L', tp='T',
            via=[(1390, 695), (1390, 1118), (1755, 1118), (1755, 1298), (1970, 1298)], color=GREEN, size=23)
    p.duplex('access', 'omni', sp='R', tp='L')
    p.text(64, 1366, '공통: 음성 입력과 ASR 계속 / Omni 1벌, 역할별 Context와 KV / User Memory는 명시 저장, 이번 지시는 임시 / 양방향 모델선은 호출과 반환', 21, MUTED)
    p.text(64, 1400, '검정 공통 / 파랑 A / 초록 B | 실선 요청, 전달 / 점선 반환 | 원본 소유자는 VIA 내부. Request Controller와 Context Manager의 반복은 같은 주체의 확대. 별도 process가 아니다.', 21, MUTED)
    return p


def lifecycle():
    p = Slide('choice45-lifecycle', '정정과 삭제: 오래된 기억을 읽지 못하게 하고, 필요한 원문을 다시 확인한다', height=1920)
    p.text(64, 138, '공통 사건: D3 수정 / 선호 삭제 / 자료 접근 철회. 같은 원본 보존 범위, 권한, 실제 삭제 시점을 적용한다.', 28)
    p.text(64, 185, '유효하지 않은 근거의 신규 사용 차단은 배경 재생산이나 물리 삭제의 완료를 기다리지 않는다.', 25, bold=True)
    groups = [
        ('A', BLUE, ['Source Owner', 'Context\nComposer', 'Request\nInterpreter', 'Request\nController', 'Model Access'], [
            (0, 1, '1  현재 원장 변경 + 동기 사용 승인 차단', False),
            (1, 1, '2  Source View Cache와 요청 근거를 무효화', False),
            (1, 3, '3  차단 버전 반영 / 영향 요청에 취소 전파', False),
            (3, 4, '4  영향받은 Context/KV와 진행 중 생성 폐기', False),
            (1, 0, '5  새 요청 / 재시도: 허용된 원본을 범위 읽기', False),
            (0, 1, '6  현재 자료 + 버전 + 누락 / 복구 불가 반환', True),
            (1, 3, '7  다시 조합한 근거 / 미해결 범위 반환', True),
            (2, 3, '8  원본은 그대로, 기억과 모순 발견 → EvidenceDispute', False),
            (3, 1, '9  기억 ID / 파생 버전 / 원문 모순을 회송', False),
            (1, 1, '10  해당 파생 버전 무효화 → 원본 재조합', False),
        ]),
        ('B', GREEN, ['Source Owner', 'Memory\nPublisher', 'Evidence\nReader', 'Request\nInterpreter', 'Request\nController'], [
            (0, 2, '1  읽기 차단 버전 / 삭제 / 권한 철회 먼저 반영', False),
            (2, 4, '2  이전 근거 차단 → 영향 Context/KV 폐기 요청', False),
            (0, 1, '3  변경 사건 + 원본 버전 + 영향 참조', False),
            (1, 1, '4  파생 관계 무효화 / 허용된 원본으로 재생산', False),
            (1, 2, '5  허용된 수정 범위만: 버전 검증 후 재게시 / 읽기 재개', False),
            (2, 0, '6  정확한 인용 / 모순 확인 → 원문 재확인', False),
            (0, 2, '7  현재 자료 / 누락 / 복구 불가 반환', True),
            (3, 4, '8  원본은 그대로, 기억과 모순 발견 → EvidenceDispute', False),
            (4, 1, '9  기억 ID / 파생 버전 / 원문 모순을 회송', False),
            (1, 1, '10  해당 파생 버전 차단 → 재생산 / 조건부 재게시', False),
        ]),
    ]
    for n, (letter, color, names, steps) in enumerate(groups):
        x = 64 + n*1276
        p.text(x, 257, letter + ('  요청별 조합을 다시 수행' if n == 0 else '  읽기 차단과 기억 복구를 분리'), 32, color, bold=True)
        xs = [x+106+i*242 for i in range(5)]
        for xx, name in zip(xs, names):
            p.box(xx-106, 328, 212, 82, 'white', color)
            p.text(xx, 340 if '\n' in name else 355, name, 23, color, 'center', True, leading=29)
            p.line([(xx, 410), (xx, 1468)], LINE, dashed=True, arrow=False)
        for j, (a, b, label, ret) in enumerate(steps):
            y = 485+j*102
            p.text(x+6, y-39, label, 23, color)
            pts = [(xs[a], y), (xs[b], y)] if a != b else [(xs[a], y), (xs[a]+65, y), (xs[a]+65, y+18), (xs[a], y+18)]
            p.line(pts, color, dashed=ret, width=2.5)
    p.line([(64, 1490), (2496, 1490)], LINE, arrow=False)
    p.text(64, 1515, '같은 계약, 다른 관리 대상', 30, bold=True)
    p.text(64, 1568, [
        '원본이 남아 있으면 재확인한다. 삭제되었거나 허용 범위 밖이면 “모름 / 자료 부재”로 반환하며 요약에서 사실을 복원하지 않는다.',
        'Context Manager의 User Memory: 명시 저장만 허용. 이번 요청의 일회성 지시가 우선하며, 과거 사례 재사용은 장기 선호가 아니다.',
        '삭제 tombstone과 권한 버전은 재구축에도 적용한다. 옛 대화와 파생 관계에서 삭제된 선호를 다시 활성화하지 않는다.',
        '최종 대상 / Task 판단은 Request Interpreter, 채택은 Request Controller 책임이다. 각 사용점의 검사와 실제 허용을 직렬화한다.',
        '하나의 Model Access / Omni를 공유한다. 배경 기억 생산은 취소 가능한 낮은 우선순위이며 음성 입력과 ASR은 계속된다.',
        '양안의 Source Owner는 실제 원본 생산자들을 묶은 생명선이다. B의 미게시 범위는 생산 후 게시를 기다리며, 직접 조합 우회는 B+ 혼합이다.'
    ], 25, leading=43)
    p.text(64, 1858, '실선 요청 / 전달, 점선 반환. 8~10은 원본 수정 없이 발생한 별도 오류 사건. A의 원본별 배경 요약도 허용. B의 표현 불가는 미게시 범위와 구별한다.', 22, MUTED)
    return p


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    drift = []
    for p in [structure(), lifecycle()]:
        p.validate()
        for ext, data in [('svg', p.svg()), ('drawio', p.drawio())]:
            root = ET.fromstring(data)
            if ext == 'drawio':
                ids = [e.get('id') for e in root.findall('.//mxCell')]
                assert len(ids) == len(set(ids))
                for cell in root.findall('.//mxCell'):
                    for attr in ['source', 'target', 'parent']:
                        assert not cell.get(attr) or cell.get(attr) in ids
            dest = OUT / (p.slug + '.' + ext)
            if args.check:
                if not dest.exists() or dest.read_text() != data:
                    drift.append(dest.name)
            else:
                dest.write_text(data)
    if drift:
        raise SystemExit('Diagram drift: ' + ', '.join(drift))
    print('PASS: 2 diagram pairs; deterministic source parity, XML, ownership, bounds and routes')


if __name__ == '__main__':
    main()
