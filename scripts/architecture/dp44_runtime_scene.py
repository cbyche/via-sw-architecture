"""Two matched DP44 runtime views. Documentation only, not VIA runtime code.

Queue, mutable state, immutable messages, code and long operations have different
symbols. One scene supplies native draw.io geometry and the SVG preview.
"""
from __future__ import annotations

import html
import xml.etree.ElementTree as ET

W, H = 3840, 2160
INK, BLUE, RED, GRAY = '#263445', '#195FBD', '#B63732', '#758294'
PALE, ORANGE = '#FFF3E5', '#B8753F'
FONT = 'Arial, Apple SD Gothic Neo, Malgun Gothic, sans-serif'


class RuntimeScene:
    def __init__(self, side):
        self.side = side
        self.slug = 'dp44-implementation-' + side.lower()
        self.title = ('A: 중앙 Dispatcher가 완료를 회수하고 다음 실행을 지시' if side == 'A' else
                      'B: 각 Stage가 자기 완료를 처리하고 고정 채널로 후속 실행을 연결')
        self.nodes, self.edges, self.notes = {}, [], []
        self.text(64, 45, 'VIA / DP-44 / 구현을 가정한 실행 구조', 26, bold=True)
        self.text(3776, 45, '문서 설계안 / A/B 미선정 / 제품 실행과 측정 없음', 24, align='right')
        self.text(64, 112, self.title, 43, bold=True)
        self.text(64, 172, '같은 기능과 공통 실행 기반. 차이는 완료의 수신자, 다음 실행의 결정 코드, Job과 대기 조건의 소유 위치다.', 29)

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
            elif n['kind'] not in ('component','trace'):
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
        if self.side=='A':
            assert all(e['target']=='events' for e in self.edges if e['kind']=='done' and e['source'] in ('interpret','prepare'))
        else:
            for source,target in (('interpret','input_done'),('notice_work','notice_done'),('prepare','compose_done')):
                assert any(e['source']==source and e['target']==target and e['kind']=='done' for e in self.edges)
            assert 'dispatcher' not in self.nodes

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
                   'queue':'rounded=0;', 'code':'rounded=1;arcSize=10;'}[kind]
            value=html.escape(n['title']) if kind in ('component','group') else ''
            vertex(key,x,y,w,h,value,shape+f'html=1;whiteSpace=wrap;fillColor={fill};swimlaneFillColor={fill};strokeColor={stroke};fontColor={INK};fontFamily=Arial;fontSize={30 if kind=="component" else 27};fontStyle=1;align=left;spacingLeft=18;',parent)
            if kind not in ('component','group'):
                for j,(xx,yy,t,size,bold,align) in enumerate(self.texts(n)):
                    vertex(key+f'-text{j}',18,yy-n['y']-size,w-36,size*1.3,html.escape(t),f'text;html=1;whiteSpace=nowrap;overflow=visible;fillColor=none;strokeColor=none;fontColor={INK};fontSize={size};fontStyle={int(bold)};align={align};spacing=0;',key)
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
            width=max(len(t)*n['size'] for t in n['lines'])
            if n['align']=='right':x=n['x']-width
            elif n['align']=='center':x=n['x']-width/2
            else:x=n['x']
            vertex('note'+str(i),x,n['y']-n['size'],width,len(n['lines'])*n['size']*1.3,value,f'text;html=1;whiteSpace=nowrap;overflow=visible;fillColor=none;strokeColor=none;labelBackgroundColor={"white" if n["mask"] else "none"};fontColor={n["color"]};fontSize={n["size"]};fontStyle={int(n["bold"])};align={n["align"]};spacing=0;')
        return ET.tostring(mx,encoding='unicode')+'\n'


