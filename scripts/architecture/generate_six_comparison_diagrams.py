#!/usr/bin/env python3
"""Presentation comparisons: independent mechanism graphs and named-lifeline traces.
Rectangle content is restricted to component/module names. Behaviors label edges.
"""
import argparse
from pathlib import Path
import xml.etree.ElementTree as ET
from stage4_diagram_design import Plate, INK, MUTED, BLUE, GREEN
ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'docs/architecture/12-decisions/decision-packages/diagrams'
TITLES={31:'요청 이해의 정확성을 위한 의미 구성 — 요청별 생성과 지속 의미 작업공간',32:'화면 지칭의 정확성을 위한 대상 결합 — 통합 해석과 분리 생산 후 결합',33:'여러 업무의 대화 정확성을 위한 협력 — 중앙 해석과 업무별 대화 처리',34:'음성 대화의 반응성과 정정 정확성을 위한 실행 — 발화 단위와 연속 수정',35:'대화와 업무의 복구 정확성을 위한 기록 — 현재 상태와 사건 이력',36:'허용된 자료만 처리하기 위한 권한 배치 — 공유 처리와 제한 처리'}
NAMES={31:('매번 완성 의미를 생성','의미 상태에 변경을 적용'),32:('말과 화면을 함께 해석','두 근거를 생산한 뒤 결합'),33:('중앙이 대화 상태를 작성','업무별 처리자가 상태를 제안'),34:('확정 발화부터 처리','잠정 작업을 수정하며 처리'),35:('현재 값을 원본으로 저장','사건을 원본으로 상태 계산'),36:('공유 권한 안의 정책 검사','권한 없는 처리와 Broker')}
CASES={31:'“작년 표는 보고서에, 메일은 초안만” → “오른쪽 표”로 보완 → 발송 금지는 유지',32:'화면 7을 가리키며 “이 표와” → 스크롤 → 화면 8의 “저 표를 비교” → 첫 대상만 정정',33:'보고서와 메일이 각각 질문 → “보고서는 PDF, 메일에는 그 결론” → 다른 업무의 참조 연결',34:'“이 표를 보내…” → “아니, 설명만 해줘” → 준비는 재사용할 수 있지만 보내기는 실행 금지',35:'질문 답변을 확정 → Agent에 전송 → 접수 확인 전에 VIA 종료 → 중복 없이 이어가기',36:'허용된 보고서를 요약하는 처리 코드에 결함 → 무관한 파일 접근을 어디에서 막는가?'}
BENEFITS={
31:(["유리: 자유로운 새 의도를 전체 맥락에서 다시 구성","대가: 앞선 조건 누락을 검증해야 함; 관련 근거 재해석","선택: 짧거나 표현 범위가 넓은 대화"],["유리: 명시한 조건을 유지하며 필요한 관계만 갱신","대가: 변경 언어/의존성 설계; 표현 밖 뜻은 질문","선택: 여러 번 보완하고 조건을 이어가는 대화"]),
32:(["유리: 복잡한 시각 표현과 말의 맥락을 함께 활용","대가: 시간/대상 오결합 가능; 관련 장면 재해석","선택: 낯선 화면과 자유로운 표현이 중요할 때"],["유리: 시간/위치 결합 근거와 정정 범위를 명시","대가: 두 생산의 누락/지연; 표현 밖 영역은 재지칭","선택: 순차 지칭과 반복 정정이 많을 때"]),
33:(["유리: 업무 간 생략/교차 참조를 한 번에 해석","대가: 중앙이 모든 대화 전이 규칙을 관리","선택: 짧고 서로 얽힌 업무 대화"],["유리: 업무별 대화 규칙/사건 처리를 해당 actor가 소유","대가: 참가 누락/제안 충돌/참조 왕복/모델 경합","선택: 독립적으로 오래 이어지는 여러 업무"]),
34:(["유리: 최종 발화만 처리해 추론 낭비를 줄임","대가: 발화 종료 뒤 읽기/해석/응답 준비 대기","선택: 짧은 발화나 추론 자원이 부족한 환경"],["유리: 발화 중 준비; 정정 뒤 유효한 읽기 재사용","대가: 취소 낭비/버전 누락/공유 모델 backlog","선택: 긴 발화에서 재사용 가능한 준비가 많을 때"]),
35:(["유리: 현재 record를 직접 읽어 단순하게 복구","대가: 새 조회/원인 분석은 보존된 기록에 제한","선택: 현재 상태 조회와 짧은 재시작이 중심"],["유리: 같은 사건으로 상태 재구성과 원인 추적","대가: 사건 schema/삭제/재생/검증 부담","선택: 과거 의미 전이의 재구성이 실제로 필요"]),
36:(["유리: 직접 호출/공유 자료로 복사와 IPC를 줄임","대가: 처리 코드 결함이 넓은 process 권한에 도달","선택: 작은 신뢰 코드와 자원 제약을 우선"],["유리: 처리 코드가 잘못돼도 직접 자료 접근 차단","대가: IPC/복사/handle 수명; Broker는 여전히 신뢰","선택: 침해 범위를 줄이는 것이 중요한 환경"])}

