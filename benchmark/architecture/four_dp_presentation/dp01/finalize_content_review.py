"""Materialize fixed-denominator author reviews from individually read goal/text hashes.
No semantic classifier or model call. Unknown text halts finalization instead of guessing.
"""
import argparse,collections,hashlib,json
from pathlib import Path
HERE=Path(__file__).resolve().parent
REPO=next(p for p in HERE.parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir())
ROOT=REPO/'results/architecture-evaluation/current/four-dp-presentation-20261019'
def read(p):return json.loads(p.read_text())
def finalize(dp):
 r=ROOT/dp;known=read(r/'content-known-texts.json')['reviewed_unique_goal_hashes'];schedule=read(HERE.parent/dp/'schedule.json');records=[read(p)for p in(r/'raw/runs').glob('*/trace.json')];rows=[];unread=[];duplicates=[]
 for spec in schedule:
  policy=spec['policy']if dp=='dp01'else(None if spec['tactic']=='NONE'else spec['tactic']);found=[x for x in records if x['variant']==spec['variant']and x['repetition']==spec['repetition']and x['tactic']==policy]
  if len(found)!=1:raise ValueError('Missing/duplicate planned episode '+repr(spec))
  record=found[0];goals=['e3-q1','e3-q2']if dp=='dp01'else['h3-deck','h3-mail','h3-meeting']
  for goal in goals:
   displays=[e for e in record['events']if e.get('kind')=='DISPLAY_PROXY'and e['data'].get('content_type')=='DETAILED_RESULT'and e['data'].get('goal_id')==goal]
   terminal=next((e['data']for e in record['events']if e.get('kind')=='GOAL_COMPLETE'and e['data']['goal_id']==goal),None)
   base={'run_id':record['run_id'],'variant':record['variant'],'tactic':record['tactic'],'repetition':record['repetition'],'goal_id':goal,'trace':str(Path(record['input']).with_name('trace.json'))}
   if not displays:rows.append({**base,'verdict':'NOT_DISPLAYED','text':None,'reason':'No actual detailed display in fixed planned goal; rejected drafts not counted','failures':[e['data']for e in record['events']if e.get('kind')=='EPISODE_ERROR'or(e.get('kind')=='GOAL_FAILED'and e['data'].get('goal_id')==goal)]});continue
   text=displays[0]['data']['text'];h=hashlib.sha256(text.encode()).hexdigest();key=goal+':'+h
   if key not in known:unread.append({'run_id':record['run_id'],'goal_id':goal,'body_sha256':h,'text':text});continue
   if terminal is None or terminal['text']!=text:raise ValueError('Actual display/host terminal disagreement')
   rows.append({**base,'text':text,'body_sha256':h,'verdict':known[key]['verdict'],'reason':known[key]['reason'],'first_reviewed_run_id':known[key]['first_reviewed_run_id'],'same_goal_same_sources_hash_review_reuse':known[key]['first_reviewed_run_id']!=record['run_id'],'host_terminal_matches_actual_display':True})
 if unread:
  (r/'content-unreviewed.json').write_text(json.dumps(unread,ensure_ascii=False,indent=2)+'\n');raise ValueError(str(len(unread))+' actual bodies need manual review; no guessed verdict')
 groups={}
 for row in rows:groups.setdefault(str(row['variant'])+'/'+str(row['tactic']),collections.Counter())[row['verdict']]+=1
 out={'status':'COMPLETE_AUTHOR_REVIEW','fixed_denominator':len(schedule)*(2 if dp=='dp01'else 3),'displayed':sum(row['verdict']!='NOT_DISPLAYED'for row in rows),'reviewed_unique_goal_text_hashes':len({row.get('body_sha256')for row in rows if row.get('body_sha256')}),'groups':{k:dict(v)for k,v in groups.items()},'rows':rows,'metric_separation':'manual final-body semantics; not QA02, N-H, literal markers or proxy first-handling coverage','model_judge_calls':0,'candidate_receives_evaluator_answers':False}
 (r/'content-review.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');lines=['# 추가 실험 실제 본문 검토','',f'고정 분모 {out["fixed_denominator"]}, 실제 표시 {out["displayed"]}. 저자 원문 대조이며 모델 판정 호출은 0회다. 같은 goal·원문조건·본문 SHA만 같은 경우 검토를 연결했다.','', '| 구성 | 판정 |','| --- | --- |']+[f'| {k} | {dict(v)} |'for k,v in groups.items()];(r/'content-review.md').write_text('\n'.join(lines)+'\n');print(json.dumps({'dp':dp,'fixed_denominator':out['fixed_denominator'],'groups':out['groups']},ensure_ascii=False))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--dp',choices=['dp01','dp05'],required=True);finalize(p.parse_args().dp)
