#!/usr/bin/env python3
"""Stage 4 review figures: one geometry source for editable draw.io and SVG.

No VIA candidate is executed. Reuses the repository's paired figure serializer.
"""
from __future__ import annotations
import argparse
import html
import xml.etree.ElementTree as ET
from generate_decision_package_diagrams import Page, OUT, ROOT, BLUE

PAGES=[]
def page(n,title,subtitle,count=2):
 p=Page(f'stage4-s{n:02}-comparison',f'S-{n:02}  |  {title}',subtitle)
 p.texts[-1]['lines']=['VIA / Stage 4 / 구조 탐색, 미채택 대안 / 구현 및 측정 결과 아님']
 step=1264 if count==2 else 836; width=1200 if count==2 else 792
 for i,label in enumerate(['T  실제 reviewed target','A  대안 구조','B  기능을 제한한 대안'][:count]):
  x=48+i*step;p.group(f'panel{i}',x,165,width,1130,'')
  p.text(f'panel-title{i}',x+24,215,label,28,bold=True)
 p.text('legend',48,1345,'박스: Component, 서브시스템, 논리 실행체 또는 저장소 이름 / 번호 화살표: 동작과 순서 / 검정: 공통 / 파랑: 달라지는 구조',21)
 PAGES.append(p);return p

class Area:
 def __init__(self,p,i,count=2):self.p=p;self.x=48+i*(1264 if count==2 else 836);self.id=f'{i}-'
 def n(self,k,x,y,w,h,title,*lines,blue=False,kind='component'):
  return self.p.node(self.id+k,self.x+x,y,w,h,title,*lines,blue=blue,kind=kind)
 def g(self,k,x,y,w,h,title,blue=False,kind='group'):
  return self.p.group(self.id+k,self.x+x,y,w,h,title,blue=blue,kind=kind)
 def e(self,k,s,t,sp='B',tp='T',via=(),label='',at=None,blue=False,dash=False):
  self.p.edge(self.id+'edge-'+k,self.id+s,self.id+t,sp,tp,[(self.x+x,y) for x,y in via],label,(self.x+at[0],at[1]) if at else None,blue,dash)
 def t(self,k,x,y,lines,size=20,blue=False):self.p.text(self.id+k,self.x+x,y,lines,size,color=BLUE if blue else '#171717')

