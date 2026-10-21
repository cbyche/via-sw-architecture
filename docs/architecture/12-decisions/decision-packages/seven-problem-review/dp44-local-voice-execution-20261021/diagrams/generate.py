"""Generate paired editable draw.io and SVG architecture figures; no product code."""
from pathlib import Path
from html import escape
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent
INK = "#253044"
BLUE = "#2257b5"
GREEN = "#087d5b"
MUTED = "#5b6576"
FONT = "Apple SD Gothic Neo, Noto Sans CJK KR, sans-serif"


class Figure:
    def __init__(self, name, title, subtitle, width, height, accent):
        self.name, self.width, self.height, self.accent = name, width, height, accent
        self.shapes, self.edges, self.labels = [], [], []
        self.label(35, 40, title, 28, INK)
        self.label(35, 74, subtitle, 17, MUTED)

    def label(self, x, y, value, size=16, color=INK, center=False):
        self.labels.append((x, y, value, size, color, center))

    def shape(self, key, x, y, w, h, value, kind="component", color=INK):
        self.shapes.append((key, x, y, w, h, value, kind, color))

    def edge(self, points, label="", at=None, color=INK, dashed=False, arrow=True):
        self.edges.append((points, label, at, color, dashed, arrow))

    def save(self):
        svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{self.width}" height="{self.height}" viewBox="0 0 {self.width} {self.height}">',
               '<rect width="100%" height="100%" fill="#fff"/>', '<defs>']
        for i, color in enumerate([INK, BLUE, GREEN]):
            svg.append(f'<marker id="arrow{i}" markerWidth="10" markerHeight="8" refX="9" refY="4" orient="auto"><path d="M0,0 L10,4 L0,8 Z" fill="{color}"/></marker>')
        svg.append('</defs>')
        mx = ET.Element("mxfile", host="app.diagrams.net")
        diag = ET.SubElement(mx, "diagram", name=self.name, id=self.name)
        model = ET.SubElement(diag, "mxGraphModel", page="1", pageWidth=str(self.width), pageHeight=str(self.height))
        root = ET.SubElement(model, "root")
        ET.SubElement(root, "mxCell", id="0")
        ET.SubElement(root, "mxCell", id="1", parent="0")

        def cell(key, value, style, x, y, w, h, parent="1"):
            c = ET.SubElement(root, "mxCell", id=key, value=value, style=style, vertex="1", parent=parent)
            ET.SubElement(c, "mxGeometry", x=str(x), y=str(y), width=str(w), height=str(h), **{"as": "geometry"})

        def text(x, y, value, size, color, center=False):
            lines = value.split("\n")
            anchor = "middle" if center else "start"
            svg.append(f'<text x="{x}" y="{y}" fill="{color}" font-family="{FONT}" font-size="{size}" text-anchor="{anchor}">')
            for k, line in enumerate(lines):
                svg.append(f'<tspan x="{x}" dy="{0 if k == 0 else size + 5}">{escape(line)}</tspan>')
            svg.append('</text>')

        # Containers behind connectors; child boxes in front of connectors.
        for key, x, y, w, h, value, kind, color in self.shapes:
            if kind == "container":
                fill = "#f4f7fd" if color == BLUE else "#f0faf6"
                svg.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill}" stroke="{color}" stroke-width="2"/>')
        for n, (points, value, at, color, dashed, arrow) in enumerate(self.edges):
            coords = " ".join(f"{x},{y}" for x, y in points)
            marker = f' marker-end="url(#arrow{[INK, BLUE, GREEN].index(color)})"' if arrow else ""
            dash = ' stroke-dasharray="6 5"' if dashed else ""
            svg.append(f'<polyline points="{coords}" fill="none" stroke="{color}" stroke-width="2"{dash}{marker}/>')
            style = f"edgeStyle=none;rounded=0;strokeColor={color};endArrow={'block' if arrow else 'none'};dashed={int(dashed)};"
            c = ET.SubElement(root, "mxCell", id=f"edge-{n}", style=style, edge="1", parent="1")
            g = ET.SubElement(c, "mxGeometry", relative="1", **{"as": "geometry"})
            ET.SubElement(g, "mxPoint", x=str(points[0][0]), y=str(points[0][1]), **{"as": "sourcePoint"})
            ET.SubElement(g, "mxPoint", x=str(points[-1][0]), y=str(points[-1][1]), **{"as": "targetPoint"})
            if len(points) > 2:
                arr = ET.SubElement(g, "Array", **{"as": "points"})
                for x, y in points[1:-1]:
                    ET.SubElement(arr, "mxPoint", x=str(x), y=str(y))
            if value:
                x, y = at
                self.labels.append((x, y, value, 15, color, True))

        for key, x, y, w, h, value, kind, color in self.shapes:
            fill = "#f4f7fd" if color == BLUE else "#f0faf6" if color == GREEN else "#f9fafc"
            style = f"whiteSpace=wrap;html=0;strokeWidth=2;strokeColor={color};fillColor={fill};fontSize=19;fontColor={color};fontFamily=Helvetica;"
            if kind == "external":
                svg.append(f'<polygon points="{x+18},{y} {x+w-18},{y} {x+w},{y+h/2} {x+w-18},{y+h} {x+18},{y+h} {x},{y+h/2}" fill="{fill}" stroke="{color}" stroke-width="2"/>')
                style += "shape=hexagon;"
            elif kind == "store":
                svg.append(f'<path d="M{x},{y+12} C{x},{y-4} {x+w},{y-4} {x+w},{y+12} L{x+w},{y+h-12} C{x+w},{y+h+4} {x},{y+h+4} {x},{y+h-12} Z M{x},{y+12} C{x},{y+28} {x+w},{y+28} {x+w},{y+12}" fill="{fill}" stroke="{color}" stroke-width="2"/>')
                style += "shape=cylinder;"
            elif kind != "container":
                rx = 14 if kind == "module" else 0
                svg.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" fill="{fill}" stroke="{color}" stroke-width="2"/>')
                style += f"rounded={int(kind == 'module')};"
            if kind == "container":
                text(x + w/2, y + 28, value, 19, color, True)
                style += "verticalAlign=top;spacingTop=9;"
            else:
                lines = len(value.split("\n"))
                text(x + w/2, y + h/2 - (lines-1)*12 + 6 + (8 if kind == "store" else 0), value, 19, color, True)
                if kind == "store":
                    style += "spacingTop=12;"
            parent = "1"
            cx, cy = x, y
            if kind == "module":
                for pk, px, py, pw, ph, pv, pt, pc in self.shapes:
                    if pt == "container" and px <= x and py <= y and x+w <= px+pw and y+h <= py+ph:
                        parent, cx, cy = pk, x-px, y-py
            cell(key, value, style, cx, cy, w, h, parent)

        for i, (x, y, value, size, color, center) in enumerate(self.labels):
            text(x, y, value, size, color, center)
            # Editable text uses the same position and line breaks as SVG.
            w = min(self.width-50, max(160, max(map(len, value.split("\n")))*size*0.82))
            h = len(value.split("\n"))*(size+5)+4
            cell(f"label-{i}", value, f"text;html=0;whiteSpace=wrap;align={'center' if center else 'left'};fontSize={size};fontColor={color};", x-w/2 if center else x, y-size, w, h)
        svg.append('</svg>')
        (ROOT / f"{self.name}.svg").write_text("\n".join(svg), encoding="utf-8")
        ET.indent(mx)
        ET.ElementTree(mx).write(ROOT / f"{self.name}.drawio", encoding="utf-8", xml_declaration=True)


