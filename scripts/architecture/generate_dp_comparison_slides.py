"""Reference-shaped editable comparison slides. Numbers are format hypotheses.

Preserves the Architecture comparisons; exports independent presentation assets.
"""
from __future__ import annotations

import argparse
import copy
import hashlib
import html
import json
import re
from pathlib import Path
import xml.etree.ElementTree as ET

from generate_dp_background_slides import Slide, INK, MUTED, TEAL, BLUE, RED, LINE

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/presentation_files/dp-comparison'
GREEN = '#D5E7C7'
PURPLE = '#7155A4'

QA = {
    'V-01': ('기능 정확성', '정확 처리율', '↑'),
    'V-03': ('기능 완전성', '기능 지원 달성률', '↑'),
    'V-04': ('응답 신속성', 'p95 반응 지연', '↓'),
    'V-05': ('요청 완료 신속성', 'p95 VIA 완료시간', '↓'),
    'V-06': ('메모리 효율성', '최대 전체 메모리', '↓'),
    'V-08': ('변경 용이성', '평균 변경 요소 수', '↓'),
}

# A row = QA, left/right value, left/right relative circle count.
DATA = {
  41: dict(title='기능 정확성을 위한 VIA Request 해석 설계',
    options=('A 모델 중심 ReAct 해석', 'B 모델 틀 생성과 코드 중심 의미 완성'),
    case='“이 표를 아까 보고서에 넣고, 메일은 초안만 만들어.” / 보고서 Task 후보 2개',
    condition='비교 예시 조건: 열린 관계 표현 포함 / 시간 비교는 양안 지원 범위의 같은 Request',
    rows=[('V-03','100%','87.2%',3,2),('V-05','6.0 s','4.5 s',2,3),('V-08','2.0개','4.0개',3,1)],
    pros=[['열린 관계 표현의 지원 확대, 지원율 100% (기능 완전성 ↑)',
           '새 관계의 코드 변경 축소, 평균 2.0개 (변경 용이성 ↑)'],
          ['지원 관계의 코드 결합으로 완료시간 4.5 s (요청 완료 신속성 ↑)']],
    cons=[['반복 조회와 모델 재판단으로 완료시간 6.0 s (요청 완료 신속성 ↓)'],
          ['유한 표현 체계 밖 관계의 지원 제한, 지원율 87.2% (기능 완전성 ↓)',
           '새 관계의 틀과 결합 규칙 확장, 평균 4.0개 (변경 용이성 ↓)']]),
  42: dict(title='변경 용이성을 위한 VIA Conversation과 Task 관리 설계',
    options=('A 모듈형 통합 VIA Core', 'B 독립 대화 서비스와 업무 서비스'),
    case='보고서와 메일 Request 접수 → Task 추적 → 같은 보고서 Task의 후속 수정',
    condition='비교 예시 조건: 업무 저장과 실행 수명의 독립 변경 / 일반 Agent adapter 변경은 제외',
    rows=[('V-05','3.2 s','3.8 s',3,2),('V-08','4.0개','2.5개',2,3),('V-06','13.2 GB','13.6 GB',3,2)],
    pros=[['관련 상태의 공동 확정으로 완료시간 3.2 s (요청 완료 신속성 ↑)',
           '추가 서비스 heap 축소로 전체 메모리 13.2 GB (메모리 효율성 ↑)'],
          ['업무 상태와 실행 경계의 독립 변경, 평균 2.5개 (변경 용이성 ↑)']],
    cons=[['공동 저장과 복구 경로의 변경 전파, 평균 4.0개 (변경 용이성 ↓)'],
          ['명령 접수와 대화 반영의 왕복으로 3.8 s (요청 완료 신속성 ↓)',
           '별도 heap과 대기열로 전체 메모리 13.6 GB (메모리 효율성 ↓)']]),
  43: dict(title='기능 정확성을 위한 VIA Request 의미 판단 설계',
    options=('A 통합 의미 생산', 'C 기능별 의미 생산과 코드 조정'),
    case='“보고서는 PDF로, 메일에는 결론을 넣어줘.” → “메일에는 결론 대신 표만 넣어줘.”',
    condition='비교 예시 조건: 교차 의미 정정 / 특정 지칭 기능의 독립 변경 / 동일한 공유 Omni',
    rows=[('V-01','94%','91%',3,2),('V-05','5.5 s','7.2 s',3,2),('V-08','4.0개','2.0개',1,3)],
    pros=[['관련 의미의 공동 판단으로 정확 처리율 94% (기능 정확성 ↑)',
           '부분 제안 간 교환 축소로 완료시간 5.5 s (요청 완료 신속성 ↑)'],
          ['특정 기능의 생산 책임 분리로 변경 평균 2.0개 (변경 용이성 ↑)']],
    cons=[['통합 의미 생산 계약의 변경 전파, 평균 4.0개 (변경 용이성 ↓)'],
          ['부분 의미 불일치와 오류 전파로 정확 처리율 91% (기능 정확성 ↓)',
           '제안 교환과 재판단으로 완료시간 7.2 s (요청 완료 신속성 ↓)']]),
  44: dict(title='반응성을 위한 VIA 지속 입력과 Response 전달 설계',
    options=('A 중앙 비동기 Orchestration', 'B 반응형 Dataflow 실행'),
    case='보고서 설명 도중 새 발화와 메일 질문 도착 / 표 설명 후 메일 질문 전달',
    condition='비교 예시 조건: 여러 사건의 집중 도착 / 양안의 입력 수신과 로컬 음성 중단은 동일',
    rows=[('V-04','0.85 s','0.62 s',2,3),('V-08','4.0개','2.5개',2,3),('V-06','13.3 GB','13.8 GB',3,2)],
    pros=[['집중된 실행 상태로 전체 메모리 13.3 GB (메모리 효율성 ↑)'],
          ['사건의 직접 활성화로 p95 반응 지연 0.62 s (응답 신속성 ↑)',
           '동일 schema 안의 단계 조합 변경, 평균 2.5개 (변경 용이성 ↑)']],
    cons=[['반환 회수와 재배정으로 p95 반응 지연 0.85 s (응답 신속성 ↓)',
           '교차 사건의 중앙 전이 변경, 평균 4.0개 (변경 용이성 ↓)'],
          ['다수 window와 buffer로 전체 메모리 13.8 GB (메모리 효율성 ↓)']]),
  45: dict(title='기능 정확성을 위한 VIA 기억과 Context 제공 설계',
    options=('A 원본 서비스 조합', 'B 공통 파생 기억의 생산과 조회'),
    case='“지난번에 내가 고친 표현 방식으로 이번 보고서도 정리해줘.”',
    condition='비교 예시 조건: 게시 범위 안 과거 관계의 반복 재사용 / A의 index와 cache도 허용',
    rows=[('V-01','90%','94%',2,3),('V-05','7.0 s','4.5 s',2,3),('V-06','13.2 GB','14.0 GB',3,2)],
    pros=[['필요한 요청 중심의 처리로 전체 메모리 13.2 GB (메모리 효율성 ↑)'],
          ['검증된 과거 관계 재사용으로 정확 처리율 94% (기능 정확성 ↑)',
           '원본별 교차 결합 축소로 완료시간 4.5 s (요청 완료 신속성 ↑)']],
    cons=[['반복 교차 결합의 관계 누락으로 정확 처리율 90% (기능 정확성 ↓)',
           '여러 원본 조회와 결합으로 완료시간 7.0 s (요청 완료 신속성 ↓)'],
          ['관계 생산 session과 KV로 전체 메모리 14.0 GB (메모리 효율성 ↓)']]),
}


