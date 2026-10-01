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
 p.text('legend',48,1345,'파랑: 바뀌는 책임과 자료 / 검정: 공통 책임 / 박스는 명시한 논리 구성 또는 runtime / 화살표 label의 경계를 비교',21)
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
 p=page(1,'근거 보완의 실행 경계를 바꾼다','T: 중간 제안 종료 후 host 재호출 / A: READ_REQUIRED에서 연산 양보 후 같은 실행 재개 (runtime 지원 미확인)')
 a=Area(p,0);b=Area(p,1)
 a.n('rc',340,270,520,140,'Request Controller','읽기 조정, 입력 revision과 예산','최종 Semantic Commit',blue=True)
 a.n('cm',30,560,500,150,'Context Manager','현재 권한으로 한정 읽기','source revision과 receipt 반환')
 a.n('ri',680,560,490,150,'Request Interpreter','1차 중간 제안 종료','추가 근거 뒤 다음 semantic 호출',blue=True)
 a.n('src',30,960,500,145,'원본 source와 owner read port','허용된 자료와 현재 상태','원본이 권위, cache는 재검증',kind='box')
 a.n('ma',680,810,490,120,'Model Access','호출 및 역할별 session/KV')
 a.n('rt',680,1060,490,150,'Shared Inference Service','공유 Omni weights 한 벌','target도 KV/prefix 재사용 가능')
 a.e('read','rc','cm','L','T',via=[(280,340)],label='2  추가 읽기 요청 / 결과 수신',at=(30,470),blue=True)
 a.e('interpret','rc','ri','R','T',via=[(925,340)],label='1, 3  제안 호출 / 반환',at=(710,470),blue=True)
 a.e('source','cm','src',label='owner/source 조회',at=(90,850))
 a.e('model','ri','ma',label='완료된 proposal 경계',at=(720,765),blue=True)
 a.e('runtime','ma','rt',label='semantic generation',at=(715,990))
 b.n('rc',340,270,520,140,'Request Controller','최종 proposal의 현재성 검사','Worker에 외부 실행 권한 없음')
 b.n('worker',30,530,520,165,'Semantic Resolution Worker','재개 handle, 임시 field와 read set','누적 예산, revision 유지',blue=True)
 b.n('broker',690,530,480,165,'Capability Read Broker','현재 권한 및 읽기 scope 검사','receipt와 근거 반환',blue=True)
 b.n('ma',30,870,520,125,'Model Access','READ_REQUIRED / 근거 주입 / 재개',blue=True)
 b.n('cm',690,870,480,125,'Context Manager','owner/source port로 실제 읽기')
 b.n('rt',30,1110,520,140,'Shared Inference Service','양보한 실행과 유한 KV','회수되면 handle 무효',blue=True)
 b.n('src',690,1110,480,140,'원본 source와 owner read port','권한 및 자료 revision','검색 방식 자체는 별도 선택',kind='box')
 b.e('request','rc','worker','L','T',via=[(290,340)],label='작업 시작 / 최종 제안 수신',at=(40,470))
 b.e('read','worker','broker','R','L',label='한정 읽기',at=(560,585),blue=True)
 b.e('resume','worker','ma',label='handle에 근거 주입',at=(80,780),blue=True)
 b.e('yield','ma','worker','L','L',via=[(16,932.5),(16,612.5)],label='READ_REQUIRED / 연산 양보',at=(45,830),blue=True,dash=True)
 b.e('runtime','ma','rt',label='같은 실행 재개',at=(70,1050),blue=True)
 b.e('cm','broker','cm',label='읽기 / receipt',at=(745,785))
 b.e('source','cm','src',label='실제 source 조회',at=(735,1050))

