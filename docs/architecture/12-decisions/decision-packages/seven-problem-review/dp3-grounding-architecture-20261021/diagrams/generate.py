#!/usr/bin/env python3
"""Generate the editable SVG/draw.io comparison from one coordinate model.

No network access, model execution, or dependencies are needed.
"""
from pathlib import Path
from html import escape
import argparse
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent
W, H = 2420, 1910
BLACK, BLUE, GREEN = "#243247", "#2463b9", "#13724c"
nodes, edges, labels = [], [], []


def node(id, name, x, y, w, h, kind="component", color=BLACK, fill="#ffffff"):
    nodes.append(dict(id=id, name=name, x=x, y=y, w=w, h=h,
                      kind=kind, color=color, fill=fill))


def label(id, text, x, y, w=300, size=22, color=BLACK, align="center", bold=False):
    labels.append(dict(id=id, text=text, x=x, y=y, w=w, size=size,
                       color=color, align=align, bold=bold))


def edge(id, points, color=BLACK, dashed=False, arrow=True):
    edges.append(dict(id=id, points=points, color=color, dashed=dashed, arrow=arrow))


label("title", "정확한 전사와 시간 정보로 ‘이거·저거’를 당시 화면에 연결", 60, 54, 2300, 34, bold=True, align="left")
label("legend", "검정: 공통   |   파랑: A의 처리 경로   |   초록: B의 처리 경로   |   실선: 요청·전달   |   점선 화살표: 결과 반환", 60, 87, 2300, 21, align="left")
node("common", "공통 입력 근거 생산과 요청 판별", 40, 125, 2320, 350, "group", fill="#f8fafc")
node("speech", "음성 입력", 80, 205, 220, 70, "external")
node("asr", "Streaming ASR", 420, 205, 250, 70, "external")
node("time", "시간 정보 생산기", 820, 205, 270, 70)
node("screen", "화면·마우스 입력", 80, 355, 240, 70, "external")
node("recorder", "입력 근거 기록기", 520, 355, 300, 70)
node("store_top", "입력 근거 저장소", 1240, 335, 320, 100, "store")
node("s2s", "클라우드 S2S", 1240, 205, 320, 70, "external")
node("gate", "화면 근거 필요 판별기", 1790, 205, 350, 70)
label("asrplace", "로컬", 420, 197, 250, 18)
label("speech_owner", "ASR·시간 정보 생산기: Streaming 음성 근거 생산자의 내부 구성", 80, 455, 1090, 17, align="left")
label("gate_owner", "요청 해석 제어기의 내부 기능", 1790, 180, 350, 17)
label("storetopram", "RAM 원본 버퍼 · 시각·판본 보존", 1200, 462, 400, 18)
edge("speech_asr", [(300, 240), (420, 240)])
label("speech_asr_label", "음성", 315, 211, 90, 19)
edge("asr_time", [(670, 240), (820, 240)], dashed=True)
label("asr_time_label", "전사", 680, 211, 130, 19)
edge("time_store", [(1090, 240), (1150, 240), (1150, 385), (1240, 385)])
label("time_store_label", "전사·단어 시각", 1025, 289, 245, 19)
edge("screen_record", [(320, 390), (520, 390)])
label("screen_record_label", "원본·이벤트", 330, 360, 180, 19)
edge("speech_record", [(190, 275), (190, 312), (670, 312), (670, 355)])
label("speech_record_label", "원음 보존", 360, 282, 180, 19)
edge("record_store", [(820, 390), (1100, 390), (1100, 410), (1240, 410)])
label("record_store_label", "화면·포인터·선택·스크롤", 825, 355, 285, 19)
edge("speech_s2s", [(190, 205), (190, 174), (1400, 174), (1400, 205)])
label("speech_s2s_label", "음성 입력", 1000, 146, 180, 19)
edge("s2s_gate", [(1560, 240), (1790, 240)], dashed=True)
label("s2s_gate_label", "S2S 전사·정제 텍스트", 1555, 288, 245, 18)
edge("gate_no", [(2140, 240), (2305, 240)])
label("gate_no_label", "불필요", 2165, 211, 135, 19)
label("ordinary", "일반 요청 처리", 2140, 280, 210, 20)
label("gate_context", "기존 대화·요청 문맥을 함께 사용", 1750, 350, 450, 19)
label("common_note", "발화 중: 수집·ASR·시간 정보 생산\n화면 근거가 필요한 요청: 아래 의미 해석 시작", 1650, 395, 640, 19)

