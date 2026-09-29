#!/usr/bin/env python3
"""Generate editable draw.io sources and GitHub-preview SVGs for VIA target Architecture."""

from __future__ import annotations

from dataclasses import dataclass, field
from html import escape
from pathlib import Path
import xml.etree.ElementTree as ET


ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT / "docs/architecture/12-decisions/target-architecture/diagrams"

COLORS = {
    "ink": "#172033",
    "muted": "#526178",
    "line": "#64748B",
    "blue": "#2563EB",
    "blue_fill": "#EAF2FF",
    "cyan": "#0891B2",
    "cyan_fill": "#E6F8FB",
    "purple": "#7C3AED",
    "purple_fill": "#F3ECFF",
    "green": "#059669",
    "green_fill": "#E9F9F3",
    "orange": "#EA580C",
    "orange_fill": "#FFF1E8",
    "amber": "#D97706",
    "amber_fill": "#FFF8E1",
    "red": "#DC2626",
    "red_fill": "#FDECEC",
    "gray": "#475569",
    "gray_fill": "#F1F5F9",
    "lane": "#F8FAFC",
    "white": "#FFFFFF",
}


@dataclass
class Lane:
    id: str
    x: int
    y: int
    w: int
    h: int
    title: str
    fill: str = COLORS["lane"]
    stroke: str = "#CBD5E1"
    dashed: bool = False


@dataclass
class Box:
    id: str
    x: int
    y: int
    w: int
    h: int
    title: str
    lines: tuple[str, ...] = ()
    fill: str = COLORS["white"]
    stroke: str = COLORS["line"]
    kind: str = "rounded"
    badge: str | None = None


@dataclass
class Edge:
    id: str
    points: tuple[tuple[int, int], ...]
    source: str
    target: str
    label: str = ""
    label_x: int | None = None
    label_y: int | None = None
    color: str = COLORS["line"]
    dashed: bool = False
    width: int = 2


@dataclass
class Caption:
    x: int
    y: int
    text: str
    size: int = 14
    color: str = COLORS["muted"]
    anchor: str = "start"
    weight: int = 500


@dataclass
class Diagram:
    slug: str
    title: str
    subtitle: str
    width: int
    height: int
    lanes: list[Lane] = field(default_factory=list)
    boxes: list[Box] = field(default_factory=list)
    edges: list[Edge] = field(default_factory=list)
    captions: list[Caption] = field(default_factory=list)


def lane(diagram: Diagram, *args, **kwargs) -> None:
    diagram.lanes.append(Lane(*args, **kwargs))


def box(diagram: Diagram, *args, **kwargs) -> None:
    diagram.boxes.append(Box(*args, **kwargs))


def edge(diagram: Diagram, *args, **kwargs) -> None:
    diagram.edges.append(Edge(*args, **kwargs))


def caption(diagram: Diagram, *args, **kwargs) -> None:
    diagram.captions.append(Caption(*args, **kwargs))


def svg_text_lines(item: Box) -> str:
    cx = item.x + item.w / 2
    title_y = item.y + (item.h / 2) - (len(item.lines) * 10) + 1
    parts = [
        f'<text x="{cx}" y="{title_y}" text-anchor="middle" '
        f'font-size="17" font-weight="700" fill="{COLORS["ink"]}">{escape(item.title)}</text>'
    ]
    for index, line_text in enumerate(item.lines):
        parts.append(
            f'<text x="{cx}" y="{title_y + 25 + index * 20}" text-anchor="middle" '
            f'font-size="13" font-weight="500" fill="{COLORS["muted"]}">{escape(line_text)}</text>'
        )
    if item.badge:
        badge_w = max(62, len(item.badge) * 7 + 18)
        bx = item.x + item.w - badge_w - 10
        by = item.y + 9
        parts.append(
            f'<rect x="{bx}" y="{by}" width="{badge_w}" height="22" rx="11" '
            f'fill="{item.stroke}" opacity="0.12"/>'
        )
        parts.append(
            f'<text x="{bx + badge_w / 2}" y="{by + 15}" text-anchor="middle" '
            f'font-size="11" font-weight="700" fill="{item.stroke}">{escape(item.badge)}</text>'
        )
    return "".join(parts)


