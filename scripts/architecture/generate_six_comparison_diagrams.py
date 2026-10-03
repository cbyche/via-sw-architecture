#!/usr/bin/env python3
"""Six mechanism comparisons: distinct structures plus same-event walkthroughs."""
import argparse
from pathlib import Path
import xml.etree.ElementTree as ET
from stage4_diagram_design import Plate, INK, MUTED, BLUE, GREEN
ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'docs/architecture/12-decisions/decision-packages/diagrams'
TITLES={31:'요청 이해의 정확성을 위한 의미 구성 — 전체 의미 생성과 제약 결합',32:'화면 지칭의 정확성을 위한 대상 획득 — 관측 영역과 source 객체',33:'여러 업무의 대화 정확성을 위한 협력 — 중앙 해석과 업무별 대화 처리',34:'음성 대화의 반응성과 정정 정확성을 위한 실행 — 발화 단위와 연속 수정',35:'대화와 업무의 복구 정확성을 위한 기록 — 현재 상태와 사건 이력',36:'허용된 자료만 처리하기 위한 권한 배치 — 공유 처리와 제한 처리'}
NAMES={31:('전체 의미 생성','후보와 제약으로 구성'),32:('관측에서 영역 해석','source 객체를 지칭에 결합'),33:('중앙이 전체 대화 해석','업무 대화 actor가 협력'),34:('확정 발화 단위 처리','잠정 작업을 계속 수정'),35:('현재 상태가 원본','확정 사건이 원본'),36:('공유 권한의 처리 코드','Broker와 제한 process')}

def footer(p,note):
 y=p.height-140;p.line([(64,y),(2496,y)],'#C7D1DD',arrow=False)
 p.text(64,y+18,'검정 = 공통 / 파랑 = A 고유 / 초록 = B 고유 / 색은 우위가 아님',19,bold=True)
 p.text(2496,y+18,'실선 → 요청/전달 / 점선 → 반환/확인',18,MUTED,'right')
 p.text(64,y+56,'제목 상자 = Component / 내부 둥근 상자 = Module / 원통 = 저장 / 접힌 종이 = 자료 / 육각형 = 연동 대상',17,MUTED)
 p.text(64,y+92,note,17,MUTED)

def plate(n,event=False):
 return Plate(f'choice{n}-'+('event' if event else 'structure'),f'04-{n}',TITLES[n],
              '같은 사건의 1~5단계 / 본문 §4와 대응 / 시간축은 측정 비율이 아님' if event else '구조도 / 실행 순서 번호 없음 / 공통 책임을 유지하면서 실제 생산 원리와 상태를 비교',height=2260)

def head(p,x,s,n):
 c=BLUE if s=='A' else GREEN
 p.box(x,236,48,48,c,'none',5);p.text(x+24,243,s,27,'white','center',True)
 p.text(x+68,235,NAMES[n][s=='B'],28,bold=True)
 return c,lambda z:s+z

def common(p,x,k,top='Request Controller',bottom='Request Controller',note_offset=800):
 p.boundary(k('via'),x,340,1184,1460,'VIA SOFTWARE','역할/논리 구조, OS process는 별도 명시')
 p.component(k('input'),x+36,400,720,135,top,role='같은 사용자 입력, 근거와 현재 허용 범위')
 p.component(k('commit'),x+36,1600,720,145,bottom,role='확정 사실의 Text/음성 게시, 실제 전달 기록' if bottom=='Response Manager' else '현재 입력/자료/권한 확인, 확정 사실만 전달')
 p.text(x+note_offset,1640,['외부 업무는 공통', 'Task Manager', '→ Agent Gateway', '→ Downstream Agent', '접수/결과는 같은 경로로 반환'],18,MUTED)

