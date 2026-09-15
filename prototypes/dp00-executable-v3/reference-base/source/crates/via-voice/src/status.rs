//! Realtime connection state and the `/api/health` aggregation.
//!
//! Ported from `server/src/voice/realtime-connection-status.mjs` and
//! `server/src/voice/realtime-gateway.mjs:2168-2210`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::mode::{Degradation, ModePlan};

/// Where one voice client's realtime connection is.
///
/// **External contract** — `realtime-connection-status.mjs:1-20`. The order of
/// the checks is the contract: a blocked provider reports `unavailable` even
/// while asleep, because the block is the fact that matters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RealtimeState {
    /// The session is usable.
    Connected,
    /// A handshake is in flight.
    Connecting,
    /// No session, and nothing wrong.
    #[default]
    Disconnected,
    /// The provider refused the session and reconnection has stopped.
    Unavailable,
    /// Wake-word-only.
    Sleeping,
    /// Coming back from sleep.
    Waking,
}

impl RealtimeState {
    /// The wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Connected => "connected",
            Self::Connecting => "connecting",
            Self::Disconnected => "disconnected",
            Self::Unavailable => "unavailable",
            Self::Sleeping => "sleeping",
            Self::Waking => "waking",
        }
    }

    /// The six states, in `/api/health` field order.
    pub const ALL: [Self; 6] = [
        Self::Connected,
        Self::Connecting,
        Self::Disconnected,
        Self::Unavailable,
        Self::Sleeping,
        Self::Waking,
    ];
}

/// What decides a client's realtime state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RealtimeStateInputs {
    /// The provider key.
    pub provider: String,
    /// The fatal error that blocked the provider, if any.
    pub blocked_error: String,
    /// The session is asleep.
    pub sleeping: bool,
    /// The session is coming back.
    pub waking: bool,
    /// The session is usable.
    pub ready: bool,
    /// A handshake is in flight.
    pub connecting: bool,
}

/// One client's realtime status.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealtimeStatus {
    /// Which provider.
    pub provider: String,
    /// Where it is.
    pub state: RealtimeState,
    /// The blocking error, when there is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Resolve one client's realtime state.
///
/// **External contract** — `realtime-connection-status.mjs:1-20`, in order:
/// blocked, waking, sleeping, ready, connecting, otherwise disconnected.
#[must_use]
pub fn realtime_connection_status(inputs: &RealtimeStateInputs) -> RealtimeStatus {
    let state = if !inputs.blocked_error.is_empty() {
        RealtimeState::Unavailable
    } else if inputs.waking {
        RealtimeState::Waking
    } else if inputs.sleeping {
        RealtimeState::Sleeping
    } else if inputs.ready {
        RealtimeState::Connected
    } else if inputs.connecting {
        RealtimeState::Connecting
    } else {
        RealtimeState::Disconnected
    };
    RealtimeStatus {
        provider: inputs.provider.clone(),
        state,
        error: (!inputs.blocked_error.is_empty()).then(|| inputs.blocked_error.clone()),
    }
}

/// A per-state tally.
///
/// **External contract** — `realtime-gateway.mjs:2172-2180`, field order
/// included: it is serialized straight into `/api/health`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateCounts {
    /// Sessions that are usable.
    pub connected: usize,
    /// Sessions handshaking.
    pub connecting: usize,
    /// Sessions with no connection.
    pub disconnected: usize,
    /// Sessions whose provider is blocked.
    pub unavailable: usize,
    /// Sessions asleep.
    pub sleeping: usize,
    /// Sessions waking.
    pub waking: usize,
    /// The blocking error, when any session in this bucket has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl StateCounts {
    fn increment(&mut self, state: RealtimeState) {
        match state {
            RealtimeState::Connected => self.connected += 1,
            RealtimeState::Connecting => self.connecting += 1,
            RealtimeState::Disconnected => self.disconnected += 1,
            RealtimeState::Unavailable => self.unavailable += 1,
            RealtimeState::Sleeping => self.sleeping += 1,
            RealtimeState::Waking => self.waking += 1,
        }
    }
}

/// A per-client-type tally.
///
/// **External contract** — `realtime-gateway.mjs:2170`. The three keys are
/// always present, even at zero, because a client reading `/api/health`
/// indexes them directly.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientTypeCounts {
    /// Electron / native shells.
    pub desktop: usize,
    /// `via chat` and other terminals.
    pub cli: usize,
    /// Browsers.
    pub web: usize,
}

/// The realtime block of `/api/health`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeAggregate {
    /// The totals across every client.
    #[serde(flatten)]
    pub totals: StateCounts,
    /// The same totals, split by provider.
    pub by_provider: BTreeMap<String, StateCounts>,
}

