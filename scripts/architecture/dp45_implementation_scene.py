"""DP45 implementation-design views. No VIA runtime or model execution."""
from __future__ import annotations

import html
import xml.etree.ElementTree as ET

W, H = 3840, 2160
INK, BLUE, RED, GRAY = '#263445', '#195FBD', '#B63732', '#758294'
PALE, ORANGE = '#FFF3E5', '#B8753F'
FONT = 'Arial, Apple SD Gothic Neo, Malgun Gothic, sans-serif'


class MemoryScene:
    def __init__(self, side):
        self.side = side
        self.slug = 'dp45-implementation-' + side.lower()
        self.title = ('A: 요청별 원본 조회와 근거 조합' if side == 'A' else
                      'B: 공통 관계 생산과 게시본 조회')
        self.nodes, self.edges, self.notes = {}, [], []
        self.text(64, 44, 'VIA / DP-45 / IMPLEMENTATION DESIGN', 26, bold=True)
        self.text(3776, 44, '구현 설계 / 제품 구현 및 모델 실행 아님', 24, align='right')
        self.text(64, 112, self.title, 44, bold=True)
        self.text(64, 168, '같은 원본, 권한과 요청. 긴 I/O와 모델 호출은 비동기 작업이며 실제 읽은 버전과 누락을 반환한다.', 28)

    def text(self, x, y, value, size=24, color=INK, bold=False, align='left', mask=False):
        self.notes.append(dict(x=x, y=y, lines=value.split('\n'), size=size, color=color,
                               bold=bold, align=align, mask=mask))

    def node(self, key, x, y, w, h, title, lines=(), kind='code', owner=None, different=False):
        assert key not in self.nodes
        self.nodes[key] = dict(id=key, x=x, y=y, w=w, h=h, title=title,
                               lines=list(lines), kind=kind, owner=owner, different=different)
        return key

    def port(self, key, side, offset=0):
        n = self.nodes[key]; x, y, w, h = (n[k] for k in ('x', 'y', 'w', 'h'))
        return {'L': (x, y+h/2+offset), 'R': (x+w, y+h/2+offset),
                'T': (x+w/2+offset, y), 'B': (x+w/2+offset, y+h)}[side]

    def edge(self, source, target, sp='R', tp='L', via=(), kind='post', sd=0, td=0, label='', at=None):
        points = [self.port(source, sp, sd), *via, self.port(target, tp, td)]
        self.edges.append(dict(id='wire'+str(len(self.edges)+1), source=source, target=target,
                               points=points, kind=kind))
        if label: self.text(*at, label, 21, color={'post':BLUE,'done':BLUE,'cancel':RED,'state':GRAY,'call':INK}[kind], mask=True)

    def ancestors(self, key):
        result = set()
        while self.nodes[key]['owner']:
            key = self.nodes[key]['owner']; result.add(key)
        return result

    def validate(self):
        for key, n in self.nodes.items():
            x,y,w,h = (n[k] for k in ('x','y','w','h'))
            assert 0 <= x and 0 <= y and x+w <= W and y+h <= H, (self.slug,key,'bounds')
            if n['owner']:
                p = self.nodes[n['owner']]
                assert p['x'] <= x and p['y']+48 <= y and x+w <= p['x']+p['w'] and y+h <= p['y']+p['h'], (self.slug,key,'containment')
                assert any(self.nodes[a]['kind']=='component' for a in self.ancestors(key)), (key,'component owner')
            elif n['kind'] not in ('component','trace','external'):
                raise AssertionError((key,'missing owner'))
        for e in self.edges:
            exempt={e['source'],e['target']} | self.ancestors(e['source']) | self.ancestors(e['target'])
            for a,b in zip(e['points'],e['points'][1:]):
                assert a[0]==b[0] or a[1]==b[1], (self.slug,e['id'],'orthogonal',a,b)
                assert 0<=a[0]<=W and 0<=a[1]<=H
                for key,n in self.nodes.items():
                    if key in exempt or n['kind'] in ('component','group'): continue
                    x,y,w,h=(n[k] for k in ('x','y','w','h'))
                    v=a[0]==b[0] and x+1<a[0]<x+w-1 and max(min(a[1],b[1]),y+1)<min(max(a[1],b[1]),y+h-1)
                    z=a[1]==b[1] and y+1<a[1]<y+h-1 and max(min(a[0],b[0]),x+1)<min(max(a[0],b[0]),x+w-1)
                    assert not (v or z), (self.slug,e['id'],'crosses node',key)
        assert not any(chr(c) in self.svg() for c in (0xB7,0x2022,0x2027,0x318D,0x30FB,0x22C5,0x2219))
    def texts(self, n):
        x,y,w,h=(n[k] for k in ('x','y','w','h')); k=n['kind']
        if k in ('component','group'):
            return [(x+18,y+34,n['title'],30 if k=='component' else 27,True,'left')]
        title_size=25 if w<480 else 27
        top=y+(42 if k=='store' else 32)
        result=[(x+w/2,top,n['title'],title_size,True,'center')]
        for j,line in enumerate(n['lines']): result.append((x+18,top+35+j*29,line,23,False,'left'))
        return result

    def svg(self):
        esc=html.escape
        out=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" role="img"><title>{esc(self.title)}</title><defs>']
        for c in (INK,BLUE,RED,GRAY):
            out.append(f'<marker id="a{c[1:]}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M1 1L9 5L1 9" fill="none" stroke="{c}" stroke-width="1.6"/></marker>')
        out += ['</defs>',f'<rect width="{W}" height="{H}" fill="white"/>',f'<g font-family="{FONT}">']
        for key,n in self.nodes.items():
            x,y,w,h=(n[k] for k in ('x','y','w','h'));kind=n['kind']
            fill=PALE if n['different'] else 'white';stroke=ORANGE if n['different'] else INK
            attrs=f'id="{key}" data-kind="{kind}" data-owner="{n["owner"] or ""}"'
            if kind not in ('component','group'):
                attrs+=f' data-node="true" data-x="{x}" data-y="{y}" data-w="{w}" data-h="{h}"'
            out.append('<g '+attrs+'>')
            if kind=='store':
                out.append(f'<path d="M{x} {y+13}C{x} {y-3} {x+w} {y-3} {x+w} {y+13}V{y+h-13}C{x+w} {y+h+3} {x} {y+h+3} {x} {y+h-13}Z M{x} {y+13}C{x} {y+29} {x+w} {y+29} {x+w} {y+13}" fill="{fill}" stroke="{stroke}" stroke-width="2"/>')
            elif kind == 'external':
                out.append(f'<path d="M{x+18} {y}H{x+w-18}L{x+w} {y+h/2}L{x+w-18} {y+h}H{x+18}L{x} {y+h/2}Z" fill="{fill}" stroke="{stroke}" stroke-width="2"/>')
            elif kind in ('data','trace'):
                out.append(f'<path d="M{x} {y}H{x+w-20}L{x+w} {y+20}V{y+h}H{x}Z M{x+w-20} {y}V{y+20}H{x+w}" fill="{fill}" stroke="{stroke}" stroke-width="2"/>')
            else:
                out.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{9 if kind=="code" else 0}" fill="{fill}" stroke="{stroke}" stroke-width="{2.5 if kind=="component" else 2}"/>')
                if kind in ('component','group'):
                    out.append(f'<path d="M{x} {y+48}H{x+w}" stroke="{stroke}" stroke-width="1.5"/>')
                if kind=='queue':
                    start=x+w-190
                    out.append(f'<path d="M{start-20} {y+h-22}H{start+165}" stroke="{BLUE}" marker-end="url(#a{BLUE[1:]})"/>')
                    for j in range(4): out.append(f'<rect x="{start+j*38}" y="{y+h-38}" width="30" height="30" fill="white" stroke="{stroke}"/>')
            for xx,yy,t,size,bold,align in self.texts(n):
                out.append(f'<text x="{xx}" y="{yy}" data-width="{w-36}" font-size="{size}" font-weight="{700 if bold else 400}" text-anchor="{"middle" if align=="center" else "start"}" fill="{INK}">{esc(t)}</text>')
            out.append('</g>')
        for e in self.edges:
            c={'post':BLUE,'done':BLUE,'cancel':RED,'state':GRAY,'call':INK}[e['kind']]
            dash=' stroke-dasharray="9 6"' if e['kind'] in ('done','cancel','state') else ''
            pts=' '.join(f'{x},{y}' for x,y in e['points'])
            out.append(f'<polyline id="{e["id"]}" data-flow="{e["kind"]}" data-source="{e["source"]}" data-target="{e["target"]}" points="{pts}" fill="none" stroke="{c}" stroke-width="2" marker-end="url(#a{c[1:]})"{dash}/>')
        for i,n in enumerate(self.notes):
            attr=' paint-order="stroke" stroke="white" stroke-width="8" stroke-linejoin="round"' if n['mask'] else ''
            out.append(f'<g id="note{i}"{attr}>')
            for j,t in enumerate(n['lines']):
                anchor={'left':'start','right':'end','center':'middle'}[n['align']]
                out.append(f'<text x="{n["x"]}" y="{n["y"]+j*n["size"]*1.3}" font-size="{n["size"]}" font-weight="{700 if n["bold"] else 400}" text-anchor="{anchor}" fill="{n["color"]}">{esc(t)}</text>')
            out.append('</g>')
        return '\n'.join(out+['</g></svg>'])+'\n'

    def drawio(self):
        mx=ET.Element('mxfile',host='app.diagrams.net',type='device')
        d=ET.SubElement(mx,'diagram',id=self.slug,name=self.title)
        g=ET.SubElement(d,'mxGraphModel',page='1',pageWidth=str(W),pageHeight=str(H))
        root=ET.SubElement(g,'root');ET.SubElement(root,'mxCell',id='0');ET.SubElement(root,'mxCell',id='1',parent='0')
        def vertex(key,x,y,w,h,value,style,parent='1'):
            c=ET.SubElement(root,'mxCell',id=key,parent=parent,vertex='1',value=value,style=style)
            ET.SubElement(c,'mxGeometry',x=str(x),y=str(y),width=str(w),height=str(h),attrib={'as':'geometry'})
        for key,n in self.nodes.items():
            x,y,w,h=(n[k] for k in ('x','y','w','h'));parent=n['owner'] or '1'
            if parent!='1':x-=self.nodes[parent]['x'];y-=self.nodes[parent]['y']
            kind=n['kind'];fill=PALE if n['different'] else 'white';stroke=ORANGE if n['different'] else INK
            shape={'component':'swimlane;startSize=48;collapsible=0;', 'group':'swimlane;startSize=48;collapsible=0;',
                   'store':'shape=cylinder;size=13;', 'data':'shape=note;size=20;', 'trace':'shape=note;size=20;',
                   'external':'shape=hexagon;', 'queue':'rounded=0;', 'code':'rounded=1;arcSize=10;'}[kind]
            value=html.escape(n['title'])
            if kind not in ('component','group'):
                value = '<div style="font-size:27px;font-weight:bold;line-height:1.3;text-align:center">'+html.escape(n['title'])+'</div>'
                value += '<div style="font-size:23px;line-height:1.26;text-align:left;margin-top:8px">'+'<br>'.join(html.escape(t) for t in n['lines'])+'</div>'
            vertex(key,x,y,w,h,value,shape+f'html=1;whiteSpace=wrap;fillColor={fill};swimlaneFillColor={fill};strokeColor={stroke};fontColor={INK};fontFamily=Arial;fontSize={30 if kind=="component" else 27};fontStyle={1 if kind in ("component","group") else 0};align=left;verticalAlign=top;spacingTop={14 if kind=="store" else 8};spacingLeft=18;spacingRight=18;',parent)
            if kind=='queue':
                for j in range(4):vertex(key+f'-slot{j}',w-190+j*38,h-38,30,30,'',f'fillColor=white;strokeColor={stroke};',key)
        for e in self.edges:
            c={'post':BLUE,'done':BLUE,'cancel':RED,'state':GRAY,'call':INK}[e['kind']]
            st=f'edgeStyle=none;rounded=0;strokeColor={c};strokeWidth=2;endArrow=open;endFill=0;'+('dashed=1;' if e['kind'] in ('done','cancel','state') else '')
            for role,p,prefix in (('source',e['points'][0],'exit'),('target',e['points'][-1],'entry')):
                n=self.nodes[e[role]];st+=f'{prefix}X={(p[0]-n["x"])/n["w"]};{prefix}Y={(p[1]-n["y"])/n["h"]};{prefix}Perimeter=0;'
            cell=ET.SubElement(root,'mxCell',id=e['id'],source=e['source'],target=e['target'],edge='1',parent='1',style=st)
            geo=ET.SubElement(cell,'mxGeometry',relative='1',attrib={'as':'geometry'})
            arr=ET.SubElement(geo,'Array',attrib={'as':'points'})
            for x,y in e['points'][1:-1]:ET.SubElement(arr,'mxPoint',x=str(x),y=str(y),attrib={'as':'point'})
        for i,n in enumerate(self.notes):
            value='<br>'.join(html.escape(t) for t in n['lines'])
            width=min(W-2, max(sum(1 if ord(c)>255 else .66 for c in t)*n['size']+8 for t in n['lines']))
            if n['align']=='right':x=n['x']-width
            elif n['align']=='center':x=n['x']-width/2
            else:x=n['x']
            vertex('note'+str(i),x,n['y']-n['size'],width,len(n['lines'])*n['size']*1.3,value,f'text;html=1;whiteSpace=nowrap;overflow=visible;fillColor=none;strokeColor=none;labelBackgroundColor={"white" if n["mask"] else "none"};fontColor={n["color"]};fontSize={n["size"]};fontStyle={int(n["bold"])};align={n["align"]};spacing=0;')
        return ET.tostring(mx,encoding='unicode')+'\n'


