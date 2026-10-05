#!/usr/bin/env python3
"""04-44 documentation only: one scene emits SVG and editable draw.io.

This generator does not implement or benchmark either interaction architecture.
"""
import argparse
from pathlib import Path
import xml.etree.ElementTree as ET

from stage4_diagram_design import Plate, INK, MUTED, BLUE, GREEN, LINE

OUT = Path(__file__).resolve().parents[2] / 'docs/architecture/12-decisions/decision-packages/diagrams'


class Slide(Plate):
    def __init__(self, slug, title):
        super().__init__(slug, '04-44', title, '', height=1440)
        self.items = []; self.serial = 0
        self.box(0, 0, 2560, 9, INK, 'none', 0)
        self.text(64, 25, 'VIA / 44 / CONTINUOUS INTERACTION', 23, MUTED, bold=True)
        self.text(2496, 25, '설계 후보 · 미선정 · 미측정', 23, MUTED, 'right')
        self.text(64, 73, title, 42, bold=True)

    @staticmethod
    def text_geometry(i):
        size = 27 if i['w'] >= 280 else 24
        leading = size*1.22
        return i['y']+(i['h']-len(i['name'])*leading)/2-2+(7 if i['type']=='store' else 0),size,leading

    def svg(self):
        return super().svg().replace('font-size="23"', 'font-size="27"')

    def drawio(self):
        return super().drawio().replace('fontSize=23;', 'fontSize=27;')

    def arrow(self, a, b, label='', at=None, sp='B', tp='T', via=(), color=INK, ret=False, sd=0, td=0, size=25):
        self.edge(a,b,sp=sp,tp=tp,via=via,color=color,ret=ret,sd=sd,td=td)
        if label:self.label(*at,label,color,size=size)

    def comp(self, key, x, y, w, name, color=INK, h=64, parent=None):
        return self.component(key,x,y,w,h,name,color,parent=parent)


