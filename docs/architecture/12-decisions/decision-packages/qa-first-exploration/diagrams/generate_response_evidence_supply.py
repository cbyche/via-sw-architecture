"""Generate paired documentation figures; no VIA execution or QA measurement."""

from pathlib import Path
import xml.etree.ElementTree as ET


OUT = Path(__file__).resolve().parent
WIDTH, HEIGHT = 1800, 1400
BLACK, BLUE, GREEN = "#223047", "#2463ad", "#287252"
svg = ET.Element("svg", xmlns="http://www.w3.org/2000/svg", width=str(WIDTH),
                 height=str(HEIGHT), viewBox=f"0 0 {WIDTH} {HEIGHT}")
ET.SubElement(svg, "title").text = "현재 요청의 근거 공급: 유지 문맥과 요청별 구성"
defs = ET.SubElement(svg, "defs")
for name, color in [("common", BLACK), ("a", BLUE), ("b", GREEN)]:
    marker = ET.SubElement(defs, "marker", id=name, markerWidth="10", markerHeight="10",
                           refX="9", refY="5", orient="auto", markerUnits="userSpaceOnUse")
    ET.SubElement(marker, "path", d="M 0 0 L 10 5 L 0 10 Z", fill=color)

mxfile = ET.Element("mxfile", host="app.diagrams.net", version="26.0.0")
diagram = ET.SubElement(mxfile, "diagram", name="근거 공급 비교", id="response-evidence")
model = ET.SubElement(diagram, "mxGraphModel", page="1", pageWidth=str(WIDTH), pageHeight=str(HEIGHT))
root = ET.SubElement(model, "root")
ET.SubElement(root, "mxCell", id="0")
ET.SubElement(root, "mxCell", id="1", parent="0")
serial = 1


def cell(value, style, **attrs):
    global serial
    serial += 1
    return ET.SubElement(root, "mxCell", id=str(serial), value=value, style=style,
                         parent="1", **attrs)


def rect(x, y, w, h, fill="#ffffff", stroke=BLACK, dash=False):
    attrs = dict(x=str(x), y=str(y), width=str(w), height=str(h), rx="12",
                 fill=fill, stroke=stroke, **{"stroke-width": "2"})
    if dash:
        attrs["stroke-dasharray"] = "8 6"
    ET.SubElement(svg, "rect", attrs)
    c = cell("", f"rounded=1;arcSize=8;fillColor={fill};strokeColor={stroke};"
             f"strokeWidth=2;dashed={int(dash)};", vertex="1")
    ET.SubElement(c, "mxGeometry", x=str(x), y=str(y), width=str(w), height=str(h),
                  **{"as": "geometry"})


def text(x, y, value, width=700, size=24, color=BLACK, bold=False, align="left"):
    anchor = {"left": "start", "center": "middle", "right": "end"}[align]
    tx = x + (width / 2 if align == "center" else width if align == "right" else 0)
    t = ET.SubElement(svg, "text", x=str(tx), y=str(y), fill=color,
                      **{"font-family": "Arial, Apple SD Gothic Neo, sans-serif",
                         "font-size": str(size), "font-weight": "700" if bold else "400",
                         "text-anchor": anchor, "data-width": str(width)})
    t.text = value
    c = cell(value, f"text;html=0;strokeColor=none;fillColor=none;align={align};"
             f"verticalAlign=middle;fontFamily=Arial;fontSize={size};fontColor={color};"
             f"fontStyle={int(bold)};spacing=0;whiteSpace=wrap;", vertex="1")
    ET.SubElement(c, "mxGeometry", x=str(x), y=str(y-size), width=str(width),
                  height=str(size+8), **{"as": "geometry"})


def edge(points, color, marker):
    coords = " ".join(f"{x},{y}" for x, y in points)
    ET.SubElement(svg, "polyline", points=coords, fill="none", stroke=color,
                  **{"stroke-width": "2.5", "marker-end": f"url(#{marker})"})
    c = cell("", f"edgeStyle=none;endArrow=block;endFill=1;strokeColor={color};strokeWidth=2.5;",
             edge="1")
    geo = ET.SubElement(c, "mxGeometry", relative="1", **{"as": "geometry"})
    for kind, (x, y) in [("sourcePoint", points[0]), ("targetPoint", points[-1])]:
        ET.SubElement(geo, "mxPoint", x=str(x), y=str(y), **{"as": kind})
    if len(points) > 2:
        arr = ET.SubElement(geo, "Array", **{"as": "points"})
        for x, y in points[1:-1]:
            ET.SubElement(arr, "mxPoint", x=str(x), y=str(y))


rect(0, 0, WIDTH, HEIGHT, stroke="#ffffff")
text(60, 63, "현재 요청을 이해할 근거를 어떻게 공급할 것인가", width=1680, size=38, bold=True)
text(60, 111, "공통 원본과 권한에서 비교  |  파생 문맥의 수명과 첫 판단 전후의 근거 취득이 다름", width=1680, size=25)
rect(60, 155, 1680, 90, fill="#f1f4f8")
text(80, 190, "공통 원본: 입력과 화면 관측 / 대화 / 업무 사실 / 실제 텍스트 및 음성 전달 범위", width=1640, size=25, bold=True)
text(80, 225, "Interaction Manager와 각 소유자가 기록 → State Store  |  모든 모델 호출은 공통 Model Access 사용", width=1640, size=22)

rect(60, 265, 810, 915, fill="#f6f9ff", stroke="#bed0e6")
rect(930, 265, 810, 915, fill="#f5faf7", stroke="#bad5c7")
text(90, 300, "A  대화별 유지 문맥 우선", size=29, color=BLUE, bold=True)
text(960, 300, "B  요청별 근거 구성 우선", size=29, color=GREEN, bold=True)

