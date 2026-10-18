//! A3 strict exceptional single-passage bound, no semantic compression.
use serde_json::{json,Value};
pub const BUDGET:usize=14000;
pub fn bound(view:&mut Vec<Value>,strict:bool)->Vec<Value>{
 let mut events=Vec::new();let budget=if strict{8192}else{BUDGET};
 while view.iter().map(|p|p.to_string().len()).sum::<usize>()>budget && view.len()>1{view.remove(0);events.push(json!({"policy":"oldest_passage","budget_bytes":budget}));}
 if view.len()==1 && view[0].to_string().len()>budget {
  let p=&view[0];let locator=json!({"source_id":p["source_id"],"revision":p["revision"],"start_line":p["start_line"],"end_line":p["end_line"],"unit":p["unit"],"kind":"SOURCE_LOCATOR_REQUIRES_CURRENT_READ"});
  events.push(json!({"policy":if strict{"A3_smaller_budget_original_to_locator"}else{"REQUIRED_hard_cap_original_to_locator"},"before_bytes":p.to_string().len(),"after_bytes":locator.to_string().len(),"budget_bytes":budget}));view[0]=locator;
 }events
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn bounded_normal_identical(){let p=json!({"text":"actual small original"});let mut a=vec![p.clone()];let mut b=a.clone();bound(&mut a,false);bound(&mut b,true);assert_eq!(a,b);}
 #[test]fn locator_has_no_facts(){let mut a=vec![json!({"source_id":"s","revision":2,"start_line":1,"end_line":80,"unit":"one_based_line_inclusive","text":"x".repeat(15000)})];bound(&mut a,true);assert!(a[0].get("text").is_none());assert_eq!(a[0]["revision"],2);assert!(a[0].to_string().len()<BUDGET);}
}
