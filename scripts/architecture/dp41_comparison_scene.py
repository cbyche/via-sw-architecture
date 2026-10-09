"""DP41's differing execution graphs and illustrative exchanged documents.

The documents are message examples, not Components or an adopted API schema.
Each alternative has its own complete, compact shared-owner path.
"""
from dp_comparison_structures import ResolutionGraph, COMMON, APRICOT, DIFFERENCE_STROKE


def document(g,key,x,y,w,h,lines,size=15):
    g.nodes[key]=(x,y,w,h)
    start=len(g.s.items)
    g.s.note(g.x+x,y,w,h,[],fill='white')
    g.s.items[-1].update(stroke=COMMON,line_width=1.0)
    g.s.label(g.x+x+12,y+8,w-27,'\n'.join(lines),size,COMMON)
    g.annotations.extend(g.s.items[start:])


def state(g,key,x,y,w,h,name,size=13):
    """Keep the name in the cylinder body, below its shallow top ellipse."""
    g.store(key,x,y,w,h,name,size=size)
    g.s.items[-2]['store_radius']=4
    g.s.items[-1]['y']=y+15


def draw(s,x,side):
    g=ResolutionGraph(s,x)
    if side==0: legend(s)
    g.group(15,268,835,305,'Request Interpreter')
    if side==0:
        g.text(35,310,800,'모델이 다음 읽기와 전체 의미를 결정',size=20)
        g.box('producer',35,360,235,52,'모델 주도 해석 제어기',True,True,size=18)
        g.box('tools',605,384,225,42,'읽기 도구 실행기',module=True,size=18)
        g.edge('producer','tools','R','T',via=[(287,386),(287,345),(717,345)])
        document(g,'read',320,339,270,73,
                 ['ReadRequest','tool: task_retrieval','selector: 아까 보고서'],16)
        g.edge('tools','producer','B','B',via=[(717,507),(152,507)],ret=True)
        document(g,'evidence',325,434,282,69,
                 ['EvidenceBundle','T7 / T8 + source revisions','두 후보 → 질문 / 답변 후 재판단'],15)
        g.text(35,448,277,'근거를 받고 다시 모델 판단',size=17)
        g.text(35,475,270,'“예산 보고서” 답변 뒤 T7',size=16)
        state(g,'temporary',35,526,235,42,'임시 해석 상태',size=14)
        g.box('check',605,532,225,30,'의미 제안 검증기',module=True,size=16)
        g.edge('producer','check','R','L',sd=18,via=[(291,404),(291,547)])
        document(g,'meaning',325,518,270,47,
                 ['MeaningProposal (모델 생산)','task: T7 / send: false'],15)
        g.edge('producer','temporary','L','L',via=[(27,386),(27,547)])
        g.edge('temporary','producer','L','L',sd=8,td=11,via=[(23,555),(23,397)],ret=True)
    else:
        g.text(35,308,800,'모델은 틀 / 코드는 항목 해결과 전체 결합',size=19)
        g.box('producer',35,344,220,37,'요청 틀 해석기',True,True,size=18)
        g.box('engine',320,432,307,43,'Request Resolution Engine',True,True,size=18)
        g.box('tools',660,432,170,43,'읽기 도구 실행기',module=True,size=15)
        g.edge('producer','engine','R','T',via=[(585,362),(585,418),(473,418)])
        document(g,'frame',288,343,287,78,
                 ['RequestFrame v1 (모델 생산)','goal: 표 추가 / target: UNRESOLVED','relation: 아까 보고서 / send: false'],14)
        g.edge('engine','tools','R','L',sd=-7,td=-7)
        g.edge('tools','engine','L','R',sd=9,td=9,ret=True)
        g.text(631,418,35,'조회',size=10)
        document(g,'candidates',652,480,190,43,
                 ['T7 / T8 → AMBIGUOUS','질문/답변 뒤 T7 → BOUND'],12)
        g.edge('engine','producer','L','B',via=[(280,453),(280,395),(145,395)])
        g.edge('producer','engine','B','L',sd=-25,td=12,via=[(120,403),(273,403),(273,465)],ret=True)
        document(g,'partial',35,407,232,84,
                 ['선택적 PartialInterpretation','field / candidateIds','evidenceVersions','부분 요청 → field result','전체 결합은 Engine 코드'],12)
        g.box('check',35,532,232,30,'의미 제안 검증기',module=True,size=16)
        g.edge('engine','check','B','R',via=[(473,547)])
        document(g,'meaning',320,490,307,69,
                 ['MeaningProposal (코드 생산)','질문 / “예산 보고서” 답변 뒤','task: T7 / send: false'],15)
        state(g,'temporary',660,529,170,39,'요청 해석 상태',size=13)
        g.edge('engine','temporary','R','L',sd=14,via=[(641,468),(641,551)])
        g.edge('temporary','engine','L','R',sd=6,td=19,via=[(638,557),(638,472)],ret=True)
    shared(g,side)
    g.finish()


