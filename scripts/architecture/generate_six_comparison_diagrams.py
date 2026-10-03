#!/usr/bin/env python3
"""Six mechanism comparisons: distinct structures plus same-event walkthroughs."""
import argparse
from pathlib import Path
import xml.etree.ElementTree as ET
from stage4_diagram_design import Plate, INK, MUTED, BLUE, GREEN
ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'docs/architecture/12-decisions/decision-packages/diagrams'
TITLES={31:'요청 이해의 정확성을 위한 의미 구성 — 요청별 생성과 지속 의미 작업공간',32:'화면 지칭의 정확성을 위한 대상 획득 — 관측 영역과 source 객체',33:'여러 업무의 대화 정확성을 위한 협력 — 중앙 해석과 업무별 대화 처리',34:'음성 대화의 반응성과 정정 정확성을 위한 실행 — 발화 단위와 연속 수정',35:'대화와 업무의 복구 정확성을 위한 기록 — 현재 상태와 사건 이력',36:'허용된 자료만 처리하기 위한 권한 배치 — 공유 처리와 제한 처리'}
NAMES={31:('요청별 완성 의미 생성','지속 의미 작업공간'),32:('관측에서 영역 해석','source 객체를 지칭에 결합'),33:('중앙이 전체 대화 해석','업무 대화 actor가 협력'),34:('확정 발화 단위 처리','잠정 작업을 계속 수정'),35:('현재 상태가 원본','확정 사건이 원본'),36:('공유 권한의 처리 코드','Broker와 제한 process')}

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

# Each option has its own dependency graph. Only drawing primitives are shared.
class Drawing:
 def __init__(self,p,x,s,n):
  self.p,self.x,self.s,self.c=p,x,s,BLUE if s=='A' else GREEN
  head(p,x,s,n)
 def comp(self,key,x,y,w,h,name,role='',color=INK):
  return self.p.component(self.s+key,self.x+x,y,w,h,name,color=color,role=role)
 def node(self,key,x,y,w,h,name,kind='module',owner=None,color=None):
  return self.p.node(self.s+key,self.x+x,y,w,h,name,kind=kind,owner=self.s+owner if owner else None,color=self.c if color is None else color)
 def bound(self,key,x,y,w,h,title,subtitle='',color=INK):
  self.p.boundary(self.s+key,self.x+x,y,w,h,title,subtitle,color=color,dashed=True)
 def edge(self,a,b,sp='B',tp='T',via=(),label='',at=None,ret=False,sd=0,td=0,color=None):
  self.p.edge(self.s+a,self.s+b,sp=sp,tp=tp,via=[(self.x+x,y) for x,y in via],label=label,at=(self.x+at[0],at[1]) if at else None,ret=ret,sd=sd,td=td,color=self.c if color is None else color)
 def text(self,x,y,lines,size=20):self.p.text(self.x+x,y,lines,size,MUTED)
 def via(self):self.bound('via',0,320,1184,1790,'VIA SOFTWARE','논리 책임 / 실제 process는 별도 표시')
 def model(self,y=1810):
  self.comp('ma',790,y,350,130,'Model Access')
  self.node('omni',790,2150,350,110,'공유 Omni 한 개\n모델 내부는 책임 밖',kind='model',color=INK)
  self.edge('ma','omni',sd=-20,td=-20,color=INK,label='추론 요청',at=(835,2070))
  self.edge('omni','ma',sp='T',tp='B',sd=20,td=20,ret=True,color=INK)
 def model_call(self,a):
  # The model dependency is routed outside the compared internal mechanism.
  y=self.p.port(self.s+a,'R',-20)[1];my=self.p.port(self.s+'ma','R',-20)[1]
  self.edge(a,'ma',sp='R',tp='R',sd=-20,td=-20,via=[(1160,y),(1160,my)],color=INK)
  y=self.p.port(self.s+a,'R',20)[1];my=self.p.port(self.s+'ma','R',20)[1]
  self.edge('ma',a,sp='R',tp='R',sd=20,td=20,via=[(1174,my),(1174,y)],ret=True,color=INK)
 def tail(self,lines):self.text(40,2330,lines,20)

