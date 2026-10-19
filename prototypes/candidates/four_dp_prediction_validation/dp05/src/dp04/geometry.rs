//! Checks self-consistency against actual observed coordinates, never evaluator geometry.
use serde_json::{json, Value};
/// One public interpretation of relation names for every producer and final writer.
pub const RELATION_CONTRACT: &str = "Relation/event contract (same for all roles): point_inside means a raw pointer_move event's (x,y) point lies INSIDE the TARGET bbox [x1,y1,x2,y2]; it does NOT mean a plotted sample point lies inside a loop. within_region means a raw pointer_path event is a CLOSED polygon (at least4points, final endpoint within2pixels of first) and each selected TARGET bbox CENTER lies inside that raw polygon. For a loop-enclosed set of plotted points use within_region, never point_inside. designated_by_cue is the speech producer's generic designation and follows the cited raw event type: pointer_move checks pointer-in-target-bbox, pointer_path checks target-center-in-closed-polygon. Use only relations allowed by your role's output schema. Read event.type and original frame coordinates; do not infer type from a cue ID's spelling. If a path is open or its designation is ambiguous, request owner evidence or return unresolved; do not use other/relative_position to evade a geometric claim. This contract supplies no target IDs or target geometry.";
pub fn repair_feedback(issues: &[String]) -> Value {
    if issues.is_empty() { return Value::Null; }
    json!({"actual_consistency_errors":issues,"instruction":"Use the shared relation/event contract in your instruction. Correct the relation interpretation and/or extents only from the same raw evidence. A type mismatch is not a hint about which target is correct. Keep uncertainty unresolved if the source does not support a complete binding."})
}

