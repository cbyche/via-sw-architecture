"""Insert the eight DP3 CPU slides into an exact 21-slide v5-r2 snapshot.

The 21 original slide XML parts change only in their existing footer text.
Their notes, relationships and all original visual dependencies stay byte-identical.
Usage: python3 package_dp3_cpu_v6.py REPOSITORY BUILD_DIRECTORY
"""
from pathlib import Path
import argparse
import copy
import json
import re
import uuid
import xml.etree.ElementTree as E

import full_presentation_package as F
import presentation_windows_policy as W


TOTAL = 29
ORIGINAL_COUNT = 21
AUTHORED_COUNT = 8
NEW_PARTS = [f'ppt/slides/slide{309 + i}.xml' for i in range(AUTHORED_COUNT)]


def replace_footer_bytes(data, old, new):
    """Replace the single footer text node without reserializing an original slide."""
    root = E.fromstring(data)
    shapes = [s for s in root.findall('.//p:sp', F.NS)
              if ''.join(t.text or '' for t in s.findall('.//a:t', F.NS)) == old]
    assert len(shapes) == 1, ('Expected one exact original footer', old, len(shapes))
    assert len(shapes[0].findall('.//a:t', F.NS)) == 1, ('Split footer text', old)
    tag = rb'(?:[A-Za-z_][\w.-]*:)?t'
    pattern = re.compile(rb'(<'+tag+rb'\b[^>]*>)'+re.escape(old.encode())+
                         rb'(</'+tag+rb'>)')
    result, count = pattern.subn(lambda m: m[1] + new.encode() + m[2], data)
    assert count == 1, ('Expected one raw footer text node', old, count)
    restored, count = re.subn(rb'(<'+tag+rb'\b[^>]*>)'+re.escape(new.encode())+
                              rb'(</'+tag+rb'>)',
                              lambda m: m[1] + old.encode() + m[2], result)
    assert count == 1 and restored == data, ('Original body bytes changed', old)
    return result


def renumber_authored_footer(data, local_page, final_page):
    root = E.fromstring(data)
    recognized = {str(local_page), f'{local_page} / 8', str(final_page),
                  f'{final_page} / {TOTAL}', f'{local_page} / {TOTAL}'}
    candidates = []
    for shape in root.findall('.//p:sp', F.NS):
        spans = shape.findall('.//a:t', F.NS)
        text = ''.join(s.text or '' for s in spans)
        off = shape.find('p:spPr/a:xfrm/a:off', F.NS)
        if (text in recognized and off is not None
                and int(off.get('y', '0')) > F.SIZE[1] * .9):
            candidates.append(shape)
    assert len(candidates) == 1, ('Authored footer not unique', local_page, len(candidates))
    shape = candidates[0]
    spans = shape.findall('.//a:t', F.NS)
    assert len(spans) == 1, ('Split authored footer', local_page)
    spans[0].text = f'{final_page} / {TOTAL}'
    shape.find('p:nvSpPr/p:cNvPr', F.NS).set('name', 'Continuous slide number')
    transform = shape.find('p:spPr/a:xfrm', F.NS)
    transform.find('a:off', F.NS).attrib.update(x=str(1836 * 9525), y=str(1052 * 9525))
    transform.find('a:ext', F.NS).attrib.update(cx=str(62 * 9525), cy=str(21 * 9525))
    return F.xml(root)


def synchronize_sections(root, rows):
    """Keep existing section metadata, adding/splitting groups in the new order."""
    lists = [n for n in root.iter() if n.tag.split('}')[-1] == 'sectionLst']
    if not lists:
        return {'present_in_source': False, 'changed': False}
    assert len(lists) == 1, 'Multiple section lists require explicit handling'
    listing = lists[0]
    namespace = listing.tag.split('}')[0] + '}'
    memberships, attributes = {}, {}
    for index, section in enumerate(list(listing)):
        key = ('original', index)
        attributes[key] = dict(section.attrib)
        for node in section.iter():
            if node.tag.split('}')[-1] == 'sldId':
                sid = node.get('id')
                assert sid not in memberships, ('Duplicate section membership', sid)
                memberships[sid] = key
    new_attributes = {
        ('new', 'main'): {'name': 'DP3 CPU 화면 생산과 실행 구조'},
        ('new', 'appendix'): {'name': '부록 · 이전 DP3 비교 보존'},
        ('original', 'unsectioned'): {'name': '원본 슬라이드'},
    }
    attributes.update(new_attributes)
    groups = []
    for row in rows:
        if row['kind'] == 'authored':
            key = ('new', 'appendix' if row['source_page'] == 8 else 'main')
        elif 9 <= row['source_page'] <= 13:
            key = ('new', 'appendix')
        else:
            key = memberships.get(str(row['slide_id']), ('original', 'unsectioned'))
        if not groups or groups[-1][0] != key:
            groups.append((key, []))
        groups[-1][1].append(row)
    for node in list(listing):
        listing.remove(node)
    used_ids, report = set(), []
    for index, (key, members) in enumerate(groups):
        attrs = dict(attributes[key])
        if not attrs.get('id') or attrs['id'] in used_ids:
            attrs['id'] = '{' + str(uuid.uuid5(
                uuid.NAMESPACE_URL, f'via:dp3-cpu-v6:section:{index}:{key}')).upper() + '}'
        used_ids.add(attrs['id'])
        section = E.SubElement(listing, namespace + 'section', attrs)
        ids = E.SubElement(section, namespace + 'sldIdLst')
        for row in members:
            E.SubElement(ids, namespace + 'sldId', {'id': str(row['slide_id'])})
        report.append({'name': attrs.get('name'), 'first_page': members[0]['page'],
                       'last_page': members[-1]['page']})
    listed = [n.get('id') for n in listing.iter() if n.tag.split('}')[-1] == 'sldId']
    assert listed == [str(r['slide_id']) for r in rows]
    return {'present_in_source': True, 'changed': True, 'sections': report}


