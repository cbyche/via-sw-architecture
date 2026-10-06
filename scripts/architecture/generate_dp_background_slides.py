"""Five editable DP background slides, grounded in 04-41 through 04-45.

One scene produces both native draw.io objects and the SVG preview. These
backgrounds explain the common problem, rather than selecting an alternative.
"""
from __future__ import annotations

import argparse
import copy
import html
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
DIAGRAMS = ROOT / 'docs/architecture/12-decisions/decision-packages/diagrams'
DECK = ROOT / 'docs/presentation_files/dp-background'
W, H = 1920, 1080
INK, MUTED, TEAL = '#18232E', '#5F6B77', '#087E8B'
BLUE, RED, LINE = '#2858A5', '#B63732', '#CDD6DE'
FONT = 'Apple SD Gothic Neo'


class Slide:
    def __init__(self, number, title, premise, explanation, question, alternatives, qualities,
                 considerations_title='구조 설계에서 어려운 점'):
        self.number, self.title, self.items = number, title, []
        self.rect(0, 0, W, H, 'white', 'none')
        self.text(64, 25, 1250, ['VIA SOFTWARE ARCHITECTURE / DP 배경'], 20, TEAL, True)
        self.text(1856, 25, 280, [f'04-{number}'], 20, MUTED, align='right')
        self.text(64, 65, 1780, [f'04-{number}. {title}'], 46, INK, True)
        self.line([(64, 127), (1856, 127)], color=LINE, arrow=False, width=2)
        self.rect(64, 151, 1792, 52, '#FFF4A8', 'none')
        self.text(82, 158, 1756, [premise], 31, INK, True)
        self.text(64, 220, 1792, [explanation], 25, MUTED)
        self.line([(1213, 285), (1213, 852)], color=LINE, arrow=False)
        self.text(1260, 291, 596, [considerations_title], 29, INK, True)
        self.line([(64, 875), (1856, 875)], color=BLUE, arrow=False, width=2)
        self.text(64, 906, 165, ['설계 과제'], 28, RED, True)
        self.text(240, 901, 1616, [question], 30, INK, True)
        if alternatives:
            self.text(240, 949, 1616, [alternatives], 24, BLUE)
        self.text(64, 1018, 1260, [f'품질 쟁점: {qualities}'], 20, MUTED)
        self.text(1856, 1018, 480, ['구조 후보 / 미선정 / 미측정'], 20, MUTED, align='right')

    def add(self, kind, **kw):
        kw.update(kind=kind, id=f'n{len(self.items)+1}')
        self.items.append(kw)
        return kw['id']

    def rect(self, x, y, w, h, fill='white', stroke=LINE, dashed=False):
        return self.add('rect', x=x, y=y, w=w, h=h, fill=fill, stroke=stroke, dashed=dashed)

    def text(self, x, y, w, lines, size=25, color=INK, bold=False, align='left', leading=None):
        return self.add('text', x=x, y=y, w=w, h=len(lines)*(leading or size*1.3),
                        lines=lines, size=size, color=color, bold=bold, align=align,
                        leading=leading or size*1.3)

    def line(self, points, label=None, at=None, color=INK, arrow=True, dashed=False, width=2, size=22):
        ident = self.add('line', points=points, color=color, arrow=arrow, dashed=dashed, width=width)
        if label:
            x, y, w = at
            self.rect(x-5, y-2, w+10, size*1.3+4, 'white', 'none')
            self.text(x, y, w, [label], size, color)
        return ident

    def component(self, x, y, w, name, h=72):
        ident = self.rect(x, y, w, h, 'white', INK)
        self.text(x+w/2, y+(h-30)/2-2, w-16, [name], 26, INK, True, 'center')
        return ident

    def note(self, x, y, w, h, lines, size=25, fill='#F2F6FA', bold=False, color=INK):
        ident = self.add('note', x=x, y=y, w=w, h=h, fill=fill, stroke=LINE)
        self.text(x+16, y+13, w-40, lines, size, color, bold)
        return ident

    def challenge(self, y, heading, body):
        self.text(1260, y, 596, [heading], 28, BLUE, True)
        self.text(1260, y+47, 596, body, 25, INK, leading=35)

    def svg(self):
        esc = html.escape
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" role="img" aria-labelledby="title">',
               f'<title id="title">04-{self.number}. {esc(self.title)} — DP 배경</title>', '<defs>']
        for c in (INK, BLUE, RED, TEAL, MUTED):
            out.append(f'<marker id="a{c[1:]}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto"><path d="M1 1L9 5L1 9" fill="none" stroke="{c}" stroke-width="1.6"/></marker>')
        out.append(f'</defs><g font-family="{FONT}, Arial, sans-serif">')
        for i in self.items:
            k = i['kind']
            if k in ('rect', 'note'):
                x, y, w, h = (i[t] for t in ('x', 'y', 'w', 'h'))
                attrs = f'id="{i["id"]}" fill="{i["fill"]}" stroke="{i["stroke"]}" stroke-width="1.7"'
                if k == 'note':
                    out.append(f'<path {attrs} d="M{x} {y}H{x+w-18}L{x+w} {y+18}V{y+h}H{x}Z M{x+w-18} {y}V{y+18}H{x+w}"/>')
                else:
                    out.append(f'<rect {attrs} x="{x}" y="{y}" width="{w}" height="{h}"'+(' stroke-dasharray="7 6"' if i['dashed'] else '')+'/>')
            elif k == 'line':
                points = ' '.join(f'{x},{y}' for x, y in i['points'])
                out.append(f'<polyline id="{i["id"]}" points="{points}" fill="none" stroke="{i["color"]}" stroke-width="{i["width"]}"'+(' stroke-dasharray="7 6"' if i['dashed'] else '')+(f' marker-end="url(#a{i["color"][1:]})"' if i['arrow'] else '')+'/>')
            else:
                anchor = {'left': 'start', 'center': 'middle', 'right': 'end'}[i['align']]
                for j, line in enumerate(i['lines']):
                    out.append(f'<text id="{i["id"]}-{j}" data-width="{i["w"]}" x="{i["x"]}" y="{i["y"]+i["size"]*.92+j*i["leading"]}" text-anchor="{anchor}" font-size="{i["size"]}" font-weight="{700 if i["bold"] else 400}" fill="{i["color"]}">{esc(line)}</text>')
        return '\n'.join(out+['</g></svg>'])+'\n'

    def diagram(self):
        d = ET.Element('diagram', id=f'dp{self.number}-background', name=f'04-{self.number} {self.title}')
        model = ET.SubElement(d, 'mxGraphModel', dx=str(W), dy=str(H), grid='1', gridSize='10', page='1',
                              pageScale='1', pageWidth=str(W), pageHeight=str(H), background='#FFFFFF')
        root = ET.SubElement(model, 'root')
        ET.SubElement(root, 'mxCell', id='0')
        ET.SubElement(root, 'mxCell', id='1', parent='0')
        for i in self.items:
            kind = i['kind']
            base = f'fontFamily={FONT};'
            if kind == 'line':
                style = base+f'edgeStyle=none;rounded=0;strokeColor={i["color"]};strokeWidth={i["width"]};endArrow={"open" if i["arrow"] else "none"};endFill=0;'+('dashed=1;' if i['dashed'] else '')
                cell = ET.SubElement(root, 'mxCell', id=i['id'], parent='1', edge='1', style=style)
                g = ET.SubElement(cell, 'mxGeometry', relative='1', attrib={'as':'geometry'})
                for point, role in ((i['points'][0], 'sourcePoint'), (i['points'][-1], 'targetPoint')):
                    ET.SubElement(g, 'mxPoint', x=str(point[0]), y=str(point[1]), attrib={'as':role})
                a = ET.SubElement(g, 'Array', attrib={'as':'points'})
                for x, y in i['points'][1:-1]:
                    ET.SubElement(a, 'mxPoint', x=str(x), y=str(y))
                continue
            x, y, w, h = (i[t] for t in ('x', 'y', 'w', 'h'))
            value = ''
            if kind == 'text':
                if i['align'] == 'center': x -= w/2
                elif i['align'] == 'right': x -= w
                value = '<div style="line-height:'+str(i['leading']/i['size'])+';">'+'<br>'.join(html.escape(t) for t in i['lines'])+'</div>'
                style = base+f'text;html=1;whiteSpace=nowrap;overflow=visible;fillColor=none;strokeColor=none;align={i["align"]};verticalAlign=top;spacing=0;fontSize={i["size"]};fontColor={i["color"]};fontStyle={int(i["bold"])};'
            else:
                style = base+('shape=note;size=18;' if kind=='note' else 'rounded=0;')+f'fillColor={i["fill"]};strokeColor={i["stroke"]};strokeWidth=1.7;'+('dashed=1;' if i.get('dashed') else '')
            cell = ET.SubElement(root, 'mxCell', id=i['id'], parent='1', vertex='1', value=value, style=style)
            ET.SubElement(cell, 'mxGeometry', x=str(x), y=str(y), width=str(w), height=str(h), attrib={'as':'geometry'})
        return d

    def drawio(self):
        root = ET.Element('mxfile', host='app.diagrams.net', type='device')
        root.append(self.diagram())
        return ET.tostring(root, encoding='unicode')+'\n'