def semantic_workspace(d):
 d.via()
 d.comp('rc',40,360,1100,140,'Request Controller',role='새 입력 전달 / 현재 조건 검사 후 의미를 명령으로 확정')
 d.comp('ri',40,680,480,140,'Request Interpreter',role='완성 실행안 대신 타입 있는 의미 변경 후보')
 d.comp('ma',790,680,350,140,'Model Access')
 d.comp('workspace',40,1020,620,770,'Meaning Workspace',role='여러 입력에 걸친 목표/후보/제약과 질문 소유',color=d.c)
 d.node('facts',80,1140,540,130,'c1: 작년 표 / c2: 발송 금지\nq1: 어느 표인가? / 후보 L, R',kind='store',owner='workspace')
 d.node('update',80,1480,540,130,'의미 변경 적용기\nNARROW q1: R / c2 보존',owner='workspace')
 d.comp('engine',790,1110,350,190,'Semantic Constraint Engine',role='후보 교집합 / 충돌과 unknown 반환',color=d.c)
 d.comp('cm',790,1470,350,140,'Context Manager',role='허용 읽기 / source 버전 변경')
 d.comp('tm',790,1760,350,140,'Task Manager',role='실제 Task 사실 / 현재 버전')
 d.comp('rm',40,1950,620,140,'Response Manager',role='미해결 필드 질문 / 실제 게시 기록 반환')
 d.edge('rc','ri',via=[(590,585),(280,585)],label='새 발화 / 보완 / 정정',at=(80,570))
 d.edge('ri','ma',sp='R',tp='L',sd=-20,td=-20,color=INK,label='변경 후보 생성 요청',at=(555,680))
 d.edge('ma','ri',sp='L',tp='R',sd=20,td=20,ret=True,color=INK)
 d.edge('ri','workspace',sp='R',tp='R',td=-315,via=[(710,750),(710,1090)],label='현재 의미/질문 조회',at=(550,840))
 d.edge('workspace','ri',sp='R',tp='R',sd=-285,td=25,via=[(740,1120),(740,775)],ret=True)
 d.edge('ri','update',sp='B',tp='L',via=[(280,920),(20,920),(20,1545)],label='이전 의미 버전에 적용할 변경',at=(80,920))
 d.edge('update','facts',sp='T',tp='B',sd=-20,td=-20,label='조건 / 질문 / 의존 관계 갱신',at=(95,1355))
 d.edge('facts','update',sd=20,td=20,ret=True)
 d.edge('facts','engine',sp='R',tp='L',sd=-20,td=-20,label='현재 후보/제약',at=(650,1135))
 d.edge('engine','update',sp='L',tp='R',sd=20,td=-20,via=[(710,1225),(710,1525)],ret=True)
 d.edge('update','cm',sp='R',tp='L',sd=-35,td=-30,via=[(735,1510)],label='읽기 / 변경 사실 반영',at=(80,1675))
 d.edge('cm','update',sp='L',tp='R',sd=0,td=-5,ret=True)
 d.edge('update','tm',sp='R',tp='L',sd=35,via=[(745,1580),(745,1830)])
 d.edge('tm','update',sp='L',tp='R',sd=25,td=55,via=[(765,1855),(765,1600)],ret=True)
 d.edge('update','rm',sd=-20,td=-20,label='미해결 필드의 질문 / 실제 게시',at=(80,1840))
 d.edge('rm','update',sp='T',tp='B',sd=20,td=20,ret=True)
 d.edge('update','rc',sp='L',tp='L',sd=35,td=-20,via=[(8,1580),(8,410)],ret=True,label='Workspace가 현재 의미 반환',at=(50,975))
 d.node('omni',790,2150,350,110,'공유 Omni 한 개\n모델 내부는 책임 밖',kind='model',color=INK)
 d.edge('ma','omni',sp='R',tp='R',sd=-20,td=-20,via=[(1172,730),(1172,2185)],color=INK)
 d.edge('omni','ma',sp='R',tp='R',sd=20,td=20,via=[(1182,2225),(1182,770)],ret=True,color=INK)
 d.text(40,2155,['질문에 대한 답변'+'은 Interaction Manager → Request Controller로 새 입력.', '확정 명령은 Task Manager → Agent Gateway → Agent.'],18)
 d.tail(['차이: 요청별 완성안 생성 ↔ 입력/자료/Task 변경이 지속 의미 상태를 갱신하는 처리망.', '언어 후보 추출의 오해는 제약 해법으로 해결되지 않는다. 단발 요청에서는 B가 과할 수 있다.'])

def source_references(d):
 d.via();d.comp('input',40,360,1100,140,'Interaction Manager',role='같은 화면/포인터/발화 시점 / 공개 객체와 당시 지칭을 결합')
 d.comp('service',40,710,650,1150,'Source Reference Service',role='객체 참조의 획득 / 내용 해석 / 수명과 무효화',color=d.c)
 d.node('provider',100,850,530,110,'객체 연동 / 시점 결합',owner='service')
 d.node('refs',100,1170,530,140,'제공자 / 문서 / 객체 / 버전\n공유 참조 기록',kind='store',owner='service')
 d.node('resolver',100,1520,530,130,'참조 해석 / 현재 유효성 확인',owner='service')
 d.comp('cm',790,800,350,140,'Context Manager',role='내용 읽기의 참조 소비자')
 d.comp('ri',790,1160,350,140,'Request Interpreter',role='지칭 후보의 참조 소비자')
 d.comp('gateway',790,1580,350,140,'Agent Gateway',role='인계 내용의 참조 소비자')
 d.comp('rc',40,1940,650,140,'Request Controller',role='현재 권한 / 입력 / 대상 검사 후 확정, Task Manager에 인계')
 d.model(1870)
 d.edge('input','provider',via=[(590,580),(365,580)],label='당시 지칭/선택 결합',at=(100,575))
 d.edge('provider','refs',label='객체 identity와 시점 기록',at=(100,1060))
 d.edge('refs','resolver',label='같은 참조로 후속 접근',at=(100,1400))
 d.edge('resolver','provider',sp='L',tp='L',via=[(80,1585),(80,905)],label='제공자 재조회 / 무효화',at=(120,1735))
 for key,lane,delta in [('cm',720,-35),('ri',740,0),('gateway',760,35)]:
  y=d.p.port(d.s+key,'L',-20)[1]
  d.edge(key,'resolver',sp='L',tp='R',sd=-20,td=delta,via=[(lane,y),(lane,1585+delta)])
  d.edge('resolver',key,sp='R',tp='L',sd=delta+12,td=20,via=[(lane+8,1597+delta),(lane+8,y+40)],ret=True)
 d.edge('ri','rc',sp='B',tp='R',via=[(965,1360),(775,1360),(775,2010)],ret=True,label='대상 후보 반환 / 현재 검사',at=(100,1865))
 d.edge('input','rc',sp='L',tp='L',via=[(4,430),(4,2010)],color=INK)
 d.edge('rc','ri',sp='R',tp='R',sd=35,td=-45,via=[(1182,2045),(1182,1185)],color=INK,label='원문 해석 요청',at=(790,2040))
 d.model_call('ri')
 d.node('source',40,2180,650,110,'같은 앱 / 문서 source\n객체 읽기 / 선택 및 버전 / 재확인',kind='model',color=INK)
 d.edge('provider','source',sp='L',tp='L',sd=-20,td=-20,via=[(16,885),(16,2215)])
 d.edge('source','provider',sp='L',tp='L',sd=20,td=20,via=[(28,2255),(28,925)],ret=True)
 d.text(80,2100,['Agent Gateway는 확정 명령의 자료만 해석해 외부 Agent에 전달.'],17)
 d.tail(['차이: 당시 화면 영역이 주 대상 ↔ source 참조를 입력/조회/해석/인계가 함께 소비.', '공통 참조/resolver로 결합 가능: 주요 DP 자격 철회. 세 번째 그림에서 반례를 확인.'])