def main(repo, build):
    repo, build = repo.resolve(), build.resolve()
    assert (repo / 'INTENT.md').is_file(), ('Not the VIA repository', str(repo))
    source_path = build / 'source-v5-r2.pptx'
    authored_path = build / 'dp3-cpu-authored.pptx'
    output_path = build / 'candidate.pptx'
    report_path = build / 'preservation.json'
    assert not output_path.exists(), 'Refusing to overwrite candidate.pptx'
    assert not report_path.exists(), 'Refusing to overwrite preservation.json'
    source_bytes, authored_bytes = source_path.read_bytes(), authored_path.read_bytes()
    original, authored_raw = F.read(source_path), F.read(authored_path)
    original_order, authored_order = F.ordered(original), F.ordered(authored_raw)
    assert len(original_order) == ORIGINAL_COUNT
    assert len(authored_order) == AUTHORED_COUNT
    for name, collection in [('original', original), ('authored', authored_raw)]:
        size = E.fromstring(collection['ppt/presentation.xml']).find('p:sldSz', F.NS)
        assert (int(size.get('cx')), int(size.get('cy'))) == F.SIZE, (name, size.attrib)
    # Preserve existing typography exactly; apply the policy only to new content.
    original_policy = W.policy_report(original)
    authored = {name: W.normalize_xml(name, data) if name.endswith('.xml') else data
                for name, data in authored_raw.items()}
    authored_policy = W.policy_report(authored)
    parts = dict(original)
    for part in NEW_PARTS:
        number = re.search(r'slide(\d+)\.xml$', part)[1]
        note = f'ppt/notesSlides/notesSlide{number}.xml'
        for reserved in (part, F.rels_path(part), note, F.rels_path(note)):
            assert reserved not in parts, ('Reserved part collision', reserved)
    importer = F.Importer(parts)
    imported, scale, media = importer.import_slides(
        authored, [part for _, part in authored_order], 'dp3cpuv6', NEW_PARTS)
    assert scale == 1
    F.add_registry(parts, importer)

    presentation = E.fromstring(parts['ppt/presentation.xml'])
    relationships = E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
    slide_list = presentation.find('p:sldIdLst', F.NS)
    original_nodes = list(slide_list)
    max_id = max(int(n.get('id')) for n in original_nodes)
    new_nodes = []
    for local_page, part in enumerate(NEW_PARTS, 1):
        rid = f'Dp3CpuV6Slide{local_page}'
        assert not any(r.get('Id') == rid for r in relationships)
        node = E.Element('{' + F.P + '}sldId',
                         {'id': str(max_id + local_page), '{' + F.R + '}id': rid})
        new_nodes.append(node)
        E.SubElement(relationships, '{' + F.PR + '}Relationship',
                     Id=rid, Type=F.R + '/slide', Target='/' + part)
    plan = ([('original', p) for p in range(1, 9)]
            + [('authored', p) for p in range(1, 8)]
            + [('original', p) for p in range(14, 22)]
            + [('authored', 8)]
            + [('original', p) for p in range(9, 14)])
    assert len(plan) == TOTAL and len(set(plan)) == TOTAL
    for node in list(slide_list):
        slide_list.remove(node)
    rows, preserved = [], []
    for page, (kind, source_page) in enumerate(plan, 1):
        is_original = kind == 'original'
        part = original_order[source_page - 1][1] if is_original else NEW_PARTS[source_page - 1]
        node = original_nodes[source_page - 1] if is_original else new_nodes[source_page - 1]
        slide_list.append(copy.deepcopy(node))
        row = {'page': page, 'kind': kind, 'source_page': source_page,
               'slide_id': int(node.get('id')), 'part': part}
        if is_original:
            old_footer, new_footer = f'{source_page} / {ORIGINAL_COUNT}', f'{page} / {TOTAL}'
            parts[part] = replace_footer_bytes(original[part], old_footer, new_footer)
            note = F.related(original, part, 'notesSlide')
            assert parts[note] == original[note]
            assert parts[F.rels_path(part)] == original[F.rels_path(part)]
            preserved.append({**row, 'original_footer': old_footer, 'footer': new_footer,
                              'source_slide_sha256': F.digest(original[part]),
                              'slide_sha256': F.digest(parts[part]),
                              'body_bytes_preserved_except_footer': True,
                              'notes_part': note, 'notes_sha256': F.digest(original[note]),
                              'notes_and_relationships_byte_preserved': True})
        else:
            parts[part] = renumber_authored_footer(parts[part], source_page, page)
            row['source_part'] = authored_order[source_page - 1][1]
        rows.append(row)
    section_report = synchronize_sections(presentation, rows)
    parts['ppt/presentation.xml'] = F.xml(presentation)
    parts['ppt/_rels/presentation.xml.rels'] = F.xml(relationships)
    app = E.fromstring(parts['docProps/app.xml'])
    for tag in ('Slides', 'Notes'):
        matches = [n for n in app if n.tag.split('}')[-1] == tag]
        assert len(matches) == 1, ('Missing/duplicate document statistic', tag)
        matches[0].text = str(TOTAL)
    parts['docProps/app.xml'] = F.xml(app)

    metadata = {'ppt/presentation.xml', 'ppt/_rels/presentation.xml.rels',
                '[Content_Types].xml', 'docProps/app.xml'}
    allowed = metadata | {part for _, part in original_order}
    changed = {name for name, data in original.items() if parts[name] != data}
    assert changed <= allowed, ('Unexpected existing-part change', sorted(changed - allowed))
    protected_prefixes = ('ppt/slideMasters/', 'ppt/slideLayouts/', 'ppt/theme/',
                          'ppt/media/', 'ppt/notesSlides/', 'ppt/notesMasters/')
    for name, data in original.items():
        if name.startswith(protected_prefixes):
            assert parts[name] == data, ('Original dependency changed', name)
    assert F.ordered(parts) == [(str(row['slide_id']), row['part']) for row in rows]
    for row in rows:
        texts = [''.join(n.text or '' for n in s.findall('.//a:t', F.NS))
                 for s in E.fromstring(parts[row['part']]).findall('.//p:sp', F.NS)]
        assert texts.count(f"{row['page']} / {TOTAL}") == 1, row
    policy = W.policy_report(parts)
    assert source_path.read_bytes() == source_bytes and authored_path.read_bytes() == authored_bytes
    F.write(output_path, parts)
    report = {
        'source': str(source_path), 'source_sha256': F.digest(source_bytes),
        'authored_source': str(authored_path), 'authored_source_sha256': F.digest(authored_bytes),
        'output': str(output_path), 'output_sha256': F.digest(output_path.read_bytes()),
        'slides': TOTAL, 'slide_size_emu': list(F.SIZE), 'slide_size_px': [1920, 1080],
        'new_dp3_pages': list(range(9, 16)), 'appendix_divider_page': 24,
        'preserved_previous_dp3_mapping': {str(p): p + 16 for p in range(9, 14)},
        'original_slide_mapping': {str(r['source_page']): r['page'] for r in preserved},
        'ordered_slides': rows, 'preserved_original_slides': preserved,
        'allowed_original_change': 'Exact footer text only; all original bodies and notes preserved',
        'original_master_layout_theme_media_notes_bytes_preserved': True,
        'all_other_existing_parts_byte_preserved': True,
        'source_files_unchanged': True, 'changed_existing_parts': sorted(changed),
        'added_parts': sorted(set(parts) - set(original)), 'new_media': media,
        'sections': section_report, 'windows_policy_original': original_policy,
        'windows_policy_authored': authored_policy, 'windows_policy': policy,
        'model_calls': 0,
    }
    report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'slides': TOTAL, 'output': str(output_path),
                      'original_bodies_and_notes_preserved': len(preserved),
                      'windows_policy_violations': policy['violations']}, ensure_ascii=False))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('repo', type=Path)
    parser.add_argument('build', type=Path)
    arguments = parser.parse_args()
    main(arguments.repo, arguments.build)
