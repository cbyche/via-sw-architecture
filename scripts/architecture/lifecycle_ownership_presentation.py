"""42 MAIN preserves full owner/commit/lifetime collaboration independently of the focused comparison.

The MAIN view enlarges the structure without the presentation's assumed QA rows.
This is documentation generation, not a candidate implementation.
"""
import copy
import re
import xml.etree.ElementTree as ET
from generate_dp_comparison_slides import Comparison, DATA
from dp_comparison_structures import ResolutionGraph, resolution_legend


def main_graph42(s,x,side):
    g=ResolutionGraph(s,x)
    if side==0: resolution_legend(s)
    g.text(24,251,820,'Q1 “상반기로 할까요?”를 P1로 제시 → u2 “응, 상반기로”',size=16)
    for y,title,desc,end in [(278,'Conversation C1','입력과 실제 응답의 연결 유지',833),
                             (297,'Request R1 / R2 / R3','최초 위임 / Q1 답변 / 결과 수정',740),
                             (316,'Task T1','보고서 업무와 결과 유지',833),
                             (335,'Execution / Question','X1 완료 → X2 / Q1 제시 → 답변',815)]:
        g.text(24,y,230,title,size=15)
        g.raw([(268,y+16),(end,y+16)],arrow=False)
        g.text(281,y-1,555,desc,size=14)
    g.text(281,353,555,'강조 경계: Core/서비스 실행 영역 / 박스: 논리 Component',size=12)
    if side==0:
        g.group(20,367,825,346,'VIA Core',execution=True,side=side)
        g.text(292,376,540,'같은 실행 경계 / 관련 변경의 공동 확정',size=15)
    else:
        g.group(20,367,420,346,'대화 서비스',execution=True,side=side)
        g.group(490,367,355,346,'업무 서비스',execution=True,side=side)
        g.text(190,377,230,'독립 상태 / 실행 수명',size=14)
        g.text(650,377,190,'독립 상태 / 실행 수명',size=14)
    g.box('controller',40,411,235,36,'Request Controller',size=18)
    g.group(550,402,280,153,'Task Manager')
    g.nodes['tasks']=(555,411,270,36)
    g.box('gate',560,450,260,24,'업무 접수·전송 게이트',module=True,size=14)
    g.box('interpreter',40,485,235,34,'Request Interpreter',size=17)
    g.box('response',40,559,235,36,'Response Manager',size=18)
    g.group(550,558,280,58,'Agent Gateway')
    g.nodes['gateway']=(555,559,270,36)
    g.box('transmit',560,596,260,18,'명령 전송·확인',module=True,size=12)
    g.text(43,450,300,'C1 / R2 / u2 → Q1 → T1 / P1 참조',size=14)
    g.text(558,480,265,'조회: T1 / X1 / Q1=OPEN,v7',size=12)
    # Same state owners and bounded semantic judgment in both options.
    g.edge('controller','tasks','R','T',sd=-9,td=-100,via=[(295,420),(295,401),(590,401)])
    g.edge('tasks','controller','T','R',sd=-85,td=-2,via=[(605,407),(285,407),(285,427)],ret=True)
    g.text(301,400,238,'질문 조회 → / ← Q1,v7',size=13)
    g.edge('controller','tasks','R','L',sd=10,td=10)
    g.text(302,428,240,'K1: Q1 답변=상반기 / v7',size=13)
    g.edge('tasks','controller','L','R',sd=-9,td=-9,via=[(533,420),(533,472),(292,472),(292,420)],ret=True)
    g.text(302,454,240,'외부 접수 / 질문 / 결과',size=14)
    g.edge('controller','interpreter',sd=-45,td=-45)
    g.edge('interpreter','controller',sd=45,td=45,ret=True)
    g.text(43,522,270,'u2 + 실제 제시 P1 → 답변 의미 제안',size=13)
    # Output content/admission and delivery/focus are distinct contracts.
    g.edge('controller','response','L','T',td=-60,via=[(30,429),(30,550),(97.5,550)])
    g.edge('response','controller','R','R',sd=-10,td=2,via=[(315,567),(315,431)],ret=True)
    g.text(321,506,218,'P2: 상반기 전달 확인\n게시 허용 / 실제 제시 반환',size=12)
    g.text(44,600,290,'P1→Q1 실제 제시 / P2 전달 원장',size=14)
    g.edge('gate','gateway')
    g.edge('gateway','gate','L','L',via=[(542,577),(542,462)],ret=True)
    g.text(558,500,265,'접수: Q1=ANSWERED,v8\nK1=PENDING / hold 상태 검사',size=12)
    g.text(558,536,265,'TM gate → AG CAS → 외부 전송',size=10)
    if side==0:
        g.group(320,605,230,103,'State Store')
        g.box('commit',325,649,220,24,'트랜잭션 관리자',True,True,size=15)
        g.store('saved',345,686,200,19,'저장 commit 상태',size=11)
        s.items[-1]['y']=689
        g.edge('controller','commit','R','T',sd=16,td=-55,via=[(305,445),(305,641),(380,641)])
        g.edge('tasks','commit','L','T',sd=16,td=55,via=[(550,445),(550,641),(490,641)])
        g.edge('commit','saved',via=[(435,680),(445,680)])
        g.edge('saved','commit',sd=55,td=55,via=[(500,679),(490,679)],ret=True)
        g.text(43,631,275,'R2↔Q1↔T1 연결과 K1을\n관련 owner 변경으로 함께 확정',size=14)
        g.text(558,631,265,'성공: 둘 다 반영\n충돌 / 실패: 둘 다 미반영',size=14)
        g.edge('commit','controller','L','R',td=15,via=[(299,661),(299,444)],ret=True)
        g.edge('commit','tasks','R','L',td=15,via=[(552,661),(552,444)],ret=True)
    else:
        g.group(32,657,395,53,'State Store')
        s.items[-1].update(size=14,leading=18)
        g.group(502,657,331,53,'State Store')
        s.items[-1].update(size=14,leading=18)
        g.store('dsaved',40,689,380,18,'대화 저장 scope / 로컬 tx',size=11)
        s.items[-1]['y']=692
        g.store('tsaved',510,689,315,18,'업무 저장 scope / 로컬 tx',size=11)
        s.items[-1]['y']=692
        g.edge('controller','dsaved','R','T',sd=16,td=50,via=[(303,445),(303,661),(280,661)])
        g.edge('tasks','tsaved','L','T',sd=16,td=-105,via=[(501,445),(501,661),(562.5,661)])
        g.edge('gate','controller','L','B',td=60,via=[(535,462),(535,477),(217.5,477)],ret=True)
        # Explicit intermediate state, followed by reflection of the receipt.
        g.text(43,620,371,'선저장: R2=접수 대기 / K1\n후반영: K1 내부 접수 확인',size=14)
        g.text(515,620,312,'Q1 변경 + K1 + 내부 접수 결과\n업무의 로컬 transaction',size=14)
        g.text(322,486,215,'K1 접수 결과 / hold ACK',size=12)
    # Response Manager owns the publication journal, not the Controller log.
    target='saved' if side==0 else 'dsaved'
    g.edge('response',target,'L','T',td=-100,via=[(34,577),(34,680),(g.port(target,'T',-100)[0],680)])
    g.box('interaction',40,737,235,32,'Interaction Manager',size=17)
    g.box('model',345,737,230,32,'Model Access',size=17)
    g.box('agent',600,737,245,32,'Downstream Agent',size=17)
    s.items[-2]['external']=True
    g.edge('interaction','controller','L','L',via=[(10,753),(10,429)])
    g.edge('response','interaction','L','T',sd=10,td=-30,via=[(24,587),(24,724),(127.5,724)])
    if side==0:
        g.edge('interaction','response','R','R',sd=7,td=10,via=[(284,760),(284,587)],ret=True)
    else:
        g.edge('interaction','response','R','L',sd=7,td=-10,via=[(286,760),(286,730),(28,730),(28,567)],ret=True)
    g.text(302,705,534,'release(P2,epoch) / receipt(P2,실제 전달 범위)',size=13)
    c1,c2,c3,c4=(318,313,310,312) if side==0 else (450,456,462,468)
    g.edge('interpreter','model','R','T',td=-70,via=[(c1,502),(c1,725),(390,725)])
    g.edge('model','interpreter','T','R',sd=-50,td=10,via=[(410,729),(c2,729),(c2,512)],ret=True)
    g.edge('response','model','R','T',sd=4,td=30,via=[(c3,581),(c3,719),(490,719)])
    g.edge('model','response','T','R',sd=50,td=10,via=[(510,723),(c4,723),(c4,587)],ret=True)
    g.edge('gateway','agent','R','T',td=-25,via=[(851,577),(851,719),(697.5,719)])
    g.edge('agent','gateway','T','R',sd=25,td=10,via=[(747.5,726),(858,726),(858,587)],ret=True)
    g.text(43,716,231,'u2 입력 / Text / Voice 실제 전달',size=13)
    g.box('cloud',345,772,230,20,'Cloud 음성/의미 모델',size=10)
    s.items[-2]['external']=True
    g.raw([(460,769),(460,772)],arrow=False)
    g.text(603,772,245,'K1 전송 / 외부 접수 확인',size=13)
    g.text(43,772,235,'로컬 VAD / 발화 종료 ≠ 전사 완료',size=12)
    # Check straight routes and unrelated actor interiors before export.
    for item in s.items[g.first:]:
        if item['kind']!='line': continue
        pts=[(xx-x,yy) for xx,yy in item['points']]
        for start,end in zip(pts,pts[1:]):
            assert start[0]==end[0] or start[1]==end[1],(side,item['id'],'non-orthogonal')
            for key,(nx,ny,nw,nh) in g.nodes.items():
                if any(nx<=px<=nx+nw and ny<=py<=ny+nh for px,py in (pts[0],pts[-1])): continue
                vertical=start[0]==end[0] and nx+1<start[0]<nx+nw-1 and max(min(start[1],end[1]),ny+1)<min(max(start[1],end[1]),ny+nh-1)
                horizontal=start[1]==end[1] and ny+1<start[1]<ny+nh-1 and max(min(start[0],end[0]),nx+1)<min(max(start[0],end[0]),nx+nw-1)
                assert not(vertical or horizontal),(side,item['id'],'crosses',key)
    g.finish()
    if not hasattr(s,'lifecycle_graphs'): s.lifecycle_graphs=[]
    s.lifecycle_graphs.append(g)


