#!/usr/bin/env python3
"""Stage 4 mechanisms and responsibility ownership; no model execution."""
from __future__ import annotations
import argparse
import html
from pathlib import Path
import xml.etree.ElementTree as ET
from stage4_diagram_design import Plate, INK, BLUE, GREEN, MUTED, LINE, AMBER

ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'docs/architecture/12-decisions/decision-packages/diagrams'


class Area:
    def __init__(self,p,i,x):self.p,self.i,self.x=p,i,x
    def key(self,k):return f'{self.i}-{k}'
    def c(self,k,x,y,w,h,name,color=INK,role=''):
        return self.p.component(self.key(k),self.x+x,y,w,h,name,color,role)
    def n(self,k,x,y,w,h,name,kind='module',color=INK,owner=None):
        return self.p.node(self.key(k),self.x+x,y,w,h,name,kind,color,self.key(owner) if owner else None)
    def e(self,s,t,sp='B',tp='T',via=(),label='',at=None,color=INK,ret=False,sd=0,td=0):
        return self.p.edge(self.key(s),self.key(t),sp,tp,[(self.x+x,y) for x,y in via],label,(self.x+at[0],at[1]) if at else None,color,ret,sd,td)
    def b(self,k,x,y,w,h,title='VIA SOFTWARE',subtitle='비교에 필요한 책임만 표시',color=INK,dashed=False):
        self.p.boundary(self.key(k),self.x+x,y,w,h,title,subtitle,color,dashed)
    def t(self,x,y,value,size=21,color=INK,**kw):self.p.text(self.x+x,y,value,size,color,**kw)
    def label(self,x,y,value,color=INK,size=18):self.p.label(self.x+x,y,value,color,size)


def plate(n,title,thesis,height=2200):return Plate(f'stage4-s{n:02}-comparison',f'S-{n:02}',title,thesis,height)


def semantic():
    p=plate(1,'요청 해석 정확성을 위한 추가 자료 확인 설계 — 해석을 끝내고 재호출할 것인가, 멈춘 해석을 이어갈 것인가?',
            'T는 호출을 끝내고 Request Controller로 돌아온다. A는 읽기를 기다리는 작업과 모델 재개 계약을 새로 갖는다.',2400)
    p.box(64,230,2432,142,'#F7F9FC',LINE)
    p.text(88,247,'공통 시작 ①~⑤',21,bold=True)
    p.text(88,291,'Interaction Manager → Request Controller ↔ Context Manager ↔ 원본 소유자',24,bold=True)
    p.text(1250,296,'같은 최종 입력과 기본 자료를 준비한 뒤 아래로 진행',21,MUTED)
    for i,x in enumerate((64,1312)):
        a=Area(p,i,x);c=BLUE if i==0 else GREEN
        p.panel(x,1184,'T' if i==0 else 'A','호출 종료 → 자료 보완 → 새 호출' if i==0 else '같은 작업에서 읽기 대기 → 재개',
                '실제 target / 추가 읽기를 Request Controller가 조정' if i==0 else '탐색안 / Worker와 Broker, 모델 중단 및 재개 계약 추가',y=410)
        a.b('via',0,508,1184,1377)
        a.c('rc',36,572,1112,294,'Request Controller',role='공통: 입력과 권한 확인, 최종 의미 확정')
        a.n('prepare',63,688,365,66,'입력과 기본 자료 결합',owner='rc')
        a.n('final',781,688,337,66,'⑱ 최종 의미 확정',owner='rc')
        if i==0:
            a.n('control',63,781,690,61,'⑩ 추가 조회 조정 → ⑭ 두 번째 해석 호출',color=BLUE,owner='rc')
        else:
            a.t(67,786,'추가 읽기의 대기와 재개는 아래 Worker가 맡는다.',20,GREEN)
        a.c('exec',36,1010,474,405 if i else 261,'Request Interpreter' if i==0 else 'Semantic Resolution Worker',c,
            '호출 한 번의 해석만 수행' if i==0 else '요청별 해석 작업의 수명을 소유')
        a.n('run',64,1123,418,64,'한 번 해석하고 제안 반환' if i==0 else '중단 상태 판별 / 작업 재개',color=c,owner='exec')
        a.n('state',64,1219,418,70,'호출 중 임시 자료' if i==0 else '멈춘 실행 ID + 읽은 자료 목록',kind='data',color=c,owner='exec') if i else a.n('state',64,1200,418,51,'호출 중 임시 자료',kind='data',color=c,owner='exec')
        if i:
            a.n('resume',64,1324,418,63,'같은 입력에 자료를 넣고 계속',color=GREEN,owner='exec')
            a.e('run','state',color=GREEN)
            a.e('state','resume',color=GREEN)
            a.c('broker',668,1010,480,211,'Capability Read Broker',GREEN,role='읽기 전용 요청을 검사하고 위임')
            a.n('check',695,1123,426,66,'읽기 범위와 남은 예산 검사',color=GREEN,owner='broker')
            a.e('exec','broker','R','L',sd=-60,via=[(582,1152.5),(582,1115.5)],label='⑨ 필요한 자료 요청',at=(510,965),color=GREEN)
            a.e('broker','exec','L','R',sd=32,td=44,via=[(607,1147.5),(607,1256.5)],label='⑭ 자료 반환',at=(520,1268),color=GREEN,ret=True)
        else:
            a.t(672,1040,['추가 읽기는 해석기 안에 없다.','⑨ 첫 호출이 끝난 뒤','Request Controller가 진행한다.'],22,BLUE)
        a.c('cm',668,1320,480,177,'Context Manager',role='원본을 읽고 현재 버전을 확인')
        a.n('read',695,1424,426,50,'허용된 자료 조회',owner='cm')
        if i:
            a.e('broker','cm',sd=-28,td=-28,label='⑩ 조회 위임',at=(715,1250),color=GREEN)
            a.e('cm','broker','T','B',sd=28,td=28,label='⑬ 자료 + 읽은 버전',at=(941,1280),color=GREEN,ret=True)
        else:
            a.e('control','cm','R','R',via=[(1170,811.5),(1170,1408.5)],label='⑩ 조회',at=(1000,1227),color=BLUE)
            a.e('cm','control','L','R',via=[(625,1408.5),(625,897),(1160,897),(1160,811.5)],ret=True,color=BLUE,label='⑬ 자료를 받은 뒤 새 호출',at=(735,907))
        a.e('prepare','exec','L','T',td=-25,via=[(22,721),(22,925),(248,925)],label='⑥ 해석 시작' if i else '⑥ 첫 해석 호출',at=(53,922),color=c)
        if i==0:
            a.e('exec','control','T','B',sd=50,via=[(323,928),(408,928)],label='⑨ 자료 부족 제안',at=(374,919),color=BLUE,ret=True)
            a.e('control','exec','L','T',td=-50,via=[(14,811.5),(14,976),(223,976)],label='⑭ 새 호출',at=(51,967),color=BLUE)
            a.e('exec','final','R','B',via=[(599,1140.5),(599,903),(949.5,903)],label='⑰ 최종 제안',at=(664,947),color=BLUE,ret=True)
        else:
            a.e('exec','final','T','B',sd=50,via=[(323,897),(949.5,897)],label='⑰ 최종 제안만 반환',at=(372,949),color=GREEN,ret=True)
        a.c('ma',36,1596,1112,246,'Model Access',role='같은 Omni 모델 사용 / 최종 요청 확정 권한은 없음')
        a.n('adapter',64,1722,438,82,'새 해석 호출 / 결과 변환' if i==0 else '중단 알림 / 실행 ID / 재개',color=c,owner='ma')
        a.n('kv',656,1722,465,82,'기존 모델 세션과 작업 메모리' if i==0 else '대기 중 실행 상태 보관과 회수',kind='data',color=INK if i==0 else GREEN,owner='ma')
        a.e('exec','adapter',sd=-28,td=-28,via=[(245,1530),(255,1530)],label='⑦ / ⑮ 모델 실행',at=(66,1516),color=c)
        a.e('adapter','exec','T','B',sd=28,td=28,via=[(311,1558),(301,1558)],ret=True,color=c,label='⑧ / ⑯ 실행 결과',at=(325,1550))
        a.e('adapter','kv','R','L',sd=-10,td=-10,label='상태 사용',at=(522,1712),color=INK if i==0 else GREEN)
        a.e('kv','adapter','L','R',sd=12,td=12,ret=True,color=INK if i==0 else GREEN)
        a.n('model',307,1922,570,89,'공유 Omni 모델 / 가중치 한 벌',kind='model')
        a.e('ma','model',sd=-25,td=-25)
        a.e('model','ma','T','B',sd=25,td=25,ret=True)
        p.focus(x,2040,1184,'책임 이동과 계약 변경',[
            'T: 해석기는 읽기 제안 후 종료한다. 추가 조회와 새 호출은 Request Controller가 소유한다.' if i==0 else 'A: Worker가 작업을 유지하고 Broker로 자료를 읽는다. 모델 연산은 기다리는 동안 양보한다.',
            'A는 이름 교체가 아니다: 읽기 대기 상태, 권한 검사 접점, 모델 재개 인터페이스가 필요하다.',
            '두 안 모두 Context Manager가 원본을 읽고 Request Controller가 최종 의미를 확정한다.'],
            '별도 호출도 기존 모델 세션을 재사용할 수 있다.' if i==0 else '모델의 중단 및 재개 지원은 미확인. 지원되지 않으면 이 대안은 성립하지 않는다.',c)
    p.footer('⑪~⑫ 원본 조회 왕복과 전체 호출 순서는 다음 실행 상세도에 표시한다. 상자 분리는 별도 process를 뜻하지 않는다.')
    return p


