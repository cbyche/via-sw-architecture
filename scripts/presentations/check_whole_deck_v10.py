"""Read-only independent checks for the 36-slide DP44 v9 -> whole-review v10.

Checks package data, not native PowerPoint rendering or VIA measurements.  It
accepts the exact replacement-scene JSON and packaging preservation JSON.  The
latter identifies allowed edits; actual shapes, tables and resources are then
compared independently.  No artifact is modified.  Optional --report is the only
write.  Use --expected-slides 29 --appendix-start 25 only for the v8 test fixture.
"""
from __future__ import annotations
import argparse
from collections import Counter, defaultdict
from copy import deepcopy
import importlib
import json
import re
from pathlib import Path
import sys
import xml.etree.ElementTree as E

EMU = 9525
P = '{http://schemas.openxmlformats.org/presentationml/2006/main}'
A = '{http://schemas.openxmlformats.org/drawingml/2006/main}'
NS = {'p': P[1:-1], 'a': A[1:-1]}
GEOM = {'comp': 'rect', 'mod': 'roundRect', 'store': 'can', 'proc': 'rect',
        'ext': 'hexagon', 'doc': 'foldedCorner', 'band': 'rect'}
INS = ('lIns', 'tIns', 'rIns', 'bIns')


def require(value, message, **context):
    if not value:
        raise ValueError(json.dumps({'check': message, **context}, ensure_ascii=False))


def sig(n):
    if n is None:
        return None
    return (n.tag, tuple(sorted(n.attrib.items())), n.text or '', tuple(sig(c) for c in n))


def text(n):
    return ''.join(t.text or '' for t in n.iter(A + 't'))


def node_text(n):
    return '\n'.join(text(p) for p in n.findall('p:txBody/a:p', NS))


def shape_id(n):
    return n.find('p:nvSpPr/p:cNvPr', NS).get('id')


def shape_name(n):
    return n.find('p:nvSpPr/p:cNvPr', NS).get('name')


def shapes(root):
    items = root.findall('p:cSld/p:spTree/p:sp', NS)
    result = {shape_id(n): n for n in items}
    require(len(result) == len(items), 'duplicate native shape IDs')
    return result


def bounds(n):
    x = n.find('p:spPr/a:xfrm', NS)
    if x is None:
        return None
    return [int(x.find(A + child).get(attr)) for child, attr in
            [('off', 'x'), ('off', 'y'), ('ext', 'cx'), ('ext', 'cy')]]


def near(a, b, tolerance=1e-6):
    return len(a) == len(b) and all(abs(float(x) - float(y)) <= tolerance for x, y in zip(a, b))


def without_shapes(root):
    result = deepcopy(root)
    tree = result.find('p:cSld/p:spTree', NS)
    for n in list(tree):
        if n.tag == P + 'sp':
            tree.remove(n)
    return sig(result)


def notes_check(before, after, part):
    """Original note XML is unchanged after removing appended body paragraphs."""
    old, new = E.fromstring(before), E.fromstring(after)
    old_shapes = {shape_id(s): s for s in old.findall('.//p:sp', NS)}
    new_shapes = {shape_id(s): s for s in new.findall('.//p:sp', NS)}
    require(set(old_shapes) == set(new_shapes), 'note shape inventory changed', part=part)
    appended = []
    for sid, s in old_shapes.items():
        a, b = s.findall('p:txBody/a:p', NS), new_shapes[sid].findall('p:txBody/a:p', NS)
        require(len(b) >= len(a) and [sig(p) for p in b[:len(a)]] == [sig(p) for p in a],
                'original note paragraph changed/reordered', part=part, shape_id=sid)
        if len(b) > len(a):
            placeholder = new_shapes[sid].find('p:nvSpPr/p:nvPr/p:ph', NS)
            require(placeholder is not None and placeholder.get('type') == 'body',
                    'note additions outside body', part=part, shape_id=sid)
            appended += [text(p) for p in b[len(a):]]
            body = new_shapes[sid].find(P+'txBody')
            for p in b[len(a):]:
                body.remove(p)
    require(sig(old) == sig(new), 'original note properties changed', part=part)
    return {'part': part, 'source_paragraphs_and_properties_preserved': True,
            'appended_paragraphs': len(appended), 'added_text': '\n'.join(appended)}


