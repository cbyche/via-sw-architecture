"""44's documentation scene: ten Components, contained Modules and owned state.

Layout routing is only for diagrams. No candidate, model or executor is run.
"""
import heapq
import math
import re
import unicodedata

from dp_comparison_structures import COMMON, APRICOT, DIFFERENCE_STROKE, DIFFERENCE_WIDTH

COMPONENTS = ('Interaction Manager', 'Request Controller', 'Context Manager',
              'Request Interpreter', 'Task Manager', 'Agent Gateway',
              'Response Manager', 'Policy Manager', 'State Store', 'Model Access')
OWNERS = {'channel':'im', 'evidence':'im', 'timeline':'im', 'turn':'im', 'playback':'im',
          'dispatch':'rc', 'progress':'rc', 'input':'rc', 'iw':'rc', 'notice':'rc', 'nw':'rc',
          'compose':'rm', 'join':'rm', 'pw':'rm', 'publish':'rm', 'outbox':'rm'}


def stable(xml):
    return re.sub(r'(?<![\w#])-?\d+\.\d{7,}(?!\w)',
                  lambda m: f'{float(m[0]):.6f}'.rstrip('0').rstrip('.'), xml)


class Option:
    def __init__(self, slide, x, y, scale, side):
        self.s, self.x, self.y, self.scale, self.side = slide, x, y, scale, side
        self.nodes, self.labels, self.lines = {}, [], []

    def text(self, x, y, w, value, size=18, bold=False, owner=None, center=False):
        background=None
        if not owner:
            lines=value.replace('·',' / ').split('\n')
            width=min(w,max(math.fsum(size*(1 if unicodedata.east_asian_width(ch) in 'WF' else .58) for ch in line) for line in lines)+4)
            background=self.s.rect(self.x+((x-width/2 if center else x)-2)*self.scale,self.y+(y-1)*self.scale,width*self.scale,(len(lines)*size*1.17+3)*self.scale,'white','none')
        self.s.text(self.x+x*self.scale, self.y+y*self.scale, w*self.scale,
                    value.replace('·',' / ').split('\n'), size*self.scale, COMMON, bold,
                    'center' if center else 'left', leading=size*1.17*self.scale)
        if background:
            next(i for i in self.s.items if i['id']==background)['label_for']=self.s.items[-1]['id']
        self.s.items[-1]['option']=self.side
        if owner:self.s.items[-1]['owner']=self.nodes[owner]['id']

    def node(self, key, x, y, w, h, name, kind='component', different=False, owner=None, size=22):
        ident=self.s.add('store' if kind=='state' else 'rect', x=self.x+x*self.scale,
                         y=self.y+y*self.scale, w=w*self.scale, h=h*self.scale,
                         fill=APRICOT if different else 'white',
                         stroke=DIFFERENCE_STROKE if different else COMMON,
                         dashed=False, line_width=(DIFFERENCE_WIDTH if different else 1.0)*self.scale,
                         rounded=kind=='module', store_radius=min(8*self.scale,h*self.scale/6), semantic_kind=kind,
                         logical_name=name.replace('\n',' '), option=self.side)
        self.nodes[key]=dict(id=ident,x=x,y=y,w=w,h=h,kind=kind,owner=owner,name=name.replace('\n',' '))
        if owner:self.s.items[-1]['owner']=self.nodes[owner]['id']
        if kind=='external':self.s.items[-1]['external']=True
        if kind=='component' and h>100:
            self.text(x+12,y+9,w-24,name,size,True,owner=key)
        else:
            count=len(name.split('\n')); top=y+(h-count*size*1.17)/2+(4 if kind=='state' else 0)
            self.text(x+w/2,top,w-14,name,size,True,owner=owner or key,center=True)
        return ident

    def port(self,key,side,offset=0):
        n=self.nodes[key];x,y,w,h=(n[k] for k in ('x','y','w','h'))
        return {'L':(x,y+h/2+offset),'R':(x+w,y+h/2+offset),
                'T':(x+w/2+offset,y),'B':(x+w/2+offset,y+h)}[side]

    def route(self,a,b,sp,tp,sd=0,td=0):
        start,end=self.port(a,sp,sd),self.port(b,tp,td)
        exempt=set()
        for key in (a,b):
            while self.nodes[key]['owner']:
                key=self.nodes[key]['owner'];exempt.add(key)
        obstacles=[n for key,n in self.nodes.items() if key not in exempt]
        obstacles += [dict(n,h=50) for n in self.nodes.values() if n['kind']=='component' and n['h']>100]
        # A rectilinear visibility grid keeps routes out of unrelated owners.
        xs={start[0],end[0],5,1155};ys={start[1],end[1],-10,735}
        for n in self.nodes.values():
            xs.update((n['x']-9,n['x']+n['w']+9));ys.update((n['y']-9,n['y']+n['h']+9))
        xs=sorted(v for v in xs if 0<=v<=1160);ys=sorted(v for v in ys if -50<=v<=740)
        def blocked(p,q):
            for n in obstacles:
                x,y,w,h=(n[k] for k in ('x','y','w','h'))
                if p[0]==q[0] and x+.1<p[0]<x+w-.1 and max(min(p[1],q[1]),y+.1)<min(max(p[1],q[1]),y+h-.1):return True
                if p[1]==q[1] and y+.1<p[1]<y+h-.1 and max(min(p[0],q[0]),x+.1)<min(max(p[0],q[0]),x+w-.1):return True
            return False
        first=(xs.index(start[0]),ys.index(start[1]),'')
        queue=[(0,0,first)];costs={first:0};previous={};found=None
        while queue:
            _,cost,state=heapq.heappop(queue);ix,iy,direction=state
            if cost!=costs[state]:continue
            if (xs[ix],ys[iy])==end:found=state;break
            for dx,dy,axis in ((1,0,'x'),(-1,0,'x'),(0,1,'y'),(0,-1,'y')):
                nx,ny=ix+dx,iy+dy
                if not(0<=nx<len(xs) and 0<=ny<len(ys)):continue
                p,q=(xs[ix],ys[iy]),(xs[nx],ys[ny])
                if blocked(p,q):continue
                new=(nx,ny,axis);nc=cost+abs(p[0]-q[0])+abs(p[1]-q[1])+(24 if direction and direction!=axis else 0)
                if nc>=costs.get(new,math.inf):continue
                costs[new]=nc;previous[new]=state
                heuristic=abs(q[0]-end[0])+abs(q[1]-end[1])
                heapq.heappush(queue,(nc+heuristic,nc,new))
        assert found is not None,(a,b,'no diagram route')
        points=[]
        while True:
            points.append((xs[found[0]],ys[found[1]]))
            if found==first:break
            found=previous[found]
        points.reverse();out=[]
        for p in points:
            if len(out)>1 and (out[-2][0]==out[-1][0]==p[0] or out[-2][1]==out[-1][1]==p[1]):out[-1]=p
            else:out.append(p)
        return out

    def edge(self,a,b,label='',sp='B',tp='T',ret=False,sd=0,td=0,at=None):
        pts=self.route(a,b,sp,tp,sd,td)
        self.s.line([(self.x+px*self.scale,self.y+py*self.scale) for px,py in pts],
                    color=COMMON,width=1.1*self.scale,dashed=ret)
        self.s.items[-1].update(source=self.nodes[a]['id'],target=self.nodes[b]['id'],option=self.side)
        self.lines.append((a,b,pts,label,ret))
        if label and at:self.labels.append((*at,label))

    def finish(self):
        for x,y,w,value in self.labels:
            # Labels are outside name boxes and erase only their own route area.
            self.text(x,y,w,value,17)
        components=[n for n in self.nodes.values() if n['kind']=='component']
        assert len(components)==10 and {n['name'] for n in components}==set(COMPONENTS)
        edges={(a,b) for a,b,_,_,_ in self.lines}
        receiver='dispatch' if self.side=='A' else 'input'
        notice='dispatch' if self.side=='A' else 'notice'
        required={('timeline',receiver),('im',receiver),(receiver,'cm'),('cm',receiver),
                  (receiver,'ri'),('ri',receiver),('rc','task'),('task','gateway'),
                  ('gateway','agent'),('agent','gateway'),('gateway','task'),('task',notice),
                  ('publish','turn'),('channel','publish'),('publish','outbox'),('outbox','rc'),
                  ('outbox','state'),('ri','ma'),('compose','ma'),('im','ma'),('ma','omni')}
        required|=({('dispatch','compose'),('compose','dispatch'),('dispatch','publish')}
                   if self.side=='A' else {('input','compose'),('notice','compose'),
                                          ('compose','join'),('rc','join'),('outbox','join'),
                                          ('join','publish'),('join','input'),('join','notice')})
        assert required<=edges,(self.side,'missing documentation flow',required-edges)
        for key,n in self.nodes.items():
            if n['kind'] in ('module','state'):
                assert n['owner']==OWNERS[key],(key,n['owner'])
                p=self.nodes[n['owner']]
                assert p['x']<=n['x'] and p['y']+38<=n['y'] and n['x']+n['w']<=p['x']+p['w'] and n['y']+n['h']<=p['y']+p['h'],key


