#!/usr/bin/env python3
"""Stage 4 functional alternatives, paired editable sources and SVG previews."""
import argparse
from pathlib import Path
import xml.etree.ElementTree as ET
from stage4_diagram_design import Plate, BLUE, GREEN, INK, MUTED

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/diagrams'


def base(slug, number, title, thesis):
    return Plate(slug, number, title, thesis, height=2140)


def panel(p, x, s, c, title, note):
    p.box(x,237,48,48,c,'none',5)
    p.text(x+24,244,s,27,'white','center',True)
    p.text(x+68,236,title,28,bold=True)
    p.text(x+68,278,note,18,MUTED)
    p.boundary(s+'via',x,340,1184,1340,'VIA SOFTWARE','논리 책임 / process 분리 아님')


def footer(p, note):
    p.line([(64,2000),(2496,2000)],'#C7D1DD',arrow=False)
    p.text(64,2024,'검정 = 공통 / 파랑 = A 고유 / 초록 = B 고유 / 색은 선호나 우위가 아님',18,bold=True)
    p.text(64,2058,'제목 칸 = Component / 둥근 내부 상자 = Module / 원통 = 저장 / 육각형 = 외부 의존성',17,MUTED)
    p.text(64,2091,note,17,MUTED)
    p.text(2496,2024,'실선 → 요청/전달   점선 → 반환/확인',17,MUTED,align='right')


def model(p,x,k):
    p.component(k('ma'),x+36,1340,510,110,'Model Access',role='같은 Omni 한 개에 요청 / 결과 반환')
    p.node(k('omni'),x+36,1825,510,110,'공유 on-device Omni\n모델 내부는 VIA 책임 밖',kind='model')
    p.edge(k('ma'),k('omni'),sd=-35,td=-35,label='추론 요청',at=(x+65,1600))
    p.edge(k('omni'),k('ma'),sp='T',tp='B',sd=35,td=35,ret=True,label='결과 반환',at=(x+340,1725))


def semantic_calls(p,x,k):
    p.edge(k('ri'),k('ma'),sp='L',tp='R',sd=-30,td=-15,via=[(x+590,955),(x+590,1380)],label='3 의미 판단',at=(x+365,1260))
    p.edge(k('ma'),k('ri'),sp='R',tp='L',sd=15,td=30,via=[(x+618,1410),(x+618,1015)],ret=True)


