"""04-43: integrated versus function-owned semantic production.

F1-F6 are identical obligations in both options. All boxes name implementable
components/modules; report/mail are data examples, never module types.
"""
import argparse
from pathlib import Path
import xml.etree.ElementTree as ET

from stage4_diagram_design import Plate, INK, MUTED, BLUE, LINE

PURPLE = '#7446A4'
TITLE = '같은 여섯 판단을 함께 할 것인가, 기능별로 나누어 할 것인가?'


class Slide(Plate):
    def __init__(self, slug, title, height=1440):
        super().__init__(slug, '04-43', title, '', height=height)
        self.items = []
        self.serial = 0
        self.box(0, 0, 2560, 9, INK, 'none', 0)
        self.text(64, 24, 'VIA / 43 / A 통합 판단 · C 기능별 판단', 24, MUTED, bold=True)
        self.text(2496, 24, '설계 후보 / 미선정 / 미측정', 24, MUTED, 'right')
        self.text(64, 72, title, 41, bold=True)

    @staticmethod
    def text_geometry(i):
        size = 27 if i['w'] >= 290 else 25
        leading = size * 1.22
        return i['y']+(i['h']-len(i['name'])*leading)/2-2+(7 if i['type']=='store' else 0),size,leading

    def svg(self):
        svg = super().svg().replace('font-size="23"', 'font-size="27"')
        marker = f'<marker id="a{PURPLE[1:]}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M1 1L9 5L1 9" fill="none" stroke="{PURPLE}" stroke-width="1.6"/></marker>'
        return svg.replace('</defs>', marker+'</defs>')

    def drawio(self):
        return super().drawio().replace('fontSize=23;', 'fontSize=27;')

    def arrow(self, a, b, label='', at=None, sp='B', tp='T', via=(), color=INK, ret=False, sd=0, td=0, size=25):
        self.edge(a,b,sp=sp,tp=tp,via=via,color=color,ret=ret,sd=sd,td=td)
        if label:
            self.label(*at,label,color,size=size)


FUNCTIONS = [
    ('intent', 'Request Intent\nInterpreter', 'F1  원문 → 목표·완료 조건·제약'),
    ('referent', 'Referent Resolver', 'F2  화면/자료 → 지칭 대상·범위'),
    ('association', 'Request Association\nResolver', 'F3  대화/Task 후보 → 연결'),
    ('relation', 'Request Relation\nInterpreter', 'F4  F1~F3 → 요청 간 의존·조건'),
    ('revision', 'Request Revision\nInterpreter', 'F5  이전 의미 → 유지·교체·취소'),
    ('handling', 'Request Handling\nInterpreter', 'F6  F1~F5 → 직접응답·위임·확인'),
]


