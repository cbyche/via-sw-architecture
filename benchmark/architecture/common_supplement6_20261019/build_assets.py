"""Author contract20 screens/cues from geometry and actual authored selection actions.

No evaluator/oracle file is read. PNG glyphs use the pinned OS font file directly.
Run with the bundled Python runtime (Pillow); no model or network calls.
"""
import hashlib
import html
import json
import math
import subprocess
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont, __version__ as pillow_version

HERE = Path(__file__).resolve().parent
ASSETS = HERE / 'assets'
FONT = Path('/System/Library/Fonts/AppleSDGothicNeo.ttc')
NODE = Path('/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node')
SHARP = Path('/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp')
NAMES = ['요구사항', '일정', '설계', '검토', '구현', '시험', '배포', '운영', '지원', '보안', '회고', '보관']
ROWS = [{'item_id': f'B{i:02}', 'name': name, 'description': name + ' 문서 열기', 'line': i} for i, name in enumerate(NAMES, 1)]
# Authored product observations, not read from the evaluator expected-set file.
SELECTION_ACTIONS = {'S85-U01': ['B03', 'B04'], 'S85-U02': ['B03', 'B05']}
POINTER_ACTIONS = {'U01': ['B03', 'B04'], 'U02': ['B03', 'B05']}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def dump(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def render(dense):
    name = 'sup-dense-v1' if dense else 'sup-sparse-v1'
    scale = 1 if dense else 2
    items = []
    shapes = []
    texts = []
    for row in ROWS:
        y = (360 + 24 * (row['line'] - 1)) if dense else (200 + 96 * (row['line'] - 1))
        top = y + (4 if dense else 16)
        baseline = y + (20 if dense else 40)
        bbox = [1400, top, 1400 + 16 * scale, top + 16 * scale]
        items.append({**row, 'bbox': bbox, 'baseline': baseline})
        shapes.append(f'<g transform="translate(1400 {top}) scale({scale})"><rect x=".5" y=".5" width="15" height="15" fill="#e8f0fe" stroke="#2b6cb0" stroke-width="1"/><path d="M5 11 L11 5 M6 5 H11 V10" fill="none" stroke="#2b6cb0" stroke-width="1.5"/></g>')
        texts.extend([(1100, baseline, row['name']), (1310, baseline, row['item_id'])])
    header = '<svg xmlns="http://www.w3.org/2000/svg" width="2880" height="1800" viewBox="0 0 2880 1800"><rect width="2880" height="1800" fill="white"/>'
    background = ASSETS / (name + '-shapes.svg')
    background.write_text(header + ''.join(shapes) + '</svg>')
    vector = ASSETS / (name + '.svg')
    vector.write_text(header + ''.join(shapes) + '<g fill="black" font-family="Apple SD Gothic Neo" font-weight="400" font-size="20">' + ''.join(f'<text x="{x}" y="{y}">{html.escape(text)}</text>' for x, y, text in texts) + '</g></svg>')
    png = ASSETS / (name + '.png')
    subprocess.run([str(NODE), '-e', 'require(process.argv[1])(process.argv[2]).png().toFile(process.argv[3])', str(SHARP), str(background), str(png)], check=True)
    image = Image.open(png).convert('RGB')
    font = ImageFont.truetype(str(FONT), 20, index=0)
    assert font.getname() == ('Apple SD Gothic Neo', 'Regular')
    draw = ImageDraw.Draw(image)
    for x, y, text in texts:
        draw.text((x, y), text, font=font, anchor='ls', fill='black')
    image.save(png)
    return {'frame_id': name, 'image_path': str(png), 'sha256': sha(png), 'source_id': 'SUP-SCREEN', 'source_revision': 1, 'width': 2880, 'height': 1800, 'coordinate_space': 'frame_pixels', 'evidence_kind': 'AUTHORED_RENDERED_SCREEN_NOT_CAPTURED_WINDOWS_APP'}, items


def main():
    ASSETS.mkdir(parents=True, exist_ok=True)
    dense, dense_items = render(True)
    sparse, sparse_items = render(False)
    observations = {}
    for sid in ['S83', 'S84', 'S85']:
        base, layout = (sparse, sparse_items) if sid == 'S84' else (dense, dense_items)
        by_id = {item['item_id']: item for item in layout}
        for suffix in ['U01', 'U02']:
            eid = sid + '-' + suffix
            frame = dict(base)
            cues = []
            if sid == 'S85':
                selected = SELECTION_ACTIONS[eid]
                frame['native_selection'] = {'status': 'available', 'capability': 'selected_object_ranges_only', 'provider': 'AUTHORED_SELECTION_ADAPTER_OF_RENDERED_SOURCE', 'complete_scene_objects': False, 'items': [{'item_id': item, 'source_id': 'SUP-SCREEN', 'source_revision': 1, 'bbox': by_id[item]['bbox'], 'coordinate_space': 'frame_pixels', 'source_line_range': [by_id[item]['line']] * 2} for item in selected]}
                cues.append({'event_id': eid + '-native', 'type': 'native_selection', 'frame_id': frame['frame_id'], 't_ms': 800, 'observed_in': 'frame.native_selection', 'item_ids': selected})
            else:
                frame['native_selection'] = {'status': 'unavailable', 'capability': 'selected_object_ranges_only', 'complete_scene_objects': False, 'items': []}
                radius = 22 if sid == 'S84' else 11
                for i, item in enumerate(POINTER_ACTIONS[suffix]):
                    x1, y1, x2, y2 = by_id[item]['bbox']
                    cx, cy = (x1 + x2) / 2, (y1 + y2) / 2
                    start = 800 if i == 0 else 3200
                    points = [{'x': round(cx + radius * math.cos(2 * math.pi * k / 16), 3), 'y': round(cy + radius * math.sin(2 * math.pi * k / 16), 3), 't_ms': start + 50 * k} for k in range(17)]
                    cues.append({'event_id': eid + f'-path{i + 1:02}', 'type': 'pointer_path', 'frame_id': frame['frame_id'], 'button': 'none', 't_ms': start, 'points': points})
            observations[eid] = {'frames': [frame], 'cues': cues}
    dump(ASSETS / 'observations.json', {'scope': 'Authored public observations only; no evaluator/oracle read', 'events': observations})
    dump(ASSETS / 'selection-actions.json', {'evidence_kind': 'AUTHORED_PRODUCT_SELECTION_ACTIONS_NOT_ORACLE', 'actions': [{'event_id': eid, 'at_ms': 800, 'selected_item_ids': ids} for eid, ids in SELECTION_ACTIONS.items()]})
    dump(ASSETS / 'authoring-layout.json', {'scope': 'Renderer source geometry; not injected as full scene map into candidate inputs', 'dense': dense_items, 'sparse': sparse_items})
    text = '\n'.join(f"{r['item_id']} | {r['name']} | {r['description']}" for r in ROWS) + '\n'
    (ASSETS / 'SUP-SCREEN.txt').write_text(text)
    dump(ASSETS / 'render-manifest.json', {'font_path': str(FONT), 'font_sha256': sha(FONT), 'font_collection_index': 0, 'font_name': ['Apple SD Gothic Neo', 'Regular'], 'font_size_px': 20, 'pillow_version': pillow_version, 'node_path': str(NODE), 'sharp_path': str(SHARP), 'png_text_rendering': 'Pillow direct pinned-font glyphs at exact pixel baselines; vector icon rasterized with sharp', 'editable_source': 'Full SVG retains same geometry/text/font family; reproducing PNG requires pinned font and renderer script', 'all_assets': [{'path': str(p), 'sha256': sha(p), 'bytes': p.stat().st_size} for p in sorted(ASSETS.iterdir()) if p.is_file() and p.name != 'render-manifest.json'], 'actual_API_calls': 0, 'oracle_read': False})
    print(json.dumps({'assets': str(ASSETS), 'frames': 2, 'public_events': 6, 'oracle_read': False}))


if __name__ == '__main__':
    main()