class D:
 def __init__(self,p,n,s):
  self.p,self.n,self.s,self.x,self.c=p,n,s,64 if s=='A' else 1312,BLUE if s=='A' else GREEN
  p.text(self.x,285,s+'  '+NAMES[n][s=='B'],30,self.c,bold=True)
 def cpt(self,k,x,y,w,h,name,color=None):
  self.p.component(self.s+k,self.x+x,y,w,h,name,color=color or INK)
 def mod(self,k,x,y,w,h,name,owner):
  self.p.node(self.s+k,self.x+x,y,w,h,name,owner=self.s+owner,color=self.c)
 def store(self,k,x,y,w,h,name,owner=None):
  self.p.node(self.s+k,self.x+x,y,w,h,name,kind='store',owner=self.s+owner if owner else None,color=self.c)
 def ext(self,k,x,y,w,h,name):
  self.p.node(self.s+k,self.x+x,y,w,h,name,kind='model')
 def txt(self,x,y,s,size=21,color=MUTED):self.p.text(self.x+x,y,s,size,color)
 def e(self,a,b,label,sp='B',tp='T',via=(),at=None,ret=False,sd=0,td=0,color=None):
  self.p.edge(self.s+a,self.s+b,sp=sp,tp=tp,via=[(self.x+x,y) for x,y in via],label=label,at=(self.x+at[0],at[1]) if at else None,ret=ret,sd=sd,td=td,color=color or self.c)
 def model(self,x,y):
  self.cpt('ma',x,y,350,85,'Model Access');self.ext('omni',x,y+235,350,90,'공유 Omni')
  self.e('ma','omni','추론 입력',sd=-35,td=-35,at=(x+5,y+120),color=INK)
  self.e('omni','ma','결과',sp='T',tp='B',sd=35,td=35,at=(x+205,y+165),ret=True,color=INK)
 def call(self,a,at=None):
  sy=self.p.port(self.s+a,'R',-15)[1];ty=self.p.port(self.s+'ma','L',-15)[1];lane=745
  self.e(a,'ma','모델 입력',sp='R',tp='L',sd=-15,td=-15,via=[(lane,sy),(lane,ty)],at=at,color=INK)
  sy=self.p.port(self.s+a,'R',15)[1];ty=self.p.port(self.s+'ma','L',15)[1]
  self.e('ma',a,'모델 결과',sp='L',tp='R',sd=15,td=15,via=[(lane+20,ty),(lane+20,sy)],at=(at[0],at[1]+45) if at else None,ret=True,color=INK)

def semantic(d):
 d.cpt('rc',30,410,650,90,'Request Controller')
 d.txt(30,350,'같은 새 발화, 이전 질문, 허용된 자료와 Task 사실')
 if d.s=='A':
  d.cpt('ri',30,700,650,690,'Request Interpreter')
  d.mod('gen',80,840,550,100,'전체 의미 생성기','ri')
  d.mod('check',80,1150,550,100,'대상과 조건 검증기','ri')
  d.e('rc','gen','1  원문 + 관련 대화/자료/조건',at=(60,580))
  d.e('gen','check','2  목표/대상/금지 조건의 완성안',at=(90,1020))
  d.e('check','gen','불일치: 근거를 모아 재해석',sp='L',tp='L',via=[(50,1200),(50,890)],at=(85,1300))
  d.model(810,790);d.call('gen',at=(660,860))
  d.cpt('out',30,1580,650,90,'Request Controller')
  d.e('check','out','3  검증한 의미 또는 확인할 질문',ret=True,at=(65,1460))
  d.txt(800,1220,['의미 원본: 채택한 완성안','보완/정정 때 새 완성안 생산','검증/부분 재시도도 가능'],21)
 else:
  d.cpt('ri',30,680,650,90,'Request Interpreter');d.model(810,620);d.call('ri',at=(660,625))
  d.e('rc','ri','1  원문 + 현재 의미/질문 요약',at=(70,565))
  d.cpt('ws',30,960,650,530,'Meaning Workspace',d.c)
  d.mod('apply',80,1080,550,100,'의미 변경 적용기','ws')
  d.store('state',80,1340,550,90,'채택 의미와 질문','ws')
  d.cpt('engine',810,1080,350,100,'Semantic Constraint Engine',d.c)
  d.e('ri','apply','2  목표 추가 / 후보 제한 / 조건 교체',at=(65,845))
  d.e('apply','state','채택한 조건 유지; 변경만 적용',at=(90,1230),sd=-20,td=-20)
  d.e('state','apply','이전 버전',sp='T',tp='B',sd=20,td=20,ret=True,at=(410,1290))
  d.e('apply','engine','3  후보/제약',sp='R',tp='L',sd=-20,td=-20,at=(655,1070))
  d.e('engine','apply','가능 관계/미해결',sp='L',tp='R',sd=20,td=20,ret=True,at=(650,1185))
  d.cpt('out',30,1670,650,90,'Request Controller')
  d.e('ws','out','4  현재 의미 또는 미해결 질문',ret=True,at=(65,1540))
  d.cpt('cm',810,1430,350,90,'Context Manager')
  d.cpt('tm',810,1710,350,90,'Task Manager')
  d.e('cm','ws','자료 변경/현재 근거',sp='L',tp='R',td=40,via=[(730,1475),(730,1265)],at=(805,1545))
  d.e('tm','ws','Task 변경/현재 사실',sp='L',tp='R',td=95,via=[(755,1755),(755,1320)],at=(800,1830))
  d.txt(30,1810,'채택 의미는 State Store에 저장; 질문은 Response Manager가 게시')