def semantic():
 p=page(1,'추가 근거가 필요할 때 해석을 어떻게 이어가는가','①~⑤ 기본 Context 구성은 공통 / T는 첫 해석을 끝내고 다시 호출 / A는 실행을 멈췄다가 같은 실행을 재개 (지원 미확인)')
 next(t for t in p.texts if t['id']=='legend')['lines']=['박스: Component 또는 논리 실행체 이름 / 번호 화살표: 동작과 순서 / 검정: 공통 / 파랑: 달라지는 구조']
 a=Area(p,0);b=Area(p,1)
 for q in (a,b):
  q.n('im',400,260,400,65,'Interaction Manager',kind='box')
  q.n('rc',400,400,400,75,'Request Controller',kind='box')
  q.n('cm',40,600,380,70,'Context Manager',kind='box')
  q.n('src',40,850,380,75,'원본 source / owner read port',kind='box')
  q.n('ma',760,850,400,70,'Model Access',kind='box')
  q.e('input','im','rc',label='① 최종 입력과 근거 전달',at=(620,365))
  q.e('base-request','rc','cm','L','T',via=[(250,437.5),(250,560),(230,560)],label='② 기본 Context 요청',at=(55,520))
  q.e('source-read','cm','src',label='③/⑪ 원본 읽기',at=(55,785))
  q.e('source-return','src','cm','R','R',via=[(450,887.5),(450,635)],label='④/⑫ 원문, revision 반환',at=(460,790),dash=True)
  q.e('base-return','cm','rc','R','B',via=[(560,635),(560,520),(600,520)],label='⑤ 기본 Context, receipt 반환',at=(570,570),dash=True)
  q.t('common',40,1188,['공통 ①~⑤: 두 안 모두 최초 해석 전에 같은 기본 Context를 구성한다.','기본 Context는 현재 입력, 선택과 화면 근거, 최근 응답, 관련 Task 후보 등 허용된 최소 근거다.'],19)
 a.n('executor',760,600,400,70,'Request Interpreter',blue=True,kind='box')
 a.e('first-call','rc','executor','R','T',via=[(920,437.5),(920,560),(960,560)],label='⑥ 첫 해석 요청',at=(935,520),blue=True)
 a.e('model-call','executor','ma',label='⑦/⑮ semantic 호출',at=(770,785),blue=True)
 a.e('model-return','ma','executor','L','L',via=[(700,885),(700,635)],label='⑧/⑯ model result',at=(500,810),blue=True,dash=True)
 a.e('intermediate','executor','rc','R','R',via=[(1180,635),(1180,437.5)],label='⑨ 추가 읽기 제안 반환',at=(900,705),blue=True,dash=True)
 a.e('extra-read','rc','cm','L','L',via=[(18,437.5),(18,635)],label='⑩ 범위와 예산 확인 후 추가 읽기',at=(36,475),blue=True)
 a.e('extra-return','cm','rc','R','B',via=[(500,635),(500,500),(600,500)],label='⑬ 추가 근거, receipt 반환',at=(510,610),blue=True,dash=True)
 a.e('second-call','rc','executor','R','B',via=[(1140,437.5),(1140,720),(960,720)],label='⑭ 두 번째 해석 요청',at=(900,745),blue=True)
 a.e('final','executor','rc','T','R',via=[(960,540),(840,540),(840,437.5)],label='⑰ 최종 제안 반환',at=(850,575),blue=True,dash=True)
 a.t('commit',560,1000,['⑱ Request Controller가 현재 revision과 receipt를 확인한다.','조건이 맞으면 Semantic Commit, 아니면 질문 또는 실패로 보낸다.','차이: 첫 해석은 끝나며, 추가 근거를 넣은 새 호출이 시작된다.'],19,True)
 b.n('executor',760,600,400,70,'Semantic Resolution Worker',blue=True,kind='box')
 b.n('broker',460,720,260,65,'Capability Read Broker',blue=True,kind='box')
 b.e('start','rc','executor','R','T',via=[(920,437.5),(920,560),(960,560)],label='⑥ Worker 시작',at=(955,520),blue=True)
 b.e('model-call','executor','ma',label='⑦ 시작 / ⑮ 재개',at=(790,785),blue=True)
 b.e('model-return','ma','executor','R','R',via=[(1180,885),(1180,635)],label='⑧ READ_REQUIRED / ⑯ 완료',at=(895,810),blue=True,dash=True)
 b.e('read-request','executor','broker','L','R',via=[(740,635),(740,752.5)],label='⑨ 읽기 요청',at=(610,620),blue=True)
 b.e('broker-request','broker','cm','L','B',via=[(350,752.5),(350,720),(230,720)],label='⑩ 권한 확인 후 조회',at=(145,745),blue=True)
 b.e('broker-return','cm','broker','R','T',via=[(590,635)],label='⑬ 근거, receipt 반환',at=(455,660),blue=True,dash=True)
 b.e('resume','broker','executor','T','B',via=[(590,690),(960,690)],label='⑭ handle에 근거 주입',at=(760,715),blue=True)
 b.e('final','executor','rc','T','R',via=[(960,540),(840,540),(840,437.5)],label='⑰ 최종 제안 반환',at=(850,575),blue=True,dash=True)
 b.t('commit',560,1000,['⑱ Request Controller가 현재 revision과 receipt를 확인한다.','조건이 맞으면 Semantic Commit, 아니면 질문 또는 실패로 보낸다.','차이: 읽기 동안 연산을 양보하고, 근거를 받은 뒤 같은 실행을 재개한다.'],19,True)

