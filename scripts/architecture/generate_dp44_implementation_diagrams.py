"""DP44 proposed code organization, not an executable VIA implementation."""
from __future__ import annotations

import argparse
from pathlib import Path
import xml.etree.ElementTree as ET

from stage4_diagram_design import Plate, INK, MUTED, LINE
from dp_comparison_structures import APRICOT, DIFFERENCE_STROKE

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


def common(side, title, thesis):
    p = ImplementationPlate(side, title, thesis)
    p.component('ri', 64, 280, 480, 305, 'Request Interpreter')
    p.node('interpret', 104, 370, 400, 125,
           'InterpretAsync\n근거 조회 / 의미 제안\n긴 작업은 제어 루프 밖', owner='ri')
    p.component('model', 64, 625, 480, 160, 'Model Access')
    p.node('api', 104, 702, 400, 60, 'API adapter / bounded admission', owner='model')
    p.edge('interpret', 'api')
    p.label(114, 593, '같은 Context Manager / 의미 API', size=18)

    p.component('tm', 64, 835, 480, 355, 'Task Manager')
    p.node('task_handler', 104, 925, 400, 110,
           'Task command handler\nAgent Gateway → Downstream Agent\n취소 요청과 실제 종료를 구별', owner='tm')
    p.node('task_facts', 104, 1070, 400, 105,
           '확인된 Task 원본 / durable inbox\n질문 / 결과 / command 상태', kind='store', owner='tm')

    p.component('rc', 604, 280, 680, 910, 'Request Controller')
    p.component('rm', 1344, 280, 620, 910, 'Response Manager')
    p.component('im', 2024, 280, 472, 910, 'Interaction Manager')
    p.node('turn', 2064, 370, 392, 125,
           'Turn-Taking Control\n입력 감지 → 즉시 local stop\n현재 input / 출력 epoch', owner='im')
    p.node('io', 2064, 940, 392, 120,
           'Channel I/O\nrelease / epoch 재검사\n실제 표시 / 재생 / receipt', owner='im')
    p.node('playback', 2064, 1080, 392, 105,
           'Playback State\n장치 상태 / receipt buffer', kind='store', owner='im')
    p.edge('io', 'playback')
    p.text(2068, 545, ['capture / ASR는 계속 진행',
                       '전사 확정 ≠ 의미 해소',
                       '발화 감지 ≠ Task 취소',
                       '새 입력은 미전송 효과 hold'], 22, MUTED, leading=40)
    p.text(2068, 775, ['8 허용 release를 실제 전달',
                       '9 receipt는 전달 이후 생성'], 21, leading=36)

    p.node('composer', 1384, 370, 540, 125,
           'Response Composer\nPrepareAsync: 내용 / 음성 준비\n동일 Model Access / 준비 Job 상태', owner='rm')
    p.node('publish', 1384, 940, 540, 120,
           'Publication Control\n7 owner 현재성 / 권한 / 차례 / 중복 검사\n이 Module은 다른 Stage의 작업을 지시하지 않음', owner='rm')
    p.node('outbox', 1384, 1080, 540, 105,
           'Publication Outbox (내구 원본)\npublication ID / 실제 전달 범위', kind='store', owner='rm')
    p.edge('publish', 'outbox')
    p.edge('publish', 'io', sp='R', tp='L', label='8 release', at=(1953, 966))
    p.edge('io', 'outbox', sp='B', tp='R', sd=-115,
           via=[(2145, 1070), (1984, 1070), (1984, 1132.5)], ret=True)
    p.label(1950, 1143, '9', size=20)
    p.text(64, 1215, 'State Store: 각 owner의 확정 기록 저장', 18, MUTED)

    p.line([(64, 1255), (2496, 1255)], LINE, arrow=False)
    p.text(64, 1274, ['공통 실행 기반: bounded async executor + blocking worker pool. Module마다 전용 thread나 process를 두지 않는다.',
                     '짧은 제어 루프는 긴 API 작업을 기다리지 않는다. 취소는 먼저 결과를 무효화하고 provider에 중단을 요청한다.'],
           23, leading=39)
    p.text(64, 1365, '직각 = Component  /  둥근 박스 = 코드 Module  /  원통 = Queue 또는 상태  /  살구색 = A/B 실행 조직의 차이', 20, MUTED)
    p.text(64, 1400, '실선 → 요청 / 자료 전달    점선 → 완료 / 확정 사실 반환    번호는 본문 실행 표와 대응', 18, MUTED)
    p.text(2496, 1400, '구현을 가정한 설계안 / 미선정 / 미측정', 18, MUTED, align='right')
    return p


