"""DP-42 implementation-design views. Generates documents, not VIA runtime code."""
from __future__ import annotations

import argparse
import hashlib
import html
import json
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/diagrams'
DOC = ROOT / 'docs/architecture/12-decisions/decision-packages/04-42-lifecycle-ownership.md'
W, H = 3840, 2160
INK, MUTED, LINE = '#233247', '#596A7C', '#C8D2DC'
ACCENT, PALE, RED = '#A65E2E', '#FFF2E4', '#A63A32'
FONT = 'Arial, Apple SD Gothic Neo, Malgun Gothic, sans-serif'


class Scene:
    def __init__(self, slug, title, subtitle):
        self.slug, self.title = slug, title
        self.nodes, self.edges, self.notes, self.bands = {}, [], [], []
        self.lifelines = {}
        self.text(64, 50, 'VIA / DP-42 / 구현을 가정한 상세 설계', 26, bold=True)
        self.text(3776, 50, '설계 제안 / 제품 구현과 측정 없음 / A/B 미선정', 24, align='right')
        self.text(64, 118, title, 44, bold=True)
        self.text(64, 175, subtitle, 27)

    def text(self, x, y, value, size=24, color=INK, bold=False, align='left', mask=False):
        self.notes.append(dict(x=x, y=y, text=value, size=size, color=color,
                               bold=bold, align=align, mask=mask))

    def node(self, key, x, y, w, h, name, kind='module', owner=None, different=False):
        assert key not in self.nodes
        self.nodes[key] = dict(x=x, y=y, w=w, h=h, name=name, kind=kind,
                               owner=owner, different=different)

    def band(self, x, y, w, h, title, dashed=False):
        self.bands.append(dict(x=x, y=y, w=w, h=h, title=title, dashed=dashed))

    def point(self, key, side, offset=0):
        n = self.nodes[key]; x, y, w, h = (n[k] for k in ('x', 'y', 'w', 'h'))
        return {'L':(x,y+h/2+offset),'R':(x+w,y+h/2+offset),
                'T':(x+w/2+offset,y),'B':(x+w/2+offset,y+h)}[side]

    def edge(self, src, dst, sp='R', tp='L', via=(), label='', at=None,
             ret=False, sd=0, td=0, accent=False):
        pts=[self.point(src,sp,sd),*via,self.point(dst,tp,td)]
        self.edges.append(dict(src=src,dst=dst,points=pts,ret=ret,accent=accent,sp=sp,tp=tp,sd=sd,td=td))
        if label:self.text(*at,label,21,ACCENT if accent else INK,mask=True)

    def ancestors(self,key):
        out=set()
        while self.nodes[key]['owner']:
            key=self.nodes[key]['owner'];out.add(key)
        return out

    def validate(self):
        for key,n in self.nodes.items():
            x,y,w,h=(n[k] for k in ('x','y','w','h'))
            assert 0<=x and 0<=y and x+w<=W and y+h<=H,(self.slug,key,'bounds')
            if n['owner']:
                p=self.nodes[n['owner']]
                assert p['x']<=x and p['y']+55<=y and x+w<=p['x']+p['w'] and y+h<=p['y']+p['h'],(key,'owner')
            elif n['kind']=='module':raise AssertionError((key,'module without Component'))
        for e in self.edges:
            exempt={e['src'],e['dst']}|self.ancestors(e['src'])|self.ancestors(e['dst'])
            for a,b in zip(e['points'],e['points'][1:]):
                assert a[0]==b[0] or a[1]==b[1],(self.slug,e['src'],e['dst'],'diagonal',a,b)
                for key,n in self.nodes.items():
                    if key in exempt or n['kind']=='component':continue
                    x,y,w,h=(n[k] for k in ('x','y','w','h'))
                    vertical=a[0]==b[0] and x+2<a[0]<x+w-2 and max(min(a[1],b[1]),y+2)<min(max(a[1],b[1]),y+h-2)
                    horizontal=a[1]==b[1] and y+2<a[1]<y+h-2 and max(min(a[0],b[0]),x+2)<min(max(a[0],b[0]),x+w-2)
                    assert not(vertical or horizontal),(self.slug,e['src'],e['dst'],'crosses',key)
        assert not any(c in self.svg() for c in '\u00b7\u2022\u2027\u318d\u30fb\u22c5\u2219')

    def node_text(self,n):
        x,y,w,h=(n[k] for k in ('x','y','w','h'))
        if n['kind']=='component':return [(x+18,y+37,n['name'],29,'left')]
        lines=n['name'].split('\n');size=24 if w<450 else 27
        top=y+h/2-(len(lines)-1)*17+8+(8 if n['kind']=='store' else 0)
        return [(x+w/2,top+j*34,t,size,'center') for j,t in enumerate(lines)]

    def svg(self):
        esc=html.escape
        out=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" role="img"><title>{esc(self.title)}</title><defs>']
        for color in (INK,ACCENT):
            out.append(f'<marker id="a{color[1:]}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto"><path d="M1 1L9 5L1 9" fill="none" stroke="{color}" stroke-width="1.5"/></marker>')
        out += ['</defs>',f'<rect width="{W}" height="{H}" fill="white"/>',f'<g font-family="{FONT}">']
        for b in self.bands:
            out.append(f'<rect x="{b["x"]}" y="{b["y"]}" width="{b["w"]}" height="{b["h"]}" fill="white" stroke="{ACCENT}" stroke-width="4"'+(' stroke-dasharray="16 10"' if b['dashed'] else '')+'/>')
            out.append(f'<text x="{b["x"]+18}" y="{b["y"]+35}" font-size="26" fill="{ACCENT}" font-weight="700">{esc(b["title"])}</text>')
        for key,(x,y,h) in self.lifelines.items():
            out.append(f'<path d="M{x} {y}V{y+h}" stroke="{LINE}" stroke-width="1.5" stroke-dasharray="8 8"/>')
        for key,n in self.nodes.items():
            x,y,w,h=(n[k] for k in ('x','y','w','h'));kind=n['kind']
            color=ACCENT if n['different'] else INK;fill=PALE if n['different'] else 'white'
            out.append(f'<g id="{key}" data-node="{kind}" data-x="{x}" data-y="{y}" data-w="{w}" data-h="{h}">')
            if kind=='store':
                out.append(f'<path d="M{x} {y+8}C{x} {y-3} {x+w} {y-3} {x+w} {y+8}V{y+h-8}C{x+w} {y+h+3} {x} {y+h+3} {x} {y+h-8}Z M{x} {y+8}C{x} {y+20} {x+w} {y+20} {x+w} {y+8}" fill="{fill}" stroke="{color}" stroke-width="2"/>')
            elif kind=='external':
                out.append(f'<path d="M{x+22} {y}H{x+w-22}L{x+w} {y+h/2}L{x+w-22} {y+h}H{x+22}L{x} {y+h/2}Z" fill="{fill}" stroke="{color}" stroke-width="2"/>')
            else:
                out.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{14 if kind=="module" else 0}" fill="{fill}" stroke="{color}" stroke-width="{2.6 if kind=="component" else 2}"/>')
                if kind=='component':out.append(f'<path d="M{x} {y+55}H{x+w}" stroke="{color}"/>')
            for tx,ty,t,size,align in self.node_text(n):
                out.append(f'<text x="{tx}" y="{ty}" font-size="{size}" text-anchor="{"middle" if align=="center" else "start"}" fill="{INK}" font-weight="600">{esc(t)}</text>')
            out.append('</g>')
        for j,e in enumerate(self.edges):
            color=ACCENT if e['accent'] else INK
            pts=' '.join(f'{x:g},{y:g}' for x,y in e['points'])
            out.append(f'<polyline id="edge{j}" points="{pts}" fill="none" stroke="{color}" stroke-width="2.4" marker-end="url(#a{color[1:]})"'+(' stroke-dasharray="8 6"' if e['ret'] else '')+'/>')
        for j,t in enumerate(self.notes):
            for i,line in enumerate(t['text'].split('\n')):
                out.append(f'<text id="note{j}-{i}" x="{t["x"]}" y="{t["y"]+i*t["size"]*1.45:g}" font-size="{t["size"]}" fill="{t["color"]}" text-anchor="{"end" if t["align"]=="right" else "middle" if t["align"]=="center" else "start"}" font-weight="{700 if t["bold"] else 400}"'+(' paint-order="stroke" stroke="white" stroke-width="9" stroke-linejoin="round"' if t['mask'] else '')+f'>{esc(line)}</text>')
        return '\n'.join(out+['</g></svg>'])+'\n'

    def drawio(self):
        root=ET.Element('mxfile',host='app.diagrams.net',type='device')
        diagram=ET.SubElement(root,'diagram',id=self.slug,name=self.title)
        graph=ET.SubElement(diagram,'mxGraphModel',page='1',pageWidth=str(W),pageHeight=str(H))
        cells=ET.SubElement(graph,'root');ET.SubElement(cells,'mxCell',id='0');ET.SubElement(cells,'mxCell',id='1',parent='0')
        def vertex(key,x,y,w,h,value,style,parent='1'):
            c=ET.SubElement(cells,'mxCell',id=key,value=value,style=style,vertex='1',parent=parent)
            ET.SubElement(c,'mxGeometry',x=str(x),y=str(y),width=str(w),height=str(h),attrib={'as':'geometry'})
        for i,b in enumerate(self.bands):
            vertex('boundary'+str(i),b['x'],b['y'],b['w'],b['h'],b['title'],f'fillColor=none;strokeColor={ACCENT};strokeWidth=4;dashed={int(b["dashed"])};verticalAlign=top;align=left;spacing=18;fontSize=26;fontColor={ACCENT};fontStyle=1;')
        for key,n in self.nodes.items():
            x,y,w,h=(n[k] for k in ('x','y','w','h'));kind=n['kind'];owner=n['owner'] or '1'
            if n['owner']:x-=self.nodes[owner]['x'];y-=self.nodes[owner]['y']
            shape={'store':'shape=cylinder;size=8;','external':'shape=hexagon;','module':'rounded=1;arcSize=14;','component':'shape=swimlane;startSize=55;horizontal=1;align=left;spacingLeft=18;'}.get(kind,'')
            style=shape+f'whiteSpace=wrap;html=0;fontFamily=Arial;fontSize={29 if kind=="component" else 24};fontStyle=1;fontColor={INK};fillColor={PALE if n["different"] else "#FFFFFF"};strokeColor={ACCENT if n["different"] else INK};strokeWidth=2;'
            vertex(key,x,y,w,h,n['name'],style,owner)
        for key,(x,y,h) in self.lifelines.items():
            vertex(key+'_line',x,y,1,h,'',f'shape=line;direction=south;strokeColor={LINE};dashed=1;strokeWidth=1.5;')
        for j,e in enumerate(self.edges):
            source=self.nodes[e['src']];target=self.nodes[e['dst']]
            src_id,dst_id=e['src'],e['dst']
            if self.lifelines:
                src_id+='_line';dst_id+='_line'
                sx,sy,sh=self.lifelines[e['src']];tx,ty,th=self.lifelines[e['dst']]
                source=dict(x=sx,y=sy,w=1,h=sh);target=dict(x=tx,y=ty,w=1,h=th)
            a,b=e['points'][0],e['points'][-1]
            color=ACCENT if e['accent'] else INK
            style=f'edgeStyle=none;rounded=0;endArrow=open;endFill=0;strokeColor={color};strokeWidth=2.4;dashed={int(e["ret"])};exitX={(a[0]-source["x"])/source["w"]:.6f};exitY={(a[1]-source["y"])/source["h"]:.6f};entryX={(b[0]-target["x"])/target["w"]:.6f};entryY={(b[1]-target["y"])/target["h"]:.6f};'
            c=ET.SubElement(cells,'mxCell',id='edge'+str(j),edge='1',parent='1',source=src_id,target=dst_id,style=style)
            g=ET.SubElement(c,'mxGeometry',relative='1',attrib={'as':'geometry'})
            ar=ET.SubElement(g,'Array',attrib={'as':'points'})
            for x,y in e['points'][1:-1]:ET.SubElement(ar,'mxPoint',x=str(x),y=str(y))
        for j,t in enumerate(self.notes):
            width=max(len(line)*t['size'] for line in t['text'].split('\n'))+24
            x=t['x']-width if t['align']=='right' else t['x']-width/2 if t['align']=='center' else t['x']
            vertex('note'+str(j),x,t['y']-t['size'],width,len(t['text'].split('\n'))*t['size']*1.45,t['text'],f'text;html=0;align={t["align"]};verticalAlign=top;whiteSpace=wrap;spacing=0;fontSize={t["size"]};fontColor={t["color"]};fontStyle={int(t["bold"])};fillColor={"#FFFFFF" if t["mask"] else "none"};strokeColor=none;')
        return ET.tostring(root,encoding='unicode')+'\n'


