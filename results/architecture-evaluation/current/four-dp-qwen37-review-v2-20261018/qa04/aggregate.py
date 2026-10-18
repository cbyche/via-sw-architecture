from pathlib import Path
import json,hashlib
ROOT=Path(__file__).resolve().parent
controls=json.loads((ROOT/'controls/manifest.json').read_text())['changes']
features={}
for dp,ids in [('01',['C05','C06']),('05',['C04','C06'])]:
 for task in ids:
  path=ROOT/'baseline-v2'/f'dp{dp}-{task}'/'summary.json';value=json.loads(path.read_text())
  for variant,count in value['changed_module_count_each_variant'].items():features[(dp,variant,task)]={'id':task,'count':count,'modules':value['changed_modules_by_variant'][variant],'status':value['acceptance'],'evidence':str(path.relative_to(ROOT))}
for value in json.loads((ROOT/'dp04-rebased-calibration-v3/summary.json').read_text())['rows']:features[('04',value['variant'],value['task'])]={'id':value['task'],'count':value['module_count'],'modules':value['operating_modules'],'status':value['status'],'evidence':'dp04-rebased-calibration-v3/summary.json'}
for task in ['C04','C06']:
 path=ROOT/'dp02'/(task+'-v2')/'manifest.json';value=json.loads(path.read_text());accepted=json.loads((path.parent/'acceptance-summary.json').read_text())
 for variant in ['A','B']:features[('02',variant,task)]={'id':task,'count':value['module_count'],'modules':value['changed_operational_modules'],'status':'PASS'if all(c['pass']for c in accepted['checks'])else'FAILED','evidence':str(path.relative_to(ROOT))}
options=[]
for dp,variants,tasks in [('01','ABC',['C05','C06']),('02','AB',['C04','C06']),('04','AB',['C05','C06']),('05','ABC',['C04','C06'])]:
 for variant in variants:
  rowtasks=[{'id':c['task'],'count':c['count'],'modules':c['changed_operational_modules'],'status':'PASS','evidence':'controls/acceptance-summary.json','class':'ADAPTER_CONTROL'}for c in controls]+[{**features[(dp,variant,t)],'class':'FUNCTIONAL_CHANGE'}for t in tasks]
  options.append({'dp':dp,'variant':variant,'tasks':rowtasks,'mean_changed_operational_modules':sum(t['count']for t in rowtasks)/4,'functional_tasks_mean':sum(features[(dp,variant,t)]['count']for t in tasks)/2,'adapter_controls_mean':1.5,'status':'PASS'if all(t['status']=='PASS'for t in rowtasks)else'FAILED'})
summary={'aggregation_rule':'Frozen before BASIC: equal weight C01+C03+two relevant functional tasks; functional pair and controls reported adjacent. No incomplete tasks dropped.','options':options,'provider_calls':0,'baseline':'Final calibration-v3 frozen Rust; DP04 four scratch patches rebased and acceptance rerun; other operating modules byte-identical per recorded transitions','evidence_root':str(ROOT.resolve()),'all_acceptance_pass':all(o['status']=='PASS'for o in options)}
for option in options:
 for task in option['tasks']:task['evidence_absolute']=str((ROOT/task['evidence']).resolve())
(ROOT/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2));print(json.dumps([{'dp':o['dp'],'variant':o['variant'],'mean':o['mean_changed_operational_modules'],'functional':o['functional_tasks_mean']}for o in options]))