def slide41():
    s = Slide(
        41, '기능 정확성을 위한 VIA Request 해석 설계',
        '화면 정보와 이전 Task 정보를 활용한 Request 해석',
        '사용자 표현, 화면 정보, 이전 Request 기록과 Task 상태의 연결',
        '정확한 Request 해석을 위한 정보 확인과 SW 구조 설계',
        '', '기능 정확성 / 기능 지원 범위 / 반응성',
        considerations_title='설계 고려 사항')
    s.text(64, 283, 1120, ['“이 표를 아까 보고서에 넣고, 메일은 초안만 만들어.”'], 29, INK, True)

    # Editable screen example: user-visible documents and records only.
    # No Component graph, model/code owner, resolver, processing order or
    # preselected report Task. Both A and B must understand this same situation.
    s.rect(64, 349, 1110, 336, '#E6ECF2', INK)
    s.rect(78, 362, 1082, 38, '#F5F7FA', 'none')
    s.text(96, 368, 810, ['사용자 PC 화면 예시'], 22, MUTED, True)
    s.text(1142, 369, 260, ['설명용 가상 화면'], 19, MUTED, align='right')

    # Spreadsheet window and the selected table, not a VIA implementation box.
    s.rect(78, 406, 648, 264, 'white', LINE)
    s.rect(78, 406, 648, 35, '#F0F4F8', 'none')
    s.text(94, 411, 500, ['매출 현황.xlsx'], 22, INK, True)
    s.text(98, 453, 580, ['선택 범위 A1:D4'], 21, BLUE, True)
    tx, ty, cw, rh = 100, 489, 151, 38
    s.rect(tx, ty, 4*cw, 4*rh, '#F4F8FF', BLUE)
    s.rect(tx, ty, 4*cw, rh, '#DFEAFB', 'none')
    table = [['항목', '1분기', '2분기', '합계'],
             ['제품 A', '120', '140', '260'],
             ['제품 B', '80', '100', '180'],
             ['합계', '200', '240', '440']]
    for col in range(1, 4):
        s.line([(tx+col*cw, ty), (tx+col*cw, ty+4*rh)], color=LINE, arrow=False, width=1)
    for row in range(1, 4):
        s.line([(tx, ty+row*rh), (tx+4*cw, ty+row*rh)], color=LINE, arrow=False, width=1)
    for row, values in enumerate(table):
        for col, value in enumerate(values):
            s.text(tx+col*cw+cw/2, ty+row*rh+7, cw-12, [value], 22,
                   INK, row==0, align='center')
    s.rect(tx, ty, 4*cw, 4*rh, 'none', BLUE)

    # Historical user requests plus two equally valid report Task candidates.
    s.rect(738, 406, 422, 264, 'white', LINE)
    s.rect(738, 406, 422, 35, '#F0F4F8', 'none')
    s.text(754, 411, 390, ['VIA / 이전 Request와 Task 정보'], 22, INK, True)
    s.text(755, 458, 388, ['“예산 보고서 만들어줘”'], 24, INK)
    s.text(755, 494, 388, ['예산 보고서 Task / 진행 중'], 21, BLUE)
    s.line([(754, 537), (1144, 537)], color=LINE, arrow=False, width=1)
    s.text(755, 556, 388, ['“실적 보고서 만들어줘”'], 24, INK)
    s.text(755, 592, 388, ['실적 보고서 Task / 진행 중'], 21, BLUE)
    s.text(755, 634, 388, ['수정할 보고서 Task 미확정'], 21, RED, True)

    s.text(64, 713, 1120, ['Request 해석에 필요한 Context'], 26, BLUE, True)
    information = [
        (64, '화면 정보', ['선택한 표의 범위와 내용'], 'Referent 식별'),
        (452, '이전 Request와 Task 정보', ['관련 보고서 Task 후보'], 'Task Association'),
        (840, '현재 Request의 조건', ['메일 초안 작성, 발송 제외'], '조건 보존'),
    ]
    for x, heading, details, purpose in information:
        s.text(x, 762, 350, [heading], 24, INK, True)
        s.text(x, 800, 350, details, 23, INK)
        s.text(x, 835, 350, [purpose], 22, BLUE)

    s.challenge(366, '필요한 Context의 선택', [
        '화면 정보, 이전 Request 기록과', 'Task 상태에서 필요한 정보의 확인'])
    s.challenge(527, 'Referent와 Task의 정확한 연결', [
        '“이 표”에 해당하는 표의 식별,', '“아까 보고서”에 해당하는 Task의 구분'])
    s.challenge(688, '정보 부족의 보완과 조건 보존', [
        '보고서 미확정 시 Clarification,', '“초안만 작성” 조건의 유지'])
    return s


