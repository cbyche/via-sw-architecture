"""Finite pipeline double only. Its timings/tokens are never model measurements."""
def respond(payload,meta):
 x=payload['input'];stage=meta['stage'];source=x['sources'][0];sid=source['id'];rev=source['revision'];quote='Cedar costs 96 dollars monthly'
 if 'intents' in stage:
  e=x['event'];h=x['conversation'];text=e['text'];base={'target_request_id':None,'question_id':None,'task_id':None,'source_revisions':{sid:rev},'source_claims':[{'source_id':sid,'revision':rev,'quote':quote}]}
  def intent(relation,scope,answer=None,**kwargs):return {**base,'relation':relation,'scope':scope,'targets':['Cedar','Birch'],'handling_text':'I will answer the adopted Cedar and Birch cost comparison.','complete_answer':answer,**kwargs}
  if e['id']=='R1':return {'intents':[intent('NEW',text,'Cedar costs 96 dollars monthly; Birch costs 64 dollars monthly.' if 'one sentence' in text else None)]}
  if 'Change my earlier' in text:return {'intents':[intent('CORRECTION','Monthly costs and difference only',target_request_id='R1')]}
  return {'intents':[intent('CONTINUE','Keep the document request active',target_request_id='R1'),intent('NEW','Explain processes and threads','A process owns its own address space and resources. Threads share a process address space and have their own execution stacks.',source_revisions={},source_claims=[],targets=['process','thread'])]}
 answer='Cedar costs 96 dollars monthly; Birch costs 64 dollars monthly, a difference of 32 dollars.'
 if 'difference only' not in x['current_scope']:answer+=' Cedar checkpoints every 12 seconds with 36 concurrent requests and 640 MB RAM; Birch uses 28 seconds, 20 requests and 384 MB. Only Cedar supports a 30-request load within capacity.'
 return {'answer':answer,'sections':[{'id':'cost','text':answer}],'applied_revision':x['current_revision'],'source_claims':[{'source_id':sid,'revision':rev,'quote':quote}]}
