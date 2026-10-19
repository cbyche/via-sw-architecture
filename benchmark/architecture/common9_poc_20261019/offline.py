"""Pipeline semantic/vision doubles. No model accuracy, latency, or token-cost evidence."""
import ast,copy,json,re,xml.etree.ElementTree as ET
from pathlib import Path
H=Path(__file__).resolve().parent
# Reuse only pure authored screen-double functions, never checker execution or oracle.
_module=ast.parse((H/'grounding_acceptance.py').read_text());_functions=ast.Module(body=[n for n in _module.body if isinstance(n,ast.FunctionDef)and n.name in ['candidates','refs','bind']],type_ignores=[]);exec(compile(_functions,'offline_geometry_double','exec'),globals())
def a(v):return v if isinstance(v,list)else[]
def latest_wait(state):return next((r for r in reversed(a(state.get('requests')))if r.get('status')in ['WAIT','CLARIFY']),None)
def reads(catalog,text):
 out=[]
 for s in catalog:
  if s.get('is_current')is False and'원본'in text and'아까'in text:pass
  elif s.get('is_current')is False:continue
  sid=s['id'];wanted=sid=='F-PRIVATE'and'본문'in text or sid in text or sid=='DOC-A'and any(w in text for w in ['문단','표','부분'])or sid=='H10'and any(w in text for w in ['비교','그 결과'])or sid=='H11'and'일정'in text or sid.endswith('revised-result')and'수정본'in text or':summary:body-only'in sid and'요약'in text or':answer01:delivered'in sid and'문단 설명'in text
  if not wanted:continue
  for p in s.get('allowed_segments',[]):out.append({'source_id':sid,'revision':s['revision'],'start_line':p[0],'end_line':p[1]})
 return out