def grounding(d):
 d.cpt('cm',30,410,1130,90,'Context Manager')
 d.txt(30,350,'Interaction Manager가 기록한 화면/포인터/발화 시점')
 if d.s=='A':
  d.cpt('joint',30,750,650,750,'Joint Grounding Interpreter',d.c)
  d.mod('gen',80,900,550,100,'시각과 언어 공동 해석기','joint')
  d.mod('check',80,1280,550,100,'관측 대상 검증기','joint')
  d.e('cm','gen','1  원문 + 화면 7/8 + 포인터/시점',via=[(595,620),(355,620)],at=(75,575))
  d.model(810,850);d.call('gen',at=(660,885))
  d.e('gen','check','2  d1=L@7, d2=R@8 완성 대응',at=(95,1110))
  d.cpt('out',30,1700,650,90,'Request Controller')
  d.e('check','out','3  근거 검사 후 대응 관계 반환',ret=True,at=(60,1560))
  d.txt(800,1280,['지칭 관계는 모델이 생성','틀린 ID/시점은 코드로 검사','정정: 관련 장면 다시 해석'],21)
 else:
  d.cpt('coord',30,670,1130,90,'Grounding Coordinator',d.c)
  d.cpt('scene',30,970,510,100,'Scene Candidate Producer',d.c)
  d.cpt('anchor',650,970,510,100,'Deictic Anchor Producer',d.c)
  d.cpt('join',300,1370,600,100,'Grounding Join Engine',d.c)
  d.cpt('out',300,1700,600,90,'Request Controller')
  d.e('cm','coord','1  같은 범위의 근거와 수정 버전',at=(65,555))
  d.e('coord','scene','2  화면/포인터/시점',sd=-310,at=(45,830))
  d.e('coord','anchor','2  발화/시점/이전 참조',sd=310,at=(670,830))
  d.e('scene','join','3  Coordinator 경유: 화면 후보',td=-180,via=[(285,1220),(420,1220)],at=(50,1140),ret=True)
  d.e('anchor','join','3  Coordinator 경유: 지시 조건',td=180,via=[(905,1220),(780,1220)],at=(800,1235),ret=True)
  d.e('join','out','4  Coordinator 경유: 결합 결과/버전\n없음/복수/표현 밖 → 해당 지시 질문',at=(325,1510),ret=True)
  d.e('join','coord','미해결: 질문 또는 필요한 근거 재생산',sp='L',tp='L',via=[(8,1420),(8,715)],at=(35,1280))
  d.cpt('ma',425,850,350,75,'Model Access')
  d.ext('omni',425,1110,350,90,'공유 Omni')
  d.e('scene','ma','추론 요청',sp='T',tp='L',td=-12,via=[(285,875.5)],at=(295,925),color=INK)
  d.e('ma','scene','후보 결과',sp='L',tp='T',sd=12,td=20,via=[(305,899.5)],at=(50,910),ret=True,color=INK)
  d.e('anchor','ma','추론 요청',sp='T',tp='R',td=-12,via=[(905,875.5)],at=(785,925),color=INK)
  d.e('ma','anchor','조건 결과',sp='R',tp='T',sd=12,td=20,via=[(925,899.5)],at=(965,910),ret=True,color=INK)
  d.e('ma','omni','추론',sd=-20,td=-20,at=(550,1010),color=INK)
  d.e('omni','ma','결과',sp='T',tp='B',sd=20,td=20,at=(605,1050),ret=True,color=INK)
  d.txt(30,1850,['Coordinator가 두 결과의 버전 확인; Join Engine은 조건 결합만 수행', '원본 언어/시각 해석을 Join Engine으로 옮기거나 모델을 복제하지 않음'],20)

def dialogue(d):
 d.cpt('rc',30,410,1130,90,'Interaction Manager');d.txt(30,350,'같은 발화 + 보고서/메일의 질문과 실제 전달 기록')
 if d.s=='A':
  d.cpt('owner',30,710,650,730,'Request Controller')
  d.store('state',80,840,550,90,'공동 대화 상태','owner')
  d.mod('transition',80,1200,550,100,'대화 전이 작성기','owner')
  d.cpt('ri',810,840,350,90,'Request Interpreter')
  d.e('rc','state','1  관련 질문/업무 근거 회수',sd=-240,at=(65,580))
  d.e('state','ri','2  전체 발화/맥락',sp='R',tp='L',sd=-10,td=-10,at=(650,800))
  d.e('ri','transition','3  두 업무의 의미 제안',sp='B',tp='R',via=[(985,1070),(730,1070),(730,1250)],at=(795,1090),ret=True)
  d.e('transition','state','4  공동 상태 갱신',sp='L',tp='L',via=[(50,1250),(50,885)],at=(90,1010))
  d.cpt('out',30,1700,650,90,'Request Controller');d.e('transition','out','5  명령/질문 제안',at=(65,1530),ret=True)
  d.txt(790,1360,['전이 작성자는 중앙 하나','Task별 cache/선택 갱신 가능','모든 Task를 직렬 실행하지 않음'],20)
  d.txt(30,1810,'추론: Request Interpreter → Model Access → 같은 Omni → 의미 결과 반환')
  # Owner is a module in the controller, shown as a focused component compartment.
 else:
  d.cpt('coord',30,710,1130,90,'Dialogue Coordinator',d.c)
  d.cpt('runtime',30,1050,1130,580,'Task Dialogue Runtime',d.c)
  d.mod('r',75,1180,470,100,'보고서 대화 actor','runtime');d.mod('m',650,1180,470,100,'메일 대화 actor','runtime')
  d.store('rs',75,1490,470,90,'보고서 대화 상태','runtime');d.store('ms',650,1490,470,90,'메일 대화 상태','runtime')
  d.e('rc','coord','1  입력 버전/참가 후보; 현재 조건 검사',at=(65,565))
  for k,offset,x in [('r',-285,75),('m',290,660)]:
   d.e('coord',k,'2  원문/허용 참조',sd=offset-20,td=-20,at=(x,865))
   d.e(k,'coord','3  절/다음 상태/필요 참조',sp='T',tp='B',sd=20,td=offset+20,ret=True,at=(x,950))
   d.e(k,k+'s','5  채택 통지 후 상태 반영',sd=-20,td=-20,at=(x,1360))
   d.e(k+'s',k,'이전 상태',sp='T',tp='B',sd=20,td=20,ret=True,at=(x+230,1420))
  d.txt(40,1660,['4  Coordinator가 참조 교환/재제안을 중개 → Request Controller 현재 검사', '5  State Store가 채택된 변경/명령을 함께 확정 → 각 actor에 통지'],20)
  d.cpt('ri',350,1830,500,90,'Request Interpreter')
  d.e('r','ri','자기 맥락으로 해석 요청',sp='L',tp='L',sd=-15,td=-15,via=[(8,1215),(8,1860)],at=(35,1770),color=INK)
  d.e('ri','r','자기 의미 결과',sp='L',tp='L',sd=15,td=15,via=[(20,1890),(20,1245)],at=(35,1930),ret=True,color=INK)
  d.e('m','ri','자기 맥락으로 해석 요청',sp='R',tp='R',sd=-15,td=-15,via=[(1170,1215),(1170,1860)],at=(855,1770),color=INK)
  d.e('ri','m','자기 의미 결과',sp='R',tp='R',sd=15,td=15,via=[(1182,1890),(1182,1245)],at=(860,1930),ret=True,color=INK)
  d.txt(40,2000,'추론은 Model Access/공유 Omni. Coordinator는 actor의 다음 상태를 다시 작성하지 않음.',19)

