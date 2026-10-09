"""04-41 landscape main comparison; unchanged alternatives and event diagrams.

Each alternative is complete inside its own column, including common contracts.
Every connector is also an editable edge in the companion draw.io scene.
"""
from stage4_diagram_design import Plate, INK, MUTED, BLUE, GREEN, LINE
import xml.etree.ElementTree as ET
import copy
import re
from dp_comparison_structures import COMMON, APRICOT, DIFFERENCE_STROKE, DIFFERENCE_WIDTH, append_plate_legend

TITLE = '요청 의미 확정 — 모델 주도 해석과 모델 틀/코드 완성'


class Slide(Plate):
    def __init__(self):
        super().__init__('choice41-structure', '04-41', TITLE, '', height=1440)
        self.items = []
        self.serial = 0

    @staticmethod
    def text_geometry(i):
        units=max((sum(1 if ord(c)>=0x2e80 else 0.55 for c in line) for line in i['name']),default=1)
        size=min(25,(i['w']-24)/units)
        if i['type']=='store':
            size=max(10,min(size,i['h']-43))
            return i['y']+29,size,size*1.25
        leading = size * 1.25
        return i['y'] + (i['h']-len(i['name'])*leading)/2-2, size, leading

    def validate(self):
        super().validate()
        # A and B must be self-contained, including repeated common Components.
        limits={'A': (64,1248), 'B': (1312,2496)}
        for key,(x,y,w,h) in self.nodes.items():
            lo,hi=limits[key[0]]
            assert lo<=x and x+w<=hi, (key,'outside comparison column')
        for item in self.items:
            if item['kind']=='line' and 'source' in item:
                prefix=item['source'][0]
                assert prefix==item['target'][0], (item['id'],'cross-option edge')
                lo,hi=limits[prefix]
                assert all(lo<=x<=hi for x,y in item['points']), (item['id'],'edge leaves column')

        # Payload notes are not executable vertices, but routes must also avoid them.
        docs=[i for i in self.items if i['kind']=='node' and i['type']=='data']
        for edge in self.items:
            if edge['kind']!='line':continue
            for a,b in zip(edge['points'],edge['points'][1:]):
                for doc in docs:
                    x,y,w,h=doc['x'],doc['y'],doc['w'],doc['h']
                    v=a[0]==b[0] and x+1<a[0]<x+w-1 and max(min(a[1],b[1]),y+1)<min(max(a[1],b[1]),y+h-1)
                    z=a[1]==b[1] and y+1<a[1]<y+h-1 and max(min(a[0],b[0]),x+1)<min(max(a[0],b[0]),x+w-1)
                    assert not (v or z),(edge['id'],'route crosses data note',doc['id'])

    def svg(self):
        root=ET.fromstring(super().svg().replace('font-size="23"','font-size="26"'))
        ns={'s':'http://www.w3.org/2000/svg'}
        marker=copy.deepcopy(root.find('.//s:marker',ns));marker.set('id','a000000')
        marker.find('s:path',ns).set('stroke',COMMON)
        root.find('s:defs',ns).append(marker)
        if any(i.get('different') for i in self.items):
            dm=copy.deepcopy(marker);dm.set('id','a'+DIFFERENCE_STROKE[1:]);dm.find('s:path',ns).set('stroke',DIFFERENCE_STROKE);root.find('s:defs',ns).append(dm)
        for item in self.items:
            if item.get('data_note'):
                root.find('.//s:g[@id="'+item['id']+'"]',ns).set('data-doc-ref',item['data_note'])
            if item['kind']=='line' and item.get('different'):
                line=root.find('.//s:polyline[@id="'+item['id']+'"]',ns);line.set('stroke',DIFFERENCE_STROKE);line.set('marker-end','url(#a'+DIFFERENCE_STROKE[1:]+')')
            if item['kind'] not in ('component','node'): continue
            group=root.find('.//s:g[@id="'+item['id']+'"]',ns)
            shape=next(child for child in group if child.tag.rsplit('}',1)[-1] in ('rect','path'))
            different=item['id'] in ('Aloop','Bframe','Bengine')
            shape.set('fill',APRICOT if different else 'white')
            shape.set('stroke',DIFFERENCE_STROKE if different else COMMON)
            shape.set('stroke-width','1.3')
            if shape.tag.rsplit('}',1)[-1]=='rect': shape.set('rx','6' if item['kind']=='node' and item['type']=='module' else '0')
            for path in group.findall('s:path',ns):path.set('stroke-width','1.3')
        ET.register_namespace('',ns['s'])
        return ET.tostring(root,encoding='unicode')+'\n'

    def drawio(self):
        root=ET.fromstring(super().drawio().replace('fontSize=23;','fontSize=26;'))
        items={i['id']:i for i in self.items}
        for cell in root.findall('.//mxCell'):
            item=items.get(cell.get('id'))
            if item and item['kind']=='line' and item.get('different'):
                cell.set('style',re.sub(r'strokeColor=[^;]*;','strokeColor='+DIFFERENCE_STROKE+';',cell.get('style')))
            if not item or item['kind'] not in ('component','node'): continue
            different=item['id'] in ('Aloop','Bframe','Bengine')
            style=cell.get('style')
            style=re.sub(r'fillColor=[^;]*;', 'fillColor='+ (APRICOT if different else 'white')+';',style)
            style=re.sub(r'strokeWidth=[^;]*;','strokeWidth=1.3;',style)
            style=re.sub(r'strokeColor=[^;]*;',f'strokeColor={DIFFERENCE_STROKE if different else COMMON};',style)
            # Preserve the shared external-model hexagon notation.
            if item['kind']=='component': style=style.replace('swimlane;', 'swimlane;rounded=0;')
            cell.set('style',style)
        return ET.tostring(root,encoding='unicode')+'\n'


    def arrow(self, a, b, label='', at=None, sp='B', tp='T', via=(), color=INK, ret=False, sd=0, td=0, size=24):
        self.edge(a, b, sp=sp, tp=tp, via=via, color=color, ret=ret, sd=sd, td=td)
        if label:
            self.label(*at, label, color, size=size)


