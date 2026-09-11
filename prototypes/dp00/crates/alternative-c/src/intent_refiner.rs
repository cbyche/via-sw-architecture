use crate::context_engine::ContextPackage;
use bench_core::{DecisionOwner, ModelPort, ModelProfile, ModelRequest, SemanticResponsibility};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormalizedIntent {
    LocalVolume,
    GeneralFileWork,
    LatestDownloadLookup,
    OrganizeDownloads,
    DiagnoseWifi,
    ContinueTask,
    OpenRightDocument,
    AmbiguousDocument,
    CompoundMediaDownloads,
}

pub fn refine<M: ModelPort>(model: &M, context: &ContextPackage) -> Result<NormalizedIntent, String>
where
    M::Error: std::fmt::Debug,
{
    let response = model
        .generate(ModelRequest {
            decision_owner: DecisionOwner::from("C.IntentRefiner"),
            semantic_responsibilities: vec![
                SemanticResponsibility::IntentInterpretation,
                SemanticResponsibility::ReferentResolution,
            ],
            semantic_input: context.turn.content.clone(),
            expected_output_schema: "normalized-intent.v0".into(),
            model_profile: ModelProfile {
                id: "dp00-base".into(),
                version: "v0".into(),
            },
        })
        .map_err(|error| format!("model error: {error:?}"))?;
    let output = response
        .completed_output()
        .map_err(|status| format!("model generation did not complete: {status:?}"))?;
    if output.contains("COMPOUND_MEDIA_DOWNLOADS") {
        Ok(NormalizedIntent::CompoundMediaDownloads)
    } else if output.contains("AMBIGUOUS_DOCUMENT") {
        Ok(NormalizedIntent::AmbiguousDocument)
    } else if output.contains("OPEN_RIGHT_DOCUMENT") {
        Ok(NormalizedIntent::OpenRightDocument)
    } else if output.contains("CONTINUE_T1") {
        Ok(NormalizedIntent::ContinueTask)
    } else if output.contains("DIAGNOSE_WIFI") {
        Ok(NormalizedIntent::DiagnoseWifi)
    } else if output.contains("LOOKUP_LATEST_DOWNLOAD") {
        Ok(NormalizedIntent::LatestDownloadLookup)
    } else if output.contains("ORGANIZE_DOWNLOADS") {
        Ok(NormalizedIntent::OrganizeDownloads)
    } else if output.contains("GENERAL_FILE_WORK") {
        Ok(NormalizedIntent::GeneralFileWork)
    } else if output.contains("LOCAL_VOLUME") {
        Ok(NormalizedIntent::LocalVolume)
    } else {
        Err(format!("unsupported normalized intent: {output}"))
    }
}
