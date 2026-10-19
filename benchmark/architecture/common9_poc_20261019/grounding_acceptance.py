"""Actual Rust grounding protocol acceptance with test-only authored vision/speech doubles.
The double reads SVG geometry only in this offline checker, never in the live adapter.
No evaluator oracle, cloud request or model-quality/time result is produced.
"""
from pathlib import Path
import subprocess,json,hashlib,xml.etree.ElementTree as ET,copy,re,sys,time,uuid
BASE=Path(__file__).resolve().parent;REPO=BASE.parents[2];OUT=REPO/'results/architecture-evaluation/current/common9-poc-20261019/grounding-offline-reruns'/(time.strftime('%Y%m%dT%H%M%S')+'-'+uuid.uuid4().hex[:6]);CRATE=Path('/private/tmp/via-common9-grounding-acceptance');(CRATE/'src').mkdir(parents=True,exist_ok=True);OUT.mkdir(parents=True,exist_ok=True)
source=REPO/'prototypes/candidates/common9_poc_20261019/src'
(CRATE/'Cargo.toml').write_text('[package]\nname="grounding_acceptance"\nversion="0.1.0"\nedition="2021"\n[dependencies]\nserde_json="1"\n')
(CRATE/'src/main.rs').write_text('\n'.join(f'#[path="{source/n}.rs"]mod {n};'for n in['api','context','model_wire','grounding'])+r'''
use serde_json::{json,Value};use std::io::{self,BufRead};
fn main(){let mut line=String::new();io::stdin().lock().read_line(&mut line).unwrap();let v:Value=serde_json::from_str(&line).unwrap();let x=&v["input"];let input=api::GroundingInput{event_id:x["event_id"].as_str().unwrap().into(),revision:x["revision"].as_u64().unwrap(),transcript:x["transcript"].as_str().unwrap().into(),frames:x["frames"].as_array().unwrap().clone(),cues:x["cues"].as_array().unwrap().clone(),prior:x["prior"].clone(),evidence:Value::Null};let mut c=context::Context::new();match grounding::ground(&mut c,v["variant"].as_str().unwrap(),&input){Ok(r)=>c.event("COMPONENT_RESULT",&r),Err(e)=>c.event("COMPONENT_FAILED",&json!({"error":e}))}}
''')
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
p=subprocess.run(['cargo','build','--offline','--manifest-path',str(CRATE/'Cargo.toml')],capture_output=True,text=True);(OUT/'build.log').write_text(p.stdout+p.stderr);assert p.returncode==0
observations=json.loads((BASE/'assets/observations.json').read_text())['events']
def candidates(frames):
 out=[]
 for f in frames:
  ns=list(ET.parse(Path(f['image_path']).with_suffix('.svg')).getroot().iter());rects=[n for n in ns if n.tag.endswith('rect')and'rx'in n.attrib]
  for n in ns:
   if not n.tag.endswith('text'):continue
   label=''.join(n.itertext());m=re.match(r'^([TP]\d+) / ',label)
   if not m:continue
   x=float(n.attrib['x']);y=float(n.attrib['y']);box=next(r for r in rects if float(r.attrib['x'])<=x<float(r.attrib['x'])+float(r.attrib['width'])and float(r.attrib['y'])<=y<float(r.attrib['y'])+float(r.attrib['height']));a,b,w,h=[float(box.attrib[k])*2 for k in['x','y','width','height']]
   out.append({'item_id':m[1],'frame_id':f['frame_id'],'label':label,'coordinate_space':'normalized_0_1000','bbox_2d':[round(a/f['width']*1000),round(b/f['height']*1000),round((a+w)/f['width']*1000),round((b+h)/f['height']*1000)]})
 return out
def refs(text,cues):
 if '보존할 표' in text:return[{'expression':'첫째','relation':'prior_reference','cue_ids':[],'count':1,'meaning':'first prior preserved table'},{'expression':'이 표','relation':'point_inside','cue_ids':['point-current'],'count':1,'meaning':'current indicated table'}]
 if '표들은' in text:return[{'expression':'이 표들','relation':'native_selection','cue_ids':['native-selection'],'count':None,'meaning':'preserved selected tables'},{'expression':'저 문단','relation':'point_inside','cue_ids':['point-later'],'count':1,'meaning':'final self-corrected paragraph'}]
 return[{'expression':'이 문단','relation':'native_selection','cue_ids':['native-selection'],'count':1,'meaning':'selected paragraph'}]
