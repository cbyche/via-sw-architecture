#!/usr/bin/env python3
"""One-page implementation comparison. Documentation only, no VIA execution."""
import argparse
import hashlib
import json
from pathlib import Path

import dp47_detail_scene as drawing
from dp47_detail_scene import GroundingScene
from dp47_implementation_views import connect

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/architecture/12-decisions/decision-packages/diagrams'
SLUG = 'dp47-implementation-comparison-v1'


class ComparisonScene(GroundingScene):
    def texts(self, n):
        x, y, w, h = (n[k] for k in ('x', 'y', 'w', 'h'))
        if n['kind'] in ('component', 'group'):
            return [(x+24, y+43, n['title'], 38, True, 'left')]
        titles = n['title'].split('\n')
        title_top=60 if n['kind']=='db' else 46
        result = [(x+w/2, y+title_top+j*43, t, 36, True, 'center') for j,t in enumerate(titles)]
        top = y+46+len(titles)*43
        for j, line in enumerate(n['lines']):
            result.append((x+22, top+j*39, line, 32, False, 'left'))
        return result


def scene():
    p = ComparisonScene(SLUG, 'DP-47 화면과 음성 지칭의 구현 구조 비교', '')
    p.notes.clear()
    p.text(60,72,'DP-47  화면과 음성 지칭의 구현 구조 비교',58,bold=True)
    p.text(60,132,'A는 원본에서 binding을 직접 갱신한다. B는 생산자별 후보를 게시한 뒤 결합한다.',38)
    p.node('inputs',60,190,1240,250,'Interaction Manager',kind='component')
    p.node('raw',85,260,1190,155,'Timeline & Buffer / Raw Evidence Window',[
        'RAM ring + 시간 index: 원음, 전사, frame, path, 선택',
        '과거 화면 유지 / bounded read / 누락과 시간 오차 보존'],kind='memory',owner='inputs')
    p.node('request',1370,190,1070,250,'Request Controller',kind='component')
    p.node('input',1395,260,1020,155,'1  GroundingInput / immutable DTO',[
        'requestId + inputRev + raw refs + prior + budget',
        'BeginRevision: 먼저 fence 갱신, 이어서 입력 전달'],kind='data',owner='request')
    p.node('adapters',2510,190,1270,250,'Request Interpreter / 공통 read 및 model adapter',kind='component')
    p.node('ports',2535,260,1220,155,'R: Grounding Evidence Reader / M: Grounding Model Port',[
        'R은 원본과 허용 Context 조회 / M은 Model Access 호출',
        '아래 R/M 표기는 같은 adapter 사용 / 호출한 worker로 반환'],owner='adapters')

    for side,x in [('a',60),('b',1970)]:
        p.node(side,x,560,1810,1210,'Request Interpreter / local async tasks',kind='component')
        p.node(side+'_inbox',x+30,645,300,185,'Control\nInbox',[
            'bounded FIFO'],kind='queue',owner=side)
        p.node(side+'_ctl',x+400,645,650,145,
            'Grounding Attempt Control' if side=='a' else 'Grounding Coordinator',[
                'CODE / job 제어와 완료 회수' if side=='a' else 'CODE / 생산 지시와 결합 실행 관리'],owner=side,different=True)
        p.node(side+'_registry',x+1130,645,650,145,'Grounding Attempt Registry',[
            'RAM Map / inputRev, job, 예산', 'RevisionGate / 요청별 짧은 lock'],kind='memory',owner=side)
        p.node(side+'_done',x+30,1100,300,235,'Completion\nInbox',[
            'bounded FIFO', 'job outcome'],kind='queue',owner=side)
        connect(p,'input',side+'_inbox','B','T','post')
        connect(p,side+'_inbox',side+'_ctl','R','L','post')
        connect(p,side+'_ctl',side+'_registry','R','L','state')
        connect(p,side+'_done',side+'_ctl','T','B','done')

    p.text(80,508,'A  원본 공동 해석',44,drawing.BLUE,True)
    p.text(1990,508,'B  후보 생산과 결합',44,drawing.BLUE,True)
    p.text(570,508,'비동기 workflow / 단일 binding aggregate',32)
    p.text(2510,508,'Fork–Join + feedback / 생산자별 repository',32)
    p.node('a_q',460,890,650,135,'Joint Work Queue',[
        'JointJob / refs + prior + correction'],kind='queue',owner='a',different=True)
    p.node('a_worker',460,1130,650,190,'Joint Grounding Interpreter',[
        'ASYNC WORKER / R read, M invoke',
        '원음과 화면을 함께 해석', '새 target과 binding delta 직접 생산'],owner='a',different=True)
    p.node('a_state',1190,1130,650,190,'Joint Grounding State',[
        'RAM Map / bindingId, revision', '원본 의존 + base version',
        'writer: Joint Grounding Interpreter'],kind='memory',owner='a',different=True)
    p.node('a_result',460,1490,1380,190,'5  InterpretGrounding result / typed union',[
        'Ready(delta) / NeedEvidence(scope) / Ambiguous / Failed',
        '추가 근거가 필요하면 동일 workflow로 복귀',
        '부분 patch: 처음 매출 범위와 표 유지, 아래 graph만 교체'],kind='data',owner='a')
    connect(p,'a_ctl','a_q','B','T','post')
    connect(p,'a_q','a_worker','B','T','post')
    connect(p,'a_worker','a_state','R','L','state')
    p.edge('a_worker','a_result','R','T',via=[(1150,1225),(1150,1360),(1815,1360)],td=665,kind='done')
    connect(p,'a_result','a_done','L','B','done')
    p.text(480,857,'2  실행 지시',32,drawing.BLUE)
    p.text(480,1088,'3  원본 read 및 공동 의미 판단',32,drawing.BLUE)
    p.text(1175,1088,'4  TryApply / 현재 revision만 게시',32)
    p.text(480,1432,'직접 target 생산과 binding 갱신이 하나의 의미 책임',35,drawing.BLUE,True)

    for role,x,title,state in [
        ('speech',2370,'Speech Reference Producer','Speech Candidate Revisions'),
        ('screen',3100,'Screen Target Producer','Screen Candidate Revisions')]:
        p.node(role+'_q',x,890,650,135,role.title()+' Work Queue',[
            role.title()+'Job / refs + peer + generation'],kind='queue',owner='b',different=True)
        p.node(role,x,1100,650,185,title,[
            'ASYNC WORKER / R read, M 필요 시',
            '표현, 개수, 정정 역할의 후보' if role=='speech' else '영역, 집합, 동작 해석의 후보',
            'TryPublishSpeech' if role=='speech' else 'TryPublishScreen'],owner='b',different=True)
        p.node(role+'_state',x,1360,650,145,state,[
            'RAM Map / candidateId, revision', 'source refs + peer versions + coverage'],kind='memory',owner='b',different=True)
        connect(p,'b_ctl',role+'_q','B','T','post')
        connect(p,role+'_q',role,'B','T','post')
        connect(p,role,role+'_state','B','T','state')
        connect(p,role,'b_done','L' if role=='speech' else 'R','R','done')
    p.node('binding_q',2010,1560,570,145,'Binding Work Queue',[
        'BindingJob / candidate read-set', 'prior + deadline'],kind='queue',owner='b',different=True)
    p.node('resolver',2660,1560,650,145,'Multimodal Binding Resolver',[
        'ASYNC WORKER / M / 후보와 excerpt', 'TryApplyBinding / 부족하면 Rejudge'],owner='b',different=True)
    p.node('b_state',3390,1560,350,145,'Binding Revision\nState',[
        'RAM Map / read-set'],kind='memory',owner='b',different=True)
    connect(p,'b_ctl','binding_q','L','L','post')
    connect(p,'binding_q','resolver','R','L','post')
    connect(p,'speech_state','resolver','B','T','state')
    connect(p,'screen_state','resolver','B','T','state')
    connect(p,'resolver','b_state','R','L','state')
    connect(p,'resolver','b_done','L','B','cancel')
    p.text(2390,857,'2  생산 job dispatch / 자원이 허용하면 병행',32,drawing.BLUE)
    p.text(2390,1070,'3  역할별 원본 해석',32,drawing.BLUE)
    p.text(2390,1332,'4  생산자별 게시 후 완료 통지',32)
    p.text(2010,1540,'5  호환 snapshot + 응답/deadline으로 결합 지시',31,drawing.BLUE)
    p.text(2010,1744,'6  후보 부족: 완료 inbox로 Rejudge 반환, Coordinator가 원 producer에 재지시',30,drawing.RED)

    p.node('adopt',60,1845,3720,190,'공통 결과 채택 / Request Controller와 State Store',kind='component')
    p.node('proposal',90,1910,1600,100,'GroundingProposal: binding delta + 근거 + revision',[],kind='data',owner='adopt')
    p.node('adoption',1780,1910,1050,100,'Adoption Control: 최신 입력, 사용권, base 검사',[],owner='adopt')
    p.node('db',2920,1910,830,100,'State Store / 채택 결과 영속 DB',[],kind='db',owner='adopt')
    connect(p,'a_result','proposal','B','T','done')
    p.edge('resolver','proposal','B','T',via=[(2985,1720),(3766,1720),(3766,1835),(1550,1835)],td=660,kind='done')
    connect(p,'proposal','adoption','R','L','post')
    connect(p,'adoption','db','R','L','state')
    p.text(90,1818,'공통 정정: BeginRevision(r2)와 TryApply/TryPublish가 같은 RevisionGate 사용 / 늦은 r1은 상태 반영 전 거절',32)
    p.text(60,2090,'사각형: 실행 코드   칸 상자: bounded RAM FIFO   겹친 카드: RAM data model   접힌 종이: DTO   원통: 영속 DB',32)
    p.text(60,2140,'실선: 작업 전달   점선: 완료 반환   회색: 상태 접근   빨강: 재판단   /   설계 가정, 미구현, 미측정',30)
    for note in p.notes:
        if note['color']=='#263445':note['color']='#111111'
    return p