def model(p,x,k,source,interpreter=False):
 if interpreter:
  p.component(k('ri'),x+808,760,340,160,'Request Interpreter',role='범위별 의미 제안')
  p.component(k('ma'),x+808,1140,340,140,'Model Access',role='역할별 요청 / 한 weights')
  p.edge(k('ri'),k('ma'),sd=-25,td=-25,label='의미 요청',at=(x+815,1030))
  p.edge(k('ma'),k('ri'),sp='T',tp='B',sd=25,td=25,ret=True)
  target=k('ri')
 else:
  p.component(k('ma'),x+808,800,340,140,'Model Access',role='역할별 요청 / 한 weights');target=k('ma')
 p.node(k('omni'),x+808,1940,340,120,'공유 Omni 한 개\n모델 내부는 책임 밖',kind='model')
 # Route model dependency in the unused outer lane, avoiding the external-work note.
 mx=x+1160
 sy=p.port(k('ma'),'R',-20)[1]
 p.edge(k('ma'),k('omni'),sp='R',tp='R',sd=-20,td=-20,via=[(mx,sy),(mx,1980)],label='추론 요청',at=(x+835,1840))
 sy=p.port(k('ma'),'R',20)[1]
 p.edge(k('omni'),k('ma'),sp='R',tp='R',sd=20,td=20,via=[(x+1174,2020),(x+1174,sy)],ret=True)
 return target

def call(p,src,dst,x,y,c=INK):
 # Right-facing request/return on separate horizontal lanes, offset to target centre.
 sx,sy,sw,sh=p.nodes[src];dx,dy,dw,dh=p.nodes[dst]
 yy=dy+dh/2
 p.edge(src,dst,sp='R',tp='L',sd=yy-25-(sy+sh/2),td=-25,label='요청',at=(x,y),color=c)
 p.edge(dst,src,sp='L',tp='R',sd=25,td=yy+25-(sy+sh/2),ret=True,color=c)

def flow(p,a,b,label,x,y,c=INK):
 p.edge(a,b,sd=-22,td=-22,label=label,at=(x,y),color=c)
 p.edge(b,a,sp='T',tp='B',sd=22,td=22,ret=True,color=c)

def returned(p,a,b,label,x,y,c=INK):
 p.edge(a,b,label=label,at=(x,y),color=c,ret=True)

