#!/usr/bin/env python3
"""Documentation scene: local VIA/cloud ports, paired SVG and editable draw.io."""
import argparse
import html
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/diagrams'
W, H = 1920, 1080
FONT = 'Apple SD Gothic Neo, Arial, sans-serif'


class Scene:
    def __init__(self):
        self.items = []

    def text(self, x, y, width, lines, size=22, bold=False):
        self.items.append(dict(kind='text', id=f'n{len(self.items)}', x=x, y=y,
                               w=width, h=len(lines)*size*1.35, lines=lines,
                               size=size, bold=bold))

    def box(self, ident, x, y, w, h, name, kind='component', owner=None):
        self.items.append(dict(kind=kind, id=ident, x=x, y=y, w=w, h=h,
                               name=name, owner=owner))

    def edge(self, points, x, y, width, label, both=False, dashed=False):
        self.items.append(dict(kind='edge', id=f'n{len(self.items)}', points=points,
                               both=both, dashed=dashed))
        self.text(x, y, width, label, size=19)

    def render(self):
        svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" role="img" aria-labelledby="title">',
               '<title id="title">로컬 VIA와 클라우드 모델: 공통 입력·출력 책임</title>',
               '<defs><marker id="end" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M1 1L9 5L1 9" fill="none" stroke="black" stroke-width="1.2"/></marker></defs>',
               '<rect width="1920" height="1080" fill="white"/>', f'<g font-family="{FONT}" fill="black">']
        mxfile = ET.Element('mxfile', host='app.diagrams.net', type='device')
        diagram = ET.SubElement(mxfile, 'diagram', id='common40-execution', name='공통 실행 계약')
        model = ET.SubElement(diagram, 'mxGraphModel', page='1', pageWidth=str(W), pageHeight=str(H), background='#FFFFFF')
        root = ET.SubElement(model, 'root')
        ET.SubElement(root, 'mxCell', id='0')
        ET.SubElement(root, 'mxCell', id='1', parent='0')
        byid = {i['id']: i for i in self.items}
        for i in self.items:
            kind, ident = i['kind'], i['id']
            if kind == 'edge':
                points = i['points']
                svg.append(f'<polyline points="{" ".join(f"{x},{y}" for x,y in points)}" fill="none" stroke="black" stroke-width="1.2" marker-end="url(#end)"'+(' marker-start="url(#end)"' if i['both'] else '')+(' stroke-dasharray="6 5"' if i['dashed'] else '')+'/>')
                style = 'edgeStyle=none;rounded=0;strokeWidth=1.2;strokeColor=#000000;endArrow=open;endFill=0;'
                if i['both']: style += 'startArrow=open;startFill=0;'
                if i['dashed']: style += 'dashed=1;'
                c = ET.SubElement(root, 'mxCell', id=ident, edge='1', parent='1', style=style)
                g = ET.SubElement(c, 'mxGeometry', relative='1', attrib={'as': 'geometry'})
                for (x,y),role in [(points[0],'sourcePoint'), (points[-1],'targetPoint')]:
                    ET.SubElement(g,'mxPoint',x=str(x),y=str(y),attrib={'as':role})
                a = ET.SubElement(g,'Array',attrib={'as':'points'})
                for x,y in points[1:-1]: ET.SubElement(a,'mxPoint',x=str(x),y=str(y))
                continue
            x,y,w,h = (i[k] for k in ('x','y','w','h'))
            parent = i.get('owner') or '1'
            if parent != '1':
                p = byid[parent]
                assert p['kind'] == 'component'
                assert p['x'] <= x and p['y'] <= y and x+w <= p['x']+p['w'] and y+h <= p['y']+p['h']
            if kind == 'text':
                size = i['size']
                for n,line in enumerate(i['lines']):
                    svg.append(f'<text x="{x}" y="{y+size+n*size*1.35}" data-width="{w}" font-size="{size}" font-weight="{700 if i["bold"] else 400}">{html.escape(line)}</text>')
                value = '<br>'.join(html.escape(t) for t in i['lines'])
                style = f'text;html=1;align=left;verticalAlign=top;spacing=0;whiteSpace=nowrap;overflow=visible;fontSize={size};fontStyle={int(i["bold"])};fillColor=none;strokeColor=none;'
            else:
                stroke = '#777777' if kind == 'boundary' else '#000000'
                dash = ' stroke-dasharray="8 6"' if kind == 'boundary' else ''
                rounded = ' rx="8"' if kind == 'module' else ''
                style = f'rounded={int(kind=="module")};arcSize=14;fillColor=#FFFFFF;strokeColor={stroke};strokeWidth=1.2;'
                if kind == 'boundary': style += 'dashed=1;'
                if kind == 'external':
                    svg.append(f'<polygon points="{x+18},{y} {x+w-18},{y} {x+w},{y+h/2} {x+w-18},{y+h} {x+18},{y+h} {x},{y+h/2}" fill="white" stroke="black" stroke-width="1.2"/>')
                    style += 'shape=hexagon;'
                elif kind == 'state':
                    r=8
                    svg.append(f'<path d="M{x},{y+r}C{x},{y-r} {x+w},{y-r} {x+w},{y+r}V{y+h-r}C{x+w},{y+h+r} {x},{y+h+r} {x},{y+h-r}Z M{x},{y+r}C{x},{y+3*r} {x+w},{y+3*r} {x+w},{y+r}" fill="white" stroke="black" stroke-width="1.2"/>')
                    style += 'shape=cylinder;size=8;'
                else:
                    svg.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="white" stroke="{stroke}" stroke-width="1.2"{dash}{rounded}/>')
                size=24 if kind == 'component' else 22
                left = kind in ('component', 'boundary')
                tx,ty = (x+16,y+32) if left else (x+w/2,y+h/2+8)
                svg.append(f'<text x="{tx}" y="{ty}" data-width="{w-32}" text-anchor="{"start" if left else "middle"}" font-size="{size}" font-weight="{700 if kind=="component" else 400}">{html.escape(i["name"])}</text>')
                value = i['name']
                style += f'align={"left" if left else "center"};verticalAlign={"top" if left else "middle"};spacingLeft={16 if left else 0};spacingTop={8 if left else 0};fontSize={size};fontStyle={int(kind=="component")};'
            c = ET.SubElement(root,'mxCell',id=ident,parent=parent,vertex='1',value=value,style=style+'fontFamily=Apple SD Gothic Neo;fontColor=#000000;')
            if parent != '1': x,y=x-byid[parent]['x'],y-byid[parent]['y']
            ET.SubElement(c,'mxGeometry',x=str(x),y=str(y),width=str(w),height=str(h),attrib={'as':'geometry'})
        svg += ['</g></svg>']
        svg = '\n'.join(svg)+'\n'
        drawio = ET.tostring(mxfile,encoding='unicode')+'\n'
        ET.fromstring(svg); ET.fromstring(drawio)
        return {'svg':svg, 'drawio':drawio}


