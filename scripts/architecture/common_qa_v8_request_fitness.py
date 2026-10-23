"""Projection and evidence gates for the proposed QA07 32-case protocol.

No fixture execution or candidate output is produced by this module.
"""
import re
from collections import Counter

STAGES=['REQUEST_INTERPRETATION','RESPONSE_COMPOSITION','ACTUAL_DELIVERY']


def parse_cases(text):
    cases=[];group=None
    for line in text.splitlines():
        if line.startswith('### '):group=line[4:]
        if not re.match(r'^\| Q7-C\d\d \|',line):continue
        fields=[s.strip() for s in line.strip('|').split('|')]
        assert len(fields)==4 and group
        case_id,inputs,required,forbidden=fields
        cases.append(dict(case_id=case_id,type=group,input_and_events=inputs,required=required,forbidden=forbidden,
                          modality='TEXT' if case_id=='Q7-C29' else 'VOICE',fixture_ready=False,
                          fixture_hashes=None,common_invalid_evidence=None,
                          obligations=[dict(id=f'{case_id}:{stage}',stage=stage,requirement=required,forbidden=forbidden,
                                            coverage='ALL_FIRST_ADOPTIONS_AND_REVISIONS' if stage==STAGES[0] else 'ALL_ADOPTED_RESPONSES' if stage==STAGES[1] else 'ALL_REQUIRED_CHANNELS_AND_POST_WINDOW') for stage in STAGES]))
    assert len(cases)==32 and len({c['case_id'] for c in cases})==32
    assert sorted(Counter(c['type'] for c in cases).values())==[4]*8
    return cases


def score(q):
    if q is None:return None
    assert 0<=q<=100
    if q==100:return 6
    if q<90:return 0
    return 1+int((q-90)//2)


def judged(j):
    verdict=j.get('verdict');evaluators=j.get('evaluators',[])
    if verdict not in ['PASS','FAIL'] or not j.get('evidence_ref') or len(evaluators)!=2:return 'UNKNOWN'
    if not all(e.get('evaluator_ref') and e.get('verdict') in ['PASS','FAIL'] for e in evaluators):return 'UNKNOWN'
    if evaluators[0]['evaluator_ref']==evaluators[1]['evaluator_ref']:return 'UNKNOWN'
    opinions={e['verdict'] for e in evaluators}
    if opinions=={verdict} or (len(opinions)==2 and j.get('adjudication_ref')):return verdict
    return 'UNKNOWN'


def evaluate(case,record):
    if case.get('common_invalid_evidence'):return 'INVALID'
    judgments=record.get('judgments',[])
    expected={o['id'] for o in case['obligations']}
    if any(j.get('obligation_id') in expected and judged(j)=='FAIL' for j in judgments):return 'FAIL'
    latest={j.get('obligation_id'):j for j in judgments}
    if any(judged(latest.get(key,{}))!='PASS' for key in expected):return 'UNKNOWN'
    required=['source_execution_key','observation_closed_evidence','protocol_evidence_ref','first_adoption_trace_ref']
    if not case.get('fixture_ready') or not case.get('fixture_hashes') or any(not record.get(k) for k in required):return 'UNKNOWN'
    return 'PASS'


def aggregate(cases,records,frozen=False):
    byid={c['case_id']:c for c in cases};keys=[(r.get('case_id'),r.get('repeat_id')) for r in records]
    expected={(c['case_id'],i) for c in cases for i in range(1,6)}
    executions=Counter(r['source_execution_key'] for r in records if r.get('source_execution_key'))
    statuses=[]
    for r in records:
        status=evaluate(byid[r['case_id']],r)
        if status=='PASS' and executions[r['source_execution_key']]!=1:status='UNKNOWN'
        statuses.append(status)
    counts={k:statuses.count(k) for k in ['PASS','FAIL','UNKNOWN','INVALID']}
    complete=(len(records)==160 and len(keys)==len(set(keys)) and set(keys)==expected
              and len({r.get('config_id') for r in records})==1 and not any(n>1 for n in executions.values()))
    ready=frozen and complete and not counts['INVALID'] and all(c['fixture_ready'] and c['fixture_hashes'] for c in cases)
    n=160 if ready else None
    q=100*counts['PASS']/160 if ready and not counts['UNKNOWN'] else None
    bounds=[100*counts['PASS']/160,100*(counts['PASS']+counts['UNKNOWN'])/160] if ready else None
    return dict(**counts,planned_N=160,N=n,Q=q,score=score(q),bounds=bounds,actual_executions=sum(bool(r.get('source_execution_key')) for r in records)),statuses


def project(text,configs):
    cases=parse_cases(text)
    manifest=dict(version='QA07-CASES-v3',definition='QA07-REQUEST-RESPONSE-v3',score_version='QA07-REQUEST-RESPONSE-score-v3',
                  status='AUTHOR_PROPOSAL_NOT_EXECUTED',user_adopted=False,execution_frozen=False,
                  target=90,target_status='AUTHOR_PROPOSAL',repeat_count=5,planned_denominator=160,
                  score_bands=[[0,90,0],[90,92,1],[92,94,2],[94,96,3],[96,98,4],[98,100,5],[100,100,6]],
                  score_interval_rule='Lower inclusive; upper exclusive except 100 singleton',
                  obligation_deadline_s=60,case_deadline_s=300,post_observation_s=10,stale_replay_offset_s=5,
                  turn_open_after_control_s=10,source_contract=text,cases=cases)
    records=[];summaries={}
    for cfg in configs:
        cfg_records=[dict(config_id=cfg['id'],case_id=c['case_id'],repeat_id=i,source_execution_key=None,
                          observation_closed_evidence=None,protocol_evidence_ref=None,first_adoption_trace_ref=None,
                          judgments=[],error_causes=[],status='UNKNOWN') for c in cases for i in range(1,6)]
        summaries[cfg['id']],_=aggregate(cases,cfg_records)
        records+=cfg_records
    return dict(manifest=manifest,records=records,summary=summaries)