def structural(n):
 p=plate(n)
 for x,s in [(64,'A'),(1312,'B')]:
  c,k=head(p,x,s,n);common(p,x,k,'Interaction Manager' if n in (32,34) else 'Request Controller','Response Manager' if n==34 else 'Request Controller',note_offset=840 if n==36 else 800)
  if n==31:
   p.component(k('engine'),x+36,700,720,730,'Request Interpreter')
   p.node(k('first'),x+70,810,652,100,'통합 의미 생성' if s=='A' else '표현 변환 / 후보 생산',color=c,owner=k('engine'))
   p.node(k('ir'),x+70,1030,652,105,'완성된 전체 관계 제안 (임시)' if s=='A' else '타입 있는 의미 IR / 후보 표 (임시)',kind='data',color=c,owner=k('engine'))
   p.node(k('last'),x+70,1250,652,105,'제안 검증' if s=='A' else '제약 해결 / 충돌 질문 생성',color=c,owner=k('engine'))
   p.edge(k('first'),k('ir'),label='모델 결과',at=(x+435,950),color=c)
   p.edge(k('ir'),k('last'),label='완성안 검사' if s=='A' else '관계의 해를 구성',at=(x+435,1170),color=c)
   target=model(p,x,k,k('first'));call(p,k('first'),target,x+765,785)
   flow(p,k('input'),k('engine'),'해석 요청 / 제안 반환',x+430,590)
   returned(p,k('engine'),k('commit'),'의미 / 미해결 반환',x+430,1510,c)
   p.text(x+36,1910,['A: 모델이 전체 관계 생산 → 코드가 실행 가능성 검사', 'B: 모델은 후보 생산 → 규칙/해법이 관계 구성', '같은 Request Interpreter 책임 안의 핵심 생산 구조 교체'],21,MUTED)
  elif n==32:
   p.component(k('cm'),x+36,680,720,470,'Context Manager')
   p.node(k('prod'),x+70,795,652,105,'관측 조립' if s=='A' else '객체 연동 / 시점 결합 / 참조 해석',color=c,owner=k('cm'))
   p.node(k('data'),x+70,990,652,110,'시점별 화면/지시 묶음 (RAM)' if s=='A' else '제공자/문서/객체/버전 참조 (RAM)',kind='store',color=c,owner=k('cm'))
   p.edge(k('prod'),k('data'),label='근거 생산',at=(x+430,935),color=c)
   p.component(k('mainri'),x+36,1280,720,150,'Request Interpreter',role='지칭 후보 반환 / source와 현재 사용 가능성 확인')
   flow(p,k('input'),k('cm'),'관측 전달 / 수신 확인',x+430,595)
   p.edge(k('cm'),k('mainri'),label='Request Controller 경유: 발화와 대상 근거',at=(x+180,1200),color=c)
   target=model(p,x,k,k('prod'))
   # Actual model interpretation is owned by Request Interpreter, not Context Manager.
   p.items=[i for i in p.items if not (i.get('source') in (k('prod'),target) and i.get('target') in (k('prod'),target))]
   p.edge(k('mainri'),k('ma'),sp='R',tp='L',sd=-30,td=0,via=[(x+782,1325),(x+782,870)],label='의미 요청',at=(x+800,1400))
   p.edge(k('ma'),k('mainri'),sp='L',tp='R',sd=35,td=30,via=[(x+796,905),(x+796,1385)],ret=True)
   returned(p,k('mainri'),k('commit'),'후보 반환 / 현재 유효성 확인',x+430,1500,c)
   p.node(k('source'),x+36,1940,720,120,'같은 앱/문서 환경 (VIA 밖)\nA 화면 관측 / B 공개 객체 읽기와 재확인',kind='model')
   src=k('input') if s=='A' else k('cm')
   sy=p.port(src,'L',-25)[1]
   p.edge(src,k('source'),sp='L',tp='L',sd=-25,td=-25,via=[(x+12,sy),(x+12,1975)],color=c)
   sy=p.port(src,'L',25)[1]
   p.edge(k('source'),src,sp='L',tp='L',sd=25,td=25,via=[(x+24,2025),(x+24,sy)],ret=True,color=c)
   p.text(x+36,1830,'source 요청/반환: A 관측 획득 / B 객체 읽기, 유효성 재확인',19,MUTED)
  elif n==33:
   if s=='A':
    p.component(k('owner'),x+36,700,720,415,'Request Controller',role='중앙 Conversation의 의미/질문 상태 소유')
    p.node(k('state'),x+70,870,652,140,'관련 업무/질문 관계 (내구)\n업무별 요약/cache도 사용 가능',kind='store',color=c,owner=k('owner'))
    p.node(k('engine'),x+36,1270,720,150,'전체 의미 / 대화 변경 제안\n중앙 해석 결과 (임시)',kind='data',color=c)
    flow(p,k('input'),k('owner'),'같은 발화와 실제 제시 기록',x+430,590)
    returned(p,k('owner'),k('engine'),'중앙 해석 결과를 확정 경로로',x+430,1190,c)
   else:
    p.component(k('owner'),x+36,700,720,200,'Dialogue Coordinator',color=c,role='참가자 / 절별 제안 / 충돌과 교차 참조 조정')
    p.component(k('engine'),x+36,1040,720,460,'Task Dialogue Runtime',color=c,role='논리 actor의 사건 처리 / OS process 아님')
    for j,off in enumerate((70,404)):
     p.node(k('actor'+str(j)),x+off,1160,310,100,'보고서 대화 actor' if j==0 else '메일 대화 actor',color=c,owner=k('engine'))
     p.node(k('state'+str(j)),x+off,1315,310,110,'자기 질문/제약\n내구 상태 + 우편함',kind='store',color=c,owner=k('engine'))
    flow(p,k('input'),k('owner'),'입력 / 후보 참가자',x+430,590)
    flow(p,k('owner'),k('engine'),'발화/참조 요청과 actor 제안 반환',x+430,955,c)
   target=model(p,x,k,k('owner'),True)
   # Request/return through right lane to interpreter; actor requests may finish at different times.
   src=k('owner') if s=='A' else k('engine')
   sy=p.port(src,'R',-20)[1]; ty=p.port(target,'L',-25)[1]
   p.edge(src,target,sp='R',tp='L',sd=-20,td=-25,via=[(x+780,sy),(x+780,ty)],label='해석 요청',at=(x+804,990),color=c)
   sy=p.port(src,'R',20)[1];ty=p.port(target,'L',25)[1]
   p.edge(target,src,sp='L',tp='R',sd=25,td=20,via=[(x+794,ty),(x+794,sy)],ret=True,color=c)
   if s=='A':returned(p,k('engine'),k('commit'),'변경 제안 / 유효성 검사와 확정',x+420,1530,c)
   else:
    p.edge(k('owner'),k('commit'),sp='L',tp='L',sd=0,td=0,via=[(x+16,800),(x+16,1672.5)],label='채택 제안 / 현재 guard',at=(x+65,1540),color=c)
    p.edge(k('commit'),k('owner'),sp='L',tp='L',sd=25,td=25,via=[(x+28,1697.5),(x+28,825)],ret=True,color=c)
   p.text(x+36,1880,['A: 중앙이 의미와 대화 변경을 구성', 'B: actor가 로컬 상태 전이를 제안, Coordinator가 채택', '양안 같은 단일 모델 / 질문 후보 경쟁을 도착 순서로 결정 금지'],20,MUTED)
  elif n==34:
   p.component(k('engine'),x+36,700,720,750,'Request Controller' if s=='A' else 'Incremental Interaction Runtime',color=INK if s=='A' else c)
   p.node(k('in'),x+70,815,652,115,'확정 발화 / 요청 버전' if s=='A' else '부분 발화 / 수정 버전',kind='data',color=c,owner=k('engine'))
   p.node(k('work'),x+70,1050,652,110,'확정 요청의 해석 진행' if s=='A' else '잠정 의미/읽기/응답 준비의 의존망',color=c,owner=k('engine'))
   p.node(k('state'),x+70,1260,652,110,'완성 의미 또는 미해결 제안 (요청 RAM)' if s=='A' else '유효 결과 / 무효화 및 재사용 (임시 RAM)',kind='store',color=c,owner=k('engine'))
   p.edge(k('in'),k('work'),label='확정 후 시작' if s=='A' else '갱신마다 필요한 작업',at=(x+435,975),color=c)
   p.edge(k('work'),k('state'),label='결과 기록',at=(x+435,1190),color=c)
   flow(p,k('input'),k('engine'),'같은 ASR, 연속 수신과 즉시 stop',x+405,590)
   target=model(p,x,k,k('engine'),True);call(p,k('engine'),target,x+765,785,c)
   flow(p,k('engine'),k('commit'),'Request Controller의 현재 확정 경계 통과 후',x+350,1510,c)
   p.text(x+36,1860,['Context Manager의 허용된 읽기도 작업 결과로 연결.', 'Response Manager → Interaction Manager: Text/음성 게시.', 'Interaction Manager → Response Manager: 실제 전달 반환.', 'A도 입력 ASR와 출력 음성 chunk는 streaming 가능.'],20,MUTED)
  elif n==35:
   p.component(k('store'),x+36,700,720,750,'State Store',role='같은 local transaction, 저장 형태와 원본이 다름')
   p.node(k('first'),x+70,820,652,110,'현재 변경 집합 확정' if s=='A' else 'Journal Append / 의미 사건 확정',color=c,owner=k('store'))
   p.node(k('authority'),x+70,1030,652,110,'현재 owner record + 미완료 원장\n내구 원본' if s=='A' else '의미 사건 journal\n내구 원본',kind='store',color=c,owner=k('store'))
   p.node(k('view'),x+70,1260,652,110,'현재 상태 읽기 / schema 변환' if s=='A' else 'Projection / Replay / Checkpoint',color=c,owner=k('store'))
   p.edge(k('first'),k('authority'),label='원자 확정',at=(x+430,970),color=c)
   p.edge(k('authority'),k('view'),label='복원 근거',at=(x+430,1190),color=c)
   flow(p,k('input'),k('store'),'owner의 변경 / 확정 반환',x+430,590,c)
   returned(p,k('store'),k('commit'),'검증된 현재 상태 반환',x+430,1510,c)
   p.text(x+810,790,['A도 감사 이력,', 'inbox/outbox와', 'backup을 사용할 수 있음.', '', 'B replay는 모델 추론,', '외부 명령 전송이나', '음성 재생을 실행하지 않음.'],21,MUTED)
   p.node(k('agent'),x+36,1940,720,120,'외부 Agent (VIA 밖)\n접수 불명은 현재 실행 사실 조회로 확인',kind='model')
   p.edge(k('commit'),k('agent'),sd=-30,td=-30,label='Agent Gateway 경유 조회',at=(x+440,1850))
   p.edge(k('agent'),k('commit'),sp='T',tp='B',sd=30,td=30,ret=True)
  else:
   p.boundary(k('process'),x+36,700,720,800,'Core process의 자료 처리 부분' if s=='A' else '제한 process','직접 권한 보유' if s=='A' else '전역 파일/DB/network 권한 없음',color=c,dashed=True)
   p.component(k('cm'),x+70,790,652,185,'Context Manager',role='자료 조립/cache 부분; 정책 원본은 Core')
   p.component(k('ri'),x+70,1150,652,175,'Request Interpreter',role='허용 근거의 의미 제안')
   p.edge(k('cm'),k('ri'),label='Request Controller 경유: 원문/근거 전달',at=(x+180,1040),color=c)
   p.component(k('broker'),x+808,775,340,210,'Policy Manager' if s=='A' else 'Data Access Broker',color=INK if s=='A' else c,role='현재 정책 확인' if s=='A' else '정책 owner는 Policy Manager')
   p.component(k('ma'),x+808,1160,340,140,'Model Access',role='공유 신뢰 경계에 남음')
   flow(p,k('input'),k('cm'),'유효한 자료 요청 / 근거 반환',x+420,590)
   if s=='A':
    call(p,k('cm'),k('broker'),x+765,760)
    call(p,k('ri'),k('ma'),x+765,1135)
   else:
    call(p,k('cm'),k('broker'),x+765,760,c)
    p.edge(k('ri'),k('broker'),sp='R',tp='L',sd=-25,td=50,via=[(x+774,1212.5),(x+774,930)],label='모델 job도 중개',at=(x+803,1030),color=c)
    p.edge(k('broker'),k('ri'),sp='L',tp='R',sd=80,td=25,via=[(x+790,960),(x+790,1262.5)],ret=True,color=c)
    flow(p,k('broker'),k('ma'),'허가 job / 반환',x+835,1000,c)
   returned(p,k('ri'),k('commit'),'의미 반환 / 현재 근거 검증',x+430,1530)
   p.node(k('dep'),x+36,1940,720,120,'source 자료 (VIA 밖)\nA adapter 직접 read / B Broker 경유 read',kind='model')
   if s=='A':
    p.edge(k('cm'),k('dep'),sp='L',tp='L',sd=-25,td=-25,via=[(x+12,857.5),(x+12,1975)],color=c)
    p.edge(k('dep'),k('cm'),sp='L',tp='L',sd=25,td=25,via=[(x+24,2025),(x+24,907.5)],ret=True,color=c)
   else:
    src=k('broker');sy=p.port(src,'R',-40)[1]
    p.edge(src,k('dep'),sp='R',tp='R',sd=-40,td=-20,via=[(x+1162,sy),(x+1162,1850),(x+770,1850),(x+770,1980)],label='자료 요청',at=(x+820,1830),color=c)
    sy=p.port(src,'R',40)[1]
    p.edge(k('dep'),src,sp='R',tp='R',sd=20,td=40,via=[(x+788,2020),(x+788,1880),(x+1176,1880),(x+1176,sy)],ret=True,color=c)
   p.node(k('omni'),x+808,1940,340,120,'공유 Omni 한 개\n모델 내부는 책임 밖',kind='model')
   p.edge(k('ma'),k('omni'),sd=-25,td=-25,via=[(x+953,1340),(x+796,1340),(x+796,1900),(x+953,1900)])
   p.edge(k('omni'),k('ma'),sp='T',tp='B',sd=25,td=25,via=[(x+1003,1920),(x+802,1920),(x+802,1360),(x+1003,1360)],ret=True)
   p.text(x+840,1410,['Model Access는 같은', '단일 Omni를 요청/반환.', 'Broker/Core/model', '침해 방어는 주장 밖.'],20,MUTED)
 footer(p,'설계안 / 미선정 / 미측정. Component 이름의 반복은 같은 owner의 다른 역할을 설명할 수 있다.')
 return p

