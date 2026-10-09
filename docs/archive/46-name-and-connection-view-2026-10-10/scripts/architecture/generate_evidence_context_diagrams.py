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
    s = base('46 | 요청별 근거 구성 vs 공통 근거 생산/조회', 'choice46-structure',
             '같은 원본, 권한, 유한 보존, 클라우드 모델 | 시간 관계의 준비 조건부터 과거 근거 소비까지 다르게 설계')
    for option, ox in [('A', 20), ('B', 1320)]:
        p = option.lower()
        def n(k,x,y,w,h,name,kind='module',owner=None,diff=False):
            s.node(p+k, ox+x,y,w,h,name,kind,p+owner if owner else None,diff)
        def e(a,b,pts,both=False): s.edge(p+a,p+b,[(ox+x,y) for x,y in pts],both)
        def t(x,y,w,lines,size=21,bold=False): s.text(ox+x,y,w,lines,size,bold)
        t(0,115,1210,[f'{option}  '+('요청별 원본 결합과 목적별 구성' if option=='A' else 'owner별 관계 생산/게시와 목적별 조회')],29,True)
        n('mic',20,165,210,55,'Microphone','external')
        n('os',400,165,450,55,'OS Screen / Pointer / Selection','external')
        n('cloud',925,165,275,55,'Cloud Voice / Semantic APIs','external')
        n('im',0,255,875,415,'Interaction Manager','component')
        n('ch',20,305,205,55,'Channel I/O',owner='im')
        n('vad',280,305,275,55,'Turn-Taking Control',owner='im')
        n('cap',605,305,250,55,'Evidence Capture',owner='im')
        n('raw',35,385,330,65,'Input Observation Window','state','im')
        n('tr',425,385,400,65,'Transcript Revision State','state','im')
        n('tl',20,475,835,175,'Timeline & Buffer',owner='im')
        n('ma',925,255,275,415,'Model Access','component')
        n('vc',945,325,235,70,'Voice / Transcription\nAPI Client',owner='ma')
        n('sc',945,510,235,70,'Semantic API Client',owner='ma')
        e('mic','ch',[(125,220),(125,305)])
        e('os','cap',[(730,220),(730,305)])
        e('ch','vad',[(225,332),(280,332)])
        e('vad','cap',[(555,332),(605,332)])
        e('cap','raw',[(730,360),(730,370),(330,370),(330,385)])
        e('cloud','vc',[(1060,220),(1060,325)])
        e('sc','cloud',[(1120,510),(1120,465),(1190,465),(1190,220)],True)
        e('ch','vc',[(225,320),(255,320),(255,295),(905,295),(905,355),(945,355)])
        e('vc','tr',[(945,375),(905,375),(905,417),(825,417)])
        t(475,230,400,['capture interval / clock epoch'],18)
        t(365,367,380,['sample mapping / r1→r2 / gap'],18)
        if option=='A':
            n('resolve',45,535,315,60,'Temporal Evidence Resolver',owner='tl',diff=True)
            n('qs',425,530,400,75,'Temporal Query State','state','tl',True)
            e('raw','resolve',[(330,450),(330,535)])
            e('tr','resolve',[(770,450),(770,515),(330,515),(330,535)])
            e('resolve','qs',[(360,565),(425,565)],True)
            t(395,615,430,['Q1: r1 / source vector / gaps'],19)
        else:
            n('pub',45,535,235,65,'Temporal Evidence\nPublisher',owner='tl',diff=True)
            n('te',340,530,300,85,'Temporal Evidence\nRepository','state','tl',True)
            n('reader',655,535,185,65,'Temporal\nEvidence Reader',owner='tl',diff=True)
            e('raw','pub',[(300,450),(300,568),(280,568)])
            e('tr','pub',[(770,450),(770,515),(250,515),(250,535)])
            e('pub','te',[(280,568),(340,568)])
            e('te','reader',[(640,568),(655,568)])
            t(335,620,495,['TE46@v1 + Temporal Coverage'],19)
        n('rm',20,710,370,65,'Response Manager','component')
        n('tm',485,710,380,65,'Task Manager','component')
        t(25,780,360,['E20 원문 / P20 실제 전달'],20)
        t(490,780,370,['T21/D21@v2, T22/D22@v1'],20)
        n('cm',0,820,875,405,'Context Manager','component')
        n('rc',925,820,275,110,'Request Controller','component')
        n('ri',925,995,275,110,'Request Interpreter','component')
        e('im','rc',[(875,425),(895,425),(895,695),(920,695),(920,845),(925,845)])
        t(925,765,275,['Input + Evidence Record', 'r1/r2 + refs / gap'],18)
        n('cache',40,1035,350,75,'Source View Cache','state','cm')
        n('um',535,1035,320,75,'User Memory','state','cm')
        if option=='A':
            n('compose',40,885,365,70,'Context Composer',owner='cm',diff=True)
            n('set',535,875,320,85,'Request Evidence Set','state','cm',True)
            e('compose','resolve',[(40,920),(15,920),(15,690),(330,690),(330,595)],True)
            t(25,673,400,['① TemporalQuery → 원본 결합 → Result'],19)
            e('compose','set',[(405,920),(535,920)])
            e('compose','rm',[(145,885),(145,775)],True)
            e('compose','tm',[(380,885),(380,805),(675,805),(675,775)],True)
            e('rc','compose',[(925,865),(900,865),(900,810),(255,810),(255,885)],True)
            e('cache','compose',[(220,1035),(220,955)],True)
            e('um','compose',[(535,1072),(450,1072),(450,945),(405,945)],True)
            e('ri','compose',[(925,1050),(905,1050),(905,1125),(420,1125),(420,920),(405,920)])
            e('set','ri',[(855,920),(890,920),(890,1025),(925,1025)])
            t(410,965,435,['② owner read + 필요한 의미 가공'],19)
            n('sample',40,1140,365,65,'Q1(r1): a ↔ S1/P1\nb ↔ S2/P2 / 후보와 gap','data','cm')
            n('bundleexample',535,1140,320,65,'P20 → E20 실제 범위\nT21 → D21@v2 / refs','data','cm')
        else:
            n('mp',40,885,345,70,'Memory Publisher',owner='cm',diff=True)
            n('ep',535,875,320,85,'Episodic Memory\nRepository','state','cm',True)
            n('er',535,1035,320,65,'Evidence Reader',owner='cm',diff=True)
            # User Memory remains separate; its shared state is below the reader.
            byid = {x['id']:x for x in s.nodes}
            byid[p+'um'].update(y=1140,h=60)
            e('mp','rm',[(145,885),(145,775)],True)
            e('mp','tm',[(375,885),(375,805),(675,805),(675,775)],True)
            e('rc','mp',[(925,865),(900,865),(900,810),(255,810),(255,885)],True)
            e('mp','ep',[(385,920),(535,920)])
            e('cache','mp',[(220,1035),(220,955)],True)
            e('ep','er',[(695,960),(695,1035)])
            e('um','er',[(695,1140),(695,1100)])
            e('er','reader',[(855,1067),(905,1067),(905,568),(840,568)],True)
            e('ri','er',[(925,1080),(855,1080)],True)
            t(910,675,290,['① 유효 TE + coverage'],19)
            t(410,969,440,['② P20→E20 / T21→D21@v2', 'Historical Coverage / refs'],19)
            n('sample',40,1140,345,65,'TEv1(r1): a ↔ S1/P1\nb ↔ S2/P2 / 후보와 gap','data','cm')
        e('ri','sc',[(1200,1050),(1215,1050),(1215,545),(1180,545)],True)
        source='compose' if option=='A' else 'mp'
        e(source,'sc',[(100,955),(100,1025),(880,1025),(880,680),(1220,680),(1220,565),(1180,565)],True)
        t(945,610,235,['조건부 C-CONTEXT', 'ID/시간 결합은 코드'],19)
        e('ri','rc',[(1060,995),(1060,930)],True)
        e('rc','tm',[(1060,820),(1060,790),(865,790),(865,742)])
        t(940,705,250,['C-INTERPRET', '같은 의미 LLM'],21)
        t(925,940,285,['④ 의미 제안 ↔ 채택/거절'],19)
        t(925,1120,275,['③ Bundle / Receipt'],20)
        t(925,1160,280,['새 Task는 채택 후 연결', '위임/출력은 공통 owner'],19)
        if option=='A':
            t(0,1260,1210,['첫/새 범위: 필요한 결합만 수행 | 반복: 유효 cache 재사용 가능',
                             '강점: 요청별 조절 / 비용: read/결합/재검증이 요청 경로에 남음'],23)
        else:
            t(0,1260,1210,['첫/정정 직후: 유효 게시 대기 | 반복: 같은 게시 관계 재사용',
                             '강점: 공통 재사용 / 비용: 생산/coverage/갱신/미사용 자료'],23)
    s.text(40,1360,2470,['범례: 직각 Component | 둥근 Module | 원통 owner 상태 | 육각형 외부 | 살구색 실제 차이 | 양방향 화살표 조회/반환',
                         '시간 후보는 지칭 정답이 아님. B는 시간 자료를 Context Manager에 재게시하지 않음. 배치/선 간격은 성능 수치가 아님.'],20)
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