def slide42():
    s = Slide(
        42, '변경 용이성을 위한 VIA Conversation과 Task 관리 설계',
        '여러 Request의 처리 완료 이후에도 이어지는 Task 추적',
        'Conversation 안의 발화 → Request 구분 → 새 Task 생성 또는 기존 Task 연결',
        '서로 다른 수명의 기록과 상태를 관리하는 SW의 책임과 협력 설계',
        'Delegation / Downstream Agent에 작업 전달     Agent Execution / 전달한 작업의 개별 실행',
        '기능 정확성 / 변경 용이성 / 복구성',
        considerations_title='설계 고려 사항')
    xs, cw = [64, 448, 832], 342

    # The entire chronological set of inputs and handling records belongs to
    # the SAME Conversation. It is a record container, not a conversion step
    # after speech and not an A/B process boundary.
    s.rect(64, 283, 1110, 270, 'white', LINE)
    s.rect(64, 283, 1110, 44, '#EAF0F8', 'none')
    s.text(84, 292, 760, ['Conversation / 이어지는 입력과 응답 기록'], 26, BLUE, True)
    s.text(1154, 297, 310, ['User Turn / 한 번의 발화'], 19, BLUE, align='right')
    phases = ['1  처음 발화', '2  잠시 뒤 발화', '3  보고서 완료 후 발화']
    quotes = [
        ['“이번 달 매출보고서', '만들어주고, 내일 회의 안내', '메일 초안도 만들어줘.”'],
        ['“보고서는 어디까지', '됐어?”'],
        ['“결론 부분만 짧게', '바꿔줘.”'],
    ]
    for col, (x, phase, lines) in enumerate(zip(xs, phases, quotes)):
        s.text(x+16, 336, 310, [phase], 23, INK, True)
        s.text(x+16, 371, 310, lines, 24, INK, True, leading=29)
        s.text(x+16, 462, 310, [f'해석한 Request {2 if col==0 else 1}개'], 19, MUTED)
    s.line([(412, 350), (442, 350)], color=MUTED, width=2)
    s.line([(796, 350), (826, 350)], color=MUTED, width=2)
    s.rect(80, 491, 146, 32, '#E9EDF2', 'none')
    s.text(90, 496, 126, ['보고서 작성'], 21, TEAL, True)
    s.rect(238, 491, 152, 32, '#E9EDF2', 'none')
    s.text(250, 496, 128, ['메일 초안'], 21, MUTED, True)
    for x, text in [(464, '보고서 진행 확인'), (848, '보고서 결론 수정')]:
        s.rect(x, 491, 310, 32, '#E9EDF2', 'none')
        s.text(x+12, 496, 286, [text], 21, TEAL, True)
    for x, status in zip(xs, ['Agent 접수 확인 후 처리 완료',
                             '확인된 상태로 답변 완료',
                             '수정 접수 확인 후 처리 완료']):
        s.text(x+16, 531, 310, [status], 18, MUTED)

    # Two continuous goal folders share the SAME time columns above. Their
    # distinct terminal times and retained results stay visible, including mail.
    # Delegation is an action leading to an external execution, not a long-lived
    # identity parallel to Task. Labels show common behavior, not A/B authority.
    s.rect(64, 566, 168, 20, '#E4F3F2', TEAL)
    s.rect(64, 580, 1110, 133, '#F0F8F7', TEAL)
    s.rect(64, 580, 1110, 35, '#E4F3F2', 'none')
    s.text(84, 585, 1070, ['같은 보고서 Task / 계속 추적할 목표와 결과'], 24, TEAL, True)
    s.text(80, 623, 310, ['Task 생성 후 Delegation'], 21, TEAL)
    s.text(80, 654, 310, ['Agent Execution 1'], 22, INK, True)
    s.text(80, 683, 310, ['보고서 작성 시작'], 21, TEAL)
    s.text(464, 623, 310, ['작성 실행 진행 중'], 24, TEAL, True)
    s.text(464, 664, 310, ['상태 조회 / 새 실행 없음'], 21, MUTED)
    s.text(848, 623, 310, ['작성 실행 종료'], 21, MUTED)
    s.text(848, 653, 310, ['결론 수정 Delegation'], 21, TEAL)
    s.text(848, 683, 310, ['새 Agent Execution 2 진행 중'], 22, TEAL, True)

    s.rect(64, 723, 168, 20, '#F0F2F5', MUTED)
    s.rect(64, 737, 1110, 116, '#F6F7F9', MUTED)
    s.rect(64, 737, 1110, 31, '#E9EDF2', 'none')
    s.text(84, 741, 1070, ['메일 초안 Task / 별도 목표와 결과 추적'], 23, MUTED, True)
    s.text(80, 779, 310, ['Task 생성 후 Delegation'], 21, MUTED)
    s.text(80, 815, 310, ['Agent Execution / 초안 작성 시작'], 20, MUTED)
    s.text(464, 779, 310, ['초안 작성 완료'], 24, MUTED, True)
    s.text(464, 815, 310, ['결과 보관'], 21, MUTED)
    s.text(848, 793, 310, ['완료 상태와 결과 유지'], 23, MUTED)

    # Matching time columns explain evolving state; separators are visual
    # alignment only, not Components, queues, transactions or measured time.
    for x in [426, 810]:
        s.line([(x, 336), (x, 546)], color=LINE, arrow=False, dashed=True, width=1)
        s.line([(x, 621), (x, 704)], color=LINE, arrow=False, dashed=True, width=1)
        s.line([(x, 776), (x, 846)], color=LINE, arrow=False, dashed=True, width=1)

    s.challenge(363, '발화와 처리 기록의 연결', [
        '한 발화의 Request 구분과',
        'Conversation 기록의 유지'])
    s.challenge(526, '서로 다른 완료 시점', [
        'Request 처리 완료, Task 목표 완료와',
        'Agent Execution 종료의 구분'])
    s.challenge(689, '상태 관리와 연결 복구', [
        '같은 Task의 후속 Request 및 실행 연결,',
        '관리 SW 중단 후 기록과 상태 복구'])
    return s


