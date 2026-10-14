"""One-page DP-42 comparison, redrawn from the detailed implementation designs."""
import hashlib
import json
from generate_dp42_implementation_diagrams import Scene, OUT, DOC, ACCENT, MUTED


def structure():
    p = Scene('choice42-structure', '대화와 업무 상태를 함께 확정할 것인가, 각자 확정하고 연결할 것인가',
              '같은 사례: 실제 제시된 Q1 “상반기로 할까요?” → u2 “응, 상반기로” → K1 외부 접수 확인 → P2 실제 전달')
    for side,dx in [('A',0),('B',1920)]:
        a=side=='A'
        def n(k,x,y,w,h,name,kind='module',owner=None,diff=False):
            p.node(side+k,x+dx,y,w,h,name,kind,side+owner if owner else None,diff)
        def t(x,y,text,size=24,color=MUTED,bold=False):p.text(x+dx,y,text,size,color,bold)
        def e(src,dst,sp='R',tp='L',via=(),label='',at=None,ret=False,sd=0,td=0,accent=False):
            p.edge(side+src,side+dst,sp,tp,[(x+dx,y) for x,y in via],label,(at[0]+dx,at[1]) if at else None,ret,sd,td,accent)
        t(80,240,'A  통합 Core / 관련 owner 변경의 공동 확정' if a else 'B  독립 서비스 / 명령과 접수 사실로 연결',34,ACCENT,True)
        t(80,295,'공통 수명  C1 대화 전환 뒤에도 T1 업무와 X1 외부 실행은 지속',26)
        t(80,335,'R1 위임 → Q1 질문 / P1 실제 제시 → R2 답변 → R3 수정 / X2',25)
        t(80,375,'R2 요청 처리가 끝나도 T1은 유지. 아래 상태 owner와 연결한다.',24)
        n('im',120,430,1650,160,'Interaction Manager','component')
        n('input',150,510,350,60,'입력 수집기',owner='im')
        n('stop',650,510,400,60,'로컬 출력 제어기',owner='im')
        n('io',1270,510,470,60,'출력 실행 / 실제 전달 확인',owner='im')
        e('input','stop',label='onset',at=(535,526))
        e('stop','io',label='output epoch / 즉시 stop',at=(1070,565))
        for k,x,w,name in [('ma',120,290,'Model Access'),('cm',450,300,'Context Manager'),('pm',790,310,'Policy Manager')]:
            n(k,x,620,w,65,name,'external')
        n('agent',1220,620,520,65,'Downstream Agent','external')
        t(120,410,'공통 API 의존성: 클라우드 모델 / 근거 / 현재 권한',20)
        if a:p.band(80+dx,735,1750,1250,'VIA Core process')
        else:
            p.band(80+dx,735,990,1250,'대화 서비스 process',True)
            p.band(1150+dx,735,680,1250,'업무 서비스 process',True)
        n('ri',120,810,390,295,'Request Interpreter','component')
        n('interpret',150,890,330,70,'요청 해석기',owner='ri')
        n('scratch',150,1010,330,65,'해석 작업 / RAM','store',owner='ri')
        n('rc',620,810,410,790,'Request Controller','component')
        for k,y,name,diff in [('adopt',890,'요청 채택기',False),('intent',1040,'관계 변경 준비기' if a else '명령 의도 기록기',True),('control',1190,'현재 입력 제어기',not a),('reflect',1340,'진행 사실 반영기',not a),('rebind',1490,'재개 조정기' if a else '재연결 조정기',not a)]:
            n(k,650,y,350,70,name,owner='rc',diff=diff)
        n('rm',120,1220,390,380,'Response Manager','component')
        n('publish',150,1310,330,70,'게시 제어기',owner='rm')
        n('ledger',150,1470,330,80,'실제 전달 기록기',owner='rm')
        n('tm',1190,810,600,430,'Task Manager','component')
        n('query',1220,890,230,70,'업무 조회기',owner='tm')
        n('accept',1510,890,250,70,'답변 접수기',owner='tm',diff=not a)
        n('gate',1220,1060,540,70,'업무 접수 및 전송 게이트',owner='tm',diff=not a)
        n('apply',1220,1160,540,55,'업무 사건 적용기',owner='tm')
        n('ag',1190,1320,600,280,'Agent Gateway','component')
        n('send',1220,1400,240,70,'명령 전송 및 확인',owner='ag')
        n('ingress',1510,1400,250,70,'외부 사건 수신기',owner='ag')
        n('recover',1220,1510,540,60,'전송 복구기',owner='ag')
        # A shared unit of work versus B's two local repositories.
        if a:
            n('ss',120,1710,1670,240,'State Store','component')
            n('tx',150,1790,1610,55,'트랜잭션 관리자',owner='ss',diff=True)
            for k,x,w,name in [('dialogue',150,440,'요청 관계 / 실제 전달'),('work',650,440,'업무 / 실행 / 질문'),('transport',1220,540,'명령 / gate / 외부 inbox')]:
                n(k,x,1870,w,60,name,'store',owner='ss')
                e('tx',k,sp='B',tp='T',sd=x+w/2-955)
            e('rc','tx',sp='B',tp='T',td=-130,label='4  A1: R2 관계 + Q1 v8 + K1 + gate',at=(580,1682),accent=True)
            e('tm','tx',sp='L',tp='T',sd=160,td=465,via=[(1170,1185),(1170,1650),(1420,1650)],label='5  A2 CAS / 6  A3 사실 적용',at=(1210,1682))
            e('ag','tx',sp='B',tp='T',td=700,via=[(1490,1620),(1655,1620)])
            e('rm','tx',sp='B',tp='T',td=-640,label='8  A4 actual receipt',at=(140,1682))
        else:
            for k,x,w in [('ds',120,910),('ws',1190,600)]:
                n(k,x,1710,w,240,'State Store','component')
                n(k+'tx',x+30,1790,w-60,55,'트랜잭션 관리자',owner=k)
            for k,x,w,name,owner in [('dialogue',150,390,'요청 관계 / 실제 전달','ds'),('outbox',610,390,'의도 outbox / cursor','ds'),('work',1220,250,'업무 / 실행 / 질문','ws'),('transport',1510,250,'명령 / gate / inbox','ws')]:
                n(k,x,1870,w,60,name,'store',owner=owner)
                e(owner+'tx',k,sp='B',tp='T',sd=x+w/2-(575 if owner=='ds' else 1490))
            e('rc','dstx',sp='B',tp='T',td=250,label='3  D1 의도 / 4  D2 접수 / 6  D3 사실',at=(550,1682),accent=True)
            e('rm','dstx',sp='B',tp='T',td=-260,label='8  D4 actual receipt',at=(140,1682))
            e('tm','wstx',sp='L',tp='T',sd=160,td=-70,via=[(1170,1185),(1170,1650),(1420,1650)],label='4  W1 접수 / 5  W2 CAS / 6  W3',at=(1190,1682),accent=True)
            e('ag','wstx',sp='B',tp='T',td=165,via=[(1490,1620),(1655,1620)])
        # Input and current semantic association; input does not contain a Task ID.
        e('input','adopt',sp='B',tp='T',td=100,via=[(325,600),(1120,600),(1120,790),(925,790)],label='1  u2 / onset → hold',at=(530,712))
        e('adopt','interpret',sp='L',tp='R',sd=-15,td=-15,label='2  원문 + 현재 근거',at=(485,889))
        e('interpret','adopt',ret=True,sd=20,td=20,label='Q1/T1 의미 제안',at=(485,989))
        e('interpret','scratch',sp='B',tp='T')
        e('interpret','ma',sp='T',tp='B',sd=115,td=115,via=[(430,780),(380,780)])
        e('interpret','cm',sp='R',tp='B',sd=0,td=100,via=[(545,925),(545,800),(700,800)])
        e('adopt','pm',sp='T',tp='B',sd=160,td=40)
        e('adopt','query',sd=0,td=0,label='2  Q1 v7 / T1 조회 ↔',at=(1025,910))
        e('adopt','ledger',sp='L',tp='R',sd=25,td=0,via=[(570,950),(570,1510)],label='2  P1 actual 조회 ↔',at=(520,1150))
        e('adopt','intent',sp='B',tp='T',label='현재 검사',at=(840,1020))
        e('adopt','control',sp='R',tp='R',sd=25,via=[(1045,950),(1045,1225)])
        # The same answer changes different commit dependencies.
        e('intent','accept',sp='R',tp='B',via=[(1100,1075),(1100,1000),(1635,1000)],label='3  prepare_answer(K1)' if a else '3  D1 저장 뒤 K1 / ReleaseAndReconcile',at=(1090,1035),accent=True)
        if a:e('accept','intent',sp='L',tp='R',sd=20,td=25,via=[(1480,945),(1480,980),(1080,980),(1080,1100)],ret=True,label='4  TaskWriteSet / conflict',at=(1120,985),accent=True)
        else:e('accept','reflect',sp='L',tp='R',sd=20,td=-20,via=[(1480,945),(1480,980),(1090,980),(1090,1355)],ret=True,label='4  W1 확정 → receipt / D2',at=(1100,1290),accent=True)
        e('control','gate',via=[(1110,1225),(1110,1095)],label='hold / revision / ACK ↔',at=(1040,1265),accent=not a)
        e('send','gate',sp='L',tp='L',via=[(1135,1435),(1135,1115)],td=20,label='5  claim / CAS ↔',at=(1050,1450))
        e('send','agent',sp='L',tp='L',sd=-20,via=[(1160,1415),(1160,652.5)],label='5  CAS 성공 뒤 send(K1)',at=(1195,721))
        e('agent','ingress',sp='R',tp='R',via=[(1810,652.5),(1810,1435)],ret=True,label='6  외부 receipt / event',at=(1510,1290))
        e('ingress','apply',sp='R',tp='R',via=[(1800,1435),(1800,1187.5)],ret=True)
        e('apply','reflect',sp='L',tp='R',via=[(1145,1187.5),(1145,1375)],ret=True,label='6  원본 fact' if a else '6  내구 event / D3',at=(1040,1320),accent=not a)
        e('reflect','publish',sp='L',tp='R',via=[(590,1375),(590,1345)],label='7  확인 사실 게시 허용',at=(135,1195))
        e('publish','io',sp='L',tp='L',via=[(75,1345),(75,610),(1180,610),(1180,540)],label='7  release(P2)',at=(135,1640))
        e('io','ledger',sp='R',tp='B',via=[(1850,540),(1850,1640),(315,1640)],ret=True,label='8  실제 표시 / audible range',at=(1350,1640))
        e('rebind','query',sp='R',tp='B',via=[(1075,1525),(1075,975),(1335,975)],label='same K1 조회 / 재결합' if not a else '내구 상태 재조회',at=(1035,1585),accent=not a)
        if not a:t(1200,1971,'접수 receipt / event outbox도 W1/W3에 함께 저장',20)
        e('recover','agent',sp='R',tp='R',via=[(1870,1540),(1870,652.5)],label='UNKNOWN → 외부 조회',at=(1505,795))
        t(120,2025,'A1: 관련 변경 모두 반영 / revision 충돌이면 모두 미반영' if a else 'W1 뒤 D2 전: 업무는 접수 완료, 대화는 접수 대기일 수 있음',25,ACCENT,True)
        t(120,2065,'내부 비동기 / worker 허용. 외부 send와 실제 출력은 transaction 밖.' if a else 'W2 전송은 D2와 독립. 응답 유실은 같은 K1 조회 / cursor 재결합.',24)
    p.text(64,2120,'실선 = 호출 / 저장   점선 = 반환 사실   ↔ = 조회와 응답 축약   둥근 상자 = 내부 Module   원통 = 상태   살구색 = 확정 / 인계 차이',23)
    p.text(3776,2152,'1~8은 양안 공통 처리 단계. 로컬 접수 ≠ 외부 접수 ≠ 사용자 실제 전달. B 우위와 A/B 선정 없음.',22,align='right')
    return p


def check_render():
    manifest=json.loads((OUT/'dp42-comparison-render.json').read_text())
    assert len(manifest['records'])==1
    row=manifest['records'][0]
    assert row['stem']=='choice42-structure' and row['text_geometry_issues']==0
    assert (row['width'],row['height'])==(3840,2160)
    for ext in ('svg','png'):
        assert hashlib.sha256((OUT/('choice42-structure.'+ext)).read_bytes()).hexdigest()==row[ext+'_sha256']
    body=DOC.read_text()
    assert '![42 메인' in body
    for ext in ('svg','drawio','png'):assert './diagrams/choice42-structure.'+ext in body
    for ext in ('svg','drawio','png'):
        name='dp42-comparison-before-implementation.'+ext
        assert (OUT/name).exists() and './diagrams/'+name in body
    assert '![구현 수준 재배치 전 DP-42 메인 비교도]' in body
    print('PASS: DP-42 implementation comparison render, primary document and preserved previous page')
