#!/usr/bin/env python3
"""04-41 only: model-led interpretation versus code-owned frame resolution.
The restored 04-31 generator is deliberately independent of this new comparison.
"""
import argparse
from pathlib import Path
import xml.etree.ElementTree as ET
from stage4_diagram_design import Plate, INK, BLUE, GREEN, MUTED
from request_resolution_presentation import structure, Slide
ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'docs/architecture/12-decisions/decision-packages/diagrams'
TITLE='요청 의미 확정 — 모델 주도 해석과 모델 틀/코드 완성'
NAMES=['Interaction\nManager','Request\nController','Request\nInterpreter','Model\nAccess','Cloud\nSemantic LLM','Context\nManager','Task\nManager','Policy\nManager','State\nStore','Agent\nGateway','Downstream\nAgent','Response\nManager','Cloud\nVoice Model']
class EventPlate(Slide):
 __init__=Plate.__init__
 validate=Plate.validate
 text_geometry=staticmethod(Plate.text_geometry)

# Every arrow is one logical message. RI's controller/frame/Engine are internal Modules.
def trace(option):
 b=option=='B'
 steps=[(0,0,'로컬 Turn-Taking Control: VAD 시작·종료 / 마이크·화면·포인터 시점 기록',False),
 (0,3,'계속 음성 스트림 전송; local VAD 확정 경계로 음성 buffer commit',False),
 (3,12,'cloud 음성 입력·전사 요청; 자동 업무 응답은 이 비교에서 제외',False),
 (12,3,'전사 완료/실패와 item ID; 발화 종료와 별개',True),(3,0,'전사 + 입력 item 연결. 실제 음성/화면의 local 시점으로 근거 연결',True),
 (0,1,'1  확정 발화/근거/입력 버전. 대상 Task와 신규/보완 관계는 미결정',False),
 (1,2,'2  같은 최근 대화/실제 게시 질문 + 원문. '+('유한한 버전 지정 요청 틀' if b else '사용 가능한 제한 읽기 도구 계약'),False),
 (2,3,'요청 틀 해석기: 초기 틀 해석 요청' if b else '모델 주도 해석 제어기: 읽기/질문/완성 의미 추론 요청',False),
 (3,4,'cloud semantic 추론. 음성 입력을 막는 전체 호출 mutex 없음',False),
 (4,3,'3  '+('목표·대상·Task·조건이 미결정인 typed frame' if b else '다음 행동 제안: 당시 표와 관련 업무 조회'),True),(3,2,'모델 결과 반환; 출력은 VIA 코드가 형식·범위 검사',True)]
 if b:steps +=[(2,2,'Engine 코드가 틀 검사 → 미해결 항목에 필요한 조회 구성',False)]
 else:steps +=[(2,2,'읽기 도구 실행기가 모델이 선택한 조회의 범위/예산 검사',False)]
 steps +=[(2,7,'읽기 범위의 현재 권한 확인',False),(7,2,'허용 범위 또는 거절',True),
 (2,5,'4  공통 읽기 도구 실행기: 당시 선택/화면과 관련 대화/질문 조회',False),(5,2,'근거/버전/coverage 또는 조회 실패',True),
 (2,6,'공통 읽기 도구 실행기: 관련 보고서 업무 후보 조회',False),(6,2,'예산/실적 보고서 후보와 현재 상태/버전',True)]
 if b:steps +=[(2,2,'조회 결과는 Engine으로. 코드가 표 연결; 보고서 둘은 아직 미해결',False),
 (2,2,'조건부: 범위 지정 해석이 필요하면 Engine → 요청 틀 해석기 → Model Access → semantic LLM → 역순 반환',False),
 (2,2,'이 사례는 추가 부분 추론 없이 질문. 임의 규칙 실행/모델의 전체 binding 생성은 순수 B가 아님',False)]
 else:steps +=[(2,3,'조회 근거로 다음 조회/질문/완성 의미 재판단 요청',False),(3,4,'semantic 추론 세션에 근거 제공',False),
 (4,3,'표 연결 + 보고서 두 후보 + 보고서 결과로 메일 초안/발송 금지 제안',True),(3,2,'전체 의미/질문 제안 반환',True)]
 steps +=[(2,2,'공통 의미 제안 검증기: ID·형식·금지 조건·근거/버전 검사',False),
 (2,1,'5  질문 제안: 예산 보고서인가, 실적 보고서인가?',True),
 (1,1,'현재 입력·질문·근거/권한 검사 → 채택 레코드 저장 준비',False),
 (1,8,'질문·후보·근거/버전 레코드 저장 요청',False),(8,1,'저장 성공/실패',True),
 (1,1,'저장 성공 시 질문 채택; 실패는 게시/인계 보류',False),
 (1,11,'저장 성공·현재 유효 시 허용된 질문 게시 요청',False),(11,3,'승인된 질문 내용의 음성 생성 요청',False),(3,12,'cloud voice 음성 rendering 요청',False),
 (12,3,'질문 음성/전사 반환; response.done은 생성 종료',True),(3,11,'음성 결과; 내용·현재성 확인',True),(11,0,'게시 허용 Text/음성 전달',False),
 (0,11,'실제 표시·재생·중단 구간 기록. 생성 완료와 구분',True),(11,1,'실제 게시 기록; 질문 focus 연결',True),
 (0,1,'6  새 확정 사용자 입력: “예산 보고서”. 실제 게시 질문에 대한 연결 후보',False),
 (1,2,'현재 질문/후보와 답변, 새 입력 버전; 앞선 발송 금지 유지',False),
 (2,3,'답변 의미 해석 요청',False),(3,4,'같은 cloud semantic API, 별도 요청 context',False),
 (4,3,'답변 의미: '+('예산 보고서라는 후보 제한 표현' if b else 'Task1 예산 보고서 + 표 삽입 + 결과로 메일 초안/발송 금지'),True),(3,2,'해석 결과 반환',True)]
 if b:steps +=[(2,2,'틀 해석기 → Engine. 코드가 질문/후보를 확인해 전체 binding 완성',False)]
 else:steps +=[(2,2,'모델의 의미 제안 검사, 조건 보존과 검증 가능한 부분 수정',False)]
 steps +=[(2,2,'공통 검증기를 통과한 의미 제안. RI의 해석 상태는 transient',False),
 (2,1,'7  완성 의미 + 근거/Task/질문 버전 제안',True),
 (1,7,'현재 자료/실행 인계 권한 검사',False),(7,1,'현재 허용 또는 거절',True),
 (1,5,'사용한 자료 버전의 현재 유효성 확인',False),(5,1,'유효/변경/철회 반환',True),
 (1,6,'Task/질문의 현재 상태 확인',False),(6,1,'현재 상태와 버전',True),
 (1,1,'현재 입력·자료·Task·질문 검사. 불일치면 인계 보류/재평가',False),
 (1,8,'RC 의미·질문 관계·근거·revision 레코드 durable 저장 요청',False),(8,1,'저장 성공/실패; RI Engine이 원본/별도 채택 상태를 저장하지 않음',True),
 (1,1,'저장 성공 시 의미 채택; 실패는 게시/인계 보류',False),
 (1,6,'8  저장 성공·현재 유효 시 조건부 Task 생성/변경 + command ID. 새 버전이면 거절',False),
 (6,9,'허용된 요청/조건/참조와 command ID; 보고서 결과 → 메일 초안 관계 유지',False),(9,10,'외부 업무 위임. 업무 계획/도구 실행은 Agent 책임',False),
 (10,9,'접수 결과. 완료와 구별',True),(9,6,'해당 Task의 접수/거절/불명',True),(6,1,'확인된 Task 상태',True),
 (1,11,'확인된 상태의 사용자 응답 준비 요청',False),(11,3,'조건부 응답 내용 구성: template이면 semantic 호출 생략 가능',False),
 (3,4,'호출이 필요한 경우만 semantic 응답 구성',False),(4,3,'응답 제안',True),(3,11,'내용 결과; 현재성·허용 범위 검사',True),
 (11,3,'승인된 내용의 음성 rendering 요청',False),(3,12,'cloud voice 요청',False),(12,3,'음성 결과',True),(3,11,'음성/전사 결과',True),
 (11,0,'게시 허용 Text/음성. 접수 상태를 작업 완료로 꾸미지 않음',False),
 (0,11,'실제 표시·재생·중단 receipt',True),(11,1,'실제 게시 기록 반환',True)]
 h=620+len(steps)*59
 p=EventPlate('choice41-event-'+option.lower(),'04-41',TITLE,option+'  같은 정상 사례 / local VIA·VAD + cloud 음성·semantic 모델 / 뜻의 제안과 채택 분리',height=h)
 xs=[145+188*i for i in range(len(NAMES))]
 for i,name in enumerate(NAMES):
  xx=xs[i]
  if i in (4,10,12):p.node(option+'external'+str(i),xx-87,260,174,100,name,kind='model' if i in (4,12) else 'external')
  else:
   p.box(xx-87,260,174,100,'white','#000000',0,thick=1.3)
   p.text(xx,273,name,18,'#000000','center',True,leading=25)
  p.line([(xx,360),(xx,h-170)],'#C7D1DD',dashed=True,arrow=False,width=1.3)
 for j,(a,z,label,ret) in enumerate(steps):
  yy=425+j*59
  label_width=sum(19 if ord(ch)>=0x2e80 else 11 for ch in label)
  label_x=max(64,min(min(xs[a],xs[z]),2496-label_width))
  p.label(label_x,yy-28,label,'#000000',size=19)
  pts=[(xs[a],yy),(xs[z],yy)] if a!=z else [(xs[a],yy),(xs[a]+44,yy),(xs[a]+44,yy+17),(xs[a],yy+17)]
  p.line(pts,'#000000',dashed=ret,width=1.3)
 # EventPlate supplies the same black marker and white external-model notation.
 for item in p.items:
  if item['kind']=='line' and item['color']==INK:item['color']='#000000'
 p.text(64,h-135,'모든 Component와 상태 권한은 local VIA / 모델은 외부 cloud dependency / Task 계획·실행은 외부 Agent 책임.',19,MUTED)
 p.text(64,h-95,'B Engine과 틀 해석기는 Request Interpreter 내부 Module. 해석 상태는 임시이며 채택·질문 원본의 owner는 RC.',19,MUTED)
 p.text(64,h-55,'사각=Component / 육각=외부 모델·Agent / 실선=요청·전달 / 점선=반환. 호출 분기는 조건부, 수·시간은 측정값 아님.',19,MUTED)
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