def bind(references,cs,cues,frames,prior=None):
 out=[]
 for r in references:
  cue=next((c for c in cues if r['cue_ids']and c['event_id']==r['cue_ids'][0]),None)
  if r['relation']=='native_selection':chosen=[c for c in cs if c['item_id']in cue['item_ids']]
  elif r['relation']=='point_inside':
   if cue is None:raise ValueError('Offline binder requires actual supplied designation cue')
   chosen=[]
   for c in cs:
    if c['frame_id']!=cue['frame_id']:continue
    if 'bbox'in c:b=c['bbox'];x,y=cue['x'],cue['y']
    else:
     f=next(f for f in frames if f['frame_id']==cue['frame_id']);b=c['bbox_2d'];x=cue['x']/f['width']*1000;y=cue['y']/f['height']*1000
    if b[0]<=x<=b[2]and b[1]<=y<=b[3]:chosen.append(c)
  else:
   def walk(v):
    if isinstance(v,dict):
     yield v
     for child in v.values():yield from walk(child)
    elif isinstance(v,list):
     for child in v:yield from walk(child)
   preserved=[b for b in walk(prior or{})if b.get('expression')=='이 표들'and b.get('extents')]
   if not preserved:raise ValueError('Offline prior reference requires actually delivered preserved set')
   old=preserved[-1]['extents'];ordered=sorted(old,key=lambda c:(c.get('bbox',c.get('bbox_2d'))[1],c.get('bbox',c.get('bbox_2d'))[0]));first=ordered[0]
   chosen=[c for c in cs if (c.get('canonical_target')==first.get('canonical_target')if c.get('canonical_target')and first.get('canonical_target')else c.get('item_id')==first.get('item_id')and c.get('frame_id')==first.get('frame_id'))]
  out.append({**r,'targets':[c.get('canonical_target',c['item_id'])for c in chosen],'extents':chosen,'status':'ready'})
 return{'bindings':out,'intent':'실제 원본의 지정 범위만 처리하고 유지 조건을 보존','unknowns':[],'need_expansion':False}
def execute(event,variant,prior=None,bad_initial=False):
 obs=observations[event];text={'S08-U01':'이 표들은 그대로 두고 이 문단 아니 저 문단만 고쳐줘.','S08-U02':'보존할 표는 첫째와 이 표로 바꿔. 문단 수정은 유지해.','S15-U01':'이 문단의 핵심을 설명해줘.'}[event]
 case={'variant':variant,'input':{'event_id':event,'revision':2 if event.endswith('U02')else 1,'transcript':text,**obs,'prior':prior or{}}};p=subprocess.Popen([str(CRATE/'target/debug/grounding_acceptance')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True);p.stdin.write(json.dumps(case,ensure_ascii=False)+'\n');p.stdin.flush();events=[];calls=[]
 for line in p.stdout:
  m=json.loads(line)
  if m.get('type')=='model_call':
   payload=m['payload'];stage=m['stage'];data=payload['input']
   if stage=='grounding.local_screen_parser':result={'frames':[],'scope':'OFFLINE_STRUCTURAL_DOUBLE_NOT_REAL_PARSER'}
   elif 'speech' in stage:result={'references':refs(data['transcript'],data['cues']),'intent':'指定 범위를 처리하고 유지 조건을 보존','unknowns':[]}
   elif 'screen' in stage:result={'candidates':candidates(data['frames']),'coverage':{'complete':True,'unknowns':[]}}
   elif 'joint' in stage:result=bind(refs(data['transcript'],data['cues']),candidates(data['frames']),data['cues'],data['frames'],data.get('prior'))
   else:
    cs=copy.deepcopy(data['screen']['candidates']);fs=obs['frames']
    for c in cs:
     if 'bbox_2d'not in c:
      f=next(f for f in fs if f['frame_id']==c['frame_id']);b=c['bbox'];c['bbox_2d']=[round(b[0]/f['width']*1000),round(b[1]/f['height']*1000),round(b[2]/f['width']*1000),round(b[3]/f['height']*1000)]
    result=bind(data['speech']['references'],cs,data['cues'],fs,data.get('prior'))
   if bad_initial and stage in['grounding.joint','grounding.screen_producer']:
    es=result['candidates']if'candidates'in result else result['bindings'][0]['extents'];es[0]['bbox_2d']=[-1,0,1000,1000]
   calls.append({'stage':stage,'payload':payload,'response':result,'offline_double':True});p.stdin.write(json.dumps({'call_id':m['call_id'],'result':result},ensure_ascii=False)+'\n');p.stdin.flush()
  else:events.append(m)
 p.wait();result=next((e['data']for e in events if e.get('kind')=='COMPONENT_RESULT'),None);assert p.returncode==0 and result is not None,(events,p.stderr.read())
 assert not any(e.get('kind')in['DISPLAY_PROXY','INPUT_CAPTURED','GOAL_COMPLETE']for e in events)
 assert all(not c['payload'].get('images')for c in calls if c['stage']in['grounding.speech_producer','grounding.candidate_binding'])
 assert all('expected'not in json.dumps(c['payload']).lower()and'oracle_targets'not in json.dumps(c['payload']).lower()for c in calls)
 if event=='S15-U01'and variant=='B':assert not any(c['stage']=='grounding.local_screen_parser'for c in calls)
 if event.startswith('S08')and variant=='B':assert any(c['stage']=='grounding.local_screen_parser'for c in calls)
 return{'input':case,'calls':calls,'events':events,'result':result,'provider_calls':0}
checks=[]
for v in['A','B']:
 prior={}
 for e in['S08-U01','S08-U02','S15-U01']:
  trace=execute(e,v,prior,bad_initial=e=='S08-U01');save(OUT/(e+'-'+v+'.json'),trace);prior={'delivered':[trace['result']]};checks.append({'event':e,'variant':v,'pass':True,'bindings':trace['result']['bindings'],'calls':len(trace['calls'])})
save(OUT/'summary.json',{'checks':checks,'provider_calls':0,'scope':'Real Rust component protocol/recovery/canonical/native/noendpoint authority only; semantic doubles not accuracy/latency measurements'});print(json.dumps({'paths':len(checks),'all_pass':True,'cloud_calls':0,'output':str(OUT)}))
