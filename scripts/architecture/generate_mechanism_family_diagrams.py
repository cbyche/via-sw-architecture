#!/usr/bin/env python3
"""Paired documentation figures for the broad Stage 4 mechanism review.
No candidate execution or performance measurement is performed.
"""
from pathlib import Path
import argparse
import xml.etree.ElementTree as ET
from stage4_diagram_design import Plate, INK, BLUE, GREEN, MUTED

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/diagrams'


def start(slug, num, title, thesis):
    return Plate(slug, num, title, thesis, height=1840)


def panel(p, x, code, name, note):
    p.panel(x, 1184, code, name, note)
    p.boundary(code+'via', x, 340, 1184, 1090, 'VIA SOFTWARE', '초점: 비교하는 내부 처리 구조')


def semantic():
    p = start('mechanism-semantic', 'M-2',
              '요청 이해의 정확성을 위한 해석 구조 — 통합 판단과 단계별 결합',
              '구조 및 자료 흐름: 입력 준비와 최종 확정은 공유하고, 최종 의미를 생산하는 처리 체계를 교체한다.')
    for x, s, c in [(64,'T',BLUE),(1312,'A',GREEN)]:
        panel(p,x,s,'통합 의미 해석' if s=='T' else '단계별 의미 결합',
              '관련 근거를 함께 보고 목표/대상/업무 관계 제안' if s=='T' else '중간 후보를 연결하고 규칙으로 조립; 최종 통합 호출 없음')
        k=lambda n:s+n
        p.component(k('rc'),x+36,380,1112,120,'Request Controller',role='Interaction Manager의 확정 입력/시점 근거, 현재 revision과 전체 예산 관리')
        p.component(k('cm'),x+36,635,340,125,'Context Manager',role='허용된 기본/추가 근거 조회')
        p.component(k('ri'),x+460,635,688,570,'Request Interpreter')
        p.component(k('ma'),x+36,975,340,125,'Model Access',role='한 Omni로 의미 판단 요청')
        p.component(k('end'),x+460,1275,688,115,'Request Controller',role='5 현재 근거/권한을 검증해 확정 또는 질문')
        p.node(k('omni'),x+36,1510,340,115,'같은 on-device Omni\nweights 한 개 / 모델 의존성',kind='model')
        p.text(x+460,1496,['위아래 Request Controller는 같은 owner다.',
                           '6 후속: Task Manager와 Agent Gateway의 업무 인계,',
                           'Response Manager와 Interaction Manager의 실제 전달.',
                           '단계 2~4의 모델 왕복은 같은 Model Access 경로를 쓴다.'],19,MUTED)
        if s=='T':
            p.node(k('joint'),x+490,735,628,155,'통합 의미 해석\n원문 + 화면/자료 + 관련 Task',color=c,owner=k('ri'))
            p.node(k('proposal'),x+490,1035,628,110,'목표 / 대상 / 업무 관계 / 미해결 근거\n하나의 Semantic Proposal',kind='data',color=c,owner=k('ri'))
            p.edge(k('joint'),k('proposal'),label='통합 후보',at=(x+820,945),color=c)
        else:
            names=['발화 분해\n2 목표/부정/조건/지칭 후보',
                   '근거 결합\n3 자료 ID와 버전에 후보 연결',
                   '업무 관계 결합\n4 Task/질문/처리 방식 후보',
                   '해석 조립\n4 미해결 유지, 코드로 결합']
            for j,name in enumerate(names):
                p.node(k('stage'+str(j)),x+490,725+j*110,628,85,name,color=c,owner=k('ri'))
                if j:p.edge(k('stage'+str(j-1)),k('stage'+str(j)),color=c)
        p.edge(k('rc'),k('cm'),sp='B',tp='T',sd=-350,via=[(x+242,580),(x+206,580)],label='1 기본 / 3 추가 근거 요청',at=(x+48,552))
        p.edge(k('cm'),k('rc'),sp='R',tp='B',td=-175,via=[(x+410,697.5),(x+410,535),(x+417,535)],ret=True,label='1 / 3 자료와 조회 범위 반환',at=(x+450,540))
        p.edge(k('rc'),k('ri'),sp='B',tp='T',sd=212,label='2 해석 요청 / 3 조회 뒤 재개',at=(x+790,558))
        p.edge(k('ri'),k('rc'),sp='R',tp='R',sd=-180,td=15,via=[(x+1168,740),(x+1168,455)],ret=True,label='3 추가 근거 제안',at=(x+915,602))
        p.edge(k('ri'),k('ma'),sp='L',tp='R',sd=60,td=-15,via=[(x+404,980),(x+404,1022.5)],label='2~4 모델 요청',at=(x+205,940))
        p.edge(k('ma'),k('ri'),sp='R',tp='L',sd=20,td=125,via=[(x+428,1057.5),(x+428,1045)],ret=True,label='2~4 판단 반환',at=(x+225,1125))
        p.edge(k('ri'),k('end'),label='4 Semantic Proposal 반환',at=(x+820,1230),ret=True)
        p.edge(k('ma'),k('omni'),sd=-30,td=-30,label='추론 요청',at=(x+40,1380))
        p.edge(k('omni'),k('ma'),sp='T',tp='B',sd=30,td=30,ret=True,label='결과',at=(x+250,1460))
    p.footer('검정 = 공통, 파랑/초록 = 바뀌는 처리. 단계가 많다는 사실은 정확도 우위의 증거가 아니다.')
    return p


