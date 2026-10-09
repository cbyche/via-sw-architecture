#!/usr/bin/env python3
"""Independent 46 diagrams; no shared presentation or 41/42/44/45 mutation."""
from pathlib import Path
import argparse
from evidence_context_diagram_scene import Scene
ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/diagrams'


def base(title, slug, subtitle):
    s = Scene(title, slug)
    s.text(40, 20, 2480, [title], 36, True)
    s.text(40, 75, 2480, [subtitle], 24)
    return s


def structure():
    s = base('46 | 조회가 원본 결합을 실행 vs 생산된 관계를 조회', 'choice46-structure',
             '같은 W46/r1, a와 b, S1/P1→S2/P2, E20/P20와 D21/D22 | 화살표는 발생 입력, 읽기, 구성, 쓰기와 반환을 구별')
    for option, ox in [('A',20),('B',1320)]:
        p=option.lower()
        def n(k,x,y,w,h,name,kind='module',owner=None,diff=False): s.node(p+k,ox+x,y,w,h,name,kind,p+owner if owner else None,diff)
        def e(a,b,pts,both=False,dashed=False): s.edge(p+a,p+b,[(ox+x,y) for x,y in pts],both,dashed)
        def t(x,y,w,lines,size=21,bold=False): s.text(ox+x,y,w,lines,size,bold)
        t(0,120,1210,[option+('  현재 요청에서 원본을 읽고 요청별 결과를 구성' if option=='A' else '  owner별 관계 생산 경로와 관계 소비 경로')],28,True)
        if option=='A':
            n('ri',20,235,360,100,'Request Interpreter','component')
            n('rm',430,235,250,100,'Response Manager','component')
            n('tm',705,235,250,100,'Task Manager','component')
            n('rc',980,235,230,100,'Request\nController','component')
            n('rh',445,287,220,35,'E20 / P20','data','rm')
            n('th',720,287,220,35,'D21@v2 / D22@v1','data','tm')
            t(20,175,1190,['입력 refs가 전달됨. 현재 의미/Task는 미확정. 유효 cache와 prefetch도 허용.'],22)
            n('cm',20,380,1190,435,'Context Manager','component')
            n('cq',550,435,630,95,'EvidenceQuery Q46 / u46@r1\nW46 + acoustic a,b + C20/T21/T22 후보\n목적과 권한/예산 / 현재 Task 정답 없음','data','cm')
            n('compose',70,460,365,65,'Context Composer',owner='cm',diff=True)
            n('tq',70,560,620,90,'TemporalQuery Q1 / W46 / r1 / a,b\n원본 refs와 필요한 구간 → 실제 join 요청\n공통 관계가 이미 만들어졌다는 가정 없음','data','cm')
            n('cache',70,690,365,65,'Source View Cache','state','cm')
            n('set',775,625,385,80,'Request Evidence Set','state','cm',True)
            n('bundle',775,730,385,65,'Bundle: Q1/r1 + E20/P20 refs\nD21@v2, D22@v1 + vector/gap','data','cm')
            n('im',20,875,1190,375,'Interaction Manager','component')
            n('tl',55,930,1130,305,'Timeline & Buffer',owner='im')
            n('resolver',90,980,405,65,'Temporal Evidence Resolver',owner='tl',diff=True)
            n('raw',700,970,450,65,'Input Observation Window','state','tl')
            n('rawdata',700,1040,450,45,'S1/P1 / S2/P2 / raw refs','data','tl')
            n('tr',700,1100,450,65,'Transcript Revision State','state','tl')
            n('qs',90,1085,405,65,'Temporal Query State','state','tl',True)
            n('tcache',550,1175,600,55,'Temporal Query Cache','state','tl')
            n('result',90,1160,405,65,'Q1 Result(r1): a ↔ S1/P1\nb ↔ S2/P2 / 후보 / vector / gap','data','tl')
            # Request-rooted read: the payload is visibly carried through the query path.
            e('ri','cq',[(205,335),(205,355),(1200,355),(1200,475),(1180,475)])
            e('cq','compose',[(550,490),(435,490)])
            t(70,430,365,['1 Q46 요청 범위 결정'],19)
            e('compose','tq',[(205,525),(205,560)])
            e('tq','resolver',[(680,650),(680,960),(350,960),(350,980)])
            t(710,832,475,['2 Q1이 원본 결합을 요청'],20)
            e('resolver','raw',[(495,1003),(700,1003)],True)
            t(510,965,185,['3 원본 구간 read'],18)
            e('resolver','tr',[(495,1030),(540,1030),(540,1130),(700,1130)],True)
            t(550,1090,140,['r1/span read'],18)
            e('resolver','qs',[(400,1045),(400,1085)])
            t(95,1054,265,['4 이번 query의 후보 구성'],18)
            e('qs','result',[(400,1150),(400,1160)])
            e('tcache','resolver',[(550,1200),(520,1200),(520,1020),(495,1020)],True,True)
            e('result','compose',[(90,1190),(40,1190),(40,495),(70,495)])
            e('compose','rm',[(360,460),(360,420),(555,420),(555,335)],True)
            e('compose','tm',[(390,460),(390,430),(830,430),(830,335)],True)
            e('compose','rc',[(425,460),(425,400),(1095,400),(1095,335)],True)
            t(450,710,260,['5 과거 owner read', '필요 시 의미 가공'],19)
            e('cache','compose',[(70,720),(55,720),(55,515),(70,515)],True,True)
            t(75,768,620,['유효 cache로 같은 과거 owner read를 생략할 수 있음'],18)
            e('compose','set',[(435,510),(485,510),(485,545),(730,545),(730,680),(775,680)])
            t(775,587,385,['6 Q1 결과와 과거 refs를 조합'],18)
            e('set','bundle',[(985,705),(985,730)])
            e('bundle','ri',[(1160,760),(1185,760),(1185,340),(370,340),(370,335)])
            e('ri','rc',[(380,250),(400,250),(400,225),(1095,225),(1095,235)])
            t(20,1270,1190,['준비 조건: 필요한 원본 read/join 또는 유효 cache + 요청별 Set 완료',
                              '정정 r2: 영향 query/set/cache 차단 → 필요한 원본부터 다시 구성'],22)
        else:
            # Production is rooted in source revisions, not in the consuming EvidenceQuery.
            n('rc',20,175,360,55,'Request Controller','component')
            n('rm',430,175,360,55,'Response Manager','component')
            n('tm',830,175,380,55,'Task Manager','component')
            n('im',20,260,565,915,'Interaction Manager','component')
            n('tl',45,315,515,835,'Timeline & Buffer',owner='im')
            n('raw',70,375,465,65,'Input Observation Window','state','tl')
            n('rawexample',70,445,465,30,'raw refs: S1/P1, S2/P2','data','tl')
            n('tr',70,480,465,65,'Transcript Revision State','state','tl')
            n('tp',70,600,465,65,'Temporal Evidence Publisher',owner='tl',diff=True)
            n('te',70,740,465,65,'Temporal Evidence Repository','state','tl',True)
            n('tedata',70,825,465,90,'TEv1(r1): a ↔ S1/P1, b ↔ S2/P2\nrelation + coverage + source vector\n현재 지칭/Task 정답은 아님','data','tl')
            n('trdr',70,985,465,70,'Temporal Evidence Reader',owner='tl',diff=True)
            n('cm',625,260,585,915,'Context Manager','component')
            n('hc',665,375,505,65,'Source View Cache','state','cm')
            n('hd',665,480,505,85,'E20/P20 실제 전달 범위\nT21/D21@v2, T22/D22@v1\n확정 source refs + revision','data','cm')
            n('mp',665,600,505,65,'Memory Publisher',owner='cm',diff=True)
            n('hm',665,740,505,65,'Episodic Memory Repository','state','cm',True)
            n('hmdata',665,825,505,90,'HMv7: P20 → E20 actual-range\nT21 → D21@v2 / T22 → D22@v1\n관계 + Historical Coverage + vector','data','cm')
            n('er',655,985,525,70,'Evidence Reader',owner='cm',diff=True)
            n('q',655,1075,240,85,'Q46 / r1\nW46 + a,b\nC20/T21/T22','data','cm')
            n('bundle',920,1075,260,85,'Bundle: TEv1\nHMv7 + refs\nvector / gap','data','cm')
            n('ri',20,1225,565,80,'Request Interpreter','component')
            e('raw','tp',[(535,410),(550,410),(550,585),(200,585),(200,600)])
            e('tr','tp',[(405,545),(405,600)])
            t(75,558,310,['1 원본/전사 revision 변화'],20)
            t(75,682,470,['2 지원 시간 관계 구성 → 조건부 게시'],20)
            e('tp','te',[(500,665),(500,740)])
            e('te','tedata',[(320,805),(320,825)])
            e('rm','hc',[(610,230),(610,405),(665,405)])
            e('tm','hc',[(1100,230),(1200,230),(1200,405),(1170,405)])
            e('rc','hc',[(210,230),(210,245),(1220,245),(1220,425),(1170,425)])
            e('hc','hd',[(930,440),(930,480)])
            e('hd','mp',[(930,565),(1140,565),(1140,600)])
            t(665,575,510,['3 확정 기록 변화 → read/필요한 의미 추출'],20)
            e('mp','hm',[(1140,665),(1140,740)])
            t(665,685,510,['4 과거 관계 구성 → 조건부 게시'],20)
            e('hm','hmdata',[(930,805),(930,825)])
            e('tedata','trdr',[(500,915),(500,985)])
            e('hmdata','er',[(1140,915),(1140,985)])
            t(80,930,455,['5 TEv1 유효성/coverage 확인 후 read'],20)
            t(665,930,510,['5 HMv7 유효성/coverage 확인 후 read'],20)
            e('er','trdr',[(655,1018),(535,1018)],True)
            e('trdr','tp',[(535,1035),(550,1035),(550,635),(535,635)],False,True)
            e('er','mp',[(1180,1040),(1190,1040),(1190,635),(1170,635)],False,True)
            t(80,957,455,['미게시면 2 생산/게시 후 재조회'],18)
            t(665,957,510,['미게시면 4 생산/게시 후 재조회'],18)
            e('ri','q',[(585,1265),(610,1265),(610,1115),(655,1115)])
            e('q','er',[(775,1075),(775,1055)])
            e('er','bundle',[(1065,1055),(1065,1075)])
            e('bundle','ri',[(1180,1115),(1200,1115),(1200,1300),(585,1300)])
            e('ri','rc',[(20,1265),(5,1265),(5,235),(200,235),(200,230)])
            t(655,1190,540,['6 게시된 두 자료를 선택해 Bundle 반환', 'Reader는 raw 시간/과거 관계를 재구성하지 않음'],20)
            t(20,1330,1190,['미게시면 생산 요청 → 게시 후 재조회. NOT_COVERED와 UNSUPPORTED 구별.'],20)
        t(20,1362,1190,['7 현재 의미/Task 제안과 Request Controller의 채택은 공통. 시간 후보 ≠ 지칭 정답.'],18)
    s.text(40,1390,2470,['범례: 직각 Component / 둥근 Module / 원통 owner 상태 / 문서형 실제 교환 자료 / 살구색 차이 / 점선 유효 cache 또는 미게시의 조건 경로', '공통: capture/시각 정규화. 시간 관계 구성은 코드 / 과거 의미 가공은 조건부 C-CONTEXT / 현재 의미는 C-INTERPRET → Model Access의 cloud 모델.'],17)
    for label in s.texts:
        label['erase']=False
    return s