def screen():
 p=page(2,'당시 화면의 근거를 언제 만드는가','T: 원시 timeline / A: 사전 객체 이력 + raw fallback / B: 명시 선택만 지원',3)
 for i in range(3):
  a=Area(p,i,3)
  if i<2:
   a.n('speech',30,275,330,125,'SpeechEvidence','발화 sample 시각','revision, 오차와 gap',kind='box')
   a.n('screen',422,275,340,125,'화면과 UI 사건','선택, 포인터, crop','window/document',kind='box')
  if i==0:
   a.n('im',50,555,692,195,'Interaction Manager','OS/UI 사건 + 유한 sampling + pre-roll','발화 구간 pin, RAM 원시 timeline','늦은 전사를 당시 관측에 연결',blue=True)
   a.n('cm',50,1030,692,175,'Context Manager','당시 원문, crop과 구조 자료 준비','gap과 source revision을 함께 제공')
   a.e('audio','speech','im',via=[(195,470),(396,470)],label='시각 연결',at=(45,510))
   a.e('screen','screen','im',via=[(592,470),(396,470)],label='관측 보존',at=(470,510))
   a.e('context','im','cm',label='요청 때 근거 조립',at=(120,900),blue=True)
  elif i==1:
   a.n('ground',50,485,692,165,'Screen Grounding Service','UI 구조 또는 Model Access의 공유 Omni vision','요청 전 객체와 관계 후보 생산',blue=True)
   a.n('store',50,770,692,155,'Derived Object Store','유한 RAM: 객체 ID, 시각, source revision','제한된 raw fallback + 채택한 최소 내구 근거',blue=True,kind='store')
   a.n('cm',50,1060,692,150,'Context Manager','당시 객체와 늦은 전사를 결합','원문 및 현재 Action 대상은 다시 확인')
   a.e('produce','screen','ground',via=[(592,440),(396,440)],blue=True)
   a.e('materialize','ground','store',label='파생 이력 게시',at=(140,710),blue=True)
   a.e('read','store','cm',label='준비된 객체 + 필요한 원시 근거',at=(110,1000),blue=True)
   a.e('timing','speech','cm','L','L',via=[(16,337.5),(16,1135)],label='SpeechEvidence 별도 결합',at=(65,1040),dash=True)
  else:
   a.n('select',50,285,692,135,'사용자의 명시 선택','영역/객체 선택 후 “이 자료로 요청”','선택 밖의 연속 지칭은 미지원',blue=True,kind='box')
   a.n('capture',50,575,692,150,'Interaction Manager','선택 시점의 identity, revision과 crop 획득','UI/Core process는 그대로 남음',blue=True)
   a.n('package',50,860,692,140,'Capture Package','지정 자료와 발화의 결합','허용 수명 안에서 보관',blue=True,kind='store')
   a.n('cm',50,1130,692,120,'Context Manager','패키지와 현재 source 적용 가능성 확인')
   a.e('choose','select','capture',label='수동 선택 부담',at=(155,505),blue=True)
   a.e('seal','capture','package',label='패키지 봉인',at=(170,805),blue=True)
   a.e('read','package','cm',label='선택한 자료만 읽기',at=(130,1075),blue=True)
 p.text('common',48,1380,'공통 후속: Context 근거 → Request Controller의 입력/예산 결합 → Request Interpreter 호출 및 제안 → Request Controller 확정.',20)