def state():
    p=start('mechanism-state','S-1','대화와 업무 관계의 복구를 위한 저장 구조 — 현재 상태와 확정 이력',
            '질문 답변과 command를 확정하는 범위를 확대했다. 같은 local transaction과 외부 확인 조건을 유지한다.')
    for x,s,c in [(64,'T',BLUE),(1312,'A',GREEN)]:
        panel(p,x,s,'현재 owner 기록이 원본' if s=='T' else '확정 domain 이력이 원본',
              '현재 상태와 미완료 원장에서 재개' if s=='T' else '의미 전이 → 상태 생성; current view는 재구성 가능')
        k=lambda n:s+n
        for j,name in enumerate(['Request Controller','Task Manager','Agent Gateway']):
            p.component(k('o'+str(j)),x+36+386*j,385,340,120,name,role=['질문/입력 의미 검증','업무/command 의미 검증','전송/실제 접수 상태'][j])
            p.line([(x+206+386*j,505),(x+206+386*j,590)],arrow=False)
        p.line([(x+206,590),(x+978,590)],arrow=False)
        p.component(k('ss'),x+36,700,1112,690,'State Store',role='한 local DB transaction / 실제 외부 호출은 확정 이후')
        p.line([(x+552,590),(x+552,700)])
        p.line([(x+632,700),(x+632,590)],dashed=True)
        p.label(x+55,620,'2 변경 전달 → State Store / 3 확정 결과 → 각 owner')
        p.node(k('effect'),x+670,1180,430,125,'미완료 효과 원장\ncommand / 실제 전달 receipt',kind='store',owner=k('ss'))
        if s=='T':
            p.node(k('write'),x+76,800,430,110,'현재 상태 갱신기',color=c,owner=k('ss'))
            p.node(k('current'),x+670,800,430,110,'권위 current records',kind='store',color=c,owner=k('ss'))
            p.node(k('load'),x+76,1180,430,125,'State Loader\n현재 owner 상태 복원',color=c,owner=k('ss'))
            p.edge(k('write'),k('current'),sp='R',tp='L',label='3 갱신',at=(x+552,820),color=c)
            p.edge(k('current'),k('load'),sp='L',tp='T',via=[(x+574,855),(x+574,1120),(x+291,1120)],ret=True,color=c,label='복구 읽기',at=(x+610,1080))
        else:
            p.node(k('write'),x+76,800,430,110,'Journal Append\n의미 전이 batch 확정',color=c,owner=k('ss'))
            p.node(k('journal'),x+670,800,430,110,'권위 Domain Journal',kind='store',color=c,owner=k('ss'))
            p.node(k('reduce'),x+76,975,430,110,'Projection Engine\nversioned reducer 적용',color=c,owner=k('ss'))
            p.node(k('current'),x+670,975,430,110,'파생 current projection',kind='store',color=c,owner=k('ss'))
            p.node(k('load'),x+76,1180,430,125,'Replay Engine\ncheckpoint + committed tail',color=c,owner=k('ss'))
            p.edge(k('write'),k('journal'),sp='R',tp='L',label='3 이력 기록',at=(x+548,820),color=c)
            p.edge(k('journal'),k('reduce'),sp='B',tp='T',via=[(x+885,947),(x+291,947)],color=c)
            p.edge(k('reduce'),k('current'),sp='R',tp='L',label='3 상태 생성',at=(x+528,998),color=c)
            p.edge(k('journal'),k('load'),sp='L',tp='T',via=[(x+574,855),(x+574,1137),(x+291,1137)],ret=True,color=c)
            p.edge(k('load'),k('reduce'),sp='T',tp='B',ret=True,color=c,label='재생',at=(x+115,1110))
        p.edge(k('current'),k('effect'),label='3 전송 의도도 함께 확정',at=(x+725,1113))
        p.text(x+56,1482,['1 시작: Request Controller가 해석된 질문 답변을 Task Manager에 전달해 의미 검증.',
                          '4 확정 뒤 Agent Gateway가 외부 Agent에 전송. 5 ACK/상태를 Task Manager에 반환.',
                          '6 결과는 Response Manager → Interaction Manager로 전달, 실제 receipt 반환.',
                          '재시작: owner 관계와 현재 policy 확인 → Agent 상태 조회 → 유효 범위만 개방.',
                          '재생은 이미 확정된 전이를 적용한다. 모델 재추론, Agent 재실행과 음성 재생을 하지 않는다.',
                          '그림에서 State Store 내부 원통은 모두 내구 저장이다.'],19,MUTED)
    p.footer('초점: 저장 권위와 정상 write/복구 경로. 원본 journal 손상이나 잘못된 의미 판단까지 복구한다고 주장하지 않는다.')
    return p


