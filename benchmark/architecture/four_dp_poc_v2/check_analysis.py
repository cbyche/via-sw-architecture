"""Focused v2 endpoint/population regressions; no provider calls."""
from timing import analyze_timing

def event(kind,at,**data):return {'kind':kind,'elapsed_ns':at,'observed_ns':10_000_000+at,'data':data}
def run(events,pop,calls=None):return analyze_timing({'events':events,'calls':calls or [],'start_ns':10_000_000},pop)
def main():
 p={'qa01_stimuli':['u','correction','notice'],'qa03_goals':[{'goal_id':'g','origin_stimulus_id':'u','final_revision':2}]}
 events=[event('INPUT_CAPTURED',0,stimulus_id='u',input_elapsed_ns=0),event('GOAL_FAILED',10,goal_id='g',reason='initial parse failed'),event('INPUT_CAPTURED',20,stimulus_id='correction',input_elapsed_ns=20),event('GOAL_START',30,goal_id='g',input_elapsed_ns=0),event('DISPLAY_PROXY',40,stimulus_id='correction',text='Revise Plan A and B cost',content_type='ADOPTED_HANDLING'),event('INPUT_CAPTURED',50,stimulus_id='notice'),event('DISPLAY_PROXY',51,stimulus_id='notice',text='Who receives the email?',content_type='TASK_QUESTION'),event('GOAL_COMPLETE',100,goal_id='g',stimulus_id='correction',revision=2,text='Wrong but complete result',content_contract_checked=True)]
 x=run(events,p)
 assert x['goals'][0]['qa03_ms']==.0001,'later adoption must preserve initial input start'
 assert x['qa01_expected']==3 and x['qa03_expected']==1,'notice/correction not duplicate goals'
 assert x['qa01_missing']==1 and x['comparison_status']=='INCOMPLETE','failed/missing input remains denominator'
 assert len(x['goals'][0]['failed_attempts'])==1,'failure terminal retained'
 x=run(events[:-1],p);assert x['qa03_missing']==1 and x['qa03_ms']==[],'no invented failure penalty'
 stale=events[:-1]+[event('GOAL_COMPLETE',60,goal_id='g',revision=1,text='Old revision',content_contract_checked=True)]
 assert run(stale,p)['qa03_missing']==1,'stale earlier completion cannot satisfy final revision'
 # An unrelated USER_WAIT never subtracts from active goal work.
 waits=events+[event('USER_WAIT_START',4,task_id='other'),event('USER_WAIT_END',90,task_id='other')]
 assert run(waits,p)['goals'][0]['qa03_ms']==.0001
 calls=[{'role':'RECEIVER','send_ns':10_000_020,'terminal_ns':10_000_060,'causal_trace':{'goal_ids':['g']}}]
 assert run(events,p,calls)['goals'][0]['qa03_ms']==.00006,'scoped receiver shim interval excluded'
 calls[0]['causal_trace']['goal_ids']=['other']
 assert run(events,p,calls)['goals'][0]['qa03_ms']==.0001,'unrelated receiver never excluded'
 print('9 v2 endpoint/population checks PASS')
if __name__=='__main__':main()