EVENTS={
31:[('입력과 두 표/Task 근거 확보','Interaction Manager → Request Controller ↔ Context Manager','같은 원문, 같은 근거'),('의미 생산','Request Interpreter ↔ Model Access: 전체 관계 제안','Request Interpreter ↔ Model Access: 표현/근거 후보'),('관계 구성과 반환','제안 검증 → Request Controller: 의미 또는 미해결','제약 해결 → Request Controller: 해 또는 질문 필드'),('현재 상태 확인과 확정','Request Controller: 자료/Task/권한과 입력 버전 확인','동일. 유일한 해도 사용자 의도의 증명은 아님'),('외부 인계와 결과','Task Manager → Agent Gateway → Agent / 접수와 결과 반환','동일. Response Manager → Interaction Manager: 사실 전달')],
32:[('표를 가리킨 순간','Interaction Manager: 화면/포인터/발화 시점 수집','Context Manager ↔ source: 당시 객체/선택과 버전 획득'),('대상 근거 반환','Context Manager → Request Controller: 당시 관측 묶음','Context Manager → Request Controller: 객체/문서/범위 참조'),('문장의 지칭과 결합','Request Interpreter ↔ Model Access: 영역 후보','Request Interpreter ↔ Model Access: 객체 후보'),('스크롤 뒤 현재 사용 검사','Request Controller ↔ Context Manager: 같은 대상인지 대조','같은 경로: 제공자에 객체/버전 유효성 재확인'),('인계와 접수 확인','Task Manager → Agent Gateway → Agent: 확인 대상과 관측','동일 경로: 확인 객체 참조/내용. Agent 지원 확인 후 인계')],
33:[('두 업무 질문과 실제 게시','Task Manager → Request Controller: 질문 / 실제 게시 기록','Runtime → 각 actor: 같은 질문/실제 게시 사건'),('복합 후속 발화 해석','Request Interpreter ↔ Model Access: 관련 업무 공동 해석','Coordinator → actor ↔ Request Interpreter: 로컬 의미 제안'),('관계와 상태 변경 제안','Request Interpreter → Request Controller: 전체 관계','actor ↔ Coordinator: 담당 절, 결론 참조 요청/반환과 재제안'),('현재 조건 검사 후 적용','Request Controller: 중앙 상태와 명령 intent 확정','같은 guard, Coordinator: actor 예상 버전과 채택 변경 확정'),('외부 적용과 사용자 반환','Task Manager ↔ Agent Gateway ↔ Agent / 실제 상태 전달','동일. actor에는 적용 완료 및 실제 전달 사건 반환')],
34:[('“이 표를 보내…” 부분 입력','Interaction Manager / Speech Input Worker: 연속 수신','동일 입력 → Runtime: 부분 발화 버전'),('발화 중 처리','입력 지속 / 발화 의미와 외부 명령은 확정 대기','Runtime ↔ Request Interpreter/Context Manager: 잠정 해석/읽기'),('“아니, 설명만” 정정','확정 발화 → Request Controller → Request Interpreter','Runtime: 옛 의미/응답 무효화, 유효 읽기 재사용 후 갱신'),('의미와 게시 허용','Request Controller: 최종 의미/현재 권한 확인','동일. Runtime의 유효 결과만 Response Manager에 전달'),('설명 중 “세 번째 열” 끼어들기','Interaction Manager 즉시 stop / 실제 전달 반환 / 새 요청','같은 stop / 해당 fragment의 의존 작업 수정, 옛 chunk 차단')],
35:[('질문 답변을 이해','Request Controller: 질문/Task/현재 권한 검증','동일'),('한 local transaction 확정','owner → State Store: 현재 상태 변경 + 전송 intent','owner → State Store: 사건 append + projection + intent'),('외부 전송과 확인','Agent Gateway ↔ Agent / 확인 사실로 현재 상태 변경','동일 외부 확인을 event로 기록해 현재 view 갱신'),('VIA 재시작','State Store → owner: 현재 record와 미완료 원장','State Store: checkpoint + tail 재생 → owner: 검증된 view'),('불명 접수 확인과 실제 전달','Agent Gateway ↔ Agent: 조회 / Response Manager → 사용자','동일. replay는 외부 전송/음성 재생을 직접 실행하지 않음')],
36:[('선택 문서와 허용 목적 확인','Request Controller ↔ Policy Manager: 유효 범위 확인','동일 + Request Controller → Broker: 자료 범위 준비'),('자료 획득','Context Manager ↔ source adapter: read와 근거 반환','제한 client ↔ Broker ↔ source: handle과 허용 자료 반환'),('의미 해석','Request Interpreter ↔ Model Access: 요청/제안 반환','제한 Request Interpreter ↔ Broker ↔ Model Access'),('현재 제안과 근거 확인','Request Controller: 현재 자료/입력/권한 확인','동일 + Broker의 실제 발급 근거 대조'),('외부 Agent로 인계','Task Manager → Agent Gateway ↔ Agent: command/자료','같은 command + Agent Gateway ↔ Broker: 수신자별 제공')]
}