def render_svg(diagram: Diagram) -> str:
    out = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{diagram.width}" height="{diagram.height}" '
        f'viewBox="0 0 {diagram.width} {diagram.height}" role="img" aria-labelledby="title desc">',
        f'<title id="title">{escape(diagram.title)}</title>',
        f'<desc id="desc">{escape(diagram.subtitle)}</desc>',
        '<defs>',
        '<marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">'
        f'<path d="M 0 0 L 10 5 L 0 10 z" fill="{COLORS["line"]}"/></marker>',
        '<marker id="arrow-blue" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">'
        f'<path d="M 0 0 L 10 5 L 0 10 z" fill="{COLORS["blue"]}"/></marker>',
        '<filter id="shadow" x="-10%" y="-10%" width="120%" height="140%">'
        '<feDropShadow dx="0" dy="2" stdDeviation="3" flood-color="#0F172A" flood-opacity="0.10"/>'
        '</filter>',
        '</defs>',
        f'<rect width="{diagram.width}" height="{diagram.height}" fill="#FFFFFF"/>',
        f'<text x="48" y="43" font-family="Inter, Arial, sans-serif" font-size="27" font-weight="750" fill="{COLORS["ink"]}">{escape(diagram.title)}</text>',
        f'<text x="48" y="70" font-family="Inter, Arial, sans-serif" font-size="14" font-weight="500" fill="{COLORS["muted"]}">{escape(diagram.subtitle)}</text>',
    ]

    for item in diagram.lanes:
        dash = ' stroke-dasharray="8 6"' if item.dashed else ""
        out.append(
            f'<rect x="{item.x}" y="{item.y}" width="{item.w}" height="{item.h}" rx="18" '
            f'fill="{item.fill}" stroke="{item.stroke}" stroke-width="1.5"{dash}/>'
        )
        out.append(
            f'<text x="{item.x + 18}" y="{item.y + 28}" font-family="Inter, Arial, sans-serif" '
            f'font-size="14" font-weight="750" fill="{COLORS["muted"]}">{escape(item.title)}</text>'
        )

    for item in diagram.edges:
        dash = ' stroke-dasharray="8 6"' if item.dashed else ""
        marker = "url(#arrow-blue)" if item.color == COLORS["blue"] else "url(#arrow)"
        points = " ".join(f"{x},{y}" for x, y in item.points)
        out.append(
            f'<polyline points="{points}" fill="none" stroke="{item.color}" stroke-width="{item.width}" '
            f'stroke-linejoin="round" stroke-linecap="round" marker-end="{marker}"{dash}/>'
        )
        if item.label and item.label_x is not None and item.label_y is not None:
            label_w = max(62, len(item.label) * 7 + 18)
            out.append(
                f'<rect x="{item.label_x - label_w / 2}" y="{item.label_y - 15}" width="{label_w}" height="22" '
                f'rx="8" fill="#FFFFFF" stroke="#E2E8F0"/>'
            )
            out.append(
                f'<text x="{item.label_x}" y="{item.label_y}" text-anchor="middle" '
                f'font-family="Inter, Arial, sans-serif" font-size="11" font-weight="650" fill="{COLORS["muted"]}">{escape(item.label)}</text>'
            )

    for item in diagram.boxes:
        if item.kind == "diamond":
            cx = item.x + item.w / 2
            cy = item.y + item.h / 2
            shape = f'<polygon points="{cx},{item.y} {item.x + item.w},{cy} {cx},{item.y + item.h} {item.x},{cy}"'
            shape += f' fill="{item.fill}" stroke="{item.stroke}" stroke-width="2" filter="url(#shadow)"/>'
        else:
            dash = ' stroke-dasharray="7 5"' if item.kind == "dashed" else ""
            shape = (
                f'<rect x="{item.x}" y="{item.y}" width="{item.w}" height="{item.h}" rx="16" '
                f'fill="{item.fill}" stroke="{item.stroke}" stroke-width="2" filter="url(#shadow)"{dash}/>'
            )
        out.append(shape)
        out.append(f'<g font-family="Inter, Arial, sans-serif">{svg_text_lines(item)}</g>')

    for item in diagram.captions:
        out.append(
            f'<text x="{item.x}" y="{item.y}" text-anchor="{item.anchor}" '
            f'font-family="Inter, Arial, sans-serif" font-size="{item.size}" font-weight="{item.weight}" '
            f'fill="{item.color}">{escape(item.text)}</text>'
        )

    out.append(
        f'<text x="{diagram.width - 38}" y="{diagram.height - 20}" text-anchor="end" '
        f'font-family="Inter, Arial, sans-serif" font-size="11" fill="#94A3B8">Editable source: {diagram.slug}.drawio</text>'
    )
    out.append('</svg>')
    return "\n".join(out) + "\n"


def box_value(item: Box) -> str:
    lines = "".join(f"<div style='font-size:12px;color:#526178'>{escape(line)}</div>" for line in item.lines)
    badge = f"<div style='font-size:10px;color:{item.stroke};font-weight:bold'>{escape(item.badge)}</div>" if item.badge else ""
    return f"<div style='font-size:16px;font-weight:bold'>{escape(item.title)}</div>{lines}{badge}"


