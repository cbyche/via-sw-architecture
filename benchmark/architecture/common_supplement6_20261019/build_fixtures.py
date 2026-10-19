"""Contract20 authored public inputs and physically separate host-only oracles."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
CONTRACT = REPO / 'docs/architecture/12-decisions/decision-packages/seven-problem-review/20-common-supplement-evaluation-contract.md'
COMMON = REPO / 'benchmark/architecture/common9_poc_20261019'
RELEASE = '''P01 공개 원칙: 외부 공개본에는 개인 이메일 주소가 없어야 한다. 내부 검토본과 외부 공개본은 별도 판본으로 관리한다.
P02 현재 판본: 초안 r2의 표2에는 개인 이메일 주소가 남아 있으며, 외부 공개용 정리본은 아직 작성하지 않았다.
P03 외부 공개 승인: 보안 승인과 정리본 검토가 모두 완료되어야 외부 공개를 진행할 수 있다. 한 가지 완료만으로는 충분하지 않다.
P04 승인 현황: 보안 승인은 완료됐고, 정리본 검토는 미완료다. 보안 승인은 초안의 이메일 주소를 공개해도 된다는 뜻이 아니다.
P05 일정 제안: 외부 공개 예정 시각은 월요일10시이며, 배포 지원 담당자는 월요일12시부터 지원할 수 있다. 예정 시각 자체가 승인을 대신하지 않는다.
P06 내부 검토: 기존 접근 권한이 있는 내부 검토자에게 내부 검토 링크만 공유하는 것은 가능하다. 이 허용은 외부 발송이나 새 외부 권한 부여를 포함하지 않는다.
P07 다음 조치: 개인 이메일 주소를 제거한 외부 공개용 정리본을 만들고 검토를 요청한다. 외부 공개 시각은 승인 완료와 배포 지원 가능 시각을 확인한 뒤 조정한다.
P08 요청 범위: 이 문서는 가능 여부와 조건을 검토하기 위한 자료다. 설명 요청만으로 문서 수정, 링크 발송, 외부 공개를 실행하지 않는다.
'''
MEETING = '수요일 점검 회의는14시에3층 회의실B에서 시작한다. 운영팀과 품질팀이 참석한다.\n'
FIRST = '공개 검토 문서의 여덟 항목을 대조해 월요일10시 외부 공개안을 진행할 수 있는지, 막는 조건과 다음 조치를 네 문장 이내로 설명해줘. 실제 수정이나 발송은 하지 마.'
INDEPENDENT = '회의 안내문에서 수요일 점검 회의 장소와 시작 시각을 알려줘.'
CORRECTION = '앞 요청을 정정할게. 외부 공개 대신 외부 발송 없이 내부 검토 링크만 공유하는 경우로 바꿔서, 가능 여부와 남은 주의사항을 네 문장 이내로 설명해줘. 실제 공유는 하지 마.'
SCREEN_FIRST = '표시한 두 버튼의 행 이름을 알려줘.'
SCREEN_CORRECTION = '직전 요청을 정정할게. 지금 표시한 두 버튼의 행 이름을 알려줘.'
CONFIGS = ['AAAA', 'BAAA', 'CAAA', 'ABAA', 'AABA', 'AAAB', 'AAAC']


def dump(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def source(id, title, text, extents):
    return {'id': id, 'revision': 1, 'title': title, 'kind': 'document', 'text': text, 'extents': extents, 'local_allowed': True, 'external_allowed': False}


def main():
    spec = importlib.util.spec_from_file_location('common9_authored_initial_builder', COMMON / 'build_fixtures.py')
    common = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(common)
    observations = json.loads((HERE / 'assets/observations.json').read_text())['events']
    screen = (HERE / 'assets/SUP-SCREEN.txt').read_text()
    (HERE / 'assets/SUP-RELEASE.txt').write_text(RELEASE)
    (HERE / 'assets/SUP-MEETING.txt').write_text(MEETING)
    fixtures = []
    oracles = {}
    for n in range(80, 86):
        sid = f'S{n}'
        initial = common.initial(sid)
        history = next(s for s in initial['sources'] if s['id'] == 'CONVERSATION-LOG')
        history['external_allowed'] = False
        originals = [source('SUP-RELEASE', '공개 검토 문서', RELEASE, {f'P{i:02}': [i, i] for i in range(1, 9)}), source('SUP-MEETING', '회의 안내문', MEETING, {'meeting': [1, 1]})] if n < 83 else [source('SUP-SCREEN', '업무 목록', screen, {f'B{i:02}': [i, i] for i in range(1, 13)})]
        initial['sources'] = originals + [history]
        assert len(initial['raw_history']) == 200 and sum(t['state'] == 'COMPLETED' for t in initial['tasks']) == 20
        events = []
        schedule = []
        texts = [FIRST, CORRECTION if n == 82 else INDEPENDENT] if n < 83 else [SCREEN_FIRST, SCREEN_CORRECTION]
        for i, text in enumerate(texts, 1):
            eid = f'{sid}-U{i:02}'
            event = {'id': eid, 'kind': 'user', 'text': text, 'delivery_source_id': f'{sid}:answer{i:02}:delivered', 'input_evidence_class': 'AUTHORED_TEXT_INPUT' if n < 83 else 'AUTHORED_SCHEDULED_TRANSCRIPT_NOT_ASR'}
            if eid in observations:
                event.update(copy.deepcopy(observations[eid]))
            trigger = {'kind': 'initial', 'offset_seconds': 5} if i == 1 else {'kind': 'after_actual_provided' if n in [80, 82] else 'after_actual_final_or_missing_anchor', 'anchor_event_id': f'{sid}-U01', 'offset_seconds': 2}
            events.append(event)
            schedule.append({'event_id': eid, 'original_trigger': 'Initial actual successful source provided_at + 5s input begin' if i == 1 else f'{sid}-U01 actual successful provided_at + 2s completed Text' if n in [80, 82] else f'{sid}-U01 actual final callback + 2s input begin (30s missing-anchor fallback remains failure)', 'trigger': trigger, 'authored_utterance_seconds': 0 if n < 83 else 6, 'response_id': f'{sid}-RU{i:02}', 'event': event})
        fixtures.append({'id': sid, 'initial': initial, 'events': events, 'schedule': schedule, 'population': 'COMMON_SUPPLEMENT6', 'obligation_count': {'responses': 2, 'goals': 2 if n in [80, 81] else 1, 'bindings': 2 if n >= 83 else 0}})
        response_rows = [{'event_id': f'{sid}-U{i:02}', 'response_id': f'{sid}-RU{i:02}', 'required_content': 'Concrete explanation of actual authored source/request; content correctness is separate from callback qualification'} for i in [1, 2]]
        goals = [{'id': f'{sid}-G{i:02}', 'start_event': f'{sid}-U{i:02}', 'member_events': [f'{sid}-U{i:02}'], 'completion': 'Actual direct answer to this independent request, no real external action'} for i in [1, 2]] if n in [80, 81] else [{'id': f'{sid}-G01', 'start_event': f'{sid}-U01', 'member_events': [f'{sid}-U01', f'{sid}-U02'], 'completion': 'Final corrected request revision; old U01 completion alone is insufficient; no external action'}]
        bindings = [] if n < 83 else [{'id': f'{sid}-B0{i}', 'event_id': f'{sid}-U0{i}', 'expression': '표시한 두 버튼' if i == 1 else '지금 표시한 두 버튼', 'revision': 'final', 'expected_set': ['SUP-SCREEN:v1:B03', f'SUP-SCREEN:v1:B0{4 if i == 1 else 5}']} for i in [1, 2]]
        oracles[sid] = {'responses': response_rows, 'goals': goals, 'bindings': bindings}
    counts = {key: sum(len(row[key]) for row in oracles.values()) for key in ['responses', 'goals', 'bindings']}
    assert counts == {'responses': 12, 'goals': 8, 'bindings': 6}
    dump(HERE / 'fixtures.json', fixtures)
    dump(HERE / 'oracles.json', oracles)
    functional = {}
    for sid in oracles:
        n = int(sid[1:])
        if n < 83:
            first = ['Current Monday10 external release cannot proceed', 'Personal email removal and redacted-version review remain necessary', 'Deployment support begins Monday12; recheck approvals and adjust release time', 'No actual modification or sending']
            second = ['Wednesday14:00', 'Third-floor meeting roomB'] if n in [80, 81] else ['Internal review link sharing only to existing authorized internal reviewers is permitted', 'No external sending or new external permission grant', 'No actual sharing executed']
        else:
            first = ['Selected row names are 설계 and 검토', 'No opening/modification action executed']
            second = ['Corrected selected row names are 설계 and 구현', 'Same original goal with revised current set, no opening action']
        functional[sid] = {'SCOPE': 'Evaluator-only manual/content diagnostics, never QA01 qualification or QA02 denominator additions', 'events': {sid + '-U01': first, sid + '-U02': second}}
    dump(HERE / 'functional-oracles.json', functional)
    dump(HERE / 'population.json', {'version': 'COMMON_SUPPLEMENT6_CONTRACT20', 'contract_sha256': hashlib.sha256(CONTRACT.read_bytes()).hexdigest(), 'conditions': [f'S{i}' for i in range(80, 86)], 'configurations': CONFIGS, 'repetitions': 3, 'unique_episode_runs': 126, 'per_bundle': counts, 'per_configuration': {key: value * 3 for key, value in counts.items()}, 'oracle_only_fields_not_passed_to_candidate': ['responses.required_content', 'goals', 'bindings.expected_set', 'whole_future_schedule', 'authoring-layout.json'], 'all_data_authored_synthetic': True, 'actual_API_calls': 0})
    print(json.dumps({'episodes': 6, 'per_bundle': counts, 'actual_API_calls': 0}))


if __name__ == '__main__':
    main()
