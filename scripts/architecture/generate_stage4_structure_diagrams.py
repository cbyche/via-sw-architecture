#!/usr/bin/env python3
"""Generate architecture review plates; no model or candidate execution."""
from __future__ import annotations
import argparse
import html
from pathlib import Path
import xml.etree.ElementTree as ET
from stage4_diagram_design import Plate, INK, BLUE, MUTED, LINE, PALE, AMBER

ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'docs/architecture/12-decisions/decision-packages/diagrams'


class Area:
    def __init__(self,p,i,x):self.p,self.i,self.x=p,i,x
    def node(self,key,x,y,w,h,name,**kw):return self.p.node(f'{self.i}-{key}',self.x+x,y,w,h,name,**kw)
    def edge(self,s,t,sp='B',tp='T',via=(),label='',at=None,**kw):
        return self.p.edge(f'{self.i}-{s}',f'{self.i}-{t}',sp,tp,[(self.x+x,y) for x,y in via],label,(self.x+at[0],at[1]) if at else None,**kw)
    def boundary(self,key,x,y,w,h,*args,**kw):return self.p.boundary(f'{self.i}-{key}',self.x+x,y,w,h,*args,**kw)
    def text(self,x,y,*args,**kw):return self.p.text(self.x+x,y,*args,**kw)


def plate(n,title,thesis):return Plate(f'stage4-s{n:02}-comparison',f'S-{n:02}',title,thesis)


def semantic():
    p=plate(1,'추가 근거를 읽는 동안, 해석 실행의 수명은 누가 소유하는가?',
            '같은 기본 Context에서 시작한다. 비교점은 검색 방법이 아니라, 호출을 종료하고 다시 만들지 또는 같은 실행을 재개할지다.')
    for i,x in enumerate((64,1312)):
        a=Area(p,i,x)
        p.panel(x,1184,'T' if i==0 else 'A','호출 사이에서 제어' if i==0 else '하나의 실행 안에서 재개','실제 reviewed target' if i==0 else '탐색안 / 중단 및 재개 계약 추가')
        a.boundary('via',0,335,1184,640,'VIA SOFTWARE / 비교 영역')
        a.text(48,1160,'원본 소유자 펼침 / VIA 내부 owner도 포함',17,MUTED)
        a.node('im',48,405,344,84,'Interaction Manager')
        a.node('rc',688,405,416,84,'Request Controller')
        a.node('cm',48,650,344,90,'Context Manager')
        a.node('exec',688,650,416,90,'Request Interpreter' if i==0 else 'Semantic Resolution\nWorker',focus=True,kind='component' if i==0 else 'module')
        a.node('ma',688,856,416,82,'Model Access')
        a.edge('im','rc','R','L',label='① 최종 입력과 근거 참조',at=(427,413))
        a.edge('rc','cm','B','T',sd=-120,via=[(776,554),(220,554)],label='② 기본 Context 요청',at=(265,522))
        a.edge('cm','rc','R','B',via=[(578,695),(578,533),(896,533)],label='⑤ Context + receipt',at=(602,545),ret=True)
        a.edge('rc','exec',label='⑥ / ⑭ 해석 호출' if i==0 else '⑥ 요청별 실행 시작',at=(906,582),focus=True)
        a.edge('exec','rc','R','R',via=[(1140,695),(1140,447)],label='⑨ / ⑰ 제안 반환' if i==0 else '⑰ 최종 제안 반환',at=(910,510),focus=True,ret=True)
        a.edge('exec','ma',sd=-36,td=-36,label='⑦ / ⑮ 모델 요청',at=(745,785))
        a.edge('ma','exec','T','B',sd=36,td=36,ret=True)
        a.node('source',48,1055,344,90,'원본 소유자의 읽기 접점',kind='data',tag='내부 OWNER / 외부 SOURCE')
        a.node('model',688,1055,416,90,'Shared Omni Model',kind='model')
        a.edge('cm','source',sd=-28,td=-28,label='③ / ⑪ 원본 읽기',at=(64,803))
        a.edge('source','cm','T','B',sd=28,td=28,ret=True,label='④ / ⑫ 원문 반환',at=(250,896))
        a.edge('ma','model',sd=-28,td=-28)
        a.edge('model','ma','T','B',sd=28,td=28,ret=True)
        a.text(688,1001,'모델 내부는 VIA 책임 밖 / on-device, weights 한 벌',16,MUTED)
        if i==0:
            a.edge('rc','cm','B','L',sd=-152,via=[(744,590),(22,590),(22,695)],focus=True,label='⑩ 추가 읽기',at=(45,604))
            p.focus(x,1200,1184,'종료 → 보완 → 새 호출',[
                '해석 호출 1 종료 → Request Controller가 추가 읽기 조정',
                '추가 Context 결합 → 해석 호출 2 시작',
                '⑱ Request Controller가 현재 revision과 receipt로 최종 확정'],
                '별도 호출이어도 기존 session이나 prefix의 재사용을 금지하는 구조는 아니다.')
        else:
            a.node('broker',426,773,236,86,'Capability\nRead Broker',kind='module',focus=True)
            a.edge('exec','broker','L','R',via=[(675,695),(675,816)],focus=True,label='⑨ 읽기 요청',at=(436,744))
            a.edge('broker','cm','L','R',via=[(409,816),(409,707)],td=12,focus=True)
            a.edge('cm','broker','R','L',sd=-12,via=[(400,683),(400,800)],td=-16,focus=True,ret=True)
            a.edge('broker','exec','R','L',sd=16,td=16,via=[(682,832),(682,711)],focus=True,ret=True)
            a.text(422,883,'⑩ 위임 / ⑬~⑭ 근거 반환',17,BLUE)
            p.focus(x,1200,1184,'실행 수명 유지 → 읽기 동안 연산 양보 → 재개',[
                'Semantic Resolution Worker가 재개 handle과 임시 read set 보유',
                'Capability Read Broker가 권한과 예산을 검사해 읽기 위임',
                '⑱ 최종 확정 권한은 계속 Request Controller에 있음'],
                'READ_REQUIRED와 같은 실행의 재개 지원은 미확인. 대기 중 KV 수명 관리가 추가된다.')
    p.footer('①~⑤ 기본 준비는 공통. 전체 ①~⑱ 왕복 순서와 호출 수명은 다음 실행 상세도에서 확대한다.')
    return p