def render_drawio(diagram: Diagram) -> str:
    mxfile = ET.Element(
        "mxfile",
        {"host": "app.diagrams.net", "modified": "2026-09-29T00:00:00.000Z", "agent": "Codex", "version": "24.7.17", "type": "device"},
    )
    page = ET.SubElement(mxfile, "diagram", {"id": diagram.slug, "name": diagram.title})
    model = ET.SubElement(
        page,
        "mxGraphModel",
        {
            "dx": "1600", "dy": "900", "grid": "1", "gridSize": "10", "guides": "1", "tooltips": "1",
            "connect": "1", "arrows": "1", "fold": "1", "page": "1", "pageScale": "1",
            "pageWidth": str(diagram.width), "pageHeight": str(diagram.height), "math": "0", "shadow": "0",
        },
    )
    root = ET.SubElement(model, "root")
    ET.SubElement(root, "mxCell", {"id": "0"})
    ET.SubElement(root, "mxCell", {"id": "1", "parent": "0"})

    for item in diagram.lanes:
        style = (
            "rounded=1;whiteSpace=wrap;html=1;verticalAlign=top;align=left;spacingTop=12;spacingLeft=12;"
            f"fontStyle=1;fontSize=14;fillColor={item.fill};strokeColor={item.stroke};"
            f"dashed={1 if item.dashed else 0};arcSize=14;"
        )
        cell = ET.SubElement(root, "mxCell", {"id": item.id, "value": item.title, "style": style, "vertex": "1", "parent": "1"})
        ET.SubElement(cell, "mxGeometry", {"x": str(item.x), "y": str(item.y), "width": str(item.w), "height": str(item.h), "as": "geometry"})

    for item in diagram.boxes:
        shape = "rhombus" if item.kind == "diamond" else "rectangle"
        style = (
            f"shape={shape};rounded={0 if item.kind == 'diamond' else 1};whiteSpace=wrap;html=1;"
            f"fillColor={item.fill};strokeColor={item.stroke};strokeWidth=2;fontColor={COLORS['ink']};"
            f"dashed={1 if item.kind == 'dashed' else 0};arcSize=14;spacing=10;"
        )
        cell = ET.SubElement(root, "mxCell", {"id": item.id, "value": box_value(item), "style": style, "vertex": "1", "parent": "1"})
        ET.SubElement(cell, "mxGeometry", {"x": str(item.x), "y": str(item.y), "width": str(item.w), "height": str(item.h), "as": "geometry"})

    for item in diagram.edges:
        style = (
            "edgeStyle=orthogonalEdgeStyle;rounded=1;orthogonalLoop=1;jettySize=auto;html=1;"
            f"strokeColor={item.color};strokeWidth={item.width};dashed={1 if item.dashed else 0};"
            "endArrow=block;endFill=1;fontSize=11;labelBackgroundColor=#FFFFFF;"
        )
        cell = ET.SubElement(
            root,
            "mxCell",
            {"id": item.id, "value": item.label, "style": style, "edge": "1", "parent": "1", "source": item.source, "target": item.target},
        )
        geometry = ET.SubElement(cell, "mxGeometry", {"relative": "1", "as": "geometry"})
        if len(item.points) > 2:
            array = ET.SubElement(geometry, "Array", {"as": "points"})
            for x, y in item.points[1:-1]:
                ET.SubElement(array, "mxPoint", {"x": str(x), "y": str(y)})

    for index, item in enumerate(diagram.captions):
        style = f"text;html=1;strokeColor=none;fillColor=none;align={item.anchor};fontSize={item.size};fontColor={item.color};fontStyle={1 if item.weight >= 700 else 0};"
        cell = ET.SubElement(root, "mxCell", {"id": f"caption-{index}", "value": item.text, "style": style, "vertex": "1", "parent": "1"})
        ET.SubElement(cell, "mxGeometry", {"x": str(item.x), "y": str(item.y - item.size), "width": "420", "height": str(item.size + 12), "as": "geometry"})

    ET.indent(mxfile, space="  ")
    return '<?xml version="1.0" encoding="UTF-8"?>\n' + ET.tostring(mxfile, encoding="unicode") + "\n"


def system_overview() -> Diagram:
    d = Diagram("01-system-overview", "VIA Target Architecture — 전체 구조", "Interaction, semantic commit, Task orchestration과 외부 실행의 책임 경계", 1600, 920)
    lane(d, "lane-user", 40, 92, 1520, 120, "USER INTERACTION", fill="#F8FAFC")
    lane(d, "lane-realtime", 40, 228, 1520, 210, "REAL-TIME INTERACTION", fill="#F5F9FF")
    lane(d, "lane-core", 40, 456, 1520, 255, "VIA CORE — semantic authority와 durable state", fill="#FAF8FF")
    lane(d, "lane-deps", 40, 730, 1520, 145, "DEPENDENCIES / EXTERNAL EXECUTION", fill="#FCFCFD")
    box(d, "user", 80, 118, 220, 70, "사용자", ("Voice · Text · 화면 지칭",), COLORS["white"], COLORS["blue"])
    box(d, "interaction", 95, 275, 250, 105, "Interaction Runtime", ("입력·시점별 evidence", "S2S 연결·barge-in·재생"), COLORS["blue_fill"], COLORS["blue"], badge="REAL TIME")
    box(d, "controller", 440, 270, 280, 115, "Request Controller", ("Request Graph · semantic commit", "질문 결합 · publish/dispatch admission"), COLORS["purple_fill"], COLORS["purple"], badge="AUTHORITY")
    box(d, "response", 825, 275, 250, 105, "Response Manager", ("Canonical payload", "Text · Voice · 알림 delivery"), COLORS["blue_fill"], COLORS["blue"])
    box(d, "context", 95, 520, 250, 110, "Context Manager", ("후보 탐색 · bounded read", "receipt · cache · coverage"), COLORS["cyan_fill"], COLORS["cyan"])
    box(d, "resolver", 430, 520, 250, 110, "Request Interpreter", ("semantic proposal", "field별 후보·근거·미해결"), COLORS["purple_fill"], COLORS["purple"])
    box(d, "task", 780, 515, 255, 120, "Task Manager", ("Task lifecycle", "Execution projection · 복구"), COLORS["green_fill"], COLORS["green"])
    box(d, "gateway", 1140, 515, 250, 120, "Agent Gateway", ("capability · command outbox", "event inbox · protocol adapter"), COLORS["orange_fill"], COLORS["orange"])
    box(d, "policy", 1160, 280, 210, 95, "Policy Manager", ("권한 · consent", "policy revision"), COLORS["amber_fill"], COLORS["amber"])
    box(d, "store", 1405, 280, 120, 95, "State Store", ("durable", "transactions"), COLORS["gray_fill"], COLORS["gray"])
    box(d, "s2s", 115, 760, 210, 80, "S2S 모델 1개", ("streaming speech dependency",), COLORS["gray_fill"], COLORS["gray"])
    box(d, "semantic", 450, 760, 220, 80, "Semantic LLM 1개", ("shared structured inference",), COLORS["gray_fill"], COLORS["gray"])
    box(d, "sources", 770, 760, 220, 80, "Context Sources", ("화면 · 대화 · 자료 · OS",), COLORS["gray_fill"], COLORS["gray"])
    box(d, "agents", 1160, 755, 270, 90, "Downstream Agents", ("domain reasoning · plan · tools", "실제 업무 실행"), COLORS["orange_fill"], COLORS["orange"])
    edge(d, "e-user-in", ((300, 153), (355, 153), (355, 328), (345, 328)), "user", "interaction", "입력", 355, 205, COLORS["blue"])
    edge(d, "e-output", ((825, 328), (735, 328), (735, 190), (300, 190)), "response", "user", "Text / Voice", 570, 188, COLORS["blue"])
    edge(d, "e-i-r", ((345, 328), (440, 328)), "interaction", "controller", "Input Record", 392, 316)
    edge(d, "e-r-o", ((720, 328), (825, 328)), "controller", "response", "payload", 773, 316)
    edge(d, "e-r-c", ((560, 385), (560, 455), (220, 455), (220, 520)), "controller", "context", "bounded read", 380, 445)
    edge(d, "e-c-s", ((345, 575), (430, 575)), "context", "resolver", "evidence", 388, 562)
    edge(d, "e-s-r", ((555, 520), (555, 430), (580, 430), (580, 385)), "resolver", "controller", "proposal", 602, 422)
    edge(d, "e-r-t", ((650, 385), (650, 470), (907, 470), (907, 515)), "controller", "task", "commit", 780, 462)
    edge(d, "e-t-g", ((1035, 575), (1140, 575)), "task", "gateway", "command / event", 1088, 562)
    edge(d, "e-g-a", ((1265, 635), (1265, 755)), "gateway", "agents", "typed protocol", 1315, 704)
    edge(d, "e-s-llm", ((555, 630), (555, 760)), "resolver", "semantic", "shared call", 603, 704)
    edge(d, "e-i-s2s", ((220, 380), (220, 760)), "interaction", "s2s", "one session", 268, 700)
    edge(d, "e-c-source", ((220, 630), (220, 680), (880, 680), (880, 760)), "context", "sources", "read only", 550, 671)
    edge(d, "e-policy", ((1160, 328), (1075, 328)), "policy", "response", "policy", 1117, 316, COLORS["amber"])
    edge(d, "e-store", ((1405, 328), (1385, 328), (1385, 670), (910, 670), (910, 635)), "store", "task", "durable state", 1260, 662)
    caption(d, 1495, 409, "LLM은 상태·권한·실행을 소유하지 않는다", 12, COLORS["purple"], "end", 700)
    return d


