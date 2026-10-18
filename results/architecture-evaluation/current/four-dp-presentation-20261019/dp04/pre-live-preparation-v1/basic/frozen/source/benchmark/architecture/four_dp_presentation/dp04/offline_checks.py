"""Authorized deterministic contract checks, never numerical model evidence."""
import copy,json,sys
from pathlib import Path
from coordinate_views import specs,map_extents,render
HERE=Path(__file__).resolve().parent
REPO=next(p for p in HERE.parents if (p/'benchmark/architecture/four_dp_poc_v2').is_dir())
ROOT=REPO/'results/architecture-evaluation/current/four-dp-presentation-20261019/dp04'
fixtures=json.loads((REPO/'benchmark/architecture/four_dp_poc_v2/fixtures/dp04.json').read_text())
checks=[]
def ok(name,value):
    checks.append({'check':name,'pass':bool(value)})
    if not value:raise AssertionError(name)
def rejects(name,fn):
    try:fn()
    except ValueError:ok(name,True)
    else:ok(name,False)
revision=fixtures[0]['revisions'][0];views=specs(revision);v=views[0]
ok('cue_crop_uses_raw_union_fixed_padding',v['crop_original_pixels']==[247,187,758,608])
without=copy.deepcopy(revision);without['events']=[]
ok('no_cue_full_frame',specs(without)[0]['crop_original_pixels']==[0,0,1000,650])
changed=copy.deepcopy(revision);changed['transcript']='unrelated';changed['expressions']=[]
ok('view_spec_independent_of_semantic_text',specs(changed)==views)
p={'adoptions':[{'extents':[{'frame_id':'dense','view_id':v['view_id'],'coordinate_space':'view_pixels','bbox':[0,0,100,100]}]}]}
out,mapping=map_extents(p,views)
ok('affine_inverse_exact',out['adoptions'][0]['extents'][0]['bbox']==[247,187,347,287])
ok('provider_object_not_mutated',p['adoptions'][0]['extents'][0]['bbox']==[0,0,100,100])
for name,key,value in [('unknown_view','view_id','unknown'),('wrong_frame','frame_id','other'),('overflow','bbox',[0,0,2000,2000]),('nan','bbox',[0,0,float('nan'),10])]:
    bad=copy.deepcopy(p);bad['adoptions'][0]['extents'][0][key]=value
    rejects('reject_'+name,lambda b=bad:map_extents(b,views))
for episode in fixtures:
    for rev in episode['revisions']:
        output,timing=render(rev,ROOT/'offline-view-checks'/episode['id']/str(rev['revision']))
        ok('actual_png_'+episode['id']+'_'+str(rev['revision']),all(Path(x['path']).read_bytes().startswith(b'\x89PNG\r\n\x1a\n') for x in output))
        ok('render_time_recorded_'+episode['id']+'_'+str(rev['revision']),timing['end_ns']>=timing['start_ns'])
ROOT.mkdir(parents=True,exist_ok=True)
(ROOT/'offline-coordinate-checks.json').write_text(json.dumps({'checks':checks,'all_pass':all(x['pass'] for x in checks),'model_calls':0,'candidate_oracle_access':False},indent=2)+'\n')
print(json.dumps({'checks':len(checks),'all_pass':True,'model_calls':0}))