def semantic(d):
 if d.s=='B':return semantic_workspace(d)
 d.via();d.comp('rc',40,360,720,130,'Request Controller',role='같은 원문/근거 전달, 현재 조건 검사')
 d.comp('ri',40,650,1100,1040,'Request Interpreter')
 d.node('producer',100,770,480,120,'통합 의미 생성',owner='ri')
 d.node('proposal',100,1040,480,110,'완성 관계 제안\n목표 / 대상 / Task / 조건',kind='data',owner='ri')
 d.node('validator',100,1330,480,120,'제안 검증',owner='ri')
 d.edge('rc','producer',via=[(400,580),(340,580)],label='근거를 함께 전달',at=(425,555))
 d.edge('producer','proposal',label='모델이 전체 관계 생산',at=(355,955))
 d.edge('proposal','validator',label='존재 / 현재 권한 / 모순 검사',at=(355,1230))
 d.edge('validator','producer',sp='L',tp='L',via=[(75,1390),(75,830)],label='오류이면 제한된 재해석',at=(630,1440))
 d.text(650,1090,['전체 관계를 만드는 주체: 모델','검증기는 완성안을 검사','실패: 재해석 또는 사용자 확인'],21)
 d.model();d.model_call('producer')
 d.comp('out',40,1810,620,140,'Request Controller',role='최종 의미 또는 질문을 받아 현재 조건 확인')
 d.edge('validator','out',via=[(340,1730),(350,1730)],ret=True,label='검증 결과 반환',at=(380,1735))
 d.text(40,2060,['확정 후 Task Manager → Agent Gateway → Downstream Agent', '접수/결과 반환 후 Response Manager가 사용자에게 전달'],19)
 d.tail(['차이: 요청별 완성안 생성 ↔ 지속 의미 상태에서 해와 질문 도출.', '외부 포트가 같아도 내부 생산 체계는 다르다. 주요 결정의 규모는 본문에서 별도 판단.'])

def grounding(d):
 if d.s=='B':return source_references(d)
 d.via();d.comp('input',40,360,1100,140,'Interaction Manager',role='같은 화면/포인터/발화 시점. B도 화면 보조 근거 사용 가능')
 d.comp('cm',40,650,650,650,'Context Manager')
 d.node('buffer',100,780,520,120,'시점별 화면 / 포인터\n유한 RAM 기록',kind='store',owner='cm')
 d.node('align',100,1090,520,110,'관측 조립',owner='cm')
 d.edge('input','buffer',via=[(590,565),(360,565)],label='당시 관측 전달',at=(120,575))
 d.edge('buffer','align',label='발화 시점의 장면 선택',at=(380,990))
 d.text(760,900,['이미지 속 영역을 대상으로 해석','ID 힌트도 사용할 수 있음','현재 문서와 같다는 확인은 별도'],20)
 d.comp('ri',40,1460,620,150,'Request Interpreter',role='원문과 근거를 같은 모델로 해석, 대상 후보 반환')
 src='align' if d.s=='A' else 'resolver'
 sx=d.p.port(d.s+src,'B')[0]-d.x
 d.edge(src,'ri',via=[(sx,1390),(350,1390)],label='Request Controller 경유: 지칭 근거',at=(80,1335))
 d.model(1810)
 d.edge('ri','ma',sp='R',tp='L',via=[(740,1535),(740,1875)],color=INK,label='의미 요청',at=(770,1665))
 d.edge('ma','ri',sp='L',tp='R',sd=30,td=30,via=[(760,1905),(760,1565)],ret=True,color=INK)
 d.comp('rc',40,1810,620,140,'Request Controller',role='현재 사용 가능한 대상인지 검사, 인계 또는 질문')
 d.edge('ri','rc',ret=True,label='관측 영역' if d.s=='A' else '제공자/문서/범위 참조',at=(80,1720))
 d.node('source',40,2150,620,110,'같은 앱 / 문서 환경\n화면 관측과 공개 객체 읽기',kind='model',color=INK)
 src='input' if d.s=='A' else 'provider';sy=d.p.port(d.s+src,'L',-20)[1]
 d.edge(src,'source',sp='L',tp='L',sd=-20,td=-20,via=[(16,sy),(16,2185)])
 sy=d.p.port(d.s+src,'L',20)[1]
 d.edge('source',src,sp='L',tp='L',sd=20,td=20,via=[(28,2225),(28,sy)],ret=True)
 d.text(40,2060,['A: 당시 화면을 획득 / B: 제공자에 객체 읽기와 재확인 요청', '확정 대상 → Task Manager → Agent Gateway → Agent, 접수/결과 반환'],18)
 d.tail(['차이: 화면 기록의 해석 경로 ↔ 제공자/참조의 지속 재조회 경로.', '공개 객체가 없는 영역은 B의 자동 인계가 제한된다. 혼합과 연동 개발의 규모를 별도 판단.'])

