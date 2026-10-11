#!/usr/bin/env python3
"""Compile/check one shared, mechanism-selected activity; no candidate execution."""
from collections import Counter
from fractions import Fraction as F
from pathlib import Path
import argparse,hashlib,json,re,copy
ROOT=Path(__file__).resolve().parents[2]
DIR=ROOT/'docs/architecture/12-decisions/decision-packages'
INPUT=DIR/'03-02-common-workload.json';SOURCE=DIR/'03-02-quality-attribute-definitions.md';OUTPUT=DIR/'03-02-workload-manifest.json'

def case_specs(source):
 return {i:(name,b.strip()) for i,name,b in re.findall(r'##### (S[1-6]-\d{2}) ([^\n]+)\n(.*?)(?=\n<a id=|\n#### 3\.2\.3)',source,re.S)}

def compile_workload(data,source):
 assert data['contract_version']=='ASR-QA-v6' and data['workload_id']=='P21'
 assert data['independent_repeats']==5
 assert not any(k in data for k in ('scored_run','blocks_per_activity','extra_user_by_block','burst_cases'))
 assert data['initial']['active_task_ceiling']==10 and data['initial']['cloud_call_ceiling']==4
 assert (data['source_changes_per_activity'],data['maintenance_ticks_per_activity'])==(4,2)
 assert (data['drain_deadline_seconds'],data['idle_probe_seconds'])==(30,10)
 cases={c['case_id']:c for c in data['cases']};specs=case_specs(source)
 assert len(cases)==len(data['cases'])==21 and set(cases)==set(specs)
 assert Counter(c[:2] for c in cases)==dict(S1=2,S2=5,S3=3,S4=3,S5=4,S6=4)
 assert set(data['sequence'])==set(cases) and len(data['sequence'])==len(cases)
 for a,b in data['dependency_edges']:assert data['sequence'].index(a)<data['sequence'].index(b)
 assert data['accuracy_conditions']==[f'C{i}' for i in range(1,7)]
 assert data['attribution_exclusions']==['EXTERNAL_AGENT_WORK','EXTERNAL_USER_WAIT','EXTERNAL_USER_SPEECH']
 reactions=[];cycles=[];occurrences=[]
 for cid in data['sequence']:
  c=cases[cid];name,spec=specs[cid]
  assert name==c['name'] and all(k in spec for k in ('**초기 상황:**','**입력:**','**사건 순서:**'))
  assert all(len(re.findall(fr'^\| C{i} ',spec,re.M))==1 for i in range(1,7)),cid
  assert c['trigger'] in ('USER','AGENT') and bool(c['cycle'])==(c['trigger']=='USER')
  assert c['cycle_exclusion'] is None if c['cycle'] else bool(c['cycle_exclusion'])
  assert c['reactions'] and all(r['start'] and r['end'] for r in c['reactions'])
  assert c['sensitive_dps'] and set(c['sensitive_dps'])<=set(['41','42','44','45'])
  assert c['selection_role'] in ('SENSITIVE','CONTROL') and c['selection_reason']
  assert c['name'] in source and c['selection_reason'] in source
  for r in c['reactions']:
   assert r['start']+' → '+r['end'] in source,(cid,r)
   reactions.append(dict(case_id=cid,**r))
  if c['cycle']:
   assert c['cycle']['end'] in source;cycles.append(dict(case_id=cid,**c['cycle']))
  for k in ('root_voice_seconds','planned_followup_seconds','voice_output_reference_seconds'):assert c[k]>=0
  occurrences.append(dict(case_id=cid,trigger=c['trigger'],accuracy_sample=True,conditions=data['accuracy_conditions'],canonical_case_sha256=hashlib.sha256(spec.encode()).hexdigest(),reaction_ids=[r['id'] for r in c['reactions']],cycle_id=c['cycle']['id'] if c['cycle'] else None))
 assert len(reactions)==27 and len(cycles)==19
 assert len({r['id'] for r in reactions})==27 and len({r['id'] for r in cycles})==19
 assert sum('acoustic onset' in r['start'] for r in reactions)==2
 repeats=data['independent_repeats']
 ordinary=sum(r['case_id']!='S5-04' for r in reactions)*repeats
 burst=sum(r['case_id']=='S5-04' for r in reactions)*repeats
 stops=sum('acoustic onset' in r['start'] for r in reactions)*repeats
 # The ordinary 95% guard uses this population, rather than copied counts.
 assert ordinary+burst==len(reactions)*repeats
 assert data['reaction_guards']==dict(ordinary_trials=ordinary,
  ordinary_within_2_seconds_min=(ordinary*95+99)//100,burst_trials=burst,
  burst_within_5_seconds_min=burst,local_stop_trials=stops,
  local_stop_max_seconds=0.1,missing_max=0)
 for dp in ['41','42','44','45']:assert any(dp in c['sensitive_dps'] and c['selection_role']=='SENSITIVE' for c in cases.values())
 root=sum(c['root_voice_seconds'] for c in cases.values());answers=sum(c['planned_followup_seconds'] for c in cases.values());voice_out=sum(c['voice_output_reference_seconds'] for c in cases.values())
 assert (root,answers,voice_out)==(44,6,179)
 repeats=data['independent_repeats'];N=len(cases)
 summary=dict(user_input_scenes=19,agent_triggered_scenes=2,internal_agent_events=4,total_scenes=N,independent_repeats=repeats,total_scenario_trials=N*repeats,accuracy_trials=N*repeats,accuracy_conditions=6*N*repeats,reactions_per_activity=len(reactions),cycles_per_activity=len(cycles),reaction_trials=len(reactions)*repeats,cycle_trials=len(cycles)*repeats,planned_voice_utterances=21,voice_input_root_seconds=root,voice_input_followup_seconds=answers,voice_input_seconds=root+answers,voice_input_minutes=str(F(root+answers,60)),voice_output_reference_seconds=voice_out,voice_output_reference_minutes=str(F(voice_out,60)),reference_semantic_calls=sum(c['count'] for c in data['reference_semantic_calls']),source_changes=4,maintenance_ticks=2)
 assert summary['reference_semantic_calls']==62
 return dict(contract_version=data['contract_version'],workload_id=data['workload_id'],evidence_level='DEFINED_INPUT_BLUEPRINT_NOT_EXECUTION_TRACE',source_sha256=hashlib.sha256(source.encode()).hexdigest(),input_sha256=hashlib.sha256(INPUT.read_bytes()).hexdigest(),summary=summary,category_cases=dict(Counter(c[:2] for c in cases)),occurrences=occurrences,reactions=reactions,cycles=cycles)