def screen():
 p=page(2,'말한 당시의 화면 근거를 어떻게 만드는가','공통 Component는 같은 위치 / T는 원시 timeline / A는 사전 객체 이력 / B는 명시 선택 package만 지원',3)
 for i in range(3):
  a=Area(p,i,3)
  a.n('events',170,270,452,70,'발화와 화면 사건',kind='box')
  a.n('im',170,465,452,70,'Interaction Manager',kind='box')
  a.n('cm',170,1050,452,70,'Context Manager',kind='box')
  if i<2:a.e('capture','events','im',label='① 발화 시각과 화면 사건 수집',at=(220,410))
  if i==0:
   a.n('evidence',170,745,452,80,'Raw Screen Timeline',blue=True,kind='store')
   a.e('keep','im','evidence',label='② 유한 pre-roll과 발화 구간 원시 관측 보존',at=(80,665),blue=True)
   a.e('read','evidence','cm',label='③ 요청 때 당시 관측과 늦은 전사를 결합',at=(65,975),blue=True)
   a.t('note',70,1185,['④ Context Manager가 gap, source revision과 현재 적용 가능성을 확인한다.','지원: 자연스러운 과거 지칭 / 비용: 원시 timeline 메모리와 요청 시 해석'],18,True)
  elif i==1:
   a.n('ground',170,650,452,70,'Screen Grounding Service',blue=True,kind='box')
   a.n('store',170,835,452,80,'Derived Object Store',blue=True,kind='store')
   a.e('derive','im','ground',label='② 요청 전에 객체와 관계 후보 생산',at=(130,605),blue=True)
   a.e('publish','ground','store',label='③ 객체 version과 source revision 게시',at=(105,790),blue=True)
   a.e('read','store','cm',label='④ 준비된 객체와 필요한 원시 근거 반환',at=(70,1005),blue=True)
   a.t('note',70,1185,['⑤ Context Manager가 늦은 전사와 결합하고 현재 원문을 다시 확인한다.','조건: 객체 추적 지원 / 비용: 상시 producer, 저장과 삭제 경로'],18,True)
  else:
   a.n('package',170,745,452,80,'Capture Package',blue=True,kind='store')
   a.e('select','events','im',label='① 사용자가 영역 또는 객체를 명시 선택',at=(185,410),blue=True)
   a.e('seal','im','package',label='② 선택 시점의 identity, revision과 crop 봉인',at=(95,665),blue=True)
   a.e('read','package','cm',label='③ 선택한 자료와 발화만 반환',at=(165,975),blue=True)
   a.t('note',70,1185,['④ Context Manager가 현재 source와 권한을 다시 확인한다.','제한: 선택 밖의 연속 지칭과 과거 복수 지칭은 미지원'],18,True)
 p.text('common',48,1380,'공통 후속: Context Manager → Request Controller로 근거와 receipt 반환 → 입력과 예산 결합 → Request Interpreter 호출.',20)