def screen():
    p=plate(2,'화면 지칭 정확성을 위한 화면 정보 처리 설계 — 수집한 화면 정보를 활용할 것인가, 별도 분석 결과도 활용할 것인가?',
            'A는 같은 관측에서 추가 해석 자료를 생산하고 관리한다. 조기 실행은 A만의 차이가 아니며, 품질 이익과 보강 방향은 미정이다.',2150)
    for i,x in enumerate((64,1312)):
        a=Area(p,i,x);c=BLUE if i==0 else GREEN
        p.panel(x,1184,'T' if i==0 else 'A','요청에 필요한 당시 화면 해석' if i==0 else '별도 화면 해석 자료 생산과 관리',
                '실제 target / UI 정보와 기존 cache도 활용' if i==0 else '후보 유지 / 구조적 가치 보강 필요')
        a.n('os',48,330,330,64,'OS와 앱 / 화면과 조작 정보',kind='external')
        a.n('mic',540,330,280,64,'마이크 / 사용자 음성',kind='external')
        a.b('via',0,412,874,1530,subtitle='')
        a.c('im',24,466,820,550,'Interaction Manager',role='화면과 음성 수집, 기록 보관, 시간 연결과 S2S 입력 연동')
        for key,xx,yy,hh,name in [('capture',48,580,54,'화면 관측 수집기'),('audio',526,580,54,'음성 입력 수집기'),
                                 ('manager',48,732,54,'화면 기록 관리자'),('voiceclient',526,704,54,'S2S 입력 연동기'),
                                 ('align',526,853,64,'입력 시각 연결기')]:
            a.n(key,xx,yy,294,hh,name,owner='im')
        a.n('raw',48,853,294,118,'최근 화면 기록\n이미지 / 조작 / 캡처 시각\n메모리에 임시 보관',kind='store',owner='im')
        a.e('os','capture','B','L',via=[(213,446),(12,446),(12,607)],label='변경 알림 / 주기 수집',at=(49,394))
        a.e('mic','audio',via=[(680,446),(673,446)],label='음성 도착',at=(694,425))
        a.e('capture','manager',label='발생 시각과 관측',at=(49,674))
        a.e('manager','raw',sd=-16,td=-16,label='보관 / 조회',at=(50,811))
        a.e('raw','manager','T','B',sd=16,td=16,ret=True)
        a.e('audio','manager','L','R',via=[(402,607),(402,759)],label='발화 시작\n관련 기록 유지',at=(360,664))
        a.e('audio','voiceclient',label='같은 음성 전달',at=(536,663))
        a.e('voiceclient','align',label='음성 해석과 응답 후보',at=(534,810))
        a.e('align','manager','L','R',sd=-14,td=-12,via=[(382,871),(382,747)])
        a.e('manager','align','R','L',sd=12,td=14,via=[(366,771),(366,899)],ret=True)
        a.label(350,820,'말한 시간으로\n기록 조회 / 반환')
        a.t(530,948,'인식 문장 + 당시 화면 근거 참조',18,MUTED)
        # Logical Model Access responsibility includes the ASR adapter in a separate worker.
        a.c('ma',510,1110,334,540,'Model Access')
        a.b('worker',530,1190,294,147,'Speech Input Worker','',dashed=True)
        a.t(542,1230,'ASR adapter의 독립 실행 경계',16,MUTED)
        a.n('asrport',551,1254,252,52,'음성 인식 연동기',owner='ma')
        a.n('voiceport',552,1410,265,56,'S2S 모델 연동기',owner='ma')
        a.n('semanticport',552,1532,265,56,'의미 분석 연동기',owner='ma')
        a.n('asr',930,1254,240,74,'Streaming ASR\n별도 음성 인식 모델',kind='model')
        a.n('omni',930,1460,240,100,'공유 Omni\n음성 / 의미 / 화면 분석\n가중치 한 벌',kind='model')
        a.t(927,1134,'모델 dependency',18,bold=True)
        a.t(927,1161,'로컬 실행 / 내부는 범위 밖',15,MUTED)
        a.e('audio','asrport','R','R',sd=-8,td=-10,via=[(850,599),(850,1270)])
        a.e('asrport','align','R','R',sd=10,td=-14,via=[(872,1290),(872,871)],ret=True)
        a.label(879,1060,'연속 음성 전달\n문장과 발화 시각 반환',size=17)
        a.e('asrport','asr','R','L',sd=-10,td=-10,via=[(890,1270),(890,1281)])
        a.e('asr','asrport','L','R',sd=10,td=10,via=[(906,1301),(906,1290)],ret=True)
        a.label(934,1203,'인식 요청 / 결과',size=17)
        a.e('voiceclient','voiceport','R','R',sd=-10,td=-10,via=[(858,721),(858,1428)])
        a.e('voiceport','voiceclient','R','R',sd=10,td=10,via=[(866,1448),(866,741)],ret=True)
        a.label(879,970,'S2S 음성 전달\n음성 해석 / 응답 후보 반환',size=17)
        a.e('voiceport','omni','R','L',sd=-10,td=-22,via=[(894,1428),(894,1488)])
        a.e('omni','voiceport','L','R',sd=-4,td=10,via=[(910,1506),(910,1448)],ret=True)
        a.label(930,1382,'음성 처리 요청 / 결과',size=17)
        a.e('semanticport','omni','R','L',sd=-10,td=22,via=[(888,1550),(888,1532)])
        a.e('omni','semanticport','B','R',td=12,via=[(1050,1610),(902,1610),(902,1572)],ret=True)
        a.label(921,1651,'의미 / 화면 분석\n요청과 결과',size=17)
        # A-only producer sits beside the shared model integration rather than below it.
        if i:
            a.c('sgs',24,1110,450,350,'Screen Grounding Service',GREEN)
            a.n('jobs',48,1220,180,54,'분석 작업 관리자',color=GREEN,owner='sgs')
            a.n('analyze',270,1220,180,54,'화면 대상 분석기',color=GREEN,owner='sgs')
            a.n('records',48,1340,180,54,'대상 기록 관리자',color=GREEN,owner='sgs')
            a.n('objects',270,1320,180,114,'화면 대상 기록\n항목 / 위치 / 관계\n시각 / 원본 참조\n메모리 임시 보관',kind='store',color=GREEN,owner='sgs')
            a.e('manager','jobs','L','L',via=[(8,759),(8,1247)],label='새 기록 준비 → 생산 시작',at=(49,1063),color=GREEN)
            a.e('jobs','analyze','R','L',label='범위와 예산 확인',at=(246,1178),color=GREEN)
            a.e('analyze','records','L','R',via=[(246,1247),(246,1367)],label='분석 결과',at=(51,1294),color=GREEN)
            a.e('records','objects','R','L',sd=-10,td=-10,via=[(252,1357),(252,1367)],color=GREEN)
            a.e('objects','records','L','R',sd=10,td=10,via=[(260,1387),(260,1377)],color=GREEN,ret=True)
            a.t(54,1411,'검증 후 보관 / 조회',16,GREEN)
            a.e('analyze','semanticport','R','L',sd=-10,td=-12,via=[(490,1237),(490,1548)],color=GREEN)
            a.e('semanticport','analyze','L','R',sd=12,td=10,via=[(500,1572),(500,1257)],color=GREEN,ret=True)
            a.label(516,1478,'이미지 해석이 필요할 때만 요청 / 결과',GREEN,16)
        else:
            a.t(48,1150,['별도 화면 대상 생산자는 없다.','요청에 필요한 당시 관측을 조회해','대상과 의도를 해석한다.'],23,BLUE,leading=37)
            a.t(48,1320,['UI 구조와 기존 cache는 활용한다.','매번 이미지부터 다시 분석한다고','가정하지 않는다.'],21,MUTED,leading=33)
        a.c('cm',24,1520,450,150,'Context Manager')
        a.n('context',48,1605,402,42,'화면 근거 구성기',owner='cm')
        a.e('context','manager','L','L',sd=-8,td=-12,via=[(2,1618),(2,747)],color=c)
        a.e('manager','context','L','L',sd=12,td=8,via=[(20,771),(20,1634)],color=c,ret=True)
        a.label(49,1488,'당시 기록 조회 / 반환' if i==0 else '미준비 또는 불확실하면 원래 기록 조회',c,17)
        if i:
            a.e('cm','records','T','B',sd=-20,td=-10,via=[(229,1480),(128,1480)],color=GREEN)
            a.e('records','cm','B','T',sd=10,td=20,via=[(148,1498),(269,1498)],color=GREEN,ret=True)
            a.label(289,1448,'대상 정보 조회 / 반환',GREEN,16)
        a.c('rc',24,1770,450,150,'Request Controller')
        a.n('control',48,1852,402,44,'요청 해석 조정기',owner='rc')
        a.c('ri',510,1770,334,150,'Request Interpreter')
        a.n('interpret',552,1852,265,44,'요청 의미 해석기',owner='ri')
        a.e('align','rc','B','T',via=[(673,1036),(482,1036),(482,1733),(249,1733)],label='인식 문장과 당시 근거 전달',at=(515,1033))
        a.e('audio','rc','B','T',sd=-90,td=36,via=[(583,660),(470,660),(470,1024),(486,1024),(486,1750),(285,1750)],label='발화 시작 알림',at=(350,944))
        a.e('rc','cm','T','B',sd=-18,td=-18,label='근거 요청',at=(48,1695))
        a.e('cm','rc',sd=18,td=18,ret=True,label='Context 반환',at=(285,1695))
        a.e('rc','ri','R','L',sd=-12,td=-12,label='해석 요청',at=(385,1739))
        a.e('ri','rc','L','R',sd=12,td=12,ret=True,label='의미 제안',at=(388,1924))
        a.e('ri','ma','T','B',sd=-18,td=-18,via=[(659,1715),(659,1678),(659,1650)],label='의미 해석 요청',at=(528,1690))
        a.e('ma','ri',sd=18,td=18,via=[(695,1727)],ret=True,label='결과',at=(741,1690))
        a.t(34,1958,'최종 검증: Request Controller / 모호하면 질문 또는 선택 요청 / 원래 기록과 현재 권한 확인',17,MUTED)
    p.footer('점선 Speech Input Worker는 ASR 연동 코드의 실행 경계다. 모델의 외부 표시는 VIA 책임 범위이며 원격 배치가 아니다.')
    return p