def screen():
    p=plate(2,'화면 근거를 언제, 어떤 형태로 만들어 둘 것인가?',
            '원시 관측, 사전 객체 이력, 사용자 선택 package. 같은 지칭 문제에서 근거의 생산 시점과 지원 범위가 달라진다.')
    for i,x in enumerate((64,888,1712)):
        a=Area(p,i,x)
        p.panel(x,784,['T','A','B'][i],['원시 timeline 유지','객체 이력 사전 생산','명시 선택만 지원'][i],
                ['실제 reviewed target','탐색안 / producer와 파생 저장 추가','제한 기능 탐색안 / 연속 지칭 포기'][i])
        a.node('source',204,334,376,72,'OS / 화면 / 사용자 선택',kind='data',tag='INPUT SOURCE')
        a.boundary('via',0,449,784,616)
        a.node('im',204,506,376,82,'Interaction Manager')
        a.edge('source','im',label='① 관측 사건 수집' if i<2 else '① 사용자가 먼저 선택',at=(415,415),focus=i==2)
        if i==1:
            a.node('producer',204,638,376,86,'Screen Grounding Service',kind='module',focus=True)
            a.node('store',204,792,376,86,'Derived Object Store',kind='store',focus=True)
            a.edge('im','producer',label='② 객체와 관계 후보 생산',at=(413,596),focus=True)
            a.edge('producer','store',label='③ version별 객체 이력 게시',at=(412,741),focus=True)
        else:
            a.node('store',204,710,376,100,'Raw Screen Timeline' if i==0 else 'Capture Package',kind='store',focus=True)
            a.edge('im','store',label='② 관측 구간 보존' if i==0 else '② 선택 시점 자료 봉인',at=(413,638),focus=True)
            a.text(33,840,'유한 RAM / pre-roll / gap' if i==0 else '선택 밖의 과거 관측은 없음',18,MUTED)
        a.node('cm',204,960,376,82,'Context Manager')
        a.edge('cm','store','T','B',sd=-24,td=-24,focus=True)
        a.edge('store','cm','B','T',sd=24,td=24,ret=True,focus=True,label='③ 당시 근거 반환' if i!=1 else '④ 객체 근거 반환',at=(425,912))
        if i==1:
            a.node('ma',22,638,154,86,'Model\nAccess',tag='연동 PORT')
            a.edge('producer','ma','L','R',sd=-13,td=-13,focus=True)
            a.edge('ma','producer','R','L',sd=13,td=13,ret=True,focus=True)
            a.text(24,1100,['조건부 의존: UI 구조 정보가 부족할 때','Model Access → 공유 Omni vision session'],19,MUTED)
        else:a.text(24,1100,['공통 후속: 현재 원문과 권한 확인','Context Manager → Request Controller'],19,MUTED)
        p.focus(x,1200,784,['관측을 남기고 요청 때 결합','요청 전 계산을 별도 생산자로 이동','생산 비용과 사용자 선택의 교환'][i],
                [['발화 구간: 원시 화면과 선택 사건 보존','늦은 전사: 당시 관측에 시간으로 결합','현재 적용: source revision 재검사'],
                 ['객체 후보: 생산 → version 게시 → 소비','원시 fallback과 객체 이력을 함께 보존','현재 적용: 객체 ID와 원본 identity 대조'],
                 ['선택: identity + revision + 시각 + crop','소비: 해당 package 범위에서만 해석','재선택과 수동 연결은 사용자의 추가 작업']][i],
                ['비용: raw timeline 메모리와 요청 시 해석','비용: 상시 producer, 모델 연산과 철회 전파','손실: 과거 복수 지칭과 선택 밖 연속 지칭'][i])
    p.footer('저장소는 모두 논리 자료 구조다. 원시 화면의 무제한 저장이나 객체 ID의 영구 유효성을 가정하지 않는다.')
    return p