def structure():
    p=Slide('choice44-structure','계속 듣고, 겹쳐 들어오는 질문과 결과를 한 대화로 잇는 두 실행 구조')
    p.comp('input',64,170,470,'Interaction Manager')
    p.comp('gate',620,170,350,'Input Control Gate')
    p.comp('task',1390,170,310,'Task Manager')
    p.comp('gateway',1800,170,310,'Agent Gateway')
    p.node('agent',2210,170,286,64,'Downstream Agent',kind='external')
    p.arrow('input','gate','발화 시작 / u2',(539,133),sp='R',tp='L')
    p.arrow('gate','task','u2: 미전송 효과 hold',(1020,133),sp='R',tp='L')
    p.arrow('agent','gateway','질문·결과',(2130,240),sp='L',tp='R',ret=True,size=22)
    p.arrow('gateway','task','Q1 / r1',(1730,240),sp='L',tp='R',ret=True)
    p.text(64,249,'P1 보고서 설명 재생 중 → u2 “잠깐, 표부터 설명해줘.”',26,bold=True)
    p.text(64,316,'A  중앙 비동기 Orchestration / Mediator',32,BLUE,bold=True)
    p.text(1306,316,'B  반응형 Dataflow / Pipes-and-Filters',32,GREEN,bold=True)
    p.line([(1270,316),(1270,1315)],LINE,arrow=False)

    # A: actual command-and-return star. No blocking calls are implied.
    p.comp('Aowner',440,455,420,'Interaction Orchestrator',BLUE,h=520)
    p.node('dispatch',475,580,350,94,'Dialogue Dispatcher',owner='Aowner',color=BLUE)
    p.node('Astate',475,827,350,86,'Dialogue Progress State',kind='store',owner='Aowner',color=BLUE)
    p.comp('Arc',64,570,310,'Request Controller')
    p.comp('Arm',925,570,310,'Response Manager')
    p.comp('Avoice',64,833,310,'Voice Runtime')
    p.comp('Aarb',440,1075,420,'Output Arbiter',h=200)
    p.node('Aguard',463,1147,374,38,'Publication Guard',owner='Aarb')
    p.node('Afocus',463,1197,374,64,'Presentation State',kind='store',owner='Aarb')
    p.comp('Aio',440,1330,420,'Interaction Manager')

    p.arrow('gate','dispatch','1  InputFinal(u2)',(475,389),sp='B',tp='L',td=-32,
            via=[(795,365),(425,365),(425,595)],color=BLUE)
    p.arrow('task','dispatch','2  Q1 / r1',(888,410),sp='B',tp='T',
            via=[(1545,293),(875,293),(875,545),(690,545)],td=40,color=BLUE)
    p.arrow('dispatch','Arc','3  입력·근거',(80,503),sp='L',tp='R',sd=-17,td=-17,
            via=[(413,610),(413,585)],color=BLUE)
    p.arrow('Arc','dispatch','의미 / 질문 / Direct 통과',(65,682),sp='R',tp='L',sd=17,td=17,
            via=[(402,619),(402,644)],color=BLUE,ret=True)
    p.arrow('dispatch','Arm','4  확정 근거',(942,503),sp='R',tp='L',sd=-17,td=-17,
            via=[(888,610),(888,585)],color=BLUE)
    p.arrow('Arm','dispatch','응답 후보 반환',(920,683),sp='L',tp='R',sd=17,td=17,
            via=[(900,619),(900,644)],color=BLUE,ret=True)
    p.arrow('dispatch','Astate','5  대기·후속·후보',(494,715),color=BLUE,sd=-30,td=-30)
    p.arrow('Astate','dispatch','revision / waiting IDs',(494,767),sp='T',tp='B',sd=30,td=30,color=BLUE,ret=True)
    p.arrow('Avoice','Arc','Direct 후보 → admission',(67,744),sp='T',tp='B',color=INK,size=23)
    p.arrow('dispatch','Aguard','6  유효 후보 / P2·Q1',(905,1023),sp='R',tp='R',sd=0,td=0,
            via=[(883,627),(883,1166)],color=BLUE)
    p.arrow('Afocus','dispatch','실제 전달 / P2·Q1',(65,1010),sp='L',tp='L',
            via=[(406,1229),(406,713),(475,713)],td=86,color=BLUE,ret=True)
    p.arrow('Aarb','Aio','7  Text / Voice',(440,1284),sd=-95,td=-95)
    p.arrow('Aio','Aarb','receipt',(728,1284),sp='T',tp='B',sd=140,td=140,ret=True)

    # B: two independent producers, direct downstream activation, explicit join.
    p.comp('Bruntime',1306,365,1190,'Reactive Interaction Runtime',GREEN,h=688)
    p.comp('Binput',1340,465,310,'Input Resolution Stage',GREEN,h=165,parent='Bruntime')
    p.node('Biw',1360,549,270,64,'Input Window',kind='store',owner='Binput',color=GREEN)
    p.comp('Bnotice',2150,465,310,'Task Notice Stage',GREEN,h=165,parent='Bruntime')
    p.node('Bnw',2170,549,270,64,'Notice Window',kind='store',owner='Bnotice',color=GREEN)
    p.comp('Brc',1340,745,310,'Request Controller',parent='Bruntime')
    p.comp('Brm',2150,745,310,'Response Manager',parent='Bruntime')
    p.comp('Bvoice',1340,957,310,'Voice Runtime',parent='Bruntime')
    p.comp('Bjoin',1765,815,320,'Publication Join',GREEN,h=192,parent='Bruntime')
    p.node('Bjw',1785,924,280,64,'Publication Window',kind='store',owner='Bjoin',color=GREEN)
    p.comp('Barb',1715,1075,420,'Output Arbiter',h=200)
    p.node('Bguard',1738,1147,374,38,'Publication Guard',owner='Barb')
    p.node('Bfocus',1738,1197,374,64,'Presentation State',kind='store',owner='Barb')
    p.comp('Bio',1715,1330,420,'Interaction Manager')
    p.arrow('gate','Binput','1  InputFinal(u2)',(1327,429),sp='B',tp='T',sd=100,
            via=[(895,354),(1495,354)],color=GREEN)
    p.arrow('task','Bnotice','2  Q1 / r1',(2212,429),sp='B',tp='T',sd=60,
            via=[(1605,310),(2305,310)],color=GREEN)
    p.arrow('Binput','Brc','3  u2·근거',(1350,665),color=GREEN,sd=-50,td=-50)
    p.arrow('Brc','Brm','4  의미·질문 / Direct 통과',(1688,677),sp='R',tp='L',
            via=[(1680,777),(1680,716),(2130,716),(2130,777)],color=GREEN)
    p.arrow('Bnotice','Brm','Q1 / r1',(2170,665),color=GREEN)
    p.arrow('Brc','Bjoin','u2 / 종류',(1638,850),sp='R',tp='L',sd=20,td=-15,
            via=[(1700,797),(1700,896)],color=GREEN)
    p.arrow('Brm','Bjoin','5  응답 / Direct 통과',(2146,847),sp='B',tp='R',td=-15,
            via=[(2305,895),(2110,895),(2110,896)],color=GREEN)
    p.arrow('Bvoice','Brc','Direct 후보 → admission',(1336,870),sp='T',tp='B',color=INK,size=23)
    p.arrow('Bjoin','Bguard','6  P2 / Q1',(1760,1035),color=GREEN,sd=-50,td=-50)
    p.arrow('Bfocus','Bjoin','past snapshot',(2030,1035),sp='R',tp='B',sd=0,td=70,via=[(2146,1229),(2146,1065),(1995,1065)],color=GREEN,ret=True,size=23)
    p.arrow('Bguard','Bjoin','credit',(2170,1064),sp='R',tp='R',td=73,
            via=[(2164,1166),(2164,1044),(2100,1044),(2100,984)],color=GREEN,ret=True,size=23)
    p.arrow('Bjoin','Brm','credit / cancel',(2157,984),sp='R',tp='B',sd=50,td=90,
            via=[(2445,961),(2445,840),(2395,840)],color=GREEN,ret=True,size=23)
    p.arrow('Bjoin','Binput','credit / cancel',(1700,476),sp='T',tp='R',sd=-75,td=-35,
            via=[(1850,443),(1690,443),(1690,512.5)],color=GREEN,ret=True,size=23)
    p.arrow('Barb','Bio','7  Text / Voice',(1715,1284),sd=-95,td=-95)
    p.arrow('Bio','Barb','receipt',(2003,1284),sp='T',tp='B',sd=140,td=140,ret=True)
    p.arrow('Bnotice','Bjoin','Q1.rev / r1.rev',(2170,916),sp='L',tp='R',sd=5,td=34,
            via=[(2124,552.5),(2124,945)],color=GREEN)
    p.arrow('gate','Bruntime','u2: control barrier',(1880,329),sp='R',tp='T',
            via=[(1060,202),(1060,285),(1901,285)],color=INK)

    # Independent control path, deliberately outside both normal execution graphs.
    p.arrow('gate','Aguard','u2: output fence',(65,1102),sp='L',tp='L',
            via=[(580,202),(580,150),(38,150),(38,1166)],size=23)
    p.arrow('gate','Bguard','u2: output fence',(2228,1102),sp='R',tp='R',
            via=[(1010,202),(1010,150),(2522,150),(2522,1166)],size=23)

    for key in ('Arc','Arm','Avoice','Brc','Brm','Bvoice'):
        x,y,w,h=p.nodes[key];p.text(x+w-17,y+17,'m',23,MUTED,'right')
    for key in ('Arc','Brc'):
        x,y,w,h=p.nodes[key];p.text(x+8,y+h+9,'t → Task Manager',23,MUTED)
    p.arrow('Aguard','Arc','v: 현재성 조회',(65,713),sp='L',tp='B',td=120,
            via=[(417,1166),(417,692),(339,692)],size=23)
    p.arrow('Bguard','Brc','v: 현재성 조회',(1390,1148),sp='L',tp='B',td=120,
            via=[(1680,1166),(1680,831),(1615,831)],size=23)
    p.text(910,1090,'v → Task Manager\nu2 / source.rev / permission',23,MUTED,leading=31)
    p.text(2170,1165,'v → Task Manager\nu2 / source.rev / permission',23,MUTED,leading=31)
    p.comp('modelaccess',940,1372,340,'Model Access')
    p.node('omni',1320,1372,340,64,'Shared on-device Omni',kind='model')
    p.arrow('modelaccess','omni','m',(1289,1362),sp='R',tp='L')
    p.text(2170,1235,'P1: interrupted\nQ1: presented / focus',24,MUTED,leading=31)
    p.text(910,1200,'P1: interrupted\nQ1: presented / focus',24,MUTED,leading=31)
    p.text(65,1370,'실선: 명령·자료',22,MUTED)
    p.text(65,1400,'점선: 반환·credit/cancel',22,MUTED)
    p.text(440,1405,'원통: 실행 상태   m/t/v: 공통 서비스 포트',22,MUTED)
    return p


