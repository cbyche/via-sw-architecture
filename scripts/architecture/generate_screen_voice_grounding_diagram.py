#!/usr/bin/env python3
"""DP-47 documentation only. One scene generates editable draw.io and SVG."""
import argparse
import html
from pathlib import Path
import re
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/diagrams'
W, H = 2560, 1440
BLACK, BLUE, GREEN = '#20252b', '#1763a6', '#087852'
FONT = 'Apple SD Gothic Neo, Arial, sans-serif'


class Scene:
    def __init__(self):
        self.items = []

    def text(self, x, y, w, lines, size=22, color=BLACK, bold=False):
        self.items.append(dict(kind='text', id=f't{len(self.items)}', x=x, y=y,
                               w=w, h=len(lines)*size*1.3, lines=lines, size=size,
                               color=color, bold=bold))

    def box(self, ident, x, y, w, h, name, kind='component', owner=None, color=BLACK, size=23):
        self.items.append(dict(kind=kind, id=ident, x=x, y=y, w=w, h=h,
                               lines=name.split('\n'), owner=owner, color=color, size=size))

    def edge(self, ident, source, target, points, dashed=False, color=BLACK):
        self.items.append(dict(kind='edge', id=ident, source=source, target=target,
                               points=points, dashed=dashed, color=color))

    def render(self):
        byid = {i['id']: i for i in self.items}
        assert len(byid) == len(self.items)
        for e in self.items:
            if e['kind']!='edge': continue
            for n in self.items:
                if n['kind'] not in ('module','state','data','external') or n['id'] in (e['source'],e['target']): continue
                for (x1,y1),(x2,y2) in zip(e['points'],e['points'][1:]):
                    if x1==x2:
                        hit=n['x']<x1<n['x']+n['w'] and max(min(y1,y2),n['y'])<min(max(y1,y2),n['y']+n['h'])
                    else:
                        hit=n['y']<y1<n['y']+n['h'] and max(min(x1,x2),n['x'])<min(max(x1,x2),n['x']+n['w'])
                    assert not hit, f"{e['id']} crosses {n['id']}"
        svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" role="img" aria-labelledby="title">',
               '<title id="title">DP-47 화면과 음성 동기화: 원본 공동 해석과 근거 보존 후보 교환</title>',
               '<rect width="2560" height="1440" fill="white"/>', '<defs>']
        for n,c in enumerate([BLACK, BLUE, GREEN]):
            svg.append(f'<marker id="a{n}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M1 1L9 5L1 9" fill="none" stroke="{c}" stroke-width="1.5"/></marker>')
        svg += ['</defs>', f'<g font-family="{FONT}">']
        mx = ET.Element('mxfile', host='app.diagrams.net', type='device')
        d = ET.SubElement(mx,'diagram',id='choice47-structure',name='DP-47 MAIN')
        m = ET.SubElement(d,'mxGraphModel',page='1',pageWidth=str(W),pageHeight=str(H),background='#FFFFFF')
        r = ET.SubElement(m,'root')
        ET.SubElement(r,'mxCell',id='0'); ET.SubElement(r,'mxCell',id='1',parent='0')
        for i in self.items:
            k, ident, color = i['kind'],i['id'],i['color']
            if k == 'edge':
                assert i['source'] in byid and i['target'] in byid
                pts=i['points']
                assert all(x1==x2 or y1==y2 for (x1,y1),(x2,y2) in zip(pts,pts[1:]))
                assert all(0<=x<=W and 0<=y<=H for x,y in pts)
                # Proper endpoints are editable node-bound mxCell edges.
                for endpoint,node in [(pts[0],byid[i['source']]),(pts[-1],byid[i['target']])]:
                    x,y=endpoint
                    assert node['x']<=x<=node['x']+node['w'] and node['y']<=y<=node['y']+node['h'], (ident,endpoint,node['id'])
                idx=[BLACK,BLUE,GREEN].index(color)
                svg.append(f'<polyline points="{" ".join(f"{x},{y}" for x,y in pts)}" fill="none" stroke="{color}" stroke-width="2" marker-end="url(#a{idx})"'+(' stroke-dasharray="7 6"' if i['dashed'] else '')+'/>')
                sn,tn=byid[i['source']],byid[i['target']]
                sx,sy=(pts[0][0]-sn['x'])/sn['w'],(pts[0][1]-sn['y'])/sn['h']
                tx,ty=(pts[-1][0]-tn['x'])/tn['w'],(pts[-1][1]-tn['y'])/tn['h']
                st=f'edgeStyle=none;rounded=0;strokeColor={color};strokeWidth=2;endArrow=open;endFill=0;exitX={sx};exitY={sy};exitPerimeter=0;entryX={tx};entryY={ty};entryPerimeter=0;'+('dashed=1;' if i['dashed'] else '')
                c=ET.SubElement(r,'mxCell',id=ident,edge='1',parent='1',source=i['source'],target=i['target'],style=st)
                g=ET.SubElement(c,'mxGeometry',relative='1',attrib={'as':'geometry'})
                arr=ET.SubElement(g,'Array',attrib={'as':'points'})
                for x,y in pts[1:-1]: ET.SubElement(arr,'mxPoint',x=str(x),y=str(y))
                continue
            x,y,w,h=(i[z] for z in ['x','y','w','h'])
            assert 0<=x and 0<=y and x+w<=W and y+h<=H,ident
            parent=i.get('owner') or '1'
            if parent!='1':
                o=byid[parent]
                assert o['kind']=='component'
                assert o['x']<=x and o['y']<=y and x+w<=o['x']+o['w'] and y+h<=o['y']+o['h'],ident
            size=i['size']
            if k=='text':
                st=f'text;html=1;align=left;verticalAlign=top;spacing=0;whiteSpace=nowrap;overflow=visible;fillColor=none;strokeColor=none;fontStyle={int(i["bold"])};'
                for j,line in enumerate(i['lines']):
                    svg.append(f'<text x="{x}" y="{y+size+j*size*1.3}" data-width="{w}" font-size="{size}" font-weight="{700 if i["bold"] else 400}" fill="{color}">{html.escape(line)}</text>')
            else:
                st=f'fillColor=#ffffff;strokeColor={color};strokeWidth=2;'
                if k=='external':
                    svg.append(f'<polygon points="{x+18},{y} {x+w-18},{y} {x+w},{y+h/2} {x+w-18},{y+h} {x+18},{y+h} {x},{y+h/2}" fill="white" stroke="{color}" stroke-width="2"/>')
                    st+='shape=hexagon;'
                elif k=='state':
                    svg.append(f'<path d="M{x},{y+9}C{x},{y-3} {x+w},{y-3} {x+w},{y+9}V{y+h-9}C{x+w},{y+h+3} {x},{y+h+3} {x},{y+h-9}Z M{x},{y+9}C{x},{y+21} {x+w},{y+21} {x+w},{y+9}" fill="white" stroke="{color}" stroke-width="2"/>')
                    st+='shape=cylinder;size=9;'
                elif k=='data':
                    svg.append(f'<path d="M{x},{y}H{x+w-18}L{x+w},{y+18}V{y+h}H{x}Z M{x+w-18},{y}V{y+18}H{x+w}" fill="white" stroke="{color}" stroke-width="2"/>')
                    st+='shape=note;size=18;'
                else:
                    svg.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{10 if k=="module" else 0}" fill="white" stroke="{color}" stroke-width="2"/>')
                    st+=f'rounded={int(k=="module")};arcSize=12;'
                component=k=='component'
                yy=y+32 if component else y+(h-len(i['lines'])*size*1.3)/2+size
                for j,line in enumerate(i['lines']):
                    svg.append(f'<text x="{x+16 if component else x+w/2}" y="{yy+j*size*1.3}" text-anchor="{"start" if component else "middle"}" data-width="{w-32}" font-size="{size}" font-weight="{700 if component else 400}" fill="{color}">{html.escape(line)}</text>')
                st+=f'align={"left" if component else "center"};verticalAlign={"top" if component else "middle"};spacingTop={8 if component else 0};spacingLeft={16 if component else 0};fontStyle={int(component)};'
            c=ET.SubElement(r,'mxCell',id=ident,parent=parent,vertex='1',value='<br>'.join(html.escape(t) for t in i['lines']),style=st+f'html=1;fontSize={size};fontColor={color};fontFamily=Apple SD Gothic Neo;')
            if parent!='1': x,y=x-byid[parent]['x'],y-byid[parent]['y']
            ET.SubElement(c,'mxGeometry',x=str(x),y=str(y),width=str(w),height=str(h),attrib={'as':'geometry'})
        svg+=['</g></svg>']
        out={'svg':'\n'.join(svg)+'\n','drawio':ET.tostring(mx,encoding='unicode')+'\n'}
        for content in out.values(): ET.fromstring(content)
        return out