/// One session's contribution to the aggregate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceClientStatus {
    /// Which client type.
    pub client_type: via_protocol::ClientType,
    /// Its realtime status.
    pub realtime: RealtimeStatus,
    /// The mode it negotiated.
    pub mode: ModePlan,
}

/// One mode degradation, as `/api/health` reports it.
///
/// `docs/architecture.md` §2: the degradation is reported, **never silent**.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DegradedMode {
    /// What the client asked for.
    pub requested: String,
    /// What it actually runs as.
    pub effective: String,
    /// Why.
    pub reason: String,
    /// How many sessions are in this state.
    pub sessions: usize,
}

/// The `voiceClients` block of `/api/health`.
///
/// **External contract** — `realtime-gateway.mjs:2168-2210`, field order
/// included. [`degraded_modes`](Self::degraded_modes) is VIA's own addition,
/// which `docs/architecture.md` §2 requires.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceClientsHealth {
    /// How many clients are connected.
    pub connected: usize,
    /// How many owners have an active voice client.
    pub active_owners: usize,
    /// The per-type tally.
    pub by_type: ClientTypeCounts,
    /// The realtime aggregate.
    pub realtime: RealtimeAggregate,
    /// Every mode running as something other than what was asked for.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub degraded_modes: Vec<DegradedMode>,
}

