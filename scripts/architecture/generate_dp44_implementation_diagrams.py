"""DP44 proposed code organization, not an executable VIA implementation."""
from __future__ import annotations

import argparse
from pathlib import Path
import xml.etree.ElementTree as ET

from stage4_diagram_design import Plate, INK, MUTED, LINE
from dp_comparison_structures import APRICOT, DIFFERENCE_STROKE
from dp44_runtime_scene import a, b

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/diagrams'


class ImplementationPlate(Plate):
    def __init__(self, side, title, thesis):
        super().__init__('dp44-implementation-' + side.lower(), 'DP-44', title, thesis, height=1440)
        self.differences = set()

    @staticmethod
    def text_geometry(item):
        size = 22 if item['w'] >= 390 else 20
        if item['id'] == 'task_handler':
            size = 20
        leading = size * 1.3
        top = item['y'] + (item['h'] - len(item['name']) * leading) / 2 - 2
        if item['type'] == 'store':
            top = max(top + 9, item['y'] + 34)
        return top, size, leading

    def different(self, *keys):
        self.differences.update(keys)

    def svg(self):
        root = ET.fromstring(super().svg())
        ns = {'s': 'http://www.w3.org/2000/svg'}
        for group in root.findall('.//s:g', ns):
            key = group.get('id')
            if key in self.meta and self.meta[key]['parent']:
                group.set('data-owner', self.meta[key]['parent'])
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


def common_path():
    p = ImplementationPlate('COMMON', 'DP-44의 공통 대화 경로와 A/B 비교 범위',
                            '핵심은 Interaction Manager, Request Controller, Response Manager의 협력. 업무 사실 확인과 업무 제어는 양안 공통이다.')
    p.slug = 'dp44-common-path'
    p.component('im', 64, 310, 600, 560, 'Interaction Manager')
    p.component('rc', 824, 310, 800, 560, 'Request Controller')
    p.component('rm', 1784, 310, 712, 560, 'Response Manager')
    p.text(104, 410, ['계속 듣기와 입력 취득', '입력 시작 알림과 전사 전달',
                      '발화 중 끼어들면 즉시 중단', '허용된 Text와 음성 실제 전달',
                      '실제 전달 범위 확인'], 28, leading=58)
    p.text(864, 410, ['입력 해석 요청과 의미 채택', '대화, Request와 질문 연결',
                      '정정 범위와 미전송 hold', '채택한 입력을 후속 실행에 연결',
                      'Task Manager와 업무 제어'], 28, leading=58)
    p.text(1824, 410, ['응답 내용과 음성 준비', '대기 후보의 게시 차례 관리',
                       '현재성, 권한과 중복 최종 검사', 'Interaction Manager에 release',
                       '실제 전달 기록의 내구 저장'], 27, leading=58)
    p.node('user', 64, 990, 600, 175, '사용자\n말하기 / 듣기 / 읽기', kind='external')
    p.component('task', 864, 990, 620, 200, 'Task Manager')
    p.component('gateway', 1610, 990, 420, 200, 'Agent Gateway')
    p.node('agent', 2140, 1020, 356, 140, 'Downstream Agent\n비동기 질문 / 결과', kind='external')
    p.text(894, 1080, ['업무 원본 확인과 저장', '질문 / 결과 / command 상태'], 23, leading=35)
    p.text(1640, 1080, ['외부 사건 접수', '명령 전달'], 23, leading=35)
    p.edge('user', 'im', sp='T', tp='B', sd=-140, td=-140)
    p.label(100, 916, '1 발화', size=20)
    p.edge('im', 'rc', sp='R', tp='L')
    p.label(682, 541, '2 입력\nonset / 전사', size=20)
    p.edge('agent', 'gateway', sp='L', tp='R', ret=True)
    p.edge('gateway', 'task', sp='L', tp='R', ret=True)
    p.label(2043, 1058, '3 접수', size=18)
    p.label(1496, 1058, '3 확인', size=18)
    p.edge('task', 'rc', sp='T', tp='B', sd=140, td=90, ret=True)
    p.label(1326, 916, '4 확인 사실', size=18)
    p.edge('rc', 'task', sp='B', tp='T', sd=-90, td=-40)
    p.label(887, 916, '공통 업무 명령', size=18)
    p.text(864, 1223, 'Request Controller ↔ Task Manager: A/B 공통', 20, MUTED)
    p.edge('rc', 'rm', sp='R', tp='L')
    p.label(1640, 541, '5 채택 입력\n허용 Notice', size=20)
    p.edge('rm', 'im', sp='T', tp='T', via=[(2140, 250), (364, 250)])
    p.label(922, 220, '6 내용 / 음성 / release → 실제 출력', size=22)
    p.edge('im', 'user', sp='B', tp='T', sd=140, td=140)
    p.label(440, 916, '7 실제 전달', size=20)
    p.edge('im', 'rm', sp='B', tp='B', sd=180, td=300,
           via=[(544, 890), (2440, 890)], ret=True)
    p.label(1500, 893, '8 actual receipt / 내구 저장', size=18)
    p.edge('rm', 'rc', sp='L', tp='R', sd=140, td=140, ret=True)
    p.label(1640, 741, '8 Presented\n질문 focus', size=18)
    p.line([(64, 1255), (2496, 1255)], LINE, arrow=False)
    p.text(64, 1277, ['A/B 차이: 입력과 확인된 업무 사건으로 시작한 이해 / 준비 / 게시의 완료, 대기, 정정과 취소를 누가 이어가는가.',
                     'Request Interpreter, Context Manager, Policy Manager, State Store, Model Access의 공통 기능은 아래 상세 구조에서 유지한다.'],
           23, leading=39)
    p.text(64, 1377, '직각 = Component  /  육각형 = 사용자 또는 외부 Agent  /  실선 = 입력 및 요청  /  점선 = 확인된 사건과 전달 사실', 20, MUTED)
    p.text(2496, 1410, '공통 계약 설명 / A/B 미선정 / 미측정', 18, MUTED, align='right')
    return p


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    drift = []
    for plate in (common_path(), a(), b()):
        plate.validate()
        for ext, data in [('svg', plate.svg()), ('drawio', plate.drawio())]:
            root = ET.fromstring(data)
            if ext == 'drawio':
                ids = [cell.get('id') for cell in root.findall('.//mxCell')]
                assert len(ids) == len(set(ids))
                for cell in root.findall('.//mxCell'):
                    for attr in ('source', 'target', 'parent'):
                        assert not cell.get(attr) or cell.get(attr) in ids
            dest = OUT / (plate.slug + '.' + ext)
            if args.check:
                if not dest.exists() or dest.read_text() != data:
                    drift.append(dest.name)
            else:
                dest.write_text(data)
                print(dest.relative_to(ROOT))
    if drift:
        raise SystemExit('Diagram drift: ' + ', '.join(drift))
    print('PASS: DP44 implementation-view geometry, ownership, XML and artifact consistency')


if __name__ == '__main__':
    main()