def common(side):
    p=MemoryScene(side); n=p.node
    n('im',64,238,600,170,'Interaction Manager',kind='component')
    n('input',104,310,520,88,'InputReady',['R23 / input revision / evidence refs'],owner='im')
    n('scenario',744,238,2360,170,'같은 사용자 장면과 원본',[
        'R23: “지난번 설명한 기준에 제품 비교와 견적을 맞춰 제안서를 만들어줘.”',
        '과거 원본: E20 설명 / P20 실제 전달 [0,52) / T21 → D21 개정 2 / T22 → D22 개정 1',
        '검색 시작 전에는 위 ID가 target binding으로 주어지지 않는다. 검색 결과로 후보를 얻는다.'],kind='trace')
    n('legend',3184,238,592,170,'도형과 실행 범위',[
        '직각 상자: Component / 내부 둥근 상자: 코드',
        '원통: RAM 또는 DURABLE 상태',
        '살구색: A/B에서 달라지는 내부 조직'],kind='trace')
    n('rc',64,470,600,450,'Request Controller',kind='component')
    n('scope',104,550,520,175,'Scope / Adoption Gate',[
        '입력 개정, 허용 source 범위와 budget 등록',
        'MeaningProposal의 현재성 검사와 채택',
        '채택 후 Task Manager에 위임 요청',
        '확정 Request와 사용 refs는 State Store'],owner='rc')
    n('history',104,780,520,110,'History Read Port',[
        '주제 / 입력 / 채택 정정의 검색과 읽기',
        '명시 REVISES / 당시 revision 반환'],owner='rc')
    n('ri',64,1030,600,410,'Request Interpreter',kind='component')
    n('read_tool',104,1110,520,125,'Read Tool Runner',[
        'query_id / read_id / Request / attempt',
        'EvidenceQuery → async 완료를 같은 caller로'],owner='ri')
    n('meaning',104,1290,520,115,'Current Meaning Job',[
        'Bundle로 현재 목표와 Task 연결 제안',
        '추가 읽기 또는 질문 / 자동 채택 아님'],owner='ri')
    n('model',64,1520,600,195,'Model Access',kind='component')
    n('api',104,1590,520,115,'Semantic API Client',[
        'scope / call ID / timeout / cancel / usage',
        '현재 의미와 조건부 생산의 역할 분리'],owner='model')
    n('cloud',64,1785,600,115,'Cloud semantic model',[
        '외부 dependency / 입력 허용 범위만 전송',
        '취소 실패 후 완료와 청구도 추적'],kind='external')
    n('cm',744,470,2360,1250,'Context Manager',kind='component')
    n('rm',3184,470,592,360,'Response Manager',kind='component')
    n('response_read',3224,550,512,225,'Response History Port',[
        'search(topic, range, cursor)',
        'read(E20, revision, span)',
        'read(P20, actual Text / Voice spans)',
        '초안과 실제 전달을 구별',
        '원문과 actual receipt: DURABLE'],owner='rm')
    n('tm',3184,910,592,360,'Task Manager',kind='component')
    n('task_read',3224,990,512,225,'Task History Port',[
        'search(topic, range, cursor)',
        'read(T21 / T22, result refs)',
        'D21@v2 / D22@v1 / source revision',
        'Agent Gateway의 확인 사건에서 저장',
        'Task / 결과 참조: DURABLE'],owner='tm')
    n('pm',3184,1350,592,370,'Policy Manager',kind='component')
    n('policy',3224,1430,512,210,'Use Authorization Port',[
        '현재 scope와 permission epoch',
        'checkAndRegisterUse(expected epoch)',
        '철회 commit → 구 epoch 사용 차단',
        '긴 read / 모델 완료를 기다리지 않음',
        '권한 및 삭제 상태: DURABLE'],owner='pm')
    p.edge('input','scope','B','T',sd=180,td=180,label='입력 등록',at=(390,451))
    p.edge('scope','read_tool','L','L',via=[(34,637.5),(34,1172.5)],kind='call')
    p.text(64,982,'0  허용된 현재 요청에서 조회 시작',24,bold=True)
    p.edge('read_tool','meaning','B','T',kind='done',label='현재 요청 해석',at=(365,1270))
    p.edge('meaning','scope','R','R',via=[(684,1347.5),(684,637.5)],kind='done')
    p.text(64,1484,'공통 후속: 의미 제안 → 현재 검사 → 채택',24,mask=True)
    p.edge('meaning','api','B','T',sd=-80,td=-80,kind='call')
    p.edge('api','meaning','R','R',via=[(650,1647.5),(650,1367.5)],td=20,kind='done')
    p.edge('api','cloud','B','T',sd=-80,td=-80,kind='call')
    p.edge('cloud','api','T','B',sd=80,td=80,kind='done')
    p.text(64,1960,'위임은 채택 후에만 수행',26,bold=True)
    p.text(64,2003,'Task Manager → Agent Gateway\n→ Downstream Agent: 제안서 작성',24)
    p.text(64,2106,'실선 = 요청 / 점선 파랑 = 완료 / 빨강 = 사용 차단 / 회색 = 상태 접근. 선 교차는 접속점이 아니다.',24)
    p.text(64,2145,'양안 공통: 로컬 VIA, owner별 State Store 저장, bounded async executor. 상자 수는 thread / process / DB 제품 수가 아니다.',24)
    return p