def shared(g,side):
    """Same complete owners/ports repeated compactly inside each alternative."""
    g.text(35,576,800,'공통 조회/채택/전달 경로 / 10개 Component / 아래 자료 표기는 예시',size=13)
    g.box('context',35,596,230,28,'Context Manager',size=16)
    g.box('task',318,596,230,28,'Task Manager',size=16)
    g.box('model',620,636,220,28,'Model Access',size=16)
    g.text(625,612,205,'로컬 cloud adapter',size=12)
    g.group(35,644,235,76,'Request Controller')
    g.nodes['controller']=(35,644,235,76)
    state(g,'accepted',47,677,211,36,'채택 의미 / 질문 / 출처',size=11)
    g.box('policy',318,638,230,27,'Policy Manager',size=15)
    g.box('store',318,680,230,27,'State Store',size=15)
    g.group(35,726,235,63,'Interaction Manager')
    g.nodes['input']=(35,726,235,63)
    g.box('io',47,759,90,17,'Channel I/O',module=True,size=9)
    g.box('turn',146,759,112,17,'Turn-Taking Control',module=True,size=8)
    g.text(146,779,113,'로컬 VAD',size=9)
    g.box('response',318,734,230,28,'Response Manager',size=15)
    g.box('gateway',620,734,220,28,'Agent Gateway',size=15)
    for key,xx,name in [('voice',620,'클라우드\n음성 모델'),('llm',738,'클라우드\n의미 LLM')]:
        g.box(key,xx,686,102,27,name,size=10)
        g.s.items[-2]['external']=True
    g.box('agent',620,774,220,17,'Downstream Agent',size=11)
    g.s.items[-2]['external']=True
    # Same read executor talks directly to each source. No Context-only gate.
    for key,yy in [('context',585),('task',588)]:
        tx=g.port(key,'T')[0]
        ry=g.port('tools','R',-6)[1]
        g.edge('tools',key,'R','T',sd=-6,via=[(854,ry),(854,yy),(tx,yy)])
        g.edge(key,'tools','T','R',sd=9,td=7,via=[(tx+9,yy+3),(846,yy+3),(846,g.port('tools','R',7)[1])],ret=True)
    # Model-role dependencies are common, outside the main comparison graph.
    g.edge('producer','model','L','R',via=[(18,g.port('producer','L')[1]),(18,632),(856,632),(856,650)])
    g.edge('model','producer','R','L',sd=8,td=12,via=[(858,658),(858,629),(10,629),(10,g.port('producer','L',12)[1])],ret=True)
    g.edge('model','voice','B','T',sd=-50,via=[(680,674),(671,674)])
    g.edge('voice','model','T','B',sd=9,td=-41,via=[(680,679),(689,679)],ret=True)
    g.edge('model','llm','B','T',sd=55,via=[(785,674),(789,674)])
    g.edge('llm','model','T','B',sd=9,td=64,via=[(798,679),(794,679)],ret=True)
    # Interpretation invocation and proposal/adoption use independent lanes.
    g.edge('controller','producer','L','L',sd=-22,td=-9,via=[(16,655),(16,g.port('producer','L',-9)[1])])
    checkx=g.port('check','B')[0]
    g.edge('check','controller','B','T',via=[(checkx,640),(152,640)])
    g.edge('controller','policy','R','L',sd=-23,td=-4)
    g.edge('policy','controller','L','R',sd=8,td=-12,ret=True)
    g.edge('controller','store','R','L',sd=15,td=-5)
    g.edge('store','controller','L','R',sd=6,td=26,ret=True)
    # New input is distinct from actual presentation receipt/focus.
    g.edge('input','controller','T','B',sd=-65,td=-65)
    g.edge('controller','response','R','T',sd=5,via=[(287,682),(287,720),(433,720)])
    g.edge('response','input','L','R',sd=-6,td=-6)
    g.edge('input','response','R','L',sd=8,td=8,ret=True)
    g.edge('response','controller','R','L',sd=-9,td=4,via=[(567,739),(567,718),(22,718),(22,681)])
    # Conditional output content/voice calls remain explicit.
    g.edge('response','model','R','L',sd=1,td=-5,via=[(582,749),(582,645)])
    g.edge('model','response','L','R',sd=8,td=10,via=[(590,658),(590,758)],ret=True)
    g.edge('io','model','B','R',via=[(92,792),(852,792),(852,648)],td=-2)
    g.edge('model','io','R','B',sd=11,td=11,via=[(849,661),(849,788),(103,788)],ret=True)
    # Task adoption and external handoff stay code-controlled.
    g.edge('controller','task','R','L',sd=-29,td=8,via=[(288,648),(288,618)])
    g.edge('task','gateway','R','T',via=[(566,610),(566,725),(730,725)])
    g.edge('gateway','agent')
    g.text(320,712,231,'질문/focus / release / 실제 receipt',size=10)
    g.text(620,719,223,'조건부 구성/음성 / 위임/접수',size=10)


def legend(s):
    s.rect(47,277,86,512,'white',COMMON);s.items[-1]['line_width']=1.0
    s.text(90,292,82,['Legend'],15,COMMON,True,'center')
    for y,name,label,kind,difference in [(313,'Component','논리 책임','component',False),
        (369,'Module','내부 구현','module',False),
        (425,'Data','교환 예시','document',False),
        (481,'State','owner 상태','state',False),
        (537,'Service','외부 의존성','external',False),
        (692,'Module','차이: 살구색','module',True)]:
        if kind=='document':
            s.note(53,y,74,27,[],fill='white');s.items[-1].update(stroke=COMMON,line_width=1.0)
            s.text(90,y+7,70,[name],10,COMMON,align='center')
        elif kind=='state':
            s.store(53,y,74,27,name,size=10);s.items[-2].update(fill='white',stroke=COMMON,line_width=1.0);s.items[-1]['y']=y+7
        else:
            s.box(53,y,74,27,name,COMMON,fill=APRICOT if difference else 'white',size=10)
            s.items[-2].update(stroke=DIFFERENCE_STROKE if difference else COMMON,line_width=1.0,rounded=kind=='module',external=kind=='external')
        s.label(54,y+33,76,label,12,COMMON)
    for y,dashed,label in [(601,False,'요청 / 전달'),(648,True,'응답 / 반환')]:
        s.line([(54,y),(123,y)],color=COMMON,width=1.1,dashed=dashed)
        s.label(54,y+9,76,label,12,COMMON)
    s.label(54,750,76,'공통: 검정\n흰 바탕',12,COMMON)