def screen_sequence(p,key,y,title,actors,events,note='',color=INK,unique_steps=()):
    """One independent event sequence; integer numbering matches its prose section."""
    centers=[220+j*(2120/(len(actors)-1)) for j in range(len(actors))]
    # Integer coordinates avoid cross-platform float rendering differences.
    centers=[round(v) for v in centers]
    end=y+200+len(events)*74
    p.box(64,y,2432,end-y+90,'#FAFBFD',LINE)
    p.text(88,y+18,title,26,color,bold=True)
    for j,(name,kind) in enumerate(actors):
        cx=centers[j]
        if kind=='empty':
            p.text(cx,y+112,name,18,MUTED,align='center');continue
        if kind=='component':p.component(f'{key}-actor{j}',cx-115,y+80,230,104,name,GREEN if name=='Screen Grounding\nService' else INK)
        else:p.node(f'{key}-actor{j}',cx-115,y+80,230,104,name,kind='external')
        p.line([(cx,y+184),(cx,end)],LINE,True,False,1)
    for row,(s,t,label,ret) in enumerate(events):
        yy=y+235+row*74;start,stop=centers[s],centers[t]
        ec=color if row+1 in unique_steps else INK
        if s==t:
            direction=-1 if s==len(actors)-1 else 1
            p.line([(start,yy),(start+45*direction,yy),(start+45*direction,yy+20),(start,yy+20)],ec,ret)
            p.label(start+55 if direction==1 else start-570,yy-25,label,ec,20)
        else:
            p.line([(start,yy),(stop,yy)],ec,ret)
            p.label(min(start,stop)+12,yy-29,label,ec,20)
    if note:p.text(88,end+25,note,19,MUTED)
    return end+130


def screen_input():
    p=Plate('stage4-s02-input-flow','S-02 / 공통 입력','화면 변화와 사용자 음성은 서로 다른 경로로 들어온다',
            'T와 A의 공통 입력. 위아래 흐름은 병행하며 각 흐름의 번호는 독립적이다. 모델은 연동 대상이고 원격 배치를 뜻하지 않는다.',2100)
    y=screen_sequence(p,'screen',240,'화면 수집 / 입력 활성 + 수집 권한이 있을 때',
        [('OS와 앱','external'),('Interaction Manager','component')],
        [(1,0,'1. 화면 변경과 사용자 조작 알림 구독',False),
         (0,1,'2. 변경 알림 또는 주기 수집 → 화면 이미지와 조작 정보 확보',False),
         (1,1,'3. 화면 기록 관리자가 발생 시각과 함께 보관',False)],
        '같은 화면은 참조 재사용. A는 새 기록이 준비되면 Screen Grounding Service에 알린다.')
    screen_sequence(p,'voice',y,'음성 입력 / 사용자가 말할 때',
        [('마이크','external'),('Interaction\nManager','component'),('Speech Input Worker\n인식 실행 경계','external'),('Streaming ASR\n인식 모델','external'),('Model Access','component'),('공유 Omni\n음성 처리','external')],
        [(0,1,'1. 음성 도착 → 발화 시작 감지, 관련 화면 기록 유지, Request Controller에 시작 알림',False),
         (1,2,'2. 연속 음성 전달',False),(2,3,'3. 음성 인식 요청',False),
         (3,2,'4. 인식 문장, 수정과 발화 시간 반환',True),
         (2,1,'5. 인식 근거 전달 → 입력 시각 연결기가 당시 화면과 연결',True),
         (1,4,'6. 같은 음성 참조 전달 / 2~5와 병행',False),(4,5,'7. 공유 Omni 음성 처리 / ASR 완료를 기다리지 않음',False)],
        '1의 발화 시작은 ASR 문장이 나온 뒤가 아니다. S2S 결과와 불일치 처리는 별도 응답 계약을 따른다.')
    p.footer('화면 수집은 발화 조각마다 시작되지 않는다. 관련 기록 유지와 시간 연결은 Interaction Manager의 서로 다른 내부 책임이다.')
    return p


def screen_target_flow():
    p=Plate('stage4-s02-target-flow','S-02 / T 실행','T: 당시 화면 근거를 조회하고 요청에 맞게 해석한다',
            '공통 입력 수집 뒤의 요청 처리 예. 발화 시작 때 준비한 기본 근거와 기존 UI 정보 및 cache는 유효한 범위에서 재사용한다.',1600)
    screen_sequence(p,'target',240,'요청 처리 / 대상 의미 해석이 필요한 입력',
        [('Interaction\nManager','component'),('별도 생산자 없음','empty'),('Context Manager','component'),('Request\nController','component'),('Request\nInterpreter','component'),('Model Access','component'),('공유 Omni\n의미 해석','external')],
        [(0,3,'1. 인식 문장, 입력 버전과 당시 화면 근거 참조 전달',False),
         (3,2,'2. 허용 근거 구성 요청',False),(2,0,'3. 발화 당시 화면과 조작 기록 조회',False),
         (0,2,'4. 보관된 이미지, UI 정보, 시간 오차와 누락 범위 반환',True),
         (2,3,'5. 필요한 현재 원문과 권한 확인 후 Context 반환',True),
         (3,4,'6. 입력과 Context로 해석 요청',False),(4,5,'7. 필요한 화면을 포함해 의미 해석 요청',False),
         (5,6,'8. 모델 입력 전달',False),(6,5,'9. 해석 결과 반환',True),(5,4,'10. 결과 반환',True),
         (4,3,'11. 대상과 의도 제안 또는 확인 필요성 반환',True),
         (3,3,'12. 현재 입력과 근거 검증 → 확정 또는 보류',False)],
        '부족한 근거는 허용된 추가 조회 또는 사용자 확인으로 보완한다. 이 흐름은 외부 업무 실행이 아니다.',BLUE,(3,4))
    p.footer('Context Manager의 현재 원문 조회는 해당 자료 소유자의 인터페이스를 사용한다. 최종 확정 주체는 Request Controller다.')
    return p


