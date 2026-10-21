#!/usr/bin/env python3
"""Generate paired editable CPU runtime/residency diagrams; no model execution."""
from pathlib import Path
from html import escape
import argparse
import xml.etree.ElementTree as ET
ROOT = Path(__file__).resolve().parent
W, H = 2820, 2130
BLACK, BLUE, GREEN = "#243247", "#2463b9", "#13724c"
nodes, edges, labels = [], [], []

def node(id, name, x, y, w, h, kind="component", color=BLACK, fill="#ffffff"):
    nodes.append(dict(id=id,name=name,x=x,y=y,w=w,h=h,kind=kind,color=color,fill=fill))

def label(id,text,x,y,w=300,size=22,color=BLACK,align="center",bold=False):
    labels.append(dict(id=id,text=text,x=x,y=y,w=w,size=size,color=color,align=align,bold=bold))

def edge(id,points,color=BLACK,dashed=False,arrow=True):
    edges.append(dict(id=id,points=points,color=color,dashed=dashed,arrow=arrow))

label("title","B의 CPU 모델 실행: 모델 수명과 process 경계를 분리해서 선택",50,56,2720,36,align="left",bold=True)
label("legend","검정: 공통 계약  |  초록: 비교하는 로컬 실행 구조  |  실선: 요청·호출  |  점선 화살표: 결과·복원  |  점선 경계: 실제 process",50,99,2720,22,align="left")
node("common","공통 실행 계약 — 아래 R1/R2/R3 중 하나로 CPU 추론 Worker를 배치",40,140,2740,450,"panel",BLACK,"#f8fafc")
node("core","VIA Core Process",70,195,1360,345,"group")
node("raw","원본 근거 저장소",110,295,330,100,"store")
node("producer","화면 대상 생산기",580,305,330,80)
node("control","모델 실행 제어기",1050,305,330,80)
node("workers","CPU 추론 Worker",1550,305,240,80,"external",GREEN,"#e7f5ed")
node("core_repeat","VIA Core (동일 Process)",1880,195,420,345,"group")
node("binder","지칭 결합기",1940,305,300,80)
node("cloud","클라우드 의미 LLM",2430,305,280,80,"external")
label("raw_scope","RAM 원본·판본·시각",110,425,330,18)
label("control_scope","작업큐·CPU 실행 허용·취소·재시작",985,425,460,18)
label("worker_scope","논리적 연결점\n실제 process 배치는 아래 비교",1460,425,420,18)
label("binder_scope","VIA 로컬",1940,285,300,18)
edge("raw_read",[(580,325),(440,325)])
label("raw_read_l","1 원본 조회",440,280,140,18)
edge("raw_return",[(440,365),(580,365)],dashed=True)
edge("produce_job",[(910,345),(1050,345)])
label("produce_job_l","2 생산 작업",910,290,140,18)
edge("call_worker",[(1380,325),(1550,325)])
label("call_worker_l","3 IPC 요청",1385,280,160,18)
edge("return_worker",[(1550,365),(1380,365)],dashed=True)
label("return_worker_l","4 결과·판본",1385,400,160,18)
edge("control_result",[(1215,385),(1215,485),(745,485),(745,385)],dashed=True)
label("control_result_l","5 후보·경계·추적 결과 반환",780,472,400,19)
edge("producer_binder",[(745,305),(745,245),(2090,245),(2090,305)])
label("producer_binder_l","6 후보·동작 관계·원본 crop + 동일 전사·timestamp",1110,229,800,20)
edge("binder_cloud",[(2240,325),(2430,325)])
label("binder_cloud_l","7 최종 해석",2245,280,180,18)
edge("cloud_return",[(2430,365),(2240,365)],dashed=True)
label("cloud_return_l","대상·요청 의미 제안",2240,421,190,18)
label("common_note","화면 근거가 필요할 때만 시작. Core는 worker 장애와 오래된 판본을 처리하고 원본을 보존한다. IPC는 파일/공유 메모리 참조와 작은 메시지를 전달한다.",90,565,2650,20,align="left")

