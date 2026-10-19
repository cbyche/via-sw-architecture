//! Actual common in-process adapter boundary for downstream requests and receipts.
use serde_json::{json, Value};
pub fn encode_request(value: &Value) -> Value {
    json!({"wire_version":1,"request":value})
}
pub fn decode_request(value: &Value) -> Result<Value, String> {
    value
        .get("request")
        .cloned()
        .ok_or("Agent request envelope missing".into())
}
pub fn encode_receipt(value: &Value) -> Value {
    json!({"wire_version":1,"receipt":value})
}