def common(p):
    n=p.node
    n('im',64,240,3712,200,'Interaction Manager',kind='component')
    n('turn',104,308,560,112,'Turn-Taking Control',['onInputStarted: local stop / epoch++','입력 감지만으로 Task 취소를 확정하지 않음'],owner='im')
    n('input_emit',720,308,560,112,'Timeline & Buffer',['전사와 evidence refs → InputReady','capture / ASR는 긴 Job과 무관하게 계속'],owner='im')
    n('playback',2520,308,560,112,'Playback State (RAM)',['현재 output epoch / 실제 재생 범위'],kind='store',owner='im')
    n('io',3140,308,580,112,'Channel I/O',['release + epoch 검사 → 실제 표시 / 재생','실제 전달 뒤 ActualReceipt callback'],owner='im')
    n('tm',64,540,500,775,'Task Manager',kind='component')
    n('task_facts',104,650,420,190,'Task 원본 / Inbox (DURABLE)',['Task / question / command ID','확인된 상태와 source revision','Agent Gateway 사건 확인 / 저장'],kind='store',owner='tm')
    n('task_emit',104,910,420,130,'ConfirmedTaskEvent emit',['durable 사실의 참조를 enqueue','대화 연결과 질문은 Request Controller'],owner='tm')
    n('task_command',104,1100,420,175,'Task command handler',['미전송 철회 또는 외부 cancel 요청','Agent Gateway 경유 외부 cancel','Downstream Agent 요청','접수 / 실제 종료 / UNKNOWN 구별'],owner='tm')
    n('ri',64,1380,500,300,'Request Interpreter',kind='component')
    n('interpret',104,1460,420,160,'InterpretAsync',['Context Manager / Model Access','immutable input + source refs','callback: MeaningProposal 반환'],owner='ri')
    n('rc',620,540,1120,1140,'Request Controller',kind='component')
    n('rm',1800,540,1976,1140,'Response Manager',kind='component')
    n('publication',3140,595,580,1060,'Publication Control',kind='group',owner='rm')
    n('pub_q',3180,660,500,110,'Publish Request Queue',['candidate ref / admission refs'],kind='queue',owner='publication')
    n('receipt_q',3180,795,500,110,'Actual Receipt Queue',['publication ID / actual range'],kind='queue',owner='publication')
    n('pub_control',3180,940,500,110,'Control Queue',['hold / adopted invalidation'],kind='queue',owner='publication')
    n('pub_handler',3180,1100,500,175,'Pump / Handle(event)',['control 우선 / owner current check','권한 / 차례 / 중복 → commit → release','receipt → durable actual → Presented'],owner='publication')
    n('outbox',3180,1305,500,190,'Publication Outbox (DURABLE)',['publication ID → release identity','actual range / status / Presented','원본 저장: State Store'],kind='store',owner='publication')
    n('pub_result',3180,1530,500,95,'Presented (immutable)',['질문 ID / publication ID / actual range'],kind='data',owner='publication')
    p.edge('task_facts','task_emit','B','T',kind='state')
    p.edge('pub_handler','outbox','B','T',kind='state')
    p.edge('pub_handler','io','R','B',via=[(3750,1187.5),(3750,465),(3430,465)],kind='call',label='release',at=(3630,465))
    p.edge('io','receipt_q','R','R',via=[(3794,364),(3794,850)],kind='done',label='ActualReceipt',at=(3550,780))
    p.edge('io','playback','L','R',kind='state')
    # Each queue is consumed by one short handler, without crossing other queues.
    for q in ('pub_q','receipt_q','pub_control'):
        cy=p.port(q,'R')[1]
        p.edge(q,'pub_handler','R','R',via=[(3700,cy),(3700,1187.5)],kind='call')
    p.text(64,1740,'같은 정정: R1 rev1의 전체 요약 Job J1 실행 중 → “아니, 결론만” → R1 rev2 채택. J1은 취소 요청 뒤에도 늦게 완료할 수 있다.',27,bold=True)
    p.text(64,2030,'실행 가정: 공통 bounded async executor와 blocking worker pool. 각 Pump는 자기 상태를 순차 갱신하고 긴 Job을 기다리지 않는다.',25)
    p.text(64,2070,'검정 실선 = 직접 호출 / dequeue    파랑 실선 = enqueue    파랑 점선 = 완료 callback의 enqueue    빨강 점선 = 제어 / 취소    회색 점선 = 상태 읽기와 갱신',23)
    p.text(64,2110,'칸이 있는 박스 = queue    둥근 박스 = 코드    원통 = RAM 상태 또는 DURABLE 기록    접힌 문서 = immutable 메시지. Port는 typed 계약이며 queue와 구별한다.',23)
    p.text(64,2145,'Context Manager, Policy Manager, State Store, Model Access 및 Agent Gateway의 공통 계약 유지. 별도 thread / process 수와 라이브러리는 미선정.',22,GRAY)