def dialogue(d):
 d.via()
 if d.s=='A':
  d.comp('owner',40,360,650,1450,'Request Controller',role='발화 수신 / 전체 대화 상태의 확정 owner')
  d.node('state',100,650,520,150,'중앙의 대화 관계 / 질문\n내구 상태 + 관련 Task cache',kind='store',owner='owner')
  d.node('gather',100,1030,520,120,'전체 발화 해석 조정',owner='owner')
  d.node('apply',100,1530,520,120,'대화 상태 / 명령 확정',owner='owner')
  d.comp('ri',790,1010,350,150,'Request Interpreter')
  d.model(1590)
  d.edge('state','gather',label='관련 대화 공동 회수',at=(380,900))
  d.edge('gather','ri',sp='R',tp='L',sd=-5,label='전체 발화 요청',at=(650,1040))
  d.edge('ri','apply',sp='L',tp='R',sd=35,via=[(735,1120),(735,1590)],ret=True,label='전체 의미/상태 제안',at=(755,1280))
  d.edge('apply','state',sp='L',tp='L',via=[(75,1590),(75,725)],label='중앙 상태 갱신',at=(120,1435))
  d.edge('ri','ma',sd=-20,td=-20,color=INK)
  d.edge('ma','ri',sp='T',tp='B',sd=20,td=20,ret=True,color=INK)
  d.text(100,1880,['같은 중앙 해석 결과로 업무 간 참조와 상태 변경을 구성.', 'Task별 cache가 있어도 독립 대화 실행자는 없음.'],20)
 else:
  d.comp('rc',40,360,1100,140,'Request Controller',role='현재 질문/Task/권한 확인, 유효 제안만 적용 허용')
  d.comp('coord',40,660,1100,150,'Dialogue Coordinator',role='참가 / 참조 연결 / 제안 채택. 로컬 의미를 다시 작성하지 않음',color=d.c)
  d.comp('runtime',40,1010,1100,720,'Task Dialogue Runtime',role='논리 실행 단위. 같은 process / 같은 모델 자원',color=d.c)
  for name,x,title in [('report',100,'보고서'),('mail',700,'메일')]:
   d.node(name,x,1120,390,120,title+' 대화 actor',owner='runtime')
   d.node(name+'state',x,1450,390,140,('qR: 형식 / 결론 R.result@4\n자기 제약 / 우편함 / 내구 상태' if name=='report' else 'qM: 본문 / 다른 결론 참조 요청\n자기 제약 / 우편함 / 내구 상태'),kind='store',owner='runtime')
   d.edge(name,name+'state',sd=-20,td=-20,label='채택된 전이 / 상태 조회',at=(x+15,1310))
   d.edge(name+'state',name,sp='T',tp='B',sd=20,td=20,ret=True)
   offset=-295 if name=='report' else 305
   d.edge('coord',name,sd=offset-20,td=-20,label='입력/참조 전달',at=(x,900))
   d.edge(name,'coord',sp='T',tp='B',sd=20,td=offset+20,ret=True)
  d.edge('rc','coord',sd=-20,td=-20,color=INK,label='입력 / 현재 적용 허용',at=(615,560))
  d.edge('coord','rc',sp='T',tp='B',sd=20,td=20,ret=True,color=INK)
  d.comp('ri',40,1880,620,140,'Request Interpreter',role='actor가 자신의 맥락과 원문 구간을 제공')
  d.model(1885)
  # Each actor calls the common interpreter. Its mailbox, state and proposal are separate.
  d.edge('report','ri',sp='L',tp='L',via=[(65,1180),(65,1810),(20,1810),(20,1950)])
  d.edge('ri','report',sp='L',tp='L',sd=25,td=25,via=[(8,1975),(8,1790),(85,1790),(85,1205)],ret=True)
  d.edge('mail','ri',sp='R',tp='R',via=[(1160,1180),(1160,1790),(730,1790),(730,1950)])
  d.edge('ri','mail',sp='R',tp='R',sd=25,td=25,via=[(750,1975),(750,1810),(1174,1810),(1174,1205)],ret=True)
  d.edge('ri','ma',sp='R',tp='L',sd=-20,td=-20,color=INK)
  d.edge('ma','ri',sp='L',tp='R',sd=20,td=20,ret=True,color=INK)
  d.text(40,2060,['actor 제안은 Coordinator로 반환. 참조 요청도 Coordinator 중개.', '현재 검사 후 채택 상태/명령을 local transaction으로 확정하고 통지.'],18)
 d.tail(['확정 명령은 Task Manager → Agent Gateway → Agent. 외부 질문/결과와 실제 전달은 대화 owner로 복귀.', '차이: 중앙의 공동 해석/갱신 ↔ 여러 상태 보유 실행자의 제안/조정/채택.'])

def voice(d):
 d.via();d.comp('input',40,360,1100,140,'Interaction Manager',role='Speech Input Worker의 연속 ASR / 재생 중 local stop은 양안 공통')
 d.comp('ri',790,740,350,140,'Request Interpreter')
 d.comp('cm',790,1080,350,140,'Context Manager')
 d.comp('rm',790,1460,350,140,'Response Manager')
 d.model(1840);d.model_call('ri')
 d.edge('rm','ma',sd=-40,td=-40,color=INK,label='응답 준비 추론',at=(790,1740))
 d.edge('ma','rm',sp='T',tp='B',sd=40,td=40,ret=True,color=INK)
 if d.s=='A':
  d.comp('rc',40,670,650,1290,'Request Controller')
  for key,y,label in [('utterance',790,'확정 발화 / 요청 버전'),('read',1040,'근거 읽기 조정'),('interpret',1290,'확정 요청 해석'),('commit',1750,'현재 의미 / 권한 확정')]:
   d.node(key,100,y,520,110,label,kind='data' if key=='utterance' else 'module',owner='rc')
  d.edge('input','utterance',via=[(590,585),(360,585)],label='발화 확정 후 전달',at=(100,560))
  d.edge('utterance','read',label='확정 전에는 이 요청을 시작하지 않음',at=(100,945))
  d.edge('read','cm',sp='R',tp='L',via=[(735,1095),(735,1150)],label='근거 요청',at=(645,1050))
  d.edge('cm','interpret',sp='L',tp='R',sd=25,via=[(755,1175),(755,1345)],ret=True)
  d.edge('interpret','ri',sp='R',tp='L',via=[(710,1345),(710,810)],label='전체 해석',at=(640,920))
  d.edge('ri','commit',sp='L',tp='R',sd=25,via=[(730,835),(730,1805)],ret=True)
  d.edge('commit','rm',sp='R',tp='L',via=[(755,1805),(755,1530)],label='확정 응답 요청',at=(110,1670))
 else:
  d.comp('runtime',40,670,650,1080,'Incremental Interaction Runtime',color=d.c)
  d.node('fragment',100,790,520,110,'부분 발화 / 수정 버전',kind='data',owner='runtime')
  d.node('deps',100,1050,520,140,'잠정 의미 / 읽기 / 응답의\n의존 그래프 (RAM)',kind='store',owner='runtime')
  d.node('jobs',100,1410,520,140,'의존 작업 배정 / 무효화 / 재사용',owner='runtime')
  d.edge('input','fragment',via=[(590,585),(360,585)],label='부분 갱신마다 전달',at=(100,560))
  d.edge('fragment','deps',label='새 입력으로 영향 범위 갱신',at=(100,945))
  d.edge('deps','jobs',sd=-20,td=-20,label='유효 결과 / 필요한 작업',at=(100,1270))
  d.edge('jobs','deps',sp='T',tp='B',sd=20,td=20,ret=True)
  for key,lane in [('ri',710),('cm',735),('rm',760)]:
   ty=d.p.port(d.s+key,'L',-15)[1]
   delta={'ri':-35,'cm':0,'rm':35}[key]
   d.edge('jobs',key,sp='R',tp='L',sd=delta,td=-15,via=[(lane,1480+delta),(lane,ty)])
   d.edge(key,'jobs',sp='L',tp='R',sd=15,td=delta+10,via=[(lane+7,ty+30),(lane+7,1490+delta)],ret=True)
  d.comp('rc',40,1840,650,130,'Request Controller',role='유효한 의미 / 현재 권한으로 확정 경계 갱신')
  d.edge('jobs','rc',via=[(360,1780),(365,1780)],label='유효 결과만 확정 요청',at=(100,1770))
  d.edge('rc','rm',sp='R',tp='B',via=[(745,1905),(745,1690),(965,1690)],color=INK,label='게시 허용',at=(790,1680))
  d.text(100,1580,['정정: 관련 작업 폐기', '늦은 옛 결과는 무시'],20)
 d.text(40,2070,['Response Manager → Interaction Manager: 허용된 Text/음성, 실제 전달 반환.', 'B의 응답 준비는 게시 허용과 다름. 승인 전 외부 변경은 양안 모두 금지.'],18)
 d.tail(['차이: 확정 요청의 단계 진행 ↔ 수정 사건이 의미/읽기/응답 작업을 갱신하는 실행망.', '단순 ASR/TTS streaming이나 읽기 한 번의 prefetch만으로 B 전체를 정당화하지 않는다.'])

