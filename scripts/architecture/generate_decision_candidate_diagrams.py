#!/usr/bin/env python3
"""Generate paired, editable comparison views for discussion candidates.

These are architecture-document assets, not executable VIA candidates.
Black means common; blue means a structural difference on EITHER side.
Inner blocks explain responsibilities, not new approved Components.
"""
from __future__ import annotations

import argparse
from dataclasses import dataclass, field
from html import escape
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT / 'docs/architecture/12-decisions/decision-packages/candidates/diagrams'
BLACK = '#161616'
BLUE = '#0057B8'
WHITE = '#FFFFFF'
FONT = 'Arial, Apple SD Gothic Neo, sans-serif'


@dataclass
class Node:
    id: str
    x: float
    y: float
    w: float
    h: float
    title: str
    lines: tuple[str, ...] = ()
    kind: str = 'module'
    diff: bool = False
    parent: str = '1'


@dataclass
class Link:
    id: str
    source: str
    target: str
    points: list[tuple[float, float]]
    label: str = ''
    at: tuple[float, float] | None = None
    diff: bool = False
    state: bool = False
    both: bool = False


@dataclass
class Figure:
    slug: str
    title: str
    scenario: str
    labels: tuple[str, str]
    common: str
    notes: tuple[tuple[str, str, str], tuple[str, str, str]]
    nodes: list[Node] = field(default_factory=list)
    links: list[Link] = field(default_factory=list)
    captions: list[tuple[float, float, str, bool, int]] = field(default_factory=list)
    width: int = 2600
    height: int = 1710


class Pane:
    def __init__(self, fig: Figure, side: int):
        self.f = fig
        self.prefix = 'a-' if side == 0 else 'b-'
        self.ox = 40 + 1280 * side
        self.oy = 210

    def node(self, ident, x, y, w, h, title, lines=(), kind='module', diff=False, parent=None):
        key = self.prefix + ident
        self.f.nodes.append(Node(key, self.ox+x, self.oy+y, w, h, title, tuple(lines), kind, diff,
                                 self.prefix+parent if parent else '1'))
        return key

    def group(self, ident, x, y, w, h, title, diff=False, kind='component', parent=None):
        return self.node(ident,x,y,w,h,title,kind=kind,diff=diff,parent=parent)

    def leaf(self, ident, x, y, w, h, title, *lines, diff=False, parent=None, kind='module'):
        return self.node(ident,x,y,w,h,title,lines,kind,diff,parent)

    def link(self, src, dst, points, label='', at=None, diff=False, state=False, both=False):
        self.f.links.append(Link(self.prefix+'link-'+str(len(self.f.links)),self.prefix+src,self.prefix+dst,
            [(self.ox+x,self.oy+y) for x,y in points],label,
            (self.ox+at[0],self.oy+at[1]) if at else None,diff,state,both))

    def note(self,x,y,text,diff=False,size=20):
        self.f.captions.append((self.ox+x,self.oy+y,text,diff,size))


def request_figure():
    f=Figure('request-resolution-boundary','요청 해석과 확정의 책임 경계',
      '상황  |  “이 그래프를 아까 자료에 넣어줘”를 해석하는 동안 “아니, 옆 표야”라는 정정이 들어온다.',
      ('방안 1  |  의미 제안과 요청 제어를 별도 Component로 분리','방안 2  |  요청 작업공간 안에 해석 orchestration 통합'),
      '같은 입력 근거·공유 Omni·host 검증·추가 조회 예산·정정 처리를 유지한다.',
      (('얻는 것  |  모델의 제안 계약과 확정 권한을 별도로 변경·검토','부담  |  proposal·읽기 제안·revision의 Component 간 정합성','검토 질문  |  독립 계약이 실제 변경 경계를 만드는가?'),
       ('얻는 것  |  조회·해석·정정을 한 요청 작업공간에서 조정','부담  |  prompt/schema 변화와 요청 lifecycle 변경이 한 owner에 모임','검토 질문  |  내부 모듈 경계만으로 같은 통제를 유지할 수 있는가?')))
    for s in range(2):
        p=Pane(f,s)
        p.leaf('input',30,90,350,100,'Interaction Manager','InputFinal · evidence refs','InputStarted → hold',kind='component')
        p.leaf('policy',450,90,300,100,'Policy Manager','scope · policy revision',kind='component')
        p.leaf('context',870,90,340,100,'Context Manager','bounded read · receipt',kind='component')
        p.group('control',30,280,560 if s==0 else 1180,440,'Request Controller',diff=True)
        if s==0: p.group('interpret',690,280,520,440,'Request Interpreter',diff=True)
        owner='interpret' if s==0 else 'control'
        p.leaf('workspace',60,360,225,95,'Turn Workspace','입력·Context revision','요청별 임시 상태',parent='control')
        p.leaf('read',335,360,225,95,'조회·예산 제어','기본 / 추가 읽기','deadline · revision',parent='control')
        p.leaf('hold',60,570,225,95,'정정·hold 처리','늦은 제안 무효화','미전송 요청 보류',parent='control')
        p.leaf('commit',335,570,225,95,'의미 확정·admission','field·coverage·권한','Semantic Commit',parent='control')
        p.leaf('prompt',720,360,215,95,'Prompt / Schema','근거·목표·제약 입력','Model Call Envelope',parent=owner)
        p.leaf('proposal',975,570,205,95,'제안 해석','field별 근거·미해결','추가 읽기 제안',parent=owner)
        p.leaf('temp',975,360,205,95,'해석 임시 상태','권한·업무 상태 없음',parent=owner)
        p.link('input','workspace',[(330,190),(330,345),(172,345),(172,360)],'① 입력·정정',(330,242))
        p.link('read','context',[(447,360),(447,230),(1040,230),(1040,190)],'② bounded read / receipt',(824,230),both=True)
        p.link('policy','commit',[(600,190),(630,190),(630,690),(560,690),(560,625)],state=True)
        p.link('read','prompt',[(560,405),(720,405)],'③ 해석',(640,387),diff=True,both=True)
        p.link('proposal','commit',[(975,620),(560,620)],'④ Semantic Proposal' if s==0 else '④ 내부 해석 결과',(770,600),diff=True)
        p.link('workspace','read',[(285,405),(335,405)])
        p.link('workspace','hold',[(172,455),(172,570)],'revision',(232,510))
        p.link('hold','commit',[(285,615),(335,615)])
        p.group('store',30,810,560,165,'State Store')
        p.leaf('records',60,875,500,90,'Conversation · Request · Semantic Commit','Request Controller 소유 / 내구 기록',kind='store',parent='store')
        p.leaf('model',690,810,520,145,'Model Access','구조화 추론 · session/KV 분리','공유 Omni 1회 적재',kind='component')
        p.link('commit','records',[(447,665),(447,875)],'⑤ 확정 기록',(457,763),state=True)
        p.link('prompt','model',[(935,405),(955,405),(955,490),(1225,490),(1225,880),(1210,880)],'동일 semantic 호출',(1050,490))
        p.link('model','proposal',[(1080,810),(1080,665)])
        p.leaf('task',30,1020,560,85,'Task Manager','확정 목표·대상·Task binding으로 업무 구성',kind='component')
        p.leaf('response',690,1020,520,85,'Response Manager','허용된 응답·clarification 게시',kind='component')
        p.note(30,780,'점선: 현재 policy 확인 / 내구 상태 접근',False,18)
        p.note(30,995,'확정 뒤 인계: 업무는 Task Manager, 응답·질문은 Response Manager',False,19)
    return f