node("panel_a", "A  원본 근거 직접 해석", 40, 560, 1120, 1240, "panel", BLUE, "#f7faff")
node("panel_b", "B  대상·동작 생산 후 지칭 결합", 1200, 560, 1160, 1240, "panel", GREEN, "#f7fcf9")
node("local_a", "VIA 로컬 Process", 80, 620, 650, 1110, "group", BLACK)
node("local_b", "VIA 로컬 Process", 1240, 620, 650, 1110, "group", BLACK)
label("cloud_a", "클라우드 모델 의존성", 785, 685, 330, 21)
label("cloud_b", "클라우드 모델 의존성", 1950, 685, 350, 21)

node("a_store", "입력 근거 저장소", 250, 680, 360, 100, "store")
node("a_reader", "근거 조회기", 250, 910, 360, 90, color=BLUE, fill="#eaf2ff")
node("a_interpreter", "직접 지칭 해석기", 250, 1370, 360, 100, color=BLUE, fill="#eaf2ff")
node("a_model", "공통 의미 모델", 830, 1370, 260, 100, "external")
node("a_adopt", "채택·응답 처리", 250, 1605, 360, 90)
label("a_storeram", "RAM 원본 버퍼", 250, 810, 360, 18)
edge("a_read_request", [(490, 910), (490, 780)], BLUE)
label("a_read_request_label", "원본 조회", 482, 834, 165, 20, BLUE)
edge("a_read_return", [(370, 780), (370, 910)], BLUE, dashed=True)
label("a_read_return_label", "근거·판본", 177, 834, 180, 20, BLUE)
edge("a_expand", [(370, 1370), (370, 1000)], BLUE)
label("a_expand_label", "필요 구간·확대 요청", 93, 1190, 255, 21, BLUE)
edge("a_evidence", [(490, 1000), (490, 1370)], BLUE, dashed=True)
label("a_evidence_label", "원본 화면·동작 이력\n전사·시각·네이티브 사실", 507, 1125, 220, 19, BLUE)
edge("a_model_call", [(610, 1395), (830, 1395)], BLUE)
label("a_model_call_label", "근거로 직접 해석", 617, 1343, 210, 19, BLUE)
edge("a_model_result", [(830, 1445), (610, 1445)], BLUE, dashed=True)
label("a_model_result_label", "추가 조회 또는\n대상·의미 제안", 625, 1480, 200, 19, BLUE)
edge("a_adopt_request", [(430, 1470), (430, 1605)], BLUE)
label("a_adopt_label", "GroundedRequest 제안", 450, 1545, 275, 19, BLUE)
edge("a_out", [(430, 1695), (430, 1765)])
label("a_out_label", "채택된 GroundedRequest", 220, 1770, 420, 22, bold=True)

node("b_store", "입력 근거 저장소", 1370, 680, 360, 100, "store")
node("b_producer", "화면 대상 생산기", 1370, 910, 360, 90, color=GREEN, fill="#e7f5ed")
node("b_record", "대상·동작 기록", 1370, 1135, 360, 110, "store", GREEN, "#e7f5ed")
node("b_binder", "지칭 결합기", 1370, 1370, 360, 100, color=GREEN, fill="#e7f5ed")
node("b_models", "전문 검출·분할·추적\n모델", 1970, 910, 330, 90, "external", GREEN, "#e7f5ed")
node("b_model", "공통 의미 모델", 1970, 1370, 330, 100, "external")
node("b_adopt", "채택·응답 처리", 1370, 1605, 360, 90)
label("b_storeram", "RAM 원본 버퍼", 1370, 810, 360, 18)
edge("b_read_request", [(1610, 910), (1610, 780)], GREEN)
label("b_read_request_label", "원본 조회", 1620, 834, 160, 20, GREEN)
edge("b_read_return", [(1490, 780), (1490, 910)], GREEN, dashed=True)
label("b_read_return_label", "근거·판본", 1302, 834, 180, 20, GREEN)
edge("b_perception_call", [(1730, 935), (1970, 935)], GREEN)
label("b_perception_call_label", "원본 구간·영역", 1735, 882, 230, 19, GREEN)
edge("b_perception_return", [(1970, 980), (1730, 980)], GREEN, dashed=True)
label("b_perception_return_label", "대상 후보·마스크·추적", 1735, 1011, 230, 19, GREEN)
edge("b_write", [(1550, 1000), (1550, 1135)], GREEN)
label("b_write_label", "후보와 동작 관계 생산·갱신", 1570, 1057, 300, 19, GREEN)
label("b_record_content", "RAM: 객체·영역 ID\n시간별 추적·동작 관계\n근거·판본·불확실성", 1580, 1265, 290, 18, GREEN)
edge("b_record_read", [(1550, 1245), (1550, 1370)], GREEN, dashed=True)
label("b_record_read_label", "전사·시각\n원본 근거", 1370, 1315, 160, 19, GREEN)
edge("b_expand", [(1370, 1420), (1250, 1420), (1250, 1050), (1400, 1050), (1400, 1000)], GREEN)
label("b_expand_label", "후보 확장\n원본 재처리", 1257, 1175, 110, 18, GREEN)
edge("b_model_call", [(1730, 1395), (1970, 1395)], GREEN)
label("b_model_call_label", "전사·시각\n대상·관계·원본 근거", 1737, 1340, 225, 19, GREEN)
edge("b_model_return", [(1970, 1445), (1730, 1445)], GREEN, dashed=True)
label("b_model_return_label", "확장 요청 또는\n대상·의미 제안", 1740, 1480, 220, 19, GREEN)
edge("b_adopt_request", [(1550, 1470), (1550, 1605)], GREEN)
label("b_adopt_label", "GroundedRequest 제안", 1570, 1545, 300, 19, GREEN)
edge("b_out", [(1550, 1695), (1550, 1765)])
label("b_out_label", "채택된 GroundedRequest", 1340, 1770, 420, 22, bold=True)

