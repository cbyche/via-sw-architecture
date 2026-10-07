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
        self.text(40,133 if n==44 else 142,1530,[DATA[n]['case']],23,INK)
        self.text(1880,145,290,['수치와 점수 / 예상 예시'],19,RED,True,align='right')
        if n==44:
            self.text(40,168,1530,['양안 공통: Agent Gateway ↔ Downstream Agent / Model Access ↔ 공유 Omni 1벌'],14,MUTED)
        self.rect(40,188,1840,55,GREEN,LINE)
        for y,h,label in [(188,55,'설계안'),(243,552,'구조'),(795,80,'장점'),(875,80,'단점'),(955,95,'QA\nTrade-off')]:
            if y!=188: self.rect(40,y,1840,h,'white',LINE)
            self.text(90,y+17,98,label.split('\n'),21,INK,True,'center',leading=26)
        for x in [140,1010]: self.line([(x,188),(x,1050)],color=LINE,arrow=False)
        for side,x in enumerate([140,1010]):
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


from dp_comparison_structures import graph41, graph42, graph43, graph44, graph45


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
