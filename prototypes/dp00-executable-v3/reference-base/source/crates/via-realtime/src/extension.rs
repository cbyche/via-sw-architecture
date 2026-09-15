//! The seam a host plugs a provider into at runtime.
//!
//! Ported from `server/src/voice/realtime-provider-extension.mjs`, which is eight
//! lines of re-exports and is exactly as important as it is short: it is the
//! difference between "VIA ships five providers" and "VIA has a provider
//! interface". `via-realtime-mock`, `via-realtime-dashscope` and every local
//! provider come in through this door, and so does anything an embedder writes.
//!
//! Upstream re-exports five names plus one protocol adapter. VIA re-exports the
//! same set, plus the second dialect — upstream's `ga-protocol.mjs` is reachable
//! only through `providers/s2s.mjs`, which means a host that wants to speak the
//! GA dialect has to import a *provider* to get at it. That is an oversight
//! rather than a contract, and it costs one line to fix.
//!
//! ```
//! use std::sync::Arc;
//!
//! use via_realtime::extension::{
//!     RealtimeProviderRegistry, define_realtime_provider, openai_compatible_protocol,
//! };
//! use via_realtime::testing::TestProvider;
//!
//! // A host validates its provider once, up front …
//! let provider = define_realtime_provider(Arc::new(
//!     TestProvider::new("house-provider")
//!         .with_aliases(["house"])
//!         .with_protocol(Box::new(openai_compatible_protocol())),
//! ))?;
//!
//! // … and the registry validates it again on the way in, so a provider that
//! // reached the table is one that passed.
//! let mut registry = RealtimeProviderRegistry::with_default("house");
//! registry.register(provider)?;
//! assert_eq!(registry.resolve(Some("HOUSE"))?.key(), "house-provider");
//! # Ok::<(), via_realtime::RealtimeError>(())
//! ```
//!
//! # What makes it a real extension point
//!
//! Three things, and none of them is the trait on its own:
//!
//! **A provider is validated at registration, not discovered mid-turn.**
//! [`define_realtime_provider`] and
//! [`RealtimeProviderRegistry::register`] both run
//! [`validate_realtime_provider`], so a key a client could never send, a blank
//! alias, or a model profile with no voice is refused while the Gateway is still
//! starting.
//!
//! **A name is claimed once.** Registering over another provider's key or alias
//! is an error rather than a silent shadowing, because the alias table is what a
//! client's provider selection resolves through.
//!
//! **The dialect is separable from the service.** A host that speaks an existing
//! dialect implements [`RealtimeProvider`] and returns one of the two shipped
//! adapters from [`RealtimeProvider::protocol`]; only a host with a genuinely new
//! wire format implements [`RealtimeProtocol`]. That split is what will let a
//! local model reuse the OpenAI-Realtime dialect without reimplementing it.

pub use crate::capabilities::{CAPABILITY_FLAGS, ProviderCapabilities};
pub use crate::error::RealtimeError;
pub use crate::event_error::ErrorClass;
pub use crate::protocol::{
    GaRealtimeProtocol, OpenAiCompatibleProtocol, PROTOCOL_METHODS, RealtimeProtocol,
    ga_realtime_protocol, openai_compatible_protocol,
};
pub use crate::provider::{
    AgentContext, AgentContextPatch, ConnectionInfo, Injection, InputProjection,
    InputProjectionRequest, PROVIDER_METHODS, PermissionRequest, RealtimeProvider, SessionRequest,
    Visibility, clean_key, define_realtime_provider, is_valid_provider_key,
    validate_realtime_provider,
};
pub use crate::registry::{
    ActiveRealtime, ListOptions, ProviderDescriptor, RealtimeProviderRegistry, describe_provider,
};
pub use crate::session::{RealtimeSession, SessionOptions, Transport};