class Comparison(Slide):
    def __init__(self, n):
        self.number, self.items = n, []
        self.slug = f'dp{n}-comparison'
        self.caption = f'04-{n}. {DATA[n]["title"]}'
        self.rect(0,0,1920,1080,'white','none')
        self.text(40,22,1300,['VIA SOFTWARE ARCHITECTURE / DP 설계 비교'],19,TEAL,True)
        self.text(1880,22,450,['구조 비교 / 장단점 / QA Trade-off'],19,MUTED,align='right')
        self.text(40,65,1840,[self.caption],39,INK,True)
        self.line([(40,121),(1880,121)],color=LINE,arrow=False)
        self.text(40,142,1530,[DATA[n]['case']],23,INK)
        self.text(1880,145,290,['수치와 점수 / 예상 예시'],19,RED,True,align='right')
        self.rect(40,188,1840,55,GREEN,LINE)
        for y,h,label in [(188,55,'설계안'),(243,494,'구조'),(737,85,'장점'),(822,85,'단점'),(907,125,'QA\nTrade-off')]:
            if y!=188: self.rect(40,y,1840,h,'white',LINE)
            self.text(90,y+17,98,label.split('\n'),21,INK,True,'center',leading=26)
        for x in [140,1010]: self.line([(x,188),(x,1032)],color=LINE,arrow=False)
        for side,x in enumerate([140,1010]):
            self.text(x+435,203,838,[DATA[n]['options'][side]],28,INK,True,'center')
            for label,y in [('pros',751),('cons',836)]:
                for i,line in enumerate(DATA[n][label][side]):
                    self.text(x+18,y+i*31,836,['▪ '+line],21,INK)
            for i,(qa,left,right,lc,rc) in enumerate(DATA[n]['rows']):
                y=920+i*34;name,metric,direction=QA[qa]
                self.text(x+18,y,335,[f'{qa} {name}'],21,INK)
                self.text(x+375,y-1,112,['●'*(lc if side==0 else rc)+'○'*(3-(lc if side==0 else rc))],25,INK)
                self.text(x+503,y,140,[left if side==0 else right],23,INK,True)
                self.text(x+652,y+2,206,[f'{metric} {direction}'],18,MUTED)
        self.text(40,1050,1520,[DATA[n]['condition']],18,MUTED)
        self.text(1880,1050,315,['● 많을수록 우수 / 미측정'],18,MUTED,align='right')

    def box(self,x,y,w,h,name,color=INK,fill='white',size=21):
        self.rect(x,y,w,h,fill,color)
        lines=name.split('\n')
        self.text(x+w/2,y+(h-len(lines)*size*1.30)/2,w-18,lines,size,color,True,'center',leading=size*1.30)

    def group(self,x,y,w,h,name,color):
        self.rect(x,y,w,h,'#FAFBFC',color)
        self.text(x+13,y+10,w-26,[name],21,color,True)

    def store(self,x,y,w,h,name,size=18):
        self.add('store',x=x,y=y,w=w,h=h,fill='#FFF7DE',stroke=MUTED,dashed=False)
        self.text(x+w/2,y+17,w-18,name.split('\n'),size,INK,False,'center',leading=size*1.2)

    def label(self,x,y,w,text,size=18,color=MUTED):
        self.text(x,y,w,text.split('\n'),size,color,leading=size*1.2)

    def transformed(self):
        return [dict(i,kind='rect') if i['kind']=='store' else i for i in self.items]

    def svg(self):
        original=self.items
        try:
            self.items=self.transformed();result=super().svg().replace('— DP 배경','— 설계 비교')
            marker=f'<marker id="a{PURPLE[1:]}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto"><path d="M1 1L9 5L1 9" fill="none" stroke="{PURPLE}" stroke-width="1.6"/></marker>'
            result=result.replace('</defs>',marker+'</defs>')
        finally: self.items=original
        for i in original:
            if i['kind']!='store': continue
            x,y,w,h=(i[k] for k in ('x','y','w','h'));r=8
            path=f'<path id="{i["id"]}" fill="{i["fill"]}" stroke="{i["stroke"]}" stroke-width="1.7" d="M{x} {y+r}C{x} {y-r} {x+w} {y-r} {x+w} {y+r}V{y+h-r}C{x+w} {y+h+r} {x} {y+h+r} {x} {y+h-r}Z M{x} {y+r}C{x} {y+3*r} {x+w} {y+3*r} {x+w} {y+r}"/>'
            result=re.sub(r'<rect id="'+i['id']+r'"[^>]*/>',lambda _:path,result)
        return result

    def diagram(self):
        original=self.items
        try: self.items=self.transformed();d=super().diagram()
        finally: self.items=original
        stores={i['id'] for i in original if i['kind']=='store'}
        for cell in d.findall('.//mxCell'):
            if cell.get('id') in stores: cell.set('style',cell.get('style')+'shape=cylinder;size=8;')
        return d


