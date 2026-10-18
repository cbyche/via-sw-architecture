"""Generate editable comparison figures for the 2026-10-18 design analysis.

Documentation assets only. No candidate execution or performance measurement.
Run with the bundled Python runtime, which provides Pillow.
"""
from pathlib import Path
import html
import json
import math
import xml.etree.ElementTree as ET

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/seven-problem-review/diagrams'
W, H = 1800, 1180
FONT = '/System/Library/Fonts/AppleSDGothicNeo.ttc'
INK, BLUE, GRAY, ORANGE = '#182B40', '#176EAD', '#516275', '#A45A18'


class Figure:
    def __init__(self, slug, title, question, width=W, height=H):
        self.slug = slug
        self.width, self.height = width, height
        self.nodes, self.edges, self.labels = [], [], []
        self.label(55, 47, title, 42, INK, True)
        self.label(55, 112, question, 27, GRAY)
        self.label(55, 170, '기본 구조의 선택   |   파랑: 처리 경로   |   주황: 유지 상태   |   하단: 선택 후 보완 기법', 22, BLUE)
        self.label(55, height-55, '설계 분석 그림. 성능 수치와 측정 결과가 아니며 실패 및 복구 계약은 연결된 본문에 수록.', 21, GRAY)

    def label(self, x, y, text, size=24, color=INK, bold=False):
        self.labels.append(dict(x=x, y=y, text=text, size=size, color=color, bold=bold))

    def node(self, key, x, y, w, h, lines, state=False, panel=False):
        self.nodes.append(dict(key=key, x=x, y=y, w=w, h=h, lines=lines,
                               state=state, panel=panel))

    def edge(self, a, b, points, label='', dashed=False):
        self.edges.append(dict(a=a, b=b, points=points, label=label, dashed=dashed))

    def note(self, x, y, text, w=800):
        self.node(f'note{len(self.nodes)}', x, y, w, 70, text)

    def save(self):
        W, H = self.width, self.height
        OUT.mkdir(parents=True, exist_ok=True)
        img = Image.new('RGB', (W, H), 'white')
        draw = ImageDraw.Draw(img)
        svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">',
               f'<title>{html.escape(self.slug)}</title>',
               '<defs><marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="#516275"/></marker></defs>',
               f'<rect width="{W}" height="{H}" fill="white"/>']
        mx = ET.Element('mxfile', host='app.diagrams.net')
        diagram = ET.SubElement(mx, 'diagram', name=self.slug)
        model = ET.SubElement(diagram, 'mxGraphModel', page='1', pageWidth=str(W), pageHeight=str(H))
        root = ET.SubElement(model, 'root')
        ET.SubElement(root, 'mxCell', id='0')
        ET.SubElement(root, 'mxCell', id='1', parent='0')

        def text_at(x, y, text, size, color=INK, bold=False, width=None):
            font = ImageFont.truetype(FONT, size)
            if width:
                while draw.textlength(text, font=font) > width and size > 18:
                    size -= 1
                    font = ImageFont.truetype(FONT, size)
                if draw.textlength(text, font=font) > width:
                    raise ValueError(f'Overfull text in {self.slug}: {text}')
            draw.text((x, y), text, font=font, fill=color, stroke_width=0)
            svg.append(f'<text x="{x}" y="{y+size}" font-family="Apple SD Gothic Neo,Arial,sans-serif" font-size="{size}" font-weight="{700 if bold else 400}" fill="{color}">{html.escape(text)}</text>')

        for n in self.nodes:
            x, y, w, h = (n[k] for k in ('x', 'y', 'w', 'h'))
            fill = '#F8FAFC' if n['panel'] else '#E9F4FB' if not n['state'] else '#FFF4E7'
            stroke = '#C6D1DE' if n['panel'] else BLUE if not n['state'] else ORANGE
            draw.rounded_rectangle((x, y, x+w, y+h), radius=10, fill=fill, outline=stroke, width=2)
            svg.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="10" fill="{fill}" stroke="{stroke}" stroke-width="2"/>')
            size = 28 if n['panel'] else 25
            for i, line in enumerate(n['lines']):
                text_at(x+18, y+16+i*35, line, size, INK, i == 0, w-36)
            value = '<br>'.join(html.escape(s) for s in n['lines'])
            c = ET.SubElement(root, 'mxCell', id=n['key'], value=value, vertex='1', parent='1',
                              style=f'rounded=1;whiteSpace=wrap;html=1;fillColor={fill};strokeColor={stroke};fontColor={INK};fontFamily=Arial;fontSize={size};align=left;verticalAlign=top;spacing=16;')
            ET.SubElement(c, 'mxGeometry', x=str(x), y=str(y), width=str(w), height=str(h), **{'as': 'geometry'})

        for i, e in enumerate(self.edges):
            ps = e['points']
            for (x1, y1), (x2, y2) in zip(ps, ps[1:]):
                if not e['dashed']:
                    draw.line((x1, y1, x2, y2), fill=GRAY, width=3)
                else:
                    d = math.hypot(x2-x1, y2-y1)
                    for start in range(0, int(d), 16):
                        end = min(start+8, d)
                        draw.line((x1+(x2-x1)*start/d, y1+(y2-y1)*start/d,
                                   x1+(x2-x1)*end/d, y1+(y2-y1)*end/d), fill=GRAY, width=3)
            (px, py), (ex, ey) = ps[-2:]
            angle = math.atan2(ey-py, ex-px)
            pts = [(ex, ey)] + [(ex-14*math.cos(angle+q), ey-14*math.sin(angle+q)) for q in (-.5, .5)]
            draw.polygon(pts, fill=GRAY)
            coords = ' '.join(f'{x},{y}' for x, y in ps)
            dash = 'stroke-dasharray="8 8"' if e['dashed'] else ''
            svg.append(f'<polyline points="{coords}" fill="none" stroke="{GRAY}" stroke-width="3" {dash} marker-end="url(#arrow)"/>')
            c = ET.SubElement(root, 'mxCell', id=f'edge{i}', value=e['label'], edge='1', parent='1', source=e['a'], target=e['b'],
                              style=f'edgeStyle=orthogonalEdgeStyle;endArrow=block;html=1;strokeColor={GRAY};strokeWidth=2;dashed={int(e["dashed"])};')
            g = ET.SubElement(c, 'mxGeometry', relative='1', **{'as': 'geometry'})
            ar = ET.SubElement(g, 'Array', **{'as': 'points'})
            for x, y in ps[1:-1]:
                ET.SubElement(ar, 'mxPoint', x=str(x), y=str(y))

        for i, t in enumerate(self.labels):
            text_at(t['x'], t['y'], t['text'], t['size'], t['color'], t['bold'], W-t['x']-40)
            c = ET.SubElement(root, 'mxCell', id=f'label{i}', value=html.escape(t['text']), vertex='1', parent='1',
                              style=f'text;html=1;whiteSpace=wrap;align=left;fontFamily=Arial;fontSize={t["size"]};fontColor={t["color"]};fontStyle={int(t["bold"])};')
            ET.SubElement(c, 'mxGeometry', x=str(t['x']), y=str(t['y']), width=str(W-t['x']-40), height=str(t['size']+12), **{'as': 'geometry'})
        svg.append('</svg>')
        (OUT / f'{self.slug}.svg').write_text('\n'.join(svg)+'\n')
        ET.indent(mx)
        (OUT / f'{self.slug}.drawio').write_text(ET.tostring(mx, encoding='unicode')+'\n')
        img.save(OUT / f'{self.slug}.png')


