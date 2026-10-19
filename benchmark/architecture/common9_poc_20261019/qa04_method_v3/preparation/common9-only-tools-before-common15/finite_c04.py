"""Finite C04 live-protocol checks with deterministic provider-free responses."""
import argparse
import copy
import hashlib
import json
import queue
import time
import traceback
from pathlib import Path
import driver

CFG={'id':'AAAA','evidence':'A','execution':'A','grounding':'A','handoff':'A'}
BODY='## Summary\nOld actual summary\n## Empty\n##  Details   \nKeep exact details'


def drain(s,seconds=.08):
    end=time.monotonic()+seconds
    while time.monotonic()<end:
        try:m=s.q.get(timeout=.01)
        except queue.Empty:continue
        if m.get('type')=='model_call':s.reply(m)


def setup(crate,window=60,cfg=CFG,body=BODY,hold_agent=False):
    initial=driver.initial_feature()
    initial['sources'].append({'id':'SAFE-QA','revision':1,'text':'permitted auxiliary remains','local_allowed':True,'external_allowed':True,'extents':{'all':[1,1]}})
    s=driver.Session(crate,cfg,initial,window_seconds=window);s.kind('EPISODE_READY')
    s.input({'id':'request-agent-result','kind':'user','text':'F-QA를 바탕으로 발표자료 파일을 만들어줘.'})
    ingress=driver.harness.wire_data(s.kind('AGENT_INGRESS')['data'],'request')
    receipt=driver.harness.wire_data(s.kind('RECEIVER_RESULT_AVAILABLE')['data'],'receipt');assert receipt['accepted'] is True
    outcome=driver.harness.actual_external({'id':'actual-agent-feature-result','kind':'agent','event_class':'RESULT'},ingress,receipt)
    outcome['text']=body;outcome['result_source']['text']=body
    if hold_agent:
        old=copy.deepcopy(outcome['result_source']);old['text']=body.replace('Old actual summary','Previous same-ID summary');source(s,old)
        s.controls.hold_publication=lambda m:m['data'].get('phase')=='AGENT_RESULT'
    s.input(outcome)
    displayed=s.pump(lambda m:m.get('kind')=='DISPLAY_PROXY'and m['data'].get('stimulus_id')==outcome['id']and m['data'].get('phase')=='AGENT_RESULT')['data']
    if not hold_agent:
        s.pump(lambda m:m.get('kind')=='RESPONSE_FINAL'and m['data'].get('stimulus_id')==outcome['id'])
        s.pump(lambda m:m.get('kind')=='STATE_EFFECT_COMMIT'and m['data'].get('effect_kind')=='SOURCE_REGISTER'and m['data'].get('data',{}).get('stimulus_id')==outcome['id'])
        drain(s)
    edit={'kind':'qa04_section_update','id':'edit-1','target_event_id':outcome['id'],'task_id':displayed['task_id'],'section':'Summary','text':'Revised actual summary','expected_section_revision':1}
    return s,outcome,displayed,edit


def section(s,event,kind='QA04_SECTION_UPDATE_APPLIED'):
    s.input(event);m=s.pump(lambda m:m.get('kind')==kind and m['data'].get('event_id')==event['id'])
    s.pump(lambda m:m.get('kind')=='QA04_FEATURE_RESPONSE_FINAL'and m['data'].get('event_id')==event['id'])
    return m['data']


def source(s,value):
    s.input({'kind':'source','source':value})
    s.pump(lambda m:m.get('kind')=='STATE_EFFECT_COMMIT'and m['data'].get('effect_kind')=='SOURCE_REGISTER'and m['data'].get('data',{}).get('source_id')==value['id'])


def subsequent(s,source_id='SAFE-QA',event_id='after-edit'):
    start=len(s.calls);s.input({'id':event_id,'kind':'user','text':source_id+'의 내용을 설명해줘.'})
    s.pump(lambda m:m.get('kind')=='RESPONSE_FINAL'and m['data'].get('stimulus_id')==event_id);drain(s)
    calls=s.calls[start:];meaning=[c for c in calls if c['stage']=='meaning_interpret_and_respond'and c['payload']['input']['event']['id']==event_id]
    assert meaning;return calls,meaning[-1]['payload']


