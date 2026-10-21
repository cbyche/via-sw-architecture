"""Replace DP3 pages 10–12 in the 29-slide v6 deck without rewriting other slides.

All source parts remain present. The 26 unselected slides, notes, relationships,
masters, layouts, media and other dependencies are byte-identical to the input.
The three new native slides are imported with a unique dependency prefix; their
existing presentation slide IDs and section membership are retained.

Usage: python3 package_dp3_cpu_v7.py REPOSITORY BUILD_DIRECTORY
Inputs: BUILD/source-v6.pptx and BUILD/dp3-cpu-v7-authored.pptx
Outputs: BUILD/candidate.pptx and BUILD/preservation.json
"""
from pathlib import Path
import argparse
import copy
import json
import re
import xml.etree.ElementTree as E

import full_presentation_package as F
import package_dp3_cpu_v6 as V6
import presentation_windows_policy as W


TOTAL = 29
REPLACED_PAGES = (10, 11, 12)
NEW_PARTS = tuple(f'ppt/slides/slide{n}.xml' for n in (410, 411, 412))
PREFIX = 'dp3cpuv7'
ALLOWED_EXISTING_CHANGES = {
    'ppt/presentation.xml',
    'ppt/_rels/presentation.xml.rels',
    '[Content_Types].xml',
}


def element_signature(node):
    """Compare semantic XML independent of namespace prefix serialization."""
    if node is None:
        return None
    return (node.tag, tuple(sorted(node.attrib.items())), node.text or '',
            tuple(element_signature(child) for child in node))


def sections_signature(presentation):
    return [element_signature(n) for n in presentation.iter()
            if n.tag.split('}')[-1] == 'sectionLst']


def footer(data, local_page, final_page):
    # The v6 helper recognizes final page markers and single local numbers.
    # Accept the conventional three-slide local footer before applying it.
    root = E.fromstring(data)
    local_text = f'{local_page} / {len(REPLACED_PAGES)}'
    matches = [s for s in root.findall('.//p:sp', F.NS)
               if ''.join(t.text or '' for t in s.findall('.//a:t', F.NS)) == local_text]
    if matches:
        data = V6.replace_footer_bytes(data, local_text, f'{final_page} / {TOTAL}')
    return V6.renumber_authored_footer(data, local_page, final_page)


def hash_parts(parts, names):
    return [{'part': name, 'sha256': F.digest(parts[name]), 'bytes': len(parts[name])}
            for name in sorted(names)]


def own_shape_labels(data):
    """Move an adjacent native label into its enclosing shape, preserving its text rectangle.

    Artifact Tool exports the author's geometry and text separately. The repository
    requires Component/Module labels to be editable as the shape's own text. Insets
    retain the original label rectangle, including Component header compartments.
    Flow annotations are not paired with a surrounding Component and stay separate.
    """
    root = E.fromstring(data)
    tree = root.find('p:cSld/p:spTree', F.NS)
    paired = []

    def bounds(shape):
        transform = shape.find('p:spPr/a:xfrm', F.NS)
        if transform is None:
            return None
        off, ext = transform.find('a:off', F.NS), transform.find('a:ext', F.NS)
        if off is None or ext is None:
            return None
        return tuple(int(v) for v in (off.get('x'), off.get('y'), ext.get('cx'), ext.get('cy')))

    previous = None
    for child in list(tree):
        if child.tag != '{' + F.P + '}sp':
            previous = None
            continue
        text = ''.join(n.text or '' for n in child.findall('.//a:t', F.NS))
        body = child.find('p:txBody', F.NS)
        geom = child.find('p:spPr/a:prstGeom', F.NS)
        if text and body is not None and previous is not None:
            outer, inner = bounds(previous), bounds(child)
            if outer and inner:
                x, y, w, h = outer
                tx, ty, tw, th = inner
                if x <= tx and y <= ty and tx + tw <= x + w and ty + th <= y + h:
                    moved = copy.deepcopy(body)
                    props = moved.find('a:bodyPr', F.NS)
                    # Include any existing inset of the text shape itself.
                    for name, extra in [('lIns', tx - x), ('tIns', ty - y),
                                        ('rIns', x + w - tx - tw), ('bIns', y + h - ty - th)]:
                        props.set(name, str(extra + int(props.get(name, '0'))))
                    old = previous.find('p:txBody', F.NS)
                    if old is not None:
                        previous.remove(old)
                    previous.append(moved)
                    previous.find('p:nvSpPr/p:cNvPr', F.NS).set('name', text.replace('\n', ' / '))
                    tree.remove(child)
                    paired.append(text)
                    previous = None
                    continue
        # An empty preset geometry immediately followed by its own label is eligible.
        previous = child if not text and geom is not None else None
    return F.xml(root), paired