for i,x in enumerate((40,960,1880),1):
    titles={1:"R1  공유 Worker · 한 모델 세션 교체",2:"R2  공유 Worker · 모델 세션 동시 상주",3:"R3  모델군별 전용 Worker 상주"}
    node(f"p{i}",titles[i],x,640,900,1390,"panel",GREEN,"#f7fcf9")
    label(f"route{i}","같은 3 IPC 요청 / 4 결과 반환 계약",x+30,719,840,20)

for i,x in ((1,40),(2,960)):
    node(f"w{i}","CPU Worker Process",x+40,810,820,860,"group",GREEN)
    node(f"adapter{i}","검출·OCR·SAM2 Adapter",x+90,920,720,80,color=GREEN,fill="#e7f5ed")
    node(f"rt{i}","PyTorch CPU Runtime",x+90,1080,720,80,color=GREEN,fill="#e7f5ed")
    edge(f"ipc{i}",[(x+330,755),(x+330,920)],GREEN)
    label(f"ipcl{i}","3 요청",x+180,780,140,19,GREEN)
    edge(f"ipcback{i}",[(x+590,920),(x+590,755)],GREEN,True)
    label(f"ipcbackl{i}","4 결과",x+605,780,140,19,GREEN)
    edge(f"call{i}",[(x+330,1000),(x+330,1080)],GREEN)
    edge(f"return{i}",[(x+590,1080),(x+590,1000)],GREEN,True)
    label(f"rtscope{i}","공유 thread pool · 기본 실행 허용 1건",x+90,1205,720,20,GREEN)
    node(f"state{i}","Episode 상태",x+485,1280,325,105,"store",GREEN,"#e7f5ed")
    edge(f"stateedge{i}",[(x+650,1160),(x+650,1280)],GREEN)
    label(f"stateedge_l{i}","요청별 생성·해제",x+465,1250,345,19,GREEN)
    node(f"disk{i}","모델 파일 저장소",x+135,1780,630,100,"store")
    label(f"disknote{i}","디스크 가중치 + 모델 구성 파일",x+135,1917,630,20)

node("session1","모델 세션 슬롯",130,1280,325,105,"store",GREEN,"#e7f5ed")
edge("sessionload1",[(295,1780),(295,1385)],GREEN)
label("sl1","모델 전환 시\n읽기·로드 / 해제",70,1550,220,21,GREEN)
edge("runtime_session1",[(295,1160),(295,1280)],GREEN)
label("slot1","검출 또는 OCR 또는 SAM2",125,1430,350,21,GREEN)
label("state1_l","RAM: embedding·mask·추적 상태\n모델 해제 후에도 호환성 확인 후 보존",500,1430,380,18,GREEN)
label("r1_note","Runtime는 유지한다. 가중치·실행 메모리를 해제해도\nallocator가 OS에 RAM을 즉시 반환한다고 보장하지 않는다.",85,1975,810,19,align="left")

node("session2a","검출·OCR 세션",1050,1280,325,105,"store",GREEN,"#e7f5ed")
node("session2b","SAM2 세션",1050,1480,325,105,"store",GREEN,"#e7f5ed")
edge("sessionload2",[(1215,1780),(1215,1585)],GREEN)
edge("session_ui_load2",[(1095,1830),(1000,1830),(1000,1330),(1050,1330)],GREEN)
edge("runtime_session2",[(1215,1160),(1215,1280)],GREEN)
edge("runtime_sam2",[(1050,1120),(1030,1120),(1030,1530),(1050,1530)],GREEN)
label("load2_l","최초 필요 시 로드\n모델 가중치 유지",1375,1585,390,20,GREEN)
label("state2_l","R1과 같은 Episode 보존 정책\nEpisode 종료·무효화 시 재계산",1425,1430,390,19,GREEN)
label("r2_note","가중치는 함께 상주하되 CPU 작업은 기본 직렬 실행.\n상주 ≠ 병렬. 실행 허용 정책은 별도 선택이다.",1005,1975,810,19,align="left")

