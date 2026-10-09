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
  41: dict(title='VIA 요청 이해의 모델 주도와 코드 주도 완성',
    options=('A 모델 주도 해석', 'B 모델 요청 틀과 코드 주도 완성'),
    case='“이 표를 아까 보고서에 넣고, 메일은 초안만 만들어.” / 보고서 Task 후보 2개',
    status='CONDITIONAL_QUALITATIVE_NOT_MEASURED',
    condition='미선정 / 미측정. B도 부분 의미 판단을 다시 호출할 수 있다. 새 관계의 지원 범위와 질문 비용을 함께 비교한다.',
    tradeoffs=[
      ('정확성 / 지원 범위','열린 의미를 함께 판단 / 검색 누락과 모델 오판 가능','지원 관계의 코드 결합 / 틀 오류와 표현 밖 관계의 한계'),
      ('응답성','조회 후 재판단마다 클라우드 왕복 / 부분 수정 허용','지원 틀이 충분하면 재판단 축소 / 추가 해석과 질문 대기'),
      ('모델 호출 비용','반복 해석과 긴 Context가 비용 증가 요인','초기 틀 + 필요한 부분 해석 / 항상 1회라는 전제 없음'),
      ('변경 용이성','새 관계는 prompt/tool 변경 가능 / 채택 계약 변경은 별도','schema와 Engine 규칙 확장 / 기존 지원 관계는 코드 제어')]),
  42: dict(title='변경 용이성을 위한 VIA Conversation과 Task 관리 설계',
    options=('A 모듈형 통합 VIA Core', 'B 독립 대화 서비스와 업무 서비스'),
    case='보고서 질문 Q1을 실제 제시(P1) → “상반기로” 답변(u2) → 접수와 실제 전달 확인',
    condition='비교 예시 조건: 업무 저장과 실행 수명의 독립 변경 / 일반 Agent adapter 변경은 제외',
    rows=[('V-05','3.2 s','3.8 s',3,2),('V-08','4.0개','2.5개',2,3),('V-06','13.2 GB','13.6 GB',3,2)],
    pros=[['관련 상태의 공동 확정으로 평균 VIA 처리시간 3.2 s (요청 완료 신속성 ↑)',
           '추가 서비스 heap 축소로 전체 메모리 13.2 GB (메모리 효율성 ↑)'],
          ['업무 상태와 실행 경계의 독립 변경, 평균 2.5개 (변경 용이성 ↑)']],
    cons=[['공동 저장과 복구 경로의 변경 전파, 평균 4.0개 (변경 용이성 ↓)'],
          ['명령 접수와 대화 반영의 왕복으로 평균 VIA 처리시간 3.8 s (요청 완료 신속성 ↓)',
           '별도 heap과 대기열로 전체 메모리 13.6 GB (메모리 효율성 ↓)']]),
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
  44: dict(title='반응성을 위한 VIA 지속 입력과 Response 전달 설계',
    options=('A 중앙 조정 방식', 'B 이벤트 흐름을 연결하는 방식'),
    case='보고서 설명 도중 새 발화와 메일 질문 도착 / 표 설명 후 메일 질문 전달',
    condition='비교 예시 조건: 여러 사건의 집중 도착 / 양안의 입력 수신과 로컬 음성 중단은 동일',
    rows=[('V-04','0.85 s','0.62 s',2,3),('V-08','4.0개','2.5개',2,3),('V-06','13.3 GB','13.8 GB',3,2)],
    pros=[['집중된 실행 상태로 전체 메모리 13.3 GB (메모리 효율성 ↑)'],
          ['사건의 직접 활성화로 평균 반응시간 0.62 s (응답 신속성 ↑)',
           '동일 schema 안의 단계 조합 변경, 평균 2.5개 (변경 용이성 ↑)']],
    cons=[['반환 회수와 재배정으로 평균 반응시간 0.85 s (응답 신속성 ↓)',
           '교차 사건의 중앙 전이 변경, 평균 4.0개 (변경 용이성 ↓)'],
          ['다수 window와 buffer로 전체 메모리 13.8 GB (메모리 효율성 ↓)']]),
  45: dict(title='VIA 과거 근거의 요청별 구성과 공통 생산/조회',
    options=('A 원본 서비스 조합형', 'B 공통 파생 기억 저장소형'),
    case='R23 “지난번 네가 설명한 평가 기준에 맞춰, 앞서 조사한 제품 비교 결과와 받아둔 견적을 이번 제안서에 반영해줘.”',
    status='CONDITIONAL_QUALITATIVE_NOT_MEASURED',
    condition='미선정 / 미측정. A cache의 같은 효과는 B 고유 이익을 줄인다. B+ 원본 조합 우회는 명시적 혼합안.',
    tradeoffs=[
      ('새 관계 / 미묘한 조건', '요청 목적에 맞춘 원문 구성의 유연성; 검색 누락 가능', '지원 표현 / 추출 조건에 제한; 정확성의 고정 우열 없음'),
      ('준비된 반복 조회 R24', 'warm 관계 cache로 조회 / 재해석 축소 가능', '유효 게시가 원본 교차 결합을 대체하면 경로 단축 가능'),
      ('첫 조회 / 정정 직후', '현재 원본으로 구성; fan-out / 해석 비용', '생산 / 갱신 게시 대기; NOT_COVERED와 미지원 구별'),
      ('전체 모델 호출 / 자원', 'cache 생산 / 유지 + 요청 구성 / 재검증 포함', '초기 / 갱신 / backfill / 미사용 생산 포함; 고정 우열 없음'),
      ('변경 용이성', 'Source adapter로 국소화; cache schema 변경은 무효화', 'Source 형식은 adapter; 관계 의미는 schema / 재생산 / 독자 변경')]),
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
        self.text(40,135,1840,[d['case']],20,INK)
        self.text(40,168,1840,['같은 원본 / 현재 T23 연결은 판단 결과 / VIA: 허용 근거 연결, Agent: 기준 적용과 제안서 작성'],17,INK)
        for side,x in enumerate([140,1010]):
            self.text(x+18,192,820,[d['options'][side]],25,INK,True)
            self.text(x+18,226,820,['현재 요청의 owner 읽기 결과 조합 / 누락 해결' if side==0 else '과거 관계 생산・검증・게시 / 공통 읽기 계약 소비'.replace('・',' / ')],17,INK)
        self.line([(990,195),(990,795)],color=LINE,arrow=False)
        self.text(40,786,1840,['장면: 첫 요청 → 필요한 원본 / 미게시 생산 | 반복 R24 → warm cache / 유효 게시 | 정정 직후 → fence 후 갱신 | 표현 밖 새 관계 → 미지원'],16,INK)
        from memory_context_scene import page_legend
        page_legend(self)
        for j,(condition,left,right) in enumerate(d['tradeoffs']):
            yy=871+j*29
            self.line([(40,yy-5),(1880,yy-5)],color=LINE,arrow=False)
            self.text(42,yy,266,[condition],16,INK,True)
            self.text(315,yy,745,[left],16,INK)
            self.text(1080,yy,795,[right],16,INK)
        self.text(40,1020,1840,[d['condition']],15,INK)
        self.text(40,1046,1840,['Reference (§9): A LangChain Retrieval / B LangMem Background + Memory API / 실행 Delayed Processing / 연결 참고 Mem0 Graph (typed 관계 증거 아님)'],15,INK)
        self.text(40,1067,1840,['공식 메커니즘 선례 / VIA 구현・성능 증거 아님 / 실행・서비스 경계 추가 없음'.replace('・',' / ')],10,MUTED)

    def request_page(self):
        d=DATA[41]
        self.text(40,140,1840,[d['case']],23,INK)
        self.text(40,169,1840,['공통: 로컬 VIA / 로컬 VAD / 클라우드 음성 + 의미 LLM / 채택은 코드 / 조회 예시: E7@v2 표, T7/T8 후보, M3 발송 금지'],17,INK)
        self.rect(40,195,1840,48,GREEN,LINE)
        self.text(90,209,98,['설계안'],19,INK,True,'center')
        for side,x in enumerate([140,1010]):
            self.text(x+435,205,838,[d['options'][side]],26,INK,True,'center')
        self.line([(1010,195),(1010,795)],color=LINE,arrow=False)
        self.line([(40,795),(1880,795)],color=LINE,arrow=False)
        self.text(42,810,1840,['비교 대상: 다음 읽기와 전체 의미 결합의 결정권 / 임시 해석 상태는 Request Interpreter / 채택 의미/질문/출처/revision은 Request Controller'],17,INK)
        self.text(42,836,1840,['음성/전사 비용은 공통. 재시도/폐기된 모델 호출도 집계. 도구 조회 자체는 모델 호출이 아니며 45의 의미 가공이 추가되면 별도 집계.'],17,INK)
        for j,(quality,left,right) in enumerate(d['tradeoffs']):
            yy=883+j*34
            self.line([(40,yy-8),(1880,yy-8)],color=LINE,arrow=False)
            self.text(42,yy,245,[quality],18,INK,True)
            self.text(300,yy,720,[left],18,INK)
            self.text(1050,yy,820,[right],18,INK)
        self.text(40,1032,1840,[d['condition']],17,INK)
        self.text(40,1060,1840,['교환 자료/필드는 검토용 예시 / A도 구조화 출력 가능 / B의 BOUND는 정답 보증이 아님 / 품질 우열과 호출 비용은 측정 전 가설'],14,MUTED)

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
            if self.number in (41,42,44,45):
                result=result.replace('</defs>','<marker id="a000000" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M1 1L9 5L1 9" fill="none" stroke="#000000" stroke-width="1.2"/></marker></defs>')
        finally: self.items=original
        for i in original:
            if i['kind'] in ('rect','note') and ('line_width' in i or i.get('rounded')):
                def restyle(match):
                    tag=match[0].replace('stroke-width="1.7"',f'stroke-width="{i.get("line_width",1.7)}"')
                    return tag.replace('/>',' rx="7"/>') if i.get('rounded') else tag
                result=re.sub(r'<(?:rect|path) id="'+i['id']+r'"[^>]*/>',restyle,result)
            if self.number==41 and i['kind']=='rect' and i.get('external'):
                x,y,w,h=(i[k] for k in ('x','y','w','h'))
                poly=f'<polygon id="{i["id"]}" points="{x+9},{y} {x+w-9},{y} {x+w},{y+h/2} {x+w-9},{y+h} {x+9},{y+h} {x},{y+h/2}" fill="white" stroke="#000000" stroke-width="1"/>'
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
            if self.number==41 and item and item.get('external'):
                cell.set('style',cell.get('style')+'shape=hexagon;')
        if self.number in (44,45):
            from continuous_interaction_scene import decorate_diagram
            d=decorate_diagram(self,d)
        return d