def verify_legend(old_shape, new_shape, record):
    sid = record['id']
    require(text(old_shape) == record['old_text'] and
            node_text(new_shape) == record['new_text'],
            'legend text differs from declared patch', shape_id=sid)
    require(bounds(old_shape) == record['old_bounds_emu'] and
            bounds(new_shape) == record['new_bounds_emu'],
            'legend bounds differ from declared patch', shape_id=sid)
    old, new = deepcopy(old_shape), deepcopy(new_shape)
    old_body, new_body = old.find(P+'txBody'), new.find(P+'txBody')
    require(sig(old_body.find(A+'bodyPr')) == sig(new_body.find(A+'bodyPr')) and
            sig(old_body.find(A+'lstStyle')) == sig(new_body.find(A+'lstStyle')),
            'legend text insets/style hierarchy changed', shape_id=sid)
    template = old_body.find('.//a:rPr', NS)
    require(template is not None, 'legend original run style missing', shape_id=sid)
    expected_run = deepcopy(template)
    expected_run.set('sz', str(record['size_pt']*100))
    for rp in new_body.findall('a:p/a:r/a:rPr', NS):
        require(sig(rp) == sig(expected_run), 'legend non-size run style changed', shape_id=sid)
    require(bool(new_body.findall('a:p/a:r/a:rPr', NS)), 'legend runs missing', shape_id=sid)
    old.remove(old_body); new.remove(new_body)
    tf = new.find('p:spPr/a:xfrm', NS)
    for child, attr, value in zip(('off','off','ext','ext'),('x','y','cx','cy'),record['old_bounds_emu']):
        tf.find(A+child).set(attr,str(value))
    require(sig(old) == sig(new), 'legend non-text properties changed', shape_id=sid)


