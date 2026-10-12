"""Scoped problem intros. One scene supplies SVG, draw.io and native PPTX.

Alternative mechanisms and quality definitions are preserved. Legacy background
generation remains available in generate_dp_background_slides.py.
"""
from __future__ import annotations
import argparse
import html
import json
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
DIAGRAMS = ROOT / 'docs/architecture/12-decisions/decision-packages/diagrams'
DECK = ROOT / 'docs/presentations_files/dp-background'
W, H = 1920, 1080
BLACK, TEAL, RED, GRAY = '#000000', '#00889A', '#C00000', '#A6A6A6'


class Intro:
    def __init__(self, slug, title, notes, question=None):
        self.slug, self.title, self.notes, self.items = slug, title, notes, []
        self.text(title, 60, 30, 1040, 60, 30, bold=True, role='title')
        self.line([(52, 104), (1872, 104)], GRAY, 1.5)
        if question:
            self.line([(52, 908), (1868, 908)], GRAY, 1.5)
            self.text('설계 질문', 52, 935, 180, 50, 23, color=RED, bold=True)
            self.text(question, 260, 932, 1608, 100, 23, bold=True)

    def add(self, kind, **kwargs):
        self.items.append(dict(id=f'{self.slug}-{len(self.items)+1}', kind=kind, **kwargs))

    def text(self, value, x, y, w, h, size=21, color=BLACK, bold=False, align='left', role=None):
        self.add('text', text=value, x=x, y=y, w=w, h=h, sizePt=size,
                 color=color, bold=bold, align=align, role=role)

    def box(self, value, x, y, w, h, size=21, fill='#FFFFFF', stroke=GRAY,
            bold=False, align='center', geometry='rect', color=BLACK):
        self.add('box', text=value, x=x, y=y, w=w, h=h, sizePt=size, color=color,
                 bold=bold, align=align, geometry=geometry, fill=fill, stroke=stroke)

    def line(self, points, color=GRAY, width=2, arrow=False, dashed=False):
        self.add('line', points=points, color=color, width=width, arrow=arrow, dashed=dashed)

    def svg(self):
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">',
               '<rect width="1920" height="1080" fill="#FFFFFF"/>', '<g font-family="Calibri, sans-serif">']
        for i in self.items:
            if i['kind'] == 'line':
                pts=i['points']; out.append(f'<polyline id="{i["id"]}" points="'+ ' '.join(f'{x},{y}' for x,y in pts)+
                    f'" fill="none" stroke="{i["color"]}" stroke-width="{i["width"]}"'+
                    (' stroke-dasharray="8 7"' if i['dashed'] else '')+'/>')
                if i['arrow']:
                    import math
                    a,b=pts[-2:]; ang=math.atan2(b[1]-a[1],b[0]-a[0]); length=13
                    p1=(b[0]-length*math.cos(ang-.5),b[1]-length*math.sin(ang-.5))
                    p2=(b[0]-length*math.cos(ang+.5),b[1]-length*math.sin(ang+.5))
                    out.append(f'<polyline points="{p1[0]},{p1[1]} {b[0]},{b[1]} {p2[0]},{p2[1]}" fill="none" stroke="{i["color"]}" stroke-width="{i["width"]}"/>')
                continue
            if i['kind']=='box':
                if i['geometry']=='ellipse':
                    out.append(f'<ellipse cx="{i["x"]+i["w"]/2}" cy="{i["y"]+i["h"]/2}" rx="{i["w"]/2}" ry="{i["h"]/2}" fill="{i["fill"]}" stroke="{i["stroke"]}" stroke-width="2"/>')
                else:
                    out.append(f'<rect x="{i["x"]}" y="{i["y"]}" width="{i["w"]}" height="{i["h"]}" fill="{i["fill"]}" stroke="{i["stroke"]}" stroke-width="2"/>')
            lines=i['text'].split('\n'); px=i['sizePt']*4/3; leading=px*1.2
            top=i['y']+(i['h']-len(lines)*leading)/2
            anchor={'left':'start','center':'middle','right':'end'}[i['align']]
            x=i['x']+(0 if anchor=='start' else i['w']/2 if anchor=='middle' else i['w'])
            if i['kind']=='box' and anchor=='start': x+=14
            for n,line in enumerate(lines):
                out.append(f'<text id="{i["id"]}-text-{n}" x="{x}" y="{top+px*.94+n*leading}" font-size="{px}" text-anchor="{anchor}" font-weight="{700 if i["bold"] else 400}" fill="{i["color"]}" data-width="{i["w"]}">{html.escape(line)}</text>')
        return '\n'.join(out+['</g></svg>'])+'\n'

    def drawio(self):
        mx=ET.Element('mxfile',host='app.diagrams.net',type='device')
        d=ET.SubElement(mx,'diagram',id=self.slug,name=self.title)
        g=ET.SubElement(d,'mxGraphModel',page='1',pageWidth=str(W),pageHeight=str(H),background='#FFFFFF')
        root=ET.SubElement(g,'root');ET.SubElement(root,'mxCell',id='0');ET.SubElement(root,'mxCell',id='1',parent='0')
        for i in self.items:
            if i['kind']=='line':
                style=f'edgeStyle=none;rounded=0;strokeColor={i["color"]};strokeWidth={i["width"]};endArrow={"open" if i["arrow"] else "none"};endFill=0;'+('dashed=1;' if i['dashed'] else '')
                c=ET.SubElement(root,'mxCell',id=i['id'],parent='1',edge='1',style=style)
                geo=ET.SubElement(c,'mxGeometry',relative='1',attrib={'as':'geometry'})
                for p,role in [(i['points'][0],'sourcePoint'),(i['points'][-1],'targetPoint')]:
                    ET.SubElement(geo,'mxPoint',x=str(p[0]),y=str(p[1]),attrib={'as':role})
                arr=ET.SubElement(geo,'Array',attrib={'as':'points'})
                for x,y in i['points'][1:-1]:ET.SubElement(arr,'mxPoint',x=str(x),y=str(y))
            else:
                value='<br>'.join(html.escape(t) for t in i['text'].split('\n'))
                px=i['sizePt']*4/3
                style=f'fontFamily=Calibri;fontSize={px};fontColor={i["color"]};fontStyle={int(i["bold"])};html=1;whiteSpace=wrap;align={i["align"]};verticalAlign=middle;spacing=0;'
                if i['kind']=='box':style+=f'shape={i["geometry"]};fillColor={i["fill"]};strokeColor={i["stroke"]};strokeWidth=2;spacing=14;'
                else:style+='text;fillColor=none;strokeColor=none;'
                c=ET.SubElement(root,'mxCell',id=i['id'],parent='1',vertex='1',value=value,style=style)
                ET.SubElement(c,'mxGeometry',x=str(i['x']),y=str(i['y']),width=str(i['w']),height=str(i['h']),attrib={'as':'geometry'})
        return ET.tostring(mx,encoding='unicode')+'\n'


