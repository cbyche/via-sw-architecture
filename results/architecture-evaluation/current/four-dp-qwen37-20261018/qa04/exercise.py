"""Actual scratch Rust contract migrations. No credentials, network, model calls or live edits."""
import copy, difflib, hashlib, json, pathlib, shutil, subprocess, tempfile, time
ROOT=pathlib.Path(__file__).resolve().parents[5]
LIVE=ROOT/'prototypes/candidates/four_dp_poc'
OUT=pathlib.Path(__file__).resolve().parent
FIXTURES=ROOT/'benchmark/architecture/four_dp_poc/fixtures'
TMP=pathlib.Path(tempfile.mkdtemp(prefix='via-qa04-'))
LOG=[]
def dump(p,x):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def run(cmd,cwd,allow_fail=False):
    p=subprocess.run(cmd,cwd=cwd,text=True,capture_output=True)
    LOG.append({'cmd':cmd,'cwd':str(cwd),'returncode':p.returncode,'stdout':p.stdout,'stderr':p.stderr})
    if p.returncode and not allow_fail:raise RuntimeError(p.stderr)
    return p
before={str(p.relative_to(LIVE)):hashlib.sha256(p.read_bytes()).hexdigest() for p in LIVE.rglob('*') if p.is_file() and 'target' not in p.relative_to(LIVE).parts}
BASE=OUT/'baseline'
BASE.mkdir(exist_ok=True)
for p in LIVE.rglob('*'):
    if p.is_file() and 'target' not in p.relative_to(LIVE).parts:
        q=BASE/p.relative_to(LIVE);q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q)
dump(OUT/'baseline-manifest.json',{'source':str(LIVE),'sha256':before,'scratch_root':str(TMP),'captured_at_unix':time.time()})
WIRE='''use serde_json::{json,Value};
pub fn encode(id:u64,stage:&str,payload:&Value)->Value { json!({"type":"model_call","call_id":id,"stage":stage,"request":{"context":payload},"wire_version":2}) }
pub fn decode(response:&Value)->Result<Value,String> {
    if let Some(error)=response.get("error") {return Err(error.to_string())}
    let envelope=response.get("provider_response").ok_or_else(||"Missing provider response envelope".to_string())?;
    let mut result=envelope.get("json").cloned().ok_or_else(||"Missing provider JSON".to_string())?;
    if let Some(usage)=envelope.get("usage") {
        result["_provider_usage"]=json!({"input_tokens":usage["input_tokens"],"output_tokens":usage["output_tokens"]});
    }
    Ok(result)
}
'''
NORMALIZER='''/// V2 provider source metadata becomes the stable canonical source/frame contract.
pub fn normalize(value:&Value)->Value {
    let mut v=value.clone();
    if let Some(version)=value.get("version") {
        let revision=version["number"].clone();
        if value.get("frame_id").is_some() {v["source_revision"]=revision;} else {v["revision"]=revision;}
        v.as_object_mut().unwrap().remove("version");
    }
    if let Some(space)=value.get("coordinate") {
        v["width"]=space["dimensions"][0].clone();v["height"]=space["dimensions"][1].clone();
        v["coordinate_space"]=json!("frame_pixels");v.as_object_mut().unwrap().remove("coordinate");
    }
    if let Some(t)=value.get("captured_at") {
        if t["unit"]=="microseconds" {v["observed_at_ms"]=json!(t["value"].as_u64().unwrap()/1000);}
        v.as_object_mut().unwrap().remove("captured_at");
    }
    v
}
/// One input-admission compatibility pass, including inline source evidence.
pub fn normalize_input(value:&Value)->Value {
    match value {
        Value::Array(xs)=>Value::Array(xs.iter().map(normalize_input).collect()),
        Value::Object(xs)=>{
            let mapped:serde_json::Map<String,Value>=xs.iter().map(|(k,v)|(k.clone(),normalize_input(v))).collect();
            let v=Value::Object(mapped);
            if v.get("version").is_some() && (v.get("frame_id").is_some() || (v.get("id").is_some() && (v.get("path").is_some() || v.get("text").is_some()))) {normalize(&v)} else {v}
        },
        _=>value.clone()
    }
}
'''
C01PROBE='''#[path="../model_wire.rs"] mod model_wire;
use serde_json::json;
fn main(){
    let p=json!({"instruction":"probe","input":{"source_revision":2}});
    let request=model_wire::encode(7,"contract_probe",&p);
    assert_eq!(request["wire_version"],2);assert_eq!(request["request"]["context"],p);
    assert!(request.get("payload").is_none());
    let result=model_wire::decode(&json!({"call_id":7,"provider_response":{"json":{"answer":"preserved"},"usage":{"input_tokens":17,"output_tokens":9}}})).unwrap();
    assert_eq!(result["answer"],"preserved");assert_eq!(result["_provider_usage"],json!({"input_tokens":17,"output_tokens":9}));
    assert!(model_wire::decode(&json!({"error":"provider unavailable"})).is_err());
    println!("PASS C01 request/context + provider JSON + usage + error propagation");
}
'''
C03PROBE='''#[path="../source_contract.rs"] mod source_contract;
use serde_json::{json,Value};
fn main(){
    let args:Vec<String>=std::env::args().collect();
    let input:Value=serde_json::from_str(&std::fs::read_to_string(&args[1]).unwrap()).unwrap();
    let expected:Value=serde_json::from_str(&std::fs::read_to_string(&args[2]).unwrap()).unwrap();
    let actual=source_contract::normalize_input(&input);assert_eq!(actual,expected);
    let frame=source_contract::normalize(&json!({"frame_id":"f","source_id":"document","version":{"number":2},"coordinate":{"unit":"pixels","dimensions":[1000,650]},"captured_at":{"value":800000,"unit":"microseconds"}}));
    assert_eq!(frame,json!({"frame_id":"f","source_id":"document","source_revision":2,"width":1000,"height":650,"coordinate_space":"frame_pixels","observed_at_ms":800}));
    assert!(source_contract::read(&json!({"id":"denied","revoked":true})).is_err());
    println!("PASS C03 exact whole-fixture canonical equality + pixel/time/revision conversion + revoke guard");
}
'''
def scratch(name):
    p=TMP/name;shutil.copytree(BASE,p);return p