def scene():
    s=Scene()
    s.text(40,18,2480,['DP-47 / 화면이 바뀌어도 말과 손동작이 지정한 대상을 연결하고 일부 정정을 이어간다'],36,bold=True)
    s.text(40,76,2470,['선택한 매출 범위 → PDF 표 지칭 → scrollbar drag로 페이지 이동 → 버튼 없는 원형 지칭 → “아니, 아래 graph만”'],24)
    s.text(40,114,2470,['전사는 늦게 도착한다. foreground, pointer 대상은 다를 수 있다. 당시 frame, path, selection, 시간 오차를 양안 모두 보존한다.'],22)
    s.box('audio',45,220,260,70,'사용자 음성','external')
    s.box('os',45,335,260,70,'Windows 관측','external')
    s.box('im',365,210,900,210,'Interaction Manager')
    s.box('io',390,270,235,60,'Channel I/O','module','im',size=22)
    s.box('vad',690,270,240,60,'Turn-Taking Control','module','im',size=20)
    s.box('capture',390,350,270,50,'Evidence Capture','module','im',size=22)
    s.box('timeline',975,285,270,90,'Timeline & Buffer','module','im',size=22)
    s.box('ma_voice',1380,210,450,210,'Model Access')
    s.box('voice_client',1410,285,390,70,'Voice API Client','module','ma_voice')
    s.box('cloud_voice',1940,215,560,85,'클라우드 음성 모델','external')
    s.box('rc_in',1940,350,560,95,'Request Controller')
    s.text(1970,391,500,['입력 등록, revision / 동일 owner의 입력 책임'],21)
    s.edge('audio_in','audio','io',[(305,255),(335,255),(335,300),(390,300)])
    s.text(45,180,280,['1 원음, sample 시각'],19)
    s.edge('capture_in','os','capture',[(305,370),(390,370)])
    s.text(50,410,280,['1 frame, pointer, 선택, 앱 상태'],18)
    s.edge('vad_in','io','vad',[(625,300),(690,300)])
    s.edge('vad_timeline','vad','timeline',[(930,300),(975,300)])
    s.edge('audio_record','io','timeline',[(610,330),(610,340),(945,340),(945,330),(975,330)])
    s.edge('capture_timeline','capture','timeline',[(660,375),(890,375),(890,350),(975,350)])
    s.text(675,381,290,['1 당시 관측, 누락, 시간 기준'],18)
    s.edge('audio_cloud','io','voice_client',[(625,285),(660,285),(660,195),(1605,195),(1605,285)])
    s.text(825,152,520,['2 음성 전송 / capture와 의미 inference는 병행'],19)
    s.edge('voice_req','voice_client','cloud_voice',[(1800,307),(1880,307),(1880,255),(1940,255)])
    s.text(1860,306,630,['2 audio → voice API / 전사 revision ←'],19)
    s.edge('voice_ret','cloud_voice','voice_client',[(2180,300),(2180,335),(1870,335),(1870,340),(1800,340)],dashed=True)
    s.edge('transcript','voice_client','timeline',[(1605,355),(1605,450),(1110,450),(1110,375)],dashed=True)
    s.text(1130,422,460,['2 전사, audio 대응 / span 제공 범위만'],18)
    s.edge('input_registration','timeline','rc_in',[(1245,360),(1310,360),(1310,478),(2220,478),(2220,445)])
    s.text(1410,480,750,['3 GroundingInput: 원음/전사, 관측 참조, 이전 binding, 권한, 예산'],21)
    s.text(45,515,1170,['A  원본 공동 해석'],29,BLUE,True)
    s.text(1300,515,1220,['B  근거를 보존하는 후보 교환과 결합'],29,GREEN,True)
    s.box('ri_a',450,565,770,475,'Request Interpreter')
    s.box('a_input',480,630,200,90,'GroundingInput','data','ri_a',size=21)
    s.box('joint',730,635,445,90,'Joint Grounding Interpreter','module','ri_a',BLUE,24)
    s.box('a_state',730,865,445,90,'Joint Grounding State','state','ri_a',BLUE)
    s.box('a_proposal',480,865,205,90,'Grounding\nProposal','data','ri_a')
    s.box('ma_a',70,835,340,110,'Model Access')
    s.text(90,889,300,['동일 cloud semantic client'],20)
    s.edge('input_a','rc_in','a_input',[(2040,445),(2040,558),(705,558),(705,615),(580,615),(580,630)])
    s.edge('a_read','a_input','joint',[(680,675),(730,675)],color=BLUE)
    s.text(710,601,440,['4 필요한 범위 원본 read'],19,BLUE)
    s.edge('a_call','joint','ma_a',[(815,725),(815,795),(240,795),(240,835)],color=BLUE)
    s.text(80,728,570,['5 원음/전사, frame/path/native 공동 해석'],20,BLUE)
    s.edge('a_return','ma_a','joint',[(410,915),(430,915),(430,755),(750,755),(750,725)],dashed=True,color=BLUE)
    s.edge('a_state_write','joint','a_state',[(1000,725),(1000,865)],color=BLUE)
    s.text(1015,773,200,['6 근거, revision','필요한 관계 patch'],19,BLUE)
    s.edge('a_state_read','a_state','joint',[(920,865),(920,725)],dashed=True,color=BLUE)
    s.edge('a_produce','joint','a_proposal',[(730,680),(705,680),(705,910),(685,910)],color=BLUE)
    s.text(480,819,220,['7 지칭, 교체 제안'],20,BLUE)
    s.text(70,982,480,['직접 target 생산 / 선택적 read, patch'],20,BLUE)
    s.text(730,982,480,['전체 단서 결합 vs media, 재판단 비용'],20,BLUE)
    s.box('ri_b',1650,565,870,475,'Request Interpreter')
    s.box('b_input',1330,640,280,95,'GroundingInput','data',size=22)
    s.box('coord',1675,735,350,70,'Grounding Coordinator','module','ri_b',GREEN,23)
    s.box('speech',1675,625,350,75,'Speech Reference Producer','module','ri_b',GREEN,23)
    s.box('screen',1675,820,350,75,'Screen Target Producer','module','ri_b',GREEN,23)
    s.box('candidates',2105,625,385,95,'Candidate Exchange State','state','ri_b',GREEN)
    s.box('resolver',2105,820,385,85,'Multimodal Binding Resolver','module','ri_b',GREEN,23)
    s.box('b_binding',2105,960,385,65,'Binding Revision State','state','ri_b',GREEN,22)
    s.box('ma_b',1330,910,265,100,'Model Access')
    s.text(1348,954,232,['동일 의미 호출 경로'],20)
    s.edge('input_b','rc_in','b_input',[(2380,445),(2380,551),(1470,551),(1470,640)])
    s.edge('b_read','b_input','coord',[(1610,685),(1640,685),(1640,770),(1675,770)],color=GREEN)
    s.edge('speech_start','coord','speech',[(1840,735),(1840,700)],color=GREEN)
    s.text(1330,752,305,['4 원음/전사, 관측 read','양쪽 생산 요청'],19,GREEN)
    s.edge('screen_start','coord','screen',[(1840,805),(1840,820)],color=GREEN)
    s.edge('speech_candidates','speech','candidates',[(2025,662),(2105,662)],color=GREEN)
    s.text(2095,590,410,['5 후보 ID, 근거, coverage, revision'],19,GREEN)
    s.edge('screen_candidates','screen','candidates',[(2025,847),(2065,847),(2065,693),(2105,693)],color=GREEN)
    s.edge('candidate_read','candidates','coord',[(2250,720),(2250,732),(2060,732),(2060,750),(2025,750)],dashed=True,color=GREEN)
    s.edge('fusion_input','coord','resolver',[(2025,792),(2180,792),(2180,820)],color=GREEN)
    s.text(2240,790,250,['6 후보, 근거 결합'],19,GREEN)
    s.edge('feedback','resolver','coord',[(2105,885),(2080,885),(2080,770),(2025,770)],dashed=True,color=GREEN)
    s.text(2098,765,405,['6 누락 → 원 생산자 재판단'],18,GREEN)
    s.edge('speech_call','speech','ma_b',[(1675,655),(1635,655),(1635,885),(1462,885),(1462,910)],color=GREEN)
    s.edge('speech_result','ma_b','speech',[(1595,925),(1615,925),(1615,680),(1675,680)],dashed=True,color=GREEN)
    s.edge('screen_call','screen','ma_b',[(1675,855),(1625,855),(1625,935),(1595,935)],color=GREEN)
    s.edge('screen_result','ma_b','screen',[(1595,945),(1615,945),(1615,875),(1675,875)],dashed=True,color=GREEN)
    s.edge('resolver_call','resolver','ma_b',[(2145,905),(2145,945),(1630,945),(1630,960),(1595,960)],color=GREEN)
    s.edge('resolver_result','ma_b','resolver',[(1595,985),(1645,985),(1645,915),(2200,915),(2200,905)],dashed=True,color=GREEN)
    s.text(1750,914,350,['5 / 6 필요 시 의미 판단'],18,GREEN)
    s.edge('b_binding_write','resolver','b_binding',[(2360,905),(2360,960)],color=GREEN)
    s.text(2380,916,120,['7 제안'],18,GREEN)
    s.text(1330,1045,275,['새 target은 producer가 생산'],18,GREEN)
    s.text(1670,1045,555,['후보 재사용, 추적 vs 누락, 불일치, 왕복'],19,GREEN)
    s.box('semantic',970,1090,550,85,'클라우드 의미 모델','external')
    s.edge('a_semantic','ma_a','semantic',[(70,900),(45,900),(45,1120),(970,1120)])
    s.edge('a_semantic_ret','semantic','ma_a',[(970,1145),(25,1145),(25,930),(70,930)],dashed=True)
    s.text(430,1080,530,['5 joint 입력 → / 의미 결과 ←'],19)
    s.edge('b_semantic','ma_b','semantic',[(1595,960),(1620,960),(1620,1120),(1520,1120)])
    s.edge('b_semantic_ret','semantic','ma_b',[(1520,1145),(1640,1145),(1640,990),(1595,990)],dashed=True)
    s.text(1680,1100,810,['5 / 6 producer, resolver 입력 → / 의미 결과 ←'],19)
    s.box('rc_adopt',430,1230,430,90,'Request Controller')
    s.text(450,1270,390,['동일 owner / 최신성, 권한, 참조 검사'],20)
    s.box('task',1050,1230,310,90,'Task Manager')
    s.box('gateway',1550,1230,310,90,'Agent Gateway')
    s.box('agent',2070,1230,430,90,'Downstream Agent','external')
    s.edge('a_adopt','a_proposal','rc_adopt',[(582,955),(582,1195),(560,1195),(560,1230)],dashed=True)
    s.edge('b_adopt','b_binding','rc_adopt',[(2260,1025),(2260,1208),(720,1208),(720,1230)],dashed=True)
    s.text(1130,1175,1070,['7 공통 proposal: 대상, 집합, 근거, 유지/교체 / 미해결이면 확인 질문'],21)
    s.edge('task_handoff','rc_adopt','task',[(860,1275),(1050,1275)])
    s.text(865,1236,190,['후속 요청 처리'],18)
    s.edge('gateway_handoff','task','gateway',[(1360,1275),(1550,1275)])
    s.text(1365,1236,190,['유효 업무 명령'],18)
    s.edge('agent_handoff','gateway','agent',[(1860,1275),(2070,1275)])
    s.text(1870,1236,200,['지정 자료, 업무 위임'],18)
    s.text(40,1340,2490,['공통: 입력 계속 수신, 늦은 결과 무효화, 원본 예산, 권한 철회, 불명확한 지칭 확인. 외부 업무 reasoning/Action은 Downstream Agent 책임.'],21)
    s.text(40,1380,2490,['작성자 검토용 SW 설계 / 양안 동일 cloud 모델 조건 / 알고리즘, 실제 API 지원, 품질 우위 미검증 / process, 모델 수 또는 승자 비교 아님'],20)
    return s