def common():
    s=Intro('via-common-request-background', '말과 화면, 이전 업무를 섞은 요청',
        '사용자는 실행 ID나 자료 경로를 지정하지 않는다. 화면 지칭, 이전 대화와 진행 업무를 섞어 말하고 일부만 정정한다. 표 대상과 보고서 업무 연결은 해석의 결과이며 입력에 정답으로 제공하지 않는다. “두 번째 표만”은 대상 범위의 정정이고 메일 발송 금지는 보존한다. VIA는 입력과 근거를 연결하고 요청·정정·기존 제약을 정리한다. 보고서 작성의 도메인 추론, 계획, 도구 선택과 실제 실행은 Downstream Agent 책임이다. 이 공통 요구 장면은 주요 DP에서 제외한 41을 복원하지 않는다. 44의 사건 연결, 45의 과거 맥락, 42의 서로 다른 수명과 새 화면·음성 지칭 문제로 이어진다.',
        '이 연결을 유지하며 새 입력, 과거 맥락과 업무를 어떻게 다룰까?')
    s.text('“이 표를 아까 보고서에 넣어줘.\n아니, 두 번째 표만. 메일은 보내지 말고.”',52,150,1816,110,27,bold=True)
    for x,title in [(52,'화면에서 가리킨 자료'),(676,'앞서 진행한 업무'),(1300,'바꾼 부분과 유지할 조건')]:s.text(title,x,292,568,52,22,bold=True)
    s.box('첫 번째 표',80,365,504,72,22,fill='#F2F2F2')
    s.box('두 번째 표',80,460,504,72,22,stroke=TEAL,bold=True)
    s.box('보고서 작성 업무',704,365,504,72,22)
    s.box('메일 초안 업무',704,460,504,72,22)
    s.text('표는 두 번째만\n메일 발송 금지 유지',1328,372,504,150,24,bold=True)
    for x in [332,956,1580]:s.line([(x,560),(x,598)],TEAL,2.5,arrow=True)
    s.text('실제 자료와 범위',52,615,568,52,24,bold=True,align='center')
    s.text('수정할 보고서 업무',676,615,568,52,24,bold=True,align='center')
    s.text('정정 범위와 기존 조건',1300,615,568,52,24,bold=True,align='center')
    s.text('잘못 연결하면 다른 자료를 넣거나, 금지한 메일을 보내게 됩니다.',52,714,1816,50,23,bold=True)
    s.text('VIA는 대상과 업무 연결, 정정 범위를 함께 판단하고 기존 조건을 보존해야 합니다.',52,786,1816,50,22)
    s.text('보고서를 어떻게 작성할지 추론하고 실제로 만드는 일은 Downstream Agent가 맡습니다.',52,843,1816,46,19)
    return s


