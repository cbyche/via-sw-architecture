//! Added C05 user-facing query operation over actual displayed adoptions only.
use serde_json::{json,Value};
#[derive(Default)]
pub struct ExplanationStore{records:Vec<Value>}
impl ExplanationStore{
    pub fn delivered(&mut self,adoption_id:&str,adoptions:&[Value],observed:&Value,proposal:&Value){
        for a in adoptions {
            if a["status"]!="ready" || a["targets"].as_array().is_none_or(|x|x.is_empty()){continue;}
            let expression=&a["expression_id"];
            let speech=proposal["speech_state"]["expressions"].as_array().and_then(|xs|xs.iter().find(|x|x["expression_id"]==*expression));
            let cues=speech.map(|s|&s["cue_ids"]).unwrap_or(&a["cue_ids"]);
            let source_extents:Vec<Value>=a["extents"].as_array().into_iter().flatten().filter_map(|extent|{
                let frame=observed["revision"]["frames"].as_array()?.iter().find(|f|f["frame_id"]==extent["frame_id"])?;
                Some(json!({"target_id":extent["target_id"],"source_id":frame["source_id"],"source_revision":frame["source_revision"],"frame_id":frame["frame_id"],"bbox":extent["bbox"],"source_sha256":frame["sha256"]}))
            }).collect();
            let actual_cues:Vec<Value>=observed["revision"]["events"].as_array().into_iter().flatten().filter(|e|cues.as_array().is_some_and(|xs|xs.contains(&e["event_id"]))).cloned().collect();
            self.records.push(json!({"adoption_id":adoption_id,"expression_id":expression,"input_revision":a["revision"],"targets":a["targets"],"source_extents":source_extents,"observed_cues":actual_cues,"candidate_read_set":proposal["candidate_read_set"],"basis":"ACTUALLY_DISPLAYED_ADOPTION"}));
        }
    }
    pub fn query(&self,q:&Value)->Value{
        let matches=self.records.iter().filter(|r|r["adoption_id"]==q["adoption_id"] && r["expression_id"]==q["expression_id"] && r["input_revision"]==q["revision"]).collect::<Vec<_>>();
        if matches.len()==1{json!({"status":"FOUND","record":matches[0]})}else{json!({"status":"NOT_FOUND"})}
    }
}