def selection():
    f = Figure('problem06-agent-selection', 'Agent 선택: 요청 판단에 결합할 것인가, 선택 실행을 분리할 것인가',
               '같은 업무: 수정한 비교 결과로 편집 가능한 발표자료를 만들고, 이번에는 다른 Agent에게 맡긴다.')
    for side, x in [('A', 55), ('B', 925)]:
        f.node(side, x, 235, 820, 760, [side + ('  요청 해석과 선택 결합' if side == 'A' else '  요청 계약 뒤 별도 선택')], panel=True)
        f.node(side+'in', x+35, 315, 750, 90, ['사용자 입력과 허용 근거', '같은 후보 정보 및 독립 성과 근거' if side == 'A' else '요청의 의미를 해석할 원본과 최근 문맥'])
        f.node(side+'state', x+35, 760, 750, 100, ['유지 상태', '요청 문맥과 후보 판본' if side == 'A' else '요청 계약과 별도 선택 문맥 / 후보 판본'], state=True)
        f.node(side+'gate', x+35, 890, 750, 75, ['현재 요청 및 적격성 확인 → 업무 인계'], state=False)
    f.node('aj', 90, 460, 750, 170, ['Request Interpreter', '요청의 목적과 대상을 해석하면서 Agent도 제안', '후보 변경 시 결합 판단 재실행'])
    f.edge('Ain','aj',[(465,405),(465,460)])
    f.edge('aj','Astate',[(465,630),(465,760)])
    f.edge('Astate','Agate',[(465,860),(465,890)])
    f.node('bi', 960, 440, 750, 95, ['Request Interpreter', '목적 / 대상 / 조건과 원본 참조를 계약으로 생산'])
    f.node('bs', 960, 600, 750, 105, ['선택 전용 판단: 같은 후보 및 성과 근거 공급', '계약과 후보를 비교. 후보만 바뀌면 선택만 반복'])
    f.edge('Bin','bi',[(1335,405),(1335,440)])
    f.edge('bi','bs',[(1335,535),(1335,600)])
    f.edge('bs','Bstate',[(1335,705),(1335,760)])
    f.edge('Bstate','Bgate',[(1335,860),(1335,890)])
    f.edge('bs','bi',[(1710,650),(1730,650),(1730,490),(1710,490)],dashed=True)
    f.label(75, 1025, 'A1 후보 설명 조회 / A2 선택만 갱신', 24, ORANGE)
    f.label(945, 1025, 'B1 선택 결과 재사용 / B2 부족한 계약만 보완', 24, ORANGE)
    f.save()