def graph41(s,x,side):
    c=BLUE if side==0 else TEAL
    s.box(x+245,275,380,47,'Request Controller')
    s.line([(x+435,322),(x+435,374)],color=c)
    s.label(x+450,333,315,'원문과 후보 정보',18)
    s.box(x+665,399,180,49,'Model Access',size=20)
    s.label(x+674,462,170,'공유 Omni 1벌',18)
    if side==0:
        s.group(x+25,374,590,220,'Request Interpreter',c)
        s.box(x+50,421,250,53,'ReAct 해석 제어기',c)
        s.box(x+335,421,255,53,'읽기 도구 실행기',c)
        s.line([(x+300,437),(x+335,437)],color=c)
        s.line([(x+335,459),(x+300,459)],color=c)
        s.label(x+333,477,270,'모델 선택 조회 / 정보 반환',17)
        s.box(x+50,528,540,44,'의미 제안 검증기',c)
        s.line([(x+175,474),(x+175,528)],color=c)
        s.label(x+190,507,330,'완성 의미 제안과 조건 검사',17)
        s.line([(x+665,423),(x+630,423),(x+630,408),(x+175,408),(x+175,421)],color=MUTED,dashed=True)
        s.line([(x+590,447),(x+630,447),(x+630,626),(x+525,626),(x+525,652)],color=c)
        s.label(x+642,551,205,'허용 범위의\n정보 조회',17)
        s.line([(x+175,572),(x+175,652)],color=c)
        s.label(x+190,610,255,'현재 검사와 의미 채택',17)
    else:
        s.box(x+55,387,540,57,'Request Interpreter',c)
        s.line([(x+595,416),(x+665,416)],color=MUTED)
        s.line([(x+325,444),(x+325,511)],color=c)
        s.label(x+340,471,270,'미해결 항목을 포함한 요청 틀',17)
        s.group(x+25,511,590,112,'Request Resolution Engine',c)
        s.box(x+50,551,250,51,'미해결 항목 처리기',c,size=20)
        s.box(x+335,551,255,51,'관계 결합기',c,size=20)
        s.line([(x+300,576),(x+335,576)],color=c)
        s.line([(x+50,576),(x+12,576),(x+12,415),(x+55,415)],color=c,dashed=True)
        s.label(x+28,464,260,'필요한 부분의 모델 해석',17)
        s.line([(x+175,602),(x+175,613),(x+630,613),(x+630,638),(x+525,638),(x+525,652)],color=c)
        s.line([(x+525,652),(x+525,645),(x+638,645),(x+638,582),(x+590,582)],color=c,dashed=True)
        s.line([(x+462,602),(x+462,635),(x+175,635),(x+175,652)],color=c)
    s.box(x+40,652,270,45,'Request Controller',size=20)
    s.box(x+400,652,250,45,'Context Manager',size=20)
    s.box(x+680,652,165,45,'Task Manager',size=19)
    s.line([(x+680,674),(x+650,674)],color=MUTED)
    s.label(x+28,711,805,'공통: 동일 정보와 권한 / 채택 후 응답 또는 Task Delegation',18)