def owner_reads(p, broker, y_request, y_return, history_rail):
    # Each owner call and completion uses a different lane and its own read ID.
    bx=p.nodes[broker]['x']+p.nodes[broker]['w']; by=p.nodes[broker]['y']+p.nodes[broker]['h']/2
    p.edge(broker,'response_read','R','L',sd=y_request-by,td=-42.5,
           via=[(3124,y_request),(3124,620)],kind='call',label='원문 / 실제 전달 읽기',at=(3190,858))
    p.edge('response_read',broker,'L','R',sd=37.5,td=y_return-by,
           via=[(3144,700),(3144,y_return)],kind='done')
    p.edge(broker,'task_read','R','L',sd=y_request+20-by,td=-22.5,
           via=[(3164,y_request+20),(3164,1080)],kind='call')
    p.edge('task_read',broker,'L','R',sd=37.5,td=y_return+20-by,
           via=[(3180,1140),(3180,y_return+20)],kind='done')
    top=p.port(broker,'T',-65)
    p.edge(broker,'history','T','R',sd=-65,td=-15,
           via=[(top[0],history_rail),(714,history_rail),(714,820)],kind='call')
    top2=p.port(broker,'T',-25)
    p.edge('history',broker,'R','T',sd=15,td=-25,
           via=[(704,850),(704,history_rail+20),(top2[0],history_rail+20)],kind='done')
    p.text(900,history_rail-10,'원본별 read port / 완료는 read_id와 source revision으로 연결',22,color=GRAY,mask=True)