def component(option):
    a = option == "A"
    color = BLUE if a else GREEN
    f = Figure(f"{option.lower()}-components", f"DP44 {option} | " + ("응답별 독립 음성 준비" if a else "현재 발화에 연결한 음성 준비"),
               "검정: 공통 책임   파랑: A 고유   초록: B 고유   /   로컬 논리 Component이며 개별 process를 뜻하지 않음", 1600, 1160, color)
    f.shape("mic", 40, 150, 170, 70, "Microphone", "external")
    f.shape("input", 280, 150, 270, 70, "Voice Input\nController")
    f.shape("dp2", 700, 150, 330, 70, "Request Interpretation\nService (DP2)", "external")
    f.shape("registry", 1180, 150, 280, 70, "Response Registry")
    f.shape("agent", 1180, 300, 280, 70, "Downstream Agent", "external")
    f.shape("ui", 1180, 460, 280, 70, "UI Renderer")
    f.shape("history", 40, 450, 260, 75, "Delivery History")
    f.shape("playout", 280, 900, 270, 80, "Audio Playout")
    f.shape("adapter", 780, 900, 330, 80, "S2S Runtime Adapter")
    f.shape("s2s", 1180, 900, 280, 80, "S2S Runtime", "external")
    f.edge([(210,185),(280,185)], "음성", (245,172))
    f.edge([(550,185),(700,185)], "6 입력 확정", (625,172))
    f.edge([(1030,185),(1180,185)], "7 채택 정정", (1105,172))
    f.edge([(1320,300),(1320,220)], "1 내용", (1370,263))
    f.edge([(1460,185),(1530,185),(1530,495),(1460,495)], "독립 표시", (1520,410))
    f.edge([(1320,530),(1320,560),(170,560),(170,525)], "표시 확인", (680,550), dashed=True)
    f.edge([(300,488),(650,488),(650,260),(840,260),(840,220)], "5 입력 당시 제시 근거", (640,290), dashed=True)
    f.edge([(280,940),(150,940),(150,525)], "10 실제 출력 범위", (154,840), dashed=True)
    f.edge([(1110,925),(1180,925)], "생성, 취소", (1145,912))
    f.edge([(1180,960),(1110,960)], "audio, usage", (1145,995), dashed=True)
    if a:
        f.shape("preparation", 780, 450, 330, 80, "Speech Preparation\nCoordinator", color=color)
        f.shape("store", 780, 680, 330, 85, "Prepared Audio Store", "store", color)
        f.shape("playback", 280, 680, 330, 85, "Speech Playback\nCoordinator", color=color)
        f.edge([(1180,185),(1140,185),(1140,400),(945,400),(945,450)], "1 등록 / 8 정정", (1030,388), color)
        f.edge([(1110,490),(1150,490),(1150,860),(1000,860),(1000,900)], "1 생성 / 8 취소", (1160,825), color)
        f.edge([(810,900),(730,900),(730,490),(780,490)], "2 audio, usage", (720,625), color, True)
        f.edge([(945,530),(945,680)], "2 판본 검사 후 보관", (1030,625), color)
        f.edge([(610,695),(780,695)], "9 범위 요청", (695,682), color)
        f.edge([(780,743),(610,743)], "유효 audio 참조", (695,730), color, True)
        f.edge([(415,220),(415,680)], "3 발화 시작 → 출력 차단", (432,350))
        f.edge([(1320,220),(1490,220),(1490,600),(535,600),(535,680)], "7 정정 반영 / 9 발화 선택", (1215,590))
        f.edge([(415,765),(415,900)], "4 중단 / 9 재생", (505,840), color)
        f.label(790, 1060, "준비 job과 audio는 발화 전환 뒤에도 유지 가능", 19, color, True)
    else:
        f.shape("turn", 260, 625, 870, 200, "Speech Turn Controller", "container", color)
        f.shape("execution", 300, 695, 310, 75, "Turn Execution", "module", color)
        f.shape("buffer", 780, 695, 310, 75, "Turn Audio Buffer", "module", color)
        f.edge([(415,220),(415,695)], "3 발화 시작 → 출력 차단", (432,350))
        f.edge([(1320,220),(1490,220),(1490,590),(535,590),(535,625)], "7 정정 반영 / 9 발화 선택", (1200,580))
        f.edge([(610,730),(680,730),(680,865),(895,865),(895,900)], "9 선택 후 생성 / 전환 시 취소", (790,853), color)
        f.edge([(1040,900),(1040,845),(1145,845),(1145,795),(610,795),(610,755)], "audio, usage 반환", (990,786), color, True)
        f.edge([(610,720),(780,720)], "9 유효 audio", (695,706), color)
        f.edge([(935,770),(935,805),(740,805),(740,885),(485,885),(485,900)], "9 audio 공급", (660,872), color)
        f.edge([(380,770),(380,900)], "4 중단 / 9 play", (425,840), color)
        f.label(940, 405, "Q1, R1은 선택 전까지 텍스트로 대기", 19, color, True)
        f.label(790, 1060, "미선택 항목에는 음성 생성 실행과 준비 audio가 없음", 19, color, True)
    f.label(35, 1115, "번호는 본문 사건 표와 대응. 입력용 S2S 호출은 공통 입력 확대도 참조. UI와 음성은 독립.", 18, MUTED)
    f.save()


