#!/usr/bin/env python3
"""04-42 presentation plates. One scene produces editable draw.io and SVG.

This is documentation generation, not an implementation of either candidate.
Existing comparison generators and the reference architecture are not changed.
"""
import argparse
from pathlib import Path
import xml.etree.ElementTree as ET

from stage4_diagram_design import Plate, INK, MUTED, BLUE, GREEN
from lifecycle_ownership_presentation import structure
from dp_comparison_structures import COMMON, APRICOT, DIFFERENCE_STROKE, DIFFERENCE_WIDTH, BOUNDARY_WIDTH, append_plate_legend

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/diagrams'
PALE = '#F0F5FD'
MINT = '#F0F8F4'
LINE = '#D5DEE8'


class Slide(Plate):
    def __init__(self, slug, title, subtitle):
        super().__init__(slug, '04-42', title, subtitle, height=1440)
        self.items = []
        self.serial = 0
        self.box(0, 0, 2560, 10, INK, 'none', 0)
        self.text(64, 32, 'VIA / DECISION POINT CANDIDATE', 23, MUTED, bold=True)
        self.text(2496, 32, '42  /  설계 비교', 23, MUTED, 'right')
        self.text(64, 83, title, 49, bold=True)
        self.text(64, 154, subtitle, 28, MUTED)
        self.line([(64, 211), (2496, 211)], LINE, arrow=False)

    @staticmethod
    def text_geometry(i):
        size = 30 if i['w'] >= 290 else 25
        leading = size * 1.28
        return i['y'] + (i['h'] - len(i['name']) * leading) / 2 - 2 + (8 if i['type'] == 'store' else 0), size, leading

    def svg(self):
        # Component headings are intentionally larger than the older tall plates.
        svg = super().svg().replace('font-size="23"', 'font-size="28"')
        svg=svg.replace('#263445',COMMON).replace('a263445','a000000').replace('#F7F9FC','white')
        svg=svg.replace('stroke-width="2.2"','stroke-width="1.3"')
        return svg

    def drawio(self):
        xml = super().drawio().replace('fontSize=23;', 'fontSize=28;').replace('#263445',COMMON).replace('#F7F9FC','white')
        xml=xml.replace('strokeWidth=2.2;','strokeWidth=1.3;')
        # Plate.box does not serialize its explicit line width by default.
        root=ET.fromstring(xml)
        boxes={i['id']:i for i in self.items if i['kind']=='box'}
        for cell in root.findall('.//mxCell'):
            if cell.get('id') in boxes:
                item=boxes[cell.get('id')];cell.set('style',cell.get('style')+f'strokeWidth={item["thick"]};'+('rounded=0;' if item['radius']==0 else ''))
        xml=ET.tostring(root,encoding='unicode')+'\n'
        return xml

    def footer(self, note='설계 가설 / 구현과 측정 결과 아님'):
        if self.slug=='choice42-lifetimes':
            self.line([(64,1342),(2496,1342)],LINE,arrow=False,width=1.3)
            self.text(64,1360,'막대: 데이터의 존속 / 세로 점선: 사건 시점 / Component나 process 경계가 아님',23,MUTED)
            self.text(2496,1399,note,21,MUTED,'right')
            return
        self.line([(64,1342),(2496,1342)],LINE,arrow=False,width=1.3)
        self.text(2496,1346,note,17,COMMON,'right')
        append_plate_legend(self)

    def arrow(self, a, b, label, at, sp='B', tp='T', via=(), color=INK, ret=False, sd=0, td=0, size=25):
        self.edge(a,b,sp=sp,tp=tp,via=list(via),color=INK,ret=ret,sd=sd,td=td)
        self.items[-1]['width']=1.3
        if label:
            self.label(*at, label, color, size=size)



