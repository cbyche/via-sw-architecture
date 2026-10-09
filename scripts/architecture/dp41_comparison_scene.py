"""DP41 entry, model-led repeat and code-led repeat with conditional model help.

Diamonds are control-flow branches, not Components or Modules. Data notes are
illustrative messages; the document MAIN retains the complete common graph.
"""
from dp_comparison_structures import ResolutionGraph, COMMON, APRICOT, DIFFERENCE_STROKE


def document(g,key,x,y,w,h,lines,size=15):
    g.nodes[key]=(x,y,w,h)
    start=len(g.s.items)
    g.s.note(g.x+x,y,w,h,[],fill='white')
    g.s.items[-1].update(stroke=COMMON,line_width=1.0)
    g.s.label(g.x+x+12,y+8,w-30,'\n'.join(lines),size,COMMON)
    g.annotations.extend(g.s.items[start:])


def condition(g,key,x,y,w,h,label):
    g.box(key,x,y,w,h,label,size=18)
    g.s.items[-2]['flow_decision']=True


def draw(s,x,side):
    g=ResolutionGraph(s,x)
    if side==0: legend(s)
    # The same input path activates each alternative; no correct Task is prebound.
    document(g,'utterance',25,253,150,35,['사용자 발화'],16)
    g.box('input',210,253,240,35,'Interaction Manager',size=18)
    g.box('controller',495,253,250,35,'Request Controller',size=18)
    g.edge('utterance','input','R','L')
    g.edge('input','controller','R','L')
    g.group(15,342,835,390,'Request Interpreter')
    g.nodes['interpreter']=(15,342,835,390)
    g.edge('controller','interpreter','B','T',td=190,via=[(620,335),(622.5,335)])
    g.text(640,310,175,'해석 시작',size=17)
    g.box('llm',35,301,250,32,'클라우드 LLM',size=20)
    g.s.items[-2]['external']=True
    g.text(308,308,305,'Model Access 경유 호출',size=17)
    if side==0:
        g.text(35,377,790,'A 반복: 모델 판단 → 조회 → 근거로 모델 재판단',size=20)
        g.box('producer',35,412,245,44,'모델 주도 해석 제어기',True,True,size=19)
        condition(g,'branch',326,406,175,56,'다음 행동?')
        g.box('tools',610,412,210,44,'읽기 도구 실행기',module=True,size=18)
        g.edge('producer','branch','R','L')
        g.edge('branch','tools','R','L')
        g.text(526,410,75,'조회',size=17)
        document(g,'read',322,481,255,69,
                 ['조회 요청 (ReadRequest)','“아까 보고서” 업무 조회','“이 표” 화면 조회'],15)
        document(g,'evidence',610,481,210,90,
                 ['조회 결과 (EvidenceBundle)','Task1: 예산 보고서','Task2: 실적 보고서','자료: 선택한 표'],14)
        g.edge('tools','producer','B','B',via=[(715,615),(157.5,615)],ret=True)
        g.text(35,491,268,'모델이 다음 행동을 선택',size=18)
        g.text(180,590,450,'조회 결과 → 다시 모델 판단 → 필요하면 반복',size=18)
    else:
        g.text(35,377,790,'B 반복: 코드가 항목 검사 → 조회 / 부분 해석 → 다시 검사',size=19)
        g.box('producer',35,412,245,44,'요청 틀 해석기',True,True,size=19)
        g.box('engine',340,412,300,44,'Request Resolution Engine',True,True,size=18)
        g.box('tools',660,491,170,44,'읽기 도구 실행기',module=True,size=15)
        condition(g,'branch',340,480,170,66,'다음 처리?')
        g.edge('producer','engine','R','L',sd=-8,td=-8)
        g.edge('engine','branch','B','T',via=[(490,470),(425,470)])
        g.edge('branch','tools','R','L')
        g.text(541,489,92,'조회',size=17)
        g.edge('tools','engine','R','R',td=10,via=[(842,513),(842,635),(650,635),(650,444)],ret=True)
        document(g,'frame',35,481,255,94,
                 ['요청 틀 (RequestFrame)','목표: 표 추가 + 메일 초안','표 / 보고서: 아직 미확정','보고서 결과 → 메일 / 발송 금지'],15)
        document(g,'items',340,553,285,61,
                 ['코드가 관리하는 해결 상태','보고서 후보: Task1 / Task2','답변 뒤 Task1로 연결'],14)
        g.text(660,618,177,'결과 반영 → 코드 검사',size=13)
        document(g,'evidence',660,552,170,61,
                 ['조회 결과','Task1 / Task2','선택한 표'],14)
        # Conditional model help returns into the same code-owned item loop.
        g.edge('engine','producer','L','R',sd=10,td=10,via=[(320,444),(320,470),(295,470),(295,444)])
        g.text(332,453,310,'항목 해석이 필요하면 LLM 재호출',size=16)
        document(g,'partial',35,588,255,46,
                 ['부분 해석 결과 → 코드에 반환','코드가 다시 항목 검사 / 전체 결합'],14)
    g.box('check',610,664,210,37,'의미 제안 검증기',module=True,size=17)
    if side==0:
        g.edge('branch','check','B','L',via=[(413.5,474),(590,474),(590,682.5)])
        g.text(330,617,260,'완성 의미 / 확인 질문 제안',size=16)
    else:
        g.edge('branch','check','B','T',via=[(425,548),(645,548),(645,644),(715,644)])
        g.text(340,617,300,'완성 의미 / 확인 질문 제안',size=16)
    document(g,'meaning',35,642,530,69,
             [('모델이 만든 의미 제안' if side==0 else '코드가 만든 의미 제안')+' (MeaningProposal)',
              '예산 보고서(Task1)에 선택한 표 추가',
              '수정된 보고서 결과 → 메일 초안 / 메일 발송 금지'],15)
    for key,xx,name,yy in [('context',318,'Context Manager',739),('task',605,'Task Manager',744)]:
        g.box(key,xx,759,230,30,name,size=18)
        tx=g.port(key,'T')[0];ry=g.port('tools','R',-7)[1]
        g.edge('tools',key,'R','T',sd=-7,via=[(854,ry),(854,yy),(tx,yy)])
        g.edge(key,'tools','T','R',sd=9,td=8,via=[(tx+9,yy+4),(846,yy+4),(846,g.port('tools','R',8)[1])],ret=True)
    g.text(35,759,260,'화면/대화 / 업무 기록 조회',size=16)
    px=g.port('producer','L')[1]
    g.edge('producer','llm','L','L',via=[(20,px),(20,317)])
    g.edge('llm','producer','L','L',td=12,via=[(10,317),(10,px+12)],ret=True)
    g.finish()