def sequence(option):
    a = option == "A"
    color = BLUE if a else GREEN
    f = Figure(f"{option.lower()}-sequence", f"DP44 {option} | 같은 사용자 개입의 호출 흐름",
               "사건 번호는 본문 1~12. 세로 간격은 실측 시간이 아님. 입력용 S2S 호출은 공통 입력 확대도 참조.", 2490, 1480, color)
    names = ["Voice Input\nController", "Request Interpretation\nService (DP2)", "Response\nRegistry",
             "Speech Preparation\nCoordinator" if a else "Speech Turn\nController",
             "Prepared Audio\nStore" if a else "Turn Audio Buffer\n(Turn Controller 소유)",
             "S2S Runtime\nAdapter", "S2S\nRuntime",
             "Speech Playback\nCoordinator" if a else "",
             "Audio\nPlayout", "Delivery\nHistory"]
    xs = [130 + i*245 for i in range(len(names))]
    for i, (x, name) in enumerate(zip(xs, names)):
        if not name:
            continue
        c = color if i in (3,4,7) else INK
        kind = "store" if i == 4 and a else "module" if i == 4 else "component"
        f.shape(f"lane-{i}", x-108, 120, 216, 70, name, kind, c)
        f.edge([(x,190),(x,1400)], dashed=True, arrow=False, color=INK)
    def msg(y, src, dst, label, diff=False, dashed=False):
        f.edge([(xs[src],y),(xs[dst],y)], label, ((xs[src]+xs[dst])/2,y-11), color if diff else INK, dashed)
    msg(245,2,3,"1 Q1, R1-v1 등록" if a else "1 Q1, R1-v1 텍스트 대기",True)
    if a:
        msg(290,3,5,"1 Q1, R1 각각 생성 요청",True)
        msg(335,5,6,"generate_audio × 2",True)
        msg(380,6,5,"2 audio_chunk / status / usage",True,True)
        msg(425,5,3,"2 판본별 결과 반환",True,True)
        msg(470,3,4,"2 유효 chunk 보관",True)
    else:
        f.label(980,350,"S2S 호출 없음: 아직 현재 발화로 선택되지 않음",18,color,True)
    msg(520,0,7 if a else 3,"3 사용자 발화 시작 → 출력 차단")
    msg(565,7 if a else 3,8,"4 halt, 출력 버전 변경")
    msg(610,8,9,"5 실제 정지 위치, 중첩 출력 기록",dashed=True)
    msg(660,9,1,"5 입력 시점 UI/audio 제시 근거",dashed=True)
    msg(710,0,1,"6 사용자 발화 종료, 입력 확정")
    msg(765,1,2,"7 보고서 정정 / Q1 유지, 우선 / R0 보존",dashed=True)
    msg(820,2,3,"8 R1-v1 무효화, Q1 유지",True)
    if a:
        msg(860,3,4,"8 R1-v1 보관 audio 폐기",True)
        msg(905,3,5,"8 진행 중이면 생성 취소",True)
        msg(950,5,6,"cancel(R1-v1)",True)
        msg(995,2,7,"9 Q1 발화 허가")
        msg(1040,7,4,"9 Q1 유효 audio 요청",True)
        msg(1085,4,7,"audio 참조, 준비 범위",True,True)
        msg(1130,7,8,"9 준비 audio 참조로 play",True)
    else:
        msg(875,2,3,"9 Q1 선택, 생성 시작 허가",True)
        msg(920,3,5,"generate_audio(Q1)",True)
        msg(965,5,6,"S2S 호출",True)
        msg(1010,6,5,"첫 유효 audio",True,True)
        msg(1055,5,3,"audio 반환",True,True)
        msg(1100,3,4,"유효 chunk 보관",True)
        msg(1150,3,8,"9 buffer 참조, 첫 유효 범위 play",True)
    msg(1210,8,9,"10 Q1 실제 출력 범위 갱신",dashed=True)
    msg(1270,0,1,"11 다음 사용자 입력 ‘응’")
    msg(1330,9,1,"11 입력 당시 제시 근거, 질문 판본",dashed=True)
    f.label(35,1445,"12 새 결과, R0 재개는 본문 표 참조. 생성 취소의 완료, usage 회수는 재생 차단과 별도로 진행.",18,MUTED)
    f.save()