def respond(payload,meta):
 stage=meta['stage'];x=payload.get('input',{})
 if stage.startswith('grounding.'):
  if stage=='grounding.local_screen_parser':return {'frames':[],'scope':'OFFLINE_GEOMETRY_DOUBLE_NOT_MODEL'}
  if'speech'in stage:return {'references':refs(x['transcript'],x['cues']),'intent':'지정한 범위를 처리하고 유지 조건을 보존','unknowns':[]}
  if'screen'in stage:return {'candidates':candidates(x['frames']),'coverage':{'complete':True,'unknowns':[]}}
  if'joint'in stage:return bind(refs(x['transcript'],x['cues']),candidates(x['frames']),x['cues'],x['frames'],x.get('prior'))
  # A semantic binder double receives only the same public candidates/cues/
  # delivered prior as the real model. No fixture lookup supplies missing cues.
  return bind(x['speech']['references'],copy.deepcopy(x['screen']['candidates']),x.get('cues',[]),[],x.get('prior'))
 if stage in ['evidence_publish_relations','handoff_publish_work_record']:
  evidence=x.get('observed_evidence',x.get('actual_evidence',[]));return {'records':[{'record_id':'source:'+p['source_id'],'interpretation':p['text'],'source_spans':[{k:p[k]for k in ['source_id','revision','start_line','end_line']}]}for p in evidence],'gaps':[],'unknowns':[]}
 if stage=='evidence_select_ranges':return {'reads':reads(x['catalog'],x['event']['text']),'bindings':[],'unresolved':[]}
 if stage.startswith('handoff_'):
  if stage=='handoff_compose_capsule':return {'summary':'实际提供的原文见附录；保留全部当前约束。','unknowns':[]}
  originals=x.get('actually_acquired_originals',[]);return {'mode':'DIRECT','summary':'依据已实际获得的原文执行指定目的。','references':[],'selected_spans':[{k:p[k]for k in ['source_id','revision','start_line','end_line']}for p in originals],'selected_record_ids':[],'reads':[],'unknowns':[]}
 if stage.startswith('receiver_'):
  contract=x.get('contract',{});mandatory=contract.get('mandatory',x.get('mandatory',{}));supplied=x.get('actual_supplied_originals',x.get('actual_evidence',[]));required=mandatory.get('required_source_ids',[])
  if any(sid not in [p['source_id']for p in supplied]for sid in required):
   manifest=contract.get('manifest',[]);return {'mode':'READ','reads':[{'source_id':s['id'],'revision':s['revision'],'start_line':r[0],'end_line':r[1]}for s in manifest if s['id']in required for r in s['allowed_segments']],'result':None}
  facts=[{**{k:p[k]for k in ['source_id','revision','start_line','end_line']},'statement':p['text'],'quote':p['text'].splitlines()[0]}for p in supplied if p['source_id']in required];result={'text':mandatory.get('purpose','')+' '+','.join(mandatory.get('targets',[]))+' '+' '.join(f['statement']for f in facts),'facts':facts,'used_source_ids':required,'source_spans':[{k:f[k]for k in ['source_id','revision','start_line','end_line']}for f in facts],'conditions_respected':mandatory.get('constraints',[]),'actions_performed':[],'unknowns':[]}
  return {'mode':'COMPLETE','result':result,'reads':[]}if stage=='receiver_select_or_complete'else result
 if stage!='meaning_interpret_and_respond':raise ValueError('Unknown offline stage '+stage)
 e=x['event'];t=e['text'];state=x['public_state'];ev=x.get('actually_supplied_originals',[]);ground=x.get('grounding',{});reqs=a(state.get('requests'));wait=latest_wait(state);target=wait['goal_id']if wait else None
 p={'relation':'NEW','target_goal_id':None,'target_event_id':None,'task_id':None,'route':'DIRECT','purpose':t,'handling_text':'지정한 요청의 실제 내용을 확인해 답변하겠습니다.','answer':'TCP는 신뢰성 있는 순서 보장 전달을 제공하고 UDP는 독립 데이터그램을 전달합니다.','targets':[],'bindings':[],'source_revisions':{},'source_claims':[],'constraints':[],'exclusions':[],'allowed_actions':['draft_only'],'completed_actions':[],'unknown_actions':[],'permission_operations':[],'control_operations':[],'recipient':'minimal-draft-agent','artifact_type':'file','completion_condition':'요청한 실제 결과의 상태와 자료를 표시한다.'}
 if ev:
  p['answer']=' '.join(z['text']for z in ev);p['source_revisions']={z['source_id']:z['revision']for z in ev};p['source_claims']=[{**{k:z[k]for k in ['source_id','revision','start_line','end_line']},'quote':z['text'].splitlines()[0]}for z in ev];p['targets']=[z['source_id']for z in ev]
 if ground.get('bindings'):p['bindings']=ground['bindings'];p['targets']=[v for b in ground['bindings']for v in b['targets']]
 if any(w in t for w in ['고쳐줘','보존할 표','발표자료 파일']):
  p['route']='DELEGATE';p['handling_text']='지정 대상과 보존 조건에 맞춰 별도 업무를 위임하겠습니다.'
  if'보존할 표'in t:
   earlier=next(r for r in reqs if r.get('adoption',{}).get('route')=='DELEGATE');p['relation']='CORRECTION';p['target_goal_id']=earlier['goal_id'];old=earlier['adoption'];p['targets']+= [v for v in old.get('targets',[])if v.endswith(':P2')]
 if'F-PRIVATE'in t:p.update(route='WAIT',answer='파일 본문과 첨부 중 어떤 범위를 읽도록 허용하시겠습니까?',handling_text='F-PRIVATE 접근 목적과 범위 동의를 확인하겠습니다.')
 if'본문만 허용'in t:
  p.update(relation='ANSWER',target_goal_id=target,handling_text='본문만 읽고 첨부를 제외하여 요약하겠습니다.')
  if not ev:p.update(permission_operations=[{'source_id':'F-PRIVATE','read_segments':['body']}],answer=None)
 if'Agent에게 파일'in t:p.update(route='WAIT',answer='이 본문 요약을 외부 Agent에게 전달하도록 허용하시겠습니까?',handling_text='요약의 외부 전달 범위를 확인하겠습니다.')
 if'외부 전달은 하지 마'in t:p.update(relation='CONTROL',target_goal_id=target,route='CONTROL',answer='외부 전달을 거부했으므로 전송하지 않았습니다.',handling_text='외부 전달 거부와 미전송 상태를 유지하겠습니다.')
 if t.strip()=='응.':p.update(route='CLARIFY',answer='TA와 TB 중 어느 업무의 질문에 답하셨습니까?',handling_text='둘 중 어떤 업무에 대한 답인지 확인하겠습니다.')
 if'PDF로 만들어'in t:
  p.update(relation='ANSWER',target_goal_id=target,route='CONTROL',handling_text='TA는PDF로 변경하고TB는메일을 보내지 않겠습니다.',control_operations=[{'task_id':'TA','question_id':'Q-TA','action':'set_format','value':'PDF','disposition':'UPDATE'},{'task_id':'TB','question_id':'Q-TB','action':'send','value':'no','disposition':'DENY'}])
 if'TA만 승인'in t:p.update(relation='ANSWER',target_goal_id=target,route='CONTROL',handling_text='TA발송만 승인하고TB는발송하지 않겠습니다.',control_operations=[{'task_id':'TA','question_id':'Q-TA-SEND','action':'send','value':'yes','disposition':'APPROVE'},{'task_id':'TB','question_id':'Q-TB-SEND','action':'send','value':'no','disposition':'DENY'}])
 if'두 장으로 줄여'in t:p.update(route='CONTROL',task_id='TA',handling_text='TA보고서의결론을먼저놓고두장으로수정하겠습니다.',bindings=[{'expression':'그 보고서','targets':['TA:current-request']}],control_operations=[{'task_id':'TA','question_id':None,'action':'revise_report','value':'결론을 먼저 놓고 두 장으로 줄임','disposition':'UPDATE'}])
 if'비교 결과'in t or'그 결과'in t:
  p['bindings'].append({'expression':'그 비교 결과'if'그 비교 결과'in t else'비용 비교 결과'if'비용 비교 결과'in t else'그 결과','targets':['H10:v1:delivered']})
  if'일정 설명'in t:p['bindings'].append({'expression':'일정 설명','targets':['H11:v1:delivered']})
 if'원본 부분'in t:p['bindings']=[{'expression':'설명했던 원본 부분','targets':['DOC-A:v1:P1']}]
 if'바뀐 그 부분'in t:p['bindings']=[{'expression':'바뀐 그 부분','targets':['DOC-A:v2:P1']}]
 if'아까 문단 설명'in t:
  sid=next((z['source_id']for z in ev if':answer01:delivered'in z['source_id']),None)
  if sid:p['bindings']=[{'expression':'아까 문단 설명','targets':[sid]}]
 if'그 수정본'in t:p['bindings']=[{'expression':'그 수정본','targets':['TA:revised-result']}]
 if'이 요약'in t:p['bindings']=[{'expression':'이 요약','targets':['S26:summary:body-only']}]
 return p
