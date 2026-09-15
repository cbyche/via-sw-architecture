//! VIA's frozen contract surface.
//!
//! Everything an external party can observe about the Gateway and cannot be
//! changed without breaking someone: the protocol version and capability list,
//! the three WebSocket event vocabularies, the session mode, the Work record's
//! public shape, and the coded errors that belong at this layer.
//!
//! This crate is a **leaf**: it depends on no other VIA crate
//! (`docs/architecture.md` §9), so nothing here can grow a dependency on
//! runtime behaviour and quietly stop being a contract.
//!
//! # Where the values come from
//!
//! | Surface | Upstream |
//! | --- | --- |
//! | [`GATEWAY_PROTOCOL_VERSION`], [`GATEWAY_CAPABILITIES`] | `server/src/core/gateway-protocol.mjs` |
//! | [`GatewayClientEvent`], [`GatewayServerEvent`], [`GatewayTaskEvent`] | `shared/realtime-events.mjs` |
//! | [`WorkStatus`], [`WorkState`], [`WorkKind`] | `server/src/task/task-manager.mjs` |
//! | [`ClientType`], [`ClientInputCapabilities`] | `shared/client-input-capabilities.mjs` |
//! | [`ProtocolError`] codes | `shared/gateway-setup.mjs`, `shared/gateway-instance-lock.mjs`, `server/src/voice/input-arbitration.mjs` |
//! | [`SessionMode`] | **new to VIA** — `docs/architecture.md` §2 |
//!
//! Values that reproduce an upstream literal say so in their own
//! documentation and name the file they came from. The upstream is
//! `QwenAudio/qwen-audio-agent` v1.11.0, surveyed 2026-08-22; the machine-
//! readable acceptance criteria are `docs/reference/contracts.json`.
//!
//! # Direction is a type
//!
//! Upstream gates event direction at runtime with two `Set`s. Here it is the
//! type system's job — a [`GatewayServerEvent`] cannot be sent by a client
//! because the inbound decoder only ever yields a [`GatewayClientEvent`]. The
//! runtime gate is still available where a frame arrives as a bare string:
//! every vocabulary offers `as_str`, `from_wire`, [`Display`](core::fmt::Display)
//! and [`FromStr`](core::str::FromStr), and [`GatewayOutboundEvent`] reproduces
//! the union set upstream validates outbound frames against.
//!
//! ```
//! use via_protocol::{GatewayClientEvent, GatewayServerEvent, GatewayOutboundEvent};
//!
//! // A wire string parses only in its own direction.
//! assert_eq!(
//!     GatewayClientEvent::from_wire("audio.append"),
//!     Some(GatewayClientEvent::AudioAppend),
//! );
//! assert_eq!(GatewayClientEvent::from_wire("audio.delta"), None);
//! assert_eq!(
//!     GatewayOutboundEvent::from_wire("audio.delta"),
//!     Some(GatewayOutboundEvent::Session(GatewayServerEvent::AudioDelta)),
//! );
//! assert_eq!(GatewayOutboundEvent::from_wire("audio.append"), None);
//! ```
//!
//! # Map order is a contract
//!
//! The workspace configures `serde_json` with `preserve_order` because two
//! external contracts depend on JSON map insertion order. Struct field order in
//! this crate is therefore deliberate, and a `HashMap` must never stand where
//! order is observable.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]

mod macros;

pub mod client;
pub mod error;
pub mod events;
pub mod gateway;
pub mod session;
pub mod work;

pub use client::{
    CLIENT_INPUT_CAPABILITY_FIELDS, ClientInputCapabilities, ClientType, DEFAULT_CLIENT_TYPE,
};
pub use error::{
    CODE_GATEWAY_ALREADY_RUNNING, CODE_GATEWAY_SETUP_REQUIRED, CODE_ILLEGAL_TRANSITION,
    CODE_INPUT_OWNER_REQUIRED, CODE_UNKNOWN_WIRE_VALUE, CONTRACT_ERROR_CODES, MissingSetting,
    ProtocolError,
};
pub use events::{GatewayClientEvent, GatewayOutboundEvent, GatewayServerEvent, GatewayTaskEvent};
pub use gateway::{
    CAPABILITY_GATEWAY_INSTANCE_LEASE, CAPABILITY_GATEWAY_SETTINGS_STORE,
    CAPABILITY_GATEWAY_SETUP_GATE, CAPABILITY_INPUT_SUSPEND_ACK,
    CAPABILITY_INPUT_SUSPEND_CLEARS_PLAYBACK, CAPABILITY_INPUT_SUSPEND_PROTOCOL,
    CAPABILITY_INPUT_SUSPEND_TTL, DROPPED_UPSTREAM_CAPABILITIES, GATEWAY_CAPABILITIES,
    GATEWAY_PROTOCOL_VERSION, advertises_capability,
};
pub use session::SessionMode;
pub use work::{WorkKind, WorkState, WorkStatus};
