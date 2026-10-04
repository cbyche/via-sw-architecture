#!/usr/bin/env python3
"""04-41 only: model-led interpretation versus code-owned frame resolution.
The restored 04-31 generator is deliberately independent of this new comparison.
"""
import argparse
from pathlib import Path
import xml.etree.ElementTree as ET
from stage4_diagram_design import Plate, INK, BLUE, GREEN, MUTED
ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'docs/architecture/12-decisions/decision-packages/diagrams'
TITLE='요청 의미 확정 — 모델 중심 ReAct와 모델 틀/코드 완성'


def structure():
 p=Plate('choice41-structure','04-41',TITLE,'같은 발화와 자료: “이 표를 아까 보고서에 넣고, 메일은 초안만” → 업무 확인 → 같은 현재 검사와 위임',height=3340)
 for option,x in [('A',64),('B',1312)]:
  c=BLUE if option=='A' else GREEN
  p.text(x,235,option+'  '+('모델이 조회와 최종 관계를 결정' if option=='A' else '모델은 틀 제공, 코드는 조회와 관계 결합'),27,c,bold=True)
  p.text(x,278,'유리: 열린 표현의 공동 해석 / 대가: 반복 추론과 잘못된 연결 검증' if option=='A' else '유리: 명시 관계의 추적/통제 / 대가: 표현 범위 제한과 엔진 확장',18,MUTED)
  p.box(x,315,1184,2285,'#FAFBFD','none',0)
  p.box(x,2600,760,575,'#FAFBFD','none',0)
  p.line([(x,315),(x+1184,315),(x+1184,2600),(x+760,2600),(x+760,3175),(x,3175),(x,315)],'#C7D1DD',dashed=True,arrow=False)
  p.text(x+22,330,'VIA SOFTWARE / 논리 책임 경계',18,INK,bold=True)
  def cp(k,xx,y,w,h,name,color=INK):p.component(option+k,x+xx,y,w,h,name,color)
  def mod(k,xx,y,w,h,name,owner):p.node(option+k,x+xx,y,w,h,name,owner=option+owner,color=c)
  def st(k,xx,y,w,h,name,owner):p.node(option+k,x+xx,y,w,h,name,kind='store',owner=option+owner,color=c)
  def edge(a,b,label,at,sp='B',tp='T',via=(),ret=False,sd=0,td=0,color=None):
   p.edge(option+a,option+b,sp=sp,tp=tp,via=[(x+xx,y) for xx,y in via],label=label,at=(x+at[0],at[1]),ret=ret,sd=sd,td=td,color=color or c)
  cp('im',30,390,630,85,'Interaction Manager')
  cp('rc',30,565,630,90,'Request Controller')
  edge('im','rc','1  발화/시점/버전; 신규/보완은 미결정',(65,505),color=INK)
  cp('ma',800,830,350,85,'Model Access')
  # Model is outside VIA software responsibility, even though on the same PC.
  p.node(option+'omni',x+800,3185,350,85,'공유 Omni',kind='model')
  p.text(x+800,3290,'모델 의존성 / PC에서 한 벌 공유',17,MUTED)
  # Explicit MA/Omni pair is expanded in the sequence view; route on outside edge.
  edge('ma','omni','추론 입력',(1045,1040),sp='R',tp='R',via=[(1160,872.5),(1160,3227.5)],sd=0,td=0,color=INK)
  edge('omni','ma','추론 결과',(1045,1120),sp='R',tp='R',via=[(1175,3247.5),(1175,892.5)],sd=20,td=20,ret=True,color=INK)
  cp('cm',800,1450,350,85,'Context Manager')
  cp('tm',800,1780,350,85,'Task Manager')
  if option=='A':
   cp('ri',30,815,630,1215,'Request Interpreter')
   mod('loop',110,970,470,105,'ReAct 해석 제어기','ri')
   mod('tools',110,1260,470,105,'읽기 도구 실행기','ri')
   st('temp',110,1530,470,85,'임시 해석 상태','ri')
   mod('check',110,1820,470,100,'의미 제안 검증기','ri')
   edge('rc','loop','2  같은 원문/최근 안내 + 읽기 도구 계약',(65,720))
   edge('loop','ma','추론 요청',(670,957),sp='R',tp='L',via=[(720,1007.5),(720,857.5)],sd=-15,td=-15,color=INK)
   edge('ma','loop','조회/질문/완성안',(670,1022),sp='L',tp='R',via=[(755,887.5),(755,1037.5)],sd=15,td=15,ret=True,color=INK)
   edge('loop','tools','3  모델이 선택한 조회',(125,1135),sd=-95,td=-95)
   edge('tools','loop','4  결과로 모델 재판단',(395,1200),sp='T',tp='B',sd=95,td=95,ret=True)
   edge('tools','temp','근거/조회 기록 보관',(150,1430))
   edge('temp','loop','유효 기록 회수',(40,1655),sp='L',tp='L',via=[(70,1572.5),(70,1022.5)],ret=True)
   edge('loop','check','5  모델의 최종 관계\n또는 질문 제안',(45,1740),sp='L',tp='L',via=[(48,1002.5),(48,1850)],sd=-20,td=-20)
   edge('check','loop','검증 오류: 제한 재생성',(575,1635),sp='R',tp='R',via=[(630,1870),(630,1022.5)],ret=True)
   read='tools';readY=1312.5
   result='check'
  else:
   cp('ri',30,815,630,150,'Request Interpreter')
   edge('rc','ri','2  같은 원문/최근 안내 + 요청 틀 형식',(65,720))
   edge('ri','ma','틀/부분 해석 요청',(670,800),sp='R',tp='L',via=[(735,875),(735,857.5)],sd=-15,td=-15,color=INK)
   edge('ma','ri','틀/후보/미해석',(670,925),sp='L',tp='R',via=[(770,887.5),(770,905)],sd=15,td=15,ret=True,color=INK)
   cp('engine',30,1130,630,900,'Request Resolution Engine',c)
   mod('planner',110,1260,470,105,'미해결 항목 처리기','engine')
   st('state',110,1530,470,85,'요청 해석 상태','engine')
   mod('bind',110,1820,470,100,'관계 결합기','engine')
   edge('ri','planner','3  모델의 요청 틀: 대상/Task/조건은 미결정 가능',(65,1030))
   edge('planner','ri','4  부분 해석이 필요하면\n코드가 범위 지정 후 요청',(35,1055),sp='L',tp='L',via=[(10,1312.5),(10,890)])
   edge('planner','state','조회한 후보/근거 버전',(130,1430))
   edge('state','bind','허용된 관계/조건으로 결합',(130,1710))
   edge('bind','planner','후보 부족/충돌: 코드가\n추가 조회 또는 질문 결정',(590,1680),sp='R',tp='R',via=[(630,1870),(630,1312.5)],ret=True)
   read='planner';readY=1312.5;result='bind'
  for k,yy,dy,lane in [('cm',1492.5,-20,705),('tm',1822.5,20,750)]:
   edge(read,k,'4  자료 조회' if k=='cm' else '4  Task 조회',(lane+5,yy-55),sp='R',tp='L',via=[(lane,readY+dy),(lane,yy-15)],sd=dy,td=-15)
   edge(k,read,'근거/버전' if k=='cm' else '사실/버전',(lane+25,yy+25),sp='L',tp='R',via=[(lane+20,yy+15),(lane+20,readY+dy+10)],sd=15,td=dy+10,ret=True)
  p.text(x+800,1950,['동일 자료/권한', '추가 해석도 같은 Omni', '없는 근거는 미해결'],18,MUTED)
  cp('out',30,2160,630,90,'Request Controller')
  edge(result,'out','5  완성 의미 / 확인 질문 / 지원 한계',(60,2065),ret=True)

  cp('tmout',800,2160,350,90,'Task Manager')
  cp('ag',800,2440,350,85,'Agent Gateway')
  p.node(option+'agent',x+800,2695,350,85,'Downstream Agent',kind='external')
  # Inner boundary indicates external agent responsibility rather than a VIA component.
  p.text(x+800,2805,'외부 업무 실행 책임',18,MUTED)
  edge('out','tmout','7  현재 검사 후\n8  Task 생성/변경',(665,2110),sp='R',tp='L',sd=-15,td=-15,color=INK)
  edge('tmout','out','접수/상태',(680,2255),sp='L',tp='R',sd=15,td=15,ret=True,color=INK)
  edge('tmout','ag','8  채택한 요청 인계',(805,2320),sd=-35,td=-35,color=INK)
  edge('ag','tmout','접수/결과',(1010,2370),sp='T',tp='B',sd=35,td=35,ret=True,color=INK)
  edge('ag','agent','실제 작업 위임',(805,2590),sd=-35,td=-35,color=INK)
  edge('agent','ag','접수/결과',(1010,2640),sp='T',tp='B',sd=35,td=35,ret=True,color=INK)
  cp('rm',30,2550,630,85,'Response Manager')
  cp('imout',30,2850,630,85,'Interaction Manager')
  edge('out','rm','5  미해결이면 질문 / 8  확인된 상태 전달',(65,2320),sd=-95,td=-95,color=INK)
  edge('rm','out','실제 게시 기록',(420,2470),sp='T',tp='B',sd=95,td=95,ret=True,color=INK)
  edge('imout','rc','6  사용자 답변은 새 입력',(15,2410),sp='L',tp='L',via=[(4,2892.5),(4,610)],color=INK)
  edge('rm','imout','질문/응답 Text와 음성',(65,2720),sd=-95,td=-95,color=INK)
  edge('imout','rm','실제 전달 구간',(420,2800),sp='T',tp='B',sd=95,td=95,ret=True,color=INK)
  p.text(x+30,2990,['A: 모델의 다음 판단을 코드가 실행. 완성안은 검사 후 채택.' if option=='A' else 'B: 코드가 틀을 해결. 표현 밖 관계는 질문/미지원으로 반환.',
                         '현재 권한은 Policy Manager에 확인. 응답 추론은 Model Access 경유.',
                         '채택 기록은 State Store 보관. 이 공통 왕복은 사건도에서 펼침.'],19,MUTED,leading=31)
 p.text(64,3200,'검정: 공통 / 파랑: A / 초록: B. 네모: Component/Module 이름.',18,MUTED)
 p.text(64,3235,'실선: 요청/전달 / 점선: 반환. 원통: 상태. 별도 process 가정 없음.',18,MUTED)
 p.text(64,3270,'같은 이름은 동일 인스턴스. 그림은 설계안이며 구현/측정 결과가 아님.',18,MUTED)
 return p