def task_figure():
    f=Figure('conversation-task-ownership','대화와 지속 업무의 상태 소유권',
      '상황  |  다른 대화로 이동한 뒤 보고서를 취소하는 순간, Agent의 완료 event와 승인 질문이 교차한다.',
      ('방안 1  |  Conversation / Request와 Task / Execution owner 분리','방안 2  |  독립 lifecycle을 유지하며 상태 owner 통합'),
      'Task identity·Agent Gateway·내구 command/event·질문 결합·대화 종료 후 업무 지속은 동일하다.',
      (('얻는 것  |  대화 기능과 장기 업무 상태의 변경 책임 분리','부담  |  admission·domain event·질문 종료의 owner 간 결합','검토 질문  |  공유 transaction 안에서도 분리할 가치가 있는가?'),
       ('얻는 것  |  취소·완료·질문 종료를 한 owner가 조정','부담  |  대화 변화와 업무 lifecycle 변화가 같은 Component에 모임','검토 질문  |  통합이 Task를 대화 수명에 종속시키지 않는가?')))
    for s in range(2):
        p=Pane(f,s)
        p.leaf('input',30,80,550,100,'Interaction Manager','대화 전환 · 특정 Task 취소 / 후속 입력',kind='component')
        p.leaf('gateway',690,80,520,100,'Agent Gateway','command 전송 · durable inbox · source query',kind='component')
        p.group('rc',30,270,550 if s==0 else 1180,465,'Request Controller',diff=True)
        if s==0:p.group('tm',690,270,520,465,'Task Manager',diff=True)
        owner='tm' if s==0 else 'rc'
        p.leaf('request',60,345,235,95,'Request Graph','정정·의존·admission','Conversation별 전이',parent='rc')
        p.leaf('question',325,345,225,95,'질문·승인 결합','Task / question ID','답변·만료·종료',parent='rc')
        p.leaf('conv',60,565,490,100,'Conversation / Request 상태','업무 참조 · 질문 revision · publication intent',parent='rc')
        p.leaf('projection',720,345,220,95,'Task projection','phase / control','verification state',parent=owner)
        p.leaf('commands',970,345,210,95,'Command 구성','epoch·대상 version','실행 binding',parent=owner)
        p.leaf('taskstate',720,565,460,100,'Task / Execution 상태','대화와 독립 identity · Task별 mailbox',parent=owner)
        p.link('input','request',[(330,180),(330,333),(180,333),(180,345)],'① 요청/취소',(330,230))
        p.link('request','projection',[(180,440),(180,490),(770,490),(770,440)],'② admission' if s==0 else '② 내부 전이',(625,490),diff=True)
        p.link('gateway','projection',[(810,180),(810,250),(950,250),(950,333),(830,333),(830,345)],'③ inbox event',(920,235),both=True,diff=True)
        p.link('commands','gateway',[(1080,345),(1080,180)],'command',(1120,258),diff=True)
        p.link('taskstate','conv',[(720,615),(550,615)],'④ domain event' if s==0 else '④ 내부 상태 결합',(633,588),diff=True)
        p.link('projection','commands',[(940,395),(970,395)])
        p.link('request','conv',[(180,440),(180,525),(220,525),(220,565)])
        p.link('projection','taskstate',[(880,440),(880,565)])
        p.link('question','conv',[(437,440),(437,565)])
        p.group('store',30,815,1180,170,'State Store')
        p.leaf('convdb',60,880,490,90,'Conversation · Request','질문 · publication intent',kind='store',parent='store')
        p.leaf('taskdb',720,880,460,90,'Task · Execution','command · domain event',kind='store',parent='store')
        p.link('conv','convdb',[(305,665),(305,880)],'⑤ owner write',(320,787),state=True,diff=True)
        p.link('taskstate','taskdb',[(950,665),(950,880)],'⑤ owner write',(970,787),state=True,diff=True)
        p.note(450,777,'Unit of Work: 질문 종료 + Task 변경',True,19)
        p.leaf('response',30,1030,550,75,'Response Manager','결과·질문 전달과 실제 receipt',kind='component')
        p.leaf('identity',690,1030,520,75,'공통 lifecycle 규칙','Conversation 종료 ≠ Task 종료',kind='note')
        p.note(30,1005,'재시작: 기록 복원 → source 상태 조정 → 질문·응답 재결합',False,20)
    return f


