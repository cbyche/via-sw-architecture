//! Which client owns the microphone and the speaker.
//!
//! Ported from `server/src/voice/active-voice-clients.mjs`.
//!
//! Exactly one client per owner holds the voice session. The interesting cases
//! are the two failure modes:
//!
//! - **A dead previous owner must not hold the slot forever.** An unclean
//!   disconnect — a network drop that never fires a socket close — leaves a
//!   client that will never release. Only a *still-live* owner can refuse a
//!   non-takeover claim; a dead one is treated as absent.
//! - **A takeover is explicit.** A second client claiming without
//!   `takeover: true` is refused rather than silently stealing the microphone
//!   from a session the user is talking into.

use std::collections::HashMap;
use std::sync::Arc;

/// One connected client that can hold the voice session.
pub trait VoiceClient: Send + Sync {
    /// A stable identity for this client, used for slot comparison.
    fn client_id(&self) -> &str;

    /// Whether the socket is still open.
    ///
    /// **External contract** — `active-voice-clients.mjs:17`: upstream's
    /// `isAlive?.() !== false` treats an *absent* implementation as alive, so
    /// the default here is `true` for the same reason — a caller that does not
    /// know cannot be allowed to evict.
    fn is_alive(&self) -> bool {
        true
    }

    /// Called on the client that just lost the slot.
    fn deactivate(&self, _replacement: Option<&dyn VoiceClient>) {}
}

/// What a claim answered.
pub struct ActivationResult {
    /// Whether the claim succeeded.
    pub granted: bool,
    /// The client that held the slot before, when there was one **and** the
    /// slot actually changed hands. A re-claim by the current holder reports
    /// `None`, because nothing moved.
    pub previous: Option<Arc<dyn VoiceClient>>,
}

impl std::fmt::Debug for ActivationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActivationResult")
            .field("granted", &self.granted)
            .field(
                "previous",
                &self.previous.as_ref().map(|client| client.client_id()),
            )
            .finish()
    }
}

/// The per-owner voice slot table.
#[derive(Default)]
pub struct ActiveVoiceClients {
    clients: HashMap<String, Arc<dyn VoiceClient>>,
}

impl std::fmt::Debug for ActiveVoiceClients {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActiveVoiceClients")
            .field("owners", &self.clients.len())
            .finish()
    }
}

impl ActiveVoiceClients {
    /// An empty table.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Claim the slot for `client`.
    ///
    /// **External contract** — `active-voice-clients.mjs:6-24`, in upstream's
    /// order: a re-claim by the current holder is granted with no previous; a
    /// live previous holder refuses a non-takeover claim; otherwise the
    /// previous holder is deactivated and the slot changes hands.
    pub fn activate(
        &mut self,
        owner_id: &str,
        client: Arc<dyn VoiceClient>,
        takeover: bool,
    ) -> ActivationResult {
        let key = owner_id.to_owned();
        let previous = self.clients.get(&key).cloned();
        if let Some(previous) = &previous
            && previous.client_id() == client.client_id()
        {
            return ActivationResult {
                granted: true,
                previous: None,
            };
        }
        let previous_alive = previous
            .as_ref()
            .is_some_and(|previous| previous.is_alive());
        if previous.is_some() && previous_alive && !takeover {
            return ActivationResult {
                granted: false,
                previous,
            };
        }
        if let Some(previous) = &previous {
            previous.deactivate(Some(client.as_ref()));
        }
        self.clients.insert(key, client);
        ActivationResult {
            granted: true,
            previous,
        }
    }

    /// Release the slot, if `client_id` still holds it.
    ///
    /// Answers whether anything changed, so the caller knows whether to
    /// broadcast a new ownership frame.
    pub fn release(&mut self, owner_id: &str, client_id: &str) -> bool {
        let holds = self
            .clients
            .get(owner_id)
            .is_some_and(|held| held.client_id() == client_id);
        if !holds {
            return false;
        }
        self.clients.remove(owner_id);
        true
    }

    /// Whether `client_id` holds `owner_id`'s slot.
    #[must_use]
    pub fn is_active(&self, owner_id: &str, client_id: &str) -> bool {
        self.clients
            .get(owner_id)
            .is_some_and(|held| held.client_id() == client_id)
    }

    /// The client holding `owner_id`'s slot.
    #[must_use]
    pub fn active(&self, owner_id: &str) -> Option<Arc<dyn VoiceClient>> {
        self.clients.get(owner_id).cloned()
    }

