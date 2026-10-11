"""Preserve native PPTX parts while rounding typography and replacing prose.

Artifact Tool authors the requested text. This packager copies only that prose
into the existing shapes, so media, relationships and native geometry survive.
"""
from pathlib import Path
from collections import Counter
import copy
import html
import json
import re
import sys
import xml.etree.ElementTree as E

import full_presentation_package as F


FONT_TAGS = {'rPr', 'defRPr', 'endParaRPr'}


def font_values(data):
    root = E.fromstring(data)
    for node in root.iter():
        if node.tag in {'{'+F.A+'}'+tag for tag in FONT_TAGS} and node.get('sz'):
            yield int(node.get('sz'))
        if node.tag == '{'+F.A+'}buSzPts':
            yield int(node.get('val'))


def round_fonts(data):
    count = 0
    changes = Counter()

    def replace_tag(match):
        nonlocal count
        qname, attrs = match.group(1), match.group(2)
        local = qname.split(':')[-1]
        attr = 'sz' if local in FONT_TAGS else 'val' if local == 'buSzPts' else None
        if attr is None:
            return match.group(0)

        def replace_size(size):
            nonlocal count
            old = int(size.group(1))
            new = ((old+50)//100)*100
            assert new > 0
            if new != old:
                count += 1
                changes[(old, new)] += 1
            return f'{attr}="{new}"'

        attrs = re.sub(r'(?<![\w:])'+attr+r'="(\d+)"', replace_size, attrs)
        return '<'+qname+attrs+'>'

    result = re.sub(r'<([\w:]+)([^<>]*)>', replace_tag, data.decode('utf-8-sig')).encode()
    assert all(value % 100 == 0 for value in font_values(result))
    return result, count, changes


def build(source_path, authored_path, previous_path, plan_path, output_path, manifest_path):
    source_path, authored_path, previous_path, plan_path, output_path, manifest_path = map(
        Path, (source_path, authored_path, previous_path, plan_path, output_path, manifest_path))
    source, authored = F.read(source_path), F.read(authored_path)
    previous, plan = json.loads(previous_path.read_text()), json.loads(plan_path.read_text())
    assert F.digest(source_path.read_bytes()) == previous['output_sha256'] == plan['source_sha256']
    order, authored_order = F.ordered(source), F.ordered(authored)
    assert len(order) == len(authored_order) == 61
    parts = dict(source)
    for change in plan['replacements']:
        page = change['page']
        authored_text = ''.join(n.text or '' for n in E.fromstring(authored[authored_order[page-1][1]]).findall('.//a:t', F.NS))
        assert change['text'] in authored_text, (page, change['text'])
        part = order[page-1][1]
        old = '<a:t>'+html.escape(change['old'], quote=False)+'</a:t>'
        new = '<a:t>'+html.escape(change['text'], quote=False)+'</a:t>'
        content = parts[part].decode('utf-8-sig')
        assert content.count(old) == 1, (page, change['old'])
        parts[part] = content.replace(old, new, 1).encode()
    for addition in plan['notes_append']:
        note = F.related(source, order[addition['page']-1][1], 'notesSlide')
        root = E.fromstring(parts[note])
        bodies = [sp.find('p:txBody', F.NS) for sp in root.findall('.//p:sp', F.NS)
                  if sp.find('p:nvSpPr/p:nvPr/p:ph', F.NS) is not None
                  and sp.find('p:nvSpPr/p:nvPr/p:ph', F.NS).get('type') == 'body']
        assert len(bodies) == 1 and bodies[0] is not None
        paragraph = E.SubElement(bodies[0], '{'+F.A+'}p')
        run = E.SubElement(paragraph, '{'+F.A+'}r')
        E.SubElement(run, '{'+F.A+'}t').text = addition['text']
        parts[note] = F.xml(root)
    changes, changed_parts, size_changes = 0, {}, Counter()
    for name, data in list(parts.items()):
        if name.endswith('.xml'):
            normalized, count, sizes = round_fonts(data)
            parts[name] = normalized
            changes += count
            size_changes.update(sizes)
            if count:
                changed_parts[name] = count
    assert F.ordered(parts) == order
    for name, data in source.items():
        if name.startswith('ppt/media/') or name.endswith('.rels'):
            assert parts[name] == data
    F.write(output_path, parts)
    rows = copy.deepcopy(previous['slides'])
    for row in rows:
        row['source_page'] = row['page']
        row['source_key'] = row['key']
        row['kind'] = 'typography-normalized'
        row['source_slide_sha256'] = F.digest(source[row['part']])
        row['slide_sha256'] = F.digest(parts[row['part']])
    manifest = {'version': 4, 'source_path': 'docs/presentations_files/VIA-architecture-presentation_v3.pptx',
                'source_sha256': plan['source_sha256'],
                'output_path': 'docs/presentations_files/VIA-architecture-presentation_v4.pptx',
                'output_sha256': F.digest(output_path.read_bytes()), 'main_slides': 24,
                'appendix_divider_page': 25, 'preserved_v2_background_pages': [58,59,60,61],
                'slide_order_ids_and_keys_preserved': True, 'media_and_relationships_byte_identical': True,
                'font_size_policy': {'unit': 'pt', 'rounding': 'nearest_integer_half_up',
                                     'scope': 'all DrawingML text and absolute bullet sizes, including notes, masters, layouts and themes',
                                     'changed_properties': changes, 'remaining_fractional_properties': 0,
                                     'size_mapping': [{'from_pt':old/100, 'to_pt':new//100, 'properties':count}
                                                      for (old,new),count in sorted(size_changes.items())]},
                'question_and_context_replacements': plan['replacements'],
                'prior_context_preserved_in_notes_pages': [15,17,19,21], 'slides': rows}
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2)+'\n')
    print(json.dumps({'slides':61, 'text_replacements':len(plan['replacements']),
                      'font_properties_normalized':changes, 'parts_normalized':len(changed_parts)}))


if __name__ == '__main__':
    build(*sys.argv[1:])
