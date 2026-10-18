"""Read original frozen BASIC request bytes to audit the claimed QA01 common path."""
import collections,hashlib,json
from pathlib import Path
HERE=Path(__file__).resolve().parent
REPO=next(p for p in HERE.parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir())
OLD=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2'
OUT=REPO/'results/architecture-evaluation/current/four-dp-presentation-20261019/dp05'
groups={};requests={};rows=[]
for p in sorted((OLD/'raw/runs').glob('*/trace.json')):
 r=json.loads(p.read_text());g=groups.setdefault(r['variant'],{'calls':0,'input_tokens':0,'output_tokens':0})
 for c in r['calls']:
  if c['stage']!='dp05_interpret_user_request':continue
  g['calls']+=1;g['input_tokens']+=c['input_tokens'];g['output_tokens']+=c['output_tokens'];v=json.loads(Path(c['request_file']).read_text());data=json.dumps(v,ensure_ascii=False,sort_keys=True).encode();goal=c['causal_trace']['goal_ids'][0];key=(r['episode'],r['repetition'],goal);h=hashlib.sha256(data).hexdigest();requests[(r['variant'],key)]=h
  rows.append({'run_id':r['run_id'],'variant':r['variant'],'episode':r['episode'],'repetition':r['repetition'],'goal_id':goal,'request_file':c['request_file'],'request_sha256':h,'request_user_message_characters':len(v['messages'][-1]['content']),'input_tokens':c['input_tokens'],'output_tokens':c['output_tokens']})
pairs={}
for a,b in [('A','B'),('A','C'),('B','C')]:
 keys=sorted({k for variant,k in requests if variant==a}&{k for variant,k in requests if variant==b});pairs[a+'/'+b]={'matched_inputs':len(keys),'actual_request_values_equal':sum(requests[(a,k)]==requests[(b,k)]for k in keys),'different':[{'episode':k[0],'repetition':k[1],'goal_id':k[2]}for k in keys if requests[(a,k)]!=requests[(b,k)]]}
out={'source_phase':str(OLD.relative_to(REPO)),'stage':'dp05_interpret_user_request','scope':'All 21 first normal semantic calls per option, including later failed admission; excludes separate interpretation repair and all producer/receiver calls','per_option':groups,'pairwise':pairs,'rows':rows,'conclusion_ko':'공통 함수/출력 계약은 같지만 실제 원문 입력이 다르므로 QA01 구조적 동점의 충분한 근거가 아니다. 작은 시간차의 우열은 미확정이며 C 선택은 이 불확실성을 공개한 조건부 권고로 구분한다.','old_source_raw_and_metrics_modified':False,'new_API_calls':0}
(OUT/'basic-prefix-boundary-audit.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'groups':groups,'pairs':pairs},ensure_ascii=False))
