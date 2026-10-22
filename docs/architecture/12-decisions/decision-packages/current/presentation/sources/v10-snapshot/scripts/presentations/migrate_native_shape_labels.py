"""Scoped migration of reviewed geometry/label pairs into native shape text.

Only operates on active slides and a reviewed explicit mapping. No filename,
shape-name or containment heuristics select new pairs at migration time.

Import usage (repository scripts/presentations must be on sys.path):
    changed_parts, report = migrate_parts(parts, reviewed_map_path)

The input dictionary is not modified. Missing/inactive slide parts and pairs
removed by replacement are recorded as skipped. A present pair whose identity,
text, preset, geometry, adjustments, or bodyPr differs fails closed.

CLI writes only explicit destination paths and refuses to overwrite files.
"""
from __future__ import annotations

from collections import Counter, defaultdict
from copy import deepcopy
from pathlib import Path
import argparse
import importlib
import json
import sys
import xml.etree.ElementTree as ET

P_NS = 'http://schemas.openxmlformats.org/presentationml/2006/main'
A_NS = 'http://schemas.openxmlformats.org/drawingml/2006/main'
P, A = '{' + P_NS + '}', '{' + A_NS + '}'
INS = ('lIns', 'tIns', 'rIns', 'bIns')
DEFAULT_MAP = Path(__file__).with_name('v8-own-label-map.json')


def signature(node):
    if node is None:
        return None
    return (node.tag, tuple(sorted(node.attrib.items())), node.text or '',
            tuple(signature(child) for child in node))


def shape_text(shape):
    return '\n'.join(''.join(t.text or '' for t in p.iter(A + 't'))
                     for p in shape.findall(P + 'txBody/' + A + 'p'))


def bounds(shape):
    transform = shape.find(P + 'spPr/' + A + 'xfrm')
    if transform is None:
        return None
    return [int(transform.find(A + child).get(attr))
            for child, attr in [('off', 'x'), ('off', 'y'), ('ext', 'cx'), ('ext', 'cy')]]


def description(shape):
    identity = shape.find(P + 'nvSpPr/' + P + 'cNvPr')
    geom = shape.find(P + 'spPr/' + A + 'prstGeom')
    body_pr = shape.find(P + 'txBody/' + A + 'bodyPr')
    transform = shape.find(P + 'spPr/' + A + 'xfrm')
    return {
        'shape_id': identity.get('id'),
        'shape_name': identity.get('name'),
        'geometry': geom.get('prst') if geom is not None else 'custom',
        'bounds_emu': bounds(shape),
        'text': shape_text(shape),
        'bodyPr': dict(body_pr.attrib) if body_pr is not None else None,
        'adjustments': {g.get('name'): g.get('fmla') for g in geom.findall(A + 'avLst/' + A + 'gd')}
                       if geom is not None else {},
        'rotation': transform.get('rot') if transform is not None else None,
        'link_refs': [dict(n.attrib) for n in shape.findall('.//' + A + 'hlinkClick')],
    }


def require(condition, message, **context):
    if not condition:
        raise ValueError(json.dumps({'error': message, **context}, ensure_ascii=False))


def assert_identity(actual, expected, *, part, role):
    keys = ('shape_id', 'shape_name', 'geometry', 'bounds_emu', 'text',
            'bodyPr', 'adjustments', 'rotation', 'link_refs')
    for key in keys:
        require(actual[key] == expected[key], 'Mapped shape identity mismatch',
                part=part, role=role, shape_id=expected['shape_id'], field=key,
                expected=expected[key], actual=actual[key])


def assert_no_references(root, ids, part):
    for n in root.iter():
        local = n.tag.split('}')[-1]
        if local in ('stCxn', 'endCxn'):
            require(n.get('id') not in ids, 'Mapped ID has native connector reference',
                    part=part, reference=dict(n.attrib))
        if local == 'spTgt':
            require(n.get('spid') not in ids, 'Mapped ID has animation reference',
                    part=part, reference=dict(n.attrib))


def body_without_insets(body):
    copied = deepcopy(body)
    pr = copied.find(A + 'bodyPr')
    for name in INS:
        pr.attrib.pop(name, None)
    return signature(copied)


def non_text_shape_signature(shape):
    copied = deepcopy(shape)
    body = copied.find(P + 'txBody')
    if body is not None:
        copied.remove(body)
    return signature(copied)


