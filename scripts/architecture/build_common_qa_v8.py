#!/usr/bin/env python3
"""Apply the authored QA07 v3 protocol in a new unmeasured evaluation generation."""
import argparse
import copy
import csv
import hashlib
import json
from pathlib import Path
import common_qa_v8_request_fitness as fitness

ROOT=Path(__file__).resolve().parents[2]
CURRENT=ROOT/'docs/architecture/12-decisions/decision-packages/current'
BASE=CURRENT/'evaluation/common-qa-v7'
OUT=CURRENT/'evaluation/common-qa-v8'
SOURCES=['qa07-request-meaning-accuracy.md','qa07-evaluation-plan.md','qa07-evaluation-cases.md','research-notes/qa07-meaning-research.md']


def read(p):return json.loads(p.read_text())
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(p,v):
    p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')


def build(out):
    sources={};texts={}
    for name in SOURCES:
        src=CURRENT/'qa'/name;rel=Path('definition-snapshots')/(Path(name).name+'.txt');pinned=OUT/rel
        data=pinned.read_bytes() if pinned.exists() else src.read_bytes();dest=out/rel;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(data)
        sources[name]=dict(source=str(src.relative_to(ROOT)),snapshot=str(rel),sha256=hashlib.sha256(data).hexdigest());texts[name]=data.decode()
    d=copy.deepcopy(read(BASE/'artifacts/ledger.json'))
    d.update(version='COMMON-QA-MODEL-8.0',parent_ledger_sha256=digest(BASE/'artifacts/ledger.json'),generator_sha256=digest(Path(__file__)),
             fitness_module_sha256=digest(Path(fitness.__file__)),legacy_qa07=d['qa07'])
    d['qa07']=fitness.project(texts['qa07-evaluation-cases.md'],d['configs'])
    d['contracts'].update(qa07='QA07-REQUEST-RESPONSE-v3',qa07_population='QA07-CASES-v3',qa07_score='QA07-REQUEST-RESPONSE-score-v3')
    d['scope_review']['qa07']='AUTHOR_DEFINED_V3_32_CASES_5_REPEATS_NO_EXECUTION'
    d['policy']={
        'legacy_24_input_analytical':dict(scope='QA01/02/03/05/06/08 analysis and preserved v2 QA07 only; never apply to QA07 v3',rules=d['policy']),
        'qa04':dict(scope='QA04-CHANGE-v4 actual eight independent changes; no patch means NOT_MEASURED',source='qa04'),
        'qa07_v3':dict(scope='QA07-CASES-v3 32 cases x 5 repeats; author proposal, not executed',
                       source='qa07.manifest.source_contract',schedule='Causal response/event triggers from pinned v3 contract; not the old fixed input schedule',
                       target=90,score_version='QA07-REQUEST-RESPONSE-score-v3',user_adopted=False,actual_executions=0),
        'reporting':'One representative per QA; QA07 v3 has a distinct population; no cross-QA pooled score',
    }
    d['appendices']['legacy_qa07_types']=d['appendices']['qa07_types']
    groups=list(dict.fromkeys(c['type'] for c in d['qa07']['manifest']['cases']))
    d['appendices']['qa07_types']=[dict(config_id=cfg['id'],type=group,case_families=4,planned_repeats=20,unknown=20,observed=0,ratio=None,
                                            failure_causes={s:None for s in fitness.STAGES},overlapping_causes=True) for cfg in d['configs'] for group in groups]
    for cfg,s in d['summary'].items():s.update(qa07=None,qa07_score=None,qa07_planned_repeats=160,qa07_observed=0,qa07_target=90,qa07_target_status='AUTHOR_PROPOSAL')
    save(out/'artifacts/ledger.json',d);save(out/'artifacts/summary.json',d['summary']);save(out/'artifacts/appendices.json',d['appendices'])
    save(out/'qa07-case-manifest.json',d['qa07']['manifest']);save(out/'artifacts/qa07-results.json',d['qa07'])
    save(out/'definition-sources.json',sources)
    save(out/'preserved-v7-sha256.json',{str(p.relative_to(ROOT)):digest(p) for p in sorted(BASE.rglob('*')) if p.is_file()})
    with (out/'artifacts/comparison.csv').open('w') as f:
        w=csv.writer(f,lineterminator='\n');w.writerow(['config_id','qa01_s','qa02_pct','qa03_s','qa04_modules','qa05_usd_per_bundle','qa06_mb','qa07_pct','qa07_score','qa07_planned_repeats','qa08_gb','evidence'])
        for cfg in d['configs']:
            s=d['summary'][cfg['id']];w.writerow([cfg['id'],*[s[k] for k in ['qa01','qa02','qa03','qa04','qa05','qa06','qa07','qa07_score','qa07_planned_repeats','qa08_gb']],d['evidence_kind']])
    lines=['# QA07 v3 사례와 반복', '', '[목록](README.md) / [정의](../../qa/qa07-request-meaning-accuracy.md) / [32사례 정본](../../qa/qa07-evaluation-cases.md) / [기계 원장](qa07-case-manifest.json)', '',
           '32사례 × 5회 = 구성당 계획 160회이며 7구성의 1,120기록은 모두 UNKNOWN이다. 실제 실행은 0회다. 사례마다 최초 해석/응답 구성/실제 전달 세 단계에 모든 revision과 필수 의무를 연결한다. 3단계 × 1,120행은 3,360개 판정 작성 슬롯이며 성공률 분모가 아니다.', '',
           '목표 90% 이상(144/160), 점수 0~6, 반복과 기한은 작성자 제안이다. 실행 fixture/원음/화면 해시와 계측 조건은 아직 동결하지 않았으므로 실제 분모 N, 충족률과 점수는 미산출이다. 사례 원문 전체와 공통 자료/사건 진행 규칙은 manifest.source_contract 및 정의 snapshot으로 보존한다.', '',
           '| 사례 | 유형 | 입력/사건 | 필수 해석·응답 | 금지 조건 |', '| --- | --- | --- | --- | --- |']
    for c in d['qa07']['manifest']['cases']:lines.append('| '+' | '.join(c[k] for k in ['case_id','type','input_and_events','required','forbidden'])+' |')
    (out/'02-qa07-cases.md').write_text('\n'.join(lines)+'\n')
    print(json.dumps(dict(version=d['version'],qa07_cases=32,planned_repeats_per_config=160,records=1120,actual_executions=0)))


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,default=OUT);build(p.parse_args().output)