def draw_option(s,x,y,scale,side):
    g=Option(s,x,y,scale,side)
    g.node('user',20,-45,170,40,'사용자',kind='external',size=20)
    g.node('im',20,20,340,280,'Interaction Manager',size=25)
    g.node('channel',35,78,145,42,'Channel I/O','module',owner='im',size=19)
    g.node('turn',198,78,145,42,'Turn-Taking\nControl','module',owner='im',size=18)
    g.node('evidence',35,160,145,44,'Evidence\nCapture','module',owner='im',size=18)
    g.node('timeline',198,160,145,44,'Timeline &\nBuffer','module',owner='im',size=18)
    g.node('playback',105,235,175,52,'Playback State','state',owner='im',size=18)
    g.node('rc',450,20,690,280,'Request Controller',size=25)
    if side=='A':
        g.node('dispatch',470,82,280,60,'Dialogue Dispatcher','module',True,'rc',22)
        g.node('progress',810,82,305,70,'Dialogue Progress State','state',True,'rc',20)
    else:
        g.node('input',470,82,280,54,'Input Resolution Stage','module',True,'rc',21)
        g.node('iw',810,82,305,60,'Input Window','state',True,'rc',20)
        g.node('notice',470,197,280,54,'Task Notice Stage','module',True,'rc',21)
        g.node('nw',810,197,305,60,'Notice Window','state',True,'rc',20)
    g.node('cm',20,375,260,54,'Context Manager',size=22)
    g.node('ri',20,463,260,54,'Request Interpreter',size=21)
    g.node('rm',430,375,450,255,'Response Manager',size=25)
    g.node('compose',445,430,190,55,'Response\nComposer','module',owner='rm',size=21)
    if side=='B':
        g.node('join',675,430,190,55,'Publication Join','module',True,'rm',19)
        g.node('pw',675,501,190,54,'Publication Window','state',True,'rm',16)
    g.node('publish',445,562,190,54,'Publication\nControl','module',owner='rm',size=21)
    g.node('outbox',675,562,190,54,'Publication Outbox','state',owner='rm',size=17)
    g.node('task',950,375,190,54,'Task Manager',size=22)
    g.node('gateway',950,463,190,54,'Agent Gateway',size=22)
    g.node('agent',950,550,190,54,'Downstream Agent','external',size=19)
    g.node('policy',20,582,260,54,'Policy Manager',size=22)
    g.node('state',20,677,260,54,'State Store',size=22)
    g.node('ma',950,642,190,54,'Model Access',size=22)
    g.node('omni',950,720,190,42,'Shared Omni','external',size=20)
    # The same ordinary input, interpretation, task and delivery contracts.
    g.edge('user','channel',sp='B',tp='T')
    g.edge('turn','channel',sp='L',tp='R')
    g.text(36,127,320,'InputStarted: 즉시 stop / epoch 무효화',16)
    g.edge('evidence','timeline',sp='R',tp='L')
    g.text(35,211,300,'시점별 evidence ↔ 입력 revision',16)
    g.edge('channel','playback')
    receiver='dispatch' if side=='A' else 'input'
    g.edge('timeline',receiver,'1 입력·revision·evidence',sp='R',tp='L',at=(35,308,345))
    g.edge('channel',receiver,'InputStarted',sp='R',tp='L',sd=-7,td=-12,at=(365,-20,300))
    g.edge('im',receiver,'Direct 후보 → 좁은 admission',sp='R',tp='L',sd=-1,td=7,at=(366,159,330))
    g.edge(receiver,'cm','2 허용된 근거 조회',sp='L',tp='T',sd=12,at=(22,335,240))
    g.edge('cm',receiver,sp='R',tp='L',ret=True,td=18)
    g.edge(receiver,'ri','3 입력·근거·revision·예산',sp='L',tp='T',sd=23,at=(22,437,335))
    g.edge('ri',receiver,'의미 제안 / 추가 근거 요구',sp='R',tp='L',ret=True,td=24,at=(22,531,340))
    g.edge('rc','task','4 채택 의미 / hold·admission',sp='R',tp='T',sd=90,at=(817,341,338))
    g.edge('task','gateway','명령·hold / 반환 사건',at=(916,436,240))
    g.edge('gateway','agent','전송 / 외부 접수·질문·결과',at=(924,523,228))
    g.edge('gateway','task',sp='T',tp='B',ret=True,sd=40,td=40)
    g.edge('agent','gateway',sp='T',tp='B',ret=True,sd=40,td=40)
    notice='dispatch' if side=='A' else 'notice'
    g.edge('task',notice,'Q1: 확인·저장 후 대화 연결',sp='L',tp='R',ret=True,td=15,at=(847,313,308))
    g.edge('publish','turn','7 release / P2·Q1 identity',sp='L',tp='B',at=(410,643,345))
    g.edge('channel','publish','실제 전달·중단 receipt / actual range',sp='L',tp='L',ret=True,td=15,at=(18,649,600))
    g.edge('publish','outbox','전달 기록 갱신',sp='R',tp='L',at=(653,621,213))
    g.edge('outbox','rc','8 확인된 제시 → 질문 focus',sp='R',tp='B',ret=True,td=30,at=(451,351,345))
    if side=='A':
        g.edge('dispatch','progress','job / revision / 대기 참조',sp='R',tp='L',at=(770,162,365))
        g.edge('progress','dispatch',sp='L',tp='R',ret=True,sd=15,td=15)
        g.edge('dispatch','compose','5 채택 의미·허용 사실 / job ID',at=(448,323,315))
        g.edge('compose','dispatch','후보 반환 / 시작 revision',sp='T',tp='B',ret=True,sd=40,td=40,at=(446,493,330))
        g.edge('dispatch','publish','6 유효 후보 → 게시 요청',sp='B',tp='L',sd=-90,at=(447,536,332))
    else:
        g.edge('input','iw','입력 / job / revision',sp='R',tp='L',at=(818,151,310))
        g.edge('iw','input',sp='L',tp='R',ret=True,sd=13,td=13)
        g.edge('notice','nw','Q1 참조 / 준비·대기',sp='R',tp='L',at=(818,261,300))
        g.edge('nw','notice',sp='L',tp='R',ret=True,sd=13,td=13)
        g.edge('input','compose','5 채택 의미·허용 Notice 소비',at=(448,323,315))
        g.edge('notice','compose',sp='B',tp='T',sd=70,td=45)
        g.edge('compose','join','후보 / Direct',sp='R',tp='L',at=(660,408,225))
        g.edge('join','pw',sp='B',tp='T')
        g.edge('pw','join',sp='T',tp='B',ret=True,sd=40,td=40)
        g.edge('join','publish','6 조건 충족 / 대기 / 폐기',sp='L',tp='T',at=(445,533,225))
        g.edge('rc','join',sp='B',tp='T',td=60)
        g.edge('outbox','join',sp='R',tp='R',ret=True)
        g.edge('join','compose',sp='L',tp='R',ret=True,sd=17,td=17)
        g.edge('join','input','credit / cancel: 처리량 조절',sp='T',tp='R',ret=True,sd=30,at=(752,287,390))
        g.edge('join','notice',sp='T',tp='B',ret=True,sd=-30,td=30)
        g.edge('publish','join',sp='T',tp='B',ret=True,sd=-30,td=-30)
    # Auxiliary access is explicit without introducing new infrastructure boxes.
    g.edge('publish','rc',sp='R',tp='B',sd=-12,td=-30)
    g.edge('rc','publish',sp='B',tp='R',ret=True,sd=-10,td=-20)
    g.edge('publish','task',sp='R',tp='L',sd=10)
    g.edge('task','publish',sp='L',tp='R',ret=True,sd=10,td=15)
    g.edge('publish','policy',sp='L',tp='R',sd=-10,td=-10)
    g.edge('rc','policy',sp='L',tp='R',sd=100)
    g.edge('policy','rc',sp='R',tp='L',ret=True,sd=15,td=110)
    g.edge('outbox','state',sp='B',tp='R')
    g.edge('state','outbox',sp='R',tp='B',ret=True,sd=15,td=30)
    g.edge('ri','ma',sp='B',tp='L')
    g.edge('compose','ma',sp='B',tp='L',sd=-30,td=15)
    g.edge('im','ma',sp='R',tp='L',sd=110,td=-15)
    g.edge('ma','ri',sp='L',tp='B',ret=True,sd=10,td=30)
    g.edge('ma','compose',sp='L',tp='B',ret=True,sd=-10,td=30)
    g.edge('ma','im',sp='L',tp='R',ret=True,sd=-20,td=100)
    g.edge('ma','omni',sp='B',tp='T')
    g.edge('omni','ma',sp='T',tp='B',ret=True,sd=45,td=45)
    g.finish()
    return g