def graph42(s,x,side):
    c=BLUE if side==0 else TEAL
    if side==0:
        s.group(x+22,279,820,397,'VIA Core',c)
        s.box(x+48,341,280,53,'대화 처리기',c)
        s.box(x+371,341,280,53,'업무 관리기',c)
        s.label(x+61,403,290,'Conversation과 Request',18)
        s.label(x+386,403,265,'Task와 Agent Execution',18)
        s.box(x+205,458,300,51,'트랜잭션 관리자',c)
        s.line([(x+48,367),(x+36,367),(x+36,442),(x+278,442),(x+278,458)],color=c)
        s.line([(x+630,394),(x+630,442),(x+434,442),(x+434,458)],color=c)
        s.store(x+220,567,280,64,'통합 상태 저장소')
        s.line([(x+355,509),(x+355,567)],color=c)
        s.label(x+374,535,250,'관련 변경의 공동 확정',18)
        s.box(x+630,466,190,48,'Agent 연동기',c)
        s.line([(x+651,367),(x+725,367),(x+725,466)],color=c)
        s.label(x+656,411,169,'저장 후 전송',17)
        s.box(x+630,692,205,35,'Downstream Agent',size=18)
        s.line([(x+725,514),(x+725,692)],color=MUTED)
        s.label(x+38,690,560,'공동 상태 확정 / 대화와 업무 owner의 같은 Core 수명',17)
    else:
        s.group(x+22,279,355,348,'대화 서비스',c)
        s.group(x+423,279,419,348,'업무 서비스',c)
        s.box(x+49,341,300,53,'대화 처리기',c)
        s.box(x+446,341,370,53,'업무 관리기',c)
        s.label(x+60,402,288,'Conversation과 Request',18)
        s.label(x+457,402,350,'Task와 Agent Execution',18)
        s.line([(x+349,365),(x+446,365)],color=c)
        s.label(x+354,321,100,'명령',17,c)
        s.line([(x+446,378),(x+403,378),(x+403,441),(x+199,441),(x+199,394)],color=c,dashed=True)
        s.label(x+165,459,360,'접수 결과와 event /\n대화 쪽 별도 반영',17)
        s.store(x+65,541,267,64,'대화 상태 저장소')
        s.store(x+446,541,225,64,'업무 상태 저장소')
        s.line([(x+49,379),(x+37,379),(x+37,518),(x+199,518),(x+199,541)],color=c)
        s.line([(x+816,378),(x+828,378),(x+828,525),(x+559,525),(x+559,541)],color=c)
        s.box(x+692,541,131,57,'Agent\n연동기',c,size=20)
        s.line([(x+671,573),(x+692,573)],color=c)
        s.box(x+599,692,236,35,'Downstream Agent',size=18)
        s.line([(x+757,598),(x+757,692)],color=MUTED)
        s.label(x+38,649,570,'독립 원본과 로컬 확정 / 독립 서비스 수명',18)
        s.label(x+38,688,520,'서비스 접수와 외부 Agent 접수의 구분',17)