def scope():
    p=start('mechanism-scope','S-2','허용 범위 내 자료 처리를 위한 구조 — 공유 처리와 권한별 격리',
            '자료 조회와 semantic 입력 구성의 권한 경계를 확대했다. Task 의미/최종 전송 확정과 한 개 Omni는 공유한다.')
    for x,s,c in [(64,'T',BLUE),(1312,'A',GREEN)]:
        panel(p,x,s,'공유 Core의 정책 검사' if s=='T' else '중개자와 제한 process',
              '정상 권한 port + 기존 위험 connector 격리' if s=='T' else '처리 코드의 전역 자료/credential/임의 연동 권한 제거')
        k=lambda n:s+n
        if s=='T':
            p.boundary(k('core'),x+24,375,1136,955,'Core process','비교하는 자료 처리 코드도 같은 신뢰 영역',color=c,dashed=True)
        else:
            p.boundary(k('core'),x+24,375,510,955,'권한 있는 Core process','',dashed=True)
            p.boundary(k('cell'),x+604,375,556,955,'제한된 자료 처리 process','',color=c,dashed=True)
        p.component(k('rc'),x+50,445,452,110,'Request Controller',role='유효 요청과 최종 의미 확정')
        p.component(k('policy'),x+50,610,452,110,'Policy Manager',role='현재 범위 / 수신자 / policy epoch')
        p.component(k('cm'),x+634,445,500,360,'Context Manager',role='자료 처리 부분: 조립 및 읽은 범위/버전')
        p.node(k('cache'),x+664,605,440,120,'공유 Core의 자료/cache (RAM)' if s=='T' else '현재 범위의 자료/cache (RAM)',kind='store',color=c,owner=k('cm'))
        p.component(k('ri'),x+634,965,500,120,'Request Interpreter',role='허용 근거에서 해석 후보 생성')
        p.component(k('ma'),x+50,1145,452,115,'Model Access',role='동일한 공유 모델 service 연동')
        p.node(k('omni'),x+50,1520,452,110,'같은 Omni / 공유 inference service\n양안 모두 신뢰 경계 안의 의존성',kind='model')
        p.node(k('source'),x+634,1520,500,110,'같은 Context source\n실제 조회는 허용된 adapter/worker 사용',kind='external')
        p.edge(k('rc'),k('cm'),sp='R',tp='L',sd=-15,td=-140,label='1 준비 요청',at=(x+516,438))
        p.edge(k('cm'),k('rc'),sp='L',tp='R',sd=-110,td=15,ret=True,label='2 자료 참조',at=(x+516,555))
        p.edge(k('rc'),k('ri'),sp='R',tp='L',sd=35,td=-30,via=[(x+618,535),(x+618,995)],label='3 허용 자료로 해석 요청',at=(x+655,908))
        p.edge(k('ri'),k('rc'),sp='R',tp='T',via=[(x+1152,1025),(x+1152,420),(x+276,420)],ret=True,label='3 제안 반환 / 4 현재 조건 검증',at=(x+690,415))
        if s=='A':
            p.component(k('broker'),x+50,825,452,120,'Data Access Broker',color=c,role='권한 검사, 자료 열기 및 참조 발급')
            p.edge(k('policy'),k('broker'),label='1 현재 권한 적용',at=(x+95,755))
            p.edge(k('cm'),k('broker'),sp='L',tp='R',sd=130,td=-20,via=[(x+554,755),(x+554,865)],color=c,label='2 조회 요청',at=(x+425,782))
            p.edge(k('broker'),k('cm'),sp='R',tp='L',sd=10,td=160,via=[(x+578,895),(x+578,785)],ret=True,color=c,label='2 허용 자료 참조',at=(x+670,820))
            p.edge(k('ri'),k('broker'),sp='L',tp='R',sd=-20,td=40,via=[(x+554,1005),(x+554,925)],color=c,label='3 모델 요청',at=(x+420,1058))
            p.edge(k('broker'),k('ri'),sp='R',tp='L',sd=20,td=20,via=[(x+590,905),(x+590,1045)],ret=True,color=c)
            p.edge(k('broker'),k('ma'),sd=-35,td=-35,label='3 검증 후 요청',at=(x+80,1030),color=c)
            p.edge(k('ma'),k('broker'),sp='T',tp='B',sd=35,td=35,ret=True,color=c)
            p.edge(k('broker'),k('source'),sp='R',tp='T',sd=25,via=[(x+580,910),(x+580,1465),(x+884,1465)],color=c,label='2 중개자가 자료 조회',at=(x+630,1427))
            p.edge(k('source'),k('broker'),sp='T',tp='R',sd=35,td=45,via=[(x+919,1490),(x+610,1490),(x+610,930)],color=c,ret=True,label='2 허용 자료 반환',at=(x+660,1478))
        else:
            p.edge(k('policy'),k('cm'),sp='R',tp='L',sd=25,td=65,label='1 정책 적용',at=(x+515,728))
            p.edge(k('ri'),k('ma'),sp='L',tp='R',sd=-20,td=-10,via=[(x+554,1005),(x+554,1192.5)],label='3 모델 요청',at=(x+412,1070))
            p.edge(k('ma'),k('ri'),sp='R',tp='L',sd=20,td=20,via=[(x+578,1222.5),(x+578,1045)],ret=True)
            p.edge(k('cm'),k('source'),sp='B',tp='T',via=[(x+884,865),(x+1172,865),(x+1172,1465),(x+884,1465)],label='2 정책상 허용된 자료 조회',at=(x+630,1427),color=c)
            p.edge(k('source'),k('cm'),sp='T',tp='B',sd=35,td=35,via=[(x+919,1490),(x+1158,1490),(x+1158,890),(x+919,890)],color=c,ret=True,label='2 허용 자료 반환',at=(x+660,1478))
        p.edge(k('ma'),k('omni'),sd=-35,td=-35,label='추론 요청',at=(x+70,1367))
        p.edge(k('omni'),k('ma'),sp='T',tp='B',sd=35,td=35,ret=True,label='결과',at=(x+330,1460))
        p.text(x+630,1348,['5 Task Manager / Agent Gateway: 현재 조건을 검증해 전송.',
                          'A의 자료 제공은 Data Access Broker가 다시 검사한다.',
                          '6 결과: Response Manager → Interaction Manager 및 receipt.'],17,MUTED)
    p.footer('OS/중개자/공유 model은 여전히 신뢰 영역이다. 실제 외부 제공과 철회/삭제의 전체 흐름 및 보호 한계는 본문 §3~6.')
    return p


def main():
    ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args()
    pages=[semantic(),state(),scope()]
    drift=[]
    for p in pages:
        p.validate()
        for ext,data in [('svg',p.svg()),('drawio',p.drawio())]:
            root=ET.fromstring(data)
            if ext=='drawio':
                ids=[c.get('id') for c in root.findall('.//mxCell')]
                assert len(ids)==len(set(ids))
                for cell in root.findall('.//mxCell'):
                    for attr in ('parent','source','target'):
                        assert cell.get(attr) is None or cell.get(attr) in ids
            path=OUT/f'{p.slug}.{ext}'
            if args.check:
                if not path.exists() or path.read_text()!=data:drift.append(path.name)
            else:path.write_text(data)
    if drift:raise SystemExit('Diagram drift: '+', '.join(drift))
    print('PASS: 3 mechanism pairs; XML, ownership, bounds, routes and source parity')

if __name__=='__main__':main()