def slide43():
    s = Slide(
        43, '기능 정확성을 위한 VIA Request 의미 판단 설계',
        '정정한 내용뿐 아니라 관련 조건과 연결까지 함께 판단',
        '메일의 결론을 표로 바꿀 때, 보고서 형식과 관련 Task도 함께 확인',
        '서로 영향을 주는 의미 판단의 책임과 협력 설계',
        '', '기능 정확성 / 기능 지원 범위 / 변경 용이성',
        considerations_title='설계 고려 사항')
    s.text(64, 282, 1110, ['상황 / 보고서 작성과 메일 초안 작성 진행 중'], 25, MUTED)

    # User-visible conditions and their correction, not Modules or processing stages.
    for x, w, fill in [(64, 540, '#EAF0F8'), (652, 522, '#FFF0EC')]:
        s.rect(x, 328, w, 140, fill, 'none')
    s.text(84, 340, 500, ['먼저 / User Turn'], 20, BLUE, True)
    s.text(84, 375, 500, ['“보고서는 PDF로, 메일에는', '그 보고서의 결론을 넣어줘.”'], 27, INK, True, leading=32)
    s.text(84, 440, 500, ['Request 해석 / PDF 조건과 메일 내용'], 19, BLUE)
    s.text(672, 340, 482, ['잠시 뒤 / 정정 User Turn'], 20, RED, True)
    s.text(672, 375, 482, ['“메일에는 결론 대신', '표만 넣어줘.”'], 27, INK, True, leading=32)
    s.text(672, 440, 482, ['Request 해석 / 메일 내용 정정'], 19, RED)
    s.line([(616, 390), (640, 390)], color=MUTED)

    s.text(64, 476, 1110, ['정정으로 바뀌는 내용과 유지할 조건'], 26, INK, True)
    # Conceptual documents show requested properties, not completed Agent facts.
    s.note(64, 525, 350, 151, ['보고서 Task의 조건'], 25, '#F0F8F7', True, TEAL)
    s.text(84, 578, 310, ['형식 / PDF 유지', '참조할 결과 / 결론과 표'], 24, INK, leading=34)
    s.line([(426, 609), (686, 609)], color=BLUE)
    s.text(441, 551, 235, ['보고서 결과 참조'], 23, BLUE, True)
    s.text(441, 637, 235, ['결과 준비 후 사용'], 21, MUTED)
    s.note(698, 525, 476, 151, ['메일 초안 Task의 조건'], 25, '#F3F6FA', True, BLUE)
    s.text(718, 580, 436, ['기존 / 보고서 결론'], 24, MUTED)
    s.line([(797, 595), (972, 595)], color=RED, arrow=False)
    s.text(718, 624, 436, ['변경 / 같은 보고서의 표만 사용'], 24, RED, True)

    s.text(64, 704, 1110, ['함께 판단할 의미 / SW 모듈이나 실행 순서가 아닌 판단 항목'], 22, MUTED)
    values = [
        (64, 744, '의도', 'PDF 보고서와 메일 초안'),
        (448, 744, 'Referent', '같은 보고서의 표'),
        (832, 744, 'Task Association', '관련 보고서 및 메일 Task'),
        (64, 807, 'Request 관계', '보고서 결과를 메일에 사용'),
        (448, 807, '정정 범위', 'PDF 유지 / 메일 내용 교체'),
        (832, 807, '처리 방향', 'Delegation 및 자료 대기'),
    ]
    for x, y, name, value in values:
        s.text(x, y, 342, [name], 23, BLUE, True)
        s.text(x, y+29, 342, [value], 22, INK)
    s.challenge(363, '현재 Request의 Task Association', [
        '전체 발화와 이전 기록을 활용한',
        '관련 Task의 판단'])
    s.challenge(526, '변경과 유지 범위의 구분', [
        '메일 내용과 보고서 참조의 변경,',
        '기존 PDF 조건의 유지'])
    s.challenge(689, '서로 관련된 판단의 일치', [
        'Referent, Task Association과 결과 사용의',
        '불일치 및 조건 누락 방지'])
    return s


