from pathlib import Path
import shutil,json,hashlib,difflib
ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/'calibration-v1/frozen/source/prototypes/candidates/four_dp_poc_v2'
TASK=Path(__file__).resolve().parent/'C04';CRATE=TASK/'candidate'
if CRATE.exists():raise SystemExit('Preserve existing patch; choose a new task generation')
shutil.copytree(BASE,CRATE)
p=CRATE/'src/dp02/state.rs';s=p.read_text().replace('pub last_relation_node:Option<String>','pub pending_voice:Vec<Value>,pub section_history:Vec<Value>,pub last_relation_node:Option<String>')
s+='''
/// Future C04: revise a delivered section while preserving goal and unrelated section state.
pub fn queue_sections(state:&Shared,goal:&str,value:&Value){let mut s=state.0.lock().unwrap();for section in value["sections"].as_array().into_iter().flatten(){s.pending_voice.push(json!({"goal_id":goal,"section_id":section["id"],"section_revision":1,"voice_prepared":false}));}}
pub fn patch_section(state:&Shared,patch:&Value)->Result<Value,String>{
 let mut s=state.0.lock().unwrap();let id=patch["goal_id"].as_str().ok_or("patch goal missing")?;let section_id=patch["section_id"].as_str().ok_or("patch section missing")?;let new_revision=patch["section_revision"].as_u64().ok_or("patch revision missing")?;let replacement=patch["text"].as_str().filter(|s|!s.is_empty()).ok_or("patch text missing")?;
 let r=s.requests.get_mut(id).ok_or("patch goal unavailable")?;let delivered=r.delivered.as_mut().ok_or("no actual delivered answer to patch")?;let sections=delivered["result"]["sections"].as_array_mut().ok_or("delivered sections missing")?;let target=sections.iter_mut().find(|x|x["id"]==section_id).ok_or("section not delivered")?;let old=target["section_revision"].as_u64().unwrap_or(1);if new_revision<=old{return Err("stale section patch".into())}let before=target.clone();target["text"]=json!(replacement);target["section_revision"]=json!(new_revision);let rendered=sections.iter().filter_map(|s|s["text"].as_str()).collect::<Vec<_>>().join("\\n");delivered["text"]=json!(rendered);delivered["result"]["answer"]=delivered["text"].clone();let updated=delivered.clone();let request_revision=r.revision;
 s.pending_voice.retain(|v|!(v["goal_id"]==id&&v["section_id"]==section_id));s.pending_voice.push(json!({"goal_id":id,"section_id":section_id,"section_revision":new_revision,"voice_prepared":false}));let history=json!({"goal_id":id,"goal_revision":request_revision,"section_id":section_id,"before":before,"after_revision":new_revision,"text":replacement,"full_current_delivery":updated,"pending_voice":s.pending_voice});s.section_history.push(history.clone());for output in &mut s.outputs{if output["goal_id"]==id{*output=updated.clone();}}Ok(history)
}
''';p.write_text(s)
p=CRATE/'src/dp02/execution.rs';s=p.read_text().replace('ctx.complete(goal_id,&terminal)?;', 'ctx.complete(goal_id,&terminal)?;state::queue_sections(state,goal_id,&value);')
s=s.replace('    let s=shared.0.lock().unwrap();Ok(json!(','''    for patch in input["incremental_updates"].as_array().into_iter().flatten(){match state::patch_section(&shared,patch){Ok(value)=>ctx.event("CONTENT_SECTION_PATCH",&value),Err(reason)=>ctx.event("SECTION_PATCH_REJECTED",&json!({"patch":patch,"reason":reason}))}}
    let s=shared.0.lock().unwrap();Ok(json!(''').replace('"variant":input["variant"]}))','"section_history":s.section_history,"pending_voice":s.pending_voice,"variant":input["variant"]}))');p.write_text(s)
changed=[];diff=[]
for source in sorted(BASE.rglob('*.rs')):
 other=CRATE/source.relative_to(BASE)
 if source.read_bytes()!=other.read_bytes():changed.append(str(source.relative_to(BASE)));diff.extend(difflib.unified_diff(source.read_text().splitlines(True),other.read_text().splitlines(True),fromfile='a/'+str(source.relative_to(BASE)),tofile='b/'+str(source.relative_to(BASE))))
TASK.mkdir(parents=True,exist_ok=True);(TASK/'candidate.patch').write_text(''.join(diff));(TASK/'manifest.json').write_text(json.dumps({'task':'C04 delivered section incremental update','baseline_freeze_sha256':hashlib.sha256((ROOT/'calibration-v1/freeze.json').read_bytes()).hexdigest(),'changed_operational_modules':changed,'module_count':len(changed),'alternatives':['A','B'],'scope':'No voice synthesis; actual structured pending section refs and callback. Independently patched frozen baseline.'},indent=2));print(json.dumps({'crate':str(CRATE),'changed':changed}))
