"""Common9 public fixtures and physically separate evaluator registry from COMMON-28."""
from pathlib import Path
import json,hashlib
H=Path(__file__).resolve().parent;R=H.parents[2];CAT=R/'docs/architecture/12-decisions/decision-packages/qa-candidate-package/common-scenarios.json'
IDS=['S02','S08','S11','S12','S15','S18','S19','S20','S26']
DOC=['T1 | 비용 표 | Cedar 월 96달러, Birch 월 64달러','T2 | 일정 표 | 시작 10월 20일, 완료 10월 30일','T3 | 자원 표 | Cedar RAM 640MB, Birch RAM 384MB','P1 | 핵심 설명 | 준비된 파일은 10개이며 결론을 먼저 제시한다.','P2 | 수정 대상 | 이 문단은 두 문장으로 명확하게 수정해야 한다.']
def src(id,text,**kw):return {'id':id,'revision':1,'title':id,'kind':'document','text':text,'local_allowed':True,'external_allowed':True,**kw}
def initial(sid):
 sources=[src('DOC-A','\n'.join(DOC),extents={'T1':[1,1],'T2':[2,2],'T3':[3,3],'P1':[4,4],'P2':[5,5]})]
 relevant=[]
 if sid=='S02':sources=[src('F01','결론: 작은 읽기 작업을 병합하면 준비 비용을 줄일 수 있다.\n기존 권한과 판본 검사는 유지해야 한다.'),src('M01','보낸 사람: 민수\n제목: 10월 검토 일정\n보낸 시각: 2026-10-18 16:30'),src('CAL01','제목: 구조 검토 회의\n시간: 2026-10-19 15:00-16:00'),src('WEB01','제품 가격: 129000원\n기준 시각: 2026-10-19 09:00'),src('PUB01','오늘 운영 시간: 09:00-18:00\n출처: 공개 안내 PUB01'),src('BK01','제품명: Cedar 도킹 스테이션\n설명: USB-C와 외부 화면 두 개를 연결하는 허브')]
 if sid in ['S11','S12']:
  relevant=[{'id':'H10','revision':1,'kind':'delivered','text':'실제 표시한 비용 비교: Cedar 월96달러, Birch 월64달러. Birch가 월32달러 저렴하다.','source_spans':[]},{'id':'H11','revision':1,'kind':'delivered','text':'실제 표시한 일정 설명: Cedar는10월20일 시작해10월30일 완료한다. Birch는11월3일 완료한다.','source_spans':[]},{'id':'H12','revision':1,'kind':'delivered','text':'관계없는 질문 답: 무지개는 빛의 굴절과 분산으로 생긴다.','source_spans':[]}]
 if sid=='S12':relevant=[{'id':'H20','revision':1,'kind':'delivered','text':'실제 표시한 설명: DOC-A v1 P1은 준비된 파일이10개이며 결론을 먼저 제시한다고 설명했다.','source_spans':[{'source_id':'DOC-A','revision':1,'start_line':4,'end_line':4}]}]
 for h in relevant:sources.append(src(h['id'],h['text'],kind='delivered',extents={'delivered':[1,1]},provenance=h.get('source_spans',[])))
 if sid=='S26':sources=[src('F-PRIVATE','본문: 비공개 검토의 핵심은 원본 판본과 권한을 확인하는 것이다.\n본문: 외부 전달은 별도 동의가 필요하다.\n첨부: PRIVATE_ATTACHMENT_DO_NOT_DISCLOSE',local_allowed=False,external_allowed=False,segments={'body':[1,2],'attachments':[3,3]},allowed_segments=[],external_segments=[])]
 tasks=[{'task_id':f'T-DONE-{i:02}','title':f'완료한 과거 작업 {i}','state':'COMPLETED','result':f'archive-{i}.txt'}for i in range(1,21)]
 tasks += [{'task_id':'TA','title':'보고서 작업','state':'COMPLETED'if sid=='S18'else'WAITING','current_request':'보고서 두 장 작성','artifact':'report-v1.txt','questions':[],'allowed_actions':['revise_report','make_pdf','send'],'completed_actions':[],'unknown_actions':[]},{'task_id':'TB','title':'메일 검색과 발송 작업','state':'RUNNING','current_request':'관련 메일 검색','questions':[],'allowed_actions':['send','deny_send'],'completed_actions':[],'unknown_actions':[]},{'task_id':'BG','title':'다른 배경 작업','state':'RUNNING','questions':[],'allowed_actions':['observe']},{'task_id':'TD','title':'대기 작업 D','state':'WAITING','questions':[],'allowed_actions':[]},{'task_id':'TE','title':'대기 작업 E','state':'WAITING','questions':[],'allowed_actions':[]}]
 if sid=='S18':
  tasks=[t for t in tasks if t['task_id']!='T-DONE-20']
 for task in tasks:
  if task['task_id']=='TA'and sid!='S18':task['state']='WAITING'if sid in ['S20','S26']else'IDLE'
  if task['task_id']=='TB':task['state']='RUNNING'if sid in ['S18','S19','S20','S26']else'IDLE'
  if task['task_id']=='BG':task['state']='RUNNING'if sid in ['S11','S18','S19']else'IDLE'
  if task['task_id']in ['TD','TE']:task['state']='IDLE'
 if sid=='S18':sources.append(src('TA:current-request','TA 보고서 요청: 결론을 포함한 보고서 두 장. 현재 완료 확인됨.\n결과: report-v1.txt',kind='task',extents={'current-request':[1,2]}))
 # The same 200 authored synthetic raw turns exist in every comparison; no precomputed semantic summary is supplied.
 history=[{'event_id':f'INIT-{i:03}','kind':'user'if i%2 else'assistant','text':f'기록 {i}: 허용한 자료의 판본과 출처를 확인한다.'}for i in range(1,201)]
 for index,h in enumerate(relevant):history[index]={'event_id':h['id'],'kind':'assistant','text':h['text'],'source_spans':h.get('source_spans',[])}
 sources.append(src('CONVERSATION-LOG','\n'.join(f"{h['event_id']} {h['kind']}: {h['text']}"for h in history),kind='conversation_history',extents={'all':[1,200]}))
 return {'sources':sources,'raw_history':history,'historical_delivered':relevant,'tasks':tasks}