def architecture(side):
    a=side=='A'
    p=Scene('dp42-implementation-'+side.lower(),
            f'{side}. '+('하나의 Core에서 관련 owner 변경을 공동 확정' if a else '대화와 업무가 각각 확정하고 내구 메시지로 연결'),
            '같은 사례: Q1을 실제 제시한 뒤 “응, 상반기로” → 질문 답변 연결 → 외부 전달 확인 → 사용자에게 P2 제시')
    p.text(650,239,'공통 의존성: 위치는 논리적 연동 영역이며 신규 process 분리를 선택하지 않는다.',22,MUTED)
    for key,x,w,name,module in [('cm',650,650,'Context Manager','근거 조회기'),('pm',1420,650,'Policy Manager','현재 권한 조회기'),('ma',2190,650,'Model Access','모델 API 어댑터')]:
        p.node(key,x,270,w,240,name,'component')
        p.node(key+'_m',x+50,365,w-100,90,module,owner=key)
    p.node('model',3000,305,735,145,'클라우드 음성 / 의미 모델','external')
    p.edge('ma_m','model',label='조건부 모델 호출',at=(2820,345),sd=-25,td=7.5)
    p.edge('model','ma_m',sp='L',tp='R',ret=True,sd=40,td=7.5,label='결과 / 오류 / 사용량',at=(2820,481))
    if a:p.band(640,650,3136,1350,'VIA Core process')
    else:
        p.band(640,650,1510,1350,'대화 서비스 process',True)
        p.band(2300,650,1476,1350,'업무 서비스 process',True)
    p.node('im',64,730,480,870,'Interaction Manager','component')
    for key,y,title in [('input',820,'입력 수집기'),('stop',1050,'로컬 출력 제어기'),('output',1250,'출력 실행기'),('actual',1450,'실제 전달 확인기')]:
        p.node(key,104,y,400,80,title,owner='im')
    p.text(64,1660,'음성 / Text / 화면 입력\n발화 시작 즉시 로컬 stop\n업무 취소와 별도',25,MUTED)
    p.node('ri',720,730,570,350,'Request Interpreter','component')
    p.node('interpret',770,820,470,100,'요청 해석기',owner='ri')
    p.node('scratch',770,950,470,90,'해석 작업 상태 / RAM','store',owner='ri')
    p.node('rc',1410,730,680,870,'Request Controller','component')
    for key,y,name,diff in [('adopt',820,'요청 채택기',False),('intent',980,'관계 변경 준비기' if a else '명령 의도 기록기',True),('reflect',1140,'진행 사실 반영기',not a),('control',1320,'현재 입력 제어기',not a),('reconcile',1470,'재개 조정기' if a else '재연결 조정기',not a)]:
        p.node(key,1460,y,580,80,name,owner='rc',different=diff)
    p.node('rm',720,1150,570,450,'Response Manager','component')
    p.node('publish',770,1250,470,80,'게시 제어기',owner='rm')
    p.node('ledger',770,1450,470,80,'실제 전달 기록기',owner='rm')
    p.node('tm',2370,730,620,870,'Task Manager','component')
    for key,y,name in [('query',820,'업무 조회기'),('accept',980,'답변 접수기'),('gate',1160,'업무 접수 및 전송 게이트'),('apply',1420,'업무 사건 적용기')]:
        p.node(key,2420,y,520,90,name,owner='tm',different=(not a and key in ('accept','gate')))
    p.node('ag',3120,730,590,870,'Agent Gateway','component')
    for key,y,name in [('send',820,'명령 전송 및 확인'),('ingress',1160,'외부 사건 수신기'),('recover',1420,'전송 복구기')]:
        p.node(key,3170,y,490,90,name,owner='ag')
    p.node('agent',3165,550,500,90,'Downstream Agent','external')
    # Common resolution, input and publication paths.
    p.edge('input','adopt',sp='R',tp='T',via=[(585,860),(585,710),(1750,710)],td=0,label='입력 u2 / onset → hold',at=(100,690))
    p.edge('adopt','interpret',sp='L',tp='R',sd=-20,td=-30,label='해석 요청',at=(1250,831))
    p.edge('interpret','adopt',sp='R',tp='L',sd=20,td=30,ret=True,label='의미 제안',at=(1260,939))
    p.edge('interpret','cm_m',sp='T',tp='B',sd=115,td=25,via=[(1120,575),(1000,575)],label='bounded read',at=(735,570))
    p.edge('cm_m','interpret',sp='B',tp='T',sd=90,td=145,via=[(1065,600),(1150,600)],ret=True,label='근거 / revision',at=(1180,591))
    p.edge('interpret','ma_m',sp='T',tp='B',sd=185,td=-200,via=[(1190,540),(2315,540)],label='필요한 입력만 모델에 제공',at=(1670,534))
    p.edge('ma_m','interpret',sp='B',tp='T',sd=-120,td=225,via=[(2395,610),(1230,610)],ret=True,label='해석 결과 / 실패',at=(1650,602))
    p.edge('adopt','pm_m',sp='T',tp='B',sd=200,td=205,label='권한 확인',at=(1900,633))
    p.edge('pm_m','adopt',sp='B',tp='T',sd=265,td=260,ret=True,label='현재 revision',at=(2030,633))
    p.edge('interpret','scratch',sp='B',tp='T')
    p.edge('adopt','intent',sp='B',tp='T',label='현재 검사 뒤 채택',at=(1780,959))
    p.edge('input','stop',sp='B',tp='T',label='onset',at=(325,994))
    p.edge('stop','output',sp='B',tp='T',label='output epoch / stop',at=(114,1190))
    p.edge('output','actual',sp='B',tp='T',label='Text / audible range',at=(114,1390))
    p.edge('adopt','control',sp='L',tp='L',via=[(1435,860),(1435,1360)],label='onset 때 hold 요청',at=(1510,1275))
    p.edge('adopt','query',sd=0,td=-5,label='Task / 질문 후보 조회',at=(2110,841))
    p.edge('query','adopt',sp='L',tp='R',sd=25,td=30,ret=True,label='Q1 v7 / T1 / X1',at=(2110,930))
    p.edge('adopt','ledger',sp='L',tp='R',sd=30,via=[(1340,890),(1340,1490)],label='P1 실제 제시 조회',at=(1330,1113))
    p.edge('ledger','adopt',sp='R',tp='L',sd=25,td=45,via=[(1365,1515),(1365,905)],ret=True)
    p.edge('reflect','publish',sp='L',tp='R',via=[(1390,1180),(1390,1290)],label='외부 확인 사실로 P2 허용',at=(760,1123))
    p.edge('publish','output',sp='L',tp='R',label='release P2',at=(555,1269))
    p.edge('actual','ledger',ret=True,label='actual receipt',at=(551,1467))
    # The different handoff and the common work control/observation paths.
    p.edge('intent','accept',label='prepare_answer(K1)' if a else 'ReleaseAndReconcile + K1',at=(2100,997),td=-5,accent=True)
    if a:p.edge('accept','intent',sp='L',tp='R',sd=25,td=30,ret=True,accent=True,label='TaskWriteSet / conflict',at=(2100,1089))
    else:p.edge('accept','reflect',sp='L',tp='R',sd=25,via=[(2200,1050),(2200,1180)],ret=True,accent=True,label='AcceptanceReceipt / HELD',at=(2100,1119))
    p.edge('control','gate',via=[(2250,1360),(2250,1205)],label='hold / selective release' if a else 'Hold / session / sequence → ACK',at=(2110,1290),accent=True)
    p.edge('send','gate',sp='L',tp='R',sd=-20,td=-20,via=[(3030,845),(3030,1185)],label='claim(K1)',at=(3008,1040))
    p.edge('gate','send',label='CAS 결과',at=(3008,1090),sp='R',tp='L',via=[(3050,1205),(3050,865)],ret=True)
    p.edge('send','agent',sp='T',tp='B',sd=100,td=100,label='CAS 확정 뒤 send(K1)',at=(2990,699))
    p.edge('agent','ingress',sp='B',tp='R',sd=210,td=0,via=[(3625,690),(3745,690),(3745,1205)],ret=True,label='외부 receipt / event',at=(3380,1085))
    p.edge('ingress','apply',sp='L',tp='R',via=[(3090,1205),(3090,1465)],ret=True,label='inbox 저장 뒤 source event',at=(2970,1365))
    p.edge('apply','reflect',sp='L',tp='R',via=[(2220,1465),(2220,1180)],ret=True,accent=not a,label='확인 사실 / 원본 revision' if a else '내구 event / cursor / 재전달',at=(2105,1573))
    p.edge('recover','agent',sp='R',tp='R',via=[(3760,1465),(3760,595)],label='UNKNOWN은 외부 상태 확인',at=(3140,1650))
    p.edge('reconcile','query',sp='R',tp='L',td=40,via=[(2275,1510),(2275,905)],label='내구 상태 재조회' if a else 'same K1 조회 / cursor 재결합',at=(1980,1640),accent=not a)
    # Physical storage adapters. Semantic ownership stays in the owner Components.
    if a:
        p.node('ss',720,1730,2990,250,'State Store','component')
        p.node('tx',770,1810,2890,58,'트랜잭션 관리자',owner='ss',different=True)
        stores=[('requests',770,530,'요청 관계'),('work',1360,530,'업무와 질문'),('commands',1950,530,'명령과 gate'),('publication',2540,530,'실제 전달 원장'),('inbox',3130,530,'외부 사건 inbox')]
        for key,x,w,name in stores:p.node(key,x,1900,w,62,name,'store',owner='ss')
        for key,x,_,_ in stores:p.edge('tx',key,sp='B',tp='T',sd=x+265-2215)
        p.edge('rc','tx',sp='B',tp='T',td=-465,via=[(1750,1670),(1750,1670)],label='A1: owner별 변경 집합 + expected revisions',at=(1340,1710),accent=True)
        p.edge('tm','tx',sp='B',tp='T',td=465,label='A2: CAS / A3: 외부 fact',at=(2390,1710))
        p.edge('ag','tx',sp='B',tp='T',td=1200,label='I1: 외부 raw inbox',at=(3140,1710),ret=True)
        p.edge('rm','tx',sp='B',tp='T',td=-1210,label='A4: actual receipt',at=(760,1710),ret=True)
    else:
        for key,x,w,title in [('ds',720,1370,'State Store'),('ws',2370,1340,'State Store')]:
            p.node(key,x,1730,w,250,title,'component')
            p.node(key+'_tx',x+50,1810,w-100,58,'트랜잭션 관리자',owner=key)
        for key,x,w,name,owner in [('relations',770,360,'요청 관계','ds'),('outbox',1180,420,'의도 outbox','ds'),('receipts',1650,390,'전달 / cursor','ds'),('task',2420,360,'업무와 질문','ws'),('command',2830,410,'명령 / gate / inbox','ws'),('events',3290,370,'event outbox','ws')]:
            p.node(key,x,1900,w,62,name,'store',owner=owner)
            p.edge(owner+'_tx',key,sp='B',tp='T',sd=x+w/2-(1405 if owner=='ds' else 3040))
        p.edge('rc','ds_tx',sp='B',tp='T',td=345,label='D1 의도 / D2 접수 / D3 외부 확인',at=(1310,1710),accent=True)
        p.edge('rm','ds_tx',sp='B',tp='T',td=-400,label='D4 actual receipt',at=(760,1710),ret=True)
        p.edge('tm','ws_tx',sp='B',tp='T',td=-360,label='W1 접수 / W2 CAS / W3 fact',at=(2375,1710),accent=True)
        p.edge('ag','ws_tx',sp='B',tp='T',td=375,label='I1 외부 raw inbox',at=(3160,1710),ret=True)
    p.text(64,2050,'실선 → 호출 / 명령     점선 → 반환 / 확인 사실     둥근 상자 = 내부 Module     원통 = 내구 상태 (RAM은 별도 표시)',24)
    p.text(64,2092,'상태 저장선은 각 Component의 repository 접근이다. 저장소 안의 원통은 의미 owner를 바꾸지 않는다. 번호별 호출과 확정 순서는 다음 흐름도에서 펼친다.',24,MUTED)
    p.text(3776,2131,'A1 전체가 외부 send나 실제 출력까지 원자적으로 묶지는 않는다.' if a else '업무 W2는 대화 D2 반영을 기다리지 않는다. 외부 접수와 사용자 실제 전달은 별도 사실이다.',23,ACCENT,align='right')
    return p