def structure():
    p=Slide('choice43-structure',TITLE)
    p.text(64,137,'예시 입력  “보고서는 PDF로, 메일에는 그 보고서의 결론을 넣어줘.” → “메일에는 결론 대신 표만.”',29,bold=True)
    p.text(64,184,'같은 근거: 원문·화면 시점, 관련 자료, 이전 조건·실제 질문, Task 후보와 확인된 결과. Task 연결은 아직 판단할 대상이다.',25,MUTED)
    p.line([(64,224),(2496,224)],LINE,arrow=False)
    p.text(64,240,'A  한 모듈이 전체 의미를 생산',35,BLUE,bold=True)
    p.text(1306,240,'C  기능별 모듈이 자기 의미를 생산',35,PURPLE,bold=True)
    p.text(64,288,'F1~F6을 함께 판단 / Task별 검색·부분 수정·모듈화 가능',25,BLUE)
    p.text(1306,288,'F1~F6을 분리 / 코드가 교환·의존 관리 / 충돌은 원 생산자 재판단',25,PURPLE)
    p.component('Arc',96,338,1120,64,'Request Controller')
    p.component('Crc',1336,338,1120,64,'Request Controller')
    p.component('Aowner',64,510,1190,610,'Request Interpreter',BLUE)
    p.component('Cowner',1306,510,1190,610,'Request Interpreter',PURPLE)
    p.node('Ajudge',226,650,860,96,'Integrated Semantic Interpreter',owner='Aowner',color=BLUE)
    p.arrow('Arc','Ajudge','1  원문 + 관련 근거',(140,429),sd=-170,td=-170,color=BLUE)
    p.arrow('Ajudge','Arc','3  전체 의미 제안 / 미해결 반환',(720,467),sp='T',tp='B',sd=170,td=170,color=BLUE,ret=True)
    p.node('Astate',276,998,760,78,'Interpretation Attempt State',kind='store',owner='Aowner',color=BLUE)
    p.arrow('Ajudge','Astate','2  F1 의도 / F2 대상 / F3 대화·Task 연결\n    F4 요청 관계 / F5 정정 범위 / F6 처리 방향',
            (210,789),sd=-35,td=-35,color=BLUE)
    p.arrow('Astate','Ajudge','이전 제안·근거 읽기 / 같은 판단 책임으로 수정',(210,908),sp='T',tp='B',sd=35,td=35,color=BLUE,ret=True)
    p.label(140,1083,'임시 전체 제안과 근거 / 소유: Request Interpreter',BLUE,size=24)

    # A logical coordinator owns scheduling and the ledger, never other modules' values.
    p.node('coord',1761,630,270,380,'Interpretation\nCoordinator',owner='Cowner',color=INK)
    p.arrow('Crc','coord','1  같은 원문 + 기능별 허용 근거',(1390,429),sd=-60,td=-60,color=PURPLE)
    p.arrow('coord','Crc','3  참조로 결합한 의미 / 미해결',(2050,467),sp='T',tp='B',sd=60,td=60,color=PURPLE,ret=True)
    p.label(1738,577,'2  코드: 배정·검사·재판단 회송',INK,size=22)
    for index,(key,name,caption) in enumerate(FUNCTIONS):
        left=index<3
        row=index%3
        x=1336 if left else 2116
        y=630+145*row
        p.node(key,x,y,350,88,name,owner='Cowner',color=PURPLE)
        p.label(x,592+145*row,caption,PURPLE,size=23)
        delta=y+44-820
        p.arrow('coord',key,sp='L' if left else 'R',tp='R' if left else 'L',
                sd=delta-13,td=-13,color=PURPLE)
        p.arrow(key,'coord',sp='R' if left else 'L',tp='L' if left else 'R',
                sd=13,td=delta+13,color=PURPLE,ret=True)
    p.node('Cstate',1636,1027,520,80,'Partial Interpretation Ledger',kind='store',owner='Cowner',color=PURPLE)
    p.arrow('coord','Cstate',sd=-22,td=-22,color=PURPLE)
    p.arrow('Cstate','coord',sp='T',tp='B',sd=22,td=22,color=PURPLE,ret=True)
    p.text(64,1137,'공통 4  Request Controller: 현재 입력·근거·질문·권한 검사 → State Store에 채택 기록. 미채택 제안은 실행·게시 금지.',25,bold=True)
    p.text(64,1175,'위임: Task Manager → Agent Gateway ↔ Downstream Agent  |  직접 응답/질문: Response Manager ↔ Interaction Manager',24)
    p.line([(64,1220),(2496,1220)],LINE,arrow=False)
    p.text(64,1234,['정정: 한 모듈이 메일 조건만 교체한 전체 의미를 제안. PDF 유지.',
                     '이익 후보: 전체 관계 이해·적은 조정  /  대가: 맥락 혼입·통합 판단 부담'],25,BLUE,leading=37)
    p.text(1306,1234,['정정: F2 표 → F5 교체 → F4 의존 갱신. 기존 PDF 조건 유지.',
                       '이익 후보: 기능별 집중·오류 분석  /  대가: 불일치·추가 호출·왕복'],25,PURPLE,leading=37)
    p.text(64,1320,'모든 해석 모듈 → Model Access ↔ 공유 on-device Omni 1벌. 역할별 Context/KV·호출·조정 비용은 남고 음성 입력/ASR은 계속.',24,bold=True)
    p.text(64,1362,'외곽은 Component / 내부 둥근 상자는 Module / 원통은 채택 전 임시 상태. 실선: 근거·유효 제안 전달 / 점선 화살표: 판단·미해결 반환.',22,MUTED)
    p.text(64,1400,'공통 자료 읽기는 Request Controller → Context Manager 및 각 원본 소유자. 최종 Task 사실은 Task Manager 소유. 여섯 기능은 고정 호출 횟수가 아니다.',21,MUTED)
    return p