def event(n):
 p=plate(n,True)
 p.text(64,238,'공통 입력과 끝점은 유지한다. 각 칸은 사건 설명이며 Component 또는 process 배치를 뜻하지 않는다.',23,MUTED)
 for x,s in [(64,'A'),(1312,'B')]:
  c=BLUE if s=='A' else GREEN
  p.text(x,300,s+'  '+NAMES[n][s=='B'],28,c,bold=True)
  for j,(label,a,b) in enumerate(EVENTS[n],1):
   y=375+(j-1)*290
   p.box(x,y,1184,225,'white','#C7D1DD',8)
   p.box(x,y,8,225,c,'none',0)
   p.text(x+28,y+18,str(j)+'. '+label,27,c,bold=True)
   # Explicit request/return arrows remain text in event cards; structure plates draw directed links.
   import textwrap
   value=(a if s=='A' else b).replace(' ↔ ', ' → [요청] / ← [반환] ')
   if n==34:value=value.replace('Runtime','Incremental Interaction Runtime')
   if n==33:value=value.replace('Runtime','Task Dialogue Runtime').replace('Coordinator','Dialogue Coordinator')
   if n==36:value=value.replace('Broker','Data Access Broker')
   lines=textwrap.wrap(value, width=59,break_long_words=False,break_on_hyphens=False)
   p.text(x+28,y+83,lines,23,leading=32)
   if j<5:p.line([(x+592,y+225),(x+592,y+290)],c)
  p.text(x+20,1870,['같은 정상 사건의 흐름. 실패/철회/재시작은 본문 §6.', '표시/생성 완료와 실제 청취, 접수와 업무 완료를 구별한다.', '구조의 유리한 조건과 반증은 본문 §8~9에서 검토한다.'],22,MUTED)
 footer(p,'실제 소요 시간/성능을 표시하지 않는다. 요청과 반환의 송수신 책임은 각 단계와 본문 표에 명시한다.')
 return p

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args();drift=[]
 for n in range(31,37):
  for p in (structural(n),event(n)):
   p.validate()
   for edge in p.items:
    if edge['kind']=='line' and 'source' in edge:
     for role,point in [('source',edge['points'][0]),('target',edge['points'][-1])]:
      x,y,w,h=p.nodes[edge[role]];px,py=point
      assert x<=px<=x+w and y<=py<=y+h,(p.slug,edge[role],'endpoint outside node')
   for ext,data in [('svg',p.svg()),('drawio',p.drawio())]:
    root=ET.fromstring(data)
    if ext=='drawio':
     ids=[e.get('id') for e in root.findall('.//mxCell')];assert len(ids)==len(set(ids))
     for e in root.findall('.//mxCell'):
      for attr in ('source','target','parent'):assert not e.get(attr) or e.get(attr) in ids
    dest=OUT/(p.slug+'.'+ext)
    if args.check:
     if not dest.exists() or dest.read_text()!=data:drift.append(dest.name)
    else:dest.write_text(data)
 if drift:raise SystemExit('Diagram drift: '+', '.join(drift))
 print('PASS: 12 pairs; SVG/draw.io source parity, XML references, ownership, bounds and routes')
if __name__=='__main__':main()
