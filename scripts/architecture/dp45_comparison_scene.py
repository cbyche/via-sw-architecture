"""DP45: request-time assembly versus independently published relation reads."""
from dp_comparison_structures import APRICOT, DIFFERENCE_STROKE
from dp41_comparison_scene import FeedbackGraph, document, step
from dp44_comparison_scene import v3_legend as legend


def state(g,key,x,y,w,h,name,different=True,size=16):
    before=len(g.s.items);g.store(key,x,y,w,h,name,size=size)
    g.s.items[before].update(store_radius=5)
    if different:g.s.items[before].update(fill=APRICOT,stroke=DIFFERENCE_STROKE)
    g.s.items[-1]['y']=y+(h-size*1.2)/2


def draw(s,x,side):
    g=FeedbackGraph(s,x)
    if side==0:legend(s)
    g.box('ri',35,253,250,32,'Request Interpreter',size=17)
    g.box('rm',340,253,235,32,'Response Manager',size=17)
    g.box('tm',605,253,230,32,'Task Manager',size=18)
    g.text(35,292,280,'Query1: 평가 기준과 과거 결과',size=13)
    g.text(345,291,215,'설명 원문 / 실제 전달한 범위',size=12)
    g.text(610,291,220,'Task1 제품 조사 / Task2 견적',size=12)
    g.text(345,308,215,'“가격이 같은 경우…”',size=12)
    g.text(610,308,220,'제품 결과 개정 2 / 견적 개정 1',size=11)
    g.group(25,344,810,284,'Context Manager')
    if side==0:
        provider='composer'
        g.box(provider,55,394,270,38,'Context Composer',True,True,size=19)
        state(g,'cache',55,545,270,40,'Source View Cache',False,17)
        state(g,'set',485,545,310,40,'Request Evidence Set',size=17)
        raw_reads(g,provider,False)
        g.text(435,354,370,'② Query에 맞춘 원본 읽기 / 반환',size=14)
        g.edge(provider,'cache','B','T',sd=-25,td=-25)
        g.edge('cache',provider,'T','B',sd=25,td=25,ret=True)
        g.text(65,454,265,'③ 원본 확인 / 유효 cache 재사용',size=13)
        g.text(65,477,265,'필요한 원본 추가 조회 / 부분 갱신',size=13)
        g.edge(provider,'set','R','T',via=[(365,413),(365,537),(640,537)],color=DIFFERENCE_STROKE)
        g.edge('set',provider,'T','R',sd=15,td=10,via=[(655,532),(375,532),(375,423)],ret=True,color=DIFFERENCE_STROKE)
        document(g,'bundle',405,394,390,118,[
            '이번 Query의 Bundle1 / Receipt1 (필드 축약)',
            'status:PARTIAL / acquisition:OWNER_COMPOSITION',
            'evidence: 실제 전달한 예외 문구 발췌',
            '  “가격이 같은 경우 유지보수 기간을 우선합니다.”',
            '  Task1 → 제품 비교 결과 / Task2 → 견적 결과',
            'checked_revisions / read_ranges: 원본 확인 범위'],11)
        g.text(55,598,305,'선택 cache는 재사용 / 구성 책임은 이번 조회',size=11)
        g.text(485,598,310,'이번 Query 수명의 발췌와 결과 참조',size=12)
    else:
        provider='reader'
        g.box('publisher',55,394,270,38,'Memory Publisher',True,True,size=19)
        state(g,'repo',485,394,310,42,'Episodic Memory Repository',size=15)
        g.box(provider,55,548,270,38,'Evidence Reader',True,True,size=19)
        state(g,'coverage',485,575,310,33,'Coverage State',size=16)
        raw_reads(g,'publisher',True)
        g.text(435,354,380,'① 원본 변경 / 생산 / 검증 / 같은 개정 게시',size=13)
        g.edge('publisher','repo','R','L',sd=-4,td=-6,color=DIFFERENCE_STROKE)
        g.edge('publisher','coverage','R','L',sd=10,td=-8,via=[(463,423),(463,583.5)],color=DIFFERENCE_STROKE)
        document(g,'record',485,452,310,99,[
            'Memory1 / relation_kind:EXPLAINS_ITEM',
            'from/to: 실제 전달 → 설명 원문',
            'value: “가격이 같은 경우 유지보수 기간을',
            '  우선합니다.” / MODEL_DERIVED',
            'HAS_RESULT: 제품 Task → 제품 결과 / EXPLICIT',
            'HAS_RESULT: 견적 Task → 견적 결과 / EXPLICIT',
            'source_spans / dependency_vector / 개정'],10.5)
        g.edge(provider,'repo','R','B',sd=-8,via=[(415,559),(415,443),(640,443)],color=DIFFERENCE_STROKE)
        g.edge('repo',provider,'B','R',sd=15,td=4,via=[(655,447),(425,447),(425,571)],ret=True,color=DIFFERENCE_STROKE)
        g.edge('coverage',provider,'L','R',td=13,via=[(355,591.5),(355,580)],ret=True,color=DIFFERENCE_STROKE)
        g.edge(provider,'publisher','T','B',sd=-25,td=-25,color=DIFFERENCE_STROKE)
        g.edge('publisher',provider,'B','T',sd=25,td=25,ret=True,color=DIFFERENCE_STROKE)
        g.text(65,454,275,'NOT_COVERED: 지원 범위 생산 요청',size=11)
        g.text(65,477,275,'게시 개정 반환 후 Reader가 저장소 재조회',size=11)
        g.text(55,598,390,'지원 밖 관계: UNSUPPORTED_RELATION / 반복 생산 없음',size=11)
        g.text(485,610,310,'예외 문구만 생산 / 나머지 기준은 미생산',size=11)
        # A thin common owner verification route remains in B; relation reads
        # cannot replace permission, revision and requested exact-quote checks.
        g.edge(provider,'tm','B','R',sd=100,td=8,via=[(290,620),(849,620),(849,277)])
        g.edge('tm',provider,'R','B',sd=13,td=105,via=[(854,282),(854,623),(295,623)],ret=True)
        g.edge(provider,'rm','R','B',sd=-10,td=110,via=[(838,557),(838,329),(567.5,329)])
        g.edge('rm',provider,'B','R',sd=114,td=-6,via=[(571.5,333),(842,333),(842,561)],ret=True)
        g.text(55,612,350,'현재 개정과 요청한 원문은 owner 확인',size=11)
    # Query/result circulation is separate from the producer's source changes.
    g.edge('ri',provider,'L','L',via=[(12,269),(12,413 if side==0 else 567)],color=DIFFERENCE_STROKE)
    g.edge(provider,'ri','L','L',sd=9,td=8,via=[(20,422 if side==0 else 576),(20,277)],ret=True,color=DIFFERENCE_STROKE)
    g.text(55,381 if side==0 else 529,300,'① 현재 Query1' if side==0 else '② 현재 Query1 / Reader로 별도 시작',size=10 if side==0 else 13)
    g.box('ma',55,638,270,25,'Model Access',size=16)
    g.box('cloud',525,638,300,25,'클라우드 의미 LLM',size=16)
    g.s.items[-2]['external']=True
    meaning=provider if side==0 else 'publisher'
    g.edge(meaning,'ma','L','L',via=[(32,413),(32,650.5)])
    g.edge('ma',meaning,'L','L',sd=6,td=10,via=[(40,656.5),(40,423)],ret=True)
    g.edge('ma','cloud','R','L',sd=-4,td=-4)
    g.edge('cloud','ma','L','R',sd=5,td=5,ret=True)
    g.text(350,637,165,'필요한 과거 의미만 가공',size=11)
    trace(g,side)
    g.finish()
    return g


