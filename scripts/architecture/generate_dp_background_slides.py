"""An overview and five editable DP backgrounds, grounded in 04-41 through 04-45.

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
DECK = ROOT / 'docs/presentations_files/dp-background'
W, H = 1920, 1080
INK, MUTED, TEAL = '#18232E', '#5F6B77', '#087E8B'
BLUE, RED, LINE = '#2858A5', '#B63732', '#CDD6DE'
FONT = 'Apple SD Gothic Neo'


class Slide:
    def __init__(self, number, title, premise, explanation, question, alternatives, qualities,
                 considerations_title='구조 설계에서 어려운 점'):
        self.number, self.title, self.items = number, title, []
        overview = number == 'overview'
        self.slug = 'dp-background-overview' if overview else f'dp{number}-background'
        self.caption = title if overview else f'04-{number}. {title}'
        self.rect(0, 0, W, H, 'white', 'none')
        self.text(64, 25, 1250, ['VIA SOFTWARE ARCHITECTURE / DP 배경'], 20, TEAL, True)
        self.text(1856, 25, 280, ['설계 문제 지도' if overview else f'04-{number}'], 20, MUTED, align='right')
        self.text(64, 65, 1780, [self.caption], 40, INK, True)
        self.line([(64, 127), (1856, 127)], color=LINE, arrow=False, width=2)
        self.rect(64, 151, 1792, 64, '#FFF4A8', 'none')
        self.text(82, 165, 1756, [premise], 36, INK, True)
        if explanation:
            self.text(64, 236, 1792, [explanation], 24, MUTED)
        if not overview:
            self.line([(1213, 282), (1213, 859)], color=LINE, arrow=False)
            self.text(1260, 283, 596, [considerations_title], 26, MUTED, True)
        self.line([(64, 888), (1856, 888)], color=LINE, arrow=False, width=2)
        self.text(64, 925, 165, ['설계 과제'], 26, RED, True)
        self.text(240, 920, 1616, [question], 31, INK, True)
        if alternatives:
            self.text(240, 974, 1616, [alternatives], 21, MUTED)
        self.text(64, 1035, 1260, [f'품질 관점: {qualities}'], 18, MUTED)
        self.text(1856, 1035, 480, ['구조 후보 / 미선정 / 미측정'], 18, MUTED, align='right')

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
        if lines:
            self.text(x+16, y+13, w-40, lines, size, color, bold)
        return ident

    def challenge(self, y, heading, body):
        self.text(1260, y, 596, [heading], 24, INK, True)
        self.text(1260, y+42, 596, body, 23, MUTED, leading=33)

    def svg(self):
        esc = html.escape
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" role="img" aria-labelledby="title">',
               f'<title id="title">{esc(self.caption)} — DP 배경</title>', '<defs>']
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
        d = ET.Element('diagram', id=self.slug, name=self.caption)
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


PRESENTATION_ORDER = (41, 44, 45, 42)
OVERVIEW_SCRIPT = """사용자는 화면의 표를 가리키며 제안서에 넣고, 공유 메일은 초안만 만들어 달라고 합니다. VIA는 표가 무엇인지, 두 업무를 어떻게 나눌지, 초안이라는 조건을 이해해야 합니다.
그런데 처리가 끝나기 전에 가격을 빼 달라는 정정이 들어오고, 메일 업무에서는 수신자를 묻습니다. VIA는 각각의 업무를 연결하고, 늦게 끝난 결과도 현재 조건에서 유효한지 확인해야 합니다.
이어서 사용자는 VIA가 전에 설명한 기준과 제품 비교, 견적 결과를 함께 참조합니다. VIA는 실제 전달한 설명과 허용된 결과 참조를 연결하고, 분석과 제안서 작성은 업무 수행 Agent에 맡깁니다.
위임 요청 처리가 끝나도 대화와 업무는 이어집니다. 음성 대화가 끝난 뒤 사용자가 돌아오면 같은 업무의 진행과 결과를 연결해야 합니다.
먼저, 이처럼 대상과 조건이 섞인 요청을 VIA가 어떤 방식으로 이해하고 완성할지부터 설명하겠습니다."""
TRANSITIONS = {
    41: '요청을 이해하는 중에도 새 입력과 업무 알림은 계속 들어옵니다. 다음으로, 겹친 사건과 여러 처리를 이어가는 실행 구조를 설명하겠습니다.',
    44: '들어온 사건을 처리하는 데 더해, 현재 요청이 가리키는 과거 설명과 업무 결과를 연결해야 합니다. 다음으로, 여러 활동의 과거 맥락을 제공하는 방식을 설명하겠습니다.',
    45: '이 연결을 유지하려면 대화·요청·업무·실행의 상태를 각자의 수명에 맞게 관리해야 합니다. 마지막으로, 서로 다른 수명과 상태 관리 책임을 설명하겠습니다.',
}


def overview():
    # User experience scene, deliberately independent of Architecture symbols.
    s = Slide.__new__(Slide)
    s.number, s.slug, s.items = 'overview', 'dp-background-overview', []
    s.title = s.caption = '사용자가 계속 말하는 동안에도, VIA는 대화와 업무를 이어가야 합니다'
    s.notes = OVERVIEW_SCRIPT
    s.rect(0, 0, W, H, 'white', 'none')
    s.text(64, 28, 1792, ['VIA 사용자 경험'], 20, MUTED, True)
    s.text(64, 68, 1792, [s.title], 40, INK, True)
    s.text(64, 126, 1792, ['한 번의 이해로 끝나지 않습니다. 새 입력, 업무 알림, 과거 정보와 서로 다른 종료 시점이 겹칩니다.'], 26, MUTED)
    s.line([(64, 169), (1856, 169)], color=LINE, arrow=False)

    s.text(64, 190, 160, ['이전에'], 24, MUTED, True)
    for x, title, detail in [
        (232, 'VIA의 직접 설명', '평가 기준 · 사용자에게 실제 전달'),
        (770, '제품 조사 Agent의 확인된 결과', '제품 비교 · 결과 참조 / 필요한 발췌'),
        (1340, '견적 Agent의 확인된 결과', '견적 · 결과 참조 / 필요한 발췌'),
    ]:
        s.text(x, 188, 500, [title], 24, INK, True)
        s.text(x, 224, 500, [detail], 22, MUTED)
    # References join the later context, not a fabricated single time sequence.
    for x in (435, 985, 1540):
        s.line([(x, 257), (x, 275), (1355, 275)], color=MUTED, arrow=False, dashed=True, width=1.7)
    s.line([(1355, 275), (1490, 275), (1490, 672), (1320, 672)], color=MUTED, dashed=True, width=1.7)

    s.text(64, 295, 130, ['시간 →'], 22, MUTED, True)
    for x,w,t in [(225,390,'현재 요청'),(670,355,'처리 중 겹치는 사건'),(1080,410,'이후 과거 정보 재사용'),(1540,160,'음성 종료'),(1730,120,'돌아옴')]:
        s.text(x, 295, w, [t], 22, MUTED, True)
    s.line([(220, 330), (1850, 330)], color=LINE, width=1.7)
    s.text(64, 359, 140, ['사용자', '입력·대화'], 25, INK, True)
    s.text(225, 358, 400, ['“이 표를 제안서에 넣고,', '공유 메일은 초안만', '만들어줘.”'], 27, INK, True, leading=35)
    s.text(225, 470, 400, ['화면의 표를 가리킴'], 22, MUTED)
    s.text(670, 358, 390, ['“잠깐, 가격은', '비교에서 빼줘.”'], 27, INK, True, leading=35)
    s.text(735, 468, 330, ['메일 업무의 질문', '“수신자는 누구인가요?”'], 24, INK, leading=31)
    s.text(1080, 358, 410, ['“지난번 네가 설명한 기준으로,', '제품 비교 결과와 견적도', '반영해줘.”'], 25, INK, True, leading=34)
    s.text(1530, 362, 150, ['대화는', '끝나도…'], 24, MUTED, leading=32)
    s.text(1710, 390, 170, ['같은 업무의', '진행·결과'], 24, INK, leading=32)
    s.line([(1540, 456), (1540, 890)], color=LINE, arrow=False, dashed=True, width=1.7)

    s.text(64, 590, 140, ['VIA', '처리·연결'], 25, INK, True)
    s.line([(225, 580), (1850, 580)], color=LINE, arrow=False, width=7)
    s.text(225, 612, 395, ['표의 대상 확인 · 업무 구분', '메일은 초안만 조건 유지'], 25, INK, leading=34)
    s.text(670, 612, 385, ['정정의 적용 업무 확인', '질문을 메일 업무에 연결'], 24, INK, leading=33)
    s.text(1080, 612, 405, ['과거 설명·결과 참조 연결', '현재 제안서 업무에 제공'], 24, INK, leading=33)
    s.text(1640, 612, 230, ['업무의 진행·결과를', '사용자에게 전달'], 23, INK, leading=32)
    s.line([(280, 504), (280, 570)], width=1.7)
    s.line([(695, 438), (695, 570)], width=1.7)
    s.line([(855, 570), (855, 536)], width=1.7)
    s.line([(1110, 471), (1110, 570)], width=1.7)
    s.line([(1760, 570), (1760, 476)], width=1.7)
    s.text(900, 693, 440, ['늦게 끝난 이전 조건의 후보 → 현재 유효성 확인'], 22, MUTED)

    s.text(64, 785, 140, ['외부 Agent', '업무 수행'], 25, INK, True)
    s.text(225, 756, 325, ['제안서 업무'], 23, INK, True)
    s.line([(555, 790), (1850, 790)], color=LINE, arrow=False, width=7)
    s.text(1110, 752, 410, ['기준 적용·분석·제안서 작성'], 23, MUTED)
    s.line([(575, 680), (575, 780)], label='위임', at=(510, 717, 55), size=22, width=1.7)
    s.line([(650, 780), (650, 682)], label='접수 확인', at=(604, 717, 102), size=20, width=1.7)
    s.text(225, 698, 300, ['위임 요청의 처리 완료', '(외부 접수 확인 후)'], 21, MUTED, leading=28)
    s.line([(1320, 688), (1320, 740)], width=1.7)
    s.line([(1760, 780), (1760, 690)], width=1.7)
    s.text(1550, 820, 325, ['대화가 끝나도 업무는 지속'], 22, MUTED)
    s.text(225, 859, 320, ['공유 메일 초안 업무'], 23, INK, True)
    s.line([(555, 900), (1450, 900)], color=LINE, arrow=False, width=7)
    s.line([(555, 684), (555, 890)], width=1.7)
    s.line([(725, 890), (725, 682)], width=1.7)
    s.text(610, 813, 120, ['메일 질문'], 21, MUTED)
    s.text(755, 850, 690, ['수신자 질문은 VIA로 → 답변 연결 후 초안 준비 (발송 없음)'], 22, MUTED)
    s.line([(985, 780), (985, 725)], width=1.7)

    s.line([(64, 930), (1856, 930)], color=LINE, arrow=False)
    s.line([(64, 951), (117, 951)], width=1.7)
    s.text(133, 936, 330, ['입력·위임·알림·전달'], 20, MUTED)
    s.line([(485, 951), (538, 951)], color=MUTED, dashed=True, width=1.7)
    s.text(554, 936, 200, ['과거 정보 참조'], 20, MUTED)
    s.line([(780, 951), (833, 951)], color=LINE, arrow=False, width=7)
    s.text(849, 936, 960, ['계속되는 처리·업무   /   위치·길이는 설명용이며 측정 시간이 아님'], 20, MUTED)
    for x, lines in [(64,['① 지금 요청의 대상과', '조건 이해']), (516,['② 새 입력·업무 알림이', '겹쳐도 처리 이어가기']), (968,['③ 과거 설명과 여러', '업무 결과 연결']), (1420,['④ 대화와 업무의', '서로 다른 수명 관리'])]:
        s.text(x, 972, 425, lines, 23, INK, True, leading=30)
    s.text(64, 1041, 1792, ['VIA: 사용자 상호작용·요청 이해·업무 연결과 위임·결과 전달   /   업무 수행 Agent: 도메인 분석·계획·도구 선택·업무 실행'], 20, MUTED)
    return s


def slide41():
    s = Slide(41, '기능 정확성을 위한 VIA Request 해석 설계',
        '선택한 표와 이전 보고서의 오연결 위험', '',
        '정보 조회와 의미 완성의 진행 책임 설계', '',
        '기능 정확성 / 기능 지원 범위 / 반응성', considerations_title='설계 고려 사항')
    s.text(64, 265, 1110, ['“이 표를 아까 보고서에 넣고, 메일은 초안만 만들어.”'], 29, INK, True)
    s.rect(64, 330, 1110, 411, '#E6ECF2', LINE)
    s.text(86, 346, 1034, ['사용자 화면과 VIA 기록 / 설명용 가상 화면'], 21, MUTED)
    s.rect(80, 389, 568, 333, 'white', LINE)
    s.text(100, 406, 530, ['매출 현황.xlsx'], 25, INK, True)
    s.text(100, 452, 530, ['선택 범위 A1:D4'], 23, BLUE, True)
    tx,ty,cw,rh=100,505,132,43
    s.rect(tx,ty,4*cw,4*rh,'#F3F7FC',BLUE)
    s.rect(tx,ty,4*cw,rh,'#E4EDF8','none')
    table=[['항목','1분기','2분기','합계'],['제품 A','120','140','260'],['제품 B','80','100','180'],['합계','200','240','440']]
    for c in range(1,4): s.line([(tx+c*cw,ty),(tx+c*cw,ty+4*rh)],color=LINE,arrow=False,width=1)
    for r in range(1,4): s.line([(tx,ty+r*rh),(tx+4*cw,ty+r*rh)],color=LINE,arrow=False,width=1)
    for r,values in enumerate(table):
        for c,value in enumerate(values):
            s.text(tx+c*cw+cw/2,ty+r*rh+11,cw-12,[value],20,INK,r==0,align='center')
    s.rect(670,389,488,333,'white',LINE)
    s.text(690,406,448,['이전 Request와 Task 후보'],23,MUTED,True)
    s.text(690,455,448,['예산 보고서 Task'],26,INK,True)
    s.text(690,495,448,['“예산 보고서 만들어줘” / 진행 중'],20,MUTED)
    s.line([(690,536),(1138,536)],color=LINE,arrow=False,width=1)
    s.text(690,557,448,['실적 보고서 Task'],26,INK,True)
    s.text(690,597,448,['“실적 보고서 만들어줘” / 진행 중'],20,MUTED)
    s.text(690,665,448,['연결할 Task 미확정'],29,RED,True)
    s.rect(64,778,1110,67,'#F1F4F7','none')
    s.text(84,795,1068,['메일 Request의 고정 조건 / 초안 작성, 발송 제외'],27,INK,True)
    s.challenge(355,'선택 자료의 확인',['화면 정보와 Referent 범위 확인'])
    s.challenge(515,'관련 Task의 구분',['이전 Request와 현재 Task 상태 확인'])
    s.challenge(675,'정보 보완과 조건 유지',['미확정 Task의 Clarification과','초안 조건의 보존'])
    return s


def slide42():
    s = Slide(42, '변경 용이성을 위한 VIA Conversation과 Task 관리 설계',
        'Request 완료와 Task 완료의 혼동 위험', '',
        'Conversation과 Task 상태의 관리 책임 및 실행 경계 설계',
        'Delegation / Downstream Agent에 작업 전달     Agent Execution / 개별 작업 실행',
        '기능 정확성 / 변경 용이성 / 복구성', considerations_title='설계 고려 사항')
    xs=[64,448,832]
    s.rect(64,275,1110,273,'white',LINE)
    s.rect(64,275,1110,40,'#EAF0F8','none')
    s.text(84,283,760,['Conversation / 입력과 Response 기록'],25,INK,True)
    s.text(1154,288,290,['User Turn / 한 번의 발화'],18,MUTED,align='right')
    quotes=[['“이번 달 매출보고서','만들어주고, 내일 회의 안내','메일 초안도 만들어줘.”'],['“보고서는 어디까지','됐어?”'],['“결론 부분만 짧게','바꿔줘.”']]
    for i,(x,phase,lines) in enumerate(zip(xs,['처음 발화','잠시 뒤 / 진행 확인','보고서 완료 후 / 수정'],quotes)):
        s.text(x+16,326,310,[phase],22,MUTED,True)
        s.text(x+16,363,310,lines,24,INK,True,leading=28)
        s.text(x+16,451,310,[f'Request {2 if i==0 else 1}개'],18,MUTED)
    s.line([(411,337),(440,337)],color=MUTED)
    s.line([(795,337),(824,337)],color=MUTED)
    for x,w,t in [(80,150,'보고서 작성'),(242,148,'메일 초안'),(464,310,'보고서 진행 확인'),(848,310,'보고서 결론 수정')]:
        s.rect(x,477,w,32,'#EFF2F5','none');s.text(x+10,483,w-20,[t],21,INK)
    for x,lines in zip(xs,[['Downstream Agent 접수 확인 후','Request 처리 완료'],['상태 답변 후','Request 처리 완료'],['Downstream Agent 수정 접수 후','Request 처리 완료']]):
        s.text(x+16,511,310,lines,17,MUTED,leading=22)
    # The middle column makes the two completion times visible, without a
    # service/process boundary or a measured time axis.
    s.rect(64,575,1110,147,'#F0F8F7',TEAL)
    s.rect(448,619,342,95,'#DCEFEA','none')
    s.text(84,583,410,['보고서 Task'],26,TEAL,True)
    s.text(1154,588,620,['Task / 목표, 상태와 결과의 추적'],20,MUTED,align='right')
    s.text(80,627,310,['Task 생성 후 Delegation'],21,INK)
    s.text(80,664,310,['Agent Execution 1'],23,INK,True)
    s.text(80,698,310,['보고서 작성 시작'],20,MUTED)
    s.text(464,634,310,['Task와 실행 계속'],29,TEAL,True)
    s.text(464,681,310,['상태 조회 / 새 실행 없음'],20,INK)
    s.text(848,627,310,['작성 종료 후 수정 Delegation'],21,INK)
    s.text(848,664,310,['Agent Execution 2 시작'],22,TEAL,True)
    s.text(848,698,310,['같은 Task의 결론 수정'],20,MUTED)
    s.rect(64,746,1110,114,'#F5F6F8',LINE)
    s.text(84,754,1068,['메일 초안 Task'],23,MUTED,True)
    s.text(80,796,310,['Delegation / 초안 작성 시작'],21,INK)
    s.text(80,829,310,['Agent Execution'],19,MUTED)
    s.text(464,808,310,['초안 완료 / 결과 보관'],23,INK)
    s.text(848,808,310,['완료 상태와 결과 유지'],23,INK)
    for x in [426,810]:
        for a,b in [(323,539),(623,712),(790,850)]: s.line([(x,a),(x,b)],color=LINE,arrow=False,dashed=True,width=1)
    s.challenge(355,'서로 다른 완료 시점',['외부 접수와 답변 완료, Task 목표 완료의 구분'])
    s.challenge(515,'후속 Request와 실행 연결',['같은 Task의 상태 조회와 새 수정 실행 연결'])
    s.challenge(675,'관리 책임과 복구 범위',['기록과 상태의 관리 경계 및 중단 후 복구'])
    return s


def slide43():
    s = Slide(43, '기능 정확성을 위한 VIA Request 의미 판단 설계',
        '메일 정정에 따른 관련 참조 오류와 기존 조건 누락 위험', '',
        '연관된 의미 판단의 생산 책임과 조정 방식 설계', '',
        '기능 정확성 / 기능 지원 범위 / 변경 용이성', considerations_title='설계 고려 사항')
    for x,w,fill in [(64,540,'#EFF3F8'),(652,522,'#FFF0EC')]: s.rect(x,275,w,147,fill,'none')
    s.text(84,287,500,['진행 중인 두 Task의 조건 지정'],20,MUTED)
    s.text(84,324,500,['“보고서는 PDF로, 메일에는','그 보고서의 결론을 넣어줘.”'],27,INK,True,leading=32)
    s.text(84,391,500,['Request / PDF 조건과 메일 내용'],19,MUTED)
    s.text(672,287,482,['잠시 뒤 / 메일 내용 정정'],20,RED)
    s.text(672,324,482,['“메일에는 결론 대신','표만 넣어줘.”'],27,INK,True,leading=32)
    s.text(672,391,482,['Request / 메일 내용 교체'],19,MUTED)
    s.line([(616,346),(640,346)],color=MUTED)
    s.note(64,476,300,204,['보고서 Task'],26,'#F3F7F7',True,INK)
    s.text(84,537,260,['PDF 조건 유지'],29,TEAL,True)
    s.text(84,604,260,['결과 / 결론과 표'],22,MUTED)
    s.note(752,476,422,204,['메일 초안 Task'],26,'#F3F6FA',True,INK)
    s.text(772,537,382,['기존 / 보고서 결론'],24,MUTED)
    s.line([(853,553),(1028,553)],color=RED,arrow=False)
    s.text(772,605,382,['같은 보고서의 표만 사용'],28,TEAL,True)
    # A semantic data relationship, not an invocation path or Module graph.
    s.line([(380,578),(730,578)],color=TEAL,width=3)
    s.text(399,495,322,['메일의 보고서 참조 변경'],24,RED,True)
    s.text(399,626,322,['보고서 결과 준비 후 사용'],21,MUTED)
    s.text(64,720,1110,['정정 시 함께 맞춰야 할 판단'],23,MUTED)
    for x,name,value in [(80,'Referent','같은 보고서의 표'),(464,'Task Association','메일 초안 Task'),(848,'정정 범위','메일 내용만 교체')]:
        s.text(x,763,310,[name],21,MUTED)
        s.text(x,804,310,[value],26,INK,True)
    s.line([(390,819),(434,819)],color=LINE,arrow=False,width=2)
    s.line([(772,819),(818,819)],color=LINE,arrow=False,width=2)
    s.challenge(355,'의도와 처리 방향',['요청 목표와 Delegation 및 자료 대기의 판단'])
    s.challenge(515,'Referent와 Task Association',['자료와 관련 Task 및 결과 사용 관계의 일치'])
    s.challenge(675,'정정 범위와 기존 조건',['메일 내용만 교체하고 PDF 조건 유지'])
    return s


def slide44():
    s = Slide(44, '반응성을 위한 VIA 지속 입력과 Response 전달 설계',
        '새 발화 누락과 현재 대화에 맞지 않는 Response 전달 위험', '',
        '겹친 사건의 후속 실행과 전달을 이어가는 실행 구조 설계', '',
        '반응성 / 기능 정확성 / 자원 활용성', considerations_title='설계 고려 사항')
    s.text(64,258,1110,['처음 / “보고서 내용 설명해주고, 내일 회의 안내 메일 초안도 만들어줘.”'],24,INK)
    xs=[64,448,832]
    for x,label,c in zip(xs,['처리 시작','설명 도중 / 두 사건 도착','발화 해석 후'],[MUTED,RED,MUTED]):
        s.text(x+16,310,310,[label],23,c,True)
    s.line([(411,326),(440,326)],color=MUTED)
    s.line([(795,326),(824,326)],color=MUTED)
    for x,fill in zip(xs,['#F2F5F9','#FFF0EC','#F2F5F9']): s.rect(x,356,342,189,fill,'none')
    s.text(80,370,310,['Response'],20,MUTED)
    s.text(80,415,310,['보고서 설명 중'],29,INK,True)
    s.text(80,485,310,['사용자 발화 계속 수신'],21,MUTED)
    for i,h in enumerate([8,14,23,11,18,9]): s.line([(89+i*22,532-h),(89+i*22,532)],color=BLUE,arrow=False,width=5)
    s.text(464,370,310,['새 User Turn'],20,RED)
    s.text(464,408,310,['“잠깐, 표부터','설명해줘.”'],28,INK,True,leading=34)
    s.text(464,506,310,['현재 음성 중단과 발화 해석'],20,MUTED)
    s.text(848,370,310,['현재 Request의 Response'],20,MUTED)
    s.text(848,415,310,['표 설명으로 전환'],28,TEAL,True)
    s.text(848,485,310,['Text 및 음성 전달'],21,MUTED)
    s.text(64,591,1110,['메일 초안 Task / Downstream Agent의 작업 실행'],24,INK,True)
    for x,fill in zip(xs,['#F3F7F7','#FFF0EC','#F3F7F7']): s.rect(x,638,342,164,fill,'none')
    s.text(80,656,310,['Delegation 후 초안 작성'],23,INK,True)
    s.text(80,705,310,['Agent Execution 진행 중'],21,MUTED)
    s.text(464,656,310,['메일 Task의 질문 도착'],23,RED,True)
    s.text(464,702,310,['“받는 사람은','누구인가요?”'],26,INK,True,leading=34)
    s.text(848,656,310,['표 설명 후 메일 질문 전달'],23,TEAL,True)
    s.text(848,702,310,['“메일 초안의 받는 사람은','누구인가요?”'],22,INK,leading=30)
    s.text(848,814,310,['Agent Execution / 답변 대기'],20,MUTED)
    s.text(64,847,1110,['음성 중단 / Task 취소 의도 확인은 별도'],20,MUTED)
    for x in [426,810]: s.line([(x,350),(x,834)],color=LINE,arrow=False,dashed=True,width=1)
    s.challenge(355,'수신과 중단의 우선 처리',['긴 해석 중에도 새 발화의 수신과 음성 중단'])
    s.challenge(515,'후속 실행과 대기 상태',['현재 Request와 메일 질문의 처리 연결'])
    s.challenge(675,'현재성 확인과 실제 전달',['정정 전 Response 보류와 전달 차례 결정'])
    return s


def slide45():
    s = Slide(45, '기능 정확성을 위한 VIA 기억과 Context 제공 설계',
        '여러 활동의 과거 근거 누락과 실제 전달 내용의 오연결 위험', '',
        '과거 근거를 연결하는 Context의 생산, 조회와 갱신 책임 설계',
        'VIA는 허용 근거를 연결 / 업무 판단과 제안서 작성은 Downstream Agent',
        '기능 정확성 / 반응성 / 자원 활용성', considerations_title='설계 고려 사항')
    s.text(64,263,1110,['과거 활동의 시간 흐름 / VIA가 받은 기록과 허용된 참조'],24,MUTED)
    for x in [64,444,824]: s.note(x,329,350,287,[],24,'#F4F6F8')
    s.text(84,347,310,['앞선 대화 C20'],24,INK,True)
    s.text(84,402,310,['VIA의 직접 설명 E20','평가 기준'],26,INK,True,leading=37)
    s.text(84,495,310,['실제 전달 P20의','설명 내용과 범위'],23,TEAL,True,leading=34)
    s.text(84,580,310,['생성과 실제 전달 구별'],19,MUTED)
    s.text(464,347,310,['이후 제품 조사 T21'],24,INK,True)
    s.text(464,402,310,['한 Downstream Agent','제품 비교 수행'],25,INK,True,leading=37)
    s.text(464,495,310,['VIA가 확인한 결과 참조','D21@v2'],23,TEAL,True,leading=34)
    s.text(464,580,310,['C21 대화에 연결'],19,MUTED)
    s.text(844,347,310,['이후 견적 업무 T22'],24,INK,True)
    s.text(844,402,310,['다른 Downstream Agent','견적 업무 수행'],25,INK,True,leading=37)
    s.text(844,495,310,['VIA가 확인한 결과 참조','D22@v1'],23,TEAL,True,leading=34)
    s.text(844,580,310,['C22 대화에 연결'],19,MUTED)
    s.line([(416,367),(442,367)],color=MUTED)
    s.line([(796,367),(822,367)],color=MUTED)
    # Dashed references converge on today's evidence, independently of time order.
    for xx in [239,619,999]:
        s.line([(xx,622),(xx,645)],color=TEAL,dashed=True,arrow=False)
    s.line([(239,645),(1192,645),(1192,841),(1174,841)],color=TEAL,dashed=True,width=2)
    s.text(64,662,1110,['이번 C23 / R23: 여러 활동을 참조하는 새 제안서 요청'],23,MUTED)
    s.rect(64,702,1110,103,'#EAF0F8','none')
    s.text(84,717,1070,['“지난번 네가 설명한 평가 기준에 맞춰, 앞서 조사한 제품 비교 결과와',
                        '받아둔 견적을 이번 제안서에 반영해줘.”'],26,INK,True,leading=37)
    s.rect(64,821,1110,46,'#F0F8F7','none')
    s.text(84,831,1070,['VIA: P20 전달 범위와 D21/D22 발췌 또는 참조를 이번 Context에 연결'],23,INK,True)
    s.challenge(355,'여러 활동의 근거 연결',['직접 설명과 두 업무 결과의 후보 및 버전 확인'])
    s.challenge(515,'실제 전달과 자료 범위',['설명 원문과 실제 표시/재생 범위 구별', '결과 본문과 허용된 참조 구별'])
    s.challenge(675,'제공 방식과 비용',['단순 실행 맥락이나 index/cache로 충분한지', '공통 파생 관계의 유지가 필요한지 검토'])
    return s


def build():
    slides = [overview(), slide41(), slide42(), slide43(), slide44(), slide45()]
    outputs = {}
    deck = ET.Element('mxfile', host='app.diagrams.net', type='device')
    full_deck = ET.Element('mxfile', host='app.diagrams.net', type='device')
    for slide in slides:
        outputs[DIAGRAMS / f'{slide.slug}.svg'] = slide.svg()
        outputs[DIAGRAMS / f'{slide.slug}.drawio'] = slide.drawio()
        if slide.number != 'overview':
            deck.append(copy.deepcopy(slide.diagram()))
    presentation_slides = [slides[0]] + [next(s for s in slides if s.number == n) for n in (*PRESENTATION_ORDER, 43)]
    for slide in presentation_slides:
        page = copy.deepcopy(slide.diagram())
        if slide.number == 43:
            page.set('name', '보충 자료 — '+slide.caption)
        full_deck.append(page)
    outputs[DECK / 'VIA-DP-background-41-45.drawio'] = ET.tostring(deck, encoding='unicode')+'\n'
    outputs[DECK / 'VIA-DP-background-overview-41-45.drawio'] = ET.tostring(full_deck, encoding='unicode')+'\n'
    sections = ''.join(f'<section id="{s.slug}"><h2>{html.escape(('보충 자료 — ' if s.number == 43 else '')+s.caption)}</h2><img src="../../architecture/12-decisions/decision-packages/diagrams/{s.slug}.svg" alt="{html.escape(s.caption)}"><p>{'45: VIA 직접 설명의 실제 전달 범위와 제품 비교/견적 결과 참조를 새 제안서 Context에 연결. 업무 판단과 작성은 Agent 책임. ' if s.number==45 else ''}<a href="../../architecture/12-decisions/decision-packages/diagrams/{s.slug}.drawio">draw.io 원본</a> / <a href="{s.slug}.png">PNG</a></p></section>' for s in presentation_slides)
    outputs[DECK / 'index.html'] = '<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA 사용자 경험 도입과 발표 배경</title><style>body{margin:0;background:#ecf0f3;font-family:Arial,"Apple SD Gothic Neo",sans-serif;color:#18232e}header{padding:22px 4vw;background:white}h1{font-size:26px;margin:0 0 12px}nav{display:flex;gap:22px;flex-wrap:wrap}a{color:#2858a5}main{max-width:1920px;margin:auto}section{padding:22px 2vw;scroll-margin-top:20px}h2{font-size:20px}img{display:block;width:100%;height:auto;background:white}p{font-size:16px}@media print{header,h2,p{display:none}section{padding:0;break-after:page}body{background:white}@page{size:16in 9in;margin:0}}</style><header><h1>VIA 사용자 경험 도입과 발표 배경</h1><nav><a href="../VIA-DP-background-and-comparison-41-45.pptx">본 발표 9장 + 부록 2장 PPTX</a><a href="VIA-DP-background-overview-41-45.drawio">발표 흐름 6페이지 draw.io</a><a href="VIA-DP-background-41-45.drawio">번호순 배경 참조 5페이지 draw.io</a>'+''.join(f'<a href="#{s.slug}">'+('사용자 경험 도입' if s.number=='overview' else ('보충: 43' if s.number==43 else f'04-{s.number}'))+'</a>' for s in presentation_slides)+'</nav></header><p>본 발표: 도입 → 41 → 44 → 45 → 42. 43은 보충 자료이며 후보 자격이나 설계 선택의 변경이 아닙니다.</p><main>'+sections+'</main></html>\n'
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