def scene():
    s=Scene()
    s.text(40,22,1820,['공통 실행 계약 / 로컬 VIA가 입력과 상태를 소유하고 클라우드 모델을 사용한다'],34,True)
    s.text(40,78,1820,['로컬 VAD 승인 / 상세 계약 초안 / 공통 책임 확대도 / DP·A/B 선정이나 구현·측정 결과가 아님'],22)
    s.box('local',30,125,1440,855,'기기 내 VIA','boundary')
    s.box('cloud',1510,125,380,855,'클라우드 의존성','boundary')
    s.box('im',55,205,610,440,'Interaction Manager')
    s.box('channel',80,270,230,65,'Channel I/O','module','im')
    s.box('turn',355,270,285,65,'Turn-Taking Control','module','im')
    s.text(360,344,280,['로컬 VAD / 시작·종료 후보','즉시 stop / 출력 세대 무효화'],19)
    s.box('capture',80,430,230,65,'Evidence Capture','module','im')
    s.box('timeline',355,430,285,65,'Timeline & Buffer','module','im')
    s.box('playback',80,540,230,75,'Playback State','state','im')
    s.text(355,553,270,['권위 시각: 로컬 sample/time','근거 gap·전사 revision 보존'],19)
    s.box('ma',770,205,675,155,'Model Access')
    s.box('voice',790,270,290,65,'Voice API Client','module','ma')
    s.box('semantic',1105,270,315,65,'Semantic API Client','module','ma')
    s.text(790,365,650,['provider/session 대응·제한·cancel·usage는 Model Access 책임'],19)
    s.box('rc',770,430,300,110,'Request Controller')
    s.box('ri',1130,430,315,110,'Request Interpreter')
    s.box('voicecloud',1540,235,325,130,'음성 모델 / 전사 의존성','external')
    s.box('llmcloud',1540,445,325,130,'의미 해석 LLM','external')
    s.text(1540,630,325,['서버 session은 임시 context','Request·Task·실제 전달의','영속 권위는 로컬 VIA에 있음'],21)
    s.box('rm',770,715,675,240,'Response Manager')
    s.box('composer',790,785,285,65,'Response Composer','module','rm')
    s.box('publication',1105,785,315,65,'Publication Control','module','rm')
    s.box('outbox',1105,875,315,70,'Publication Outbox','state','rm')
    s.edge([(80,302),(65,302),(65,180),(745,180),(745,302),(790,302)],320,140,480,['1 audio stream / 전사 결과 대응'],both=True)
    s.edge([(310,302),(355,302)],82,345,250,['로컬 audio → VAD'])
    s.edge([(640,315),(650,315),(650,409),(610,409),(610,430)],360,398,255,['시작·종료 경계'])
    s.edge([(310,462),(355,462)],82,500,250,['화면·pointer 시각'])
    s.edge([(640,462),(770,462)],651,399,475,['2 InputStarted / 확정 입력 + 근거'])
    s.edge([(1070,493),(1130,493)],840,550,590,['3 해석 job / 의미 제안·코드 채택'],both=True)
    s.edge([(1285,430),(1285,335)],1110,403,335,['C-INTERPRET'],both=True)
    s.edge([(1080,300),(1090,300),(1090,185),(1485,185),(1485,300),(1540,300)],1120,145,320,['audio / transcript'],both=True)
    s.edge([(1420,315),(1485,315),(1485,510),(1540,510)],1498,404,390,['판단 요청 / 제안'],both=True)
    s.edge([(1030,540),(1030,680),(1300,680),(1300,715)],1050,617,380,['채택 의미 / 후속 응답 준비'])
    s.edge([(935,715),(935,610),(710,610),(710,298),(770,298)],718,585,690,['4 내용·음성 준비 / Model Access 호출'],both=True,dashed=True)
    s.edge([(770,817),(460,817),(460,645)],73,751,680,['5 유효 release → 실제 표시·재생'],both=True)
    s.text(73,844,675,['6 실제 receipt → Publication Outbox / 이후 질문 focus 연결','모델 생성 완료 ≠ 실제 전달 완료'],22)
    s.text(73,917,650,['Context·Task·Gateway·Policy·State Store 경로는 본문 참조'],20)
    s.text(40,1000,1830,['Legend: 직각 Component / 둥근 내부 Module / 원통 owner의 상태 / 육각 외부 의존성 / 점선 큰 배치 영역'],22)
    s.text(40,1038,1830,['모든 요소는 공통이므로 흰색·검정. Voice API Client·Semantic API Client는 adapter 역할명이며 독립 Component가 아니다.'],20)
    return s


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    for ext,value in scene().render().items():
        path=OUT/f'common40-execution.{ext}'
        if args.check:
            if not path.exists() or path.read_text()!=value: raise SystemExit(f'Drift: {path}')
        else: path.write_text(value)
    print('PASS: common execution SVG/draw.io parity and contained module/state owners')


if __name__=='__main__':
    main()