def lifetimes():
    p = Slide('choice42-lifetimes', '같은 사용자 사례, 서로 다른 수명',
              'A와 B 모두 지원해야 하는 기능이다.  기록의 수명과 owner process의 수명은 별개다.')
    xs = [535, 865, 1195, 1525, 1855, 2185]
    events = [('E1', '보고서 요청', 'Agent 접수'), ('E2', '기간 질문', 'C1에 제시'),
              ('E3', '다른 대화 C2', '업무는 유지'), ('E4', 'C1에서 답변', '“상반기로”'),
              ('E5', '보고서 완료', 'X1 종료'), ('E6', '결론 수정', 'X2 시작')]
    for x, (k, title, sub) in zip(xs, events):
        p.text(x, 257, k, 25, MUTED, 'center', True)
        p.text(x, 300, title, 30, INK, 'center', True)
        p.text(x, 344, sub, 25, MUTED, 'center')
        p.line([(x, 405), (x, 1175)], LINE, dashed=True, arrow=False)
    rows = [('Conversation', '대화·참조'), ('Request', '요청 처리'), ('Question', '질문'),
            ('Task', '사용자 업무'), ('Execution', '외부 실행')]
    ys = [465, 615, 765, 915, 1065]
    for (a, b), y in zip(rows, ys):
        p.text(72, y+3, a, 31, bold=True)
        p.text(72, y+48, b, 25, MUTED)
    def bar(x1, x2, y, text, c=INK, fill='#F3F5F8'):
        p.box(x1, y, x2-x1, 58, fill, c, 12, thick=2)
        p.text((x1+x2)/2, y+10, text, 26, c, 'center', True)
    bar(480, 2420, 465, 'C1  기록·참조는 유지 / 화면에서 다른 대화로 이동해도 삭제·취소되지 않음')
    bar(1130, 1420, 539, 'C2')
    bar(480, 690, 635, 'R1 위임 접수')
    bar(1450, 1670, 635, '답변 요청')
    bar(2110, 2420, 635, '후속 수정 요청')
    bar(820, 1620, 785, 'Q1  Agent 질문 → 실제 제시 P1 → 유효한 답변')
    bar(480, 2420, 935, 'T1  같은 업무 identity / 결과 보존 → 수정 시 revision 증가')
    bar(480, 1920, 1085, 'X1  외부 실행 → 완료 / 완료 이력 유지')
    bar(2110, 2420, 1085, 'X2  새 실행')
    p.text(72, 1235, '대화 종료 ≠ 요청 완료 ≠ Task 종료 ≠ 외부 실행 종료 ≠ 기록 삭제', 35, bold=True)
    p.footer('시간축은 사건 순서만 표시하며 실제 소요시간·저장 기간을 뜻하지 않음')
    return p