from dp_comparison_structures import graph41, graph42, graph43, graph44, graph45


def build(dp41_only=False):
    outputs={};deck=ET.Element('mxfile',host='app.diagrams.net',type='device')
    for n in ([41] if dp41_only else DATA):
        s=Comparison(n)
        for side,x in enumerate([140,1010]): globals()[f'graph{n}'](s,x,side)
        outputs[OUT/f'{s.slug}.svg']=s.svg()
        outputs[OUT/f'{s.slug}.drawio']=s.drawio()
        deck.append(copy.deepcopy(s.diagram()))
    deck_path=OUT/'VIA-DP-comparison-41-45.drawio'
    data_path=OUT/'comparison-data.json'
    if dp41_only:
        old=deck_path.read_text()
        page=ET.tostring(deck[0],encoding='unicode')
        outputs[deck_path]=re.sub(r'<diagram id="dp41-comparison".*?</diagram>',lambda _:page,old,flags=re.S)
        data=json.loads(data_path.read_text());data['slides']['41']=DATA[41]
    else:
        outputs[deck_path]=ET.tostring(deck,encoding='unicode')+'\n'
        data=dict(status='FORMAT_HYPOTHESES_NOT_MEASURED',qa=QA,slides=DATA)
    outputs[data_path]=json.dumps(data,ensure_ascii=False,indent=2)+'\n'
    blocks=''.join(f'<article><h2>04-{n} 설계'+f' 비교</h2><p><a href="dp{n}-comparison.drawio">draw.io</a> / <a href="dp{n}-comparison.svg">SVG</a> / <a href="dp{n}-comparison.png">PNG</a></p><img src="dp{n}-comparison.svg" alt="04-{n} 설계 비교"></article>' for n in DATA)
    outputs[OUT/'index.html']='<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA DP 설계 비교</title><style>body{margin:30px auto;max-width:1440px;font-family:Arial,sans-serif;background:#f3f5f7;color:#18232e}article{margin:30px 0}img{width:100%;background:white}a{color:#087e8b}</style><h1>VIA DP 설계 비교 41~45</h1><p>41~44의 수치와 동그라미 점수는 형식 검토용 예상 예시. 45는 조건별 정성 비교. 실측 결과 또는 대안 선정 아님.</p><p>45: 같은 평가 기준의 실제 전달과 제품 비교/견적 결과 참조에서 요청별 owner 읽기 조합과 공통 관계 생산/검증/게시/조회를 비교한다. 첫 요청, 반복, 정정 직후, 표현 밖 관계의 조건별 손익과 공식 Reference를 함께 제시한다. 새 사례의 실측이나 선정 결과가 아니다.</p><p><a href="VIA-DP-comparison-41-45.drawio">5페이지 편집 원본</a></p>'+blocks+'</html>\n'
    if dp41_only:
        outputs[OUT/'index.html']=(OUT/'index.html').read_text().replace('41~44의 수치와 동그라미 점수는 형식 검토용 예상 예시. 45는 조건별 정성 비교.','42~44의 수치와 동그라미 점수는 형식 검토용 예상 예시. 41과 45는 조건별 정성 비교.')
    else:
        outputs[OUT/'index.html']=outputs[OUT/'index.html'].replace('41~44의 수치와 동그라미 점수는 형식 검토용 예상 예시. 45는 조건별 정성 비교.','42~44의 수치와 동그라미 점수는 형식 검토용 예상 예시. 41과 45는 조건별 정성 비교.')
    return outputs