def retrieval():
 p=page(3,'숨은 자료의 후보를 어떻게 찾는가','공통 Request Controller와 Context Manager는 같은 위치 / T는 요청별 조회 / A는 사전 색인 생산과 검색 서비스')
 a=Area(p,0);b=Area(p,1)
 for q in (a,b):
  q.n('source',40,260,380,75,'원본 source / owner read port',kind='box')
  q.n('rc',760,260,400,75,'Request Controller',kind='box')
  q.n('cm',760,1030,400,75,'Context Manager',kind='box')
 a.n('cache',40,600,380,80,'기존 metadata/keyword index와 cache',kind='store')
 a.n('read',760,600,400,70,'Owner Read Adapter',blue=True,kind='box')
 a.e('request','rc','read',label='① 허용 scope와 이름, 기간, Task 단서 전달',at=(650,520))
 a.e('cache','cache','read','R','L',via=[(600,640),(600,635)],label='② 기존 index와 cache에서 후보 조회',at=(365,575))
 a.e('source','read','source','L','R',via=[(650,635),(650,297.5)],label='③ 후보 원문과 revision 읽기',at=(450,430))
 a.e('return','read','cm',label='④ 원문, coverage와 receipt 반환',at=(775,900),dash=True)
 a.t('note',450,1165,['⑤ Context Manager가 현재 권한과 identity를 확인해 Context를 조립한다.','별도 embedding helper, 색인 worker와 vector 저장소는 없다.'],19)
 b.n('worker',40,470,380,70,'Indexing Worker',blue=True,kind='box')
 b.n('embed',40,675,380,70,'Embedding Runtime',blue=True,kind='box')
 b.n('index',40,920,380,80,'Vector Index + Source Manifest',blue=True,kind='store')
 b.n('retrieve',760,570,400,70,'Semantic Retrieval Service',blue=True,kind='box')
 b.e('collect','source','worker',label='① source 변경과 삭제 수집',at=(65,420),blue=True)
 b.e('encode','worker','embed',label='② 문서 embedding 생성',at=(65,625),blue=True)
 b.e('publish','embed','index',label='③ 완성 generation을 원자 게시',at=(65,870),blue=True)
 b.e('query','rc','retrieve',label='④ 같은 허용 scope로 검색 요청',at=(765,500),blue=True)
 b.e('lookup','retrieve','index','L','R',via=[(620,605),(620,960)],label='⑤ 의미, lexical 후보 조회',at=(430,825),blue=True)
 b.e('candidates','index','retrieve','R','B',via=[(690,960),(690,760),(960,760)],label='⑥ 후보와 source revision 반환',at=(700,800),blue=True,dash=True)
 b.e('consume','retrieve','cm',label='⑦ 후보, coverage와 manifest 반환',at=(775,900),blue=True,dash=True)
 b.e('original-request','cm','source','L','T',via=[(650,1067.5),(650,200),(230,200)],label='⑧ 현재 원문과 권한 요청',at=(370,225),blue=True)
 b.e('original-return','source','cm','R','L',via=[(680,297.5),(680,1067.5)],label='⑨ 원문, revision과 권한 반환',at=(690,980),dash=True)
 b.t('note',710,1230,['⑩ Context Manager가 검증된 원문으로 Context를 조립한다.','후보 1위도 대상 확정이나 실행 권한은 아니다.'],19,True)