def sequence(option):
    b=option=='B';c=DIFFERENCE_STROKE
    p=Slide('choice42-event-'+option.lower(),option+' 같은 답변의 확정과 실제 전달',
            'E4: u2 “상반기로” → R2/Q1/T1 연결 → K1 접수 → P2 실제 제시 / P1은 이미 제시된 질문 기록')
    xs=[170,530,900,1260,1620,1980,2370]
    names=['Interaction Manager','Request Controller','Response Manager','트랜잭션 관리자','Task Manager','Agent Gateway','Downstream Agent']
    if b:
        p.box(354,249,734,1025,'white',DIFFERENCE_STROKE,0,dashed=True,thick=BOUNDARY_WIDTH*4/3)
        p.box(1444,249,720,1025,'white',DIFFERENCE_STROKE,0,dashed=True,thick=BOUNDARY_WIDTH*4/3)
        p.text(374,264,'대화 서비스',24,DIFFERENCE_STROKE,bold=True)
        p.text(1464,264,'업무 서비스',24,DIFFERENCE_STROKE,bold=True)
    else:
        p.box(354,249,1810,1025,'white',DIFFERENCE_STROKE,0,thick=BOUNDARY_WIDTH*4/3)
        p.text(374,264,'VIA Core',24,DIFFERENCE_STROKE,bold=True)
    for j,(x,name) in enumerate(zip(xs,names)):
        if b and j==3:continue
        p.box(x-150,313,300,62,APRICOT if j==3 else 'white',DIFFERENCE_STROKE if j==3 else COMMON,7 if j==3 else 0,thick=DIFFERENCE_WIDTH*4/3 if j==3 else 1.3)
        p.text(x,331,name,21,INK,'center',True)
        p.line([(x,379),(x,1260)],'#A5B3C3',dashed=True,arrow=False,width=1)
    def msg(a,z,y,text,ret=False,color=INK):
        points=[(xs[a],y),(xs[z],y)] if a!=z else [(xs[a],y),(xs[a]+42,y),(xs[a]+42,y+13),(xs[a],y+13)]
        p.line(points,COMMON,width=1.3,dashed=ret)
        p.label(min(xs[a],xs[z])+14,y-28,text,color,size=21)
    steps=[(0,1,'u2 원문 입력 / Conversation C1',False),
           (1,4,'Q1/T1/X1 현재 상태 조회 / P1은 이미 제시된 기록',False),
           (4,1,'Q1=OPEN / expected revision v7',True),
           (1,1,'Request Interpreter의 의미 제안 채택 / u2는 Q1 답변',False)]
    if b:
        steps += [(1,1,'대화 저장: R2=접수 대기 / K1 / u2→Q1→T1',False),
                  (1,4,'K1: Q1 답변=상반기 / v7 / 권한·입력 조건',False),
                  (4,4,'업무 로컬 저장: Q1=ANSWERED,v8 / K1 / 내부 접수 결과',False),
                  (4,1,'K1 내부 접수 확인 / 외부 접수는 아직 별개',True),
                  (1,1,'대화 저장: R2에 K1 내부 접수 결과 반영',False)]
    else:
        steps += [(1,3,'R2/u2→Q1→T1 연결 / P1 참조의 변경 제안',False),
                  (3,4,'업무 현재 검사 / 변경 집합 요청',False),
                  (4,3,'검증한 Q1 답변·revision과 K1 전송 준비',True),
                  (3,3,'State Store: 관련 연결·Q1 변경·K1을 하나의 transaction으로 저장',False),
                  (3,1,'로컬 저장 결과 / 성공이면 관련 변경 함께 반영',True),
                  (3,4,'같은 로컬 저장 결과 / 실패하면 모두 미반영',True)]
    steps += [(4,5,'저장된 명령 K1 / 현재 전송 조건',False),
              (5,6,'K1 외부 답변 전달',False),
              (6,5,'K1 외부 접수 확인',True),
              (5,4,'source-confirmed K1 접수 사실',True),
              (4,1,'확인된 외부 접수 상태',True),
              (1,2,'P2: “상반기 조건 전달 확인” / 게시 허용과 source 참조',False),
              (2,0,'release(P2,output epoch) / Text·Voice',False),
              (0,2,'receipt(P2,실제 표시·재생 범위)',True),
              (2,2,'P2 publication/실제 전달 원장 저장',False),
              (2,1,'P2 실제 제시 사실 / Conversation 참조 갱신',True)]
    for j,(a,z,text,ret) in enumerate(steps):
        msg(a,z,425+j*820/(len(steps)-1),text,ret,c if 4<=j<(9 if b else 10) else INK)
    p.text(64,1290,'실제 제시 기록 원본: Response Manager / 답변·질문 연결: Request Controller / 모델·정책 호출은 공통 축약',23,MUTED)
    p.footer('입력 수신/재생과 요청 의미·업무 확정 구별 / 같은 해석·모델 조건 / 선 간격은 소요시간이 아님')
    return p


def change():
    p = Slide('choice42-change', '모듈화의 이익과 독립 운영의 이익을 구별한다',
              '같은 Agent 변화: 상태 조회를 polling에서 event stream으로 바꾸되 VIA의 사용자 의미는 유지')
    for opt, dx, c in [('A',0,DIFFERENCE_STROKE),('B',1250,DIFFERENCE_STROKE)]:
        p.text(80+dx, 253, opt+'  '+('통합 Core' if opt=='A' else '독립 서비스'), 39, c, bold=True)
        if opt=='A':
            p.box(90,365,1090,455,'white',c,0,thick=BOUNDARY_WIDTH*4/3)
            p.text(120, 385, 'VIA Core process', 28, c, bold=True)
        else:
            p.box(1340,365,420,455,'white',c,0,dashed=True,thick=BOUNDARY_WIDTH*4/3)
            p.box(1880,365,550,455,'white',c,0,dashed=True,thick=BOUNDARY_WIDTH*4/3)
            p.text(1370, 385, '대화 서비스 process', 28, c, bold=True)
            p.text(1910, 385, '업무 서비스 process', 28, c, bold=True)
        p.component(opt+'d', 130+dx, 493, 335, 150, 'Request Controller')
        p.component(opt+'w', 680+dx, 493, 400, 80, 'Task Manager')
        p.component(opt+'ad', 715+dx, 605, 330, 80, 'Agent Gateway')
        p.arrow(opt+'d', opt+'w', '같은 VIA 업무 계약', (480+dx, 522), sp='R', tp='L', td=35, size=25)
        p.text(130+dx, 747, '대화 코드 유지 가능', 29, bold=True)
        p.text(680+dx, 747, '연동·수신 경로 변경', 29, c, bold=True)
        p.text(100+dx, 880, '일반 protocol 변경은 양안 모두 국소화 가능', 29, INK, bold=True)
        p.text(100+dx, 950, '차이가 남는 조건', 28, c, bold=True)
        lines = (['Core process 재시작이 필요한 갱신이면', '대화 owner와 업무 owner가 함께 멈춤.', 'worker·동적 교체로 줄이는 혼합안도 검토.'] if opt=='A' else
                 ['공개 계약이 호환되면 업무 서비스만 갱신.', '대화 서비스는 유지하며 업무 요청은 보류 가능.', '새 승인 의미 등 계약 변화는 양쪽으로 전파.'])
        p.text(100+dx, 1000, lines, 29, MUTED, leading=48)
    p.text(80, 1234, 'V-08  변경 범위 감소는 조건부  /  독립 갱신의 이익을 변경 요소 수 우위로 바꾸어 말하지 않는다.', 30, bold=True)
    p.footer('양안 공통 Agent Gateway / 배치 차이는 바깥 경계 / 미선정과 미측정')
    return p