def patch(name,p):
    changed=[];lines=[]
    for f in sorted((BASE/'src').rglob('*.rs')):
        rel=f.relative_to(BASE);other=p/rel
        if f.read_bytes()!=other.read_bytes():
            changed.append(str(rel));lines+=list(difflib.unified_diff(f.read_text().splitlines(True),other.read_text().splitlines(True),fromfile='baseline/'+str(rel),tofile=name+'/'+str(rel)))
    (OUT/(name+'.patch')).write_text(''.join(lines));return changed
# C01: actual baseline rejection, then one operating-module migration.
c01=scratch('C01');(c01/'src/bin').mkdir();(c01/'src/bin/contract_probe.rs').write_text(C01PROBE)
negative=run(['cargo','run','--offline','--bin','contract_probe'],c01,True)
assert negative.returncode!=0
(c01/'src/model_wire.rs').write_text(WIRE)
run(['cargo','test','--offline','--bin','four_dp_poc'],c01)
run(['cargo','run','--offline','--bin','contract_probe'],c01)
run(['cargo','build','--offline','--bins'],c01)
c01mods=patch('C01',c01)
# C03: source metadata migration at the common admission point, not fake screen coverage.
c03=scratch('C03');source=c03/'src/source_contract.rs'
s=source.read_text();start=s.index('pub fn normalize(');s=s[:start]+NORMALIZER;source.write_text(s)
main=c03/'src/main.rs';s=main.read_text();needle='let input:Value=serde_json::from_str(&line).unwrap();'
assert needle in s;s=s.replace(needle,needle+'\n    let input=source_contract::normalize_input(&input);');main.write_text(s)
(c03/'src/bin').mkdir();(c03/'src/bin/contract_probe.rs').write_text(C03PROBE)
run(['cargo','test','--offline','--bin','four_dp_poc'],c03)
run(['cargo','build','--offline','--bins'],c03)
c03mods=patch('C03',c03)
def migrate(v):
    if isinstance(v,list):return [migrate(x) for x in v]
    if not isinstance(v,dict):return v
    x={k:migrate(y) for k,y in v.items()}
    if 'frame_id' in x and 'image_path' in x:
        x['version']={'number':x.pop('source_revision')};x['coordinate']={'unit':'pixels','dimensions':[x.pop('width'),x.pop('height')]};x.pop('coordinate_space')
        x['captured_at']={'value':x.pop('observed_at_ms')*1000,'unit':'microseconds'}
    elif 'id' in x and 'revision' in x and ('path'in x or 'text'in x):x['version']={'number':x.pop('revision')}
    return x