def sequence(side):
    a=side=='A'
    p=Scene('dp42-flow-'+side.lower(),f'{side}. 같은 Q1 답변을 입력부터 실제 전달까지 추적',
            '초기: T1 / X1의 Q1=OPEN v7, P1 실제 제시 확인. u2에는 Task ID가 없으며 읽기 근거를 본 뒤 의미를 채택한다.')
    names=[('im','Interaction Manager'),('rc','Request Controller'),('ri','Request Interpreter'),('tm','Task Manager')]
    names += [('ss','State Store')] if a else [('ds','State Store\n대화'),('ws','State Store\n업무')]
    names += [('ag','Agent Gateway'),('agent','Downstream Agent'),('rm','Response Manager')]
    xs={k:180+i*(3480/(len(names)-1)) for i,(k,_) in enumerate(names)}
    for key,name in names:
        p.node(key,xs[key]-155,260,310,104,name,'external' if key=='agent' else 'participant')
        # Lifelines are background guides, not a separate executable Module.
        p.lifelines[key]=(xs[key],382,1430)
    rows=[]
    def row(src,dst,label,ret=False,commit=False):rows.append((src,dst,label,ret,commit))
    row('im','rc','u2 onset / 새 입력 → 보호 hold 요청')
    row('rc','tm','hold 요청 / 로컬 gate' if a else 'Hold 요청 / control session + sequence')
    row('tm','rc','HoldAck: 적용한 gate revision / 영향 범위 / 충돌',True)
    row('rc','tm','질문 / Task 후보 bounded 조회')
    row('tm','rc','Q1=OPEN v7 / T1 / X1 / source revision',True)
    row('rc','rm','P1 실제 제시 범위 조회')
    row('rm','rc','P1 → Q1 actual reference',True)
    row('rc','ri','원문 u2 + 현재 근거 / 필요한 추가 조회와 모델 호출')
    row('ri','rc','MeaningProposal: 상반기 답변 → Q1 / T1 (아직 미채택)',True)
    if a:
        row('rc','tm','prepare_answer(K1, Q1 v7, input / permission revision)')
        row('tm','rc','TaskWriteSet: Q1 v8 + K1 PENDING + 선택적 hold 해제',True)
        row('rc','ss','A1: RequestWriteSet + TaskWriteSet / expected revisions 검사',commit=True)
        row('ss','rc','COMMITTED: R2 관계 + Q1 + K1 / 충돌이면 모두 미반영',True)
        row('ag','tm','claim(K1): 현재 gate / Task / 권한 조건 검사')
        row('tm','ss','A2: PENDING → DISPATCHING 조건부 CAS',commit=True)
        row('ss','ag','claim 성공 / 실패면 외부 send 없음',True)
    else:
        row('rc','ds','D1: R2 ACCEPTANCE_PENDING + K1 의도 + outbox',commit=True)
        row('rc','tm','ReleaseAndReconcile + K1 / same ID / expected gate revision')
        row('tm','ws','W1: inbox 중복 검사 + Q1 v8 + K1 + gate + receipt/event',commit=True)
        row('tm','rc','AcceptanceReceipt(K1, ACCEPTED, PENDING, revision)',True)
        row('rc','ds','D2: 접수 사실 + Request projection + event cursor',commit=True)
        row('ag','tm','claim(K1): D2와 독립 / 현재 gate와 권한 검사')
        row('tm','ws','W2: PENDING → DISPATCHING 조건부 CAS',commit=True)
        row('ws','ag','claim 성공 / 실패면 외부 send 없음',True)
    row('ag','agent','send(K1, X1, Q1, answer=상반기) / transaction 밖')
    row('agent','ag','외부 receipt / 수신 확인된 K1 / 단순 socket 성공과 구별',True)
    row('ag','ss' if a else 'ws','I1: raw receipt / event inbox 내구 저장, 확인 응답은 저장 뒤',commit=True)
    row('ag','tm','외부 사실 적용 요청 / 원본 event ID와 revision')
    row('tm','ss' if a else 'ws',('A3' if a else 'W3')+': EXTERNAL_ACCEPTED + 실행 관측 + 내구 반환 event',commit=True)
    row('tm','rc','확인된 외부 사실 / 내구 원본 참조' if a else '확인된 외부 사실 / event 재전달과 cursor 반영',True)
    if not a:row('rc','ds','D3: 외부 확인 사실 projection + event cursor 함께 반영',commit=True)
    row('rc','rm','현재성 검사 뒤 P2 게시 허용 / 확인된 사실만 구성')
    row('rm','im','release(P2, output epoch) → 현재 발화 상태 검사 뒤 실제 출력')
    row('im','rm','actual receipt(P2, Text / audible range)',True)
    row('rm','ss' if a else 'ds',('A4' if a else 'D4')+': 실제 전달 범위와 UNKNOWN 경계를 내구 원장에 기록',commit=True)
    row('rm','rc','P2 actual reference 반환 / Request의 실제 제시 관계 갱신',True)
    step=52 if a else 47
    for i,(src,dst,label,ret,commit) in enumerate(rows,1):
        y=407+(i-1)*step
        if commit:
            # A small color chip marks a local DB commit, never an end-to-end transaction.
            p.text(30,y-7,'▣',21,ACCENT)
        sx,tx=xs[src],xs[dst]
        # Sequence messages have their own time-row attachment points.
        p.edges.append(dict(src=src,dst=dst,points=[(sx,y),(tx,y)],ret=ret,accent=commit,sp='B',tp='B',sd=0,td=0))
        p.text(min(sx,tx)+12,y-10,f'{i:02d}  {label}',20,ACCENT if commit else INK,mask=True)
    p.text(64,1848,'확정 전 실패',27,bold=True)
    p.text(64,1893,'A1 충돌: 관련 변경 모두 미반영, 현재 원본 재조회.\nA2 CAS 실패: 외부 send 없이 현재 gate / Task 확인.' if a else 'W1 충돌: 업무 변경 미반영, 현재 원본 재조회.\nD1 의도는 거절 / 보류 사실로 갱신.',25)
    p.text(1370,1848,'확정 뒤 응답 유실',27,bold=True)
    p.text(1370,1893,'같은 K1로 공동 확정 결과 조회.\n새 업무 ID로 재실행하지 않음. inbox에서 미반영 사실 복구.' if a else '같은 K1로 접수 결과 조회. 새 업무 ID로 재실행하지 않음.\nW1 뒤 D2 전 단절: 업무 원본 유지, receipt / cursor 재결합.',25)
    p.text(2640,1848,'정정과 전송의 경합',27,bold=True)
    p.text(2640,1893,'hold가 CAS보다 먼저: K1 보류 후 새 K2로 교체.\nCAS가 먼저: 외부 효력을 조회하고 지원되는 정정 / 취소.',25)
    p.text(64,2030,'각 줄은 설명용 정상 실행 순서다. 입력 수신과 다른 Task는 계속된다. 모델 / 네트워크 / 실제 출력 대기는 local transaction 밖이다.',25,MUTED)
    p.text(64,2073,'내부 접수 ACCEPTED ≠ 외부 EXTERNAL_ACCEPTED ≠ 사용자 실제 제시. 전송 timeout은 UNKNOWN이며 외부 확인 없이 새 start를 만들지 않는다.',25,ACCENT)
    p.text(64,2116,'A: 관련 owner 변경만 공동 확정한다. 모델 / 외부 API / 물리 출력은 포함하지 않는다. 내부 비동기와 worker, 다른 Task의 처리를 계속 허용한다.' if a else 'B: W2는 D2를 기다리지 않는다. 같은 ID의 같은 payload는 저장된 결과를 반환하고, 다른 payload는 거절한다. 늦은 release는 새 hold를 해제하지 않는다.',24,MUTED)
    return p