def check_figure_registration():
    main_path = OUT.parent / '04-47-screen-voice-grounding.md'
    body = main_path.read_text()
    embedded = set(re.findall(r'!\[[^\]]*\]\(\./diagrams/([^\s)]+\.svg)\)', body))
    figures = {path.name for pattern in ('choice47-*.svg', 'dp47-*.svg', 'screen-voice-grounding-*.svg')
               for path in OUT.glob(pattern)}
    missing = figures - embedded
    assert not missing, f'DP-47 figures missing from main document: {sorted(missing)}'
    for name in figures:
        source = Path(name).with_suffix('.drawio').name
        assert (OUT / source).exists(), f'Missing editable source: {source}'
        assert f'./diagrams/{source}' in body, f'Missing source link in main document: {source}'
    return len(figures)


def main():
    p=argparse.ArgumentParser(); p.add_argument('--check',action='store_true'); args=p.parse_args()
    content=scene().render()
    for ext,value in content.items():
        path=OUT/f'choice47-structure.{ext}'
        if args.check:
            assert path.read_text()==value, f'Out of sync: {path}'
        else:
            path.write_text(value)
    if args.check:
        count = check_figure_registration()
        print(f'PASS: all {count} DP-47 figures embedded in main document with editable-source links')
    print('PASS: DP-47 SVG/draw.io; deterministic source, XML, bounds, ownership, IDs and edge endpoints')


if __name__=='__main__': main()
