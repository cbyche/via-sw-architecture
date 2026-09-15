//! Which clients are connected, and which one holds the microphone.
//!
//! Two tables, both upstream's, both consulted by `/api/health`:
//!
//! - **every** open connection, keyed by a per-socket id — upstream's
//!   `voiceConnections: Map<ownerId, Set<client>>`, which is what a broadcast
//!   iterates;
//! - the **one** connection per owner that holds the voice slot —
//!   [`via_voice::ActiveVoiceClients`], which is what a takeover contends for.
//!
//! A connection is in the first from its upgrade to its close, and in the
//! second only while it holds. `voiceClients.connected` counts the first.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use via_voice::{
    ActiveVoiceClients, ModePlan, RealtimeState, RealtimeStatus, VoiceClient, VoiceClientStatus,
    VoiceClientsHealth, aggregate,
};

use crate::realtime::frames::{ClientDescriptor, ServerFrame};

/// One connection's outbound half.
///
/// A bounded channel rather than a shared socket handle: the connection task
/// owns the socket, and a broadcast from an HTTP handler or another connection
/// enqueues rather than writing. A full channel means a client that is not
/// reading, and dropping the frame is better than stalling the broadcaster —
/// which is upstream's behaviour too, since `ws.send` on a saturated socket
/// buffers rather than blocking the caller.
pub type Outbound = mpsc::Sender<ServerFrame>;

/// How many frames may queue for one client.
pub const OUTBOUND_QUEUE_DEPTH: usize = 256;

/// One live connection, as the registry sees it.
#[derive(Clone)]
pub struct Connection {
    /// The per-socket id — upstream's object identity, made explicit.
    pub id: String,
    /// Which owner it belongs to.
    pub owner_id: String,
    /// Which client it is.
    pub descriptor: ClientDescriptor,
    /// What the session negotiated at `connect`.
    pub mode: ModePlan,
    /// Where its realtime connection is.
    pub realtime: RealtimeStatus,
    /// Its outbound queue.
    pub outbound: Outbound,
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("id", &self.id)
            .field("owner_id", &self.owner_id)
            .field("descriptor", &self.descriptor)
            .field("mode", &self.mode)
            .finish_non_exhaustive()
    }
}

/// A [`VoiceClient`] backed by one connection's outbound queue.
///
/// [`is_alive`](VoiceClient::is_alive) is *"the outbound channel is still
/// open"*, which is exactly the fact upstream's
/// `active-voice-clients.mjs:17` needs: an unclean disconnect that never fired
/// a close must not hold the slot for good.
struct SlotHolder {
    id: String,
    descriptor: ClientDescriptor,
    outbound: Outbound,
}

impl VoiceClient for SlotHolder {
    fn client_id(&self) -> &str {
        &self.id
    }

    fn is_alive(&self) -> bool {
        !self.outbound.is_closed()
    }

    /// Deliberately empty.
    ///
    /// Upstream's `deactivate(replacement)` sends `playback.clear` and then
    /// `voice.deactivated` carrying **the replacement's descriptor**
    /// (`realtime-gateway.mjs:12884`). A `&dyn VoiceClient` cannot supply a
    /// descriptor, so [`VoiceClientRegistry::activate`] sends both frames
    /// itself, in that order, once it can see who won — see
    /// [`deactivation_frames`]. Doing it here with `holder: null` would drop a
    /// field a client uses to say *"the desktop app took the microphone"*.
    fn deactivate(&self, _replacement: Option<&dyn VoiceClient>) {}
}

impl SlotHolder {
    fn descriptor(&self) -> &ClientDescriptor {
        &self.descriptor
    }
}

/// The two frames a client that just lost the voice slot is sent.
///
/// **External contract** — `realtime-gateway.mjs:12884`, in this order:
/// `playback.clear` with no reason, then `voice.deactivated` naming the new
/// holder.
#[must_use]
pub fn deactivation_frames(replacement: Option<&ClientDescriptor>) -> [ServerFrame; 2] {
    [
        crate::realtime::frames::playback_clear(None),
        ServerFrame::new(via_protocol::GatewayServerEvent::VoiceDeactivated.as_str()).with(
            "holder",
            replacement.map_or(serde_json::Value::Null, |descriptor| {
                serde_json::to_value(descriptor).unwrap_or(serde_json::Value::Null)
            }),
        ),
    ]
}