def screen_alternative_flow():
    p=Plate('stage4-s02-alternative-flow','S-02 / A 실행','A: 같은 화면 관측에서 추가 해석 자료를 생산한다',
            '생산 흐름과 요청 흐름은 병행한다. 준비된 정보가 없거나 불확실하면 원래 화면 기록을 조회한다. 두 흐름은 각각 1부터 읽는다.',2750)
    y=screen_sequence(p,'produce',240,'배경 생산 / Interaction Manager의 새 화면 기록 알림',
        [('Interaction Manager','component'),('Screen Grounding\nService','component'),('Model Access','component'),('공유 Omni\n화면 분석','external')],
        [(0,1,'1. 변경 알림과 허용된 화면 근거 참조 전달',False),
         (1,1,'2. 변경 범위, 현재 권한과 처리 예산 확인',False),
         (1,2,'3. 이미지 해석이 필요한 경우에만 분석 요청',False),
         (2,3,'4. 항목 위치, 종류와 변경 관계 후보 요청',False),
         (3,2,'5. 분석 후보 반환',True),(2,1,'6. 분석 결과 반환',True),
         (1,1,'7. 원래 화면과 캡처 시각 연결 → 검증 후 보관',False)],
        '앱의 구조 정보로 충분하면 모델 요청과 4~6을 생략한다. 발화 시작을 기다리지 않으며 단순 포인터 이동마다 모델을 호출하지 않는다.',GREEN,tuple(range(1,8)))
    screen_sequence(p,'query',y,'요청 처리 / 발화 당시 대상 정보가 필요한 입력',
        [('Interaction\nManager','component'),('Screen Grounding\nService','component'),('Context Manager','component'),('Request\nController','component'),('Request\nInterpreter','component'),('Model Access','component'),('공유 Omni\n의미 해석','external')],
        [(0,3,'1. 인식 문장, 입력 버전과 당시 근거 참조 전달',False),
         (3,2,'2. 허용 근거 구성 요청',False),(2,1,'3. 발화 시간과 창, 문서에 맞는 대상 정보 조회',False),
         (1,2,'4. 항목, 변경 관계, 분석 범위와 준비 상태 반환',True),
         (2,0,'5. 미준비, 불확실 또는 원래 이미지가 필요하면 당시 기록 조회',False),
         (0,2,'6. 보관 범위의 화면과 조작 기록 반환 / 5를 생략하면 함께 생략',True),
         (2,3,'7. 현재 원문과 권한 확인 후 Context 반환',True),
         (3,4,'8. 입력과 Context로 해석 요청',False),(4,5,'9. 발화의 대상 역할과 의도 해석 요청',False),
         (5,6,'10. 모델 입력 전달',False),(6,5,'11. 해석 결과 반환',True),(5,4,'12. 결과 반환',True),
         (4,3,'13. 의미 제안 또는 확인 필요성 반환',True),
         (3,3,'14. 현재 입력과 근거 검증 → 확정 또는 보류',False)],
        '추가 해석 자료는 사용자 의도를 확정하지 않는다. 생산 완료를 무한 대기하지 않으며 없는 과거는 확인 질문으로 처리한다.',GREEN,(3,4,5,6))
    p.footer('A는 보강 방향 미정인 후보로 유지한다. 조기 실행은 A만의 차이가 아니며, 추가 자료의 정확성과 시간 및 자원 이익은 미확인이다.')
    return p


def retrieval():
    p=plate(3,'자료 검색 정확성을 위한 자료 찾기 설계 — 이름과 키워드로 검색할 것인가, 내용의 의미로도 검색할 것인가?',
            'A는 T의 조회 경로를 유지하면서 검색과 색인 생산을 추가한다. 검색 결과는 후보이며 현재 원문의 확인을 대신하지 않는다.',2450)
    for i,x in enumerate((64,1312)):
        a=Area(p,i,x);c=BLUE if i==0 else GREEN
        p.panel(x,1184,'T' if i==0 else 'A','기존 자료의 이름과 내용으로 조회' if i==0 else '의미 검색 + 원본 조회 유지',
                '실제 target / 전용 의미 색인과 모델 없음' if i==0 else '탐색안 / 생산, 검색, 갱신과 삭제 경로 추가')
        a.b('via',0,335,1184,1515)
        a.c('rc',30,395,1124,167,'Request Controller',role='입력에서 필요한 자료와 허용 범위를 정한다')
        a.n('req',62,510,1060,34,'자료 요청 / 검증한 결과 수신',owner='rc')
        a.c('cm',30,670,1124,429,'Context Manager',role='후보 탐색, 원문 읽기와 현재 권한 확인의 공통 소유자')
        a.n('lookup',62,790,450,70,'이름 / 기간 / 단어 조회',owner='cm')
        a.n('cache',62,945,450,91,'기존 검색 색인과 캐시',kind='store',owner='cm')
        a.n('verify',690,790,432,70,'원문과 현재 권한 확인',owner='cm')
        a.n('read',690,945,432,70,'원본 조회 연동',owner='cm')
        a.e('req','lookup',via=[(592,622),(287,622)],label='① 자료 단서 전달' if i==0 else '자료 단서 전달',at=(68,593))
        a.e('verify','req','T','B',via=[(906,637),(592,637)],ret=True,label='⑤ 검증된 Context 반환' if i==0 else '⑩ 검증된 Context 반환',at=(702,599))
        a.e('lookup','cache',sd=-24,td=-24,label='② 기존 후보 조회' if i==0 else '기존 후보 조회',at=(78,880))
        a.e('cache','lookup','T','B',sd=24,td=24,ret=True)
        a.e('lookup','verify','R','L',label='후보',at=(547,790))
        a.e('verify','read',sd=-25,td=-25)
        a.e('read','verify','T','B',sd=25,td=25,ret=True)
        a.n('source',768,1900,386,87,'문서 / 대화 / 업무 원본',kind='external')
        a.e('read','source','R','T',via=[(1167,980),(1167,1875),(936,1875)],td=-25,label='③ 현재 원문 읽기' if i==0 else '⑧ 현재 원문 읽기',at=(908,1818))
        a.e('source','read','T','R',sd=25,td=20,via=[(986,1888),(1179,1888),(1179,1000)],ret=True,label='④ 원문과 버전 반환' if i==0 else '⑨ 원문과 버전 반환',at=(889,1877))
        if i==0:
            a.t(70,1210,['이 경로는 A에도 남는다.','별도 검색 서비스나 의미 색인을 유지하지 않는다.'],25)
            a.t(70,1360,['요청할 때 조회 → 원문 확인 → 사용','정확한 파일명이나 단어가 없으면','후보를 찾지 못하거나 다시 물을 수 있다.'],24,BLUE)
            a.t(70,1670,'색인 조회만으로 자료가 유일하다고 확정하지 않는다.',21,MUTED)
        else:
            a.b('searchsystem',30,1170,1124,651,'의미 검색 Subsystem','점선은 기능 묶음 / 별도 서버를 뜻하지 않음',GREEN,True)
            a.c('service',62,1238,466,395,'Semantic Retrieval Service',GREEN,role='요청을 검색 가능한 표현으로 바꿔 후보 반환')
            a.n('query',89,1355,412,60,'질문 표현 생성과 후보 검색',color=GREEN,owner='service')
            a.n('index',89,1494,412,102,'의미 색인 + 원본 버전 목록',kind='store',color=GREEN,owner='service')
            a.e('query','index',sd=-24,td=-24,color=GREEN,label='⑤ 조회',at=(110,1443))
            a.e('index','query','T','B',sd=24,td=24,color=GREEN,ret=True,label='⑥ 후보',at=(331,1450))
            a.c('worker',650,1238,472,395,'Indexing Worker',GREEN,role='원본 변경과 삭제를 추적하며 색인 갱신')
            a.n('extract',679,1355,414,60,'문서 분할과 표현 생성 요청',color=GREEN,owner='worker')
            a.n('publish',679,1494,414,65,'완성 색인 게시 / 삭제 반영',color=GREEN,owner='worker')
            a.e('publish','index','L','R',via=[(590,1526.5),(590,1545)],label='③ 게시',at=(547,1472),color=GREEN)
            a.c('runtime',62,1694,1060,110,'Embedding Runtime',GREEN,role='문서와 질문에 같은 모델 적용 / 생성 결과 반환')
            a.e('service','runtime',sd=-120,td=-370,via=[(175,1664),(222,1664)],color=GREEN)
            a.e('runtime','service','T','B',sd=-310,td=-60,via=[(282,1680),(235,1680)],ret=True,color=GREEN)
            a.e('worker','runtime',sd=-30,td=320,via=[(856,1651),(912,1651)],color=GREEN)
            a.e('runtime','worker','T','B',sd=370,td=30,via=[(962,1678),(916,1678)],ret=True,color=GREEN)
            a.e('lookup','service','L','L',sd=-12,td=-58,via=[(16,813),(16,1377.5)],color=GREEN,label='④ 의미 후보 검색',at=(68,1123))
            a.e('service','lookup','L','L',sd=-25,td=16,via=[(8,1410.5),(8,841)],ret=True,color=GREEN,label='⑦ 후보와 조회 범위 반환',at=(68,1146))
            a.e('source','worker','R','R',via=[(1160,1943.5),(1160,1435.5)],color=GREEN,label='① 원본 변경 수집',at=(775,1847))
            a.n('model',62,1900,530,87,'추가 의미 표현 모델 (Embedding)',kind='model',color=GREEN)
            a.e('runtime','model',sd=-290,td=-25,via=[(302,1858)],color=GREEN,label='② 의미 표현 생성',at=(83,1854))
            a.e('model','runtime','T','B',sd=25,td=-240,via=[(352,1879)],ret=True,color=GREEN)
        a.t(773,2000,'내부 owner와 외부 자료를 함께 펼친 참조',16,MUTED)
        p.focus(x,2060,1184,'원본 확인은 유지하고, 후보 생산 방법을 비교한다',[
            'T의 Context Manager 안에 있는 조회와 캐시는 A에도 남는다. 검색 기능이 없는 target이 아니다.',
            'A의 새 Subsystem은 검색 Component, 색인 생산 Component, 모델 연동 Component와 저장으로 구성된다.',
            '검색이 아직 준비되지 않으면 기존 조회 경로를 사용한다. 의미 검색과 같은 지원이라고 보지는 않는다.'],
            '이름을 모르는 자료의 발견 범위는 확인이 필요하다.' if i==0 else '추가 모델, 색인 갱신, 원본 삭제 전파와 손상 후 재구축 비용이 생긴다.',c)
    p.footer('추가 검색 경로는 초록, 그대로 유지하는 조회 경로는 검정이다. Subsystem 안의 각 Component는 독립된 책임을 표시한다.')
    return p