def stage(p,key,x,w,name,queue_labels,state_name,state_lines,handler_lines,work_name=None,work_lines=()):
    p.node(key,x,595,w,1060,name,kind='group',owner='rc' if key in ('input','notice') else 'rm',different=True)
    xx=x+30;ww=w-60
    for suffix,y,title,body in queue_labels:
        p.node(key+'_'+suffix,xx,y,ww,110,title,[body],kind='queue',owner=key,different=True)
    p.node(key+'_handler',xx,1100,ww,175,'Pump / Handle(event)',handler_lines,owner=key,different=True)
    p.node(key+'_state',xx,1305,ww,190,state_name,state_lines,kind='store',owner=key,different=True)
    p.edge(key+'_handler',key+'_state','B','T',kind='state')
    rail=x+w-10
    for suffix,*_ in queue_labels:
        cy=p.port(key+'_'+suffix,'R')[1]
        p.edge(key+'_'+suffix,key+'_handler','R','R',via=[(rail,cy),(rail,1187.5)],kind='call')
    if work_name:
        work_key='prepare' if key=='compose' else key+'_work'
        p.node(work_key,xx,1530,ww,95,work_name,work_lines,owner=key)
    return key+'_handler'


def trace(p,labels):
    for i,(title,lines) in enumerate(labels):
        p.node('trace'+str(i),64+i*750,1785,710,190,title,lines,kind='trace',different=i in (1,2,3))


