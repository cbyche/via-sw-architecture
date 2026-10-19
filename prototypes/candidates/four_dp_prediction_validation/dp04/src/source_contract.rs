use serde_json::{json,Value};
pub fn read(source:&Value)->Result<Value,String> {
    if source["revoked"].as_bool()==Some(true) {return Err("SOURCE_ACCESS_REVOKED".into())}
    let mut result=source.clone();
    if let Some(path)=source["path"].as_str() {result["text"]=json!(std::fs::read_to_string(path).map_err(|e|e.to_string())?);}
    Ok(result)
}
/// Frame-source contract normalization, shared by joint and producer-owned grounding.
/// Version 1 already carries canonical pixel space, milliseconds, and source_revision.
pub fn normalize(value:&Value)->Value { value.clone() }