def workflow():
    p=plate(4,'요청 복구성을 위한 대기 및 재개 설계 — 요청 관계와 상태를 저장할 것인가, 실행 위치와 상태를 저장할 것인가?',
            '같은 예: “보고서가 끝나면 발표자료를 만들어줘.” 차이는 후속 조건의 소유자, 저장 내용과 복원 실행이다.',2750)
    layout_start=len(p.items)
    for i,x in enumerate((64,888,1712)):
        a=Area(p,i,x);c=BLUE if i==0 else GREEN
        p.panel(x,784,['T','A','B'][i],['요청 상태를 직접 저장','실행 절차와 대기를 저장','열린 세션에서만 연결'][i],
                ['실제 target / Request Controller가 재개 소유','탐색안 / 전용 Workflow Runtime이 재개 소유','제한 기능 탐색안 / 요청 관계 복원 포기'][i])
        a.b('via',0,335,784,1513)
        a.c('rc',26,401,732,407,'Request Controller',role='공통: 의미와 현재 권한 확인, 다음 요청 승인')
        a.n('admit',59,516,666,63,'입력과 답변 검증 / 후속 요청의 현재 조건 확인',owner='rc')
        if i==0:
            a.n('advance',59,630,666,64,'목표 관계 갱신 / 질문 대기 / 후속 조건 판단',color=BLUE,owner='rc')
            a.n('graph',59,723,666,61,'요청 관계 + 질문 상태',kind='data',color=BLUE,owner='rc')
            a.e('admit','advance',sd=-180,td=-180,color=BLUE,label='② 목표 등록',at=(68,588))
            a.e('admit','advance',sd=-30,td=-30,color=BLUE,label='④b 답변 반영',at=(239,603))
            a.e('advance','graph',sd=-18,td=-18,color=BLUE)
            a.e('graph','advance','T','B',sd=18,td=18,ret=True,color=BLUE)
            a.e('advance','admit','T','B',sd=20,td=20,color=BLUE,label='⑤ 후속 조건 충족',at=(430,588))
            a.t(49,940,['업무 결과가 도착하면','저장한 목표 관계의 조건을 검사하고','Request Controller가 후속 처리를 정한다.'],24,BLUE)
            a.t(49,1180,'별도 workflow 실행기는 없다.',23,BLUE)
        else:
            a.t(60,639,['대기 상태와 조건 진행은 아래에 위임한다.','최종 승인 책임은 여기 남는다.'],22,MUTED)
            a.c('runtime',26,910,732,379,'Interaction Workflow Runtime' if i==1 else 'Session Request Coordinator',GREEN,
                '대기 중 작업의 실행 위치와 다음 활동을 소유' if i==1 else '이번 실행 세션의 관계만 소유')
            a.n('signal',59,1026,303,64,'답변/결과 수신과 중복 제거' if i==1 else '답변/결과 연결',color=GREEN,owner='runtime')
            a.n('step',426,1026,299,64,'다음 활동 / 시간 만료 처리' if i==1 else '세션 안에서 후속 조건 판단',color=GREEN,owner='runtime')
            a.n('waiting',59,1164,666,91,'현재 실행 위치 + 대기 질문 + 전송할 활동' if i==1 else '요청 관계와 질문 / RAM만 사용',kind='data',color=GREEN,owner='runtime')
            a.e('signal','step','R','L',color=GREEN)
            a.e('step','waiting',via=[(575.5,1130),(392,1130)],color=GREEN)
            a.e('admit','runtime',sd=-40,td=-40,label='② 요청 시작',at=(93,852),color=GREEN)
            a.e('rc','signal','B','L',sd=-240,td=-20,via=[(152,878),(11,878),(11,1038)],color=GREEN,label='④b 검증한 답변 전달',at=(47,883))
            a.e('runtime','admit','R','R',via=[(772,1099.5),(772,547.5)],ret=True,color=GREEN,label='⑤ 후속 요청 승인 요청',at=(444,843))
        a.c('store',26,1400,732,228,'State Store',role='저장과 transaction 제공 / 요청 의미의 소유자는 아님')
        if i<2:
            a.n('record',59,1517,304,89,'현재 요청/질문 기록' if i==0 else '실행 위치/활동 기록',kind='store',color=c,owner='store')
        else:a.t(66,1533,['요청 관계와 질문은','저장하지 않음'],22,GREEN)
        a.n('taskrecord',426,1517,299,89,'업무/전송 원장',kind='store',owner='store')
        if i==0:
            a.e('advance','record','L','T',via=[(16,642),(16,1364),(211,1364)],sd=-20,label='③ 상태 기록',at=(49,1299),color=BLUE)
            a.e('record','advance','T','L',via=[(211,1382),(8,1382),(8,662)],ret=True,color=BLUE,label='⑦ 복원',at=(268,1348))
        elif i==1:
            a.e('waiting','record',sd=-181,td=0,label='③ 수신 + 전이 + 활동을 함께 기록',at=(49,1323),color=GREEN)
            a.e('record','signal','T','L',sd=20,via=[(231,1382),(16,1382),(16,1058)],ret=True,color=GREEN,label='⑦ 대기 상태 복원',at=(450,1368))
        else:a.t(49,1318,'재시작하면 위 RAM 상태를 잃는다.',22,GREEN)
        a.c('task',26,1694,347,115,'Task Manager',role='실제 업무 상태 소유')
        a.c('gateway',411,1694,347,115,'Agent Gateway',role='외부 업무 전송과 상태 수신')
        a.e('task','gateway','R','L',sd=-12,td=-12)
        a.e('gateway','task','L','R',sd=12,td=12,ret=True)
        a.e('task','taskrecord','T','B',sd=-28,via=[(171.5,1664),(575.5,1664)],label='⑥ 명령과 업무 원장',at=(267,1640))
        a.e('taskrecord','task','B','T',td=28,via=[(575.5,1680),(227.5,1680)],ret=True)
        a.e('admit','task','L','L',via=[(7,547.5),(7,1751.5)],label='⑥ 승인한 명령',at=(48,1638))
        recipient='advance' if i==0 else 'signal'
        # Task result returns to the owner of pending request relationships.
        if i==2:
            a.e('task','rc','L','L',sd=23,td=110,via=[(19,1774.5),(19,714.5)],ret=True,color=c,label='④c 업무 결과',at=(49,1351))
        else:
            a.e('task',recipient,'L','L',sd=23,via=[(19,1774.5),(19,662 if i==0 else 1058)],ret=True,color=c,label='④c 업무 결과',at=(49,1248 if i==0 else 1290))
        a.n('agent',411,1920,347,85,'외부 업무 Agent',kind='external')
        a.e('gateway','agent',sd=-20,td=-20,label='업무 실행 요청',at=(421,1852))
        a.e('agent','gateway','T','B',sd=20,td=20,ret=True)
        p.focus(x,2070,784,['재시작: 현재 요청 상태를 복원','재시작: 대기 중 실행을 복원','재시작: 기존 업무만 확인'][i],
                [['요청 관계와 질문 상태를 다시 읽음','외부 업무의 현재 결과를 확인','조건이 맞으면 후속 요청을 승인'],
                 ['현재 실행 위치와 대기 질문을 읽음','보내지 못한 활동과 실제 결과를 조정','Runtime이 진행하고 Controller가 승인'],
                 ['업무와 전송 원장은 계속 복원','보고서 다음 발표자료라는 관계는 유실','사용자가 후속 관계를 다시 지정']][i],
                ['audit log 전체를 재실행하는 구조가 아니다.','별도 내구 timer와 활동 전달 계약이 필요하다.','복구 기능의 포기이며 빠른 정상 완료가 아니다.'][i],c)
    for item in p.items[layout_start:]:
        if 'y' in item:item['y']+=300
        if 'points' in item:item['points']=[(x,y+300) for x,y in item['points']]
    p.nodes={k:(x,y+300,w,h) for k,(x,y,w,h) in p.nodes.items()}
    p.boundary('preparation',64,230,2432,287,'공통 입력과 의미 확인','새 목표와 나중에 도착한 답변은 서로 다른 사건')
    for key,x,name,role in [('im',110,'Interaction Manager','새 목표 또는 후속 답변 전달'),('rc',711,'Request Controller','입력, 근거와 현재 질문 확인'),('cm',1312,'Context Manager','허용된 근거와 질문 후보 조회'),('ri',1913,'Request Interpreter','목표 또는 답변 대상의 의미 제안')]:
        p.component('shared-'+key,x,321,440,110,name,role=role)
    p.edge('shared-im','shared-rc','R','L',label='사용자 입력',at=(558,350))
    p.edge('shared-rc','shared-cm','R','L',sd=-18,td=-18,label='기본 근거 요청',at=(1160,342))
    p.edge('shared-cm','shared-rc','L','R',sd=18,td=18,ret=True,label='근거 반환',at=(1160,409))
    p.edge('shared-rc','shared-ri','T','T',via=[(931,296),(2133,296)],label='입력과 근거로 해석 요청',at=(1370,266))
    p.edge('shared-ri','shared-rc','B','B',via=[(2133,470),(931,470)],ret=True,label='① 목표 제안 / ④a 후속 답변의 대상 제안',at=(1310,475))
    p.footer('업무 결과는 대기 상태 소유자로 돌아간다. VIA는 요청 사이 관계를 다루며 외부 Agent의 내부 실행 계획은 소유하지 않는다.')
    return p