def background():
    s=base('46의 문제 | 말한 당시 화면과 행동, 과거 설명과 업무 결과를 함께 연결', 'dp46-background',
           '사용자 발화 뒤에 전사가 도착해도, 마지막 화면 하나나 전사 도착 시각으로 앞선 지칭을 대체할 수 없다')
    s.node('h',40,140,2480,250,'관련 과거 원본','boundary')
    for ident,x,name in [('ex',80,'C20 / E20 / P20\n평가 기준 설명 원문\n실제 전달한 범위'),('p',890,'T21 / D21@v2\n제품 비교의 확인된 결과 참조\nAgent 내부 자료 전체 아님'),('q',1700,'T22 / D22@v1\n견적의 확인된 결과 참조\n현재 표와 동일하다고 미리 확정하지 않음')]:
        s.node(ident,x,220,740,130,name,'data','h')
    s.node('w',40,445,2480,480,'W46: 같은 음성과 관측 원본 window','boundary')
    for ident,x,y,w,h,name in [
        ('a',80,510,700,95,'a: “이 표와”\n가능한 acoustic span / sample 범위'),
        ('b',1020,510,700,95,'b: “저 표를 아까 네가 설명한 기준으로…”\n가능한 acoustic span / sample 범위'),
        ('s1',80,690,700,120,'S1 / P1\n첫 viewport의 표, pointer/선택/drag 경로\ncapture interval / 좌표계 / 오차'),
        ('change',810,710,170,75,'스크롤 /\n창 전환'),
        ('s2',1020,690,700,120,'S2 / P2\n바뀐 viewport의 표와 당시 pointer/선택\ncapture interval / 좌표계 / 오차'),
        ('tr',1840,510,620,190,'u46@r1 → r2\n전사는 화면 변경 뒤 도착/정정 가능\nprovider item ↔ local sample mapping\nspan timing 미제공이면 unknown')]:
        s.node(ident,x,y,w,h,name,'data','w')
    s.edge('a','s1',[(430,605),(430,690)])
    s.edge('b','s2',[(1370,605),(1370,690)])
    s.edge('s1','change',[(780,750),(810,750)])
    s.edge('change','s2',[(980,750),(1020,750)])
    s.edge('b','tr',[(1720,560),(1840,560)])
    s.text(90,830,2360,['음성/화면/행동 순서와 오차를 보존 → 시간 관계 후보 구성 → 현재 Request Interpreter가 지칭과 Task 관계 판단',
                       'VAD는 발화 경계만 제공. 단어 시각 미지원/gap/원본 만료는 양안 같은 한계. 시각 overlap이 의도 정답은 아님.'],24)
    s.node('input',80,980,1430,170,'현재 u46\n“이 표와 저 표를 아까 기준으로 비교해서 제안서 초안을 만들어줘. 메일은 보내지 마.”\n지칭, 과거 설명 연결, Task 관계, 위임과 금지 조건이 아직 잠정 상태','data')
    s.node('out',1720,980,740,170,'채택 뒤의 R46 / 필요 시 T46\n관련 근거와 조건을 연결해 외부 Agent에 위임\n비교 분석과 제안서 작성은 Agent 책임','data')
    s.edge('input','out',[(1510,1065),(1720,1065)])
    s.text(80,1210,2370,['후속 u47: “두 번째 표 대신 새 견적으로 다시 만들어줘. 메일은 계속 보내지 마.”',
                       'u46 전사 정정은 같은 입력 revision. u47은 별도 입력이며 현재 의미가 채택돼야 수정 범위가 정해진다.',
                       '이 그림은 공통 문제와 원본을 설명한다. A/B의 관계 구성과 게시/조회는 MAIN에서 비교한다.'],27)
    return s


