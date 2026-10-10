"""DP41: Module collaboration above, a separate execution flowchart below.

Flowchart actions/decisions are explanatory control flow, not Architecture
Components or Modules. The document MAIN retains the complete common graph.
"""
from dp_comparison_structures import ResolutionGraph, COMMON, APRICOT, DIFFERENCE_STROKE
from dp41_output_contract import STRUCTURER
from unicodedata import east_asian_width


class FeedbackGraph(ResolutionGraph):
    def text(self,x,y,w,text,color=None,size=16):
        # Integer width accumulation gives identical scenes on Python 3.12/3.14.
        before=len(self.s.items)
        lines=text.split('\n')
        units=max(sum(100 if east_asian_width(ch) in 'WF' else 58 for ch in line)
                  for line in lines)
        width=min(w,units*size/100+4)
        self.s.rect(self.x+x-2,y-1,width+4,min(len(lines)*size*1.2+2,794-(y-1)),'white','none')
        self.s.label(self.x+x,y,w,text,size,color or COMMON)
        self.annotations.extend(self.s.items[before:])


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
    """Emphasize the distinct feedback destinations, not only box names."""
    g=FeedbackGraph(s,x)
    if side==0: legend(s,expanded=True)
    document(g,'utterance',25,253,150,35,['사용자 발화'],16)
    g.box('input',210,253,240,35,'Interaction Manager',size=18)
    g.box('controller',495,253,250,35,'Request Controller',size=18)
    g.edge('utterance','input','R','L')
    g.edge('input','controller','R','L')
    g.box('llm',35,303,265,32,'클라우드 LLM',size=20)
    g.s.items[-2]['external']=True
    g.text(335,310,490,'Model Access 경유 / 조회 결과마다 모델 재판단' if side==0 else
           'Model Access 경유 / 초기 프레임과 조건부 부분 해석',size=13)
    g.text(350 if side==0 else 35,349,480 if side==0 else 800,
           'A: 모델이 결과를 보고 다음 조회 선택' if side==0 else
           'B: 항목별 해결 상태를 보고 Engine 코드가 다음 처리 결정',size=18 if side==0 else 19)
    g.group(15,380,835,300,'Request Interpreter')
    if side==0:
        # Leave both model round-trip lanes visible across the Component header.
        g.s.items[-1]['x']=g.x+350
    g.nodes['interpreter']=(15,380,835,300)
    g.text(650,328 if side==0 else 350,185,'해석 시작',size=15)
    if side==0:
        g.box('producer',35,422,265,40,'모델 주도 해석 제어기',True,True,size=18)
        g.text(35,466,280,'코드: 모델 호출, 도구 결과 대응',size=12)
        g.box('tools',570,422,235,40,'읽기 도구 실행기',module=True,size=17)
        g.store('state',35,523,265,55,'모델에 제공할 해석 맥락',size=16)
        g.text(35,582,265,'발화, 조회 결과, 이전 조건 / 임시 상태',size=11)
        # The real model is part of every round, not a one-time initializer.
        g.edge('producer','llm','T','B',sd=-100,td=-100,color=DIFFERENCE_STROKE)
        g.edge('llm','producer','B','T',sd=110,td=110,ret=True,color=DIFFERENCE_STROKE)
        g.text(80,350,182,'① 결과와 원문으로 재호출',size=12)
        g.text(80,368,182,'모델: 조회 / 질문 / 의미 선택',size=11)
        g.edge('producer','tools','R','L',color=DIFFERENCE_STROKE)
        g.text(330,430,220,'② 모델이 선택한 조회',size=14)
        # A large return path encloses the two successive output examples.
        g.edge('tools','state','R','R',via=[(825,442),(825,611),(325,611),(325,550.5)],ret=True,color=DIFFERENCE_STROKE)
        g.text(350,466,455,'모델 출력 예시 / 다음 조회는 결과를 본 뒤 선택',size=12)
        g.edge('state','producer','T','B',sd=110,td=110,color=DIFFERENCE_STROKE)
        g.text(35,497,232,'③ 결과 반영 후 같은 모델 재호출',size=12)
        document(g,'read',350,484,455,52,
                 ['첫 모델 판단: 보고서와 표 조회',
                  'task_retrieve(query="보고서", limit=5)',
                  'context_retrieve(selector="INPUT_SELECTION")'],11)
        g.text(350,538,455,'조회 결과: Task1 예산 보고서 / Task2 실적 보고서 / 선택한 표',size=11)
        document(g,'read_again',350,555,455,52,
                 ['결과를 받은 다음 모델 판단: 추가 대화 조회',
                  'interaction_retrieve(query="보고서", limit=5)',
                  '추가 결과도 같은 맥락으로 반환 / 모델 재판단'],11)
        g.box('check',570,632,235,32,'의미 제안 검증기',module=True,size=17)
        g.edge('producer','check','R','R',sd=12,via=[(835,454),(835,648)])
        document(g,'meaning',35,612,490,52,
                 ['⑤ 모델이 만든 MeaningProposal (사용자 확인 뒤)',
                  'Task1 보고서 + 표 / 보고서 결과로 메일 초안 / SEND 금지'],12)
        g.text(570,614,235,'④ 의미 또는 확인 질문 제안',size=12)
    else:
        g.box('producer',35,422,245,40,STRUCTURER,True,True,size=18)
        g.edge('producer','llm','L','L',via=[(20,442),(20,319)])
        g.edge('llm','producer','L','L',td=10,via=[(10,319),(10,452)],ret=True)
        # Engine is a contained Module, its children are code functions/Modules.
        g.group(340,414,470,195,'Request Resolution Engine',execution=True)
        g.s.items[-2]['rounded']=True
        g.s.items[-1]['size']=18
        g.nodes['engine']=(340,414,470,195)
        g.box('inspect',360,450,190,34,'프레임 / 항목 검사기',True,True,size=15)
        g.box('resolve',575,450,215,34,'미해결 항목 처리기',True,True,size=16)
        g.store('state',360,516,190,53,'항목별 해결 상태',size=16)
        g.box('bind',575,543,215,34,'관계 결합기',True,True,size=16)
        g.box('tools',575,632,215,32,'읽기 도구 실행기',module=True,size=16)
        g.box('check',325,632,215,32,'의미 제안 검증기',module=True,size=16)
        g.edge('producer','inspect','R','L',sd=-5,via=[(315,437),(315,467)])
        g.edge('inspect','resolve','R','L',color=DIFFERENCE_STROKE)
        g.edge('resolve','tools','R','R',via=[(835,467),(835,648)],color=DIFFERENCE_STROKE)
        g.text(575,490,240,'② 코드가 선택한 조회',size=12)
        g.text(575,504,235,'task_retrieve / context_retrieve',size=10)
        g.edge('tools','state','L','L',via=[(558,648),(558,677),(310,677),(310,542.5)],ret=True,color=DIFFERENCE_STROKE)
        g.s.label(g.x+575,668,245,'③ 조회 결과를 해결 상태에 반영',11,COMMON)
        g.edge('inspect','state','B','T',sd=-25,td=-25,color=DIFFERENCE_STROKE)
        g.edge('state','inspect','T','B',sd=25,td=25,color=DIFFERENCE_STROKE)
        g.text(575,519,235,'④ 해결 상태로 코드 재검사',size=11)
        g.edge('state','bind','R','L',via=[(562,542.5),(562,560)],color=DIFFERENCE_STROKE)
        g.text(356,577,200,'report: 두 후보 → 확인 뒤 Task1\ntable: 선택한 표 연결',size=10)
        g.edge('bind','check','B','T',via=[(682.5,622),(432.5,622)],color=DIFFERENCE_STROKE)
        g.text(575,583,230,'⑤ 지원 연산으로 전체 의미 구성',size=11)
        # Only scoped semantic help returns to the same code-owned loop.
        g.edge('resolve','producer','R','R',td=10,via=[(828,467),(828,405),(300,405),(300,452)])
        g.text(35,467,265,'필요 시: 특정 항목만 모델에 해석 요청',size=11)
        document(g,'frame',35,490,245,120,
                 ['모델 출력: RequestFrame (필드 축약)',
                  'schema_version:1 / input_ref:Input1',
                  'goals: G1=UPDATE_TASK(report,table)',
                  '  G2=NEW_TASK(메일 초안)',
                  'references: report=RECENT_TASK("보고서")',
                  '  table=INPUT_SELECTION',
                  'relations: USE_RESULT(G1 → G2)',
                  'constraints: FORBID_ACTION(G2, SEND)',
                  'unparsed_spans:[] / 실제 Task는 미확정'],10)
        document(g,'meaning',35,614,245,52,
                 ['코드가 만든 MeaningProposal',
                  '확인 뒤 Task1 + 표 / 결과로 초안',
                  '전송 금지 유지'],11)
        g.text(345,384,480,'다음 처리 결정과 전체 결합: Engine 코드',size=13)
    # Start at the real receiving Module, without routing through header text.
    trigger=g.port('producer','T',120 if side==0 else 110)[0]
    g.edge('controller','producer','B','T',td=120 if side==0 else 110,
           via=[(620,296),(842,296),(842,375),(315,375),(315,414),(trigger,414)])
    for key,xx in [('context',318),('task',605)]:
        g.box(key,xx,698,230,27,'Context Manager' if key=='context' else 'Task Manager',size=17)
        tx=g.port(key,'T')[0];ry=g.port('tools','R',-7)[1]
        g.edge('tools',key,'R','T',sd=-7,via=[(854,ry),(854,687),(tx,687)])
        g.edge(key,'tools','T','R',sd=9,td=8,via=[(tx+9,692),(846,692),(846,g.port('tools','R',8)[1])],ret=True)
    g.text(35,700,270,'정보 / 대화 / Task 기록 조회',size=15)
    # Separate flowchart: retain the prior notation, move it below the graph.
    first=len(g.s.items)
    flowchart(g,side)
    for item in g.s.items[first:]:
        if 'y' in item:item['y']+=76
        if 'points' in item:item['points']=[(xx,yy+76) for xx,yy in item['points']]
    g.finish()