def raw_reads(g,producer,changed):
    for owner,xx,sd in [('rm',560,80),('tm',825,105)]:
        endpoint=g.port(owner,'B')[0];offset=xx-endpoint
        if changed:
            g.edge(owner,producer,'B','T',sd=offset-8,td=sd-8,
                   via=[(xx-8,322),(190+sd-8,322)],color=DIFFERENCE_STROKE)
        g.edge(producer,owner,'T','B',sd=sd,td=offset,
               via=[(190+sd,329),(xx,329)])
        g.edge(owner,producer,'B','T',sd=offset+4,td=sd+4,
               via=[(xx+4,337),(194+sd,337)],ret=True)


def trace(g,side):
    g.raw([(25,677),(835,677)],arrow=False)
    g.text(35,681,790,'실행 순서도 / 생산 시점과 조회 경로',size=14)
    rows=([['현재 Query1 도착','Composer: 원본 조회 / 결합','Bundle1을 caller에 반환'],
           ['같은 정보 반복 요청','유효 cache / 부분 갱신','현재 목적의 묶음 구성']] if side==0 else
          [['원본 변경 또는 생산 요청','Publisher: 공통 관계 게시','Repository와 Coverage 유지'],
           ['현재 Query1 도착','Reader: 게시 관계 읽기','원본 확인 후 Bundle1 반환']])
    for row,(yy,labels) in enumerate(zip([705,762],rows)):
        for i,(xx,label) in enumerate(zip([35,315,595],labels)):
            step(g,f'trace{row}_{i}',xx,yy,235 if i<2 else 230,27,label)
            if i:g.edge(f'trace{row}_{i-1}',f'trace{row}_{i}','R','L',color=DIFFERENCE_STROKE)
    g.text(35,742,790,'A의 cache 허용 / Request별 구성 책임 유지' if side==0 else
           '미생산 지원 범위: Publisher 게시 뒤 Reader 재조회 / 현재 Request 정답은 저장하지 않음',size=12)
