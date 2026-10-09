"""DP41: Module collaboration above, a separate execution flowchart below.

Flowchart actions/decisions are explanatory control flow, not Architecture
Components or Modules. The document MAIN retains the complete common graph.
"""
from dp_comparison_structures import ResolutionGraph, COMMON, APRICOT, DIFFERENCE_STROKE


def document(g,key,x,y,w,h,lines,size=13):
    g.nodes[key]=(x,y,w,h)
    start=len(g.s.items)
    g.s.note(g.x+x,y,w,h,[],fill='white')
    g.s.items[-1].update(stroke=COMMON,line_width=1.0)
    g.s.label(g.x+x+10,y+6,w-28,'\n'.join(lines),size,COMMON)
    g.annotations.extend(g.s.items[start:])


def step(g,key,x,y,w,h,label,decision=False):
    # This region has its own notation: rectangles contain actions, not Modules.
    g.box(key,x,y,w,h,label,size=15)
    g.s.items[-2].update(flow_decision=decision,flow_action=not decision)


def draw(s,x,side):
    g=ResolutionGraph(s,x)
    if side==0: legend(s)
    document(g,'utterance',25,253,150,35,['사용자 발화'],16)
    g.box('input',210,253,240,35,'Interaction Manager',size=18)
    g.box('controller',495,253,250,35,'Request Controller',size=18)
    g.edge('utterance','input','R','L')
    g.edge('input','controller','R','L')
    g.group(15,342,835,210,'Request Interpreter')
    g.nodes['interpreter']=(15,342,835,210)
    g.text(635,348,185,'Module 협력 구조',size=12)
    g.edge('controller','interpreter','B','T',td=190,via=[(620,335),(622.5,335)])
    g.text(640,310,175,'해석 시작',size=17)
    g.box('llm',35,301,250,32,'클라우드 LLM',size=20)
    g.s.items[-2]['external']=True
    g.text(308,308,305,'Model Access 경유 호출',size=17)
    g.box('producer',35,385,245,36,'모델 주도 해석 제어기' if side==0 else '요청 틀 해석기',True,True,size=18)
    g.box('tools',660,385,170,36,'읽기 도구 실행기',module=True,size=15)
    g.box('check',610,506,210,32,'의미 제안 검증기',module=True,size=17)
    if side==0:
        # Actual Module-to-Module collaboration, with no flowchart diamonds.
        g.edge('producer','tools','R','L',sd=-8,td=-8)
        g.edge('tools','producer','L','R',sd=8,td=8,ret=True)
        g.text(310,375,285,'조회 요청 / 근거 반환',size=14)
        document(g,'read',320,432,270,57,
                 ['조회 요청 (ReadRequest)','“아까 보고서” / “이 표”'],13)
        document(g,'evidence',610,432,220,57,
                 ['조회 결과 (EvidenceBundle)','Task1 / Task2 + 선택한 표'],13)
        g.edge('producer','check','L','L',via=[(25,403),(25,545),(590,545),(590,522)])
    else:
        g.box('engine',325,385,300,36,'Request Resolution Engine',True,True,size=17)
        g.edge('producer','engine','R','L',sd=-8,td=-8)
        g.edge('engine','producer','L','R',sd=8,td=8,via=[(310,411),(310,426),(292,426),(292,411)])
        g.text(305,365,335,'요청 틀 / 부분 해석 결과',size=13)
        g.edge('engine','tools','R','L',sd=-8,td=-8)
        g.edge('tools','engine','L','R',sd=8,td=8,ret=True)
        g.text(642,365,195,'조회 / 근거',size=13)
        document(g,'frame',35,435,245,54,
                 ['요청 틀 (RequestFrame)','표 / 보고서: 미확정','보고서 결과 → 초안 / 발송 금지'],12)
        document(g,'items',325,435,285,54,
                 ['Engine 해결 상태','보고서 후보: Task1 / Task2','사용자 답변 뒤 Task1 연결'],12)
        document(g,'evidence',660,435,170,54,
                 ['조회 결과','후보 업무 / 선택한 표'],12)
        g.edge('engine','check','R','T',sd=14,via=[(642,417),(642,498),(715,498)])
    document(g,'meaning',35,496,530,47,
             [('모델' if side==0 else '코드')+'의 의미 제안 (MeaningProposal): Task1에 표 추가',
              '수정된 보고서 결과 → 메일 초안 / 메일 발송 금지'],13)
    for key,xx,name,yy in [('context',318,'Context Manager',563),('task',605,'Task Manager',568)]:
        g.box(key,xx,578,230,27,name,size=17)
        tx=g.port(key,'T')[0];ry=g.port('tools','R',-7)[1]
        g.edge('tools',key,'R','T',sd=-7,via=[(854,ry),(854,yy),(tx,yy)])
        g.edge(key,'tools','T','R',sd=9,td=8,via=[(tx+9,yy+4),(846,yy+4),(846,g.port('tools','R',8)[1])],ret=True)
    g.text(35,580,260,'자료 / 대화 / 업무 기록 조회',size=15)
    g.text(310,555,505,'양방향 연결은 Module 간 호출 / 결과 반환',size=13)
    px=g.port('producer','L')[1]
    g.edge('producer','llm','L','L',via=[(20,px),(20,317)])
    g.edge('llm','producer','L','L',td=12,via=[(10,317),(10,px+12)],ret=True)
    # Separately bounded execution explanation. No edge connects these actions
    # to a Module or Component in the collaboration diagram above.
    flowchart(g,side)
    g.finish()