def lifecycle() -> Diagram:
    d = Diagram("02-lifecycle-and-ownership", "Conversation부터 Agent Execution까지", "서로 다른 수명과 단일 상태 소유자를 분리한다", 1600, 860)
    lane(d, "conversation-lane", 45, 100, 1510, 170, "CONVERSATION — Request Controller 소유", fill=COLORS["blue_fill"], stroke="#93C5FD")
    lane(d, "request-lane", 45, 292, 1510, 210, "TURN / REQUEST GRAPH — Request Controller 소유", fill=COLORS["purple_fill"], stroke="#C4B5FD")
    lane(d, "task-lane", 45, 525, 1510, 270, "TASK / EXECUTION — Task Manager projection + Agent source state", fill=COLORS["green_fill"], stroke="#86EFAC")
    box(d, "conversation", 85, 145, 230, 80, "Conversation", ("대화·참조 관계",), COLORS["white"], COLORS["blue"])
    box(d, "turn1", 405, 145, 210, 80, "Turn N", ("하나의 사용자 입력",), COLORS["white"], COLORS["blue"])
    box(d, "turn2", 715, 145, 210, 80, "Turn N+1", ("clarification / 정정",), COLORS["white"], COLORS["blue"])
    box(d, "response", 1130, 140, 300, 90, "Response Publication", ("실제 Text · audible prefix", "Conversation 근거로 연결"), COLORS["blue_fill"], COLORS["blue"])
    box(d, "graph", 90, 345, 240, 100, "Request Graph", ("node와 dependency edge", "durable identity"), COLORS["purple_fill"], COLORS["purple"])
    box(d, "req-a", 420, 345, 220, 100, "Request A", ("직접 응답", "Task 없음"), COLORS["white"], COLORS["purple"])
    box(d, "req-b", 720, 345, 220, 100, "Request B", ("새 업무 위임", "Task A와 연결"), COLORS["white"], COLORS["purple"])
    box(d, "pending", 1030, 345, 300, 100, "Pending User Interaction", ("clarification · consent · approval", "어느 질문의 답인지 단일 결합"), COLORS["amber_fill"], COLORS["amber"])
    box(d, "task", 180, 600, 230, 110, "VIA Task A", ("사용자 업무 identity", "Conversation보다 오래 지속 가능"), COLORS["green_fill"], COLORS["green"])
    box(d, "exec1", 520, 600, 230, 110, "Agent Execution A1", ("외부 실행 identity", "source-confirmed projection"), COLORS["orange_fill"], COLORS["orange"])
    box(d, "command", 860, 575, 230, 85, "Agent Command", ("start · follow-up · cancel",), COLORS["orange_fill"], COLORS["orange"])
    box(d, "event", 860, 685, 230, 85, "Agent Event", ("progress · question · result",), COLORS["green_fill"], COLORS["green"])
    box(d, "artifact", 1210, 600, 230, 110, "Result / Artifact", ("versioned source result", "후속 Request의 입력"), COLORS["gray_fill"], COLORS["gray"])
    edge(d, "l-e1", ((315, 185), (405, 185)), "conversation", "turn1", "contains", 360, 173)
    edge(d, "l-e2", ((615, 185), (715, 185)), "turn1", "turn2", "continues", 665, 173)
    edge(d, "l-e3", ((820, 225), (820, 270), (210, 270), (210, 345)), "turn2", "graph", "creates / updates", 520, 262)
    edge(d, "l-e4", ((330, 395), (420, 395)), "graph", "req-a")
    edge(d, "l-e5", ((330, 420), (650, 420), (650, 395), (720, 395)), "graph", "req-b", "dependency", 650, 408)
    edge(d, "l-e6", ((940, 395), (1030, 395)), "req-b", "pending", "may wait", 985, 383)
    edge(d, "l-e7", ((830, 445), (830, 520), (295, 520), (295, 600)), "req-b", "task", "commits", 560, 512)
    edge(d, "l-e8", ((410, 655), (520, 655)), "task", "exec1", "has 0..N", 465, 643)
    edge(d, "l-e9", ((750, 630), (860, 617)), "exec1", "command", "control", 805, 608)
    edge(d, "l-e10", ((860, 727), (750, 682)), "event", "exec1", "observed", 805, 700)
    edge(d, "l-e11", ((750, 655), (1210, 655)), "exec1", "artifact", "produces", 980, 643)
    edge(d, "l-e12", ((1325, 600), (1325, 500), (1280, 500), (1280, 230)), "artifact", "response", "publishes", 1373, 480)
    edge(d, "l-e13", ((1030, 370), (970, 370), (970, 185), (925, 185)), "pending", "turn2", "answered by", 972, 275, COLORS["amber"])
    caption(d, 80, 828, "Voice Connection은 이 lifecycle들과 별도이며 종료되어도 Conversation·Task는 유지된다.", 13, COLORS["muted"])
    return d