probe_result={"text":"CONTRACT_PROBE_ONLY","answer":"CONTRACT_PROBE_ONLY","targets":[],"adoptions":[],"candidates":[],"cue_relations":[],"coverage":{"complete":True,"unknown":[],"competing":[]},"expressions":[],"reads":[],"records":[],"blocks":[],"conditions":[],"unknowns":[],"relation":"NEW","target_request_id":None,"question_id":None,"need_expansion":False,"need_evidence":False}
checks=[]
for dp,variants in [('01','ABC'),('02','AB'),('04','AB'),('05','ABC')]:
    fixture=json.loads((FIXTURES/f'dp{dp}.json').read_text())[0]
    for variant in variants:
        case={**fixture,'variant':variant};case['dp']=dp
        for change,p in [('C01',c01),('C03',c03)]:
            outgoing=migrate(case) if change=='C03' else case
            prefix=f'{change}-dp{dp}-{variant}';inp=OUT/'probes'/(prefix+'-input.json');expected=OUT/'probes'/(prefix+'-canonical.json');dump(inp,outgoing);dump(expected,case)
            if change=='C03':run([str(p/'target/debug/contract_probe'),str(inp),str(expected)],p)
            proc=subprocess.Popen([str(p/'target/debug/four_dp_poc')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
            proc.stdin.write(json.dumps(outgoing,ensure_ascii=False)+'\n');proc.stdin.flush();events=[];calls=[]
            for line in proc.stdout:
                msg=json.loads(line)
                if msg.get('type')=='model_call':
                    payload=msg.get('payload',msg.get('request',{}).get('context',{}));assert payload.get('instruction')
                    calls.append(msg)
                    r=copy.deepcopy(probe_result)
                    response={'call_id':msg['call_id'],'result':r,'usage':{'prompt_tokens':17,'completion_tokens':9}}
                    if change=='C01':
                        assert msg['wire_version']==2
                        response={'call_id':msg['call_id'],'provider_response':{'json':r,'usage':{'input_tokens':17,'output_tokens':9}}}
                    proc.stdin.write(json.dumps(response)+'\n');proc.stdin.flush()
                else:events.append(msg)
            proc.wait(timeout=10);stderr=proc.stderr.read();assert proc.returncode==0,(prefix,stderr)
            assert not any(e.get('kind')=='EPISODE_ERROR' for e in events),(prefix,events)
            assert any(e.get('kind')=='EPISODE_RESULT' for e in events)
            if change=='C03':
                def keys(v):
                    if isinstance(v,list):return sum((keys(y) for y in v),[])
                    if isinstance(v,dict):return list(v)+sum((keys(y) for y in v.values()),[])
                    return []
                assert not any(k in keys(calls) for k in ['version','coordinate','captured_at'])
            result={'change':change,'dp':dp,'variant':variant,'calls':len(calls),'exit':proc.returncode,'result':'PASS','model':'NO_MODEL; deterministic contract responder only','semantics_claimed':False}
            checks.append(result);dump(OUT/'probes'/(prefix+'-trace.json'),{'check':result,'calls':calls,'events':events,'stderr':stderr})
after={str(p.relative_to(LIVE)):hashlib.sha256(p.read_bytes()).hexdigest() for p in LIVE.rglob('*') if p.is_file() and 'target' not in p.relative_to(LIVE).parts}
assert before==after,'Live baseline modified during scratch exercise; inspect concurrent changes before claiming unchanged.'
summary={'mean_changed_operational_modules':(len(c01mods)+len(c03mods))/2,'changes':[{'id':'C01','operational_modules':c01mods,'count':len(c01mods),'contract':'Provider wire request.context + provider_response.json + renamed usage input/output tokens','baseline_negative_probe':'expected failure; new wire absent','checks':10},{'id':'C03','operational_modules':c03mods,'count':len(c03mods),'contract':'Source version object; frame pixel dimensions and microseconds timestamp envelope normalized at common admission','checks':10}],'by_alternative':[{'dp':dp,'variant':v,'C01':len(c01mods),'C03':len(c03mods),'mean':(len(c01mods)+len(c03mods))/2} for dp,vs in [('01','ABC'),('02','AB'),('04','AB'),('05','ABC')] for v in vs],'checks':checks,'claim_scope':'Actual two Rust compatibility migrations in independent scratch copies; finite PoC scope only. All10 alternatives tie. Coordinates/time applicable to DP04; source revisions exercised for inline DP02 and DP01/05 descriptors. Not full product change population. Test-only bin and harness excluded from module count. No model calls or cost.','live_baseline_unchanged':True,'scratch_root':str(TMP)}
dump(OUT/'summary.json',summary);dump(OUT/'commands.json',LOG)
(OUT/'README.md').write_text('# QA04 actual Rust change exercises\n\nC01 changes one operating Rust module, C03 two. Mean **1.5 modules**, tied across all ten alternatives. Independent scratch copies preserve the live candidate. Actual unified diffs, baseline snapshot, compile/test logs and twenty end-to-end contract traces are retained.\n\nC01 changes the provider request/response and usage envelope. A negative baseline probe fails, the migrated adapter passes JSON/usage/error checks. C03 changes source version metadata and frame coordinate/time envelopes. Shared source adapter plus main input admission restore exact canonical fixture objects; all ten alternatives execute against the migrated contract with a deterministic contract responder. No semantic accuracy or model timing is inferred from these responders. Frame coordinates/time apply to DP04; source revision compatibility applies to all four DPs. Test harness and probe modules are excluded from operational module counts.\n\nThis is a finite PoC compatibility exercise, not the full original product change-task population. No live baseline edits, model calls, or additional spending occur.\n')
print(json.dumps({'summary':str(OUT/'summary.json'),'C01':c01mods,'C03':c03mods,'checks':len(checks),'mean':summary['mean_changed_operational_modules']}))
