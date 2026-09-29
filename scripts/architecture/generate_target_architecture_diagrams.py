#!/usr/bin/env python3
"""Generate editable draw.io sources and GitHub-preview SVGs for VIA target Architecture."""

from __future__ import annotations

import argparse
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
    bidirectional: bool = False


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
class LegendItem:
    id: str
    x: int
    y: int
    w: int
    label: str
    fill: str
    stroke: str


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
    legend_items: list[LegendItem] = field(default_factory=list)


def lane(diagram: Diagram, *args, **kwargs) -> None:
    diagram.lanes.append(Lane(*args, **kwargs))


def box(diagram: Diagram, *args, **kwargs) -> None:
    diagram.boxes.append(Box(*args, **kwargs))


def edge(diagram: Diagram, *args, **kwargs) -> None:
    diagram.edges.append(Edge(*args, **kwargs))


def caption(diagram: Diagram, *args, **kwargs) -> None:
    diagram.captions.append(Caption(*args, **kwargs))


def legend(diagram: Diagram, *args, **kwargs) -> None:
    diagram.legend_items.append(LegendItem(*args, **kwargs))


def svg_text_lines(item: Box) -> str:
    cx = item.x + item.w / 2
    title_y = item.y + (item.h / 2) - (len(item.lines) * 10) + 1
    parts = [
        f'<text x="{cx}" y="{title_y}" text-anchor="middle" '
        f'font-size="19" font-weight="700" fill="{COLORS["ink"]}">{escape(item.title)}</text>'
    ]
    for index, line_text in enumerate(item.lines):
        parts.append(
            f'<text x="{cx}" y="{title_y + 25 + index * 20}" text-anchor="middle" '
            f'font-size="15" font-weight="500" fill="{COLORS["muted"]}">{escape(line_text)}</text>'
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
    marker_colors = {
        "arrow": COLORS["line"],
        "arrow-blue": COLORS["blue"],
        "arrow-cyan": COLORS["cyan"],
        "arrow-purple": COLORS["purple"],
        "arrow-green": COLORS["green"],
        "arrow-orange": COLORS["orange"],
        "arrow-amber": COLORS["amber"],
        "arrow-gray": COLORS["gray"],
        "arrow-red": COLORS["red"],
    }
    marker_for_color = {color: marker_id for marker_id, color in marker_colors.items()}
    out = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{diagram.width}" height="{diagram.height}" '
        f'viewBox="0 0 {diagram.width} {diagram.height}" role="img" aria-labelledby="title desc">',
        f'<title id="title">{escape(diagram.title)}</title>',
        f'<desc id="desc">{escape(diagram.subtitle)}</desc>',
        '<defs>',
        *[
            f'<marker id="{marker_id}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">'
            f'<path d="M 0 0 L 10 5 L 0 10 z" fill="{color}"/></marker>'
            for marker_id, color in marker_colors.items()
        ],
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
            f'font-size="17" font-weight="750" fill="{COLORS["muted"]}">{escape(item.title)}</text>'
        )

    for item in diagram.edges:
        dash = ' stroke-dasharray="8 6"' if item.dashed else ""
        marker = f'url(#{marker_for_color.get(item.color, "arrow")})'
        marker_start = f' marker-start="{marker}"' if item.bidirectional else ""
        points = " ".join(f"{x},{y}" for x, y in item.points)
        out.append(
            f'<polyline points="{points}" fill="none" stroke="{item.color}" stroke-width="{item.width}" '
            f'stroke-linejoin="round" stroke-linecap="round" marker-end="{marker}"{marker_start}{dash}/>'
        )
        if item.label and item.label_x is not None and item.label_y is not None:
            label_w = text_width(item.label, 14) + 16
            out.append(
                f'<rect x="{item.label_x - label_w / 2}" y="{item.label_y - 15}" width="{label_w}" height="22" '
                f'rx="8" fill="#FFFFFF" stroke="#E2E8F0"/>'
            )
            out.append(
                f'<text x="{item.label_x}" y="{item.label_y}" text-anchor="middle" '
                f'font-family="Inter, Arial, sans-serif" font-size="14" font-weight="650" fill="{COLORS["muted"]}">{escape(item.label)}</text>'
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

    for item in diagram.legend_items:
        out.append(
            f'<rect x="{item.x}" y="{item.y}" width="{item.w}" height="25" rx="8" '
            f'fill="{item.fill}" stroke="{item.stroke}" stroke-width="1.5"/>'
        )
        out.append(
            f'<text x="{item.x + item.w / 2}" y="{item.y + 17}" text-anchor="middle" '
            f'font-family="Inter, Arial, sans-serif" font-size="14" font-weight="700" '
            f'fill="{item.stroke}">{escape(item.label)}</text>'
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


def text_width(value: str, size: float) -> int:
    """Conservative label width for mixed Korean/Latin text (not a font metric)."""
    return round(sum(size if ord(c) > 255 else size * 0.62 for c in value)) + 8


def validate_routes(diagram: Diagram) -> None:
    """Reject box penetration and edge crossings in all orthogonal figures.

    This checks line geometry, not glyph bounds, arrowheads or label readability;
    rendered visual review is still required. Shared junctions are deliberately
    avoided: use separate endpoints so the destination remains unambiguous.
    """
    boxes = {item.id: item for item in diagram.boxes}
    segments = []
    for item in diagram.edges:
        for ident, (x, y) in [(item.source, item.points[0]), (item.target, item.points[-1])]:
            node = boxes[ident]
            on_border = (
                (x in (node.x, node.x + node.w) and node.y <= y <= node.y + node.h)
                or (y in (node.y, node.y + node.h) and node.x <= x <= node.x + node.w)
            )
            if not on_border:
                raise ValueError(f"{item.id}: endpoint is not on {ident}'s border")
        for (x1, y1), (x2, y2) in zip(item.points, item.points[1:]):
            if (x1 == x2) == (y1 == y2):
                raise ValueError(f"{item.id}: expected a nonzero orthogonal segment")
            left, right = sorted((x1, x2))
            top, bottom = sorted((y1, y2))
            for node in diagram.boxes:
                penetrates = (
                    node.x < x1 < node.x + node.w and max(top, node.y) < min(bottom, node.y + node.h)
                    if x1 == x2 else
                    node.y < y1 < node.y + node.h and max(left, node.x) < min(right, node.x + node.w)
                )
                if penetrates:
                    raise ValueError(f"{item.id}: line enters {node.id}")
            segments.append((item.id, left, top, right, bottom))
    for index, (ident, left, top, right, bottom) in enumerate(segments):
        for other, ol, ot, oright, ob in segments[index + 1:]:
            if ident == other:
                continue
            if max(left, ol) <= min(right, oright) and max(top, ot) <= min(bottom, ob):
                raise ValueError(f"{ident} crosses or touches {other}")


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
        value = box_value(item)
        value = value.replace("font-size:16px", "font-size:19px").replace("font-size:12px", "font-size:15px")
        cell = ET.SubElement(root, "mxCell", {"id": item.id, "value": value, "style": style, "vertex": "1", "parent": "1"})
        ET.SubElement(cell, "mxGeometry", {"x": str(item.x), "y": str(item.y), "width": str(item.w), "height": str(item.h), "as": "geometry"})

    boxes_by_id = {item.id: item for item in diagram.boxes}
    for item in diagram.edges:
        source_box = boxes_by_id[item.source]
        target_box = boxes_by_id[item.target]
        exit_x = min(1.0, max(0.0, (item.points[0][0] - source_box.x) / source_box.w))
        exit_y = min(1.0, max(0.0, (item.points[0][1] - source_box.y) / source_box.h))
        entry_x = min(1.0, max(0.0, (item.points[-1][0] - target_box.x) / target_box.w))
        entry_y = min(1.0, max(0.0, (item.points[-1][1] - target_box.y) / target_box.h))
        style = (
            "edgeStyle=orthogonalEdgeStyle;rounded=1;orthogonalLoop=1;jettySize=auto;html=1;"
            f"strokeColor={item.color};strokeWidth={item.width};dashed={1 if item.dashed else 0};"
            f"startArrow={'block' if item.bidirectional else 'none'};startFill=1;endArrow=block;endFill=1;"
            f"exitX={exit_x:.3f};exitY={exit_y:.3f};entryX={entry_x:.3f};entryY={entry_y:.3f};"
            "exitPerimeter=1;entryPerimeter=1;fontSize=11;labelBackgroundColor=#FFFFFF;"
        )
        # Preserve designed waypoints rather than letting the editor reroute them.
        style = style.replace("edgeStyle=orthogonalEdgeStyle", "edgeStyle=none").replace("exitPerimeter=1;entryPerimeter=1", "exitPerimeter=0;entryPerimeter=0")
        cell = ET.SubElement(
            root,
            "mxCell",
            {"id": item.id, "value": "", "style": style, "edge": "1", "parent": "1", "source": item.source, "target": item.target},
        )
        geometry = ET.SubElement(cell, "mxGeometry", {"relative": "1", "as": "geometry"})
        if len(item.points) > 2:
            array = ET.SubElement(geometry, "Array", {"as": "points"})
            for x, y in item.points[1:-1]:
                ET.SubElement(array, "mxPoint", {"x": str(x), "y": str(y)})
        if item.label and item.label_x is not None and item.label_y is not None:
            width = text_width(item.label, 14) + 16
            label_cell = ET.SubElement(root, "mxCell", {"id": item.id + "-label", "value": escape(item.label), "style": "rounded=1;html=1;whiteSpace=wrap;align=center;verticalAlign=middle;fontSize=14;fontStyle=1;fillColor=#FFFFFF;strokeColor=#E2E8F0;", "vertex": "1", "parent": "1"})
            ET.SubElement(label_cell, "mxGeometry", {"x": str(item.label_x-width/2), "y": str(item.label_y-15), "width": str(width), "height": "22", "as": "geometry"})

    for index, item in enumerate(diagram.captions):
        drawio_align = {"start": "left", "middle": "center", "end": "right"}.get(item.anchor, "left")
        style = f"text;html=1;strokeColor=none;fillColor=none;align={drawio_align};fontSize={item.size};fontColor={item.color};fontStyle={1 if item.weight >= 700 else 0};"
        width = text_width(item.text, item.size)
        x = item.x - (width / 2 if item.anchor == "middle" else width if item.anchor == "end" else 0)
        cell = ET.SubElement(root, "mxCell", {"id": f"caption-{index}", "value": escape(item.text), "style": style, "vertex": "1", "parent": "1"})
        ET.SubElement(cell, "mxGeometry", {"x": str(x), "y": str(item.y - item.size), "width": str(width), "height": str(item.size + 12), "as": "geometry"})

    for item in diagram.legend_items:
        style = (
            "rounded=1;whiteSpace=wrap;html=1;shadow=0;"
            f"fillColor={item.fill};strokeColor={item.stroke};fontColor={item.stroke};"
            "strokeWidth=1.5;fontSize=10;fontStyle=1;align=center;verticalAlign=middle;"
        )
        style = style.replace("fontSize=10", "fontSize=14")
        cell = ET.SubElement(root, "mxCell", {"id": item.id, "value": item.label, "style": style, "vertex": "1", "parent": "1"})
        ET.SubElement(cell, "mxGeometry", {"x": str(item.x), "y": str(item.y), "width": str(item.w), "height": "25", "as": "geometry"})

    for ident, value, y, size in [("page-title", diagram.title, 15, 27), ("page-subtitle", diagram.subtitle, 49, 14)]:
        cell = ET.SubElement(root, "mxCell", {"id": ident, "value": escape(value), "style": f"text;html=1;align=left;verticalAlign=middle;strokeColor=none;fillColor=none;fontSize={size};", "vertex": "1", "parent": "1"})
        ET.SubElement(cell, "mxGeometry", {"x": "48", "y": str(y), "width": str(diagram.width-96), "height": "30", "as": "geometry"})
    ET.indent(mxfile, space="  ")
    return '<?xml version="1.0" encoding="UTF-8"?>\n' + ET.tostring(mxfile, encoding="unicode") + "\n"


def system_overview() -> Diagram:
    d = Diagram("01-system-overview", "VIA — 논리 Component와 요청·결과 흐름", "PROPOSED · Accuracy → Responsiveness → Modifiability → Recoverability · 논리 구조이며 process 배치도가 아니다", 2400, 1490)
    lane(d, "via-system", 300, 155, 1700, 1275, "VIA SOFTWARE — 사용자 PC에서 실행하는 설계 책임 범위", fill="#F8FAFC", stroke="#64748B")
    lane(d, "coordination", 325, 205, 1650, 835, "INTERACTION & ORCHESTRATION — 모든 실선 Component 박스는 동일한 논리 수준", fill="#FFFFFF", stroke="#D5DDE9")
    lane(d, "shared-services", 325, 1070, 1650, 335, "VIA SHARED SERVICES — 저장과 모델 연동도 VIA 내부 책임", fill="#F1F5F9", stroke="#CBD5E1")
    caption(d, 2080, 190, "EXTERNAL RESPONSIBILITY", 17, COLORS["gray"], weight=700)
    caption(d, 2080, 218, "같은 PC / 원격 배치 모두 가능", 15)

    box(d, "user", 30, 545, 210, 145, "사용자", ("Voice · Text", "화면 지칭 · 선택"), COLORS["blue_fill"], COLORS["blue"])
    box(d, "interaction", 350, 545, 280, 145, "Interaction Manager", ("입력 · evidence timeline", "재생 · barge-in · S2S client", "M1 · 세부 모듈은 본문 §4"), COLORS["blue_fill"], COLORS["blue"])
    box(d, "controller", 830, 545, 300, 145, "Request Controller", ("Conversation · Request 소유", "의미·대상 검증 · 경로 확정", "dispatch / publication admission"), COLORS["purple_fill"], COLORS["purple"])
    box(d, "task", 1310, 545, 270, 145, "Task Manager", ("Task · Execution 연결 소유", "command 생성 · 상태 반영", "Gateway의 요청 주체"), COLORS["green_fill"], COLORS["green"])
    box(d, "gateway", 1700, 545, 260, 145, "Agent Gateway", ("protocol · capability adapter", "outbox 전송 · inbox 수신", "Agent 결과의 VIA 수신점"), COLORS["orange_fill"], COLORS["orange"])
    box(d, "agents", 2080, 545, 270, 145, "Downstream Agents", ("업무 추론 · 계획 · 도구", "실제 작업 실행", "progress · question · result"), COLORS["orange_fill"], COLORS["orange"])
    box(d, "context", 350, 285, 280, 135, "Context Manager", ("허용된 기본 Context 준비", "bounded read · 후보·근거", "cache · receipt · User Memory"), COLORS["cyan_fill"], COLORS["cyan"])
    box(d, "resolver", 830, 285, 300, 135, "Request Interpreter", ("목표 · 대상 · Task 후보 제안", "추가 근거 / clarification 제안", "M2 · 의미 확정 권한 없음"), COLORS["purple_fill"], COLORS["purple"])
    box(d, "policy", 1310, 285, 270, 135, "Policy Manager", ("접근 · 외부 제공 · consent", "현재 policy revision", "Controller가 판단 적용"), COLORS["amber_fill"], COLORS["amber"])
    box(d, "sources", 2080, 285, 270, 135, "Context Sources", ("OS · App · File · Mail", "Calendar · Browser · Web", "허용된 범위의 읽기 전용"), COLORS["gray_fill"], COLORS["gray"])
    box(d, "response", 830, 825, 300, 145, "Response Manager", ("승인된 payload의 게시·기록", "Text · Voice · 알림 조정", "M3 · 실제 전달 receipt 보존"), COLORS["blue_fill"], COLORS["blue"])

    # Distinct outbound and inbound Agent lines: no shared segment or ambiguous junction.
    edge(d, "u-input", ((240, 585), (350, 585)), "user", "interaction", "입력", 290, 571, COLORS["blue"], width=3)
    edge(d, "u-output", ((350, 650), (240, 650)), "interaction", "user", "표시·재생", 288, 635, COLORS["blue"], width=3)
    edge(d, "request-input", ((630, 590), (830, 590)), "interaction", "controller", "① 입력 이벤트·근거", 730, 577, COLORS["blue"], width=3)
    edge(d, "context-read", ((830, 560), (730, 560), (730, 360), (630, 360)), "controller", "context", "② 조회 ↔ 근거", 715, 465, COLORS["cyan"], bidirectional=True)
    edge(d, "interpret", ((980, 545), (980, 420)), "controller", "resolver", "③ 해석 요청 ↔ 제안", 987, 490, COLORS["purple"], bidirectional=True)
    edge(d, "authorize", ((1130, 560), (1215, 560), (1215, 360), (1310, 360)), "controller", "policy", "④ 허용 범위 확인", 1250, 465, COLORS["amber"], bidirectional=True)
    edge(d, "commit-task", ((1130, 595), (1310, 595)), "controller", "task", "⑤ 검증된 업무 의도", 1220, 581, COLORS["green"], width=3)
    edge(d, "task-update", ((1310, 660), (1130, 660)), "task", "controller", "⑧ 확인된 상태·질문", 1220, 714, COLORS["green"], width=3)
    edge(d, "agent-command", ((1580, 595), (1700, 595)), "task", "gateway", "⑥ command", 1640, 581, COLORS["orange"], width=3)
    edge(d, "agent-event", ((1700, 660), (1580, 660)), "gateway", "task", "⑦ event", 1640, 714, COLORS["green"], width=3)
    edge(d, "dispatch", ((1960, 595), (2080, 595)), "gateway", "agents", "request", 2020, 577, COLORS["orange"], width=3)
    edge(d, "receive", ((2080, 660), (1960, 660)), "agents", "gateway", "결과·상태", 2020, 714, COLORS["green"], width=3)
    edge(d, "publish", ((930, 690), (930, 825)), "controller", "response", "⑨ 응답 게시 승인", 858, 771, COLORS["blue"], width=3)
    edge(d, "publication-ack", ((1040, 825), (1040, 690)), "response", "controller", "게시 사실", 1102, 792, COLORS["blue"])
    edge(d, "output-release", ((830, 870), (465, 870), (465, 690)), "response", "interaction", "⑩ 표시·재생 / release·cancel", 665, 854, COLORS["blue"], width=3)
    edge(d, "output-receipt", ((390, 690), (390, 995), (1050, 995), (1050, 970)), "interaction", "response", "generation handle · delivery receipt", 670, 982, COLORS["blue"])
    edge(d, "source-read", ((490, 285), (490, 250), (2215, 250), (2215, 285)), "context", "sources", "bounded source access ↔ evidence", 1780, 243, COLORS["cyan"], bidirectional=True)

    # Shared dependencies are expressed by named ports, not duplicate Component boxes.
    # Dashed ports are a dependency index; no event bus / extra runtime is implied.
    box(d, "store-clients", 350, 1130, 520, 50, "S · state-owner repository ports", (), COLORS["white"], COLORS["gray"], kind="dashed")
    box(d, "store", 350, 1220, 520, 155, "State Store", ("Conversation · Request · Task · Memory · Policy", "command outbox · event inbox · publication record", "owner별 transaction · 복구 가능한 영속 저장"), COLORS["gray_fill"], COLORS["gray"])
    edge(d, "durable-access", ((610, 1180), (610, 1220)), "store-clients", "store", color=COLORS["gray"], dashed=True, bidirectional=True)
    for i, title, color, y in [(1, "M1 · Interaction Manager", "blue", 1150), (2, "M2 · Request Interpreter", "purple", 1220), (3, "M3 · Response Manager", "blue", 1290)]:
        box(d, f"model-client-{i}", 965, y, 275, 46, title, (), COLORS["white"], COLORS[color], kind="dashed")
        label = {1: "S2S stream", 2: "semantic", 3: "응답 생성"}[i]
        edge(d, f"model-call-{i}", ((1240, y+23), (1380, y+23)), f"model-client-{i}", "model", label, 1310, y+10, COLORS[color], dashed=True, bidirectional=True)
    box(d, "model", 1380, 1135, 380, 220, "Model Access", ("VIA-owned provider adapter", "session · queue · timeout · cancel", "model runtime과 VIA 계약 연결"), COLORS["gray_fill"], COLORS["gray"])
    box(d, "s2s", 2080, 1135, 270, 85, "S2S Model × 1", ("streaming audio · transcript",), COLORS["gray_fill"], COLORS["gray"])
    box(d, "semantic", 2080, 1270, 270, 85, "Semantic LLM × 1", ("해석 · 필요한 응답 생성",), COLORS["gray_fill"], COLORS["gray"])
    edge(d, "s2s-provider", ((1760, 1175), (2080, 1175)), "model", "s2s", "audio / transcript / generation", 1920, 1162, COLORS["blue"], bidirectional=True)
    edge(d, "semantic-provider", ((1760, 1310), (2080, 1310)), "model", "semantic", "prompt / structured output", 1920, 1297, COLORS["purple"], bidirectional=True)

    caption(d, 1350, 815, "STREAMING: chunk와 Request를 구분", 19, COLORS["ink"], weight=700)
    for y, t in [(851,"audio chunk는 Interaction Manager ↔ Model Access"), (881,"Controller에는 시작·정정·확정 이벤트 + evidence 참조"), (911,"Context는 기본 준비 + 변경 시 갱신 + 부족한 근거 조회"), (941,"barge-in은 즉시 로컬 재생 중단; Agent 취소는 의미 확인 후")]:
        caption(d, 1350, y, t, 16)
    caption(d, 370, 1025, "번호는 연결 계약 ID: 필수 직렬 단계가 아님 · ②~④는 필요에 따라 반복 · ⑦~⑩는 새 발화 없이도 진행", 16, COLORS["muted"])
    caption(d, 350, 1118, "Controller · Context · Task · Gateway · Response · Policy", 15, COLORS["gray"])
    caption(d, 980, 1383, "점선 S / M1~M3 = 위 Component의 접근 port 표기 · 추가 Component나 메시지 bus가 아님", 15)
    caption(d, 350, 1457, "State Store는 VIA 내부 기반이다. Core process 종료 후 데이터 보존은 필요하지만 별도 DB process를 강제하지 않는다.", 15)

    # Explicit legend: box role vs edge contract; no implicit significance of line weight.
    for i, (label, color) in enumerate([("Interaction / 응답", "blue"), ("의미 제안·확정", "purple"), ("Context / 근거", "cyan"), ("Policy / 동의", "amber"), ("Task / 상태·결과", "green"), ("Agent / 명령", "orange"), ("저장·모델 연동", "gray")]):
        legend(d, f"legend-{color}", 48+i*210, 92, 195, label, COLORS[color+"_fill"], COLORS[color])
    caption(d, 1560, 110, "박스 색 = 책임 · 선 색 = 전달 계약", 15, weight=700)
    caption(d, 1560, 136, "→ 전달 방향   ↔ 조회/반환   점선 = 공통 서비스 접근", 15)
    return d


def detail(slug: str, title: str, subtitle: str, height: int = 1120) -> Diagram:
    d = Diagram(slug, title, 'PROPOSED · ' + subtitle, 1900, height)
    for i, (label, color) in enumerate([('입력·응답', 'blue'), ('의미·요청', 'purple'), ('Context·근거', 'cyan'), ('정책·판정', 'amber'), ('업무·상태', 'green'), ('명령·Agent', 'orange'), ('저장·의존성', 'gray'), ('중단·실패·불확실', 'red')]):
        legend(d, f'key-{color}', 48+i*185, 92, 172, label, COLORS[color+'_fill'], COLORS[color])
    caption(d, 1580, 110, '→ 관계·전달  ·  점선 = 보조 관계', 14)
    return d


def card(d, ident, x, y, title, lines=(), color='purple', w=340, h=132):
    box(d, ident, x, y, w, h, title, tuple(lines), COLORS[color+'_fill'], COLORS[color])


def right(d, ident, source, target, label='', color='gray'):
    a = next(b for b in d.boxes if b.id == source)
    b = next(b for b in d.boxes if b.id == target)
    y = a.y+a.h/2
    assert y == b.y+b.h/2
    edge(d, ident, ((a.x+a.w, y), (b.x, y)), source, target, label, (a.x+a.w+b.x)/2, y-17, COLORS[color])


def down(d, ident, source, target, label='', color='gray'):
    a = next(b for b in d.boxes if b.id == source)
    b = next(b for b in d.boxes if b.id == target)
    x = a.x+a.w/2
    assert x == b.x+b.w/2
    edge(d, ident, ((x, a.y+a.h), (x, b.y)), source, target, label, x+65, (a.y+a.h+b.y)/2, COLORS[color])


def strip(d, ident, y, title, color, steps, note):
    lane(d, ident, 45, y, 1810, 260, title, fill='#F8FAFC', stroke='#D5DDE9')
    for i, (name, lines) in enumerate(steps):
        card(d, f'{ident}-{i}', 90+i*450, y+65, name, lines, color)
        if i:
            right(d, f'{ident}-link-{i}', f'{ident}-{i-1}', f'{ident}-{i}', color=color)
    caption(d, 90, y+235, note, 16)


def lifecycle() -> Diagram:
    d = detail('02-lifecycle-and-ownership', 'Identity와 수명 — 무엇이 이어지고 무엇이 새로 생기는가', '박스는 데이터 entity · 화살표는 관계이며 실행 순서가 아니다', 1110)
    lane(d, 'identity', 45, 150, 1810, 520, '관계 — Request Controller / Task Manager / 외부 Agent의 권위를 구분', fill='#F8FAFC')
    top = [
        ('conv', 'Conversation', ('Controller 소유 · 대화와 실제 응답', 'Voice 연결이 끝나도 유지'), 'blue'),
        ('turn', 'Turn', ('Controller 소유 · 제출한 입력', '원문·시점·input revision'), 'blue'),
        ('request', 'Request', ('Controller 소유 · 논리적 요청', 'Request Graph의 node·의존 관계'), 'purple'),
        ('task', 'Task', ('Task Manager 소유 · 사용자 업무', '직접 응답은 Task가 없어도 됨'), 'green')]
    for i, (ident, title, lines, color) in enumerate(top):
        card(d, ident, 90+i*450, 225, title, lines, color)
    right(d, 'contains-turn', 'conv', 'turn', '1 : N', 'blue')
    right(d, 'resolves-request', 'turn', 'request', '생성·보완', 'purple')
    right(d, 'binds-task', 'request', 'task', '선택적 연결', 'green')
    card(d, 'connection', 90, 490, 'Voice Connection', ('Interaction Manager 소유 · 연결 session', '같은 대화에 여러 번 재연결 가능'), 'gray')
    card(d, 'question', 540, 490, 'Pending Interaction', ('Controller 소유 · 질문 identity', '새 Turn의 답을 기존 Request에 결합'), 'amber')
    card(d, 'response-record', 990, 490, 'Response Record', ('Response Manager 소유 · 게시 identity', 'Text·실제 audible 범위를 분리 기록'), 'blue')
    card(d, 'execution', 1440, 490, 'Agent Execution', ('외부 실행 상태의 VIA projection', 'Task 하나에 후속 실행 0..N'), 'orange')
    down(d, 'connection-relation', 'conv', 'connection', '수명 분리', 'gray')
    down(d, 'answer-relation', 'turn', 'question', '질문 ID 결합', 'amber')
    down(d, 'publication-relation', 'request', 'response-record', '응답 0..N', 'blue')
    down(d, 'execution-relation', 'task', 'execution', '실행 0..N', 'orange')
    strip(d, 'example', 700, '예시 — 설명 → 파일 작성 → 결과 수정 → 새 대화', 'purple', [
        ('T1 · 개념을 설명해줘', ('R1 직접 응답 · Task 없음', '전달된 답변은 후속 참조 가능')),
        ('T2 · 그걸 파일로 만들어', ('새 R2 → 새 Task A', 'Agent 실행 E1과 결과 v1 연결')),
        ('T3 · 결론을 추가해줘', ('새 R3 → 기존 Task A', '완료 실행이면 후속 E2 연결')),
        ('새 Conversation', ('기존 Task A는 계속 유지', '원래 대화 + 명시적 참조로 접근')),
    ], 'Clarification 답변은 새 Turn이어도 원래 Request를 보완한다. 새 목표·새 결과물은 기존 결과를 참조하는 새 Task다.')
    caption(d, 70, 1010, 'Request의 처리 종료, Task의 업무 완료, Execution 종료, Voice 연결 종료는 서로 다른 사건이다.', 18, COLORS['ink'], weight=700)
    caption(d, 70, 1045, 'Command / Event / Artifact는 Task·Execution·Request identity와 version으로 결합한다. Agent의 thread ID가 VIA identity를 대신하지 않는다.', 16)
    return d


def request_resolution() -> Diagram:
    d = detail('03-request-resolution', '요청 확정 — 최소 근거로 시작하고 필요한 부분만 보완', 'Core semantic 경로 · S2S 출력 경로는 그림 08 · 모든 입력은 같은 revision 규칙 적용', 1060)
    lane(d, 'primary', 45, 150, 1810, 255, '① 최초 해석 — 입력 중 기본 Context 준비는 finalization과 병렬 진행', fill='#F8FAFC')
    for ident, x, title, lines, col in [
        ('input',90,'Final Input + Evidence',('확정 입력·시점별 화면 근거','기본 Context·허용 source·Task 후보'),'blue'),
        ('proposal',540,'Interpreter · 호출 1',('목표·대상·Task·handling 후보','field별 근거·충돌·추가 읽기 제안'),'purple'),
        ('host',990,'Controller · 검증',('coverage·freshness·현재 policy','입력·Task·evidence revision 확인'),'amber'),
        ('commit',1440,'Semantic Commit',('필수 field 해결·불변 revision','직접 처리 / 내부 변경 / Agent 위임'),'green')]:
        card(d,ident,x,225,title,lines,col)
    right(d,'input-proposal','input','proposal',color='purple')
    right(d,'proposal-host','proposal','host',color='purple')
    right(d,'host-commit','host','commit','충족','green')
    lane(d,'outcomes',45,445,1810,335,'② 미해결 분기 — 업무 실행 허가 아님',fill='#FFFFFF')
    card(d,'more',540,555,'Context 추가 조회',('허용된 read 묶음 → Interpreter 호출 2','조회 실패·budget 소진도 receipt에 기록'),'cyan')
    card(d,'clarify',990,555,'Clarification',('사용자만 구분할 후보·누락 조건','질문 등록·게시 후 새 Turn으로 보완'),'amber')
    card(d,'failure',1440,555,'확인 불가 / 보류',('source 실패·기능 미지원·deadline','이유와 다음 행동을 같은 요청에 기록'),'red')
    edge(d,'need-read',((1030,357),(1030,420),(740,420),(740,555)),'host','more','추가 근거',890,410,COLORS['cyan'])
    edge(d,'retry',((620,555),(620,357)),'more','proposal','bounded refinement',620,490,COLORS['cyan'])
    edge(d,'need-user',((1160,357),(1160,555)),'host','clarify','사용자 판단',1160,500,COLORS['amber'])
    edge(d,'cannot',((1280,357),(1280,425),(1610,425),(1610,555)),'host','failure','해결 불가',1520,470,COLORS['red'])
    caption(d,90,740,'초기 resource policy: 같은 확정 input revision당 semantic 해석 최대 2회. 추가 모델이나 무한 tool loop를 만들지 않는다.',16)
    lane(d,'rules',45,810,1810,175,'③ 변경·예외 — READY 형식만으로 실행하지 않는다',fill='#F1F5F9')
    caption(d,90,875,'정정 / 관련 source 변경 → 이전 proposal·admission 무효화 → 영향 field 재검증. 재시도는 전체 deadline 안에서만 허용.',18,COLORS['ink'],weight=700)
    caption(d,90,920,'Clarification·실패 안내는 미확정 정보를 보존한 publication admission으로 게시한다. 업무 dispatch에는 Semantic Commit이 필요하다.',16)
    return d


def grounding_timeline() -> Diagram:
    d = detail('04-interaction-evidence-timeline', '지칭의 시간축 — 마지막 화면으로 과거 발화를 해석하지 않는다', '“이 그래프… 아니, 저 표를 넣어줘” · 가로는 사건 순서, 세로는 근거 결합', 1100)
    rows=[(155,'① 발화·화면 사건'),(390,'② 시점별 관측 근거'),(655,'③ 지칭 결과·유효성')]
    for i,(y,title) in enumerate(rows):
        lane(d,f'time-row-{i}',45,y,1810,215,title,fill='#F8FAFC')
    columns=[
        ('a','t0 · “이 그래프”',('표현 A · acoustic interval','시각 오차·clock domain 포함'),'S17 + Pointer P42',('Document D · Graph G7','지칭 당시의 객체·영역'),'잠정 대상 G7',('근거는 보존','정정 뒤 실행 대상으로 사용하지 않음'),'blue'),
        ('b','t1 · “아니, 저 표”',('표현 B · correction relation','전사 도착이 늦어도 시점은 t1'),'S18 + Selection Q9',('Document D · Table T3','정정 시점의 선택 범위'),'최종 대상 T3',('A → B supersession','최종 입력·evidence revision으로 확정'),'purple'),
        ('c','t2 · 발화 종료 후 scroll',('현재 화면만 달라짐','최종 전사는 이후 도착 가능'),'S19 · viewport 변경',('같은 T3 identity·내용 version','원래 지칭 근거를 덮어쓰지 않음'),'당시 대상 유지',('실행 전 내용·권한 재검증','내용 변경이면 관련 field 재해석'),'green'),
        ('d','늦은 근거 / 수집 공백',('수신 지연·buffer 초과 표시','사건 시각을 수신 시각으로 대체 금지'),'Sequence + Gap',('producer sequence·watermark','unknown을 현재 화면으로 채우지 않음'),'미확정 범위 보존',('후보·시각 오차가 해소되지 않으면','추가 조회 / clarification / 보류'),'red')]
    for i,(ident,t,l,s,sl,r,rl,col) in enumerate(columns):
        x=90+i*450
        card(d,ident+'-speech',x,215,t,l,'blue')
        card(d,ident+'-evidence',x,450,s,sl,'cyan')
        card(d,ident+'-resolve',x,715,r,rl,col)
        down(d,ident+'-align',ident+'-speech',ident+'-evidence',color='cyan')
        down(d,ident+'-ground',ident+'-evidence',ident+'-resolve',color=col)
    caption(d,90,930,'발화 시작 전부터 유지된 선택도 유효 구간으로 참조한다. 관련 구간은 pin하고, 보관 한도 초과·capture gap은 명시한다.',18,COLORS['ink'],weight=700)
    caption(d,90,972,'Timeline은 Interaction Manager, source 후보·조회 receipt는 Context Manager, 최종 referent와 dependency read set은 Controller가 소유한다.',16)
    caption(d,90,1012,'늦은 근거로 commit을 덮어쓰지 않는다. 전송 전이면 재검증하고, 전송 뒤면 실행 상태를 확인한 정정으로 처리한다.',16)
    return d


def dispatch_recovery() -> Diagram:
    d = detail('05-dispatch-and-recovery', '내구 경계 — 명령·결과·알림 사이의 유실을 닫는다', '각 행은 다른 transaction / 전달 경로 · 네트워크와 물리적 재생은 DB transaction 밖', 1130)
    strip(d,'send',150,'A · COMMAND — Controller의 admission → Task Manager의 intent → Gateway 전송','orange',[
        ('의도·Outbox 원자 기록',('Request revision·Task·command ID','허용 범위·Agent·중복 방지 key')),
        ('전송 시작 CAS',('PENDING → DISPATCHING','hold·epoch·policy를 원자 확인')),
        ('Agent ingress',('동일 command key로 correlation','접수 / 거절 / 결과 불명 구분')),
        ('불명 상태 조정',('조회 + idempotency 지원 시 재전송','미지원이면 임의 재실행 금지')),
    ],'Crash 전후: outbox 미기록이면 보내지 않음 · DISPATCHING 이후에는 전송 여부 불명 가능 · 취소 접수는 외부 rollback이 아님.')
    strip(d,'event',435,'B · EVENT — Agent source → Gateway inbox → Task state → Conversation에 전달','green',[
        ('Inbox 내구 수신',('Agent·Execution·event identity','중복 key·source revision 보존')),
        ('Projection + 후속 event',('적용 cursor·Task state·domain outbox','관련 질문 종료도 owner별 원자 전이')),
        ('Controller 적용',('원래 Conversation·Request로 결합','event ID dedupe + publication intent')),
        ('Response 인계',('publication ID로 멱등 인계','새 발화 없이 결과·질문 전달')),
    ],'Task 저장 직후 crash해도 domain outbox를 재처리한다. Source gap·역순·재접속이면 snapshot으로 조정하고 확인 불가를 남긴다.')
    strip(d,'publish',720,'C · PUBLICATION — durable intent와 실제 사용자 전달은 다른 사실','blue',[
        ('게시 계획 기록',('내용·출처·identity·출력 세대','publication outbox와 내용 version')),
        ('Text / Voice 전달',('Text는 같은 ID로 upsert','Voice는 유효한 release만 재생')),
        ('전달 receipt 기록',('Text ack·확인된 audible 범위','interrupted / unknown 구분')),
        ('재시작 후 복원',('Text reconcile · 불명 음성 자동 재생 금지','Task 결과·허용 control 다시 연결')),
    ],'물리적 재생 뒤 receipt 저장 전 crash하면 일부 전달은 UNKNOWN이다. 계획한 전체 음성을 들려준 것으로 복원하지 않는다.')
    caption(d,90,1040,'새 입력 hold와 dispatch CAS는 같은 revision 경계에서 순서를 정한다. Acoustic 시작과 host가 보류를 기록한 시점은 별도로 남긴다.',16,COLORS['ink'],weight=700)
    return d


def runtime_processes() -> Diagram:
    d = detail('06-runtime-and-fault-boundaries', '배치와 장애 경계 — 논리 Component를 process 수와 혼동하지 않는다', '주 배치안 · VIA는 사용자 PC · 모델 / Agent의 local·remote 위치는 별도 binding', 1240)
    lane(d,'pc',45,150,1295,905,'VIA / USER PC — 상태 권위는 Core, 즉시 재생 중단은 Voice process',fill='#F8FAFC',stroke='#94A3B8')
    lane(d,'dependencies',1380,150,475,905,'DEPENDENCY BINDINGS — local 또는 remote',fill='#FCFCFD',dashed=True)
    card(d,'voice',90,225,'Voice Process',('Interaction Manager: audio·barge-in','Model Access: S2S adapter·session','Core 승인 lease·출력 세대 검사'),'blue',w=540,h=155)
    card(d,'s2s-port',835,225,'S2S stream port',('audio·transcript·generation','응답 생성도 같은 session owner 경유'),'blue',w=460,h=155)
    card(d,'s2s-dep',1420,225,'S2S Runtime × 1',('provider capability에 따라','local process / remote 연결'),'gray',w=390,h=155)
    right(d,'voice-s2s-port','voice','s2s-port',color='blue')
    right(d,'port-s2s','s2s-port','s2s-dep',color='blue')
    card(d,'core',90,480,'Core Process',('Controller·Interpreter·Context·Task','Response·Policy·Gateway 상태 권위','Model Access: semantic queue·adapter','Interaction Manager: timeline 결합'),'purple',w=540,h=155)
    card(d,'llm-port',835,480,'Semantic call port',('현재 입력 우선 · summary 예산','취소·deadline·versioned contract'),'purple',w=460,h=155)
    card(d,'llm-dep',1420,480,'Semantic LLM × 1',('모든 요청·요약이 같은 모델 공유','동시 처리·선점은 provider 기능'),'gray',w=390,h=155)
    right(d,'core-llm-port','core','llm-port',color='purple')
    right(d,'port-llm','llm-port','llm-dep',color='purple')
    edge(d,'voice-core',((360,380),(360,480)),'voice','core','bounded IPC ↔ lease',525,430,COLORS['blue'],bidirectional=True)
    card(d,'ui',90,745,'UI Process',('Chat·Task view·화면 capture','IM: Text·evidence buffer','Core와 bounded IPC'),'blue',w=250,h=155)
    card(d,'store',380,745,'Durable Store',('owner별 transaction','Core 종료 후에도 보존','별도 DB process는 선택'),'gray',w=250,h=155)
    card(d,'workers',835,745,'Connector Workers',('위험한 native·blocking 연동만 격리','Context / Agent adapter별 restart','Task별 process를 만들지 않음'),'orange',w=460,h=155)
    card(d,'external',1420,745,'Sources / Agents',('VIA Context는 bounded read','실제 업무 실행은 Agent 책임'),'orange',w=390,h=155)
    edge(d,'ui-local-stop',((90,800),(65,800),(65,425),(160,425),(160,380)),'ui','voice','local stop',160,415,COLORS['blue'])
    edge(d,'core-ui',((215,635),(215,745)),'core','ui','IPC',215,697,COLORS['blue'],bidirectional=True)
    edge(d,'core-store',((505,635),(505,745)),'core','store','repository',505,697,COLORS['gray'],bidirectional=True)
    edge(d,'core-worker',((630,600),(730,600),(730,822),(835,822)),'core','workers','bounded IPC',730,710,COLORS['orange'],bidirectional=True)
    right(d,'worker-dependency','workers','external',color='orange')
    for item in d.edges:
        if item.id in {'voice-s2s-port', 'port-s2s', 'core-llm-port', 'port-llm', 'worker-dependency'}:
            item.bidirectional = True
    for item in d.boxes:
        if item.id in {'s2s-port', 'llm-port'}:
            item.kind = 'dashed'
    caption(d,90,957,'Supervisor: process 재시작·health·backoff만 담당한다. Request·Task 의미 상태나 성공 여부를 결정하지 않는다.',16)
    caption(d,90,994,'Core 장애: 새 admission 중단 · Voice lease 만료 시 재생 중단 · UI는 마지막 확인 상태와 연결 문제 표시.',16)
    caption(d,90,1120,'Model Access는 하나의 논리 계약이다. S2S adapter는 Voice, semantic adapter는 Core에 배치하며 모델을 복제하지 않는다.',18,COLORS['ink'],weight=700)
    caption(d,90,1162,'Voice stop은 Core 응답을 기다리지 않는다. Task 취소는 Core의 내구 접수·Agent 확인이 필요하며 Core 장애 중 성공 접수를 약속하지 않는다.',16)
    return d


def four_asr_paths() -> Diagram:
    d = detail('07-four-asr-critical-paths', '네 품질 경로 — 구조가 좋아졌는지 무엇으로 판단하는가', '설계 우선순위 19 → 09 → 29 → 39 · 성능 수치가 아닌 인과 경로', 1430)
    strip(d,'accuracy',150,'1 · QA-19 SEMANTIC ACCURACY — 잘못 확정된 일관성도 실패다','purple',[
        ('입력·시점 근거',('원문·지칭 interval·정정','capture gap·source coverage')),
        ('의미·대상·Task',('경쟁 후보·필수 field·제약','host는 모델의 진실을 증명하지 못함')),
        ('처리·위임 계약',('direct / clarify / delegate','Agent·목표·대상·허용 Context')),
        ('실제 결과 연결',('응답·dispatch·상태 field','필수 association·uncertainty 보존')),
    ],'QA-19는 frozen QA-11/12 field의 중복 없는 micro-average. 추가 Task binding·continuity 검사는 별도 회귀 의미를 유지한다.')
    strip(d,'latency',435,'2 · QA-09 RESPONSIVENESS — 사용자·Agent source의 실제 사건에서 의미 있는 반응까지','blue',[
        ('실제 시작 사건',('사용자 input end / status source','final transcript·VIA receive로 대체 금지')),
        ('VIA 처리 경로',('잔여 해석·read·queue·Store·IPC','필요한 외부 연동·validation')),
        ('경로별 경계',('위임: Agent ingress까지 + 결과 이후','직접: Agent 실행 구간 없음')),
        ('실제 사용자 전달',('의미 있는 audible / UI endpoint','생성·enqueue·filler는 종료가 아님')),
    ],'입력 중 사전 준비는 종료 후 중복 합산하지 않는다. Barge-in QA-04는 별도 회귀이며 QA-09 평균에 넣지 않는다.')
    strip(d,'change',720,'3 · QA-29 MODIFIABILITY — 파일 수가 아니라 바뀐 Architecture Element','cyan',[
        ('변경 입력',('Model·Agent·Context·schema','동일 사용자 기능 유지')),
        ('변화 흡수 위치',('provider adapter·canonical port','capability 차이를 숨기지 않음')),
        ('전파 범위',('Component·Interface·State·Runtime','migration·호환성 영향 기록')),
        ('기능 유지 확인',('영향 Element의 합집합','미해결 변경을 작은 수치로 포장 금지')),
    ],'Adapter가 있어도 canonical 계약 자체가 달라지면 여러 owner가 바뀔 수 있다. 그림의 박스 개수는 점수가 아니다.')
    strip(d,'recovery',1005,'4 · QA-39 RELIABILITY / RECOVERABILITY — 재기동이 아니라 올바른 사용자 상태 복원','green',[
        ('Fault 발생',('Core·Voice·source·Agent·Store','영향 기능·Task 범위 식별')),
        ('격리·내구 복구',('queue·process·transaction 경계','command / event / publication')),
        ('Source와 조정',('외부 실행·결과·질문 재확인','중복 실행·stale state 방지')),
        ('복구 판정',('identity·상태·결과·control 복원','deadline·evidence·containment')),
    ],'확인 불가를 성공으로 세지 않는다. Safety·evidence qualification과 QA-41 memory 진단은 별도로 보존한다.')
    caption(d,90,1340,'이후 Decision Package에서는 네 축의 applicability와 trade-off를 각각 보고한다. 한 축으로 미리 후보를 제거하거나 가중 합산하지 않는다.',16,COLORS['ink'],weight=700)
    return d


def response_delivery() -> Diagram:
    d = detail('08-response-and-interruption', '응답 생성·게시·중단 — 한 요청에는 하나의 출력 소유권', '박스는 실행 단계 · 화면 상세와 Voice 요약은 같은 사실에서 별도로 구성한다', 1440)
    strip(d,'s2s',150,'A · S2S 직접 응답 — 명백한 자체 지식 질문만 허용','blue',[
        ('입력 stream',('Interaction Manager → Model Access','같은 S2S가 transcript·답변 생성')),
        ('Provisional generation',('IM buffer에 handle·audio 보류','input revision·출력 세대 결합')),
        ('Controller admission',('좁은 direct 허용 / 나머지는 Core','최소 admission 계약은 설계 중')),
        ('Response Manager',('publication 기록 후 handle release','IM에서만 실제 표시·재생')),
    ],'단순 첫 질문도 허용한다. 과거 대화 지칭·자료·Task 해석은 Core 책임이며 S2S에 Context 탐색·업무 planning을 붙이지 않는다.')
    strip(d,'core-response',435,'B · CORE / AGENT 응답 — 확인된 사실과 질문을 같은 출력 계약으로 전달','blue',[
        ('게시할 사실·질문',('Controller가 identity·scope admission','Task event는 새 발화 없이 도착')),
        ('필요한 응답 구성',('Response Manager → Model Access','template / 공유 LLM 요약 / 같은 S2S')),
        ('Publication outbox',('내용·source·version·출력 세대','확정 payload와 생성 내용 검사')),
        ('Interaction Manager',('상세 Text 표시 · Voice는 차례 대기','채널별 실제 전달 receipt 반환')),
    ],'상세 Text와 Voice 요약은 같은 문자열이 아니다. 대상·상태·중요한 실패는 일치시키고 Voice 요약문과 audio를 연결한다.')
    strip(d,'interrupt',720,'C · BARGE-IN — 의미 해석과 Agent 취소를 기다리지 않는 로컬 경로','red',[
        ('새 발화 감지',('acoustic onset과 감지 시각 구분','Interaction Manager의 local event')),
        ('즉시 재생 중단',('output epoch 증가·buffer 폐기','늦은 audio·이전 release 거절')),
        ('중단 사실 전달',('Response에 실제 audible 범위','Controller에 새 입력·hold event')),
        ('새 요청 해석',('정정 / 새 질문 / 업무 취소 구분','재개 의도가 불명확할 때만 확인')),
    ],'Core 장애·포화 중에도 로컬 stop은 유지한다. 재시작 후 확인되지 않은 음성 구간은 UNKNOWN이며 자동으로 재생하지 않는다.')
    strip(d,'notification',1005,'D · 비동기 알림 — 화면에 먼저 표시하고 사용자 발화가 끝난 뒤 말한다','blue',[
        ('Agent 결과 확인',('Task·Controller가 상태·결과 연결','실패·완료·입력 필요를 보존')),
        ('상세 Text 게시',('결과·실패 항목·근거·파일 링크','Voice 생성·대기를 기다리지 않음')),
        ('Response 발화 대기열',('사용자 발화 중 모든 음성 알림 보류','종료 뒤 새 입력과 충돌 여부 확인')),
        ('짧은 Voice 요약',('IM이 재생 직전 차례·epoch 검사','중요한 실패·불확실성 생략 금지')),
    ],'새 요청·기존 답변·대기 결과의 세부 순서는 제안 단계다. 짧은 침묵을 종료로 단정하지 않고 사용자가 다시 말하면 즉시 멈춘다.')
    caption(d,90,1340,'Text 표시와 Voice 요약의 실제 전달을 따로 기록한다. “방금 말한 것”은 상세 Text 전체가 아닌 실제 들려준 내용에서 찾는다.',16,COLORS['ink'],weight=700)
    return d


def compound_requests() -> Diagram:
    d = detail('09-compound-and-task-routing', '업무 구분 — 독립 목표는 VIA Task, 업무 내부 단계는 Agent', '“요약해서 메일로 보내줘”는 하나의 목표 · “보고서와 일정 등록”은 독립된 두 업무', 1130)
    strip(d,'single-goal',150,'A · 하나의 업무 — 요약과 발송을 통째로 같은 Agent에 위임','orange',[
        ('요청 의미 확정',('자료·수신자·제약·완료 조건','목표: 요약한 내용을 메일로 전달')),
        ('하나의 VIA Task',('Controller → Task Manager','요약·발송 전체 목표로 command')),
        ('Agent 내부 수행',('요약 → 발송의 순서·도구·재시도','VIA가 내부 단계별로 지휘하지 않음')),
        ('동일 Task 결과',('성공 / 부분 완료 / 실패 구분','요약만 됐으면 발송 완료가 아님')),
    ],'“조사해서 보고서를 만들어줘”도 같은 위임 경로다. Agent가 지원하지 못하면 VIA가 내부 계획을 대신 짜서 실행하지 않는다.')
    strip(d,'independent',435,'B · 독립된 두 업무 — 같은 Agent를 사용해도 Task identity는 분리','green',[
        ('독립 목표 식별',('보고서 작성 + 일정 등록','Request Interpreter가 의미 구분')),
        ('Task A / Task B',('각각 목표·결과·질문·제어 상태','동일 Agent 또는 서로 다른 Agent')),
        ('각각 진행·실패',('Task·Execution identity로 event 결합','한쪽 실패가 다른 업무를 막지 않음')),
        ('별도 결과 전달',('Task A 완료 / Task B 실패 등','전체 성공으로 뭉뚱그리지 않음')),
    ],'독립된 지속 업무는 별도 Task다. 일회성 직접 답변은 Request만으로 충분하며 모든 질문에 장기 Task를 만들지 않는다.')
    strip(d,'cross-goal',720,'C · 목표 사이 실제 의존 — 별도 Task의 확정 결과를 인계해야 할 때만 VIA가 연결','purple',[
        ('명시된 목표 간 관계',('별도 Task B가 Task A 결과를 사용','한 업무의 내부 단계와 구분')),
        ('선행 결과 확인',('실제 artifact ID·내용 version','실패·불명이면 후속 목표 보류')),
        ('후속 admission',('조건·권한·최신 요청 재검증','graph·node·result로 중복 해제 방지')),
        ('후속 Agent 수행',('Task B가 받은 결과 version 사용','내부 계획·실행은 해당 Agent 책임')),
    ],'사용자가 명시한 조건은 보존한다. 한 업무 내부 조건은 Agent가 수행하고 VIA는 업무 경계에서 확인된 사실만 연결한다.')
    caption(d,90,1040,'Task 개수는 동사나 Tool 개수가 아니라 독립된 사용자 업무로 정한다. 동일 Agent의 불투명한 실행 하나로 독립 Task를 합치지 않는다.',16,COLORS['ink'],weight=700)
    return d


def policy_memory() -> Diagram:
    d = detail('10-context-policy-and-memory', 'Context의 사용 수명 — 읽기·제공·기억을 각각 통제', 'Policy Manager가 권한 의미를 소유하고 실제 사용 port가 현재 revision을 강제한다', 1140)
    strip(d,'read',150,'A · CONTEXT READ — 읽을 수 있는 정보와 외부에 보낼 수 있는 정보는 다르다','cyan',[
        ('Controller 요청',('source·범위·목적·consumer 지정','현재 policy / consent 확인')),
        ('Context Manager',('허용 envelope 안에서 bounded read','source revision·receipt·후보 coverage')),
        ('Consumer view',('필요한 근거·출처만 구성','Conversation·Task는 owner read port')),
        ('실제 제공 직전 gate',('Model Access / Agent Gateway','recipient·scope·policy revision 검사')),
    ],'Cache hit도 새 사용이다. Source identity·revision·권한 범위가 맞지 않으면 재사용하지 않고 재조회 또는 동의를 요청한다.')
    strip(d,'revoke',435,'B · REVOCATION — “그 정보는 더 이상 쓰지 마”','amber',[
        ('Policy revision 변경',('새로운 사용·제공부터 차단','관련 request·consumer dependency 식별')),
        ('Cache·session 무효화',('Context view·모델 history 재사용 중단','진행 중 생성 취소 / 결과 폐기')),
        ('미전송·미게시 보류',('dispatch·release gate에서 재검사','오래된 admission으로 재개 금지')),
        ('이미 제공된 정보',('회수·외부 삭제 가능 범위 확인','지원되지 않는 회수를 성공으로 말하지 않음')),
    ],'사용자가 일반 질문을 할 때마다 승인하지 않는다. 현재 허용 범위가 충분하면 진행하고 필요한 새로운 범위만 확인한다.')
    strip(d,'memory',720,'C · USER MEMORY — 지속 선호를 대화 원문과 구분','green',[
        ('명시적 기억 요청',('확인 / 등록 / 수정 / 삭제','현재 요청의 지시가 저장 선호보다 우선')),
        ('Context owner 전이',('memory ID·version·삭제 tombstone','허용 범위 안에서 내구 기록')),
        ('파생 view 갱신',('cache·prompt·session·대기 생성 무효화','삭제된 선호를 history로 되살리지 않음')),
        ('결과와 한계 안내',('실제 적용·확인 범위를 Text로 기록','외부 제공 이력·보관 정책은 별도 관리')),
    ],'Memory 삭제는 원래 대화 전체 삭제와 같은 명령이 아니다. 보관기간·백업·외부 provider 삭제 보장은 제품 정책으로 별도 확정한다.')
    caption(d,90,1050,'Consent와 Agent Action Approval은 별도 계약이다. VIA는 질문·답을 정확히 중계하며 실제 업무 Action의 권한 강제는 Agent 책임이다.',16,COLORS['ink'],weight=700)
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
        response_delivery(),
        compound_requests(),
        policy_memory(),
    ]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Validate routes and checked-in pairs without writing files")
    args = parser.parse_args()
    generated = []
    for diagram in diagrams():
        validate_routes(diagram)
        for extension, render in [("svg", render_svg), ("drawio", render_drawio)]:
            value = render(diagram)
            root = ET.fromstring(value)
            ids = [node.attrib["id"] for node in root.iter() if "id" in node.attrib]
            if len(ids) != len(set(ids)):
                raise ValueError(f"{diagram.slug}.{extension}: duplicate XML IDs")
            generated.append((OUTPUT / f"{diagram.slug}.{extension}", value))
    if args.check:
        mismatches = [str(path.relative_to(ROOT)) for path, value in generated if not path.exists() or path.read_text(encoding="utf-8") != value]
        if mismatches:
            parser.error("missing or outdated diagram artifacts: " + ", ".join(mismatches))
        print(f"PASS: {len(generated)//2} diagram pairs match source; routes and XML IDs validated")
        return
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for path, value in generated:
        path.write_text(value, encoding="utf-8")
    print(f"generated {len(generated)//2} .drawio + .svg pairs")


if __name__ == "__main__":
    main()