edge("gate_branch", [(1965, 275), (1965, 300), (2320, 300), (2320, 520), (140, 520)], arrow=False)
edge("gate_a", [(140, 520), (140, 955), (250, 955)])
edge("gate_b", [(1300, 520), (1300, 955), (1370, 955)])
label("gate_yes_label", "화면 근거 필요 → 해당 기본안의 해석 경로 시작", 700, 507, 900, 22, bold=True)
label("a_start_label", "시작", 154, 922, 88, 19)
label("b_start_label", "시작", 1295, 920, 85, 19)
edge("adoption_a", [(95, 1575), (715, 1575)], dashed=True, arrow=False)
edge("adoption_b", [(1255, 1575), (1875, 1575)], dashed=True, arrow=False)
label("adoption_note_a", "최초 의미 확정 경계", 95, 1560, 320, 18, align="left")
label("adoption_note_b", "최초 의미 확정 경계", 1255, 1560, 320, 18, align="left")
label("foot1", "같은 입력 근거 저장소를 각 안에 반복 표시. 공통 의미 모델은 같은 사양·조건으로 비교하며, 두 안을 함께 실행한다는 뜻이 아님.", 60, 1830, 2290, 21, align="left")
label("foot2", "A는 여러 번 원본을 재조회할 수 있음. B는 A의 결과를 검사하지 않으며, 독립 영역 처리는 병렬화할 수 있음. 모델 우열·실측 결과를 나타낸 그림이 아님.", 60, 1870, 2290, 21, align="left")


def text_svg(text, x, y, width, size=22, color=BLACK, align="center", bold=False):
    lines = text.split("\n")
    anchor = "start" if align == "left" else "middle"
    xpos = x if align == "left" else x + width / 2
    line_h = size * 1.33
    weight = "700" if bold else "400"
    s = []
    for i, line in enumerate(lines):
        ypos = y + i * line_h
        s.append(f'<text x="{xpos:g}" y="{ypos:g}" text-anchor="{anchor}" fill="{color}" font-family="Apple SD Gothic Neo, Noto Sans KR, Malgun Gothic, sans-serif" font-size="{size}" font-weight="{weight}">{escape(line)}</text>')
    return ''.join(s)