def normalize_master_registry(parts):
    """Repair inherited/imported presentation registries without changing any master.

    Open XML names the entry sldMasterId, and PresentationPart may register only
    one notesMaster. Individual notes-slide relationships and master parts remain
    byte-identical; only redundant presentation-level registrations are removed.
    """
    root = E.fromstring(parts['ppt/presentation.xml'])
    rels = E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
    repaired = 0
    for node in root.find('p:sldMasterIdLst', F.NS):
        if node.tag == '{' + F.P + '}slideMasterId':
            node.tag = '{' + F.P + '}sldMasterId'
            repaired += 1
    notes = root.find('p:notesMasterIdLst', F.NS)
    assert notes is not None and len(notes) >= 1
    keep = notes[0].get('{' + F.R + '}id')
    removed = []
    for node in list(notes)[1:]:
        notes.remove(node)
    for rel in list(rels):
        if rel.get('Type') == F.R + '/notesMaster' and rel.get('Id') != keep:
            removed.append(rel.get('Id'))
            rels.remove(rel)
    assert len([r for r in rels if r.get('Type') == F.R + '/notesMaster']) == 1
    parts['ppt/presentation.xml'] = F.xml(root)
    parts['ppt/_rels/presentation.xml.rels'] = F.xml(rels)
    return {'corrected_sldMasterId_tags': repaired,
            'removed_duplicate_notes_master_relationship_ids': removed,
            'preserved_notes_master_relationship_id': keep,
            'master_parts_and_notes_slide_relationships_unchanged': True}


