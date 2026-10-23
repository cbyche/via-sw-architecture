#!/usr/bin/env python3
"""Validate v9 mapping, preserved artifacts and absent observations, without models."""
import contextlib
import io
import json
import tempfile
from pathlib import Path
import build_common_qa_v9 as m


def main():
    d=m.read(m.OUT/'artifacts/ledger.json');base=m.read(m.BASE/'artifacts/ledger.json')
    assert d['generator_sha256']==m.digest(Path(m.__file__))
    for p,h in m.read(m.OUT/'preserved-v8-sha256.json').items():assert m.digest(m.ROOT/p)==h,p
    for source in m.read(m.OUT/'definition-sources.json').values():assert m.digest(m.OUT/source['snapshot'])==source['sha256']
    for k in ['cases','scenarios','configs','qa04','changes','legacy_changes','legacy_qa04','deltas']:
        assert d[k]==base[k],k
    assert d['legacy_qa07_v3']==base['qa07']
    for cfg,s in base['summary'].items():
        for k,v in s.items():
            if not k.startswith('qa07'):assert d['summary'][cfg][k]==v,(cfg,k)
    q=d['qa07'];mf=q['manifest'];pop=mf['population'];cases=mf['cases']
    assert (mf['contract_version'],mf['input_mode'],mf['target'],mf['required_successes'],mf['planned_denominator'])==('QA07-ORDER-LINK-v4','TEXT',85,119,140)
    assert mf['scope_user_adopted'] and not mf['target_score_user_adopted'] and not mf['execution_frozen']
    assert mf['candidate_fixture_status']=='NOT_PREPARED' and mf['source_records_role'].startswith('EVALUATOR_ONLY')
    assert pop==m.read(m.OUT/'definition-snapshots/qa07-order-link-population.json.txt')
    assert m.digest(m.ROOT/pop['source'])==pop['source_sha256']
    original=m.read(m.ROOT/pop['source'])
    user_events={e['id']:e for s in original['scenarios'] for e in s['events'] if e['kind']=='user'}
    included={c['source_event_id'] for c in cases};excluded={c['source_event_id'] for c in pop['excluded_user_events']}
    assert len(included)==28 and len(excluded)==56 and not included&excluded and included|excluded==set(user_events)
    assert all(c['input']==user_events[c['source_event_id']]['input'] for c in cases)
    assert [c['source_event_id'] for c in cases if c['before']]==['S17-U02','S17-U03']
    assert [c['source_event_id'] for c in cases if c['context_adapted']]==['S17-U03']
    assert all(c['candidate_payload_ref'] is None and not c['fixture_ready'] and c['common_invalid_ref'] is None for c in cases)
    assert {c['source_event_id'] for c in cases if c['explicit_independence']}=={'S17-U01'}
    assert len(q['records'])==980 and q['actual_executions']==0
    expected={(c['id'],eid,r) for c in d['configs'] for eid in included for r in range(1,6)}
    assert {(r['config_id'],r['source_event_id'],r['repeat_id']) for r in q['records']}==expected
    for r in q['records']:
        assert r['status']=='UNKNOWN' and not r['fixture_ready'] and r['judgment_history']==[]
        assert all(v is None for k,v in r.items() if k not in ['config_id','source_event_id','repeat_id','contract_version','status','fixture_ready','judgment_history'])
    assert all(s==dict(planned=140,S=0,F=0,U=140,INVALID=0,N=None,Q=None,score=None,actual_executions=0) for s in q['summary'].values())
    for cfg in d['configs']:
        rows=[r for r in d['appendices']['qa07_types'] if r['config_id']==cfg['id']]
        assert [(r['input_count'],r['planned'],r['U']) for r in rows]==[(2,10,10),(26,130,130)]
    assert d['policy']['legacy_qa07_v3']==base['policy']['qa07_v3'] and not d['policy']['qa07_v4']['old_timing_cost_transfer']
    assert len(d['qa07_applicability'])==4 and 'voice queue' in d['qa07_applicability']['DP44']
    with tempfile.TemporaryDirectory(prefix='via-qa9-') as tmp:
        with contextlib.redirect_stdout(io.StringIO()):m.build(Path(tmp))
        for f in ['artifacts/ledger.json','artifacts/summary.json','artifacts/appendices.json','artifacts/qa07-results.json','artifacts/comparison.csv','qa07-case-manifest.json','definition-sources.json','02-qa07-cases.md']:
            assert (Path(tmp)/f).read_bytes()==(m.OUT/f).read_bytes(),f
    result=dict(status='PASS',selected_inputs=28,excluded_inputs=56,planned_per_config=140,
                records=980,order_positive_inputs=2,order_empty_inputs=26,actual_executions=0,
                parent_v8='BYTE_PRESERVED',other_qa_values='UNCHANGED',definition_snapshot='PINNED',
                observations='ALL_UNKNOWN',candidate_fixtures='NOT_PREPARED',reproduction='PASS')
    m.save(m.OUT/'artifacts/validation.json',result);print(json.dumps(result))


if __name__=='__main__':main()