def a():
    p = common('A', 'A 구현 구조 — 완료를 중앙에 반환하고 다음 작업을 지시',
               '대기열 + 짧은 Dispatcher 루프 + 비동기 Job. priority queue의 순서와 실행 중 작업의 취소는 별도다.')
    p.node('control_q', 644, 370, 280, 125,
           'Control Lane (RAM)\nonset / hold / invalidate\n일반 event와 분리 수용', kind='store', owner='rc')
    p.node('event_q', 964, 370, 280, 125,
           'Event Mailbox (RAM)\nInput / Task / JobDone\nreceipt / 우선순위 + seq', kind='store', owner='rc')
    p.node('dispatcher', 714, 590, 460, 130,
           'Dialogue Dispatcher\nPump → 짧은 Handle(event)\n채택 / 유효성 / 다음 Job 지시', owner='rc')
    p.node('progress', 644, 895, 280, 190,
           'Dialogue Progress State\nRAM / waiting IDs\n후속 실행 / 후보 참조\n확정 원본은 복제하지 않음', kind='store', owner='rc')
    p.node('jobs', 964, 895, 280, 190,
           'Job Index (RAM)\njob ID / Request revision\nsource refs / cancel handle\nRUNNING → terminal', kind='store', owner='rc')
    p.node('candidate', 1384, 570, 540, 140,
           'Prepared Candidate (RAM)\n내용 / 음성 / 시작 source refs\n중앙에는 이 자료의 참조를 반환', kind='store', owner='rm')
    p.different('control_q', 'event_q', 'dispatcher', 'progress', 'jobs')

    p.edge('turn', 'control_q', sp='T', tp='T', sd=150, td=100,
           via=[(2410, 235), (884, 235)])
    p.label(900, 211, '1 onset / 제어 event', size=19)
    p.edge('turn', 'event_q', sp='T', tp='T', sd=170,
           via=[(2430, 257), (1104, 257)])
    p.label(1470, 235, '1 확정 Input', size=19)
    p.edge('task_facts', 'event_q', sp='R', tp='L',
           via=[(568, 1122.5), (568, 552), (944, 552), (944, 432.5)])
    p.label(112, 805, '1 확인된 질문 / 결과 / 상태', size=18)
    p.edge('control_q', 'dispatcher', sd=0, td=-130,
           via=[(784, 566), (814, 566)])
    p.edge('event_q', 'dispatcher', td=130,
           via=[(1104, 566), (1074, 566)])
    p.label(803, 546, '2 우선 처리 / fairness', size=19)

    p.edge('dispatcher', 'interpret', sp='L', tp='R', sd=-20,
           via=[(584, 635), (584, 447.5)], td=15)
    p.label(620, 750, '3 InterpretAsync 시작', size=19)
    p.edge('interpret', 'event_q', sp='R', tp='T', sd=-20,
           via=[(562, 412.5), (562, 352), (1104, 352)], ret=True)
    p.label(605, 338, '4 JobDone → Mailbox', size=19)

    p.edge('dispatcher', 'composer', sp='R', tp='L', sd=-20,
           via=[(1304, 635), (1304, 447.5)], td=15)
    p.label(1390, 740, '5 Prepare / cancel → / 6 JobDone ←', size=19)
    p.edge('composer', 'event_q', sp='L', tp='R', sd=-20, td=-20,
           via=[(1320, 412.5)], ret=True)
    p.edge('composer', 'candidate')
    p.edge('dispatcher', 'publish', sp='R', tp='L', sd=30,
           via=[(1312, 685), (1312, 1000)])
    p.label(1390, 815, '7 유효 후보의 게시 요청', size=20)
    p.text(1390, 858, '준비 완료만으로 재생하지 않음', 21, MUTED)
    p.edge('dispatcher', 'progress', sd=-130,
           via=[(814, 795), (784, 795)])
    p.edge('dispatcher', 'jobs', sd=160)
    p.label(729, 840, '짧은 루프에서 조회 / 갱신', size=18)
    p.edge('dispatcher', 'task_handler', sp='B', tp='R', sd=-220,
           via=[(724, 780), (584, 780), (584, 1015)], td=35)
    p.label(114, 1040, '채택한 취소 / 변경 명령', size=18)
    p.edge('outbox', 'dispatcher', sp='B', tp='L', td=45,
           via=[(1654, 1215), (586, 1215), (586, 700)], ret=True)
    p.label(708, 1223, '10 내구 Presented → Request Controller 질문 focus', size=18)
    return p


