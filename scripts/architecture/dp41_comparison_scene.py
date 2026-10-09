"""Focused DP41 presentation: reads, meaning producers and exchanged examples.

Common capture, adoption, storage, response and delegation are summarized outside
this diagram. The document MAIN retains their complete Component graph.
"""
from dp_comparison_structures import ResolutionGraph, COMMON, APRICOT, DIFFERENCE_STROKE


def document(g,key,x,y,w,h,lines,size=18):
    g.nodes[key]=(x,y,w,h)
    start=len(g.s.items)
    g.s.note(g.x+x,y,w,h,[],fill='white')
    g.s.items[-1].update(stroke=COMMON,line_width=1.0)
    g.s.label(g.x+x+12,y+9,w-30,'\n'.join(lines),size,COMMON)
    g.annotations.extend(g.s.items[start:])


def draw(s,x,side):
    g=ResolutionGraph(s,x)
    if side==0: legend(s)
    g.group(15,313,835,419,'Request Interpreter')
    g.box('llm',35,259,250,38,'클라우드 LLM',size=20)
    g.s.items[-2]['external']=True
    g.text(308,269,530,'모델 호출은 Model Access 경유',size=18)
    if side==0:
        g.box('producer',35,382,250,60,'모델 주도 해석 제어기',True,True,size=19)
        g.text(35,347,800,'모델이 다음 조회와 전체 의미를 판단',size=21)
        g.box('tools',605,414,225,46,'읽기 도구 실행기',module=True,size=19)
        g.edge('producer','tools','R','T',via=[(300,412),(300,374),(717.5,374)])
        document(g,'read',325,388,265,83,
                 ['① 조회 요청 (ReadRequest)','업무 조회: “아까 보고서”','화면 조회: “이 표”'],18)
        g.edge('tools','producer','B','B',via=[(717.5,600),(160,600)],ret=True)
        document(g,'evidence',325,487,282,104,
                 ['② 조회 결과 (EvidenceBundle)','Task1: 예산 보고서','Task2: 실적 보고서','자료: 선택한 표'],18)
        g.text(35,496,270,'③ 근거를 보고 모델 재판단',size=18)
        g.text(35,531,270,'추가 조회 / 확인 질문 / 완성',size=17)
    else:
        g.box('producer',35,369,235,44,'요청 틀 해석기',True,True,size=19)
        g.text(35,347,250,'모델은 요청 틀을 생산',size=18)
        g.box('engine',320,495,300,45,'Request Resolution Engine',True,True,size=18)
        g.box('tools',660,495,170,45,'읽기 도구 실행기',module=True,size=15)
        g.edge('producer','engine','R','T',via=[(290,391),(290,484),(470,484)])
        document(g,'frame',320,349,300,125,
                 ['① 요청 틀 (RequestFrame)','목표: 표 추가 + 메일 초안','표 / 보고서: 아직 미확정','관계: 보고서 결과 → 메일','조건: 메일 발송 금지'],18)
        g.edge('engine','tools','R','L',sd=-8,td=-8)
        g.edge('tools','engine','L','R',sd=9,td=9,ret=True)
        g.text(635,469,195,'② 코드가 조회를 선택',size=16)
        g.edge('engine','producer','L','B',via=[(290,517.5),(290,429),(152.5,429)])
        g.edge('producer','engine','B','L',sd=-25,td=12,via=[(127.5,421),(279,421),(279,529.5)],ret=True)
        document(g,'partial',35,449,235,106,
                 ['필요할 때만 부분 해석','모델에 특정 항목을 질문','예: 답변이 가리킨 업무','모델 결과 → 코드에 반환'],17)
        document(g,'items',320,552,300,62,
                 ['③ 항목을 코드로 해결','보고서: Task1 / Task2 → 질문'],18)
        g.text(660,558,180,'답변 뒤 Task1 연결',size=15)
        g.text(660,586,180,'코드가 전체 의미 결합',size=15)
    g.box('check',605,667,225,42,'의미 제안 검증기',module=True,size=18)
    if side==0:
        g.edge('producer','check','L','L',via=[(25,412),(25,718),(595,718),(595,688)])
    else:
        g.edge('engine','check','R','T',sd=12,via=[(642,529.5),(642,650),(717.5,650)])
    document(g,'meaning',35,623,550,86,
             [('④ 모델이 만든 의미 제안' if side==0 else '④ 코드가 만든 의미 제안')+' (MeaningProposal)',
              '예산 보고서(Task1)에 선택한 표 추가',
              '수정된 보고서 결과 → 메일 초안 / 메일 발송 금지'],18)
    # Direct owner reads are the only common Component paths needed here.
    for key,xx,name,yy in [('context',318,'Context Manager',739),('task',605,'Task Manager',744)]:
        g.box(key,xx,759,230,30,name,size=18)
        tx=g.port(key,'T')[0];ry=g.port('tools','R',-7)[1]
        g.edge('tools',key,'R','T',sd=-7,via=[(854,ry),(854,yy),(tx,yy)])
        g.edge(key,'tools','T','R',sd=9,td=8,via=[(tx+9,yy+4),(846,yy+4),(846,g.port('tools','R',8)[1])],ret=True)
    g.text(35,759,260,'화면/대화 / 업무 기록 조회',size=16)
    # Host modules call the same cloud LLM; the modules are not models.
    px=g.port('producer','L')[1]
    g.edge('producer','llm','L','L',via=[(20,px),(20,278)])
    g.edge('llm','producer','L','L',td=12,via=[(10,278),(10,px+12)],ret=True)
    g.finish()


def legend(s):
    s.rect(47,313,86,476,'white',COMMON);s.items[-1]['line_width']=1.0
    s.text(90,328,82,['Legend'],15,COMMON,True,'center')
    for y,name,label,kind,difference in [(363,'Component','논리 책임','component',False),
        (429,'Module','내부 구현','module',False),
        (495,'Data','교환 예시','document',False),
        (561,'Model','외부 모델','external',False),
        (701,'Module','차이: 살구색','module',True)]:
        if kind=='document':
            s.note(53,y,74,27,[],fill='white');s.items[-1].update(stroke=COMMON,line_width=1.0)
            s.text(90,y+7,70,[name],10,COMMON,align='center')
        else:
            s.box(53,y,74,27,name,COMMON,fill=APRICOT if difference else 'white',size=10)
            s.items[-2].update(stroke=DIFFERENCE_STROKE if difference else COMMON,line_width=1.0,rounded=kind=='module',external=kind=='external')
        s.label(54,y+33,76,label,12,COMMON)
    for y,dashed,label in [(621,False,'요청 / 전달'),(660,True,'응답 / 반환')]:
        s.line([(54,y),(123,y)],color=COMMON,width=1.1,dashed=dashed)
        s.label(54,y+9,76,label,12,COMMON)
    s.label(54,759,76,'공통: 검정',12,COMMON)
