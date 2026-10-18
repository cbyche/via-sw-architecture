"""Explicit derived selector: old DP01/02/04 84 + corrected DP05 36, not a new measured phase."""
import csv,hashlib,json,shutil
from pathlib import Path
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir());ROOT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
def read(p):return json.loads(Path(p).read_text())
def save(p,v):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def csvsave(p,rows):
 with p.open('w')as f:
  fields=list(dict.fromkeys(k for r in rows for k in r));w=csv.DictWriter(f,fieldnames=fields);w.writeheader();w.writerows({k:json.dumps(v,ensure_ascii=False)if isinstance(v,(dict,list))else v for k,v in r.items()}for r in rows)
def main():
 sources=[(ROOT/'basic-v1',lambda r:r['dp']!='05'),(ROOT/'basic-dp05-reuse-union-corrected-v2',lambda r:r['dp']=='05')];out=ROOT/'effective-main-comparison';out.mkdir(exist_ok=True);selected=[];episodes=[];summary=[];cases=[];pairs=[];memory=[];sourcefiles=[]
 for folder,choose in sources:
  raw=[json.loads(x)for x in(folder/'runs.jsonl').read_text().splitlines()if x];rs=[r for r in raw if choose(r)];assert len(rs)==(84 if folder.name=='basic-v1'else 36)
  for r in rs:
   trace=Path(r['input']).with_name('trace.json');selected.append({'run_id':r['run_id'],'dp':r['dp'],'variant':r['variant'],'episode':r['episode'],'repetition':r['repetition'],'actual_phase':r['phase'],'source_phase_root':str(folder),'trace':str(trace),'trace_sha256':sha(trace),'input':r['input'],'input_sha256':sha(r['input']),'freeze_sha256':r['freeze_sha256'],'candidate_binary_sha256':r['candidate_binary_sha256']})
  episodes.extend(r for r in read(folder/'episode-metrics.json')if choose(r));summary.extend({**r,'source_phase_root':str(folder)}for r in read(folder/'summary.json')['rows']if choose(r));cases.extend(r for r in read(folder/'case-summary.json')if choose(r));pairs.extend(r for r in read(folder/'pairwise-endpoint-coverage.json')if choose(r));memory.extend(r for r in read(folder/'memory-option-peaks.json')if choose(r));sourcefiles.append({'path':str(folder/'episode-metrics.json'),'sha256':sha(folder/'episode-metrics.json')})
 assert len(selected)==len(episodes)==120 and len(summary)==len(memory)==10 and len(cases)==40
 assert len({r['run_id']for r in selected})==120
 expected1=sum(r['qa01_expected']for r in episodes);expected3=sum(r['qa03_expected']for r in episodes);assert(expected1,expected3)==(261,243)
 selection={'version':'EXPLICIT_EFFECTIVE_MAIN_120_AFTER_DP05_REUSE_CORRECTION','derived_not_measured_phase':True,'rule':'DP01/02/04 select original BASIC84; DP05 select corrected ABC36. No failed trial is individually substituted; whole DP05 all alternatives/cases/repetitions are rerun under one correction freeze.','original_basic_120_preserved':True,'history_not_selected_as_headline':{'phase':'basic-v1','dp':'05','trials':36,'reason':'Actual VIA-read retention and intrinsic valid-support prefix view omissions; corrected supplied-span gapless-union admission; raw failures/costs remain in campaign totals.'},'selected_trials':120,'expected_qa01':expected1,'expected_qa03':expected3,'observed_callback_proxy':sum(r['qa01_n']for r in episodes),'observed_host_completion':sum(r['qa03_n']for r in episodes),'qa01_claim':'Actual substantive callback latency proxy; first full semantic correctness separately bounded, not fully proven.','qa03_claim':'Frozen host-contract completion; final requested-content fulfillment separately diagnosed. Receiver-shim proxy and full wall adjacent.','source_derived_files':sourcefiles,'selection':selected,'code_sha256':sha(Path(__file__))}
 save(out/'selection.json',selection);save(ROOT/'effective-main-selection.json',selection)
 save(out/'summary.json',{'derived_not_measured_phase':True,'selection':str((out/'selection.json').resolve()),'rows':summary});save(out/'episode-metrics.json',episodes);save(out/'case-summary.json',cases);save(out/'pairwise-endpoint-coverage.json',pairs);save(out/'memory-option-peaks.json',memory)
 csvsave(out/'case-summary.csv',cases);csvsave(out/'memory-option-peaks.csv',memory);csvsave(out/'pairwise-endpoint-coverage.csv',pairs)
 flat=[]
 for r in episodes:flat.append({k:v for k,v in r.items()if not isinstance(v,(dict,list))})
 csvsave(out/'episode-metrics.csv',flat)
 for name in ['stage-calls.csv','goal-endpoints.csv','response-endpoints.csv','memory-breakdown.csv','all-failure-callbacks.csv','proposal-rejections.csv','policy-waits.csv','preparation.csv']:
  values=[]
  for folder,choose in sources:
   with(folder/name).open()as f:values.extend(r for r in csv.DictReader(f)if choose(r))
  csvsave(out/name,values)
 fmt=lambda x:'NOT_OBSERVED'if x is None else f'{x:.3f}'
 lines=['# 발표용 명시적 120회 선택 비교','','새 측정 phase가 아니라 원시 선택과 파생 표다. 기존01/02/04 84회와 DP05 전체 교정36회를 사용한다. 원래 DP05 36회·calibration·보완·입력 진단은 모두 보존하고 전체 비용에 포함한다. 개별 실패를 성공 trial로 치환하지 않았다.','','QA01은 실제 구체적 callback latency proxy이며 완전한 첫 의미 정답 검증이 아니다. QA03는 동결 host-contract 완료이며 최종 본문 충족과 별도다. 평균은 관측된 endpoint에 조건부이고 누락/오답/내용 조건과 함께 읽는다.','','|DP|안|실제source phase|첫 callback ms (관측/예정)|host 완료 proxy ms (관측/예정)|전체wall ms|receiver 제외 ms|지칭 정답/예정|QA04 모듈|VIA USD|receiver USD|calls|동시RSS 최대MB|계약/완료 실패 episode|','|---|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|']
 for r in summary:lines.append(f"|{r['dp']}|{r['variant']}|{Path(r['source_phase_root']).name}|{fmt(r['qa01_ms'])} ({r['qa01_n']}/{r['qa01_expected']})|{fmt(r['qa03_ms'])} ({r['qa03_n']}/{r['qa03_expected']})|{fmt(r['qa03_full_elapsed_ms'])}|{fmt(r['receiver_shim_excluded_ms'])}|{r['qa02_correct']}/{r['qa02_total']}|{r['qa04_modules_mean']}|{r['qa05_via_usd']:.7f}|{r['qa05_receiver_usd']:.7f}|{r['model_calls']}|{fmt(r['qa06_peak_rss_mb'])}|{r['errors']}/{r['episodes']}|")
 lines+=['',f'고정 예정 분모: QA01 {expected1}, QA03 {expected3}. Actual observed callback {selection["observed_callback_proxy"]}, host completion {selection["observed_host_completion"]}.','', 'selection.json contains each actual source phase/input/trace/freeze/binary hash. Case medians/min/max and individual3 values remain in per-phase individual-trials.md and combined case-summary.json. Memory is sampled concurrent Python+Rust only, initial is startup rather than steady baseline, logical bytes are not RSS, and experiment evidence buffers are separate.']
 (out/'report.md').write_text('\n'.join(lines)+'\n');print(json.dumps({'effective_trials':120,'qa01_expected':expected1,'qa03_expected':expected3,'options':len(summary)}))
if __name__=='__main__':main()