edge([(465, 245), (465, 350)], BLUE, "a")
edge([(1335, 245), (1335, 350)], GREEN, "b")
text(490, 334, "1  사건 반영", width=330, size=21, color=BLUE)
text(1360, 334, "1  요청 + 최소 문맥", width=340, size=21, color=GREEN)

rect(150, 350, 630, 140, stroke=BLUE)
text(170, 387, "Context Manager", width=590, size=27, color=BLUE, bold=True)
text(170, 426, "대화별 view 유지: 최근 원문과 선택된 근거", width=590, size=24)
text(170, 464, "이전 view 갱신 / 필요 시 축약 / 출처 보존", width=590, size=22)
rect(1020, 350, 630, 140, stroke=GREEN)
text(1040, 387, "Request Interpreter", width=590, size=27, color=GREEN, bold=True)
text(1040, 426, "최소 문맥에서 현재 요청 판단", width=590, size=24)
text(1040, 464, "최근 원문 / 채택 anchor / 명시 선택과 cache", width=590, size=22)

edge([(465, 490), (465, 580)], BLUE, "a")
edge([(1335, 490), (1335, 580)], GREEN, "b")
text(490, 539, "2  준비된 view로 판단", width=340, size=21, color=BLUE)
text(1360, 539, "2  부족하면 근거 조회", width=340, size=21, color=GREEN)
rect(150, 580, 630, 120, stroke=BLUE)
text(170, 620, "Request Interpreter", width=590, size=27, color=BLUE, bold=True)
text(170, 666, "현재 요청 + view → 의미와 대응 제안", width=590, size=24)
rect(1020, 580, 630, 120, stroke=GREEN, dash=True)
text(1040, 620, "Context Manager", width=590, size=27, color=GREEN, bold=True)
text(1040, 666, "현재 요청의 근거 읽기 → 원본과 출처 반환", width=590, size=23)

edge([(465, 700), (465, 800)], BLUE, "a")
edge([(1335, 700), (1335, 800)], GREEN, "b")
text(490, 755, "3  부족하면 보완", width=340, size=21, color=BLUE)
text(1360, 755, "3  근거를 넣어 재판단", width=340, size=21, color=GREEN)
rect(150, 800, 630, 180, stroke=BLUE, dash=True)
text(170, 840, "Context Manager → Request Interpreter", width=590, size=25, color=BLUE, bold=True)
text(170, 886, "제한된 원본 읽기와 반환 후 재판단", width=590, size=24)
text(170, 927, "cache / 병렬 읽기 / 부분 갱신 허용", width=590, size=23)
text(170, 960, "재사용할 근거는 대화 view에 반영 가능", width=590, size=22)
rect(1020, 800, 630, 180, stroke=GREEN, dash=True)
text(1040, 840, "Request Interpreter", width=590, size=27, color=GREEN, bold=True)
text(1040, 886, "요청별 임시 view로 의미와 대응 제안", width=590, size=24)
text(1040, 927, "추가 조회 / 병렬 읽기 / 부분 갱신 허용", width=590, size=23)
text(1040, 960, "탐색 한도와 미확인 범위를 함께 관리", width=590, size=22)

rect(150, 1030, 630, 80, fill="#e9f0fb", stroke="#bed0e6")
text(170, 1061, "상태: 대화별 파생 view가 다음 요청에 이어짐", width=590, size=22, bold=True)
text(170, 1094, "원본과 채택 상태가 권위. view는 갱신 / 재구성 가능", width=590, size=21)
rect(1020, 1030, 630, 80, fill="#e8f3ed", stroke="#bad5c7")
text(1040, 1061, "상태: 요청 view 종료. 채택 상태와 cache는 유지", width=590, size=22, bold=True)
text(1040, 1094, "다음 요청이 같은 전체 탐색 문맥을 기본 상속하지 않음", width=590, size=21)

edge([(150, 640), (105, 640), (105, 1190), (400, 1190), (400, 1210)], BLUE, "a")
edge([(780, 890), (825, 890), (825, 1190), (600, 1190), (600, 1210)], BLUE, "a")
edge([(1650, 420), (1695, 420), (1695, 1190), (1500, 1190), (1500, 1210)], GREEN, "b")
edge([(1020, 890), (975, 890), (975, 1190), (1150, 1190), (1150, 1210)], GREEN, "b")
text(150, 1152, "근거가 충분하면 3 생략 → 공통 채택", width=630, size=23, color=BLUE)
text(1020, 1152, "근거가 충분하면 2 / 3 생략 → 공통 채택", width=630, size=23, color=GREEN)

rect(60, 1210, 1680, 120, fill="#f1f4f8")
text(80, 1245, "Request Controller → Response Manager", width=1640, size=27, bold=True)
text(80, 1283, "4  요청 revision과 권한 확인 후 채택    5  첫 내용의 실제 UI 표시 → 상세 표시와 음성 요약 준비", width=1640, size=24)
text(80, 1313, "QA01은 첫 실제 내용 표시 / QA03은 VIA 인과 경로 / QA05는 사전 준비와 폐기를 포함한 전체 비용", width=1640, size=21)
text(60, 1374, "실선: 기본 단계  |  점선 상자: 근거 부족 시 단계  |  색: 대안 구분  |  논리 구조이며 프로세스 수와 품질 우열을 뜻하지 않음", width=1680, size=21)

ET.indent(mxfile)
ET.ElementTree(svg).write(OUT / "response-evidence-supply.svg", encoding="utf-8", xml_declaration=True)
ET.ElementTree(mxfile).write(OUT / "response-evidence-supply.drawio", encoding="utf-8", xml_declaration=True)
print("Generated paired SVG and draw.io documentation figures.")
