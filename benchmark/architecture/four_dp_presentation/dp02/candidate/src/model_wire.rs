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
// Immutable borrowed view: the transport sees the identical JSON values without
// owning copies of source corpus, payload or the wire envelope while awaiting a result.
use std::io::{self,Write};
fn emit<W:Write>(w:&mut W,v:&Value)->io::Result<()>{serde_json::to_writer(&mut *w,v).map_err(io::Error::other)}
fn key<W:Write>(w:&mut W,k:&str)->io::Result<()>{serde_json::to_writer(&mut *w,k).map_err(io::Error::other)?;w.write_all(b":")}
fn borrowed_input<W:Write>(w:&mut W,input:&Value,sources:&Value)->io::Result<()>{
    w.write_all(b"{")?;let mut first=true;for(k,v)in input.as_object().unwrap(){if !first{w.write_all(b",")?}first=false;key(w,k)?;emit(w,if k=="sources"{sources}else{v})?;}w.write_all(b"}")
}
fn borrowed_payload<W:Write>(w:&mut W,p:&Value,trace:&Value,sources:&Value)->io::Result<()>{
    w.write_all(b"{\"_trace\":")?;emit(w,trace)?;for(k,v)in p.as_object().unwrap(){if k=="_trace"{continue}w.write_all(b",")?;key(w,k)?;if k=="input"{borrowed_input(w,v,sources)?}else{emit(w,v)?}}w.write_all(b"}")
}
pub fn write_borrowed_fields<W:Write>(w:&mut W,id:u64,stage:&str,p:&Value,trace:&Value,sources:&Value)->io::Result<()>{
    write!(w,"\"type\":\"model_call\",\"call_id\":{},\"stage\":",id)?;serde_json::to_writer(&mut *w,stage).map_err(io::Error::other)?;w.write_all(b",\"payload\":")?;borrowed_payload(w,p,trace,sources)?;w.write_all(b",\"wire_version\":1")
}
struct Count(usize);impl Write for Count{fn write(&mut self,b:&[u8])->io::Result<usize>{self.0+=b.len();Ok(b.len())}fn flush(&mut self)->io::Result<()>{Ok(())}}
pub fn borrowed_payload_len(p:&Value,trace:&Value,sources:&Value)->usize{let mut count=Count(0);borrowed_payload(&mut count,p,trace,sources).unwrap();count.0}
