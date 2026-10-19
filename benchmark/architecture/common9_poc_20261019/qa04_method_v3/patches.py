"""Common requested protocol migrations. Run only against independent frozen-source copies."""
from pathlib import Path
import copy
from extensions import once
from protocol import is_control

MODEL_WIRE_V2=r'''use serde_json::{json,Value};
pub fn encode(id:u64,stage:&str,payload:&Value)->Value{json!({"type":"model_call","call_id":id,"stage":stage,"request":{"context":payload},"wire_version":2})}
pub fn decode(response:&Value)->Result<Value,String>{if let Some(error)=response.get("error"){return Err(error.to_string())}let mut result=response.get("provider_response").and_then(|r|r.get("json")).cloned().ok_or("Model v2 provider_response.json missing")?;if let Some(usage)=response["provider_response"].get("usage"){result["_provider_usage"]=json!({"input_tokens":usage["input_tokens"],"output_tokens":usage["output_tokens"]});}Ok(result)}
#[cfg(test)]mod qa04_c01{use super::*;#[test]fn actual_payload_and_response_preserved(){let input=json!({"role":"VIA","input":{"source":"actual"}});let wire=encode(9,"stage",&input);assert_eq!(wire["request"]["context"],input);let output=json!({"answer":"actual","revision":2,"targets":["source:v1:P1"],"cancelled":true,"execution_id":"exec-actual"});let decoded=decode(&json!({"call_id":9,"provider_response":{"json":output,"usage":{"input_tokens":3,"output_tokens":4}}})).unwrap();assert_eq!(decoded["answer"],output["answer"]);assert_eq!(decoded["targets"],output["targets"]);assert_eq!(decoded["cancelled"],true);assert_eq!(decoded["execution_id"],output["execution_id"]);assert_eq!(decoded["_provider_usage"],json!({"input_tokens":3,"output_tokens":4}));assert!(decode(&json!({"error":{"kind":"CANCELLED","reason":"synthetic-cancel"}})).unwrap_err().contains("synthetic-cancel"));}}
'''
AGENT_WIRE_V2=r'''use serde_json::{json,Value};
pub fn encode_request(value:&Value)->Value{json!({"wire_version":2,"envelope":{"request":value},"protocol":"common-agent-v2"})}
pub fn decode_request(value:&Value)->Result<Value,String>{if value["wire_version"]!=2{return Err("Agent v2 wire required".into())}value.get("envelope").and_then(|x|x.get("request")).cloned().ok_or("Agent v2 request absent".into())}
pub fn encode_receipt(value:&Value)->Value{json!({"wire_version":2,"envelope":{"receipt":value},"protocol":"common-agent-v2"})}
#[cfg(test)]mod qa04_c02{use super::*;#[test]fn actual_request_receipt_identity_preserved(){let x=json!({"task_id":"T","goal_id":"g","revision":2,"accepted":false,"execution_id":"exec-actual","cancellation":{"confirmed":true,"no_duplicate":true},"contract":{"source":"observed"}});assert_eq!(decode_request(&encode_request(&x)).unwrap(),x);assert_eq!(encode_receipt(&x)["envelope"]["receipt"],x);assert!(decode_request(&json!({"wire_version":2,"request":x})).is_err());}}
'''
INPUT_FORMAT_V2=r'''use serde_json::{json,Value};
pub fn normalize(value:Value)->Value{if matches!(value["type"].as_str(),Some("clock_probe"|"observation_deadline"|"cancel_work"|"publication_receipt"|"end_input")){return value}match value{Value::Array(a)=>Value::Array(a.into_iter().map(normalize).collect()),Value::Object(mut o)=>{for v in o.values_mut(){*v=normalize(v.take());}let frame=o.contains_key("frame_id")&&(o.contains_key("image_path")||o.contains_key("source_id"));let source=o.contains_key("id")&&(o.contains_key("text")||o.contains_key("path"))&&!matches!(o.get("kind").and_then(Value::as_str),Some("user"|"agent"));let reference=o.contains_key("source_id")&&(o.contains_key("start_line")||o.contains_key("bbox"));if(frame||source||reference)&&o.get("version").and_then(|v|v.get("number")).is_some(){let rev=o.remove("version").unwrap()["number"].clone();o.insert(if frame||o.contains_key("bbox"){"source_revision"}else{"revision"}.into(),rev);}if frame{if let Some(c)=o.remove("coordinate"){if c["unit"]=="pixels"{o.insert("width".into(),c["dimensions"][0].clone());o.insert("height".into(),c["dimensions"][1].clone());}}if let Some(t)=o.remove("captured_at"){if t["unit"]=="microseconds"{o.insert("observed_at_ms".into(),json!(t["value"].as_u64().unwrap_or(0)/1000));}}}Value::Object(o)},v=>v}}
#[cfg(test)]mod qa04_c03{use super::*;#[test]fn frame_source_revision_not_input_revision(){let x=json!({"revision":2,"frames":[{"frame_id":"f","source_id":"DOC","image_path":"actual.png","version":{"number":1},"coordinate":{"unit":"pixels","dimensions":[2880,1800]},"captured_at":{"unit":"microseconds","value":12000},"native_selection":{"items":[{"source_id":"DOC","version":{"number":1},"bbox":[100,200,300,400]}]}}],"sources":[{"id":"DOC","version":{"number":1},"text":"actual original"}]});let y=normalize(x);assert_eq!(y["revision"],2);assert_eq!(y["frames"][0]["source_revision"],1);assert_eq!(y["frames"][0]["native_selection"]["items"][0]["source_revision"],1);assert_eq!(y["frames"][0]["observed_at_ms"],12);assert_eq!(y["sources"][0]["revision"],1);}}
'''

def apply_c01(crate:Path):
 (crate/'src/model_wire.rs').write_text(MODEL_WIRE_V2)
def apply_c02(crate:Path):
 (crate/'src/agent_wire.rs').write_text(AGENT_WIRE_V2)
def apply_c03(crate:Path):
 (crate/'src/input_format.rs').write_text(INPUT_FORMAT_V2)
 p=crate/'src/main.rs';s=p.read_text();needle='let input:Value=serde_json::from_str(&line).unwrap();'
 s='mod input_format;\n'+s;s=once(s,needle,needle+'let input=input_format::normalize(input);');p.write_text(s)
 p=crate/'src/context.rs';s=p.read_text();needle='if let Ok(value)=serde_json::from_str::<Value>(&line) {'
 s=once(s,needle,needle+'\n                    let value=crate::input_format::normalize(value);');p.write_text(s)

def migrate_c03(value):
 if is_control(value):return copy.deepcopy(value)
 if isinstance(value,list):return[migrate_c03(v)for v in value]
 if not isinstance(value,dict):return value
 x={k:migrate_c03(v)for k,v in value.items()}
 frame='frame_id'in x and ('image_path'in x or'source_id'in x)
 source='id'in x and ('text'in x or'path'in x) and x.get('kind') not in ('user','agent')
 reference='source_id'in x and ('start_line'in x or'bbox'in x)
 if frame and'source_revision'in x:x['version']={'number':x.pop('source_revision')}
 elif source and'revision'in x:x['version']={'number':x.pop('revision')}
 elif reference:
  k='source_revision'if'source_revision'in x else'revision'
  if k in x:x['version']={'number':x.pop(k)}
 if frame:
  if 'width'in x and'height'in x:x['coordinate']={'unit':'pixels','dimensions':[x.pop('width'),x.pop('height')]}
  if 'observed_at_ms'in x:x['captured_at']={'unit':'microseconds','value':x.pop('observed_at_ms')*1000}
 return x