def a():
    p=RuntimeScene('A');common(p);n=p.node
    n('control',660,650,490,145,'Control Lane (bounded)',['InputStarted / adopted Invalidate','별도 수용량 / 제어를 먼저 소비'],kind='queue',owner='rc',different=True)
    n('events',1200,650,500,145,'Event Mailbox (bounded)',['Input / TaskEvent / JobDone / Presented','priority + seq + aging; causal order 유지'],kind='queue',owner='rc',different=True)
    n('dispatcher',660,890,1040,355,'Dialogue Dispatcher',kind='group',owner='rc',different=True)
    n('pump',690,955,420,250,'Pump',['select(Control, Event)','짧은 Handle(event) 호출','긴 Job 완료를 await하지 않음','다음 사건을 계속 소비'],owner='dispatcher',different=True)
    n('handle',1160,955,510,250,'Handle(event)',['Input → Interpret Job 시작','Meaning → 채택 / Prepare Job 지시','Prepared → 조건 확인 / Publish 지시','Invalidate → mark unusable / cancel','stale Done → discard / ref release','Presented → 실제 질문 focus 갱신'],owner='dispatcher',different=True)
    n('jobs',660,1305,490,215,'Job Index (RAM)',['job ID → input / Request revision','read set / Future / CancelHandle','RUNNING / CANCEL_REQUESTED','terminal status + result ref','이 Map은 Handle(event)만 갱신'],kind='store',owner='rc',different=True)
    n('progress',1200,1305,500,215,'Dialogue Progress State (RAM)',['Request ID → continuation / waiting IDs','candidate ref / admission / source refs','후속 실행과 대기 조건을 중앙에 연결','확정 Request와 Task 원본을 대체하지 않음'],kind='store',owner='rc',different=True)
    n('job_msg',660,1550,1040,105,'JobDescriptor (immutable)',['job ID / Request revision / read set / replyTo: Event Mailbox'],kind='data',owner='rc')
    n('compose',1850,595,600,1060,'Response Composer',kind='group',owner='rm')
    n('prepare_q',1890,660,520,110,'Prepare Command Queue',['Dispatcher가 승인한 JobDescriptor'],kind='queue',owner='compose')
    n('cancel_q',1890,795,520,110,'Cancel Control Queue',['중앙에서 지정한 job ID / CancelHandle'],kind='queue',owner='compose')
    n('prepare_adapter',1890,965,520,240,'Job adapter',['Prepare → executor.start(PrepareAsync)','Cancel → provider에 중단 요청','Future callback → 중앙 JobDone enqueue','이 코드는 다음 이해 / 게시를 지시하지 않음','제어 처리는 긴 Job / output credit과 분리'],owner='compose')
    n('prepare_msg',1890,1240,520,160,'Job invocation (immutable)',['J1 / R1 rev1 / source vector','captured replyTo + payload ref','중앙 Job Index의 handle로 취소 요청'],kind='data',owner='compose')
    n('prepare',1890,1460,520,165,'PrepareAsync',['Context Manager / Model Access','내용과 음성 준비; 긴 I/O는 await','완료 자료는 candidate ref로 반환'],owner='compose')
    n('candidate',2520,1305,550,215,'Prepared Candidate (immutable)',['candidate ID / Request revision','content ref / audio ref / read set','Response Manager가 자료를 보유','중앙에는 ref를 반환; 불필요하면 해제'],kind='data',owner='rm')
    p.text(2520,700,'A의 후속 실행 권한',28,bold=True)
    p.text(2520,750,'완료와 조건은 중앙에 반환한다.\nResponse Composer는 Job을 실행하고\nPublication Control은 최종 게시만 맡는다.',25)
    p.text(2520,960,'대기 / 정정 / 늦은 완료의 연결은\nDialogue Progress State와\nJob Index를 갱신하는 중앙 코드에 있다.',25)
    p.edge('control','pump','B','T',sd=-15,td=100,via=[(890,850),(1000,850)],kind='call')
    p.edge('events','pump','B','R',via=[(1450,850),(1130,850),(1130,1080)],kind='call')
    p.edge('pump','handle',kind='call')
    p.edge('handle','jobs','B','T',sd=-170,via=[(1245,1260),(905,1260)],kind='state')
    p.edge('handle','progress','B','T',sd=100,via=[(1515,1278),(1450,1278)],kind='state')
    p.edge('handle','interpret','T','R',sd=-80,via=[(1335,862),(584,862),(584,1540)],kind='call',label='InterpretAsync / cancel(handle)',at=(645,1270))
    p.edge('interpret','events','R','L',via=[(596,1540),(596,818),(1180,818),(1180,722.5)],kind='done',label='JobDone: MeaningProposal',at=(675,815))
    p.edge('handle','prepare_q','R','L',sd=-65,via=[(1760,1015),(1760,715)],kind='post',label='Prepare Job',at=(1750,605))
    p.edge('handle','cancel_q','R','L',sd=45,via=[(1772,1125),(1772,850)],kind='cancel',label='Cancel(J1)',at=(1810,795))
    p.edge('handle','pub_q','R','L',sd=90,via=[(1750,1170),(1750,520),(3130,520),(3130,715)],kind='post',label='PublishRequest: 중앙이 지시',at=(2700,505))
    for q in ('prepare_q','cancel_q'):
        cy=p.port(q,'R')[1];p.edge(q,'prepare_adapter','R','R',via=[(2430,cy),(2430,1085)],kind='call')
    p.edge('prepare_adapter','prepare','L','L',sd=75,via=[(1866,1160),(1866,1542.5)],kind='call')
    p.edge('prepare','events','L','R',sd=35,td=40,via=[(1784,1577.5),(1784,505),(1720,505),(1720,762.5)],kind='done',label='JobDone: candidate ref',at=(2030,500))
    p.edge('prepare','candidate','R','L',via=[(2490,1542.5),(2490,1412.5)],kind='call')
    p.edge('task_emit','events','R','L',via=[(576,975),(576,830),(1188,830),(1188,722.5)],kind='post',label='확인된 TaskEvent',at=(645,860))
    p.edge('input_emit','events','B','T',via=[(1000,486),(1450,486)],kind='post',label='InputReady',at=(1190,485))
    p.edge('turn','control','B','L',via=[(384,470),(604,470),(604,722.5)],kind='cancel',label='InputStarted',at=(670,475))
    p.edge('turn','pub_control','B','L',via=[(384,470),(3120,470),(3120,995)],kind='cancel')
    p.edge('handle','pub_control','R','L',sd=25,via=[(1758,1105),(1758,532),(3104,532),(3104,995)],kind='cancel',label='채택한 Invalidate / 관련 게시 hold',at=(2600,565))
    p.edge('handle','task_command','B','R',sd=-150,via=[(1265,1250),(606,1250),(606,1187.5)],kind='call')
    p.edge('pub_handler','events','R','R',sd=50,td=-40,via=[(3758,1237.5),(3758,1698),(1794,1698),(1794,503),(1728,503),(1728,682.5)],kind='done',label='durable Presented → 중앙 focus 갱신',at=(2370,1708))
    trace(p,[('1 InputStarted',['Interaction Manager: 즉시 stop / epoch++','Dispatcher: 미전송과 관련 게시 hold','Task 취소 여부는 아직 미확정']),
             ('2 R1 rev2 채택',['Handle: J1을 unusable로 표시','중앙 Job Index의 handle로 Cancel 요청','provider 종료 확인까지 J1 추적']),
             ('3 늦은 JobDone(J1)',['완료는 Event Mailbox로 돌아온다','Handle: rev1 결과 폐기 / ref 해제','이미 청구한 비용과 terminal 상태 보존']),
             ('4 새 Prepare Job J2',['Handle이 rev2로 J2 시작을 지시','완료 회수 → 중앙 조건 확인','유효 candidate의 PublishRequest 지시']),
             ('5 공통 최종 전달',['Publication Control: current check / commit','Interaction Manager: epoch 재검사 / 전달','ActualReceipt 저장 뒤 Presented 반환'])])
    return p