def timeline():
    s=base('46 | 같은 늦은 전사 정정에서 다시 준비되는 근거', 'choice46-timeline',
           'named participant 사건도 | 행 간격은 설명 순서이며 실행 시간이나 순차 스케줄링 제약을 뜻하지 않는다')
    for opt, ox in [('A',30),('B',1320)]:
        pre=opt.lower()
        s.text(ox,125,1200,[opt+('  요청별 구성과 재조회' if opt=='A' else '  게시 유효성 차단과 재게시/재조회')],29,True)
        positions=[ox+155,ox+455,ox+755,ox+1055]
        names=['Interaction\nManager','Context\nManager','Request\nInterpreter','Request\nController']
        for i,(cx,name) in enumerate(zip(positions,names)):
            s.node(pre+str(i),cx-135,195,270,1100,name,'lifeline')
        def event(row,a,b,label):
            yy=310+row*65
            x1,x2=positions[a],positions[b]
            s.edge(pre+str(a),pre+str(b),[(x1,yy),(x2,yy)])
            start=min(x1,x2)+8
            s.text(start,yy-29,abs(x2-x1)-16,[label],17)
        event(0,3,2,'D1 input u46@r1 / W46')
        event(1,2,1,'D2 EvidenceQuery(r1)')
        event(2,1,0,'D3 '+('TemporalQuery: 원본 결합' if opt=='A' else 'TemporalRead: 유효 TEv1'))
        event(3,0,1,'D4 '+('Q1 Result / source vector' if opt=='A' else 'TEv1 / coverage / refs'))
        event(4,1,2,'D5 Bundle b1 / Receipt')
        event(5,0,3,'D6 late r2 → 같은 입력 정정, 기존 근거/의미 사용 차단')
        event(6,0,1,'D7 '+('query/set/cache dirty' if opt=='A' else 'TEv1 DIRTY / 소비 차단'))
        event(7,3,2,'D8 j1 폐기 / j2 시작')
        event(8,2,3,'D9 늦은 r1 의미: 거절')
        event(9,2,1,'D10 EvidenceQuery(r2)')
        event(10,1,0,'D11 '+('Q2: 필요한 원본 재결합' if opt=='A' else '유효 TEv2 없으면 대기'))
        event(11,0,1,'D12 '+('Q2 Result / r2 vector' if opt=='A' else '조건부 게시 TEv2 → read'))
        event(12,1,2,'D13 Bundle b2 / 새 vector')
        event(13,2,3,'D14 r2 의미 제안 → 채택')
        s.text(ox+20,1280,1160,[('A: 살아 있는 원본에서 필요한 관계를 다시 구성. 유효 부분/cache는 재사용 가능.' if opt=='A' else
                              'B: Publisher가 relation + coverage를 함께 확정. Reader는 재결합하지 않음.'),
                             '양안: 원본/gap/timing 한계, 현재성 검사는 동일. 원본 소실 시 자동 복원 보장 없음.'],20)
    s.text(40,1390,2450,['B의 최초 TEv1 생산은 read보다 먼저 또는 요청 뒤에 일어날 수 있다. Input 확정/local stop는 생산 완료를 기다리지 않는다.'],20)
    return s