def response():
    p=plate(5,'응답성을 위한 음성 응답 생성 설계 — 직접 음성 생성도 사용할 것인가, 확정된 문장만 음성으로 바꿀 것인가?',
            'A도 요청 이해와 음성 응답을 수행한다. 포기하는 것은 자체 지식 질문에 대한 직접 S2S 경로이며, 생성 순서가 달라진다.',2550)
    for i,x in enumerate((64,1312)):
        a=Area(p,i,x);c=BLUE if i==0 else GREEN
        p.panel(x,1184,'T' if i==0 else 'A','음성 선생성 + 좁은 직접 허용' if i==0 else '모든 요청 해석 → Text 승인 → 음성',
                '실제 target / 직접 경로와 일반 경로를 함께 유지' if i==0 else '제한 기능 탐색안 / 일반 경로로 응답, 직접 S2S는 없음')
        a.b('via',0,335,1184,1595)
        a.c('im',30,400,1124,311,'Interaction Manager',role='공통: 입력 수집, 표시, 재생과 즉시 중단')
        a.n('input',62,512,422,61,'사용자 입력 수집',owner='im')
        a.n('output',700,512,422,61,'승인된 Text 표시 / 음성 재생',owner='im')
        if i==0:a.n('held',700,620,422,66,'미리 만든 음성 + 보류 실행 ID',kind='data',color=BLUE,owner='im')
        else:a.t(66,633,'직접 음성을 미리 만들거나 보류하지 않는다.',22,GREEN)
        a.c('rc',30,830,1124,280,'Request Controller',role='공통: 요청의 의미와 현재 권한을 확인해 응답 허용')
        if i==0:a.n('direct',62,950,422,122,'이번 발화의 일반 지식 질문인가?\n맞으면 직접 허용 / 아니면 일반 해석',color=BLUE,owner='rc')
        else:a.t(64,967,['모든 입력을 아래 일반 해석으로 전달','직접 경로를 따로 판단하지 않음'],22,GREEN)
        a.n('admit',700,965,422,77,'응답 내용과 현재 입력 확인',owner='rc')
        a.e('input','rc',via=[(273,764),(592,764)],label='정규 입력 전달' if i==0 else '① 정규 입력 전달',at=(76,738))
        a.c('ri',30,1200,480,292,'Request Interpreter',role='T와 A 모두 같은 해석 책임')
        a.n('interpret',62,1313,416,64,'목표와 대상 해석',owner='ri')
        a.n('answer',62,1410,416,55,'응답 내용 제안',owner='ri')
        a.e('rc','ri',sd=-340,td=-18,via=[(252,1152)],label='④a 외부 자료 등이 필요할 때' if i==0 else '② 모든 요청',at=(50,1148),color=BLUE if i==0 else GREEN)
        a.e('ri','admit','R','B',sd=-40,via=[(568,1306),(568,1133),(911,1133)],label='④b 해석과 응답 제안 반환' if i==0 else '③ 해석과 응답 제안 반환',at=(585,1165),ret=True)
        a.c('rm',674,1200,480,380,'Response Manager',role='공통: 허용된 내용을 게시하고 음성 생성 연결')
        a.n('render',705,1313,418,65,'승인된 Text의 음성 생성 요청',owner='rm')
        a.n('release',705,1412,418,56,'생성된 음성 전달 / 실제 전달 확인',color=INK,owner='rm')
        if i==0:
            a.n('directrelease',705,1510,418,49,'⑥ 허용된 응답과 보류 실행 ID 확인',color=BLUE,owner='rm')
            a.e('directrelease','output','R','R',td=-15,via=[(1187,1534.5),(1187,527.5)],color=BLUE,label='⑦ 보류 해제 지시',at=(973,1604))
        else:a.t(712,1521,'직접 응답 해제 단계 없음',20,GREEN)
        a.e('admit','rm',sd=-22,td=-25,label='⑤ 응답 허용' if i==0 else '④ 응답 허용',at=(746,1129))
        a.e('rm','output','R','R',sd=-80,via=[(1169,1310),(1169,542.5)],label='⑧ Text 게시' if i==0 else '⑤ Text 먼저 게시',at=(841,759))
        a.e('release','output','R','R',td=10,via=[(1158,1440),(1158,552.5)],label='일반 응답: 검사한 음성' if i==0 else '⑧ 검사한 음성 전달',at=(940,1580))
        a.e('output','rm','R','R',sd=20,td=0,via=[(1179,562.5),(1179,1390)],ret=True,label='⑧ 실제 전달 기록',at=(938,1170))
        a.c('ma',30,1700,1124,200,'Model Access',role='공유 모델을 사용하되 역할별 실행을 구분')
        if i==0:a.n('voice',60,1820,282,55,'직접 음성 응답 생성',color=BLUE,owner='ma')
        else:a.t(62,1829,'직접 생성 경로 없음',20,GREEN)
        a.n('semantic',443,1820,280,55,'요청 해석 모델 호출',owner='ma')
        a.n('speech',810,1820,314,55,'Text → 음성 생성',owner='ma')
        a.e('ri','semantic','R','T',sd=65,via=[(549,1411),(549,1653),(559,1653)],td=-24,label='모델 해석 요청',at=(57,1590))
        a.e('semantic','ri','T','R',sd=24,td=83,via=[(607,1670),(587,1670),(587,1429)],ret=True)
        a.e('render','speech','L','T',td=-20,via=[(652,1345.5),(652,1637),(947,1637)],label='일반 응답: 승인된 Text' if i==0 else '⑥ 승인된 Text',at=(708,1598),color=INK if i==0 else GREEN)
        a.e('speech','release','T','R',sd=20,via=[(987,1669),(1139,1669),(1139,1440)],ret=True,label='일반 응답: 음성 결과' if i==0 else '⑦ 음성 결과',at=(961,1643),color=INK if i==0 else GREEN)
        if i==0:
            a.e('input','voice','L','L',via=[(12,542.5),(12,1847.5)],color=BLUE,label='① 원음으로 미리 생성',at=(49,1530))
            a.e('voice','held','B','R',via=[(201,1920),(1160,1920),(1160,653)],ret=True,color=BLUE,label='② 보류 음성 반환',at=(734,1622))
            a.e('held','direct','B','T',via=[(911,797),(273,797)],color=BLUE,label='③ 제안과 실행 ID',at=(523,794))
            a.e('direct','admit','R','L',via=[(590,1011),(590,1003.5)],color=BLUE,label='직접 허용',at=(521,958))
        a.n('model',307,1980,570,87,'공유 Omni 모델 / 가중치 한 벌',kind='model')
        a.e('ma','model',sd=-25,td=-25)
        a.e('model','ma','T','B',sd=25,td=25,ret=True)
        p.focus(x,2160,1184,'남는 기능, 이동하는 경로, 포기하는 기능',[
            'T: 자체 지식 질문의 음성은 먼저 준비할 수 있다. 허용 전에는 재생하지 않는다.' if i==0 else 'A: 음성 응답은 계속 제공한다. 모든 입력은 Request Interpreter를 거친다.',
            '일반 해석과 Text 기반 음성 생성은 양안 공통이다. A의 해석기에 S2S 기능이 옮겨진 것이 아니다.',
            'Text는 승인되면 음성 생성 완료를 기다리지 않고 게시할 수 있다. 실제 전달 기록은 별도다.'],
            '자료, 화면이나 이전 업무가 필요하면 T도 일반 해석 경로로 보낸다.' if i==0 else '직접 S2S 기능은 미지원. 입력 해석 뒤 음성을 생성하므로 실행 의존성과 지연이 달라진다.',c)
    p.footer('S2S는 음성 입력에서 직접 음성 답변을 만드는 경로다. 검정 해석 모듈은 공통이며 초록은 A의 요청 처리 경로다.')
    return p


