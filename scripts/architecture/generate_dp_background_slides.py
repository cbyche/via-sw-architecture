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
    def __init__(self, number, title, premise, explanation, question, alternatives, qualities):
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
        self.text(1260, 291, 596, ['구조 설계에서 어려운 점'], 29, INK, True)
        self.line([(64, 875), (1856, 875)], color=BLUE, arrow=False, width=2)
        self.text(64, 906, 165, ['설계 과제'], 28, RED, True)
        self.text(240, 901, 1616, [question], 30, INK, True)
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
        41, '요청 의미 해결 제어',
        '사용자의 말만으로는 실제 대상과 업무를 확정할 수 없다',
        'VIA는 화면·대화·업무 근거를 조회해 요청을 정리하고, 실제 업무 수행은 Downstream Agent에 위임한다.',
        '다음 조회와 최종 대상·Task 결합을 누가 결정할 것인가?',
        'A 모델 중심 ReAct / B 모델의 요청 틀을 코드가 해결',
        '정확성·기능 지원 범위와 조회/해석 왕복 시간')
    s.text(64, 288, 1120, ['“이 표를 아까 보고서에 넣고, 메일은 초안만 만들어.”'], 29, INK, True)
    s.note(64, 347, 510, 97, ['화면 근거: 사용자가 가리킨 표', '업무 근거: 보고서 Task가 두 개'], 25)
    s.text(665, 359, 475, ['대상·업무는 아직 미확정', '‘초안만’ 조건도 함께 보존'], 25, RED)
    s.component(180, 491, 370, 'Request Interpreter')
    s.component(767, 491, 370, 'Context Manager')
    s.component(767, 708, 370, 'Task Manager')
    s.line([(574, 398), (625, 398), (625, 462), (365, 462), (365, 491)], arrow=True, color=MUTED)
    s.line([(550, 511), (767, 511)], '1 어떤 근거를 읽을까?', (552, 470, 214), size=21)
    s.line([(767, 547), (550, 547)], '2 근거를 보고 재판단', (552, 573, 225), color=BLUE, size=21)
    s.line([(490, 563), (490, 734), (767, 734)], '3 기존 업무 후보 조회', (500, 692, 250), size=21)
    s.line([(767, 764), (420, 764), (420, 563)], color=BLUE)
    s.text(70, 646, 365, ['해석이 조회 범위를 정하고', '조회 결과가 해석을 바꾼다'], 26, BLUE, True)
    s.text(64, 809, 1120, ['완성 의미도 현재 입력·자료·질문·권한 검사 후 채택한다.'], 25, MUTED)
    s.text(64, 846, 1120, ['해석·조회 의존을 보여주는 도식이며, 두 안의 호출 구조는 설계 페이지에서 비교한다.'], 20, MUTED)
    s.challenge(364, '01  해석과 조회의 상호 의존', ['읽기 전에 필요한 자료를', '전부 정하기 어렵다.'])
    s.challenge(515, '02  잘못된 결합의 실제 피해', ['다른 표나 Task를 고르면', 'Agent에 다른 일을 시킨다.'])
    s.challenge(666, '03  통제와 표현 범위의 긴장', ['모델 반복에는 범위·종료 통제,', '코드 해결에는 표현 한계가 따른다.'])
    return s


def slide42():
    s = Slide(42, '대화와 업무의 서로 다른 수명',
        '대화가 바뀌거나 요청 처리가 끝나도, 업무와 대기 질문은 남아 있다',
        '같은 업무의 후속 수정은 Task를 유지하면서 새 실행으로 이어져야 한다.',
        '서로 다른 수명의 상태와 실행을 어떤 소유 경계로 관리할 것인가?',
        'A 모듈형 통합 Core와 공동 확정 / B 독립 대화·업무 서비스의 명령·접수',
        '대화·업무 연결의 정확성, 변경 범위와 교차 상태 복구')
    s.text(64, 288, 1120, ['보고서 위임 후 다른 대화로 이동하고, 돌아와 답한 뒤 수정'], 28, INK, True)
    xs = [340, 486, 640, 795, 950, 1100]
    events = [('위임 접수', 'R1 종료'), ('질문 제시', 'Q1 대기'), ('다른 대화', 'C2 이동'), ('복귀·답변', 'Q1 해소'), ('결과 도착', 'X1 종료'), ('후속 수정', 'X2 시작')]
    for x, (a, b) in zip(xs, events):
        s.text(x, 354, 150, [a, b], 22, MUTED, align='center')
        s.line([(x, 420), (x, 768)], color=LINE, arrow=False, dashed=True, width=1)
    rows = [('Conversation C1', 453, 340, 1135, INK), ('Request R1', 522, 315, 358, BLUE),
            ('Question Q1', 591, 486, 795, TEAL), ('Task T1', 660, 340, 1135, BLUE),
            ('Execution X1 / X2', 729, 340, 950, MUTED)]
    for name, y, start, end, color in rows:
        s.text(64, y-15, 266, [name], 23, color, True)
        ongoing = name in ('Conversation C1', 'Task T1')
        s.line([(start, y), (end, y)], color=color, arrow=ongoing, width=5)
        if not ongoing:
            s.rect(end-4, y-8, 8, 16, color, 'none')
    s.line([(1100, 729), (1160, 729)], color=TEAL, arrow=True, width=5)
    s.text(64, 794, 1120, ['화살표는 계속 유지, 끝 막대는 처리 종료를 뜻한다. 종료 후에도 기록은 남는다.'], 23, MUTED)
    s.text(64, 829, 1120, ['선의 길이는 측정 시간이 아니다.'], 22, MUTED)
    s.challenge(364, '01  유지할 관계가 서로 다름', ['위임 요청은 끝나도', 'Task와 질문의 연결은 유지한다.'])
    s.challenge(515, '02  답변·정정의 연결과 확정', ['“응”을 실제 제시 질문에 연결하고,', '관련 대화·업무 변경을 맞춘다.'])
    s.challenge(666, '03  논리 수명과 실행 경계', ['양안 모두 다른 논리 수명을 지원.', '독립 소유는 별도의 구조 선택이다.'])
    return s


