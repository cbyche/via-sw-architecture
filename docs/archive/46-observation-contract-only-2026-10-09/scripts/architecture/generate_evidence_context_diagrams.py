#!/usr/bin/env python3
"""Generate only the separate 46 review figures. Never imports a 45 scene."""
import argparse
from pathlib import Path
from evidence_context_diagram_scene import Scene

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/diagrams'


def structure():
    s = Scene('46 현재 관측과 과거 기록을 요청 Context로 연결하기', 'choice46-structure')
    s.text(35, 18, 2450, ['46  현재 관측과 과거 기록을 요청 Context로 연결하기'], 38, True)
    s.text(35, 77, 2450, ['같은 u46: “이 표와 [스크롤/창 전환] 저 표를 아까 설명한 기준으로 비교해서 제안서 초안을 만들어줘. 메일은 보내지 마.”'], 25)
    s.text(35, 122, 2450, ['45의 과거 근거 조합을 확장한 검토안 / 04-40 로컬 VIA + 클라우드 모델 / A/B 미선정 / 경로 번호는 직렬 순서가 아님'], 22)
    for p, x, heading in [('a', 30, 'A  요청별 원본 근거 구성'), ('b', 1310, 'B  공통 파생 근거 생산과 조회')]:
        def n(i, xx, yy, w, h, name, kind='module', owner=None, different=False):
            s.node(p+i, x+xx, yy, w, h, name, kind, p+owner if owner else None, different)
        def e(i, j, pts, both=False):
            s.edge(p+i, p+j, [(x+xx, yy) for xx, yy in pts], both)
        def t(xx, yy, w, lines, size=20, bold=False):
            s.text(x+xx, yy, w, lines, size, bold)
        t(20, 174, 1170, [heading], 31, True)
        n('mic', 25, 233, 220, 60, 'Microphone', 'external')
        n('os', 490, 233, 355, 60, 'OS Screen / Pointer', 'external')
        n('voice', 915, 233, 270, 60, 'Cloud Voice / Transcript', 'external')
        n('im', 20, 335, 865, 235, 'Interaction Manager', 'component')
        n('channel', 40, 385, 215, 55, 'Channel I/O', owner='im')
        n('capture', 545, 385, 310, 55, 'Evidence Capture', owner='im')
        n('vad', 40, 480, 225, 60, 'Turn-Taking Control', owner='im')
        n('timeline', 310, 480, 255, 60, 'Timeline & Buffer', owner='im')
        n('window', 610, 475, 245, 75, 'Input Observation\nWindow', 'state', 'im')
        n('mav', 915, 365, 270, 125, 'Model Access', 'component')
        n('vapi', 930, 419, 240, 50, 'Voice API Client', owner='mav')
        e('mic', 'channel', [(135,293),(135,385)])
        e('channel', 'vad', [(145,440),(145,480)])
        e('os', 'capture', [(695,293),(695,385)])
        e('channel', 'vapi', [(255,410),(280,410),(280,318),(900,318),(900,444),(930,444)])
        e('vapi', 'voice', [(1060,419),(1060,293)], True)
        e('vapi', 'timeline', [(1170,452),(1198,452),(1198,590),(440,590),(440,540)])
        e('capture', 'window', [(735,440),(735,475)])
        e('vad', 'timeline', [(265,515),(310,515)])
        e('timeline', 'window', [(565,515),(610,515)], True)
        t(25, 297, 230, ['1 audio samples'], 18)
        t(325, 296, 550, ['1 cloud audio / session-item ↔ local samples'], 18)
        t(515, 344, 330, ['2 screen / pointer / drag + capture time'], 17)
        t(950, 310, 240, ['1 audio ↔ transcript revision'], 17)
        t(935, 500, 245, ['3 acoustic span + item mapping', 'receive time ≠ speech time', 'word timing 없으면 unknown'], 17)
        t(50, 445, 225, ['local VAD / stop'], 17)
        t(585, 447, 280, ['2 S1/P1 → S2/P2 + gap'], 17)
        n('rc', 20, 655, 280, 72, 'Request Controller', 'component')
        n('ri', 20, 835, 280, 85, 'Request Interpreter', 'component')
        e('im', 'rc', [(155,570),(155,655)])
        t(20, 602, 295, ['4 Input + Evidence Record', 'u46@r1 / W46 / observation refs'], 18)
        e('rc', 'ri', [(155,727),(155,835)], True)
        t(25, 753, 290, ['5 input / revision / bounded read', '의미 제안 → 현재 검사/채택'], 18)
        # The identical authorities remain sources, never a pre-resolved current Task.
        n('rm', 850, 655, 335, 65, 'Response Manager', 'component')
        n('tm', 850, 775, 335, 65, 'Task Manager', 'component')
        t(855, 728, 325, ['E20 원문 / P20 실제 전달 범위'], 18)
        t(855, 848, 325, ['T21/D21@v2 / T22/D22@v1'], 18)
        n('cm', 365, 655, 455, 470, 'Context Manager', 'component')
        n('um', 855, 938, 320, 65, 'User Memory', 'state')
        t(855, 1008, 320, ['Context Manager 소유 / 명시 저장'], 17)
        n('mas', 850, 1083, 335, 112, 'Model Access', 'component')
        n('sapi', 867, 1137, 300, 43, 'Semantic API Client', owner='mas')
        n('llm', 850, 1250, 335, 62, 'Cloud Semantic LLM', 'external')
        e('sapi', 'llm', [(1020,1180),(1020,1250)], True)
        e('ri', 'sapi', [(155,920),(155,1205),(837,1205),(837,1159),(867,1159)], True)
        t(170, 1148, 630, ['7 C-INTERPRET: 현재 지칭/Task 연결은 Request Interpreter', '채택은 Request Controller / 현재 R46/T46은 판단 결과'], 19)
        t(25, 1270, 760, ['공통: 원본/권한/epoch 검사와 gap / 유한 보존 / 수정 시 새 revision', '모델 제공은 Model Access / 실제 Agent 실행/응답 출력은 04-40', '화면/pointer/전사 도착 순서를 현재 지칭 정답으로 쓰지 않음'], 19)
        if p == 'a':
            n('producer', 395, 745, 380, 65, 'Context Composer', owner='cm', different=True)
            n('cache', 395, 931, 380, 55, 'Source View Cache', 'state', 'cm')
            n('set', 395, 1040, 380, 55, 'Request Evidence Set', 'state', 'cm', True)
            e('producer','timeline',[(530,745),(530,630),(440,630),(440,540)],True)
            e('producer','rc',[(395,758),(345,758),(345,690),(300,690)],True)
            e('producer','rm',[(775,769),(836,769),(836,687),(850,687)],True)
            e('producer','tm',[(775,793),(832,793),(832,807),(850,807)],True)
            e('producer','um',[(775,798),(828,798),(828,970),(855,970)],True)
            e('ri','producer',[(300,880),(330,880),(330,778),(395,778)],True)
            e('producer','cache',[(585,810),(585,931)],True)
            e('producer','set',[(420,810),(420,916),(380,916),(380,1068),(395,1068)],True)
            e('producer','sapi',[(755,810),(755,890),(802,890),(802,1169),(867,1169)],True)
            t(552, 595, 320, ['6 요청한 시간 범위/후보 읽기'], 18)
            t(380, 701, 420, ['6 C20 / 과거 입력과 질문 연결도 owner read'], 17)
            t(375, 838, 420, ['6 필요한 owner read / 발췌/출처 구성', 'Bundle + QueryReceipt → 현재 해석', 'cache/부분 갱신/병렬 read도 허용'], 18)
            t(399, 996, 375, ['범위/revision/오차 / Request별 유효 set'], 17)
            t(405, 1100, 410, ['요청별 구성은 모델 재해석과 다름'], 17)
            t(400, 1214, 410, ['7 필요 시 C-CONTEXT', '단순 ID/time join은 코드'], 18)
        else:
            n('producer', 395, 722, 380, 60, 'Evidence Publisher', owner='cm', different=True)
            n('repo', 395, 875, 380, 80, 'Context Evidence\nRepository', 'state', 'cm', True)
            n('reader', 395, 1040, 380, 55, 'Evidence Reader', owner='cm', different=True)
            e('producer','timeline',[(530,722),(530,630),(440,630),(440,540)],True)
            e('producer','rc',[(395,740),(345,740),(345,690),(300,690)],True)
            e('producer','rm',[(775,740),(832,740),(832,687),(850,687)],True)
            e('producer','tm',[(775,766),(832,766),(832,807),(850,807)],True)
            e('producer','um',[(775,770),(826,770),(826,970),(855,970)],True)
            e('producer','repo',[(585,782),(585,875)])
            e('reader','repo',[(585,1040),(585,955)],True)
            e('ri','reader',[(300,880),(330,880),(330,1068),(395,1068)],True)
            e('reader','producer',[(395,1074),(380,1074),(380,752),(395,752)],True)
            e('producer','sapi',[(755,782),(755,800),(800,800),(800,1169),(867,1169)],True)
            t(552, 595, 340, ['6 owner 변경/필요 범위 → 생산 read'], 18)
            t(380, 693, 420, ['6 C20 / 과거 입력과 질문 연결도 owner read'], 17)
            t(405, 793, 395, ['6 temporal / historical + dependency', 'expected revision 검사/게시'], 18)
            t(405, 973, 400, ['6 유효 record + coverage 읽기', '미게시라면 생산/게시 후 재조회'], 18)
            t(399, 1100, 395, ['Bundle + Receipt / NOT_COVERED'], 17)
            t(400, 1214, 410, ['7 필요 시 C-CONTEXT', '미사용/정정/재생산도 비용'], 18)
    s.text(35, 1360, 2490, ['범례: 직각 Component / 둥근 Module / 원통 상태 / 육각형 외부 / 살구색 차이. 각 칸의 두 Model Access 표시는 같은 Component의 client 확대 보기.'], 21)
    return s