def graph43(s,x,side):
    c=BLUE if side==0 else PURPLE
    s.box(x+245,279,380,47,'Request Controller')
    s.line([(x+435,326),(x+435,381 if side==0 else 431)],color=c)
    s.label(x+451,336,360,'원문과 정보 / Task 연결은 판단 결과',17)
    if side==1: s.label(x+451,359,360,'실선: 제안 / 점선: 재판단 요청',17)
    s.group(x+22,381,820,259,'Request Interpreter',c)
    if side==1: s.line([(x+435,381),(x+435,431)],color=c)
    if side==0:
        s.box(x+135,436,597,58,'Integrated Semantic Interpreter',c,size=23)
        s.store(x+245,548,380,64,'Interpretation Attempt State')
        s.line([(x+435,494),(x+435,548)],color=c)
        s.label(x+39,520,785,'공동 판단 / 검색과 부분 수정 및 전문 보조 허용',18)
    else:
        names=['Request Intent\nInterpreter','Referent Resolver','Request Association\nResolver',
               'Request Relation\nInterpreter','Request Revision\nInterpreter','Request Handling\nInterpreter']
        for i,name in enumerate(names):
            col=i//3;row=i%3;xx=x+40+col*551;yy=431+row*61
            s.box(xx,yy,237,50,name,c,size=18)
            if col==0:
                s.line([(xx+237,yy+15),(x+310,yy+15)],color=c)
                s.line([(x+310,yy+36),(xx+237,yy+36)],color=c,dashed=True)
            else:
                s.line([(xx,yy+15),(x+557,yy+15)],color=c)
                s.line([(x+557,yy+36),(xx,yy+36)],color=c,dashed=True)
        s.box(x+310,431,247,172,'Interpretation\nCoordinator',INK,size=22)
        s.store(x+310,603,247,36,'Partial Interpretation Ledger',size=15)
    s.box(x+30,679,205,45,'Model Access',size=20)
    s.box(x+291,679,520,45,'Request Controller',size=21)
    s.line([(x+22,432),(x+12,432),(x+12,701),(x+30,701)],color=MUTED,dashed=True)
    s.label(x+31,650,246,'공유 Omni 1벌',17)
    s.line([(x+551,640),(x+551,679)],color=c) if side==0 else s.line([(x+557,486),(x+850,486),(x+850,664),(x+551,664),(x+551,679)],color=c)
    s.label(x+570,640,263,'현재 검사와 의미 채택',17)