def structure():
    p=Slide()
    p.box(0,0,2560,8,COMMON,'none',0)
    p.text(64,25,'VIA / 04-41 / 다음 조회와 의미 생산의 판단 주체',23,COMMON,bold=True)
    p.text(2496,25,'설계 비교 · 미선정 · 미측정',23,COMMON,'right')
    p.text(64,65,'모델이 근거를 읽고 전체 의미를 판단할까, 코드가 요청 틀의 항목을 해결할까?',40,bold=True)
    p.text(64,122,'“이 표를 아까 보고서에 넣고, 메일은 보내지 말고 결과를 바탕으로 초안만 만들어.”',28,bold=True)
    p.text(64,162,'Task1: 예산 보고서 / Task2: 실적 보고서 → 확인 질문 → “예산 보고서” 답변 → Task1 연결 / 보고서 결과로 메일 초안',24,COMMON)
    p.text(64,196,'조회 전 정답 업무는 미정 / VIA는 기기 내, 모델은 클라우드 / 같은 Component의 반복 표기는 같은 인스턴스 / 데이터는 설명용 예시 / Input1=발화 / G1=보고서 수정 / G2=메일 초안',17,COMMON)
    for prefix,dx,title in [('A',0,'A  모델이 조회 선택 → 근거로 재판단 → 전체 의미 제안'),('B',1248,'B  모델의 요청 틀 → 코드의 항목 해결·전체 결합')]:
        p.box(64+dx,226,1184,1131,'white',LINE,0)
        p.text(80+dx,231,title,27,COMMON,bold=True)
        def component(key,x,y,w,h,name): return p.component(prefix+key,x+dx,y,w,h,name,COMMON)
        def node(key,x,y,w,h,name,owner=None,kind='module'): return p.node(prefix+key,x+dx,y,w,h,name,owner=prefix+owner if owner else None,kind=kind,color=COMMON)
        def arrow(a,b,label='',at=None,different=False,**kw):
            if 'via' in kw: kw['via']=[(x+dx,y) for x,y in kw['via']]
            size=kw.pop('size',20)
            p.edge(prefix+a,prefix+b,color=COMMON,**kw)
            p.items[-1]['different']=different
            if label:p.label(at[0]+dx,at[1],label,DIFFERENCE_STROKE if different else COMMON,size)
        def datum(x,y,w,h,title,lines,size=20):
            # A note/document is an exchanged contract example, never a processing Module.
            doc=p._add('node',x=x+dx,y=y,w=w,h=h,name=[],type='data',color=COMMON,parent=None)
            header='교환 데이터 예시 · '+title
            # Integer accumulation avoids Python-version-dependent float sum output.
            units=sum(100 if ord(c)>=0x2e80 else 55 for c in header)/100
            p.text(x+12+dx,y+8,header,min(16,(w-34)/units),COMMON,bold=True)
            p.items[-1]['data_note']=doc
            p.text(x+12+dx,y+33,lines,size,COMMON,leading=size*1.24)
            p.items[-1]['data_note']=doc
        component('input',80,275,335,62,'Interaction Manager')
        component('start',500,275,335,62,'Request Controller')
        node('voice',925,275,290,62,'Cloud Voice Model',kind='model')
        component('ri',80,360,815,624,'Request Interpreter')
        component('ma',925,360,290,62,'Model Access')
        node('semantic',925,475,290,62,'Cloud Semantic LLM',kind='model')
        component('cm',925,695,290,62,'Context Manager')
        component('tm',925,820,290,62,'Task Manager')
        p.text(925+dx,550,'실제 의미 판단: 클라우드 LLM',18,COMMON)
        p.text(925+dx,578,'코드/모델 구분은 책임의 구분',17,COMMON)
        p.text(925+dx,626,['Interaction Manager:','로컬 VAD·지속 입력 / 시점 연결'],15,COMMON,leading=20)
        arrow('input','start','발화·근거·버전',(419,263),sp='R',tp='L',size=17)
        arrow('input','ma',sp='B',tp='L',sd=-50,td=0,via=[(197.5,346),(910,346),(910,391)])
        arrow('ma','input',sp='L',tp='B',sd=15,td=70,via=[(915,406),(915,350),(317.5,350)],ret=True)
        arrow('ma','voice',sp='T',tp='B',sd=-30,td=-30)
        arrow('voice','ma',sp='B',tp='T',sd=30,td=30,ret=True)
        arrow('ma','semantic','추론 요청 / 모델 결과',(925,435),sd=-30,td=-30,size=17)
        arrow('semantic','ma',sp='T',tp='B',sd=30,td=30,ret=True)
        node('check',110,905,330,64,'의미 제안 검증기','ri')
        node('temp',500,905,300,64,'임시 해석 상태' if prefix=='A' else '요청 해석 상태','ri','store')
        if prefix=='A':
            node('loop',180,455,375,66,'모델 주도 해석 제어기','ri')
            node('tools',500,645,300,64,'읽기 도구 실행기','ri')
            p.text(180+dx,426,'VIA 코드: 모델 제안을 실행·제한',19,COMMON)
            arrow('start','loop',sp='B',tp='T',via=[(667.5,350),(907,350),(907,443),(367.5,443)])
            arrow('loop','ma',sp='T',tp='B',sd=140,td=-70,via=[(507.5,448),(1000,448)],different=True)
            arrow('ma','loop',sp='L',tp='T',sd=25,td=70,via=[(916,416),(916,450),(437.5,450)],ret=True,different=True)
            arrow('loop','tools','1  모델의 다음 읽기 선택',(500,515),sp='R',tp='T',via=[(855,488),(855,632),(650,632)],different=True,size=22)
            arrow('tools','loop','3  근거로 모델 재판단',(490,750),sp='L',tp='B',via=[(465,677),(465,548),(367.5,548)],ret=True,different=True,size=22)
            datum(500,541,300,87,'모델 출력: tool_calls', ['task_retrieve(query=보고서,limit=5)','context_retrieve(INPUT_SELECTION)'],12.5)
            datum(110,568,340,126,'조회 결과 (EvidenceBundle)', ['Task1: 예산 보고서','Task2: 실적 보고서','자료: 선택한 표','출처·조회 시점·검색 범위'],18)
            arrow('loop','check','4  전체 관계 / 질문 제안',(115,721),sp='L',tp='L',via=[(100,488),(100,937)],different=True,size=22)
            arrow('check','loop',sp='T',tp='B',sd=120,td=140,via=[(395,894),(475,894),(475,535),(507.5,535)],ret=True,different=True)
            arrow('tools','temp','사용한 자료·조회 기록',(500,878),sp='B',tp='R',via=[(650,718),(835,718),(835,937)],size=18)
            p.text(490+dx,789,['반복: 근거 → 모델 재호출 → 다음 tool_calls','추가 예: interaction_retrieve(보고서)', 'VIA는 범위·예산·형식·자료 변경을 검사'],15,COMMON,leading=24)
            p.text(925+dx,928,['A도 구조화 출력 가능','부분 수정·cache 가능'],17,COMMON,leading=24)
        else:
            node('frame',110,455,330,66,'요청 구조화기','ri')
            node('tools',520,455,280,66,'읽기 도구 실행기','ri')
            node('engine',500,680,300,66,'Request Resolution Engine','ri')
            p.text(110+dx,426,'모델: 초기 틀·지정 항목의 의미',19,COMMON)
            arrow('start','frame',sp='B',tp='T',via=[(667.5,350),(907,350),(907,443),(275,443)])
            arrow('frame','ma',sp='T',tp='B',sd=135,td=-70,via=[(410,449),(1000,449)],different=True)
            arrow('ma','frame',sp='L',tp='T',sd=25,td=60,via=[(916,416),(916,450),(335,450)],ret=True,different=True)
            arrow('frame','engine','1  초기 요청 틀 / 부분 해석 결과',(115,526),sp='R',tp='T',sd=10,via=[(460,498),(460,675),(650,675)],different=True,size=20)
            datum(110,568,340,150,'RequestFrame v1 고정 필드', ['schema_version:1 / input_ref:Input1','goals:G1=UPDATE_TASK,G2=NEW_TASK','references:report=RECENT_TASK', '                 table=INPUT_SELECTION', 'relations:USE_RESULT(G1,G2)', 'constraints:FORBID_ACTION(G2,SEND)', 'unparsed_spans:[]'],13)
            arrow('engine','tools','2  코드가 다음 읽기 선택',(824,600),sp='R',tp='R',via=[(855,713),(855,488)],different=True,size=19)
            arrow('tools','engine',sp='R',tp='R',sd=15,td=16,via=[(875,503),(875,729)],ret=True,different=True)
            arrow('engine','frame','필요한 항목만 모델에 해석 요청',(116,548),sp='L',tp='B',via=[(470,713),(470,541),(275,541)],different=True,size=18)
            datum(500,545,300,125,'선택적 부분 요청·반환', ['질문: 답변이 가리킨 업무는?','후보: Task1 / Task2','후보의 상태·조회 시점 포함','→ 항목의 의미 / 불확실성'],17.5)
            p.text(500+dx,750,'코드: 항목 상태 → 전체 결합',19,COMMON)
            datum(500,772,300,125,'항목 상태 (설명용)', ['자료: 선택한 표','업무: Task1 / Task2 중 미정','질문 → 새 답 “예산 보고서”','업무: Task1로 연결'],18)
            arrow('engine','check','3  코드 결합 / 질문 / 지원 한계',(112,723),sp='L',tp='T',via=[(460,713),(460,889),(275,889)],different=True,size=21)
            arrow('check','engine',sp='T',tp='B',sd=100,td=-110,via=[(375,890),(480,890),(480,758),(540,758)],ret=True,different=True)
            arrow('engine','temp',sp='B',tp='R',sd=95,via=[(745,747),(835,747),(835,937)])
            p.text(925+dx,928,['B도 추가 모델 호출 가능','코드 연결 ≠ 의미 정답 보장'],17,COMMON,leading=24)
        datum(110,768,340,109,'의미 제안 (MeaningProposal)', ['Task1 보고서에 선택한 표 추가','보고서 결과 → 메일 초안','조건: 메일 발송 금지'],20)
        # Common reads remain direct to both authoritative source Components.
        if prefix=='A':
            yy=677
            for target,cy,offset,lane in [('cm',726,-10,906),('tm',851,12,913)]:
                arrow('tools',target,'2  같은 범위의 정보 조회' if target=='cm' else '',(925,671),sp='R',tp='L',sd=offset,td=-10,via=[(lane,yy+offset),(lane,cy-10)],size=17)
                arrow(target,'tools',sp='L',tp='R',sd=12,td=offset+10,via=[(lane+5,cy+12),(lane+5,yy+offset+10)],ret=True)
        else:
            yy=488
            for target,cy,offset,lane in [('cm',726,-10,900),('tm',851,12,909)]:
                arrow('tools',target,sp='R',tp='L',sd=offset,td=-10,via=[(lane,yy+offset),(lane,cy-10)])
                arrow(target,'tools',sp='L',tp='R',sd=12,td=offset+10,via=[(lane+5,cy+12),(lane+5,yy+offset+10)],ret=True)
        p.text(925+dx,776,'자료·업무 후보·조회 시점·검색 범위',16,COMMON)
        p.text(925+dx,891,'읽기 권한은 공통 Policy 계약',17,COMMON)
        # The shared authority/commit and delivery tail is deliberately subordinate.
        component('finish',485,1005,350,132,'Request Controller')
        node('adopted',515,1072,290,64,'채택 의미·질문·근거 버전','finish','store')
        component('saved',110,1020,300,62,'State Store')
        component('policy',925,1020,290,62,'Policy Manager')
        component('response',110,1165,300,62,'Response Manager')
        component('task',485,1165,350,62,'Task Manager')
        component('gateway',925,1165,290,62,'Agent Gateway')
        component('output',110,1280,300,62,'Interaction Manager')
        node('agent',925,1280,290,62,'Downstream Agent',kind='external')
        arrow('check','finish','공통 검증을 통과한 의미·질문 제안',(480,984),via=[(275,995),(660,995)],size=18)
        arrow('finish','saved','레코드 저장 요청 / 성공·실패',(110,997),sp='L',tp='R',sd=-30,td=0,via=[(446,1041),(446,1051)],size=17)
        arrow('saved','finish',sp='R',tp='L',sd=15,td=-5,ret=True)
        arrow('finish','policy','현재 권한 조회 / 허용·거절',(892,997),sp='R',tp='L',sd=-30,td=0,via=[(875,1041),(875,1051)],size=16)
        arrow('policy','finish',sp='L',tp='R',sd=15,td=-5,ret=True)
        arrow('finish','task','저장 성공·현재 유효 시 인계',(485,1142),sd=-90,td=-90,size=17)
        arrow('task','finish',sp='T',tp='B',sd=90,td=90,ret=True)
        arrow('task','gateway','조건 / 명령 식별자',(832,1143),sp='R',tp='L',sd=-12,td=-12,size=16)
        arrow('gateway','task',sp='L',tp='R',sd=14,td=14,ret=True)
        arrow('gateway','agent','업무 위임 / 접수·결과',(925,1240),sd=-60,td=-60,size=16)
        arrow('agent','gateway',sp='T',tp='B',sd=60,td=60,ret=True)
        arrow('finish','response','저장 성공 후 질문/응답 게시 요청',(111,1102),sp='L',tp='T',sd=42,via=[(445,1113),(445,1155),(260,1155)],size=17)
        arrow('response','finish',sp='R',tp='L',sd=15,td=52,via=[(455,1211),(455,1123)],ret=True)
        arrow('response','output','승인 Text / 음성',(110,1240),sd=-70,td=-70,size=17)
        arrow('output','response','실제 표시·재생·중단',(240,1240),sp='T',tp='B',sd=70,td=70,ret=True,size=16)
        arrow('response','ma',sp='R',tp='R',via=[(430,1196),(430,1268),(1234,1268),(1234,391)])
        p.text(485+dx,1280,['입력·자료·업무·질문이 바뀌었는지 검사','저장 성공 후 채택 / 실제 전달 후 답변 연결','응답 구성·음성 생성은 Model Access 경유'],17,COMMON,leading=23)
        p.text(925+dx,1340,'외부 업무 계획·실행 책임',16,COMMON)
    for item in p.items:
        if item['kind']=='line':item['width']=1.3
    append_plate_legend(p)
    p._add('node',x=1530,y=1380,w=60,h=28,name=[],type='external',color=COMMON,parent=None)
    p.text(1530,1408,'외부 대상',16,COMMON)
    # Replace the redundant deployment sample in this scene's key with the data-note key.
    p.items=[i for i in p.items if not (i['kind']=='box' and i.get('y')==1373 and (600<=i.get('x',0)<760 or 2210<=i.get('x',0)<2370)) and not (i['kind']=='text' and i.get('y')==1378 and i.get('x') in (638.5,721.5,2248.5,2331.5))]
    p._add('node',x=600,y=1373,w=160,h=28,name=[],type='data',color=COMMON,parent=None)
    for item in p.items:
        if item['kind']=='text' and item['x']==578 and item['y']==1408:item['lines']=['교환 자료 예시 / 실행 Module 아님']
        if item['kind']=='text' and item['x']==2160 and item['y']==1408:item['lines']=['설계 차이: 판단·생산 경로']
    p.line([(2210,1387),(2370,1387)],COMMON,width=1.3)
    p.items[-1]['different']=True
    return p
