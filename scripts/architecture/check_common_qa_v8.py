#!/usr/bin/env python3
"""Check the new QA07 protocol without running any candidate/model."""
import contextlib
import copy
import io
import json
from pathlib import Path
import tempfile
import build_common_qa_v8 as m
import common_qa_v8_request_fitness as f


def main():
    d=m.read(m.OUT/'artifacts/ledger.json');base=m.read(m.BASE/'artifacts/ledger.json');q=d['qa07'];manifest=q['manifest']
    assert d['generator_sha256']==m.digest(Path(m.__file__)) and d['fitness_module_sha256']==m.digest(Path(f.__file__))
    for p,h in m.read(m.OUT/'preserved-v7-sha256.json').items():assert m.digest(m.ROOT/p)==h,p
    for source in m.read(m.OUT/'definition-sources.json').values():assert m.digest(m.OUT/source['snapshot'])==source['sha256']
    for key in ['cases','scenarios','configs','qa04','changes','legacy_changes','legacy_qa04','deltas']:
        assert d[key]==base[key],key
    assert d['legacy_qa07']==base['qa07'] and d['appendices']['legacy_qa07_types']==base['appendices']['qa07_types']
    assert d['policy']['legacy_24_input_analytical']['rules']==base['policy']
    assert d['policy']['qa07_v3']['source']=='qa07.manifest.source_contract'
    assert 'scenario_weight' not in d['policy'] and d['policy']['qa07_v3']['actual_executions']==0
    for cfg,s in base['summary'].items():
        for k,v in s.items():assert d['summary'][cfg][k]==v,(cfg,k)
    assert manifest==m.read(m.OUT/'qa07-case-manifest.json')
    assert manifest['cases']==f.parse_cases(manifest['source_contract'])
    assert manifest['repeat_count']==5 and manifest['planned_denominator']==160 and manifest['target']==90
    assert manifest['execution_frozen'] is manifest['user_adopted'] is False
    assert (manifest['obligation_deadline_s'],manifest['case_deadline_s'],manifest['post_observation_s'])==(60,300,10)
    assert len(q['records'])==1120 and len({(r['config_id'],r['case_id'],r['repeat_id']) for r in q['records']})==1120
    for cfg in d['configs']:
        records=[r for r in q['records'] if r['config_id']==cfg['id']]
        summary,statuses=f.aggregate(manifest['cases'],records)
        assert summary==q['summary'][cfg['id']] and statuses==['UNKNOWN']*160
        assert summary['N'] is summary['Q'] is summary['score'] is None
        assert summary['actual_executions']==0
    # Isolated aggregation software probes; labels below are never written as observations.
    checks=[];case=copy.deepcopy(manifest['cases'][0]);record=copy.deepcopy(q['records'][0])
    def expect(label,status):
        assert f.evaluate(case,record)==status,label;checks.append(label)
    expect('empty_unknown','UNKNOWN')
    record['judgments']=[dict(obligation_id=o['id'],verdict='PASS',evidence_ref='test',evaluators=[dict(evaluator_ref='test-reviewer-1',verdict='PASS'),dict(evaluator_ref='test-reviewer-2',verdict='PASS')]) for o in case['obligations']]
    expect('judgments_without_trace_unknown','UNKNOWN')
    record.update(source_execution_key='test-run',observation_closed_evidence='test-close',protocol_evidence_ref='test-protocol',first_adoption_trace_ref='test-first')
    expect('no_fixture_unknown','UNKNOWN');case.update(fixture_ready=True,fixture_hashes={'input':'test-hash'})
    expect('all_evidence_pass','PASS')
    record['judgments'][0]['evaluators'][1]['evaluator_ref']='test-reviewer-1';expect('same_reviewer_not_two','UNKNOWN')
    record['judgments'][0]['evaluators'][1]=dict(evaluator_ref='test-reviewer-2',verdict='FAIL');expect('disagreement_unresolved','UNKNOWN')
    record['judgments'][0]['adjudication_ref']='test-resolution';expect('disagreement_resolved','PASS')
    past=copy.deepcopy(record['judgments'][0]);past['verdict']='FAIL';record['judgments'].insert(0,past);expect('first_proven_fail_survives_pass','FAIL')
    record['observation_closed_evidence']=None;expect('fail_survives_incomplete_end','FAIL')
    case['common_invalid_evidence']='test-common-fixture';expect('common_invalid_preserves_history','INVALID')
    cases=copy.deepcopy(manifest['cases']);records=[]
    for c in cases:
        c.update(fixture_ready=True,fixture_hashes={'input':'test-hash'})
        for repeat in range(1,6):
            records.append(dict(config_id='TEST',case_id=c['case_id'],repeat_id=repeat,source_execution_key=f'test-{c["case_id"]}-{repeat}',
                                observation_closed_evidence='test-close',protocol_evidence_ref='test-protocol',first_adoption_trace_ref='test-first',
                                judgments=[dict(obligation_id=o['id'],verdict='PASS',evidence_ref='test',evaluators=[dict(evaluator_ref='r1',verdict='PASS'),dict(evaluator_ref='r2',verdict='PASS')]) for o in c['obligations']]))
    s,_=f.aggregate(cases,records,True);assert s['N']==160 and s['Q']==100 and s['score']==6;checks.append('160_unique_complete')
    s,_=f.aggregate(cases,records,False);assert s['Q'] is None;checks.append('unfrozen_no_score')
    for r in records[:16]:
        j=r['judgments'][0];j['verdict']='FAIL'
        for e in j['evaluators']:e['verdict']='FAIL'
    s,_=f.aggregate(cases,records,True);assert s['Q']==90 and s['score']==1;checks.append('144_of_160_target')
    saved=copy.deepcopy(records[-1]);records[-1]['judgments']=[]
    s,_=f.aggregate(cases,records,True);assert s['Q'] is s['score'] is None and s['bounds']==[143/160*100,90];checks.append('unknown_not_success_denominator')
    records[-1]=saved;records[-1]['source_execution_key']=records[-2]['source_execution_key'];s,_=f.aggregate(cases,records,True);assert s['N'] is None;checks.append('duplicate_execution_no_ratio')
    records[-1]=saved;cases[0]['common_invalid_evidence']='test';s,_=f.aggregate(cases,records,True);assert s['N'] is None and s['INVALID']==5;checks.append('invalid_not_smaller_denominator')
    cases[0]['common_invalid_evidence']=None;records[-1]['config_id']='OTHER';s,_=f.aggregate(cases,records,True);assert s['N'] is None;checks.append('mixed_config_no_ratio')
    for value,score in [(0,0),(89.999,0),(90,1),(91.999,1),(92,2),(94,3),(96,4),(98,5),(99.999,5),(100,6)]:assert f.score(value)==score
    checks.append('all_score_boundaries')
    with tempfile.TemporaryDirectory(prefix='via-qa8-') as tmp:
        with contextlib.redirect_stdout(io.StringIO()):m.build(Path(tmp))
        for p in ['artifacts/ledger.json','artifacts/summary.json','artifacts/appendices.json','artifacts/qa07-results.json','artifacts/comparison.csv','qa07-case-manifest.json','definition-sources.json','02-qa07-cases.md']:
            assert (Path(tmp)/p).read_bytes()==(m.OUT/p).read_bytes(),p
    report=dict(status='PASS',qa07_cases=32,planned_repeats_per_config=160,records=1120,actual_executions=0,
                parent_v7='BYTE_PRESERVED',definition_snapshot='PINNED',unchanged_other_qas='PASS',probes=checks,reproduction='PASS')
    m.save(m.OUT/'artifacts/validation.json',report);print(json.dumps(report,ensure_ascii=False))


if __name__=='__main__':main()
