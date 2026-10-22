#!/usr/bin/env python3
"""Check QA07 fixture coverage, aggregation edge cases and v5 preservation."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import contextlib
import io

ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('qa6',ROOT/'scripts/architecture/build_common_qa_v6.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)


def main():
    d=m.read(m.OUT/'artifacts/ledger.json');base=m.read(m.BASE/'artifacts/ledger.json')
    q=d['qa07'];manifest=q['manifest']
    assert manifest==m.read(m.OUT/'qa07-case-manifest.json')
    assert d['parent_ledger_sha256']==m.digest(m.BASE/'artifacts/ledger.json')
    for key in ['cases','scenarios','configs','changes','summary','deltas','profile','evidence_kind']:
        assert d[key]==base[key],('v5 changed',key)
    for index in ['preserved-v5-sha256.json','definition-sources.json']:
        for p,h in m.read(m.OUT/index).items():assert m.digest(ROOT/p)==h,p
    assert manifest['status']=='PROPOSED_NOT_FROZEN' and manifest['frozen_denominator'] is None
    assert manifest['target'] is manifest['score_bands'] is manifest['repeat_count'] is None
    assert len(manifest['cases'])==24
    assert {c['source_case_id'] for c in manifest['cases']}=={s['id'] for s in d['scenarios']}
    allids=[]
    for c in manifest['cases']:
        assert not c['contract_ready'] and c['functional_deadline_s'] is c['post_observation_s'] is None
        assert {o['stage'] for o in c['obligations']}==set(m.STAGES)
        assert all(c['case_id'] in o['id'] for o in c['obligations'])
        assert {'D-screen','D-audio','D-close'}<={o['id'].split(':')[-1] for o in c['obligations']}
        if c['proposed_root_input'] is None:
            assert any(o['id'].endswith(':I-origin') for o in c['obligations'])
        for o in c['obligations']:
            if 'event_ref' in o:assert o['event_ref'] in {e['id'] for e in c['events']}
        allids += [o['id'] for o in c['obligations']]
    assert len(allids)==len(set(allids))==162
    keys=[(r['config_id'],r['case_id']) for r in q['records']]
    assert len(keys)==len(set(keys))==168
    for r in q['records']:
        assert r['status']=='UNKNOWN' and r['judgments']==[]
        assert r['repeat_id'] is r['source_execution_key'] is r['observation_closed_evidence'] is None
    for s in q['summary'].values():assert s==m.aggregate(['UNKNOWN']*24)
    # These are isolated software tests of aggregation, never candidate observations.
    case=copy.deepcopy(manifest['cases'][0]);record={'judgments':[]}
    checks=[]
    def expect(label,status):
        assert m.evaluate_case(case,record)==status,label
        checks.append(label)
    expect('empty_is_unknown','UNKNOWN')
    record['judgments']=[dict(obligation_id=o['id'],verdict='PASS',evidence_ref='test',evaluator_ref='test') for o in case['obligations']]
    expect('oracle_or_pass_without_contract_not_success','UNKNOWN')
    case['contract_ready']=True;record.update(source_execution_key='test-run',repeat_id='test-repeat',observation_closed_evidence='test-end')
    expect('all_obligations_and_closed_contract_pass','PASS')
    first=copy.deepcopy(record['judgments'][0]);first['verdict']='UNKNOWN';record['judgments'].insert(0,first)
    expect('earlier_unknown_resolved_by_final_pass','PASS')
    missing=copy.deepcopy(record['judgments'][1]);missing['verdict']='FAIL';missing['evidence_ref']=None;record['judgments'].append(missing)
    expect('latest_unproven_failure_not_old_pass','UNKNOWN');record['judgments'].pop()
    record['case_id']=case['case_id']
    statuses,summary=m.evaluate_records([case],[record,copy.deepcopy(record)],True)
    assert statuses==['UNKNOWN','UNKNOWN'] and summary['N'] is None;checks.append('duplicate_execution_blocks_json_denominator')
    record['judgments'].pop(0)
    record['observation_closed_evidence']=None;expect('no_end_evidence_unknown','UNKNOWN');record['observation_closed_evidence']='test-end'
    record['judgments'][0]['evidence_ref']=None;expect('no_evidence_unknown','UNKNOWN');record['judgments'][0]['evidence_ref']='test'
    record['judgments'][0]['evaluator_ref']=None;expect('no_evaluator_unknown','UNKNOWN');record['judgments'][0]['evaluator_ref']='test'
    record['judgments'].append(dict(obligation_id=case['obligations'][0]['id'],verdict='FAIL',evidence_ref='test-earlier',evaluator_ref='test'))
    expect('first_failure_survives_later_pass','FAIL')
    record['judgments']=record['judgments'][-1:];case['contract_ready']=False;expect('failure_survives_incomplete_trace','FAIL')
    record['common_invalid_evidence']='candidate-only-exclusion';expect('candidate_exclusion_cannot_hide_failure','FAIL')
    case['common_invalid_evidence']='common-fixture-invalid';expect('common_invalid_separate','INVALID')
    assert m.aggregate(['PASS','FAIL','UNKNOWN'])['Q'] is None;checks.append('unfrozen_denominator_not_numeric')
    s=m.aggregate(['PASS','FAIL','UNKNOWN'],True);assert s['Q'] is None and s['bounds']==[100/3,200/3];checks.append('unknown_bounds_keep_full_denominator')
    assert m.aggregate(['PASS','FAIL','INVALID'],True)['Q']==50;checks.append('invalid_separate_from_failure')
    assert m.aggregate([],True)['Q'] is None;checks.append('zero_denominator_unknown')
    with tempfile.TemporaryDirectory(prefix='via-qa6-') as tmp:
        with contextlib.redirect_stdout(io.StringIO()):m.build(Path(tmp))
        for p in ['qa07-case-manifest.json','artifacts/ledger.json','artifacts/qa07-results.json','artifacts/summary.json','artifacts/comparison.csv','artifacts/qa07-cases.csv','02-qa07-case-mapping.md']:
            assert (Path(tmp)/p).read_bytes()==(m.OUT/p).read_bytes(),p
    result=dict(status='PASS',case_families=24,obligations_per_configuration=162,draft_records=168,
                unchanged_v5='PASS',deterministic_reproduction='PASS',aggregation_probes=checks,actual_executions=0)
    m.write(m.OUT/'artifacts/validation.json',result);print(json.dumps(result,ensure_ascii=False))


if __name__=='__main__':main()
