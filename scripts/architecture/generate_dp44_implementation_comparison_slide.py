"""One editable DP44 comparison page derived from the detailed A/B runtime views.

This page supplements the existing comparison. It is a documentation design,
not an implemented VIA runtime or a measured preference for either option.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import xml.etree.ElementTree as ET

from generate_dp_comparison_slides import Comparison
from generate_dp_background_slides import INK, MUTED, BLUE, RED, LINE


ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/presentations_files/dp-comparison'
SLUG = 'dp44-implementation-comparison'
ORANGE = '#B8753F'
PALE = '#FFF3E5'
PALE_BLUE = '#EAF2FB'
PALE_GRAY = '#F4F7FA'


def slide():
    s = Comparison.__new__(Comparison)
    s.number, s.slug, s.items = 44, SLUG, []
    s.caption = 'DP44. 중앙 회수와 단계별 완료 처리의 구현 구조'
    s.rect(0, 0, 1920, 1080, 'white', 'none')
    s.text(54, 30, 1770, [s.caption], 42, INK, True)
    s.line([(54, 103), (1866, 103)], color=LINE, arrow=False, width=2)
    s.text(54, 123, 1812, ['같은 사건: R1 rev1의 응답 Job J1 실행 중 새 발화와 메일 질문 도착, 이후 J1이 늦게 완료'], 22, INK)
    s.text(54, 158, 1812, ['공통: Interaction Manager가 입력을 계속 받고 즉시 출력을 멈춘다. Task Manager는 확인된 업무 사건을 제공한다.'], 20, MUTED)

    def label(x, y, w, value, size=18, color=INK, bold=False, center=False):
        s.text(x+w/2 if center else x, y, w, [value], size, color, bold, 'center' if center else 'left')

    def box(x, y, w, h, title, body='', fill='white', stroke=INK, title_size=19, body_size=16):
        s.rect(x, y, w, h, fill, stroke)
        label(x+10, y+9, w-20, title, title_size, INK, True)
        if body:
            label(x+10, y+36, w-20, body, body_size, MUTED)

    def queue(x, y, w, h, title, body='', small=False):
        s.rect(x, y, w, h, 'white', ORANGE)
        label(x+9, y+7, w-18, title, 16 if small else 18, INK, True)
        if body: label(x+9, y+33, w-18, body, 15, MUTED)
        # Four cells and an exit arrow make this visually distinct from a state store.
        cell = 15 if small else 18
        gap = 4
        first = x+w-(4*cell+3*gap)-12
        yy = y+h-cell-7
        for j in range(4): s.rect(first+j*(cell+gap), yy, cell, cell, 'white', MUTED)

    def state(x, y, w, h, title, fill=PALE):
        s.add('store', x=x, y=y, w=w, h=h, fill=fill, stroke=ORANGE, dashed=False, store_radius=7)
        label(x+9, y+(23 if h>=50 else 18), w-18, title, 17 if h>=50 else 15, INK, False)

    def edge(points, kind='data', arrow=True, width=2):
        color = {'data':BLUE, 'done':BLUE, 'control':RED, 'call':INK, 'state':MUTED}[kind]
        s.line(points, color=color, arrow=arrow, dashed=kind in ('done', 'control', 'state'), width=width)

    # Two option headers match the earlier page without replacing it.
    s.rect(54, 207, 870, 55, PALE_BLUE, LINE)
    s.rect(996, 207, 870, 55, '#FFF5DB', LINE)
    label(66, 220, 850, 'A  중앙 Mailbox와 Dispatcher가 다음 실행을 결정', 25, INK, True, True)
    label(1008, 220, 850, 'B  각 Stage가 자기 Done을 처리하고 다음 Queue로 전달', 25, INK, True, True)

    # A: one event consumer owns the job index and continuation state.
    s.rect(60, 282, 865, 325, 'white', INK)
    label(77, 291, 830, 'Request Controller', 21, INK, True)
    label(84, 315, 310, 'InputStarted', 16, RED, True)
    label(428, 315, 470, 'InputReady / TaskEvent / JobDone / Presented', 16, BLUE, True)
    queue(83, 338, 320, 70, 'Control Lane', 'onset / adopted Invalidate')
    queue(428, 338, 470, 70, 'Event Mailbox', 'priority + seq + aging')
    edge([(73, 327), (73, 373), (83, 373)], 'control')
    box(220, 460, 540, 73, 'Dialogue Dispatcher: Pump / Handle(event)',
        '짧은 전이에서 의미 채택, 다음 Job 지시, 오래된 결과 폐기', PALE, ORANGE, 21, 17)
    state(83, 548, 350, 51, 'Job Index: revision / Future / CancelHandle')
    state(465, 548, 433, 51, 'Dialogue Progress State: 대기 ID / 후보 참조')
    edge([(243, 408), (243, 437), (335, 437), (335, 460)], 'call')
    edge([(663, 408), (663, 437), (640, 437), (640, 460)], 'call')
    edge([(342, 533), (342, 541), (260, 541), (260, 548)], 'state')
    edge([(640, 533), (640, 541), (680, 541), (680, 548)], 'state')
    edge([(285, 533), (285, 548)], 'control')

    box(83, 648, 245, 74, 'Request Interpreter', 'InterpretAsync', 'white', INK, 18, 16)
    s.rect(340, 619, 573, 123, 'white', INK)
    label(354, 624, 540, 'Response Manager', 20, INK, True)
    box(355, 648, 245, 74, 'Response Composer', 'PrepareAsync', 'white', INK, 18, 16)
    box(627, 648, 270, 74, 'Publication Control', '현재성 검사 후 게시', 'white', INK, 18, 16)
    edge([(220, 497), (192, 497), (192, 636), (205, 636), (205, 648)], 'call')
    edge([(442, 533), (442, 611), (477, 611), (477, 648)], 'call')
    edge([(760, 497), (913, 497), (913, 637), (762, 637), (762, 648)], 'call')
    # Both long jobs return a completion message to the same central queue.
    edge([(83, 686), (69, 686), (69, 430), (416, 430), (416, 373), (428, 373)], 'done')
    edge([(600, 686), (613, 686), (613, 755), (917, 755), (917, 419), (907, 419), (907, 373), (898, 373)], 'done')
    label(87, 790, 800, 'J1 취소 요청 뒤 늦은 완료: Event Mailbox 수신, Dispatcher가 rev1 결과 폐기', 18, INK, True)
    label(87, 818, 800, 'Queue 순서와 실행 중 Job의 CancelHandle은 별개', 17, MUTED)

    # B: each stage has its own bounded intake and local completion loop.
    s.rect(996, 282, 870, 247, 'white', INK)
    label(1012, 291, 838, 'Request Controller', 21, INK, True)
    label(1013, 316, 830, 'InputStarted와 채택된 Invalidate는 관련 Control Queue마다 복사', 16, RED, True)
    edge([(1005, 335), (1430, 335)], 'control', arrow=False)
    edge([(1005, 335), (1005, 653)], 'control', arrow=False)
    edge([(1430, 335), (1430, 653)], 'control', arrow=False)
    for left,yy in ((1022,407),(1452,407),(1022,655),(1452,655)):
        edge([(1005 if left==1022 else 1430, yy), (left, yy)], 'control')

    def stage(x, y, w, h, title, queue_names, handler, state_name):
        s.rect(x, y, w, h, PALE, ORANGE)
        label(x+10, y+8, w-20, title, 19, INK, True)
        qw=(w-42)/3
        for j, name in enumerate(queue_names):
            queue(x+10+j*(qw+11), y+42, qw, 52, name, small=True)
        box(x+10, y+99, w-20, 35, handler, fill='white', stroke=ORANGE, title_size=17)
        state(x+10, y+142, w-20, 40, state_name)
        # Async completion is enqueued into this stage's Done Queue.
        if 'Done' in queue_names:
            qx=x+10+2*(qw+11)
            edge([(x+w-10, y+124), (x+w-5, y+124), (x+w-5, y+69), (qx+qw, y+69)], 'done', width=1.7)

    stage(1012, 340, 408, 182, 'Input Resolution Stage',
          ('Control', 'Input', 'Done'), 'Pump / InterpretAsync', 'Input Window (RAM)')
    stage(1442, 340, 408, 182, 'Task Notice Stage',
          ('Control', 'Task', 'Done'), 'Pump / NoticeAsync', 'Notice Window (RAM)')
    s.rect(996, 549, 870, 259, 'white', INK)
    label(1012, 558, 838, 'Response Manager', 21, INK, True)
    stage(1012, 588, 408, 182, 'Response Composer',
          ('Control', 'Prepare', 'Done'), 'Pump / PrepareAsync', 'Prepare Job Index (RAM)')
    stage(1442, 588, 408, 182, 'Publication Join',
          ('Control', 'Candidate', 'Condition'), 'Pump / match ID and revision', 'Publication Window (RAM)')
    # The Join's eligible output enters the common publication path below.
    edge([(1216, 522), (1216, 588)], 'data')
    edge([(1646, 522), (1646, 539), (1310, 539), (1310, 588)], 'data')
    edge([(1420, 656), (1442, 656)], 'data')
    edge([(1646, 522), (1646, 588)], 'data')
    edge([(1646, 770), (1646, 791)], 'data')
    label(1330, 783, 300, 'eligible → Publication Control', 16, BLUE)
    label(1015, 824, 835, 'J1 늦은 완료: Composer Done Queue에서 자기 Job 무효 검사 후 폐기', 18, INK, True)
    label(1015, 850, 835, '후보와 조건은 Join의 별도 Queue에 도착하며 ID와 revision으로 결합', 17, MUTED)

    # Common publication path, separate from the A/B execution difference.
    s.rect(54, 880, 1812, 67, PALE_GRAY, LINE)
    label(70, 891, 230, '양안 공통 최종 전달', 19, INK, True)
    label(315, 891, 1535,
          'Publication Control 현재성 및 권한 확인 → Interaction Manager 실제 표시와 재생 → ActualReceipt → Presented',
          19, INK)
    label(70, 918, 1775, '발화 감지는 즉시 출력 중단과 관련 게시 hold를 유발한다. Task나 Job 취소는 정정 의도 채택 후 별도로 판단한다.', 17, MUTED)

    # Legend and decision status remain visible at presentation size.
    queue(54, 975, 178, 46, 'Queue', small=True)
    state(260, 975, 178, 46, '상태 원통', fill='white')
    edge([(470, 998), (535, 998)], 'data')
    label(545, 982, 130, 'enqueue', 17, INK)
    edge([(690, 998), (755, 998)], 'done')
    label(765, 982, 175, '완료 callback', 17, INK)
    edge([(970, 998), (1035, 998)], 'control')
    label(1045, 982, 185, '제어와 취소', 17, INK)
    label(1270, 982, 575, 'A/B 미선정, 제품 구현과 품질 측정 없음', 17, MUTED)
    return s


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    s = slide()
    outputs = {
        OUT / f'{SLUG}.svg': s.svg(),
        OUT / f'{SLUG}.drawio': s.drawio(),
        OUT / f'{SLUG}.scene.json': json.dumps({'slug':SLUG,'title':s.caption,'items':s.items},ensure_ascii=False,indent=2)+'\n',
    }
    for path, content in outputs.items():
        if args.check:
            assert path.read_text() == content, path
        else:
            path.write_text(content)
    ET.fromstring(outputs[OUT / f'{SLUG}.svg'])
    ET.fromstring(outputs[OUT / f'{SLUG}.drawio'])
    assert all('·' not in value for value in outputs.values())
    print(f'PASS: DP44 implementation comparison source {"verified" if args.check else "written"}')


if __name__ == '__main__':
    main()