def response_figure():
    f=Figure('response-publication-ownership','응답 게시와 실제 전달의 책임 경계',
      '상황  |  직접 설명 중 보고서 완료·승인 질문이 도착하고, 사용자가 끼어든 뒤 “방금 말한 것”을 참조한다.',
      ('방안 1  |  응답 종류를 가로지르는 공통 publication owner','방안 2  |  종류별 publication owner + 공통 장치 arbiter'),
      'Request Controller의 admission·상세 Text/Voice 요약·단일 발화·local stop·전달 불명 보존은 동일하다.',
      (('얻는 것  |  모든 응답의 원장·발화 차례·복구 계약을 한곳에서 관리','부담  |  직접 답변과 업무 알림 기능 변경이 공통 owner에 모임','검토 질문  |  공통 계층이 전달 의미를 통일하는가, 변화만 묶는가?'),
       ('얻는 것  |  대화 응답과 업무 알림의 독립적인 변경·원장 관리','부담  |  전달 이력 조회·중단·복구가 두 원장과 arbiter에 걸침','검토 질문  |  arbiter를 얇게 유지하면서 원장 정합성을 지킬 수 있는가?')))
    for s in range(2):
        p=Pane(f,s)
        p.leaf('rc',30,80,1180,100,'Request Controller','① Canonical Response Payload · publication admission · Request/Task/policy revision',kind='component')
        if s==0:p.group('rm',30,265,1180,385,'Response Manager',diff=True)
        else:
            p.group('direct',30,265,550,385,'Direct Response Delivery',diff=True)
            p.group('notify',660,265,550,385,'Task Notification Delivery',diff=True)
        a='rm' if s==0 else 'direct';b='rm' if s==0 else 'notify'
        p.leaf('compose1',60,340,230,90,'直接 답변 구성'.replace('直接','직접'),'S2S handle / Core','확정 사실 보존',parent=a)
        p.leaf('compose2',690,340,230,90,'업무 알림 구성','template / 결과 요약','실패·불확실성 보존',parent=b)
        p.leaf('publish1',320,340,230,90,'게시·version 검사','Text 멱등 upsert','Voice content version',parent=a)
        p.leaf('publish2',950,340,230,90,'게시·version 검사','Task/result revision','질문·완료 보존',parent=b)
        if s==0:
            p.leaf('queue',60,520,490,90,'발화 차례·대기열','foreground · 질문/terminal · progress',parent='rm',diff=True)
            p.leaf('receipt',690,520,490,90,'공통 전달 기록 조정','audible prefix · interrupted · UNKNOWN',parent='rm',diff=True)
        else:
            p.leaf('queue',60,520,490,90,'직접 응답 대기·receipt','publication별 audible prefix / UNKNOWN',parent=a,diff=True)
            p.leaf('receipt',690,520,490,90,'업무 알림 대기·receipt','Task별 보존 · 병합 · UNKNOWN',parent=b,diff=True)
        p.link('rc','compose1',[(555,180),(555,325),(175,325),(175,340)],'payload',(555,225),diff=True)
        p.link('rc','compose2',[(1175,180),(1175,325),(805,325),(805,340)],'payload',(1175,225),diff=True)
        p.link('compose1','publish1',[(290,385),(320,385)])
        p.link('compose2','publish2',[(920,385),(950,385)])
        p.link('publish1','queue',[(435,430),(435,520)])
        if s==0:
            p.link('publish2','queue',[(1065,430),(1065,470),(435,470),(435,520)],diff=True)
            p.link('receipt','queue',[(690,565),(550,565)],'receipt',(620,548),diff=True)
        else:p.link('publish2','receipt',[(1065,430),(1065,520)],diff=True)
        p.group('im',30,765,1180,185,'Interaction Manager',diff=True)
        p.leaf('arbiter',60,830,490,90,'Turn-Taking Control' if s==0 else '출력 arbiter + Turn-Taking Control',
               '단일 Voice lease · output epoch','사용자 발화 검사 / 즉시 local stop',parent='im',diff=True)
        p.leaf('io',690,830,490,90,'Channel I/O','Text 표시 · audio 재생 · delivery receipt',parent='im')
        p.link('queue','arbiter',[(305,610),(305,830)],'② release / cancel / receipt',(308,710),diff=True,both=True)
        if s==0:p.link('io','receipt',[(940,830),(940,610)],'③ delivery receipt',(940,710),diff=True)
        else:p.link('receipt','arbiter',[(690,565),(620,565),(620,740),(435,740),(435,830)],'② 후보 / release / receipt',(730,740),diff=True,both=True)
        p.link('arbiter','io',[(550,875),(690,875)],'출력 / receipt',(620,858),both=True)
        p.group('store',30,1010,800,140,'State Store')
        p.leaf('db',60,1070,740 if s==0 else 330,70,'publication outbox + 전달 receipt' if s==0 else '직접 응답 원장',kind='store',parent='store',diff=True)
        if s==1:p.leaf('db2',450,1070,350,70,'업무 알림 원장',kind='store',parent='store',diff=True)
        p.leaf('model',885,1010,325,120,'Model Access','공유 Omni · 요약/음성','가중치 복제 없음',kind='component')
        p.note(30,985,'③ 응답 owner가 intent·receipt 저장 / 불명 음성 자동 재생 금지',False,20)
        p.link('queue','db',[(60,565),(15,565),(15,995),(225,995),(225,1070)],state=True,diff=True)
        if s==1:p.link('receipt','db2',[(1180,565),(1235,565),(1235,995),(625,995),(625,1070)],state=True,diff=True)
        p.link('compose2','model',[(805,430),(805,475),(1225,475),(1225,1070),(1210,1070)])
        p.note(885,975,'구성·SpeechRender 공통 경로',False,18)
    return f


def context_figure():
    f=Figure('context-preparation-ownership','Context 구성과 파생 view의 소유권',
      '상황  |  “아까 설명한 표로 발표자료를 만들어줘”의 해석과 결과 안내가 같은 자료·대화·업무 근거를 사용한다.',
      ('방안 1  |  Context Manager가 원본을 결합하고 근거 view 소유','방안 2  |  소비자가 view 소유 / Context Manager는 source·기억 접근'),
      '원본 owner·허용 source·read port·source revision·철회/삭제 효력·당시 화면 근거는 유지한다.',
      (('얻는 것  |  receipt·coverage·파생 view 무효화를 공통으로 관리','부담  |  소비자별 요구 변경이 공통 Context 계약에 모임','검토 질문  |  공통 구성이 중복을 줄이는가, 불필요한 근거를 운반하는가?'),
       ('얻는 것  |  해석용·응답용 근거 선택과 형식을 독립적으로 변경','부담  |  view·dependency index·무효화 책임이 소비자별로 분산','검토 질문  |  분산된 view가 같은 revision·삭제 상태를 보존하는가?')))
    for s in range(2):
        p=Pane(f,s)
        p.group('sources',30,80,1180,190,'원본 owner의 versioned read port / 허용된 Context Source',kind='boundary')
        for ident,x,name,a,b in [('im',60,'Interaction Manager','timeline · selection','당시 evidence refs'),('tm',350,'Task Manager','Task/artifact revision','확인된 상태'),('rmport',640,'Response Manager','실제 표시·청취 범위','publication revision'),('source',930,'Context Source','문서·메일·화면 자료','범위·원문·source version')]:
            p.leaf(ident,x,150,250,95,name,a,b,parent='sources')
        if s==0:
            p.group('cm',30,390,1180,320,'Context Manager',diff=True)
            p.leaf('read',60,465,245,95,'조회 조정','owner read port','bounded source adapter',parent='cm')
            p.leaf('receipt',350,465,245,95,'Receipt / coverage','후보·누락·revision','읽은 근거의 범위',parent='cm',diff=True)
            p.leaf('view',640,465,245,95,'Evidence Package','원문·근거·요약 view','소비 목적별 구성',parent='cm',diff=True)
            p.leaf('cache',930,465,250,95,'Cache / invalidation','source·policy epoch','memory epoch·의존 index',parent='cm',diff=True)
            p.leaf('viewstate',350,615,535,65,'공통 파생 view 상태','VALID / DIRTY / INVALID · owner 원본은 별도',parent='cm',diff=True)
            p.link('read','sources',[(182,465),(182,455),(330,455),(330,330),(630,330),(630,270)],'① owner snapshot / source receipt',(640,330),diff=True,both=True)
            p.link('read','receipt',[(305,510),(350,510)])
            p.link('receipt','view',[(595,510),(640,510)],diff=True)
            p.link('view','cache',[(885,510),(930,510)],diff=True)
            p.link('view','viewstate',[(760,560),(760,615)],state=True,diff=True)
            p.leaf('rc',30,825,550,110,'Request Controller','② 해석 근거 수신 · Conversation 근거 결합','허용 범위·예산 · Request Interpreter 호출',kind='component',diff=True)
            p.leaf('rm',660,825,550,110,'Response Manager','③ admission된 사실로 응답 구성','새 업무 판단은 Request Controller로 반환',kind='component',diff=True)
            p.link('viewstate','rc',[(500,680),(500,765),(305,765),(305,825)],'근거 package',(460,765),diff=True)
            p.link('rc','rm',[(580,880),(660,880)],'payload',(620,860),diff=True)
        else:
            p.group('rc',30,390,550,320,'Request Controller',diff=True)
            p.group('rm',660,390,550,320,'Response Manager',diff=True)
            for owner,x,title,line in [('rc',60,'요청용 Context 준비','Conversation·Task·지칭 후보'),('rm',690,'응답용 Context 준비','admission된 사실의 표현 근거')]:
                p.leaf(owner+'-view',x,465,490,95,title,line,'owner read port / receipt · coverage',parent=owner,diff=True)
                p.leaf(owner+'-cache',x,615,490,65,'소비자별 view · dependency index','source/policy/memory epoch로 무효화',parent=owner,diff=True)
                p.link(owner+'-view',owner+'-cache',[(x+245,560),(x+245,615)],state=True,diff=True)
            p.link('rc-view','sources',[(305,465),(305,325),(450,325),(450,270)],'① 해석 근거',(305,330),diff=True,both=True)
            p.link('rm-view','sources',[(935,465),(935,325),(800,325),(800,270)],'① 허용된 응답 근거',(935,330),diff=True,both=True)
            p.leaf('cm',30,825,1180,110,'Context Manager','공통 source adapter·read cache·명시적 User Memory owner는 유지','소비자별 view를 소유하지 않음 / 조회·무효화 알림 port 제공',kind='component',diff=True)
            p.link('rc-cache','cm',[(305,680),(305,825)],'② 읽기 / 무효화',(305,770),diff=True,both=True)
            p.link('rm-cache','cm',[(935,680),(935,825)],'② 읽기 / 무효화',(935,770),diff=True,both=True)
            p.link('rc','rm',[(580,585),(660,585)],'③ payload',(620,566),diff=True)
        p.leaf('policy',30,1020,310,95,'Policy Manager','scope · recipient · epoch',kind='component')
        p.leaf('store',395,1020,440,95,'State Store','원본·memory revision·tombstone','view는 원본을 대체하지 않음',kind='component')
        p.leaf('model',890,1020,320,95,'Model Access','동일한 공유 Omni','요약은 재생성 가능한 view',kind='component')
        p.note(30,985,'공통 사용 계약: 허용 범위 검사 → source revision 보존 → 철회·삭제 시 사용 차단',False,20)
    return f