def graph44(s,x,side):
    c=BLUE if side==0 else TEAL
    s.box(x+29,279,331,48,'Interaction Manager',size=21)
    s.box(x+475,279,350,48,'Task Manager',size=21)
    s.label(x+40,338,320,'확정 입력 / 즉시 stop와 hold',17)
    s.label(x+487,338,338,'확인된 질문과 결과',17)
    if side==0:
        s.group(x+150,400,547,148,'Interaction Orchestrator',c)
        s.box(x+174,445,246,53,'Dialogue Dispatcher',c,size=19)
        s.store(x+463,438,211,66,'Dialogue Progress\nState',size=17)
        s.line([(x+29,303),(x+15,303),(x+15,377),(x+130,377),(x+130,471),(x+174,471)],color=c)
        s.line([(x+650,327),(x+650,384),(x+543,384),(x+543,400)],color=c)
        s.line([(x+420,461),(x+463,461)],color=c)
        s.line([(x+463,490),(x+420,490)],color=c,dashed=True)
        s.box(x+29,601,331,48,'Request Controller',size=21)
        s.box(x+475,601,350,48,'Response Manager',size=21)
        s.line([(x+250,548),(x+194,548),(x+194,601)],color=c)
        s.line([(x+550,548),(x+650,548),(x+650,601)],color=c)
        s.label(x+28,565,352,'해석 요청 / 반환 회수',17)
        s.label(x+476,565,350,'후속 준비와 게시 요청',17)
        s.line([(x+825,624),(x+851,624),(x+851,470),(x+697,470)],color=c,dashed=True)
        s.label(x+713,470,125,'반환',17)
    else:
        s.label(x+32,386,795,'Reactive Interaction Runtime / typed channel과 bounded credit',18,c)
        s.box(x+29,430,331,49,'Input Resolution Stage',c,size=20)
        s.box(x+475,430,350,49,'Task Notice Stage',c,size=20)
        s.line([(x+29,303),(x+15,303),(x+15,454),(x+29,454)],color=c)
        s.line([(x+650,327),(x+650,430)],color=c)
        s.box(x+29,516,331,48,'Request Controller',size=21)
        s.box(x+475,516,350,48,'Response Manager',size=21)
        s.line([(x+194,479),(x+194,516)],color=c)
        s.line([(x+650,479),(x+650,516)],color=c)
        s.line([(x+360,541),(x+475,541)],color=c)
        s.label(x+250,577,380,'채택 의미와 유효한 Notice',17)
        s.box(x+293,605,367,44,'Publication Join',c,size=21)
        s.line([(x+650,564),(x+650,588),(x+600,588),(x+600,605)],color=c)
        s.line([(x+293,627),(x+12,627),(x+12,454),(x+29,454)],color=c,dashed=True)
        s.label(x+30,601,215,'credit / cancel',17,c)
    if side==0:
        s.box(x+275,686,378,40,'Interaction Manager',size=21)
        s.line([(x+650,649),(x+650,673),(x+464,673),(x+464,686)],color=c)
        s.label(x+675,657,175,'현재성 확인과 게시',17)
        s.label(x+30,700,235,'실제 표시와 재생',17)
    else:
        s.box(x+500,684,325,42,'Response Manager',size=21)
        s.box(x+29,684,350,42,'Interaction Manager',size=21)
        s.line([(x+660,627),(x+850,627),(x+850,673),(x+795,673),(x+795,684)],color=c)
        s.label(x+505,657,275,'현재성 확인과 게시',17)
        s.line([(x+500,705),(x+379,705)],color=c)
        s.label(x+390,681,105,'실제 전달',16)