def retrieval():
    p=plate(3,'자료를 찾을 때 읽을 것인가, 의미 색인을 미리 생산할 것인가?',
            'Context Manager의 원문 확인 권한은 유지한다. 대안은 의미 후보를 생산하고 갱신하는 별도 서브시스템을 추가한다.')
    for i,x in enumerate((64,1312)):
        a=Area(p,i,x)
        p.panel(x,1184,'T' if i==0 else 'A','요청 시 owner 조회' if i==0 else '지속 의미 색인 + 요청 시 검증',
                '실제 reviewed target / metadata와 keyword 검색 있음' if i==0 else '탐색안 / 발견 기능과 사전 생산 비용의 교환')
        a.boundary('via',0,335,1184,670,'VIA SOFTWARE / 비교 영역')
        a.text(735,1158,'원본 소유자 펼침 / VIA 내부 owner도 포함',17,MUTED)
        a.node('rc',48,398,365,82,'Request Controller')
        a.node('cm',735,398,400,82,'Context Manager')
        a.edge('rc','cm','R','L',sd=-12,td=-12,label='허용 scope로 Context 요청',at=(448,392))
        a.edge('cm','rc','L','R',sd=14,td=14,ret=True,label='검증한 원문 + receipt',at=(471,469))
        a.node('read',735,625,400,88,'Owner Read Adapter',kind='module')
        a.edge('cm','read',sd=-24,td=-24,label='현재 원문 읽기',at=(956,545))
        a.edge('read','cm','T','B',sd=24,td=24,ret=True)
        a.node('source',735,1056,400,88,'문서 / 대화 / Task 원본',kind='data',tag='내부 OWNER / 외부 SOURCE')
        a.edge('read','source',sd=-25,td=-25,label='revision과 권한 재확인',at=(766,900))
        a.edge('source','read','T','B',sd=25,td=25,ret=True)
        if i==0:
            a.boundary('focus',32,546,626,386,'요청 기반 후보 발견','Context Manager 내부',True)
            a.node('lookup',74,610,534,90,'Metadata / Keyword Lookup',kind='module',focus=True)
            a.node('cache',74,798,534,88,'Revision Cache / 기존 Index',kind='store')
            a.edge('cm','lookup','L','R',via=[(687,439),(687,655)],focus=True,label='① 이름, 기간, Task 단서',at=(440,512))
            a.edge('lookup','cache',sd=-25,td=-25,label='② 기존 후보 조회',at=(362,745))
            a.edge('cache','lookup','T','B',sd=25,td=25,ret=True)
            a.edge('lookup','read','R','L',via=[(680,655),(680,669)],focus=True)
            a.text(48,1070,['별도 embedding model 없음','별도 의미 색인 generation 없음'],21,MUTED)
        else:
            a.boundary('focus',32,546,626,446,'의미 후보 생산 서브시스템','추가',True)
            a.node('service',73,600,535,84,'Semantic Retrieval Service',kind='module',focus=True)
            a.node('index',74,791,267,90,'Vector Index\n+ Source Manifest',kind='store',focus=True)
            a.node('worker',382,791,225,90,'Indexing\nWorker',kind='module',focus=True)
            a.edge('cm','service','L','R',via=[(684,439),(684,630)],td=-12,focus=True,label='④ 검색 / ⑦ 후보 반환',at=(436,512))
            a.edge('service','cm','R','L',sd=12,via=[(704,654),(704,455)],td=16,focus=True,ret=True)
            a.edge('service','index','B','T',via=[(340.5,723),(207.5,723)],focus=True,label='⑤ 조회',at=(73,720))
            a.edge('index','service','T','B',sd=34,td=34,via=[(241.5,757),(374.5,757)],focus=True,ret=True,label='⑥ 후보',at=(407,717))
            a.edge('worker','index','L','R',focus=True)
            a.text(322,768,'③ 게시',16,BLUE)
            a.edge('source','worker','L','R',via=[(690,1100),(690,836)],focus=True,label='① 변경과 삭제 수집',at=(460,1015))
            a.node('runtime',382,921,225,60,'Embedding Runtime',kind='module',focus=True)
            a.node('model',50,1056,540,88,'Embedding Model',kind='model',focus=True)
            a.edge('worker','runtime',sd=-16,td=-16,focus=True)
            a.edge('runtime','worker','T','B',sd=16,td=16,focus=True,ret=True)
            a.edge('runtime','model','L','T',via=[(292,951)],td=-28,focus=True)
            a.edge('model','runtime','T','L',sd=28,td=16,via=[(348,967)],focus=True,ret=True)
            a.text(44,952,'② embedding 생성',17,BLUE)
        p.focus(x,1200,1184,'원문 검증' if i==0 else '색인 수명과 권위 분리',
                ['후보 발견 → 원본 읽기 → Context Manager의 identity와 권한 확인',
                 '기존 index와 cache가 있으므로 “검색 없음”과 비교하는 것이 아니다.',
                 '자료가 없다는 판정과 검색 범위 부족을 구별한다.'] if i==0 else
                ['Indexing Worker → Embedding Runtime → 완성 generation 원자 게시',
                 '검색 후보는 권위 원본이 아니다. ⑧~⑩ 현재 원문을 다시 읽어 검증한다.',
                 '삭제와 철회는 검색, 늦은 색인 작업과 파생 저장까지 전파한다.'],
                '표현 차이에 따른 누락 가능성. 후보 검색과 원문 검증 비용을 함께 본다.' if i==0 else '추가 helper의 weights, 갱신과 재구축 비용. cold start에는 기존 조회 경로를 유지한다.')
    p.footer('확대 영역은 논리 서브시스템이다. 별도 서버 배포를 뜻하지 않으며, 검색 결과 1위도 실행 권한이 아니다.')
    return p