def agent_figure():
    f=Figure('agent-integration-boundary','Agent protocol과 실행 연동 상태의 경계',
      '상황  |  Agent A는 순서 있는 push event를, Agent B는 polling을 제공한다. submit 응답이 유실될 수도 있다.',
      ('방안 1  |  공통 Agent Gateway lifecycle + provider adapter','방안 2  |  Agent별 integration Component가 lifecycle 소유'),
      'VIA Task identity·업무 payload owner·capability 검증·UNKNOWN 보존·조회 전 재전송 금지는 동일하다.',
      (('얻는 것  |  전송·inbox·cursor·capability 규칙을 공통 계약으로 관리','부담  |  provider별 차이가 공통 lifecycle의 예외·확장으로 쌓일 수 있음','검토 질문  |  adapter만으로 Agent 차이를 충분히 가둘 수 있는가?'),
       ('얻는 것  |  provider별 상태·재접속·version의 독립 변경','부담  |  lifecycle·내구 원장·capability extension 관리가 분산','검토 질문  |  기존 adapter와 달라진 소유권이 실제로 존재하는가?')))
    for s in range(2):
        p=Pane(f,s)
        p.leaf('tm',30,80,1180,130,'Task Manager','① immutable Agent Command / Task·Execution binding / source-confirmed projection','업무 payload의 의미와 상태는 이 Component가 소유',kind='component')
        if s==0:
            p.group('gateway',30,340,1180,440,'Agent Gateway',diff=True)
            p.leaf('delivery',60,415,300,100,'전송 상태·재조회','PENDING → DISPATCHING','ACK / UNKNOWN · key 조회',parent='gateway',diff=True)
            p.leaf('inbox',470,415,300,100,'Inbox / cursor','중복·gap·source sequence','canonical Agent Event',parent='gateway',diff=True)
            p.leaf('cap',880,415,300,100,'Capability profile','제어·query·idempotency','profile revision',parent='gateway',diff=True)
            p.leaf('a',60,650,490,90,'Agent A adapter','push event · protocol/schema 변환',parent='gateway')
            p.leaf('b',690,650,490,90,'Agent B adapter','polling · protocol/schema 변환',parent='gateway')
            p.link('tm','delivery',[(390,210),(390,400),(210,400),(210,415)],'② 공통 command port',(390,278),diff=True)
            p.link('inbox','tm',[(620,415),(620,210)],'④ canonical event',(625,295),diff=True)
            p.link('delivery','cap',[(330,415),(330,400),(1030,400),(1030,415)],'capability 검증',(865,400),both=True,diff=True)
            p.link('delivery','a',[(210,515),(210,650)])
            p.link('delivery','b',[(360,470),(390,470),(390,575),(935,575),(935,650)],'provider binding',(690,575))
            p.link('a','inbox',[(550,695),(620,695),(620,515)])
            p.link('b','inbox',[(935,650),(935,600),(715,600),(715,515)])
        else:
            for ident,x,title in [('ga',30,'Agent Integration A'),('gb',660,'Agent Integration B')]:
                p.group(ident,x,340,550,440,title,diff=True)
                p.leaf(ident+'-delivery',x+30,415,225,100,'전송·재조회','ACK / UNKNOWN','command key',parent=ident,diff=True)
                p.leaf(ident+'-inbox',x+295,415,225,100,'Inbox / cursor','sequence / gap','canonical event',parent=ident,diff=True)
                p.leaf(ident+'-cap',x+30,550,490,65,'Capability · version · epoch','공통 최소 port + 명시적 extension',parent=ident,diff=True)
                p.leaf('a' if ident=='ga' else 'b',x+30,650,490,90,'Agent A adapter' if ident=='ga' else 'Agent B adapter','push event · protocol/schema 변환' if ident=='ga' else 'polling · protocol/schema 변환',parent=ident)
                p.link('tm',ident+'-delivery',[(x+265,210),(x+265,400),(x+145,400),(x+145,415)],'② command',(x+220,276),diff=True)
                p.link(ident+'-inbox','tm',[(x+440,415),(x+440,210)],'④ event',(x+440,295),diff=True)
                p.link(ident+'-delivery',ident+'-cap',[(x+145,515),(x+145,550)],diff=True)
                p.link(ident+'-cap','a' if ident=='ga' else 'b',[(x+145,615),(x+145,650)],diff=True)
                p.link('a' if ident=='ga' else 'b',ident+'-inbox',[(x+520,695),(x+535,695),(x+535,530),(x+440,530),(x+440,515)])
        p.group('store',30,835,1180,150,'State Store')
        p.leaf('db',60,900,1120 if s==0 else 490,70,'공통 delivery attempt · inbox · cursor / command ID로 Task payload와 연결' if s==0 else 'Agent A delivery · inbox · cursor',kind='store',parent='store',diff=True)
        if s==1:p.leaf('dbb',690,900,490,70,'Agent B delivery · inbox · cursor',kind='store',parent='store',diff=True)
        p.link('gateway' if s==0 else 'ga','db',[(570,780),(570,805),(500,805),(500,900)],state=True,diff=True)
        if s==1:p.link('gb','dbb',[(1130,780),(1130,900)],state=True,diff=True)
        p.leaf('agent-a',30,1040,550,85,'Downstream Agent A','업무 reasoning·planning·실행 / 외부 dependency',kind='dependency')
        p.leaf('agent-b',660,1040,550,85,'Downstream Agent B','업무 reasoning·planning·실행 / 외부 dependency',kind='dependency')
        p.link('a','agent-a',[(60,695),(15,695),(15,1000),(305,1000),(305,1040)],'③ submit / event',(305,1000),both=True)
        p.link('b','agent-b',[(1180,695),(1225,695),(1225,1000),(935,1000),(935,1040)],'③ submit / poll',(935,1000),both=True)
        p.note(650,810,'전송·수신 상태 owner만 변경 / 업무 의미는 Task Manager',True,18)
    return f


