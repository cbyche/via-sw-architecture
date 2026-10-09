"""04-45: request-composed evidence versus a maintained episodic read model."""
import argparse
from pathlib import Path
import xml.etree.ElementTree as ET

from stage4_diagram_design import Plate, INK, MUTED, LINE
from dp_comparison_structures import APRICOT, DIFFERENCE_STROKE

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
        root=ET.fromstring(data)
        ns={'s':'http://www.w3.org/2000/svg'}
        byid={i['id']:i for i in self.items}
        for group in root.findall('.//s:g',ns):
            item=byid.get(group.get('id'))
            if item and item['kind']=='component':
                for r in group.findall('s:rect',ns):r.set('rx','0');r.set('stroke-width','1.0')
            if item and item['kind']=='node' and item['type']=='module':
                for r in group.findall('s:rect',ns):
                    r.set('fill',APRICOT);r.set('stroke',DIFFERENCE_STROKE);r.set('stroke-width','1.8')
                for text in group.findall('s:text',ns):text.set('fill',INK)
        ET.register_namespace('', 'http://www.w3.org/2000/svg')
        return ET.tostring(root,encoding='unicode')+'\n'

    def drawio(self):
        root = ET.fromstring(super().drawio())
        duplex = {i['id'] for i in self.items if i.get('duplex')}
        for cell in root.findall('.//mxCell'):
            if cell.get('id') in duplex:
                cell.set('style', cell.get('style') + 'startArrow=open;startFill=0;')
        byid={i['id']:i for i in self.items}
        for cell in root.findall('.//mxCell'):
            item=byid.get(cell.get('id'))
            if item and item['kind']=='node' and item['type']=='module':
                cell.set('style',cell.get('style')+f'fillColor={APRICOT};strokeColor={DIFFERENCE_STROKE};fontColor={INK};strokeWidth=1.8;')
            if item and item['kind']=='component':cell.set('style',cell.get('style')+'strokeWidth=1.0;')
            if item and item['kind']=='box' and not item['radius']:cell.set('style',cell.get('style').replace('rounded=1;','rounded=0;'))
        return ET.tostring(root, encoding='unicode') + '\n'


def structure():
    from generate_dp_comparison_slides import Comparison
    from dp_comparison_structures import graph45
    p=Comparison(45)
    p.slug='choice45-structure'
    for side,x in enumerate([140,1010]):graph45(p,x,side)
    return p


def lifecycle():
    p = Slide('choice45-lifecycle', '정정과 삭제: 오래된 기억을 읽지 못하게 하고, 필요한 원문을 다시 확인한다', height=1920)
    p.text(64, 138, '공통 사건: D21@v2 → v3 수정 / 선호 삭제 / D21 접근 철회. 같은 원본 보존 범위, 권한, 실제 삭제 시점을 적용한다.', 28)
    p.text(64, 185, '유효하지 않은 근거의 신규 사용 차단은 배경 재생산이나 물리 삭제의 완료를 기다리지 않는다.', 25, bold=True)
    groups = [
        ('A', INK, ['Source Owner', 'Context\nComposer', 'Request\nInterpreter', 'Request\nController', 'Model Access'], [
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
        ('B', INK, ['Source Owner', 'Memory\nPublisher', 'Evidence\nReader', 'Request\nInterpreter', 'Request\nController'], [
            (0, 2, '1  읽기 차단 버전 / 삭제 / 권한 철회 먼저 반영', False),
            (2, 4, '2  이전 근거 차단 → 영향 Context/KV 폐기 요청', False),
            (0, 1, '3  변경 사건 + 원본 버전 + 영향 참조', False),
            (1, 1, '4  파생 관계 무효화 / 허용된 원본으로 재생산', False),
            (1, 2, '5  검증된 관계 + coverage 게시; revision/상태 반환 → Reader 재조회', True),
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
        owner=letter+'cm'
        p.component(owner,x+242,328,212 if n==0 else 454,132,'Context Manager')
        for k,(xx,name) in enumerate(zip(xs,names)):
            if k==0:
                p.text(xx,325,['원본 owner (사건 접점)', 'Request Controller', 'Response Manager', 'Task Manager'],17,color,'center',leading=21)
            elif (n==0 and k==1) or (n==1 and k in (1,2)):
                p.node(letter+str(k),xx-96,390,192,55,name,owner=owner,color=DIFFERENCE_STROKE)
            else:
                p.box(xx-106,328,212,82,'white',color,radius=0,thick=1)
                p.text(xx,340 if '\n' in name else 355,name,23,color,'center',True,leading=29)
            p.line([(xx,460),(xx,1468)],LINE,dashed=True,arrow=False)
        for j, (a, b, label, ret) in enumerate(steps):
            y = 485+j*102
            p.text(x+6, y-39, label, 23, color)
            pts = [(xs[a], y), (xs[b], y)] if a != b else [(xs[a], y), (xs[a]+65, y), (xs[a]+65, y+18), (xs[a], y+18)]
            p.line(pts, color, dashed=ret, width=1.3)
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
        if hasattr(p,'validate'):p.validate()
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