def event():
    p=Slide('choice43-event','같은 정정: 누가 바꾸고, 무엇을 보존하며, 어디로 재판단을 돌리는가?',height=1780)
    p.text(64,140,'이미 채택된 조건: 보고서 PDF / 메일에 보고서 결론. 새 입력: “메일에는 결론 대신 표만 넣어줘.”',28)
    p.text(64,184,'보고서·메일은 데이터 예시다. R/M 연결이나 표 대상이 모호하면 후보를 유지하고 추가 근거/확인을 요청한다.',25,MUTED)
    groups=[
        ('A',BLUE,['Request\nController','Integrated Semantic\nInterpreter','Interpretation\nAttempt State'],[
            (0,1,'1  새 원문 + 이전 의미 + Task 후보 + 표 근거',False),
            (1,2,'2  F1~F6 전체 제안: 메일 내용만 교체 / PDF 유지',False),
            (2,1,'현재 제안·근거 / 수정 이력 반환',True),
            (1,0,'3  새 통합 의미와 근거 / 경쟁 후보가 있으면 미해결',True),
            (0,0,'4  현재 검사 → State Store 채택 → 공통 위임/응답',False),
            (0,1,'5  뒤늦은 새 근거: 영향을 받은 의미만 재판단',False),
            (1,0,'부분 수정 또는 새 전체 제안 / 필요한 확인 질문',True),
        ]),
        ('C',PURPLE,['Request\nController','Interpretation\nCoordinator','Referent\nResolver','Request Revision\nInterpreter','Request Relation\nInterpreter'],[
            (0,1,'1  같은 입력/근거 / F1 의도·F3 연결·F6 방향도 유효성 확인',False),
            (1,2,'2  F2: 표 후보 + 원문 + 관련 자료',False),
            (2,1,'표 참조의 새 제안 / 불명확하면 후보 유지',True),
            (1,3,'F5: 이전 의미 + F1/F3/F4/F2 참조 → 교체 범위 판단',False),
            (3,1,'메일 내용 조건 교체 / 보고서 PDF 조건 보존',True),
            (1,4,'F4: 새 표 참조 + F5 변경 → 의존 관계 갱신',False),
            (4,1,'결론 의존 제거 / 표 의존 생성',True),
            (1,0,'3  유효한 F1~F6 제안 참조를 결합해 반환',True),
            (0,0,'4  같은 현재 검사/채택. 필수 의미 미해결이면 보류',False),
            (1,2,'5  새 근거로 대상 충돌: 원 생산자 F2에 재판단 요청',False),
            (1,1,'영향받은 F4/F5/F6 재판단 / 중앙에서 값 덮기 금지',False),
        ])]
    for j,(letter,color,names,steps) in enumerate(groups):
        x=64+j*1242
        p.text(x,250,letter+'  같은 결과, 다른 의미 생산 책임',32,color,bold=True)
        xs=[x+140,x+590,x+1040] if j==0 else [x+100+i*242 for i in range(5)]
        w=270 if j==0 else 210
        for xx,name in zip(xs,names):
            p.box(xx-w/2,320,w,88,'white',color,4)
            p.text(xx,333,name,24 if j==0 else 22,color,'center',True,leading=30)
            p.line([(xx,408),(xx,1410)],LINE,dashed=True,arrow=False)
        for i,(a,b,label,ret) in enumerate(steps):
            y=488+i*(140 if j==0 else 85)
            p.text(x+5,y-42,label,23,color)
            pts=[(xs[a],y),(xs[b],y)] if a!=b else [(xs[a],y),(xs[a]+55,y),(xs[a]+55,y+20),(xs[a],y+20)]
            p.line(pts,color,dashed=ret,width=2.5)
    p.text(64,1450,'공통 예외와 경계',29,bold=True)
    p.text(64,1500,[
        '새 입력은 관련 미전송 의도를 보류. 전송 후에는 실제 외부 접수/실행 확인 없이 취소 완료나 rollback을 주장하지 않는다.',
        '철회/삭제: 관련 근거·전체/부분 제안·모델 Context/KV의 신규 사용을 차단. 이미 제공한 정보의 소급 회수는 보장하지 않는다.',
        '실패/재시작: 내구 채택 기록과 외부 Task 사실을 복원. 임시 제안은 재검증/재계산하며 이전 명령을 중복 실행하지 않는다.',
        'C의 F1/F3/F6과 원장은 메인에 표시. 이 보충 사건도는 정정으로 변하는 F2/F5/F4를 확대하며 고정 실행 순서를 뜻하지 않는다.',
        '모든 의미 추론은 Model Access의 공유 Omni. 외부 업무 계획·도구 선택·실행은 Downstream Agent 책임.'
    ],25,leading=40)
    p.text(64,1730,'실선 = 요청/전달 · 점선 화살표 = 결과 반환. 메인 비교는 choice43-structure.svg. 두 안 모두 설계 후보이며 미측정.',22,MUTED)
    return p


OUT = Path(__file__).resolve().parents[2] / "docs/architecture/12-decisions/decision-packages/diagrams"

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args();drift=[]
 plates=[structure(),event()]
 for p in plates:
  p.validate()
  for i in p.items:
   if i['kind']=='component':assert not i['role'],(p.slug,'behavior inside component')
   if i['kind']=='text':
    for node in p.items:
     if node['kind']=='component':
      assert not (node['x'] < i['x'] < node['x']+node['w'] and node['y']+62 < i['y'] < node['y']+node['h']), (p.slug,'free prose inside component',i['lines'])
  for ext,data in [('svg',p.svg()),('drawio',p.drawio())]:
   root=ET.fromstring(data)
   if ext=='drawio':
    ids=[e.get('id') for e in root.findall('.//mxCell')];assert len(ids)==len(set(ids))
    for e in root.findall('.//mxCell'):
     for attr in ('source','target','parent'):assert not e.get(attr) or e.get(attr) in ids
   dest=OUT/(p.slug+'.'+ext)
   if args.check:
    if not dest.exists() or dest.read_text()!=data:drift.append(dest.name)
   else:dest.write_text(data)
 if drift:raise SystemExit('Diagram drift: '+', '.join(drift))
 print(f'PASS: {len(plates)} pairs; source parity, XML, ownership, bounds, routes, name-only component headers')
if __name__=='__main__':main()
