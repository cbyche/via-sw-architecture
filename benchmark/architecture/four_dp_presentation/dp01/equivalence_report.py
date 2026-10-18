"""Apply the unchanged pre-formal20261018 parallel unit/date diagnostic; never candidate input."""
import argparse,hashlib,importlib.util,json
from pathlib import Path
HERE=Path(__file__).resolve().parent
REPO=next(p for p in HERE.parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir())
ROOT=REPO/'results/architecture-evaluation/current/four-dp-presentation-20261019'
def main(dp):
 code=REPO/'benchmark/architecture/four_dp_poc_v2/reports/evaluate_bounded_content_equivalence.py';spec=importlib.util.spec_from_file_location('prior_equivalence',code);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
 r=ROOT/dp;metrics=json.loads((r/'episode-metrics.json').read_text());oracle=json.loads((r/'frozen/source/benchmark/architecture/four_dp_poc_v2/oracles'/f'{dp}.json').read_text());oracle={e['id']:e for e in oracle}if isinstance(oracle,list)else oracle;rows=[];groups={}
 for row in metrics:
  value=module.evaluate(row['result'],oracle[row['episode']]);rows.append({'run_id':row['run_id'],'variant':row['variant'],'tactic':row['tactic'],'repetition':row['rep'],**value});g=groups.setdefault(row['variant']+'/'+str(row['tactic']),{'literal_correct':0,'equivalent_correct':0,'total':0})
  for k in g:g[k]+=value[k]
 out={'evaluator_version':module.VERSION,'code_sha256':hashlib.sha256(code.read_bytes()).hexdigest(),'rules_unchanged_from_preformal_20261018':True,'candidate_input':False,'model_calls':0,'groups':groups,'rows':rows,'separate_from_manual_meaning':True};(r/'parallel-content-equivalence.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps(groups,ensure_ascii=False))
if __name__=='__main__':p=argparse.ArgumentParser();p.add_argument('--dp',choices=['dp01','dp05'],required=True);main(p.parse_args().dp)