def speech():
    p=plate(6,'음성 인식 정확성과 응답성을 위한 음성 입력 처리 설계 — 별도 인식 모델을 사용할 것인가, 공유 모델에 인식도 맡길 것인가?',
            'A의 인식 책임은 사라지지 않는다. Omni의 음성 실행과 새 결과 변환 모듈로 옮겨가며 같은 추론 자원과 장애에 의존한다.',2350)
    for i,x in enumerate((64,1312)):
        a=Area(p,i,x);c=BLUE if i==0 else GREEN
        p.panel(x,1184,'T' if i==0 else 'A','독립 ASR 인식 + 공유 Omni' if i==0 else '공유 Omni가 인식 근거까지 생산',
                '실제 target / 인식 실행과 모델을 따로 유지' if i==0 else '탐색안 / 전용 ASR 제거, 공유 추론에 인식 결합')
        a.b('via',0,335,1184,1391)
        a.c('im',30,401,1124,304,'Interaction Manager',role='모델과 무관하게 녹음과 재생 중단은 계속 수행')
        a.n('capture',62,515,414,63,'원음 수집 / 즉시 재생 중단',owner='im')
        a.n('normalize',690,515,432,63,'전사와 시간 근거를 입력에 연결',owner='im')
        a.n('record',690,614,432,65,'정규 입력 기록 + 누락 구간',kind='data',owner='im')
        a.e('normalize','record',label='⑤ 입력 기록' if i==0 else '③ 입력 기록',at=(713,587))
        a.c('ma',30,847,1124,825,'Model Access',role='VIA의 모델 연동 책임 / 아래 점선은 실제 실행 경계')
        a.b('asrprocess',54,969,456,632,'Speech Input Worker' if i==0 else '제거되는 인식 전용 실행',
            '독립 process' if i==0 else 'A에는 없음',BLUE if i==0 else LINE,True)
        a.b('omniprocess',625,969,505,632,'Shared Inference Service','공유 추론 process',INK,True)
        if i==0:
            a.n('asrqueue',82,1069,400,88,'인식 전용 입력 대기열',kind='store',color=BLUE,owner='ma')
            a.n('asr',82,1236,400,96,'Streaming ASR Adapter\n전사와 발화 시각 변환',color=BLUE,owner='ma')
            a.n('revision',82,1424,400,88,'인식 결과 버전과 누락 검사',color=BLUE,owner='ma')
            a.e('capture','asrqueue',sd=-35,td=-35,via=[(234,777),(247,777)],color=BLUE,label='① 같은 원음',at=(72,759))
            a.e('asrqueue','asr',color=BLUE)
            a.e('asr','revision',color=BLUE,label='전사와 시각',at=(84,1370))
            a.e('revision','normalize','L','L',via=[(17,1468),(17,746),(662,746),(662,546.5)],ret=True,color=BLUE,label='③ 인식 근거 반환',at=(333,713))
        else:
            a.t(85,1090,['Speech Input Worker 없음','인식 전용 대기열 없음','Streaming ASR 가중치 없음'],23,GREEN)
            a.t(85,1335,['인식은 오른쪽 공유 모델이 수행한다.','초록 모듈은 그 결과를 변환한다.','없는 시간 정보를 만들어내지는 않는다.'],20,GREEN)
        a.n('scheduler',654,1069,448,76,'공유 작업 대기열과 실행 배분',owner='ma')
        a.n('omni',654,1229,448,93,'Omni Adapter\n음성/의미 역할별 모델 호출',owner='ma')
        a.e('scheduler','omni',sd=-20,td=-20)
        a.e('omni','scheduler','T','B',sd=20,td=20,ret=True)
        a.e('capture','scheduler','R','T',via=[(590,546.5),(590,805),(878,805)],label='② 원음' if i==0 else '① 원음 / 인식도 같은 모델이 처리',at=(639,770),color=INK)
        if i==1:
            a.n('native',654,1440,448,100,'Native Evidence Adapter\n전사, 시각과 수정 알림 변환',color=GREEN,owner='ma')
            a.e('omni','native',color=GREEN,label='모델이 생산한 인식 결과',at=(654,1367))
            a.e('native','normalize','R','R',via=[(1170,1490),(1170,546.5)],ret=True,color=GREEN,label='② 인식 근거 반환',at=(893,713))
        else:
            a.t(665,1450,['Omni는 음성 응답과 의미 해석을 담당','ASR와 중요한 결과 충돌은 보존'],20,MUTED)
            a.e('omni','normalize','R','R',via=[(1170,1275.5),(1170,546.5)],ret=True,label='④ 해석 결과',at=(913,713))
        if i==0:
            a.n('asrmodel',54,1790,456,88,'Streaming ASR 모델',kind='model',color=BLUE)
            a.e('asr','asrmodel','L','T',via=[(65,1284),(65,1710),(257,1710)],td=-25,color=BLUE)
            a.e('asrmodel','asr','T','R',sd=25,via=[(307,1733),(493,1733),(493,1284)],ret=True,color=BLUE)
        a.n('omnimodel',625,1790,505,88,'공유 Omni 모델 / 한 벌',kind='model')
        a.e('omni','omnimodel','L','T',via=[(637,1275.5),(637,1709),(852.5,1709)],td=-25)
        a.e('omnimodel','omni','T','R',sd=25,via=[(902.5,1731),(1116,1731),(1116,1275.5)],ret=True)
        p.focus(x,1950,1184,'Omni가 멈췄을 때의 실제 차이',[
            'T: 녹음은 계속되고 독립 ASR는 인식 근거를 계속 만들 수 있다. 의미 해석과 음성 생성은 멈춘다.' if i==0 else 'A: 녹음은 계속되지만 인식, 의미 해석과 음성 생성이 함께 멈춘다.',
            '정규 입력 기록은 Interaction Manager 소유다. Request Controller가 입력 버전 변경을 최종 제어한다.',
            '모델은 VIA 밖의 책임이지만 PC 안에서 실행한다. Component 경계와 process 경계를 구별한다.'],
            '추가 모델과 인식 전용 CPU 비용. process 분리가 전력과 메모리 대역폭까지 격리하지는 않는다.' if i==0 else 'Omni의 전사, 발화 시각과 동시 인식 지원은 미확인. 녹음 지속을 인식 성공으로 보지 않는다.',c)
    p.footer('ASR는 음성을 글자로 바꾸는 인식이다. A는 모델 결과 변환을 추가하며 인식도 공통 실행 배분과 같은 모델에 의존한다.')
    return p