def interaction():
    s=Intro('dp44-background','DP44. 정정과 업무 알림이 겹치는 대화',
        '설명용 사건 순서이며 간격과 선 길이는 측정 시간이 아니다. 보고서 설명 중 정정, 별도 메일 업무의 수신자 질문, 정정 전 응답 후보의 늦은 준비를 함께 보여준다. 발화 시작 때 로컬 음성 중단과 지속 입력은 공통 기능이다. 음성 중단은 업무 취소 완료가 아니다. 질문은 해당 업무와 연결하고 실제 제시 확인 뒤에 답변 focus의 근거로 쓴다. 도착·후보 준비·게시 허용·실제 전달은 다르다. 고정 전달 우선순위나 특정 A/B의 실행 조직을 선택하지 않는다. 강한 A의 비동기 실행과 B의 Window/Join 비용, 현재성 및 권한 검사 계약은 기존 비교에 남는다.',
        '사용자와 여러 Agent가 보내는 입력·질문·결과를\n사용자 통제를 유지하며 어떻게 처리할까?')
    s.text('VIA가 보고서를 설명하는 동안, 사용자는 “잠깐, 표부터 설명해줘”라고 고칩니다.',52,151,1816,58,25,bold=True)
    s.text('그 사이 메일 업무의 질문이 오고, 정정 전 답변도 뒤늦게 준비됩니다.',52,217,1816,48,23)
    for y,label in [(350,'사용자'),(438,'VIA 설명'),(548,'메일 업무'),(650,'이전 답변 준비')]:
        s.text(label,52,y-23,280,48,22,bold=True)
        s.line([(345,y),({350:1850,438:730,548:910,650:1050}[y],y)],'#D0D0D0',1.4,dashed=True)
    s.text('“잠깐, 표부터 설명해줘.”',740,292,710,50,24,bold=True)
    s.line([(745,350),(1290,350)],TEAL,8)
    s.line([(360,438),(745,438)],TEAL,8)
    s.line([(745,417),(745,460)],RED,4)
    s.text('즉시 중단 · 입력은 계속 수신',780,402,1050,60,22,bold=True)
    s.text('보고서 설명 중',360,466,350,44,19)
    s.box('',915,534,28,28,geometry='ellipse',stroke=TEAL)
    s.text('“받는 사람은 누구인가요?”',910,578,620,48,22,bold=True)
    s.line([(943,548),(1430,548)],TEAL,2.5,dashed=True,arrow=True)
    s.text('메일 업무로 연결\n적절한 차례에 제시',1460,516,390,86,22,bold=True)
    s.box('',1066,636,28,28,geometry='ellipse',stroke=RED)
    s.line([(1094,650),(1295,650)],RED,2.5,arrow=True)
    s.line([(1320,636),(1348,664)],RED,3);s.line([(1348,636),(1320,664)],RED,3)
    s.text('정정 전 후보는 다시 말하지 않음',1390,627,460,52,21,bold=True)
    s.text('정정 전 답변을 다시 말하거나 메일 질문을 놓치면, 사용자의 제어가 무너집니다.',52,733,1816,54,23,bold=True)
    s.text('도착 순서만으로 다음 처리를 정할 수 없습니다. 준비 완료와 실제 전달도 구별해야 합니다.',52,817,1816,64,22)
    return s