def state(d):
 d.via();d.comp('rc',40,360,1100,140,'Request Controller',role='Task Manager 등 의미 owner도 같은 State Store 계약을 사용')
 d.comp('ss',40,680,1100,1310,'State Store',role='한 local transaction으로 원본, 현재 view와 전송 intent의 일관성 유지')
 if d.s=='A':
  d.node('write',100,820,430,120,'현재 변경 집합 적용',owner='ss')
  d.node('current',100,1160,430,140,'현재 owner record\n내구 원본',kind='store',owner='ss')
  d.node('ledger',700,1160,390,140,'미완료 inbox/outbox\n내구 전송 원장',kind='store',owner='ss')
  d.node('read',100,1590,430,120,'현재 상태 읽기 / 복원',owner='ss')
  d.edge('rc','write',via=[(590,570),(315,570)],label='새 상태와 전송 intent',at=(100,555))
  d.edge('write','current',label='원자 갱신',at=(335,1030))
  d.edge('write','ledger',sp='R',tp='T',via=[(605,880),(605,1050),(895,1050)],label='같은 transaction',at=(680,1000))
  d.edge('current','read',ret=True,label='현재 값을 그대로 복원',at=(335,1410))
  d.edge('ledger','read',sp='B',tp='R',via=[(895,1470),(605,1470),(605,1650)],ret=True)
  d.edge('read','rc',sp='L',tp='L',via=[(20,1650),(20,430)],ret=True)
  d.text(700,1590,['감사 log와 backup은 추가 가능.', '현재 상태를 매번 사건에서', '다시 만들 필요는 없음.'],22)
 else:
  d.node('append',100,820,430,110,'의미 사건 확정',owner='ss')
  d.node('journal',100,1060,430,130,'의미 사건 journal\n내구 원본',kind='store',owner='ss')
  d.node('reduce',700,1060,390,130,'상태 전이 reducer',owner='ss')
  d.node('replay',100,1410,430,110,'checkpoint + 후속 사건 재생',owner='ss')
  d.node('view',700,1410,390,130,'현재 projection\n다시 만들 수 있는 view',kind='store',owner='ss')
  d.node('checkpoint',100,1740,430,110,'검증된 checkpoint\n파생 snapshot',kind='store',owner='ss')
  d.node('ledger',700,1740,390,110,'전송 intent / 원장',kind='store',owner='ss')
  d.edge('rc','append',via=[(590,570),(315,570)],label='새 상태 값 대신 의미 전이',at=(100,555))
  d.edge('append','journal',label='사건 append',at=(335,970))
  d.edge('journal','reduce',sp='R',tp='L',label='순서 있는 사건',at=(555,1070))
  d.edge('reduce','view',label='결정적 전이로 상태 생성',at=(720,1290))
  d.edge('journal','replay',ret=True,label='재시작 시 후속 사건',at=(100,1290))
  d.edge('checkpoint','replay',sp='T',tp='B',ret=True,label='검증된 시작점',at=(100,1600))
  d.edge('replay','reduce',sp='R',tp='L',sd=-20,td=30,via=[(615,1445),(615,1155)],label='같은 전이 함수',at=(540,1340))
  d.edge('view','checkpoint',sp='L',tp='R',sd=25,via=[(630,1500),(630,1795)],label='검증 후 주기적 보관',at=(690,1595))
  d.edge('append','ledger',sp='R',tp='R',via=[(1110,875),(1110,1795)])
  d.edge('view','rc',sp='R',tp='R',sd=-25,via=[(1160,1450),(1160,430)],ret=True)
 d.text(40,2070,['Agent Gateway ↔ Downstream Agent: 전송 / 실제 접수 조회.', '현재 불명 접수는 양안 모두 외부 조회. replay는 외부 명령을 실행하지 않음.'],19)
 d.tail(['차이: 현재 원본 직접 갱신/읽기 ↔ 사건 원본과 파생 view의 생산/재생.', 'B의 역전환은 더 쉬울 수 있다. 그 사실을 전환 비용 판단에 포함한다.'])

