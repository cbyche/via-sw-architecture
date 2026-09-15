//! Protocol version and capability list.
//!
//! Ported from `server/src/core/gateway-protocol.mjs`.
//!
//! Both values are echoed verbatim by `GET /api/health` as `protocolVersion`
//! and `capabilities`. Clients branch on a **capability**, never on a product
//! version, so a Gateway that predates a feature degrades instead of failing.
//!
//! ## Why VIA starts at `1.0.0` rather than inheriting upstream's `2.0.0`
//!
//! Upstream's own versioning rule (`gateway-protocol.mjs:1-18`) is that a
//! *removed* capability is a breaking change. VIA is core-only: the React SPA,
//! the Electron host, the floating orb and the skin store are dropped
//! (`docs/fidelity.md`, "Dropped"), so nine of upstream's sixteen capabilities
//! cannot honestly be advertised — the seven `web.*` / `desktop.*` entries plus
//! the two `host.*` entries, which name Node/Electron package entry points that
//! a Rust binary does not have.
//!
//! Claiming `2.0.0` while advertising a strict subset of `2.0.0`'s capabilities
//! would lie to exactly the clients the version exists for. VIA therefore
//! declares its own line starting at `1.0.0`, and the capability list — which
//! is what clients actually branch on — carries the truth.

/// The Gateway protocol version, echoed as `/api/health.protocolVersion`.
///
/// **VIA-owned value, not an upstream contract.** Upstream's
/// `GATEWAY_PROTOCOL_VERSION` is `'2.0.0'`
/// (`server/src/core/gateway-protocol.mjs:19`); VIA starts its own line at
/// `1.0.0` because it advertises a strict subset of upstream's capabilities and
/// a removed capability is a breaking change under upstream's own rule. See the
/// module documentation and `docs/fidelity.md`.
///
/// Must match `^\d+\.\d+\.\d+$`. The minor rises for an additive capability,
/// the major for a breaking change to any endpoint or event named in
/// [`GATEWAY_CAPABILITIES`].
pub const GATEWAY_PROTOCOL_VERSION: &str = "1.0.0";

/// A lease in the config directory names the running instance (origin,
/// `instanceId`, pid) and `/api/health` echoes `gatewayInstanceId`, so a client
/// can locate an instance without port bookkeeping and never mistakes a foreign
/// process on the same port for this Gateway.
///
/// External contract, from `server/src/core/gateway-protocol.mjs:32-37`.
pub const CAPABILITY_GATEWAY_INSTANCE_LEASE: &str = "gateway.instance-lease";

/// The Gateway refuses to start while required realtime credentials are
/// missing, reporting what is missing instead of serving an instance whose
/// voice cannot work.
///
/// The refusal carries [`ProtocolError::SetupRequired`], whose code is
/// `VIA_GATEWAY_SETUP_REQUIRED`.
///
/// External contract, from `server/src/core/gateway-protocol.mjs:38-42`.
///
/// [`ProtocolError::SetupRequired`]: crate::ProtocolError::SetupRequired
pub const CAPABILITY_GATEWAY_SETUP_GATE: &str = "gateway.setup-gate";

/// This crate's workspace owns configuration persistence: the settings store
/// keeps settings in the config directory, and a host names no setting and no
/// file of its own.
///
/// External contract, from `server/src/core/gateway-protocol.mjs:43-46`
/// (upstream says "this package"; in VIA the owner is `via-store`).
pub const CAPABILITY_GATEWAY_SETTINGS_STORE: &str = "gateway.settings-store";

/// `POST /api/input/suspend|resume`, `GET /api/input`; the Gateway relays the
/// suspension to clients through
/// [`GatewayServerEvent::InputSuspend`] / [`GatewayServerEvent::InputResume`].
///
/// External contract, from `server/src/core/gateway-protocol.mjs:56-58`.
///
/// [`GatewayServerEvent::InputSuspend`]: crate::GatewayServerEvent::InputSuspend
/// [`GatewayServerEvent::InputResume`]: crate::GatewayServerEvent::InputResume
pub const CAPABILITY_INPUT_SUSPEND_PROTOCOL: &str = "input.suspend-protocol";

/// `input.suspend` also clears playback so host recording stays clean.
///
/// External contract, from `server/src/core/gateway-protocol.mjs:59-60`.
pub const CAPABILITY_INPUT_SUSPEND_CLEARS_PLAYBACK: &str = "input.suspend-clears-playback";

/// A suspension expires on its own when the holder never sends resume.
///
/// External contract, from `server/src/core/gateway-protocol.mjs:61-62`.
pub const CAPABILITY_INPUT_SUSPEND_TTL: &str = "input.suspend-ttl";

/// Clients confirm a suspension with
/// [`GatewayClientEvent::InputSuspendAck`].
///
/// External contract, from `server/src/core/gateway-protocol.mjs:63-64`.
///
/// [`GatewayClientEvent::InputSuspendAck`]: crate::GatewayClientEvent::InputSuspendAck
pub const CAPABILITY_INPUT_SUSPEND_ACK: &str = "input.suspend-ack";

/// The capabilities VIA advertises, echoed as `/api/health.capabilities`.
///
/// **Array order and the exact strings are contract**; clients branch on
/// membership. Every entry matches
/// `^[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*)+$` and appears exactly once.
///
/// This is upstream's `GATEWAY_CAPABILITIES`
/// (`server/src/core/gateway-protocol.mjs:21-74`) restricted, in upstream's own
/// order, to the `gateway.*` and `input.*` entries VIA actually honours. See
/// [`DROPPED_UPSTREAM_CAPABILITIES`] for what was removed and why.
pub const GATEWAY_CAPABILITIES: &[&str] = &[
    CAPABILITY_GATEWAY_INSTANCE_LEASE,
    CAPABILITY_GATEWAY_SETUP_GATE,
    CAPABILITY_GATEWAY_SETTINGS_STORE,
    CAPABILITY_INPUT_SUSPEND_PROTOCOL,
    CAPABILITY_INPUT_SUSPEND_CLEARS_PLAYBACK,
    CAPABILITY_INPUT_SUSPEND_TTL,
    CAPABILITY_INPUT_SUSPEND_ACK,
];

/// The upstream capabilities VIA deliberately does **not** advertise, in
/// upstream's declaration order.
///
/// Kept as a constant rather than only as prose so `via-conformance` can assert
/// the two lists partition upstream's sixteen entries exactly, and so a future
/// contributor who re-adds a GUI surface has a single place to move the string
/// from.
///
/// - the five `desktop.*` entries and `web.skin-assets` — the Electron host,
///   the floating orb and the skin store are dropped (`docs/fidelity.md`);
/// - `web.same-origin-ui` — there is no bundled SPA to host;
/// - `host.electron-entry` and `host.gateway-process` — these name CommonJS/ESM
///   package entry points (`qwen-audio-agent/electron`,
///   `qwen-audio-agent/gateway-process`). A single Rust binary supervises
///   itself; there is nothing for a Node host to `require`.
pub const DROPPED_UPSTREAM_CAPABILITIES: &[&str] = &[
    "web.same-origin-ui",
    "web.skin-assets",
    "host.electron-entry",
    "host.gateway-process",
    "desktop.orb-shell",
    "desktop.orb-window-factory",
    "desktop.orb-placement",
    "desktop.orb-position-store",
    "desktop.skin-store",
];

/// Whether this Gateway advertises `capability`.
///
/// The membership test clients are told to use in place of a version compare.
pub fn advertises_capability(capability: &str) -> bool {
    GATEWAY_CAPABILITIES.contains(&capability)
}