def request_resolution() -> Diagram:
    d = Diagram("03-request-resolution", "Accuracy-first Request Resolution", "모델은 후보를 제안하고 host가 evidence coverage와 revision으로 semantic commit을 결정한다", 1600, 910)
    lane(d, "resolution-main", 40, 100, 1520, 440, "PRIMARY FLOW", fill="#F8FAFC")
    lane(d, "resolution-branches", 40, 565, 1520, 285, "BOUNDED OUTCOMES", fill="#FCFCFD")
    box(d, "input", 75, 245, 190, 105, "Input Revision", ("final transcript / text", "Interaction timeline"), COLORS["blue_fill"], COLORS["blue"])
    box(d, "discover", 335, 235, 220, 125, "Candidate Discovery", ("bounded source search", "coverage · truncation · failures"), COLORS["cyan_fill"], COLORS["cyan"])
    box(d, "proposal", 630, 230, 230, 135, "Semantic Proposal", ("goal · referent · Task · handling", "경쟁 후보·배제 근거"), COLORS["purple_fill"], COLORS["purple"])
    box(d, "readiness", 955, 215, 220, 165, "Host Readiness", ("field status · coverage", "conflict · freshness · policy"), COLORS["amber_fill"], COLORS["amber"], kind="diamond", badge="HOST")
    box(d, "commit", 1290, 240, 220, 115, "Semantic Commit", ("immutable resolution", "dependency revision vector"), COLORS["green_fill"], COLORS["green"], badge="AUTHORITY")
    box(d, "more", 170, 640, 240, 105, "More Evidence", ("한정된 read 묶음", "영향 field만 재해석"), COLORS["cyan_fill"], COLORS["cyan"])
    box(d, "clarify", 510, 640, 240, 105, "Clarification", ("Pending User Interaction", "사용자만 구분할 최소 차이"), COLORS["amber_fill"], COLORS["amber"])
    box(d, "failure", 850, 640, 240, 105, "Safe Failure", ("근거 부족 · 기능 미지원", "확인 불가를 명시"), COLORS["red_fill"], COLORS["red"])
    box(d, "handle", 1210, 605, 280, 175, "Committed Handling", ("Direct Response", "VIA state / memory", "Task + Agent dispatch / control"), COLORS["green_fill"], COLORS["green"])
    edge(d, "r-e1", ((265, 298), (335, 298)), "input", "discover")
    edge(d, "r-e2", ((555, 298), (630, 298)), "discover", "proposal", "Evidence Receipt", 592, 286)
    edge(d, "r-e3", ((860, 298), (955, 298)), "proposal", "readiness", "Resolution Record", 907, 286)
    edge(d, "r-e4", ((1175, 298), (1290, 298)), "readiness", "commit", "all required fields resolved", 1233, 286, COLORS["green"])
    edge(d, "r-e5", ((1065, 380), (1065, 570), (290, 570), (290, 640)), "readiness", "more", "NEED EVIDENCE", 680, 558, COLORS["cyan"])
    edge(d, "r-e6", ((1085, 380), (1085, 590), (630, 590), (630, 640)), "readiness", "clarify", "USER DECISION", 855, 578, COLORS["amber"])
    edge(d, "r-e7", ((1105, 380), (1105, 610), (970, 610), (970, 640)), "readiness", "failure", "CANNOT RESOLVE", 1035, 598, COLORS["red"])
    edge(d, "r-e8", ((410, 640), (410, 500), (745, 500), (745, 365)), "more", "proposal", "bounded refinement", 578, 488, COLORS["cyan"])
    edge(d, "r-e9", ((1400, 355), (1400, 605)), "commit", "handle", "publish / dispatch", 1450, 500, COLORS["green"])
    caption(d, 80, 820, "초기 정책: input revision당 semantic call 최대 2회. 미해결 field는 추측하지 않는다.", 13, COLORS["purple"], "start", 700)
    return d


