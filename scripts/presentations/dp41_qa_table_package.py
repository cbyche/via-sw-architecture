"""Replace only DP41's QA table, retaining every other native object verbatim."""
from pathlib import Path
import hashlib
import html
import json
import re
import sys
import xml.etree.ElementTree as ET
import zipfile

from dp_v2_intro_package import read, slides

NS = {'p': 'http://schemas.openxmlformats.org/presentationml/2006/main',
      'a': 'http://schemas.openxmlformats.org/drawingml/2006/main'}
SHAPE = re.compile(r'<p:sp(?:\s[^>]*)?>.*?</p:sp>', re.S)
LABELS = ['요청 처리 정확성', '상호작용 응답시간', 'VIA 요청 처리시간',
          'VIA 모델 사용 비용', 'VIA 변경 용이성', '로컬 VIA 메모리 사용량']


def table_shapes(raw):
    text = raw.decode('utf-8')
    tree = ET.fromstring(raw)
    shapes = tree.findall('./p:cSld/p:spTree/p:sp', NS)
    chunks = list(SHAPE.finditer(text))
    assert len(shapes) == len(chunks), 'Unexpected native object nesting'
    selected = []
    for shape, chunk in zip(shapes, chunks):
        props = shape.find('./p:nvSpPr/p:cNvPr', NS)
        assert f'id="{props.get("id")}"' in chunk[0]
        offset = shape.find('./p:spPr/a:xfrm/a:off', NS)
        if offset is not None and 934 <= int(offset.get('y')) / 9525 < 1053:
            selected.append((shape, chunk))
    assert selected, 'DP41 QA table not found'
    return text, shapes, selected


def graft(candidate, source, report):
    before, generated = read(source), read(candidate)

    def find(package):
        matches = [s for s in slides(package)
                   if any(n.startswith('dp41-comparison / ') for n in s['names'])]
        assert len(matches) == 1, matches
        return matches[0]['part']

    old, new = find(before), find(generated)
    original, shapes, old_table = table_shapes(before[old])
    _, _, new_table = table_shapes(generated[new])
    texts = [s.find('.//a:t', NS).text for s, _ in new_table
             if s.find('.//a:t', NS) is not None]
    assert len(new_table) == 24 and len(texts) == 18
    assert texts[::3] == LABELS, texts
    old_indices = {id(s) for s, _ in old_table}
    retained = [s for s in shapes if id(s) not in old_indices]
    next_id = max(int(s.find('./p:nvSpPr/p:cNvPr', NS).get('id'))
                  for s in retained) + 1
    replacements = []
    for index, (_, chunk) in enumerate(new_table):
        block = re.sub(r'(<p:cNvPr id=")\d+',
                       lambda m: m[1]+str(next_id+index), chunk[0], count=1)
        block = re.sub(r'(<p:cNvPr\b[^>]*\bname=")[^"]*',
                       lambda m: m[1]+html.escape(
                           f'dp41-comparison / asr-qa-table / {index+1}', quote=True),
                       block, count=1)
        replacements.append(block)
    spans = {c.start(): c for _, c in old_table}
    first = old_table[0][1].start()
    updated = SHAPE.sub(lambda m: (''.join(replacements) if m.start() == first else '')
                        if m.start() in spans else m[0], original)
    # Prove preservation at the raw object level, not merely after rendering.
    old_blocks = [m[0] for m in SHAPE.finditer(original) if m.start() not in spans]
    new_blocks = [m[0] for m in SHAPE.finditer(updated)
                  if 'name="dp41-comparison / asr-qa-table / ' not in m[0]]
    assert new_blocks == old_blocks
    ids = [e.get('id') for e in ET.fromstring(updated).findall('.//p:cNvPr', NS)]
    assert len(ids) == len(set(ids)), 'Duplicate native shape IDs'
    parts = dict(before)
    parts[old] = updated.encode('utf-8')
    changed = {n for n in before if before[n] != parts[n]}
    assert changed == {old}
    tmp = candidate.with_suffix('.qa-only.pptx')
    with zipfile.ZipFile(source) as original_zip, zipfile.ZipFile(tmp, 'w') as out:
        for info in original_zip.infolist():
            out.writestr(info, parts[info.filename])
    tmp.replace(candidate)
    report.write_text(json.dumps(dict(
        source=str(source), source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
        slide_count=len(slides(parts)), changed_parts=sorted(changed),
        old_table_shapes=len(old_table), new_table_shapes=len(new_table),
        retained_native_objects=len(old_blocks),
        all_other_objects_preserved_verbatim=True,
        all_other_parts_and_notes_preserved=True), indent=2)+'\n')


if __name__ == '__main__':
    assert len(sys.argv) == 4
    graft(*map(Path, sys.argv[1:]))