def migrate_parts(parts: dict[str, bytes], mapping_path=DEFAULT_MAP, *,
                  skip_parts: dict[str, str] | None = None):
    """Return new package dict plus detailed preservation report.

    skip_parts is for slides deliberately replaced by the caller. Each value must
    give a nonempty reason. Missing shapes are recorded, not guessed. If only the
    original label is absent and the native node already has the expected own
    text/insets, the report explicitly records an already-migrated pair.
    """
    F = importlib.import_module('full_presentation_package')
    mapping = json.loads(Path(mapping_path).read_text())
    reviewed_pairs = mapping['confirmed_pairs']
    skip_parts = dict(skip_parts or {})
    require(all(reason.strip() for reason in skip_parts.values()), 'Empty skip reason')
    before_order = F.ordered(parts)
    active = {part: index for index, (_, part) in enumerate(before_order, 1)}
    grouped = defaultdict(list)
    node_keys, label_keys = set(), set()
    for pair in reviewed_pairs:
        nk = (pair['slide_part'], pair['node']['shape_id'])
        lk = (pair['slide_part'], pair['label']['shape_id'])
        require(nk not in node_keys and lk not in label_keys, 'Duplicate mapped node or label', node=nk, label=lk)
        node_keys.add(nk); label_keys.add(lk)
        grouped[pair['slide_part']].append(pair)
    output = dict(parts)
    report = {
        'mapping_path': str(Path(mapping_path).resolve()),
        'mapping_source_sha256': mapping.get('source_sha256'),
        'applied_pairs': [], 'skipped_pairs': [], 'slides': [],
        'preservation': {},
    }
    for part, pairs in grouped.items():
        def skip(pair, reason, **extra):
            report['skipped_pairs'].append({
                'source_page': pair['page'], 'current_page': active.get(part),
                'slide_part': part, 'node_id': pair['node']['shape_id'],
                'label_id': pair['label']['shape_id'], 'reason': reason, **extra})
        if part not in parts or part not in active or part in skip_parts:
            reason = skip_parts.get(part) or ('slide_part_missing' if part not in parts else 'slide_part_not_active')
            for pair in pairs:
                skip(pair, reason)
            continue
        root = ET.fromstring(parts[part])
        tree = root.find(P + 'cSld/' + P + 'spTree')
        require(tree is not None, 'Active slide has no shape tree', part=part)
        before = deepcopy(root)
        shapes = [child for child in tree if child.tag == P + 'sp']
        shape_by_id = {s.find(P + 'nvSpPr/' + P + 'cNvPr').get('id'): s for s in shapes}
        require(len(shape_by_id) == len(shapes), 'Duplicate shape IDs', part=part)
        applied = []
        removed_ids = set()
        for pair in pairs:
            node_id, label_id = pair['node']['shape_id'], pair['label']['shape_id']
            node, label = shape_by_id.get(node_id), shape_by_id.get(label_id)
            if node is None or label is None:
                already = False
                if node is not None and label is None:
                    actual = description(node)
                    already = (actual['text'] == pair['label']['text'] and
                               actual['bounds_emu'] == pair['node']['bounds_emu'] and
                               actual['geometry'] == pair['node']['geometry'] and
                               actual['adjustments'] == pair['node']['adjustments'] and
                               all(actual['bodyPr'].get(name) == str(pair['proposed_bodyPr_insets_emu'][name])
                                   for name in INS))
                skip(pair, 'already_migrated' if already else 'mapped_shape_absent_after_rewrite',
                     node_present=node is not None, label_present=label is not None)
                continue
            assert_identity(description(node), pair['node'], part=part, role='node')
            assert_identity(description(label), pair['label'], part=part, role='label')
            require(not shape_text(node), 'Node is not blank', part=part, shape_id=node_id)
            assert_no_references(root, {node_id, label_id}, part)
            label_pr = label.find(P + 'spPr')
            require(label_pr.find(A + 'solidFill') is None and
                    label_pr.find(A + 'ln/' + A + 'solidFill') is None,
                    'Label has visible fill or stroke', part=part, label_id=label_id)
            node_body = node.find(P + 'txBody')
            if node_body is not None:
                node.remove(node_body)
            original_body = label.find(P + 'txBody')
            require(original_body is not None, 'Label missing txBody', part=part, label_id=label_id)
            moved_body = deepcopy(original_body)
            body_pr = moved_body.find(A + 'bodyPr')
            require(body_pr is not None, 'Label missing bodyPr', part=part, label_id=label_id)
            for name in INS:
                value = pair['proposed_bodyPr_insets_emu'][name]
                require(isinstance(value, int), 'Inset is not integer EMU', part=part, name=name, value=value)
                body_pr.set(name, str(value))
            node.append(moved_body)
            require(body_without_insets(original_body) == body_without_insets(moved_body),
                    'Run/paragraph style changed', part=part, node_id=node_id)
            tree.remove(label)
            removed_ids.add(label_id)
            applied.append(pair)
            report['applied_pairs'].append({
                'source_page': pair['page'], 'current_page': active[part], 'slide_part': part,
                'node_id': node_id, 'label_id': label_id, 'node_name': pair['node']['shape_name'],
                'text': pair['label']['text'], 'geometry': pair['node']['geometry'],
                'node_bounds_emu': pair['node']['bounds_emu'], 'old_label_bounds_emu': pair['label']['bounds_emu'],
                'preset_text_rect_inward_offsets_emu': pair['preset_text_rect_inward_offsets_emu'],
                'new_bodyPr_insets_emu': pair['proposed_bodyPr_insets_emu'],
                'paragraph_and_run_styles_preserved': True,
            })
        if not applied:
            continue
        before_runs = Counter(t.text or '' for t in before.iter(A + 't'))
        after_runs = Counter(t.text or '' for t in root.iter(A + 't'))
        require(before_runs == after_runs, 'Slide visible text changed', part=part)
        before_shapes = {s.find(P + 'nvSpPr/' + P + 'cNvPr').get('id'): s
                         for s in before.find(P + 'cSld/' + P + 'spTree') if s.tag == P + 'sp'}
        after_shapes = {s.find(P + 'nvSpPr/' + P + 'cNvPr').get('id'): s
                        for s in tree if s.tag == P + 'sp'}
        require(set(before_shapes) - removed_ids == set(after_shapes), 'Unexpected shape changes', part=part)
        changed_node_ids = {p['node']['shape_id'] for p in applied}
        for shape_id, shape in after_shapes.items():
            original = before_shapes[shape_id]
            require(signature(original.find(P + 'spPr')) == signature(shape.find(P + 'spPr')),
                    'Shape geometry/style changed', part=part, shape_id=shape_id)
            if shape_id in changed_node_ids:
                require(non_text_shape_signature(original) == non_text_shape_signature(shape),
                        'Non-text shape properties changed', part=part, shape_id=shape_id)
            else:
                require(signature(original) == signature(shape), 'Unselected shape changed', part=part, shape_id=shape_id)
        require([signature(t) for t in before.iter(A + 'tbl')] == [signature(t) for t in root.iter(A + 'tbl')],
                'Native table changed', part=part)
        # Every non-shape object stays structurally identical (native tables,
        # connectors, pictures, grouping metadata, timing and notes references).
        before_other = [signature(n) for n in before.find(P + 'cSld/' + P + 'spTree') if n.tag != P + 'sp']
        after_other = [signature(n) for n in tree if n.tag != P + 'sp']
        require(before_other == after_other, 'Non-shape slide object changed', part=part)
        output[part] = F.xml(root)
        report['slides'].append({
            'current_page': active[part], 'part': part, 'migrated_pairs': len(applied),
            'before_shape_count': len(before_shapes), 'after_shape_count': len(after_shapes),
            'visible_text_runs_preserved': True, 'all_remaining_shape_geometry_and_styles_preserved': True,
            'native_tables_preserved': True, 'unselected_shapes_and_non_shape_objects_preserved': True,
        })
    changed = sorted(part for part in parts if parts[part] != output[part])
    require(set(output) == set(parts), 'Package inventory changed')
    require(set(changed) <= {s['part'] for s in report['slides']}, 'Changed a non-selected part', changed=changed)
    require(F.ordered(output) == before_order, 'Presentation ordering changed')
    report['preservation'] = {
        'changed_slide_parts': changed,
        'non_slide_parts_byte_identical': all(parts[p] == output[p] for p in parts if p not in changed),
        'package_inventory_preserved': True, 'presentation_slide_order_preserved': True,
        'original_geometry_and_all_visible_text_preserved': True,
        'native_tables_preserved': True,
        'rendering_status': 'not rendered by this helper; inspect final before/after renders',
    }
    report['summary'] = {'applied_pairs': len(report['applied_pairs']),
                         'skipped_pairs': len(report['skipped_pairs']),
                         'changed_slides': len(changed),
                         'skip_reasons': dict(Counter(p['reason'] for p in report['skipped_pairs']))}
    return output, report


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--repo', required=True, type=Path)
    parser.add_argument('--source', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--report', required=True, type=Path)
    parser.add_argument('--mapping', type=Path, default=DEFAULT_MAP)
    args = parser.parse_args()
    require(not args.output.exists() and not args.report.exists(), 'Refusing to overwrite output/report')
    sys.path.insert(0, str(args.repo / 'scripts/presentations'))
    F = importlib.import_module('full_presentation_package')
    W = importlib.import_module('presentation_windows_policy')
    original = F.read(args.source)
    F.verify(original)
    changed, report = migrate_parts(original, args.mapping)
    F.verify(changed)
    report['source'] = str(args.source.resolve())
    report['source_sha256'] = F.digest(args.source.read_bytes())
    report['typography_policy'] = W.policy_report(changed)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    F.write(args.output, changed)
    report['output'] = str(args.output.resolve())
    report['output_sha256'] = F.digest(args.output.read_bytes())
    report['package_verified'] = True
    args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2))
    print(json.dumps({'output': report['output'], 'summary': report['summary'],
                      'package_verified': True, 'typography_violations': report['typography_policy']['violations']}, ensure_ascii=False))


if __name__ == '__main__':
    main()