def b():
    p=RuntimeScene('B');common(p)
    input_handler=stage(p,'input',660,490,'Input Resolution Stage',[
        ('control',660,'Control Queue','InputStarted / adopted Invalidate'),
        ('data',795,'Input Queue','InputReady / durable Presented'),
        ('done',940,'Done Queue','JobDone: MeaningProposal')],
        'Input Window (RAM)',['input ID → revision / hold','own job: read set / Future / cancel','admission / pending output refs','이 Map은 이 Pump만 갱신'],
        ['control 우선 / Input → InterpretAsync','Done → current check / 의미 채택','채택 → prepare / conditions 별도 전달','Invalidate → 자기 Job cancel'])
    notice_handler=stage(p,'notice',1210,500,'Task Notice Stage',[
        ('control',660,'Control Queue','onset hold / related Invalidate'),
        ('data',795,'Task Event Queue','confirmed Task / question refs'),
        ('done',940,'Done Queue','JobDone: NoticeReady')],
        'Notice Window (RAM)',['Task / question ID → revision','own job / cancel / pending Notice','connection / question / admission refs','중요 원본은 Task Manager에 보존'],
        ['control 우선 / Task → NoticeAsync','Done → 대화 연결 / 질문 등록','admission과 Notice를 별도로 전달','관련된 자기 Job만 cancel'],
        'NoticeAsync',['Context Manager / owner reads'])
    compose_handler=stage(p,'compose',1850,600,'Response Composer',[
        ('control',660,'Control Queue','hold / related Invalidate / cancel'),
        ('data',795,'Prepare Input Queue','AdoptedInput or NoticeReady'),
        ('done',940,'Done Queue','JobDone: PreparedCandidate')],
        'Prepare Job Index (RAM)',['job ID → revision / read set','own Future / CancelHandle / status','pending candidate refs; bounded','출력 credit이 없어도 control 소비'],
        ['control 우선 / Input → PrepareAsync','Done → 자기 revision / read set 검사','유효 Candidate를 다음 queue로 전달','Invalidate → 자기 Job 무효화 / cancel'],
        'PrepareAsync',['Model Access / 내용과 음성 / Future'])
    join_handler=stage(p,'join',2520,550,'Publication Join',[
        ('candidate',660,'Candidate Queue','candidate ID / payload / read set'),
        ('condition',795,'Condition Queue','settled / admission / source / past actual'),
        ('control',940,'Control Queue','hold / related Invalidate')],
        'Publication Window (RAM)',['candidate ID → required ID / revision','WAIT / ELIGIBLE / DISCARDED','역색인: source ref → 대기 candidate IDs','pending eligible refs / deadline / credit'],
        ['control 우선 / 후보와 조건을 select','ID / revision이 일치하는 참조 갱신','조건 충족 → PublishRequest enqueue','부족하면 대기 / 무효는 폐기'])
    p.node('eligible',2550,1530,490,95,'PublishRequest (immutable)',['candidate ref / source vector / admission'],kind='data',owner='join')
    # Long work returns to its initiating Stage, never to a central dispatcher.
    p.edge(input_handler,'interpret','L','R',sd=-25,via=[(578,1162.5),(578,1540)],kind='call',label='InterpretAsync / cancel own handle',at=(650,1278))
    p.edge('interpret','input_done','R','L',via=[(590,1540),(590,995)],kind='done',label='own JobDone',at=(600,925))
    p.edge(notice_handler,'notice_work','R','R',via=[(1690,1187.5),(1690,1577.5)],kind='call')
    p.edge('notice_work','notice_done','R','R',via=[(1730,1577.5),(1730,995)],kind='done')
    p.edge(compose_handler,'prepare','L','L',sd=35,via=[(1866,1222.5),(1866,1577.5)],kind='call')
    p.edge('prepare','compose_done','R','R',via=[(2446,1577.5),(2446,995)],kind='done',label='own JobDone',at=(2290,1520))
    p.edge(compose_handler,'join_candidate','R','L',sd=-20,via=[(2480,1167.5),(2480,715)],kind='post',label='Candidate{ID, rev, refs}',at=(2460,590))
    p.edge(join_handler,'pub_q','R','L',via=[(3116,1187.5),(3116,715)],kind='post',label='eligible만 PublishRequest',at=(2810,605))
    p.edge(input_handler,'compose_data','R','L',sd=55,via=[(1172,1242.5),(1172,532),(1788,532),(1788,850)],kind='post',label='adopted input → prepare port',at=(1820,790))
    p.edge(notice_handler,'compose_data','R','L',sd=55,via=[(1756,1242.5),(1756,850)],kind='post')
    p.edge(input_handler,'join_condition','R','L',sd=-10,via=[(1184,1177.5),(1184,510),(2504,510),(2504,850)],kind='post',label='조건은 별도 queue에 fan-out',at=(2530,790))
    p.edge(notice_handler,'join_condition','R','L',sd=5,via=[(1748,1192.5),(1748,520),(2492,520),(2492,850)],kind='post')
    p.edge('input_emit','input_data','B','L',via=[(1000,496),(604,496),(604,850)],kind='post',label='InputReady',at=(650,790))
    p.edge('task_emit','notice_data','R','R',via=[(576,975),(576,528),(1720,528),(1720,850)],kind='post',label='ConfirmedTaskEvent',at=(1270,790))
    # Separate destinations receive their own copy; a competing-consumer queue is not fan-out.
    for target,rail in [('input_control',604),('notice_control',1194),('compose_control',1782),('join_control',2508),('pub_control',3100)]:
        cy=p.port(target,'L')[1]
        p.edge('turn',target,'B','L',via=[(384,470),(rail,470),(rail,cy)],kind='cancel')
    for target,rail in [('notice_control',1188),('compose_control',1774),('join_control',2498),('pub_control',3108)]:
        cy=p.port(target,'L')[1]
        p.edge(input_handler,target,'R','L',sd=-60,via=[(1168,1127.5),(1168,486),(rail,486),(rail,cy)],kind='cancel')
    p.text(1380,465,'InputStarted fan-out: local stop 후 각 Control Queue에 복사',20,RED,mask=True)
    p.text(2210,485,'채택된 Invalidate: ID / revision / scope를 각 consumer에 전달',20,RED,mask=True)
    p.edge(input_handler,'task_command','L','R',sd=50,via=[(610,1237.5),(610,1187.5)],kind='call')
    p.edge('pub_handler','input_data','R','L',sd=50,via=[(3758,1237.5),(3758,1698),(598,1698),(598,850)],kind='done',label='durable Presented → Input Resolution Stage의 focus 처리',at=(1820,1708))
    p.edge('pub_handler','join_condition','R','R',sd=65,td=30,via=[(3738,1252.5),(3738,1718),(3090,1718),(3090,880)],kind='done',label='과거 확정 delivery snapshot',at=(2560,1690))
    trace(p,[('1 InputStarted',['Interaction Manager: 즉시 stop / epoch++','각 consumer가 자기 hold 처리','control은 가득 찬 출력 queue를 기다리지 않음']),
             ('2 R1 rev2 채택',['Input Stage: scope를 담은 Invalidate 발행','Composer: 자기 J1 무효화 / cancel','Join: 관련 후보의 Window만 폐기']),
             ('3 늦은 JobDone(J1)',['완료는 Composer의 Done Queue로 반환','자기 Job Index: rev1 결과 폐기','기존 후보도 Join / 최종 검사에서 탈락']),
             ('4 rev2를 소비해 J2 시작',['Composer: 자기 PrepareAsync 시작','완료 → Candidate Queue; 조건은 별도 도착','Join: 참조 일치 / ready → PublishRequest']),
             ('5 공통 최종 전달',['Publication Control: current check / commit','Interaction Manager: epoch 재검사 / 전달','receipt 내구 저장 → focus / 과거 snapshot'])])
    return p
