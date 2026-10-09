#!/usr/bin/env python3
"""44 documentation only. Shared editable scene, SVG and draw.io source parity."""
import argparse
from pathlib import Path
import xml.etree.ElementTree as ET

from generate_dp_comparison_slides import Comparison
from continuous_interaction_scene import draw_option, legend, stable, Option

OUT=Path(__file__).resolve().parents[2]/'docs/architecture/12-decisions/decision-packages/diagrams'


class Slide(Comparison):
    def __init__(self,slug,title):
        self.number=44;self.slug=slug;self.caption=title;self.items=[]
        self.rect(0,0,2560,1440,'white','none')
        self.text(64,28,1900,['VIA / 04-44 / 지속 입력과 업무 질문·결과의 실행 연결'],22,'#000000',True)
        self.text(2496,28,520,['A/B 미선정 / 미측정'],22,'#000000',align='right')
        self.text(64,77,2432,[title],42,'#000000',True)
        self.text(64,143,2432,['VIA가 요청 이해·업무 위임·응답 처리를 하는 중에도 계속 들어오는 사용자 입력과 Agent의 질문·결과를 어떤 SW 구조로 처리할 것인가?'],24,'#000000')
        self.line([(1280,195),(1280,1328)],color='#DDDDDD',arrow=False,width=1)
        for side,x in [('A',64),('B',1340)]:
            self.text(x,198,1156,[side+' '+('중앙 조정 방식' if side=='A' else '이벤트 흐름을 연결하는 방식')],31,'#000000',True)
            self.text(x,242,1156,['중앙 비동기 Orchestration' if side=='A' else '반응형 Dataflow'],21,'#000000')

    def svg(self):
        return stable(super().svg().replace('width="1920" height="1080" viewBox="0 0 1920 1080"',
                                              'width="2560" height="1440" viewBox="0 0 2560 1440"'))

    def diagram(self):
        d=super().diagram();m=d.find('mxGraphModel');m.set('pageWidth','2560');m.set('pageHeight','1440')
        return d

    def drawio(self):return stable(super().drawio())


def structure():
    s=Slide('choice44-structure','계속 들어오는 입력과 질문·결과를 처리하는 SW 구조')
    for side,x in [('A',64),('B',1340)]:
        draw_option(s,x,345,1,side)
        s.text(x,1118,1160,['C1 대화 / R2: u2 “잠깐, 표부터 설명해줘” / T1 메일 업무·Q1 수신자 질문 / P1 보고서 재생·P2 표 설명 후보'],18,'#000000')
        s.text(x,1150,1160,[('Dialogue Progress State: job/revision/대기·후보·중단 참조 / 완료 검사·후속 요청 / 늦은 u1 폐기' if side=='A' else 'Publication Join: 입력 해소·admission·source.rev·과거 receipt / 확인 질문 별도 admission / 자기 receipt 후행')],18,'#000000')
        s.text(x,1182,1160,['권한: owner ↔ Policy Manager / 내구 확정: owner → State Store / 같은 Request·Task·질문·publication 원본'],17,'#000000')
        s.text(x,1212,1160,['Voice Runtime에는 Interaction Manager의 음성 입출력·즉시 중단 기능과 Model Access client가 배치된다.'],18,'#000000')
        s.text(x,1242,1160,['공통: 동시 실행 수가 제한된 비동기 executor. Blocking 작업은 제한된 worker pool. 모델은 Model Access의 API admission.'],17,'#000000')
        support=('사건 대기열·비동기 작업 실행·완료 결과 반환은 실행 기반이 지원한다. 짧은 상태 전이·중요 알림 기아 방지. 용량·우선순위 수치는 미정.' if side=='A' else
                 '이벤트 채널·구독·단계별 수용량 조절·취소 전달은 실행 라이브러리가 지원한다. credit/cancel은 Task 취소가 아니다.')
        s.text(x,1272,1160,[support],17,'#000000')
        s.text(x,1302,1160,['capture·ASR·즉시 stop은 이해 완료와 무관 / hold는 Task 취소가 아님 / 로컬 VIA / 클라우드 음성·의미 API'],17,'#000000')
    legend(s)
    return s


