//! What the voice layer needs to know about the realtime provider.
//!
//! **`via-realtime` owns the provider seam** — `RealtimeProvider`,
//! `RealtimeProtocol`, the registry and the session machine
//! (`docs/architecture.md` §3, §9). This module deliberately declares **no
//! second `RealtimeProvider` trait**. It declares only the narrow read-only
//! view the *voice* layer consumes, so `via-voice` is testable without a
//! socket and so a change to the provider seam does not ripple into every
//! gateway branch.
//!
//! Four facts reach this layer, and each one changes observable behaviour:
//!
//! | | Why the voice layer reads it |
//! | --- | --- |
//! | [`ProviderView::key`] / [`ProviderView::label`] | `voice.ready`, `voice.connection` and `/api/health` carry both |
//! | [`ProviderView::input_sample_rate`] | sent on `voice.ready`; a mismatch is *heard*, as a chipmunk |
//! | [`ProviderView::output_sample_rate`] | the fallback on `audio.delta` when the provider omits the rate |
//! | [`ProviderView::capabilities`] | the two flags the gateway branches on — see below |
//!
//! # The two flags the gateway actually branches on
//!
//! [`ProviderCapabilities::per_response_instructions`] gates the response
//! guards: a correction *is* per-response instructions and nothing else, so a
//! provider without the flag never receives one.
//! [`ProviderCapabilities::single_response_slot`] is why a refusal is replayed
//! rather than surfaced. The other three are read by the session machine in
//! `via-realtime`, and are carried here so `/api/health` can report the whole
//! declaration.
//!
//! The point of the struct is that **the frontend never branches on a provider
//! name** — upstream's own reason, at `realtime-provider.mjs:61-64`.

/// The five behavioural capability flags a provider declares.
///
/// **External contract** — `realtime-provider.mjs:64-88`
/// (`DEFAULT_CAPABILITIES`), reproduced flag for flag with upstream's own
/// reasons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProviderCapabilities {
    /// Acknowledges `session.update` with `session.updated`.
    ///
    /// Default `true`. A provider that opts out is treated as ready the moment
    /// `session.created` arrives, because waiting for an acknowledgement it
    /// never sends would hang the connect.
    pub acknowledges_session_update: bool,
    /// Accepts concurrent `response.create` requests (queues instead of
    /// refusing).
    ///
    /// Default `false` — meaning concurrency is *fine*. Setting it to `true`
    /// declares the constraint: the provider has one response slot, so a
    /// refusal is transient and the exact payload is replayed rather than
    /// surfaced to the user.
    pub single_response_slot: bool,
    /// Echoes response metadata so client-created responses can be correlated
    /// without confusing them with automatic server-side responses.
    ///
    /// Default `false`. With it, correlation is by request id; without it the
    /// session pairs `response.created` with the head of its own pending
    /// queue, which is correct only because a second concurrent creation is
    /// refused at that queue.
    pub response_metadata_correlation: bool,
    /// Applies `instructions` supplied on one `response.create` without
    /// requiring a persistent conversation item.
    ///
    /// Default `false`. The response guards need it — see the module
    /// documentation.
    pub per_response_instructions: bool,
    /// Echoes a client-assigned item id in `conversation.item.created`.
    ///
    /// Default `true`. Some providers acknowledge the item but replace its id,
    /// so those opt out and the session falls back to the single pending item
    /// waiter.
    pub conversation_item_id_echo: bool,
}

impl Default for ProviderCapabilities {
    /// `DEFAULT_CAPABILITIES` — the shared protocol baseline.
    ///
    /// **External contract** — `realtime-provider.mjs:65-80`.
    fn default() -> Self {
        Self {
            acknowledges_session_update: true,
            single_response_slot: false,
            response_metadata_correlation: false,
            per_response_instructions: false,
            conversation_item_id_echo: true,
        }
    }
}

impl ProviderCapabilities {
    /// The five flag names, in declaration order.
    ///
    /// **External contract** — the keys of `DEFAULT_CAPABILITIES`; a provider
    /// declaring a name outside this set is rejected at registration.
    pub const FLAG_NAMES: [&'static str; 5] = [
        "acknowledgesSessionUpdate",
        "singleResponseSlot",
        "responseMetadataCorrelation",
        "perResponseInstructions",
        "conversationItemIdEcho",
    ];

