//! The coded errors that belong to the contract surface.
//!
//! Upstream returns ad-hoc `{ ok, error, code }` objects and attaches `code` to
//! a plain `Error`. **The codes are contract; the shape around them is not**
//! (`docs/fidelity.md`, "Type system"), so VIA carries them on a `thiserror`
//! enum with a [`code`](ProtocolError::code) accessor.
//!
//! Three of the five codes are upstream contracts with the `QWAUDIO_` prefix
//! renamed to `VIA_` per `docs/rebrand.md`; the other two are this crate's own
//! and say so in their names.
//!
//! ## On messages
//!
//! Upstream's messages are Chinese literals at the throw site. VIA hoists every
//! user-facing string into the `via-i18n` catalog keyed by
//! [`code`](ProtocolError::code) — `zh` holds upstream's exact text, `en` and
//! `ko` are authored peers (`docs/architecture.md` §16). `via-protocol` is a
//! leaf crate with no i18n dependency, so [`Display`](core::fmt::Display) here
//! is a developer-facing English rendering, not the string a user sees. The
//! *code* is the contract; the message beside it is localized.

use crate::WorkStatus;

/// One entry of the setup gate's `missing` list.
///
/// External contract, from `shared/gateway-setup.mjs:14-33`. Serialised field
/// order is `field`, `key`, `message`, matching upstream's object literal —
/// insertion order is observable through `serde_json`'s `preserve_order`, so
/// the declaration order below is deliberate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct MissingSetting {
    /// The settings field, e.g. `dashscopeApiKey`.
    pub field: String,
    /// The environment variable, e.g. `DASHSCOPE_API_KEY`.
    pub key: String,
    /// A message fit for a UI, explaining what to set.
    pub message: String,
}

/// The setup gate's refusal code.
///
/// External contract. Upstream is `QWAUDIO_GATEWAY_SETUP_REQUIRED`
/// (`shared/gateway-setup.mjs:49`), renamed per `docs/rebrand.md`.
pub const CODE_GATEWAY_SETUP_REQUIRED: &str = "VIA_GATEWAY_SETUP_REQUIRED";

/// The instance-lease conflict code.
///
/// External contract. Upstream is `QWAUDIO_GATEWAY_ALREADY_RUNNING`
/// (`shared/gateway-instance-lock.mjs:157`), renamed per `docs/rebrand.md`.
pub const CODE_GATEWAY_ALREADY_RUNNING: &str = "VIA_GATEWAY_ALREADY_RUNNING";

/// The input control plane's only error code.
///
/// External contract. Upstream is `QWAUDIO_INPUT_OWNER_REQUIRED`
/// (`server/src/voice/input-arbitration.mjs:73`), renamed per
/// `docs/rebrand.md`.
pub const CODE_INPUT_OWNER_REQUIRED: &str = "VIA_INPUT_OWNER_REQUIRED";

/// VIA-owned: a string was not a member of the vocabulary it was parsed
/// against. Upstream has no code for this — it rejects unknown event types by
/// set membership without naming the failure.
pub const CODE_UNKNOWN_WIRE_VALUE: &str = "VIA_PROTOCOL_UNKNOWN_WIRE_VALUE";

/// VIA-owned: a Work status change was refused. Upstream has no code for this
/// because it has no transition check.
pub const CODE_ILLEGAL_TRANSITION: &str = "VIA_PROTOCOL_ILLEGAL_TRANSITION";

/// The three error codes that are external contracts inherited from upstream,
/// in the order they are documented in `docs/rebrand.md`.
///
/// Kept as a constant so `via-conformance` can assert the rename was applied to
/// all three atomically: a half-applied rename is exactly the failure mode
/// `docs/architecture.md` §13 warns about.
pub const CONTRACT_ERROR_CODES: &[&str] = &[
    CODE_GATEWAY_SETUP_REQUIRED,
    CODE_GATEWAY_ALREADY_RUNNING,
    CODE_INPUT_OWNER_REQUIRED,
];