def workflow():
    p=plate(4,'대기 중인 목표와 질문의 권위 원본을 어디에 둘 것인가?',
            '요청의 의미 확정과 외부 업무 실행은 공통이다. 비교하는 것은 목표 관계, 질문 대기와 재개를 소유하는 상태다.')
    for i,x in enumerate((64,888,1712)):
        a=Area(p,i,x)
        p.panel(x,784,['T','A','B'][i],['Domain 상태에서 재개','Workflow에서 재개','세션 안에서만 연결'][i],
                ['실제 reviewed target','탐색안 / continuation이 권위 원본','제한 기능 탐색안 / graph 복원 포기'][i])
        a.boundary('via',0,335,784,753)
        a.node('rc',183,402,418,84,'Request Controller')
        a.text(185,371,'① 의미가 확정된 목표',16,MUTED)
        if i==0:
            a.boundary('owner',111,556,562,240,'대기 상태의 소유자','Request Controller 내부',True)
            a.node('graph',140,614,504,62,'Request Graph',kind='data',focus=True)
            a.node('question',140,704,504,62,'Pending User Interaction',kind='data',focus=True)
            a.edge('rc','graph',label='② 목표 / ④ 답변과 결과',at=(412,511),focus=True)
        else:
            a.boundary('owner',111,556,562,240,'대기 상태의 소유자','내구' if i==1 else 'RAM ONLY',True)
            a.node('graph',140,614,504,100,'Interaction Workflow Runtime' if i==1 else 'Session Request Coordinator',kind='module',focus=True)
            a.text(144,738,'Signal Inbox / Timer / Activity Dispatcher' if i==1 else '목표 관계 + 질문 focus / 세션 수명',18,BLUE)
            a.edge('rc','graph',label='② 시작 / ④ signal' if i==1 else '② 관계 / ③ 답변과 결과',at=(413,511),focus=True)
            a.edge('graph','rc','R','R',via=[(710,664),(710,444)],focus=True,ret=True,label='⑤ admission 요청' if i==1 else '④ 후속 제안',at=(530,531))
        a.node('store',32,893,325,91,'State Store',kind='store')
        a.node('task',433,893,319,91,'Task Manager')
        a.edge('task','store','L','R',sd=-12,td=-12,label='원장',at=(365,902))
        a.edge('store','task','R','L',sd=14,td=14,ret=True)
        a.edge('rc','task','R','T',via=[(758,444),(758,848),(592.5,848)],label='⑥ 현재 admission',at=(464,835))
        if i==1:
            a.edge('task','graph','R','R',via=[(774,938.5),(774,685)],td=21,ret=True,focus=True)
        else:
            a.edge('task','rc','R','T',via=[(774,938.5),(774,388),(392,388)],ret=True)
        if i<2:
            p.line([(x+320,796),(x+320,840),(x+169,840),(x+169,893)],BLUE)
            p.line([(x+217,893),(x+217,864),(x+360,864),(x+360,796)],BLUE,True)
            a.text(30,806,'③ 기록 / ⑦ 복원',17,BLUE)
        else:a.text(36,812,['graph 저장 없음','Task / command 원장은 유지'],18,AMBER)
        a.node('gateway',433,1010,319,65,'Agent Gateway')
        a.edge('task','gateway',sd=-18,td=-18)
        a.edge('gateway','task','T','B',sd=18,td=18,ret=True)
        a.node('agent',433,1120,319,66,'Downstream Agent',kind='module',tag='EXTERNAL EXECUTION')
        a.edge('gateway','agent',sd=-18,td=-18)
        a.edge('agent','gateway','T','B',sd=18,td=18,ret=True)
        a.text(25,1109,['④ 답변과 Task 결과를 현재 대기에 결합' if i<2 else '③ 답변과 결과를 세션 관계에 결합','외부 업무 내부 계획은 Agent 책임'],17,MUTED)
        p.focus(x,1230,784,['복구 원본: current records','복구 원본: continuation','복구 범위의 명시적 축소'][i],
                [['graph와 질문은 Request Controller 소유','State Store는 transaction과 보관 담당','재시작 → graph 복원 → source 확인'],
                 ['signal 수락 + 전이 + activity intent','같은 State Store transaction에 기록','재시작 → activity와 실제 source 조정'],
                 ['기존 Task와 전송 원장은 복구','미완료 목표 관계와 질문은 복원하지 않음','사용자가 후속 관계를 다시 지정']][i],
                ['audit log를 실행 원본으로 replay하지 않는다.','workflow 성공이 외부 실행 성공은 아니다.','자동 재개 손실을 빠른 완료로 세지 않는다.'][i])
    p.footer('대기 상태는 Component가 소유한다. State Store나 모델의 KV가 그 의미 권위를 대신하지 않는다.')
    return p