def lifecycle():
    s=base('46 | 원본 revision, 근거의 준비 상태와 최종 사용 경계', 'choice46-lifecycle',
           '같은 r2, 권한 철회, 원본 만료 | A의 요청 상태와 B의 게시 상태는 원본 권위나 현재 의미를 대신하지 않는다')
    for opt,ox in [('A',30),('B',1320)]:
        pre=opt.lower()
        def n(k,x,y,w,h,name,kind='state',owner=None,diff=False): s.node(pre+k,ox+x,y,w,h,name,kind,pre+owner if owner else None,diff)
        def e(a,b,pts): s.edge(pre+a,pre+b,[(ox+x,y) for x,y in pts])
        def t(x,y,w,lines,size=23,bold=False): s.text(ox+x,y,w,lines,size,bold)
        t(0,135,1200,[opt+('  조회에 종속된 임시 상태'.replace('조회에 종속된 임시 상태','조회에 종속된 임시 상태') if opt=='A' else '  producer 소유 게시 상태와 coverage')],29,True)
        n('im',0,205,1200,365,'Interaction Manager','component')
        n('raw',30,290,310,80,'Input Observation Window',owner='im')
        n('tr',30,430,310,80,'Transcript Revision State',owner='im')
        n('tl',380,265,790,280,'Timeline & Buffer','module','im')
        n('fn',410,325,310,70,'Temporal Evidence\n'+('Resolver' if opt=='A' else 'Publisher'),'module','tl',True)
        n('state',795,315,345,95,'Temporal Query State' if opt=='A' else 'Temporal Evidence\nRepository','state','tl',True)
        n('rev',795,445,345,70,'Q1 dirty → Q2 Result' if opt=='A' else 'TEv1 DIRTY → TEv2 VALID','data','tl')
        e('raw','fn',[(340,330),(410,330)])
        e('tr','fn',[(340,470),(370,470),(370,375),(410,375)])
        e('fn','state',[(720,360),(795,360)])
        e('state','rev',[(965,410),(965,445)])
        t(410,430,365,['r2 / gap / permission 변경', 'expected source vector 검사'],21)
        n('cm',0,620,1200,325,'Context Manager','component')
        n('consume',30,700,380,75,'Context Composer' if opt=='A' else 'Evidence Reader','module','cm',True)
        n('hist',30,830,380,80,'Source View Cache' if opt=='A' else 'Episodic Memory Repository','state','cm',opt=='B')
        n('bundle',555,700,615,95,'Request Evidence Set' if opt=='A' else 'Bundle / Receipt','state' if opt=='A' else 'data','cm',opt=='A')
        if opt=='A':
            e('rev','consume',[(965,515),(965,595),(215,595),(215,700)])
        else:
            n('reader',410,475,310,60,'Temporal Evidence Reader','module','tl',True)
            e('rev','reader',[(795,490),(720,490)])
            e('reader','consume',[(565,535),(565,595),(215,595),(215,700)])
            s.texts[-1]['y']=400
        e('hist','consume',[(215,830),(215,775)])
        e('consume','bundle',[(410,737),(555,737)])
        t(460,825,690,['input revision / temporal view 또는 query ID', 'historical refs / gap / permission epoch / lease'],22)
        n('rc',0,1010,380,170,'Request Controller','component')
        n('fence',25,1080,330,65,'Adoption Control','module','rc')
        n('meaning',555,1010,615,100,'Request Interpreter의 잠정 Meaning Proposal\njob j1 / r1 / b1은 현재 r2에 채택 불가\nj2 / r2 / b2도 owner 유효성 확인 후 사용','data')
        e('bundle','meaning',[(860,795),(860,1010)])
        e('meaning','fence',[(555,1090),(355,1090)])
        t(430,1150,755,['채택 뒤 전송/게시 직전에도 owner의 현재성 재검사', '늦은 invalidation event만 믿지 않음. read 성공 ≠ 실행 허가.'],22)
        t(0,1250,1200,[('A: 수정된 query/set/cache 무효화 → 원본에서 요청별 재구성' if opt=='A' else
                        'B: 시간/과거 view 사용 차단 → owner별 재생산/게시 → 소비자 재조회'),
                       '공통: DENIED / EXPIRED / GAP / UNKNOWN_TIMING / UNAVAILABLE을 명시',
                       '원본 만료/철회: active query나 view ref가 원본 수명을 무한 연장하지 않음'],23)
    s.text(40,1380,2480,['살구색 = 차이 Module/상태. 문서형 예시는 상태 이름이 아님. 과거 원본과 시간 원본의 owner별 revision을 남기며 전역 snapshot은 가정하지 않음.'],20)
    return s


