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
    p.text(64, 131, 'R23  “지난번 네가 설명한 평가 기준에 맞춰, 앞서 조사한 제품 비교 결과와 받아둔 견적을 이번 제안서에 반영해줘.”', 28, bold=True)
    p.text(64, 178, '같은 이력: C20/E20 평가 기준 설명 / P20 실제 전달, T21/D21@v2 제품 비교, T22/D22@v1 견적. 결과 본문과 참조는 구별한다.', 25, MUTED)
    p.line([(64, 225), (2496, 225)], LINE, arrow=False)
    p.text(64, 242, 'A  원본 서비스 조합', 35, BLUE, bold=True)
    p.text(1340, 242, 'B  공유 파생 기억 저장소', 35, GREEN, bold=True)
    p.text(64, 291, '요청의 질문 → 원본별 읽기 → 이번 요청의 근거 묶음', 25, BLUE)
    p.text(1340, 291, '확정된 변경 → 출처 있는 과거 관계 게시 → 여러 요청이 읽기', 25, GREEN)

    body_start = len(p.items)
    for prefix, x in [('A', 84), ('B', 1360)]:
        for j, (key, name, data) in enumerate([
            ('request', 'Request Controller', 'C20 입력 / 설명 연결'),
            ('response', 'Response Manager', 'E20 / P20 실제 전달 범위'),
            ('task', 'Task Manager', 'T21/D21@v2 / T22/D22@v1'),
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
                via=([(614, 625), (320, 625), (320, 482), (sx-14, 482)] if j == 0 else [(614, 625), (sx-14, 625)]), color=BLUE)
        p.arrow('A'+key, 'composer', sp='B', tp='T', sd=14, td=30,
                via=([(sx+14, 482), (340, 482), (340, 684), (674, 684)] if j == 0 else [(sx+14, 684), (674, 684)]), color=BLUE, ret=True)
    p.label(90, 584, '읽기 범위, 질문, 권한 →', BLUE, 24)
    p.label(730, 686, '← 원문 / 버전 / 누락 범위', BLUE, 24)
    p.arrow('composer', 'cache', '읽기 / 오류 무효화', (138, 873), sp='L', tp='T',
            via=[(291, 800)], color=BLUE)
    p.arrow('cache', 'composer', sp='R', tp='B', sd=-13, td=-80,
            via=[(564, 977)], color=BLUE, ret=True)
    p.arrow('composer', 'attempt', '이번 요청에 결합', (815, 873), sp='R', tp='T',
            via=[(991, 800)], color=BLUE)
    p.text(116, 1044, '원본별 요약, 색인, cache / 배경 갱신 가능', 22, BLUE)
    p.text(814, 1044, 'R23의 발췌 / 참조 / 범위', 22, BLUE)
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
                via=([(sx, 482), (1620, 482), (1620, 578), (1640, 578)] if j == 0 else [(sx, 578), (1640, 578)]), color=GREEN)
        p.arrow('publisher', 'B'+key, sp='T', tp='B', sd=30, td=16,
                via=([(1700, 608), (1640, 608), (1640, 494), (sx+16, 494)] if j == 0 else [(1700, 608), (sx+16, 608)]), color=GREEN)
        p.arrow('B'+key, 'publisher', sp='B', tp='T', sd=-16,
                via=([(sx-16, 470), (1600, 470), (1600, 632), (1670, 632)] if j == 0 else [(sx-16, 632), (1670, 632)]), color=GREEN, ret=True)
    p.label(1400, 580, '변경 알림', GREEN, 22)
    p.label(2025, 582, '↑ 범위 읽기 / ↓ 원문 + 버전', GREEN, 22)
    p.arrow('publisher', 'repository', '검사 후 게시', (1885, 666), sp='R', tp='L',
            sd=12.5, color=GREEN, size=22)
    p.text(2000, 789, ['P20→E20 실제 전달 범위', 'T21→D21@v2 / T22→D22@v1', '출처 있는 과거 후보 / R23 정답 아님'], 23, GREEN, leading=34)
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
    p.text(1415, 932, ['R23 / 후속 재참조 R24', '같은 평가 기준과 결과 참조'], 21, MUTED, leading=28)
    p.text(1415, 1047, '표현 불가 → 미지원 반환', 22, GREEN)
    for j, key in enumerate(['request', 'response', 'task']):
        sx = 1525 + 370*j
        p.arrow('B'+key, 'reader', sp='B', tp='L',
                via=[(sx, 482), (1348, 482), (1348, 892), (1710, 892), (1710, 995)], color=INK)
    p.label(1390, 882, '변경 즉시 기존 근거 차단', INK, 22)

    # Keep typography readable while reserving a complete consumer/model area per option.
    def compact(y):
        return round(350 + (y - 350) * 0.74, 2)

    for item in p.items[body_start:]:
        if 'y' in item:
            item['y'] = compact(item['y'])
        if 'h' in item:
            item['h'] = round(item['h'] * 0.74, 2)
        if 'leading' in item:
            item['leading'] = max(item['size'] + 3, item['leading'] * 0.74)
        if 'points' in item:
            item['points'] = [(x, compact(y)) for x, y in item['points']]
    for key, (x, y, w, h) in p.nodes.items():
        p.nodes[key] = (x, compact(y), w, round(h * 0.74, 2))

    x, y, w, _ = p.nodes['repository']
    p.nodes['repository'] = (x, y, w, 105)
    for item in p.items[body_start:]:
        if item.get('id') == 'repository':
            item['h'] = 105
        if item['kind'] == 'text' and item.get('x') == 2000:
            item.update(y=687, size=21, leading=25)
        if item['kind'] == 'text' and item.get('x') == 1430:
            item.update(y=648, size=21, leading=26)
        if item.get('source') == 'reader' and item.get('target') == 'repository':
            item['points'] = [p.port('reader', 'R', -15), (2470, p.port('reader', 'R', -15)[1]),
                              (2470, p.port('repository', 'R')[1]), p.port('repository', 'R')]
        if item.get('source') == 'repository' and item.get('target') == 'reader':
            item['points'] = [p.port('repository', 'L'), (1990, p.port('repository', 'L')[1]),
                              (1990, 780), (1960, 780), p.port('reader', 'T')]
        if item['kind'] == 'label' and item.get('lines') == ['범위 조회']:
            item.update(x=2210, y=776, size=21)
    p.items = [item for item in p.items if item.get('lines') != ['유효한 관계와 원문 참조']]
    p.label(1998, 645, '반환', GREEN, 21)

    p.line([(1280, 235), (1280, 1380)], LINE, arrow=False)
    for prefix, dx, color, provider in [('A', 0, BLUE, 'composer'), ('B', 1276, GREEN, 'reader')]:
        rc, ri = prefix+'current', prefix+'interpreter'
        memory, cm = prefix+'memory', prefix+'memory-owner'
        access, omni = prefix+'access', prefix+'omni'
        p.node(rc, dx+84, 1010, 360, 70, 'Request Controller', kind='external')
        p.node(ri, dx+84, 1190, 360, 70, 'Request Interpreter', kind='external')
        p.component(cm, dx+784, 1010, 440, 200, 'Context Manager')
        p.node(memory, dx+812, 1094, 384, 90, 'User Memory', kind='store', owner=cm)
        px, py, pw, ph = p.nodes[provider]
        center = px+pw/2
        p.arrow(rc, provider, '조회 / 추가 조회 / 근거 오류 신고', (dx+170, 917),
                sp='T', tp='B', sd=-20, td=-20,
                via=[(dx+244, 951), (center-20, 951)], color=color, size=22)
        p.arrow(provider, rc, '근거 + 출처 + 버전 + 누락 반환', (dx+570, 973),
                sp='B', tp='T', sd=20, td=20,
                via=[(center+20, 987), (dx+284, 987)], color=color, ret=True, size=22)
        p.arrow(rc, ri, '현재 요청 + 근거', (dx+90, 1100), sd=-70, td=-70, size=22)
        p.arrow(ri, rc, '추가 조회 / 의미 제안', (dx+290, 1148), sp='T', tp='B', sd=70, td=70, ret=True, size=22)
        p.arrow(rc, memory, '명시 저장 / 삭제 / 조회', (dx+480, 1024), sp='R', tp='L', sd=-15, td=-10,
                via=[(dx+650, 1030), (dx+650, 1129)], size=21)
        p.arrow(memory, rc, '허용 선호 / 삭제 표식', (dx+480, 1074), sp='L', tp='R', sd=10, td=15,
                via=[(dx+710, 1149), (dx+710, 1060)], ret=True, size=21)
        p.text(dx+490, 1220, '사용 / 채택: 권한과 버전 검사 + 허용 직렬화', 21, bold=True)
        p.text(dx+90, 1270, '현재 참조 / 새 T23 연결 판단', 21, bold=True)
        p.node(access, dx+500, 1310, 300, 62, 'Model Access', kind='external')
        p.node(omni, dx+920, 1310, 300, 62, 'Shared Omni', kind='model')
        p.duplex(ri, access, label='현재 의미 해석', at=(dx+280, 1308), sp='B', tp='L',
                via=[(dx+264, 1341)], size=22)
        p.duplex(access, omni, sp='R', tp='L')
        p.text(dx+790, 1259, '음성 입력 / ASR 계속, Omni 1벌', 21, MUTED)
    p.duplex('publisher', 'Baccess', label='필요한 과거 관계 추론', at=(1760, 1276), sp='L', tp='L',
            via=[(1390, compact(695)), (1390, 962), (1750, 962), (1750, 1341)], color=GREEN, size=21)
    p.text(64, 1400, 'VIA: 발췌와 참조 연결 / Agent: 기준 적용과 제안서 작성. 검정 공통, 파랑 A, 초록 B. 실선 요청, 점선 반환, 양방향 모델. 반복 이름은 동일 주체 확대.', 21, MUTED)
    # Guard the presentation contract: no node or operational path may span both alternatives.
    for key, (x, y, w, h) in p.nodes.items():
        assert x+w <= 1240 or x >= 1340, (key, 'cross-option node')
    for item in p.items:
        if item['kind'] == 'line' and item.get('arrow'):
            xs = [x for x, _ in item['points']]
            assert max(xs) <= 1240 or min(xs) >= 1340, (item['id'], 'cross-option route')
    return p


def lifecycle():
    p = Slide('choice45-lifecycle', '정정과 삭제: 오래된 기억을 읽지 못하게 하고, 필요한 원문을 다시 확인한다', height=1920)
    p.text(64, 138, '공통 사건: D21@v2 → v3 수정 / 선호 삭제 / D21 접근 철회. 같은 원본 보존 범위, 권한, 실제 삭제 시점을 적용한다.', 28)
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