def graph45(s,x,side):
    c=BLUE if side==0 else TEAL
    s.box(x+245,279,380,47,'Request Controller')
    s.label(x+452,339,347,'같은 원문과 현재 적용 목적',17)
    s.group(x+22,381,820,128 if side==0 else 267,'Context Manager',c)
    if side==0:
        s.box(x+227,429,420,53,'Context Composer',c,size=23)
        s.line([(x+435,326),(x+435,429)],color=c)
        for xx,name in [(x+45,'Request Controller'),(x+306,'Task Manager'),(x+567,'Response Manager')]:
            s.box(xx,557,240,43,name,size=19)
            s.line([(x+435,482),(x+435,518),(xx+120,518),(xx+120,557)],color=c)
        s.label(x+44,613,744,'요청별 원본 조회와 교차 결합 / index와 cache 허용',18)
        s.line([(x+227,454),(x+12,454),(x+12,705),(x+245,705)],color=c)
    else:
        s.box(x+49,431,273,48,'Memory Publisher',c,size=21)
        s.box(x+559,431,253,48,'Evidence Reader',c,size=21)
        s.store(x+350,508,318,77,'Episodic Memory\nRepository',size=20)
        s.line([(x+435,326),(x+435,367),(x+685,367),(x+685,431)],color=c)
        s.line([(x+186,479),(x+186,547),(x+350,547)],color=c)
        s.label(x+44,492,295,'원본별 허용 기록 → 생산과 게시',17)
        s.line([(x+668,547),(x+685,547),(x+685,479)],color=c)
        s.label(x+689,502,137,'조회',17,c)
        s.label(x+48,607,770,'정정과 결과 버전 및 전달 관계 / coverage와 유효성 확인',18)
        s.line([(x+812,456),(x+855,456),(x+855,705),(x+625,705)],color=c)
        s.store(x+30,650,775,30,'원본 기록 / Request Controller, Task Manager, Response Manager',size=16)
        s.line([(x+30,665),(x+8,665),(x+8,455),(x+49,455)],color=c)
        s.label(x+28,346,405,'허용 기록 변경 또는 미생산 범위 보완',17)
    s.box(x+245,684,380,42,'Request Interpreter',size=21)
    s.label(x+640,687,210,'현재 의미 판단',18)


def build():
    outputs={};deck=ET.Element('mxfile',host='app.diagrams.net',type='device')
    for n in DATA:
        s=Comparison(n)
        for side,x in enumerate([140,1010]): globals()[f'graph{n}'](s,x,side)
        outputs[OUT/f'{s.slug}.svg']=s.svg()
        outputs[OUT/f'{s.slug}.drawio']=s.drawio()
        deck.append(copy.deepcopy(s.diagram()))
    outputs[OUT/'VIA-DP-comparison-41-45.drawio']=ET.tostring(deck,encoding='unicode')+'\n'
    outputs[OUT/'comparison-data.json']=json.dumps(dict(status='FORMAT_HYPOTHESES_NOT_MEASURED',qa=QA,slides=DATA),ensure_ascii=False,indent=2)+'\n'
    blocks=''.join(f'<article><h2>04-{n} 설계'+f' 비교</h2><p><a href="dp{n}-comparison.drawio">draw.io</a> / <a href="dp{n}-comparison.svg">SVG</a> / <a href="dp{n}-comparison.png">PNG</a></p><img src="dp{n}-comparison.svg" alt="04-{n} 설계 비교"></article>' for n in DATA)
    outputs[OUT/'index.html']='<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA DP 설계 비교</title><style>body{margin:30px auto;max-width:1440px;font-family:Arial,sans-serif;background:#f3f5f7;color:#18232e}article{margin:30px 0}img{width:100%;background:white}a{color:#087e8b}</style><h1>VIA DP 설계 비교 41~45</h1><p>수치와 동그라미 점수는 형식 검토용 예상 예시. 실측 결과 또는 대안 선정 아님.</p><p><a href="VIA-DP-comparison-41-45.drawio">5페이지 편집 원본</a></p>'+blocks+'</html>\n'
    return outputs