    /// How many owners have an active client.
    #[must_use]
    pub fn len(&self) -> usize {
        self.clients.len()
    }

    /// Whether no owner has an active client.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }
}

/// What a client declared it can do with audio.
///
/// **External contract** — the `connect` frame's `voiceEnabled` /
/// `inputEnabled` / `outputEnabled` / `textOnly` fields.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeclaredCapabilities {
    /// The legacy single flag. Used only when the explicit pair is absent.
    pub voice_enabled: bool,
    /// Explicit capture capability, when the client stated one.
    pub input_enabled: Option<bool>,
    /// Explicit playback capability, when the client stated one.
    pub output_enabled: Option<bool>,
    /// A client that will never play audio — `via chat`, the WebUI transcript
    /// pane.
    pub text_only: bool,
}

/// What the Gateway concluded.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VoiceCapabilities {
    /// May capture.
    pub input_enabled: bool,
    /// May play.
    pub output_enabled: bool,
    /// Competes for the per-owner voice slot.
    pub participates_in_voice_arbitration: bool,
}

/// Resolve a client's declaration.
///
/// **External contract** — `active-voice-clients.mjs:46-64`. Three rules, and
/// the asymmetry between them is the contract:
///
/// - output defaults to the legacy `voiceEnabled` when the client did not say;
/// - input defaults to *output and not text-only*, so a client that can play
///   but never said it can capture is assumed to be able to;
/// - `text_only` vetoes input and arbitration but **not** output — a text
///   client still hears announcements read to it, it just never takes the
///   microphone.
#[must_use]
pub fn client_voice_capabilities(declared: DeclaredCapabilities) -> VoiceCapabilities {
    let can_output = declared.output_enabled.unwrap_or(declared.voice_enabled);
    let can_input = match declared.input_enabled {
        Some(enabled) => enabled && !declared.text_only,
        None => can_output && !declared.text_only,
    };
    VoiceCapabilities {
        input_enabled: can_input,
        output_enabled: can_output,
        participates_in_voice_arbitration: can_output && !declared.text_only,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    #[derive(Debug)]
    struct TestClient {
        id: String,
        alive: AtomicBool,
        deactivated: AtomicUsize,
    }

    impl TestClient {
        fn new(id: &str) -> Arc<Self> {
            Arc::new(Self {
                id: id.to_owned(),
                alive: AtomicBool::new(true),
                deactivated: AtomicUsize::new(0),
            })
        }
    }

    impl VoiceClient for TestClient {
        fn client_id(&self) -> &str {
            &self.id
        }
        fn is_alive(&self) -> bool {
            self.alive.load(Ordering::SeqCst)
        }
        fn deactivate(&self, _replacement: Option<&dyn VoiceClient>) {
            self.deactivated.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn the_first_claim_is_granted_with_no_previous() {
        let mut clients = ActiveVoiceClients::new();
        let result = clients.activate("owner", TestClient::new("a"), false);
        assert!(result.granted);
        assert!(result.previous.is_none());
        assert!(clients.is_active("owner", "a"));
        assert_eq!(clients.len(), 1);
    }

    #[test]
    fn re_claiming_reports_no_previous_because_nothing_moved() {
        let mut clients = ActiveVoiceClients::new();
        let client = TestClient::new("a");
        clients.activate("owner", Arc::clone(&client) as Arc<dyn VoiceClient>, false);
        let result = clients.activate("owner", Arc::clone(&client) as Arc<dyn VoiceClient>, false);
        assert!(result.granted);
        assert!(result.previous.is_none());
        assert_eq!(client.deactivated.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn a_live_holder_refuses_a_claim_without_takeover() {
        let mut clients = ActiveVoiceClients::new();
        let first = TestClient::new("a");
        clients.activate("owner", Arc::clone(&first) as Arc<dyn VoiceClient>, false);
        let result = clients.activate("owner", TestClient::new("b"), false);
        assert!(!result.granted);
        assert_eq!(
            result.previous.as_ref().map(|client| client.client_id()),
            Some("a"),
        );
        assert!(clients.is_active("owner", "a"), "the holder kept the slot");
        assert_eq!(first.deactivated.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn takeover_evicts_the_live_holder_and_tells_it() {
        let mut clients = ActiveVoiceClients::new();
        let first = TestClient::new("a");
        clients.activate("owner", Arc::clone(&first) as Arc<dyn VoiceClient>, false);
        let result = clients.activate("owner", TestClient::new("b"), true);
        assert!(result.granted);
        assert_eq!(
            result.previous.as_ref().map(|client| client.client_id()),
            Some("a"),
        );
        assert_eq!(first.deactivated.load(Ordering::SeqCst), 1);
        assert!(clients.is_active("owner", "b"));
    }

    #[test]
    fn a_dead_holder_never_blocks_a_new_claim() {
        let mut clients = ActiveVoiceClients::new();
        let first = TestClient::new("a");
        first.alive.store(false, Ordering::SeqCst);
        clients.activate("owner", Arc::clone(&first) as Arc<dyn VoiceClient>, false);
        // No takeover flag: the network dropped, nobody pressed anything.
        let result = clients.activate("owner", TestClient::new("b"), false);
        assert!(result.granted, "a dead holder is treated as absent");
        assert!(clients.is_active("owner", "b"));
    }

    #[test]
    fn a_client_that_does_not_report_liveness_is_treated_as_alive() {
        struct Silent(String);
        impl VoiceClient for Silent {
            fn client_id(&self) -> &str {
                &self.0
            }
        }
        let mut clients = ActiveVoiceClients::new();
        clients.activate("owner", Arc::new(Silent("a".to_owned())), false);
        let result = clients.activate("owner", Arc::new(Silent("b".to_owned())), false);
        assert!(!result.granted, "unknown liveness must not permit eviction");
    }

    #[test]
    fn release_only_succeeds_for_the_current_holder() {
        let mut clients = ActiveVoiceClients::new();
        clients.activate("owner", TestClient::new("a"), false);
        assert!(
            !clients.release("owner", "b"),
            "a stale client releases nothing"
        );
        assert!(clients.is_active("owner", "a"));
        assert!(clients.release("owner", "a"));
        assert!(clients.is_empty());
        assert!(
            !clients.release("owner", "a"),
            "releasing twice changes nothing"
        );
    }

    #[test]
    fn owners_are_independent() {
        let mut clients = ActiveVoiceClients::new();
        clients.activate("alice", TestClient::new("a"), false);
        clients.activate("bob", TestClient::new("b"), false);
        assert_eq!(clients.len(), 2);
        assert!(clients.is_active("alice", "a"));
        assert!(clients.is_active("bob", "b"));
        assert!(!clients.is_active("alice", "b"));
    }

    #[test]
    fn the_legacy_flag_is_used_only_when_the_explicit_pair_is_absent() {
        let legacy = client_voice_capabilities(DeclaredCapabilities {
            voice_enabled: true,
            ..DeclaredCapabilities::default()
        });
        assert_eq!(
            legacy,
            VoiceCapabilities {
                input_enabled: true,
                output_enabled: true,
                participates_in_voice_arbitration: true,
            },
        );
        let explicit = client_voice_capabilities(DeclaredCapabilities {
            voice_enabled: true,
            output_enabled: Some(false),
            ..DeclaredCapabilities::default()
        });
        assert!(!explicit.output_enabled);
        assert!(!explicit.input_enabled, "input defaults to output");
    }

    #[test]
    fn text_only_vetoes_input_and_arbitration_but_not_output() {
        let capabilities = client_voice_capabilities(DeclaredCapabilities {
            voice_enabled: true,
            input_enabled: Some(true),
            output_enabled: Some(true),
            text_only: true,
        });
        assert!(!capabilities.input_enabled);
        assert!(
            capabilities.output_enabled,
            "a text client still hears results"
        );
        assert!(!capabilities.participates_in_voice_arbitration);
    }

    #[test]
    fn a_silent_client_participates_in_nothing() {
        let capabilities = client_voice_capabilities(DeclaredCapabilities::default());
        assert_eq!(capabilities, VoiceCapabilities::default());
    }

    #[test]
    fn an_output_only_client_still_takes_the_voice_slot() {
        // It owns the speaker, so a second client must not talk over it.
        let capabilities = client_voice_capabilities(DeclaredCapabilities {
            input_enabled: Some(false),
            output_enabled: Some(true),
            ..DeclaredCapabilities::default()
        });
        assert!(!capabilities.input_enabled);
        assert!(capabilities.participates_in_voice_arbitration);
    }
}