def main():
 cat=json.loads(CAT.read_text());ss=[next(s for s in cat['scenarios']if s['id']==sid)for sid in IDS];obs={}
 if(H/'assets/observations.json').exists():obs=json.loads((H/'assets/observations.json').read_text()).get('events',{})
 public=[];oracles={};transform=[]
 for s in ss:
  sid=s['id'];events=[];schedule=[];userno=0
  for e in s['events']:
   raw={'id':e['id'],'kind':e['kind'],'text':e['input']}
   if e['kind']=='user':
    userno+=1;raw['delivery_source_id']=f'{sid}:answer{userno:02}:delivered'
    if sid=='S26'and userno==2:raw['delivery_source_id']='S26:summary:body-only'
   else:
    raw['task_id']='BG'if e.get('background_load')else None
    raw['event_class']='PROGRESS'if e.get('background_load')and'진행'in e['input']else'RESULT'
    if sid=='S20'and e['id'].endswith(('A01','A02')):
     task='TA'if e['id'].endswith('A01')else'TB';raw.update(task_id=task,event_class='QUESTION',questions=[{'question_id':f'Q-{task}','task_id':task,'text':e['input'],'kind':'FORMAT'if task=='TA'else'APPROVAL'}])
    if sid=='S26'and e['id'].endswith('A01'):raw.update(event_class='QUESTION',questions=[{'question_id':'Q-TA-SEND','task_id':'TA','text':'민수에게 발송할까요?','kind':'APPROVAL'},{'question_id':'Q-TB-SEND','task_id':'TB','text':'지연에게 발송할까요?','kind':'APPROVAL'}])
   if e['id']in obs:raw.update(obs[e['id']])
   events.append(raw);schedule.append({'event_id':e['id'],'original_trigger':e['trigger'],'authored_utterance_seconds':e['input_audio_seconds'],'response_id':e['response']['id'],'event':raw})
   transform.append({'event_id':e['id'],'original_mode':e['input_mode'],'original_audio_seconds':e['input_audio_seconds'],'new_mode':'SCHEDULED_TRANSCRIPT'if e['kind']=='user'else'EXTERNAL_SOURCE','voice_output':'OFF','new_endpoint':'actual common host text callback','trigger_adjustment':'Firstinput_start=source+5s; finaltranscript submit after original authored utterance duration; no actual ASR. Audible response anchors use same text response callback. Retain specified2s/200ms/100ms/8s offsets and original transcript duration.'})
  public.append({'id':sid,'initial':initial(sid),'events':events,'schedule':schedule})
  oracles[sid]={'responses':[{'event_id':e['id'],'response_id':e['response']['id'],'required_content':e['response']['required_content']}for e in s['events']],'goals':s['goals'],'bindings':s['bindings']}
 for name,value in [('fixtures.json',public),('oracles.json',oracles),('voice-off-transform.json',transform)]: (H/name).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n')
 totals={'version':'COMMON9-TEXT-PROXY-V1','source_registry_sha256':hashlib.sha256(CAT.read_bytes()).hexdigest(),'episodes':IDS,'per_rep':{'responses':sum(len(s['events'])for s in ss),'goals':sum(len(s['goals'])for s in ss),'bindings':sum(len(s['bindings'])for s in ss)},'repetitions':5,'configurations':['AAAA','BAAA','CAAA','ABAA','AABA','AAAB','AAAC'],'unique_episode_runs':315,'raw_history_turns_each':200,'completed_tasks_each':20,'active_or_known_task_slots_each':5,'candidate_never_receives':['oracles.json','required_content','expected_set','oracle_goal_id','whole_future_schedule']};(H/'population.json').write_text(json.dumps(totals,indent=2)+'\n');print(totals)
if __name__=='__main__':main()