def response():
    p=plate(5,'자체 지식 응답에 S2S 직접 경로를 남길 것인가?',
            'T는 좁은 직접 경로와 Core 경로를 함께 갖는다. A는 모든 요청을 Core에서 해석하고 승인된 Text로 음성을 만든다.')
    for i,x in enumerate((64,1312)):
        a=Area(p,i,x)
        p.panel(x,1184,'T' if i==0 else 'A','직접 경로 + Core 경로' if i==0 else 'Core 공통 경로 + SpeechRender',
                '실제 reviewed target / current-Turn-only 직접 admission' if i==0 else '제한 기능 탐색안 / S2S 직접 기능 미지원')
        a.boundary('via',0,335,1184,682)
        a.node('im',48,402,350,86,'Interaction Manager')
        a.node('rc',728,402,405,86,'Request Controller')
        a.node('ri',728,630,405,86,'Request Interpreter',focus=i==1)
        a.node('rm',728,866,405,86,'Response Manager')
        a.node('ma',48,866,350,86,'Model Access')
        a.edge('im','rc','R','L',label='③ 정규 입력 + VoiceProposal' if i==0 else '① 정규 입력',at=(430,407))
        a.edge('rc','ri',sd=-26,td=-26,focus=i==1,label='④a Core 필요 시' if i==0 else '② 모든 요청 해석',at=(762,534))
        a.edge('ri','rc','T','B',sd=26,td=26,ret=True,label='④b 제안 반환' if i==0 else '③ 제안 반환',at=(951,585))
        a.edge('rc','rm','R','R',via=[(1150,445),(1150,909)],label='⑤ 게시 허용' if i==0 else '④ 게시 허용',at=(980,788))
        a.edge('rm','im','L','R',via=[(592,909),(592,458)],td=13,label='⑥~⑧ release\n게시와 재생' if i==0 else '⑤ Text\n⑧ Voice',at=(604,751))
        a.node('omni',48,1068,350,84,'Shared Omni Model',kind='model')
        a.edge('ma','omni',sd=-22,td=-22)
        a.edge('omni','ma','T','B',sd=22,td=22,ret=True)
        a.text(446,1076,['공유 weights 한 벌 / Voice와 semantic session 분리','Interaction Manager가 실제 표시, 재생과 local stop 소유'],20,MUTED)
        if i==0:
            a.boundary('direct',25,562,502,236,'직접 경로 확대','Model Access client',True)
            a.node('handle',53,624,440,99,'VoiceProposal / Generation Handle',kind='data',focus=True)
            a.edge('im','ma','L','L',via=[(16,445),(16,909)],focus=True,label='① 원음',at=(30,810))
            a.edge('ma','im','R','R',sd=-20,td=-12,via=[(548,889),(548,433)],ret=True,focus=True,label='② 제안 + handle',at=(401,505))
            p.line([(x+223,488),(x+223,624)],BLUE,True,False)
            a.text(55,746,'입력 revision에 묶인 보류 자료 / 추가 실행체 아님',17,BLUE)
        else:
            p.box(x+25,562,502,236,'#FAFBFD',LINE,8,True)
            a.text(55,589,'제거되는 경로',18,MUTED,bold=True)
            a.text(55,634,['VoiceProposal 직접 admission','speculative S2S generation','전용 보류 handle과 전환 상태'],23,MUTED,leading=37)
            a.edge('rm','ma','L','R',sd=-14,td=-14,focus=True,label='⑥ 승인 Text → SpeechRender',at=(414,841))
            a.edge('ma','rm','R','L',sd=16,td=16,ret=True,focus=True,label='⑦ 음성 결과 반환',at=(427,956))
        p.focus(x,1230,1184,'직접 경로의 허용 경계' if i==0 else '생성 경로 단순화와 기능 손실',
                ['현재 질문의 일반 지식만 사용. 개인 자료, 화면, Task나 과거 대화는 Core로 보낸다.',
                 'Request Controller의 허용 뒤 Response Manager가 Interaction Manager에 release한다.',
                 '생성된 음성, 게시 기록, 사용자가 실제로 들은 범위를 구분한다.'] if i==0 else
                ['자체 지식 질문도 Request Interpreter를 거친다. Text는 음성 생성 전에 게시할 수 있다.',
                 'Response Manager → Model Access → 공유 Omni SpeechRender → 음성 결과 반환',
                 '출력 epoch, 실제 전달 receipt와 Task 후속 연결은 계속 유지한다.'],
                '정규 전사와 현재 revision 확인은 직접 경로에도 필요하다.' if i==0 else 'Core 부하와 SpeechRender 의존성이 늘 수 있다. S2S와 동등한 기능으로 표시하지 않는다.')
    p.footer('양안의 공통 semantic 모델 호출은 생략했다. 모든 모델 연동은 Model Access를 통하며 확대 표시는 추가 인스턴스가 아니다.')
    return p