def isolation(d):
 d.via()
 if d.s=='A':
  d.bound('core',40,360,1100,1590,'신뢰 Core process','아래 자료 처리 코드도 같은 OS 권한',d.c)
  d.comp('rc',100,480,430,140,'Request Controller')
  d.comp('policy',700,480,390,140,'Policy Manager')
  d.comp('cm',100,900,430,140,'Context Manager')
  d.node('raw',700,900,390,140,'원문 / cache / DB\n공유 접근 영역',kind='store')
  d.comp('ri',100,1300,430,140,'Request Interpreter')
  d.comp('ma',700,1300,390,140,'Model Access')
  d.comp('gateway',700,1760,390,130,'Agent Gateway')
  d.edge('rc','policy',sp='R',tp='L',sd=-20,td=-20,color=INK,label='현재 정책 검사',at=(555,480))
  d.edge('policy','rc',sp='L',tp='R',sd=20,td=20,ret=True,color=INK)
  d.edge('rc','cm',sd=-20,td=-20,label='허용 자료 요청',at=(335,735))
  d.edge('cm','rc',sp='T',tp='B',sd=20,td=20,ret=True)
  d.edge('cm','raw',sp='R',tp='L',sd=-20,td=-20,label='직접 읽기 / 공유 참조',at=(555,890))
  d.edge('raw','cm',sp='L',tp='R',sd=20,td=20,ret=True)
  d.edge('rc','ri',sp='L',tp='L',sd=-20,td=-20,via=[(75,530),(75,1350)],label='근거 전달',at=(115,1160))
  d.edge('ri','rc',sp='L',tp='L',sd=20,td=20,via=[(60,1390),(60,570)],ret=True)
  d.edge('ri','ma',sp='R',tp='L',sd=-20,td=-20,color=INK,label='직접 모델 요청',at=(555,1290))
  d.edge('ma','ri',sp='L',tp='R',sd=20,td=20,ret=True,color=INK)
  d.edge('rc','gateway',via=[(315,700),(600,700),(600,1700),(895,1700)],label='Task Manager 경유 확정 명령',at=(100,1760))
  d.node('source',100,2150,430,110,'앱 / 문서 source\nCore adapter 직접 읽기',kind='model',color=INK)
  d.edge('cm','source',sp='L',tp='L',sd=-20,td=-20,via=[(16,950),(16,2185)])
  d.edge('source','cm',sp='L',tp='L',sd=20,td=20,via=[(28,2225),(28,990)],ret=True)
  d.node('omni',700,2150,390,110,'공유 Omni 한 개\n신뢰 의존성',kind='model',color=INK)
  d.edge('ma','omni',sp='R',tp='R',sd=-20,td=-20,via=[(1150,1350),(1150,2185)],color=INK)
  d.edge('omni','ma',sp='R',tp='R',sd=20,td=20,via=[(1166,2225),(1166,1390)],ret=True,color=INK)
 else:
  d.bound('restricted',40,770,550,940,'제한 process','전역 파일/DB/network 권한 제거',d.c)
  d.bound('core',660,360,480,1590,'신뢰 Core process','Broker/정책/모델 연동',INK)
  d.comp('rc',700,480,400,130,'Request Controller')
  d.comp('policy',700,730,400,130,'Policy Manager')
  d.comp('broker',700,1020,400,230,'Data Access Broker',role='허용 읽기 / 자료 handle / job 중개',color=d.c)
  d.comp('ma',700,1460,400,130,'Model Access')
  d.comp('gateway',700,1770,400,130,'Agent Gateway')
  d.comp('cm',80,900,470,140,'Context Manager')
  d.comp('ri',80,1360,470,140,'Request Interpreter')
  d.edge('broker','policy',sp='T',tp='B',sd=-20,td=-20,label='정책 확인',at=(720,940),color=INK)
  d.edge('policy','broker',sd=20,td=20,ret=True,color=INK)
  d.edge('rc','broker',sp='R',tp='R',sd=-20,td=-20,via=[(1120,525),(1120,1115)],label='유효 요청/범위',at=(80,580),color=INK)
  d.edge('rc','cm',sp='L',tp='T',via=[(315,545)],label='허용 근거 요청 / 반환',at=(80,670),color=INK)
  d.edge('cm','rc',sp='T',tp='L',sd=25,td=25,via=[(340,570)],ret=True,color=INK)
  d.edge('cm','broker',sp='R',tp='L',sd=-20,td=-40,via=[(610,950),(610,1095)],label='자료 요청',at=(570,880))
  d.edge('broker','cm',sp='L',tp='R',sd=-10,td=20,via=[(625,1125),(625,990)],ret=True)
  d.edge('rc','ri',sp='L',tp='T',sd=45,td=-20,via=[(645,590),(645,1280),(295,1280)])
  d.edge('ri','rc',sp='T',tp='L',sd=20,td=55,via=[(335,1260),(650,1260),(650,600)],ret=True)
  d.edge('ri','broker',sp='R',tp='L',sd=-20,td=45,via=[(610,1410),(610,1180)],label='모델 job 요청',at=(80,1580))
  d.edge('broker','ri',sp='L',tp='R',sd=75,td=20,via=[(625,1210),(625,1450)],ret=True)
  d.edge('broker','ma',sd=-20,td=-20,label='검증된 job만',at=(720,1350))
  d.edge('ma','broker',sp='T',tp='B',sd=20,td=20,ret=True)
  d.edge('gateway','broker',sp='R',tp='R',sd=-20,td=40,via=[(1160,1815),(1160,1175)])
  d.edge('broker','gateway',sp='R',tp='R',sd=65,td=20,via=[(1174,1200),(1174,1855)],ret=True)
  d.node('source',80,2150,470,110,'앱 / 문서 source\nBroker만 실제 읽기 권한 보유',kind='model',color=INK)
  d.edge('broker','source',sp='L',tp='R',sd=100,td=-20,via=[(635,1235),(635,2185)])
  d.edge('source','broker',sp='R',tp='L',sd=20,td=110,via=[(650,2225),(650,1245)],ret=True)
  d.node('omni',700,2150,400,110,'공유 Omni 한 개\n신뢰 의존성',kind='model',color=INK)
  d.edge('ma','omni',sp='R',tp='L',sd=20,td=-20,via=[(1148,1545),(1148,2070),(680,2070),(680,2185)],color=INK)
  d.edge('omni','ma',sp='L',tp='R',sd=20,td=40,via=[(668,2225),(668,2050),(1130,2050),(1130,1565)],ret=True,color=INK)
  d.text(80,1810,['제한 process에서 source/model로 직접 가는 경로 없음.', 'Agent Gateway의 자료 제공도 Broker에서 검증.'],18)
 d.text(40,1995,['확정 명령은 Task Manager → Agent Gateway → Agent. 접수/결과는 같은 경로로 반환.'],18)
 d.tail(['차이: 직접 접근 가능한 공유 주소/권한 공간 ↔ 실제 권한이 없는 처리 영역과 중개 통로.', 'Broker, Core와 공유 모델 자체의 침해는 이 격리의 보호 주장 밖이다.'])

