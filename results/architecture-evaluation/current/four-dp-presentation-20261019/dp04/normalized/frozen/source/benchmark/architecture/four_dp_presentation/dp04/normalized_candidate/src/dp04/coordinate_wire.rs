//! Common model-output wire; source geometry/oracle never supplies an answer.
use serde_json::{json,Value};
pub const CONTRACT:&str="Coordinate wire shared by joint and screen roles: emit every target extent as coordinate_space:'qwen_normalized_0_1000', bbox_2d:[nx1,ny1,nx2,ny2], with INTEGER coordinates from 0 through 1000 on ORIGINAL FRAME axes. Do not emit bbox or pixel extents. Raw pointer/path coordinates, frame width/height and event times stay in ORIGINAL PIXELS. The host converts each normalized axis using round-to-nearest/ties-even(n/1000*original frame dimension), then applies unchanged pixel bounds/count/type/cue geometry checks. Do not compare raw pixel numbers directly to normalized bbox values. State the actual frame_id; do not normalize relative to a crop, window or another frame. Unknown, reversed, nonfinite/out-of-range units fail; the host never clamps, guesses units or fills target identities. No expected bbox or target is provided.";
fn convert(obs:&Value,c:&mut Value)->Result<Value,String>{
    if c["coordinate_space"]!="qwen_normalized_0_1000"{return Err("EXPLICIT_NORMALIZED_COORDINATE_UNIT_REQUIRED".into());}
    if c.get("bbox").is_some(){return Err("AMBIGUOUS_DUPLICATE_PIXEL_BBOX_ON_NORMALIZED_WIRE".into());}
    let frames=obs["revision"]["frames"].as_array().ok_or("SOURCE_FRAMES_MISSING")?;
    let fs=frames.iter().filter(|f|f["frame_id"]==c["frame_id"]).collect::<Vec<_>>();
    if fs.len()!=1{return Err("UNKNOWN_OR_DUPLICATE_SOURCE_FRAME".into());}
    let f=fs[0];
    for key in ["source_id","source_revision"]{if let Some(v)=c.get(key){if *v!=f[key]{return Err(format!("SOURCE_METADATA_MISMATCH:{}",key));}}}
    let w=f["width"].as_f64().ok_or("FRAME_WIDTH_REQUIRED")?;let h=f["height"].as_f64().ok_or("FRAME_HEIGHT_REQUIRED")?;
    if !w.is_finite() || !h.is_finite() || w<=0. || h<=0.{return Err("INVALID_FRAME_DIMENSIONS".into());}
    let a=c["bbox_2d"].as_array().ok_or("BBOX_2D_ARRAY_REQUIRED")?;
    if a.len()!=4{return Err("BBOX_2D_LENGTH".into());}
    let mut b=[0.;4];
    for(i,v)in a.iter().enumerate(){let n=v.as_f64().ok_or("NUMERIC_NORMALIZED_COORDINATE_REQUIRED")?;if !n.is_finite() || n.fract()!=0. || n<0. || n>1000.{return Err("NORMALIZED_INTEGER_OUT_OF_RANGE".into());}b[i]=n;}
    if b[0]>=b[2] || b[1]>=b[3]{return Err("REVERSED_OR_EMPTY_NORMALIZED_BBOX".into());}
    let pixel=[(b[0]/1000.*w).round_ties_even(),(b[1]/1000.*h).round_ties_even(),(b[2]/1000.*w).round_ties_even(),(b[3]/1000.*h).round_ties_even()];
    c["bbox"]=json!(pixel);c["wire_coordinate_space"]=json!("qwen_normalized_0_1000");c["coordinate_space"]=json!("frame_pixels");
    Ok(json!({"frame_id":c["frame_id"],"target_id":c["target_id"],"source_id":f["source_id"],"source_revision":f["source_revision"],"normalized_bbox":b,"canonical_pixel_bbox":pixel,"width":w,"height":h,"rounding":"nearest_ties_even","inference_or_clamping":false}))
}
pub fn normalize(obs:&Value,result:&mut Value,screen:bool)->Result<Vec<Value>,String>{
    let mut next=result.clone();let mut mappings=Vec::new();
    if screen{
        for c in next["candidates"].as_array_mut().ok_or("CANDIDATES_ARRAY_REQUIRED")?{mappings.push(convert(obs,c)?);}
    }else{
        for a in next["adoptions"].as_array_mut().ok_or("ADOPTIONS_ARRAY_REQUIRED")?{
            if a["status"]=="ready"{for c in a["extents"].as_array_mut().ok_or("READY_EXTENTS_ARRAY_REQUIRED")?{mappings.push(convert(obs,c)?);}}
        }
    }
    *result=next;Ok(mappings)
}
#[cfg(test)]mod tests{
    use super::*;
    fn obs()->Value{json!({"revision":{"frames":[{"frame_id":"f","source_id":"s","source_revision":3,"width":1000,"height":650}]}})}
    fn c()->Value{json!({"target_id":"visible","frame_id":"f","coordinate_space":"qwen_normalized_0_1000","bbox_2d":[0,500,1000,1000]})}
    #[test]fn axes_and_corners(){let mut x=c();convert(&obs(),&mut x).unwrap();assert_eq!(x["bbox"],json!([0.,325.,1000.,650.]));}
    #[test]fn non_square_and_ties_even(){let mut o=obs();o["revision"]["frames"][0]["width"]=json!(200);o["revision"]["frames"][0]["height"]=json!(500);let mut x=c();x["bbox_2d"]=json!([0,1,1000,3]);convert(&o,&mut x).unwrap();assert_eq!(x["bbox"],json!([0.,0.,200.,2.]));}
    #[test]fn rejects_unknown_units_and_frame(){for (key,v) in [("coordinate_space",json!("pixels")),("coordinate_space",json!("normalized_0_999")),("frame_id",json!("other")),("source_revision",json!(2)),("source_id",json!("other"))]{let mut x=c();x[key]=v;assert!(convert(&obs(),&mut x).is_err());}}
    #[test]fn rejects_malformed_and_overflow_without_clamp(){for bbox in [json!([-1,0,1000,1000]),json!([0,0,1001,1000]),json!([0,0,1.5,1000]),json!([2,0,1,1000]),json!([0,0,0,1000]),json!([0,0,"1000",1000]),Value::Null]{let mut x=c();x["bbox_2d"]=bbox;assert!(convert(&obs(),&mut x).is_err());}}
    #[test]fn rejects_ambiguous_bbox_and_invalid_dimensions(){let mut x=c();x["bbox"]=json!([0,0,1000,650]);assert!(convert(&obs(),&mut x).is_err());let mut o=obs();o["revision"]["frames"][0]["width"]=json!(0);assert!(convert(&o,&mut c()).is_err());}
    #[test]fn failed_conversion_is_transactional(){let mut a=c();a["frame_id"]=json!("bad");let mut r=json!({"candidates":[c(),a]});let old=r.clone();assert!(normalize(&obs(),&mut r,true).is_err());assert_eq!(r,old);}
    #[test]fn same_conversion_for_both_owners(){let mut s=json!({"candidates":[c()]});let mut a=json!({"adoptions":[{"status":"ready","extents":[c()]}]});normalize(&obs(),&mut s,true).unwrap();normalize(&obs(),&mut a,false).unwrap();assert_eq!(s["candidates"][0]["bbox"],a["adoptions"][0]["extents"][0]["bbox"]);}
    #[test]fn screen_wire_rejection_cannot_be_overridden_by_binder_pixel_bbox(){
        let mut o=obs();o["revision"]["expressions"]=json!([{"expression_id":"e"}]);o["revision"]["events"]=json!([{"event_id":"p","type":"pointer_move","frame_id":"f","x":15,"y":15}]);
        let proposal=json!({"adoptions":[{"expression_id":"e","status":"ready","targets":["visible"],"count":1,"cue_ids":["p"],"relation":"point_inside","evidence":"actual raw point"}]});
        let screen=json!({"coordinate_wire_valid":false,"candidates":[{"target_id":"visible","frame_id":"f","bbox":[10,10,20,20]}]});
        let issues=super::super::geometry::final_issues(&o,&proposal,Some(&screen),None);
        assert!(issues.iter().any(|s|s=="NORMALIZED_SCREEN_WIRE_NOT_ADMISSIBLE"));
        assert!(!issues.iter().any(|s|s.starts_with("POINT_OUTSIDE")||s.starts_with("INVALID_FRAME_OR_BBOX")));
    }
}
