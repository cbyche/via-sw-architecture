"""Link existing frozen QA02/N-R judgments to actual first callbacks; no new oracle or timing changes."""
import collections,json
from pathlib import Path
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir());ROOT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
def read(p):return json.loads(p.read_text())
def main():
 groups={};known=[]
 for folder in sorted(ROOT.iterdir()):
  p=folder/'episode-metrics.json'
  if not p.exists()or not(folder/'MODEL-CALLS-AUTHORIZED.json').exists():continue
  for r in read(p):
   key=(folder.name,r['dp'],r['variant'],r.get('tactic'));g=groups.setdefault(key,{'phase_folder':folder.name,'dp':r['dp'],'variant':r['variant'],'tactic':r.get('tactic'),'expected_callbacks':0,'observed_callbacks':0,'known_wrong_referent_first_callbacks':0,'wrong_existing_nr_adoption_inputs':0,'first_full_semantic_correctness':'NOT_EXHAUSTIVELY_VALIDATED','checked_scope':'Existing fixed QA02 first referent/version fields; existing N-R adopted association fields. No new first-content semantic oracle; original latency preserved.'})
   timing=r['timing_details_by_stimulus'];g['expected_callbacks']+=r['qa01_expected'];g['observed_callbacks']+=r['qa01_n']
   for sid,t in timing.items():
    c=t.get('first_content')
    if not isinstance(c,dict):continue
    linked=[]
    for d in r.get('qa02_details',[]):
     if d.get('correct')is not False:continue
     a=d.get('actual_first_adoption');a=a if isinstance(a,dict)else{};expected=d.get('expected',{});match=False
     if r['dp']=='01':match=bool(a.get('adoption_id'))and a.get('adoption_id')==c.get('adoption_id')
     elif r['dp']=='04':
      scope=c.get('scope');scope=scope if isinstance(scope,list)else[]
      match=any(isinstance(x,dict)and x.get('expression_id')==expected.get('expression_id')and x.get('revision',c.get('revision'))==expected.get('revision')for x in scope)
     if match:linked.append(d)
    if linked:
     g['known_wrong_referent_first_callbacks']+=1;known.append({'phase_folder':folder.name,'run_id':r['run_id'],'dp':r['dp'],'variant':r['variant'],'stimulus_id':sid,'proxy_latency_ms':t['qa01_ms'],'first_actual_content':c,'existing_wrong_scope_judgments':linked,'reason':'Known referent-scope error in actual first adopted/displayed content. Does not define all semantic correctness dimensions.'})
   if r['dp']=='02':
    for d in r.get('nr_details',[]):
     if not d['correct']:
      g['wrong_existing_nr_adoption_inputs']+=1;t=timing.get(d['event_id'],{});known.append({'phase_folder':folder.name,'run_id':r['run_id'],'dp':'02','variant':r['variant'],'stimulus_id':d['event_id'],'proxy_latency_ms':t.get('qa01_ms'),'first_actual_content':t.get('first_content'),'existing_nr_judgment':d,'reason':'Wrong adopted relation/association fields. This is NOT automatically a wrong visible first sentence; inspect actual text and fields separately.'})
 out={'version':'EXISTING_SCOPE_CHECK_LINKAGE_ONLY','timing_or_oracle_changed':False,'full_correct_first_endpoint_measured':False,'rows':list(groups.values()),'known_field_error_details':known,'limitations':['QA01 remains actual substantive callback latency proxy; known wrong times remain in existing means.','No first full semantic correctness score inferred from QA02/N-R/N-H/final-content success.','Missing callback is not a displayed wrong callback. Final-body omission/error is not automatically a wrong first handling guide.','DP05 prefix review and final-body diagnosis remain separate; N-H is not QA02 or complete first semantics.']}
 (ROOT/'analysis/first-callback-scope-diagnostics.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'groups':len(groups),'known_field_error_examples':len(known)}))
if __name__=='__main__':main()
