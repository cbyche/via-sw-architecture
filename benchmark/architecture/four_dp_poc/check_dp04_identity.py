"""Focused positive/negative checks for evaluator-only visible identity normalization."""
import copy,json,tempfile
from pathlib import Path
from score_dp04 import canonicalize_adoptions
from bridge import dump
BASE=Path(__file__).resolve().parent
ROOT=BASE.parents[2]
OUT=ROOT/'results/architecture-evaluation/current/four-dp-qwen37-20261018'
checks=[]
def check(name,condition):
 if not condition:raise AssertionError(name)
 checks.append({'check':name,'status':'PASS'})
def source_record(fixture,variant='A',state=None):
 path=Path(tempfile.mkdtemp(prefix='via-dp04-id-'))/'input.json';dump(path,fixture)
 events=[]
 if state is not None:events=[{'kind':'SCREEN_CANDIDATES_PUBLISHED','data':{'revision':1,'state':state}}]
 return {'input':str(path),'variant':variant,'events':events}
fixtures=json.loads((BASE/'fixtures/dp04.json').read_text());correction=next(x for x in fixtures if x['id']=='dp04-partial-correction')
def adoption(target):return {'expression_id':'keep-first','revision':1,'targets':[target]}
r=source_record(correction)
check('Exact full visible label canonicalizes without target oracle',canonicalize_adoptions(r,[adoption('C1 / Baseline')])['adoptions'][0]['targets']==['C1'])
check('Incorrect visible target stays incorrect target',canonicalize_adoptions(r,[adoption('C2 / Expansion')])['adoptions'][0]['targets']==['C2'])
check('Unknown internal alias is not guessed from embedded ID',canonicalize_adoptions(r,[adoption('chart_c1_baseline')])['adoptions'][0]['targets']==['chart_c1_baseline'])
f=correction['revisions'][0]['frames'][0];snapshot={k:f[k]for k in ['frame_id','source_id','source_revision','sha256']}
state={'candidates':[{'target_id':'chart_c1_baseline','label':'C1 / Baseline','frame_id':'correction'}],'source_snapshot':[snapshot]}
r=source_record(correction,'B',state)
check('Same-run published alias maps through exact label and source snapshot',canonicalize_adoptions(r,[adoption('chart_c1_baseline')])['adoptions'][0]['targets']==['C1'])
bad=copy.deepcopy(state);bad['source_snapshot'][0]['sha256']='other-source'
check('Wrong source snapshot blocks alias normalization',canonicalize_adoptions(source_record(correction,'B',bad),[adoption('chart_c1_baseline')])['adoptions'][0]['targets']==['chart_c1_baseline'])
bad=copy.deepcopy(state);bad['candidates'][0]['frame_id']='not-observed'
check('Wrong frame blocks alias normalization',canonicalize_adoptions(source_record(correction,'B',bad),[adoption('chart_c1_baseline')])['adoptions'][0]['targets']==['chart_c1_baseline'])
bad=copy.deepcopy(state);bad['candidates'].append(copy.deepcopy(bad['candidates'][0]))
check('Duplicate producer aliases remain unresolved',canonicalize_adoptions(source_record(correction,'B',bad),[adoption('chart_c1_baseline')])['adoptions'][0]['targets']==['chart_c1_baseline'])
# Duplicate visual label from another original frame/version is not collapsed.
ambiguous=copy.deepcopy(correction);ambiguous['revisions'][0]['frames'].append({**f,'frame_id':'another-observation','source_revision':2})
check('Same label across different source frames remains unresolved',canonicalize_adoptions(source_record(ambiguous),[adoption('C1 / Baseline')])['adoptions'][0]['targets']==['C1 / Baseline'])
check('Ambiguous bare canonical string carries unresolved flag for scorer',canonicalize_adoptions(source_record(ambiguous),[adoption('C1')])['adoptions'][0]['_target_identity_unresolved']is True)
# Do not accept a source label that is not actually visible in the given frame.
bad=copy.deepcopy(state);bad['candidates'][0]['label']='C1 / Invented chart'
check('Unobserved lookalike label is not fuzzy matched',canonicalize_adoptions(source_record(correction,'B',bad),[adoption('chart_c1_baseline')])['adoptions'][0]['targets']==['chart_c1_baseline'])
records=[json.loads(s)for s in (OUT/'runs.jsonl').read_text().splitlines()if s]
for rec in records:
 if rec['phase']=='BASIC'and rec['dp']=='04'and rec['repetition']==1 and rec['episode']in ['dp04-dense','dp04-free-region']:
  adopted=[e['data']for e in rec['events']if e.get('kind')=='ADOPTION'];new=canonicalize_adoptions(rec,adopted)['adoptions']
  check('Real wrong dense/free targets preserved: '+rec['run_id'],all(a['targets']==b['targets']for a,b in zip(adopted,new)))
dump(OUT/'dp04-identity-checks.json',{'count':len(checks),'checks':checks,'model_calls':0,'scope':'Source identity normalization only; no target oracle consulted by normalizer'})
print(json.dumps({'checks_passed':len(checks),'model_calls':0}))
