"""Previously frozen semantic_checks, separately reported; no models or primary-score edits."""
import argparse,collections,hashlib,json
from pathlib import Path
BASE=Path(__file__).resolve().parent
REPO=BASE.parents[2]
def read(p):return json.loads(Path(p).read_text())
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def resolve(p):
 p=Path(p);return p if p.is_absolute()else REPO/p
def keyed(value):
 if isinstance(value,list):return {x['id']:x for x in value}
 if isinstance(value,dict)and'id'in value:return {value['id']:value}
 return value

def analyze(root):
 root=Path(root).resolve();base_freeze=read(root/'freeze.json');base_oracles={dp:keyed(read(BASE/'oracles'/f'dp{dp}.json'))for dp in ['01','02','04','05']};runs=[json.loads(x)for x in(root/'runs.jsonl').read_text().splitlines()if x];details=[];groups=collections.defaultdict(list);source_inventory=[]
 for dp in ['01','02','04','05']:
  p=BASE/'oracles'/f'dp{dp}.json';oracles=keyed(read(p));source_inventory.append({'dp':dp,'oracle_file':str(p),'sha256':sha(p),'matches_base_freeze':base_freeze['source'].get(str(p.relative_to(REPO)))==sha(p),'conditions_per_episode':{k:len(v.get('semantic_checks',[]))for k,v in oracles.items()}})
 for r in runs:
  source=BASE/'oracles'/f"dp{r['dp']}.json";source_type='BASE_FROZEN_ORACLE'
  if r['phase']in ['TACTIC','DIAGNOSTIC']:
   tactic_oracle=root/'tactics'/f"dp{r['dp']}-{r['tactic']}"/'oracle.json'
   if tactic_oracle.exists():source=tactic_oracle;source_type='TACTIC_FROZEN_ORACLE'
  oracle=keyed(read(source)).get(r['episode'],{});conditions=oracle.get('semantic_checks',[]);key=(r['phase'],r['dp'],r['variant'],r.get('tactic'))
  fixture=read(resolve(r['input']));run_details=[]
  for index,condition in enumerate(conditions):
   expr=condition.get('expression_id');events=[e for e in fixture.get('events',[])if e.get('expression_id')==expr]
   if condition.get('revision')is not None:events=[e for e in events if e.get('revision')==condition['revision']]
   record={'run_id':r['run_id'],'phase':r['phase'],'dp':r['dp'],'variant':r['variant'],'tactic':r.get('tactic'),'episode':r['episode'],'repetition':r['repetition'],'oracle_file':str(source),'oracle_source_type':source_type,'oracle_sha256':sha(source),'condition_index':index,'frozen_condition':condition,'condition_matches_original_frozen_entry':condition in base_oracles[r['dp']][r['episode']].get('semantic_checks',[])}
   if len(events)!=1:
    record.update({'status':'UNRESOLVED_EXPRESSION_OR_REVISION','matching_input_events':len(events)});details.append(record);run_details.append(record);continue
   e=events[0];sid=e.get('stimulus_id')or e.get('goal_id');rev=e.get('revision');displays=[x['data']for x in r['events']if x.get('kind')=='DISPLAY_PROXY'and(x['data'].get('stimulus_id')or x['data'].get('goal_id'))==sid]
   # First actual nonempty answer; never give credit for supplied evidence or an internal draft.
   actual=next((d['text']for d in displays if isinstance(d.get('text'),str)and d['text'].strip()),None)
   text=actual or'';facts=[{'marker':x,'exact_present':x in text}for x in condition.get('required_facts',[])];forbidden=[{'marker':x,'exact_present':x in text}for x in condition.get('forbidden_claims',[])]
   adopted=next((x['data']for x in r['events']if x.get('kind')in ['ADOPTION','SEMANTIC_ADOPTION']and x['data'].get('expression_id')==expr and x['data'].get('revision')==rev),None)
   versions={}
   if isinstance(adopted,dict):
    raw=adopted.get('source_revisions')
    if isinstance(raw,list):versions={x.get('source_id'):x.get('revision')for x in raw if isinstance(x,dict)}
    elif isinstance(raw,dict):versions=raw
   required=condition.get('required_source_revisions',{});version_ok=all(versions.get(k)==v for k,v in required.items())if required else None
   present=sum(x['exact_present']for x in facts);forbidden_hits=sum(x['exact_present']for x in forbidden)
   status='NO_DISPLAYED_CONTENT'if actual is None else'ALL_REQUIRED_FACT_MARKERS_ABSENT'if facts and present==0 else'REQUIRED_FACT_MARKERS_PARTIAL'if present<len(facts)else'SOURCE_IDENTITY_VERSION_MISMATCH'if version_ok is False else'FORBIDDEN_MARKER_PRESENT_REVIEW_REQUIRED'if forbidden_hits else'EXACT_MARKER_CHECK_PASS'
   record.update({'status':status,'stimulus_id':sid,'expression_revision':rev,'actual_first_display_text':actual,'display_attempt_count':len(displays),'required_fact_checks':facts,'required_fact_markers_present':present,'required_fact_markers_total':len(facts),'forbidden_marker_checks':forbidden,'forbidden_marker_hits':forbidden_hits,'required_source_identity_versions_match':version_ok,'scope_note':'Exact marker presence only. Format/paraphrase may differ; forbidden phrase hit can be negated/quoted and requires review. Whole omission here is evident because recorded answer repeats the referring phrase without any frozen facts.'})
   details.append(record);run_details.append(record)
  groups[key].append({'run_id':r['run_id'],'condition_count':len(conditions),'details':run_details})
 summaries=[]
 for(phase,dp,variant,tactic),records in sorted(groups.items(),key=lambda x:str(x[0])):
  ds=[d for r in records for d in r['details']];summaries.append({'phase':phase,'dp':dp,'variant':variant,'tactic':tactic,'runs_in_population':len(records),'runs_with_frozen_semantic_checks':sum(r['condition_count']>0 for r in records),'runs_without_defined_semantic_checks':sum(r['condition_count']==0 for r in records),'condition_instances':len(ds),'exact_marker_pass_instances':sum(d['status']=='EXACT_MARKER_CHECK_PASS'for d in ds),'whole_required_fact_omission_instances':sum(d['status']=='ALL_REQUIRED_FACT_MARKERS_ABSENT'for d in ds),'required_fact_markers_present':sum(d.get('required_fact_markers_present',0)for d in ds),'required_fact_markers_total':sum(d.get('required_fact_markers_total',0)for d in ds),'forbidden_marker_hits':sum(d.get('forbidden_marker_hits',0)for d in ds),'unresolved_or_missing_display':sum(d['status']in ['UNRESOLVED_EXPRESSION_OR_REVISION','NO_DISPLAYED_CONTENT']for d in ds),'status':'NOT_DEFINED'if not ds else'SEPARATE_FROZEN_CONTENT_DIAGNOSTIC'})
 result={'version':'FROZEN_SEMANTIC_CONDITIONS_DIAGNOSTIC_V1','total_runs_seen':len(runs),'primary_qa_changed':False,'raw_or_oracle_changed':False,'model_calls':0,'status':'SEPARATE_FROZEN_DIAGNOSTIC_NOT_FORMAL_QA','source_inventory':source_inventory,'summary_rows':summaries,'details':details,'required_fact_marker_instances':sum(d.get('required_fact_markers_total',0)for d in details),'required_fact_marker_instances_present':sum(d.get('required_fact_markers_present',0)for d in details),'limits':['Exact substrings do not recognize formatted numbers, date variants, synonyms or entailment. Absence alone can be representation mismatch; the complete omission in current E3 is directly visible in preserved short answers.','Forbidden marker absence is not proof that all equivalent forbidden assertions are absent; a hit is not proof of positive assertion because quotation/negation needs review.','No frozen semantic_checks for a run means NOT_DEFINED, never content pass. N-H and QA02 are separate.','Instrumented GOAL_COMPLETE records callback/readiness; it does not prove frozen content facts are displayed.']}
 (root/'semantic-conditions.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
 md=['# Frozen content-condition diagnostic','','Previously frozen `semantic_checks` are now reported separately. Existing QA02 and QA01/03 values remain unchanged. This is not a new formal QA. No source, oracle, raw record, prompt or execution condition changed, and no model judge/call was used.','','|Phase / DP / option / tactic|Runs|Frozen condition instances|Exact-marker pass|Whole required-fact omission|Required markers present/total|Forbidden literal hits|Runs without content checks|','|---|---:|---:|---:|---:|---:|---:|---:|']
 for s in summaries:md.append(f"|{s['phase']} / {s['dp']} / {s['variant']} / {s['tactic']or'—'}|{s['runs_in_population']}|{s['condition_instances']}|{s['exact_marker_pass_instances']}|{s['whole_required_fact_omission_instances']}|{s['required_fact_markers_present']}/{s['required_fact_markers_total']}|{s['forbidden_marker_hits']}|{s['runs_without_defined_semantic_checks']}|")
 md+=['','The only nonempty frozen content condition is DP01 E3 q2. It requires 1180000, 36, and 11월 1일, plus current source identities and a forbidden-claim guard. BASIC has nine such displayed answers (ABC×3) and selected A2 adds three. Source/version identity may pass QA02 while the displayed response omits these facts. The preserved answers repeat the referring phrase; they do not contain the frozen summary facts.','', 'This is an omitted evaluation/closure check, not evidence that the whole summary task completed. The plan scopes DP01 through actual evidence production and request interpretation; QA02 specifically measures referent sets/version identity. The separate prewritten content condition means it is also inaccurate to say all summary-content verification was outside the PoC. Keep identity completion and missing content-condition satisfaction separate.','','Exact-string checks have representation limits: `1,180,000` or a paraphrase need not contain `1180000`. Forbidden literal absence is not semantic safety proof. Here all three markers are wholly absent from short echo responses, which is preserved as an observable omission rather than a formatting-only conclusion.','','## Original source references','','- `oracles/dp01.json`, E3 `semantic_checks`: frozen required facts, source versions and forbidden claim.','- `11-minimal-poc-plan.md` §5: DP01 executes actual derived-state production through request interpretation.','- `benchmark/architecture/four_dp_poc/README.md`, frozen QA02: exact adopted target set and required versions; content facts are not QA02.','- `IMPLEMENTATION-dp01-dp05.md`: actual-evidence guard for direct completion and bounded source completion. E3 baseline q2 rejects direct completion but proceeds with no new body read.','- `analyze.py` before this separate diagnostic did not consume `semantic_checks`; this script does not change its primary metrics.','','## Preserved actual display text']
 for d in details:md.extend(['',f"### {d['run_id']}",'',f"Status: {d['status']}; displayed text: `{d.get('actual_first_display_text')}`."])
 (root/'semantic-conditions.md').write_text('\n'.join(md)+'\n');return result
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();r=analyze(a.results);print(json.dumps({'runs':r['total_runs_seen'],'frozen_condition_instances':len(r['details']),'required_markers_present':r['required_fact_marker_instances_present'],'required_markers_total':r['required_fact_marker_instances'],'model_calls':0}))