def grounding_timeline() -> Diagram:
    d = Diagram("04-interaction-evidence-timeline", "Interaction Evidence Timeline", "발화 span을 당시 화면·pointer·selection revision에 연결하고 late evidence의 불확실성을 보존한다", 1600, 860)
    lane(d, "speech-lane", 55, 110, 1490, 165, "VOICE / TEXT INPUT", fill=COLORS["blue_fill"], stroke="#93C5FD")
    lane(d, "ui-lane", 55, 295, 1490, 245, "SCREEN / UI EVIDENCE", fill=COLORS["cyan_fill"], stroke="#67E8F9")
    lane(d, "resolve-lane", 55, 560, 1490, 235, "GROUNDING RESULT", fill=COLORS["purple_fill"], stroke="#C4B5FD")
    caption(d, 100, 325, "t0", 12, COLORS["muted"], "middle", 700)
    caption(d, 420, 325, "t1", 12, COLORS["muted"], "middle", 700)
    caption(d, 760, 325, "t2", 12, COLORS["muted"], "middle", 700)
    caption(d, 1110, 325, "t3", 12, COLORS["muted"], "middle", 700)
    caption(d, 1450, 325, "watermark", 12, COLORS["muted"], "middle", 700)
    box(d, "span1", 135, 155, 360, 75, "utterance span A", ("“이거” · acoustic interval ± uncertainty",), COLORS["white"], COLORS["blue"])
    box(d, "span2", 610, 155, 440, 75, "utterance span B", ("“지난번 발표자료” · final transcript revision",), COLORS["white"], COLORS["blue"])
    box(d, "correction", 1160, 155, 300, 75, "Transcript Revision", ("partial → final 변경",), COLORS["amber_fill"], COLORS["amber"])
    box(d, "screen1", 125, 355, 300, 90, "Screen Revision S17", ("display·window·document·viewport", "graph visible"), COLORS["white"], COLORS["cyan"])
    box(d, "pointer", 490, 355, 260, 90, "Pointer / Selection", ("event interval P42", "candidate graph region"), COLORS["white"], COLORS["cyan"])
    box(d, "screen2", 820, 355, 300, 90, "Screen Revision S18", ("scroll / focus changed", "same document identity"), COLORS["white"], COLORS["cyan"])
    box(d, "gap", 1210, 355, 250, 90, "Capture Gap", ("late event / buffer overflow", "공백을 현재 화면으로 채우지 않음"), COLORS["red_fill"], COLORS["red"])
    box(d, "bundle", 180, 625, 390, 110, "Grounding Evidence Bundle", ("span A → S17 + P42", "clock mapping · producer sequence"), COLORS["purple_fill"], COLORS["purple"])
    box(d, "candidates", 700, 625, 320, 110, "Candidate Referents", ("Graph-7 · Graph-8", "coverage · exclusion reason"), COLORS["purple_fill"], COLORS["purple"])
    box(d, "dependency", 1160, 625, 300, 110, "Resolution Dependency", ("receipt IDs · validity interval", "변경 시 해당 field만 무효화"), COLORS["green_fill"], COLORS["green"])
    edge(d, "g-e1", ((315, 230), (315, 355)), "span1", "screen1", "time aligned", 365, 295, COLORS["blue"])
    edge(d, "g-e2", ((380, 230), (620, 355)), "span1", "pointer", "gesture overlap", 500, 285, COLORS["blue"])
    edge(d, "g-e3", ((830, 230), (970, 355)), "span2", "screen2", "same document", 900, 285, COLORS["blue"])
    edge(d, "g-e4", ((1310, 230), (1335, 355)), "correction", "gap", "revision / late event", 1370, 285, COLORS["amber"])
    edge(d, "g-e5", ((275, 445), (275, 625)), "screen1", "bundle")
    edge(d, "g-e6", ((620, 445), (620, 570), (375, 570), (375, 625)), "pointer", "bundle")
    edge(d, "g-e7", ((570, 680), (700, 680)), "bundle", "candidates", "materialize", 635, 668)
    edge(d, "g-e8", ((1020, 680), (1160, 680)), "candidates", "dependency", "selected + unresolved", 1090, 668)
    edge(d, "g-e9", ((1335, 445), (1335, 625)), "gap", "dependency", "uncertainty", 1385, 540, COLORS["red"], dashed=True)
    return d