def main(repo, build):
    repo, build = repo.resolve(), build.resolve()
    assert (repo / 'INTENT.md').is_file(), ('Not the VIA repository', str(repo))
    source_path = build / 'source-v6.pptx'
    authored_path = build / 'dp3-cpu-v7-authored.pptx'
    output_path, report_path = build / 'candidate.pptx', build / 'preservation.json'
    assert not output_path.exists(), 'Refusing to overwrite candidate.pptx'
    assert not report_path.exists(), 'Refusing to overwrite preservation.json'
    source_bytes, authored_bytes = source_path.read_bytes(), authored_path.read_bytes()
    source, authored_raw = F.read(source_path), F.read(authored_path)
    source_order, authored_order = F.ordered(source), F.ordered(authored_raw)
    assert len(source_order) == TOTAL, ('Unexpected source count', len(source_order))
    assert len(authored_order) == len(REPLACED_PAGES), ('Unexpected authored count', len(authored_order))
    F.verify(source)
    F.verify(authored_raw)
    for name, collection in [('source', source), ('authored', authored_raw)]:
        size = E.fromstring(collection['ppt/presentation.xml']).find('p:sldSz', F.NS)
        assert (int(size.get('cx')), int(size.get('cy'))) == F.SIZE, (name, size.attrib)

    source_policy = W.policy_report(source)
    # Existing source content is never passed through the normalization pass.
    authored = {name: W.normalize_xml(name, data) if name.endswith('.xml') else data
                for name, data in authored_raw.items()}
    authored_policy = W.policy_report(authored)
    label_report = []
    for page, (_, part) in zip(REPLACED_PAGES, authored_order):
        authored[part], labels = own_shape_labels(authored[part])
        label_report.append({'page': page, 'shape_owned_labels': len(labels), 'labels': labels})
        assert len(labels) >= 20, ('Expected native shape-owned labels', page, len(labels))
    parts = dict(source)
    for part in NEW_PARTS:
        number = re.search(r'slide(\d+)\.xml$', part)[1]
        note = f'ppt/notesSlides/notesSlide{number}.xml'
        for reserved in (part, F.rels_path(part), note, F.rels_path(note)):
            assert reserved not in parts, ('Reserved part collision', reserved)
    assert not any('/' + PREFIX + '_' in n for n in parts), 'Import prefix already present'
    importer = F.Importer(parts)
    imported, scale, media = importer.import_slides(
        authored, [part for _, part in authored_order], PREFIX, list(NEW_PARTS))
    assert scale == 1
    assert [imported[p] for _, p in authored_order] == list(NEW_PARTS)
    F.add_registry(parts, importer)
    registry_report = normalize_master_registry(parts)

    source_presentation = E.fromstring(source['ppt/presentation.xml'])
    presentation = E.fromstring(parts['ppt/presentation.xml'])
    old_nodes = list(source_presentation.find('p:sldIdLst', F.NS))
    new_nodes = list(presentation.find('p:sldIdLst', F.NS))
    assert element_signature(source_presentation.find('p:sldIdLst', F.NS)) == element_signature(
        presentation.find('p:sldIdLst', F.NS)), 'Slide IDs/order changed during import'
    assert sections_signature(source_presentation) == sections_signature(presentation), 'Sections changed'
    relationships = E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
    relationship_by_id = {r.get('Id'): r for r in relationships}
    assert len(relationship_by_id) == len(relationships), 'Duplicate relationship IDs'
    replacements = {}
    for local_page, (page, part) in enumerate(zip(REPLACED_PAGES, NEW_PARTS), 1):
        node = new_nodes[page - 1]
        rid = node.get('{' + F.R + '}id')
        rel = relationship_by_id[rid]
        assert rel.get('Type') == F.R + '/slide'
        assert F.target('ppt/presentation.xml', rel.get('Target')) == source_order[page - 1][1]
        rel.set('Target', '/' + part)
        parts[part] = footer(parts[part], local_page, page)
        replacements[page] = part
    parts['ppt/_rels/presentation.xml.rels'] = F.xml(relationships)

    # Only selected slide relationship targets may change among old entries.
    replaced_rids = {old_nodes[p - 1].get('{' + F.R + '}id') for p in REPLACED_PAGES}
    for old_rel in F.relations(source, 'ppt/presentation.xml'):
        if old_rel.get('Id') in registry_report['removed_duplicate_notes_master_relationship_ids']:
            assert old_rel.get('Type') == F.R + '/notesMaster'
            continue
        new_rel = relationship_by_id[old_rel.get('Id')]
        old_attrs, new_attrs = dict(old_rel.attrib), dict(new_rel.attrib)
        if old_rel.get('Id') in replaced_rids:
            old_attrs.pop('Target')
            new_attrs.pop('Target')
        assert old_attrs == new_attrs, ('Unexpected old relationship change', old_rel.attrib)

    expected_order = [(sid, replacements.get(page, part))
                      for page, (sid, part) in enumerate(source_order, 1)]
    assert F.ordered(parts) == expected_order, 'Slide ordering/IDs do not match the replacement plan'
    changed = {name for name, data in source.items() if parts[name] != data}
    assert changed <= ALLOWED_EXISTING_CHANGES, ('Unexpected existing-part changes', sorted(changed))
    assert set(source) <= set(parts), 'A source part was removed'
    preserved, rows = [], []
    for page, (sid, part) in enumerate(expected_order, 1):
        source_part = source_order[page - 1][1]
        note = F.related(parts, part, 'notesSlide')
        row = {'page': page, 'slide_id': int(sid), 'part': part,
               'source_part': source_part, 'notes_part': note,
               'slide_sha256': F.digest(parts[part]), 'notes_sha256': F.digest(parts[note]),
               'kind': 'authored' if page in replacements else 'preserved'}
        if page not in replacements:
            source_note = F.related(source, source_part, 'notesSlide')
            for protected in (part, F.rels_path(part), source_note, F.rels_path(source_note)):
                assert parts[protected] == source[protected], ('Slide/notes dependency changed', protected)
            row.update(slide_xml_byte_preserved=True, notes_xml_byte_preserved=True,
                       slide_relationships_byte_preserved=True,
                       notes_relationships_byte_preserved=True)
            preserved.append(row.copy())
        texts = [''.join(t.text or '' for t in s.findall('.//a:t', F.NS))
                 for s in E.fromstring(parts[part]).findall('.//p:sp', F.NS)]
        assert texts.count(f'{page} / {TOTAL}') == 1, ('Footer mismatch', page)
        rows.append(row)
    assert len(preserved) == TOTAL - len(REPLACED_PAGES)
    F.verify(parts)
    policy = W.policy_report(parts)
    assert source_path.read_bytes() == source_bytes, 'Source file changed during packaging'
    assert authored_path.read_bytes() == authored_bytes, 'Authored file changed during packaging'
    F.write(output_path, parts)
    assert F.ordered(F.read(output_path)) == expected_order
    added = set(parts) - set(source)
    report = {
        'version': 'dp3-cpu-diagram-v7',
        'source': str(source_path), 'source_sha256': F.digest(source_bytes),
        'authored_source': str(authored_path), 'authored_source_sha256': F.digest(authored_bytes),
        'output': str(output_path), 'output_sha256': F.digest(output_path.read_bytes()),
        'slides': TOTAL, 'slide_size_emu': list(F.SIZE), 'slide_size_px': [1920, 1080],
        'replaced_pages': list(REPLACED_PAGES), 'ordered_slides': rows,
        'preserved_slides': preserved,
        'slide_ids_order_and_section_membership_preserved': True,
        'all_source_parts_retained': True,
        'all_unselected_slide_notes_relationship_bytes_preserved': True,
        'all_existing_masters_layouts_theme_media_notes_bytes_preserved': True,
        'all_other_existing_parts_byte_preserved': True,
        'source_files_unchanged': True,
        'changed_existing_parts': sorted(changed),
        'changed_metadata_hashes': [dict(part=n, before_sha256=F.digest(source[n]),
                                         after_sha256=F.digest(parts[n])) for n in sorted(changed)],
        'unchanged_source_part_hashes': hash_parts(source, set(source) - changed),
        'added_parts': sorted(added), 'added_part_hashes': hash_parts(parts, added),
        'new_media': media,
        'native_shape_label_ownership': label_report,
        'openxml_master_registry_repair': registry_report,
        'windows_policy_source': source_policy, 'windows_policy_authored': authored_policy,
        'windows_policy': policy, 'model_calls': 0,
    }
    report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'slides': TOTAL, 'output': str(output_path),
                      'replaced_pages': list(REPLACED_PAGES), 'preserved_slides': len(preserved),
                      'windows_policy_violations': policy['violations']}, ensure_ascii=False))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('repo', type=Path)
    parser.add_argument('build', type=Path)
    arguments = parser.parse_args()
    main(arguments.repo, arguments.build)