def timeline():
    s = Scene('46 발화 시각과 관측, 전사 도착과 근거 준비', 'choice46-timeline')
    s.text(40, 18, 2470, ['46  발화 시각과 관측 시각, 전사 도착과 근거 준비를 구별하기'], 37, True)
    s.text(40, 78, 2470, ['같은 u46 / 모든 시각은 설명용 local monotonic 예시이며 성능 목표나 측정값이 아님 / 단어 timing은 provider capability 미확인'], 23)
    headers = [(40, '로컬 발화 / acoustic 구간'), (650, '당시 화면 / 행동'), (1260, '클라우드 전사 도착'), (1870, '코드가 보존하는 의미 전 상태')]
    for x, title in headers:
        s.text(x, 138, 575, [title], 26, True)
    data = [
        ('speech1',40,205,'E1  t=100.0~100.6\n“이 표와” / audio sample range'),
        ('screen1',650,205,'S1 / P1  t=100.0~100.7\nwindow A / viewport v1 / pointer path'),
        ('arrival1',1260,205,'t=101.2  provisional r0\n수신 시각은 E1 발화 시각이 아님'),
        ('state1',1870,205,'W46 / input ID / clock epoch\nS1/P1 원본 refs와 capture interval'),
        ('speech2',40,347,'E2  t=100.8~101.5\n“저 표를 아까 설명한 기준으로…”'),
        ('screen2',650,347,'S2 / P2  t=100.8~101.6\nscroll / window B / viewport v2'),
        ('arrival2',1260,347,'t=102.1  final r1 + item mapping\nspan 가능 여부/오차는 별도 정보'),
        ('state2',1870,347,'S2/P2 및 source seq / watermark\n누락은 gap / 같은 좌표 ≠ 같은 표'),
        ('speech3',40,489,'E3  t=101.7  로컬 VAD 종료\nprovider final 완료와 다른 경계'),
        ('screen3',650,489,'E6  늦은 capture 또는 전사 r2\n이전 job 취소 / 새 input revision'),
        ('arrival3',1260,489,'timing 없으면 span=unknown\n최신 화면으로 E1/E2를 대체하지 않음'),
        ('state3',1870,489,'E8  누락/오차로 불명확한 지칭\n다른 근거 / 재지칭 / 확인 필요'),
    ]
    for i,x,y,name in data:
        s.node(i,x,y,570,85,name,'data')
    s.node('im',40,640,1800,225,'Interaction Manager','component')
    s.node('cap',70,719,370,60,'Evidence Capture','module','im')
    s.node('tl',590,719,400,60,'Timeline & Buffer','module','im')
    s.node('win',1160,719,610,70,'Input Observation Window','state','im')
    s.node('rc',1900,719,620,70,'Request Controller','component')
    s.edge('screen2','cap',[(1220,390),(1235,390),(1235,610),(255,610),(255,719)])
    s.edge('arrival2','tl',[(1260,390),(1248,390),(1248,590),(1070,590),(1070,690),(790,690),(790,719)])
    s.edge('cap','tl',[(440,749),(590,749)])
    s.edge('tl','win',[(990,749),(1160,749)],True)
    s.edge('win','rc',[(1770,754),(1900,754)])
    s.text(445,785,660,['capture/time/sample 대응은 코드 / 단어와 대상의 의미는 별도'],19)
    s.text(1905,803,605,['Input + Evidence Record / u46@r1 / W46', '원문/후보refs/gap / 아직 R46/T46의 정답이 아님'],20)
    s.text(75,825,1740,['공통 원본: acoustic span ↔ local sample ↔ capture interval + 오차. 수신 순서나 VAD만으로 단어 시각을 만들지 않음.'],20)
    # Two lower lanes expose the different readiness dependency for the identical first input.
    s.text(40,906,1200,['A  E4: 필요한 원본에서 이번 Bundle 구성'],27,True)
    s.text(1320,906,1200,['B  E4: 유효 게시 record가 Context 읽기의 준비 조건'],27,True)
    s.node('acm',40,973,1210,230,'Context Manager','component')
    s.node('acomposer',75,1050,335,60,'Context Composer','module','acm',True)
    s.node('aset',500,1045,335,70,'Request Evidence Set','state','acm',True)
    s.node('ari',930,1035,280,80,'Request Interpreter','component','acm')
    # RI is an independent Component: its box is outside the Context Manager boundary.
    s.nodes[-1]['owner'] = None
    s.nodes[-4]['w'] = 810
    s.edge('win','acomposer',[(1310,789),(1310,880),(20,880),(20,1080),(75,1080)])
    s.edge('acomposer','aset',[(410,1080),(500,1080)])
    s.edge('aset','ari',[(835,1080),(930,1080)])
    s.text(80,1127,750,['같은 P20 / Task-result / 명시 User Memory를 owner read', 'E5 재사용: 유효 cache/부분 갱신. 현재 정정 의미는 재판단.'],20)
    s.node('bcm',1320,973,950,230,'Context Manager','component')
    s.node('bpub',1350,1050,260,60,'Evidence Publisher','module','bcm',True)
    s.node('brepo',1645,1045,295,80,'Context Evidence\nRepository','state','bcm',True)
    s.node('breader',1980,1050,250,60,'Evidence Reader','module','bcm',True)
    s.node('bri',2300,1035,220,85,'Request\nInterpreter','component')
    s.edge('win','bpub',[(1450,789),(1450,1050)])
    s.edge('bpub','brepo',[(1610,1080),(1645,1080)])
    s.edge('breader','brepo',[(1980,1080),(1940,1080)],True)
    s.edge('breader','bpub',[(2105,1050),(2105,1020),(1495,1020),(1495,1050)])
    s.edge('breader','bri',[(2230,1080),(2300,1080)])
    s.text(1370,1135,870,['NOT_COVERED → 필요 범위 생산/검사/게시 → 재조회 → 같은 현재 해석', 'E5 유효 record 재사용 / E6 dependency 차단/재생산 / 원본 우회는 B+.'],19)
    s.text(45,1240,2470,['E7 취소: capture 지속, 옛 의미 job과 미전송 처리 차단. E9 장애/E11 재시작: 원본이 없는 파생 view는 복원 불가.'],22)
    s.text(45,1285,2470,['E10 만료/철회: 원본/cache/파생 record와 미사용 job의 이용 gate 차단과 cleanup. 짧은 화면 view를 장기 기억에 자동 복제하지 않음.'],22)
    s.text(45,1360,2470,['그림은 관측과 Context 공급 확대. 입력 확정/Context 준비/현재 의미 채택/실제 전송은 다른 경계. A/B 품질 우위는 미검증.'],22)
    return s


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    changed = []
    for scene in (structure(), timeline()):
        for ext, value in scene.render().items():
            path = OUT / f'{scene.slug}.{ext}'
            if args.check:
                if not path.exists() or path.read_text() != value:
                    changed.append(str(path.relative_to(ROOT)))
            else:
                path.write_text(value)
    if changed:
        raise SystemExit('Out of sync: ' + ', '.join(changed))
    print('PASS: 46 SVG/draw.io generation matches' if args.check else 'Wrote 46 MAIN and timeline SVG/draw.io')


if __name__ == '__main__':
    main()