def voice(d):
 d.cpt('im',30,410,1130,90,'Interaction Manager');d.txt(30,350,'공통: Streaming ASR 연속 수신; 음성 끼어들기는 먼저 재생 중단')
 if d.s=='A':
  d.cpt('rc',30,780,600,90,'Request Controller');d.cpt('cm',800,780,360,90,'Context Manager')
  d.cpt('ri',30,1140,600,90,'Request Interpreter');d.cpt('rm',30,1550,600,90,'Response Manager')
  d.e('im','rc','1  발화가 확정되면 전체 입력 전달',sd=-265,at=(65,625))
  d.e('rc','cm','2  근거 요청',sp='R',tp='L',sd=-15,td=-15,at=(640,750))
  d.e('cm','rc','근거/버전 반환',sp='L',tp='R',sd=15,td=15,ret=True,at=(640,880))
  d.e('rc','ri','3  확정 발화/근거로 의미 요청',at=(65,980),sd=-20,td=-20)
  d.e('ri','rc','의미 반환',sp='T',tp='B',sd=20,td=20,ret=True,at=(385,1050))
  d.e('rc','rm','4  현재 검사 후 응답 허용',sp='L',tp='L',via=[(8,825),(8,1595)],at=(65,1410))
  d.txt(800,1130,['부분 전사는 수신만 유지','발화 확정 뒤 의미 처리','후속 정정은 새 요청으로'],21)
  d.txt(30,1770,['Request Interpreter/Response Manager → Model Access → 공유 Omni', '5  Response Manager → Interaction Manager: 음성/Text; 실제 전달 구간 반환'],20)
 else:
  d.cpt('run',30,720,640,810,'Incremental Interaction Runtime',d.c)
  d.mod('scheduler',80,900,540,100,'잠정 작업 배정기','run');d.store('deps',80,1310,540,100,'작업과 근거의 의존 관계','run')
  d.cpt('ri',810,810,350,90,'Request Interpreter');d.cpt('cm',810,1150,350,90,'Context Manager');d.cpt('rm',810,1490,350,90,'Response Manager')
  d.cpt('rc',30,1760,640,90,'Request Controller')
  d.e('im','scheduler','1  부분 발화 + 수정 버전',via=[(595,600),(350,600)],at=(65,595))
  for k,lane,dy,lab,y in [('ri',710,-25,'2  잠정 의미',840),('cm',735,0,'2  허용 읽기',1130),('rm',760,25,'2  응답 준비',1450)]:
   sy=950+dy;ty=d.p.port(d.s+k,'L',-15)[1]
   d.e('scheduler',k,lab,sp='R',tp='L',sd=dy,td=-15,via=[(lane,sy),(lane,ty)],at=(800,y-85))
   d.e(k,'deps','결과/버전',sp='L',tp='R',sd=15,td=dy,via=[(lane+10,ty+30),(lane+10,1360+dy)],ret=True,at=(810,y+150))
  d.e('deps','scheduler','3  정정: 관련 결과 폐기/재배정',sp='L',tp='L',via=[(50,1360),(50,950)],at=(90,1130))
  d.e('deps','rc','4  유효 결과 + 확정 입력',via=[(350,1640)],at=(65,1620),ret=True)
  d.e('rc','rm','5  현재 검사 뒤 게시 허용',sp='R',tp='B',via=[(985,1805)],at=(720,1680))
  d.txt(30,1890,'추론은 같은 Model Access/Omni; 실제 출력은 Interaction Manager, 외부 실행은 별도 승인',19)

def state(d):
 d.cpt('rc',30,410,1130,90,'Request Controller');d.txt(30,350,'질문 답변의 현재 버전/권한 확인; Task/전달 변경도 각 owner가 생성')
 d.cpt('ss',30,760,1130,880,'State Store')
 if d.s=='A':
  d.mod('write',80,880,530,100,'Record Update Module','ss');d.store('record',80,1220,530,120,'현재 상태와 미완료 원장','ss')
  d.mod('restore',750,1220,350,120,'Record Load Module','ss')
  d.e('rc','write','1  상태 변경 + 명령 intent',via=[(595,625),(345,625)],at=(65,590))
  d.e('write','record','2  상태/inbox/outbox를 한 transaction에 기록',at=(95,1070))
  d.e('record','restore','3  재시작: 현재 record',sp='R',tp='L',at=(635,1180))
  d.txt(30,1970,'감사 log/backup도 가능. 정상 복구 원본은 현재 record.',21)
 else:
  d.mod('append',80,880,490,100,'Journal Append Module','ss');d.store('journal',80,1180,490,100,'확정 사건 원장','ss')
  d.mod('reduce',710,1180,390,100,'Projection Module','ss');d.store('view',710,1440,390,100,'조회 상태와 사건 위치','ss')
  d.e('rc','append','1  의미 사건 + 예상 버전 + intent',via=[(595,625),(325,625)],at=(65,590))
  d.e('append','journal','2  append',at=(90,1050))
  d.e('journal','reduce','사건 적용: 결정적 코드로 계산',sp='R',tp='L',at=(580,1120))
  d.e('reduce','view','같은 transaction에 상태/위치/intent 확정',at=(625,1320))
  d.mod('checkpoint',80,1450,490,90,'Checkpoint Module','ss')
  d.e('checkpoint','reduce','3  재시작: 검증한 중간 상태 + 후속 사건',sp='R',tp='L',td=30,via=[(640,1495),(640,1260)],at=(85,1330))
  d.e('view','checkpoint','검증한 상태/사건 위치 저장',sp='B',tp='B',via=[(905,1590),(325,1590)],at=(110,1570))
  d.txt(30,1970,'재생은 상태만 계산. 외부 명령/음성은 재실행하지 않음.',21)
 d.cpt('ag',30,1820,530,90,'Agent Gateway');d.ext('agent',780,1820,380,90,'Downstream Agent')
 source='restore' if d.s=='A' else 'view'
 d.e(source,'ag','4  복원된 미완료 intent; 현재 권한 재검사',via=[(925 if d.s=='A' else 905,1710),(295,1710)],at=(70,1680),ret=True)
 d.e('ag','agent','5  접수 상태 조회',sp='R',tp='L',sd=-15,td=-15,at=(580,1780),color=INK)
 d.e('agent','ag','접수/결과/불명',sp='L',tp='R',sd=15,td=15,ret=True,at=(580,1900),color=INK)