def workflow():
 p=page(4,'대기 중인 목표를 어디에 기록하고 어떻게 다시 잇는가','공통 Component는 같은 위치 / T는 domain graph / A는 workflow continuation / B는 세션 메모리만 사용',3)
 areas=[Area(p,i,3) for i in range(3)]
 for q in areas:
  q.n('source',170,250,452,70,'사용자 입력 / Task 결과 source',kind='box')
  q.n('rc',170,420,452,70,'Request Controller',kind='box')
  q.n('store',170,940,452,75,'State Store',kind='store')
  q.n('task',170,1135,452,70,'Task Manager → Agent Gateway',kind='box')
  q.e('initial','source','rc',label='① 확정된 목표 전달',at=(205,380))
 a,b,c=areas
 a.n('owner',170,650,452,105,'durable Request Graph / Pending User Interaction',blue=True,kind='module')
 a.e('start','rc','owner',label='② 목표 관계와 대기 조건 생성',at=(100,600),blue=True)
 a.e('persist','owner','store',label='③ graph, 질문 identity와 전이 저장',at=(85,885),blue=True)
 a.e('signal','source','owner','L','L',via=[(70,285),(70,702.5)],label='④ 사용자 답변 또는 Task 결과',at=(82,515),blue=True)
 a.e('propose','owner','rc','R','R',via=[(700,702.5),(700,455)],label='⑤ 후속 요청 제안 반환',at=(470,560),blue=True,dash=True)
 a.e('command','rc','task','R','R',via=[(735,455),(735,1170)],label='⑥ 현재성 확인 뒤 명령 전달',at=(420,1095))
 a.e('restore','store','owner','L','L',via=[(110,977.5),(110,702.5)],label='⑦ 재시작 뒤 graph 복원',at=(120,825),blue=True,dash=True)
 a.t('note',70,1240,['차이: 대기와 질문의 권위 원본이 VIA domain 상태다.','외부 실행 결과는 source에서 다시 확인한다.'],18,True)
 b.n('owner',170,650,452,105,'Interaction Workflow Runtime',blue=True,kind='process')
 b.e('start','rc','owner',label='② continuation 시작 signal',at=(140,600),blue=True)
 b.e('persist','owner','store',label='③ continuation, signal과 activity intent 저장',at=(55,885),blue=True)
 b.e('signal','source','owner','L','L',via=[(70,285),(70,702.5)],label='④ 답변 또는 결과 signal',at=(82,515),blue=True)
 b.e('propose','owner','rc','R','R',via=[(700,702.5),(700,455)],label='⑤ admission 요청 반환',at=(490,560),blue=True,dash=True)
 b.e('command','rc','task','R','R',via=[(735,455),(735,1170)],label='⑥ 현재성 확인 뒤 명령 전달',at=(420,1095))
 b.e('restore','store','owner','L','L',via=[(110,977.5),(110,702.5)],label='⑦ 재시작 뒤 continuation 복원',at=(105,825),blue=True,dash=True)
 b.t('note',70,1240,['차이: 대기와 재개의 권위 원본이 workflow continuation이다.','Task lifecycle과 Agent 실행 책임은 옮기지 않는다.'],18,True)
 c.n('owner',170,650,452,105,'Session Request Coordinator',blue=True,kind='module')
 c.e('start','rc','owner',label='② 세션 메모리에 목표 관계 생성',at=(95,600),blue=True)
 c.e('signal','source','owner','L','L',via=[(70,285),(70,702.5)],label='③ 답변 또는 결과 수신',at=(82,515),blue=True)
 c.e('propose','owner','rc','R','R',via=[(700,702.5),(700,455)],label='④ 후속 요청 제안 반환',at=(475,560),blue=True,dash=True)
 c.e('ledger','rc','store','L','L',via=[(115,455),(115,977.5)],label='⑤ Task, command와 publication 원장만 저장',at=(125,835))
 c.e('command','rc','task','R','R',via=[(735,455),(735,1170)],label='⑥ 현재성 확인 뒤 명령 전달',at=(420,1095))
 c.t('note',70,1240,['재시작: 기존 Task는 source와 원장으로 확인한다.','잃은 목표 관계와 질문은 복원하지 않고 사용자에게 다시 묻는다.'],18,True)
 p.text('common',48,1380,'공통: Request Controller가 의미와 현재성을 최종 확인한다. 저장된 상태를 재생하는 것만으로 외부 실행 또는 실제 전달 성공을 만들지 않는다.',20)