def decorate_svg(slide,svg):
    for i in slide.items:
        if i.get('external'):
            x,y,w,h=(i[k] for k in ('x','y','w','h'));d=12*max(.7,w/190)
            shape=f'<path id="{i["id"]}" d="M{x+d} {y}H{x+w-d}L{x+w} {y+h/2}L{x+w-d} {y+h}H{x+d}L{x} {y+h/2}Z" fill="white" stroke="{COMMON}" stroke-width="{i["line_width"]}"/>'
            svg=re.sub(r'<rect id="'+i['id']+r'"[^>]*/>',lambda _:shape,svg)
        if i.get('label_for'):
            svg=svg.replace(f'id="{i["id"]}"',f'id="{i["id"]}" data-label-for="{i["label_for"]}"')
        if i.get('semantic_kind') or i.get('owner'):
            attrs=f' data-kind="{i.get("semantic_kind","text")}"'+(f' data-owner="{i["owner"]}"' if i.get('owner') else '')
            svg=svg.replace(f'id="{i["id"]}"',f'id="{i["id"]}"'+attrs)
    return stable(svg)


def decorate_diagram(slide,d):
    byid={i['id']:i for i in slide.items}
    for cell in d.findall('.//mxCell'):
        i=byid.get(cell.get('id'))
        if not i:continue
        owner=i.get('owner')
        if owner:
            assert owner!=i['id']
            cell.set('parent',owner);g=cell.find('mxGeometry');p=byid[owner]
            for axis in ('x','y'):g.set(axis,str(float(g.get(axis))-p[axis]))
        if i.get('external'):cell.set('style',cell.get('style').replace('rounded=0;','shape=hexagon;'))
        if i['kind']=='line' and 'source' in i:
            st=cell.get('style')
            for role,pt,prefix in [('source',i['points'][0],'exit'),('target',i['points'][-1],'entry')]:
                key=i[role];n=byid[key];cell.set(role,key)
                st+=f'{prefix}X={(pt[0]-n["x"])/n["w"]};{prefix}Y={(pt[1]-n["y"])/n["h"]};{prefix}Perimeter=0;'
            cell.set('style',st)
    return d