def dispatch_recovery() -> Diagram:
    d = Diagram("05-dispatch-and-recovery", "Dispatch, Event와 Response의 내구 경계", "세 durable 경로와 dispatch 선형화 지점을 분리해 crash와 duplicate를 복구한다", 1600, 940)
    lane(d, "outbound", 45, 100, 1510, 230, "1 — OUTBOUND COMMAND", fill=COLORS["orange_fill"], stroke="#FDBA74")
    lane(d, "inbound", 45, 350, 1510, 230, "2 — INBOUND AGENT EVENT", fill=COLORS["green_fill"], stroke="#86EFAC")
    lane(d, "publication", 45, 600, 1510, 270, "3 — USER RESPONSE PUBLICATION", fill=COLORS["blue_fill"], stroke="#93C5FD")
    box(d, "commit", 85, 165, 210, 100, "Semantic Commit", ("immutable revision vector",), COLORS["white"], COLORS["purple"])
    box(d, "tx", 380, 145, 300, 140, "Domain Transaction", ("Task link + command", "+ outbox in one commit"), COLORS["white"], COLORS["orange"])
    box(d, "linear", 780, 145, 260, 140, "DISPATCHING", ("epoch · precondition", "linearization point"), COLORS["amber_fill"], COLORS["amber"], badge="FENCE")
    box(d, "agent", 1150, 155, 280, 120, "Agent Ingress", ("idempotency key · target version", "ACCEPTED / REJECTED / UNKNOWN"), COLORS["orange_fill"], COLORS["orange"])
    box(d, "source-event", 90, 415, 230, 100, "Agent Event", ("source sequence / revision",), COLORS["white"], COLORS["green"])
    box(d, "inbox", 420, 395, 300, 140, "Durable Inbox", ("dedupe + source cursor", "+ Task projection transaction"), COLORS["white"], COLORS["green"])
    box(d, "projection", 835, 405, 260, 120, "Task Projection", ("source-confirmed state", "question lifecycle"), COLORS["green_fill"], COLORS["green"])
    box(d, "reconcile", 1200, 405, 240, 120, "Reconciliation", ("gap · reconnect · restart", "snapshot query"), COLORS["gray_fill"], COLORS["gray"])
    box(d, "payload", 85, 680, 260, 115, "Canonical Payload", ("facts · Task/result identity", "source · staleness"), COLORS["white"], COLORS["blue"])
    box(d, "pub-outbox", 440, 665, 290, 145, "Publication Outbox", ("planned Text / Voice generation", "durable publication ID"), COLORS["white"], COLORS["blue"])
    box(d, "channels", 835, 665, 260, 145, "Channel Delivery", ("Text ack", "audible prefix / interrupted"), COLORS["blue_fill"], COLORS["blue"])
    box(d, "conversation", 1200, 680, 250, 115, "Conversation Record", ("사용자가 실제 접한 내용", "후속 referent의 근거"), COLORS["gray_fill"], COLORS["gray"])
    edge(d, "d-e1", ((295, 215), (380, 215)), "commit", "tx")
    edge(d, "d-e2", ((680, 215), (780, 215)), "tx", "linear", "atomic intent", 730, 203)
    edge(d, "d-e3", ((1040, 215), (1150, 215)), "linear", "agent", "send", 1095, 203)
    edge(d, "d-e4", ((320, 465), (420, 465)), "source-event", "inbox")
    edge(d, "d-e5", ((720, 465), (835, 465)), "inbox", "projection", "commit", 777, 453)
    edge(d, "d-e6", ((1095, 465), (1200, 465)), "projection", "reconcile", "if gap", 1147, 453, COLORS["gray"], dashed=True)
    edge(d, "d-e7", ((1320, 405), (1320, 340), (965, 340), (965, 405)), "reconcile", "projection", "verified snapshot", 1140, 332, COLORS["gray"])
    edge(d, "d-e8", ((345, 738), (440, 738)), "payload", "pub-outbox")
    edge(d, "d-e9", ((730, 738), (835, 738)), "pub-outbox", "channels", "publish", 782, 726)
    edge(d, "d-e10", ((1095, 738), (1200, 738)), "channels", "conversation", "receipt", 1147, 726)
    caption(d, 335, 120, "CRASH WINDOW A", 11, COLORS["red"], "middle", 700)
    caption(d, 745, 120, "CRASH WINDOW B", 11, COLORS["red"], "middle", 700)
    caption(d, 1120, 120, "CRASH WINDOW C", 11, COLORS["red"], "middle", 700)
    caption(d, 365, 372, "event 수신 후 projection 전", 11, COLORS["red"], "middle", 700)
    caption(d, 785, 622, "게시 계획 후 실제 전달 전", 11, COLORS["red"], "middle", 700)
    caption(d, 80, 902, "재시작은 완료가 아니다. command·event·publication이 확인 가능한 상태로 다시 연결되어야 복구다.", 13, COLORS["red"], "start", 700)
    return d


