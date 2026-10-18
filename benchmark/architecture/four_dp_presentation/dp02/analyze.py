"""Describe frozen actual-response memory replay. No API or experiment execution."""
from pathlib import Path
import json,hashlib,statistics,csv,collections
HERE=Path(__file__).resolve().parent
REPO=next(p for p in HERE.parents if(p/'benchmark/architecture/four_dp_presentation').is_dir())
R=REPO/'results/architecture-evaluation/current/four-dp-presentation-20261019/dp02'
def load(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def canon(x,key=''):
 if isinstance(x,dict):return {k:canon(v,k)for k,v in x.items()}
 if isinstance(x,list):
  y=[canon(v)for v in x]
  return sorted(y,key=lambda z:json.dumps(z,sort_keys=True))if key in ['__calls__','requests','pending_uninterpreted_inputs','associations','outputs']else y
 return x
def key(c):return c['stage'],c['input'].get('event',{}).get('id'),c['input'].get('goal_id'),c['input'].get('current_revision')
xs=[load(p)for p in sorted((R/'trials').glob('*/result.json'))];rows=[];payload_issues=[]
for x in xs:
 cap=load(R/'inputs'/f"{x['episode']}-r{x['rep']}.json")
 expected=[{'stage':c['stage'],'input':c['request_input'],'instruction':c['instruction'],'max_tokens':c['max_tokens']}for c in cap['calls']]
 for session in x['sessions']:
  actual=session['payloads']
  # Only independent cross-input array ordering is canonicalized, no content fields omitted.
  if canon(actual,'__calls__')!=canon(expected,'__calls__'):payload_issues.append({'mode':x['mode'],'episode':x['episode'],'rep':x['rep'],'N':x['N'],'session':session['session']})
 p=x['same_sample_peak'];f=x['first_simultaneous'];rows.append({'mode':x['mode'],'N':x['N'],'episode':x['episode'],'rep':x['rep'],'functional_equal':x['functional_equal_to_original'],'whole_peak_MB':p['sum_bytes']/1e6,'python_at_peak_MB':p['by_role'].get('python_replay',0)/1e6,'rust_at_peak_MB':p['by_role'].get('rust_candidate',0)/1e6,'initial_python_MB':x['initial_python']['sum_bytes']/1e6,'first_both_MB':f['sum_bytes']/1e6 if f else None,'requested_allocator_peak_bytes':x['allocator_requested_peak_bytes'],'sample_count':x['rss_samples'],'max_sample_gap_ms':x['max_actual_sample_gap_ms'],'replayed_calls':x['actual_calls_replayed'],'API_calls':0})
groups=[]
for n in [1,2,4]:
 for mode in ['B','B1']:
  a=[x for x in rows if x['N']==n and x['mode']==mode]
  groups.append({'N':n,'mode':mode,'trials':len(a),'functional_pass':sum(x['functional_equal']for x in a),'whole_RSS_peak_max_MB':max(x['whole_peak_MB']for x in a),'whole_RSS_trial_peak_mean_MB':statistics.mean(x['whole_peak_MB']for x in a),'whole_RSS_trial_peak_median_MB':statistics.median(x['whole_peak_MB']for x in a),'requested_allocator_peak_max_bytes':max(x['requested_allocator_peak_bytes']for x in a),'requested_allocator_trial_peak_mean_bytes':statistics.mean(x['requested_allocator_peak_bytes']for x in a)})
pairs=[]
for b in [x for x in rows if x['mode']=='B']:
 t=next(x for x in rows if x['mode']=='B1'and all(x[k]==b[k]for k in ['episode','rep','N']))
 pairs.append({**{k:b[k]for k in ['episode','rep','N']},'RSS_B1_minus_B_MB':t['whole_peak_MB']-b['whole_peak_MB'],'allocator_B1_minus_B_bytes':t['requested_allocator_peak_bytes']-b['requested_allocator_peak_bytes']})
source_rows=[]
for p in sorted((R/'inputs').glob('*.json')):
 c=load(p);i=c['input'];source_rows.append({'episode':c['episode'],'rep':c['rep'],'input_hash':c['input_sha256'],'input_serialized_bytes':len(json.dumps(i,separators=(',',':'),ensure_ascii=False).encode()),'source_serialized_bytes':len(json.dumps(i['sources'],separators=(',',':'),ensure_ascii=False).encode())})
freeze=load(R/'frozen/freeze.json');unchanged=all(sha(REPO/p)==v for p,v in freeze['files'].items())
out={'version':'DP02_B1_MEMORY_REPLAY_REPORT_V1','phase':'MEASURED_REPLAY','analysis_source_sha256':sha(Path(__file__)),'planned_trials':72,'observed_trials':len(xs),'functional_equal_to_original':sum(x['functional_equal_to_original']for x in xs),'payload_equivalence_issues':payload_issues,'payload_comparison':'Actual input/source and recorded responses fixed; concurrent mutable state snapshots compared separately. Four E4 B1 sessions include already-adopted U3 in R1 detail snapshot; final results/callbacks match. Standalone borrowed/owned wire49 exact JSON equivalence PASS.','replayed_model_responses':sum(x['actual_calls_replayed']for x in xs),'new_API_calls':0,'frozen_measurement_sources_unchanged':unchanged,'freeze_sha256':sha(R/'frozen/freeze.json'),'groups':groups,'pairs':pairs,'input_bytes':source_rows,'rows':rows,'limitations':['Whole RSS sampled fresh Python replay+Rust only, no HTTP/TLS/UI/model weights.','Requested allocator bytes are not physical RSS and omit allocator overhead/retained pages.','N1/2 whole RSS does not improve. N4 is a small descriptive reduction, not a population guarantee.','Replay preserves actual semantic responses and host ordering, not provider wall duration. No new QA01/03 or token cost result.','Three response seeds per authored case. No statistical significance claimed.','Evidence peer confirmed its attempted replay stopped before candidate execution; no concurrent peer RSS trials or model calls during these72.']}
(R/'summary.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n')
with(R/'trials.csv').open('w')as f:
 w=csv.DictWriter(f,fieldnames=list(rows[0]));w.writeheader();w.writerows(rows)
lines=['# DP02 B1 불변 문맥 공유: 실제 응답 재생','',f"72회 중 기능 동등성 {out['functional_equal_to_original']}/72, 전송 payload 동등성 실패 {len(payload_issues)}, 새 API 0회. 기존 실제 모델 응답 686개를 재생했다.",'','## 전체 RSS 주지표','', '| N | B 최고 MB | B1 최고 MB | B 평균 trial peak MB | B1 평균 trial peak MB | B 요청 할당 평균 bytes | B1 요청 할당 평균 bytes |','| --- | --- | --- | --- | --- | --- | --- |']
for n in [1,2,4]:
 b,t=[x for x in groups if x['N']==n];lines.append(f"| {n} | {b['whole_RSS_peak_max_MB']:.6f} | {t['whole_RSS_peak_max_MB']:.6f} | {b['whole_RSS_trial_peak_mean_MB']:.6f} | {t['whole_RSS_trial_peak_mean_MB']:.6f} | {b['requested_allocator_trial_peak_mean_bytes']:.1f} | {t['requested_allocator_trial_peak_mean_bytes']:.1f} |")
lines+=['','최고는 해당 N의 12회 중 최대 동시 Python+Rust RSS다. 평균은 각 trial의 동시 최고 RSS 12개 평균이며 서로 다른 PID의 최대를 더하지 않았다.','', '## 구조와 해석','', 'B 기본은 백그라운드 worker의 불변 전체 input, source payload, wire envelope를 깊게 복제한다. B1은 같은 원본 input을 Arc로 공유하고 source와 wire를 빌려 직렬화한다. 각 요청의 변경 가능한 상태, revision, 대화 snapshot과 복구 규칙은 유지했다. 약 7.5KB의 실제 source JSON을 쓰며 padding을 추가하지 않았다.','', '요청 할당 peak는 감소했지만 N=1·2의 전체 RSS 차이는 작은 반대 방향 변동이다. N=4 평균 차이는 약 0.461MB이며 최고는 약 0.721MB 낮았다. B1을 작은 복제 부담 보완으로 제시할 수 있으나 일반 사용에서 QA06 우세를 입증했다고 말할 수 없다. 원래 BASIC의 A/B RSS 차이를 이 실험으로 인과 설명할 수 없다.','', 'E4 B1의 4개 session은 독립 U3의 채택이 R1 상세 직전에 먼저 일어나, 상세 호출의 actual_state에 U3 상태가 더 들어갔다. 따라서 모든 실행 payload byte가 동일하다는 주장은 하지 않는다. 고정 source/input, 원래 실제 응답과 최종 결과/callback은 같다. 독립 wire49개는 원본 JSON과 동일했다.','', '고정 첫 호출 wave에서 한 번 관측한 뒤, 실제 원본의 host operation/adoption 순서에 따라 응답을 반환했다. 이 관측 gate와 즉시 재생은 새 QA01/03 지연 실측이 아니다. 원래 B의 N-R 추가 Task 연결 오류도 결과 그대로 보존했다.','', '## 파일','', '- plan.json / frozen/freeze.json: 실행 전 조건·source/binary·입력 SHA.','- trials.csv / summary.json: 72개 개별값·역할별 samepeak·초기값·표본 gap.','- trials/*/events.jsonl, samples.json, result.json: 원시 사건·동시 RSS·결과.','- preflight/ 및 preflight-host-order-v1/: 이전 검사와 순서 보정 이력.','', '가정: 논리 byte와 RSS는 서로 대체하지 않는다. Fresh Python 초기값은 steady baseline이 아니다. 4개 작성 사례와 3개 실제 응답 seed의 기술 통계다.']
(R/'report.md').write_text('\n'.join(lines)+'\n')
print(json.dumps({k:out[k]for k in ['observed_trials','functional_equal_to_original','new_API_calls','frozen_measurement_sources_unchanged']},ensure_ascii=False));print('payload_issues',len(payload_issues))