def evidence():
    f = Figure('problem01-evidence-supply', '현재 요청을 판단할 근거: 유지 문맥과 요청별 구성',
               '같은 요청: “아까 실제로 설명한 결론 중 비용 비교만 다시 알려줘.” 원본과 실제 전달 기록은 모두 보존한다.')
    data = [
        ('A',55,'대화 작업 문맥 유지', ['이전 입력 / 실제 전달 / 관련 근거','작업 문맥을 계속 갱신'], ['현재 요청을 유지 문맥에서 판단','부족하거나 불일치하면 원본 읽기'], ['대화 작업 문맥','source 참조 / 판본 / 사용 범위']),
        ('B',635,'요청마다 원본 근거 구성', ['현재 요청과 최소 대화 상태','필요 범위와 원본을 선택'], ['원본을 읽고 이번 묶음을 구성','원본별 실제 전달 / 수정 관계 결합'], ['요청별 근거 묶음','조회 ID / source 판본 / 누락 범위']),
        ('C',1215,'게시된 관계 근거를 조회', ['원본 변경 → 독립 생산과 게시','source를 가진 관계 근거 유지'], ['현재 요청에 맞는 게시 범위 읽기','미지원 / stale 범위는 원본 보완'], ['게시 근거 / coverage / 생산 job','요청별 근거 묶음'])]
    for side,x,title,first,second,state in data:
        f.node(side,x,235,530,745,[side+'  '+title],panel=True)
        f.node(side+'p',x+25,325,480,120,first)
        f.node(side+'s',x+25,500,480,120,second if side == 'B' else state,state=side != 'B')
        f.node(side+'q',x+25,675,480,120,state if side == 'B' else second,state=side == 'B')
        f.node(side+'g',x+25,850,480,90,['현재 source / 권한 / 입력 판본 확인','근거와 부족 범위를 반환'])
        f.edge(side+'p',side+'s',[(x+265,445),(x+265,500)])
        f.edge(side+'s',side+'q',[(x+265,620),(x+265,675)])
        f.edge(side+'q',side+'g',[(x+265,795),(x+265,850)])
    f.label(65,1010,'주 비교 A↔B: 문맥 수명. B↔C: 사전 관계 생산. A+C도 가능하며 별도 구조 조합으로 기록한다.',25,BLUE)
    f.label(65,1055,'선택 A 이후 A1 입력 prefix cache / A2 변경 조각 갱신. 다른 안의 보완과 결합 비용은 본문 참조.',23,ORANGE)
    f.save()


