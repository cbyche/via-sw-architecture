"""Focused editable DP44 graph: central continuation versus connected consumers."""
from dp_comparison_structures import ResolutionGraph
from dp41_comparison_scene import document, legend as base_legend
from dp_comparison_structures import APRICOT, DIFFERENCE_STROKE, COMMON

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
    g.s.items[before].update(fill=APRICOT,stroke=DIFFERENCE_STROKE)



def draw(s,x,side):
    g=ResolutionGraph(s,x)
    if side==0:legend(s)
    g.box('im',35,253,270,34,'Interaction Manager',size=18)
    g.box('task',525,253,300,34,'Task Manager',size=18)
    g.text(35,291,365,'Input1: “잠깐, 표부터 설명해줘”',size=15)
    g.text(525,291,315,'Task1: 메일 / Question1: 수신자 질문',size=14)
    g.group(25,326,810,172,'Request Controller')
    if side==0:
        g.box('dispatch',55,381,285,38,'Dialogue Dispatcher',True,True,size=19)
        state(g,'wait',475,381,325,43,'Dialogue Progress State',size=17)
        g.edge('im','dispatch','B','T',via=[(170,317),(197.5,317)])
        g.edge('task','dispatch','B','T',td=85,via=[(675,313),(282.5,313)])
        g.edge('dispatch','wait','R','L',td=-2.5)
        g.edge('wait','dispatch','L','R',sd=9.5,td=12,via=[(455,412),(455,432),(360,432),(360,412)],ret=True)
        g.text(55,451,742,'모든 완료 회수 → 중앙 대기 상태 갱신 → 다음 job / 게시 지시',size=16)
        receiver='dispatch'
    else:
        g.box('input',55,381,285,38,'Input Resolution Stage',True,True,size=18)
        g.box('notice',475,381,325,38,'Task Notice Stage',True,True,size=19)
        state(g,'iw',55,441,285,36,'Input Window',size=16)
        state(g,'nw',475,441,325,36,'Notice Window',size=16)
        g.edge('im','input','B','T',via=[(170,317),(197.5,317)])
        g.edge('task','notice','B','T',via=[(675,317),(637.5,317)])
        g.edge('input','iw','B','T')
        g.edge('notice','nw','B','T')
        g.text(55,420,300,'input/job・정정 세대'.replace('・',' / '),size=12)
        g.text(475,420,320,'질문 참조・준비/대기'.replace('・',' / '),size=12)
        receiver='input'
    g.box('ri',35,535,235,34,'Request Interpreter',size=17)
    g.group(295,518,540,118,'Response Manager')
    g.box('compose',315,558,230,33,'Response Composer',module=True,size=17)
    if side==0:
        g.box('publish',590,558,225,33,'Publication Control',module=True,size=17)
        g.edge('dispatch','compose','B','T',via=[(197.5,509),(430,509)])
        g.edge('compose','dispatch','T','R',sd=35,td=8,via=[(465,506),(350,506),(350,408)],ret=True)
        g.edge('dispatch','publish','R','T',td=30,via=[(355,400),(355,500),(732.5,500)])
        g.text(320,601,494,'準備結果を回収'.replace('準備結果を回収','후보도 중앙에 반환 / 유효 후보만 게시 지시'),size=14)
    else:
        g.box('join',590,558,225,33,'Publication Join',True,True,size=18)
        state(g,'pw',590,600,225,36,'Publication Window',size=13)
        g.edge('notice','compose','B','T',td=35,via=[(637.5,510),(465,510)])
        g.edge('input','compose','B','T',via=[(197.5,506),(430,506)])
        g.edge('compose','join','R','L')
        g.edge('input','join','R','T',td=35,via=[(355,400),(355,501),(737.5,501)])
        g.text(590,490,235,'InputSettled / admission',size=12)
        g.edge('join','pw','B','T',sd=50,td=50)
        g.box('publish',315,604,230,25,'Publication Control',module=True,size=15)
        g.edge('join','publish','B','T',via=[(702.5,597),(430,597)])
    g.edge(receiver,'ri','L','T',via=[(25,400),(25,523),(152.5,523)])
    g.edge('ri',receiver,'L','L',td=12,via=[(15,552),(15,412)],ret=True)
    g.text(35,578,235,'job 완료 → 시작한 owner',size=12)
    document(g,'data',35,648,790,79,
             (['중앙 실행 자료: job=Job1, next=PREPARE_RESPONSE, waiting=[AdoptedMeaning1]',
               'Job1 완료: {input_ref:Input1, generation:2, status:OK, meaning_ref:AdoptedMeaning1}',
               '중앙이 generation / 원본 확인 후 다음 job을 호출하거나 늦은 결과 폐기',
               '질문 후보와 표 설명 후보도 중앙에서 대기를 연결한다.'] if side==0 else
              ['연결된 입력: AdoptedMeaning{input_ref:Input1, generation:2, meaning_ref:AdoptedMeaning1}',
               'NoticeReady{task_ref:Task1, question_ref:Question1, source_revision:3}',
               'CandidateReady + 해당 admission / source / 과거 전달 snapshot → Join',
               '조건 부족: 대기 / 정정으로 무효: 폐기 / 조건 충족: 게시 단계 활성화']),13)
    g.raw([(25,738),(835,738)],arrow=False)
    g.text(35,744,790,'같은 겹침: 새 입력 + 별도 업무 질문 + 이전 응답 준비 완료',size=15)
    g.text(35,766,790,('완료마다 중앙으로 복귀해 다음 작업 지시' if side==0 else
                           '완료가 연결된 소비자의 입력이 됨 / 중앙 회수 없이 이어감'),size=15)
    g.finish()
