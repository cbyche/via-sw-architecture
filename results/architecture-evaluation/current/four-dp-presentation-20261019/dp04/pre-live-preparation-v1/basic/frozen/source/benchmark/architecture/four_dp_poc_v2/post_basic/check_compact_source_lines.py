"""Pure offline checks and coverage of all32 actual E4r1 model requests; no provider calls."""
import hashlib,json
from pathlib import Path
from compact_source_lines import transform,FormatError,VERSION
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir());ROOT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
checks=[]
def check(name,fn):
 try:assert fn();checks.append({'name':name,'pass':True})
 except Exception as e:checks.append({'name':name,'pass':False,'error':repr(e)});raise
p={'source_id':'actual','revision':2,'start_line':17,'end_line':19,'text':'alpha\n\nbeta'}
check('offset_and_blank_line_exact',lambda:transform(p)[0]['text']=='17| alpha\n18| \n19| beta')
check('original_not_mutated',lambda:p['text']=='alpha\n\nbeta'and'_original_line_format'not in p)
f,_=transform(p);check('no_double_numbering',lambda:transform(f)[0]==f)
check('literal_source_prefix_not_removed',lambda:transform({**p,'end_line':17,'text':'17| original literal'})[0]['text']=='17| 17| original literal')
check('summary_statement_unmodified',lambda:transform({'statement':'raw summary','text':'raw summary'})[0]=={'statement':'raw summary','text':'raw summary'})
check('delivered_fact_not_numbered',lambda:transform({**p,'kind':'ACTUALLY_DELIVERED_FACT'})[0]['text']==p['text'])
for name,proposal in [('invalid_zero_offset',{**p,'start_line':0}),('invalid_reversed_range',{**p,'start_line':20}),('line_count_no_truncation',{**p,'end_line':18}),('line_count_no_padding',{**p,'end_line':20})]:
 def reject(v=proposal):
  try:transform(v)
  except FormatError:return True
  return False
 check(name,reject)
check('trailing_blank_line_preserved',lambda:transform({**p,'end_line':18,'text':'alpha\n'})[0]['text']=='17| alpha\n18| ')
rows=[json.loads(x)for x in(ROOT/'basic-v1/runs.jsonl').read_text().splitlines()if x];rs=[r for r in rows if r['dp']=='05'and r['episode']=='dp05-e4'and r['repetition']==1];assert len(rs)==3
coverage=[]
for r in rs:
 for c in r['calls']:
  body=json.loads(Path(c['request_file']).read_text());raw=body['messages'][1]['content'];assert isinstance(raw,str);original=json.loads(raw);formatted,changes=transform(original)
  assert transform(formatted)[0]==formatted
  assert all(x['roundtrip_exact']for x in changes)
  # A model-input copy only: ensure original request file has not been changed.
  assert json.loads(raw)==original
  coverage.append({'variant':r['variant'],'stage':c['stage'],'role':c['role'],'call_uuid':c['call_uuid'],'request_file':c['request_file'],'request_sha256':hashlib.sha256(Path(c['request_file']).read_bytes()).hexdigest(),'numbered_passage_occurrences':len(changes),'changes':changes,'extra_source_text_utf8_bytes':sum(x['after_utf8_bytes']-x['before_utf8_bytes']for x in changes)})
assert len(coverage)==32
out={'formatter_version':VERSION,'formatter_sha256':hashlib.sha256(Path(__file__).with_name('compact_source_lines.py').read_bytes()).hexdigest(),'scope':'Offline copy transform/read-only actual E4r1 requests. No candidate/host/source/guard/oracle/request/API mutation. Calls without actual passage remain unnumbered metadata.','checks':checks,'actual_requests_examined':len(coverage),'request_coverage':coverage,'total_compact_text_extra_utf8_bytes':sum(x['extra_source_text_utf8_bytes']for x in coverage),'provider_calls':0}
p=ROOT/'analysis/compact-source-lines-offline-checks.json';p.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'pure_checks':len(checks),'actual_requests':len(coverage),'extra_utf8_bytes':out['total_compact_text_extra_utf8_bytes'],'provider_calls':0}))