# R3: two real worker processes, each with its own runtime and thread pool.
for suffix,dx,title,adapt,session in (("ui",1920,"UI Worker Process","검출·OCR Adapter","검출·OCR 세션"),("sam",2340,"SAM Worker Process","SAM2 Adapter","SAM2 세션")):
    node(f"w3_{suffix}",title,dx,810,400,860,"group",GREEN)
    node(f"a3_{suffix}",adapt,dx+25,920,350,80,color=GREEN,fill="#e7f5ed")
    node(f"rt3_{suffix}","PyTorch CPU Runtime",dx+25,1080,350,80,color=GREEN,fill="#e7f5ed")
    edge(f"ip3_{suffix}",[(dx+330,755),(dx+330,920)],GREEN)
    edge(f"ipr3_{suffix}",[(dx+370,920),(dx+370,755)],GREEN,True)
    label(f"ip3l_{suffix}","개별 IPC 큐",dx+25,785,260,19,GREEN)
    edge(f"call3_{suffix}",[(dx+125,1000),(dx+125,1080)],GREEN)
    edge(f"ret3_{suffix}",[(dx+275,1080),(dx+275,1000)],GREEN,True)
    label(f"pool3_{suffix}","개별 thread pool",dx+25,1205,350,20,GREEN)
    node(f"sess3_{suffix}",session,dx+25,1270,350,100,"store",GREEN,"#e7f5ed")
    node(f"state3_{suffix}","Episode 상태",dx+25,1490,350,100,"store",GREEN,"#e7f5ed")
    edge(f"rtsess3_{suffix}",[(dx+200,1160),(dx+200,1270)],GREEN)
    edge(f"st3_{suffix}",[(dx+200,1370),(dx+200,1490)],GREEN)
    label(f"state3l_{suffix}","요청별 생성·해제",dx+25,1435,350,19,GREEN)
node("disk3","모델 파일 저장소",2015,1780,630,100,"store")
edge("load3ui",[(2015,1830),(1900,1830),(1900,1320),(1945,1320)],GREEN)
edge("load3sam",[(2645,1830),(2770,1830),(2770,1320),(2715,1320)],GREEN)
label("disknote3","각 Worker 시작 시 자기 모델 로드",2015,1917,630,20)
label("r3_note","Core가 전체 CPU 실행량을 제한. 장애·재시작 경계 2개.\nprivate RAM 증가; 공유 library 페이지 전체를 중복 합산하지 않는다.",1925,1975,810,19,align="left")
label("footer1","R1/R2는 같은 Component·IPC·Episode 보존 정책에서 가중치 수명만 비교한다. R3는 process·큐·runtime·장애 경계가 달라진다. 모델 가중치 ≠ Episode 상태.",50,2070,2720,22,align="left",bold=True)
label("footer2","세 안은 대안이며 동시에 배치하지 않는다. 위의 논리 Worker 연결을 아래에서 각각 펼쳤다. 모든 추론은 CPU; 모델 크기·성능의 측정 결과를 나타내지 않는다.",50,2110,2720,20,align="left")

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
           '<title id="diagram-title">B의 CPU 모델 실행: 교체 상주, 동시 상주, worker 분리</title>',
           '<desc id="diagram-desc">위에는 VIA 로컬 Core와 CPU 추론 worker의 공통 실행 계약, 아래에는 R1 단일 세션 교체, R2 공유 worker 복수 세션 상주, R3 전용 worker 분리를 표시한다. 모델 가중치와 episode 상태를 구별하며 상주가 병렬 실행을 뜻하지 않는다.</desc>',
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
    diagram=ET.SubElement(mxfile,"diagram",id="dp3-runtime-residency",name="CPU 실행과 모델 상주 구조")
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
    outputs={'runtime-residency.svg':svg_text(),'runtime-residency.drawio':drawio_text()}
    for name,content in outputs.items():
        dest=ROOT/name
        if args.check:
            assert dest.exists() and dest.read_text(encoding='utf-8')==content, f'Out of sync: {name}'
        else:
            dest.write_text(content,encoding='utf-8')
    print('Runtime residency SVG/draw.io: synchronized; XML, IDs, bounds, orthogonal paths and node crossings checked.')


if __name__=='__main__':
    main()