def slide44():
    s = Slide(
        44, '반응성을 위한 VIA 지속 입력과 Response 전달 설계',
        '설명 도중 겹치는 새 발화와 다른 Task의 질문',
        '현재 음성 중단과 Request 처리 중에도 이어지는 메일 Task의 질문 수신',
        '후속 처리, 대기 상태와 전달 차례를 이어가는 실행 구조 설계',
        '사용자 발화 시작 시 현재 음성 중단 / Task 취소는 취소 의도 확인 후 별도 처리',
        '반응성 / 기능 정확성 / 자원 활용성',
        considerations_title='설계 고려 사항')
    xs = [64, 448, 832]
    for x, label in zip(xs, ['1  처음 발화와 처리 시작', '2  보고서 설명 도중', '3  새 발화 해석 후']):
        s.text(x+16, 286, 310, [label], 23, INK, True)
    s.line([(410, 303), (441, 303)], color=MUTED)
    s.line([(794, 303), (825, 303)], color=MUTED)

    # All panels describe occurrences and user-visible delivery. No scheduler,
    # mediator, stream graph, queue or Component graph prejudges the A/B choice.
    for x, fill in zip(xs, ['#EAF0F8', '#FFF0EC', '#EAF0F8']):
        s.rect(x, 330, 342, 153, fill, 'none')
    s.text(80, 342, 310, ['User Turn / 최초 발화'], 20, BLUE, True)
    s.text(80, 377, 310, ['“보고서 내용 설명해주고,', '내일 회의 안내 메일', '초안도 만들어줘.”'], 24, INK, True, leading=29)
    s.text(464, 342, 310, ['User Turn / 끼어들기'], 20, RED, True)
    s.text(464, 382, 310, ['“잠깐, 표부터', '설명해줘.”'], 27, INK, True, leading=34)
    s.text(848, 342, 310, ['Response / 현재 Request'], 20, BLUE, True)
    s.text(848, 382, 310, ['표 설명으로 전환'], 27, BLUE, True)
    s.text(848, 431, 310, ['Text 기록과 음성 전달'], 22, MUTED)

    s.rect(64, 503, 1110, 123, '#F3F6FA', LINE)
    s.text(84, 515, 1060, ['보고서 설명 / 계속 듣기와 현재 음성 제어'], 24, BLUE, True)
    s.text(80, 562, 310, ['Response 음성 전달 중'], 23, INK, True)
    for i,h in enumerate([8,14,23,11,18,9]):
        s.line([(89+i*22,617-h), (89+i*22,617)], color=BLUE, arrow=False, width=5)
    s.text(464, 562, 310, ['현재 음성 즉시 중단'], 23, RED, True)
    s.text(464, 597, 310, ['새 발화 수신 및 해석'], 21, INK)
    s.text(848, 562, 310, ['현재 발화에 맞춘 설명'], 23, BLUE, True)
    s.text(848, 597, 310, ['이전 설명의 전달 범위 기록'], 21, MUTED)

    s.rect(64, 650, 1110, 185, '#F0F8F7', TEAL)
    s.text(84, 664, 1060, ['별도 메일 초안 Task / Downstream Agent의 실행과 질문'], 24, TEAL, True)
    s.text(80, 709, 310, ['Delegation 후 초안 작성'], 23, TEAL, True)
    s.text(80, 750, 310, ['Agent Execution 진행 중'], 21, INK)
    s.text(80, 791, 310, ['보고서 설명과 병행'], 21, MUTED)
    s.text(464, 709, 310, ['메일 Task의 질문 도착'], 23, TEAL, True)
    s.text(464, 749, 310, ['“메일 받는 사람은', '누구인가요?”'], 23, INK, leading=30)
    s.text(848, 709, 310, ['전달 차례 결정 후 질문 전달'], 22, TEAL, True)
    s.text(848, 742, 310, ['“메일 초안의 받는 사람은', '누구인가요?”'], 22, INK, leading=28)
    s.text(848, 807, 310, ['사용자 답변까지 실행 대기'], 19, MUTED)
    for x in [426, 810]:
        s.line([(x, 550), (x, 617)], color=LINE, arrow=False, dashed=True, width=1)
        s.line([(x, 702), (x, 826)], color=LINE, arrow=False, dashed=True, width=1)
    s.challenge(363, '설명 중에도 계속 듣기', [
        '새 발화의 수신과 현재 음성 중단,',
        '해석 지연에 따른 입력 누락 방지'])
    s.challenge(526, '겹친 사건의 후속 처리', [
        '표 설명 Request와 메일 질문의',
        '진행 및 대기 상태 연결'])
    s.challenge(689, '전달 차례와 실제 전달', [
        '현재 발화에 맞는 Response 선택,',
        '질문 생성과 실제 전달의 구분'])
    return s