def main(argv=None):
    ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args(argv)
    scenes=[architecture('A'),architecture('B'),sequence('A'),sequence('B')]
    drift=[]
    for p in scenes:
        if 'implementation' in p.slug:p.validate()
        else:
            assert all(0 <= x <= W and 0 <= y < 1812 for e in p.edges for x,y in e['points'])
            assert all(len(e['points']) == 2 and e['points'][0][1] == e['points'][1][1] for e in p.edges)
        for ext,data in [('svg',p.svg()),('drawio',p.drawio())]:
            tree=ET.fromstring(data)
            if ext=='drawio':
                ids=[c.get('id') for c in tree.findall('.//mxCell')]
                assert len(ids)==len(set(ids))
                assert all(c.get(attr) in ids for c in tree.findall('.//mxCell') for attr in ('source','target','parent') if c.get(attr))
            path=OUT/(p.slug+'.'+ext)
            if args.check:
                if not path.exists() or path.read_text()!=data:drift.append(path.name)
            else:path.write_text(data)
    if args.check:
        body=DOC.read_text()
        for p in scenes:
            for ext in ('svg','drawio','png'):
                assert f'./diagrams/{p.slug}.{ext}' in body,(p.slug,ext,'not linked in primary document')
            assert any(line.startswith('![') and f'](./diagrams/{p.slug}.svg)' in line for line in body.splitlines())
        record=json.loads((OUT/'dp42-implementation-render.json').read_text())
        assert {r['stem'] for r in record['records']} == {p.slug for p in scenes}
        assert len(record['records']) == len(scenes)
        for row in record['records']:
            assert row['width'] == W and row['height'] == H and row['text_geometry_issues'] == 0
            for ext in ('svg','png'):
                assert hashlib.sha256((OUT/(row['stem']+'.'+ext)).read_bytes()).hexdigest()==row[ext+'_sha256'],(row['stem'],ext,'render drift')
    if drift:raise SystemExit('Diagram drift: '+', '.join(drift))
    print('PASS: four DP-42 detailed SVG/draw.io views, containment, routes, XML and source parity')


if __name__=='__main__':main()
