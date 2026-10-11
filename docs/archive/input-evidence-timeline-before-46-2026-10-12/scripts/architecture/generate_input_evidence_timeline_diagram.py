#!/usr/bin/env python3
"""Standalone documentation scene; no dependency on the concurrently edited DP41 scene."""
import argparse
import html
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/diagrams'
STEM = 'input-evidence-timeline-structure'
W, H = 2560, 1440
FONT = 'Apple SD Gothic Neo, Arial, sans-serif'


class Scene:
    def __init__(self):
        self.nodes = []
        self.edges = []
        self.texts = []

    def node(self, ident, x, y, w, h, name, kind='component', owner=None, different=False):
        self.nodes.append(dict(id=ident, x=x, y=y, w=w, h=h, name=name,
                               kind=kind, owner=owner, different=different))

    def text(self, x, y, w, lines, size=22, bold=False):
        self.texts.append(dict(x=x, y=y, w=w, lines=lines, size=size, bold=bold))

    def edge(self, source, target, points, both=False):
        self.edges.append(dict(source=source, target=target, points=points, both=both))

    def render(self):
        byid = {n['id']: n for n in self.nodes}
        assert len(byid) == len(self.nodes)
        containers = {n['owner'] for n in self.nodes if n['owner']}
        svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" role="img" aria-labelledby="title">',
               '<title id="title">관측 타임라인: 공통 원본 인수와 생산자별 원본 조회 비교 초안</title>',
               '<defs><marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M1 1L9 5L1 9" fill="none" stroke="#333" stroke-width="1.2"/></marker></defs>',
               f'<rect width="{W}" height="{H}" fill="white"/>', f'<g font-family="{FONT}" fill="#111">']
        mx = ET.Element('mxfile', host='app.diagrams.net', type='device')
        d = ET.SubElement(mx, 'diagram', id=STEM, name='원본 인수와 연합 조회')
        model = ET.SubElement(d, 'mxGraphModel', page='1', pageWidth=str(W), pageHeight=str(H), background='#FFFFFF')
        root = ET.SubElement(model, 'root')
        ET.SubElement(root, 'mxCell', id='0')
        ET.SubElement(root, 'mxCell', id='1', parent='0')

        def svg_text(x, y, width, lines, size, bold=False, anchor='start'):
            for k, line in enumerate(lines):
                svg.append(f'<text x="{x}" y="{y+size+k*size*1.3}" data-width="{width}" font-size="{size}" font-weight="{700 if bold else 400}" text-anchor="{anchor}">{html.escape(line)}</text>')

        for n in self.nodes:
            x, y, w, h = (n[k] for k in ('x', 'y', 'w', 'h'))
            assert x >= 0 and y >= 0 and x+w <= W and y+h <= H
            parent = n['owner'] or '1'
            if parent != '1':
                p = byid[parent]
                assert p['kind'] in ('component', 'module', 'boundary')
                assert p['x'] <= x and p['y'] <= y and x+w <= p['x']+p['w'] and y+h <= p['y']+p['h']
            fill = '#FCE4D6' if n['different'] else '#FFFFFF'
            stroke = '#9B502B' if n['different'] else '#222222'
            sw = 2.4 if n['different'] else 1.6
            kind = n['kind']
            dash = ' stroke-dasharray="9 6"' if kind == 'boundary' else ''
            style = f'html=1;fillColor={fill};strokeColor={stroke};strokeWidth={sw};fontFamily=Apple SD Gothic Neo;fontColor=#111111;'
            if kind == 'external':
                svg.append(f'<polygon points="{x+22},{y} {x+w-22},{y} {x+w},{y+h/2} {x+w-22},{y+h} {x+22},{y+h} {x},{y+h/2}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"/>')
                style += 'shape=hexagon;'
            elif kind == 'state':
                r = 9
                svg.append(f'<path d="M{x},{y+r}C{x},{y-r} {x+w},{y-r} {x+w},{y+r}V{y+h-r}C{x+w},{y+h+r} {x},{y+h+r} {x},{y+h-r}Z M{x},{y+r}C{x},{y+3*r} {x+w},{y+3*r} {x+w},{y+r}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"/>')
                style += 'shape=cylinder;size=10;'
            else:
                svg.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{9 if kind=="module" else 0}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"{dash}/>')
                style += f'rounded={int(kind=="module")};arcSize=10;'
                if kind == 'boundary':
                    style += 'dashed=1;'
            lines = n['name'].split('\n')
            top = n['id'] in containers or kind in ('component', 'boundary')
            size = 25 if kind == 'component' else 21
            if top:
                svg_text(x+15, y+8, w-30, lines, size, kind=='component')
            else:
                svg_text(x+w/2, y+h/2-len(lines)*size*0.65, w-24, lines, size, False, 'middle')
            style += f'fontSize={size};fontStyle={int(kind=="component")};align={"left" if top else "center"};verticalAlign={"top" if top else "middle"};spacingLeft={15 if top else 0};spacingTop={8 if top else 0};'
            cell = ET.SubElement(root, 'mxCell', id=n['id'], value='<br>'.join(html.escape(s) for s in lines), parent=parent, vertex='1', style=style)
            if parent != '1':
                p = byid[parent]
                x, y = x-p['x'], y-p['y']
            ET.SubElement(cell, 'mxGeometry', x=str(x), y=str(y), width=str(w), height=str(h), attrib={'as':'geometry'})
        for k, e in enumerate(self.edges):
            points = e['points']
            assert e['source'] in byid and e['target'] in byid
            for nid, point in [(e['source'], points[0]), (e['target'], points[-1])]:
                n = byid[nid]
                px, py = point
                assert n['x'] <= px <= n['x']+n['w'] and n['y'] <= py <= n['y']+n['h'], (nid, point)
                assert px in (n['x'], n['x']+n['w']) or py in (n['y'], n['y']+n['h']), (nid, point)
            for a, b in zip(points, points[1:]):
                assert a[0] == b[0] or a[1] == b[1]
            svg.append(f'<polyline points="{" ".join(f"{x},{y}" for x,y in points)}" fill="none" stroke="#333" stroke-width="1.6" marker-end="url(#arrow)"'+(' marker-start="url(#arrow)"' if e['both'] else '')+'/>')
            # Explicit normalized terminals keep editable draw.io edges on the same ports.
            s, t = byid[e['source']], byid[e['target']]
            sx, sy = points[0]
            tx, ty = points[-1]
            style = f'edgeStyle=none;rounded=0;strokeColor=#333333;strokeWidth=1.6;endArrow=open;endFill=0;exitX={(sx-s["x"])/s["w"]};exitY={(sy-s["y"])/s["h"]};entryX={(tx-t["x"])/t["w"]};entryY={(ty-t["y"])/t["h"]};exitPerimeter=0;entryPerimeter=0;'
            if e['both']:
                style += 'startArrow=open;startFill=0;'
            c = ET.SubElement(root, 'mxCell', id=f'e{k}', edge='1', parent='1', source=e['source'], target=e['target'], style=style)
            g = ET.SubElement(c, 'mxGeometry', relative='1', attrib={'as':'geometry'})
            a = ET.SubElement(g, 'Array', attrib={'as':'points'})
            for x, y in points[1:-1]:
                ET.SubElement(a, 'mxPoint', x=str(x), y=str(y))
        for k, t in enumerate(self.texts):
            svg_text(t['x'], t['y'], t['w'], t['lines'], t['size'], t['bold'])
            c = ET.SubElement(root, 'mxCell', id=f't{k}', vertex='1', parent='1', value='<br>'.join(html.escape(s) for s in t['lines']), style=f'text;html=1;align=left;verticalAlign=top;spacing=0;whiteSpace=nowrap;overflow=visible;fillColor=none;strokeColor=none;fontSize={t["size"]};fontStyle={int(t["bold"])};fontFamily=Apple SD Gothic Neo;')
            ET.SubElement(c, 'mxGeometry', x=str(t['x']), y=str(t['y']), width=str(t['w']), height=str(len(t['lines'])*t['size']*1.3), attrib={'as':'geometry'})
        svg += ['</g></svg>']
        values = {'svg':'\n'.join(svg)+'\n', 'drawio':ET.tostring(mx, encoding='unicode')+'\n'}
        for v in values.values():
            ET.fromstring(v)
        return values