def memory():
    s=Intro('dp45-background','DP45. 과거 설명과 업무 결과의 재사용',
        '기존 C20/P20, T21/D21@v2, T22/D22@v1 사례를 쉬운 말로 표현한다. 직접 설명은 Task 없는 응답일 수 있고 제품 비교와 견적은 서로 다른 Agent의 확인된 결과다. 생성 원문과 실제 표시·재생 범위, 결과 본문과 허용 참조는 다르다. 정정과 삭제는 사용 가능한 근거를 바꾼다. 현재 의미·Task 연결의 제안/채택은 기존 owner에 남는다. 자료 분석·기준 적용·제안서 작성은 Downstream Agent 책임이다. 원본 수정·삭제와 현재 권한 및 실제 전달 확인은 양안 공통 요구다. A의 선별·요약·색인·cache와 B의 갱신 택틱을 다음 기본 구조 페이지에서 분리하며 기존 MAIN은 유지한다.',
        'VIA의 설명, 여러 Agent 결과와 정정·실제 전달 기록을\n나중 요청에서 어떻게 다시 연결해 사용할까?')
    s.text('“지난번 네가 설명한 기준으로,\n제품 비교와 견적도 이번 제안서에 반영해줘.”',52,150,1816,110,27,bold=True)
    for x,title,body in [(52,'VIA에게 들은 설명','평가 기준\n원문과 실제 전달 범위'),(676,'제품 조사 Agent 결과','제품 비교\n확인된 결과와 자료 참조'),(1300,'견적 Agent 결과','견적\n확인된 결과와 버전')]:
        s.text(title,x,310,568,50,23,bold=True);s.box(body,x,382,568,117,22)
    for x in [336,960,1584]:
        s.line([(x,503),(x,523)],TEAL,2.5)
        s.line([(x,590),(x,605)],TEAL,2.5,arrow=True)
    s.text('그 뒤 “가격은 비교에서 빼줘”라고 조건을 고쳤습니다.',52,535,1816,50,24,bold=True)
    s.box('이번 요청의 과거 맥락\n실제 전달한 기준 · 정정한 조건 · 사용 가능한 결과',52,627,1816,109,23,fill='#F2F2F2',bold=True)
    s.text('옛 조건이나 전달하지 않은 설명을 쓰면, 이번 업무를 다른 뜻으로 맡기게 됩니다.',52,767,1816,52,23,bold=True)
    s.text('기록이 여러 곳에 흩어져 있고, 수정·삭제 후 사용할 수 있는 범위도 바뀝니다.',52,837,1816,48,22)
    return s