def slide45():
    s = Slide(
        45, '기능 정확성을 위한 VIA 기억과 Context 제공 설계',
        '여러 기록에 흩어진 과거 정정 내용과 실제 결과',
        '지난번의 표현 방식을 다시 사용하기 위한 정정, 결과 버전과 전달 기록의 연결',
        '정확한 과거 정보 제공을 위한 조회, 보관과 갱신 책임 설계',
        '이번 Request의 과거 정보 참조 / 장기 User Memory 등록은 별도 사용자 지시',
        '기능 정확성 / 반응성 / 자원 활용성',
        considerations_title='설계 고려 사항')
    s.text(64, 283, 1110, ['지난번 / 보고서를 읽은 뒤의 정정 발화'], 23, MUTED, True)
    s.rect(64, 324, 1110, 58, '#EAF0F8', 'none')
    s.text(84, 336, 1070, ['“결론을 먼저 쓰고, 문장은 짧게 바꿔줘.”'], 29, INK, True)

    # Document silhouettes and a user-visible delivery receipt show historical
    # records only. No central repository, Context Manager or chosen read path.
    s.note(64, 414, 300, 228, ['지난 보고서 / 수정 전'], 24, '#F6F7F9', True, MUTED)
    s.text(84, 475, 260, ['배경 설명'], 22, MUTED, True)
    for y in [511, 529, 547]: s.line([(84,y), (326,y)], color=LINE, arrow=False, width=4)
    s.text(84, 580, 260, ['결론 / 마지막 단락'], 22, MUTED)
    s.line([(377, 529), (429, 529)], color=BLUE)
    s.text(379, 488, 55, ['수정'], 20, BLUE, True)
    s.note(442, 414, 322, 228, ['같은 보고서 / 수정 후'], 24, '#F0F8F7', True, TEAL)
    s.text(462, 475, 280, ['결론 / 첫 단락'], 24, TEAL, True)
    for y, width in [(517, 175), (544, 150), (571, 183)]:
        s.line([(462, y), (462+width, y)], color=TEAL, arrow=False, width=4)
    s.text(462, 603, 280, ['짧은 문장으로 변경'], 21, TEAL)
    s.line([(777, 529), (814, 529)], color=BLUE)
    s.text(778, 488, 55, ['전달'], 20, BLUE, True)
    s.note(828, 414, 346, 228, ['실제 전달 기록'], 24, '#F3F6FA', True, BLUE)
    s.text(848, 475, 304, ['수정 후 보고서의', '사용자 전달 확인'], 24, INK, leading=35)
    s.text(848, 576, 304, ['어느 결과 버전을', '실제로 받았는지 확인'], 21, MUTED, leading=29)

    s.text(64, 675, 1110, ['이번 / 새 보고서의 Request'], 23, MUTED, True)
    s.rect(64, 715, 1110, 79, '#EAF0F8', 'none')
    s.text(84, 728, 1070, ['“지난번에 내가 고친 표현 방식으로 이번 보고서도 정리해줘.”'], 28, INK, True)
    s.text(64, 819, 1110, ['필요한 Context / 정정 내용 + 수정된 결과 + 실제 전달 기록'], 25, BLUE, True)

    s.challenge(363, '과거 기록 사이의 연결', [
        '정정 발화, 수정 전후 결과와',
        '실제 전달 버전의 연결'])
    s.challenge(526, '재사용할 정보의 유효성', [
        '요약의 조건 누락, 원본 수정과',
        '삭제에 따른 사용 가능 여부 확인'])
    s.challenge(689, '반복 참조와 유지 비용', [
        '필요한 과거 정보의 조회와',
        '보관, 갱신 비용의 균형'])
    return s