def svg_text():
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" role="img" aria-labelledby="diagram-title diagram-desc">',
           '<title id="diagram-title">음성 화면 지칭: A 직접 해석과 B 대상 생산 후 결합</title>',
           '<desc id="diagram-desc">공통 전사와 시간 및 원본 화면 근거를 사용한다. A는 원본을 직접 해석하고 B는 전문 모델로 대상 동작 기록을 생산한 다음 지칭을 결합한다. 동일한 요청 의미 채택 경계로 연결된다.</desc>',
           '<style>text{font-family:"Apple SD Gothic Neo","Noto Sans KR","Malgun Gothic",sans-serif}</style>',
           f'<rect x="0" y="0" width="{W}" height="{H}" fill="#ffffff"/>']
    for n in nodes:
        if n['kind'] not in ("group", "panel"):
            continue
        x,y,w,h,c=n['x'],n['y'],n['w'],n['h'],n['color']
        dash=' stroke-dasharray="9 7"' if n['kind']=="group" else ''
        out.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="9" fill="{n["fill"]}" stroke="{c}" stroke-width="2"{dash}/>')
        out.append(text_svg(n['name'],x+20,y+35,w-40,25,c,"left",True))
    for e in edges:
        points=' '.join(f'{x},{y}' for x,y in e['points'])
        dash=' stroke-dasharray="9 6"' if e['dashed'] else ''
        out.append(f'<polyline points="{points}" fill="none" stroke="{e["color"]}" stroke-width="2.5" stroke-linejoin="round"{dash}/>')
    for n in nodes:
        if n['kind'] in ("group", "panel"):
            continue
        x,y,w,h,c=n['x'],n['y'],n['w'],n['h'],n['color']
        base=f' fill="{n["fill"]}" stroke="{c}" stroke-width="2.5"'
        if n['kind']=="external":
            pts=f'{x+20},{y} {x+w-20},{y} {x+w},{y+h/2} {x+w-20},{y+h} {x+20},{y+h} {x},{y+h/2}'
            out.append(f'<polygon points="{pts}"{base}/>')
        elif n['kind']=="store":
            out.append(f'<path d="M{x},{y+18} A{w/2},18 0 0 1 {x+w},{y+18} V{y+h-18} A{w/2},18 0 0 1 {x},{y+h-18} Z"{base}/>')
            out.append(f'<ellipse cx="{x+w/2}" cy="{y+18}" rx="{w/2}" ry="18"{base}/>')
        else:
            out.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="4"{base}/>')
        line_count = len(n['name'].split('\n'))
        first_baseline = y + h/2 + 24*.33 - (line_count-1)*24*1.33/2
        out.append(text_svg(n['name'],x+10,first_baseline,w-20,24,c,bold=True))
    for e in edges:
        if not e['arrow']:
            continue
        (x0,y0),(x1,y1)=e['points'][-2:]
        length=abs(x1-x0)+abs(y1-y0)
        ux,uy=(x1-x0)/length,(y1-y0)/length
        bx,by=x1-12*ux,y1-12*uy
        pts=f'{x1:g},{y1:g} {bx-6*uy:g},{by+6*ux:g} {bx+6*uy:g},{by-6*ux:g}'
        out.append(f'<polygon points="{pts}" fill="{e["color"]}"/>')
    for item in labels:
        out.append(text_svg(item['text'],item['x'],item['y'],item['w'],item['size'],item['color'],item['align'],item['bold']))
    out.append('</svg>\n')
    return '\n'.join(out)