def b():
    p = common('B', 'B 구현 구조 — Stage의 산출물을 다음 소비자의 입력으로 연결',
               '고정 typed-channel wiring + Stage별 짧은 Pump + 국소 Job/Window. 모든 완료를 회수하는 중앙 Dispatcher는 없다.')
    p.node('input_q', 644, 370, 280, 125,
           'Input Mailbox (RAM)\nControl / Data / Done\n제어 lane 별도 수용', kind='store', owner='rc')
    p.node('notice_q', 964, 370, 280, 125,
           'Notice Mailbox (RAM)\nControl / Task / Done\n제어 lane 별도 수용', kind='store', owner='rc')
    p.node('input_stage', 644, 590, 280, 130,
           'Input Resolution Stage\nPump / HandleControl\nInterpretAsync / 채택', owner='rc')
    p.node('notice_stage', 964, 590, 280, 130,
           'Task Notice Stage\nPump / NoticeAsync\n연결 / 질문 등록 / admission', owner='rc')
    p.node('input_window', 644, 895, 280, 190,
           'Input Window (RAM)\ninput / job ID / revision\nsource refs / cancel handle\n자기 Job과 대기만 관리', kind='store', owner='rc')
    p.node('notice_window', 964, 895, 280, 190,
           'Notice Window (RAM)\nTask / question revision\n미전달 항목 / cancel handle\n자기 Job과 대기만 관리', kind='store', owner='rc')
    p.node('candidate_q', 1384, 535, 240, 120,
           'Candidate Queue\nbounded / credit\ncandidate ID', kind='store', owner='rm')
    p.node('condition_q', 1684, 535, 240, 120,
           'Condition Queue\nadmission / source\nsettled / past delivery', kind='store', owner='rm')
    p.node('join', 1384, 705, 540, 90,
           'Publication Join\nID / revision이 맞는 입력끼리 조건 검사', owner='rm')
    p.node('publication_window', 1384, 815, 540, 115,
           'Publication Window (RAM)\ncandidate별 필수 참조 / 대기 / 폐기', kind='store', owner='rm')
    p.different('input_q', 'notice_q', 'input_stage', 'notice_stage',
                'input_window', 'notice_window', 'candidate_q', 'condition_q',
                'join', 'publication_window')

    p.edge('turn', 'input_q', sp='T', tp='T', sd=150, td=100,
           via=[(2410, 235), (884, 235)])
    p.label(900, 211, '1 onset / Input / 제어', size=19)
    p.edge('turn', 'notice_q', sp='T', tp='T', sd=170,
           via=[(2430, 257), (1104, 257)])
    p.label(1470, 235, '1 control fan-out', size=18)
    p.edge('task_facts', 'notice_q', sp='R', tp='L',
           via=[(568, 1122.5), (568, 552), (944, 552), (944, 432.5)])
    p.label(112, 805, '1 확인된 Task event → Notice port', size=18)
    p.edge('input_q', 'input_stage')
    p.edge('notice_q', 'notice_stage')
    p.label(684, 530, '2 소비 / 자기 완료 처리', size=19)
    p.edge('input_stage', 'interpret', sp='L', tp='R', sd=-20,
           via=[(584, 635), (584, 447.5)], td=15)
    p.edge('interpret', 'input_q', sp='R', tp='L', sd=-20, td=-20,
           via=[(562, 412.5)], ret=True)
    p.label(606, 338, '3 자기 Done port로 반환', size=19)
    p.edge('input_stage', 'input_window')
    p.edge('notice_stage', 'notice_window')
    p.label(724, 854, '국소 조회 / 갱신 / cancel', size=18)

    p.edge('input_stage', 'composer', sp='B', tp='L', sd=100, td=15,
           via=[(884, 770), (1304, 770), (1304, 447.5)])
    p.edge('notice_stage', 'composer', sp='R', tp='L', td=35,
           via=[(1320, 655), (1320, 467.5)])
    p.label(962, 741, '4 채택 / Notice / 폐기 제어', size=18)
    p.edge('input_stage', 'condition_q', sp='B', tp='T', sd=-100, td=-50,
           via=[(684, 802), (1328, 802), (1328, 525), (1754, 525)])
    p.edge('notice_stage', 'condition_q', sp='B', tp='T', sd=100, td=50,
           via=[(1204, 815), (1336, 815), (1336, 521), (1854, 521)])
    p.label(1400, 506, '4 조건 port로 별도 fan-out', size=18)
    p.edge('composer', 'candidate_q', sd=-150)
    p.edge('candidate_q', 'join', td=-150)
    p.edge('condition_q', 'join', td=150)
    p.label(1430, 671, '5 준비 후보 + 참조별 조건', size=18)
    p.edge('join', 'publication_window')
    p.edge('join', 'publish', sp='R', tp='R',
           via=[(1940, 750), (1940, 1000)])
    p.label(1944, 850, '6', size=18)
    p.edge('input_stage', 'task_handler', sp='L', tp='R', sd=35,
           via=[(584, 690), (584, 1015)], td=35)
    p.label(114, 1040, '채택한 취소 / 변경 명령', size=18)
    p.edge('outbox', 'input_stage', sp='B', tp='L', td=45,
           via=[(1654, 1215), (586, 1215), (586, 700)], ret=True)
    p.label(708, 1223, '10 내구 Presented → Request Controller 질문 focus', size=18)
    p.edge('outbox', 'condition_q', sp='R', tp='R', td=15,
           via=[(1998, 1132.5), (1998, 610)], ret=True)
    return p


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    drift = []
    for plate in (a(), b()):
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