def speech():
    p=plate(6,'음성 인식의 진행과 장애를 Omni 추론으로부터 분리할 것인가?',
            'capture와 local stop은 공통으로 추론 밖에 둔다. 달라지는 것은 인식 근거 생산자의 실행 경계와 모델 의존성이다.')
    for i,x in enumerate((64,1312)):
        a=Area(p,i,x)
        p.panel(x,1184,'T' if i==0 else 'A','독립 ASR + 공유 Omni' if i==0 else '공유 Omni native evidence',
                '실제 reviewed target / 인식 실행 경계 분리' if i==0 else '탐색안 / 독립 인식 모델과 worker 제거')
        a.boundary('via',0,335,1184,652)
        a.node('im',385,402,414,84,'Interaction Manager')
        a.text(49,513,'capture / local stop은 모델 진행을 기다리지 않음',18,MUTED)
        a.boundary('ma',28,575,1128,375,'Model Access','논리적 연동 책임 / 아래는 실제 실행 경계')
        a.boundary('runtime',645,637,478,242,'Shared Inference Service','',True)
        a.node('omniadapter',676,701,416,85,'Omni Adapter / Scheduler',kind='module')
        a.text(674,815,'Voice / Semantic session + KV 분리',18,MUTED)
        a.node('omni',676,1070,416,86,'Shared Omni Model',kind='model')
        a.edge('omniadapter','omni',sd=-26,td=-26)
        a.edge('omni','omniadapter','T','B',sd=26,td=26,ret=True)
        a.edge('im','omniadapter','R','T',via=[(884,444)],label='② 원음' if i==0 else '① 원음',at=(900,532),focus=i==1)
        a.edge('omniadapter','im','R','R',via=[(1139,743.5),(1139,466)],td=22,ret=True,label='④ 해석 결과' if i==0 else '② native 전사와 시각',at=(911,894),focus=i==1)
        if i==0:
            a.boundary('worker',60,637,478,242,'Speech Input Worker','',True)
            a.node('asradapter',90,701,416,85,'Streaming ASR Adapter',kind='module',focus=True)
            a.text(89,815,'독립 CPU 예산 / producer incarnation',18,MUTED)
            a.edge('im','asradapter','L','T',via=[(298,444)],focus=True,label='① 같은 원음',at=(73,547))
            a.edge('asradapter','im','L','L',via=[(45,743.5),(45,465)],td=21,focus=True,ret=True,label='③ 전사 + 시각 + gap',at=(83,895))
            a.node('asr',90,1070,416,86,'Streaming ASR Model',kind='model',focus=True)
            a.edge('asradapter','asr',sd=-26,td=-26,focus=True)
            a.edge('asr','asradapter','T','B',sd=26,td=26,focus=True,ret=True)
        else:
            p.box(x+60,637,478,242,'#FAFBFD',LINE,8,True)
            a.text(91,667,'제거되는 독립 인식 경로',18,MUTED,bold=True)
            a.text(91,709,['Speech Input Worker','Streaming ASR weights','독립 인식 실행 예산'],23,MUTED,leading=37)
            a.text(66,1077,['Native Evidence Adapter가 모델의 event를 변환한다.','모델이 제공하지 않는 시각 근거를 만들 수는 없다.'],18,AMBER)
        a.text(55,994,'모델 내부 구현 / 학습은 VIA 책임 밖. 두 모델 모두 PC 안에 배치될 수 있다.',17,MUTED)
        p.focus(x,1230,1184,'장애 경계 확대: Omni 중단' if i==0 else '장애 경계 확대: 같은 runtime 의존',
                ['capture 지속 | 독립 ASR 전사 생산 지속 가능 | 의미 이해와 Voice 생성 중단',
                 '⑤ Interaction Manager가 정규 입력 기록을 구성하고 중요한 근거 충돌을 보존한다.',
                 'Request Controller가 입력 revision의 최종 변경 권한을 가진다.'] if i==0 else
                ['capture 지속 | recognition과 의미 처리가 함께 중단 | 유한 backlog 이후 gap',
                 '③ Interaction Manager가 native 근거를 정규 입력 기록에 연결한다.',
                 '동시 인식, partial/final, 시각 및 취소 계약이 실제로 제공되어야 한다.'],
                '추가 모델 비용이 있다. process 분리가 PC 전력과 메모리 대역폭까지 격리하지는 않는다.' if i==0 else 'native capability는 미확인. 인식이 멎은 동안 녹음만 계속된 것을 기능 성공으로 세지 않는다.')
    p.footer('Model Access는 client, worker adapter와 공유 추론 scheduler를 아우른다. 모델 자체와 VIA의 연동 코드를 구별한다.')
    return p