def flowchart(g,side):
    g.raw([(15,666),(850,666)],arrow=False)
    g.text(35,669,800,'실행 순서도 — 반복 제어와 종료 조건',size=14)
    if side==0:
        step(g,'f_model',35,697,180,28,'모델: 다음 행동 판단')
        step(g,'f_next',285,690,150,42,'다음 행동?',True)
        step(g,'f_read',535,697,220,28,'도구: 지정한 자료 조회')
        step(g,'f_out',590,759,235,29,'모델: 의미 / 질문 제안')
        g.edge('f_model','f_next','R','L')
        g.edge('f_next','f_read','R','L')
        g.text(450,694,70,'조회',size=14)
        g.edge('f_read','f_model','B','B',via=[(645,741),(125,741)])
        g.text(173,722,400,'조회 결과로 모델 재호출 / 필요하면 반복',size=14)
        g.edge('f_next','f_out','B','L',via=[(360,773.5)])
        g.text(380,755,200,'완성 / 확인 질문 필요',size=13)
    else:
        step(g,'f_frame',25,697,170,28,'모델: RequestFrame')
        step(g,'f_code',240,697,155,28,'코드: 항목 검사')
        step(g,'f_next',440,690,140,42,'다음 처리?',True)
        step(g,'f_read',650,697,180,28,'도구: 자료 조회')
        step(g,'f_partial',35,759,220,29,'모델: 특정 항목 해석')
        step(g,'f_out',630,759,200,29,'코드: 의미 / 질문 제안')
        g.edge('f_frame','f_code','R','L')
        g.edge('f_code','f_next','R','L')
        g.edge('f_next','f_read','R','L')
        g.text(591,694,55,'조회',size=13)
        g.edge('f_read','f_code','B','B',via=[(740,741),(317.5,741)])
        g.text(600,743,220,'해결 상태 갱신 후 코드 재검사',size=13)
        g.edge('f_next','f_partial','B','R',via=[(510,752),(270,752),(270,773.5)])
        g.text(286,755,222,'의미 해석이 필요한 항목',size=13)
        g.raw([(145,759),(145,749),(317.5,749),(317.5,741)],arrow=False)
        g.text(35,730,250,'부분 결과를 코드에 반환',size=13)
        g.edge('f_next','f_out','B','L',via=[(510,773.5)])
        g.text(522,755,72,'완성/질문',size=10)