def slide43():
    s = Slide(43, '요청 의미의 생산 책임',
        '한 번의 정정이 대상·Task 연결·요청 관계를 함께 바꿀 수 있다',
        'VIA는 의도부터 처리 방향까지 판단하고, 기존 조건을 유지하면서 바뀐 부분을 반영해야 한다.',
        '서로 얽힌 의미를 통합 생산할 것인가, 기능별 생산자가 협력할 것인가?',
        'A 통합 의미 생산 / C 여섯 기능의 의미 생산과 코드 조정',
        '의미 정확성·누락 방지와 부분 판단의 조정·추론 비용')
    s.text(64, 288, 1120, ['“보고서는 PDF로, 메일에는 그 보고서의 결론을 넣어줘.”'], 28, INK, True)
    s.text(64, 333, 1120, ['정정  “메일에는 결론 대신 표만 넣어줘.”'], 28, RED, True)
    s.rect(64, 402, 1110, 363, 'white', LINE, dashed=True)
    s.text(88, 416, 1060, ['Request Interpreter'], 26, INK, True)
    values = [(['F1 요청 의도', '목표와 PDF 조건'], 88, 476),
              (['F2 지칭 대상', '보고서의 결론 / 표'], 452, 476),
              (['F3 대화·Task 연결', '보고서 / 메일 후보'], 816, 476),
              (['F4 요청 간 관계', '어느 결과를 사용할까'], 88, 628),
              (['F5 정정 범위', '교체할 내용과 유지 조건'], 452, 628),
              (['F6 처리 방향', '직접 응답 / 위임 / 확인'], 816, 628)]
    for lines, x, y in values:
        s.note(x, y, 330, 106, lines, 24)
    s.line([(418, 529), (452, 529)], color=BLUE, arrow=False, dashed=True)
    s.line([(782, 529), (816, 529)], color=BLUE, arrow=False, dashed=True)
    s.line([(253, 582), (253, 628)], color=BLUE, arrow=False, dashed=True)
    s.line([(617, 582), (617, 628)], color=BLUE, arrow=False, dashed=True)
    s.line([(981, 582), (981, 628)], color=BLUE, arrow=False, dashed=True)
    s.text(64, 794, 1120, ['메일 내용·결과 의존은 변경 / 보고서의 PDF 조건은 유지'], 26, BLUE, True)
    s.text(64, 834, 1120, ['F1~F6은 공통 판단 기능이며, 고정 실행 순서나 모델 호출 수가 아니다.'], 22, MUTED)
    s.challenge(364, '01  Task 연결도 판단 결과', ['어느 업무인지 미리 정해 두면', '새 요청·직접 대화를 놓친다.'])
    s.challenge(515, '02  부분 의미의 상호 의존', ['대상이 바뀌면 관계와 정정 범위도', '다시 판단해야 할 수 있다.'])
    s.challenge(666, '03  집중 판단과 전체 일관성', ['기능별 집중은 개선의 후보지만,', '불일치·왕복·맥락 중복도 생긴다.'])
    return s