def interpretation():
    f=Figure('problem02-interpretation-execution','후속 입력의 의미 판단: 한 대화 실행과 요청별 실행',
             'D1을 읽고 긴 판단을 하는 도중 “그건 계속하고 D2 결론도 알려줘.”라는 별도 요청이 들어온다.')
    for s,x,title in [('A',55,'한 대화 해석 실행'),('B',925,'공통 대화와 요청별 해석 실행')]:
        f.node(s,x,235,820,745,[s+'  '+title],panel=True)
        f.node(s+'in',x+35,315,750,90,['새 입력 / 확인된 Agent 통지 수신','입력 보존과 로컬 음성 중단은 계속'])
        f.node(s+'out',x+35,865,750,85,['현재 관계 / revision 확인 → 필요한 재결합 → 표시'])
    f.node('ac',90,470,750,135,['현재 VIA 판단 step → 다음 판단 경계','새 입력의 관계와 내용 판단','앞선 목표 전체의 완료를 기다리는 구조는 아님'])
    f.node('as',90,680,750,125,['하나의 작업 문맥 + 요청별 잠정 상태','읽은 근거와 미해결 목표를 함께 보존','외부 Agent 대기에는 모델 호출 자원을 놓음'],state=True)
    f.edge('Ain','ac',[(465,405),(465,470)])
    f.edge('ac','as',[(465,605),(465,680)])
    f.edge('as','Aout',[(465,805),(465,865)])
    f.node('bc',960,445,750,95,['공통 대화 해석: 입력 관계 판단 / 짧은 답','긴 읽기와 판단을 요청별 실행에 연결'])
    f.node('br1',960,610,355,180,['R1 실행: D1 판단','요청별 문맥 / revision','사용 근거 / 잠정 결과','실제 종료까지 slot 점유'],state=True)
    f.node('br2',1355,610,355,180,['R2 실행: D2 판단','별도 문맥 / revision','같은 전체 호출 예산','여유가 있어야 동시 진행'],state=True)
    f.edge('Bin','bc',[(1335,405),(1335,445)])
    f.edge('bc','br1',[(1140,540),(1140,610)])
    f.edge('bc','br2',[(1530,540),(1530,610)])
    f.edge('br1','Bout',[(1140,790),(1140,865)])
    f.edge('br2','Bout',[(1530,790),(1530,865)])
    f.edge('bc','Bout',[(1710,490),(1730,490),(1730,905),(1710,905)])
    f.label(70,1015,'B: 앞선 step 대기 감소 가능 ↔ 실행별 문맥 / 재결합 / 동시 상태 비용. 외부 업무 대기는 공통.',24,BLUE)
    f.label(70,1060,'선택 B 이후 B1 불변 근거 참조 공유 / B2 같은 호출 한도 안에서 새 입력용 여유 예약',24,ORANGE)
    f.save()


