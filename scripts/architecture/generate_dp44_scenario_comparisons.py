"""Four focused, editable DP44 A/B execution comparisons.

The existing overview and implementation figures remain unchanged. Each scene
shows one event boundary so a reader can follow the actual queue destination.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import xml.etree.ElementTree as ET

from generate_dp_comparison_slides import Comparison
from generate_dp_background_slides import INK, MUTED, BLUE, RED, LINE


ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/diagrams'
PALE_A = '#EAF2FB'
PALE_B = '#FFF5DB'
PALE_MODULE = '#FFF3E5'
PALE_COMMON = '#F4F7FA'
ORANGE = '#B8753F'
POSITIONS = {5: [80, 435, 790, 1145, 1500], 4: [90, 555, 1020, 1485]}
WIDTHS = {5: 310, 4: 345}

SCENARIOS = [
    dict(slug='input', title='일반 사용자 발화: 이해 완료는 어디로 돌아오는가',
         a_header='A  Event Mailbox가 완료를 회수하고 Dispatcher가 다음 처리를 지시',
         b_header='B  Input Resolution Stage가 자기 Done Queue에서 완료를 처리',
         event='Interaction Manager가 InputReady(u1)를 Request Controller에 전달한다.',
         condition='발화 시작의 즉시 재생 중단은 공통이다. 이 장은 확정 입력의 이해 경로만 따라간다.',
         a=[
             ('queue','Request Controller','Event Mailbox','InputReady(u1)','일반 사건을 수용'),
             ('module','Request Controller','Dialogue Dispatcher','InterpretAsync(J1) 지시','Job Index에 J1 등록'),
             ('module','Request Interpreter','InterpretAsync(J1)','의미 제안을 생산','필요하면 모델 호출'),
             ('queue','Request Controller','Event Mailbox','JobDone(J1)','첫 칸과 같은 Queue'),
             ('module','Request Controller','Dialogue Dispatcher','결과 현재성 확인','의미 채택 후 준비 지시'),
         ],
         b=[
             ('queue','Request Controller','Input Resolution Stage','Input Queue: u1','입력 Stage가 직접 수용'),
             ('module','Request Controller','Input Resolution Stage Pump','InterpretAsync(J1) 시작','Input Window에 J1 기록'),
             ('module','Request Interpreter','InterpretAsync(J1)','의미 제안을 생산','A와 같은 이해 책임'),
             ('queue','Request Controller','Input Resolution Stage','Done Queue: J1','시작한 Stage에 반환'),
             ('module','Request Controller','Input Resolution Stage Pump','결과 현재성 확인','채택 후 Composer에 전달'),
         ],
         a_edges=['data','data','done','data'], b_edges=['data','data','done','data'],
         takeaway='A는 같은 Event Mailbox로 완료를 회수한다. B는 입력 Stage의 Done Queue가 완료를 받는다.'),
    dict(slug='agent', title='Agent 질문: 확인된 Task 사건은 어디로 들어가는가',
         a_header='A  TaskEvent와 준비 완료가 중앙 Dispatcher를 경유',
         b_header='B  Task Notice Stage에서 Composer와 Join으로 전달',
         event='Downstream Agent의 질문 Q1을 Agent Gateway와 Task Manager가 확인한다.',
         condition='확인된 Question(T1, r2)만 대화에 연결한다. 외부 Agent 접수와 사용자 실제 제시는 별도 사실이다.',
         a=[
             ('queue','Request Controller','Event Mailbox','TaskEvent(T1,Q1)','확인된 업무 사건 수용'),
             ('module','Request Controller','Dialogue Dispatcher','대화와 Task 연결 확인','질문 등록과 준비 지시'),
             ('module','Response Manager','Response Composer','PrepareAsync(J2)','사용자용 질문 준비'),
             ('queue','Request Controller','Event Mailbox','JobDone(J2)','같은 중앙 Queue로 반환'),
             ('module','Request Controller','Dialogue Dispatcher','준비 결과 확인','Publication Control 요청'),
         ],
         b=[
             ('queue','Request Controller','Task Notice Stage','Task Queue: Q1','업무 Stage가 직접 수용'),
             ('module','Request Controller','Task Notice Stage Pump','연결과 허용 범위 확인','Notice Window에 보류'),
             ('queue','Response Manager','Response Composer','Prepare Queue: Notice','연결된 소비자 활성화'),
             ('queue','Response Manager','Response Composer','Done Queue: J2','준비 완료를 자체 수용'),
             ('module','Response Manager','Publication Join','Candidate와 Condition 결합','유효하면 Publication Control'),
         ],
         a_edges=['data','data','done','data'], b_edges=['data','data','done','data'],
         takeaway='A의 TaskEvent와 준비 완료는 Dispatcher를 경유한다. B는 Notice, Composer, Join의 입력으로 이어진다.'),
    dict(slug='onset', title='새 발화 시작: 출력 중단과 미전송 hold',
         a_header='A  Control Lane에서 Dispatcher가 hold를 연결',
         b_header='B  관련 Stage의 Control Queue에 hold를 전달',
         event='Interaction Manager가 InputStarted(u2)를 감지하고 현재 재생을 즉시 멈춘다.',
         condition='R1 응답 J1이 준비 중이다. u2의 뜻은 아직 모르므로 J1과 Task 취소를 확정하지 않는다.',
         a=[
             ('queue','Request Controller','Control Lane','InputStarted(u2)','일반 사건 backlog와 분리'),
             ('module','Request Controller','Dialogue Dispatcher','우선 Handle(onset)','미전송 효과 hold 요청'),
             ('module','Request Controller','Admission hold','미전송 효과 차단','진행 중 J1은 추적 유지'),
             ('module','Response Manager','Publication Control','관련 release 보류','게시 직전 다시 확인'),
         ],
         b=[
             ('queue','Request Controller','Input Resolution Stage','Control Queue: u2','입력 제어를 우선 수용'),
             ('module','Request Controller','Input Resolution Stage Pump','미전송 효과 hold','관련 소비자에 제어 전달'),
             ('queue','Response Manager','Response Composer','Control Queue: hold','J1의 후보 배출 보류'),
             ('queue','Response Manager','Publication Join','Control Queue: hold','관련 후보 게시 보류'),
         ],
         a_edges=['control']*3, b_edges=['control']*3,
         takeaway='A는 Control Lane 뒤 Dispatcher가 hold를 연결한다. B는 관련 Stage의 Control Queue에 제어를 전달한다.'),
    dict(slug='correction', title='정정 채택과 늦은 완료: J1은 어디에서 폐기되는가',
         a_header='A  Job Index에서 무효화하고 Event Mailbox 완료를 폐기',
         b_header='B  Composer의 자기 Done 처리와 Join에서 폐기',
         event='Request Controller가 R1 rev2를 채택한다. rev1에 의존한 J1은 더는 유효하지 않다.',
         condition='취소 요청 뒤에도 provider가 J1을 끝낼 수 있다. 결과 폐기와 실제 실행 종료는 구별한다.',
         a=[
             ('module','Request Controller','Dialogue Dispatcher','R1 rev2 채택 반영','J1 무효화 결정'),
             ('state','Request Controller','Job Index','J1 unusable','CancelHandle로 취소 요청'),
             ('queue','Request Controller','Event Mailbox','늦은 JobDone(J1)','rev1 결과가 도착'),
             ('module','Request Controller','Dialogue Dispatcher','J1 결과 폐기','종료 상태와 비용 추적'),
             ('module','Response Manager','Publication Control','현재성 최종 확인','구 후보의 게시 차단'),
         ],
         b=[
             ('module','Request Controller','Input Resolution Stage Pump','R1 rev2 채택 반영','관련 Invalidate 발행'),
             ('queue','Response Manager','Response Composer','Control Queue: Invalidate','J1 무효화와 취소 요청'),
             ('queue','Response Manager','Response Composer','Done Queue: J1','늦은 완료가 도착'),
             ('module','Response Manager','Response Composer Pump','자기 J1 결과 폐기','준비 Job 상태와 비용 추적'),
             ('queue','Response Manager','Publication Join','Control Queue: Invalidate','구 Candidate 폐기'),
         ],
         a_edges=['control','done','data','data'], b_edges=['control','done','data','control'],
         takeaway='A는 중앙 Job Index와 Dispatcher가 폐기한다. B는 Composer의 자기 Done 처리와 Join이 관련 결과를 막는다.'),
]


def scene(spec, index):
    s = Comparison.__new__(Comparison)
    s.number, s.slug, s.items = 44, f'dp44-scenario-{spec["slug"]}', []
    s.caption = f'DP44 {index}/4. {spec["title"]}'
    s.rect(0, 0, 1920, 1080, 'white', 'none')
    s.text(54, 30, 1790, [s.caption], 40, INK, True)
    s.line([(54, 103), (1866, 103)], color=LINE, arrow=False, width=2)
    s.rect(54, 129, 1812, 103, PALE_COMMON, LINE)
    s.text(72, 141, 170, ['같은 사건'], 20, INK, True)
    s.text(241, 140, 1600, [spec['event']], 23, INK, True)
    s.text(241, 178, 1600, [spec['condition']], 18, MUTED)

    def lane(y, label, color, steps, edges):
        s.rect(54, y, 1812, 54, color, LINE)
        s.text(75, y+13, 1740, [label], 24, INK, True)
        n=len(steps); xs=POSITIONS[n]; w=WIDTHS[n]
        top=y+87
        for j,(kind,owner,title,body,foot) in enumerate(steps):
            x=xs[j]
            s.text(x, top-28, w, [owner], 18, INK, True)
            if kind=='queue':
                s.rect(x, top, w, 120, 'white', ORANGE)
                s.text(x+12, top+13, w-24, [title], 21, INK, True)
                s.text(x+12, top+46, w-24, [body], 18, INK)
                for k in range(4):
                    s.rect(x+12+k*27, top+86, 20, 20, 'white', MUTED)
            elif kind=='state':
                s.add('store',x=x,y=top,w=w,h=120,fill=PALE_MODULE,stroke=ORANGE,dashed=False,store_radius=8)
                s.text(x+14, top+42, w-28, [title], 21, INK, True)
                s.text(x+14, top+78, w-28, [body], 18, INK)
            else:
                s.rect(x, top, w, 120, PALE_MODULE, ORANGE)
                s.text(x+12, top+16, w-24, [title], 21, INK, True)
                s.text(x+12, top+59, w-24, [body], 18, INK)
            s.text(x, top+141, w, [foot], 18, MUTED)
            if j<n-1:
                x1=x+w+6;x2=xs[j+1]-8;cy=top+65
                color=RED if edges[j]=='control' else BLUE
                s.line([(x1,cy),(x2,cy)],color=color,arrow=True,
                       dashed=edges[j] in ('control','done'),width=3)

    lane(263,spec['a_header'],PALE_A,spec['a'],spec['a_edges'])
    lane(610,spec['b_header'],PALE_B,spec['b'],spec['b_edges'])
    s.rect(54, 966, 1812, 64, PALE_COMMON, LINE)
    s.text(72, 981, 1760, [spec['takeaway']], 22, INK, True)
    s.text(54, 1042, 1812, ['파란선: 일반 사건과 다음 처리    파란 점선: 비동기 완료    붉은 점선: 제어와 무효화    A/B 미선정, 미구현, 미측정'], 17, MUTED)
    return s


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    for index,spec in enumerate(SCENARIOS,1):
        s=scene(spec,index)
        content={
            OUT/f'{s.slug}.svg':s.svg(),
            OUT/f'{s.slug}.drawio':s.drawio(),
            OUT/f'{s.slug}.scene.json':json.dumps({'slug':s.slug,'title':s.caption,'items':s.items,
                                                  'notes':spec['event']+'\n'+spec['condition']+'\n'+spec['takeaway']},ensure_ascii=False,indent=2)+'\n',
        }
        for path,value in content.items():
            if args.check:
                assert path.read_text()==value,path
            else:
                path.write_text(value)
            if path.suffix in ('.svg','.drawio'):
                ET.fromstring(value)
            assert '·' not in value,path
    print(f'PASS: four DP44 scenario scenes {"verified" if args.check else "written"}')


if __name__=='__main__':
    main()
