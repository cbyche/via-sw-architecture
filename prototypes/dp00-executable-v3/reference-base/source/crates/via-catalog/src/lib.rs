//! VIA's static tables.
//!
//! A leaf crate with no VIA dependencies (`docs/architecture.md` §9). It holds
//! the four catalogs every other layer reads and nobody else should restate:
//!
//! | Module | What | Upstream |
//! | --- | --- | --- |
//! | [`realtime_model`] | The four DashScope model profiles, plus VIA's `local` family | `shared/realtime-model-catalog.mjs` |
//! | [`realtime_provider`] | Provider keys, aliases, labels and default endpoints | `shared/realtime-provider-catalog.mjs` |
//! | [`signature`] | The `sha256` configuration signature on `/api/health` | `shared/realtime-provider-catalog.mjs:122-137` |
//! | [`backend`] | The twelve backend agents and their environment allow-policy | `shared/backend-catalog.mjs` |
//!
//! # Fidelity
//!
//! Nearly every literal in this crate is an **external contract**: a vendor model
//! id or voice id that goes on the wire, a provider key a client sends on the
//! connect event, a field order a published JSON payload depends on, or an
//! environment-variable allow-list that is a security boundary. Each is
//! doc-commented with the upstream file it came from, and
//! `docs/reference/contracts.json` is the acceptance spec. Vendor-owned strings —
//! DashScope model and voice ids, the `qwen` provider alias, the Qwen Code
//! backend and its `QWEN_CODE_` namespace — are KEEP under `docs/rebrand.md`;
//! only VIA's own identity strings (`QWEN_AUDIO_AGENT_*` → `VIA_*`, `QWAUDIO_*` →
//! `VIA_*`) are renamed.
//!
//! # Two deliberate divergences
//!
//! **An unknown realtime model id is an error.** Upstream answers one with an
//! all-capabilities-false profile, which `providers/dashscope.mjs:84-87` turns
//! into a session with no audio input and no turn detection — it connects and
//! never hears anything. [`realtime_model::resolve_dashscope_realtime_model_profile`]
//! returns `Err` instead, and [`realtime_model::ModelFamily::Local`] gives the
//! on-device providers real capability flags. See `docs/architecture.md` §7.
//!
//! **Map order is never incidental.** The workspace configures `serde_json` with
//! `preserve_order` because the configuration signature is a hash over an object
//! whose key order is JS insertion order. There is no `HashMap` in this crate.
//!
//! # Purity
//!
//! Nothing here reads the environment, the filesystem or the clock. Where a
//! table's meaning depends on an environment variable — the family-scoped voice
//! override, the `acp` backend's explicit forward list — this crate publishes the
//! *name* and the rule, and the caller does the lookup.

pub mod backend;
pub mod error;
pub mod realtime_model;
pub mod realtime_provider;
pub mod signature;

pub use backend::{
    BACKEND_NONE_SENTINEL, BackendDefinition, EnvironmentPolicy, Integration, Ownership,
    SkillsSpec, backend_definition, backend_definitions, backend_names,
    effective_backend_permission_mode, normalize_backend_protocol, resolve_backend_ownership,
};
pub use error::CatalogError;
pub use realtime_model::{
    DEFAULT_DASHSCOPE_REALTIME_MODEL, DEFAULT_DASHSCOPE_REALTIME_VOICE, ModelCapabilities,
    ModelFamily, ModelProfile, SessionDefaults, TransportCapabilities, TurnDetection,
    TurnDetectionKind, dashscope_realtime_model_profiles, local_realtime_model_profile,
    resolve_dashscope_realtime_model_profile,
};
pub use realtime_provider::{
    DEFAULT_DASHSCOPE_REALTIME_URL, DEFAULT_REALTIME_PROVIDER,
    DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL, IdentityShape, ProviderDefinition,
    normalize_realtime_provider, realtime_provider_definition, realtime_provider_names,
    realtime_providers,
};
pub use signature::RealtimeIdentity;
