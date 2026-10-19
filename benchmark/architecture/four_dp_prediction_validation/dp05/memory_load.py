"""NATIVE_OFFLINE load diagnostics: actual candidate with explicit80ms state observation holds.
No API/model accuracy or response-time observation; logical source/state bytes are not RSS.
"""
import argparse,copy,json,time
from pathlib import Path
from control import HERE,RESULT,CRATE,read,sha
import run
class Hold(run.OfflineClient):
 def call(self,payload,meta):
  time.sleep(.08) # only gives50ms sampler an observation window; never a service latency measurement.
  return super().call(payload,meta)
def main():
 p=argparse.ArgumentParser();p.add_argument('--load',type=int,choices=[1,4,8],required=True);p.add_argument('--variant',choices=list('ABC'),required=True);p.add_argument('--rep',type=int,default=1);a=p.parse_args()
 base=copy.deepcopy(read(HERE/'fixtures.json')[0 if HERE.name=='dp01'else 1]);sources=[]
 for i in range(a.load):
  src=copy.deepcopy(base['events'][0]['source']);src.update(id='load-'+str(i),path=str(HERE/'sources'/'memory-load.txt'),line_count=256,outline=[{'section':'bounded load evidence','lines':[1,256]}]);src['allowed_lines']=[1,256];sources.append({'kind':'source','source':src})
 if HERE.name=='dp01':
  goal=base['events'][-1];goal.update(utterance='공급된 모든 load 자료에서 첫 번째 줄에 적힌 사실을 설명해줘.',expression='모든 load 자료');events=sources+[goal]
 else:
  state=base['events'][1];state['state']['required_source_ids']=[s['source']['id']for s in sources];goal=base['events'][-1];goal['utterance']='공급된 모든 load 자료의 첫 번째 사실로 짧은 초안을 작성해줘. 실제 외부 행동은 하지 마.';events=sources+[state,goal]
 case={**base,'id':HERE.name+'-native-load-'+str(a.load),'events':events};root=RESULT/'memory-load'/f'L{a.load}-{a.variant}-r{a.rep}';root.mkdir(parents=True,exist_ok=True);run.CRATE=CRATE;rec=run.execute(Hold(root),case,a.variant,1,'NATIVE_OFFLINE_MEMORY_LOAD')
 samples=[s for s in rec['memory']if s.get('all_registered_observed')];peak=max(samples,key=lambda s:s['concurrent_sum_bytes'])if samples else None
 states=[{'at_ns':s['at_ns'],'bytes':sum(v['bytes']for v in s.get('logical_buffers',{}).values()if v.get('owner')=='rust_candidate'),'entries':{k:v for k,v in s.get('logical_buffers',{}).items()if v.get('owner')=='rust_candidate'}}for s in samples]
 out={'load_sources':a.load,'source_bytes_each':(HERE/'sources/memory-load.txt').stat().st_size,'variant':a.variant,'rep':a.rep,'same_time_peak_sample':peak,'logical_rust_peak_sample':max(states,key=lambda s:s['bytes'])if states else None,'trace':str(root/'raw/runs'/rec['run_id']/'trace.json'),'failures':[e for e in rec['events']if e.get('kind')in['GOAL_FAILED','EPISODE_ERROR']],'artificial_observation_hold_ms':80,'provider_calls':0,'claim':'Actual native offline allocation/RSS under controlled synthetic load, not model latency/quality or product/global model memory.'};(root/'summary.json').write_text(json.dumps(out,indent=2));print(json.dumps({'load':a.load,'variant':a.variant,'rep':a.rep,'rss_peak':peak['concurrent_sum_bytes']if peak else None,'logical_peak':out['logical_rust_peak_sample']['bytes']if out['logical_rust_peak_sample']else None}))
if __name__=='__main__':main()