NAMES=['Interaction\nManager','Request\nController','Request\nInterpreter','Model\nAccess','공유 Omni','Context\nManager','Task\nManager','Request\nResolution\nEngine','Policy\nManager','Agent\nGateway','Downstream\nAgent','Response\nManager']
# Every arrow is one concrete message; notes are separate from component names.
def trace(option):
 b=option=='B';c=GREEN if b else BLUE
 steps=[(0,1,'1  원문/입력 버전/선택 시점. 관련 Task와 신규/보완 관계는 미결정',False),
 (1,2,'2  같은 최근 대화/게시 안내와 원문. '+('요청 틀 형식' if b else '사용 가능한 제한 조회 도구'),False),
 (2,3,'원문 해석 요청',False),(3,4,'같은 Omni의 해석 세션에 입력',False),
 (4,3,'3  '+('목표 + 화면 지칭 + 기존 업무 지칭 + 금지 조건의 미해결 틀' if b else '모델의 선택: 화면 자료와 관련 업무 조회'),True),
 (3,2,'모델 결과 반환',True)]
 owner=7 if b else 2
 if b:steps +=[(2,7,'틀 전달. 임의 코드/새 규칙은 허용하지 않음',False),(7,7,'틀 검사 → 미해결 항목의 종류로 조회 구성',False)]
 else:steps +=[(2,2,'읽기 도구 실행기가 선택된 조회의 범위/형식을 검사',False)]
 steps += [(owner,8,'읽기 범위의 현재 권한 확인',False),(8,owner,'허용 범위 또는 거절',True),
 (owner,5,'4  당시 선택/화면과 관련 대화/질문 조회',False),(5,owner,'근거/버전/coverage 또는 조회 실패',True),
 (owner,6,'관련 보고서 업무 후보 조회',False),(6,owner,'보고서 Task 두 개와 현재 상태/버전',True)]
 if b:steps += [(7,7,'관계 결합: 표 참조는 결합, 보고서 둘은 아직 미해결',False)]
 else:steps += [(2,3,'조회 결과로 재판단 요청',False),(3,4,'근거를 해석 세션에 제공',False),
 (4,3,'표 연결 + 보고서 두 후보 + 메일 초안/금지 조건 제안',True),(3,2,'전체 의미/질문 제안 반환',True),
 (2,2,'ID/조건/근거 검증. 근거 없는 단정은 질문으로 반환',False)]
 steps += [(owner,1,'5  질문 제안: 예산 보고서인가, 실적 보고서인가?',True),
 (1,11,'현재 질문/입력 확인 후 게시 요청',False),
 (11,3,'질문 Text에 대응하는 음성 생성 요청',False),(3,4,'분리된 응답 세션에서 질문 음성 생성',False),
 (4,3,'질문 음성 반환',True),(3,11,'질문 음성 결과',True),(11,0,'후보를 Text/음성으로 제시',False),
 (0,11,'실제 게시 구간',True),(11,1,'게시 기록',True)]
 if b:steps +=[(1,7,'해당 질문의 게시 기록/대기 상태 채택 통지',False)]
 steps +=[(0,1,'6  새 사용자 입력: “예산 보고서”. 실제 게시 질문에 대한 연결 후보',False),
 (1,2,'같은 답변/입력 버전과 현재 질문 안내',False),
 (2,3,'답변 해석 요청',False),(3,4,'질문/후보와 답변 해석',False),
 (4,3,'답변 의미 반환: '+('예산 보고서라는 후보 제한 표현' if b else '예산 Task 연결 + 표 삽입 + 메일 초안/발송 금지'),True),
 (3,2,'해석 결과 반환',True)]
 if b:steps +=[(2,7,'변경 틀 전달. 질문 버전/후보 검사',False),(7,7,'코드로 Task 후보 결합. 앞선 금지 조건 유지',False)]
 else:steps +=[(2,2,'관계/조건 보존 검사, 필요하면 검증 가능한 부분 수정',False)]
 steps +=[(owner,1,'7  완성 의미 + 근거/Task 버전 제안',True),
 (1,8,'현재 자료 제공/실행 인계 권한 검사',False),(8,1,'현재 허용 또는 거절',True),
 (1,5,'사용한 자료 버전의 현재 유효성 확인',False),(5,1,'유효/변경/철회 반환',True),
 (1,6,'Task/질문의 현재 상태 확인',False),(6,1,'현재 상태와 버전',True),
 (1,1,'입력/질문과 채택 버전 검사. 불일치면 인계하지 않고 재평가',False)]
 if b:steps +=[(1,7,'채택 버전 통지; Engine 소유 상태를 State Store에 보관',False),(7,1,'저장 성공/실패 반환. 실패면 외부 인계 보류',True)]
 else:steps +=[(1,1,'채택 의미/질문 연결을 State Store에 보관. 실패면 인계 보류',False)]
 steps +=[(1,6,'8  조건부 Task 생성/변경 + command ID. 버전이 또 바뀌면 거절',False),
 (6,9,'허용된 요청/조건/참조와 command ID',False),(9,10,'외부 업무 위임. 업무 계획/도구 실행은 Agent 책임',False),
 (10,9,'접수 결과. 완료와 구별',True),(9,6,'해당 Task의 접수/거절/불명',True),(6,1,'확인된 상태',True),
 (1,11,'사용자에게 확인된 상태 전달 요청',False),(11,3,'필요한 응답 구성 요청',False),
 (3,4,'같은 Omni의 분리된 응답 세션',False),(4,3,'응답 결과',True),(3,11,'Text/음성 구성 결과',True),
 (11,0,'같은 사용자 출력: 접수 상태/질문/보류. 작업 완료로 꾸미지 않음',False),
 (0,11,'실제 전달 기록',True),(11,1,'게시 기록 반환',True)]
 if b:steps +=[(1,7,'관련 질문/채택 상태에 실제 게시 기록 연결',False)]
 h=640+len(steps)*59
 p=Plate('choice41-event-'+option.lower(),'04-41',TITLE,option+'  같은 사례의 정상 조회 → 보고서 확인 질문 → 보완 → 현재 검사 → 위임 → 실제 사용자 전달',height=h)
 xs=[150+205*i for i in range(12)]
 for i,name in enumerate(NAMES):
  if not b and i==7:continue
  xx=xs[i]
  if i==4:p.node(option+'omni',xx-94,260,188,100,name,kind='model')
  else:
   p.box(xx-94,260,188,100,'white',c if i==(7 if b else 2) else INK,4)
   p.text(xx,271,name,18,INK,'center',True,leading=25)
  p.line([(xx,360),(xx,h-170)],'#C7D1DD',dashed=True,arrow=False)
 for j,(a,z,label,ret) in enumerate(steps):
  yy=425+j*59
  label_width=sum(19 if ord(ch)>=0x2e80 else 11 for ch in label)
  label_x=max(64,min(min(xs[a],xs[z]),2496-label_width))
  p.label(label_x,yy-28,label,c,size=19)
  pts=[(xs[a],yy),(xs[z],yy)] if a!=z else [(xs[a],yy),(xs[a]+44,yy),(xs[a]+44,yy+17),(xs[a],yy+17)]
  p.line(pts,c,dashed=ret,width=2.2)
 p.text(64,h-135,'같은 열/시작/끝. A의 Engine 열은 비움. 모델 가중치 한 벌, 해석/응답 세션은 분리. 채택 상태 보관은 State Store.',19,MUTED)
 p.text(64,h-95,'B의 조건부 부분 해석: Engine → Interpreter → Model Access → Omni → 역순 후보 반환 → Engine 결합. 이 사례는 추가 해석 없이 질문.',19,MUTED)
 p.text(64,h-55,'실선=전달/요청, 점선=반환. 논리 순서를 수작업으로 추적한 설계이며 호출 횟수/간격은 성능 측정값이 아님.',19,MUTED)
 return p


def main():
 ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args()
 plates=[structure(),trace('A'),trace('B')];drift=[]
 for p in plates:
  p.validate()
  for i in p.items:
   if i['kind']=='component':assert not i['role']
  for ext,value in [('svg',p.svg()),('drawio',p.drawio())]:
   root=ET.fromstring(value)
   if ext=='drawio':
    ids=[v.get('id') for v in root.findall('.//mxCell')];assert len(ids)==len(set(ids))
    for v in root.findall('.//mxCell'):
     for attr in ('source','target','parent'):assert not v.get(attr) or v.get(attr) in ids
   dest=OUT/(p.slug+'.'+ext)
   if args.check:
    if not dest.exists() or dest.read_text()!=value:drift.append(str(dest))
   else:dest.write_text(value)
 if drift:raise SystemExit('Diagram drift: '+', '.join(drift))
 print('PASS: 04-41 three SVG/draw.io pairs; parity, XML, ownership, bounds and routes')
if __name__=='__main__':main()