def memory_basics():
    s=Intro('dp45-basic-structures','DP45. 기본 구조와 보완 택틱',
        '기존 45 A/B의 기본을 먼저 설명하는 추가 페이지다. A는 원본 owner의 읽기를 현재 요청에서 조합한다. 요약·색인·관계 cache와 부분 갱신을 허용하며 모든 이력을 재처리하지 않는다. B는 생산자가 유지하는 공통 과거 관계의 조회 계약에 정상 요청이 의존한다. 선택적 cache만으로 B가 되지 않는다. 상자는 기능/데이터 설명이며 새 Component/배치가 아니다. 수정·삭제·권한 철회와 실제 전달 범위는 공통 요구다. B의 미생산·표현 밖·접근 불가, 생산 후 재조회와 명시적 B+ 원본 조합 우회는 기존 계약과 비용을 따른다. 택틱은 기본 구조와 별도로 비교한다. A/B 미선정이며 구현이나 수치 우세를 주장하지 않는다.')
    s.text('같은 과거 기록과 같은 현재 요청을 사용합니다.',52,154,1816,50,24,bold=True)
    s.box('A  필요할 때 원본에서 구성',52,228,880,62,23,fill='#E8EEF5',bold=True)
    s.box('B  가공한 과거 기억을 유지해 조회',982,228,886,62,23,fill='#FFF6DF',bold=True)
    for x in [112,1042]:s.box('설명·업무 결과·정정·실제 전달의 원본',x,323,760,72,21)
    s.line([(492,395),(492,456)],TEAL,2.5,arrow=True)
    s.box('이번 요청에 필요한 맥락 구성',112,469,760,86,23,bold=True)
    s.line([(492,555),(492,598)],TEAL,2.5,arrow=True)
    s.text('이번 요청의 근거로 반환',112,614,760,45,22,align='center')
    s.line([(1422,395),(1422,449)],TEAL,2.5,arrow=True)
    s.box('가공한 과거 기억 생산·유지',1042,459,760,80,23,bold=True)
    s.line([(1422,539),(1422,583)],TEAL,2.5,arrow=True)
    s.text('이번 요청이 필요한 기억을 조회',1042,604,760,55,22,align='center')
    s.line([(52,704),(1868,704)],GRAY,1.5)
    s.text('구조를 보완하는 택틱',52,727,1816,45,23,bold=True)
    s.text('A  요약·색인·cache와 부분 갱신',52,789,880,50,22)
    s.text('B  생산 범위 선택·증분 갱신·무효화',982,789,886,50,22)
    s.text('수정·삭제와 실제 전달 범위 반영은 양안 공통 요구입니다.',52,868,1816,50,23,bold=True)
    s.text('A의 구성 비용과 B의 생산·유지 비용을, 첫 사용·반복·정정 이후 장면에서 비교합니다.',52,944,1816,60,21)
    return s


