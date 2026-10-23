#!/usr/bin/env python3
"""Validate common QA reporting, preserved evidence and actual-change evidence gates."""
import contextlib
import copy
import importlib.util
import io
import json
import math
from pathlib import Path
import tempfile

ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('qa7',ROOT/'scripts/architecture/build_common_qa_v7.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)


def main():
    d=m.read(m.OUT/'artifacts/ledger.json');base=m.read(m.BASE/'artifacts/ledger.json');ap=d['appendices']
    assert d['version']==m.VERSION and d['generator_sha256']==m.digest(Path(m.__file__))
    assert d['parent_ledger_sha256']==m.digest(m.BASE/'artifacts/ledger.json')
    for key in ['cases','scenarios','configs','qa07','profile','evidence_kind']:
        assert d[key]==base[key],key
    assert d['legacy_changes']==base['changes']
    assert len(d['legacy_changes'])==42
    for p,h in m.read(m.OUT/'preserved-v6-sha256.json').items():assert m.digest(ROOT/p)==h,p
    for source in m.read(m.OUT/'definition-sources.json').values():
        assert m.digest(m.OUT/source['snapshot'])==source['sha256'],source
    assert d['qa04']==m.read(m.OUT/'qa04-change-manifest.json')
    assert d['qa04']['tasks']==m.tasks() and len(d['qa04']['records'])==56
    assert len({(r['config_id'],r['task_id']) for r in d['qa04']['records']})==56
    for r in d['qa04']['records']:
        assert r['module_changes'] is r['module_count'] is r['patch_ref'] is None
        assert r['status']==r['completion_status']=='NOT_MEASURED'
    for reg in d['qa04']['registries']:assert reg['status']=='NOT_REGISTERED' and reg['modules']==[]
    for cfg,s in d['summary'].items():
        for key,val in base['summary'][cfg].items():
            if key not in ['qa04','change_tasks']:assert s[key]==val,(cfg,key)
        assert s['qa04'] is s['qa04_score'] is None and s['change_tasks']==8
        assert s['qa08_gb']==s['qa08']/1000 and s['qa06']>512
        assert d['legacy_qa04'][cfg]==base['summary'][cfg]['qa04']
        labels=[x for x in ap['qa01_event_labels'] if x['config_id']==cfg]
        assert len(labels)==len({x['event_id'] for x in labels})==42
        groups=[x for x in ap['qa01_types'] if x['config_id']==cfg]
        assert sum(x['planned'] for x in groups)==42
        normal=[x for x in groups if x['profile']=='NORMAL']
        assert sum(x['planned'] for x in normal)==36
        assert math.isclose(sum(x['mean_s']*x['analytical_onsets'] for x in normal)/36,s['qa01'])
        refs=[x for x in ap['qa02_expressions'] if x['config_id']==cfg]
        assert len(refs)==len({x['expression_key'] for x in refs})==33
        bykey={x['expression_key']:x for x in refs}
        assert 'SELECTION' not in bykey['U13:ref1:r1']['tags']
        assert 'PROVENANCE_LOOKUP' in bykey['U13:ref2:r1']['tags']
        assert 'SELECTION' in bykey['U21:ref1:r1']['tags']
        assert 'SELECTION' not in bykey['U21:ref2:r1']['tags']
        assert 'CORRECTION' not in bykey['U24:ref1:r1']['tags']
        assert 'CORRECTION' in bykey['U24:ref2:r1']['tags']
        for tag in [x for x in ap['qa02_tags'] if x['config_id']==cfg]:
            assert tag['planned']==sum(tag['tag'] in x['tags'] for x in refs)
            assert tag['correct'] is tag['ratio'] is None and tag['observed']==0
        buckets=[x for x in ap['qa05_buckets'] if x['config_id']==cfg]
        assert math.isclose(sum(x['usd'] for x in buckets),s['qa05'])
        cases=[x for x in d['cases'] if x['config_id']==cfg]
        assert sum(x['items'] for x in buckets)==sum(len(c['usage']) for c in cases)
        for qa in ['qa06','qa08']:
            peak=next(x for x in ap['memory_peaks'] if x['config_id']==cfg and x['qa']==qa)
            assert peak['value_mb']==max(c[qa] for c in cases)==s[qa]
    unused=sum(u['usd'] for c in d['cases'] if c['config_id']=='DP5-B' for u in c['usage'] if u['node']=='relation_unused')
    assert math.isclose(unused,.0114)
    assert sum(x['usd'] for x in ap['qa05_buckets'] if x['config_id']=='DP5-B' and x['phase']=='CANCELLED_OR_UNUSED')>=unused
    # Software evidence-gate probes; no candidate observations are created.
    probes=[]
    reg={'python:app.model':dict(language='Python',source='app/model.py',public_boundary='request',state_owner='session',asset_ownership='schema',dependencies=[],baseline_commit='test-base')}
    r=dict(baseline_commit='test-base',baseline_verification='test',fixture_ref='test',registry_ref='test',acceptance_ref='test',regression_ref='test',completion_status='PASS',module_changes=[],already_supported_evidence='test')
    def expect(label,expected,record=None,registry=None):
        assert m.evaluate_change(r if record is None else record,reg if registry is None else registry)==expected,label
        probes.append(label)
    expect('complete_zero_proof',0)
    expect('empty_registry_never_zero',None,registry={})
    for field in ['dependencies','baseline_commit']:
        bad=copy.deepcopy(reg);bad['python:app.model'].pop(field);expect('missing_'+field,None,registry=bad)
    bad=copy.deepcopy(reg);bad['python:app.model']['baseline_commit']='other';expect('registry_baseline_mismatch',None,registry=bad)
    expect('unproved_zero',None,record={**r,'already_supported_evidence':None})
    changed={**r,'module_changes':[dict(module_id='python:app.model',reason='wire schema',diff_ref='test',operation='MODIFY')]}
    expect('missing_patch',None,record=changed)
    changed['patch_ref']='test';expect('real_module_once',1,record=changed)
    changed['module_changes']*=2;expect('repeated_module_not_double_counted',1,record=changed)
    expect('failed_acceptance_not_zero',None,record={**r,'completion_status':'FAIL'})
    records=[{**copy.deepcopy(changed),'task_id':f'Q4-C{i:02}'} for i in range(1,9)]
    assert m.aggregate_changes(records,reg)==dict(value=1,score=6,status='IMPLEMENTED_CHANGE_REVIEW');probes.append('eight_completed')
    assert m.aggregate_changes(records[:-1],reg)['value'] is None;probes.append('missing_task_no_mean')
    records[-1]['baseline_commit']='other';assert m.aggregate_changes(records,reg)['value'] is None;probes.append('nonindependent_baseline_no_mean')
    for x,score in [(0,6),(1,6),(1.0001,5),(1.4,5),(1.8,4),(2.2,3),(2.6,2),(3,1),(3.0001,0)]:assert m.score_qa04(x)==score
    probes.append('all_score_boundaries')
    with tempfile.TemporaryDirectory(prefix='via-qa7-') as tmp:
        with contextlib.redirect_stdout(io.StringIO()):m.build(Path(tmp))
        for name in ['artifacts/ledger.json','artifacts/summary.json','artifacts/appendices.json','artifacts/comparison.csv','qa04-change-manifest.json','02-qa04-changes.md']:
            assert (Path(tmp)/name).read_bytes()==(m.OUT/name).read_bytes(),name
    result=dict(status='PASS',parent_v6='BYTE_PRESERVED',qa04_tasks=8,qa04_records=56,qa04_measured=0,
                qa01_events=294,qa02_unique_expressions_per_config=33,evidence_gate_probes=probes,
                unchanged_time_cost_memory='PASS',appendix_reconciliation='PASS',reproduction='PASS',actual_executions=0)
    m.save(m.OUT/'artifacts/validation.json',result);print(json.dumps(result,ensure_ascii=False))


if __name__=='__main__':main()