def validate(outputs):
    """Check comparisons, rather than assuming the circle counts tell the truth."""
    presentation=json.loads((ROOT/'docs/presentations_files/quality-attributes/quality-attributes.json').read_text())
    names={r['id']:r['name'] for r in presentation['rows']}
    for qa,(name,_,_) in QA.items():
        if names[qa]!=name: raise SystemExit(f'Presentation QA name changed: {qa}')
    targets={r['id']:r['target'] for r in presentation['rows']}
    for qa in ('V-04','V-05'):
        if not targets[qa].startswith(QA[qa][1]):
            raise SystemExit(f'Presentation time metric changed: {qa}')
    for n,data in DATA.items():
        if n in (41,45):
            assert len(data['tradeoffs'])==(4 if n==41 else 5) and 'rows' not in data
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
            if '·' in source: raise SystemExit(f'Unexpected middle dot: {p}')
        elif p.suffix=='.drawio':
            tree=ET.fromstring(source)
            for page in tree.findall('diagram'):
                m=page.find('mxGraphModel');cells=page.findall('.//mxCell')
                ids=[c.get('id') for c in cells]
                if len(ids)!=len(set(ids)) or m.get('pageWidth')!='1920' or m.get('pageHeight')!='1080':
                    raise SystemExit(f'Invalid native page: {p}')


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');parser.add_argument('--dp41-only',action='store_true');args=parser.parse_args()
    outputs=build(args.dp41_only);validate(outputs)
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
