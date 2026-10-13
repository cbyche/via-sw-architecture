"""Check DP44 figure visibility, punctuation, paired sources and rendered previews."""
from __future__ import annotations

import hashlib
import html
import json
import re
from pathlib import Path
import xml.etree.ElementTree as ET

from via_problem_intro_scenes import interaction

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / 'docs/architecture/12-decisions/decision-packages'
DIAGRAMS = BASE / 'diagrams'
FOCUS = ROOT / 'docs/presentations_files/dp-comparison'
BACKGROUND = ROOT / 'docs/presentations_files/dp-background'
FIGURES = {
    name: (DIAGRAMS, DIAGRAMS) for name in (
        'dp44-common-path', 'choice44-structure', 'choice44-event',
        'dp44-implementation-a', 'dp44-implementation-b')
}
FIGURES.update({'dp44-background': (DIAGRAMS, BACKGROUND),
                'dp44-comparison': (FOCUS, FOCUS)})
DOTS = {chr(code) for code in (0x00B7, 0x318D, 0x2022, 0x2027, 0x30FB, 0x22C5, 0x2219)}


def main():
    document = BASE / '04-44-continuous-interaction.md'
    text = document.read_text()
    embedded = {(BASE / target).resolve() for target in re.findall(r'!\[[^\]]*\]\(([^)]+)\)', text)}
    expected = {directory / (stem + '.svg') for stem, (directory, _) in FIGURES.items()}
    assert embedded == expected, f'DP44 figures missing or unexpected: {embedded ^ expected}'
    files = [document, BASE / '04-44-interaction-review.md', BASE / 'dp44-implementation-structure.md',
             DIAGRAMS / 'choice44-review.html']
    for stem, (directory, png_directory) in FIGURES.items():
        for extension in ('svg', 'drawio'):
            path = directory / (stem + '.' + extension)
            ET.fromstring(path.read_text())
            files.append(path)
        assert (png_directory / (stem + '.png')).is_file(), f'Missing PNG: {stem}'
    for path in files:
        assert not (DOTS & set(html.unescape(path.read_text()))), f'Forbidden punctuation: {path}'

    scene = interaction()
    for extension in ('svg', 'drawio'):
        assert (DIAGRAMS / f'dp44-background.{extension}').read_text() == getattr(scene, extension)(), f'Background {extension} drift'
    bundle = ET.parse(BACKGROUND / 'VIA-problem-intros-v8.drawio').getroot()
    page = bundle.find("diagram[@id='dp44-background']")
    assert ET.tostring(page) == ET.tostring(ET.fromstring(scene.drawio())[0]), 'Background bundle page drift'

    records = json.loads((DIAGRAMS / 'dp44-document-render.json').read_text())['records']
    assert {row['stem'] for row in records} == set(FIGURES), 'Incomplete render record'
    for row in records:
        assert row['text_geometry_issues'] == 0, row
        directory, png_directory = FIGURES[row['stem']]
        for extension, parent in (('svg', directory), ('png', png_directory)):
            digest = hashlib.sha256((parent / (row['stem'] + '.' + extension)).read_bytes()).hexdigest()
            assert digest == row[extension + '_sha256'], f'Stale {extension} render: {row["stem"]}'
    print('PASS: all seven DP44 figures embedded; no middle-dot punctuation; paired source and render consistency')


if __name__ == '__main__':
    main()