def stable_numbers(xml):
    """Keep subpixel XML coordinates identical across Python float summations."""
    return re.sub(r'(?<![\w#])-?\d+\.\d{7,}(?!\w)',
                  lambda m: f'{float(m[0]):.6f}'.rstrip('0').rstrip('.'), xml)


class MainSlide(Comparison):
    def __init__(self):
        super().__init__(42)
        self.items=[]
        self.slug='choice42-structure'
        self.title='대화와 업무 상태의 소유 및 확정'
        self.caption='04-42. '+self.title
        self.rect(0,0,1920,1080,'white','none')
        self.text(40,25,1500,['VIA / 04-42 / 같은 기능과 다른 상태 확정 경계'],20,'#000000',True)
        self.text(1880,25,300,['A/B 미선정 / 미측정'],19,'#000000',align='right')
        self.text(40,70,1840,[self.caption],39,'#000000',True)
        self.text(40,131,1840,['같은 답변: 실제 제시된 보고서 질문 Q1에 “응, 상반기로”라고 답한다'],24,'#000000')
        self.rect(40,188,1840,55,'#D5E7C7','#000000');self.items[-1]['line_width']=1.0
        for side,x in enumerate((140,1010)):
            self.text(x+435,203,838,[DATA[42]['options'][side]],27,'#000000',True,'center')
        self.text(90,260,98,['구조'],20,'#000000',True,'center')
        first=len(self.items)
        for side,x in enumerate((140,1010)):main_graph42(self,x,side)
        # Expand the structure vertically on the standalone plate.
        for item in self.items[first:]:
            if 'y' in item:item['y']=250+(item['y']-243)*1.25
            if 'h' in item:item['h']*=1.25
            if 'points' in item:item['points']=[(x,250+(y-243)*1.25) for x,y in item['points']]
        self.text(40,984,1840,['u2: 답변 입력 / R2: 답변 요청 / Q1: 보고서 질문 / T1: 업무 / X1: 외부 실행 / P1·P2: 게시 / K1: 명령'],20,'#000000')
        self.text(40,1019,1840,['육각형: 외부 모델·Agent / 내부 접수 ≠ 외부 접수 / B: hold 요청 ≠ 업무 gate 적용 / 세부 프로토콜은 본문 §4.2'],18,'#000000')
        # Author at comparison resolution, export the architecture's 2560×1440.
        self.items=copy.deepcopy(self.items)
        for item in self.items:
            for key in ('x','y','w','h','size','leading','width','line_width'):
                if key in item:item[key]*=4/3
            if 'points' in item:item['points']=[(x*4/3,y*4/3) for x,y in item['points']]

    def svg(self):
        rendered=super().svg().replace('width="1920" height="1080" viewBox="0 0 1920 1080"',
                                     'width="2560" height="1440" viewBox="0 0 2560 1440"')
        for item in self.items:
            if not item.get('external'):continue
            x,y,w,h=(item[k] for k in ('x','y','w','h'))
            poly=f'<polygon id="{item["id"]}" points="{x+12},{y} {x+w-12},{y} {x+w},{y+h/2} {x+w-12},{y+h} {x+12},{y+h} {x},{y+h/2}" fill="white" stroke="#000000" stroke-width="1.3"/>'
            rendered=re.sub(r'<rect id="'+item['id']+r'"[^>]*/>',lambda _:poly,rendered)
        return stable_numbers(rendered)

    def drawio(self):
        return stable_numbers(super().drawio())

    def diagram(self):
        d=super().diagram();m=d.find('mxGraphModel')
        m.set('pageWidth','2560');m.set('pageHeight','1440')
        external={i['id'] for i in self.items if i.get('external')}
        for cell in d.findall('.//mxCell'):
            if cell.get('id') in external:cell.set('style',cell.get('style')+'shape=hexagon;')
        return d

    def validate(self):
        ids=[i['id'] for i in self.items]
        assert len(ids)==len(set(ids)),'Duplicate IDs'
        for item in self.items:
            if item['kind'] in ('rect','store'):
                assert 0<=item['x']<=item['x']+item['w']<=2560
                assert 0<=item['y']<=item['y']+item['h']<=1440
            if item['kind']=='line':
                for a,b in zip(item['points'],item['points'][1:]):
                    assert a[0]==b[0] or a[1]==b[1],('Non-orthogonal',item['id'],a,b)
                    assert 0<=a[0]<=2560 and 0<=a[1]<=1440
        text=' '.join(line for i in self.items if i['kind']=='text' for line in i['lines'])
        for name in ('Interaction Manager','Request Controller','Request Interpreter','Response Manager','Task Manager','Agent Gateway','Model Access'):
            assert text.count(name)>=2,name+' missing in one option'
        for obsolete in ('대화 처리기','업무 관리기','Agent 연동기'):
            assert obsolete not in text,obsolete
        for obsolete in ('공유 Omni 1벌','Shared Inference Service','Speech Input Worker'):
            assert obsolete not in text,obsolete
        assert 'P1' in text and 'P2' in text and 'K1' in text
        assert text.count('State Store')==3,'One joint scope in A / two independent scopes in B'
        assert sum(bool(i.get('external')) for i in self.items)==4,'Cloud model and external Agent per option'
        for side,g in enumerate(self.lifecycle_graphs):
            owned={'gate':(550,402,280,153),'transmit':(550,558,280,58)}
            owned.update({'commit':(320,605,230,103),'saved':(320,605,230,103)} if side==0 else
                         {'dsaved':(32,657,395,53),'tsaved':(502,657,331,53)})
            for key,(ox,oy,ow,oh) in owned.items():
                nx,ny,nw,nh=g.nodes[key]
                assert ox<=nx and oy<=ny and nx+nw<=ox+ow and ny+nh<=oy+oh,('Owner containment',side,key)
        ET.fromstring(self.svg())


def structure():
    return MainSlide()