def gallery(plates):
    names = ['메인 구조 비교', '같은 사례의 서로 다른 수명', 'A 사건 흐름', 'B 사건 흐름', '변경 범위와 독립 운영']
    links = ''.join(f'<a href="#{p.slug}">{name}</a>' for p, name in zip(plates, names))
    sections = ''.join(f'<section id="{p.slug}"><h2>{name}</h2><img src="{p.slug}.svg" alt="{name}"><p><a href="{p.slug}.svg">SVG 원본</a> · <a href="{p.slug}.drawio">편집 가능한 draw.io</a></p></section>' for p, name in zip(plates, names))
    return f'''<!doctype html>
<html lang="ko"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>VIA 42 — 대화와 업무의 수명</title>
<style>body{{margin:0;background:#e9eef3;color:#263445;font-family:Arial,'Apple SD Gothic Neo',sans-serif}}header{{padding:24px 4vw;background:white}}h1{{margin:0 0 10px}}nav{{display:flex;gap:24px;flex-wrap:wrap}}a{{color:#195fbd}}main{{max-width:1800px;margin:auto}}section{{padding:24px 2vw 36px;scroll-margin-top:16px}}h2{{font-size:22px}}img{{display:block;width:100%;height:auto;box-shadow:0 6px 28px #26344522}}p{{line-height:1.65}}@media print{{nav,section p{{display:none}}header{{display:none}}section{{break-after:page;padding:0}}h2{{display:none}}img{{box-shadow:none}}}}</style>
<header><h1>42 — 서로 다른 대화와 업무 수명</h1><p>A/B 미선정 · 설계 가설 · 구현/측정 없음. 모든 그림은 16:9, 같은 좌표에서 생성한 SVG/draw.io 쌍.</p><nav>{links}</nav><p><a href="../04-42-lifecycle-ownership.md">설계 문서</a></p></header><main>{sections}</main></html>\n'''


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--check', action='store_true')
    args = ap.parse_args()
    plates = [structure(), lifetimes(), sequence('A'), sequence('B'), change()]
    generated = {}
    for p in plates:
        p.validate()
        for n in p.items:
            if n['kind'] == 'component':
                assert not n['role'], 'Names only in component rectangles'
        for ext, value in [('svg', p.svg()), ('drawio', p.drawio())]:
            root = ET.fromstring(value)
            if ext == 'drawio':
                cells = root.findall('.//mxCell')
                ids = [cell.get('id') for cell in cells]
                assert len(ids) == len(set(ids)), 'Duplicate cell ID'
                for cell in cells:
                    for attr in ('parent', 'source', 'target'):
                        assert not cell.get(attr) or cell.get(attr) in ids
            generated[p.slug+'.'+ext] = value
    generated['choice42-review.html'] = gallery(plates)
    drift = []
    for filename, value in generated.items():
        path = OUT / filename
        if args.check:
            if not path.exists() or path.read_text(encoding='utf-8') != value:
                drift.append(filename)
        else:
            path.write_text(value, encoding='utf-8')
    if drift:
        raise SystemExit('Diagram drift: '+', '.join(drift))
    print('PASS: 04-42 five 16:9 SVG/draw.io pairs; gallery; XML, IDs, ownership, bounds, orthogonal routes, source parity')


if __name__ == '__main__':
    main()