def validate(outputs):
    """Check comparisons, rather than assuming the circle counts tell the truth."""
    presentation=json.loads((ROOT/'docs/presentations_files/quality-attributes/quality-attributes.json').read_text())
    names={r['id']:r['name'] for r in presentation['rows']}
    for qa,(name,_,_) in QA.items():
        if names[qa]!=name: raise SystemExit(f'Presentation QA name changed: {qa}')
    for n,data in DATA.items():
        expected=[set(),set()]
        for qa,left,right,lc,rc in data['rows']:
            lv=float(re.search(r'[0-9.]+',left)[0]);rv=float(re.search(r'[0-9.]+',right)[0])
            left_wins=lv>rv if QA[qa][2]=='↑' else lv<rv
            if lv==rv or (lc>rc)!=left_wins or not(0<=lc<=3 and 0<=rc<=3):
                raise SystemExit(f'Numeric and circle ranking conflict: {n}/{qa}')
            expected[0 if left_wins else 1].add(QA[qa][0])
        all_names={QA[r[0]][0] for r in data['rows']}
        for side in (0,1):
            for kind,direction,wanted in [('pros','↑',expected[side]),('cons','↓',all_names-expected[side])]:
                found=set()
                for line in data[kind][side]:
                    match=re.search(r'\(([^()]+) ([↑↓])\)$',line)
                    if not match or match[2]!=direction: raise SystemExit(f'QA ending missing: {n}/{kind}')
                    found.add(match[1])
                if found!=wanted: raise SystemExit(f'Benefits/costs disagree with numeric winners: {n}/{side}/{kind}')
    for p,source in outputs.items():
        if p.suffix=='.svg':
            tree=ET.fromstring(source)
            ids={e.get('id') for e in tree.iter()}
            for marker in re.findall(r'marker-end="url\(#([^)]*)\)"',source):
                if marker not in ids: raise SystemExit(f'Missing rendered arrow marker: {p}/{marker}')
            if '·' in source: raise SystemExit(f'Unexpected middle dot: {p}')
        elif p.suffix=='.drawio':
            tree=ET.fromstring(source)
            for page in tree.findall('diagram'):
                m=page.find('mxGraphModel');cells=page.findall('.//mxCell')
                ids=[c.get('id') for c in cells]
                if len(ids)!=len(set(ids)) or m.get('pageWidth')!='1920' or m.get('pageHeight')!='1080':
                    raise SystemExit(f'Invalid native page: {p}')


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');args=parser.parse_args()
    outputs=build();validate(outputs)
    for p,s in outputs.items():
        if args.check:
            if not p.exists() or p.read_text()!=s: raise SystemExit(f'Mismatch: {p}')
        else: p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s)
    if args.check:
        manifest=json.loads((OUT/'render-manifest.json').read_text())
        if len(manifest['slides'])!=5: raise SystemExit('Expected five rendered slides')
        for slide in manifest['slides']:
            for ext in ('svg','png'):
                if hashlib.sha256((OUT/f'{slide["slug"]}.{ext}').read_bytes()).hexdigest()!=slide[f'{ext}_sha256']:
                    raise SystemExit(f'Stale preview: {slide["slug"]}.{ext}')
    print(f'PASS: {len(outputs)} comparison source/preview files '+('verified' if args.check else 'written'))


if __name__=='__main__': main()