def grounding():
    p=base('functional-grounding','F-01','화면 지칭의 정확성을 위한 대상 연결 — 사건별 재해석과 지속 객체 추적',
           '같은 음성 시점과 화면/포인터 관측에서 시작한다. 요청에 공급하는 화면 대상의 생산 경로가 다르다.')
    for x,s,c in [(64,'A',BLUE),(1312,'B',GREEN)]:
        k=lambda n:s+n
        panel(p,x,s,c,'발화 관측으로 재해석' if s=='A' else '지속 대상 모델에서 결합',
              '새 표현을 raw 관측과 함께 판단' if s=='A' else '동일성/지시 관계를 유지; 전체 raw 최종 판단 없음')
        p.component(k('im'),x+36,385,1112,115,'Interaction Manager',role='1 같은 화면/포인터/선택/창 관측 + Speech Input Worker의 발화/전사 시점')
        p.component(k('cm'),x+36,610,510,590,'Context Manager',role='화면 근거 생산 / gap와 유효 범위 관리')
        p.node(k('mod'),x+66,735,450,120,'발화 구간 조립' if s=='A' else '객체 추적 / 지시 관계',color=c,owner=k('cm'))
        p.node(k('data'),x+66,975,450,135,'요청 관측 묶음 (RAM)\n화면 조각과 포인터/선택 시점' if s=='A' else '대상 모델 / 시점 snapshot (RAM)\n객체 ID, 관계, revision과 불확실성',kind='store',color=c,owner=k('cm'))
        p.edge(k('mod'),k('data'),label='1 관측으로 구성' if s=='A' else '1 변경에 따라 갱신',at=(x+130,900),color=c)
        p.component(k('rc'),x+650,610,498,130,'Request Controller',role='2 입력 revision / 조회 시점과 범위')
        p.component(k('ri'),x+650,900,498,170,'Request Interpreter',role='3 지칭의 대상/집합/역할 후보 생성')
        p.text(x+675,1020,'원문 + 관측 묶음' if s=='A' else '원문 + 대상 snapshot (전체 raw 없음)',18,c)
        p.component(k('end'),x+650,1340,498,110,'Request Controller',role='4 현재 근거를 확인해 의미 확정')
        p.edge(k('im'),k('cm'),sd=-301,label='1 같은 관측',at=(x+70,550))
        p.edge(k('im'),k('rc'),sd=307,label='2 요청 시작',at=(x+925,550))
        p.edge(k('rc'),k('cm'),sp='L',tp='R',sd=-20,td=-250,label='2 조회',at=(x+565,618))
        p.edge(k('cm'),k('rc'),sp='R',tp='L',sd=-195,td=35,ret=True,label='2 반환',at=(x+565,735))
        p.edge(k('rc'),k('ri'),label='3 해석 요청',at=(x+940,810))
        p.edge(k('ri'),k('end'),ret=True,label='3 후보 반환 / 4 검증',at=(x+920,1230))
        model(p,x,k);semantic_calls(p,x,k)
        if s=='B':
            p.edge(k('cm'),k('ma'),sd=-35,td=-35,label='1 시각 판단 필요 시',at=(x+65,1260),color=c)
            p.edge(k('ma'),k('cm'),sp='T',tp='B',sd=35,td=35,ret=True,color=c)
        p.text(x+650,1510,['5 Request Controller → Task Manager → Agent Gateway:',
                          '   확정한 표/문서의 참조와 목표를 외부 Agent에 전달.',
                          '   접수/결과는 같은 Task를 통해 돌아온다.',
                          '위아래 Request Controller는 같은 Component다.'],18,MUTED)
        p.text(x+650,1810,['A: 요청 시 분석 비용 / 새 표현에 유연',
                          'B: 갱신 비용 / 잘못된 객체 연결이 뒤 요청에 전파',
                          '최종 사용 권한과 외부 업무 실행은 공통 경계'],19,MUTED)
    footer(p,'구조 제안 / 미선정 / 미측정. B의 파생 대상 모델은 독립적인 source 사실이 아니다.')
    return p