def build():
    slides = [slide41(), slide42(), slide43(), slide44(), slide45()]
    outputs = {}
    deck = ET.Element('mxfile', host='app.diagrams.net', type='device')
    for slide in slides:
        slug = f'dp{slide.number}-background'
        outputs[DIAGRAMS / f'{slug}.svg'] = slide.svg()
        outputs[DIAGRAMS / f'{slug}.drawio'] = slide.drawio()
        deck.append(copy.deepcopy(slide.diagram()))
    outputs[DECK / 'VIA-DP-background-41-45.drawio'] = ET.tostring(deck, encoding='unicode')+'\n'
    sections = ''.join(f'<section id="dp{s.number}"><h2>04-{s.number}. {html.escape(s.title)}</h2><img src="../../architecture/12-decisions/decision-packages/diagrams/dp{s.number}-background.svg" alt="04-{s.number} DP 배경"><p><a href="../../architecture/12-decisions/decision-packages/diagrams/dp{s.number}-background.drawio">draw.io 원본</a> / <a href="dp{s.number}-background.png">PNG</a></p></section>' for s in slides)
    outputs[DECK / 'index.html'] = '<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA DP 배경 41–45</title><style>body{margin:0;background:#ecf0f3;font-family:Arial,"Apple SD Gothic Neo",sans-serif;color:#18232e}header{padding:22px 4vw;background:white}h1{font-size:26px;margin:0 0 12px}nav{display:flex;gap:22px}a{color:#2858a5}main{max-width:1920px;margin:auto}section{padding:22px 2vw;scroll-margin-top:20px}h2{font-size:20px}img{display:block;width:100%;height:auto;background:white}p{font-size:16px}@media print{header,h2,p{display:none}section{padding:0;break-after:page}body{background:white}@page{size:16in 9in;margin:0}}</style><header><h1>VIA DP 배경 — 04-41~45</h1><nav><a href="VIA-DP-background-41-45.drawio">5페이지 draw.io</a>'+''.join(f'<a href="#dp{s.number}">04-{s.number}</a>' for s in slides)+'</nav></header><main>'+sections+'</main></html>\n'
    return outputs


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    outputs = build()
    for path, content in outputs.items():
        if args.check:
            assert path.exists() and path.read_text() == content, f'Out of sync: {path}'
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
    print(f'{len(outputs)} background slide source/preview files '+('verified' if args.check else 'written'))


if __name__ == '__main__':
    main()