def held_edit(s,edit):
    s.controls.hold_publication=lambda m:m['data'].get('section_edit_event_id')==edit['id']
    s.input(edit);s.pump(lambda m:m.get('kind')=='DISPLAY_PROXY'and m['data'].get('section_edit_event_id')==edit['id'])
    assert s.controls.pending_receipts;return copy.deepcopy(s.controls.pending_receipts[-1])


def finish_trace(s,folder,allow_closed=False):
    trace=s.finish(allow_closed=allow_closed);driver.dump(folder/'trace.json',trace);return trace


def case(crate,root,name):
    folder=root/name;folder.mkdir(exist_ok=False);s=None;checks=[]
    try:
        cfg=copy.deepcopy(CFG)
        if name=='revocation_evidence_preparation':cfg.update(id='CAAA',evidence='C')
        if name=='revocation_handoff_preparation':cfg.update(id='AAAB',handoff='B')
        s,outcome,displayed,edit=setup(crate,window=1 if name=='cutoff_during_ack'else 60,cfg=cfg,body='## Summary\nfirst\n## Summary\nsecond'if name=='duplicate_section_names'else BODY,hold_agent=name=='early_registration_pending')
        if name=='early_registration_pending':
            section(s,edit,'QA04_SECTION_UPDATE_REJECTED')
            assert not any(m.get('kind')=='DISPLAY_PROXY'and m['data'].get('section_edit_event_id')==edit['id']for m in s.log)
            s.controls.submit_receipt(s.controls.pending_receipts[0]);s.pump(lambda m:m.get('kind')=='RESPONSE_FINAL'and m['data'].get('stimulus_id')==outcome['id'])
            s.pump(lambda m:m.get('kind')=='STATE_EFFECT_COMMIT'and m['data'].get('effect_kind')=='SOURCE_REGISTER'and m['data'].get('data',{}).get('stimulus_id')==outcome['id'])
            checks+=['pending_actual_Agent_receipt_cannot_use_previous_same_ID_source']
        elif name=='duplicate_section_names':
            section(s,edit,'QA04_SECTION_UPDATE_REJECTED')
            assert not any(m.get('kind')=='DISPLAY_PROXY'and m['data'].get('section_edit_event_id')==edit['id']for m in s.log)
            checks+=['duplicate_section_name_rejected_before_display']
        elif name=='sections_and_task_sync':
            applied=section(s,edit);expected=BODY.replace('Old actual summary','Revised actual summary')
            assert applied['after']['text']==expected
            task=next(t for t in applied['task_state_after']if t['task_id']==displayed['task_id'])
            assert task['last_result']==task['result_response']['text']==expected
            empty=section(s,{**edit,'id':'edit-empty','section':'Empty','text':'Fill formerly empty','expected_section_revision':2})
            assert empty['after']['text']==expected.replace('## Empty\n','## Empty\nFill formerly empty\n')
            cleared=section(s,{**edit,'id':'clear-empty','section':'Empty','text':'','expected_section_revision':3})
            assert cleared['after']['text']==expected
            for label,value in [('missing',{**edit,'id':'missing','section':'Absent','expected_section_revision':4}),('replay',edit),('bad-preparation',{**edit,'id':'bad-source-prep','text':{'not':'text'},'expected_section_revision':4})]:section(s,value,'QA04_SECTION_UPDATE_REJECTED');checks.append(label)
            calls,payload=subsequent(s,displayed['delivery_source_id']);serialized=json.dumps(payload,ensure_ascii=False)
            assert 'Revised actual summary'in serialized and 'Keep exact details'in serialized and '##  Details   'in serialized
            task=next(t for t in payload['input']['public_state']['tasks']if t['task_id']==displayed['task_id']);assert task['last_result']==task['result_response']['text']==expected
            checks+=['outside_selected_body_exact','empty_named_section_preserved_and_editable','empty_replacement_supported','heading_whitespace_preserved','Task_both_results_and_subsequent_source_agree']
        elif name=='pre_admission_divergence':
            updated=copy.deepcopy(outcome['result_source']);updated['revision']=2;updated['text']=BODY.replace('Keep exact details','NEWER Details retained');source(s,updated)
            section(s,edit,'QA04_SECTION_UPDATE_REJECTED')
            assert not any(m.get('kind')=='DISPLAY_PROXY'and m['data'].get('section_edit_event_id')==edit['id']for m in s.log)
            _,payload=subsequent(s,displayed['delivery_source_id']);raw=payload['input']['actually_supplied_originals'];assert any(p['revision']==2 and 'NEWER Details retained'in p['text']for p in raw if p['source_id']==displayed['delivery_source_id'])
            checks+=['no_edit_display_before_current_source_correspondence','newer_pointer_and_Details_preserved']
        elif name.startswith('revocation'):
            section(s,edit);drain(s);at=len(s.calls)
            revoked=copy.deepcopy(driver.initial_feature()['sources'][0]);revoked['revoked']=True;source(s,revoked)
            calls,payload=subsequent(s)
            examined=s.calls[at:]
            # Complete subsequent payloads include preparation and meaning roles.
            assert any(c['stage']=='meaning_interpret_and_respond'for c in examined)
            required='evidence_publish_relations'if cfg['evidence']=='C'else'handoff_publish_work_record'
            assert any(c['stage']==required for c in examined),[c['stage']for c in examined]
            for c in examined:
                public=c.get('payload',{}).get('input',{}).get('public_state',{})
                for task in public.get('tasks',[]):
                    if task['task_id']==displayed['task_id']:assert 'result_response'not in task and 'last_result'not in task
                serialized=json.dumps(c.get('payload',{}),ensure_ascii=False)
                assert 'Revised actual summary'not in serialized and 'Old actual summary'not in serialized and 'PRIVATE MIDDLE'not in serialized,c['stage']
            assert any(t['task_id']=='TB'and t['title']=='Other actual Task'for t in payload['input']['public_state']['tasks'])
            assert 'permitted auxiliary remains'in json.dumps(payload)
            checks+=['full_meaning_and_preparation_payloads_redacted','unrelated_Task_and_permitted_source_retained']
        else:
            receipt=held_edit(s,edit)
            if name=='source_during_ack':
                updated=copy.deepcopy(outcome['result_source']);updated['revision']=2;updated['text']=BODY.replace('Keep exact details','Concurrent newer Details');source(s,updated)
            elif name=='authority_during_ack':
                revoked=copy.deepcopy(driver.initial_feature()['sources'][0]);revoked['revoked']=True;source(s,revoked)
            elif name=='target_during_ack':
                other={**edit,'id':'concurrent-other-section','section':'Details','text':'Concurrent Details preserved'};section(s,other)
            elif name=='unrelated_task_during_ack':
                s.input({'id':'unrelated-progress','kind':'agent','event_class':'PROGRESS','task_id':'TB','text':'Unrelated progress retained'})
                s.pump(lambda m:m.get('kind')=='RESPONSE_FINAL'and m['data'].get('stimulus_id')=='unrelated-progress')
            elif name=='mismatched_and_duplicate_ack':
                wrong={**receipt,'display_event_id':'wrong-actual-display'};s.controls.submit_receipt(wrong);s.kind('PUBLICATION_RECEIPT_REJECTED')
            elif name=='cancel_during_ack':s.controls.cancel('QA04_NEGATIVE_CANCEL_DURING_ACK')
            elif name=='cutoff_during_ack':
                while time.monotonic_ns()<=s.controls.deadline_ns:time.sleep(.005)
            elif name=='missing_ack':
                rejected=s.kind('QA04_SECTION_UPDATE_REJECTED')['data'];assert 'RECEIPT'in rejected['error'];checks.append('missing_ACK_no_authoritative_edit')
            if name!='missing_ack':
                s.controls.submit_receipt(receipt)
                rejected=name in ['source_during_ack','authority_during_ack','target_during_ack','cancel_during_ack','cutoff_during_ack']
                result=s.kind('QA04_SECTION_UPDATE_REJECTED'if rejected else'QA04_SECTION_UPDATE_APPLIED')['data']
                if name in ['source_during_ack','authority_during_ack','target_during_ack']:assert result['historical_display_recorded']is True and result['history']['applied']is False
                if name=='source_during_ack':
                    _,payload=subsequent(s,displayed['delivery_source_id']);raw=payload['input']['actually_supplied_originals']
                    assert any(p['revision']==2 and 'Concurrent newer Details'in p['text']for p in raw if p['source_id']==displayed['delivery_source_id'])
                if name=='target_during_ack':
                    _,payload=subsequent(s,displayed['delivery_source_id']);serialized=json.dumps(payload,ensure_ascii=False)
                    assert 'Concurrent Details preserved'in serialized and 'Revised actual summary'not in serialized
                if name=='authority_during_ack':
                    _,payload=subsequent(s);task=next(t for t in payload['input']['public_state']['tasks']if t['task_id']==displayed['task_id'])
                    assert 'result_response'not in task and 'last_result'not in task
                if name=='unrelated_task_during_ack':assert any(t['task_id']=='TB'and t.get('last_progress')=='Unrelated progress retained'for t in result['task_state_after'])
                if name=='mismatched_and_duplicate_ack':
                    s.controls.submit_receipt(receipt);s.kind('PUBLICATION_RECEIPT_REJECTED');section(s,edit,'QA04_SECTION_UPDATE_REJECTED')
                checks.append(name)
        trace=finish_trace(s,folder,allow_closed=name in ['cutoff_during_ack','cancel_during_ack'])
        result=driver.events(trace,'EPISODE_RESULT')[-1];history=result['qa04_section_history']
        if name in ['missing_ack','cancel_during_ack','cutoff_during_ack','pre_admission_divergence','early_registration_pending','duplicate_section_names']:assert not history
        if name in ['source_during_ack','authority_during_ack']:assert len(history)==1 and history[0]['applied']is False
        if name=='mismatched_and_duplicate_ack':assert len(history)==1
        return {'name':name,'status':'PASS','checks':checks,'trace':str(folder/'trace.json'),'provider_calls':0}
    except Exception as error:
        if s:
            s.close();driver.dump(folder/'failed-trace.json',{'events':s.log,'calls':s.calls,**s.controls.snapshot(),'returncode':s.proc.returncode})
        driver.dump(folder/'failure.json',{'error':repr(error),'traceback':traceback.format_exc(),'provider_calls':0});raise
    finally:
        if s:s.close()


