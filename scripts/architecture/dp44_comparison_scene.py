"""DP44: real completion destinations above, execution traces separately below."""
from dp_comparison_structures import APRICOT, DIFFERENCE_STROKE, COMMON
from dp41_comparison_scene import FeedbackGraph, document, step, legend as base_legend


def v3_legend(s):
    """DP41 symbols, compacted vertically without changing their meaning."""
    before=len(s.items)
    base_legend(s,expanded=True)
    for index,item in enumerate(s.items[before:]):
        item['id']=f'legend_continuation_{index}'
        if 'y' in item:item['y']=342+(item['y']-342)*.84
        if 'h' in item:item['h']*=.84
        if 'points' in item:item['points']=[(x,342+(y-342)*.84) for x,y in item['points']]


def legend(s):
    before=len(s.items)
    base_legend(s)
    entries=[i for i in s.items[before:] if not (i.get('y',0)>=715 and i.get('x',0)<140)]
    for index,i in enumerate(entries):
        i['id']=f'legend44_{index}'
        if i.get('external') and i.get('y')==555:
            i.update(kind='store',external=False,store_radius=5)
        if i.get('lines')==['LLM']:i['lines']=['State']
        if i.get('lines')==['외부 모델']:i['lines']=['owner 상태']
    s.items=s.items[:before]+entries
    s.label(54,736,76,'공통: 검정',11,COMMON)
    s.label(54,758,76,'차이: 살구색',11,COMMON)

def state(g,key,x,y,w,h,name,size=16):
    before=len(g.s.items)
    g.store(key,x,y,w,h,name,size=size)
    g.s.items[before].update(fill=APRICOT,stroke=DIFFERENCE_STROKE,store_radius=5)
    g.s.items[-1]['y']=y+(h-size*1.2)/2