def structural(n):
 p=Plate(f'choice{n}-structure',f'04-{n}',TITLES[n],
  ('기능 설계 비교로 유지 / 주요 구조 DP 자격 철회 / 공통 참조로 결합 가능한 경로는 세 번째 그림 참조' if n==32 else '공통 조건은 유지하고 각 안의 실제 생산/상태/실행 관계를 펼침. 배열이나 색 차이 자체는 구조 자격의 증거가 아님.'),height=2600)
 for x,s in [(64,'A'),(1312,'B')]:
  {31:semantic,32:grounding,33:dialogue,34:voice,35:state,36:isolation}[n](Drawing(p,x,s,n))
 footer(p,'실제 설계 차이와 전환 비용은 본문 §8~9. 이 그림은 구현/측정 또는 주요 DP 선정의 증거가 아니다.')
 return p

EVENTS={
31:[('같은 복합 요청과 근거','Interaction Manager → Request Controller: 원문 / Context Manager와 Task Manager: 근거 반환','Request Interpreter → Meaning Workspace: 의미 변경 후보 / 같은 자료와 Task 사실'),('의미 생산','Request Interpreter ↔ Model Access: 전체 관계 생성 후 검사','Meaning Workspace ↔ Semantic Constraint Engine: 후보/제약에서 해 또는 미해결'),('표 확인 질문과 “오른쪽 것” 답변','Response Manager: 질문 / 새 답변과 앞 대화로 완성 의미 재생성','Meaning Workspace: 실제 게시 질문에 변경 후보 연결 / 제약 재평가'),('자료와 Task 변경, 최종 검사','Request Controller: 현재 근거 재검증 / 필요한 해석 재실행','Meaning Workspace: 의존 후보 무효화와 재조회 / Request Controller 최종 검사'),('외부 인계와 사용자 반환','Task Manager → Agent Gateway → Agent: 확정 명령 / 실제 결과 반환','동일. 의미 해의 생성과 실제 실행 접수/완료는 구별')],
32:[('표를 가리킨 순간','Interaction Manager: 화면/포인터/발화 시점 수집','Source Reference Service ↔ source: 당시 객체/선택과 버전 획득'),('근거 읽기','Context Manager → Request Controller: 당시 관측 묶음','Context Manager ↔ Source Reference Service: 객체 참조의 허용 내용'),('문장의 지칭과 결합','Request Interpreter ↔ Model Access: 관측 영역 후보','Request Interpreter ↔ Source Reference Service: 객체 후보 확인 / 같은 모델 해석'),('현재 사용 가능성 검사','Request Controller ↔ Context Manager: 현재 source와 대상 대조','Request Controller: Source Reference Service의 현재 참조/자료 확인 후 확정'),('Agent 자료 인계와 확인','Task Manager → Agent Gateway → Agent: 관측/확인 대상과 명령','Agent Gateway ↔ Source Reference Service: 수신자별 참조 해석 / Agent에 확정 자료 전달')],
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

CASES={
31:[
 ('u1: 작년 표 / 메일은 초안만', 'Request Interpreter가 완성 P1 생성: source=L/R, year=2025, send=false. Request Controller가 질문 q1 게시 요청.', 'Meaning Workspace: c1=year 2025, c2=send false, source=L/R. Semantic Constraint Engine이 미해결 source를 반환, q1 게시.'),
 ('u2: “오른쪽 것”', 'Request Interpreter가 P1/질문/원문으로 전체 P2 재생성: source=R@7, year=2025, send=false. 기존 조건과 비교 검사.', 'Request Interpreter: NARROW(q1,{R@7}) 제안. Meaning Workspace가 q1 버전 검사 후 source만 변경. c1/c2 유지.'),
 ('u3: “표는 올해 것, 메일은 그대로”', '같은 재조회에서 올해 표 S@8 확인. Request Interpreter가 P3 생성: source=S@8, year=2026, send=false. 조건 검증.', 'Request Interpreter: REPLACE(c1,2025→2026). Meaning Workspace가 같은 재조회로 S@8 후보 평가. c2는 변경 대상이 아님.'),
 ('새로 선택한 S@8 삭제', 'Context Manager의 근거 확인에서 삭제 발견. Request Controller가 해당 제안 보류, 새 대상 해석/질문 요청.', 'Context Manager가 무효 근거 반환. Meaning Workspace가 S@8의 의존 후보 제거. Engine 재평가. 메일 조건 c2 유지.'),
 ('q1의 옛 답변이 늦게 도착', 'Request Controller가 현재 질문/입력 버전 확인. 옛 답으로 새 대상을 확정하지 않음.', 'Meaning Workspace가 expected 버전과 q1 수명 검사. 옛 변경 적용 거절. 현재 질문에 다시 연결.')],
33:[
 ('실제로 제시된 두 질문', '중앙 대화 상태: 보고서 qR=형식 질문, 메일 qM=본문 질문. 이전에 제시한 결론 R.result@4.', '보고서 actor: qR / R.result@4. 메일 actor: qM. Task Manager가 외부 질문/결과 사실의 원본.'),
 ('“보고서는 PDF로, 메일에는 그 결론”', 'Request Interpreter가 두 업무 근거와 원문을 함께 읽고 qR=PDF, qM.content=R.result@4를 제안.', '보고서 actor → Coordinator: qR=PDF, export=R.result@4. 메일 actor → Coordinator: qM, need=보고서 결론.'),
 ('결론 참조를 연결', '전체 제안에서 두 대상과 관계 검사. Context Manager로 현재 자료 확인.', 'Coordinator → 메일 actor: R.result@4 전달. 메일 actor가 허용 근거/질문 의미 확인 후 자기 변경 재제안. 중앙의 본문 재작성 없음.'),
 ('채택 직전 메일 질문 취소가 확인됨', 'Task Manager의 qM 버전 변경 확인. Request Controller가 아직 보내지 않은 qM 답 적용을 보류.', 'Task Manager → 메일 actor: qM 종료. 메일 actor 상태 버전 변경. Coordinator가 옛 M 제안의 채택을 거절.'),
 ('현재 검사 후 가능한 부분 적용', 'qR/자료/권한 검사 후 독립적인 PDF 선택만 저장/전달. M의 옛 답변은 보내지 않고 질문 종료 안내.', '같은 guard 후 Coordinator가 유효한 R 제안만 저장/전달하고 actor에 통지. M은 현재 질문 종료 상태를 알림.')]
}

def mechanism_case(n):
 import textwrap
 p=Plate(f'choice{n}-mechanism',f'04-{n}',TITLES[n],
         '실제 자료를 사용한 설계 추적 / 본문 §4.1 / 모델 실행 또는 정확도 측정 결과가 아님',height=2260)
 for x,side in [(64,'A'),(1312,'B')]:
  color=BLUE if side=='A' else GREEN
  p.text(x,250,side+'  '+NAMES[n][side=='B'],28,color,bold=True)
  for j,(name,a,b) in enumerate(CASES[n],1):
   y=345+(j-1)*305
   p.box(x,y,1184,245,'white','#C7D1DD',8)
   p.box(x,y,8,245,color,'none',0)
   p.text(x+28,y+20,str(j)+'. '+name,26,color,bold=True)
   value=a if side=='A' else b
   if n==33:value=value.replace('Coordinator','Dialogue Coordinator')
   lines=textwrap.wrap(value,width=61,break_long_words=False,break_on_hyphens=False)
   p.text(x+28,y+85,lines,23,leading=33)
   if j<5:p.line([(x+592,y+245),(x+592,y+305)],color)
  note=(['A도 기존 조건을 검사할 수 있음. B의 이익은 올바른 변경만 적용하는 경로.', '잘못된 변경 해석은 B도 실패. 외부 실행 승인/동의는 같은 별도 검사.'] if n==31 else ['A도 업무별 버전 검사/부분 진행 가능. B만의 결과라고 주장하지 않음.', 'B의 차이는 상태 전이 작성자와 참조 협력. 모델/DB/프로세스 격리 아님.'])
  p.text(x+20,1925,note,20,MUTED)
 footer(p,'예시의 입력/근거는 양안에 동일. 실제 사용 빈도, 정확도와 시간 이익은 미측정.')
 return p

def reference_extension():
 p=Plate('choice32-extension','04-32','화면 지칭의 두 경로를 결합하는 최소 확장 — 주요 구조 DP 자격을 철회한 이유',
         '공통 대상 참조와 resolver를 사용하는 설계 반례 / 새 주요 대안 또는 구현 결과가 아님',height=2250)
 p.node('screen',150,270,520,130,'같은 앱의 화면 / 포인터 / 선택',kind='model')
 p.node('app',930,270,520,130,'같은 앱의 공개 객체 API',kind='model')
 p.boundary('via',64,480,2432,1330,'VIA SOFTWARE','Context 획득/참조 부분만 확대',dashed=True)
 p.component('cm',110,550,1430,1190,'Context Manager',role='화면/객체의 획득, 공통 참조와 현재 자료의 해석')
 p.node('obs',170,720,490,130,'관측 조립 Module',owner='cm',color=BLUE)
 p.node('obj',930,720,550,130,'객체 제공자 Module',owner='cm',color=GREEN)
 p.node('ref',470,1090,710,160,'공통 대상 참조\n관측 근거 + 가능한 객체 ID/버전\n출처 / 유효 범위 / unknown',kind='data',owner='cm')
 p.node('resolve',470,1480,710,150,'참조 해석 Module\n자료 읽기 / 재확인 필요 반환',owner='cm')
 p.component('ri',1770,650,660,160,'Request Interpreter',role='같은 원문으로 지칭을 해석. 객체도 후보 근거')
 p.component('rc',1770,1090,660,160,'Request Controller',role='현재 대상/의도/권한 확인. 기존 확정 경계 유지')
 p.component('ag',1770,1510,660,160,'Agent Gateway',role='Agent capability에 맞는 자료로 변환/인계')
 p.text(120,440,'관측 수집은 Interaction Manager 경유',18,BLUE)
 p.edge('obs','screen',sp='T',tp='B',sd=-20,td=-20,via=[(395,480),(390,480)],color=BLUE)
 p.edge('screen','obs',sd=20,td=20,via=[(430,460),(435,460)],ret=True,color=BLUE)
 p.edge('obj','app',sp='T',tp='B',sd=-20,td=-20,via=[(1185,480),(1170,480)],color=GREEN)
 p.edge('app','obj',sd=20,td=20,via=[(1210,460),(1225,460)],ret=True,color=GREEN)
 p.edge('obs','ref',td=-170,via=[(415,980),(655,980)],color=BLUE,label='관측 참조 생성',at=(185,930))
 p.edge('obj','ref',td=170,via=[(1205,980),(995,980)],color=GREEN,label='가능하면 객체 참조 보강',at=(1010,930))
 p.edge('ref','resolve',label='같은 표현을 두 경로에서 사용',at=(490,1330))
 for key,lane,delta in [('ri',1570,-35),('rc',1630,0),('ag',1690,35)]:
  y=p.port(key,'L',-20)[1]
  p.edge(key,'resolve',sp='L',tp='R',sd=-20,td=delta,via=[(lane,y),(lane,1555+delta)])
  p.edge('resolve',key,sp='R',tp='L',sd=delta+12,td=20,via=[(lane+14,1567+delta),(lane+14,y+40)],ret=True)
 p.edge('rc','ri',sp='T',tp='B',sd=-20,td=-20,label='의미 요청 / 대상 반환',at=(1810,940))
 p.edge('ri','rc',sd=20,td=20,ret=True)
 p.edge('rc','ag',sd=-20,td=-20,label='Task Manager 경유: 확정 인계',at=(1790,1370))
 p.edge('ag','rc',sp='T',tp='B',sd=20,td=20,ret=True)
 p.text(110,1870,['객체 API가 없는 영역은 관측 참조 유지. 객체가 있어도 시점/유효성 확인은 필요.',
                   '추론은 기존 Model Access/공유 Omni를 사용. 외부 업무는 같은 Downstream Agent가 실행.',
                   '제공자와 resolver의 개발량은 남지만, 대화/Task 원본과 확정 구조를 유지하며 점진 도입 가능.'],24,MUTED)
 footer(p,'파랑 = 관측 경로 / 초록 = 객체 확장 / 검정 = 공통. 새 Component를 늘려야만 얻는 성질이 아니다.')
 return p

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args();drift=[]
 plates=[p for n in range(31,37) for p in (structural(n),event(n))]
 plates += [mechanism_case(31),mechanism_case(33),reference_extension()]
 for p in plates:
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
 print(f'PASS: {len(plates)} pairs; SVG/draw.io source parity, XML references, ownership, bounds and routes')
if __name__=='__main__':main()
