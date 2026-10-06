#!/usr/bin/env python3
"""Check source coverage and agreement of the VIA quality presentation assets."""

from pathlib import Path
from zipfile import ZipFile
from html.parser import HTMLParser
import json
import re
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs/presentations_files/quality-attributes"
DATA = json.loads((OUT / "quality-attributes.json").read_text())
SOURCE = (ROOT / DATA["source"]).read_text()
ROWS = DATA["rows"]
NS = {"a": "http://schemas.openxmlformats.org/drawingml/2006/main"}


def texts(element):
    return "".join(node.text or "" for node in element.findall(".//a:t", NS))


def main():
    source_names = dict(re.findall(r"^\| \*\*(V-\d\d) (.+?)\*\* \|", SOURCE, re.M))
    rank_section = SOURCE.split("### 2.1 이번 비교의 우선순위", 1)[1].split("### 2.2", 1)[0]
    source_order = []
    source_priorities = {}
    for line in rank_section.splitlines():
        rank = re.match(r"\| (?:공동 )?(\d)순위 \|", line)
        if rank:
            for item in re.findall(r"V-\d\d", line):
                source_order.append(item)
                source_priorities[item] = int(rank.group(1))
    assert len(ROWS) == 13 and len(source_names) == 13
    assert [row["id"] for row in ROWS] == source_order
    for row in ROWS:
        assert row["name"] == source_names[row["id"]], row["id"]
        assert row["priority"] == source_priorities[row["id"]], row["id"]
        assert len(re.findall(r"[.!?。]", row["description"])) == 1
        assert row["description"].endswith(".")
        for field in ("target", "method", "detail", "targetBasis", "boundary", "category"):
            assert row[field].strip(), (row["id"], field)

    markdown_path = OUT.parent / "quality-attributes.md"
    markdown = markdown_path.read_text()
    table_rows = [[cell.strip() for cell in line.strip("|").split("|")] for line in markdown.splitlines()
                  if re.match(r"\| (?:공동 )?\d순위 \| V-", line)]
    assert len(table_rows) == 13
    for row, cells in zip(ROWS, table_rows):
        assert cells[1] == row["id"]
        assert cells[2:6] == [row["name"], row["description"], row["target"], row["method"]]
    for document in (markdown_path, OUT / "README.md", OUT / "verification.md"):
        for target in re.findall(r"!?\[[^\]]*\]\(([^)]+)\)", document.read_text()):
            local = target.split("#", 1)[0]
            if local and not local.startswith(("https:", "http:")):
                assert (document.parent / local).exists(), local

    all_body = []
    with ZipFile(OUT / "VIA-quality-attributes.pptx") as pptx:
        for number in (1, 2):
            root = ET.fromstring(pptx.read(f"ppt/slides/slide{number}.xml"))
            tables = root.findall(".//a:tbl", NS)
            assert len(tables) == 1, "Exactly one editable native table per page"
            body = tables[0].findall("a:tr", NS)[1:]
            assert len(body) == (8 if number == 1 else 5)
            notes = texts(ET.fromstring(pptx.read(f"ppt/notesSlides/notesSlide{number}.xml")))
            for native, row in zip(body, ROWS[:8] if number == 1 else ROWS[8:]):
                cells = native.findall("a:tc", NS)
                assert len(cells) == 4
                assert texts(cells[0]) == row["id"]
                name_paras = cells[1].findall("a:txBody/a:p", NS)
                native_name = " ".join(texts(p) for p in name_paras).replace("\n", " ")
                assert native_name == row["name"], (row["id"], native_name)
                paras = cells[2].findall("a:txBody/a:p", NS)
                assert [texts(p) for p in paras] == [row["description"], "목표  " + row["target"], "측정  " + row["method"]]
                expected_rank = str(row["priority"]) + (" (공동)" if row["priority"] in (1, 2, 6) else "")
                assert texts(cells[3]) == expected_rank
                for field in ("detail", "targetBasis", "boundary"):
                    assert row[field] in notes, (row["id"], "notes", field)
                all_body.append(row["id"])
        assert all_body == source_order

    class Links(HTMLParser):
        def handle_starttag(self, tag, attrs):
            for key, value in attrs:
                if key in ("src", "href"):
                    assert value and (OUT / value).exists(), value

    Links().feed((OUT / "index.html").read_text())
    for number in (1, 2):
        assert (OUT / f"quality-attributes-0{number}.png").read_bytes().startswith(b"\x89PNG\r\n\x1a\n")
    print("PASS: source V names/priorities, all 13 IDs, one-sentence explanations, MD/PPT/notes agreement, 2 native tables and local asset links")


if __name__ == "__main__":
    main()