def scene():
    s = Scene()
    s.text(40, 15, 2470, ['발화·화면·사용자 행동의 관측 타임라인 — 원본 소유와 정상 읽기 계약'], 35, True)
    s.text(40, 68, 2470, ['04-40 적용 / 로컬 VIA·로컬 VAD·클라우드 모델 / 검토 초안·DP 번호 미선정·구현/측정 없음'], 24)
    for key, x in [('a', 30), ('b', 1310)]:
        def nid(n): return key+'-'+n
        def node(n, dx, y, w, h, name, kind='component', owner=None, diff=False):
            s.node(nid(n), x+dx, y, w, h, name, kind, nid(owner) if owner else None, diff)
        def edge(a, b, points, both=False):
            s.edge(nid(a), nid(b), [(x+px, py) for px,py in points], both)
        def text(dx, y, w, lines, size=21, bold=False):
            s.text(x+dx, y, w, lines, size, bold)
        title = 'A 중앙 공통 기록형' if key=='a' else 'B 생산자별 기록 조합형'
        text(25, 120, 1170, [title], 30, True)
        text(25, 168, 1170, ['원본 소유 인수 → 공통 원장 읽기' if key=='a' else '각 source가 원본 유지 → 유효 참조/coverage 조합'], 24)
        text(25, 210, 1170, ['같은 사건: “이 표와” + S1/P1 → scroll/창 전환 → “저 표를 비교해줘” + S2/P2'], 21)
        node('mic', 50, 265, 310, 60, '마이크', 'external')
        node('os', 420, 265, 350, 60, 'OS 화면·입력', 'external')
        node('voice', 855, 265, 330, 95, '클라우드 음성 모델\n전사 의존성', 'external')
        node('im', 25, 350, 800, 600, 'Interaction Manager')
        node('turn', 420, 390, 350, 55, 'Turn-Taking Control', 'module', 'im')
        node('channel', 50, 475, 310, 165, 'Channel I/O', 'module', 'im')
        node('capture', 420, 475, 350, 165, 'Evidence Capture', 'module', 'im')
        node('timeline', 50, 735, 720, 190, 'Timeline & Buffer', 'module', 'im')
        node('ma', 855, 435, 330, 490, 'Model Access')
        node('voiceapi', 875, 480, 290, 65, 'Voice API Client', 'module', 'ma')
        node('semanticapi', 875, 780, 290, 65, 'Semantic API Client', 'module', 'ma')
        node('rc', 50, 1040, 310, 85, 'Request Controller')
        node('ri', 420, 1040, 350, 85, 'Request Interpreter')
        node('llm', 855, 1160, 330, 85, '클라우드 의미 LLM', 'external')
        if key=='a':
            # Producer staging has no past-read authority after the ownership transfer.
            node('ledger', 80, 790, 280, 85, 'Observation Ledger', 'state', 'timeline', True)
            node('reader', 420, 800, 330, 65, 'Temporal Evidence Reader', 'module', 'timeline', True)
            edge('channel','ledger',[(130,640),(130,750),(220,750),(220,790)])
            edge('capture','ledger',[(470,640),(395,640),(395,780),(300,780),(300,790)])
            edge('ledger','reader',[(360,837),(420,837)],True)
            text(75, 885, 670, ['4 원본 인수·coverage → 원장 조회 / 참조·gap 반환'], 20)
        else:
            node('audio', 75, 550, 260, 65, 'Audio Evidence Buffer', 'state', 'channel', True)
            node('observations', 445, 550, 300, 65, 'Observation Buffers', 'state', 'capture', True)
            node('receipt', 80, 790, 280, 85, 'Query Receipt', 'state', 'timeline', True)
            node('reader', 420, 800, 330, 65, 'Evidence Query Coordinator', 'module', 'timeline', True)
            edge('reader','audio',[(420,820),(375,820),(375,630),(335,630),(335,582)],True)
            edge('reader','observations',[(750,820),(795,820),(795,630),(745,630),(745,582)],True)
            edge('reader','receipt',[(420,837),(360,837)],True)
            text(75, 885, 670, ['4 source별 조회·유효 참조·coverage → receipt 조합'], 20)
        edge('mic','channel',[(205,325),(205,475)])
        # OS observations go directly to Evidence Capture around the VAD module.
        edge('os','capture',[(740,325),(790,325),(790,510),(770,510)])
        edge('channel','turn',[(360,490),(385,490),(385,417),(420,417)])
        edge('turn','timeline',[(420,417),(400,417),(400,715),(580,715),(580,735)])
        text(50,390,340,['2 VAD 경계','onset / 종료 후보·확정'],20)
        edge('channel','voiceapi',[(360,525),(380,525),(380,665),(810,665),(810,510),(875,510)])
        text(415,642,370,['1 audio / 실제 sample·전송 대응'],18)
        edge('voiceapi','voice',[(1020,480),(1020,360)],True)
        text(865,375,315,['cloud audio / 전사','item ↔ 로컬 input 대응'],19)
        edge('voiceapi','timeline',[(875,535),(835,535),(835,705),(660,705),(660,735)])
        text(415,681,390,['3 전사 revision / item·sample 대응'],18)
        edge('reader','rc',[(580,865),(580,975),(205,975),(205,1040)])
        text(50,985,335,['5 확정 입력 + 당시 근거 참조','revision·시각 오차·gap'],20)
        edge('rc','ri',[(360,1082),(420,1082)],True)
        text(420,985,350,['6 해석 job / 의미 제안','코드 채택은 Request Controller'],20)
        edge('ri','semanticapi',[(770,1082),(825,1082),(825,812),(875,812)],True)
        edge('semanticapi','llm',[(1165,812),(1210,812),(1210,1202),(1185,1202)],True)
        text(875,953,300,['C-INTERPRET','관련 화면·행동 + 전사','최종 지칭은 의미 제안'],20)
        text(875,1070,300,['provider session은 임시','로컬 관측의 유일 원본 아님'],19)
        text(50,1160,730,[
            'A: 인수 후 생산자 종료에도 남은 원본 조회 가능' if key=='a' else 'B: 각 source의 읽기·만료·철회 계약에 계속 의존',
            '대가: 공통 인수/원장 계약·보관 부담' if key=='a' else '대가: 다중 read/lease·조합 실패/무효화 처리',
            '양안: 유한 보존 / timing 미제공이면 unknown·확인',
            '원본 생존은 실제 process·내구성 조건에 따름'],21)
    s.text(40,1325,2480,['Legend: 직각 Component / 둥근 Module / 원통 상태 / 육각 외부 의존성 / 살구색 차이 요소. 큰 경계는 process가 아님.'],22)
    s.text(40,1370,2480,['두 칸은 독립 대안. source별 사건/clock/gap은 공통. 시간 대응 ≠ 지칭 확정. 응답·위임·hold·삭제의 전체 연결은 본문과 04-40 참조.'],22)
    return s


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    for ext, value in scene().render().items():
        path = OUT / f'{STEM}.{ext}'
        if args.check:
            if not path.exists() or path.read_text() != value:
                raise SystemExit(f'Drift: {path}')
        else:
            path.write_text(value)
    print('PASS: timeline SVG/draw.io parity, XML, owner containment and orthogonal terminal ports')


if __name__ == '__main__':
    main()