def coordination():
    f=Figure('problem03-execution-coordination','DP-44: 비동기 작업의 다음 단계를 누가 시작하는가',
             '설명 준비 중 사용자 수정과 Agent 질문이 들어온다. 의미 판단 기능과 총 실행 예산은 두 안이 같다.')
    for s,x,title in [('A',55,'중앙 비동기 Dispatcher'),('B',925,'Stage 연결과 Publication Join')]:
        f.node(s,x,235,820,745,[s+'  '+title],panel=True)
        f.node(s+'in',x+35,315,750,90,['사용자 입력 / 확정 업무 사실 / 실제 전달 receipt'])
        f.node(s+'gate',x+35,875,750,80,['원본 owner의 채택 / 현재성 검사 → 실제 표시'])
    f.node('ad',90,460,750,125,['Dispatcher: 짧은 상태 전이와 후속 job 시작','긴 모델 호출 완료를 기다리지 않고 다음 사건 수신','완료 회수 뒤 다음 단계와 게시 조건 연결'])
    f.node('aw',90,650,400,145,['공통 bounded worker','자료 읽기 / 의미 판단','응답 준비 작업'])
    f.node('ast',520,650,320,145,['중앙 실행 상태','job / continuation','revision / candidate'],state=True)
    f.edge('Ain','ad',[(465,405),(465,460)])
    f.edge('ad','aw',[(290,585),(290,650)])
    f.edge('aw','ad',[(90,720),(72,720),(72,515),(90,515)])
    f.edge('ad','ast',[(680,585),(680,650)])
    f.edge('ast','Agate',[(680,795),(680,875)])
    f.node('bi',960,465,355,145,['Input Resolution Stage','입력과 해석 결과 소비','job / revision window'],state=True)
    f.node('bn',1355,465,355,145,['Task Notice Stage','확인된 업무 사실 소비','job / revision window'],state=True)
    f.node('bc',960,675,750,80,['Response Composer: 해당 응답에 필요한 입력만 결합'])
    f.node('bj',960,790,750,60,['Publication Join: 후보 / 허가 / source / 전달 참조'],state=True)
    f.edge('Bin','bi',[(1140,405),(1140,465)])
    f.edge('Bin','bn',[(1530,405),(1530,465)])
    f.edge('bi','bc',[(1140,610),(1140,675)])
    f.edge('bn','bc',[(1530,610),(1530,675)])
    f.edge('bc','bj',[(1335,755),(1335,790)])
    f.edge('bj','Bgate',[(1335,850),(1335,875)])
    f.label(70,1015,'B가 줄이는 것은 중앙 완료 회수와 활성화 비용. 같은 의미 계산과 외부 Agent 대기는 남는다.',24,BLUE)
    f.label(70,1060,'선택 A 이후 A1 단순 진행 통지 준비 합치기 / A2 변경된 근거에 의존하는 작업 인덱스',24,ORANGE)
    f.save()