def dialogue():
    p=base('functional-dialogue','F-02','여러 업무의 대화 정확성을 위한 연결 — 전체 대화 해석과 업무별 대화 상태',
           '한 VIA 창과 같은 Task/질문/실제 전달 사실. 업무 대화의 상태 소유 및 해석에 들어가는 순서를 비교한다.')
    for x,s,c in [(64,'A',BLUE),(1312,'B',GREEN)]:
        k=lambda n:s+n
        panel(p,x,s,c,'전체 대화에서 연결 판단' if s=='A' else '업무 대화에 배정 후 판단',
              '관련 Task/질문을 함께 보고 연결 제안' if s=='A' else '단위별 제약/미해결 질문 유지; 교차 요청은 다시 조합')
        p.component(k('top'),x+36,385,1112,115,'Request Controller',role='1 Agent Gateway/Task Manager의 질문과 실제 전달 기록 / 2 Interaction Manager의 사용자 발화')
        p.component(k('owner'),x+36,610,510,590,'Request Controller' if s=='A' else 'Task Dialogue Manager',color=INK if s=='A' else c,
                    role='Conversation의 대화 상태' if s=='A' else 'VIA 내부 업무별 대화 상태 / 새 Component')
        p.node(k('mod'),x+66,715,450,85,'근거 조회 조정' if s=='A' else '단위별 입력/참조 조정',color=c,owner=k('owner'))
        p.node(k('q1'),x+66,860,450,110,'공통 질문/대화 관계 (내구)\n관련 Task를 함께 조회' if s=='A' else '보고서 대화 단위 (내구)\n제약 / 질문 / 실제 전달',kind='store',color=c,owner=k('owner'))
        p.node(k('q2'),x+66,1020,450,110,'발화별 관련 근거 (임시)\n명시 대상 / 실제 제시 / 대기 질문' if s=='A' else '메일 대화 단위 등 (내구)\n제약 / 질문 / 실제 전달',kind='store',color=c,owner=k('owner'))
        p.component(k('ri'),x+650,900,498,170,'Request Interpreter',role='3 현재 범위의 의미/질문/Task 제안')
        p.text(x+675,1020,'관련 업무의 근거를 함께 판단' if s=='A' else '배정된 단위의 근거로 판단',18,c)
        p.component(k('end'),x+650,1340,498,110,'Request Controller',role='4 단위 결과/발화 관계와 현재 유효성 검증')
        p.edge(k('top'),k('owner'),sd=-301,label='2 전체 요청' if s=='A' else '2 절별 배정 후보',at=(x+70,550),color=c)
        p.edge(k('owner'),k('ri'),sp='R',tp='L',sd=50,td=-30,label='3 해석 요청',at=(x+552,915),color=c)
        p.edge(k('ri'),k('owner'),sp='L',tp='R',sd=15,td=95,ret=True,label='3 제안 반환',at=(x+552,1030),color=c)
        p.edge(k('owner'),k('end'),via=[(x+291,1250),(x+899,1250)],ret=True,label='3 제안 취합',at=(x+670,1210),color=c)
        model(p,x,k)
        p.edge(k('ri'),k('ma'),sp='B',tp='R',sd=-35,td=-15,via=[(x+864,1280),(x+590,1280),(x+590,1380)],label='3 의미 판단',at=(x+365,1260))
        p.edge(k('ma'),k('ri'),sp='R',tp='B',sd=15,td=35,via=[(x+618,1410),(x+618,1305),(x+934,1305)],ret=True)
        if s=='B':
            p.edge(k('top'),k('ri'),sd=407,td=100,label='2 필요 시 범위 판단',at=(x+685,680),color=c)
            p.edge(k('ri'),k('top'),sp='T',tp='B',sd=170,td=477,ret=True,label='2 배정 후보 반환',at=(x+690,800),color=c)
        p.text(x+650,1510,['5 Request Controller → Task Manager → Agent Gateway:',
                          '   대상이 확인된 질문 답변/수정/취소만 전달.',
                          '   실제 상태는 Response Manager를 통해 사용자에게 반환.',
                          '근거 조회는 Context Manager에 요청/반환한다.',
                          '같은 이름의 Request Controller는 같은 owner다.'],18,MUTED)
        p.text(x+650,1800,['A: 자유로운 교차 대화 / 후보 혼입과 문맥 회수 부담',
                          'B: 긴 업무 대화 보존 / 선배정 오류와 참조 왕복 부담',
                          '“응”의 대상이 실제로 불명확하면 양안 모두 질문'],19,MUTED)
    footer(p,'업무별 단위는 VIA 대화 상태다. Agent 내부 thread, 별도 process 또는 모델 복제가 아니다.')
    return p