def main():
    p=argparse.ArgumentParser();p.add_argument('--crate',type=Path,required=True);p.add_argument('--benchmark',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--case');a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
    names=['sections_and_task_sync','duplicate_section_names','early_registration_pending','pre_admission_divergence','revocation_evidence_preparation','revocation_handoff_preparation','source_during_ack','authority_during_ack','target_during_ack','unrelated_task_during_ack','mismatched_and_duplicate_ack','cancel_during_ack','cutoff_during_ack','missing_ack']
    if a.case:names=[a.case]
    rows=[]
    try:
        for name in names:rows.append(case(a.crate,a.output,name));print(name+' PASS',flush=True)
    finally:driver.dump(a.output/'summary.json',{'status':'PASS'if len(rows)==len(names)else'FAIL_PRESERVED','rows':rows,'planned':names,'provider_calls':0,'source_sha256':{str(f.relative_to(a.crate)):hashlib.sha256(f.read_bytes()).hexdigest()for f in(a.crate/'src').rglob('*.rs')},'tool_sha256':{f.name:hashlib.sha256(f.read_bytes()).hexdigest()for f in Path(__file__).parent.glob('*.py')},'scope':'Finite copied-candidate code/protocol acceptance; not primary model evidence or full42 QA04 measurement'})


if __name__=='__main__':main()