def legend(s,expanded=False):
    # 42/44/45 reuse the old compact key; DP41 adds its state/flowchart entries.
    s.rect(47,342,86,525 if expanded else 447,'white',COMMON);s.items[-1]['line_width']=1.0
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
    if expanded:
        s.store(53,668,74,27,'State',size=10)
        s.items[-2].update(fill='white',stroke=COMMON,line_width=1.0)
        s.items[-1]['y']-=5
        s.label(54,699,76,'소유 상태',12,COMMON)
    for y,dashed,label in ([(728,False,'요청'),(749,True,'반환')] if expanded else [(674,False,'요청'),(695,True,'반환')]):
        s.line([(54,y),(95,y)],color=COMMON,width=1.1,dashed=dashed)
        s.label(101,y-7,30,label,10,COMMON)
    s.label(54,775 if expanded else 715,76,'아래: 순서도',10,COMMON)
    s.box(53,797 if expanded else 733,30,18,'',COMMON,size=10);s.items[-2].update(line_width=1.0)
    s.label(89,800 if expanded else 736,35,'동작',10,COMMON)
    s.box(53,830 if expanded else 759,30,18,'',COMMON,size=10);s.items[-2].update(flow_decision=True,line_width=1.0)
    s.label(89,833 if expanded else 762,35,'분기',10,COMMON)