fn frame<'a>(obs: &'a Value, id: &Value) -> Option<&'a Value> {
    obs["revision"]["frames"].as_array()?.iter().find(|f| f["frame_id"] == *id)
}
fn bbox(c: &Value, obs: &Value) -> Option<[f64; 4]> {
    let f = frame(obs, &c["frame_id"])?;
    let a = c["bbox"].as_array()?;
    if a.len() != 4 { return None; }
    let b = [a[0].as_f64()?, a[1].as_f64()?, a[2].as_f64()?, a[3].as_f64()?];
    if b.iter().any(|n| !n.is_finite()) || b[0] < 0. || b[1] < 0. || b[2] <= b[0] || b[3] <= b[1] || b[2] > f["width"].as_f64()? || b[3] > f["height"].as_f64()? { return None; }
    Some(b)
}
pub fn screen_issues(obs: &Value, screen: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(cs) = screen["candidates"].as_array() else { return vec!["CANDIDATES_ARRAY_REQUIRED".into()]; };
    let mut ids = std::collections::HashSet::new();
    for c in cs {
        if c["target_id"].as_str().is_none() || !ids.insert(c["target_id"].to_string()) { errors.push("CANDIDATE_ID_MISSING_OR_DUPLICATE".into()); }
        if bbox(c, obs).is_none() { errors.push(format!("INVALID_FRAME_OR_BBOX:{}", c["target_id"])); }
    }
    errors
}
fn inside(p: [f64; 2], points: &[Value]) -> bool {
    if points.len() < 3 { return false; }
    let mut hit = false;
    for i in 0..points.len() {
        let (a, b) = (&points[i], &points[(i + 1) % points.len()]);
        let (Some(ax),Some(ay),Some(bx),Some(by))=(a[0].as_f64(),a[1].as_f64(),b[0].as_f64(),b[1].as_f64()) else { return false; };
        if (ay > p[1]) != (by > p[1]) && p[0] < (bx-ax)*(p[1]-ay)/(by-ay)+ax { hit = !hit; }
    }
    hit
}
/// Narrow, declared raw-language cardinality check shared by both alternatives.
/// This is not general intent parsing; unsupported/ambiguous wording stays model dependent.
fn explicit_count(text:&str)->Option<usize>{
    let lower=text.to_lowercase();let words=lower.split_whitespace().collect::<Vec<_>>();
    let mut found=Vec::new();
    for unit in ["점","개","항목"] {
        for (word,n) in [("한",1),("두",2),("세",3),("네",4)] {
            if lower.contains(&format!("{} {}",word,unit)) || lower.contains(&format!("{}{}",word,unit)){found.push(n);}
        }
    }
    for pair in words.windows(2){
        if ["point","points","item","items","object","objects","점","개","항목"].contains(&pair[1].trim_matches(|c:char|!c.is_alphanumeric())){
            let n=match pair[0]{"one"=>Some(1),"two"=>Some(2),"three"=>Some(3),"four"=>Some(4),_=>pair[0].parse::<usize>().ok()};
            if let Some(n)=n{found.push(n);}
        }
    }
    for word in &words {
        for unit in ["점","개","항목"] {if let Some(prefix)=word.strip_suffix(unit){if let Ok(n)=prefix.parse::<usize>(){found.push(n);}}}
    }
    found.sort_unstable();found.dedup();if found.len()==1 {Some(found[0])}else{None}
}
pub fn final_issues(obs: &Value, proposal: &Value, screen: Option<&Value>, speech: Option<&Value>) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(adoptions) = proposal["adoptions"].as_array() else { return vec!["ADOPTIONS_ARRAY_REQUIRED".into()]; };
    for e in obs["revision"]["expressions"].as_array().into_iter().flatten() {
        let found: Vec<_> = adoptions.iter().filter(|a| a["expression_id"] == e["expression_id"]).collect();
        if found.len() != 1 { errors.push(format!("EXPRESSION_MISSING_OR_DUPLICATE:{}", e["expression_id"])); continue; }
        let a = found[0];
        if a["evidence"].as_str().is_none_or(|s|s.chars().count()>100) {errors.push("COMPACT_EVIDENCE_REQUIRED".into());}
        if !["ready", "ambiguous", "need_evidence"].contains(&a["status"].as_str().unwrap_or("")) { errors.push("INVALID_STATUS".into()); continue; }
        if a["status"] != "ready" { continue; }
        let Some(targets) = a["targets"].as_array() else { errors.push("TARGETS_ARRAY_REQUIRED".into()); continue; };
        if targets.is_empty() || targets.iter().any(|t| t.as_str().is_none()) { errors.push("EMPTY_OR_INVALID_READY_TARGETS".into()); }
        let unique:std::collections::HashSet<_>=targets.iter().map(|x|x.to_string()).collect();
        if unique.len()!=targets.len(){errors.push("DUPLICATE_READY_TARGETS".into());}
        let semantic = speech.and_then(|s|s["expressions"].as_array()).and_then(|xs|xs.iter().find(|x|x["expression_id"]==e["expression_id"])).unwrap_or(a);
        if semantic.get("count").is_none() || (!semantic["count"].is_null() && semantic["count"].as_u64().is_none()){errors.push("COUNT_INTEGER_OR_EXPLICIT_NULL_REQUIRED".into());}
        if !semantic["cue_ids"].is_array(){errors.push("CUE_IDS_ARRAY_REQUIRED".into());}
        if !["point_inside","designated_by_cue","within_region","explicit_ids","relative_position","correction","retain_prior","other"].contains(&semantic["relation"].as_str().unwrap_or("")){errors.push("UNKNOWN_GROUNDING_RELATION".into());}
        if let Some(n)=e["text"].as_str().and_then(explicit_count){
            if targets.len()!=n || semantic["count"].as_u64()!=Some(n as u64){errors.push(format!("RAW_EXPLICIT_COUNT_CONTRADICTION:{}:{}:{}",e["expression_id"],n,targets.len()));}
        }
        if let Some(n) = semantic["count"].as_u64() {
            if targets.len() != n as usize { errors.push(format!("COUNT_CONTRADICTION:{}:{}:{}",e["expression_id"],n,targets.len())); }
        }
        let extents = screen.and_then(|s|s["candidates"].as_array()).or_else(||a["extents"].as_array());
        for target in targets {
            let matches: Vec<_> = extents.into_iter().flatten().filter(|c|c["target_id"]==*target).collect();
            if matches.len()!=1 { errors.push(format!("EXTENT_MISSING_OR_DUPLICATE:{}",target)); continue; }
            let c=matches[0];
            let Some(b)=bbox(c,obs) else { errors.push(format!("INVALID_FRAME_OR_BBOX:{}",target)); continue; };
            for cue in semantic["cue_ids"].as_array().into_iter().flatten() {
                let event=obs["revision"]["events"].as_array().and_then(|xs|xs.iter().find(|x|x["event_id"]==*cue));
                let Some(event)=event else {errors.push(format!("UNKNOWN_CUE:{}",cue));continue;};
                if event["frame_id"]!=c["frame_id"] {errors.push(format!("CUE_FRAME_CONTRADICTION:{}",target));continue;}
                match semantic["relation"].as_str().unwrap_or("") {
                    "point_inside"|"designated_by_cue" if event["type"]=="pointer_move" => {
                        let x=event["x"].as_f64().unwrap_or(f64::NAN);let y=event["y"].as_f64().unwrap_or(f64::NAN);
                        if !(b[0]<=x && x<=b[2] && b[1]<=y && y<=b[3]) {errors.push(format!("POINT_OUTSIDE_CLAIMED_EXTENT:{}",target));}
                    },
                    "within_region"|"designated_by_cue" if event["type"]=="pointer_path" => {
                        let closed=event["points"].as_array().is_some_and(|ps|ps.len()>=4 && ps.first().zip(ps.last()).is_some_and(|(a,z)|a[0].as_f64().zip(a[1].as_f64()).zip(z[0].as_f64().zip(z[1].as_f64())).is_some_and(|((ax,ay),(zx,zy))|(ax-zx).powi(2)+(ay-zy).powi(2)<=4.)));
                        if !closed {errors.push(format!("UNCLOSED_OR_INVALID_REGION_PATH:{}",cue));}
                        else if !event["points"].as_array().is_some_and(|ps|inside([(b[0]+b[2])/2.,(b[1]+b[3])/2.],ps)) {errors.push(format!("CENTER_OUTSIDE_CLAIMED_REGION:{}",target));}
                    },
                    "point_inside"|"within_region"|"designated_by_cue"=>errors.push(format!("CUE_RELATION_TYPE_CONTRADICTION:{}",cue)),
                    _=>{}
                }
            }
        }
    }
    errors
}
pub fn unresolved(obs: &Value, reason: &[String]) -> Value {
    json!({"adoptions":obs["revision"]["expressions"].as_array().unwrap().iter().map(|e|json!({"expression_id":e["expression_id"],"targets":[],"status":"need_evidence","evidence":"Consistency validation failed after bounded source rejudgment"})).collect::<Vec<_>>(),"contract_failures":reason})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn obs()->Value {json!({"revision":{"frames":[{"frame_id":"f","width":100,"height":100}],"expressions":[{"expression_id":"e"}],"events":[{"event_id":"p","type":"pointer_move","frame_id":"f","x":15,"y":15},{"event_id":"loop","type":"pointer_path","frame_id":"f","points":[[0,0],[50,0],[50,50],[0,50],[0,0]]}]}})}
    fn proposal()->Value {json!({"adoptions":[{"expression_id":"e","status":"ready","evidence":"actual observed geometry","targets":["visible"],"count":1,"relation":"point_inside","cue_ids":["p"],"extents":[{"target_id":"visible","frame_id":"f","bbox":[10,10,20,20]}]}]})}
    #[test] fn accepts_consistent_geometry_without_oracle(){assert!(final_issues(&obs(),&proposal(),None,None).is_empty());}
    #[test] fn rejects_frame_overflow_and_point_contradiction(){let mut p=proposal();p["adoptions"][0]["extents"][0]["bbox"]=json!([50,50,120,120]);assert!(!final_issues(&obs(),&p,None,None).is_empty());p["adoptions"][0]["extents"][0]["bbox"]=json!([50,50,70,70]);assert!(final_issues(&obs(),&p,None,None).iter().any(|s|s.contains("POINT_OUTSIDE")));}
    #[test] fn final_model_count_is_checked_against_speech_not_binder(){let p=proposal();let s=json!({"expressions":[{"expression_id":"e","count":3,"relation":"point_inside","cue_ids":["p"]}]});assert!(final_issues(&obs(),&p,None,Some(&s)).iter().any(|s|s.contains("COUNT_CONTRADICTION")));}
    #[test] fn loop_uses_actual_path(){let mut p=proposal();p["adoptions"][0]["relation"]=json!("within_region");p["adoptions"][0]["cue_ids"]=json!(["loop"]);assert!(final_issues(&obs(),&p,None,None).is_empty());p["adoptions"][0]["extents"][0]["bbox"]=json!([70,70,80,80]);assert!(final_issues(&obs(),&p,None,None).iter().any(|s|s.contains("OUTSIDE_CLAIMED_REGION")));}
    #[test] fn narrow_raw_cardinality_is_not_model_self_report(){assert_eq!(explicit_count("이렇게 둘러싼 세 점"),Some(3));assert_eq!(explicit_count("these three points"),Some(3));assert_eq!(explicit_count("3개"),Some(3));assert_eq!(explicit_count("price 3 dollars"),None);let mut o=obs();o["revision"]["expressions"][0]["text"]=json!("이 세 점");assert!(final_issues(&o,&proposal(),None,None).iter().any(|s|s.contains("RAW_EXPLICIT_COUNT")));}
    #[test] fn malformed_semantic_fields_cannot_skip_geometry(){let mut p=proposal();p["adoptions"][0].as_object_mut().unwrap().remove("count");p["adoptions"][0]["relation"]=json!("invented_relation");p["adoptions"][0]["cue_ids"]=Value::Null;let e=final_issues(&obs(),&p,None,None);assert!(e.iter().any(|s|s=="COUNT_INTEGER_OR_EXPLICIT_NULL_REQUIRED"));assert!(e.iter().any(|s|s=="CUE_IDS_ARRAY_REQUIRED"));assert!(e.iter().any(|s|s=="UNKNOWN_GROUNDING_RELATION"));}
    #[test] fn relation_type_mismatch_and_open_path_are_not_checked_as_success(){let mut p=proposal();p["adoptions"][0]["cue_ids"]=json!(["loop"]);assert!(final_issues(&obs(),&p,None,None).iter().any(|s|s.contains("CUE_RELATION_TYPE_CONTRADICTION")));p["adoptions"][0]["relation"]=json!("within_region");let mut o=obs();o["revision"]["events"][1]["points"]=json!([[0,0],[50,0],[50,50],[10,20]]);assert!(final_issues(&o,&p,None,None).iter().any(|s|s.contains("UNCLOSED_OR_INVALID_REGION_PATH")));}

}