def a():
    p=common('A'); n=p.node
    n('cache',824,565,750,260,'Source View Cache / 관계 cache',[
        '재사용 상태 / RAM + 선택적 영속화',
        'key: source refs + 목적 / scope + 형식 version',
        '값: 허용 발췌 / 요약 / 명시 또는 파생 관계',
        'dependency vector / policy epoch / 유효성',
        'hit에도 원본 확인 / 누락은 요청 경로에서 해결'],kind='store',owner='cm',different=True)
    n('invalidate',1744,565,580,260,'Invalidation Handler',[
        '원본 변경 / 철회 / EvidenceDispute 수용',
        'dependency로 cache와 활성 read 표시',
        '원본 변경 → DIRTY / 재조회',
        '철회 → 사용 차단 / purge 재시도',
        'old job 완료는 현재 read에 합치지 않음'],owner='cm',different=True)
    n('changes',2464,565,560,260,'Change / Control Inbox',[
        'owner ID / source revision / epoch',
        '일반 cache 정리는 bounded queue',
        '철회 gate는 이 backlog를 기다리지 않음',
        '신호 gap은 사용 전 원본 검사로 탐지',
        'cache 복구를 위한 변경이며 필수 생산 아님'],kind='queue',owner='cm',different=True)
    n('composer',784,990,2280,690,'Context Composer / 요청별 조합 작업',kind='group',owner='cm',different=True)
    n('query',824,1110,500,155,'Read Inbox / Admission',[
        'query + read ID / 목적 / deadline',
        'scope와 자원 상한 검사',
        'Cancel(read_id) / terminal reply 1회'],kind='queue',owner='composer',different=True)
    n('planner',1474,1110,700,185,'Read Pump / Planner',[
        '검색 단서 → owner별 bounded search',
        'cache 확인 → 누락된 read만 병렬 시작',
        '완료를 read set에 반영 / 추가 조회 결정',
        '자유문 현재 의미는 Request Interpreter'],owner='composer',different=True)
    n('broker',2424,1110,600,185,'Owner Read Broker',[
        'owner adapter / bounded executor',
        'search / read / validate async 작업',
        '서로 다른 source의 전역 snapshot 없음',
        'read_id + dependency + 결과를 callback'],owner='composer',different=True)
    n('bundle',824,1450,500,180,'Bundle Builder / Use Guard',[
        '실제 전달 span과 원문 일치 검사',
        'EvidenceBundle + QueryReceipt',
        '누락 / source vector / read 범위',
        '현재 권한과 attempt 확인 후 reply'],owner='composer',different=True)
    n('evidence',1474,1450,700,180,'Request Evidence Set (RAM)',[
        'read_id → 후보 refs / source vector',
        '발췌 / 명시 관계 / gaps / pending reads',
        '일부 source 변경 시 해당 부분만 재조회',
        '현재 Task binding의 확정 원본 아님'],kind='store',owner='composer',different=True)
    n('jobs',2424,1450,600,180,'Read Job Index (RAM)',[
        'read_id / job_id / input revision',
        'pending / done / cancelled / dependency',
        'cancel handle / 실제 종료 / usage',
        '재시작은 원본으로 read 재개 또는 재발행'],kind='store',owner='composer',different=True)
    p.edge('read_tool','query','R','L',sd=15,td=0,via=[(714,1187.5)],label='1  Query',at=(678,1168))
    p.edge('query','planner','R','L',td=-15,label='2 시작',at=(1342,1168))
    p.edge('planner','cache','T','B',sd=-180,td=-100,via=[(1644,870),(1099,870)],kind='call',label='3 cache 조회 / 검증한 항목 저장',at=(1120,861))
    p.edge('cache','planner','B','T',sd=130,td=180,via=[(1329,903),(2004,903)],kind='done',label='hit + 유효성 / miss',at=(1610,891))
    p.edge('planner','broker','R','L',sd=-20,td=-20,label='4 누락 read 시작',at=(2190,1164))
    owner_reads(p,'broker',1180,1220,938)
    p.edge('broker','jobs','B','T',sd=140,td=140,kind='state',label='job 등록 / 종료',at=(2868,1385))
    p.edge('broker','evidence','B','T',sd=-110,via=[(2614,1365),(1824,1365)],kind='done',label='5 완료: snapshot / refs / gaps',at=(1920,1352))
    p.edge('planner','evidence','B','T',sd=-200,td=-200,kind='state',label='read set 갱신',at=(1490,1375))
    p.edge('evidence','bundle','L','R',kind='state',label='6 조합',at=(1346,1520))
    p.edge('bundle','read_tool','L','R',sd=20,td=35,via=[(734,1560),(734,1207.5)],kind='done',label='8 Bundle / receipt',at=(743,1417))
    p.edge('changes','invalidate','L','R',kind='cancel',label='9 변화',at=(2338,680))
    p.edge('invalidate','cache','L','R',kind='cancel',label='dirty',at=(1600,680))
    p.edge('invalidate','jobs','R','L',sd=75,td=30,via=[(2360,770),(2360,1570)],kind='cancel',label='활성 read 무효화 / cancel',at=(2200,1738))
    p.edge('response_read','changes','L','R',sd=-75,td=-75,via=[(3094,587.5),(3094,620)],kind='done')
    p.edge('task_read','changes','L','R',sd=75,td=75,via=[(3080,1177.5),(3080,770)],kind='done')
    p.edge('bundle','policy','B','L',sd=80,td=0,via=[(1154,1756),(3120,1756),(3120,1535)],label='7 use check + register / source 검사 결과 동반',at=(1530,1750))
    p.edge('policy','bundle','L','B',sd=45,td=-80,via=[(3140,1580),(3140,1796),(994,1796)],kind='done',label='허용 / 철회 / 만료 → reply 또는 보류',at=(1830,1787))
    # Revocation is immediate at use gates; queue invalidation is cleanup only.
    p.text(3184,1322,'원본은 owner별로 State Store에 저장',22,color=GRAY)
    n('trace',744,1850,1110,205,'같은 요청의 정상 경로',[
        '1~4 후보 검색 → 유효 cache 활용 / 필요한 원본만 읽기',
        '5~8 E20 [0,52), D21@v2, D22@v1 + 실제 읽은 범위 반환',
        '같은 R24에서는 검증한 cache 재사용. 새 관계는 새 조회로 조합.',
        '단순 코드 조합에 C-CONTEXT 호출을 강제하지 않는다.'],kind='trace')
    n('failure',1924,1850,1180,205,'변경과 취소에서 실행하는 코드',[
        'D21 개정 3 도착 → 관련 cache / read DIRTY → 그 부분 재조회',
        'Cancel(read_id) → read set 사용 중지 → I/O cancel 요청',
        '늦은 callback은 Job Index에서 종료만 회수. 새 요청에 합치지 않음.',
        '공통 사용 gate가 변경 이후 사용을 다시 검사한다.'],kind='trace')
    p.text(3184,1810,'공통 User Memory',26,bold=True)
    p.text(3184,1854,'Context Manager의 명시 저장과 삭제 원장\n양안 동일 / 과거 관계에서 자동 등록하지 않음\n그림은 과거 기록 read 경로를 확대함',23)
    return p