def flowchart(g,side):
    g.raw([(15,617),(850,617)],arrow=False)
    g.text(35,621,800,'실행 순서도 — 반복 제어와 종료 조건',size=16)
    if side==0:
        step(g,'f_model',35,657,180,34,'모델: 다음 행동 판단')
        step(g,'f_next',285,649,150,50,'다음 행동?',True)
        step(g,'f_read',535,657,220,34,'도구: 지정한 자료 조회')
        step(g,'f_out',590,746,235,32,'모델: 의미 / 질문 제안')
        g.edge('f_model','f_next','R','L')
        g.edge('f_next','f_read','R','L')
        g.text(450,650,70,'조회',size=14)
        g.edge('f_read','f_model','B','B',via=[(645,720),(125,720)])
        g.text(173,700,400,'근거를 받아 모델 재호출 / 필요하면 반복',size=14)
        g.edge('f_next','f_out','B','L',via=[(360,762)])
        g.text(380,742,200,'완성 / 확인 질문 필요',size=13)
    else:
        step(g,'f_frame',25,657,170,34,'모델: 요청 틀 생산')
        step(g,'f_code',240,657,155,34,'코드: 항목 검사')
        step(g,'f_next',440,649,140,50,'다음 처리?',True)
        step(g,'f_read',650,657,180,34,'도구: 자료 조회')
        step(g,'f_partial',35,746,220,32,'모델: 특정 항목 해석')
        step(g,'f_out',630,746,200,32,'코드: 의미 / 질문 제안')
        g.edge('f_frame','f_code','R','L')
        g.edge('f_code','f_next','R','L')
        g.edge('f_next','f_read','R','L')
        g.text(591,650,55,'조회',size=13)
        g.edge('f_read','f_code','B','B',via=[(740,713),(317.5,713)])
        g.text(555,716,270,'코드 검사 반복 (LLM 없이)',size=13)
        g.edge('f_next','f_partial','B','R',via=[(510,735),(270,735),(270,762)])
        g.text(286,746,222,'의미 해석이 필요한 항목',size=13)
        g.raw([(145,746),(145,731),(317.5,731),(317.5,713)],arrow=False)
        g.text(35,700,250,'부분 결과를 코드에 반환',size=13)
        g.edge('f_next','f_out','B','L',via=[(510,762)])
        g.text(532,742,95,'완성 / 질문',size=12)


def legend(s):
    s.rect(47,342,86,447,'white',COMMON);s.items[-1]['line_width']=1.0
    s.text(90,351,82,['Legend'],15,COMMON,True,'center')
    for y,name,label,kind,difference in [(379,'Component','논리 책임','component',False),
        (438,'Module','내부 구현','module',False),
        (497,'Data','교환 예시','document',False),
        (555,'LLM','외부 모델','external',False),
        (614,'Module','차이: 살구색','module',True)]:
        if kind=='document':
            s.note(53,y,74,27,[],fill='white');s.items[-1].update(stroke=COMMON,line_width=1.0)
            s.text(90,y+7,70,[name],10,COMMON,align='center')
        else:
            s.box(53,y,74,27,name,COMMON,fill=APRICOT if difference else 'white',size=10)
            s.items[-2].update(stroke=DIFFERENCE_STROKE if difference else COMMON,line_width=1.0,rounded=kind=='module',external=kind=='external')
        s.label(54,y+30,76,label,12,COMMON)
    for y,dashed,label in [(674,False,'요청'),(695,True,'반환')]:
        s.line([(54,y),(95,y)],color=COMMON,width=1.1,dashed=dashed)
        s.label(101,y-7,30,label,10,COMMON)
    s.label(54,715,76,'아래: 순서도',10,COMMON)
    s.box(53,733,30,18,'',COMMON,size=10);s.items[-2].update(line_width=1.0)
    s.label(89,736,35,'동작',10,COMMON)
    s.box(53,759,30,18,'',COMMON,size=10);s.items[-2].update(flow_decision=True,line_width=1.0)
    s.label(89,762,35,'분기',10,COMMON)