/// A failure on VIA's frozen contract surface.
///
/// Every variant has a stable [`code`](Self::code); see the module
/// documentation for why the message beside it is not part of the contract.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProtocolError {
    /// The Gateway refuses to start because required configuration is missing.
    ///
    /// Backs the `gateway.setup-gate` capability. The process exits non-zero
    /// **before the instance lease is touched**, so a refused start never
    /// disturbs a running instance.
    ///
    /// Code: `VIA_GATEWAY_SETUP_REQUIRED`
    /// ([`CODE_GATEWAY_SETUP_REQUIRED`]).
    #[error(
        "gateway start refused; missing required configuration: {}",
        DisplayMissing(missing)
    )]
    SetupRequired {
        /// Every missing setting, in the order the gate discovered them.
        missing: Vec<MissingSetting>,
    },

    /// Another Gateway already holds the instance lease.
    ///
    /// Callers branch on this to attach to the running instance instead of
    /// failing. `origin` is the incumbent's HTTP origin when its lease records
    /// one; the full lease document belongs to `via-lock`.
    ///
    /// Code: `VIA_GATEWAY_ALREADY_RUNNING`
    /// ([`CODE_GATEWAY_ALREADY_RUNNING`]).
    #[error("a Gateway is already running{}", DisplayOrigin(origin))]
    AlreadyRunning {
        /// The incumbent's origin, e.g. `http://127.0.0.1:8787`, when known.
        origin: Option<String>,
    },

    /// `input.suspend` was called without an owner.
    ///
    /// Surfaced as HTTP 400. Every microphone hold must be attributable,
    /// because an unattributable hold can never be expired against its holder.
    ///
    /// Code: `VIA_INPUT_OWNER_REQUIRED` ([`CODE_INPUT_OWNER_REQUIRED`]).
    #[error("input.suspend requires an owner")]
    InputOwnerRequired,

    /// A string is not a member of the vocabulary it was parsed against.
    ///
    /// The error [`FromStr`](core::str::FromStr) returns for every wire
    /// vocabulary in this crate.
    ///
    /// Code: `VIA_PROTOCOL_UNKNOWN_WIRE_VALUE` ([`CODE_UNKNOWN_WIRE_VALUE`]).
    #[error("`{value}` is not a member of the {vocabulary} vocabulary")]
    UnknownWireValue {
        /// The vocabulary that rejected it, e.g. `GatewayClientEvent`.
        vocabulary: &'static str,
        /// The string that failed to parse.
        value: String,
    },

    /// A Work status change is not an edge of the lifecycle graph.
    ///
    /// See [`WorkStatus::can_transition_to`].
    ///
    /// Code: `VIA_PROTOCOL_ILLEGAL_TRANSITION` ([`CODE_ILLEGAL_TRANSITION`]).
    #[error("illegal Work transition: {from} -> {to}")]
    IllegalTransition {
        /// The status the Work is in.
        from: WorkStatus,
        /// The status that was refused.
        to: WorkStatus,
    },
}

impl ProtocolError {
    /// The stable error code for this failure.
    ///
    /// This is the value clients and the CLI branch on, and the `code` field of
    /// an HTTP error body. It is never localized.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::SetupRequired { .. } => CODE_GATEWAY_SETUP_REQUIRED,
            Self::AlreadyRunning { .. } => CODE_GATEWAY_ALREADY_RUNNING,
            Self::InputOwnerRequired => CODE_INPUT_OWNER_REQUIRED,
            Self::UnknownWireValue { .. } => CODE_UNKNOWN_WIRE_VALUE,
            Self::IllegalTransition { .. } => CODE_ILLEGAL_TRANSITION,
        }
    }

    /// Whether this error's code is one inherited from upstream
    /// ([`CONTRACT_ERROR_CODES`]) rather than owned by VIA.
    pub fn is_upstream_contract(&self) -> bool {
        CONTRACT_ERROR_CODES.contains(&self.code())
    }
}

/// Renders the setup gate's `missing` list as `KEY (message); KEY (message)`.
///
/// Upstream joins with the fullwidth forms `（`, `）` and `；`
/// (`shared/gateway-setup.mjs:44-47`); that exact text is `via-i18n`'s `zh`
/// entry, not this developer-facing rendering.
struct DisplayMissing<'a>(&'a [MissingSetting]);

impl core::fmt::Display for DisplayMissing<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (index, item) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str("; ")?;
            }
            write!(f, "{} ({})", item.key, item.message)?;
        }
        Ok(())
    }
}

/// Renders an optional origin as `: <origin>`, or nothing.
struct DisplayOrigin<'a>(&'a Option<String>);

impl core::fmt::Display for DisplayOrigin<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            Some(origin) => write!(f, ": {origin}"),
            None => Ok(()),
        }
    }
}