def b():
    p=common('B'); n=p.node
    n('publisher',784,540,2280,540,'Memory Publisher / 현재 요청과 독립적인 생산 수명',kind='group',owner='cm',different=True)
    n('derive',824,630,600,160,'Relation Builder',[
        'PRESENTS / HAS_RESULT / REVISES: 코드',
        'EXPLAINS_ITEM: 조건부 C-CONTEXT',
        '원문 span / uncertainty / producer version'],owner='publisher',different=True)
    n('source_worker',1634,630,600,160,'Source Read Worker',[
        'ProductionJob의 source vector로 읽기',
        'bounded async 원본 읽기 / 현재성 확인',
        '미공개 설명과 미래 Request binding 제외'],owner='publisher',different=True)
    n('admit',2444,630,580,160,'Change Admission / Planner',[
        '확정 변경 또는 지원 범위 backfill 수용',
        '원본 변경 → 해당 dependency 사용 차단',
        '권한 / budget 확인 후 production 등록'],owner='publisher',different=True)
    n('candidate',824,890,600,155,'Candidate Batch (RAM)',[
        'job ID / source vector / epoch',
        'proposed relations / spans / gaps',
        '게시 전이므로 Reader가 읽지 않음'],kind='store',owner='publisher',different=True)
    n('commit',1634,890,600,155,'Publish Guard / Commit',[
        '현재 dependency / epoch / expected rev 검사',
        'records + Coverage + cursors 함께 확정',
        '충돌 / 오래된 생산 → RETRY 또는 REJECT'],owner='publisher',different=True)
    n('journal',2444,890,580,155,'Production Journal (DURABLE)',[
        'job key / queued / running / retry',
        'source refs + producer version 중복 제거',
        'callback / 재시도 / 게시 통지 outbox'],kind='store',owner='publisher',different=True)
    n('waits',824,1160,600,200,'Read Wait Registry (RAM)',[
        'read_id → 필요한 범위 / min publish rev',
        'deadline / 취소 / consumer 연결',
        'PublishResult는 알림. 저장소 재조회 필수',
        'consumer 종료와 생산 작업 종료는 별개'],kind='store',owner='cm',different=True)
    n('repository',1634,1140,1390,230,'Episodic Memory Repository (DURABLE)',[
        'MemoryRecord + Coverage + published revision + source별 committed cursor',
        'topic / owner / 시간 색인, dependency 역색인, VALID / DIRTY / REVOKED',
        'relation과 Coverage는 같은 snapshot으로 읽기. 미생산과 미지원은 서로 다른 상태.',
        '원본 사실과 현재 Task 상태는 해당 owner 권위. 원본 전체의 동시 snapshot이 아님.',
        '물리 저장은 State Store / 원본이 보존된 범위에서만 재생산 가능'],kind='store',owner='cm',different=True)
    n('reader',784,1420,2280,265,'Evidence Reader / 현재 read 수명',kind='group',owner='cm',different=True)
    n('read',824,1500,600,160,'Read Pump / Bundle Builder',[
        'query / read ID / 허용 scope / budget',
        '후보와 근거 + 실제 확인 범위로 Bundle 생성',
        '최종 Use Guard 검사 → caller에 reply 1회'],owner='reader',different=True)
    n('coverage',1634,1500,600,160,'Index / Coverage Resolver',[
        '게시된 후보 검색 → 범위와 kind 확인',
        'COVERED → 검증 / NOT_COVERED → 생산 대기',
        'UNSUPPORTED → 제한 반환 / B+는 별도'],owner='reader',different=True)
    n('origin',2444,1500,580,160,'Origin Verifier',[
        '현재 source revision / policy 확인',
        '정확한 인용: 게시 locator의 원문만 읽기',
        'stale / 삭제 / 접근 실패를 반환'],owner='reader',different=True)
    p.edge('response_read','admit','L','R',sd=-15,td=-62.5,via=[(3094,647.5)],kind='done',label='1 확정 변경',at=(3040,572))
    p.edge('task_read','admit','L','R',sd=75,td=62.5,via=[(3078,1177.5),(3078,772.5)],kind='done')
    p.edge('admit','journal','B','T',kind='state',label='job 등록',at=(2740,848))
    p.edge('journal','source_worker','L','R',td=35,via=[(2374,967.5),(2374,745)],kind='done',label='2 claimed job',at=(2252,845))
    p.edge('source_worker','journal','R','L',sd=-15,td=-35,via=[(2394,695),(2394,932.5)],kind='state')
    p.edge('source_worker','derive','L','R',kind='call',label='3 원본 snapshot',at=(1438,694))
    # Production owner reads use the upper corridor; Reader never searches all owners on a miss.
    p.edge('source_worker','response_read','T','T',sd=80,td=-75,via=[(2014,514),(3405,514)],kind='call')
    p.edge('response_read','source_worker','T','T',sd=65,td=200,via=[(3545,531),(2134,531)],kind='done')
    p.edge('source_worker','history','T','R',sd=-200,td=-15,via=[(1734,438),(714,438),(714,820)],kind='call')
    p.edge('history','source_worker','R','T',sd=15,td=-160,via=[(704,850),(704,451),(1774,451)],kind='done')
    p.edge('source_worker','task_read','R','L',sd=55,td=-60,via=[(2320,765),(2320,1090),(3144,1090),(3144,1042.5)],kind='call')
    p.edge('task_read','source_worker','L','R',sd=-30,td=75,via=[(3130,1072.5),(3130,1107),(2304,1107),(2304,785)],kind='done')
    p.edge('derive','candidate','B','T',kind='done',label='4 추출 / 명시 연결 결과',at=(1135,852))
    p.edge('derive','api','L','R',sd=-20,td=-25,via=[(734,690),(734,1622.5)],kind='call')
    p.edge('api','derive','R','L',sd=20,td=25,via=[(724,1667.5),(724,735)],kind='done')
    p.text(790,881,'조건부 의미 API',22,color=BLUE,mask=True)
    p.edge('commit','source_worker','T','B',sd=-190,td=-190,kind='call',label='게시 전 재검사',at=(1638,840))
    p.edge('source_worker','commit','B','T',sd=-130,td=-130,kind='done')
    p.edge('candidate','commit','R','L',kind='post',label='5 검증 요청',at=(1442,952))
    p.edge('commit','repository','B','T',sd=-80,td=-475,via=[(1854,1100)],kind='state',label='6 조건부 transaction',at=(1692,1130))
    p.edge('repository','commit','T','B',sd=-235,td=160,via=[(2094,1118)],kind='done')
    p.edge('commit','journal','R','L',sd=40,td=40,kind='state',label='게시 결과',at=(2252,993))
    p.edge('admit','repository','R','R',sd=45,td=-55,via=[(3050,755),(3050,1200)],kind='cancel',label='변경 범위 먼저 DIRTY',at=(2770,1100))
    p.edge('read_tool','read','R','L',sd=15,td=-30,via=[(694,1187.5),(694,1550)],label='7 Query',at=(674,1490))
    p.edge('read','coverage','R','L',sd=-25,td=-25,label='8 후보 / coverage 조회',at=(1440,1541))
    p.edge('coverage','repository','T','B',sd=60,td=-335,kind='call')
    p.edge('repository','coverage','B','T',sd=-185,td=210,kind='done')
    p.edge('coverage','origin','R','L',sd=20,td=20,label='9 covered',at=(2252,1581))
    p.edge('origin','history','T','R',sd=-20,td=-15,via=[(2714,1390),(3124,1390),(3124,219),(698,219),(698,820)],kind='call')
    p.edge('history','origin','R','T',sd=15,td=80,via=[(690,850),(690,209),(3138,209),(3138,1399),(2814,1399)],kind='done')
    p.edge('origin','response_read','R','L',sd=-20,td=65,via=[(3164,1560),(3164,727.5)],kind='call')
    p.edge('response_read','origin','L','R',sd=90,td=10,via=[(3178,752.5),(3178,1590)],kind='done')
    p.edge('origin','task_read','R','L',sd=35,td=55,via=[(3150,1615),(3150,1157.5)],kind='call')
    p.edge('task_read','origin','L','R',sd=90,td=60,via=[(3114,1192.5),(3114,1640)],kind='done')
    p.edge('origin','read','B','B',sd=-100,td=100,via=[(2634,1701),(1224,1701)],kind='done',label='검증한 snapshot / gaps',at=(1980,1695))
    p.edge('read','read_tool','L','R',sd=40,td=40,via=[(714,1620),(714,1212.5)],kind='done',label='10 Bundle / receipt',at=(744,1760))
    p.edge('coverage','waits','L','R',sd=40,td=40,via=[(1544,1620),(1544,1300)],kind='state',label='11 미생산: read 대기 등록',at=(1100,1410))
    p.edge('coverage','admit','R','R',sd=-40,td=-20,via=[(2350,1540),(2350,1395),(3070,1395),(3070,690)],kind='post',label='11 지원 범위 backfill',at=(2370,1415))
    p.edge('commit','waits','L','T',sd=40,td=-150,via=[(1594,1007.5),(1594,1097),(974,1097)],kind='done',label='12 게시 알림 후 재조회',at=(812,1130))
    p.edge('waits','read','B','T',sd=260,td=260,kind='post',label='resume',at=(1387,1482))
    p.edge('read','policy','B','L',sd=220,td=0,via=[(1344,1745),(3120,1745),(3120,1535)],label='현재 epoch 검사 + 사용 등록',at=(1820,1740))
    p.edge('policy','read','L','B',sd=45,td=180,via=[(3140,1580),(3140,1780),(1304,1780)],kind='done')
    p.text(3184,1322,'원본은 owner별로 State Store에 저장',22,color=GRAY)
    n('trace',744,1850,1110,205,'준비된 조회와 첫 조회를 구별',[
        '준비됨: 7 → 8 → 9 → 10. 동일한 E20 원문과 두 결과 refs 반환.',
        '미생산: 7 → 8 → 11 → 생산 2~6 → 12 → 8 재조회 → 9 → 10.',
        '생산 대기는 해당 read만 묶는다. 입력과 다른 요청은 계속 처리.',
        'Publisher는 Bundle을 직접 caller에 반환하지 않는다.'],kind='trace')
    n('failure',1924,1850,1180,205,'원본 변경, 취소와 재시작',[
        'D21 변경: dependency 사용 차단 → 새 생산 → 조건부 게시.',
        'read 취소: waiter 해제. 허용된 공유 생산은 유용성과 budget 재검사.',
        '게시 전 crash: cursor는 미진행. 게시 후 통지 유실: revision 재조회.',
        '철회된 epoch의 늦은 생산은 commit 거절. 물리 purge는 재시도.'],kind='trace')
    p.text(3184,1810,'공통 User Memory',26,bold=True)
    p.text(3184,1854,'Context Manager의 명시 저장과 삭제 원장\n양안 동일 / 과거 관계에서 자동 등록하지 않음\nB+ 원본 조합 우회는 이 순수 B 그림 밖',23)
    return p