def lifecycle():
    s=Intro('dp42-background','DP42. 요청이 끝나도 이어지는 보고서 업무',
        '위임 Request 처리 종료는 외부 Agent의 접수 확인 뒤이며 실제 보고서 완료와 다르다. 진행 확인은 별도 Request이고 새 작성 실행을 시작하지 않는다. Voice를 끄는 동안에도 대화 기록·Task·외부 실행·질문은 남는다. 질문은 답변/만료를 기다린다. 재연결 후 유효 질문에 답하고 최초 작성이 완료된 뒤, 결론 수정의 새 실행을 같은 업무에 연결할 수 있다. 선은 설명용 순서이며 측정 시간이나 자동 실행 조건이 아니다. 대화·Request·Task·Execution·질문은 개념과 상태이며 Component/process가 아니다. 강한 A도 논리 수명 분리, 비동기 실행과 background Core를 허용한다. 다른 논리 수명만으로 B의 서비스 분리 필요성이나 우위가 성립하지 않는다. 부모 세션의 B 강화와 기존 MAIN은 변경하지 않는다.',
        '대화·요청·질문·업무·실행의 서로 다른 수명을\n어떻게 관리하고 연결할까?')
    s.text('“보고서 만들어줘”를 전달한 뒤에도, 보고서 작성은 계속됩니다.',52,150,1816,58,26,bold=True)
    s.text('“어디까지 했어?”는 새 요청이고, 완료 후 “결론만 고쳐줘”는 같은 업무의 새 실행이 될 수 있습니다.',52,215,1816,58,22)
    xs=[342,752,1162,1572]
    for x,title,detail in zip(xs,['“보고서 만들어줘”','“어디까지 했어?”','Voice 끄기','“결론만 고쳐줘”'],['외부 접수 확인','진행 상태로 답변','업무·대기 질문 유지','최초 작성 완료 후']):
        s.text(title,x-65,312,386,50,22,bold=True);s.text(detail,x-65,369,386,44,19)
    for y,label in [(465,'대화'),(538,'개별 요청'),(615,'보고서 업무'),(699,'외부 실행')]:s.text(label,52,y-22,230,48,22,bold=True)
    s.line([(340,465),(1820,465)],TEAL,5)
    s.text('Voice를 꺼도 기록과 연결은 유지',867,431,800,42,19)
    for x,w in [(340,210),(750,210),(1570,220)]:s.line([(x,538),(x+w,538)],TEAL,7)
    s.text('위임 처리 종료',330,554,300,40,18);s.text('새 작성 실행 없음',750,554,340,40,18)
    s.line([(340,615),(1820,615)],TEAL,7)
    s.text('같은 보고서 업무로 연결',1310,639,500,40,19,bold=True)
    s.line([(340,699),(1400,699)],TEAL,7);s.line([(1565,699),(1820,699)],TEAL,7)
    s.text('최초 작성 실행',340,716,310,42,19);s.text('새 수정 실행',1565,716,260,42,19)
    s.box('대기 질문 유지\n답변·만료로 종료',1060,750,460,74,19,fill='#F2F2F2')
    s.text('요청·Voice 종료를 업무 종료로 다루면, 연결을 잃거나 같은 작성을 다시 시작할 수 있습니다.',52,839,1816,58,22,bold=True)
    return s


def grounding():
    s=Intro('screen-voice-grounding-background','화면과 음성 지칭의 결합 (문제)',
        '새 주제의 공통 문제이며 DP 번호와 A/B 미정이다. 46의 A/B나 32의 제외된 소유권 비교를 계승하지 않는다. 첫 문서 표를 가리키며 “이 표와”라고 하고 다른 문서로 옮겨 “저 표를 비교해줘”라고 한다. 전사 도착 때 첫 표는 사라졌거나 pointer가 벗어났을 수 있다. “아니, 여기”는 둘째 지칭을 아래 표로 정정하는 사례다. 실제 발화 시간과 오차, 당시 화면/문서 identity, pointer 경로와 선택 범위가 근거다. 시간 겹침은 후보 근거이며 의도 확정이 아니다. 정보 누락/우연한 이동은 확인이 필요할 수 있다. 사전 선택도 고정 UC이며 상시 수집·새 helper·시선 추적은 전제하지 않는다. 저장·생산·분해 기제는 부모 세션의 대안 탐색에 남는다.',
        '말과 손동작을 당시 화면에 연결해,\n사용자가 뜻한 대상을 어떻게 찾을까?')
    s.text('첫 표를 가리켜 “이 표와…”라고 말하고, 화면을 바꿔 “…저 표를 비교해줘”라고 이어 갑니다.',52,149,1816,63,24,bold=True)
    xs=[52,676,1300]
    for x,title in zip(xs,['말하는 순간 1','말하는 순간 2','전사가 도착한 순간']):s.text(title,x,252,568,52,23,bold=True)
    for x,title in zip(xs,['문서 A의 화면','문서 B의 화면','현재 문서 B의 화면']):s.box(title,x,325,568,54,20,fill='#F2F2F2',align='left')
    s.box('첫 표',80,405,512,153,25,stroke=TEAL,bold=True)
    s.box('둘째 표',704,399,512,81,22,stroke=TEAL,bold=True)
    s.box('아래 표',704,515,512,81,22)
    s.box('둘째 표',1328,399,512,81,22)
    s.box('정정한 아래 표\n선택 범위',1328,515,512,81,20,stroke=TEAL,bold=True)
    s.line([(149,578),(193,552),(230,505)],TEAL,2.5,arrow=True)
    s.line([(690,500),(725,470),(760,445)],TEAL,2.5,arrow=True)
    s.line([(1315,610),(1350,585),(1385,555)],TEAL,2.5,arrow=True)
    s.text('pointer로 첫 표 지시',80,602,512,44,19)
    s.text('다른 창의 표로 이동',704,619,512,44,19)
    s.text('첫 표는 이미 보이지 않음',1328,619,512,44,19,bold=True)
    s.line([(620,479),(656,479)],GRAY,2.5,arrow=True);s.line([(1244,479),(1280,479)],GRAY,2.5,arrow=True)
    s.text('“아니, 여기.”로 정정하면 선택 범위와 바꿀 지칭도 다시 연결해야 합니다.',52,700,1816,54,23,bold=True)
    s.text('근거: 말한 구간, 당시 화면, pointer 경로와 선택 범위',52,766,1816,50,22)
    s.text('마지막 화면만 쓰면 첫 표를 잃고, 시각이 겹친다는 이유만으로 대상을 정하면 틀릴 수 있습니다.',52,837,1816,58,22,bold=True)
    s.text('새 주제 탐색 · 번호와 A/B 미정',52,1034,1816,30,17)
    return s