def main():
    parser=argparse.ArgumentParser(); parser.add_argument('--check',action='store_true'); args=parser.parse_args()
    drawing.INK='#111111'; drawing.GRAY='#777777'; drawing.PALE='#F0F6F5'; drawing.ORANGE='#238176'
    drawing.FONT='Calibri, Malgun Gothic, Apple SD Gothic Neo, sans-serif'
    p=scene(); p.validate()
    outputs={'svg':p.svg(),'drawio':p.drawio().replace('fontFamily=Arial','fontFamily=Calibri'),
             'json':json.dumps(dict(width=3840,height=2160,title=p.title,nodes=list(p.nodes.values()),edges=p.edges,notes=p.notes),ensure_ascii=False,indent=2)+'\n'}
    for ext,text in outputs.items():
        path=OUT/(SLUG+'.'+ext)
        if args.check: assert path.read_text()==text, f'Stale comparison: {path.name}'
        else:path.write_text(text)
    if args.check:
        manifest=json.loads((OUT/(SLUG+'-render.json')).read_text())
        for ext in ('svg','png'):
            assert manifest[ext+'_sha256']==hashlib.sha256((OUT/(SLUG+'.'+ext)).read_bytes()).hexdigest()
    print('PASS: DP-47 one-page comparison sources, owners and routes')


if __name__=='__main__':main()