def input_flow():
    f = Figure("common-input", "DP44 공통 입력 경로 | 발화 감지와 S2S 입력은 재생 중에도 계속",
               "입력 생산, 발화 종료 판단은 A/B 동일. 중단의 빠른 경로는 모델 응답, DP2 완료를 기다리지 않음.", 1500, 790, INK)
    for key,x,y,w,h,value,kind in [
        ("mic",40,150,190,70,"Microphone","external"),
        ("input",330,150,290,70,"Voice Input Controller","component"),
        ("adapter",760,150,290,70,"S2S Runtime Adapter","component"),
        ("s2s",1170,150,270,70,"S2S Runtime","external"),
        ("control",40,440,290,85,"Speech Playback Coordinator\n/ Speech Turn Controller","component"),
        ("playout",40,640,290,70,"Audio Playout","component"),
        ("history",440,640,290,70,"Delivery History","component"),
        ("dp2",1020,440,370,85,"Request Interpretation Service\n(DP2)","external")]:
        f.shape(key,x,y,w,h,value,kind)
    f.edge([(230,185),(330,185)],"음성",(280,170))
    f.edge([(620,170),(760,170)],"append / commit",(690,155))
    f.edge([(1050,170),(1170,170)],"입력 세션",(1110,155))
    f.edge([(1170,205),(1050,205)],"입력 결과",(1110,240),dashed=True)
    f.edge([(760,205),(620,205)],"전사, 입력 판본",(690,240),dashed=True)
    f.edge([(420,220),(420,330),(185,330),(185,440)],"3 로컬 발화 시작 감지",(260,315))
    f.edge([(185,525),(185,640)],"4 halt",(135,595))
    f.edge([(260,640),(260,525)],"정지 위치",(310,570),dashed=True)
    f.edge([(330,675),(440,675)],"5 출력 기록",(385,662),dashed=True)
    f.edge([(585,640),(585,570),(1180,570),(1180,525)],"5 제시 근거 snapshot",(925,558),dashed=True)
    f.edge([(520,220),(520,380),(1205,380),(1205,440)],"6 발화 종료 + 입력 확정",(925,367))
    f.edge([(40,675),(15,675),(15,270),(360,270),(360,220)],"재생 참조 신호 → 에코 제거",(185,257),dashed=True)
    f.label(35,760,"외부 detector를 쓰는 경우도 감지 경로를 양안 동일하게 비교. 원문, 전사, 불확실성은 입력 계약에 따라 보존.",18,MUTED)
    f.save()


if __name__ == "__main__":
    input_flow()
    for option in ("A", "B"):
        component(option)
        sequence(option)