def speech_figure():
    f=Figure('speech-evidence-boundary','음성 입력 근거와 공유 추론의 장애 경계',
      '상황  |  긴 semantic 추론 중 사용자가 표를 다시 가리킨다. 녹음뿐 아니라 인식·시간 근거도 계속 제공해야 한다.',
      ('방안 1  |  독립 Speech Input Worker + Omni 동시 session','방안 2  |  Omni 입력 capability에 전사·시간 근거 통합'),
      '공유 Omni weights 1회 적재·동시 VOICE/SEMANTIC·유한 자원·capture/local stop·gap 보고는 유지한다.',
      (('얻는 것  |  Omni 부하·재시작과 입력 근거 생성을 분리','부담  |  ASR weights·CPU·buffer, 중복 인식·불일치 조정','검토 질문  |  독립 입력 경로가 그 전체 비용을 정당화하는가?'),
       ('얻는 것  |  별도 ASR·worker·전사 불일치 경로 제거 가능','부담  |  입력 인식과 semantic이 같은 runtime 장애에 묶임','조건부  |  동시 입력·revision·시간 근거 capability는 아직 미확인')))
    for s in range(2):
        p=Pane(f,s)
        p.group('im',30,80,1180,190,'Interaction Manager')
        p.leaf('capture',60,150,300,95,'Channel I/O','audio sample sequence','capture · RAM ring',parent='im')
        p.leaf('timeline',470,150,300,95,'Timeline & Buffer','전사 revision · 시점·gap','화면·포인터 근거 결합',parent='im')
        p.leaf('stop',880,150,300,95,'Turn-Taking Control','playback · output epoch','local stop / 추론 대기 없음',parent='im')
        p.leaf('access',30,350,1180,95,'Model Access','① audio/control stream → adapter · ② SpeechEvidence/revision → 입력 기록','아래 runtime에 client·adapter·session 관리 책임이 나뉘어 배치됨',kind='component',diff=True)
        p.link('capture','access',[(210,245),(210,350)],'audio',(210,310))
        p.link('access','timeline',[(620,350),(620,245)],'SpeechEvidence',(620,310),diff=True)
        if s==0:
            p.group('asr',30,540,550,360,'Speech Input Worker',diff=True,kind='runtime')
            p.leaf('asradapter',60,615,490,80,'Model Access · ASR adapter','CPU 예산 · transcript revision · 시간 정규화',parent='asr',diff=True)
            p.leaf('asrmodel',60,755,225,110,'Streaming ASR','독립 경량 dependency','weights · 인식 오류',parent='asr',kind='dependency',diff=True)
            p.leaf('asrbuffer',325,755,225,110,'입력 근거 buffer','partial / final','span timing · gap',parent='asr',diff=True)
            p.group('omni',660,540,550,360,'Shared Inference Service',diff=True,kind='runtime')
            p.leaf('scheduler',690,615,490,80,'Model Access · scheduler','Voice·semantic 최소 연산/KV · safe-point cancel',parent='omni')
            p.leaf('voice',690,755,225,110,'VOICE session','Omni 입력 해석','승인 음성 생성',parent='omni',diff=True)
            p.leaf('semantic',955,755,225,110,'SEMANTIC session','구조화 해석','역할별 context/KV',parent='omni')
            p.link('access','asradapter',[(310,445),(310,615)],'독립 입력',(310,495),diff=True,both=True)
            p.link('access','scheduler',[(1040,445),(1040,615)],'Omni stream',(1040,495),diff=True,both=True)
            p.link('asradapter','asrmodel',[(172,695),(172,755)],diff=True)
            p.link('asrmodel','asrbuffer',[(285,810),(325,810)],diff=True)
            p.link('asrbuffer','asradapter',[(437,755),(437,695)],diff=True)
        else:
            p.group('omni',30,540,1180,360,'Shared Inference Service',diff=True,kind='runtime')
            p.leaf('inputcap',60,615,490,80,'필수 Omni 입력 capability','전사 revision · span timing·오차 · gap',parent='omni',diff=True)
            p.leaf('scheduler',690,615,490,80,'Model Access · scheduler','Voice·semantic 최소 연산/KV · safe-point cancel',parent='omni')
            p.leaf('inputstate',60,755,490,110,'통합 입력 근거 상태','상세 SpeechEvidence 제공 필요','별도 ASR fallback을 숨겨 추가하지 않음',parent='omni',diff=True)
            p.leaf('voice',690,755,225,110,'VOICE session','입력 근거까지 제공','승인 음성 생성',parent='omni',diff=True)
            p.leaf('semantic',955,755,225,110,'SEMANTIC session','구조화 해석','역할별 context/KV',parent='omni')
            p.link('access','inputcap',[(600,445),(600,605),(310,605),(310,615)],'통합 입력',(600,495),diff=True,both=True)
            p.link('access','scheduler',[(1040,445),(1040,615)],'Omni stream',(1040,495),diff=True,both=True)
            p.link('inputcap','inputstate',[(300,695),(300,755)],diff=True)
        p.leaf('rc',30,1010,550,105,'Request Controller','③ 입력 revision 확정 · 정정/hold · 근거 부족 처리','ASR/Omni 불일치 조정' if s==0 else '통합 입력의 revision·gap 검증',kind='component',diff=True)
        p.leaf('weights',660,1010,550,105,'공유 Omni · resident weights','Thinker + encoder + Talker/decoder 별도 inventory','두 역할의 KV·workspace는 별도 비용',kind='dependency')
        p.note(30,950,'③ 인식/시간 근거가 없으면 commit 금지 · gap·확인 불가 안내',False,20)
        p.link('timeline','rc',[(470,195),(450,195),(450,290),(15,290),(15,985),(305,985),(305,1010)],'InputFinal + evidence',(160,985))
        p.link('scheduler','voice',[(802,695),(802,755)])
        p.link('scheduler','semantic',[(1067,695),(1067,755)])
        p.link('omni','weights',[(1120,900),(1120,1010)],state=True)
    return f


