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


def overview():
    s = Slide('overview', 'VIA 사용자 경험을 위한 다섯 설계 문제',
        '정확한 Request 해석, 지속 대화와 과거 정보 재사용을 위한 설계',
        '', '다섯 문제의 구조 대안과 품질 손익 비교', '',
        '기능 정확성 / 반응성 / 변경 용이성 / 복구성')
    s.text(64, 258, 1792, ['사용자 상황별로 구분할 다섯 설계 책임'], 27, MUTED)
    s.rect(64, 317, 1792, 45, '#F1F4F7', 'none')
    for x,w,t in [(84,115,'문서'),(224,285,'사용자 경험'),(554,566,'예방할 위험'),(1164,665,'비교할 SW 구조의 결정')]:
        s.text(x, 326, w, [t], 23, MUTED, True)
    rows = [
        (41, '지금 요청의 이해', '선택 자료와 이전 Task의 오연결', ['정보 조회와 의미 완성의', '진행 책임']),
        (42, '작업의 지속 추적', 'Request 완료와 Task 완료의 혼동', ['Conversation과 Task 상태의', '관리 책임 및 실행 경계']),
        (43, '정정의 정확한 반영', '관련 참조 오류와 기존 조건 누락', ['연관된 의미 판단의', '생산 책임과 조정 방식']),
        (44, '끊김 없는 대화', '새 발화 누락과 부적절한 Response 전달', ['겹친 사건의 후속 실행과', '전달을 이어가는 실행 구조']),
        (45, '과거 정보의 활용', '과거 정정 누락과 유효하지 않은 결과 재사용', ['교차 기록 Context의', '생산, 조회와 갱신 책임']),
    ]
    for i,(n,experience,risk,decision) in enumerate(rows):
        y=375+i*98
        s.text(84,y+14,115,[f'04-{n}'],27,TEAL,True)
        s.text(224,y+15,285,[experience],26,INK,True)
        s.text(554,y+18,566,[risk],24,INK)
        s.text(1164,y+9,665,decision,24,MUTED,leading=32)
        s.line([(64,y+86),(1856,y+86)],color=LINE,arrow=False,width=1)
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
        '과거 정정의 누락과 유효하지 않은 결과 재사용 위험', '',
        '과거 기록을 연결한 Context의 생산, 조회와 갱신 책임 설계',
        '과거 정보 참조 / 장기 User Memory 등록은 별도 사용자 지시',
        '기능 정확성 / 반응성 / 자원 활용성', considerations_title='설계 고려 사항')
    s.text(64,263,1110,['지난번 / 서로 연결해야 할 세 종류의 기록'],25,MUTED)
    for x,w in [(64,350),(444,350),(824,350)]: s.note(x,329,w,287,[],24,'#F4F6F8')
    s.text(84,347,310,['정정 Request'],25,INK,True)
    s.text(84,408,310,['“결론 먼저,','짧은 문장으로','바꿔줘.”'],29,INK,True,leading=37)
    s.text(84,572,310,['사용자가 고친 내용'],21,MUTED)
    s.text(464,347,310,['수정 전후 결과'],25,INK,True)
    s.text(464,403,310,['수정 전'],19,MUTED)
    s.text(464,437,310,['배경 설명 … 결론'],23,MUTED)
    s.line([(464,481),(774,481)],color=LINE,arrow=False,width=1)
    s.text(464,506,310,['수정 후'],19,TEAL)
    s.text(464,548,310,['결론 먼저 / 짧은 문장'],25,TEAL,True)
    s.text(844,347,310,['실제 전달 기록'],25,INK,True)
    s.text(844,412,310,['수정 후 결과 전달'],27,TEAL,True)
    s.text(844,491,310,['어느 결과 버전을','실제로 받았는지 확인'],23,MUTED,leading=34)
    s.line([(423,471),(435,471)],color=MUTED)
    s.line([(803,471),(815,471)],color=MUTED)
    # These links identify the needed past Context, without a repository or
    # a chosen production/read schedule. They do not pre-bind today's Task.
    s.line([(239,622),(239,646),(999,646),(999,622)],color=LINE,arrow=False,width=2)
    s.text(64,664,1110,['이번 / 새 보고서의 Request'],23,MUTED)
    s.rect(64,708,1110,81,'#EAF0F8','none')
    s.text(84,728,1070,['“지난번에 내가 고친 표현 방식으로 이번 보고서도 정리해줘.”'],28,INK,True)
    s.rect(64,811,1110,55,'#F0F8F7','none')
    s.line([(619,646),(1192,646),(1192,839),(1174,839)],color=TEAL,width=3)
    s.text(84,825,1070,['이번 Request에 연결할 과거 Context / 정정 내용, 결과 버전과 실제 전달'],24,INK,True)
    s.challenge(355,'기록 간 관계의 복원',['정정 내용과 결과 버전 및 실제 전달의 연결'])
    s.challenge(515,'재사용 정보의 유효성',['원본 수정과 삭제 및 요약 누락의 확인'])
    s.challenge(675,'반복 조회와 유지 비용',['Context 생산과 조회 및 갱신 비용의 균형'])
    return s