/// Every connected client.
#[derive(Default)]
pub struct VoiceClientRegistry {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    connections: BTreeMap<String, Connection>,
    slots: ActiveVoiceClients,
    holders: BTreeMap<String, Arc<SlotHolder>>,
}

impl std::fmt::Debug for VoiceClientRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceClientRegistry")
            .field("connections", &self.len())
            .finish()
    }
}

/// What a slot claim answered.
///
/// Upstream broadcasts `voice.ownership` after **every** claim, granted or
/// not (`realtime-gateway.mjs`'s `activateVoiceClient`), so there is no
/// "changed" flag to consult: a refusal still tells the asking client that
/// somebody else is holding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotClaim {
    /// Whether this connection now holds the slot.
    pub granted: bool,
}

impl VoiceClientRegistry {
    /// Register a new connection.
    pub fn insert(&self, connection: Connection) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.connections.insert(connection.id.clone(), connection);
        }
    }

    /// Forget a connection, releasing its slot if it held one.
    ///
    /// Answers whether the slot was released, so the caller knows whether to
    /// broadcast a fresh ownership frame.
    pub fn remove(&self, id: &str) -> bool {
        let Ok(mut inner) = self.inner.lock() else {
            return false;
        };
        let Some(connection) = inner.connections.remove(id) else {
            return false;
        };
        inner.holders.remove(id);
        inner.slots.release(&connection.owner_id, id)
    }

    /// Update the mode and realtime status a connection reports.
    pub fn update(&self, id: &str, mode: ModePlan, realtime: RealtimeStatus) {
        if let Ok(mut inner) = self.inner.lock()
            && let Some(connection) = inner.connections.get_mut(id)
        {
            connection.mode = mode;
            connection.realtime = realtime;
        }
    }

    /// Record the descriptor a `connect` frame supplied.
    pub fn describe(&self, id: &str, descriptor: ClientDescriptor) {
        if let Ok(mut inner) = self.inner.lock() {
            if let Some(connection) = inner.connections.get_mut(id) {
                connection.descriptor = descriptor.clone();
            }
            if let Some(holder) = inner.holders.get(id) {
                let refreshed = Arc::new(SlotHolder {
                    id: holder.id.clone(),
                    descriptor,
                    outbound: holder.outbound.clone(),
                });
                inner.holders.insert(id.to_owned(), refreshed);
            }
        }
    }

    /// Claim the voice slot for `id`.
    ///
    /// **External contract** — [`ActiveVoiceClients::activate`]: a re-claim by
    /// the holder is granted and changes nothing, a **live** holder refuses a
    /// claim without `takeover`, and a dead one is treated as absent.
    pub fn activate(&self, id: &str, takeover: bool) -> SlotClaim {
        let Ok(mut inner) = self.inner.lock() else {
            return SlotClaim { granted: false };
        };
        let Some(connection) = inner.connections.get(id).cloned() else {
            return SlotClaim { granted: false };
        };
        let holder = Arc::new(SlotHolder {
            id: connection.id.clone(),
            descriptor: connection.descriptor.clone(),
            outbound: connection.outbound.clone(),
        });
        inner.holders.insert(id.to_owned(), holder.clone());
        let result = inner.slots.activate(&connection.owner_id, holder, takeover);
        if !result.granted {
            inner.holders.remove(id);
            return SlotClaim { granted: false };
        }
        // The slot moved. Tell whoever just lost it, naming who took it — the
        // half `SlotHolder::deactivate` cannot do.
        if let Some(previous) = &result.previous {
            let previous_id = previous.client_id().to_owned();
            if previous_id != connection.id
                && let Some(losing) = inner.holders.remove(&previous_id)
            {
                for frame in deactivation_frames(Some(&connection.descriptor)) {
                    let _ = losing.outbound.try_send(frame);
                }
            }
        }
        SlotClaim { granted: true }
    }

    /// Release `id`'s slot. Answers whether anything changed.
    pub fn release(&self, id: &str) -> bool {
        let Ok(mut inner) = self.inner.lock() else {
            return false;
        };
        let Some(owner_id) = inner
            .connections
            .get(id)
            .map(|connection| connection.owner_id.clone())
        else {
            return false;
        };
        inner.holders.remove(id);
        inner.slots.release(&owner_id, id)
    }

    /// Whether `id` holds `owner_id`'s slot.
    #[must_use]
    pub fn is_active(&self, owner_id: &str, id: &str) -> bool {
        self.inner
            .lock()
            .map(|inner| inner.slots.is_active(owner_id, id))
            .unwrap_or(false)
    }

    /// The `voice.ownership` frames one owner's connections should receive.
    ///
    /// **External contract** — `broadcastVoiceOwnership`: the holder is told
    /// `active`, everyone else is told `busy` while somebody holds and
    /// `available` when nobody does, and every frame carries the holder's
    /// descriptor (or `null`).
    #[must_use]
    pub fn ownership_broadcast(&self, owner_id: &str) -> Vec<(Outbound, ServerFrame)> {
        let Ok(inner) = self.inner.lock() else {
            return Vec::new();
        };
        let holder_id = inner
            .slots
            .active(owner_id)
            .map(|client| client.client_id().to_owned());
        let holder = holder_id
            .as_ref()
            .and_then(|id| inner.holders.get(id))
            .map(|holder| holder.descriptor().clone());
        inner
            .connections
            .values()
            .filter(|connection| connection.owner_id == owner_id)
            .map(|connection| {
                let state = if Some(&connection.id) == holder_id.as_ref() {
                    "active"
                } else if holder_id.is_some() {
                    "busy"
                } else {
                    "available"
                };
                (
                    connection.outbound.clone(),
                    crate::realtime::frames::voice_ownership(state, holder.as_ref()),
                )
            })
            .collect()
    }

    /// Every connection's outbound queue — the fan-out an input suspension
    /// uses.
    #[must_use]
    pub fn all_outbound(&self) -> Vec<Outbound> {
        self.inner
            .lock()
            .map(|inner| {
                inner
                    .connections
                    .values()
                    .map(|connection| connection.outbound.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Every connection belonging to `owner_id`.
    #[must_use]
    pub fn outbound_for(&self, owner_id: &str) -> Vec<Outbound> {
        self.inner
            .lock()
            .map(|inner| {
                inner
                    .connections
                    .values()
                    .filter(|connection| connection.owner_id == owner_id)
                    .map(|connection| connection.outbound.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// How many connections are open.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner
            .lock()
            .map(|inner| inner.connections.len())
            .unwrap_or(0)
    }

    /// Whether nothing is connected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The `voiceClients` block of `/api/health`.
    #[must_use]
    pub fn health(&self) -> VoiceClientsHealth {
        let Ok(inner) = self.inner.lock() else {
            return VoiceClientsHealth::default();
        };
        let statuses: Vec<VoiceClientStatus> = inner
            .connections
            .values()
            .map(|connection| VoiceClientStatus {
                client_type: connection.descriptor.client_type,
                realtime: connection.realtime.clone(),
                mode: connection.mode,
            })
            .collect();
        aggregate(&statuses, inner.slots.len())
    }
}

/// The status a connection reports before it has a realtime session.
#[must_use]
pub fn disconnected(provider: &str) -> RealtimeStatus {
    RealtimeStatus {
        provider: provider.to_owned(),
        state: RealtimeState::Disconnected,
        error: None,
    }
}

/// The per-connection id — one per socket, not one per client.
///
/// A client may reconnect with the same `clientInstanceId`; the slot table must
/// still see two different holders, or the second connection would silently
/// inherit the first's slot.
#[must_use]
pub fn new_connection_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use via_protocol::{ClientType, SessionMode};

    fn connection(
        id: &str,
        owner: &str,
        client_type: ClientType,
    ) -> (Connection, mpsc::Receiver<ServerFrame>) {
        let (outbound, inbox) = mpsc::channel(OUTBOUND_QUEUE_DEPTH);
        (
            Connection {
                id: id.to_owned(),
                owner_id: owner.to_owned(),
                descriptor: ClientDescriptor {
                    client_type,
                    label: None,
                    instance_id: None,
                },
                mode: ModePlan::new(SessionMode::Agent, true),
                realtime: disconnected("mock"),
                outbound,
            },
            inbox,
        )
    }

    #[test]
    fn a_live_holder_refuses_a_claim_that_is_not_a_takeover() {
        let registry = VoiceClientRegistry::default();
        let (first, _first_inbox) = connection("a", "user_1", ClientType::Desktop);
        let (second, _second_inbox) = connection("b", "user_1", ClientType::Web);
        registry.insert(first);
        registry.insert(second);

        assert!(registry.activate("a", false).granted);
        assert!(!registry.activate("b", false).granted);
        assert!(registry.is_active("user_1", "a"));
        assert!(registry.activate("b", true).granted);
        assert!(registry.is_active("user_1", "b"));
    }

    #[test]
    fn a_dead_holder_does_not_hold_the_slot_for_good() {
        let registry = VoiceClientRegistry::default();
        let (first, first_inbox) = connection("a", "user_1", ClientType::Desktop);
        let (second, _second_inbox) = connection("b", "user_1", ClientType::Web);
        registry.insert(first);
        registry.insert(second);
        assert!(registry.activate("a", false).granted);

        // The socket went away without a close frame.
        drop(first_inbox);
        assert!(
            registry.activate("b", false).granted,
            "an unclean disconnect must not hold the microphone for good",
        );
    }

    #[test]
    fn ownership_tells_the_holder_active_and_everyone_else_busy() {
        let registry = VoiceClientRegistry::default();
        let (first, _first_inbox) = connection("a", "user_1", ClientType::Desktop);
        let (second, _second_inbox) = connection("b", "user_1", ClientType::Web);
        let (other, _other_inbox) = connection("c", "user_2", ClientType::Cli);
        registry.insert(first);
        registry.insert(second);
        registry.insert(other);
        registry.activate("a", false);

        let states: Vec<String> = registry
            .ownership_broadcast("user_1")
            .into_iter()
            .map(|(_, frame)| {
                frame.into_value()["state"]
                    .as_str()
                    .unwrap_or("")
                    .to_owned()
            })
            .collect();
        assert_eq!(states, ["active", "busy"]);
        assert_eq!(
            registry.ownership_broadcast("user_2").len(),
            1,
            "another owner's connections are a separate broadcast",
        );
    }

    #[test]
    fn with_nobody_holding_every_connection_is_told_available() {
        let registry = VoiceClientRegistry::default();
        let (first, _inbox) = connection("a", "user_1", ClientType::Web);
        registry.insert(first);
        let frames = registry.ownership_broadcast("user_1");
        let value = frames.into_iter().next().expect("one frame").1.into_value();
        assert_eq!(value["state"], "available");
        assert_eq!(value["holder"], serde_json::Value::Null);
    }

    #[test]
    fn health_counts_connections_by_type_and_owners_by_slot() {
        let registry = VoiceClientRegistry::default();
        let (first, _a) = connection("a", "user_1", ClientType::Desktop);
        let (second, _b) = connection("b", "user_1", ClientType::Web);
        let (other, _c) = connection("c", "user_2", ClientType::Cli);
        registry.insert(first);
        registry.insert(second);
        registry.insert(other);
        registry.activate("a", false);
        registry.activate("c", false);

        let health = registry.health();
        assert_eq!(health.connected, 3);
        assert_eq!(health.active_owners, 2);
        assert_eq!(health.by_type.desktop, 1);
        assert_eq!(health.by_type.web, 1);
        assert_eq!(health.by_type.cli, 1);
        assert_eq!(health.realtime.totals.disconnected, 3);
    }

    #[test]
    fn removing_a_connection_releases_the_slot_it_held() {
        let registry = VoiceClientRegistry::default();
        let (first, _a) = connection("a", "user_1", ClientType::Desktop);
        registry.insert(first);
        registry.activate("a", false);
        assert!(registry.remove("a"), "the slot was released");
        assert!(!registry.is_active("user_1", "a"));
        assert_eq!(registry.len(), 0);
        assert!(!registry.remove("a"), "removing twice releases nothing");
    }
}