def event():
    p=Slide('choice44-event','같은 정정·교차 질문: 중앙 후속 명령과 소비자 활성화의 사건 경로')
    p.text(64,142,'공통 E3: u2 발화 시작 → local stop / 미전송 hold. E4: 늦은 u1 폐기. E5: u2 해소. E6: Q1 실제 제시 후 focus.',25)

    def participants(prefix, y, specs, bottom, color):
        x=64; lanes={}
        for key,name,width in specs:
            p.comp(prefix+key,x,y,width,name,color)
            lanes[key]=x+width/2
            p.line([(x+width/2,y+64),(x+width/2,bottom)],LINE,dashed=True,arrow=False)
            x+=width+16
        return lanes

    def message(lanes,a,b,y,label,color,ret=False):
        x1,x2=lanes[a],lanes[b]
        p.line([(x1,y),(x2,y)],color,dashed=ret)
        p.label(min(x1,x2)+14,y-29,label,color,size=23)

    p.text(64,187,'A  각 반환을 회수한 중앙 소유자가 다음 명령을 발행한다',29,BLUE,bold=True)
    a=participants('seqA',238,[
        ('gate','Input Control Gate',300),('task','Task Manager',300),
        ('hub','Interaction Orchestrator',420),('rc','Request Controller',310),
        ('rm','Response Manager',310),('arb','Output Arbiter',310),
        ('io','Interaction Manager',310)],783,BLUE)
    message(a,'gate','arb',334,'E3 u2: output fence',INK)
    message(a,'task','hub',367,'E2 Q1/r1 확정 자료',BLUE)
    message(a,'gate','hub',409,'InputFinal(u2)',BLUE)
    message(a,'rc','hub',451,'E4 u1 반환 → 중앙 revision 검사로 폐기',BLUE,True)
    message(a,'hub','rc',493,'E5 u2 입력·근거 해석/채택 요청',BLUE)
    message(a,'rc','hub',535,'의미 / 확인 질문 / 승인된 Direct 후보',BLUE,True)
    message(a,'hub','rm',577,'생성이 필요한 확정 근거만 준비 명령',BLUE)
    message(a,'rm','hub',619,'응답 후보 반환',BLUE,True)
    message(a,'hub','arb',661,'현재 후보 → Guard → Text/Voice',BLUE)
    message(a,'arb','io',703,'E6 P2 / Q1 실제 제시',INK)
    message(a,'io','arb',745,'receipt → Presentation State',INK,True)
    message(a,'arb','hub',779,'실제 전달 / 질문 focus snapshot',BLUE,True)

    p.text(64,787,'B  생산된 자료를 다음 소비자가 받고, Join과 sink가 게시 조건을 결합한다',29,GREEN,bold=True)
    b=participants('seqB',836,[
        ('gate','Input Control Gate',280),('notice','Task Notice Stage',310),
        ('input','Input Resolution Stage',310),('rc','Request Controller',290),
        ('rm','Response Manager',290),('join','Publication Join',280),
        ('arb','Output Arbiter',280),('io','Interaction Manager',300)],1370,GREEN)
    message(b,'gate','arb',930,'E3 u2: control barrier / output fence',INK)
    message(b,'notice','rm',959,'E2 Task Manager의 Q1/r1 확정 자료',GREEN)
    message(b,'gate','input',997,'InputFinal(u2) → Input Window 활성화',GREEN)
    message(b,'rm','join',1035,'E4 u1 → window/Join에서 superseded 폐기',GREEN)
    message(b,'input','rc',1073,'E5 u2 입력·근거 해석/채택',GREEN)
    message(b,'rc','rm',1111,'의미 / InteractionNotice / Direct 통과',GREEN)
    message(b,'rc','join',1149,'종류별 admission(u2)',GREEN)
    message(b,'rm','join',1187,'후보·source refs → Window / join',GREEN)
    message(b,'join','arb',1225,'유효 P2 / Q1 → Guard',GREEN)
    message(b,'arb','io',1263,'E6 Text/Voice 실제 제시',INK)
    message(b,'io','arb',1301,'receipt → Presentation State',INK,True)
    message(b,'arb','join',1339,'Guard의 credit / State의 past snapshot',GREEN,True)
    p.text(64,1391,'E7 “응”: 양안 모두 실제 Q1 focus + 현재 질문 원본으로 연결. 미해결이면 확인. sequence 간격은 시간 측정값이 아니다.',23,MUTED)
    return p


def main():
    ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args()
    drift=[]
    for p in (structure(),event()):
        p.validate()
        for i in p.items:
            if i['kind']=='component':assert not i['role']
        for ext,data in (('svg',p.svg()),('drawio',p.drawio())):
            root=ET.fromstring(data)
            if ext=='drawio':
                ids=[e.get('id') for e in root.findall('.//mxCell')];assert len(ids)==len(set(ids))
                for e in root.findall('.//mxCell'):
                    for a in ('source','target','parent'):assert not e.get(a) or e.get(a) in ids
            target=OUT/(p.slug+'.'+ext)
            if args.check:
                if not target.exists() or target.read_text()!=data:drift.append(target.name)
            else:target.write_text(data)
    if drift:raise SystemExit('Diagram drift: '+', '.join(drift))
    print('PASS: 44 SVG/draw.io parity, XML, owners, orthogonal routes, canvas, name-only headers')

if __name__=='__main__':main()