def verify_sources(scene, directory):
    """Explicit current renderer contract: native nodes and source paths/labels."""
    name = scene['name']
    svg_path, draw_path = directory / (name + '.svg'), directory / (name + '.drawio')
    require(svg_path.is_file() and draw_path.is_file(), 'paired diagram sources missing', scene=name)
    svg_root, draw_root = E.parse(svg_path).getroot(), E.parse(draw_path).getroot()
    svg_ids = {n.get('id'): n for n in svg_root.iter() if n.get('id')}
    cells = {n.get('id'): n for n in draw_root.iter('mxCell')}
    svgns = '{http://www.w3.org/2000/svg}'
    edge_count = 0
    for nid, n in scene['nodes'].items():
        require(nid in svg_ids and nid in cells, 'diagram node missing', scene=name, node=nid)
        actual_text = ''.join(''.join(t.itertext()) for t in svg_ids[nid].iter(svgns + 'tspan'))
        require(actual_text == n['text'].replace('\n', ''), 'SVG own node text mismatch', scene=name, node=nid)
        group = svg_ids[nid]
        x, y, w, h = [n[k] for k in ('x','y','w','h')]
        if n['kind'] == 'ext':
            poly = group.find(svgns+'polygon')
            expected = [[x+20,y],[x+w-20,y],[x+w,y+h/2],[x+w-20,y+h],[x+20,y+h],[x,y+h/2]]
            actual = [[float(v) for v in pair.split(',')] for pair in poly.get('points','').split()] if poly is not None else []
            require(actual == expected, 'SVG external node geometry mismatch', scene=name, node=nid)
        elif n['kind'] == 'store':
            ellipse, path = group.find(svgns+'ellipse'), group.find(svgns+'path')
            radius = min(8, h*.08)
            require(ellipse is not None and near([ellipse.get(k) for k in ('cx','cy','rx','ry')],
                                                [x+w/2,y+radius,w/2,radius]),
                    'SVG storage top geometry mismatch', scene=name, node=nid)
            actual = [float(v) for v in re.findall(r'-?\d+(?:\.\d+)?(?:e[+-]?\d+)?',path.get('d',''))] if path is not None else []
            expected = [x,y+radius,w/2,radius,0,0,1,w,0,h-2*radius,w/2,radius,0,0,1,-w,0]
            require(near(actual,expected), 'SVG storage body geometry mismatch', scene=name, node=nid)
        else:
            rect = group.find(svgns+'rect')
            require(rect is not None and near([rect.get(k) for k in ('x','y','width','height')],[x,y,w,h]),
                    'SVG node rectangle mismatch', scene=name, node=nid)
        label = group.find(svgns+'text')
        require(label is not None and float(label.get('font-size')) == n['size'],
                'SVG node font size mismatch', scene=name, node=nid)
        cell = cells[nid]
        require(cell.get('value') == n['text'], 'drawio own node text mismatch', scene=name, node=nid)
        parent_id = n.get('parent') or '1'
        require(cell.get('parent') == parent_id, 'drawio ownership mismatch', scene=name, node=nid)
        parent = scene['nodes'].get(parent_id, {'x': 0, 'y': 0})
        g = cell.find('mxGeometry')
        require(g is not None and near([g.get(k, '0') for k in ('x', 'y', 'width', 'height')],
                                      [n['x']-parent['x'], n['y']-parent['y'], n['w'], n['h']]),
                'drawio node geometry mismatch', scene=name, node=nid)
        style = cell.get('style', '')
        geom = 'cylinder' if n['kind'] == 'store' else 'hexagon' if n['kind'] == 'ext' else 'rectangle'
        require('shape=' + geom + ';' in style and 'fontFamily=Calibri;' in style,
                'drawio node preset/font mismatch', scene=name, node=nid)
        if n['kind'] == 'mod':
            require('rounded=1;' in style, 'drawio Module is not rounded', scene=name, node=nid)
    for item in scene['items']:
        if item['kind'] != 'edge':
            continue
        edge_count += 1
        eid = item['id']
        require(eid in svg_ids and eid in cells, 'diagram edge missing', scene=name, edge=eid)
        poly = svg_ids[eid]
        points = [[float(v) for v in pair.split(',')] for pair in poly.get('points', '').split()]
        require(points == item['points'], 'SVG edge route mismatch', scene=name, edge=eid)
        cell = cells[eid]
        require(cell.get('source') == item['from'] and cell.get('target') == item['to'],
                'drawio edge endpoints mismatch', scene=name, edge=eid)
        g = cell.find('mxGeometry')
        start, end = g.find("mxPoint[@as='sourcePoint']"), g.find("mxPoint[@as='targetPoint']")
        require(start is not None and end is not None, 'drawio explicit route missing', scene=name, edge=eid)
        points = [[float(start.get('x')), float(start.get('y'))]]
        points += [[float(n.get('x')), float(n.get('y'))] for n in g.findall("Array[@as='points']/mxPoint")]
        points += [[float(end.get('x')), float(end.get('y'))]]
        require(points == item['points'], 'drawio edge route mismatch', scene=name, edge=eid)
    return {'scene': name, 'svg': str(svg_path), 'drawio': str(draw_path),
            'own_node_labels_and_source_geometry': len(scene['nodes']), 'routed_edges': edge_count}


