use bench_core::{DownstreamCapabilityDescriptor, ResourceClass};

#[test]
fn optional_resource_class_is_backward_compatible_and_round_trips() {
    let old = r#"{"executor_id":"ARGO","capability_id":"file.open"}"#;
    let old_descriptor: DownstreamCapabilityDescriptor =
        serde_json::from_str(old).expect("v1 descriptor remains valid");
    assert_eq!(old_descriptor.resource_class, None);

    let descriptor = DownstreamCapabilityDescriptor {
        executor_id: "DocumentSummaryAgent".into(),
        capability_id: "document.summarize".into(),
        resource_class: Some(ResourceClass::IoBound),
    };
    let json = serde_json::to_string(&descriptor).expect("serialize");
    assert!(json.contains(r#""resource_class":"IO_BOUND""#));
    assert_eq!(
        serde_json::from_str::<DownstreamCapabilityDescriptor>(&json).unwrap(),
        descriptor
    );
}