    /// This capability set's flags, paired with their wire names.
    #[must_use]
    pub const fn flags(&self) -> [(&'static str, bool); 5] {
        [
            (
                "acknowledgesSessionUpdate",
                self.acknowledges_session_update,
            ),
            ("singleResponseSlot", self.single_response_slot),
            (
                "responseMetadataCorrelation",
                self.response_metadata_correlation,
            ),
            ("perResponseInstructions", self.per_response_instructions),
            ("conversationItemIdEcho", self.conversation_item_id_echo),
        ]
    }
}

/// How the gateway should react to a provider error message.
///
/// **External contract** — the classification vocabulary of
/// `dashscope.mjs:16-29` and `s2s.mjs:14-23`, consumed at
/// `realtime-gateway.mjs:1353-1437`. It is an enum and not a string because
/// each variant changes what the user sees.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ErrorClassification {
    /// Unrecognized. Surfaced to the user as an `error` frame.
    ///
    /// The default is deliberately the harmless one: a provider that has not
    /// been taught a pattern shows the message, it does not block the session.
    #[default]
    Other,
    /// The provider closed an idle response scope. Housekeeping — the
    /// delegated Work is still healthy and any pending announcement has already
    /// returned to the retry queue, so nothing is shown.
    Inactivity,
    /// A single-slot provider refused a concurrent `response.create`. The
    /// session replays the exact payload; nothing user-facing happened.
    ResponseSlotBusy,
    /// The provider is still consuming input. Same treatment, but it also means
    /// a permission question collided with speech, so the gateway re-schedules
    /// the question instead of failing it.
    InputBusy,
    /// A capacity-bounded provider is still draining a previous session. Its
    /// close event drives the shared reconnect backoff, so this is neither a
    /// response failure nor a user-facing error.
    CapacityBusy,
    /// A cancel raced a response that had already finished. The provider says
    /// "no response in flight", which is meaningless to the user and must not
    /// trigger failure bookkeeping.
    NoActiveResponse,
    /// The session cannot be used at all — bad credentials, unsupported model.
    /// The gateway blocks the provider, drops buffered audio and stops
    /// reconnecting.
    Fatal,
}

impl ErrorClassification {
    /// Whether the gateway shows this to the user as an `error` frame.
    ///
    /// **External contract** — `realtime-gateway.mjs:1432-1436`: everything is
    /// shown except a recoverable inactivity close and a fatal block, which has
    /// already been reported as `voice.connection: unavailable`.
    #[must_use]
    pub const fn is_user_facing(self) -> bool {
        !matches!(self, Self::Inactivity | Self::Fatal)
    }

    /// Whether the gateway swallows this event entirely.
    ///
    /// **External contract** — `realtime-gateway.mjs:1362,1377`.
    #[must_use]
    pub const fn is_silent(self) -> bool {
        matches!(self, Self::CapacityBusy | Self::NoActiveResponse)
    }
}

/// The read-only view of the provider that the voice layer consumes.
///
/// Implemented in `via-app` over `via_realtime`'s `RealtimeProvider`; a test
/// double implements it directly.
pub trait ProviderView: Send + Sync + std::fmt::Debug {
    /// The provider key — `dashscope`, `speech-to-speech`, `mock`, …
    ///
    /// `docs/rebrand.md` keeps `dashscope` and its `qwen` alias: they name the
    /// vendor's runtime, not VIA.
    fn key(&self) -> &str;
    /// A human label, shown on `voice.ready` and in `/api/health`.
    fn label(&self) -> &str;
    /// The rate the client must capture at, in hertz.
    fn input_sample_rate(&self) -> u32;
    /// The rate response audio arrives at, in hertz.
    fn output_sample_rate(&self) -> u32;
    /// The five behavioural flags.
    fn capabilities(&self) -> ProviderCapabilities;
    /// Classify a provider error message.
    fn classify_error(&self, _message: &str) -> ErrorClassification {
        ErrorClassification::Other
    }
    /// How long to wait for a response to start, in milliseconds.
    ///
    /// `None` falls back to
    /// [`RESPONSE_START_WATCHDOG_MS`](crate::session::RESPONSE_START_WATCHDOG_MS).
    fn response_start_timeout_ms(&self) -> Option<u64> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn the_default_capabilities_are_upstreams_five() {
        let capabilities = ProviderCapabilities::default();
        assert!(capabilities.acknowledges_session_update);
        assert!(!capabilities.single_response_slot);
        assert!(!capabilities.response_metadata_correlation);
        assert!(!capabilities.per_response_instructions);
        assert!(capabilities.conversation_item_id_echo);
    }

    #[test]
    fn the_flag_names_match_the_declared_flags_in_order() {
        let names: Vec<&str> = ProviderCapabilities::default()
            .flags()
            .iter()
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(names, ProviderCapabilities::FLAG_NAMES.to_vec());
    }

    #[test]
    fn the_default_flags_carry_the_default_values() {
        assert_eq!(
            ProviderCapabilities::default().flags(),
            [
                ("acknowledgesSessionUpdate", true),
                ("singleResponseSlot", false),
                ("responseMetadataCorrelation", false),
                ("perResponseInstructions", false),
                ("conversationItemIdEcho", true),
            ],
        );
    }

    #[test]
    fn an_unclassified_message_is_other_and_is_shown_to_the_user() {
        assert_eq!(ErrorClassification::default(), ErrorClassification::Other);
        assert!(ErrorClassification::Other.is_user_facing());
        assert!(!ErrorClassification::Other.is_silent());
    }

    #[test]
    fn the_two_housekeeping_classifications_never_reach_the_user() {
        assert!(!ErrorClassification::Inactivity.is_user_facing());
        assert!(!ErrorClassification::Fatal.is_user_facing());
        assert!(ErrorClassification::CapacityBusy.is_silent());
        assert!(ErrorClassification::NoActiveResponse.is_silent());
    }
}