def isolation(d):
 d.cpt('policy',30,410,500,90,'Policy Manager');d.cpt('rc',660,410,500,90,'Request Controller')
 d.txt(30,350,'같은 허용 자료/모델; A도 정책 검사와 기존 connector worker 격리를 유지')
 if d.s=='A':
  d.p.boundary(d.s+'process',d.x+30,730,1130,720,'공유 권한 process',subtitle='',dashed=True)
  d.cpt('cm',80,880,500,550,'Context Manager');d.cpt('ri',660,880,450,90,'Request Interpreter')
  d.mod('assemble',130,1000,400,90,'자료 조립기','cm')
  d.mod('adapter',130,1280,400,90,'자료 접근 Adapter','cm');d.model(780,1240)
  d.e('policy','assemble','1  허용 범위 확인',via=[(280,630),(330,630)],at=(65,585))
  d.e('rc','ri','1  허용된 처리 요청',via=[(910,630),(885,630)],at=(675,585))
  d.e('assemble','ri','2  조립한 자료',sp='R',tp='L',sd=-15,td=-15,via=[(610,1030),(610,910)],at=(590,805))
  d.e('ri','assemble','추가 읽기 요청',sp='L',tp='R',sd=15,td=15,via=[(630,940),(630,1060)],at=(590,1110))
  d.e('assemble','adapter','3  직접 읽기',at=(145,1150),sd=-20,td=-20)
  d.e('adapter','assemble','자료 반환',sp='T',tp='B',sd=20,td=20,ret=True,at=(370,1220))
  d.e('ri','ma','4  추론 요청',via=[(865,1130),(935,1130)],sd=-20,td=-20,at=(780,1070))
  d.e('ma','ri','결과',sp='T',tp='B',via=[(975,1160),(905,1160)],sd=20,td=20,ret=True,at=(995,1160))
  d.ext('files',80,1820,500,90,'OS와 자료 source')
  d.e('adapter','files','process 보유 권한으로 접근',sd=-20,td=-20,at=(100,1690))
  d.e('files','adapter','내용',sp='T',tp='B',sd=20,td=20,ret=True,at=(385,1760))
 else:
  d.p.boundary(d.s+'restricted',d.x+30,720,540,670,'제한 process',subtitle='',dashed=True)
  d.p.boundary(d.s+'trusted',d.x+650,720,510,840,'신뢰 Core process',subtitle='',dashed=True)
  d.cpt('ri',65,850,470,90,'Request Interpreter');d.cpt('cm',65,1210,470,90,'Context Manager')
  d.cpt('broker',690,1020,430,100,'Data Access Broker',d.c);d.cpt('ma',690,1410,430,90,'Model Access')
  d.ext('omni',690,1600,430,90,'공유 Omni');d.ext('files',690,1880,430,90,'OS와 자료 source')
  d.e('rc','ri','1  입력 + 범위 handle',sp='B',tp='T',via=[(910,580),(300,580)],at=(310,535))
  d.e('policy','broker','허용 범위/철회',sp='R',tp='T',via=[(605,455),(605,650),(905,650)],at=(680,670))
  d.e('ri','cm','2  필요한 자료 요청',at=(85,1030),sd=-20,td=-20)
  d.e('cm','ri','허용된 자료',sp='T',tp='B',sd=20,td=20,ret=True,at=(340,1110))
  d.e('cm','broker','3  IPC: handle/범위',sp='R',tp='L',sd=-20,td=-20,via=[(600,1235),(600,1050)],at=(570,1140))
  d.e('broker','cm','허용된 내용만 반환',sp='L',tp='R',sd=20,td=20,via=[(625,1090),(625,1275)],ret=True,at=(570,1320))
  d.e('ri','broker','4  IPC: 허용 자료로 추론 요청',sp='R',tp='T',via=[(605,895),(605,980),(905,980)],at=(585,925))
  d.e('broker','ri','추론 결과/근거 반환',sp='L',tp='R',sd=-35,td=-30,via=[(635,1035),(635,865)],at=(660,815),ret=True)
  d.e('broker','ma','중개한 추론 요청',at=(695,1230),sd=-20,td=-20)
  d.e('ma','broker','추론 결과',sp='T',tp='B',sd=20,td=20,ret=True,at=(945,1340))
  d.e('ma','omni','추론',at=(700,1540),sd=-20,td=-20,color=INK)
  d.e('omni','ma','결과',sp='T',tp='B',sd=20,td=20,ret=True,at=(970,1540),color=INK)
  d.e('broker','files','허용/철회 재검사 뒤 source 읽기',sp='R',tp='R',via=[(1180,1070),(1180,1925)],at=(685,1790))
  d.e('files','broker','읽은 자료/버전 반환',sp='L',tp='B',via=[(675,1925),(675,1770),(1145,1770),(1145,1165),(905,1165)],ret=True,at=(690,1730))
  d.txt(30,1490,['제한 process에는 파일/네트워크/DB 자격 없음','모델도 직접 호출 불가; 요청은 Broker 경유','철회: handle 차단 + 늦은 결과 폐기'],19)
 d.txt(30,2000,'5  의미 제안은 Request Controller가 현재 검사; 외부 실행 승인 책임은 그대로',20)
 d.txt(30,2050,'상단 정책/요청 Component도 신뢰 Core 소속. 모델 런타임은 공통 의존성으로 표시.',19)