def semantic_detail():
    p=Plate('stage4-s01-execution-detail','S-01 / DETAIL','호출을 끝내고 다시 시작하는가, 같은 실행을 이어가는가?',
            '위에서 아래로 시간 진행. 화살표는 실제 송수신자, 점선은 반환. 모델 연산과 VIA의 확정 권한을 구별한다.',height=2000)
    p.boundary('common',64,241,2432,260,'COMMON / ①~⑤ 최초 기본 Context','양안 동일')
    xs=[120,704,1300,1896]; names=['Interaction Manager','Request Controller','Context Manager','원본 소유자의 읽기 접점']
    for j,(x,name) in enumerate(zip(xs,names)):p.node(f'c{j}',x,305,440,75,name,kind='component' if j<3 else 'data')
    for j,label in enumerate(['① 최종 입력','② Context 요청','③ 원본 읽기']):
        p.edge(f'c{j}',f'c{j+1}','R','L',sd=-12,td=-12,label=label,at=(xs[j]+446,307))
    p.edge('c3','c2','L','R',sd=15,td=15,ret=True,label='④ 원문 + revision',at=(1750,399))
    p.edge('c2','c1','L','R',sd=15,td=15,ret=True,label='⑤ Context + receipt',at=(1130,425))
    p.text(88,462,'기본 준비는 발화 중 시작할 수 있다. 양안은 같은 최종 입력, 허용 scope와 총예산에서 출발한다.',20,MUTED)
    for i,x in enumerate((64,1312)):
        p.box(x,550,1184,1220,'#F8FAFD',LINE,10)
        p.text(x+24,574,'T / 호출 종료와 재호출' if i==0 else 'A / 같은 실행의 중단과 재개',28,INK,bold=True)
        names=['Request\nController','Request\nInterpreter',None,'Context\nManager','Model\nAccess'] if i==0 else ['Request\nController','Semantic Resolution\nWorker','Capability\nRead Broker','Context\nManager','Model\nAccess']
        step=1120/len(names);centers=[x+32+step*(j+.5) for j in range(len(names))]
        for j,(cx,name) in enumerate(zip(centers,names)):
            if name is None:
                p.text(cx,664,'Broker 없음',17,MUTED,align='center')
                continue
            p.node(f'{i}lane{j}',cx-step/2+8,643,step-16,84,name,kind='module' if i and j in (1,2) else 'component',focus=j==1 or i and j==2)
            p.line([(cx,727),(cx,1655)],LINE,True,False,1.3)
        # Activation spans reveal the actual comparison, not just message order.
        if i==0:
            p.box(centers[1]-5,791,10,225,PALE,BLUE,0)
            p.box(centers[1]-5,1316,10,225,PALE,BLUE,0)
        else:
            p.box(centers[1]-5,791,10,750,PALE,BLUE,0)
            p.box(centers[4]-5,866,10,75,PALE,BLUE,0)
            p.box(centers[4]-5,1391,10,75,PALE,BLUE,0)
        events=[(0,1,'⑥ 첫 해석 요청',False),(1,3,'⑦ semantic 호출',False),(3,1,'⑧ model result',True),
                (1,0,'⑨ 추가 읽기 제안 / 첫 호출 종료',True),(0,2,'⑩ 허용된 추가 읽기 요청',False),
                (2,2,'⑪~⑫ 원본 조회와 반환',False),(2,0,'⑬ 근거 + receipt 반환',True),
                (0,1,'⑭ 두 번째 해석 요청',False),(1,3,'⑮ semantic 호출',False),(3,1,'⑯ model result',True),
                (1,0,'⑰ 최종 SemanticProposal',True)] if i==0 else [
                (0,1,'⑥ 요청별 작업 시작',False),(1,4,'⑦ semantic 실행 시작',False),(4,1,'⑧ READ_REQUIRED + handle',True),
                (1,2,'⑨ 추가 읽기 요청',False),(2,3,'⑩ 권한 검사 후 조회 위임',False),
                (3,3,'⑪~⑫ 원본 조회와 반환',False),(3,2,'⑬ 근거 + receipt',True),
                (2,1,'⑭ 근거 반환',True),(1,4,'⑮ handle로 같은 실행 재개',False),(4,1,'⑯ 최종 model result',True),
                (1,0,'⑰ 최종 SemanticProposal',True)]
        if i==0:
            remap={0:0,1:1,2:3,3:4}
            events=[(remap[s],remap[t],label,ret) for s,t,label,ret in events]
        for row,(s,t,label,ret) in enumerate(events):
            y=791+row*75;start,end=centers[s],centers[t]
            if s==t:
                p.line([(start,y),(start+64,y),(start+64,y+20),(start,y+20)],INK)
                p.label(start-120,y-30,label,INK,17)
            else:
                p.line([(start,y),(end,y)],BLUE if row in (3,7,8) else INK,ret)
                p.label(min(start,end)+8,y-31,label,BLUE if row in (3,7,8) else INK,17)
        p.box(x+28,1675,1128,64,PALE,'none',6)
        p.text(x+49,1692,'⑱ Request Controller: 현재 revision + receipt + 필수 field 확인 → Semantic Commit',20,BLUE,bold=True)
    p.footer('중단과 재개는 대안의 요구 계약이며 지원 미확인이다. 해석 중간 상태는 내구 업무 원본이나 외부 실행 승인으로 사용하지 않는다.')
    return p


