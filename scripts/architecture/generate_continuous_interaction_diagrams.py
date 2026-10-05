#!/usr/bin/env python3
"""04-44 documentation only: one scene emits SVG and editable draw.io.

This generator does not implement or benchmark either interaction architecture.
"""
import argparse
import re
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
        data=super().svg().replace('font-size="23"', 'font-size="27"')
        # Narrow sequence participants wrap full names; reserve the entire header.
        for i in self.items:
            if i['kind']!='component' or i['h']!=112:continue
            x,y,w,h,c=i['x'],i['y'],i['w'],i['h'],i['color']
            name=''.join(f'<text x="{x+12}" y="{y+31+j*28}" font-size="21" font-weight="700" fill="{c}">{n}</text>' for j,n in enumerate(i['name']))
            group=f'<g id="{i["id"]}" data-component="true"><rect x="{x}" y="{y}" width="{w}" height="{h}" rx="4" fill="white" stroke="{c}" stroke-width="2.2"/>{name}</g>'
            data=re.sub(f'<g id="{i["id"]}" data-component="true">.*?</g>',group,data,flags=re.S)
        return data

    def drawio(self):
        root=ET.fromstring(super().drawio().replace('fontSize=23;', 'fontSize=27;'))
        wrapped={i['id'] for i in self.items if i['kind']=='component' and i['h']==112}
        for cell in root.findall('.//mxCell'):
            if cell.get('id') in wrapped:
                cell.set('style',cell.get('style').replace('startSize=62;', 'startSize=112;').replace('fontSize=17;', 'fontSize=21;').replace('spacingLeft=18;', 'spacingLeft=12;'))
        return ET.tostring(root,encoding='unicode')+'\n'

    def validate_options(self):
        # Reject shared participants and any A↔B edge or route outside its own panel.
        bounds={'A':(32,1242),'B':(1318,2528)}
        for key,(x,y,w,h) in self.nodes.items():
            assert key[0] in bounds,(self.slug,key,'shared participant')
            lo,hi=bounds[key[0]]
            assert lo<=x and x+w<=hi,(self.slug,key,'outside option')
        for i in self.items:
            if i['kind']!='line' or not i.get('arrow'):continue
            if 'source' in i:
                side=i['source'][0]
                assert side==i['target'][0],(self.slug,i['id'],'cross-option edge')
            else:side='A' if i['points'][0][0]<1280 else 'B'
            lo,hi=bounds[side]
            assert all(lo<=x<=hi for x,y in i['points']),(self.slug,i['id'],'cross-option route')

    def arrow(self, a, b, label='', at=None, sp='B', tp='T', via=(), color=INK, ret=False, sd=0, td=0, size=25):
        self.edge(a,b,sp=sp,tp=tp,via=via,color=color,ret=ret,sd=sd,td=td)
        if label:self.label(*at,label,color,size=size)

    def comp(self, key, x, y, w, name, color=INK, h=64, parent=None):
        return self.component(key,x,y,w,h,name,color,parent=parent)