def structural(n):
 p=Plate(f'choice{n}-structure',f'04-{n}',TITLES[n],CASES[n],height=2520)
 for s in 'AB':
  d=D(p,n,s);[semantic,grounding,dialogue,voice,state,isolation][n-31](d)
  y=2100;p.line([(d.x,y),(d.x+1184,y)],'#C7D1DD',arrow=False)
  p.text(d.x+15,y+25,BENEFITS[n][s=='B'],23,MUTED,leading=39)
 p.text(64,2270,'공통 마무리: Request Controller 현재 검사 → Task Manager → Agent Gateway → Downstream Agent',22)
 p.text(64,2310,'접수/결과 반환 → Response Manager → Interaction Manager의 실제 표시/음성. 미해결이면 실행 대신 질문.',22)
 p.text(64,2370,'검정: 공통 / 파랑: A 고유 / 초록: B 고유. 실선: 요청/전달 / 점선: 결과. 네모: 이름 / 원통: 상태 / 육각형: 외부 의존.',19,MUTED)
 p.text(64,2410,'같은 이름의 반복 상자는 동일 Component. 비교 부분의 책임을 펼친 그림. 별도 process 표기가 없으면 논리 분해. 모델 가중치는 한 벌; 이익은 조건부 설계 가설, 미측정.',19,MUTED)
 return p