/// Aggregate every connected client.
#[must_use]
pub fn aggregate(clients: &[VoiceClientStatus], active_owners: usize) -> VoiceClientsHealth {
    let mut health = VoiceClientsHealth {
        connected: clients.len(),
        active_owners,
        ..VoiceClientsHealth::default()
    };
    let mut degraded: BTreeMap<(String, String, &'static str), usize> = BTreeMap::new();
    for client in clients {
        match client.client_type {
            via_protocol::ClientType::Desktop => health.by_type.desktop += 1,
            via_protocol::ClientType::Cli => health.by_type.cli += 1,
            via_protocol::ClientType::Web => health.by_type.web += 1,
        }
        health.realtime.totals.increment(client.realtime.state);
        let provider = health
            .realtime
            .by_provider
            .entry(client.realtime.provider.clone())
            .or_default();
        provider.increment(client.realtime.state);
        if let Some(error) = &client.realtime.error {
            provider.error = Some(error.clone());
        }
        if let Some(reason) = client.mode.degradation() {
            *degraded
                .entry((
                    client.mode.requested().as_str().to_owned(),
                    client.mode.effective().as_str().to_owned(),
                    reason.as_str(),
                ))
                .or_default() += 1;
        }
    }
    health.degraded_modes = degraded
        .into_iter()
        .map(|((requested, effective, reason), sessions)| DegradedMode {
            requested,
            effective,
            reason: reason.to_owned(),
            sessions,
        })
        .collect();
    health
}

/// The `Degradation` that produced a [`DegradedMode`] row.
#[must_use]
pub fn degradation_from_reason(reason: &str) -> Option<Degradation> {
    match reason {
        "no_harness" => Some(Degradation::NoHarness),
        "context_engine_unavailable" => Some(Degradation::ContextEngineUnavailable),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use via_protocol::{ClientType, SessionMode};

    fn inputs() -> RealtimeStateInputs {
        RealtimeStateInputs {
            provider: "dashscope".to_owned(),
            ..RealtimeStateInputs::default()
        }
    }

    #[test]
    fn a_blocked_provider_reports_unavailable_even_while_asleep() {
        let status = realtime_connection_status(&RealtimeStateInputs {
            blocked_error: "bad credentials".to_owned(),
            sleeping: true,
            waking: true,
            ready: true,
            connecting: true,
            ..inputs()
        });
        assert_eq!(status.state, RealtimeState::Unavailable);
        assert_eq!(status.error.as_deref(), Some("bad credentials"));
    }

    #[test]
    fn the_state_precedence_is_the_catalogued_one() {
        let cases: [(RealtimeStateInputs, RealtimeState); 5] = [
            (
                RealtimeStateInputs {
                    waking: true,
                    sleeping: true,
                    ready: true,
                    ..inputs()
                },
                RealtimeState::Waking,
            ),
            (
                RealtimeStateInputs {
                    sleeping: true,
                    ready: true,
                    ..inputs()
                },
                RealtimeState::Sleeping,
            ),
            (
                RealtimeStateInputs {
                    ready: true,
                    connecting: true,
                    ..inputs()
                },
                RealtimeState::Connected,
            ),
            (
                RealtimeStateInputs {
                    connecting: true,
                    ..inputs()
                },
                RealtimeState::Connecting,
            ),
            (inputs(), RealtimeState::Disconnected),
        ];
        for (input, expected) in cases {
            assert_eq!(realtime_connection_status(&input).state, expected);
        }
    }

    #[test]
    fn a_healthy_status_carries_no_error_field() {
        let status = realtime_connection_status(&inputs());
        assert_eq!(status.error, None);
        let json = serde_json::to_string(&status).expect("serializes");
        assert!(!json.contains("error"));
    }

    #[test]
    fn the_aggregate_counts_by_type_and_by_provider() {
        let clients = vec![
            VoiceClientStatus {
                client_type: ClientType::Desktop,
                realtime: RealtimeStatus {
                    provider: "dashscope".to_owned(),
                    state: RealtimeState::Connected,
                    error: None,
                },
                mode: ModePlan::new(SessionMode::Agent, true),
            },
            VoiceClientStatus {
                client_type: ClientType::Cli,
                realtime: RealtimeStatus {
                    provider: "mock".to_owned(),
                    state: RealtimeState::Sleeping,
                    error: None,
                },
                mode: ModePlan::new(SessionMode::Agent, true),
            },
            VoiceClientStatus {
                client_type: ClientType::Cli,
                realtime: RealtimeStatus {
                    provider: "mock".to_owned(),
                    state: RealtimeState::Unavailable,
                    error: Some("no key".to_owned()),
                },
                mode: ModePlan::new(SessionMode::Agent, true),
            },
        ];
        let health = aggregate(&clients, 1);
        assert_eq!(health.connected, 3);
        assert_eq!(health.active_owners, 1);
        assert_eq!(health.by_type.desktop, 1);
        assert_eq!(health.by_type.cli, 2);
        assert_eq!(health.by_type.web, 0);
        assert_eq!(health.realtime.totals.connected, 1);
        assert_eq!(health.realtime.totals.sleeping, 1);
        assert_eq!(health.realtime.totals.unavailable, 1);
        let mock = health.realtime.by_provider.get("mock").expect("present");
        assert_eq!(mock.sleeping, 1);
        assert_eq!(mock.unavailable, 1);
        assert_eq!(mock.error.as_deref(), Some("no key"));
        assert!(health.degraded_modes.is_empty());
    }

    #[test]
    fn a_degraded_mode_is_reported_never_silent() {
        let clients = vec![
            VoiceClientStatus {
                client_type: ClientType::Web,
                realtime: RealtimeStatus::default(),
                mode: ModePlan::new(SessionMode::Agent, false),
            },
            VoiceClientStatus {
                client_type: ClientType::Web,
                realtime: RealtimeStatus::default(),
                // An embedder with no Context Engine: the one caller that
                // still reports `context_engine_unavailable`.
                mode: ModePlan::with_context_engine(SessionMode::Interface, true, false),
            },
            VoiceClientStatus {
                client_type: ClientType::Web,
                realtime: RealtimeStatus::default(),
                mode: ModePlan::new(SessionMode::Agent, false),
            },
        ];
        let health = aggregate(&clients, 0);
        assert_eq!(health.degraded_modes.len(), 2);
        let agent = health
            .degraded_modes
            .iter()
            .find(|row| row.requested == "agent")
            .expect("the agent row");
        assert_eq!(agent.effective, "direct");
        assert_eq!(agent.reason, "no_harness");
        assert_eq!(agent.sessions, 2);
        let interface = health
            .degraded_modes
            .iter()
            .find(|row| row.requested == "interface")
            .expect("the interface row");
        assert_eq!(interface.reason, "context_engine_unavailable");
        assert_eq!(interface.sessions, 1);
        assert_eq!(
            degradation_from_reason(&interface.reason),
            Some(Degradation::ContextEngineUnavailable),
        );
    }

    #[test]
    fn an_empty_gateway_reports_zeroes_rather_than_nothing() {
        let health = aggregate(&[], 0);
        let json = serde_json::to_value(&health).expect("serializes");
        assert_eq!(json["connected"], 0);
        assert_eq!(json["byType"]["desktop"], 0);
        assert_eq!(json["byType"]["cli"], 0);
        assert_eq!(json["byType"]["web"], 0);
        assert_eq!(json["realtime"]["connected"], 0);
        assert!(
            json["realtime"]["byProvider"]
                .as_object()
                .is_some_and(serde_json::Map::is_empty)
        );
        assert!(
            json.get("degradedModes").is_none(),
            "empty rows are omitted"
        );
    }

    #[test]
    fn the_six_states_round_trip_through_their_wire_names() {
        for state in RealtimeState::ALL {
            let json = serde_json::to_string(&state).expect("serializes");
            assert_eq!(json, format!("\"{}\"", state.as_str()));
        }
    }
}