def retrieval():
 p=page(3,'후보를 미리 생산하는 검색 서브시스템','T에도 index/cache/summary가 있다 / A는 embedding helper, 파생 저장과 갱신 책임을 추가한다')
 a=Area(p,0);b=Area(p,1)
 a.n('rc',350,260,500,110,'Request Controller','허용된 범위의 자료 요청')
 a.g('cm',30,455,1140,600,'Context Manager')
 a.n('read',60,565,490,150,'owner/source read port','자료 이름, 기간, Task와 대화 참조','필요한 현재 원문 읽기',kind='box')
 a.n('cache',650,565,490,160,'기존 index와 cache','metadata/keyword index','revision cache와 summary',kind='store')
 a.n('receipt',280,870,700,145,'Evidence Query Receipt / Context','source revision, 부분 coverage와 실패','확정 대상과 권한은 검색 결과만으로 결정하지 않음',kind='box')
 a.n('source',60,1150,490,105,'원본 source / owner','허용된 원문과 identity',kind='box')
 a.e('request','rc','read',via=[(600,405),(305,405)],label='한정 읽기',at=(200,430))
 a.e('cache','cache','read','L','R',via=[(600,645),(600,640)],label='재사용',at=(565,535))
 a.e('context','read','receipt',via=[(305,800),(630,800)],label='근거 조립',at=(325,840))
 a.e('source','read','source','L','L',via=[(45,640),(45,1202.5)],label='원본 확인',at=(65,1100))
 b.n('source',30,260,480,115,'허용 collection / source','지속 가공과 보관 동의 범위',kind='box')
 b.n('worker',30,475,480,125,'Indexing Worker','변경 수집과 유한 작업 queue',blue=True)
 b.n('embed',30,735,480,140,'Embedding Runtime','document와 query는 같은 build','별도 helper 전체 비용 포함',blue=True)
 b.n('index',30,1060,480,160,'Vector Index + Source Manifest','revision, epoch, build와 generation','완성 generation을 원자 게시',blue=True,kind='store')
 b.n('rc',690,260,480,115,'Request Controller','같은 범위의 자료 요청')
 b.n('retrieve',690,530,480,170,'Semantic Retrieval Service','query encoding + 의미/lexical 후보','후보 없음도 자료 없음은 아님',blue=True)
 b.n('cm',690,970,480,170,'Context Manager','현재 원문, 권한과 identity 검증','기존 조회 fallback과 근거 조립')
 b.e('collect','source','worker',label='source 변경',at=(95,430),blue=True)
 b.e('encode','worker','embed',label='문서 embedding',at=(95,670),blue=True)
 b.e('publish','embed','index',label='generation 생산',at=(90,990),blue=True)
 b.e('query','rc','retrieve',label='검색 요청',at=(750,455),blue=True)
 b.e('qencode','retrieve','embed','L','R',via=[(580,615),(580,805)],label='query도 동일 helper',at=(700,765),blue=True,dash=True)
 b.e('candidates','index','retrieve','R','B',via=[(620,1140),(620,830),(930,830)],label='현재 게시된 후보',at=(710,885),blue=True)
 b.e('consume','retrieve','cm','R','R',via=[(1185,615),(1185,1055)],label='후보 소비 전 검사',at=(740,930),blue=True)
 b.e('original','cm','source','L','L',via=[(610,1055),(610,405),(16,405),(16,317.5)],label='원본 확인 / 기존 조회 fallback',at=(625,1255),dash=True)

def workflow():
 p=page(4,'대기와 재개의 권위 원본을 선택한다','T와 A는 내구 대기 복구 / B는 대기 graph 복구 포기, Task와 command 원장은 유지',3)
 a=Area(p,0,3);b=Area(p,1,3);c=Area(p,2,3)
 a.g('rc',30,285,732,355,'Request Controller',blue=True)
 a.n('graph',60,370,672,100,'durable Request Graph','사용자 목표 관계와 후속 해제',blue=True,kind='module')
 a.n('question',60,505,672,100,'Pending User Interaction','실제 제시 질문과 답변 binding',blue=True,kind='module')
 a.n('state',60,790,672,155,'State Store','current records + 미완료 원장','graph와 질문의 내구 원본',blue=True,kind='store')
 a.n('task',60,1120,672,110,'Task Manager → Agent Gateway','업무 lifecycle, 명령 전송과 source 확인',kind='box')
 a.e('persist','rc','state',label='owner 전이와 intent 원자 기록',at=(100,725),blue=True)
 a.e('dispatch','rc','task','L','L',via=[(16,462.5),(16,1175)],label='host admission 뒤 command',at=(90,1040))
 b.n('rc',60,265,672,120,'Request Controller','의미 확정, 현재성 검사와 admission')
 b.g('runtime',30,480,732,415,'Interaction Workflow Runtime',blue=True)
 b.n('inbox',60,565,312,115,'Signal Inbox','내구 signal 수락','중복 제거',blue=True,kind='module')
 b.n('timer',420,565,312,115,'Timer Service','deadline과 timeout','내구 재개 조건',blue=True,kind='module')
 b.n('activity',60,745,672,110,'Activity Dispatcher','고정 VIA 활동의 intent와 attempt',blue=True,kind='module')
 b.n('store',60,995,672,150,'Continuation Store','같은 State Store 안의 논리 영역','continuation + signal + activity intent',blue=True,kind='store')
 b.t('owner',65,1215,['활동 → host admission → Task Manager → Agent Gateway','Agent 내부 계획과 외부 실행은 이동하지 않음'])
 b.e('start','rc','runtime',label='내구 시작 signal / 현재 Request',at=(90,440),blue=True)
 b.e('persist','runtime','store',label='전이와 intent 원자 기록',at=(125,960),blue=True)
 b.e('admit','activity','rc','L','L',via=[(16,800),(16,325)],label='활동마다 현재 admission',at=(70,930),blue=True,dash=True)
 c.n('rc',60,265,672,120,'Request Controller','현재 입력과 최종 admission 유지')
 c.n('volatile',60,535,672,170,'세션 한정 요청 coordinator','목표 관계와 질문 focus는 RAM','Core 재시작 때 그래프 복원 없음',blue=True)
 c.n('store',60,860,672,175,'State Store','Task, command와 publication 원장 유지','대기 graph와 질문 continuation은 저장하지 않음',blue=True,kind='store')
 c.n('task',60,1160,672,100,'Task Manager → Agent Gateway','기존 실행의 source 확인 및 중복 방지 유지',kind='box')
 c.e('start','rc','volatile',label='세션 중 대기와 후속 조정',at=(105,470),blue=True)
 c.e('command','rc','store','L','L',via=[(16,325),(16,947.5)],label='내구 command admission은 유지',at=(80,800))
 c.e('propose','volatile','rc','R','R',via=[(750,620),(750,325)],label='후속 제안 / 현재성 검사',at=(440,450),blue=True,dash=True)
 c.e('release','rc','task','R','R',via=[(775,325),(775,1210)],label='현재 admission과 내구 원장 뒤 전달',at=(90,1110))
 p.text('common',48,1380,'복구는 같은 업무의 상태 확인이다. 어떤 안도 workflow 재실행만으로 외부 exactly-once나 실제 청취를 보장하지 않는다.',20)