def response():
 p=page(5,'응답을 어느 경로에서 만들고 사용자에게 전달하는가','공통 Component는 같은 위치 / T는 좁은 S2S 직접 경로 보유 / A는 모든 응답을 Core에서 확정')
 a=Area(p,0);b=Area(p,1)
 for q in (a,b):
  q.n('input',400,250,400,65,'Interaction Manager (입력)',kind='box')
  q.n('rc',400,430,400,70,'Request Controller',kind='box')
  q.n('ri',40,685,380,70,'Request Interpreter',kind='box')
  q.n('ma',780,685,380,70,'Model Access',kind='box')
  q.n('rm',400,940,400,70,'Response Manager',kind='box')
  q.n('output',400,1170,400,65,'Interaction Manager (출력)',kind='box')
 a.e('audio','input','ma','R','T',via=[(1000,282.5),(1000,645),(970,645)],label='① 원음으로 VoiceProposal 요청',at=(815,600),blue=True)
 a.e('voice-proposal','ma','input','R','R',via=[(1175,720),(1175,282.5)],label='② VoiceProposal와 보류 handle 반환',at=(865,380),blue=True,dash=True)
 a.e('canonical','input','rc',label='③ 정규 입력과 VoiceProposal 전달',at=(520,385))
 a.e('core','rc','ri','L','T',via=[(230,465),(230,645)],label='④a 직접 처리 불가 시 Core 호출',at=(55,590))
 a.e('core-return','ri','rc','R','L',via=[(450,720),(450,465)],label='④b semantic proposal 반환',at=(455,590),dash=True)
 a.e('admit','rc','rm',label='⑤ 현재성 확인 뒤 게시 허용',at=(485,875))
 a.e('release','rm','ma','R','B',via=[(970,975)],label='⑥ 보류 generation release 또는 SpeechRender',at=(785,900),blue=True)
 a.e('audio-return','ma','rm','L','R',via=[(750,720),(750,975)],label='⑦ 음성 결과 반환',at=(585,820),blue=True,dash=True)
 a.e('deliver','rm','output',label='⑧ Text 게시와 Voice 재생',at=(485,1135))
 a.t('note',40,1260,['차이: current-Turn-only 요청은 S2S VoiceProposal을 좁은 직접 경로로 처리할 수 있다.','개인 자료, 화면, Task 또는 과거 대화가 필요하면 ④의 Core 경로로 들어간다.'],18,True)
 b.e('canonical','input','rc',label='① 정규 입력 전달',at=(485,385))
 b.e('core','rc','ri','L','T',via=[(230,465),(230,645)],label='② 모든 요청을 Core에 호출',at=(70,590),blue=True)
 b.e('core-return','ri','rc','R','L',via=[(450,720),(450,465)],label='③ semantic proposal 반환',at=(455,590),blue=True,dash=True)
 b.e('admit','rc','rm',label='④ 현재성 확인 뒤 게시 허용',at=(485,875))
 b.e('text','rm','output','L','L',via=[(230,975),(230,1202.5)],label='⑤ 승인 Text 즉시 게시',at=(70,1100))
 b.e('speech','rm','ma','R','B',via=[(970,975)],label='⑥ 승인 Text로 SpeechRender 요청',at=(790,900),blue=True)
 b.e('audio-return','ma','rm','L','R',via=[(750,720),(750,975)],label='⑦ 음성 결과 반환',at=(585,820),blue=True,dash=True)
 b.e('voice','rm','output',label='⑧ 검사 뒤 Voice 재생',at=(485,1135))
 b.t('note',40,1260,['차이: VoiceProposal 직접 경로와 speculative S2S 생성을 제거한다.','S2S 직접 기능은 미지원이며, 승인 Text와 SpeechRender로 음성 응답을 만든다.'],18,True)

