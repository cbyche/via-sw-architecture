#!/usr/bin/env python3
"""Check editable DP46 deck text/geometry source and manifest, without rendering."""
import hashlib
import json
from collections import Counter
from pathlib import Path
import sys
import xml.etree.ElementTree as ET
import zipfile

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts/architecture'))
import generate_evidence_context_diagrams as g


def main():
    folder = ROOT / 'docs/presentations_files/dp46-evidence'
    manifest = json.loads((folder / 'native-source-manifest.json').read_text())
    scenes = [g.background(), g.structure(), g.quality(), g.timeline(), g.lifecycle()]
    ns = {'a': 'http://schemas.openxmlformats.org/drawingml/2006/main',
          'p': 'http://schemas.openxmlformats.org/presentationml/2006/main'}
    assert manifest['status'] == 'DOCUMENTATION_NOT_IMPLEMENTED_NOT_MEASURED'
    assert len(manifest['slides']) == len(scenes) == 5
    with zipfile.ZipFile(folder / 'VIA-DP46-evidence.pptx') as z:
        names = z.namelist()
        slide_names = [n for n in names if n.startswith('ppt/slides/slide') and n.endswith('.xml')]
        assert len(slide_names) == 5
        for index, (scene, entry) in enumerate(zip(scenes, manifest['slides']), 1):
            assert entry['slug'] == scene.slug
            source = ROOT / entry['source']
            assert source.read_text() == scene.render()['svg']
            assert hashlib.sha256(source.read_bytes()).hexdigest() == entry['svg_sha256']
            slide = ET.fromstring(z.read(f'ppt/slides/slide{index}.xml'))
            assert not slide.findall('.//p:pic', ns), 'slide must not be flattened to raster'
            expected = Counter(line for n in scene.nodes for line in n['name'].split('\n'))
            expected.update(line for t in scene.texts for line in t['lines'])
            actual = Counter(t.text or '' for t in slide.findall('.//a:t', ns))
            assert actual == expected, (scene.slug, 'native text differs from editable source')
            assert len(slide.findall('.//p:sp', ns)) >= sum(expected.values())
            notes = z.read(f'ppt/notesSlides/notesSlide{index}.xml').decode()
            assert '04-46-input-and-context-evidence.md' in notes
    print('PASS: DP46 five editable slides, exact authored text, no raster slides, notes and source hashes')


if __name__ == '__main__':
    main()
