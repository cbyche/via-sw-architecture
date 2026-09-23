//! Strict adapters from versioned Pilot JSON assets to runtime-owned values.

use std::collections::HashMap;

use bench_core::{
    ContextEvidence, ExecutionId, ExecutionRoute, InitialProductState, InitialTaskState,
    InputModality, ModelStatus, ProductFact, SemanticResponsibility, TaskId, TurnId, UserTurn,
};
use bench_replay::{ReplayAttempt, ReplayOperation};
use serde::Deserialize;
use serde_json::Value;

use crate::pilot_assets::{
    AssetError, BehaviorClass, FixtureAsset, FixtureKind, Modality, PilotCorpus, RuntimeScenario,
    SemanticResponsibility as AssetResponsibility,
};

#[derive(Clone, Debug)]
pub struct MaterializedScenario {
    pub initial_state: InitialProductState,
    pub turns: Vec<UserTurn>,
    pub acoustic_eos_offsets_micros: Vec<Option<u64>>,
    pub clarification_triggered_turns: Vec<bool>,
    pub replay_operations: Vec<ReplayOperation>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AudioData {
    audio_fixture_identity: String,
    transcript: String,
    sample_rate_hz: u32,
    ground_truth_acoustic_eos_offset_micros: u64,
    eos_annotation_authority: String,
    audio_content_generation: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextData {
    surface_snapshot_generation: u64,
    objects: Vec<SurfaceObject>,
    pointer_events: Vec<PointerEvent>,
    focus_object_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SurfaceObject {
    object_id: String,
    role: String,
    label: String,
    bounds: Bounds,
    stable_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bounds {
    x: i64,
    y: i64,
    width: u64,
    height: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PointerEvent {
    x: i64,
    y: i64,
    event: String,
    snapshot_generation: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Conversation {
    conversation_id: String,
    facts: Vec<Fact>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fact {
    key: String,
    value: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActiveTask {
    task_id: String,
    goal: String,
    executor_id: String,
    status: String,
    execution_id: String,
    continuation_ref: String,
    prior_route: ExecutionRoute,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyInitialState {
    conversation: Conversation,
    active_tasks: Vec<ActiveTask>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskInitialState {
    conversation: Conversation,
    active_tasks: Vec<ActiveTask>,
    current_turn_selected_task_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum InitialStateData {
    WithTasks(TaskInitialState),
    Empty(EmptyInitialState),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapabilityData {
    executors: Vec<CapabilityExecutor>,
    bounded_local_capabilities: Vec<String>,
    architecture_specific_route_hint: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapabilityExecutor {
    executor_id: String,
    registered: bool,
    capability_ids: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HealthData {
    statuses: Vec<HealthStatus>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HealthStatus {
    executor_id: String,
    healthy: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyData {
    consent_state: String,
    network_allowed: bool,
    local_volume_change_allowed: bool,
}

pub fn materialize_scenario(
    corpus: &PilotCorpus,
    scenario: &RuntimeScenario,
) -> Result<MaterializedScenario, AssetError> {
    let fixtures: HashMap<_, _> = corpus
        .fixtures
        .fixtures
        .iter()
        .map(|fixture| (fixture.asset_id.as_str(), fixture))
        .collect();
    let context: ContextData = fixture_data(
        &fixtures,
        &scenario.stimulus.context_evidence_ref.asset_id,
        FixtureKind::ContextEvidence,
    )?;
    let initial = fixture(
        &fixtures,
        &scenario.stimulus.initial_state_ref.asset_id,
        FixtureKind::InitialState,
    )?;
    let capabilities: CapabilityData = fixture_data(
        &fixtures,
        &scenario.stimulus.capability_profile_ref.asset_id,
        FixtureKind::CapabilityProfile,
    )?;
    let health: HealthData = fixture_data(
        &fixtures,
        &scenario.stimulus.health_profile_ref.asset_id,
        FixtureKind::HealthProfile,
    )?;
    let policy: PolicyData = fixture_data(
        &fixtures,
        &scenario.stimulus.policy_profile_ref.asset_id,
        FixtureKind::PolicyProfile,
    )?;

    let context_json = serde_json::to_string(&context_to_value(&context))
        .map_err(|error| AssetError::Invalid(error.to_string()))?;
    let make_turn = |turn_id: &str, modality: Modality, content: &str| UserTurn {
        turn_id: TurnId(turn_id.into()),
        modality: match modality {
            Modality::Voice => InputModality::Voice,
            Modality::Text => InputModality::Text,
        },
        content: content.into(),
        context_evidence: vec![ContextEvidence {
            source: "controlled-interaction-fixture".into(),
            kind: "raw-context-evidence".into(),
            value: context_json.clone(),
            observed_at_product_revision: Some(context.surface_snapshot_generation),
        }],
    };
    let mut turns = Vec::new();
    let mut acoustic_eos_offsets_micros = Vec::new();
    let mut clarification_triggered_turns = Vec::new();
    for turn in &scenario.stimulus.interaction {
        turns.push(make_turn(&turn.turn_id, turn.modality, &turn.text_fixture));
        clarification_triggered_turns.push(false);
        let acoustic_offset = (|| {
            if turn.acoustic_eos_fixture_ref == "NONE" {
                return Ok(None);
            }
            let audio: AudioData =
                fixture_data(&fixtures, &turn.audio_fixture_ref, FixtureKind::Audio)?;
            if audio.eos_annotation_authority != "BENCHMARK_FIXTURE" {
                return Err(AssetError::Invalid(format!(
                    "{} acoustic EOS authority is not benchmark-owned",
                    turn.audio_fixture_ref
                )));
            }
            let _identity_fields = (
                audio.audio_fixture_identity,
                audio.transcript,
                audio.sample_rate_hz,
                audio.audio_content_generation,
            );
            Ok(Some(audio.ground_truth_acoustic_eos_offset_micros))
        })()?;
        acoustic_eos_offsets_micros.push(acoustic_offset);
        for reply in &turn.deterministic_user_replies {
            if reply.trigger != "clarification.requested" {
                return Err(AssetError::Invalid(format!(
                    "unsupported scripted reply trigger {}",
                    reply.trigger
                )));
            }
            turns.push(make_turn(
                &reply.turn_id,
                reply.modality,
                &reply.text_fixture,
            ));
            acoustic_eos_offsets_micros.push(None);
            clarification_triggered_turns.push(true);
        }
    }

    let mut initial_state = materialize_initial_state(initial)?;
    initial_state.capability_facts = capability_facts(capabilities, health);
    initial_state.policy_facts = vec![
        ProductFact {
            key: "consent_state".into(),
            value: policy.consent_state,
        },
        ProductFact {
            key: "network_allowed".into(),
            value: policy.network_allowed.to_string(),
        },
        ProductFact {
            key: "local_volume_change_allowed".into(),
            value: policy.local_volume_change_allowed.to_string(),
        },
    ];

    let plan = corpus
        .behavior_plans
        .plans
        .iter()
        .find(|plan| plan.scenario_id == scenario.scenario_id)
        .ok_or_else(|| AssetError::Invalid("scenario behavior plan missing".into()))?;
    let replay_operations = plan
        .operations
        .iter()
        .map(|operation| {
            let responsibility = map_responsibility(operation.responsibility)?;
            let attempts = operation
                .attempts
                .iter()
                .map(|attempt| {
                    let output = if attempt.payload_ref == "NONE" {
                        if behavior_status(attempt.behavior_class) == ModelStatus::Completed {
                            return Err(AssetError::Invalid(format!(
                                "{} completed attempt requires a replay payload",
                                operation.operation_key
                            )));
                        }
                        String::new()
                    } else {
                        let payload =
                            fixture(&fixtures, &attempt.payload_ref, FixtureKind::ReplayPayload)?;
                        replay_output(&payload.data)
                    };
                    Ok(ReplayAttempt {
                        output,
                        status: behavior_status(attempt.behavior_class),
                    })
                })
                .collect::<Result<Vec<_>, AssetError>>()?;
            Ok(ReplayOperation::new(
                operation.operation_key.clone(),
                responsibility,
                attempts,
            ))
        })
        .collect::<Result<Vec<_>, AssetError>>()?;

    Ok(MaterializedScenario {
        initial_state,
        turns,
        acoustic_eos_offsets_micros,
        clarification_triggered_turns,
        replay_operations,
    })
}

fn fixture<'a>(
    fixtures: &HashMap<&str, &'a FixtureAsset>,
    id: &str,
    kind: FixtureKind,
) -> Result<&'a FixtureAsset, AssetError> {
    let value = fixtures
        .get(id)
        .copied()
        .ok_or_else(|| AssetError::Invalid(format!("fixture {id} is unresolved")))?;
    if value.fixture_kind != kind {
        return Err(AssetError::Invalid(format!(
            "fixture {id} has the wrong runtime kind"
        )));
    }
    Ok(value)
}

fn fixture_data<T: for<'de> Deserialize<'de>>(
    fixtures: &HashMap<&str, &FixtureAsset>,
    id: &str,
    kind: FixtureKind,
) -> Result<T, AssetError> {
    serde_json::from_value(fixture(fixtures, id, kind)?.data.clone())
        .map_err(|error| AssetError::Invalid(format!("fixture {id}: {error}")))
}

fn materialize_initial_state(asset: &FixtureAsset) -> Result<InitialProductState, AssetError> {
    let data: InitialStateData = serde_json::from_value(asset.data.clone())
        .map_err(|error| AssetError::Invalid(format!("fixture {}: {error}", asset.asset_id)))?;
    let (conversation, active_tasks) = match data {
        InitialStateData::WithTasks(data) => {
            if data.current_turn_selected_task_id.is_some() {
                return Err(AssetError::Invalid(
                    "initial state must not preselect the current task".into(),
                ));
            }
            (data.conversation, data.active_tasks)
        }
        InitialStateData::Empty(data) => (data.conversation, data.active_tasks),
    };
    let mut conversation_facts = conversation
        .facts
        .into_iter()
        .map(|fact| ProductFact {
            key: fact.key,
            value: fact.value,
        })
        .collect::<Vec<_>>();
    conversation_facts.push(ProductFact {
        key: "conversation_id".into(),
        value: conversation.conversation_id,
    });
    let tasks = active_tasks
        .into_iter()
        .map(|task| {
            let _fixture_only = (task.goal, task.executor_id, task.continuation_ref);
            InitialTaskState {
                task_id: TaskId(task.task_id),
                execution_id: Some(ExecutionId(task.execution_id)),
                prior_route: Some(task.prior_route),
                status: task.status,
            }
        })
        .collect();
    Ok(InitialProductState {
        conversation_facts,
        tasks,
        capability_facts: Vec::new(),
        policy_facts: Vec::new(),
    })
}

fn capability_facts(capabilities: CapabilityData, health: HealthData) -> Vec<ProductFact> {
    let health: HashMap<_, _> = health
        .statuses
        .into_iter()
        .map(|status| (status.executor_id, status.healthy))
        .collect();
    let mut facts = Vec::new();
    for executor in capabilities.executors {
        facts.push(ProductFact {
            key: "executor".into(),
            value: executor.executor_id.clone(),
        });
        facts.push(ProductFact {
            key: format!("executor_registered:{}", executor.executor_id),
            value: executor.registered.to_string(),
        });
        facts.push(ProductFact {
            key: format!("executor_healthy:{}", executor.executor_id),
            value: health
                .get(&executor.executor_id)
                .copied()
                .unwrap_or(false)
                .to_string(),
        });
        facts.extend(
            executor
                .capability_ids
                .into_iter()
                .map(|capability| ProductFact {
                    key: format!("executor_capability:{}", executor.executor_id),
                    value: capability,
                }),
        );
    }
    facts.extend(
        capabilities
            .bounded_local_capabilities
            .into_iter()
            .map(|value| ProductFact {
                key: "fast_capability".into(),
                value,
            }),
    );
    facts.push(ProductFact {
        key: "architecture_specific_route_hint".into(),
        value: capabilities.architecture_specific_route_hint.to_string(),
    });
    facts
}

fn context_to_value(context: &ContextData) -> Value {
    let objects = context
        .objects
        .iter()
        .map(|object| {
            serde_json::json!({
                "object_id": object.object_id,
                "role": object.role,
                "label": object.label,
                "bounds": {"x": object.bounds.x, "y": object.bounds.y, "width": object.bounds.width, "height": object.bounds.height},
                "stable_id": object.stable_id,
            })
        })
        .collect::<Vec<_>>();
    let pointers = context
        .pointer_events
        .iter()
        .map(|pointer| {
            serde_json::json!({"x": pointer.x, "y": pointer.y, "event": pointer.event, "snapshot_generation": pointer.snapshot_generation})
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "surface_snapshot_generation": context.surface_snapshot_generation,
        "objects": objects,
        "pointer_events": pointers,
        "focus_object_id": context.focus_object_id,
    })
}

fn map_responsibility(
    responsibility: AssetResponsibility,
) -> Result<SemanticResponsibility, AssetError> {
    Ok(match responsibility {
        AssetResponsibility::IntentInterpretation => SemanticResponsibility::IntentInterpretation,
        AssetResponsibility::ReferentResolution => SemanticResponsibility::ReferentResolution,
        AssetResponsibility::TaskAssociation => SemanticResponsibility::TaskAssociation,
        AssetResponsibility::ClarificationDecision => SemanticResponsibility::ClarificationDecision,
        AssetResponsibility::ExecutionRouteSelection => {
            SemanticResponsibility::ExecutionRouteSelection
        }
        AssetResponsibility::AgentSelection => SemanticResponsibility::AgentSelection,
        AssetResponsibility::ResultBinding => SemanticResponsibility::ResultAssociation,
        unsupported => {
            return Err(AssetError::Invalid(format!(
                "unsupported runtime responsibility {unsupported:?}"
            )));
        }
    })
}

fn behavior_status(class: BehaviorClass) -> ModelStatus {
    match class {
        BehaviorClass::Malformed => ModelStatus::Malformed,
        BehaviorClass::Timeout => ModelStatus::TimedOut,
        BehaviorClass::NoResponse => ModelStatus::NoResponse,
        _ => ModelStatus::Completed,
    }
}

fn replay_output(data: &Value) -> String {
    let object = match data.as_object() {
        Some(object) => object,
        None => return data.to_string(),
    };
    if object.contains_key("raw_structured_output") {
        return object["raw_structured_output"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
    }
    let mut tokens = Vec::new();
    if let Some(goal) = object.get("normalized_goal").and_then(Value::as_str) {
        tokens.push(match goal {
            "decrease_volume" => "LOCAL_VOLUME",
            "check_wifi_status" => "DIAGNOSE_WIFI",
            "check_dns" => "CONTINUE_T1",
            "open_referred_document" => "OPEN_RIGHT_DOCUMENT",
            "report_latest_download_name" => "LOOKUP_LATEST_DOWNLOAD",
            "organize_downloads" => "ORGANIZE_DOWNLOADS",
            "pause_media_and_organize_downloads" => "COMPOUND_MEDIA_DOWNLOADS",
            _ => "GENERAL_FILE_WORK",
        });
    }
    if object.get("resolution").and_then(Value::as_str) == Some("AMBIGUOUS") {
        tokens.push("AMBIGUOUS_DOCUMENT");
    }
    if object
        .get("referent_binding")
        .and_then(|binding| binding.get("object_id"))
        .and_then(Value::as_str)
        == Some("doc-right")
    {
        tokens.push("OPEN_RIGHT_DOCUMENT");
    }
    if let Some(requirement) = object
        .get("semantic_execution_requirement")
        .and_then(Value::as_str)
    {
        tokens.push(match requirement {
            "VOLUME_DECREASE" => "LOCAL_VOLUME",
            "NETWORK_SPECIALIST" => "NETWORK_AGENT",
            "FILE_OPEN" => "LOCAL_DOCUMENT",
            "COMPOUND_LOCAL_AND_GENERAL" => "COMPOUND_LOCAL_AND_GENERAL",
            _ => "ARGO",
        });
    }
    if object
        .get("allowed_canonical_routes")
        .and_then(Value::as_array)
        .is_some_and(|routes| {
            routes
                .iter()
                .filter_map(Value::as_str)
                .any(|route| route == "EXECUTOR_DIRECT:ARGO")
        })
    {
        tokens.push("ARGO_DIRECT");
    }
    if let Some(executor) = object.get("proposed_executor_id").and_then(Value::as_str) {
        tokens.push(match executor {
            "NetworkAgent" => "NETWORK_AGENT",
            "MailAgent" => "MAIL_AGENT",
            "FileAgent" => "FILE_AGENT",
            other => other,
        });
    }
    if object.contains_key("proposed_route") {
        let executor = object["proposed_route"]
            .get("initial_executor_id")
            .and_then(Value::as_str)
            .unwrap_or("VIA_FAST");
        tokens.push(match executor {
            "MailAgent" => "MAIL_AGENT",
            "NetworkAgent" => "NETWORK_AGENT",
            _ => "LOCAL_VOLUME",
        });
    }
    if tokens.is_empty() {
        data.to_string()
    } else {
        tokens.join(" ")
    }
}
