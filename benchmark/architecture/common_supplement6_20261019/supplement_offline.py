"""API-free integration doubles from actual supplied inputs/renderer geometry.

Not model accuracy/performance evidence. No oracle file or future event is read.
"""
import copy
import importlib.util
import json
import math
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('common9_reference_double', HERE.parent / 'common9_poc_20261019/offline.py')
legacy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(legacy)


def candidates(frames):
    layout = json.loads((HERE / 'assets/authoring-layout.json').read_text())
    result = []
    for frame in frames:
        rows = layout['sparse' if frame['frame_id'] == 'sup-sparse-v1' else 'dense']
        for row in rows:
            x1, y1, x2, y2 = row['bbox']
            result.append({'item_id': row['item_id'], 'frame_id': frame['frame_id'], 'label': row['item_id'] + ' ' + row['name'], 'coordinate_space': 'normalized_0_1000', 'bbox_2d': [math.floor(x1 / frame['width'] * 1000), math.floor(y1 / frame['height'] * 1000), math.ceil(x2 / frame['width'] * 1000), math.ceil(y2 / frame['height'] * 1000)]})
    return result


def references(transcript, cues):
    return [{'expression': '지금 표시한 두 버튼' if '지금 표시한 두 버튼' in transcript else '표시한 두 버튼', 'relation': 'native_selection' if any(c['type'] == 'native_selection' for c in cues) else 'other', 'cue_ids': [c['event_id'] for c in cues], 'count': 2, 'meaning': transcript}]


def bind(refs, items, cues, frames=None):
    selected = []
    for cue in cues:
        if cue['type'] == 'native_selection':
            selected.extend(item for item in items if item['item_id'] in cue['item_ids'] and item['frame_id'] == cue['frame_id'])
        elif cue['type'] == 'pointer_path':
            points = cue['points']
            x, y = sum(p['x'] for p in points) / len(points), sum(p['y'] for p in points) / len(points)
            for item in items:
                if item['frame_id'] != cue['frame_id']:
                    continue
                if 'bbox' in item:
                    box = item['bbox']
                else:
                    frame = next(f for f in frames if f['frame_id'] == cue['frame_id'])
                    b = item['bbox_2d']
                    box = [b[0] * frame['width'] / 1000, b[1] * frame['height'] / 1000, b[2] * frame['width'] / 1000, b[3] * frame['height'] / 1000]
                if box[0] <= x <= box[2] and box[1] <= y <= box[3]:
                    selected.append(item)
    selected = list({(i['frame_id'], i['item_id']): i for i in selected}.values())
    ref = refs[0]
    return {'bindings': [{'expression': ref['expression'], 'targets': [i.get('canonical_target', i['item_id']) for i in selected], 'extents': selected, 'status': 'ready', 'relation': ref['relation'], 'cue_ids': ref['cue_ids'], 'count': ref['count']}], 'intent': ref['meaning'], 'unknowns': [], 'need_expansion': False}


def respond(payload, meta):
    stage = meta['stage']
    data = payload.get('input', {})
    if stage.startswith('grounding.'):
        if stage == 'grounding.local_screen_parser':
            return {'frames': [], 'scope': 'OFFLINE_GEOMETRY_DOUBLE_NO_LEARNED_INFERENCE'}
        if 'speech' in stage:
            return {'references': references(data['transcript'], data['cues']), 'intent': data['transcript'], 'unknowns': []}
        if 'screen' in stage:
            return {'candidates': candidates(data['frames']), 'coverage': {'complete': True, 'unknowns': []}}
        if 'joint' in stage:
            return bind(references(data['transcript'], data['cues']), candidates(data['frames']), data['cues'], data['frames'])
        refs = copy.deepcopy(data['speech']['references'])
        for ref in refs:
            ref['meaning'] = data['speech'].get('intent', '')
        return bind(refs, copy.deepcopy(data['screen']['candidates']), data['cues'])
    if stage == 'evidence_select_ranges':
        text = data['event']['text']
        wanted = 'SUP-MEETING' if '회의 안내문' in text else 'SUP-SCREEN' if '두 버튼' in text else 'SUP-RELEASE'
        return {'reads': [{'source_id': source['id'], 'revision': source['revision'], 'start_line': r[0], 'end_line': r[1]} for source in data['catalog'] if source['id'] == wanted and source.get('is_current') is not False for r in source.get('allowed_segments', [])], 'bindings': [], 'unresolved': []}
    if stage != 'meaning_interpret_and_respond':
        return legacy.respond(payload, meta)
    event = data['event']
    text = event['text']
    originals = data.get('actually_supplied_originals', [])
    wanted = 'SUP-MEETING' if '회의 안내문' in text else 'SUP-SCREEN' if '두 버튼' in text else 'SUP-RELEASE'
    used = [p for p in originals if p['source_id'] == wanted]
    plan = {'relation': 'NEW', 'target_goal_id': None, 'target_event_id': None, 'task_id': None, 'route': 'DIRECT', 'purpose': text, 'handling_text': '실제 제공된 원본과 지정 범위에 따라 직접 설명하겠습니다.', 'answer': None, 'targets': [wanted] if used else [], 'bindings': [], 'source_revisions': {p['source_id']: p['revision'] for p in used}, 'source_claims': [{**{k: p[k] for k in ['source_id', 'revision', 'start_line', 'end_line']}, 'quote': p['text'].splitlines()[0]} for p in used], 'constraints': ['실제 수정이나 공유, 발송을 실행하지 않음'], 'exclusions': ['외부 공개나 새 권한 부여를 실행하지 않음'], 'allowed_actions': ['explain_only'], 'completed_actions': [], 'unknown_actions': [], 'permission_operations': [], 'control_operations': [], 'recipient': None, 'artifact_type': None, 'completion_condition': '현재 지정된 설명을 실제 표시한다.'}
    if '정정' in text:
        prior = next((e for e in reversed(data['public_state'].get('captured_inputs', [])) if e.get('kind') == 'user' and e.get('id') != event['id']), None)
        if prior:
            request = next((r for r in data['public_state'].get('requests', []) if r.get('origin_event_id') == prior['id']), None)
            plan.update(relation='CORRECTION', target_event_id=prior['id'], target_goal_id=request['goal_id'] if request else None)
    if used:
        if wanted == 'SUP-MEETING':
            plan['answer'] = used[0]['text'].strip()
        elif wanted == 'SUP-RELEASE':
            plan['answer'] = ('기존 접근 권한이 있는 내부 검토자에게 내부 검토 링크만 공유하는 것은 가능합니다. 외부 발송과 새 외부 권한 부여는 허용되지 않습니다. 실제 공유는 실행하지 않습니다.' if '내부 검토 링크' in text else '현 상태에서는 월요일10시 외부 공개를 진행할 수 없습니다. 개인 이메일 주소 제거와 외부 공개용 정리본 검토가 필요합니다. 배포 지원은 월요일12시부터 가능하므로 승인 완료와 지원 가능 시각을 확인해 공개 시각을 조정해야 합니다. 실제 수정이나 발송은 실행하지 않습니다.')
        else:
            bindings = data.get('grounding', {}).get('bindings', [])
            plan['bindings'] = bindings
            plan['targets'] = [t for b in bindings for t in b['targets']]
            names = {}
            for p in used:
                for line in p['text'].splitlines():
                    parts = line.split(' | ')
                    if len(parts) == 3:
                        names[parts[0]] = parts[1]
            plan['answer'] = '표시한 두 버튼의 행 이름은 ' + ', '.join(names[t.rsplit(':', 1)[-1]] for t in plan['targets']) + '입니다.'
    return plan