def grounding():
    f=Figure('dp47-seven-comparison','DP-47 통합: 발화 구간과 화면 원본을 어떻게 해석하는가',
             '사용자가 D1의 두 문단을 가리키고 페이지를 바꾼 뒤 “이것들과 방금 그래프를 비교해줘.”라고 말한다.')
    for s,x,title in [('A',55,'원본을 함께 읽는 결합 판단'),('B',925,'독립 후보 생산과 결합')]:
        f.node(s,x,235,820,745,[s+'  '+title],panel=True)
        f.node(s+'in',x+35,315,750,105,['같은 음성 / 전사 revision + 화면 / 선택 / 포인터 관측','source 시각 / 수신 시각 / locator / 원본 모두 보존'])
        f.node(s+'out',x+35,875,750,75,['지칭 대상 집합과 source 근거 채택 / 모호하면 확인'])
    f.node('aj',90,485,750,150,['Request Interpreter: 결합 의미 판단','관련 화면 구간과 발화를 함께 읽음','명확한 native 선택 범위는 양안 모두 코드 처리','이미 읽은 원본과 잠정 결과를 같은 판단에서 재사용'])
    f.node('ast',90,695,750,125,['현재 판단 문맥과 근거 의존','각 대상의 원본 참조 / 불확실성 / revision','변경된 근거에 맞춰 영향받은 판단 재수행'],state=True)
    f.edge('Ain','aj',[(465,420),(465,485)])
    f.edge('aj','ast',[(465,635),(465,695)])
    f.edge('ast','Aout',[(465,820),(465,875)])
    f.node('bp',960,470,355,150,['화면 후보 생산','parser / local semantics','영역 / 관계 / 원본 근거','누락 / 경쟁 후보와 revision'],state=True)
    f.node('bs',1355,470,355,150,['발화 후보 생산','지칭 표현 / 구간 / 조건','전사 근거와 수정 범위','불명 / 경쟁 후보와 revision'],state=True)
    f.node('bb',960,685,750,130,['후보 결합: 일의적 typed 대응은 코드로 결합','자유 의미가 필요하면 모델로 결합 / 재질의','부족한 후보는 생산자가 원본에서 확장'])
    f.edge('Bin','bp',[(1140,420),(1140,470)])
    f.edge('Bin','bs',[(1530,420),(1530,470)])
    f.edge('bp','bb',[(1140,620),(1140,685)])
    f.edge('bs','bb',[(1530,620),(1530,685)])
    f.edge('bb','bp',[(960,745),(943,745),(943,550),(960,550)],dashed=True)
    f.edge('bb','Bout',[(1335,815),(1335,875)])
    f.label(70,1015,'B: 밀집 화면의 대상을 명시적으로 구분할 가능성 ↔ 별도 생산과 후보 누락 / 추가 모델 비용.',24,BLUE)
    f.label(70,1060,'기본 추천 A. A1 원본 범위 / crop 재사용. B의 세부 helper 비용과 정확도 근거 한계는 본문 참조.',24,ORANGE)
    f.save()


def handoff():
    f=Figure('problem05-agent-handoff','Agent가 바뀌는 업무: 인계 자료를 언제, 누가 구성하는가',
             'Agent X의 비교를 사용자가 수정하고, Y에게 발표자료를 맡긴 뒤 Z에게 관련 메일을 맡긴다.')
    cols=[('A',55,'위임 시 목적별 묶음'),('B',635,'지속 업무 기록'),('C',1215,'필수 명세와 수신자 읽기')]
    for s,x,t in cols:
        f.node(s,x,235,530,745,[s+'  '+t],panel=True)
        f.node(s+'in',x+25,325,480,100,['채택한 조건 / 공개 결과 / 실제 전달','같은 원본과 현재 접근 권한'])
        f.node(s+'out',x+25,850,480,90,['계약 revision과 수행 범위 확정','실제 보낸 내용과 수신 상태 기록'])
    f.node('ap',80,485,480,135,['현재 위임 목적에 맞춰 자료 선택','설명 / 발췌 / 결과 참조 구성','이전에 확보한 유효 근거도 사용'])
    f.node('as',80,685,480,110,['준비 중 묶음 / 확정 송신 계약','새 위임 때 목적별로 다시 구성'],state=True)
    f.edge('Ain','ap',[(320,425),(320,485)])
    f.edge('ap','as',[(320,620),(320,685)])
    f.edge('as','Aout',[(320,795),(320,850)])
    f.node('bs',660,485,480,135,['업무 변경마다 기록 갱신','현재 조건 / 적용된 결과 관계','source / coverage / 생산 상태'],state=True)
    f.node('bp',660,685,480,110,['위임 목적에 맞춰 기록 추출','부족한 원본 보완과 현재성 확인'])
    f.edge('Bin','bs',[(900,425),(900,485)])
    f.edge('bs','bp',[(900,620),(900,685)])
    f.edge('bp','Bout',[(900,795),(900,850)])
    f.node('cs',1240,485,480,135,['VIA: 필수 목표 / 조건 / 자료 명세','판본 / 범위 / 권한 lease / 만료','필수 조건은 수신자 발견에 맡기지 않음'],state=True)
    f.node('cp',1240,685,480,110,['수신 Agent: 필요한 상세 자료 선택','허용 범위 안에서 읽고 업무 수행'])
    f.edge('Cin','cs',[(1480,425),(1480,485)])
    f.edge('cs','Cout',[(1720,550),(1735,550),(1735,890),(1720,890)])
    f.edge('Cout','cp',[(1480,850),(1480,795)])
    f.label(65,1010,'A↔B: 필요할 때 준비 대 반복 인계 재사용. C: 상세 자료 선택을 지원 가능한 수신자에게 맡김.',24,BLUE)
    f.label(65,1055,'선택 A 이후 A1 인계 묶음 cache / A2 같은 Agent의 native session 재사용. 공통 생산은 한 번만 계상.',23,ORANGE)
    f.save()