def runtime_figure():
    f=Figure('runtime-isolation','실시간 입출력과 상태 처리의 process 경계',
      '상황  |  Core 또는 UI 코드가 멈추거나 종료될 때 음성 stop·입력 근거·Task 추적은 어디까지 유지되는가?',
      ('방안 1  |  UI·Voice·Core process 분리 / IPC와 incarnation 검사','방안 2  |  host process 통합 / 독립 thread와 bounded queue'),
      '논리 Component·비동기 처리·추론/ASR/위험 connector 격리·내구 복구 계약·동일 모델 구성은 유지한다.',
      (('얻는 것  |  Core process-fatal 오류와 local stop의 운명 분리','부담  |  IPC·복사·queue·clock mapping·lease·재연결 관리','실패 경계  |  Core 단절 → 새 admission 중단 / Voice는 로컬 stop 유지'),
       ('얻는 것  |  내부 전달·buffer 참조 공유와 host lifecycle 단순화','부담  |  host process-fatal 오류에서 UI·Voice·Core가 함께 중단','실패 경계  |  thread 격리는 blocking 제어이며 process crash 격리는 아님')))
    for s in range(2):
        p=Pane(f,s)
        if s==0:
            p.group('ui',30,80,550,295,'UI process',diff=True,kind='runtime')
            p.group('voice',660,80,550,295,'Voice process',diff=True,kind='runtime')
            p.group('core',30,510,1180,490,'Core process',diff=True,kind='runtime')
        else:p.group('host',30,80,1180,920,'Host process',diff=True,kind='runtime')
        pu='ui' if s==0 else 'host';pv='voice' if s==0 else 'host';pc='core' if s==0 else 'host'
        p.group('imui',60,150,490,190,'Interaction Manager',parent=pu)
        p.leaf('capture',85,220,210,90,'Evidence Capture','화면·포인터·선택','producer time·gap',parent='imui')
        p.leaf('uiio',325,220,195,90,'Channel I/O','Text·UI 入出力'.replace('入出力','입출력'),'명시 stop',parent='imui')
        p.group('imvoice',690,150,490,190,'Interaction Manager',parent=pv)
        p.leaf('audio',715,220,205,90,'Channel I/O','capture · playback','유한 audio buffer',parent='imvoice')
        p.leaf('localstop',935,220,235,90,'Turn-Taking Control','output epoch · lease','즉시 local stop',parent='imvoice')
        entries=[('timeline',60,585,'Interaction Manager','Timeline & Buffer','clock mapping · revision'),('rc',460,585,'Request Controller','Request·hold·admission','Conversation 상태'),('rm',860,585,'Response Manager','release·cancel·receipt','publication 원장'),('cm',60,735,'Context Manager','근거·read cache','User Memory'),('tm',460,735,'Task Manager','Task·Execution 상태','command·reconciliation'),('policy',860,735,'Policy Manager','scope·consent revision','현재 권한'),('ag',60,885,'Agent Gateway','command·inbox·cursor','connector client'),('ma',460,885,'Model Access','semantic client','동시 session 계약'),('store',860,885,'State Store','embedded DB access','데이터는 process 종료 후 유지')]
        entries=[e for e in entries if e[0] not in ('cm','tm','policy')]
        entries += [('cm',60,735,'Context Manager','근거·read cache','User Memory'),('tm',350,735,'Task Manager','Task·Execution 상태','command·reconcile'),('ri',640,735,'Request Interpreter','의미·추가 읽기 제안','Model Call Envelope'),('policy',930,735,'Policy Manager','scope·consent revision','현재 권한')]
        for ident,x,y,title,a,b in entries:p.leaf(ident,x,y,270 if y==735 else 350,90,title,a,b,kind='component',parent=pc)
        p.link('capture','timeline',[(185,310),(185,485),(250,485),(250,585)],'① evidence IPC' if s==0 else '① evidence queue',(185,425),diff=True)
        p.link('audio','timeline',[(815,310),(815,460),(340,460),(340,585)],'② 입력·time·gap',(600,460),diff=True)
        p.link('rm','localstop',[(1035,585),(1035,310)],'③ release / receipt',(1060,425),diff=True,both=True)
        p.link('timeline','rc',[(410,630),(460,630)])
        p.link('rc','rm',[(810,630),(860,630)])
        p.link('rc','tm',[(500,675),(500,735)])
        p.link('rc','ri',[(775,675),(775,735)],both=True)
        p.link('ri','ma',[(775,825),(775,855),(635,855),(635,885)],both=True)
        p.leaf('asr',30,1060,360,65,'Speech Input Worker','독립 ASR process',kind='dependency')
        p.leaf('omni',440,1060,360,65,'Shared Inference Service','공유 Omni process',kind='dependency')
        p.leaf('connector',850,1060,360,65,'Connector Worker','위험 연동 별도 process',kind='dependency')
        p.link('ma','omni',[(635,975),(635,1060)],'공통 IPC',(635,1025))
        p.note(30,1040,'Voice의 ASR client 및 Agent/source connector 연결은 양쪽 동일',False,19)
    return f


def text_width(text,size):
    return sum(size if ord(c)>255 else size*.57 for c in text)