def structure():
    p=Slide('choice44-structure','계속 듣고, 겹쳐 들어오는 질문과 결과를 한 대화로 잇는 두 실행 구조')
    p.box(32,145,1210,1293,'white',LINE,8)
    p.box(1318,145,1210,1293,'white',LINE,8)
    p.text(64,164,'A  중앙 비동기 Orchestration / Mediator',31,BLUE,bold=True)
    p.text(1340,164,'B  반응형 Dataflow / Pipes-and-Filters',31,GREEN,bold=True)
    for side,x in [('A',64),('B',1340)]:
        p.text(x,212,'P1 설명 중 → u2 “잠깐, 표부터”   +   Q1 / r1 도착',25,bold=True)
        gx=x+556
        p.comp(side+'capture',x,260,310,'Interaction Manager')
        p.comp(side+'gate',gx,260,350,'Input Control Gate')
        p.comp(side+'task',x,355,310,'Task Manager')
        p.comp(side+'gateway',x+386,355,310,'Agent Gateway')
        p.node(side+'agent',x+861,355,310,64,'Downstream Agent',kind='external')
        p.arrow(side+'capture',side+'gate','u2 / 발화 시작',(x+347,262),sp='R',tp='L',size=24)
        p.arrow(side+'gate',side+'task','u2: 미전송 효과 hold',(x,328),
                sp='B',tp='T',via=[(gx+175,340),(x+155,340)],size=23)
        p.arrow(side+'task',side+'gateway','명령',(x+316,350),sp='R',tp='L',sd=-12,td=-12,size=22)
        p.arrow(side+'gateway',side+'task','Q/r',(x+319,423),sp='L',tp='R',sd=12,td=12,ret=True,size=22)
        p.arrow(side+'gateway',side+'agent','명령',(x+729,350),sp='R',tp='L',sd=-12,td=-12,size=22)
        p.arrow(side+'agent',side+'gateway','질문·결과',(x+715,423),sp='L',tp='R',sd=12,td=12,ret=True,size=22)

    # A: command/return star and state owned by its central coordinator.
    p.comp('Aowner',440,455,420,'Interaction Orchestrator',BLUE,h=520)
    p.node('Adispatch',475,580,350,94,'Dialogue Dispatcher',owner='Aowner',color=BLUE)
    p.node('Astate',475,827,350,86,'Dialogue Progress State',kind='store',owner='Aowner',color=BLUE)
    p.comp('Arc',64,570,310,'Request Controller')
    p.comp('Arm',925,570,310,'Response Manager')
    p.comp('Avoice',64,833,310,'Voice Runtime')
    p.comp('Aarb',440,1075,420,'Output Arbiter',h=200)
    p.node('Aguard',463,1147,374,38,'Publication Guard',owner='Aarb')
    p.node('Afocus',463,1197,374,64,'Presentation State',kind='store',owner='Aarb')
    p.comp('Aio',440,1330,420,'Interaction Manager')
    p.arrow('Agate','Adispatch','1  InputFinal(u2)',(65,475),sp='B',tp='L',td=-32,
            via=[(795,445),(425,445),(425,595)],color=BLUE)
    p.arrow('Atask','Adispatch','2  Q1 / r1',(890,475),sp='B',tp='T',td=40,
            via=[(219,435),(875,435),(875,545),(690,545)],color=BLUE)
    p.arrow('Adispatch','Arc','3  입력·근거',(80,503),sp='L',tp='R',sd=-17,td=-17,
            via=[(413,610),(413,585)],color=BLUE)
    p.arrow('Arc','Adispatch','의미 / 질문 / Direct 통과',(65,682),sp='R',tp='L',sd=17,td=17,
            via=[(402,619),(402,644)],color=BLUE,ret=True)
    p.arrow('Adispatch','Arm','4  확정 근거',(942,503),sp='R',tp='L',sd=-17,td=-17,
            via=[(888,610),(888,585)],color=BLUE)
    p.arrow('Arm','Adispatch','응답 후보 반환',(920,683),sp='L',tp='R',sd=17,td=17,
            via=[(900,619),(900,644)],color=BLUE,ret=True)
    p.arrow('Adispatch','Astate','5  대기·후속·후보',(494,715),color=BLUE,sd=-30,td=-30)
    p.arrow('Astate','Adispatch','revision / waiting IDs',(494,767),sp='T',tp='B',sd=30,td=30,color=BLUE,ret=True)
    p.arrow('Avoice','Arc','Direct 후보 → admission',(67,744),sp='T',tp='B',size=23)
    p.arrow('Adispatch','Aguard','6  유효 후보 / P2·Q1',(905,1023),sp='R',tp='R',
            via=[(883,627),(883,1166)],color=BLUE)
    p.arrow('Afocus','Adispatch','실제 전달 / P2·Q1',(65,1010),sp='L',tp='L',
            via=[(406,1229),(406,713),(475,713)],td=86,color=BLUE,ret=True)
    p.arrow('Agate','Aguard','u2: output fence',(65,1102),sp='L',tp='L',
            via=[(390,292),(390,250),(50,250),(50,1166)],size=23)
    p.arrow('Aguard','Arc','v: 현재성 조회',(65,713),sp='L',tp='B',td=120,
            via=[(417,1166),(417,692),(339,692)],size=23)

    # B: input/notice producers activate subscribers; no central return dispatcher.
    p.comp('Bruntime',1340,455,1170,'Reactive Interaction Runtime',GREEN,h=590)
    p.comp('Binput',1366,560,310,'Input Resolution Stage',GREEN,h=165,parent='Bruntime')
    p.node('Biw',1386,644,270,64,'Input Window',kind='store',owner='Binput',color=GREEN)
    p.comp('Bnotice',2190,560,310,'Task Notice Stage',GREEN,h=165,parent='Bruntime')
    p.node('Bnw',2210,644,270,64,'Notice Window',kind='store',owner='Bnotice',color=GREEN)
    p.comp('Brc',1366,815,310,'Request Controller',parent='Bruntime')
    p.comp('Brm',2190,815,310,'Response Manager',parent='Bruntime')
    p.comp('Bvoice',1366,957,310,'Voice Runtime',parent='Bruntime')
    p.comp('Bjoin',1765,865,320,'Publication Join',GREEN,h=172,parent='Bruntime')
    p.node('Bjw',1785,938,280,64,'Publication Window',kind='store',owner='Bjoin',color=GREEN)
    p.comp('Barb',1715,1075,420,'Output Arbiter',h=200)
    p.node('Bguard',1738,1147,374,38,'Publication Guard',owner='Barb')
    p.node('Bfocus',1738,1197,374,64,'Presentation State',kind='store',owner='Barb')
    p.comp('Bio',1715,1330,420,'Interaction Manager')
    p.arrow('Bgate','Binput','1  InputFinal(u2)',(1340,531),sp='B',tp='T',
            via=[(2071,440),(1328,440),(1328,540),(1521,540)],color=GREEN)
    p.arrow('Btask','Bnotice','2  Q1 / r1',(2200,531),sp='B',tp='T',
            via=[(1495,434),(2520,434),(2520,545),(2345,545)],color=GREEN)
    p.arrow('Bgate','Bruntime','u2: control barrier',(1800,420),sp='B',tp='T',sd=80,td=-45,
            via=[(2151,445),(1880,445)],size=23)
    p.arrow('Binput','Brc','3  u2·근거',(1370,752),color=GREEN,sd=-50,td=-50)
    p.arrow('Brc','Brm','4  의미·질문 / Direct 통과',(1712,746),sp='R',tp='L',
            via=[(1700,847),(1700,780),(2160,780),(2160,847)],color=GREEN)
    p.arrow('Bnotice','Brm','Q1 / r1',(2200,752),color=GREEN)
    p.arrow('Brc','Bjoin','u2 / 종류',(1670,924),sp='R',tp='L',sd=20,td=-15,
            via=[(1700,867),(1700,936)],color=GREEN,size=23)
    p.arrow('Brm','Bjoin','5  응답 / Direct 통과',(2155,910),sp='B',tp='R',td=-15,
            via=[(2345,936),(2110,936)],color=GREEN,size=23)
    p.arrow('Bvoice','Brc','Direct 후보 → admission',(1368,930),sp='T',tp='B',size=23)
    p.arrow('Bjoin','Bguard','6  P2 / Q1',(1340,1090),sp='B',tp='L',sd=-50,
            via=[(1875,1058),(1695,1058),(1695,1166)],color=GREEN)
    p.arrow('Bfocus','Bjoin','past snapshot',(2030,1046),sp='R',tp='B',td=70,
            via=[(2146,1229),(2146,1065),(1995,1065)],color=GREEN,ret=True,size=23)
    p.arrow('Bguard','Bjoin','credit',(2330,1050),sp='R',tp='R',td=73,
            via=[(2164,1166),(2164,1044),(2100,1044),(2100,1024)],color=GREEN,ret=True,size=23)
    p.arrow('Bjoin','Brm','credit / cancel',(2180,1015),sp='R',tp='B',sd=50,td=90,
            via=[(2460,1001),(2460,910),(2435,910)],color=GREEN,ret=True,size=23)
    p.arrow('Bjoin','Binput','credit / cancel',(1710,567),sp='T',tp='R',sd=-75,td=-35,
            via=[(1850,540),(1710,540),(1710,607.5)],color=GREEN,ret=True,size=23)
    p.arrow('Bnotice','Bjoin','Q1.rev / r1.rev',(2180,956),sp='L',tp='R',sd=5,td=34,
            via=[(2134,647.5),(2134,985)],color=GREEN,size=23)
    p.arrow('Bgate','Bguard','u2: output fence',(2190,1080),sp='R',tp='R',
            via=[(2265,292),(2265,250),(2522,250),(2522,1166)],size=23)
    p.arrow('Bguard','Brc','v: 현재성 조회',(1368,1148),sp='L',tp='B',td=120,
            via=[(1680,1166),(1680,895),(1641,895)],size=23)

    # Repeat every common endpoint inside its own alternative, including model access.
    for side,ax,mx,nx in [('A',440,925,910),('B',1715,2190,2180)]:
        p.arrow(side+'arb',side+'io','7  Text / Voice',(ax,1284),sd=-95,td=-95)
        p.arrow(side+'io',side+'arb','receipt',(ax+288,1284),sp='T',tp='B',sd=140,td=140,ret=True)
        p.text(nx,1112,'v → Task Manager\nu2 / source.rev / permission',23,MUTED,leading=31)
        p.text(nx,1190,'P1: interrupted\nQ1: presented / focus',24,MUTED,leading=31)
        p.comp(side+'modelaccess',mx,1255,310,'Model Access')
        p.node(side+'omni',mx,1340,310,64,'Shared on-device\nOmni',kind='model')
        p.arrow(side+'modelaccess',side+'omni','m',(mx-31,1320),size=23)
        left=64 if side=='A' else 1340
        p.text(left,1407,'실선: 명령·자료   점선: 반환·credit   원통: 상태   m/t/v: 서비스 포트',21,MUTED)
    for key in ('Arc','Arm','Avoice','Brc','Brm','Bvoice'):
        x,y,w,h=p.nodes[key];p.text(x+w-17,y+17,'m',23,MUTED,'right')
    for key in ('Arc','Brc'):
        x,y,w,h=p.nodes[key]
        p.text(x+8 if key=='Arc' else 1710,y+h+9 if key=='Arc' else 810,'t → Task Manager',23,MUTED)
    return p


