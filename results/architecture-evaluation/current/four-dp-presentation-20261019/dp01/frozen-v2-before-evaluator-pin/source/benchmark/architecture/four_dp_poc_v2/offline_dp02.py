"""Deterministic pipeline double; never reported as Qwen accuracy or latency."""
from collections import defaultdict
_seen=defaultdict(int)
def respond(payload,meta):
    x=payload['input'];stage=meta['stage'];sources=x.get('sources',[])
    if 'adopt_intents' in stage or 'repair_intents' in stage:
        e=x['event'];text=e['text'];low=text.lower();history=x['conversation'];base={'target_request_id':None,'question_id':None,'task_id':None,'source_revisions':{'D1':1}}
        def i(relation,scope,targets,handling,answer=None,**kw):return {**base,'relation':relation,'scope':scope,'targets':targets,'handling_text':handling,'complete_answer':answer,**kw}
        if 'send it to kim' in low:
            q=history['actually_displayed_questions'][0]
            return {'intents':[i('ANSWER','Deliver the recipient answer to the status email Task',['Kim'],'I will forward Kim, the deputy manager, as the recipient for the status email.', 'The status email recipient is Kim, the deputy manager.',question_id=q['id'],task_id=q['task_id'],source_revisions={})]}
        if 'change the earlier' in low:
            target=next(e['id'] for e in history['captured_inputs'] if e['id']!=x['event']['id'])
            return {'intents':[i('CORRECTION','Compare Plan A and Plan B on cost only',['Plan A','Plan B'],'I will revise the Plan A and Plan B comparison to cost only.',target_request_id=target)]}
        if 'separately explain' in low:
            target=next(e['id'] for e in history['captured_inputs'] if e['id']!=x['event']['id'])
            return {'intents':[i('CONTINUE','Continue the Plan A and Plan B comparison',['Plan A','Plan B'],'I will keep the Plan A and Plan B comparison active.',target_request_id=target),i('NEW','Explain TCP and UDP in two sentences',['TCP','UDP'],'I will answer the separate TCP and UDP question.', 'TCP provides reliable ordered delivery with connection management. UDP sends independent datagrams without delivery or ordering guarantees.',source_revisions={})]}
        if 'summarize the conclusion' in low:
            target=history['requests'][0]['request_id']
            return {'intents':[i('FOLLOWUP','Summarize the comparison conclusion',['Plan A','Plan B'],'I will summarize the Plan A and Plan B comparison conclusion.', 'Plan A costs more but supports faster recovery and greater capacity, while Plan B reduces cost and RAM.',target_request_id=target)]}
        return {'intents':[i('NEW','Compare Plan A and Plan B cost, recovery and capacity',['Plan A','Plan B'],'I will compare Plan A and Plan B on cost, recovery and capacity using D1 revision 1.')]}
    if 'complete_response' in stage or 'repair_response' in stage:
        key=(meta['run_id'],x['goal_id']);_seen[key]+=1
        if meta.get('offline_probe')=='null-once' and _seen[key]==1:return {'answer':None,'sections':[],'applied_revision':x['current_revision']}
        cost_only='cost only' in x['current_scope'].lower()
        text='Plan A costs $120 per month and Plan B $75; Plan B saves $45 per month.'
        if not cost_only:text+=' Plan A checkpoints every 10 seconds and supports 40 concurrent requests, versus 30 seconds and 24 for Plan B. Plan A uses 800 MB versus 500 MB, so capacity/recovery trade off against price and RAM.'
        quote='cost 120 dollars per month'
        return {'answer':text,'sections':[{'id':'cost','text':'Plan A $120; Plan B $75.'}]+([] if cost_only else [{'id':'recovery-capacity','text':'Plan A 10 seconds/40 requests; Plan B 30 seconds/24 requests.'}]),'applied_revision':x['current_revision'],'source_claims':[{'source_id':'D1','revision':1,'quote':quote}]}
    raise ValueError('Unknown DP02 double stage '+stage)