def speech():
 p=page(6,'사용자의 음성을 누가 인식 근거로 만드는가','공통 Component는 같은 위치 / T는 독립 Streaming ASR / A는 공유 Omni의 native evidence에 의존')
 a=Area(p,0);b=Area(p,1)
 for q in (a,b):
  q.n('input',400,250,400,65,'Interaction Manager',kind='box')
  q.g('runtime',730,535,430,285,'Shared Inference Service',kind='process')
  q.n('omni',755,630,380,75,'Omni Voice / Semantic Session',kind='module')
  q.n('ma',400,925,400,70,'Model Access',kind='box')
  q.n('record',400,1150,400,70,'SpeechEvidence / Input Record',kind='store')
 a.g('worker',40,535,430,285,'Speech Input Worker',blue=True,kind='process')
 a.n('asr',65,630,380,75,'Streaming ASR',blue=True,kind='module')
 a.e('asr-audio','input','worker','L','T',via=[(255,282.5),(255,495)],label='① 원음을 독립 인식 경로에 전달',at=(50,445),blue=True)
 a.e('omni-audio','input','runtime','R','T',via=[(945,282.5),(945,495)],label='② 원음을 Omni 경로에도 전달',at=(790,445))
 a.e('asr-evidence','worker','ma','B','L',via=[(255,875),(360,875),(360,960)],label='③ 전사 revision, 시각과 gap 반환',at=(55,850),blue=True,dash=True)
 a.e('omni-result','runtime','ma','B','R',via=[(945,875),(840,875),(840,960)],label='④ Omni 해석 결과 반환',at=(830,850),dash=True)
 a.e('commit','ma','record',label='⑤ 근거를 구별해 canonical 입력 기록',at=(440,1110))
 a.t('note',40,1260,['차이: Streaming ASR와 Omni가 서로 다른 실행 경로에서 근거를 만든다.','Omni 장애 중에도 독립 전사는 계속될 수 있지만 의미 이해와 Voice 생성은 불가하다.'],18,True)
 b.n('native',755,730,380,65,'Native Evidence Producer',blue=True,kind='module')
 b.e('omni-audio','input','runtime','R','T',via=[(945,282.5),(945,495)],label='① 원음을 공유 Omni에 전달',at=(795,445),blue=True)
 b.e('native','runtime','ma','B','R',via=[(945,875),(840,875),(840,960)],label='② native 전사 event와 시각 반환',at=(795,850),blue=True,dash=True)
 b.e('commit','ma','record',label='③ adapter가 canonical 입력으로 기록',at=(430,1110),blue=True)
 b.t('absent',40,650,['독립 Speech Input Worker 없음','인식과 의미 처리가 같은 runtime의 실행 기회와 장애 경계를 공유한다.'],18,True)
 b.t('note',40,1260,['차이: 별도 ASR weights와 실행체를 제거한다.','Omni 장애 중 capture는 계속되지만 recognition도 중단되며 gap을 남겨야 한다.'],18,True)
 p.text('common',48,1380,'공통: capture와 local stop은 추론 밖에서 계속된다. 공유 Omni weights는 한 벌이며 Voice와 semantic session의 상태 및 권한은 격리한다.',20)

BUILDERS=[semantic,screen,retrieval,workflow,response,speech]
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args()
 for build in BUILDERS:build()
 errors=[]
 for p in PAGES:
  p.validate()
  for ext,data in [('svg',p.svg()),('drawio',p.drawio())]:
   ET.fromstring(data); path=OUT/(p.slug+'.'+ext)
   if args.check:
    if not path.exists() or path.read_text()!=data:errors.append(str(path.relative_to(ROOT)))
   else:path.write_text(data)
 slides='\n'.join(f'<section><h2>{html.escape(p.title)}</h2><img src="{p.slug}.svg" alt="{html.escape(p.title)}"><p><a href="{p.slug}.svg">SVG</a> / <a href="{p.slug}.drawio">draw.io</a></p></section>' for p in PAGES)
 review='<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA Stage 4 구조 비교</title><style>body{margin:0;background:#eef1f5;font-family:Arial,sans-serif}header{padding:24px;background:white}section{max-width:1800px;margin:30px auto;background:white;padding:20px}img{display:block;width:100%;height:auto}h2{font-size:20px}p{font-size:16px}</style><header>Stage 4 / S-01~06 설계 비교 / SVG와 편집 가능한 draw.io 원본</header>'+slides+'</html>\n'
 path=OUT/'stage4-review.html'
 if args.check:
  if not path.exists() or path.read_text()!=review:errors.append(str(path.relative_to(ROOT)))
 else:path.write_text(review)
 if errors:raise SystemExit('Diagram drift: '+', '.join(errors))
 print(f'PASS: {len(PAGES)} Stage 4 diagram pairs; XML, identity, bounds, routing and source parity')
if __name__=='__main__':main()
