//! C06 actual source-region authorization and permitted pixel-view construction.
use serde_json::{json,Value};
use std::process::Command;
#[derive(Default)]pub struct RegionPolicy{epoch:u64,regions:Vec<Value>}
fn overlaps(a:&Value,b:&Value)->bool{
    let get=|v:&Value,i:usize|v[i].as_f64().unwrap_or(f64::NAN);
    get(a,0)<get(b,2) && get(b,0)<get(a,2) && get(a,1)<get(b,3) && get(b,1)<get(a,3)
}
impl RegionPolicy{
    pub fn epoch(&self)->u64{self.epoch}
    pub fn regions(&self)->Value{json!(self.regions)}
    pub fn withdraw(&mut self,event:&Value)->Result<(),String>{
        let rect=event["rect"].as_array().filter(|r|r.len()==4).ok_or("withdrawal rectangle malformed")?;
        if rect.iter().any(|v|v.as_f64().is_none_or(|n|!n.is_finite())) || rect[0].as_f64().unwrap()>=rect[2].as_f64().unwrap() || rect[1].as_f64().unwrap()>=rect[3].as_f64().unwrap(){return Err("withdrawal rectangle invalid".into());}
        let region=json!({"frame_id":event["frame_id"],"rect":event["rect"]});
        if !self.regions.contains(&region){self.regions.push(region);self.epoch+=1;}
        Ok(())
    }
    pub fn allowed(&self,extent:&Value)->bool{!self.regions.iter().any(|r|r["frame_id"]==extent["frame_id"] && overlaps(&r["rect"],&extent["bbox"]))}
    pub fn revoke_prior(&self,prior:&mut Value)->Value{
        let mut denied=Vec::new();let mut retained=Vec::new();
        if let Some(adoptions)=prior["adoptions"].as_array_mut(){for a in adoptions{
            let invalid=a["extents"].as_array().is_some_and(|xs|xs.iter().any(|e|!self.allowed(e)));
            if invalid{denied.push(a["expression_id"].clone());a["targets"]=json!([]);a["extents"]=json!([]);a["status"]=json!("need_evidence");a["evidence"]=json!("Source region withdrawn; new use prohibited");}
            else{retained.extend(a["targets"].as_array().cloned().unwrap_or_default());}
        }}json!({"denied_expression_ids":denied,"retained_targets":retained,"policy_generation":self.epoch})
    }
    pub fn mark_candidate_eligibility(&self,proposal:&mut Value)->Value{
        let mut denied=Vec::new();let mut retained=Vec::new();
        if let Some(cs)=proposal["screen_state"]["candidates"].as_array_mut(){for c in cs{let allowed=self.allowed(c);c["current_use_allowed"]=json!(allowed);if allowed{retained.push(c["target_id"].clone())}else{denied.push(c["target_id"].clone())}}}
        json!({"ineligible_candidate_ids":denied,"eligible_candidate_ids":retained,"eligibility_policy_generation":self.epoch})
    }
}
pub fn region_conflict(revision:&Value,extent:&Value)->bool{revision["region_withdrawals"].as_array().is_some_and(|rs|rs.iter().any(|r|r["frame_id"]==extent["frame_id"] && overlaps(&r["rect"],&extent["bbox"])))}
pub fn redact_revision(revision:&Value)->Result<Value,String>{
    let mut r=revision.clone();let denied=r["region_withdrawals"].as_array().cloned().unwrap_or_default();if denied.is_empty(){return Ok(r);}
    let epoch=r["policy_generation"].as_u64().unwrap_or(0);
    for frame in r["frames"].as_array_mut().ok_or("frames missing")?{
        let masks:Vec<Value>=denied.iter().filter(|d|d["frame_id"]==frame["frame_id"]).map(|d|d["rect"].clone()).collect();if masks.is_empty(){continue;}
        for (field,hash_field) in [("image_path","sha256"),("overlay_image_path","overlay_sha256")]{
            let Some(path)=frame[field].as_str().map(String::from) else{continue;};
            let dir=std::env::temp_dir().join("via-qa04-region-views");std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
            let id=frame["frame_id"].as_str().unwrap_or("frame").chars().filter(|c|c.is_ascii_alphanumeric()).collect::<String>();
            let out=dir.join(format!("{}-{}-{}-{}.png",std::process::id(),epoch,id,field));
            let script=r##"const sharp=require(process.argv[1]),fs=require('fs'),crypto=require('crypto');const masks=JSON.parse(process.argv[4]);let svg='<svg xmlns="http://www.w3.org/2000/svg" width="'+process.argv[5]+'" height="'+process.argv[6]+'">'+masks.map(b=>'<rect x="'+b[0]+'" y="'+b[1]+'" width="'+(b[2]-b[0])+'" height="'+(b[3]-b[1])+'" fill="black"/>').join('')+'</svg>';sharp(process.argv[2]).composite([{input:Buffer.from(svg),left:0,top:0}]).png().toFile(process.argv[3]).then(()=>{const b=fs.readFileSync(process.argv[3]);process.stdout.write(JSON.stringify({sha256:crypto.createHash('sha256').update(b).digest('hex'),bytes:b.length}));}).catch(e=>{process.stderr.write(String(e));process.exit(1)});"##;
            let output=Command::new("/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node").arg("-e").arg(script).arg("/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp").arg(&path).arg(&out).arg(json!(masks).to_string()).arg(frame["width"].to_string()).arg(frame["height"].to_string()).output().map_err(|e|e.to_string())?;
            if !output.status.success(){return Err(format!("source view redaction failed: {}",String::from_utf8_lossy(&output.stderr)));}
            let meta:Value=serde_json::from_slice(&output.stdout).map_err(|e|e.to_string())?;
            frame[field]=json!(out);frame[hash_field]=meta["sha256"].clone();frame["region_redaction_applied"]=json!(true);
        }
    }Ok(r)
}
#[cfg(test)]mod tests{use super::*;#[test]fn withdrawal_is_idempotent_and_preserves_disjoint(){let mut p=RegionPolicy::default();let e=json!({"frame_id":"f","rect":[0,0,30,30]});p.withdraw(&e).unwrap();p.withdraw(&e).unwrap();assert_eq!(p.epoch(),1);assert!(!p.allowed(&json!({"frame_id":"f","bbox":[10,10,20,20]})));assert!(p.allowed(&json!({"frame_id":"f","bbox":[40,40,60,60]})));}}
