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
OUT = ROOT / 'docs/presentations_files/dp-comparison'
GREEN = '#D5E7C7'
PURPLE = '#7155A4'

QA = {
    'V-01': ('기능 정확성', '정확 처리율', '↑'),
    'V-03': ('기능 완전성', '기능 지원 달성률', '↑'),
    'V-04': ('응답 신속성', '평균 반응시간', '↓'),
    'V-05': ('요청 완료 신속성', '평균 VIA 처리시간', '↓'),
    'V-06': ('메모리 효율성', '최대 전체 메모리', '↓'),
    'V-08': ('변경 용이성', '평균 변경 요소 수', '↓'),
}

# A row = QA, left/right value, left/right relative circle count.
DATA = {
  41: dict(title='VIA Request 이해의 모델 판단 반복과 코드 해결 반복',
    options=('A 모델 주도 해석', 'B 모델 요청 틀과 코드 주도 완성'),
    case='“이 표를 아까 보고서에 넣고, 메일은 보내지 말고 결과를 바탕으로 초안만 만들어.”',
    status='CONDITIONAL_QUALITATIVE_NOT_MEASURED',
    condition='미선정 / 미측정. B도 부분 의미 판단을 다시 호출할 수 있다. 새 관계의 지원 범위와 질문 비용을 함께 비교한다.',
    tradeoffs=[
      ('정확성 / 지원 범위','열린 의미를 함께 판단 / 검색 누락과 모델 오판 가능','지원 관계의 코드 결합 / 틀 오류와 표현 밖 관계의 한계'),
      ('응답성','조회 후 재판단마다 클라우드 왕복 / 부분 수정 허용','지원 틀이 충분하면 재판단 축소 / 추가 해석과 질문 대기'),
      ('모델 호출 비용','반복 해석과 긴 Context가 비용 증가 요인','초기 틀 + 필요한 부분 해석 / 항상 1회라는 전제 없음'),
      ('변경 용이성','새 관계는 모델 지침/조회 도구 변경 가능','요청 틀과 코드 규칙 확장 / 기존 지원 관계는 코드 제어')]),
  42: dict(title='대화와 Task 상태의 소유 및 확정',
    options=('A 모듈형 통합 Core / 공동 확정', 'B 독립 대화 / Task 서비스 / 별도 접수'),
    case='보고서 기간 질문을 실제 제시 → “응, 상반기로 해줘” → 내부 접수 / 외부 전달 확인',
    status='CONDITIONAL_QUALITATIVE_NOT_MEASURED',
    condition='미선정 / 미측정. B의 별도 수명 이익은 조건부다. 일반 adapter 변경이나 정상 사용에서 자동 우위는 없다.',
    tradeoffs=[('요청 처리 정확성', '같은 의미 판단 / 공동 확정은 의미 정답 보증 아님', '같은 판단 / 접수와 대화 반영의 일치 관리 필요'), ('상호작용 응답시간', '한 번의 관련 변경 확정 / 음성 반응 경로 공통', '의도 / 접수 / 반영 왕복 / 음성 반응 경로 공통'), ('VIA 요청 처리시간', '같은 Core에서 공동 확정 뒤 명령 전송', '서비스 간 명령과 receipt 연결 / pending 확인 비용'), ('VIA 모델 사용 비용', '같은 의미 판단이면 동일 / 저장은 코드 처리', '같은 의미 판단이면 동일 / 인계는 모델 호출 아님'), ('VIA 변경 용이성', '내부 Module 변경 가능 / 공동 확정 계약 영향', '저장과 실행 수명 독립 / 서비스 계약의 호환 비용'), ('로컬 VIA 메모리 사용량', '같은 실행체의 상태 / 실행 대기열', '별도 실행체와 각 대기열 / 접수 대기 상태 유지')]),
  43: dict(title='기능 정확성을 위한 VIA Request 의미 판단 설계',
    options=('A 통합 의미 생산', 'C 기능별 의미 생산과 코드 조정'),
    case='“보고서는 PDF로, 메일에는 결론을 넣어줘.” → “메일에는 결론 대신 표만 넣어줘.”',
    condition='비교 예시 조건: 교차 의미 정정 / 특정 지칭 기능의 독립 변경 / 동일한 공유 Omni',
    rows=[('V-01','94%','91%',3,2),('V-05','5.5 s','7.2 s',3,2),('V-08','4.0개','2.0개',1,3)],
    pros=[['관련 의미의 공동 판단으로 정확 처리율 94% (기능 정확성 ↑)',
           '부분 제안 간 교환 축소로 평균 VIA 처리시간 5.5 s (요청 완료 신속성 ↑)'],
          ['특정 기능의 생산 책임 분리로 변경 평균 2.0개 (변경 용이성 ↑)']],
    cons=[['통합 의미 생산 계약의 변경 전파, 평균 4.0개 (변경 용이성 ↓)'],
          ['부분 의미 불일치와 오류 전파로 정확 처리율 91% (기능 정확성 ↓)',
           '제안 교환과 재판단으로 평균 VIA 처리시간 7.2 s (요청 완료 신속성 ↓)']]),
  44: dict(title='VIA 지속 입력과 Task 알림의 후속 실행 구조',
    options=('A 중앙에서 완료 회수와 다음 실행 지시', 'B 연결된 단계가 입력과 조건으로 실행'),
    case='“잠깐, 표부터 설명해줘.” + 메일 수신자 질문 도착 + 이전 응답 준비가 늦게 완료',
    status='CONDITIONAL_QUALITATIVE_NOT_MEASURED',
    condition='미선정 / 미측정. A도 비동기 / 우선순위 / 동시 실행. B의 사건 채널만으로 정확성 / 속도 우위를 보장하지 않는다.',
    tradeoffs=[('요청 처리 정확성', '같은 의미 판단 / 중앙에서 현재 입력과 대기 참조 관리', '같은 판단 / 각 Window와 Join의 참조 일치 필요'), ('상호작용 응답시간', '중앙의 완료 회수와 다음 지시 / 짧은 dispatch 가능', '준비된 소비자 활성화 / 채널과 Join 대기 비용'), ('VIA 요청 처리시간', '단계마다 중앙 전이를 경유 / 병행 실행 허용', '중앙 경유 축소 / 조건 대기와 단계 연결 비용'), ('VIA 모델 사용 비용', '같은 의미 / 응답 작업이면 같음 / 폐기와 재시도 포함', '같은 작업이면 같음 / 중복 생산과 폐기 포함'), ('VIA 변경 용이성', '후속 실행 규칙을 Dispatcher에서 변경', '단계 계약 안 조합 변경 / 교차 규칙은 여러 단계 영향'), ('로컬 VIA 메모리 사용량', '중앙 진행 상태와 실행 대기열', '여러 Window와 사건 채널 / 수용량 제한 필요')]),
  45: dict(title='VIA 과거 정보의 Request별 구성과 공통 생산 및 조회',
    options=('A 원본 서비스 조합형', 'B 공통 파생 기억 저장소형'),
    case='“지난번 네가 설명한 평가 기준에 맞춰, 앞서 조사한 제품 비교 결과와 받아둔 견적을 이번 제안서에 반영해줘.”',
    status='CONDITIONAL_QUALITATIVE_NOT_MEASURED',
    condition='미선정 / 미측정. A cache의 같은 효과는 B 고유 이익을 줄인다. B+ 원본 조합 우회는 명시적 혼합안.',
    tradeoffs=[('요청 처리 정확성', '새 관계를 현재 원문에서 구성 / 검색 누락 가능', '게시 표현과 추출 범위에 제한 / 원본 확인 유지'), ('상호작용 응답시간', '반복 조회도 원본 구성 / 유효 관계 cache로 축소', '유효 게시 재사용 시 단축 / 첫 생산과 갱신 대기'), ('VIA 요청 처리시간', '현재 목적에 맞춘 조회와 결합 / 병렬 읽기 허용', '준비된 관계 읽기 / 미생산 범위는 생산 후 재조회'), ('VIA 모델 사용 비용', 'Request 구성과 cache 유지 / 부분 갱신 비용 포함', '초기 / 변경 / 미사용 생산 포함 / 반복 사용 시 상각'), ('VIA 변경 용이성', '조회 지침과 adapter / cache 형식 변경 시 무효화', '관계 schema / producer와 reader 호환 / 재생산'), ('로컬 VIA 메모리 사용량', '원본 읽기 view와 Request 수명의 묶음 / 선택 cache', '공통 관계 저장소와 Coverage / 미사용 관계도 유지')]),
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
        if n==45:
            self.memory_page()
            return
        if n==42:
            self.lifecycle_page()
            return
        if n==44:
            self.execution_page()
            return
        if n==41:
            self.request_page()
            return
        self.text(40,133 if n==44 else 142,1530,[DATA[n]['case']],21 if n==45 else 23,INK)
        self.text(1880,145,290,['수치와 점수 / 예상 예시'],19,RED,True,align='right')
        if n==44:
            self.text(40,168,1530,['양안 공통: Agent Gateway ↔ Downstream Agent / Model Access ↔ 공유 Omni 1벌'],14,MUTED)
        self.rect(40,188,1840,55,GREEN,LINE)
        for y,h,label in [(188,55,'설계안'),(243,552,'구조'),(795,80,'장점'),(875,80,'단점'),(955,95,'QA\nTrade-off')]:
            if y!=188: self.rect(40,y,1840,h,'white',LINE)
            self.text(90,y+17,98,label.split('\n'),21,INK,True,'center',leading=26)
        for x in [140,1010]: self.line([(x,188),(x,1050)],color=LINE,arrow=False)
        for side,x in enumerate([140,1010]):
            if n==44:
                self.text(x+435,192,838,[DATA[n]['options'][side]],25,INK,True,'center')
                self.text(x+435,224,838,['중앙 비동기 Orchestration' if side==0 else '반응형 Dataflow'],14,INK,align='center')
            else:
                self.text(x+435,203,838,[DATA[n]['options'][side]],28,INK,True,'center')
            for label,y in [('pros',808),('cons',888)]:
                for i,line in enumerate(DATA[n][label][side]):
                    self.text(x+18,y+i*29,836,['▪ '+line],20,INK)
            for i,(qa,left,right,lc,rc) in enumerate(DATA[n]['rows']):
                y=964+i*27;name,metric,direction=QA[qa]
                self.text(x+18,y,335,[f'{qa} {name}'],20,INK)
                self.text(x+375,y-1,112,['●'*(lc if side==0 else rc)+'○'*(3-(lc if side==0 else rc))],23,INK)
                self.text(x+503,y,140,[left if side==0 else right],21,INK,True)
                self.text(x+652,y+2,206,[f'{metric} {direction}'],17,MUTED)
        self.text(40,1057,1520,[DATA[n]['condition']],16,MUTED)
        self.text(1880,1057,315,['● 많을수록 우수 / 미측정'],16,MUTED,align='right')

    def memory_page(self):
        d=DATA[45]
        self.text(40,137,1840,[d['case']],20,INK)
        self.text(40,168,1840,['공통: VIA가 실제 설명한 내용 + 제품 조사와 견적의 결과 / 현재 제안서 업무 연결은 해석 결과 / 로컬 VIA + 클라우드 모델'],17,INK)
        self.rect(40,195,1840,48,GREEN,LINE)
        for side,x in enumerate([140,1010]):
            self.text(x+435,206,838,[d['options'][side]],25,INK,True,'center')
        self.line([(1010,195),(1010,795)],color=LINE,arrow=False)
        self.line([(40,795),(1880,795)],color=LINE,arrow=False)
        self.text(40,809,1840,['PARTIAL: 실제 전달한 예외 문구 한 구간과 결과 참조의 예시 / 기준 전체의 완전 추출을 뜻하지 않음 / 과거 게시와 현재 의미 채택은 별개'],16,INK)
        self.text(40,838,1840,['첫 요청 / 준비된 반복 조회 / 정정 직후 / 표현 밖 관계를 구분. A의 유효 관계 cache와 부분 갱신도 허용한다.'],16,INK)
        self.quality_footer(d)

    def request_page(self):
        d=DATA[41]
        self.text(40,140,1840,['입력 Input1: '+d['case']],23,INK)
        self.text(40,169,1840,['Task1: 예산 보고서 / Task2: 실적 보고서 → “어느 보고서인가요?” → 사용자: “예산 보고서” / 결과: 수정된 보고서 → 메일 초안'],19,INK)
        self.rect(40,195,1840,48,GREEN,LINE)
        self.text(90,209,98,['설계안'],19,INK,True,'center')
        for side,x in enumerate([140,1010]):
            self.text(x+435,205,838,[d['options'][side]],26,INK,True,'center')
        self.line([(1010,195),(1010,872)],color=LINE,arrow=False)
        self.line([(40,872),(1880,872)],color=LINE,arrow=False)
        self.text(42,882,1840,['공통 채택: Request Controller의 현재 입력과 정보, 권한 검사 및 저장 성공 후 응답 또는 Task 위임'],17,INK)
        self.text(42,907,1840,['반복 종료: 의미 완성, 확인 질문 대기, 실패, 취소, 한도 도달 / 질문을 실제 전달하고 새 답변을 받은 뒤 해석 재개'],16,INK)
        for j,(quality,left,right) in enumerate(d['tradeoffs']):
            yy=943+j*26
            self.line([(40,yy-8),(1880,yy-8)],color=LINE,arrow=False)
            self.text(42,yy,245,[quality],17,INK,True)
            self.text(300,yy,720,[left],17,INK)
            self.text(1050,yy,820,[right],17,INK)
        self.text(40,1053,1840,['미선정 / 미측정 / A도 구조화 출력과 부분 수정 가능 / B도 조건부 추가 모델 호출 가능 / 코드 결합은 의미 정답 보증이 아님'],15,MUTED)

    def lifecycle_page(self):
        d=DATA[42]
        self.text(40,140,1840,[d['case']],22,INK)
        self.text(40,171,1840,['공통: 별도 Conversation / Request / Task / Execution 수명 / 로컬 VAD와 지속 입력 / 클라우드 모델 / 같은 의미 판단'],17,INK)
        self.rect(40,195,1840,48,GREEN,LINE)
        for side,x in enumerate([140,1010]):
            self.text(x+435,206,838,[d['options'][side].replace('·',' / ')],23,INK,True,'center')
        self.line([(1010,195),(1010,795)],color=LINE,arrow=False)
        self.line([(40,795),(1880,795)],color=LINE,arrow=False)
        self.text(42,811,1840,['공통 의미 제안은 Request Interpreter, 채택은 Request Controller / 실제 전달 원장은 Response Manager / 모델·외부 통신은 transaction 밖'.replace('·',' / ')],17,INK)
        self.text(42,840,1840,['B: hold 요청 시각과 업무 gate 적용 ACK 시각은 다름 / ACK 전 DISPATCHING은 UNKNOWN·외부 확인으로 처리'.replace('·',' / ')],17,INK)
        self.quality_footer(d)

    def execution_page(self):
        d=DATA[44]
        self.text(40,140,1840,[d['case']],22,INK)
        self.text(40,171,1840,['공통: 계속 입력 수신 / 로컬 VAD와 즉시 음성 중단 / 클라우드 음성 / 의미 모델 / 같은 Request / Task / 실제 전달 원본'],17,INK)
        self.rect(40,195,1840,48,GREEN,LINE)
        for side,x in enumerate([140,1010]):
            self.text(x+435,206,838,[d['options'][side]],23,INK,True,'center')
        self.line([(1010,195),(1010,795)],color=LINE,arrow=False)
        self.line([(40,795),(1880,795)],color=LINE,arrow=False)
        self.text(42,811,1840,['공통 후속 처리: source / 권한 / 입력 세대 검사 → Publication Control → Interaction Manager 실제 표시/재생 → receipt'],17,INK)
        self.text(42,840,1840,['확인 질문은 별도 admission으로 현재 입력 해소를 기다리지 않고 제시 / 발화 시작은 업무 취소 완료가 아님'],17,INK)
        self.quality_footer(d)

    def quality_footer(self,d):
        for j,(quality,left,right) in enumerate(d['tradeoffs']):
            yy=877+j*25
            self.line([(40,yy-6),(1880,yy-6)],color=LINE,arrow=False)
            self.text(42,yy,245,[quality],16,INK,True)
            self.text(300,yy,720,[left],16,INK)
            self.text(1050,yy,820,[right],16,INK)
        self.text(40,1030,1840,[d['condition'].replace('·',' / ')],15,INK)
        footer=('Reference: A LangChain Retrieval / B LangMem Background + Memory API / 本文'.replace('本文','본문 §9의 적용 범위와 한계') if self.number==45 else
                '준비 완료 / VIA 내부 접수 / 외부 Agent 접수 / 사용자 실제 전달은 별도 확인')
        self.text(40,1057,1840,[footer],14,MUTED)

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
        for item in self.items:
            for field in ('color','fill','stroke'):
                if field in item:
                    value=item[field]
                    assert isinstance(value,str) and re.fullmatch(r'#[0-9A-Fa-f]{6}|white|black|none',value),(self.slug,item['id'],field,value)
        original=self.items
        try:
            self.items=self.transformed();result=super().svg().replace('— DP 배경','— 설계 비교')
            marker=f'<marker id="a{PURPLE[1:]}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto"><path d="M1 1L9 5L1 9" fill="none" stroke="{PURPLE}" stroke-width="1.6"/></marker>'
            result=result.replace('</defs>',marker+'</defs>')
            if self.number in (41,42,44,45):
                result=result.replace('</defs>','<marker id="a000000" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M1 1L9 5L1 9" fill="none" stroke="#000000" stroke-width="1.2"/></marker></defs>')
            if self.number in (41,42,44,45):
                result=result.replace('</defs>','<marker id="aB8753F" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M1 1L9 5L1 9" fill="none" stroke="#B8753F" stroke-width="1.2"/></marker></defs>')
        finally: self.items=original
        for i in original:
            if i['kind'] in ('rect','note') and ('line_width' in i or i.get('rounded')):
                def restyle(match):
                    tag=match[0].replace('stroke-width="1.7"',f'stroke-width="{i.get("line_width",1.7)}"')
                    return tag.replace('/>',' rx="7"/>') if i.get('rounded') else tag
                result=re.sub(r'<(?:rect|path) id="'+i['id']+r'"[^>]*/>',restyle,result)
            if self.number in (41,42,44,45) and i['kind']=='rect' and (i.get('external') or i.get('flow_decision')):
                x,y,w,h=(i[k] for k in ('x','y','w','h'))
                points=(f'{x+w/2},{y} {x+w},{y+h/2} {x+w/2},{y+h} {x},{y+h/2}' if i.get('flow_decision') else f'{x+9},{y} {x+w-9},{y} {x+w},{y+h/2} {x+w-9},{y+h} {x+9},{y+h} {x},{y+h/2}')
                poly=f'<polygon id="{i["id"]}" points="{points}" fill="white" stroke="#000000" stroke-width="1"/>'
                result=re.sub(r'<rect id="'+i['id']+r'"[^>]*/>',lambda _:poly,result)
            if i['kind']!='store': continue
            x,y,w,h=(i[k] for k in ('x','y','w','h'));r=i.get('store_radius',8)
            path=f'<path id="{i["id"]}" fill="{i["fill"]}" stroke="{i["stroke"]}" stroke-width="{i.get("line_width",1.7)}" d="M{x} {y+r}C{x} {y-r} {x+w} {y-r} {x+w} {y+r}V{y+h-r}C{x+w} {y+h+r} {x} {y+h+r} {x} {y+h-r}Z M{x} {y+r}C{x} {y+3*r} {x+w} {y+3*r} {x+w} {y+r}"/>'
            result=re.sub(r'<rect id="'+i['id']+r'"[^>]*/>',lambda _:path,result)
        if self.number in (44,45):
            from continuous_interaction_scene import decorate_svg
            result=decorate_svg(self,result)
        return result

    def diagram(self):
        original=self.items
        try: self.items=self.transformed();d=super().diagram()
        finally: self.items=original
        stores={i['id'] for i in original if i['kind']=='store'}
        for cell in d.findall('.//mxCell'):
            if cell.get('id') in stores: cell.set('style',cell.get('style')+'shape=cylinder;size=8;')
            item=next((i for i in original if i['id']==cell.get('id')),None)
            if item and 'line_width' in item:
                cell.set('style',cell.get('style').replace('strokeWidth=1.7;',f'strokeWidth={item["line_width"]};'))
            if item and item.get('rounded'):
                cell.set('style',cell.get('style').replace('rounded=0;','rounded=1;arcSize=14;'))
            if self.number in (41,42,44,45) and item and (item.get('external') or item.get('flow_decision')):
                shape='rhombus' if item.get('flow_decision') else 'hexagon'
                cell.set('style',cell.get('style')+f'shape={shape};')
        if self.number in (44,45):
            from continuous_interaction_scene import decorate_diagram
            d=decorate_diagram(self,d)
        return d


from dp_comparison_structures import graph41, graph42, graph43, graph44, graph45


def build(dp41_only=False, only=None):
    selected=41 if dp41_only else only
    outputs={};deck=ET.Element('mxfile',host='app.diagrams.net',type='device')
    for n in ([selected] if selected else DATA):
        s=Comparison(n)
        for side,x in enumerate([140,1010]): globals()[f'graph{n}'](s,x,side)
        outputs[OUT/f'{s.slug}.svg']=s.svg()
        outputs[OUT/f'{s.slug}.drawio']=s.drawio()
        deck.append(copy.deepcopy(s.diagram()))
    deck_path=OUT/'VIA-DP-comparison-41-45.drawio'
    data_path=OUT/'comparison-data.json'
    if selected:
        old=deck_path.read_text()
        page=ET.tostring(deck[0],encoding='unicode')
        outputs[deck_path]=re.sub(rf'<diagram id="dp{selected}-comparison".*?</diagram>',lambda _:page,old,flags=re.S)
        data=json.loads(data_path.read_text());data['slides'][str(selected)]=DATA[selected]
    else:
        outputs[deck_path]=ET.tostring(deck,encoding='unicode')+'\n'
        data=dict(status='FORMAT_HYPOTHESES_NOT_MEASURED',qa=QA,slides=DATA)
    outputs[data_path]=json.dumps(data,ensure_ascii=False,indent=2)+'\n'
    blocks=''.join(f'<article><h2>04-{n} 설계'+f' 비교</h2><p><a href="dp{n}-comparison.drawio">draw.io</a> / <a href="dp{n}-comparison.svg">SVG</a> / <a href="dp{n}-comparison.png">PNG</a></p><img src="dp{n}-comparison.svg" alt="04-{n} 설계 비교"></article>' for n in DATA)
    outputs[OUT/'index.html']='<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA DP 설계 비교</title><style>body{margin:30px auto;max-width:1440px;font-family:Arial,sans-serif;background:#f3f5f7;color:#18232e}article{margin:30px 0}img{width:100%;background:white}a{color:#087e8b}</style><h1>VIA DP 설계 비교 41~45</h1><p>41~44의 수치와 동그라미 점수는 형식 검토용 예상 예시. 45는 조건별 정성 비교. 실측 결과 또는 대안 선정 아님.</p><p>45: 같은 평가 기준의 실제 전달과 제품 비교/견적 결과 참조에서 요청별 owner 읽기 조합과 공통 관계 생산/검증/게시/조회를 비교한다. 첫 요청, 반복, 정정 직후, 표현 밖 관계의 조건별 손익과 공식 Reference를 함께 제시한다. 새 사례의 실측이나 선정 결과가 아니다.</p><p><a href="VIA-DP-comparison-41-45.drawio">5페이지 편집 원본</a></p>'+blocks+'</html>\n'
    numeric=', '.join(str(n) for n,d in DATA.items() if 'rows' in d)
    qualitative=', '.join(str(n) for n,d in DATA.items() if 'tradeoffs' in d)
    outputs[OUT/'index.html']=outputs[OUT/'index.html'].replace('41~44의 수치와 동그라미 점수는 형식 검토용 예상 예시. 45는 조건별 정성 비교.',f'{numeric}의 수치와 점수는 형식 검토용 예상 예시. {qualitative}는 조건별 정성 비교.')
    return outputs


def validate(outputs):
    """Check comparisons, rather than assuming the circle counts tell the truth."""
    presentation=json.loads((ROOT/'docs/presentations_files/quality-attributes/quality-attributes.json').read_text())
    names={r['id']:r['name'] for r in presentation['rows']}
    if 'V-01' in names:
        for qa,(name,_,_) in QA.items():
            if names[qa]!=name: raise SystemExit(f'Presentation QA name changed: {qa}')
        targets={r['id']:r['target'] for r in presentation['rows']}
        for qa in ('V-04','V-05'):
            if not targets[qa].startswith(QA[qa][1]):
                raise SystemExit(f'Presentation time metric changed: {qa}')
    else:
        # The unmodified 43 appendix retains its historical V labels. Active
        # qualitative rows use the six current ASRs, never a fabricated score.
        assert set(names)=={f'ASR-QA-{i:02d}' for i in range(1,7)}, names
        assert [r['priority'] for r in presentation['rows']]==list(range(1,7))
    for n,data in DATA.items():
        if n in (41,42,44,45):
            assert len(data['tradeoffs'])==({41:4,42:6,44:6,45:6}[n]) and 'rows' not in data
            continue
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
            if '·' in source.replace('업무 접수·전송 게이트','업무 접수 및 전송 게이트'): raise SystemExit(f'Unexpected middle dot: {p}')
        elif p.suffix=='.drawio':
            tree=ET.fromstring(source)
            for page in tree.findall('diagram'):
                m=page.find('mxGraphModel');cells=page.findall('.//mxCell')
                ids=[c.get('id') for c in cells]
                if len(ids)!=len(set(ids)) or m.get('pageWidth')!='1920' or m.get('pageHeight')!='1080':
                    raise SystemExit(f'Invalid native page: {p}')


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');parser.add_argument('--dp41-only',action='store_true');parser.add_argument('--only',type=int,choices=[41,42,43,44,45]);args=parser.parse_args()
    outputs=build(args.dp41_only,args.only);validate(outputs)
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
