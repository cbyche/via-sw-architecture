"""Actual Qwen validation of a changed M04 source, outside P21 timing/cost."""
from __future__ import annotations

import argparse
import asyncio
import hashlib
import json
from pathlib import Path
import sys


def source_manifest(source):
    files={str(p.relative_to(source)):hashlib.sha256(p.read_bytes()).hexdigest()
           for p in sorted(source.rglob('*')) if p.is_file() and '__pycache__' not in p.parts and p.suffix!='.pyc'}
    return {'files':files,'digest':hashlib.sha256(json.dumps(files,sort_keys=True,separators=(',',':')).encode()).hexdigest()}


async def run(args):
    source=args.source.resolve();sys.path.insert(0,str(source))
    from prototypes.candidates.dp41_45_poc.shared.evidence import Evidence
    from prototypes.candidates.dp41_45_poc.shared.providers import ModelClient
    from prototypes.candidates.dp41_45_poc.shared.runtime import LocalVIA
    from prototypes.candidates.dp41_45_poc.dp41 import a,b,contract,prompts
    assert Path(sys.modules[LocalVIA.__module__].__file__).resolve().is_relative_to(source)
    evidence=Evidence(args.output,args.variant,0)
    original_write=evidence.write
    def scoped(table,record):
        return original_write(table,{**record,'scope':'EVALUATION_CHANGE','experiment':'M04_NEW_NATURAL_LANGUAGE'})
    evidence.write=scoped
    model=ModelClient(evidence,'qwen',max_cost_usd=2,budget_file=args.budget_file)
    before=source_manifest(source)
    imported={m.__name__:str(Path(m.__file__).resolve()) for m in [a,b,contract,prompts,sys.modules[LocalVIA.__module__],sys.modules[ModelClient.__module__]]}
    if not all(Path(p).is_relative_to(source) for p in imported.values()):raise ValueError('Probe import escaped changed source')
    evidence.write('probe-provenance',{'source':str(source),'source_manifest':before,'imported_paths':imported})
    initial={'sources':[{'source_id':'D1','revision':1,'kind':'DOCUMENT','title':'D1','text':'실패 실험용 실제 원문입니다.',
        'permission':{'read':'ALLOWED','model':'ALLOWED','agent':'ALLOWED'},'metadata':{}},
        {'source_id':'민수','revision':1,'kind':'CONTACT','title':'민수','text':'minsu@example.invalid',
        'permission':{'read':'ALLOWED','model':'ALLOWED','agent':'ALLOWED'},'metadata':{}}], 'tasks':[], 'voice':{'on':False}}
    runtime=LocalVIA(initial,model,evidence,args.variant,scene_key='M04_NEW_NATURAL_LANGUAGE',
        stub_profile={'acceptance_receipt_seconds':.2,'bounded_result_seconds':.2,'final_result_seconds':8})
    transcript='D1을 보고서로 만들어줘. 보고서 작성이 실패하면 실패 안내 메일을 초안으로만 만들어줘. 발송하지 마.'
    evidence.write('probe-input',{'transcript':transcript,'initial':initial,'future_failure_hidden_from_candidate':True})
    async def fail_actual_producer():
        for _ in range(5500):
            report=next((e for e in runtime.effects if e['operation']=='CREATE_REPORT' and e.get('receipt_s') is not None),None)
            if report:
                event={'event_id':'M04:actual-report-failure','type':'FAILURE','payload':{
                    'task_id':report['task_id'],'execution_id':report['execution_id'],
                    'request_revision':report['request_revision'],'reason':'FIXED_EXTERNAL_FAILURE'}}
                evidence.write('probe-source',{'event':event})
                await runtime.external_event(event)
                return report
            await asyncio.sleep(.02)
        return None
    failure=asyncio.create_task(fail_actual_producer())
    error=None;report=None
    try:
        try:
            async with asyncio.timeout(120):
                await runtime.input_started('M04:input')
                await runtime.speech_end('M04:input')
                await runtime.resolve(transcript,request_id='M04:request',input_id='M04:input')
                report=await failure
                await asyncio.wait_for(runtime.drain(),60)
        except Exception as exception:
            error=type(exception).__name__+': '+str(exception)
        draft=[e for e in runtime.effects if e['operation']=='DRAFT_MAIL']
        sent=[e for e in runtime.effects if e['operation']=='SEND_MAIL']
        failed=next((e for e in evidence.records.get('events',[]) if e['kind']=='TASK_FAILED'
                     and report and e['task']['task_id']==report['task_id']
                     and e['task']['execution_id']==report['execution_id']
                     and e['task']['request_revision']==report['request_revision']),None)
        meanings=[r.get('meaning') for r in runtime.requests.values() if r.get('meaning')]
        relations=[r for m in meanings for r in m.get('relations',[])]
        failure_branch=bool(report and len(draft)==1 and any(r.get('op')=='WHEN'
            and r.get('from_goal')==report['goal_id'] and r.get('to_goal')==draft[0]['goal_id']
            and r.get('predicate')=={'producer_goal_id':report['goal_id'],'field':'STATE','comparison':'EQ','value':'FAILED'}
            for r in relations))
        grounded=bool(report and any(s.get('source_id')=='D1' and s.get('revision')==1
            and s.get('text')==initial['sources'][0]['text'] for s in report['payload']['sources']))
        checks={'actual_report_failed':report is not None and failed is not None,
                'draft_once_after_actual_failure':len(draft)==1 and failed is not None and draft[0]['send_s']>=failed['scene_seconds'],
                'explicit_failed_producer_branch':failure_branch,'actual_d1_report_source':grounded,
                'send_zero':not sent,'watchdog_and_runtime_ok':error is None,
                'source_unchanged':source_manifest(source)==before}
        semantic=None
        if all(checks.values()):
            try:
                judge=ModelClient(evidence,'openai',max_cost_usd=1,budget_file=args.judge_budget_file)
                semantic=await judge.call([{'role':'system','content':
                    'Judge the supplied actual adopted Meaning and received command payloads against this user input. '
                    'Require a report from the actual D1 source, an explicit report FAILED condition before a failure-notice '
                    'email draft, and preserved no-send intent. The downstream agent writing is a stub; judge the actual '
                    'delegated instruction, not a fabricated authored email. Do not repair anything. Return only '
                    'JSON {"passed":boolean,"reason":string}. A generic report email without failure-notice intent fails.'},
                    {'role':'user','content':json.dumps({'input':transcript,'meanings':meanings,
                        'received_effects':runtime.effects,'failed_event':failed},ensure_ascii=False)}],
                    'CHANGE_JUDGE',request_id='M04:semantic-judge',max_tokens=3000,purpose='EVALUATION_CHANGE',
                    reasoning_effort='high',enforce_request_budget=False)
                checks['actual_failure_notice_draft_semantics']=type(semantic.get('passed')) is bool and semantic['passed']
                evidence.write('change-semantic-judge',semantic)
            except Exception as exception:
                error=type(exception).__name__+': '+str(exception)
                checks['actual_failure_notice_draft_semantics']=False
        else:
            checks['actual_failure_notice_draft_semantics']=False
        record={'variant':args.variant,'status':'PASS' if all(checks.values()) else 'FAIL',
            'checks':checks,'effects':runtime.effects,'meanings':meanings,'semantic_judgement':semantic,'error':error,
            'provider_usage_cost_usd':model.spent,'evidence_level':'MEASURED_MODEL_CHANGE_VALIDATION',
            'excluded_from_p21_qa04':True,'source':str(source),'source_digest':before['digest'],'imported_paths':imported}
        evidence.write('change-semantic-validation',record)
        print(json.dumps({k:record[k] for k in ['variant','status','checks','error','provider_usage_cost_usd']},ensure_ascii=False),flush=True)
    finally:
        failure.cancel()
        await asyncio.gather(failure,return_exceptions=True)
        for job in list(runtime.jobs):job.cancel()
        await asyncio.gather(*list(runtime.jobs),return_exceptions=True)
        evidence.close()


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--source',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True);parser.add_argument('--variant',choices=['A','B'],required=True)
    parser.add_argument('--budget-file',type=Path,required=True)
    parser.add_argument('--judge-budget-file',type=Path,required=True)
    asyncio.run(run(parser.parse_args()))
