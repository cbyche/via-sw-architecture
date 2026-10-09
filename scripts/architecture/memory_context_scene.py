"""45's shared editable scene: owner reads versus published past evidence.

Drawing and route validation only; no memory producer, model or Agent is run.
"""
from continuous_interaction_scene import Option, legend

COMPONENTS={'Request Controller','Context Manager','Request Interpreter','Task Manager','Response Manager','Model Access'}


class MemoryOption(Option):
    def text(self,*args,**kwargs):
        super().text(*args,**kwargs)
        item=self.s.items[-1]
        item['leading']=item['size']*1.32
        item['h']=len(item['lines'])*item['leading']
        if len(self.s.items)>1 and self.s.items[-2].get('label_for')==item['id']:
            self.s.items[-2]['h']=item['h']+3*self.scale


def draw_option(s,x,y,scale,side):
    start=len(s.items)
    g=MemoryOption(s,x,y,scale,side)
    g.node('rc',20,20,300,55,'Request Controller',size=23)
    g.node('rm',430,20,280,55,'Response Manager',size=23)
    g.node('tm',820,20,310,55,'Task Manager',size=23)
    g.node('cm',380,175,755,430,'Context Manager',size=25)
    g.node('ri',20,460,300,125,'Request Interpreter',size=23)
    g.node('read',40,516,260,42,'읽기 도구 실행기','module',owner='ri',size=20)
    g.node('ma',20,625,300,45,'Model Access',size=23)
    g.node('omni',440,625,290,45,'클라우드 의미 LLM','external',size=21)
    g.node('um',870,527,240,46,'User Memory','state',owner='cm',size=21)
    g.text(25,82,300,'C20: 과거 대화 / 정정\nInput1 / R23: 현재 요청',18)
    g.text(435,82,280,'E20: 평가 기준 원문\nP20: 실제 전달 구간',18)
    g.text(825,82,310,'T21 제품 조사 → D21 개정 2\nT22 견적 → D22 개정 1',16)
    g.text(870,483,248,'명시 저장 / 삭제만\n지속 관리, 자동 선호 추출 금지',15)
    if side=='A':
        provider='composer'
        g.node(provider,585,242,300,55,'Context Composer','module',True,'cm',24)
        g.node('cache',430,378,285,60,'Source View Cache','state',owner='cm',size=21)
        g.node('set',810,378,295,60,'Request Evidence Set','state',True,'cm',20)
        for key in ('rc','rm','tm'):
            g.edge(provider,key,sp='T',tp='B',sd=-12,td=-10)
            g.edge(key,provider,sp='B',tp='T',ret=True,sd=10,td=12)
        g.text(388,145,740,'2 owner 범위 읽기 ↑ / 원문・발췌・참조・revision・누락 ↓'.replace('・',' / '),17)
        g.edge(provider,'cache',sp='L',tp='T')
        g.edge('cache',provider,sp='T',tp='L',ret=True,sd=22,td=12)
        g.edge(provider,'set',sp='R',tp='T')
        g.text(425,320,690,'3 유효 cache 재사용 / 4 이번 요청의 근거 기록',18)
        g.text(430,447,290,'요약 / index / 관계 cache\n재사용 수명, 부분 갱신 가능',17)
        g.text(815,438,290,'R23 발췌 / 참조 / 읽은 범위\n요청 수명, 실행 Module 아님',15)
        g.text(425,550,430,'원문 해석은 RI / 별도 가공만 C-CONTEXT',14)
        g.edge(provider,'ma',sp='L',tp='R',sd=16,td=-9)
        g.edge('ma',provider,sp='R',tp='L',ret=True,sd=9,td=-16)
        g.edge('rc','cache',sp='R',tp='L',sd=15)
        g.text(25,328,345,'변경 / EvidenceDispute\n→ 해당 파생 버전 사용 차단\n→ 원본 재검증 / 부분 갱신',16)
    else:
        provider='reader'
        g.node('publisher',430,242,285,55,'Memory Publisher','module',True,'cm',23)
        g.node('repo',820,242,285,74,'Episodic Memory\nRepository','state',True,'cm',21)
        g.node(provider,430,435,285,55,'Evidence Reader','module',True,'cm',23)
        g.text(700,185,420,'생산・갱신 경로'.replace('・',' / '),18,True)
        for key in ('rc','rm','tm'):
            g.edge(key,'publisher',sp='B',tp='T',sd=-17,td=-17)
            g.edge('publisher',key,sp='T',tp='B',sd=0,td=0)
            g.edge(key,'publisher',sp='B',tp='T',ret=True,sd=17,td=17)
        g.text(385,145,745,'1 확정 변경 ID・revision 알림 / 2 범위 읽기 ↔ 본문・참조 반환'.replace('・',' / '),16)
        g.edge('publisher','repo',sp='R',tp='L')
        g.text(725,221,405,'4 출처・권한・지원 종류 검사'.replace('・',' / '),16)
        g.text(725,325,405,'관계 + coverage 동일 revision 게시',14)
        g.text(430,347,695,'Memory1: P20 → E20[27:52] / MODEL_DERIVED',15)
        g.text(430,367,695,'“가격이 같은 경우 유지보수 기간을 우선합니다.”',14)
        g.text(430,387,695,'Memory2/3: HAS_RESULT / 확인된 결과 개정 참조',14)
        g.text(730,406,395,'Coverage1: EXPLAINS_ITEM[27:52]\ngap[0:27] / 게시 revision 1',13)
        g.edge(provider,'repo',sp='R',tp='B',sd=-12)
        g.edge('repo',provider,sp='B',tp='R',ret=True,sd=22,td=12)
        g.text(728,448,395,'게시 후보 / locator / Coverage1\nPARTIAL이면 부족한 범위 유지',14)
        g.edge(provider,'publisher',sp='T',tp='B',sd=-40,td=-40)
        g.edge('publisher',provider,sp='B',tp='T',ret=True,sd=35,td=35)
        g.text(25,317,345,'NOT_COVERED / 파생 오류\n→ 지원 범위 생산 / 재검증\n← 게시 revision / 상태\n게시 확인 뒤 Reader 재조회',14)
        g.text(425,502,435,'UNSUPPORTED_RELATION ≠ NOT_COVERED\nNO_MATCH ≠ DENIED / UNAVAILABLE',15)
        for key in ('rc','rm','tm'):
            g.edge(provider,key,sp='R',tp='R',sd=20,td=15)
            g.edge(key,provider,sp='R',tp='R',ret=True,sd=-15,td=-20)
        g.text(730,584,390,'현재 metadata / 인용・충돌 원문 확인\n삭제・철회 즉시 fence / 옛 job 게시 금지'.replace('・',' / '),15)
        g.edge('rc','publisher',sp='R',tp='L',sd=22,td=-19)
        g.text(25,408,345,'EvidenceDispute → 해당 버전 차단 / 재검증',12)
        g.edge('publisher','ma',sp='L',tp='R',sd=12,td=-9)
        g.edge('ma','publisher',sp='R',tp='L',ret=True,sd=9,td=-12)
        g.text(25,600,680,'C-CONTEXT: 지원 과거 항목만 조건부 호출 / 명시 ID 연결은 코드',13)
    g.edge('read',provider,sp='R',tp='L',sd=-12,td=-10)
    g.edge(provider,'read',sp='L',tp='R',ret=True,sd=10,td=12)
    g.text(25,193,345,'Query1: request / attempt / revision\n목적 / 과거 후보 / 원문 조건 / budget',15)
    g.text(25,250,345,'← Bundle1 + Receipt1 / PARTIAL\n실제 전달 예외 문구 / 두 결과 참조\n출처 / revision / 누락 범위',15)
    g.edge('rc','ri',sp='L',tp='L',sd=-9,td=-9)
    g.edge('ri','rc',sp='R',tp='R',ret=True,sd=12,td=24)
    g.text(25,430,345,'입력 / scope ↓  의미 제안 ↑ / 채택은 RC',14)
    g.edge('rc','tm',sp='T',tp='T')
    g.text(390,-15,745,'채택 후 T23 위임 Context → / 진행・질문・결과는 대화에 연결'.replace('・',' / '),16)
    g.edge('rc','um',sp='B',tp='L',sd=-40)
    g.edge('um','rc',sp='L',tp='B',ret=True,sd=12,td=-15)
    g.text(40,563,265,'도구 호출 / 결과로 반복 / Task 직접 조회',12,owner='ri')
    g.edge('read','tm',sp='R',tp='R',sd=10,td=19)
    g.edge('tm','read',sp='R',tp='R',ret=True,sd=-18,td=16)
    g.text(730,652,400,'현재 Task read → TM 직접\n채택 / T23 연결은 RC의 저장 후',14)
    g.edge('ri','ma')
    g.edge('ma','ri',sp='T',tp='B',ret=True,sd=25,td=25)
    g.edge('ma','omni',sp='R',tp='L')
    g.edge('omni','ma',sp='L',tp='R',ret=True,sd=12,td=12)
    g.text(25,687,1110,'동일 cloud model/API / 생산과 현재 해석 call 분리 / 로컬 VAD와 지속 capture는 생산 대기 없음',15)
    # Labels are emitted after connectors, within their actual owner or outside names.
    for label in g.labels:g.text(*label,size=17)
    names={n['name'] for n in g.nodes.values() if n['kind']=='component'}
    assert names==COMPONENTS and len([n for n in g.nodes.values() if n['kind']=='component'])==6
    assert {(a,b) for a,b,_,_,_ in g.lines}>={(provider,'read'),('read',provider),('read','tm'),('tm','read'),('rc','ri'),('ri','rc'),('rc','tm'),('ri','ma'),('ma','omni')}
    if side=='A':assert ('composer','tm') in {(a,b) for a,b,_,_,_ in g.lines}
    for key,n in g.nodes.items():
        if n['kind'] in ('module','state'):
            assert n['owner']==('ri' if key=='read' else 'cm')
            p=g.nodes[n['owner']];assert p['x']<=n['x'] and p['y']+50<=n['y'] and n['x']+n['w']<=p['x']+p['w'] and n['y']+n['h']<=p['y']+p['h'],key
    # Same geometric guard for MAIN, draw.io and editable presentation scenes.
    for a,b,pts,_,_ in g.lines:
        ancestors=set()
        for key in (a,b):
            while g.nodes[key]['owner']:key=g.nodes[key]['owner'];ancestors.add(key)
        for key,n in g.nodes.items():
            if key in (a,b) or key in ancestors:continue
            for p,q in zip(pts,pts[1:]):
                x,y,w,h=(n[k] for k in ('x','y','w','h'))
                assert not (p[0]==q[0] and x+.1<p[0]<x+w-.1 and max(min(p[1],q[1]),y+.1)<min(max(p[1],q[1]),y+h-.1)),(a,b,key)
                assert not (p[1]==q[1] and y+.1<p[1]<y+h-.1 and max(min(p[0],q[0]),x+.1)<min(max(p[0],q[0]),x+w-.1)),(a,b,key)
    # White annotation masks and their text must follow all connectors in SVG
    # and native PPTX alike. Names stay with their owning shape.
    items=s.items[start:]
    annotation={i['label_for'] for i in items if i.get('label_for')}
    is_annotation=lambda i:i.get('label_for') or i['id'] in annotation
    s.items[start:]=([i for i in items if i['kind']!='line' and not is_annotation(i)]
                    +[i for i in items if i['kind']=='line']
                    +[i for i in items if is_annotation(i)])
    return g


def page_legend(s):
    legend(s,40,815,.75)