def main():
    ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args()
    pages=[semantic(),screen(),retrieval(),workflow(),response(),speech(),semantic_detail()]
    errors=[]
    for p in pages:
        p.validate()
        for ext,data in [('svg',p.svg()),('drawio',p.drawio())]:
            ET.fromstring(data);path=OUT/f'{p.slug}.{ext}'
            if args.check:
                if not path.exists() or path.read_text()!=data:errors.append(path.name)
            else:path.write_text(data)
    sections='\n'.join(f'<section><h2>{html.escape(p.title)}</h2><img src="{p.slug}.svg" alt="{html.escape(p.title)}"><p><a href="{p.slug}.svg">SVG 크게 보기</a> / <a href="{p.slug}.drawio">draw.io 편집 원본</a></p></section>' for p in pages)
    gallery='<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA Architecture Review</title><style>body{margin:0;background:#eef2f7;color:#233247;font-family:Arial,sans-serif}header{padding:32px 5%;background:#233247;color:white}section{max-width:1800px;margin:36px auto;background:white;padding:28px;border:1px solid #c9d2de;border-radius:10px}img{width:100%;height:auto}h2{font-size:20px}a{color:#195fbd}</style><header><h1>VIA / Architecture Review</h1><p>S-01~06 구조 비교 + S-01 실행 상세 / 설계 탐색, 미채택 대안</p></header>'+sections+'</html>\n'
    path=OUT/'stage4-review.html'
    if args.check:
        if path.read_text()!=gallery:errors.append(path.name)
    else:path.write_text(gallery)
    if errors:raise SystemExit('Diagram drift: '+', '.join(errors))
    print(f'PASS: {len(pages)} architecture plate pairs; XML, IDs, bounds, orthogonal routes and source parity')


if __name__=='__main__':main()