def event():
    s=Slide('choice44-event','같은 u2·Q1 사례의 실제 소유자와 후속 실행')
    for side,x in [('A',64),('B',1340)]:
        g=Option(s,x,0,1,side)
        lanes={}
        for key,name,xx,w in [('im','Interaction Manager',0,205),('rc','Request Controller',225,390),
                              ('task','Task Manager',635,185),('rm','Response Manager',840,320)]:
            g.node(key,xx,282,w,168,name,size=18 if w<210 else 22)
            lanes['tm' if key=='task' else key]=x+xx+w/2
            s.line([(x+xx+w/2,450),(x+xx+w/2,1260)],color='#DDDDDD',arrow=False,width=1,dashed=True)
        g.node('channel',12,331,181,27,'Channel I/O','module',owner='im',size=16)
        g.node('turn',12,367,181,32,'Turn-Taking Control','module',owner='im',size=14)
        g.node('playback',12,408,181,35,'Playback State','state',owner='im',size=14)
        if side=='A':
            g.node('dispatch',237,336,180,43,'Dialogue Dispatcher','module',True,'rc',16)
            g.node('progress',435,336,168,55,'Dialogue Progress\nState','state',True,'rc',14)
        else:
            g.node('input',237,330,180,38,'Input Resolution\nStage','module',True,'rc',14)
            g.node('iw',435,330,168,38,'Input Window','state',True,'rc',14)
            g.node('notice',237,392,180,38,'Task Notice Stage','module',True,'rc',15)
            g.node('nw',435,392,168,38,'Notice Window','state',True,'rc',14)
        g.node('compose',852,331,145,35,'Response\nComposer','module',owner='rm',size=14)
        g.node('publish',1005,331,143,35,'Publication\nControl','module',owner='rm',size=14)
        if side=='B':
            g.node('join',852,375,145,33,'Publication Join','module',True,'rm',13)
            g.node('pw',1005,375,143,35,'Publication Window','state',True,'rm',11)
        g.node('outbox',852,415,296,31,'Publication Outbox','state',owner='rm',size=14)
        def msg(a,b,y,label,ret=False):
            s.line([(lanes[a],y),(lanes[b],y)],color='#000000',dashed=ret,width=1.1)
            left=min(lanes[a],lanes[b])+12; width=x+1160-left
            s.rect(left-5,y-32,width,27,'white','none')
            s.text(left,y-31,width,[label],19,'#000000')
        s.text(x,455,1160,['P1 실제 재생 중. u2 시작 즉시 Turn-Taking Control → Channel I/O stop, output epoch 무효화.'],19,'#000000')
        msg('im','rc',530,'InputStarted(u2) / revision → 관련 미전송 admission hold')
        msg('rc','tm',590,'hold·권한·명령 확인 → Agent Gateway 전송 제어')
        msg('tm','rc',650,'Q1 확인·저장 → 원래 대화 연결 / Pending User Interaction / 게시 admission',True)
        msg('im','rc',710,'Input(u2,evidence refs) → Context Manager 조회·Request Interpreter 제안·채택')
        s.text(x+228,740,927,['늦은 u1 job/candidate는 revision 검사로 폐기. Q1 원본 보존, 준비·게시 조건 대기.'],18,'#000000')
        msg('rc','rm',815,'Dialogue Dispatcher: 준비 요청 / job ID' if side=='A' else '채택 의미·허용 Notice 게시 → Response Composer 소비')
        if side=='A':
            msg('rm','rc',880,'준비 후보 반환 → Dialogue Progress State에서 후속 조건 연결',True)
            msg('rc','rm',945,'Dialogue Dispatcher → Publication Control: 유효 P2 / Q1 게시 요청')
        else:
            s.text(x+840,849,320,['Response Composer → Publication Join','후보·관련 admission·ID/revision 결합'],15,'#000000')
            s.text(x+840,905,320,['Publication Window: 대기·폐기','조건 충족 → Publication Control'],17,'#000000')
        s.text(x+228,982,930,['원본 현재성·권한·차례·중복 최종 검사. 현재 입력의 확인 질문은 별도 admission.'],18,'#000000')
        msg('rm','im',1045,'release / publication identity → 유효 Q1 실제 표시·재생')
        msg('im','rm',1110,'실제 표시·재생·중단 receipt / actual range → Publication Outbox 갱신',True)
        msg('rm','rc',1175,'확인된 실제 Q1 제시 기록 → 질문 focus / 후속 답변 연결 근거',True)
        s.text(x,1237,1160,['자기 후보 receipt는 게시 뒤 생성 / 과거 snapshot은 known-empty·UNKNOWN 구분 / 예시는 고정 우선순위 아님'],18,'#000000')
        s.text(x,1280,1160,['실행 기반·배치 조건·모델 조건은 MAIN과 같음. 사건 간격과 선 길이는 측정 시간이 아님.'],18,'#000000')
    legend(s)
    return s


def validate(s):
    ids={i['id'] for i in s.items};assert len(ids)==len(s.items)
    byid={i['id']:i for i in s.items}
    for i in s.items:
        if i.get('owner') and i.get('option')!='legend':
            p=byid[i['owner']];assert p.get('semantic_kind') in ('component','external')
        if i['kind']=='line':
            for a,b in zip(i['points'],i['points'][1:]):assert a[0]==b[0] or a[1]==b[1]
            if 'option' in i:
                low,high=(64,1224) if i['option']=='A' else (1340,2500)
                assert all(low<=x<=high for x,y in i['points'])
        if i.get('semantic_kind') in ('module','state') and i.get('option')!='legend':assert i.get('owner')
    ET.fromstring(s.svg())
    root=ET.fromstring(s.drawio());cells=root.findall('.//mxCell');cellids={c.get('id') for c in cells}
    assert len(cellids)==len(cells)
    for cell in cells:
        for role in ('source','target','parent'):
            assert not cell.get(role) or cell.get(role) in cellids
        i=byid.get(cell.get('id'))
        if i and i.get('owner'):assert cell.get('parent')==i['owner']


def main():
    ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args();drift=[]
    for s in (structure(),event()):
        validate(s)
        for ext in ('svg','drawio'):
            value=getattr(s,ext)();path=OUT/(s.slug+'.'+ext)
            if args.check:
                if not path.exists() or path.read_text()!=value:drift.append(path.name)
            else:path.write_text(value)
    if drift:raise SystemExit('Diagram drift: '+', '.join(drift))
    print('PASS: 44 paired editable scenes; ten Component owners, contained Modules/state, named endpoints, orthogonal routes and A/B isolation')


if __name__=='__main__':main()