def slide44():
    s = Slide(44, '계속 듣고 응답하는 대화의 실행 구조',
        'VIA가 생각하거나 말하는 동안에도 새 발화와 업무 사건이 도착한다',
        '입력은 계속 받고, 현재 정정과 기존 업무의 질문·결과를 같은 대화에서 올바르게 이어가야 한다.',
        '겹쳐 도착하는 사건의 후속 실행과 중간 상태를 어떻게 조직할 것인가?',
        'A 중앙 비동기 Orchestration/Mediator / B 반응형 Dataflow/Pipes-and-Filters',
        '반응성·전달 정확성과 대기열·취소·자원 경합')
    s.text(64, 288, 1120, ['보고서 설명을 듣다가  “잠깐, 표부터 설명해줘.”'], 28, INK, True)
    s.note(64, 355, 535, 102, ['사용자: 새 발화·정정 u2', '현재 음성 P1을 즉시 중단'], 25)
    s.note(650, 355, 524, 102, ['별도 업무: 질문 Q1 / 결과 r1 도착', '현재 발화와 다른 시점에 발생'], 25)
    s.component(64, 539, 440, 'Interaction Manager')
    s.component(734, 539, 440, 'Task Manager')
    s.component(64, 732, 440, 'Request Controller')
    s.component(734, 732, 440, 'Response Manager')
    s.line([(284, 457), (284, 539)], '1 지속 수신·인식', (303, 476, 275), size=22)
    s.line([(954, 457), (954, 539)], '2 확인된 업무 사건', (677, 476, 270), size=22)
    s.line([(284, 611), (284, 732)], '3 현재 입력 해소', (303, 652, 275), size=22)
    s.line([(954, 611), (954, 732)], '4 질문·결과 전달 준비', (677, 652, 290), size=22)
    s.line([(504, 768), (734, 768)], '현재 의미·허용', (506, 811, 226), color=BLUE, size=22)
    s.line([(1174, 752), (1192, 752), (1192, 513), (470, 513), (470, 539)], color=BLUE, dashed=True)
    s.text(619, 581, 226, ['게시 허용 후 실제 출력', '전달 결과는 다시 기록'], 22, BLUE, align='center')
    s.text(64, 839, 1120, ['음성 중단은 업무 취소와 다르다. 생성 완료는 실제 전달과 다르다.'], 23, MUTED)
    s.challenge(364, '01  입력 보호와 처리 병행', ['긴 해석 중에도 입력과 중단을 받음.', '비동기는 두 안의 공통 조건이다.'])
    s.challenge(515, '02  정정 전 효과와 전달 제어', ['미전송 효과·오래된 응답을 보류하고,', '실제로 들은 범위에 맞게 이어간다.'])
    s.challenge(666, '03  분기·대기·합류의 조직', ['중앙의 명령·반환 관리와', '소비자 활성화·국소 결합을 비교한다.'])
    return s


def slide45():
    s = Slide(45, '기억과 Context의 공급 구조',
        '과거에 무엇을 고쳤는지는 대화·결과 버전·실제 제시 기록에 흩어져 있다',
        '“지난번에 내가 고친 표현 방식으로 이번 보고서도 정리해줘.”에는 기록 사이의 연결이 필요하다.',
        '과거 관계를 요청마다 조합할 것인가, 공통 파생 기억으로 유지할 것인가?',
        'A 원본 서비스 조합 / B 공통 파생 기억 저장소',
        '과거 근거의 정확성·지원 범위, 반복 조회와 생산·저장·삭제 비용')
    s.text(64, 288, 1120, ['같은 과거 사건을, 새 보고서 요청의 근거로 다시 사용'], 28, INK, True)
    records = [(64, ['정정 R7', '“짧은 문장, 결론 먼저”'], 'Request Controller'),
               (444, ['결과 D2 / D3', '수정 전후의 결과 버전'], 'Task Manager'),
               (824, ['실제 제시 P4', '수정 결과 D3의 전달 기록'], 'Response Manager')]
    for x, lines, owner in records:
        s.note(x, 362, 350, 107, lines, 24)
        s.component(x, 535, 350, owner)
        s.line([(x+175, 469), (x+175, 535)], arrow=False, color=MUTED)
    s.component(444, 719, 350, 'Context Manager')
    s.line([(239, 607), (239, 669), (570, 669), (570, 719)], color=INK)
    s.line([(619, 607), (619, 719)], color=INK)
    s.line([(999, 607), (999, 669), (670, 669), (670, 719)], color=INK)
    s.text(64, 684, 400, ['각 소유자의 허용된 읽기 계약'], 23, MUTED)
    s.text(64, 757, 352, ['출처·버전·누락 범위를', '함께 확인해 공급'], 25, BLUE, True)
    s.text(824, 757, 350, ['이번 대상·Task는', '현재 요청에서 판단'], 25, BLUE, True)
    s.text(64, 832, 1120, ['과거 정정은 자동으로 장기 선호가 되지 않는다. 원본·파생 기억·현재 판단을 구별한다.'], 23, MUTED)
    s.challenge(364, '01  검색 일치와 관계의 차이', ['‘보고서’라는 단어만 찾아서는', '정정의 전후와 실제 제시를 모른다.'])
    s.challenge(515, '02  반복 조합과 공통 생산 비용', ['요청별 결합 부담과 파생 기억의', '생산·유지·미게시 범위를 비교한다.'])
    s.challenge(666, '03  압축·오류·삭제의 파급', ['요약이 조건을 잃거나 원본이 바뀌면', '재사용 근거도 다시 확인해야 한다.'])
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