def runtime_processes() -> Diagram:
    d = Diagram("06-runtime-and-fault-boundaries", "Runtime 배치와 Fault Boundary", "logical Component와 process isolation을 구분하고 Voice control·durable state를 보존한다", 1600, 900)
    lane(d, "pc", 45, 100, 1110, 730, "USER PC", fill="#F8FAFC", stroke="#94A3B8")
    lane(d, "external", 1190, 100, 365, 730, "REMOTE / EXTERNAL", fill="#FCFCFD", stroke="#CBD5E1", dashed=True)
    box(d, "ui", 95, 165, 250, 115, "UI Process", ("Chat · Task view · 알림", "명시적 사용자 control"), COLORS["blue_fill"], COLORS["blue"])
    box(d, "voice", 95, 335, 250, 135, "Voice Process", ("audio I/O · S2S session", "barge-in · playback buffer"), COLORS["blue_fill"], COLORS["blue"], badge="LOW LATENCY")
    box(d, "core", 445, 155, 470, 365, "Core Process", ("Request Controller · Context Manager", "Request Interpreter · Task Manager", "Response · Policy · Model Access"), COLORS["purple_fill"], COLORS["purple"], badge="LOGICAL MODULES")
    box(d, "worker-a", 965, 170, 145, 120, "Context Worker", ("blocking / native", "source adapter"), COLORS["cyan_fill"], COLORS["cyan"])
    box(d, "worker-b", 965, 350, 145, 120, "Agent Worker", ("protocol adapter", "restart / backoff"), COLORS["orange_fill"], COLORS["orange"])
    box(d, "store", 445, 600, 300, 125, "Durable State Store", ("Core crash와 독립된 persistence", "transactions · migration"), COLORS["gray_fill"], COLORS["gray"], badge="DURABLE")
    box(d, "supervisor", 810, 600, 300, 125, "Runtime Supervisor", ("worker restart · queue budget", "health · circuit breaker"), COLORS["amber_fill"], COLORS["amber"])
    box(d, "s2s", 1245, 160, 250, 100, "S2S Runtime", ("local 또는 remote", "모델 instance 1개"), COLORS["gray_fill"], COLORS["gray"])
    box(d, "llm", 1245, 310, 250, 100, "Semantic LLM Runtime", ("local 또는 remote", "모델 instance 1개"), COLORS["gray_fill"], COLORS["gray"])
    box(d, "sources", 1245, 470, 250, 100, "Context Sources", ("OS · App · file · service",), COLORS["cyan_fill"], COLORS["cyan"])
    box(d, "agent", 1245, 630, 250, 100, "Downstream Agents", ("reasoning · planning · tools",), COLORS["orange_fill"], COLORS["orange"])
    edge(d, "p-e1", ((345, 220), (445, 220)), "ui", "core", "IPC", 395, 208)
    edge(d, "p-e2", ((345, 400), (395, 400), (395, 330), (445, 330)), "voice", "core", "input / output", 397, 385, COLORS["blue"])
    edge(d, "p-e3", ((220, 335), (220, 300), (1245, 210)), "voice", "s2s", "streaming", 790, 275, COLORS["blue"])
    edge(d, "p-e4", ((915, 360), (1180, 360), (1180, 360), (1245, 360)), "core", "llm", "shared calls", 1080, 348, COLORS["purple"])
    edge(d, "p-e5", ((915, 240), (965, 230)), "core", "worker-a")
    edge(d, "p-e6", ((915, 420), (965, 410)), "core", "worker-b")
    edge(d, "p-e7", ((1110, 230), (1180, 230), (1180, 520), (1245, 520)), "worker-a", "sources", "read", 1180, 450, COLORS["cyan"])
    edge(d, "p-e8", ((1110, 410), (1160, 410), (1160, 680), (1245, 680)), "worker-b", "agent", "commands / events", 1188, 600, COLORS["orange"])
    edge(d, "p-e9", ((680, 520), (680, 600)), "core", "store", "transaction", 730, 560)
    edge(d, "p-e10", ((960, 520), (960, 600)), "core", "supervisor", "health", 1010, 560, COLORS["amber"])
    edge(d, "p-e11", ((960, 660), (910, 660), (910, 470), (1037, 470)), "supervisor", "worker-b", "restart", 915, 545, COLORS["amber"], dashed=True)
    caption(d, 930, 145, "FAULT ISOLATION", 11, COLORS["red"], "middle", 700)
    caption(d, 80, 790, "Voice stop과 UI control은 Core·connector 포화 시에도 별도 process와 자원 경계로 보존한다.", 13, COLORS["muted"], "start", 700)
    return d


def four_asr_paths() -> Diagram:
    d = Diagram("07-four-asr-critical-paths", "네 Core ASR의 Architecture 경로", "같은 구조적 선택이 네 품질 축에 서로 다른 인과 효과를 만든다", 1600, 900)
    lane(d, "qa19", 45, 110, 1510, 160, "1 — QA-19 SEMANTIC ACCURACY", fill=COLORS["purple_fill"], stroke="#C4B5FD")
    lane(d, "qa09", 45, 290, 1510, 160, "2 — QA-09 RESPONSIVENESS", fill=COLORS["blue_fill"], stroke="#93C5FD")
    lane(d, "qa29", 45, 470, 1510, 160, "3 — QA-29 MODIFIABILITY", fill=COLORS["cyan_fill"], stroke="#67E8F9")
    lane(d, "qa39", 45, 650, 1510, 160, "4 — QA-39 RELIABILITY / RECOVERABILITY", fill=COLORS["green_fill"], stroke="#86EFAC")
    rows = [
        ("a", 145, COLORS["purple_fill"], COLORS["purple"], ["Input + Timeline", "Evidence Coverage", "Semantic Commit", "Agent / Response Binding"]),
        ("b", 325, COLORS["blue_fill"], COLORS["blue"], ["Input End", "Parallel Read + Inference", "Commit / Dispatch", "Meaningful Output"]),
        ("c", 505, COLORS["cyan_fill"], COLORS["cyan"], ["Provider Change", "Adapter + Canonical Port", "Schema Migration", "Changed Elements"]),
        ("d", 685, COLORS["green_fill"], COLORS["green"], ["Fault", "Contain + Persist", "Reconcile Source State", "Verified Recovery"]),
    ]
    xs = [105, 470, 835, 1200]
    for prefix, y, fill, stroke, titles in rows:
        for index, title in enumerate(titles):
            box(d, f"{prefix}{index}", xs[index], y, 250, 80, title, (), fill, stroke)
            if index:
                edge(d, f"{prefix}-e{index}", ((xs[index - 1] + 250, y + 40), (xs[index], y + 40)), f"{prefix}{index-1}", f"{prefix}{index}")
    caption(d, 80, 850, "우선순위는 설계 선택의 순서다. 이후 Decision Package는 네 ASR을 모두 applicability 분류하고 applicable 축을 독립 측정한다.", 13, COLORS["muted"], "start", 700)
    return d


def diagrams() -> list[Diagram]:
    return [
        system_overview(),
        lifecycle(),
        request_resolution(),
        grounding_timeline(),
        dispatch_recovery(),
        runtime_processes(),
        four_asr_paths(),
    ]


def main() -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for diagram in diagrams():
        (OUTPUT / f"{diagram.slug}.svg").write_text(render_svg(diagram), encoding="utf-8")
        (OUTPUT / f"{diagram.slug}.drawio").write_text(render_drawio(diagram), encoding="utf-8")
        print(f"generated {diagram.slug}.drawio + .svg")


if __name__ == "__main__":
    main()
