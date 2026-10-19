"""Four independent actual Rust patches, finite protocol acceptance, no provider calls."""
import argparse,copy,difflib,json,shutil,subprocess,sys
from pathlib import Path
from control import HERE,RESULT,CRATE,REPO,read,sha
import run
Q=RESULT/'qa04';INPUT=HERE/'qa04_inputs'
def command(args,cwd=None):
 p=subprocess.run(args,cwd=cwd,capture_output=True,text=True)
 with (Q/'commands.jsonl').open('a')as f:f.write(json.dumps({'args':args,'cwd':str(cwd)if cwd else None,'returncode':p.returncode,'stdout':p.stdout,'stderr':p.stderr})+'\n')
 if p.returncode:raise RuntimeError(p.stdout+p.stderr)
 return p

def scratch(task,v):return Q/task/v/'candidate'
def prepare():
 if not (RESULT/'freeze.json').exists():raise RuntimeError('Freeze formal baseline before independent QA04 tasks')
 constants=read(INPUT/'control-constants.json');Q.mkdir(exist_ok=True)
 for task in (['C01','C03','C05','C06']if HERE.name=='dp01'else['C01','C03','C04','C06']):
  for v in 'ABC':
   out=scratch(task,v)
   if out.exists():raise RuntimeError('Preserve prior patch attempt')
   shutil.copytree(CRATE,out,ignore=shutil.ignore_patterns('target'));(out/'src/bin').mkdir(exist_ok=True)
   if task=='C01':
    (out/'src/model_wire.rs').write_text(constants['WIRE']);(out/'src/bin/contract_probe.rs').write_text(constants['C01PROBE'])
   elif task=='C03':
    f=out/'src/source_contract.rs';s=f.read_text();f.write_text(s[:s.index('pub fn normalize(')]+constants['NORMALIZER']);f=out/'src/main.rs';s=f.read_text();needle='let input:Value=serde_json::from_str(&line).unwrap();';assert needle in s;f.write_text(s.replace(needle,needle+'\n    let input=source_contract::normalize_input(&input);'));(out/'src/bin/contract_probe.rs').write_text(constants['C03PROBE'])
   else:command(['/usr/bin/patch','-p1','--batch','-i',str((INPUT/(task+'-'+v+'.patch')).resolve())],out)
   changed=[];patch=[]
   for p in (out/'src').rglob('*.rs'):
    if 'bin'in p.relative_to(out).parts:continue
    base=CRATE/p.relative_to(out)
    if not base.exists()or base.read_bytes()!=p.read_bytes():
     changed.append(str(p.relative_to(out)));patch.extend(difflib.unified_diff(base.read_text().splitlines(True)if base.exists()else[],p.read_text().splitlines(True),fromfile='frozen/'+str(p.relative_to(out)),tofile='patched/'+str(p.relative_to(out))))
   (out.parent/'actual.patch').write_text(''.join(patch));(out.parent/'manifest.json').write_text(json.dumps({'baseline_freeze_sha256':sha(RESULT/'freeze.json'),'changed_operational_modules':sorted(changed),'count':len(changed),'source_sha256':{str(p.relative_to(out)):sha(p)for p in out.rglob('*.rs')},'independent_patch':True},indent=2))
 print('Prepared12 independent Rust patch copies')

def migrate(value):
 if isinstance(value,list):return[migrate(v)for v in value]
 if not isinstance(value,dict):return value
 x={k:migrate(v)for k,v in value.items()}
 if'id'in x and'revision'in x and('path'in x or'text'in x):x['version']={'number':x.pop('revision')}
 return x

def execute(task,v,case):
 run.CRATE=scratch(task,v);client=run.OfflineClient(Q/task/v/'runtime');rec=run.execute(client,case,v,1,'QA04_FRESH_OFFLINE');assert not any(e.get('kind')in['EPISODE_ERROR','GOAL_FAILED']for e in rec['events']),rec['events'];return rec

def test():
 for task in (['C01','C03','C05','C06']if HERE.name=='dp01'else['C01','C03','C04','C06']):
  for v in 'ABC':
   crate=scratch(task,v);command(['cargo','test','--offline','--manifest-path',str(crate/'Cargo.toml')]);command(['cargo','build','--offline','--manifest-path',str(crate/'Cargo.toml')])
   if task in ['C01','C03']:
    case=copy.deepcopy(read(HERE/'fixtures.json')[0 if HERE.name=='dp01'else 1]);case['variant']=v
    if task=='C01':command([str(crate/'target/debug/contract_probe')]);outgoing=case
    else:
     outgoing=migrate(case);a=crate.parent/'migrated.json';b=crate.parent/'expected.json';a.write_text(json.dumps(outgoing,ensure_ascii=False));b.write_text(json.dumps(case,ensure_ascii=False));command([str(crate/'target/debug/contract_probe'),str(a),str(b)])
    assert any(e.get('kind')=='GOAL_COMPLETE'for e in execute(task,v,outgoing)['events'])
   if task=='C05':
    case=copy.deepcopy(read(HERE/'fixtures.json')[0]);sid=case['events'][1]['goal_id'];case['events'].append({'kind':'query_delivered_provenance','goal_id':'lookup','stimulus_id':'lookup','response_goal_id':sid});case['events'].append({'kind':'query_delivered_provenance','goal_id':'missing','stimulus_id':'missing','response_goal_id':'unpresented'});case['events'].append(copy.deepcopy(read(HERE/'fixtures.json')[-1]['events'][-2]));case['events'].append({'kind':'query_delivered_provenance','goal_id':'stale','stimulus_id':'stale','response_goal_id':sid});rec=execute(task,v,case);lookups=[e['data']['provenance']for e in rec['events']if e.get('kind')=='DELIVERED_PROVENANCE_LOOKUP'];assert lookups[0]['status']=='DELIVERED'and lookups[0]['source_spans'][0]['current_use_status']=='CURRENT';assert lookups[1]['status']=='NO_MATCH';assert lookups[2]['source_spans'][0]['current_use_status']=='STALE'
 # Feature driver uses independently rebased sources and actual source-backed fixture flow.
 command([sys.executable,str(HERE/'qa04_features.py')])
 rows=[]
 for task in (['C01','C03','C05','C06']if HERE.name=='dp01'else['C01','C03','C04','C06']):
  for v in 'ABC':rows.append({'task':task,'variant':v,'status':'PASS',**read(scratch(task,v).parent/'manifest.json')})
 out={'rows':rows,'mean_changed_modules_by_variant':{v:sum(r['count']for r in rows if r['variant']==v)/4 for v in 'ABC'},'provider_calls':0,'claim':'Fresh independent patches and acceptance against new formal source; model semantic/latency not measured by these tests.'};(Q/'summary.json').write_text(json.dumps(out,indent=2));print(json.dumps(out['mean_changed_modules_by_variant']))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','test']);a=p.parse_args();prepare()if a.action=='prepare'else test()
