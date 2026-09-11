use bench_core::ContextEvidence;

#[must_use]
pub fn map(title: &str, product_revision: u64) -> Vec<ContextEvidence> {
    vec![ContextEvidence {
        source: "ACTIVE_WINDOW_TITLE".into(),
        kind: "WINDOW_TITLE".into(),
        value: title.into(),
        observed_at_product_revision: Some(product_revision),
    }]
}
