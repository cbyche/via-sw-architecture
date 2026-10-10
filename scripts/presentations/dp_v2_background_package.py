"""Graft selected native background slides into v2; preserve every other part."""
from __future__ import annotations

import hashlib
import json
import sys
import zipfile
from pathlib import Path
import xml.etree.ElementTree as E

from dp_pptx_package import A, R, NS, related, rels_path, target_part
from dp_v2_intro_package import PR, read, slides, xml_bytes


def graft(candidate_path, source_path, manifest_path, report_path):
    generated, before = read(candidate_path), read(source_path)
    parts = dict(before)
    manifest = json.loads(manifest_path.read_text())
    current, authored = slides(before), slides(generated)
    assert len(current) == 12 and len(authored) == len(manifest)
    mutable = set()
    selected = set()
    for item, src in zip(manifest, authored):
        slug = item['slug']
        assert any(n.startswith(slug+' / ') for n in src['names'])
        matches = [s for s in current if any(n.startswith(slug+' / ') for n in s['names'])]
        assert len(matches) == 1, (slug, len(matches))
        dst = matches[0]
        selected.add(dst['part'])
        old_note = related(before, dst['part'], 'notesSlide')
        new_note = related(generated, src['part'], 'notesSlide')
        rels = E.fromstring(before[rels_path(dst['part'])])
        for rel in list(rels):
            if rel.get('Id', '').startswith('RviaBackgroundImage'):
                rels.remove(rel)
        root = E.fromstring(generated[src['part']])
        count = 0
        for rel in E.fromstring(generated[rels_path(src['part'])]):
            if not rel.get('Type', '').endswith('/image'):
                continue
            count += 1
            image = target_part(src['part'], rel.get('Target'))
            media = f"ppt/media/via-v2-bg{item['dp']}-{count}{Path(image).suffix}"
            parts[media] = generated[image]
            mutable.add(media)
            rid = f'RviaBackgroundImage{count}'
            assert rid not in {r.get('Id') for r in rels}
            for node in root.findall('.//a:blip', NS):
                if node.get(f'{{{R}}}embed') == rel.get('Id'):
                    node.set(f'{{{R}}}embed', rid)
            E.SubElement(rels, f'{{{PR}}}Relationship', {'Id':rid, 'Type':rel.get('Type'), 'Target':'/'+media})
        assert 1 <= count <= 3
        pictures = root.findall('.//p:pic', NS)
        assert len(pictures) == len(item['crops']) == 3
        # artifact-tool currently omits asymmetric authored crop metadata.
        # Apply native PowerPoint image framing; the image pixels stay intact.
        for picture, values in zip(pictures, item['crops']):
            crop = picture.find('.//a:srcRect', NS)
            assert crop is not None
            crop.attrib.update(dict(zip(('l','t','r','b'), map(str, values))))
        E.register_namespace('a', A)
        parts[dst['part']] = xml_bytes(root)
        parts[rels_path(dst['part'])] = xml_bytes(rels)
        parts[old_note] = generated[new_note]
        mutable.update((dst['part'], rels_path(dst['part']), old_note))

    # v2 already declares PNG from its two intro illustrations.
    ct = E.fromstring(parts['[Content_Types].xml'])
    assert any(n.get('Extension') == 'png' for n in ct)
    changed = [p for p in before if before[p] != parts[p]]
    assert set(changed) <= mutable, sorted(set(changed)-mutable)
    for s in current:
        if s['part'] not in selected:
            assert parts[s['part']] == before[s['part']]
            note = related(before, s['part'], 'notesSlide')
            assert parts[note] == before[note]
    assert len(slides(parts)) == 12
    tmp = Path(str(candidate_path)+'.scoped')
    with zipfile.ZipFile(tmp,'w',zipfile.ZIP_DEFLATED) as z:
        for name, data in parts.items():
            z.writestr(name,data)
    tmp.replace(candidate_path)
    report_path.write_text(json.dumps({
        'source':str(source_path),
        'source_sha256':hashlib.sha256(source_path.read_bytes()).hexdigest(),
        'selected_backgrounds':[m['dp'] for m in manifest],
        'unselected_slides_preserved':len(current)-len(selected),
        'all_other_package_parts_preserved':True,
        'changed_existing_parts':changed,
        'new_parts':sorted(set(parts)-set(before)),
    },indent=2)+'\n')


if __name__ == '__main__':
    assert len(sys.argv) == 5
    graft(*map(Path,sys.argv[1:]))
