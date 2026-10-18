use serde_json::{json,Value};
pub fn encode(id:u64,stage:&str,payload:&Value)->Value { json!({"type":"model_call","call_id":id,"stage":stage,"payload":payload,"wire_version":1}) }
pub fn decode(response:&Value)->Result<Value,String> {
    if let Some(error)=response.get("error") {return Err(error.to_string())}
    let mut result=response.get("result").cloned().ok_or_else(||"Missing model result".to_string())?;
    if let Some(usage)=response.get("usage") {
        result["_provider_usage"]=json!({"input_tokens":usage["prompt_tokens"],"output_tokens":usage["completion_tokens"]});
    }
    Ok(result)
}