def drawio_text():
    mxfile=ET.Element("mxfile",host="app.diagrams.net",type="device")
    diagram=ET.SubElement(mxfile,"diagram",id="dp3-grounding-comparison",name="A-B 구조 비교")
    model=ET.SubElement(diagram,"mxGraphModel",dx=str(W),dy=str(H),grid="1",gridSize="10",guides="1",tooltips="1",connect="1",arrows="1",fold="1",page="1",pageScale="1",pageWidth=str(W),pageHeight=str(H),math="0",shadow="0")
    root=ET.SubElement(model,"root")
    ET.SubElement(root,"mxCell",id="0")
    ET.SubElement(root,"mxCell",id="1",parent="0")
    for n in nodes:
        k=n['kind']
        style=f'whiteSpace=wrap;html=0;strokeColor={n["color"]};fillColor={n["fill"]};strokeWidth=2;fontColor={n["color"]};fontFamily=Arial;fontSize=24;fontStyle=1;'
        if k in ("group","panel"):
            style+='rounded=1;arcSize=2;verticalAlign=top;align=left;spacingTop=12;spacingLeft=18;'
            if k=="group": style+='dashed=1;dashPattern=9 7;'
        elif k=="store": style+='shape=cylinder3;boundedLbl=1;backgroundOutline=1;size=18;'
        elif k=="external": style+='shape=hexagon;perimeter=hexagonPerimeter2;size=20;fixedSize=1;'
        else: style+='rounded=0;'
        cell=ET.SubElement(root,"mxCell",id=n['id'],value=n['name'],style=style,vertex="1",parent="1")
        ET.SubElement(cell,"mxGeometry",x=str(n['x']),y=str(n['y']),width=str(n['w']),height=str(n['h']),**{"as":"geometry"})
    for e in edges:
        style=f'edgeStyle=none;rounded=0;html=0;strokeWidth=2.5;strokeColor={e["color"]};endArrow={"block" if e["arrow"] else "none"};endFill=1;'
        if e['dashed']: style+='dashed=1;dashPattern=9 6;'
        terminals={}
        for key, point, prefix in (("source",e['points'][0],"exit"),("target",e['points'][-1],"entry")):
            for n in nodes:
                if n['kind'] in ("group","panel"):
                    continue
                x,y=point
                if n['x']<=x<=n['x']+n['w'] and n['y']<=y<=n['y']+n['h'] and (x in (n['x'],n['x']+n['w']) or y in (n['y'],n['y']+n['h'])):
                    terminals[key]=n['id']
                    style+=f'{prefix}X={(x-n["x"])/n["w"]:g};{prefix}Y={(y-n["y"])/n["h"]:g};{prefix}Dx=0;{prefix}Dy=0;{prefix}Perimeter=0;'
                    break
        cell=ET.SubElement(root,"mxCell",id=e['id'],style=style,edge="1",parent="1",**terminals)
        g=ET.SubElement(cell,"mxGeometry",relative="1",**{"as":"geometry"})
        ET.SubElement(g,"mxPoint",x=str(e['points'][0][0]),y=str(e['points'][0][1]),**{"as":"sourcePoint"})
        ET.SubElement(g,"mxPoint",x=str(e['points'][-1][0]),y=str(e['points'][-1][1]),**{"as":"targetPoint"})
        if len(e['points'])>2:
            arr=ET.SubElement(g,"Array",**{"as":"points"})
            for x,y in e['points'][1:-1]: ET.SubElement(arr,"mxPoint",x=str(x),y=str(y))
    for l in labels:
        count=len(l['text'].split('\n'))
        style=f'text;html=0;strokeColor=none;fillColor=none;whiteSpace=wrap;align={l["align"]};verticalAlign=top;fontFamily=Arial;fontSize={l["size"]};fontColor={l["color"]};fontStyle={1 if l["bold"] else 0};spacing=0;'
        cell=ET.SubElement(root,"mxCell",id=l['id'],value=l['text'],style=style,vertex="1",parent="1")
        ET.SubElement(cell,"mxGeometry",x=str(l['x']),y=str(l['y']-l['size']),width=str(l['w']),height=str(l['size']*1.33*count+8),**{"as":"geometry"})
    ET.indent(mxfile,space="  ")
    return ET.tostring(mxfile,encoding="unicode")+'\n'


def validate():
    ids=[item['id'] for collection in (nodes,edges,labels) for item in collection]
    assert len(ids)==len(set(ids)), "Duplicate diagram IDs"
    for n in nodes:
        assert 0<=n['x'] and n['x']+n['w']<=W
        assert 0<=n['y'] and n['y']+n['h']<=H
    for e in edges:
        for (x1,y1),(x2,y2) in zip(e['points'],e['points'][1:]):
            assert x1==x2 or y1==y2, f"Non-orthogonal path: {e['id']}"
            for n in nodes:
                if n['kind'] in ("group","panel"):
                    continue
                interior_x=(n['x']+1,n['x']+n['w']-1)
                interior_y=(n['y']+1,n['y']+n['h']-1)
                horizontal_hit=y1==y2 and interior_y[0]<y1<interior_y[1] and max(min(x1,x2),interior_x[0])<min(max(x1,x2),interior_x[1])
                vertical_hit=x1==x2 and interior_x[0]<x1<interior_x[1] and max(min(y1,y2),interior_y[0])<min(max(y1,y2),interior_y[1])
                assert not(horizontal_hit or vertical_hit),f'Path crosses node: {e["id"]} / {n["id"]}'
        for x,y in e['points']: assert 0<=x<=W and 0<=y<=H
    ET.fromstring(svg_text())
    ET.fromstring(drawio_text())


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    validate()
    outputs={'comparison.svg':svg_text(),'comparison.drawio':drawio_text()}
    for name,content in outputs.items():
        dest=ROOT/name
        if args.check:
            assert dest.exists() and dest.read_text(encoding='utf-8')==content, f'Out of sync: {name}'
        else:
            dest.write_text(content,encoding='utf-8')
    print('Comparison SVG/draw.io: synchronized; XML, IDs, bounds, orthogonal paths and node crossings checked.')


if __name__=='__main__':
    main()
