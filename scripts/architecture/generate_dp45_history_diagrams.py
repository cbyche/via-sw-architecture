"""DP45 history definitions and same-record walkthrough; documentation only."""
import argparse
from pathlib import Path
import xml.etree.ElementTree as ET

from stage4_diagram_design import Plate, INK, MUTED, LINE
from dp_comparison_structures import APRICOT, DIFFERENCE_STROKE

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/diagrams'


class HistoryPlate(Plate):
    def __init__(self, slug, title, thesis):
        super().__init__(slug, 'DP-45', title, thesis, height=1440)
        self.differences = set()

    @staticmethod
    def text_geometry(item):
        size = 24 if item['w'] >= 600 else 22
        leading = size * 1.35
        top = item['y'] + (item['h'] - len(item['name']) * leading) / 2
        return top + (10 if item['type'] == 'store' else 0), size, leading

    def different(self, key):
        self.differences.add(key)

    def svg(self):
        root = ET.fromstring(super().svg())
        ns = {'s': 'http://www.w3.org/2000/svg'}
        for group in root.findall('.//s:g', ns):
            if group.get('id') in self.differences:
                for shape in list(group):
                    if shape.tag.rsplit('}', 1)[-1] in ('rect', 'path'):
                        shape.set('fill', APRICOT)
                        shape.set('stroke', DIFFERENCE_STROKE)
        ET.register_namespace('', 'http://www.w3.org/2000/svg')
        return ET.tostring(root, encoding='unicode') + '\n'

    def drawio(self):
        root = ET.fromstring(super().drawio())
        for cell in root.findall('.//mxCell'):
            if cell.get('id') in self.differences:
                cell.set('style', cell.get('style') +
                         f'fillColor={APRICOT};strokeColor={DIFFERENCE_STROKE};')
        return ET.tostring(root, encoding='unicode') + '\n'


def definition():
    p = HistoryPlate('dp45-history-definition', '과거 이력의 범위와 현재 Context',
                     '과거 이력 = VIA를 통해 이미 수신하거나 처리하거나 전달한 내용의 원본 기록. 진행 중 업무에도 이력이 있다.')
    columns = [(64, '1 원본 과거 이력', 'A와 B에 같은 기록을 제공'),
               (900, '2 재사용을 위한 파생 자료', '원본과 dependency를 가진 가공 정보'),
               (1736, '3 현재 요청의 Context', '이번 목적과 권한에 맞춘 근거 묶음')]
    for x, title, note in columns:
        p.box(x, 240, 760, 910, 'white', LINE, 0)
        p.text(x + 24, 262, title, 30, bold=True)
        p.text(x + 24, 313, note, 23, MUTED)
    rows = [
        ('input', '사용자 입력\nRequest Controller: 질문 / 지시 / 확인 답변'),
        ('request', '요청의 해석과 변경\nRequest Controller: 채택 의미 / 정정 revision'),
        ('work', '업무 연결과 수신 사건\nTask Manager: 위임 / receipt / 결과 version'),
        ('response', '응답 준비와 실제 전달\nResponse Manager: 원문 / Text와 Voice 범위'),
        ('source', '사용 자료와 결과 참조\nContext Manager의 읽기 receipt / 채택 근거')]
    for j, (key, name) in enumerate(rows):
        p.node(key, 88, 382 + j * 142, 712, 120, name, kind='data')
    p.node('derived', 924, 382, 712, 190,
           '요약 / 검색 색인 / 과거 관계\n원본 ID와 revision / 구간 / 생성 방식\n파생값의 불확실성과 생산 범위', kind='data')
    p.text(928, 622, ['A: 필요할 때 조합, 유효 cache 재사용',
                      'B: 공통 관계를 생산 / 게시 / 유지',
                      '원본 사실과 현재 의미의 권위는 공통'], 25, leading=43)
    p.node('memory', 924, 848, 712, 185,
           'User Memory는 별도 원본\n명시적으로 저장한 선호 / routine / stable fact\n과거 반복만으로 선호 등록하지 않음', kind='data')
    p.node('context', 1760, 382, 712, 246,
           'R23에 필요한 자료\n실제 설명 E20[0:52] 원문\n제품 비교 D21 개정 2 참조\n견적 D22 개정 1 참조\n출처 / 유효성 / 읽은 범위 / 누락', kind='data')
    p.text(1764, 677, ['이번 조건과 Task 연결은 지금 판단',
                      'Request Interpreter: 의미 제안',
                      'Request Controller: 확인 / 채택',
                      'Downstream Agent: 기준 적용 / 제안서 작성'],
           24, leading=48)
    p.text(1764, 938, ['당시 기록 ≠ 지금 상태',
                      '생성된 설명 ≠ 실제 전달한 설명',
                      '결과 참조 ≠ 결과 본문 전체 보관'], 24, leading=42)
    p.line([(64, 1202), (2496, 1202)], LINE, arrow=False)
    p.text(64, 1234, ['과거 동의는 현재 권한이 아니다. 원본 수정 / 삭제 / 철회와 채널별 전달 범위를 확인한다.',
                     'PC 자료 전체와 Agent 내부 기록은 포함하지 않는다. 파생 기억은 원본을 대신하는 새로운 업무 권위가 아니다.'],
           24, leading=43)
    p.text(64, 1357, '접힌 종이 = 자료의 종류. 세 열은 설명 구분이며 새 Component나 세 개의 물리 DB가 아니다.', 22, MUTED)
    return p