def render_svg(f):
    def text(x,y,value,size=20,color=BLACK,weight=400,anchor='start'):
        return f'<text x="{x}" y="{y}" font-size="{size}" font-weight="{weight}" fill="{color}" text-anchor="{anchor}">{escape(value)}</text>'
    s=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{f.width}" height="{f.height}" viewBox="0 0 {f.width} {f.height}" role="img" aria-labelledby="title desc">',
       f'<title id="title">{escape(f.title)}</title><desc id="desc">{escape(f.scenario)} 검정은 공통, 파랑은 양쪽의 구조적 차이.</desc>',
       '<defs>'+''.join(f'<marker id="arrow-{k}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0L10 5L0 10Z" fill="{c}"/></marker>' for k,c in [('black',BLACK),('blue',BLUE)])+'</defs>',
       f'<rect width="{f.width}" height="{f.height}" fill="white"/>',f'<g font-family="{FONT}">',text(40,62,f.title,40,weight=700),text(40,108,f.scenario,24),
       text(40,157,'검정  공통 구조·기능·계약',22,weight=700),text(430,157,'파랑  양쪽 방안의 차이: 책임 경계·상태 owner·연결',22,BLUE,700),
       text(1410,157,'실선 → 호출·event    점선 → 상태 접근    원통 = 저장 기록',21)]
    for i in range(2):
        x=40+1280*i
        s.extend([f'<rect x="{x}" y="210" width="1240" height="1160" fill="white" stroke="{BLACK}" stroke-width="2"/>',
          f'<rect x="{x}" y="210" width="1240" height="56" fill="#F2F2F2" stroke="{BLACK}"/>',text(x+20,246,f.labels[i],25,weight=700)])
    for n in f.nodes:
        c=BLUE if n.diff else BLACK
        is_group=n.kind in ('component','runtime','boundary') and not n.lines
        dash=' stroke-dasharray="10 7"' if n.kind in ('runtime','dependency') else ''
        if n.kind=='store':
            s.append(f'<path d="M{n.x},{n.y+12} C{n.x},{n.y-4} {n.x+n.w},{n.y-4} {n.x+n.w},{n.y+12} L{n.x+n.w},{n.y+n.h-12} C{n.x+n.w},{n.y+n.h+4} {n.x},{n.y+n.h+4} {n.x},{n.y+n.h-12}Z" fill="white" stroke="{c}" stroke-width="2"/>')
            s.append(f'<path d="M{n.x},{n.y+12} C{n.x},{n.y+28} {n.x+n.w},{n.y+28} {n.x+n.w},{n.y+12}" fill="none" stroke="{c}" stroke-width="1.5"/>')
        else:
            s.append(f'<rect x="{n.x}" y="{n.y}" width="{n.w}" height="{n.h}" rx="{0 if is_group else 3}" fill="white" stroke="{c}" stroke-width="{3 if is_group else 2}"{dash}/>')
        if is_group:
            kind={'runtime':'process / runtime','boundary':'공개 조회 계약'}.get(n.kind,'component')
            kind += ' · 대안 전용' if n.title in ('Direct Response Delivery','Task Notification Delivery','Agent Integration A','Agent Integration B') else ''
            s.append(text(n.x+14,n.y+25,f'«{kind}»',16,c))
            s.append(text(n.x+14,n.y+51,n.title,24,c,700))
        else:
            size=22 if n.kind in ('component','dependency') else 20
            cy=n.y+(n.h-(len(n.lines)*25))/2+7
            if n.kind=='store':cy=max(cy,n.y+43)
            s.append(text(n.x+n.w/2,cy,n.title,size,c,700,'middle'))
            for j,line in enumerate(n.lines):s.append(text(n.x+n.w/2,cy+27+j*25,line,18,c,400,'middle'))
    edge_labels=[]
    for e in f.links:
        c=BLUE if e.diff else BLACK;k='blue' if e.diff else 'black'
        dash=' stroke-dasharray="7 5"' if e.state else ''
        start=f' marker-start="url(#arrow-{k})"' if e.both else ''
        s.append(f'<polyline points="'+ ' '.join(f'{x},{y}' for x,y in e.points)+'" fill="none" stroke="white" stroke-width="6" stroke-linejoin="round"/>')
        s.append(f'<polyline points="'+ ' '.join(f'{x},{y}' for x,y in e.points)+f'" fill="none" stroke="{c}" stroke-width="2.3" stroke-linejoin="round" marker-end="url(#arrow-{k})"{start}{dash}/>')
        if e.label and e.at:
            x,y=e.at;w=text_width(e.label,19)+16
            edge_labels.append(f'<rect x="{x-w/2}" y="{y-19}" width="{w}" height="27" fill="white"/>')
            edge_labels.append(text(x,y,e.label,19,c,600,'middle'))
    s.extend(edge_labels)
    for x,y,value,diff,size in f.captions:s.append(text(x,y,value,size,BLUE if diff else BLACK))
    s.append(text(40,1413,'공통 조건  |  '+f.common,22))
    for i in range(2):
        x=40+1280*i
        s.append(f'<rect x="{x}" y="1440" width="1240" height="160" fill="#F7F7F7" stroke="{BLACK}"/>')
        for j,line in enumerate(f.notes[i]):s.append(text(x+20,1477+j*44,line,22))
    s.append(text(40,1648,'내부 블록은 책임의 설명용 분해이며 새 승인 Component가 아님 · 기대 효과는 가설 · ASR·측정 수치는 후속 검토',21))
    s.append(text(40,1680,'검토 기준선과 논의용 대안의 비교 / 구현·성능 측정 없음',19))
    s.append(text(2560,1680,f.slug+'.drawio',17,anchor='end'))
    return '\n'.join(s+['</g></svg>'])+'\n'