def check(args):
    sys.path.insert(0, str(args.repo / 'scripts/presentations'))
    F = importlib.import_module('full_presentation_package')
    W = importlib.import_module('presentation_windows_policy')
    source, final = F.read(args.source), F.read(args.output)
    F.verify(source); F.verify(final)
    original_order, final_order = F.ordered(source), F.ordered(final)
    require(len(original_order) == len(final_order) == args.expected_slides,
            'slide count mismatch', expected=args.expected_slides,
            source=len(original_order), output=len(final_order))
    require(original_order == final_order, 'slide IDs/parts/order changed')
    require(set(source) == set(final), 'package part inventory changed')
    require(source['ppt/presentation.xml'] == final['ppt/presentation.xml'], 'presentation settings/order changed')
    scenes = json.loads(args.scenes.read_text())
    preservation = json.loads(args.preservation.read_text())
    migration = json.loads(args.migration.read_text()) if args.migration else preservation['labels']
    mapping = json.loads(args.mapping.read_text()) if args.mapping else None
    require(preservation['source_sha256'] == F.digest(args.source.read_bytes()), 'preservation source hash mismatch')
    require(preservation['output_sha256'] == F.digest(args.output.read_bytes()), 'preservation output hash mismatch')
    require(preservation['slide_count'] == args.expected_slides, 'report slide count mismatch')
    scene_by_page = {s['page']: s for s in scenes}
    require(len(scene_by_page) == len(scenes), 'duplicate scene page')
    patch_by_page = {p['page']: p for p in preservation['patches']}
    require(len(patch_by_page) == len(preservation['patches']) and set(patch_by_page) == set(scene_by_page),
            'scene/replacement page list mismatch')
    legend_by_part = defaultdict(dict)
    for entry in preservation.get('legend_changes', []):
        require(entry['id'] not in legend_by_part[entry['part']], 'duplicate legend patch', entry=entry)
        legend_by_part[entry['part']][entry['id']] = entry
    notes_by_part = {entry['part']:entry for entry in preservation.get('notes_appended', [])}
    require(len(notes_by_part) == len(preservation.get('notes_appended', [])), 'duplicate notes append record')
    applied = defaultdict(list)
    skipped = migration['skipped_pairs']
    for m in migration['applied_pairs']:
        applied[m['slide_part']].append(m)
    if mapping:
        mk = {(p['slide_part'], p['node']['shape_id'], p['label']['shape_id']): p for p in mapping['confirmed_pairs']}
        reports = migration['applied_pairs'] + skipped
        reported_keys = [(p['slide_part'], p['node_id'], p['label_id']) for p in reports]
        require(len(reported_keys) == len(set(reported_keys)) and set(reported_keys) == set(mk),
                'migration mapping coverage mismatch')
    slide_rows, scene_rows, note_rows, appendix_rows = [], [], [], []
    active = {part for _, part in original_order}
    allowed_note_parts = set()
    for page, (_, part) in enumerate(original_order, 1):
        old, new = E.fromstring(source[part]), E.fromstring(final[part])
        old_shapes, new_shapes = shapes(old), shapes(new)
        require(without_shapes(old) == without_shapes(new),
                'table/non-shape/slide properties changed', page=page, part=part)
        old_tables, new_tables = old.findall('.//a:tbl', NS), new.findall('.//a:tbl', NS)
        require([sig(t) for t in old_tables] == [sig(t) for t in new_tables], 'native QA table changed', page=page)
        expected_footer = f'{page} / {args.expected_slides}'
        require(sum(text(s) == expected_footer for s in old_shapes.values()) == 1 and
                sum(text(s) == expected_footer for s in new_shapes.values()) == 1,
                'footer missing/duplicated/changed', page=page, expected=expected_footer)
        require([sig(p) for p in old.findall('.//p:pic', NS)] == [sig(p) for p in new.findall('.//p:pic', NS)],
                'new/changed image object', page=page)
        patch = patch_by_page.get(page)
        removed, inserted = {}, {}
        if patch:
            require(patch['part'] == part, 'patch slide part mismatch', page=page)
            removed = {str(s['id']): s for s in patch['removed']}
            inserted = {str(s['id']): s for s in patch['inserted']}
            require(len(removed) == len(patch['removed']) and len(inserted) == len(patch['inserted']),
                    'duplicate replacement IDs', page=page)
            for sid, record in removed.items():
                require(sid in old_shapes and sid not in new_shapes, 'removed shape identity not respected', page=page, shape_id=sid)
                require(text(old_shapes[sid]) == record['text'] and near(bounds(old_shapes[sid]), [x*EMU for x in record['bounds']], 1),
                        'removed shape report disagrees with source', page=page, shape_id=sid)
            for sid, record in inserted.items():
                require(sid not in old_shapes and sid in new_shapes and shape_name(new_shapes[sid]) == record['name'],
                        'inserted shape identity mismatch', page=page, shape_id=sid)
        migrated_nodes, migrated_labels = {}, set()
        for m in applied.get(part, []):
            nid, lid = m['node_id'], m['label_id']
            require(nid in old_shapes and lid in old_shapes and nid in new_shapes and lid not in new_shapes,
                    'migration native node/label inventory mismatch', page=page, node_id=nid, label_id=lid)
            require(nid not in removed and lid not in removed, 'migration claims replaced shape', page=page, node_id=nid)
            require(not text(old_shapes[nid]), 'migrated source node was not blank', page=page, node_id=nid)
            expected = deepcopy(old_shapes[nid])
            body = expected.find(P + 'txBody')
            if body is not None:
                expected.remove(body)
            moved = deepcopy(old_shapes[lid].find(P + 'txBody'))
            require(moved is not None, 'source label body missing', page=page, label_id=lid)
            pr = moved.find(A + 'bodyPr')
            for inset in INS:
                pr.set(inset, str(m['new_bodyPr_insets_emu'][inset]))
            expected.append(moved)
            if nid in legend_by_part[part]:
                verify_legend(expected, new_shapes[nid], legend_by_part[part][nid])
            else:
                require(sig(expected) == sig(new_shapes[nid]), 'migrated geometry/text/styles changed', page=page, node_id=nid)
            require(node_text(expected) == m['text'] and bounds(expected) == m['node_bounds_emu'],
                    'migration report disagrees with output', page=page, node_id=nid)
            if mapping:
                pair = mk[(part, nid, lid)]
                require(m['new_bodyPr_insets_emu'] == pair['proposed_bodyPr_insets_emu'] and
                        bounds(old_shapes[lid]) == pair['label']['bounds_emu'] and
                        node_text(old_shapes[lid]) == pair['label']['text'],
                        'reviewed mapping changed', page=page, node_id=nid)
            migrated_nodes[nid] = m
            migrated_labels.add(lid)
        require(set(new_shapes) == (set(old_shapes) - set(removed) - migrated_labels) | set(inserted),
                'unreported native shape addition/removal', page=page)
        for sid, old_shape in old_shapes.items():
            if sid not in removed and sid not in migrated_nodes and sid not in migrated_labels:
                if sid in legend_by_part[part]:
                    verify_legend(old_shape, new_shapes[sid], legend_by_part[part][sid])
                else:
                    require(sig(old_shape) == sig(new_shapes[sid]), 'unreported source shape edit', page=page, shape_id=sid)
        for sid, record in legend_by_part[part].items():
            require(record['page'] == page and sid in old_shapes and sid in new_shapes and sid not in removed,
                    'legend record page/ID mismatch', page=page, shape_id=sid)
        for m in [s for s in skipped if s['slide_part'] == part]:
            nid, lid = m['node_id'], m['label_id']
            if m['reason'] == 'mapped_shape_absent_after_rewrite':
                require(nid not in new_shapes or lid not in new_shapes, 'skipped label pair still present', page=page, node_id=nid)
                if nid in old_shapes and lid in old_shapes:
                    require(nid in removed or lid in removed, 'missing mapped pair not explained by replacement', page=page, node_id=nid)
            elif m['reason'] == 'already_migrated':
                require(nid in new_shapes and lid not in new_shapes and bool(text(new_shapes[nid])),
                        'already-migrated report false', page=page, node_id=nid)
            else:
                require(False, 'unexpected active-slide migration skip reason', page=page, reason=m['reason'])
        if page in scene_by_page:
            scene = scene_by_page[page]
            by_name = {shape_name(s): s for s in new_shapes.values()}
            checked_nodes = 0
            for nid, model in scene['nodes'].items():
                name = 'v10 node:' + nid
                require(name in by_name, 'scene native node missing', page=page, node=nid)
                n = by_name[name]
                require(shape_id(n) in inserted, 'scene node not in reported inserted IDs', page=page, node=nid)
                require(n.find(P+'txBody') is not None and text(n) == model['text'].replace('\n',''),
                        'scene label is not correct own-shape text', page=page, node=nid)
                geom = n.find('p:spPr/a:prstGeom', NS)
                require(geom is not None and geom.get('prst') == GEOM[model['kind']],
                        'scene native geometry preset mismatch', page=page, node=nid)
                require(near(bounds(n), [model[k]*EMU for k in ('x','y','w','h')], 1),
                        'scene/native node geometry mismatch', page=page, node=nid)
                checked_nodes += 1
            expected_edges = [i for i in scene['items'] if i['kind'] == 'edge']
            for e in expected_edges:
                for prefix in ('edge:', 'arrow:'):
                    require('v10 '+prefix+e['id'] in by_name, 'scene native edge/arrow missing', page=page, edge=e['id'])
            source_result = verify_sources(scene, args.diagram_dir) if args.diagram_dir else {'status':'NOT_CHECKED', 'reason':'--diagram-dir not supplied'}
            scene_rows.append({'page':page, 'scene':scene['name'], 'shape_owned_nodes':checked_nodes,
                               'native_edge_arrow_pairs':len(expected_edges), 'source_sync':source_result})
        try:
            note_old = F.related(source, part, 'notesSlide')
        except StopIteration:
            note_old = None
        try:
            note_new = F.related(final, part, 'notesSlide')
        except StopIteration:
            note_new = None
        require(note_old == note_new, 'notes relationship target changed', page=page)
        if note_old:
            allowed_note_parts.add(note_old)
            note_result = notes_check(source[note_old], final[note_old], note_old)
            addition = notes_by_part.get(note_old)
            if addition:
                require(addition['page'] == page and addition['added_text'] == note_result['added_text'],
                        'notes append differs from declared text', page=page)
                require(page in scene_by_page and addition['added_text'] ==
                        'v10 가이드 검토에 따른 도식 설명 (기존 노트와 QA 근거 보존)\n'+scene_by_page[page]['notes'].strip(),
                        'appended note not synchronized to scene description', page=page)
            else:
                require(note_result['appended_paragraphs'] == 0, 'unreported notes append', page=page)
            note_rows.append({'page':page, **note_result})
        if page >= args.appendix_start:
            require(patch is None, 'appendix unexpectedly redrawn', page=page)
            require(Counter(t.text or '' for t in old.iter(A+'t')) == Counter(t.text or '' for t in new.iter(A+'t')),
                    'appendix visible text runs changed', page=page)
            appendix_rows.append({'page':page, 'part':part, 'text_runs_and_retained_shape_geometry_styles_preserved':True,
                                  'migrated_label_pairs':len(migrated_nodes), 'pixel_identity':'NOT_ASSERTED'})
        slide_rows.append({'page':page,'part':part,'native_tables':len(new_tables),
                           'native_table_cells':len(new.findall('.//a:tc',NS)),
                           'native_tables_exactly_preserved':True,'footer_preserved':True,
                           'pictures':len(new.findall('.//p:pic',NS)),
                           'source_shapes_preserved':len(old_shapes)-len(removed)-len(migrated_nodes)-len(migrated_labels),
                           'replaced_shapes':len(removed),'inserted_shapes':len(inserted),'migrated_label_pairs':len(migrated_nodes)})
    require(set(legend_by_part) <= active and set(notes_by_part) <= allowed_note_parts, 'report refers to inactive slide/note')
    allowed_changed = active | set(notes_by_part)
    changed = sorted(p for p in source if source[p] != final[p])
    require(set(changed) <= allowed_changed, 'non-slide/non-note resource changed', parts=sorted(set(changed)-allowed_changed))
    if 'changed_parts' in preservation:
        # Packaging may later append notes; slide changes must still match exactly.
        require({p for p in changed if p in active} == {p for p in preservation['changed_parts'] if p in active},
                'reported changed slide parts mismatch')
    for m in skipped:
        if m['slide_part'] not in active:
            require(m['reason'] in ('slide_part_missing','slide_part_not_active'), 'unexplained inactive mapping skip', entry=m)
    policy = W.policy_report(final)
    return {'status':'PASS','source_sha256':F.digest(args.source.read_bytes()),'output_sha256':F.digest(args.output.read_bytes()),
            'expected_slide_count':args.expected_slides,'slide_order_ids_and_parts_preserved':True,
            'package_relationships_masters_media_and_inventory_preserved':True,
            'legend_patches_verified':sum(len(v) for v in legend_by_part.values()),
            'notes_appended_verified':len(notes_by_part),'native_table_count':sum(r['native_tables'] for r in slide_rows),
            'native_table_cells':sum(r['native_table_cells'] for r in slide_rows),
            'new_image_objects':0,'slides':slide_rows,'scenes':scene_rows,'notes':note_rows,'appendix':appendix_rows,
            'migration_applied_pairs_verified':sum(len(v) for v in applied.values()),'migration_skipped_pairs_verified':len(skipped),
            'typography':policy,'changed_parts':changed,
            'limits':['XML and source-model checks do not establish native PowerPoint visual equivalence or text fit.',
                      'Appendix 31–36 remains historical. Label migration preserves source text runs, geometry and styles, but moves text to an earlier shape in z-order. Preset text-rectangle compensation is checked against the reviewed mapping; identical pixels require separate before/after rendering.',
                      'Own-shape checks cover explicitly modeled replacement nodes and reviewed migration pairs. Ordinary flow labels, old unmodeled nodes and all diagram aesthetics are not inferred by heuristic.',
                      'SVG/drawio checks cover node labels/geometry, drawio ownership and explicit edge routes; pixel rendering and SVG text layout are not compared.',
                      'QA table and evidence text preservation is not a new measurement, architecture acceptance or performance validation.']}


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for flag in ('repo','source','output','scenes','preservation'):
        p.add_argument('--'+flag,required=True,type=Path)
    p.add_argument('--migration',type=Path,help='Separate migration JSON; otherwise preservation.labels')
    p.add_argument('--mapping',type=Path,help='Reviewed confirmed_pairs JSON for coverage and inset verification')
    p.add_argument('--diagram-dir',type=Path,help='Paired scene-name.svg/.drawio sources')
    p.add_argument('--expected-slides',type=int,default=36)
    p.add_argument('--appendix-start',type=int,default=31)
    p.add_argument('--report',type=Path)
    args=p.parse_args()
    try:
        result=check(args)
    except Exception as exc:
        result={'status':'FAIL','error':str(exc),'error_type':type(exc).__name__}
        if args.report:args.report.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
        print(json.dumps(result,ensure_ascii=False));return 1
    if args.report:args.report.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({k:result[k] for k in ('status','expected_slide_count','native_table_count','native_table_cells',
                                          'new_image_objects','migration_applied_pairs_verified','migration_skipped_pairs_verified')},ensure_ascii=False))
    return 0

if __name__=='__main__':
    raise SystemExit(main())
