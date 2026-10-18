"""Select one actual concurrent-peak sample per phase/option; never sum per-PID maxima."""
import argparse,csv,json
from pathlib import Path
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir());ROOT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
def category_lifecycle_max(trace):
 live={};maximum={}
 for e in json.loads(Path(trace).read_text()).get('logical_buffer_events',[]):
  if e.get('action')=='RELEASE':live.pop(e['name'],None)
  elif e.get('bytes') is not None:live[e['name']]={'category':e.get('category','unknown'),'bytes':e['bytes']}
  sums={}
  for v in live.values():sums[v['category']]=sums.get(v['category'],0)+v['bytes']
  for k,v in sums.items():maximum[k]=max(maximum.get(k,0),v)
 return maximum

def main(root):
 out=[]
 for phase in sorted(root.iterdir()):
  p=phase/'memory-breakdown.json'
  if not p.exists() or not (phase/'MODEL-CALLS-AUTHORIZED.json').exists():continue
  rows=json.loads(p.read_text());groups={}
  for r in rows:groups.setdefault((r['phase'],r['dp'],r['variant'],r.get('tactic')),[]).append(r)
  selected=[]
  for key,group in sorted(groups.items(),key=lambda item:str(item[0])):
   r=max(group,key=lambda x:x['same_sample_sum_mb']or 0);cats=r['logical_peak_categories_bytes']
   lifecycle={x['run_id']:category_lifecycle_max(x['raw_trace'])for x in group}
   x={'phase_folder':phase.name,'phase':key[0],'dp':key[1],'variant':key[2],'tactic':key[3],'trials':len(group),'incomplete_trials':sum(x['incomplete']for x in group),'incomplete_definition':'No same-sample observation of both registered Python/Rust roles in this trial. Separate from GOAL_FAILED/functional failures; zero does not prove every sample or true peak was fully observed.','peak_run_id':r['run_id'],'peak_sample_start_ns':r['peak_sample_start_ns'],'peak_sample_end_ns':r['peak_sample_end_ns'],'peak_stage':r['peak_stage'],'concurrent_peak_mb':r['same_sample_sum_mb'],'python_at_same_peak_mb':r['peak_python_mb'],'rust_at_same_peak_mb':r['peak_rust_mb'],'same_run_initial_sum_mb':r['initial_simultaneous_mb'],'same_run_initial_python_mb':r['initial_python_mb'],'same_run_initial_rust_mb':r['initial_rust_mb'],'sum_peak_minus_initial_mb':r['same_sample_sum_mb']-r['initial_simultaneous_mb']if r['same_sample_sum_mb']is not None and r['initial_simultaneous_mb']is not None else None,'role_delta_bytes':r['same_role_delta_bytes'],'logical_categories_at_same_peak_bytes':cats,'logical_buffers_at_same_peak':r['logical_peak_buffers'],'experiment_retained_evidence_bytes':cats.get('retained_evidence',0),'logical_image_bytes':cats.get('image',0),'peak_trial_category_lifecycle_max_bytes':lifecycle[r['run_id']],'option_any_trial_logical_image_max_bytes':max((x.get('image',0)for x in lifecycle.values()),default=0),'peak_trial_logical_image_max_bytes':lifecycle[r['run_id']].get('image',0),'logical_rust_serialized_bytes':cats.get('rust_serialized',0),'pid_ppid_role_contributions':r['peak_process_pid_ppid_roles'],'peak_all_registered_observed':r['peak_all_registered_observed'],'peak_trial_samples':r['sample_count'],'peak_trial_both_process_samples':r['both_process_sample_count'],'peak_trial_max_actual_gap_ms':r['actual_max_sample_gap_ms'],'option_max_actual_sample_gap_ms':max((z['actual_max_sample_gap_ms']for z in group if z['actual_max_sample_gap_ms']is not None),default=None),'raw_trace':r['raw_trace'],'raw_input':r['raw_input'],'all_trial_breakdown':str(p.resolve()),'scope':'One actual max concurrent sample across trials; initial sample is fresh-run startup, not steady baseline. Logical serialized bytes overlap and do not partition RSS. Experiment events/calls are evidence overhead. Nominal 50ms ps RSS, shared-page dedup unverified; excludes UI, outer orchestrator, cloud weights.'}
   if phase.name=='diagnostic-dp05-numbered-lines':
    x['architecture_ram_comparable']=False;x['startup_scope_warning']='Fresh worker prerequisites eagerly parses prior full runs.jsonl records before execute. Python initial RSS is already about694MB; exact allocator attribution unprofiled. Preserve same-sample values but do not attribute ~704MB to formatter/candidate state or compare as BASIC QA06 ranking.'
   else:x['architecture_ram_comparable']=True;x['startup_scope_warning']=None
   selected.append(x);out.append(x)
  save(phase/'memory-option-peaks.json',selected)
 save(root/'analysis/memory-option-peaks.json',out)
 flat=[]
 for r in out:flat.append({k:json.dumps(v,ensure_ascii=False)if isinstance(v,(dict,list))else v for k,v in r.items()})
 with (root/'analysis/memory-option-peaks.csv').open('w')as f:
  if flat:w=csv.DictWriter(f,fieldnames=list(flat[0]));w.writeheader();w.writerows(flat)
 lines=['# 안별 같은 시각 최대 RSS 분해','','단위 RSS는 decimal MB, 논리 직렬화 크기는 bytes다. 각 행은 해당 phase/안의 최대 동시 합계가 기록된 **하나의 실제 run과 표본 시각**이다. 독립 PID별 최대값을 더하지 않았다. 초기값은 같은 run의 첫 동시 표본이며 steady baseline이 아니다. 메모리 계측 누락은 두 역할의 동시 관측 표본이 전혀 없는 trial 수이며 GOAL_FAILED/기능 실패와 다르다. 누락0도 모든 표본이나 실제 peak의 완전 관측을 보장하지 않는다. delta는 같은 두 표본의 차이로 메모리 원인 분해가 아니다. 실험 events/calls 보관은 별도 논리 카테고리다. 논리 크기들은 중첩될 수 있고 RSS 할당량으로 해석하거나 합산할 수 없다.','','|phase|DP/안|최대 동시 MB|같은 시각 Python|같은 시각 Rust|같은 run 초기 합계|합계 차이|논리 Rust 직렬화 bytes|같은 RSS peak의 논리 이미지 bytes|해당 run 이미지 최대 bytes|안 전체 이미지 최대 bytes|논리 실험 기록 bytes|메모리 계측 누락 trial|원시 표본|','|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|---|']
 for r in out:
  fmt=lambda x:'NOT_OBSERVED'if x is None else f'{x:.6f}'
  lines.append(f"|{r['phase_folder']}|{r['dp']}{r['variant']} {r['tactic']or''}|{fmt(r['concurrent_peak_mb'])}|{fmt(r['python_at_same_peak_mb'])}|{fmt(r['rust_at_same_peak_mb'])}|{fmt(r['same_run_initial_sum_mb'])}|{fmt(r['sum_peak_minus_initial_mb'])}|{r['logical_rust_serialized_bytes']}|{r['logical_image_bytes']}|{r['peak_trial_logical_image_max_bytes']}|{r['option_any_trial_logical_image_max_bytes']}|{r['experiment_retained_evidence_bytes']}|{r['incomplete_trials']}/{r['trials']}|[trace]({r['raw_trace']})|")
 lines+=['','**행 번호 진단 RAM은 기본 구조 비교에서 제외한다.** diagnostic-dp05-numbered-lines의 fresh worker는 측정 전에 이전 전체 runs.jsonl을 선행 파싱해 초기 Python RSS부터 약694MB다. 최고 약703MB를 formatter/후보 상태 증가로 해석하지 않는다. 정확한 allocator 원인 분해는 수행하지 않았으며 peak-minus-initial도 순수 후보 효과가 아니다. 각 원시 초기/같은시각/PID 역할값과 논리 formatter copy bytes는 보존했다.','','같은 RSS peak에서 image=0은 그 시점에 해당 논리 버퍼가 해제됐다는 뜻이며 이미지를 사용하지 않았다는 뜻이 아니다. 해당 run과 안 전체의 논리 image 최대치를 나란히 제공하고 RSS peak의 기여량과 섞지 않는다. JSON/CSV는 PID·PPID·역할, 표본 시작/끝, stage, 역할별 초기값과 차이, 전체 논리 buffer 이름/카테고리, 실제 최대 표본 gap을 제공한다. 명목 polling은 50ms이며 표본 사이 최대치를 보장하지 않는다. Python에는 TLS/HTTP/allocator/직렬화와 실험 기록이 함께 들어 있다. UI/outer orchestrator/cloud weights는 계측 범위 밖이고 shared-page 중복 제거는 검증하지 않았다.']
 (root/'analysis/memory-option-peaks.md').write_text('\n'.join(lines)+'\n');print(json.dumps({'phase_options':len(out),'basic_options':sum(r['phase']=='BASIC'for r in out)}))
if __name__=='__main__':p=argparse.ArgumentParser();p.add_argument('root',type=Path,nargs='?',default=ROOT);main(p.parse_args().root)