def response():
 p=page(5,'좁은 S2S 경로와 Core-only 응답을 비교한다','T도 정규 전사와 host admission이 필요 / A는 S2S 직접 기능을 포기하며 승인 Text에서 Voice 생성')
 a=Area(p,0);b=Area(p,1)
 a.n('input',350,255,520,115,'Interaction Manager','원음과 정규 전사, input revision')
 a.n('ma',30,505,500,165,'Model Access','S2S VoiceProposal + 보류 generation','공유 Omni의 음성 역할',blue=True)
 a.n('rc',690,505,480,165,'Request Controller','좁은 직접 admission / Core 확정','같은 Request의 단일 경로 권위',blue=True)
 a.n('rm',30,940,500,150,'Response Manager','허용 generation release 또는 응답 구성','publication과 실제 전달 기록')
 a.n('ri',690,940,480,150,'Request Interpreter','Context 기반 semantic handling','필요 시 직접 구성 또는 Agent 위임')
 a.n('output',350,1175,620,100,'Interaction Manager','Text 게시, Voice 재생, local stop와 receipt')
 a.e('audio','input','ma',via=[(610,430),(280,430)],label='음성 입력',at=(110,460))
 a.e('canonical','input','rc',via=[(610,395),(930,395)],label='정규 입력 + VoiceProposal',at=(855,445))
 a.e('proposal','ma','input','L','L',via=[(16,587.5),(16,312.5)],label='반환: VoiceProposal와 보류 handle',at=(35,480),blue=True,dash=True)
 a.e('core','rc','ri',label='직접 불가면 Core 인계',at=(720,790),blue=True)
 a.e('result','ri','rc','R','R',via=[(1185,1015),(1185,587.5)],label='제안 반환',at=(990,890),dash=True)
 a.e('publish','rc','rm','B','T',via=[(930,725),(280,725)],label='현재 admission 뒤 게시',at=(50,840))
 a.e('delivery','rm','output',via=[(280,1140),(660,1140)])
 b.n('input',350,255,520,115,'Interaction Manager','정규 입력과 현재 input revision')
 b.n('rc',30,505,500,165,'Request Controller','입력, 기본 Context와 예산 결합','최종 proposal 검사 및 admission')
 b.n('ri',690,505,480,165,'Request Interpreter','모든 요청의 semantic handling','허용 draft 또는 구성 요청',blue=True)
 b.n('rm',30,870,500,165,'Response Manager','Text 및 publication','음성 결과 검사와 현재 release')
 b.n('ma',690,870,480,165,'Model Access','승인 Text → 공유 Omni SpeechRender','generation 결과 반환',blue=True)
 b.n('output',350,1180,620,100,'Interaction Manager','채널별 전달, local stop와 receipt')
 b.e('input','input','rc',via=[(610,410),(280,410)],label='입력 확정',at=(60,460))
 b.e('interpret','rc','ri','R','L',label='호출',at=(575,565),blue=True)
 b.e('proposal','ri','rc','T','T',via=[(930,455),(280,455)],label='semantic proposal 반환',at=(490,485),blue=True,dash=True)
 b.e('admit','rc','rm',label='확정된 결과와 게시 허용',at=(80,780))
 b.e('voice','rm','ma','R','L',label='Text',at=(575,925),blue=True)
 b.e('generated','ma','rm','B','B',via=[(930,1090),(280,1090)],label='음성 결과 반환',at=(670,1135),blue=True,dash=True)
 b.e('text','rm','output','L','L',via=[(16,952.5),(16,1230)],label='Text 게시: 음성 생성 대기 없음',at=(35,1160))
 b.e('release','rm','output','B','T',via=[(280,1120),(660,1120)],label='검사 뒤 Voice release',at=(360,1160))
 b.t('removed',30,1310,'A에서 제거: VoiceProposal 직접 경로와 speculative S2S 생성. 같은 Interaction Manager의 입/출력 port를 확대 표시.',18,True)