# One row = one visible message. Cases share outcomes, not an artificial common graph.
TRACES={
31:{'A':(['Interaction Manager','Request Controller','Request Interpreter','Model Access','Response Manager'],[
(0,1,'1  “작년 표는 보고서에, 메일은 초안만”',False),(1,2,'관련 질문/Task/자료와 원문으로 전체 의미 요청',False),(2,3,'2  목표/대상/조건의 완성 의미 생성 요청',False),(3,2,'P1: 대상 미해결, send=false',True),(2,1,'검증된 P1과 미해결 대상 반환',True),(1,4,'3  “어느 표인가요?” 게시 요청',False),(4,0,'질문 표시/음성 전달',False),(0,1,'실제 게시 구간 반환',True),(0,1,'실제 질문을 본 뒤 “오른쪽 표”',False),(1,2,'질문 + 보완 + 이전 조건으로 재해석 요청',False),(2,1,'새 완성안 P2: source=R@7, send=false',True),(1,2,'4  자료가 바뀌면 현재 근거로 재검증/재해석',False),(2,1,'현재 조건을 만족하는 의미 반환',True),(1,1,'5  현재 검사 후 명령 채택; 발송 명령은 만들지 않음',False)]),
'B':(['Request Controller','Request Interpreter','Meaning Workspace','Semantic Constraint Engine','Response Manager'],[
(0,1,'1  같은 원문/근거 + 현재 의미/질문 요약',False),(1,2,'목표 추가, 표 후보 L/R, 메일 발송 금지 후보',False),(2,3,'2  현재 후보/제약/근거 버전으로 관계 계산 요청',False),(3,2,'표 대상 미해결; 발송 금지 유지',True),(2,4,'3  대상 필드 질문 게시 요청',False),(4,2,'Interaction Manager의 실제 게시 기록 반환',True),(0,1,'실제 질문을 본 뒤 같은 “오른쪽 표”',False),(1,2,'NARROW(q1, R@7, expected=12)',False),(2,3,'대상 후보만 제한; send=false는 변경하지 않음',False),(3,2,'관계 해 또는 충돌/미해결 반환',True),(2,2,'4  자료 변경: 의존 후보 무효화/재조회/재평가',False),(2,0,'현재 의미와 근거 버전 반환',True),(0,0,'5  현재 검사 후 명령 채택; 외부 승인과 의미 해는 별개',False)])},
32:{'A':(['Interaction Manager','Context Manager','Joint Grounding Interpreter','Model Access','Request Controller'],[
(0,1,'1  첫 지시: 화면 7, 포인터 p1, 시점 t1',False),(0,1,'2  스크롤 뒤 둘째 지시: 화면 8, p2, t2',False),(1,2,'같은 원문/관측 묶음; Request Controller가 해석 요청',False),(2,3,'3  말 + 화면 + 포인터/시점의 공동 해석',False),(3,2,'완성 대응 d1=L@7, d2=R@8 제안',True),(2,2,'관측 ID/창/시점 검증; 모호하면 재해석/질문',False),(2,4,'검증된 두 대상과 비교 관계 반환',True),(0,4,'4  “첫 표는 왼쪽 말고 오른쪽”',False),(4,2,'해당 관측과 정정 원문으로 대응 재해석',False),(2,4,'새 첫 참조 + 유지할 둘째 참조 반환',True),(4,4,'5  입력/자료/권한 현재 검사 후 인계',False)]),
'B':(['Grounding Coordinator','Scene Candidate Producer','Deictic Anchor Producer','Grounding Join Engine','Request Controller'],[
(0,0,'1  화면 7/p1/t1 기록을 Context Manager에서 수신',False),(0,0,'2  화면 8/p2/t2 및 최종 발화와 같은 범위로 고정',False),(0,1,'화면/포인터/시점 → 후보 생성 요청',False),(0,2,'발화/시점/이전 참조 → 지시 조건 생성 요청',False),(1,0,'3  L@7, R@8 등의 영역/종류/시점 후보',True),(2,0,'d1:t1의 표, d2:t2의 표, 관계=비교',True),(0,3,'호환되는 두 버전 확인 후 조건 결합 요청',False),(3,0,'d1=L@7, d2=R@8 또는 미해결',True),(0,2,'4  첫 표 정정: 언어 조건 수정 요청',False),(2,0,'수정한 첫 지시 조건 + 새 버전 반환',True),(0,3,'화면 후보가 충분하면 재사용; 없으면 재생산',False),(3,0,'새 결합 결과와 사용한 두 근거 버전 반환',True),(0,4,'5  새 결합 결과/버전 반환 → 현재 검사 후 인계',True)])},
33:{'A':(['Task Manager','Request Controller','Request Interpreter','State Store','Agent Gateway'],[
(0,1,'1  보고서 qR=형식?, 메일 qM=본문?; 실제 게시 연결',False),(1,2,'2  “보고서는 PDF, 메일에는 그 결론” 전체 해석 요청',False),(2,1,'qR=PDF; qM.content=R.result@4 제안',True),(1,1,'3  두 절의 관계/질문 버전/결론 참조 확인',False),(1,3,'4  현재 검사 후 두 변경 + 전송 intent를 원자 저장',False),(3,1,'채택 결과 반환',True),(1,4,'5  채택된 각 업무의 답변 전달',False),(4,0,'접수/결과 반환; 사용자 전달은 공통 응답 경로',True),(0,1,'별도 변형: 위 흐름의 채택 전에 qM 취소가 도착',False),(1,1,'옛 qM 답 거절; 독립 qR은 진행, 결합 조건이면 함께 보류',False)]),
'B':(['Dialogue Coordinator','보고서 대화 actor','메일 대화 actor','Request Controller','State Store'],[
(0,1,'1  같은 원문/버전 + 보고서의 허용 맥락',False),(0,2,'2  같은 원문/버전 + 메일의 허용 맥락',False),(1,0,'qR=PDF 변경 제안; 확정 결론 R.result@4 내보냄',True),(2,0,'내가 담당할 메일 절; 보고서 결론 참조 필요',True),(0,2,'3  허용된 결론 참조 전달 → actor가 본문 변경 재제안',False),(2,0,'qM.content=R.result@4 + 예상 버전',True),(0,3,'4  원문 절/관계/현재 질문/Task/권한 검사 요청',False),(3,0,'채택 허용 또는 변경된 의존 집합 거절',True),(0,4,'5  actor가 제안한 상태/intent를 원자 저장',False),(4,0,'채택된 버전/명령 intent 반환',True),(0,1,'저장 성공 후 채택 통지; 메일 actor에도 통지',False),(0,2,'별도 변형: 위 흐름의 채택 전 qM 취소 → 옛 제안 거절',False)])},
34:{'A':(['Interaction Manager','Request Controller','Context Manager','Request Interpreter','Response Manager'],[
(0,0,'1  “이 표를 보내…” 연속 수신; 아직 발화 미확정',False),(0,0,'2  의미 처리는 기다리고 ASR 수신/재생 중단은 계속',False),(0,1,'3  “아니 보내지 말고 설명만”까지 확정 입력',False),(1,2,'현재 허용된 표 읽기 요청',False),(2,1,'표 내용/버전 반환',True),(1,3,'확정 입력과 근거의 전체 의미 요청',False),(3,1,'설명 의도; 발송 금지 반환',True),(1,4,'4  현재 검사 후 설명 응답 허용',False),(4,0,'5  음성/Text 전달; 실제 재생 구간 반환',False),(0,1,'“세 번째 열” 정정: 먼저 재생 stop, 새 요청',False)]),
'B':(['Interaction Manager','Incremental Interaction Runtime','Context Manager','Request Interpreter','Request Controller'],[
(0,1,'1  같은 부분 발화 + 수정 버전',False),(1,3,'2  잠정 의미 요청; 외부 실행 허용은 아님',False),(1,2,'이미 동의한 범위의 표 미리 읽기',False),(2,1,'표/버전 반환; 의존 관계에 기록',True),(3,1,'잠정 의미/의존 입력 버전 반환',True),(0,1,'3  “아니 보내지 말고 설명만” 수정',False),(1,1,'보내기 의미/응답 폐기; 유효한 표 읽기는 유지',False),(1,3,'설명 의미로 재해석; 취소 불가 job은 늦은 결과 폐기',False),(3,1,'설명 의미와 새 입력 버전 반환',True),(1,4,'4  유효 준비 결과 + 최종 입력; 현재 검사 요청',False),(4,0,'5  Response Manager를 통해 허용된 음성/Text 전달',False),(0,1,'새 정정: 먼저 stop; 관련 잠정 작업만 무효화',False)])},
35:{'A':(['Request Controller','State Store','Agent Gateway','Downstream Agent','Response Manager'],[
(0,0,'1  질문 q7의 답변/Task/권한 버전 확인',False),(0,1,'2  q7 종료 + 답변 + command c9 intent 원자 저장',False),(1,0,'현재 record 확정 반환',True),(2,3,'3  현재 권한 재검사 후 c9 전송',False),(2,2,'접수 확인을 받기 전에 VIA 종료',False),(1,0,'4  재시작: 현재 record/미완료 원장 직접 읽기',True),(0,2,'5  c9 접수 여부 확인 요청',False),(2,3,'command key로 상태 조회; 무조건 재전송 금지',False),(3,2,'접수/미접수/불명 상태 반환',True),(2,4,'확인된 상태만 사용자에게 게시',False)]),
'B':(['Request Controller','State Store','Agent Gateway','Downstream Agent','Response Manager'],[
(0,0,'1  같은 q7 답변/Task/권한 버전 확인',False),(0,1,'2  답변 확정/전송 준비 사건 + c9 intent',False),(1,1,'journal + projection + 위치/intent 원자 확정',False),(1,0,'내구 확정 결과 반환',True),(2,3,'3  같은 현재 검사 뒤 c9 전송',False),(2,2,'접수 확인 전 같은 종료; 없던 ACK는 만들지 않음',False),(1,1,'4  유효 checkpoint + tail → reducer 재생',False),(1,0,'복원 상태/사건 위치 반환; replay 자체는 전송 안 함',True),(0,2,'5  c9 접수 여부 확인 요청',False),(2,3,'같은 command key로 외부 상태 조회',False),(3,2,'접수/미접수/불명 상태 반환',True),(2,4,'확인된 상태만 사용자에게 게시',False)])},
36:{'A':(['Request Controller','Context Manager','Request Interpreter','자료 접근 Adapter','Policy Manager'],[
(0,4,'1  이번 보고서의 목적/자료 범위 확인',False),(4,0,'허용 범위/현재 버전 반환',True),(0,1,'2  허용된 자료 조립 요청',False),(1,3,'3  같은 process 권한으로 실제 읽기',False),(3,1,'내용/버전 반환',True),(1,2,'4  조립된 자료; Model Access로 추론 요청/반환',False),(2,0,'5  의미 제안; 현재 검사 후 외부 경로 허용',True),(2,2,'위협: 결함 코드가 정책 호출을 건너뛴다면',False),(2,3,'process 자격으로 무관한 자료에 접근할 수 있는 위험',False),(4,1,'철회: 협력 코드/자료/cache/출력까지 차단 필요',False)]),
'B':(['Request Controller','Request Interpreter','Context Manager','Data Access Broker','Policy Manager'],[
(0,4,'1  같은 목적/자료 범위 확인',False),(4,0,'현재 허용 범위 반환; Broker handle 준비',True),(0,1,'2  제한 process에 입력 + 범위 handle',False),(1,2,'3  필요한 자료 요청; ambient 권한 없음',False),(2,3,'IPC: handle + 실제 읽을 범위',False),(3,4,'현재 허용/철회 확인',False),(3,2,'허용된 내용만 반환',True),(1,3,'4  추론도 Broker → Model Access → 같은 Omni',False),(3,1,'모델 결과와 사용한 자료 근거 반환',True),(1,0,'5  의미 제안; 신뢰 Core가 현재 검사',True),(1,1,'위협: 직접 파일/네트워크/DB 접근은 OS 경계에서 차단',False),(4,3,'철회: handle 무효화, 미완료 호출/늦은 결과 차단',False),(3,3,'Broker 자체 침해는 잔여 위험; 만능 보안은 아님',False)])}}