def walkthrough():
    p = HistoryPlate('dp45-history-walkthrough', '같은 과거 기록을 이번 요청의 근거로 만드는 두 구조',
                     'R23: “지난번 설명한 평가 기준으로 제품 비교와 견적을 이번 제안서에 반영해줘.” 후보 ID는 조회 후 발견한다.')
    for x, side, title in [(64, 'A', 'A 원본 서비스 조합형'),
                            (1312, 'B', 'B 공통 파생 기억 저장소형')]:
        key = lambda n: side + n
        p.text(x, 236, title, 32, bold=True)
        p.node(key('original'), x + 20, 298, 1130, 136,
               '같은 원본: Request Controller의 입력 / 채택 기록\nResponse Manager: E20 원문 / P20 실제 전달[0:52]\nTask Manager: T21 → D21 개정 2 / T22 → D22 개정 1', kind='data')
        p.component(key('cm'), x + 410, 508, 740, 610, 'Context Manager')
        p.component(key('ri'), x + 20, 906, 330, 212, 'Request Interpreter')
        p.text(x + 40, 989, ['현재 R23 / 허용 범위', '후보 발견 / 추가 조회',
                            '현재 조건과 Task 의미 제안'], 22, leading=33)
        if side == 'A':
            p.node(key('composer'), x + 450, 624, 660, 110,
                   'Context Composer\n2 원본 후보 읽기 / 유효 cache / 이번 조합', owner=key('cm'))
            p.different(key('composer'))
            p.node(key('cache'), x + 450, 852, 310, 100,
                   'Source View Cache\n검증한 자료 재사용', kind='store', owner=key('cm'))
            p.node(key('set'), x + 800, 852, 310, 100,
                   'Request Evidence Set\n이번 query의 근거', kind='store', owner=key('cm'))
            p.different(key('set'))
            p.edge(key('original'), key('composer'), sd=195)
            p.label(x + 745, 463, '원본 port의 검색 / 범위 읽기 / 반환')
            p.edge(key('composer'), key('cache'), sd=-175)
            p.edge(key('composer'), key('set'), sd=175)
            p.edge(key('ri'), key('composer'), sp='R', tp='L',
                   via=[(x + 384, 1012), (x + 384, 679)])
            p.edge(key('composer'), key('ri'), sp='L', tp='R', sd=20, td=14,
                   via=[(x + 398, 699), (x + 398, 1026)], ret=True)
            p.label(x + 22, 741, '1 현재 조회 → / 3 Bundle 반환 ←', size=20)
            p.text(x + 452, 978, ['필요한 원본을 추가 조회 / 누락 해결',
                                 '요청별 구성 책임은 Composer에 유지'], 22, leading=36)
        else:
            p.node(key('publisher'), x + 450, 624, 310, 110,
                   'Memory Publisher\n2 관계 생산 / 검증', owner=key('cm'))
            p.node(key('reader'), x + 800, 624, 310, 110,
                   'Evidence Reader\n4 게시 근거 조회', owner=key('cm'))
            p.node(key('repo'), x + 450, 820, 660, 160,
                   'Episodic Memory Repository\nPRESENTS / HAS_RESULT / EXPLAINS_ITEM\n3 관계와 Coverage를 같은 개정으로 게시',
                   kind='store', owner=key('cm'))
            for name in ('publisher', 'reader', 'repo'):
                p.different(key(name))
            p.edge(key('original'), key('publisher'), sd=315, td=0,
                   via=[(x + 900, 590), (x + 605, 590)])
            p.label(x + 25, 463, '1 확정 변경 → 원본 읽기', size=21)
            p.edge(key('publisher'), key('repo'), sd=0, td=-175)
            p.edge(key('repo'), key('reader'), sp='T', tp='B', sd=175)
            p.edge(key('ri'), key('reader'), sp='R', tp='B',
                   via=[(x + 384, 1012), (x + 384, 783), (x + 955, 783)])
            p.edge(key('reader'), key('ri'), sp='B', tp='R', sd=9, td=14,
                   via=[(x + 964, 797), (x + 398, 797), (x + 398, 1026)], ret=True)
            p.label(x + 25, 741, '4 현재 조회 → / 5 Bundle 반환 ←', size=20)
            p.text(x + 452, 1000, ['원문 locator / 권한 / revision 확인',
                                 '미생산: 생산 / 게시 후 Reader 재조회',
                                 '표현 미지원: 제한 또는 명시 B+'], 22, leading=34)
        p.text(x + 22, 564, [('현재 조회가 원본 조합을 시작' if side == 'A' else
                             '과거 생산과 현재 조회는 다른 시작점'),
                            '조건부 의미 가공: Model Access',
                            '→ 같은 클라우드 의미 LLM',
                            '현재 해석 호출도 양안에 유지'], 21, leading=36)
        p.text(x + 20, 1163, ['같은 반환: 실제 설명[0:52] + 제품/견적 결과 참조 + 출처 / revision / 누락',
                            '이 예시에는 REVISES가 없다. 정정 채택 후에만 추가한다.'], 23, leading=36)
    p.line([(64, 1262), (2496, 1262)], LINE, arrow=False)
    p.text(64, 1288, ['공통 후속: Request Interpreter의 현재 의미 제안 → Request Controller의 채택 → Task Manager / Agent Gateway의 위임',
                     '기준 적용과 제안서 작성은 Downstream Agent가 수행. 입력은 생산 대기를 하지 않는다.'], 24, leading=39)
    p.text(64, 1380, '검정 = 공통, 살구색 = 구성 / 생산 / 읽기 책임의 차이. 화살표는 왕복 읽기의 시작 경로를 축약. 설계 예시 / 미선정 / 미측정.', 21, MUTED)
    return p


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    drift = []
    for plate in (definition(), walkthrough()):
        plate.validate()
        for ext, data in [('svg', plate.svg()), ('drawio', plate.drawio())]:
            root = ET.fromstring(data)
            if ext == 'drawio':
                ids = [cell.get('id') for cell in root.findall('.//mxCell')]
                assert len(ids) == len(set(ids))
                for cell in root.findall('.//mxCell'):
                    for attr in ('source', 'target', 'parent'):
                        assert not cell.get(attr) or cell.get(attr) in ids
            destination = OUT / (plate.slug + '.' + ext)
            if args.check:
                if not destination.exists() or destination.read_text() != data:
                    drift.append(destination.name)
            else:
                destination.write_text(data)
    if drift:
        raise SystemExit('Diagram drift: ' + ', '.join(drift))
    print('PASS: 2 history diagram pairs; source parity, XML, ownership, bounds and routes')


if __name__ == '__main__':
    main()