def draw(s,x,side):
    g=FeedbackGraph(s,x)
    if side==0:v3_legend(s)
    g.box('im',35,253,270,34,'Interaction Manager',size=18)
    g.box('task',525,253,300,34,'Task Manager',size=18)
    g.text(35,292,410,'Input1: “잠깐, 표부터 설명해줘”',size=15)
    g.text(525,292,320,'Task1 메일 / Question1 수신자 질문',size=14)
    g.group(25,323,810,148,'Request Controller')
    if side==0:
        g.box('dispatch',55,365,285,40,'Dialogue Dispatcher',True,True,size=19)
        state(g,'wait',475,365,325,40,'Dialogue Progress State',17)
        g.edge('im','dispatch','B','T',via=[(170,314),(197.5,314)])
        g.edge('task','dispatch','B','T',td=90,via=[(675,311),(287.5,311)])
        g.edge('dispatch','wait','R','L',color=DIFFERENCE_STROKE)
        g.edge('wait','dispatch','L','R',sd=9,td=9,via=[(458,394),(458,413),(356,413),(356,394)],ret=True,color=DIFFERENCE_STROKE)
        g.text(55,425,390,'다음 실행 지시와 완료 결과 회수',size=17)
        document(g,'central_data',475,414,325,46,
                 ['Input1 / Job1: 표 설명 준비',
                  'next:PREPARE_RESPONSE / waiting:Meaning1',
                  '이전 Input0의 Candidate0: 무효'],10.5)
        receiver='dispatch'
    else:
        g.box('input',55,365,285,40,'Input Resolution Stage',True,True,size=18)
        g.box('notice',475,365,325,40,'Task Notice Stage',True,True,size=19)
        state(g,'iw',55,420,285,34,'Input Window',16)
        state(g,'nw',475,420,325,34,'Notice Window',16)
        g.edge('im','input','B','T',via=[(170,314),(197.5,314)])
        g.edge('task','notice','B','T',via=[(675,314),(637.5,314)])
        g.edge('input','iw','B','T');g.edge('notice','nw','B','T')
        g.text(55,409,300,'Input1 / Job1 / 현재 입력',size=10)
        g.text(475,409,325,'Task1 / Question1 / 정보 변경 확인',size=10)
        receiver='input'
    g.box('ri',35,500,235,33,'Request Interpreter',size=17)
    g.group(310,494,525,157,'Response Manager')
    g.box('compose',330,535,230,33,'Response Composer',module=True,size=17)
    g.edge(receiver,'ri','L','T',via=[(25,385),(25,488),(152.5,488)])
    g.edge('ri',receiver,'L','L',td=10,via=[(15,516.5),(15,395)],ret=True,color=DIFFERENCE_STROKE)
    g.text(35,540,235,'① 이해 완료: Meaning1',size=13)
    if side==0:
        g.box('publish',595,535,220,33,'Publication Control',module=True,size=16)
        g.edge('dispatch','compose','B','T',sd=20,via=[(217.5,479),(445,479)],color=DIFFERENCE_STROKE)
        g.edge('compose','dispatch','T','R',sd=35,td=9,via=[(480,484),(350,484),(350,394)],ret=True,color=DIFFERENCE_STROKE)
        g.edge('dispatch','publish','R','T',sd=-7,via=[(365,378),(365,475),(705,475)],color=DIFFERENCE_STROKE)
        g.text(390,461,410,'② 중앙의 준비 지시 / ③ 준비 결과 회수',size=12)
        document(g,'candidate',35,563,235,72,
                 ['준비 완료: Candidate1',
                  'input_ref:Input1',
                  'content_ref:표 설명',
                  '이 후보도 Dispatcher로 반환'],11)
        g.text(345,590,450,'④ 현재성 검사 후 중앙의 게시 지시',size=16)
        g.text(345,618,450,'완료마다 다음 실행을 중앙에서 결정',size=16)
    else:
        g.box('join',595,535,220,33,'Publication Join',True,True,size=18)
        state(g,'pw',595,594,220,38,'Publication Window',13)
        g.box('publish',330,605,230,28,'Publication Control',module=True,size=16)
        g.edge('input','compose','R','L',sd=9,via=[(350,394),(350,480),(302,480),(302,551.5)],color=DIFFERENCE_STROKE)
        g.edge('notice','compose','R','T',td=35,via=[(845,385),(845,482),(480,482)],color=DIFFERENCE_STROKE)
        g.edge('compose','join','R','L',color=DIFFERENCE_STROKE)
        g.edge('input','join','R','T',sd=-7,via=[(365,378),(365,476),(705,476)],color=DIFFERENCE_STROKE)
        g.text(391,461,429,'Meaning1 / NoticeReady / 입력 해소 또는 질문 허용',size=11)
        g.edge('join','pw','B','T',sd=45,td=45,color=DIFFERENCE_STROKE)
        g.edge('join','publish','B','T',sd=-45,via=[(660,582),(445,582)],color=DIFFERENCE_STROKE)
        g.text(350,573,210,'Candidate1',size=12)
        document(g,'candidate',35,563,235,72,
                 ['단계 출력: Candidate1',
                  'input_ref:Input1 / required_refs',
                  '입력과 정보 조건 충족: ELIGIBLE',
                  '부족: 대기 / 옛 후보: DISCARDED'],10.5)
        g.text(595,572,220,'후보 + 해당 입력 / 정보 / 전달 상태',size=10)
        g.text(350,637,460,'조건 충족 시 게시 단계 활성화 / 중앙 회수 없음',size=12)
    trace(g,side)
    g.finish()
    return g


def trace(g,side):
    g.raw([(25,665),(835,665)],arrow=False)
    g.text(35,671,790,'실행 순서도 / 같은 완료 결과의 다음 처리',size=14)
    labels=(['Meaning1 완료','Dispatcher: 준비 지시','Composer: 후보 반환'] if side==0 else
            ['Meaning1 완료','Composer: 후보 생산','Join: 조건 검사 후 게시'])
    for i,(xx,ww,label) in enumerate(zip([35,315,595],[235,235,230],labels)):
        step(g,'normal'+str(i),xx,695,ww,27,label)
        if i:g.edge('normal'+str(i-1),'normal'+str(i),'R','L',color=DIFFERENCE_STROKE)
    if side==0:
        g.edge('normal2','normal1','B','B',via=[(710,736),(432.5,736)],ret=True,color=DIFFERENCE_STROKE)
        g.text(442,723,385,'중앙 회수 뒤 검사 / 게시 지시',size=11)
    else:
        g.text(320,728,500,'알림 후보도 같은 Join으로 연결 / 입력별 상태 유지',size=12)
    g.text(35,747,790,'늦은 Candidate0 / 이미 새 Input1 처리 중',size=13)
    for i,(xx,label) in enumerate(zip([35,315,595],
                      ['옛 응답 준비 완료','Dispatcher: 현재성 검사','중앙에서 무효 후보 폐기'] if side==0 else
                      ['옛 응답 준비 완료','Join: 현재 입력 조건 검사','Publication Window에서 폐기'])):
        step(g,'late'+str(i),xx,765,235 if i<2 else 230,26,label)
        if i:g.edge('late'+str(i-1),'late'+str(i),'R','L',color=DIFFERENCE_STROKE)
