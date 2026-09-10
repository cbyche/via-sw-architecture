//! Strict Pilot-v0 asset loading and cross-reference validation.
//!
//! The scenario object is benchmark-owned. Only its `stimulus` projection may
//! later be materialized into AUT inputs; oracle registries are loaded through
//! a separate evaluator-only path.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PILOT_CORPUS_SCHEMA_VERSION: &str = "pilot-corpus-index-v0";
pub const RUNTIME_SCENARIO_SCHEMA_VERSION: &str = "runtime-scenario-pilot-v0";
pub const BEHAVIOR_PLAN_SCHEMA_VERSION: &str = "semantic-behavior-plan-pilot-v0";
pub const FIXTURE_REGISTRY_SCHEMA_VERSION: &str = "fixture-registry-pilot-v0";
pub const ORACLE_REGISTRY_SCHEMA_VERSION: &str = "oracle-registry-pilot-v0";
pub const FIXTURE_ASSET_SCHEMA_VERSION: &str = "fixture-asset-pilot-v0";
pub const ORACLE_ASSET_SCHEMA_VERSION: &str = "oracle-asset-pilot-v0";

#[derive(Debug)]
pub enum AssetError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    Invalid(String),
}

impl fmt::Display for AssetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Json { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Invalid(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for AssetError {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetRef {
    pub asset_id: String,
    pub asset_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileRef {
    pub scenario_id: String,
    pub path: String,
    pub scenario_class: ScenarioClass,
    pub qa_eligibility: QaEligibility,
    pub behavior_plan_ref: AssetRef,
    pub fixture_refs: Vec<AssetRef>,
    pub oracle_ref: AssetRef,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageGap {
    pub scenario_class: ScenarioClass,
    pub status: CoverageStatus,
    pub required_before: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CoverageStatus {
    NotCovered,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PilotCorpusIndex {
    pub schema_version: String,
    pub pilot_corpus_id: String,
    pub pilot_corpus_version: String,
    pub status: String,
    pub final_evaluation: bool,
    pub behavior_plan_registry_path: String,
    pub fixture_registry_path: String,
    pub oracle_registry_path: String,
    pub scenarios: Vec<FileRef>,
    pub coverage_gaps: Vec<CoverageGap>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScenarioClass {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QaFlag {
    pub eligible: bool,
    pub exclusion_reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Qa04Flag {
    pub eligible: bool,
    pub route_commit_expectation: RouteCommitExpectation,
    pub exclusion_reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RouteCommitExpectation {
    Required,
    Forbidden,
    Optional,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QaEligibility {
    pub qa01: QaFlag,
    pub qa02: QaFlag,
    pub qa04: Qa04Flag,
    pub mandatory_gate_refs: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Modality {
    Voice,
    Text,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptedReply {
    pub trigger: String,
    pub turn_id: String,
    pub modality: Modality,
    pub text_fixture: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InteractionTurn {
    pub turn_id: String,
    pub modality: Modality,
    pub text_fixture: String,
    pub audio_fixture_ref: String,
    pub acoustic_eos_fixture_ref: String,
    pub input_available_offset_micros: u64,
    pub deterministic_user_replies: Vec<ScriptedReply>,
    pub fixture_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stimulus {
    pub interaction: Vec<InteractionTurn>,
    pub context_evidence_ref: AssetRef,
    pub initial_state_ref: AssetRef,
    pub capability_profile_ref: AssetRef,
    pub health_profile_ref: AssetRef,
    pub policy_profile_ref: AssetRef,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyFixtureRefs {
    pub agent_scripts: Vec<AssetRef>,
    pub tool_scripts: Vec<AssetRef>,
    pub latency_profile_ref: AssetRef,
    pub interaction_frontend_profile_ref: AssetRef,
    pub outcome_probe_profile_ref: AssetRef,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluatorOracleRefs {
    pub constraint_manifest_ref: String,
    pub success_predicate_ref: String,
    pub expected_result_binding_ref: String,
    pub ground_truth_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Termination {
    pub success_condition_ref: String,
    pub failure_condition_refs: Vec<String>,
    pub timeout_policy: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioProvenance {
    pub scenario_schema_version: String,
    pub semantic_behavior_plan_version: String,
    pub constraint_manifest_version: String,
    pub success_predicate_version: String,
    pub fixture_registry_version: String,
    pub canonical_event_schema_version: String,
    pub benchmark_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeScenario {
    pub schema_version: String,
    pub asset_id: String,
    pub asset_version: String,
    pub scenario_id: String,
    pub scenario_version: String,
    pub catalog_scenario_ref: String,
    pub catalog_version: String,
    pub title: String,
    pub scenario_class: ScenarioClass,
    pub architecture_sensitivity_tags: Vec<String>,
    pub description: String,
    pub qa_eligibility: QaEligibility,
    pub stimulus: Stimulus,
    pub semantic_behavior_plan_ref: AssetRef,
    pub dependency_fixture_refs: DependencyFixtureRefs,
    pub evaluator_oracle_ref: AssetRef,
    pub evaluator_oracle_refs: EvaluatorOracleRefs,
    pub termination: Termination,
    pub provenance: ScenarioProvenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SemanticResponsibility {
    IntentInterpretation,
    ReferentResolution,
    TaskAssociation,
    ClarificationDecision,
    ExecutionRouteSelection,
    AgentSelection,
    CompoundDecomposition,
    ValidationDecision,
    ResultBinding,
    ResponseGeneration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BehaviorClass {
    Correct,
    Ambiguous,
    LowConfidence,
    WrongCandidate,
    Partial,
    Malformed,
    Timeout,
    NoResponse,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorAttempt {
    pub attempt: u32,
    pub behavior_class: BehaviorClass,
    pub payload_ref: String,
    pub failure_behavior: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticOperation {
    pub operation_key: String,
    pub turn_id: String,
    pub responsibility: SemanticResponsibility,
    pub input_contract_id: String,
    pub output_schema_id: String,
    pub operation_group: String,
    pub attempts: Vec<BehaviorAttempt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorPlan {
    pub schema_version: String,
    pub asset_id: String,
    pub asset_version: String,
    pub behavior_plan_id: String,
    pub behavior_plan_version: String,
    pub scenario_id: String,
    pub scenario_version: String,
    pub semantic_schema_version: String,
    pub payload_registry_version: String,
    pub allowed_owner_mapping_version: String,
    pub operations: Vec<SemanticOperation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorPlanRegistry {
    pub schema_version: String,
    pub asset_id: String,
    pub asset_version: String,
    pub plans: Vec<BehaviorPlan>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FixtureKind {
    Audio,
    InteractionFrontend,
    ContextEvidence,
    InitialState,
    CapabilityProfile,
    HealthProfile,
    PolicyProfile,
    AgentScript,
    ToolScript,
    LatencyProfile,
    OutcomeProbe,
    ReplayPayload,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureAsset {
    pub schema_version: String,
    pub asset_id: String,
    pub asset_version: String,
    pub fixture_kind: FixtureKind,
    pub description: String,
    pub data: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureRegistry {
    pub schema_version: String,
    pub asset_id: String,
    pub asset_version: String,
    pub unknown_field_policy: String,
    pub fixtures: Vec<FixtureAsset>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConstraintSet {
    Required,
    Allowed,
    Forbidden,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constraint {
    pub constraint_id: String,
    pub constraint_set: ConstraintSet,
    pub dimension: String,
    pub operator: String,
    pub expected: Value,
    pub severity: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstraintManifest {
    pub constraint_manifest_id: String,
    pub constraint_manifest_version: String,
    pub taxonomy_version: String,
    pub primary_category: String,
    pub secondary_categories: Vec<String>,
    pub corpus_version: String,
    pub eligible_for_aecr: bool,
    pub fixture_id: String,
    pub semantic_trace_set_id: String,
    pub constraints: Vec<Constraint>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessPredicate {
    pub success_predicate_id: String,
    pub success_predicate_version: String,
    pub probe_fixture_ref: String,
    pub observable_state_path: String,
    pub operator: String,
    pub expected: Value,
    pub architecture_internal_state_used: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OracleAsset {
    pub schema_version: String,
    pub asset_id: String,
    pub asset_version: String,
    pub scenario_id: String,
    pub ground_truth: Value,
    pub constraint_manifest: ConstraintManifest,
    pub expected_result_binding: Value,
    pub success_predicate: SuccessPredicate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OracleRegistry {
    pub schema_version: String,
    pub asset_id: String,
    pub asset_version: String,
    pub visibility: String,
    pub oracles: Vec<OracleAsset>,
}

#[derive(Debug)]
pub struct PilotCorpus {
    pub index: PilotCorpusIndex,
    pub scenarios: Vec<RuntimeScenario>,
    pub behavior_plans: BehaviorPlanRegistry,
    pub fixtures: FixtureRegistry,
    pub oracles: OracleRegistry,
}

fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, AssetError> {
    let bytes = fs::read(path).map_err(|source| AssetError::Io {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| AssetError::Json {
        path: path.to_owned(),
        source,
    })
}

pub fn load_runtime_scenario(path: &Path) -> Result<RuntimeScenario, AssetError> {
    load_json(path)
}

pub fn load_behavior_plan(path: &Path) -> Result<BehaviorPlan, AssetError> {
    load_json(path)
}

pub fn load_pilot_corpus(repository_root: &Path) -> Result<PilotCorpus, AssetError> {
    let index_path = repository_root.join("benchmark/scenarios/pilot-v0/index.json");
    let index: PilotCorpusIndex = load_json(&index_path)?;
    let scenarios = index
        .scenarios
        .iter()
        .map(|entry| load_json(&repository_root.join(&entry.path)))
        .collect::<Result<Vec<_>, _>>()?;
    let behavior_plans = load_json(&repository_root.join(&index.behavior_plan_registry_path))?;
    let fixtures = load_json(&repository_root.join(&index.fixture_registry_path))?;
    let oracles = load_json(&repository_root.join(&index.oracle_registry_path))?;
    let corpus = PilotCorpus {
        index,
        scenarios,
        behavior_plans,
        fixtures,
        oracles,
    };
    validate_pilot_corpus(&corpus)?;
    Ok(corpus)
}

pub fn validate_pilot_corpus(corpus: &PilotCorpus) -> Result<(), AssetError> {
    require_equal(
        "corpus schema version",
        &corpus.index.schema_version,
        PILOT_CORPUS_SCHEMA_VERSION,
    )?;
    require_equal(
        "behavior registry schema version",
        &corpus.behavior_plans.schema_version,
        BEHAVIOR_PLAN_SCHEMA_VERSION,
    )?;
    require_equal(
        "fixture registry schema version",
        &corpus.fixtures.schema_version,
        FIXTURE_REGISTRY_SCHEMA_VERSION,
    )?;
    require_equal(
        "oracle registry schema version",
        &corpus.oracles.schema_version,
        ORACLE_REGISTRY_SCHEMA_VERSION,
    )?;
    if corpus.index.final_evaluation {
        return invalid("Pilot-v0 index must not identify itself as final evaluation");
    }
    if corpus.scenarios.len() != 10 || corpus.index.scenarios.len() != 10 {
        return invalid("Pilot-v0 must contain exactly P01 through P10");
    }

    let fixtures = unique_map(
        "fixture",
        corpus
            .fixtures
            .fixtures
            .iter()
            .map(|asset| (&asset.asset_id, asset)),
    )?;
    let plans = unique_map(
        "behavior plan",
        corpus
            .behavior_plans
            .plans
            .iter()
            .map(|asset| (&asset.asset_id, asset)),
    )?;
    let oracles = unique_map(
        "oracle",
        corpus
            .oracles
            .oracles
            .iter()
            .map(|asset| (&asset.asset_id, asset)),
    )?;
    let scenario_ids: HashSet<_> = corpus
        .scenarios
        .iter()
        .map(|s| s.scenario_id.as_str())
        .collect();
    let expected_ids: HashSet<_> = (1..=10).map(|number| format!("P{number:02}")).collect();
    if scenario_ids != expected_ids.iter().map(String::as_str).collect() {
        return invalid("scenario ids must be exactly P01 through P10");
    }

    let indexed: HashMap<_, _> = corpus
        .index
        .scenarios
        .iter()
        .map(|entry| (entry.scenario_id.as_str(), entry))
        .collect();
    if indexed.len() != 10 {
        return invalid("Pilot-v0 index has duplicate scenario ids");
    }
    for scenario in &corpus.scenarios {
        validate_scenario(scenario, &fixtures, &plans, &oracles)?;
        let entry = indexed.get(scenario.scenario_id.as_str()).ok_or_else(|| {
            AssetError::Invalid(format!(
                "{} is absent from the corpus index",
                scenario.scenario_id
            ))
        })?;
        if entry.scenario_class != scenario.scenario_class
            || entry.qa_eligibility != scenario.qa_eligibility
            || entry.behavior_plan_ref != scenario.semantic_behavior_plan_ref
            || entry.oracle_ref != scenario.evaluator_oracle_ref
        {
            return invalid(format!(
                "{} index metadata differs from the scenario asset",
                scenario.scenario_id
            ));
        }
    }
    validate_behavior_plans(&corpus.behavior_plans, &fixtures)?;
    validate_fixtures(&corpus.fixtures)?;
    validate_oracles(&corpus.oracles, &fixtures)?;
    validate_pilot_semantics(corpus)?;

    let gaps: HashSet<_> = corpus
        .index
        .coverage_gaps
        .iter()
        .map(|gap| gap.scenario_class)
        .collect();
    if gaps != HashSet::from([ScenarioClass::R8, ScenarioClass::R9]) {
        return invalid("coverage gaps must explicitly contain R8 and R9");
    }
    Ok(())
}

fn validate_scenario<'a>(
    scenario: &RuntimeScenario,
    fixtures: &HashMap<&'a String, &'a FixtureAsset>,
    plans: &HashMap<&'a String, &'a BehaviorPlan>,
    oracles: &HashMap<&'a String, &'a OracleAsset>,
) -> Result<(), AssetError> {
    require_equal(
        "runtime scenario schema version",
        &scenario.schema_version,
        RUNTIME_SCENARIO_SCHEMA_VERSION,
    )?;
    if scenario.asset_id != scenario.scenario_id {
        return invalid(format!(
            "{} asset_id must equal scenario_id",
            scenario.scenario_id
        ));
    }
    let plan = plans
        .get(&scenario.semantic_behavior_plan_ref.asset_id)
        .ok_or_else(|| {
            AssetError::Invalid(format!(
                "{} behavior plan ref is unresolved",
                scenario.scenario_id
            ))
        })?;
    if plan.scenario_id != scenario.scenario_id
        || plan.asset_version != scenario.semantic_behavior_plan_ref.asset_version
    {
        return invalid(format!(
            "{} behavior plan identity mismatch",
            scenario.scenario_id
        ));
    }
    let oracle = oracles
        .get(&scenario.evaluator_oracle_ref.asset_id)
        .ok_or_else(|| {
            AssetError::Invalid(format!("{} oracle ref is unresolved", scenario.scenario_id))
        })?;
    if oracle.scenario_id != scenario.scenario_id
        || oracle.asset_version != scenario.evaluator_oracle_ref.asset_version
    {
        return invalid(format!("{} oracle identity mismatch", scenario.scenario_id));
    }

    let mut fixture_refs = vec![
        &scenario.stimulus.context_evidence_ref,
        &scenario.stimulus.initial_state_ref,
        &scenario.stimulus.capability_profile_ref,
        &scenario.stimulus.health_profile_ref,
        &scenario.stimulus.policy_profile_ref,
        &scenario.dependency_fixture_refs.latency_profile_ref,
        &scenario
            .dependency_fixture_refs
            .interaction_frontend_profile_ref,
        &scenario.dependency_fixture_refs.outcome_probe_profile_ref,
    ];
    fixture_refs.extend(scenario.dependency_fixture_refs.agent_scripts.iter());
    fixture_refs.extend(scenario.dependency_fixture_refs.tool_scripts.iter());
    for fixture_ref in fixture_refs {
        let fixture = fixtures.get(&fixture_ref.asset_id).ok_or_else(|| {
            AssetError::Invalid(format!(
                "{} fixture ref {} is unresolved",
                scenario.scenario_id, fixture_ref.asset_id
            ))
        })?;
        if fixture.asset_version != fixture_ref.asset_version {
            return invalid(format!(
                "{} fixture version mismatch for {}",
                scenario.scenario_id, fixture_ref.asset_id
            ));
        }
    }

    if scenario.qa_eligibility.qa01.eligible
        && (scenario.stimulus.interaction.is_empty()
            || scenario.stimulus.interaction.iter().any(|turn| {
                turn.modality != Modality::Voice
                    || turn.audio_fixture_ref == "NONE"
                    || turn.acoustic_eos_fixture_ref == "NONE"
                    || !fixtures.contains_key(&turn.audio_fixture_ref)
                    || !fixtures.contains_key(&turn.acoustic_eos_fixture_ref)
            }))
    {
        return invalid(format!(
            "{} is QA-01 eligible without a resolvable Voice/acoustic-EOS fixture",
            scenario.scenario_id
        ));
    }
    if scenario.qa_eligibility.qa02.eligible && oracle.constraint_manifest.constraints.is_empty() {
        return invalid(format!(
            "{} is QA-02 eligible without constraints",
            scenario.scenario_id
        ));
    }

    let stimulus = serde_json::to_value(&scenario.stimulus)
        .map_err(|error| AssetError::Invalid(error.to_string()))?;
    reject_forbidden_keys(&stimulus, &scenario.scenario_id)?;
    Ok(())
}

fn validate_behavior_plans(
    registry: &BehaviorPlanRegistry,
    fixtures: &HashMap<&String, &FixtureAsset>,
) -> Result<(), AssetError> {
    for plan in &registry.plans {
        require_equal(
            "behavior plan schema version",
            &plan.schema_version,
            BEHAVIOR_PLAN_SCHEMA_VERSION,
        )?;
        if plan.operations.is_empty() {
            return invalid(format!("{} has no operations", plan.asset_id));
        }
        let mut operation_keys = HashSet::new();
        for operation in &plan.operations {
            if !operation_keys.insert(&operation.operation_key) {
                return invalid(format!("{} has duplicate operation keys", plan.asset_id));
            }
            if operation.attempts.is_empty() {
                return invalid(format!(
                    "{} has an operation with no attempts",
                    plan.asset_id
                ));
            }
            for (index, attempt) in operation.attempts.iter().enumerate() {
                if attempt.attempt != u32::try_from(index + 1).unwrap_or(u32::MAX) {
                    return invalid(format!("{} attempts must be consecutive", plan.asset_id));
                }
                match attempt.behavior_class {
                    BehaviorClass::Timeout | BehaviorClass::NoResponse => {
                        if attempt.failure_behavior == "NONE" {
                            return invalid(format!(
                                "{} terminal behavior lacks failure semantics",
                                plan.asset_id
                            ));
                        }
                    }
                    _ => {
                        if attempt.payload_ref == "NONE"
                            || !fixtures.contains_key(&attempt.payload_ref)
                        {
                            return invalid(format!("{} payload ref is unresolved", plan.asset_id));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_fixtures(registry: &FixtureRegistry) -> Result<(), AssetError> {
    if registry.unknown_field_policy
        != "REJECT_UNKNOWN_AT_TYPED_BOUNDARIES; DATA_IS_VERSIONED_BY_FIXTURE_KIND"
    {
        return invalid("fixture unknown-field policy is not the Pilot-v0 policy");
    }
    for fixture in &registry.fixtures {
        require_equal(
            "fixture asset schema version",
            &fixture.schema_version,
            FIXTURE_ASSET_SCHEMA_VERSION,
        )?;
        if fixture.fixture_kind == FixtureKind::ContextEvidence {
            reject_forbidden_keys(&fixture.data, &fixture.asset_id)?;
        }
        if fixture.fixture_kind == FixtureKind::Audio {
            let object = fixture.data.as_object().ok_or_else(|| {
                AssetError::Invalid(format!("{} audio data must be an object", fixture.asset_id))
            })?;
            if !object.contains_key("audio_fixture_identity")
                || !object.contains_key("ground_truth_acoustic_eos_offset_micros")
                || object
                    .get("eos_annotation_authority")
                    .and_then(Value::as_str)
                    != Some("BENCHMARK_FIXTURE")
                || object
                    .get("runtime_vad_is_authoritative")
                    .and_then(Value::as_bool)
                    == Some(true)
            {
                return invalid(format!(
                    "{} lacks authoritative acoustic-EOS metadata",
                    fixture.asset_id
                ));
            }
        }
    }
    Ok(())
}

fn validate_oracles(
    registry: &OracleRegistry,
    fixtures: &HashMap<&String, &FixtureAsset>,
) -> Result<(), AssetError> {
    if registry.visibility != "EVALUATOR_ONLY" {
        return invalid("oracle registry visibility must be EVALUATOR_ONLY");
    }
    for oracle in &registry.oracles {
        require_equal(
            "oracle asset schema version",
            &oracle.schema_version,
            ORACLE_ASSET_SCHEMA_VERSION,
        )?;
        if oracle.constraint_manifest.constraints.is_empty() {
            return invalid(format!("{} has no constraints", oracle.asset_id));
        }
        let mut constraint_ids = HashSet::new();
        for constraint in &oracle.constraint_manifest.constraints {
            if !constraint_ids.insert(&constraint.constraint_id) {
                return invalid(format!("{} has duplicate constraints", oracle.asset_id));
            }
        }
        if oracle.success_predicate.architecture_internal_state_used {
            return invalid(format!(
                "{} success predicate uses AUT internals",
                oracle.asset_id
            ));
        }
        if !fixtures.contains_key(&oracle.success_predicate.probe_fixture_ref) {
            return invalid(format!("{} probe fixture is unresolved", oracle.asset_id));
        }
    }
    Ok(())
}

fn validate_pilot_semantics(corpus: &PilotCorpus) -> Result<(), AssetError> {
    let qa01_ids: HashSet<_> = corpus
        .scenarios
        .iter()
        .filter(|scenario| scenario.qa_eligibility.qa01.eligible)
        .map(|scenario| scenario.scenario_id.as_str())
        .collect();
    if qa01_ids != HashSet::from(["P01", "P02", "P03"]) {
        return invalid("QA-01 Pilot population must be exactly P01, P02 and P03");
    }

    for scenario in &corpus.scenarios {
        let expected = match scenario.scenario_id.as_str() {
            "P08" | "P10" => RouteCommitExpectation::Forbidden,
            "P09" => RouteCommitExpectation::Optional,
            _ => RouteCommitExpectation::Required,
        };
        if scenario.qa_eligibility.qa04.route_commit_expectation != expected {
            return invalid(format!(
                "{} route-commit expectation must be {expected:?}",
                scenario.scenario_id
            ));
        }
    }

    let p09_plan = corpus
        .behavior_plans
        .plans
        .iter()
        .find(|plan| plan.scenario_id == "P09")
        .ok_or_else(|| AssetError::Invalid("P09 behavior plan is absent".into()))?;
    let p09_payload_ids: HashSet<_> = p09_plan
        .operations
        .iter()
        .flat_map(|operation| operation.attempts.iter())
        .map(|attempt| attempt.payload_ref.as_str())
        .collect();
    if !p09_payload_ids.contains("PAYLOAD-WRONG-MAIL-ROUTE-v1")
        || !p09_payload_ids.contains("PAYLOAD-WRONG-MAIL-AGENT-v1")
    {
        return invalid("P09 does not inject the frozen wrong-but-valid MailAgent proposal");
    }

    let p09_oracle = corpus
        .oracles
        .oracles
        .iter()
        .find(|oracle| oracle.scenario_id == "P09")
        .ok_or_else(|| AssetError::Invalid("P09 oracle is absent".into()))?;
    let ground_truth = p09_oracle.ground_truth.as_object().ok_or_else(|| {
        AssetError::Invalid("P09 ground truth must be a structured object".into())
    })?;
    for field in [
        "wrong_executor_exists",
        "wrong_executor_registered",
        "wrong_executor_healthy",
        "wrong_request_structurally_valid",
    ] {
        if ground_truth.get(field).and_then(Value::as_bool) != Some(true) {
            return invalid(format!("P09 ground truth requires {field}=true"));
        }
    }
    if ground_truth
        .get("wrong_executor_semantically_correct")
        .and_then(Value::as_bool)
        != Some(false)
    {
        return invalid("P09 wrong executor must be semantically incorrect");
    }
    Ok(())
}

fn reject_forbidden_keys(value: &Value, scenario_id: &str) -> Result<(), AssetError> {
    const FORBIDDEN: [&str; 8] = [
        "correct_referent",
        "selected_referent",
        "expected_source_file",
        "expected_destination",
        "clarification_required",
        "expected_task_id",
        "expected_agent",
        "expected_execution_owner",
    ];
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                if FORBIDDEN.contains(&key.as_str()) {
                    return invalid(format!("{scenario_id} stimulus leaks forbidden key {key}"));
                }
                reject_forbidden_keys(nested, scenario_id)?;
            }
        }
        Value::Array(values) => {
            for nested in values {
                reject_forbidden_keys(nested, scenario_id)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn unique_map<'a, T>(
    kind: &str,
    values: impl Iterator<Item = (&'a String, &'a T)>,
) -> Result<HashMap<&'a String, &'a T>, AssetError> {
    let mut result = HashMap::new();
    for (id, value) in values {
        if result.insert(id, value).is_some() {
            return invalid(format!("duplicate {kind} id {id}"));
        }
    }
    Ok(result)
}

fn require_equal(label: &str, actual: &str, expected: &str) -> Result<(), AssetError> {
    if actual == expected {
        Ok(())
    } else {
        invalid(format!("{label}: expected {expected}, got {actual}"))
    }
}

fn invalid<T>(message: impl Into<String>) -> Result<T, AssetError> {
    Err(AssetError::Invalid(message.into()))
}
