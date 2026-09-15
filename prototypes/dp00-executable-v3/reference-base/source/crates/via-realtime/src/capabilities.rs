//! What a provider's Realtime implementation actually does.
//!
//! Ported from `server/src/voice/realtime-provider.mjs:61-88`
//! (`DEFAULT_CAPABILITIES`). The doc comment on each flag is upstream's own,
//! because the reason each one exists is the reason the frontend never branches
//! on a provider name:
//!
//! > Behavioural capabilities of a provider's Realtime implementation. Defaults
//! > encode the shared protocol baseline; optional features require opt-in and
//! > providers declare known constraints, without the frontend ever branching on
//! > a provider name.
//!
//! Upstream validates a provider's declaration against a closed list of five
//! names (`providers/provider-registry.mjs:29-35`, `CAPABILITY_FLAGS`) and
//! throws on an unknown flag or a non-boolean value. In Rust the struct *is* the
//! closed list and the type *is* the boolean check, so both of those failures
//! are compile errors rather than runtime ones. [`CAPABILITY_FLAGS`] keeps the
//! names available for the descriptor and for `via-conformance`.

use serde::{Deserialize, Serialize};

/// The five capability names, in upstream's declaration order.
///
/// External contract — `providers/provider-registry.mjs:29-35`. The order is the
/// order [`ProviderCapabilities`] declares its fields, which is the order they
/// serialize in.
pub const CAPABILITY_FLAGS: [&str; 5] = [
    "acknowledgesSessionUpdate",
    "singleResponseSlot",
    "responseMetadataCorrelation",
    "perResponseInstructions",
    "conversationItemIdEcho",
];

/// Behavioural capabilities of a provider's Realtime implementation.
///
/// Every field defaults to the shared protocol baseline
/// ([`ProviderCapabilities::DEFAULT`]); a provider opts into the rest.
///
/// ```
/// use via_realtime::ProviderCapabilities;
///
/// // The baseline every provider gets for free.
/// let baseline = ProviderCapabilities::DEFAULT;
/// assert!(baseline.acknowledges_session_update);
/// assert!(!baseline.single_response_slot);
/// assert!(!baseline.response_metadata_correlation);
/// assert!(!baseline.per_response_instructions);
/// assert!(baseline.conversation_item_id_echo);
///
/// // `Default` is the same value, so `..Default::default()` is upstream's
/// // `{ ...DEFAULT_CAPABILITIES, ...provider.capabilities }` spread.
/// let ga = ProviderCapabilities {
///     acknowledges_session_update: false,
///     single_response_slot: true,
///     response_metadata_correlation: true,
///     per_response_instructions: true,
///     ..Default::default()
/// };
/// assert!(ga.conversation_item_id_echo, "undeclared flags keep the baseline");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCapabilities {
    /// Acknowledges `session.update` with `session.updated`.
    ///
    /// A provider that does not (huggingface/speech-to-speech) applies the
    /// update silently, so the session becomes ready the moment the update is
    /// written rather than when an acknowledgement arrives.
    pub acknowledges_session_update: bool,

    /// Accepts concurrent `response.create` requests (queues instead of
    /// refusing).
    ///
    /// `false` is the baseline: a provider that queues needs no special
    /// handling. `true` declares *one* response slot per session, so a
    /// gateway-created response can race a server-VAD turn and be refused
    /// rather than queued — which is what makes the bounded busy retry safe to
    /// apply.
    pub single_response_slot: bool,

    /// Echoes response metadata so client-created responses can be correlated
    /// without confusing them with automatic server-side responses.
    ///
    /// Without it the session falls back to FIFO correlation: the first
    /// `response.created` after a `response.create` is assumed to be its
    /// answer.
    pub response_metadata_correlation: bool,

    /// Applies instructions supplied on one `response.create` without requiring
    /// a persistent conversation item.
    pub per_response_instructions: bool,

    /// Echoes a client-assigned item id in `conversation.item.created`.
    ///
    /// Some providers acknowledge the item but replace its id, so those
    /// providers must opt out and use the single pending item waiter instead.
    pub conversation_item_id_echo: bool,
}

impl ProviderCapabilities {
    /// The shared protocol baseline.
    ///
    /// External contract — `realtime-provider.mjs:65-80`. Any flag a provider
    /// does not declare takes the value here.
    pub const DEFAULT: Self = Self {
        acknowledges_session_update: true,
        single_response_slot: false,
        response_metadata_correlation: false,
        per_response_instructions: false,
        conversation_item_id_echo: true,
    };

    /// The five flags in [`CAPABILITY_FLAGS`] order.
    ///
    /// Exists so a test can walk the set rather than name each field, which is
    /// what keeps `CAPABILITY_FLAGS` and the struct from drifting apart.
    #[must_use]
    pub const fn as_array(&self) -> [bool; 5] {
        [
            self.acknowledges_session_update,
            self.single_response_slot,
            self.response_metadata_correlation,
            self.per_response_instructions,
            self.conversation_item_id_echo,
        ]
    }
}

impl Default for ProviderCapabilities {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_baseline_is_upstreams_default_capabilities() {
        assert_eq!(
            ProviderCapabilities::DEFAULT.as_array(),
            [true, false, false, false, true]
        );
    }

    #[test]
    fn default_and_the_constant_are_the_same_value() {
        assert_eq!(
            ProviderCapabilities::default(),
            ProviderCapabilities::DEFAULT
        );
    }

    #[test]
    fn the_flag_names_serialize_in_declaration_order() {
        let json = serde_json::to_string(&ProviderCapabilities::DEFAULT).expect("serialize");
        let mut cursor = 0usize;
        for flag in CAPABILITY_FLAGS {
            let found = json[cursor..]
                .find(flag)
                .unwrap_or_else(|| panic!("{flag} is missing from {json}"));
            cursor += found + flag.len();
        }
    }
}