def wrap_name(name):
 if ' ' in name:return name.replace(' ', '\n',1) if len(name)>18 else name
 return name

def event(n):
 p=Plate(f'choice{n}-event',f'04-{n}',TITLES[n],CASES[n],height=2340)
 for s in 'AB':
  d=D(p,n,s);names,steps=TRACES[n][s];xs=[d.x+116+i*238 for i in range(5)]
  p.text(d.x,345,'위에서 아래로 읽기. 가로 화살표의 시작/끝이 실제 송신자/수신자.',20,MUTED)
  for i,name in enumerate(names):
   x=xs[i];p.box(x-108,400,216,104,'white',INK,4)
   p.text(x,415,wrap_name(name),20,INK,'center',True,leading=28)
   p.line([(x,504),(x,2000)],'#C7D1DD',dashed=True,arrow=False)
  for i,(a,b,label,ret) in enumerate(steps):
   y=620+i*min(111,1300//max(1,len(steps)-1))
   if label.startswith('별도 변형:'):
    p.line([(d.x+5,y-60),(d.x+1180,y-60)],'#C7D1DD',dashed=True,arrow=False)
   p.text(d.x+10,y-40,label,21,d.c,bold=False)
   points=[(xs[a],y),(xs[b],y)] if a!=b else [(xs[a],y),(xs[a]+45,y),(xs[a]+45,y+25),(xs[a],y+25)]
   p.line(points,d.c,dashed=ret,width=2.5)
  p.text(d.x+10,2070,'읽을 결론: '+BENEFITS[n][s=='B'][0][4:],22,d.c,bold=True)
  p.text(d.x+10,2115,BENEFITS[n][s=='B'][1],21,MUTED)
 p.text(64,2200,'생략한 공통 연동: 추론은 Model Access의 한 벌 Omni; 업무 실행은 Task Manager/Agent Gateway가 Downstream Agent에 위임.',19,MUTED)
 p.text(64,2240,'실선 = 요청/전달, 점선 = 결과. 별도 언급 없는 참여자는 논리 책임. 시간 간격은 성능 수치가 아닌 사건 순서. 수작업 설계 추적, 미측정.',19,MUTED)
 return p

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args();drift=[]
 plates=[p for n in range(31,37) for p in (structural(n),event(n))]
 for p in plates:
  p.validate()
  for i in p.items:
   if i['kind']=='component':assert not i['role'],(p.slug,'behavior inside component')
   if i['kind']=='text':
    for node in p.items:
     if node['kind']=='component':
      assert not (node['x'] < i['x'] < node['x']+node['w'] and node['y']+62 < i['y'] < node['y']+node['h']), (p.slug,'free prose inside component',i['lines'])
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
 print(f'PASS: {len(plates)} pairs; source parity, XML, ownership, bounds, routes, name-only component headers')
if __name__=='__main__':main()