def legend(s,x=64,y=1368,scale=1):
    # 41/42 symbols and colors, with external dependencies added for 44.
    g=Option(s,x,y,scale,'legend')
    for key,xx,name,kind,diff in [('c',120,'Component','component',False),('m',325,'Module','module',False),
                                ('st',520,'State','state',False),('d',1250,'Module','module',True),
                                ('ext',1750,'Dependency','external',False)]:
        g.node(key,xx,0,150,32,name,kind,diff,size=16)
    g.text(0,3,110,'Legend',20,True)
    for xx,label in [(120,'논리 책임'),(325,'내부 Module'),(520,'데이터 상태'),(1250,'차이: 살구색 / 짙은 테두리'),(1750,'외부 Agent·모델')]:
        g.text(xx,39,420,label,17)
    s.line([(x+740*scale,y+16*scale),(x+890*scale,y+16*scale)],color=COMMON,width=1.1*scale)
    g.text(740,39,220,'요청 / 전달',17)
    s.line([(x+980*scale,y+16*scale),(x+1130*scale,y+16*scale)],color=COMMON,width=1.1*scale,dashed=True)
    g.text(980,39,220,'응답 / 반환',17)
    g.text(1500,3,235,'공통: 흰색 / 검정',17)
    g.text(2010,3,420,'좌우 칸: 배치용 레이아웃',17)
    g.text(2010,39,420,'실행·서비스 경계 추가 없음',17)


def compact_legend(s):
    g=Option(s,53,292,1,'legend')
    s.rect(47,277,86,512,'white',COMMON);s.items[-1]['line_width']=1.0
    g.text(37,0,80,'Legend',14,True,center=True)
    for key,y,name,label,kind,diff in [
        ('c',35,'Component','Component','component',False),
        ('m',100,'Module','내부 Module','module',False),
        ('st',165,'State','데이터 상태','state',False),
        ('d',350,'Module','차이: 살구색','module',True),
        ('ex',415,'External','외부 의존성','external',False)]:
        g.node(key,0,y,74,28,name,kind,diff,size=10)
        g.text(0,y+34,80,label,11)
    for y,ret,label in [(237,False,'요청 / 전달'),(300,True,'응답 / 반환')]:
        s.line([(54,292+y),(123,292+y)],color=COMMON,width=1.1,dashed=ret)
        g.text(0,y+10,80,label,11)
    g.text(0,474,80,'공통: 흰색\n검정 테두리',10)