def render_drawio(f):
    mx=ET.Element('mxfile',{'host':'app.diagrams.net','type':'device','version':'24.7.17'})
    diagram=ET.SubElement(mx,'diagram',{'name':f.title,'id':f.slug})
    graph=ET.SubElement(diagram,'mxGraphModel',{'page':'1','pageWidth':str(f.width),'pageHeight':str(f.height),'grid':'1','gridSize':'10'})
    root=ET.SubElement(graph,'root');ET.SubElement(root,'mxCell',{'id':'0'});ET.SubElement(root,'mxCell',{'id':'1','parent':'0'})
    by_id={n.id:n for n in f.nodes}
    def cell(ident,x,y,w,h,value,style,parent='1'):
        c=ET.SubElement(root,'mxCell',{'id':ident,'value':value,'style':style,'vertex':'1','parent':parent})
        ET.SubElement(c,'mxGeometry',{'x':str(x),'y':str(y),'width':str(w),'height':str(h),'as':'geometry'})
    def label(ident,x,y,value,size=20,color=BLACK,w=2400):
        cell(ident,x,y-size,w,size+12,escape(value),f'text;html=1;align=left;verticalAlign=middle;whiteSpace=wrap;fontSize={size};fontColor={color};')
    label('page-title',40,62,f.title,40);label('scenario',40,108,f.scenario,24)
    label('legend-common',40,157,'검정  공통 구조·기능·계약',22,w=370)
    label('legend-diff',430,157,'파랑  양쪽 방안의 차이: 책임 경계·상태 owner·연결',22,BLUE,w=950)
    label('legend-shape',1410,157,'실선 → 호출·event    점선 → 상태 접근    원통 = 저장 기록',21,w=1150)
    for i in range(2):
        x=40+1280*i
        cell('pane'+str(i),x,210,1240,1160,'',f'fillColor=#FFFFFF;strokeColor={BLACK};strokeWidth=2;')
        cell('pane-heading'+str(i),x,210,1240,56,escape(f.labels[i]),f'html=1;align=left;spacingLeft=20;fontSize=25;fontStyle=1;fillColor=#F2F2F2;strokeColor={BLACK};')
    for n in f.nodes:
        c=BLUE if n.diff else BLACK;group=n.kind in ('component','runtime','boundary') and not n.lines
        typ={'runtime':'process / runtime','boundary':'공개 조회 계약'}.get(n.kind,'component')
        typ += ' · 대안 전용' if n.title in ('Direct Response Delivery','Task Notification Delivery','Agent Integration A','Agent Integration B') else ''
        value=(f'<div style="font-size:16px">«{typ}»</div><b style="font-size:24px">{escape(n.title)}</b>' if group else f'<b>{escape(n.title)}</b>'+''.join(f'<div style="font-size:18px">{escape(t)}</div>' for t in n.lines))
        style=f'html=1;whiteSpace=wrap;fillColor=#FFFFFF;strokeColor={c};fontColor={c};fontSize=20;strokeWidth={3 if group else 2};'
        if group:style+='container=1;collapsible=0;recursiveResize=0;align=left;verticalAlign=top;spacingTop=10;spacingLeft=14;'
        else:style+='align=center;verticalAlign=middle;spacing=5;'
        if n.kind=='store':style+='shape=cylinder3;size=12;'
        if n.kind in ('runtime','dependency'):style+='dashed=1;dashPattern=10 7;'
        parent=by_id.get(n.parent)
        cell(n.id,n.x-(parent.x if parent else 0),n.y-(parent.y if parent else 0),n.w,n.h,value,style,n.parent)
    for e in f.links:
        a=by_id[e.source];b=by_id[e.target];c=BLUE if e.diff else BLACK
        sx,sy=e.points[0];tx,ty=e.points[-1]
        style=f'edgeStyle=none;jumpStyle=arc;jumpSize=8;html=1;strokeColor={c};strokeWidth=2.3;endArrow=block;endFill=1;startArrow={"block" if e.both else "none"};startFill=1;exitX={(sx-a.x)/a.w};exitY={(sy-a.y)/a.h};entryX={(tx-b.x)/b.w};entryY={(ty-b.y)/b.h};exitPerimeter=0;entryPerimeter=0;'
        if e.state:style+='dashed=1;dashPattern=7 5;'
        ec=ET.SubElement(root,'mxCell',{'id':e.id,'edge':'1','source':e.source,'target':e.target,'parent':'1','style':style})
        g=ET.SubElement(ec,'mxGeometry',{'relative':'1','as':'geometry'})
        if len(e.points)>2:
            arr=ET.SubElement(g,'Array',{'as':'points'})
            for x,y in e.points[1:-1]:ET.SubElement(arr,'mxPoint',{'x':str(x),'y':str(y)})
        if e.label and e.at:
            x,y=e.at;w=text_width(e.label,19)+16
            cell(e.id+'-label',x-w/2,y-19,w,27,escape(e.label),f'html=1;align=center;verticalAlign=middle;fillColor=#FFFFFF;strokeColor=none;fontSize=19;fontColor={c};fontStyle=1;')
    for i,(x,y,value,diff,size) in enumerate(f.captions):label('caption'+str(i),x,y,value,size,BLUE if diff else BLACK,w=text_width(value,size)+20)
    label('common',40,1413,'공통 조건  |  '+f.common,22)
    for i in range(2):
        x=40+1280*i
        cell('notes-bg'+str(i),x,1440,1240,160,'',f'fillColor=#F7F7F7;strokeColor={BLACK};')
        for j,line in enumerate(f.notes[i]):label(f'note-{i}-{j}',x+20,1477+j*44,line,22,w=1200)
    label('scope',40,1648,'내부 블록은 책임의 설명용 분해이며 새 승인 Component가 아님 · 기대 효과는 가설 · ASR·측정 수치는 후속 검토',21)
    label('status',40,1680,'검토 기준선과 논의용 대안의 비교 / 구현·성능 측정 없음',19)
    ET.indent(mx,space='  ')
    return '<?xml version="1.0" encoding="UTF-8"?>\n'+ET.tostring(mx,encoding='unicode')+'\n'


def validate(f):
    ids={n.id for n in f.nodes}
    assert len(ids)==len(f.nodes),f.slug
    by_id={n.id:n for n in f.nodes}
    groups={n.id for n in f.nodes if n.kind in ('component','runtime','boundary') and not n.lines}
    for n in f.nodes:
        assert 0<=n.x and n.x+n.w<=f.width and n.y+n.h<=1370,(f.slug,n.id)
        size=24 if n.id in groups else 22 if n.kind in ('component','dependency') else 20
        assert text_width(n.title,size)<n.w-16,(f.slug,n.id,n.title)
        for line in n.lines:
            assert text_width(line,18)<n.w-12,(f.slug,n.id,line)
        if n.parent!='1':
            p=by_id[n.parent]
            assert p.x<=n.x and p.y+55<=n.y and n.x+n.w<=p.x+p.w and n.y+n.h<=p.y+p.h,(f.slug,n.id,'containment')
    def on_border(point,n):
        x,y=point
        return ((x in (n.x,n.x+n.w) and n.y<=y<=n.y+n.h) or
                (y in (n.y,n.y+n.h) and n.x<=x<=n.x+n.w))
    for e in f.links:
        assert e.source in ids and e.target in ids
        assert on_border(e.points[0],by_id[e.source]),(f.slug,e.id,'source port')
        assert on_border(e.points[-1],by_id[e.target]),(f.slug,e.id,'target port')
        for a,b in zip(e.points,e.points[1:]):
            assert (a[0]==b[0]) != (a[1]==b[1]),(f.slug,e.id,'orthogonal')
            for n in f.nodes:
                if n.id in groups or n.id in (e.source,e.target):continue
                hit=(n.x<a[0]<n.x+n.w and max(min(a[1],b[1]),n.y)<min(max(a[1],b[1]),n.y+n.h)) if a[0]==b[0] else (n.y<a[1]<n.y+n.h and max(min(a[0],b[0]),n.x)<min(max(a[0],b[0]),n.x+n.w))
                assert not hit,(f.slug,e.id,'crosses leaf',n.id)
    for content in (render_svg(f),render_drawio(f)):
        tree=ET.fromstring(content)
        xml_ids=[e.attrib['id'] for e in tree.iter() if 'id' in e.attrib]
        assert len(xml_ids)==len(set(xml_ids)),(f.slug,'duplicate XML ID')


def figures():
    return [request_figure(),context_figure(),task_figure(),response_figure(),agent_figure(),speech_figure(),runtime_figure()]


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--check',action='store_true')
    parser.add_argument('--output-dir',type=Path,default=OUTPUT)
    args=parser.parse_args()
    for f in figures():
        validate(f)
        for suffix,content in [('.svg',render_svg(f)),('.drawio',render_drawio(f))]:
            path=args.output_dir/(f.slug+suffix)
            if args.check:
                if not path.exists() or path.read_text()!=content:raise SystemExit(f'FAIL: stale or missing {path}')
            else:
                path.parent.mkdir(parents=True,exist_ok=True)
                path.write_text(content,encoding='utf-8')
        print(('PASS: ' if args.check else 'WROTE: ')+f.slug)


if __name__=='__main__':main()