HTML='''<!doctype html><html lang="ko"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>46 구조 검토</title>
<style>body{margin:0;font:16px system-ui;background:#f5f5f5;color:#151515}header{position:sticky;top:0;background:#fff;padding:14px 22px;border-bottom:1px solid #bbb;z-index:1}button,a{margin-right:12px}button{padding:8px 12px;border:1px solid #aaa;border-radius:5px;background:white;cursor:pointer}button[aria-pressed=true]{background:#fce4d6;border-color:#9b502b}main{padding:20px;overflow:auto}img{display:block;width:100%;min-width:1000px;background:white}p{margin:8px 0}</style>
<header><strong>46: 요청별 구성과 공통 생산/조회</strong><p>A/B 미선정. 문서/그림 검토용이며 구현/성능 측정 결과가 아닙니다.</p>
<nav><button data-stem="dp46-background">문제와 같은 원본</button><button data-stem="choice46-structure" aria-pressed="true">MAIN 구조</button><button data-stem="choice46-timeline">정정 사건도</button><button data-stem="choice46-lifecycle">상태와 사용 경계</button><button id="zoom">원본 크기 / 폭 맞춤</button></nav>
<p><a href="../04-46-input-and-context-evidence.md">본문</a><a id="svg" href="choice46-structure.svg">SVG</a><a id="drawio" href="choice46-structure.drawio">편집 가능한 draw.io</a></p></header>
<main><img id="figure" src="choice46-structure.svg" alt="46 A/B 구조 비교"></main><script>
let full=false;const figure=document.getElementById('figure');document.querySelectorAll('[data-stem]').forEach(b=>b.onclick=()=>{document.querySelectorAll('[data-stem]').forEach(x=>x.setAttribute('aria-pressed',String(x===b)));figure.src=b.dataset.stem+'.svg';document.getElementById('svg').href=figure.src;document.getElementById('drawio').href=b.dataset.stem+'.drawio';});document.getElementById('zoom').onclick=()=>{full=!full;figure.style.width=full?'2560px':'100%';};
</script></html>\n'''


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');args=parser.parse_args()
    outputs={}
    for scene in [background(),structure(),timeline(),lifecycle()]:
        for suffix,data in scene.render().items(): outputs[OUT / f'{scene.slug}.{suffix}']=data
    outputs[OUT / 'choice46-review.html']=HTML
    for path,data in outputs.items():
        if args.check:
            assert path.exists() and path.read_text()==data,f'out of sync: {path}'
        else:
            path.write_text(data)
    print(f'PASS: 46 {len(outputs)} generated text artifacts; node ownership, orthogonal ports and unrelated-node crossings checked')

if __name__=='__main__':main()