def lifecycle():
    f=Figure('problem07-lifecycle-boundaries','DP-42: 대화와 업무 상태를 어디서 확정하는가',
             '대화 C1의 요청 R1을 Task T1에 연결하고 실행 X1을 위임한다. 두 안 모두 대화와 업무의 수명이 다르다.')
    for s,x,t in [('A',55,'모듈화한 통합 Core'),('B',925,'독립 대화 / 업무 서비스')]:
        f.node(s,x,235,820,745,[s+'  '+t],panel=True)
        f.node(s+'in',x+35,315,750,90,['C1 / R1은 대화 상태, T1 / X1은 업무 상태','별도 논리 수명과 owner를 두 안 모두 유지'])
    f.node('ac',90,470,750,150,['Request Controller + Task Manager','같은 Core 안의 별도 기능 Module','트랜잭션 관리자가 관련 변경을 공동 확정','짧은 transaction. 외부 I/O는 commit 밖에서 수행'])
    f.node('as',90,685,750,110,['관련 Request / Task / command 상태 저장','R1↔T1 연결과 X1 의도 / outbox를 함께 확정'],state=True)
    f.node('ao',90,865,750,80,['Agent Gateway → 외부 수신 / 수행 사실 별도 확인'])
    f.edge('Ain','ac',[(465,405),(465,470)])
    f.edge('ac','as',[(465,620),(465,685)])
    f.edge('as','ao',[(465,795),(465,865)])
    f.node('bd',960,460,355,245,['대화 서비스','R1과 pending 의도 저장','inbox / outbox / receipt','대화 저장소와 writer','업무 접수 사실 반영'],state=True)
    f.node('bw',1355,460,355,245,['업무 서비스','T1 / X1 접수와 상태','inbox / outbox / receipt','독립 업무 저장소 / writer','외부 업무 사실 보존'],state=True)
    f.edge('Bin','bd',[(1140,405),(1140,460)])
    f.edge('bd','bw',[(1315,525),(1355,525)])
    f.edge('bw','bd',[(1355,650),(1315,650)])
    f.node('bg',960,765,750,80,['업무 accepted + dispatch 현재성 검사. 대화 반영과 분리'])
    f.node('bo',960,875,750,70,['Agent Gateway → 외부 수신 / 수행 사실 별도 확인'])
    f.edge('bw','bg',[(1530,705),(1530,765)])
    f.edge('bg','bo',[(1335,845),(1335,875)])
    f.label(70,1015,'B의 독립 writer / 장애 경계 가능성 ↔ 서비스 간 접수와 반영의 복구 비용. 정상 이익은 조건부.',24,BLUE)
    f.label(70,1060,'선택 A 이후 A1 저장 작업의 짧은 단위와 우선순위 / A2 읽기용 Task snapshot',24,ORANGE)
    f.save()


if __name__ == '__main__':
    evidence()
    interpretation()
    coordination()
    grounding()
    handoff()
    selection()
    lifecycle()
    print(json.dumps({'generated': [p.name for p in sorted(OUT.glob('*'))]}, ensure_ascii=False))