def speech():
 p=page(6,'음성 근거 생산자의 실행 및 장애 경계','capture/local stop은 양안 모두 추론 밖 / 공유 Omni weights는 양안 각각 한 벌')
 a=Area(p,0);b=Area(p,1)
 for area in (a,b):area.n('input',350,260,520,130,'Interaction Manager','지속 capture와 local stop','같은 sample clock의 원음')
 a.g('worker',30,600,510,275,'Speech Input Worker',blue=True,kind='process')
 a.n('asr',55,690,460,130,'Streaming ASR','독립 CPU 인식과 시각 근거','추가 helper weights',blue=True)
 a.g('runtime',680,600,490,275,'Shared Inference Service',kind='process')
 a.n('omni',705,690,440,130,'공유 Omni weights 한 벌','Voice / semantic session 격리','인식 중에도 semantic 진행',kind='box')
 a.n('ma',270,1050,790,180,'Model Access','ASR adapter → SpeechEvidence','Omni 해석과 중요한 불일치는 보존','두 producer의 revision과 incarnation 구별',blue=True)
 a.e('asraudio','input','worker','B','T',via=[(610,490),(285,490)],label='독립 인식 경로',at=(70,535),blue=True)
 a.e('omniaudio','input','runtime','B','T',via=[(610,450),(925,450)],label='Omni 원음 경로',at=(750,535))
 a.e('evidence','worker','ma',via=[(285,950),(665,950)],label='전사 / 시각 / gap',at=(75,995),blue=True)
 a.e('omni','runtime','ma','R','R',via=[(1185,737.5),(1185,1140)],label='Omni 결과',at=(965,970),dash=True)
 b.g('runtime',120,555,990,440,'Shared Inference Service',kind='process')
 b.n('omni',160,645,910,120,'공유 Omni weights 한 벌','Voice / semantic session 및 KV 격리는 유지',kind='box')
 b.n('native',160,810,910,145,'Omni native evidence producer','음성 session에서 transcript revision과 sample 시각 생산','semantic과 실행 기회 공유, 지속 인식 지원 미확인',blue=True)
 b.n('ma',200,1100,830,160,'Model Access','Native Evidence Adapter → SpeechEvidence','없는 시각 근거를 adapter가 만들지 않음',blue=True)
 b.e('audio','input','runtime',via=[(610,480),(615,480)],label='같은 runtime에 입력과 의미 처리 의존',at=(250,520),blue=True)
 b.e('evidence','runtime','ma',label='native event와 incarnation',at=(300,1055),blue=True)
 p.text('common',48,1380,'Omni 장애 시 T는 독립 전사를 계속 만들 수 있지만 A의 recognition은 중단된다. 두 안 모두 의미 이해와 Voice 생성은 불가.',20)

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