def semantic_detail():
    p=Plate('stage4-s01-execution-detail','S-01 / FLOW','읽기 전후에 호출이 끝나는가, 같은 작업이 계속되는가?',
            '위에서 아래로 사건 순서. 막대는 작업 수명이며 실제 시간이나 연산 점유량을 뜻하지 않는다.',2100)
    p.boundary('common',64,233,2432,275,'공통 준비 ①~⑤','같은 입력과 같은 기본 자료에서 시작')
    xs=[110,711,1312,1913]
    for j,name in enumerate(['Interaction Manager','Request Controller','Context Manager']):
        p.component(f'c{j}',xs[j],304,440,110,name,role=['최종 입력 전달','자료 준비 요청','허용된 원본 읽기'][j])
    p.node('c3',xs[3],304,440,110,'원본 소유자',kind='external')
    for j,label in enumerate(['① 입력','② 자료 요청','③ 원본 읽기']):p.edge(f'c{j}',f'c{j+1}','R','L',sd=-15,td=-15,label=label,at=(xs[j]+451,300))
    p.edge('c3','c2','L','R',sd=16,td=16,ret=True,label='④ 원문과 버전',at=(xs[2]+450,427))
    p.edge('c2','c1','L','R',sd=16,td=16,ret=True,label='⑤ 기본 자료',at=(xs[1]+450,427))
    for i,x in enumerate((64,1312)):
        c=BLUE if i==0 else GREEN
        p.box(x,552,1184,1276,'#FAFBFD',LINE)
        p.text(x+22,573,'T / 첫 호출 종료 → 자료를 넣어 새 호출' if i==0 else 'A / 작업 유지 → 읽기 → 같은 실행 재개',26,c,bold=True)
        names=['Request\nController','Request\nInterpreter',None,'Context\nManager','원본 소유자','Model\nAccess'] if i==0 else ['Request\nController','Semantic\nResolution Worker','Capability\nRead Broker','Context\nManager','원본 소유자','Model\nAccess']
        centers=[x+16+192*(j+.5) for j in range(6)]
        for j,(cx,name) in enumerate(zip(centers,names)):
            if name is None:p.text(cx,674,'별도 Broker 없음',16,MUTED,align='center');continue
            if j==4:p.node(f'{i}-lane{j}',cx-91,648,182,106,name,kind='external')
            else:p.component(f'{i}-lane{j}',cx-95,648,190,106,name,c if j==1 or i==1 and j==2 else INK)
            p.line([(cx,754),(cx,1710)],LINE,True,False,1.2)
        p.box(centers[1]-5,820,10,225 if i==0 else 825,'white',c,0)
        if i==0:p.box(centers[1]-5,1420,10,225,'white',c,0)
        events=[(0,1,'⑥ 첫 번째 해석 호출',False),(1,5,'⑦ 모델 호출',False),(5,1,'⑧ 해석 결과',True),
                (1,0,'⑨ 자료 부족 제안 / 호출 종료',True),(0,3,'⑩ 허용된 추가 읽기',False),
                (3,4,'⑪ 원본 조회',False),(4,3,'⑫ 원문과 버전',True),(3,0,'⑬ 추가 자료와 읽은 버전',True),
                (0,1,'⑭ 추가 자료를 넣어 새 호출',False),(1,5,'⑮ 모델 호출',False),(5,1,'⑯ 해석 결과',True),
                (1,0,'⑰ 최종 의미 제안',True)] if i==0 else [
                (0,1,'⑥ 요청별 작업 시작',False),(1,5,'⑦ 해석 시작',False),(5,1,'⑧ 읽기 필요 + 멈춘 실행 ID',True),
                (1,2,'⑨ 추가 자료 요청',False),(2,3,'⑩ 범위와 예산 검사 후 조회',False),
                (3,4,'⑪ 원본 조회',False),(4,3,'⑫ 원문과 버전',True),(3,2,'⑬ 추가 자료와 읽은 버전',True),
                (2,1,'⑭ 자료 반환',True),(1,5,'⑮ 같은 실행 ID로 이어가기',False),(5,1,'⑯ 최종 해석 결과',True),
                (1,0,'⑰ 최종 의미 제안',True)]
        for row,(s,t,label,ret) in enumerate(events):
            y=820+row*75;start,end=centers[s],centers[t]
            if s==t:
                p.line([(start,y),(start+58,y),(start+58,y+20),(start,y+20)],INK)
                p.label(start-120,y-30,label,INK,17)
            else:
                ec=c if row in (3,4,7,8) else INK
                p.line([(start,y),(end,y)],ec,ret)
                p.label(min(start,end)+8,y-31,label,ec,17)
        p.text(x+31,1748,'⑱ Request Controller가 입력 버전, 사용한 자료와 필수 항목을 확인해 의미 확정',20,bold=True)
    p.text(64,1880,'원본 소유자는 내부 Component 또는 허용된 외부 source다. Context Manager가 조회하고 원본 소유자가 반환한다.',20,MUTED)
    p.footer('A는 모델 중단과 재개 기능이 실제로 필요하다. 미지원이면 단순 새 호출 구조로 바뀌므로 같은 대안으로 평가할 수 없다.')
    return p


BUILDERS=[semantic,screen,screen_input,screen_target_flow,screen_alternative_flow,retrieval,workflow,response,speech,semantic_detail]


def main():
    ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args()
    pages=[b() for b in BUILDERS];errors=[]
    for p in pages:
        p.validate()
        for ext,data in [('svg',p.svg()),('drawio',p.drawio())]:
            root=ET.fromstring(data)
            if ext=='drawio':
                cells=root.findall('.//mxCell');ids=[c.get('id') for c in cells]
                assert len(ids)==len(set(ids))
                for cell in cells:
                    for attr in ('parent','source','target'):assert cell.get(attr) is None or cell.get(attr) in ids
            path=OUT/f'{p.slug}.{ext}'
            if args.check:
                if not path.exists() or path.read_text()!=data:errors.append(path.name)
            else:path.write_text(data)
    sections='\n'.join(f'<section><h2>{html.escape(p.title)}</h2><img src="{p.slug}.svg" alt="{html.escape(p.title)}"><p><a href="{p.slug}.svg">SVG 크게 보기</a> / <a href="{p.slug}.drawio">draw.io 편집 원본</a></p></section>' for p in pages)
    gallery='<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA Architecture Review</title><style>body{margin:0;background:#eef2f7;color:#263445;font-family:Arial,sans-serif}header{padding:32px 5%;background:#263445;color:white}section{max-width:1800px;margin:36px auto;background:white;padding:28px;border:1px solid #c7d1dd;border-radius:10px}img{width:100%;height:auto}h2{font-size:20px}a{color:#195fbd}</style><header><h1>VIA / Architecture Review</h1><p>공통: 검정 / T 전용: 파랑 / A와 B 전용: 초록 / 책임, 실행과 저장 경로 비교</p></header>'+sections+'</html>\n'
    path=OUT/'stage4-review.html'
    if args.check:
        if not path.exists() or path.read_text()!=gallery:errors.append(path.name)
    else:path.write_text(gallery)
    if errors:raise SystemExit('Diagram drift: '+', '.join(errors))
    print(f'PASS: {len(pages)} pairs; XML, ownership nesting, IDs, bounds, routes and source parity')


if __name__=='__main__':main()