def self_check(data,source):
 compile_workload(data,source)
 # Meaningful negative checks: an omitted load, missing reaction, unscored reuse,
 # changed before-state ceiling, duplicate or invalid dependence must fail.
 for mutate in [lambda d:d.update(independent_repeats=15),lambda d:d['cases'].pop(),lambda d:d['cases'][0]['reactions'].clear(),lambda d:d['initial'].update(cloud_call_ceiling=8),lambda d:d['sequence'].reverse(),lambda d:d['cases'][0].update(trigger='AGENT'),lambda d:d['reaction_guards'].update(burst_trials=25),lambda d:d['reaction_guards'].update(ordinary_trials=115),lambda d:d['reaction_guards'].update(ordinary_within_2_seconds_min=99),lambda d:d['attribution_exclusions'].append('VIA')]:
  d=copy.deepcopy(data);mutate(d)
  try:compile_workload(d,source)
  except (AssertionError,KeyError):pass
  else:raise AssertionError('Invalid common activity accepted')

def main():
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');p.add_argument('--check',action='store_true');args=p.parse_args()
 data=json.loads(INPUT.read_text());source=SOURCE.read_text();self_check(data,source);result=compile_workload(data,source);payload=json.dumps(result,ensure_ascii=False,indent=2)+'\n'
 if args.write:OUTPUT.write_text(payload)
 else:assert OUTPUT.read_text()==payload,'Stale common activity blueprint'
 print('PASS: P21 2/5/3/3/4/4 cases, all 105/630 scored, 135 reactions/95 cycles, one 5-repeat activity, voice50/179 seconds')
if __name__=='__main__':main()