def event():
    p=Slide('choice44-event','같은 정정·교차 질문: 중앙 후속 명령과 소비자 활성화의 사건 경로')
    for side,x,c in [('A',64,BLUE),('B',1340,GREEN)]:
        p.box(x-32 if side=='A' else x-22,145,1210,1293,'white',LINE,8)
        p.text(x,164,side+'  '+('중앙 명령·반환' if side=='A' else '생산자·소비자 활성화'),31,c,bold=True)
        p.text(x,212,'같은 u2 정정 · Q1 질문 · 늦은 u1 반환 · 실제 제시/receipt',25)
    p.comp('Aqsource',925,260,310,'Task Manager')
    p.comp('Bqsource',2201,260,310,'Task Notice Stage',GREEN)

    def participants(side,specs):
        x=64 if side=='A' else 1340;w=185 if side=='A' else 156;gap=8 if side=='A' else 9
        lanes={};c=BLUE if side=='A' else GREEN
        for key,name in specs:
            p.comp(side+key,x,365,w,name,c,h=112)
            lanes[key]=x+w/2
            p.line([(x+w/2,477),(x+w/2,1275)],LINE,dashed=True,arrow=False)
            x+=w+gap
        return lanes

    def message(lanes,a,b,y,label,c,ret=False):
        x1,x2=lanes[a],lanes[b]
        p.line([(x1,y),(x2,y)],c,dashed=ret)
        p.label(min(x1,x2)+10,y-30,label,c,size=22)

    a=participants('A',[
        ('gate','Input Control\nGate'),('hub','Interaction\nOrchestrator'),
        ('rc','Request\nController'),('rm','Response\nManager'),
        ('arb','Output\nArbiter'),('io','Interaction\nManager')])
    b=participants('B',[
        ('gate','Input Control\nGate'),('input','Input\nResolution\nStage'),
        ('rc','Request\nController'),('rm','Response\nManager'),
        ('join','Publication\nJoin'),('arb','Output\nArbiter'),
        ('io','Interaction\nManager')])
    message(a,'gate','arb',525,'E3 u2: output fence',INK)
    p.line([(1080,324),(1080,348),(1231,348),(1231,590),(a['hub'],590)],BLUE)
    p.label(a['hub']+10,560,'E2 Q1 / r1 확정 자료',BLUE,size=22)
    message(a,'gate','hub',655,'InputFinal(u2)',BLUE)
    message(a,'rc','hub',720,'E4 u1 반환 → revision 검사·폐기',BLUE,True)
    message(a,'hub','rc',785,'E5 u2 입력·근거 해석/채택',BLUE)
    message(a,'rc','hub',850,'의미 / 질문 / Direct 통과',BLUE,True)
    message(a,'hub','rm',915,'생성할 확정 근거만 준비 명령',BLUE)
    message(a,'rm','hub',980,'후보 반환',BLUE,True)
    message(a,'hub','arb',1045,'현재 후보 → Publication Guard',BLUE)
    message(a,'arb','io',1110,'E6 Text / Voice',INK)
    message(a,'io','arb',1175,'receipt / actual-range',INK,True)
    message(a,'arb','hub',1240,'실제 전달 / Q1 focus snapshot',BLUE,True)

    message(b,'gate','arb',525,'E3 u2: control barrier / fence',INK)
    p.line([(2356,324),(2356,348),(2520,348),(2520,590),(b['rm'],590)],GREEN)
    p.label(b['rm']+10,560,'E2 Q1 / r1 확정 자료',GREEN,size=22)
    message(b,'gate','input',655,'InputFinal(u2)',GREEN)
    message(b,'rm','join',720,'E4 u1 → superseded 폐기',GREEN)
    message(b,'input','rc',785,'E5 u2 해석/채택',GREEN)
    message(b,'rc','rm',850,'의미 / 질문 / Direct 통과',GREEN)
    message(b,'rc','join',915,'종류별 admission(u2)',GREEN)
    message(b,'rm','join',980,'후보 / source refs',GREEN)
    message(b,'join','arb',1045,'유효 P2 / Q1',GREEN)
    message(b,'arb','io',1110,'E6 Text / Voice',INK)
    message(b,'io','arb',1175,'receipt / actual-range',INK,True)
    message(b,'arb','join',1240,'credit / past snapshot',GREEN,True)
    for x in [64,1340]:
        p.text(x,1310,'E7 “응”: 실제 Q1 focus + 현재 질문 원본으로 연결. 모호하면 확인.',23)
        p.text(x,1360,'발화 시작 ≠ Task 취소   |   확인 질문은 업무 hold를 유지하며 게시',22,MUTED)
        p.text(x,1400,'선의 길이와 사건 간격은 측정 시간이 아니다.',22,MUTED)
    return p


def main():
    ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args()
    drift=[]
    for p in (structure(),event()):
        p.validate()
        p.validate_options()
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
    print('PASS: 44 SVG/draw.io parity, XML, owners, orthogonal routes, canvas, name-only headers, independent A/B panels')

if __name__=='__main__':main()
