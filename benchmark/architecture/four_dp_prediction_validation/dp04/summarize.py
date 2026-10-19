"""New-only condition tables, complete denominators, actual helper init/inference split."""
from pathlib import Path
import sys,json,collections,statistics
BASE=Path(__file__).resolve().parent;sys.path.insert(0,str(BASE));import analyze
from bridge import dump
ROOT=BASE.parents[3]/'results/architecture-evaluation/current/four-dp-prediction-validation-20261019/dp04'
def main(phase='main'):
 root=ROOT/phase;analyze.analyze(root);rows=json.loads((root/'episode-metrics.json').read_text());records=[json.loads(l) for l in(root/'runs.jsonl').read_text().splitlines()]
 grouped=collections.defaultdict(list)
 for r in rows:grouped[(r['episode'],r['variant'])].append(r)
 cases=[]
 for (e,v),rs in grouped.items():
  n1=sum(r['qa01_n']for r in rs);n3=sum(r['qa03_n']for r in rs);cases.append({'episode':e,'variant':v,'repetitions':len(rs),'qa01_ms':sum(r['qa01_sum_ms']for r in rs)/n1 if n1 else None,'qa01_observed':n1,'qa01_expected':sum(r['qa01_expected']for r in rs),'qa02_correct':sum(r['qa02_correct']for r in rs),'qa02_expected':sum(r['qa02_total']for r in rs),'qa03_ms':sum(r['qa03_sum_ms']for r in rs)/n3 if n3 else None,'qa03_observed':n3,'qa03_expected':sum(r['qa03_expected']for r in rs),'cloud_cost_usd':sum(r['via_cost_usd']for r in rs),'peak_concurrent_rss_mb':max(r['rss_concurrent_peak_mb']or 0 for r in rs),'cloud_calls':sum(r['model_calls']for r in rs)})
 helper=[]
 for r in records:
  calls=[c for c in r['calls']if c.get('evidence_kind')=='LOCAL_PARSER_NOT_CLOUD_MODEL_CALL'];helper.append({'run_id':r['run_id'],'variant':r['variant'],'episode':r['episode'],'initialization_ns':sum((c.get('initialization')or{}).get('initialization_ns',0)for c in calls),'inference_ns_new_frames':sum(c.get('inference_ns_new_frames',0)for c in calls),'helper_operations':len(calls),'new_frames':sum(c.get('new_frames',0)for c in calls),'reused_frames':sum(c.get('reused_frames',0)for c in calls),'max_sampled_helper_rss_mb':max((m.get('rss_bytes_by_role',{}).get('real_local_detector_ocr_caption',0)/1e6 for m in r['memory']),default=0),'incomplete_process_samples':sum(not m.get('all_registered_observed',False)for m in r['memory'])})
 dump(root/'case-summary.json',cases);dump(root/'helper-breakdown.json',helper)
 lines=['# DP3 new conditional validation','','All old campaigns remain separate. Conditional observed means retain fixed scheduled denominators. Local CPU helper deployment is a diagnostic; original cloud parser cost/time/RAM NOT_MEASURED.','','|Case|Configuration|QA01 ms observed/expected|QA02 correct/expected|QA03 ms complete/expected|Cloud cost USD|Simultaneous RSS peak MB|','|---|---|---|---|---|---|---|']
 for c in cases:
  fmt=lambda x:'MISSING'if x is None else f'{x:.3f}'
  lines.append(f"|{c['episode']}|{c['variant']}|{fmt(c['qa01_ms'])} {c['qa01_observed']}/{c['qa01_expected']}|{c['qa02_correct']}/{c['qa02_expected']}|{fmt(c['qa03_ms'])} {c['qa03_observed']}/{c['qa03_expected']}|{c['cloud_cost_usd']:.7f}|{c['peak_concurrent_rss_mb']:.3f}|")
 (root/'comparison.md').write_text('\n'.join(lines)+'\n');print(json.dumps({'runs':len(records),'case_rows':len(cases),'cloud_calls':sum(c.get('evidence_kind')!='LOCAL_PARSER_NOT_CLOUD_MODEL_CALL'for r in records for c in r['calls']),'local_parser_ops':sum(len([c for c in r['calls']if c.get('evidence_kind')=='LOCAL_PARSER_NOT_CLOUD_MODEL_CALL'])for r in records)}))
if __name__=='__main__':main(sys.argv[1]if len(sys.argv)>1 else'main')
