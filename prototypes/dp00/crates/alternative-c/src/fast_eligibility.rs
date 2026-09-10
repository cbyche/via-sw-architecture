use bench_core::ProductFact;

use crate::intent_refiner::NormalizedIntent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FastCapability {
    Volume,
    DocumentOpen,
}

pub fn evaluate(
    intent: &NormalizedIntent,
    capabilities: &[ProductFact],
    policies: &[ProductFact],
) -> Option<FastCapability> {
    let policy_allows_local = policies
        .iter()
        .any(|fact| fact.key == "local_execution" && fact.value == "allowed");
    if !policy_allows_local {
        return None;
    }

    let candidate = match intent {
        NormalizedIntent::LocalVolume => Some(FastCapability::Volume),
        NormalizedIntent::OpenRightDocument => Some(FastCapability::DocumentOpen),
        NormalizedIntent::GeneralFileWork
        | NormalizedIntent::DiagnoseWifi
        | NormalizedIntent::ContinueTask
        | NormalizedIntent::AmbiguousDocument => None,
    }?;
    let required_capability = match candidate {
        FastCapability::Volume => "local_volume",
        FastCapability::DocumentOpen => "local_document_open",
    };
    capabilities
        .iter()
        .any(|fact| fact.key == "fast_capability" && fact.value == required_capability)
        .then_some(candidate)
}
