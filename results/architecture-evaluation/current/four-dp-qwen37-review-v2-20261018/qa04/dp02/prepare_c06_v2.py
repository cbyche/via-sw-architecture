from pathlib import Path
import shutil,json,hashlib,difflib
ROOT=Path(__file__).resolve().parents[2];BASE=ROOT/'calibration-v2/frozen/source/prototypes/candidates/four_dp_poc_v2';TASK=Path(__file__).resolve().parent/'C06-v2';CRATE=TASK/'candidate'
if CRATE.exists():raise SystemExit('Preserve existing independent scratch')
shutil.copytree(BASE,CRATE)
p=CRATE/'src/dp02/state.rs';s=p.read_text().replace('pub last_relation_node:Option<String>','pub range_revocations:Vec<Value>,pub last_relation_node:Option<String>')
s+='''
/// Future C06 range policy: versioned byte extents on actual supplied sources.
pub fn revoke_ranges(state:&Shared,input:&Value,ctx:&Context){
 let mut s=state.0.lock().unwrap();if !s.range_revocations.is_empty(){return}
 for event in input["source_range_revocations"].as_array().into_iter().flatten(){
  let Some(src)=input["sources"].as_array().and_then(|a|a.iter().find(|x|x["id"]==event["source_id"]&&x["revision"]==event["revision"]))else{continue};
  let text=src["text"].as_str().unwrap_or("");let a=event["start_byte"].as_u64().unwrap_or(u64::MAX)as usize;let b=event["end_byte"].as_u64().unwrap_or(0)as usize;
  if a<b&&b<=text.len()&&text.is_char_boundary(a)&&text.is_char_boundary(b){s.range_revocations.push(event.clone());ctx.event("SOURCE_RANGE_REVOKED",event);}
 }
}
pub fn allowed_sources(state:&Shared,input:&Value)->Value{
 let revoked=state.0.lock().unwrap().range_revocations.clone();let mut sources=input["sources"].clone();
 for src in sources.as_array_mut().into_iter().flatten(){let text=src["text"].as_str().unwrap_or("").to_string();let mut ranges=revoked.iter().filter(|r|r["source_id"]==src["id"]&&r["revision"]==src["revision"]).map(|r|(r["start_byte"].as_u64().unwrap()as usize,r["end_byte"].as_u64().unwrap()as usize)).collect::<Vec<_>>();ranges.sort();let mut cursor=0;let mut filtered=String::new();for(a,b)in ranges{if a>=cursor{filtered.push_str(&text[cursor..a]);filtered.push_str("[WITHDRAWN RANGE]");cursor=b;}else{cursor=cursor.max(b);}}filtered.push_str(&text[cursor..]);src["text"]=json!(filtered);}
 sources
}
pub fn draft_allowed(state:&Shared,input:&Value,draft:&Value)->bool{
 let allowed=allowed_sources(state,input);
 draft["source_claims"].as_array().into_iter().flatten().all(|c|allowed.as_array().is_some_and(|a|a.iter().any(|s|s["id"]==c["source_id"]&&s["revision"]==c["revision"]&&c["quote"].as_str().is_some_and(|q|s["text"].as_str().unwrap_or("").contains(q)))))
}
''';p.write_text(s)
p=CRATE/'src/dp02/response.rs';s=p.read_text().replace('    let mut reason=Value::Null;','    let mut reason=Value::Null;let allowed=state::allowed_sources(state,input);').replace('"sources":input["sources"]','"sources":allowed').replace('validate(&v,request,&input["sources"])','validate(&v,request,&allowed)');p.write_text(s)
p=CRATE/'src/dp02/execution.rs';s=p.read_text().replace('        state::wait_relations(state,ctx,goal_id);','''        ctx.event("RESPONSE_DRAFT_READY",&json!({"goal_id":goal_id,"draft":value}));
        state::revoke_ranges(state,input,ctx);
        if !state::draft_allowed(state,input,&value){ctx.event("REVOKED_DRAFT_REJECTED",&json!({"goal_id":goal_id,"draft":value,"action":"bounded source-policy rejudgment"}));if revision_attempt==1{return Err("revoked draft after bounded rejudgment".into())}continue;}
        state::wait_relations(state,ctx,goal_id);''');p.write_text(s)
changed=[];diff=[]
for source in sorted(BASE.rglob('*.rs')):
 other=CRATE/source.relative_to(BASE)
 if source.read_bytes()!=other.read_bytes():changed.append(str(source.relative_to(BASE)));diff.extend(difflib.unified_diff(source.read_text().splitlines(True),other.read_text().splitlines(True),fromfile='a/'+str(source.relative_to(BASE)),tofile='b/'+str(source.relative_to(BASE))))
TASK.mkdir(parents=True,exist_ok=True);(TASK/'candidate.patch').write_text(''.join(diff));(TASK/'manifest.json').write_text(json.dumps({'task':'C06 source range revocation during pending draft','baseline_freeze_sha256':hashlib.sha256((ROOT/'calibration-v2/freeze.json').read_bytes()).hexdigest(),'changed_operational_modules':changed,'module_count':len(changed),'alternatives':['A','B'],'scope':'Actual versioned byte-range masking before rejudgment; actual draft quote invalidation before publication; deterministic event at FIRST_DRAFT_READY, not model latency.'},indent=2));print(json.dumps({'crate':str(CRATE),'changed':changed}))