def scenes():return [common(),interaction(),memory(),memory_basics(),lifecycle(),grounding()]


def outputs():
    result={}
    combined=ET.Element('mxfile',host='app.diagrams.net',type='device')
    for s in scenes():
        result[DIAGRAMS/f'{s.slug}.svg']=s.svg();result[DIAGRAMS/f'{s.slug}.drawio']=s.drawio()
        combined.append(ET.fromstring(s.drawio())[0])
    result[DECK/'VIA-problem-intros-v8.drawio']=ET.tostring(combined,encoding='unicode')+'\n'
    prefix='../../architecture/12-decisions/decision-packages/diagrams/'
    sections=''.join(f'<section id="{s.slug}"><h2>{html.escape(s.title)}</h2><img src="{prefix}{s.slug}.svg" alt="{html.escape(s.title)}"><p><a href="{prefix}{s.slug}.drawio">draw.io</a> · <a href="{s.slug}.png">PNG</a></p></section>' for s in scenes())
    result[DECK/'index.html']='''<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA 문제 소개 v8</title><style>body{margin:0;background:#f2f2f2;font-family:Calibri,sans-serif;color:black}header{padding:24px 3vw;background:white}main{max-width:1920px;margin:auto}section{padding:20px 2vw}h1{font-size:28px}h2{font-size:22px}img{display:block;width:100%;height:auto;background:white}a{color:#00889a}@media print{header,h2,p{display:none}section{padding:0;break-after:page}@page{size:16in 9in;margin:0}}</style><header><h1>VIA 문제 소개 — v8</h1><p>공통 요청 → 44 → 45와 기본 구조 → 42 → 화면·음성 지칭의 새 문제. 41은 주요 DP에서 제외했으며 이전 문서와 자료는 보존합니다. 화면·음성 주제의 DP 번호와 A/B는 미정입니다.</p><nav><a href="../VIA-architecture-presentation_v8.pptx">최신 통합 PPTX</a> · <a href="VIA-problem-intros-v8.drawio">현재 6장 draw.io</a> · <a href="VIA-DP-background-overview-41-45.drawio">이전 구성 draw.io</a></nav></header><main>'''+sections+'</main></html>\n'
    return result


def main():
    p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');p.add_argument('--scene-json',type=Path)
    args=p.parse_args()
    if args.scene_json:args.scene_json.write_text(json.dumps([vars(s) for s in scenes()],ensure_ascii=False,indent=2)+'\n')
    for path,value in outputs().items():
        if args.check:assert path.read_text()==value,f'Out of sync: {path}'
        else:path.write_text(value)
    print(f'{len(outputs())} scoped figure/deck preview files '+('verified' if args.check else 'written'))


if __name__=='__main__':main()