def legend(s):
    s.rect(47,342,86,447,'white',COMMON);s.items[-1]['line_width']=1.0
    s.text(90,351,82,['Legend'],15,COMMON,True,'center')
    for y,name,label,kind,difference in [(379,'Component','논리 책임','component',False),
        (438,'Module','내부 구현','module',False),
        (497,'Data','교환 예시','document',False),
        (552,'분기','흐름 판단','decision',False),
        (721,'Module','차이: 살구색','module',True)]:
        if kind=='document':
            s.note(53,y,74,27,[],fill='white');s.items[-1].update(stroke=COMMON,line_width=1.0)
            s.text(90,y+7,70,[name],10,COMMON,align='center')
        else:
            s.box(53,y,74,27,name,COMMON,fill=APRICOT if difference else 'white',size=10)
            s.items[-2].update(stroke=DIFFERENCE_STROKE if difference else COMMON,line_width=1.0,rounded=kind=='module',flow_decision=kind=='decision')
        s.label(54,y+30,76,label,12,COMMON)
    s.box(53,607,74,27,'LLM',COMMON,size=10);s.items[-2].update(external=True,line_width=1.0)
    s.label(54,636,76,'외부 모델',12,COMMON)
    for y,dashed,label in [(662,False,'요청'),(688,True,'반환')]:
        s.line([(54,y),(95,y)],color=COMMON,width=1.1,dashed=dashed)
        s.label(101,y-7,30,label,10,COMMON)
    s.label(54,771,76,'공통: 검정',10,COMMON)
