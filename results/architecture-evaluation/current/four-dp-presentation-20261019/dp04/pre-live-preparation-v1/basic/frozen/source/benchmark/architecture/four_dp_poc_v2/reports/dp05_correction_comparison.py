"""Whole-DP fair correction comparison: all ABC four cases, never selective success replacement."""
import collections,hashlib,json
from pathlib import Path
from tactic_comparison import detail
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir());R=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
def read(p):return json.loads(p.read_text())
def raws(folder):return [json.loads(x)for x in(folder/'runs.jsonl').read_text().splitlines()if x]
def mechanism(records):
 stages=collections.Counter();reads=collections.Counter();build=collections.Counter();receiver=collections.Counter();retained=[];prefix=[]
 for r in records:
  for c in r['calls']:
   stages[c['stage']]+=1
   if c['stage']=='dp05_interpret_user_input':
    request=read(Path(c['request_file']));prefix.append({'run_id':r['run_id'],'call_uuid':c['call_uuid'],'request':c['request_file'],'request_sha256':hashlib.sha256(Path(c['request_file']).read_bytes()).hexdigest()})
  for e in r['events']:
   d=e.get('data',{});k=e.get('kind')
   if k=='HANDOFF_SOURCE_READ':reads[d.get('consumer','UNSPECIFIED')]+=1
   if k=='HANDOFF_BUILD_BRANCH':build[d.get('branch')]+=1
   if k=='RECEIVER_BRANCH':receiver[d.get('branch')]+=1
   if k=='COMMON_ORIGINAL_EVIDENCE_RETAINED':retained.append({'run_id':r['run_id'],'elapsed_ns':e['elapsed_ns'],**d})
 return {'stage_calls':dict(stages),'actual_source_file_read_calls_by_consumer':dict(reads),'build_branches':dict(build),'receiver_branches':dict(receiver),'retention_reports':retained,'prefix_actual_requests':prefix,'scope':'Original source-file reads, not physical disk IO. Same original evidence and current rights; retaining raw passages is not semantic record production. Differences in actual model choices/service times are not isolated causal estimates.'}
def main():
 b=R/'basic-v1';a=R/'basic-dp05-reuse-union-corrected-v2';bm=[r for r in read(b/'episode-metrics.json')if r['dp']=='05'];am=read(a/'episode-metrics.json');br=[r for r in raws(b)if r['dp']=='05'];ar=raws(a);assert len(bm)==len(am)==36;rows=[]
 for v in 'ABC':
  for ep in sorted({r['episode']for r in am}):
   before=[r for r in bm if r['variant']==v and r['episode']==ep];after=[r for r in am if r['variant']==v and r['episode']==ep];assert len(before)==len(after)==3
   rows.append({'variant':v,'episode':ep,'before':detail(before),'corrected':detail(after),'before_mechanism':mechanism([r for r in br if r['variant']==v and r['episode']==ep]),'corrected_mechanism':mechanism([r for r in ar if r['variant']==v and r['episode']==ep])})
 result={'version':'WHOLE_DP05_INTRINSIC_READ_REUSE_AND_AGGREGATE_UNION_CORRECTION','original_phase':str(b),'corrected_phase':str(a),'original_preserved':True,'selected_population':'All corrected DP05 ABC36; no individual trial replacement. Compare each case only with its original three repetitions.','rows':rows,'code_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()};(R/'dp05-correction-comparison.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
 f=lambda x:'NOT_OBSERVED'if x is None else f'{x:.3f}';lines=['# DP05 전체 교정의 같은 사례 비교','','기존36과 교정36을 모두 보존한다. 실제 읽은 원문의 후속 common ledger 연결 및 B 유효 원문 supporting의 prefix 임시 view 및 실제 공급된 빈틈없는 인접 범위 합집합 검증을 보완했다. 모델/정답/캡/복구 한도는 같지만 새 provider 반복이므로 시간 차이 전체를 원문 재사용의 고립 인과효과로 보지 않는다.','','|안|사례|phase|첫 callback ms(관측/예정)|host 완료 proxy ms(관측/예정)|전체wall ms|제외receiver ms|calls|VIA USD|receiver USD|계약/완료 실패|','|---|---|---|---|---|---:|---:|---:|---:|---:|---|']
 for r in rows:
  for name,key in [('ORIGINAL','before'),('CORRECTED','corrected')]:
   x=r[key];lines.append(f"|{r['variant']}|{r['episode']}|{name}|{f(x['qa01_ms'])} ({x['qa01_observed']}/{x['qa01_expected']})|{f(x['qa03_ms'])} ({x['qa03_observed']}/{x['qa03_expected']})|{f(x['qa03_full_elapsed_ms'])}|{f(x['receiver_excluded_ms'])}|{x['calls']}|{x['via_usd']:.7f}|{x['receiver_usd']:.7f}|{x['failed_contract_or_completion_episodes']}/{x['episodes']}|")
 lines+=['','실제 소비자별 source-file read, build/receiver branch, stage calls와 retention 원시 보고는 JSON에 별도 보존한다. 첫 callback의 의미 정확성과 최종 본문 충족은 이 시간표와 별도 진단이다.'];(R/'dp05-correction-comparison.md').write_text('\n'.join(lines)+'\n');print(json.dumps({'cases':len(rows),'corrected_trials':36}))
if __name__=='__main__':main()
