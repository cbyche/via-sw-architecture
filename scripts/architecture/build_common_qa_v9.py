#!/usr/bin/env python3
"""Prepare QA07 v4 mapping and empty evidence slots; never execute candidates."""
import argparse
import copy
import csv
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CURRENT = ROOT / 'docs/architecture/12-decisions/decision-packages/current'
BASE = CURRENT / 'evaluation/common-qa-v8'
OUT = CURRENT / 'evaluation/common-qa-v9'
SOURCES = ['qa07-request-meaning-accuracy.md', 'qa07-evaluation-plan.md',
           'qa07-evaluation-cases.md', 'qa07-order-link-population.json',
           'research-notes/qa07-meaning-research.md']


def read(p):
    return json.loads(p.read_text())


def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def save(p, value):
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def build(out):
    sources = {}
    for name in SOURCES:
        src = CURRENT / 'qa' / name
        rel = Path('definition-snapshots') / (Path(name).name + '.txt')
        pinned = OUT / rel
        data = pinned.read_bytes() if pinned.exists() else src.read_bytes()
        dest = out / rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(data)
        sources[name] = dict(source=str(src.relative_to(ROOT)), snapshot=str(rel),
                             sha256=hashlib.sha256(data).hexdigest())
    population = json.loads((out / sources['qa07-order-link-population.json']['snapshot']).read_text())
    d = copy.deepcopy(read(BASE / 'artifacts/ledger.json'))
    d.update(version='COMMON-QA-MODEL-9.0', parent_ledger_sha256=digest(BASE / 'artifacts/ledger.json'),
             generator_sha256=digest(Path(__file__)))
    d['legacy_qa07_v3'] = d['qa07']
    records = []
    fields = ['order_verdict', 'link_verdict', 'judgment_evidence', 'reviewer_1', 'reviewer_2',
              'normalization_review_ref', 'input_provided_ref', 'first_use_ref', 'first_use_elapsed_s',
              'complete_timeout_ref', 'source_execution_key', 'prior_proven_failure_ref',
              'input_hash', 'context_hash', 'normalized_structure_ref', 'cause_id']
    for cfg in d['configs']:
        for c in population['cases']:
            for repeat in range(1, 6):
                records.append(dict(config_id=cfg['id'], source_event_id=c['source_event_id'], repeat_id=repeat,
                                    contract_version='QA07-ORDER-LINK-v4', status='UNKNOWN',
                                    fixture_ready=False, judgment_history=[], **{k: None for k in fields}))
    cases = [dict(c, order_type='순서 있음' if c['before'] else '순서 없음',
                  explicit_independence=c['source_event_id']=='S17-U01',
                  fixture_ready=False, candidate_payload_ref=None, common_invalid_ref=None)
             for c in population['cases']]
    d['qa07'] = dict(manifest=dict(contract_version='QA07-ORDER-LINK-v4', population=population,
                                  cases=cases, input_mode='TEXT', scope_user_adopted=True,
                                  target_score_user_adopted=False, target=85, required_successes=119,
                                  planned_denominator=140, repeats=5, timeout_s=60,
                                  execution_frozen=False, candidate_fixture_status='NOT_PREPARED',
                                  source_records_role='EVALUATOR_ONLY_NOT_CANDIDATE_PAYLOAD',
                                  fixture_requirement='Prepare pre-input history/state without oracle or future-event leakage; hash and review before execution'),
                     records=records, actual_executions=0,
                     summary={cfg['id']: dict(planned=140, S=0, F=0, U=140, INVALID=0,
                                              N=None, Q=None, score=None, actual_executions=0) for cfg in d['configs']})
    d['contracts'].update(qa07='QA07-ORDER-LINK-v4', qa07_population=population['version'],
                          qa07_score='QA07-ORDER-LINK-score-v4')
    d['scope_review']['qa07']='USER_ADOPTED_ORDER_LINK_SCOPE; AUTHOR_PROPOSAL_85_PERCENT_140_TRIALS; NO_EXECUTION'
    d['policy']['legacy_qa07_v3'] = d['policy'].pop('qa07_v3')
    d['policy']['qa07_v4'] = dict(scope='28 selected COMMON-28 user inputs x 5; TEXT only',
                                 deadline='60s first adopted/used order and link; no response or delivery requirement',
                                 target=85, target_user_adopted=False, actual_executions=0,
                                 old_timing_cost_transfer=False, candidate_fixture_ready=False)
    d['policy']['reporting']='One representative per QA; QA07 v4 has distinct 28-input population; no pooled QA score'
    d['appendices']['legacy_qa07_v3_types'] = d['appendices']['qa07_types']
    d['appendices']['qa07_types'] = [dict(config_id=cfg['id'], type=label, input_count=count,
                                         planned=count*5, S=0, F=0, U=count*5, Q=None)
                                     for cfg in d['configs'] for label,count in [('순서 있음',2),('순서 없음',26)]]
    d['appendices']['qa07_explicit_independence'] = [dict(config_id=cfg['id'], source_event_id='S17-U01',
                                                       planned=5, S=0, F=0, U=5, Q=None) for cfg in d['configs']]
    d['qa07_applicability'] = {
        'DP2': 'POTENTIAL: request interpretation; effect not measured',
        'DP3': 'CONDITIONAL: referent errors must propagate to task links; TEXT test alone does not prove screen-grounding benefit',
        'DP5': 'POTENTIAL: restoring prior task relations; effect not measured',
        'DP44': 'UNRESOLVED: only changes to first adopted order/link count; voice queue cancellation/resumption alone is not coverage'}
    for s in d['summary'].values():
        s.update(qa07=None, qa07_score=None, qa07_planned_repeats=140, qa07_observed=0,
                 qa07_target=85, qa07_target_status='AUTHOR_PROPOSAL')
    for name,value in [('ledger',d),('summary',d['summary']),('appendices',d['appendices']),('qa07-results',d['qa07'])]:
        save(out / 'artifacts' / (name+'.json'), value)
    save(out / 'qa07-case-manifest.json', d['qa07']['manifest'])
    save(out / 'definition-sources.json', sources)
    save(out / 'preserved-v8-sha256.json', {str(p.relative_to(ROOT)):digest(p) for p in sorted(BASE.rglob('*')) if p.is_file()})
    with (out/'artifacts/comparison.csv').open('w') as f:
        w=csv.writer(f,lineterminator='\n')
        keys=['qa01','qa02','qa03','qa04','qa05','qa06','qa07','qa07_score','qa07_planned_repeats','qa08_gb']
        w.writerow(['config_id',*keys,'evidence'])
        for cfg in d['configs']:w.writerow([cfg['id'],*[d['summary'][cfg['id']][k] for k in keys],d['evidence_kind']])
    lines=['# QA07 v4 선정 입력과 정답', '',
           '[목록](README.md) / [원장](qa07-case-manifest.json) / [원전 정의](../../qa/qa07-evaluation-cases.md)', '',
           '84개 사용자 입력 중 28개 선정, 56개 제외. 구성당 28 × 5 = 계획 140회이며 7구성의 980기록은 모두 UNKNOWN이다. 실제 실행은 0회다. 목표 85%(119/140)와 3%p 점수 구간은 작성자 제안이다.', '',
           '순서 있음 2입력/10반복, 없음 26입력/130반복. 명시적 독립 요청 S17-U01은 없음 그룹에 포함되며 별도 진단으로만 표시한다. 정답/설명용 문맥 원장은 후보 입력이 아니다. 입력 전 이력/업무 상태를 누출 없이 구성한 실행 fixture는 미준비다.', '',
           '| 원전 입력 | 원문 | 연결 정답 | 선후 관계 | 파생 문맥 |', '| --- | --- | --- | --- | --- |']
    for c in cases:
        links='; '.join(u['anchor']+': '+u['link'] for u in c['units'])
        edges='; '.join(' → '.join(e) for e in c['before']) or '없음 (불필요한 관계도 오류)'
        lines.append('| '+' | '.join([c['source_event_id'],c['input'],links,edges,'TS17 진행 중, 원전 일정 미사용' if c['context_adapted'] else '없음'])+' |')
    lines += ['', '## 제외 56입력', '', '| 원전 입력 | 제외 이유 |', '| --- | --- |']
    lines += ['| '+c['source_event_id']+' | '+c['reason']+' |' for c in population['excluded_user_events']]
    (out/'02-qa07-cases.md').write_text('\n'.join(lines)+'\n')
    print(json.dumps(dict(version=d['version'],selected=28,excluded=56,planned_per_config=140,records=980,actual_executions=0)))


if __name__ == '__main__':
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,default=OUT);build(p.parse_args().output)