def build():
    slides = [overview(), slide41(), slide42(), slide43(), slide44(), slide45()]
    outputs = {}
    deck = ET.Element('mxfile', host='app.diagrams.net', type='device')
    full_deck = ET.Element('mxfile', host='app.diagrams.net', type='device')
    for slide in slides:
        outputs[DIAGRAMS / f'{slide.slug}.svg'] = slide.svg()
        outputs[DIAGRAMS / f'{slide.slug}.drawio'] = slide.drawio()
        full_deck.append(copy.deepcopy(slide.diagram()))
        if slide.number != 'overview':
            deck.append(copy.deepcopy(slide.diagram()))
    outputs[DECK / 'VIA-DP-background-41-45.drawio'] = ET.tostring(deck, encoding='unicode')+'\n'
    outputs[DECK / 'VIA-DP-background-overview-41-45.drawio'] = ET.tostring(full_deck, encoding='unicode')+'\n'
    sections = ''.join(f'<section id="{s.slug}"><h2>{html.escape(s.caption)}</h2><img src="../../architecture/12-decisions/decision-packages/diagrams/{s.slug}.svg" alt="{html.escape(s.caption)}"><p><a href="../../architecture/12-decisions/decision-packages/diagrams/{s.slug}.drawio">draw.io 원본</a> / <a href="{s.slug}.png">PNG</a></p></section>' for s in slides)
    outputs[DECK / 'index.html'] = '<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA DP 설계 문제 지도와 배경 41–45</title><style>body{margin:0;background:#ecf0f3;font-family:Arial,"Apple SD Gothic Neo",sans-serif;color:#18232e}header{padding:22px 4vw;background:white}h1{font-size:26px;margin:0 0 12px}nav{display:flex;gap:22px;flex-wrap:wrap}a{color:#2858a5}main{max-width:1920px;margin:auto}section{padding:22px 2vw;scroll-margin-top:20px}h2{font-size:20px}img{display:block;width:100%;height:auto;background:white}p{font-size:16px}@media print{header,h2,p{display:none}section{padding:0;break-after:page}body{background:white}@page{size:16in 9in;margin:0}}</style><header><h1>VIA DP 설계 문제 지도와 배경 41–45</h1><nav><a href="VIA-DP-background-overview-41-45.drawio">전체 6페이지 draw.io</a><a href="VIA-DP-background-41-45.drawio">배경 5페이지 draw.io</a>'+''.join(f'<a href="#{s.slug}">'+('전체 지도' if s.number=='overview' else f'04-{s.number}')+'</a>' for s in slides)+'</nav></header><main>'+sections+'</main></html>\n'
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
