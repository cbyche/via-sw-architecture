#!/usr/bin/env python3
"""Documentation-example consistency, not a scheduler, resolver or model test."""
import argparse
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
CONTRACTS=ROOT/'docs/architecture/12-decisions/decision-packages/contracts'

def check44():
    d=json.loads((CONTRACTS/'dp44-execution-examples.json').read_text())
    fields={'event_id','kind','conversation_ref','source_ref','source_revision','input_generation','payload'}
    kinds={'INPUT_READY','TASK_NOTICE','JOB_COMPLETED','DELIVERY_RECEIPT'}
    seen=set()
    for e in d['events']:
        assert set(e)==fields and e['kind'] in kinds
        assert e['event_id'] not in seen;seen.add(e['event_id'])
        assert type(e['source_revision']) is int and e['source_revision']>=1
        assert e['input_generation'] is None or (type(e['input_generation']) is int and e['input_generation']>=1)
        assert e['conversation_ref']=='Conversation1' and isinstance(e['payload'],dict)
    assert d['a_continuation']['next']=='PREPARE_RESPONSE'
    assert d['a_continuation']['input_generation']==d['b_publication_window']['input_generation']==2
    assert 'AdoptedMeaning1' in d['b_publication_window']['required_refs']
    assert d['b_clarification']['requires_input_settled'] is False
    stale=next(e for e in d['events'] if e['event_id']=='Event3')
    assert stale['input_generation']<d['a_continuation']['input_generation']
    assert d['dispositions']['Event3']=='DISCARD_SUPERSEDED'
    assert d['dispositions']['Event4']=='UPDATE_ACTUAL_DELIVERY_NOT_TASK_CANCEL'
    print('PASS: 44 fixed envelope, continuation/window, stale example and clarification admission')

def main():
    p=argparse.ArgumentParser();p.add_argument('--dp',type=int,choices=[44],default=44);p.parse_args()
    check44()
    print('Documentation examples only: no executor, model call or measured QA result.')
if __name__=='__main__':main()