def voice():
    p=base('functional-voice','F-03','음성 요청의 정확성과 대화 연속성을 위한 처리 — 원음 중심과 Text 중심',
           '같은 원음, 같은 ASR, 같은 Omni. 의미 입력과 응답 생성/실제 전달 기록의 연결을 비교한다.')
    for x,s,c in [(64,'A',BLUE),(1312,'B',GREEN)]:
        k=lambda n:s+n
        panel(p,x,s,c,'원음을 직접 이해하고 연결' if s=='A' else 'Text 확정 후 이해/음성화',
              '전사는 보조; 원음의 단서를 의미 판단에 유지' if s=='A' else '전사와 발화 Text가 공통 의미 경계; native S2S 미지원')
        p.component(k('im'),x+36,385,1112,115,'Interaction Manager',role='1 같은 microphone 입력 / 입력 revision / 모델과 무관한 즉시 local stop')
        p.component(k('asrworker'),x+36,565,450,110,'Speech Input Worker',role='1 연속 인식과 전사/단어 시점')
        p.component(k('rc'),x+650,565,498,110,'Request Controller',role='2 입력 준비 / 3 현재 의미와 권한 확정')
        p.component(k('ri'),x+650,740,498,185,'Request Interpreter',role='')
        p.node(k('meaning'),x+675,830,448,70,'원음 + 시점 / 전사 보조' if s=='A' else '확정 Text + 수정 revision',kind='data',color=c,owner=k('ri'))
        p.component(k('ma'),x+36,960,450,370,'Model Access',role='단일 weights / 역할과 Context 구분')
        p.node(k('sem'),x+66,1060,390,85,'semantic 역할 연동',owner=k('ma'))
        p.node(k('speech'),x+66,1180,390,85,'음성 역할 연동',owner=k('ma'))
        p.component(k('rm'),x+650,1120,498,210,'Response Manager')
        p.node(k('record'),x+675,1210,448,95,'허용 의미 + 생성 Text/audio 관계\n실제 전달과 연결할 내구 기록' if s=='A' else '확정 답변 / 발화 Text\n실제 읽힌 범위와 연결할 내구 기록',kind='store',color=c,owner=k('rm'))
        p.component(k('out'),x+650,1500,498,140,'Interaction Manager',role='5 Text/음성 전달과 실제 구간 반환; local stop')
        p.node(k('asr'),x+36,1825,450,110,'같은 Streaming ASR\n입력 인식 의존성',kind='model')
        p.node(k('omni'),x+650,1825,498,110,'같은 on-device Omni 한 개\n원음/의미/음성 capability는 검증 전',kind='model')
        p.edge(k('im'),k('asrworker'),sd=-331,label='1 음성',at=(x+85,520))
        p.edge(k('asrworker'),k('im'),sp='R',tp='B',td=-72,via=[(x+520,620)],ret=True)
        p.edge(k('im'),k('rc'),sd=307,label='2 입력',at=(x+960,520),color=c)
        p.edge(k('asrworker'),k('asr'),sp='L',tp='L',sd=-15,td=-15,via=[(x+12,605),(x+12,1865)])
        p.edge(k('asr'),k('asrworker'),sp='L',tp='L',sd=15,td=15,via=[(x+24,1895),(x+24,635)],ret=True)
        p.edge(k('rc'),k('ri'),label='2 해석',at=(x+950,692),color=c)
        p.edge(k('ri'),k('rc'),sp='R',tp='R',sd=0,td=25,via=[(x+1165,832.5),(x+1165,645)],ret=True)
        p.edge(k('ri'),k('ma'),sp='L',tp='R',sd=-12.5,td=-125,via=[(x+560,820),(x+560,1020)],label='2 요청',at=(x+505,940),color=c)
        p.edge(k('ma'),k('ri'),sp='R',tp='L',sd=-85,td=27.5,via=[(x+600,1060),(x+600,860)],ret=True)
        p.edge(k('rc'),k('rm'),sp='R',tp='T',sd=45,via=[(x+1180,665),(x+1180,1090),(x+899,1090)],label='3 확정 사실',at=(x+940,1010))
        p.edge(k('rm'),k('ma'),sp='L',tp='R',sd=-100,td=-20,label='4 생성 요청',at=(x+500,1095),color=c)
        p.edge(k('ma'),k('rm'),sp='R',tp='L',sd=110,td=30,ret=True,label='4 결과 반환',at=(x+500,1288),color=c)
        p.edge(k('rm'),k('out'),sd=-35,td=-35,label='5 게시',at=(x+755,1410))
        p.edge(k('out'),k('rm'),sp='T',tp='B',sd=35,td=35,ret=True,label='5 실제 전달',at=(x+950,1410))
        p.edge(k('ma'),k('omni'),via=[(x+261,1740),(x+899,1740)],label='2 / 4 모델 요청',at=(x+55,1520))
        p.edge(k('omni'),k('ma'),sp='T',tp='B',sd=35,td=35,via=[(x+934,1780),(x+296,1780)],ret=True,label='2 / 4 반환',at=(x+330,1580))
        p.text(x+650,1695,'5 후속 발화는 실제 전달 범위와 함께 단계 2로 돌아간다.',17,MUTED)
    footer(p,'생성 완료 ≠ 실제 청취. A도 게시 검증을 거치며 B도 streaming/중첩을 사용할 수 있다.')
    return p


def main():
    ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args()
    drift=[]
    for p in [grounding(),dialogue(),voice()]:
        p.validate()
        for ext,data in [('svg',p.svg()),('drawio',p.drawio())]:
            root=ET.fromstring(data)
            if ext=='drawio':
                ids=[e.get('id') for e in root.findall('.//mxCell')]
                assert len(ids)==len(set(ids))
                for e in root.findall('.//mxCell'):
                    for key in ['parent','source','target']:
                        assert e.get(key) is None or e.get(key) in ids
            dest=OUT/f'{p.slug}.{ext}'
            if args.check:
                if not dest.exists() or dest.read_text()!=data:drift.append(dest.name)
            else:dest.write_text(data)
    if drift:raise SystemExit('Diagram drift: '+', '.join(drift))
    print('PASS: 3 functional comparison pairs; source parity, XML, ownership, bounds and routes')

if __name__=='__main__':main()
