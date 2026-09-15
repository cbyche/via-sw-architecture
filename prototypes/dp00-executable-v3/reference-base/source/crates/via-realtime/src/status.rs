//! The realtime connection's six states, and the one order they are decided in.
//!
//! Ported from `server/src/voice/realtime-connection-status.mjs` — twenty lines
//! upstream, and every one of them a contract. The result is published on
//! `GET /api/health` under `voiceClients.realtime` and
//! `voiceClients.realtime.byProvider`, and drives every UI connection
//! indicator.
//!
//! Two properties are asserted by name in the catalogue:
//!
//! - **the precedence** — `'disconnected'` (default) `< 'connecting' <
//!   'connected' < 'waking' < 'sleeping'`, with `blockedError` overriding all;
//! - **the result shape** — `{ provider, state, error? }`, frozen, and the
//!   `unavailable` case is deep-equality asserted, so an extra key is a failure.

use serde::{Deserialize, Serialize};

/// One realtime connection state.
///
/// External contract — `state-name` / *realtimeConnectionStatus states*. The
/// wire form is the lowercase variant name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    /// Nothing is connected and nothing is trying. The default.
    Disconnected,
    /// A connection attempt is in flight.
    Connecting,
    /// The realtime session is established and usable.
    Connected,
    /// Leaving sleep: the connection is being restored.
    Waking,
    /// Wake-word-only listening. Outranks [`Waking`](Self::Waking) so a session
    /// that is both never reports the weaker of the two.
    Sleeping,
    /// The frontend cannot be used at all — a missing credential, a fatal
    /// provider refusal. Overrides every other state and is the only one that
    /// carries an `error`.
    Unavailable,
}

impl ConnectionState {
    /// The wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disconnected => "disconnected",
            Self::Connecting => "connecting",
            Self::Connected => "connected",
            Self::Waking => "waking",
            Self::Sleeping => "sleeping",
            Self::Unavailable => "unavailable",
        }
    }
}

impl core::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What the status is computed from.
///
/// Upstream takes one options object whose every field defaults to `false` or
/// `''`, so [`Default`] plus struct-update syntax is the same call shape:
///
/// ```
/// use via_realtime::{ConnectionState, RealtimeConnectionInputs, realtime_connection_status};
///
/// let status = realtime_connection_status(&RealtimeConnectionInputs {
///     provider: "dashscope".into(),
///     ready: true,
///     waking: true,
///     ..Default::default()
/// });
/// assert_eq!(status.state, ConnectionState::Waking);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RealtimeConnectionInputs {
    /// The provider key, echoed straight back into the result.
    pub provider: String,
    /// A non-empty reason the frontend cannot be used. Overrides everything.
    ///
    /// Upstream defaults it to `''` and tests it for truthiness, so an empty
    /// string is *not* blocked — which is why this is a `String` rather than an
    /// `Option<String>`: the empty case has to behave the same way.
    pub blocked_error: String,
    /// Wake-word-only listening.
    pub sleeping: bool,
    /// Leaving sleep.
    pub waking: bool,
    /// The realtime session is established.
    pub ready: bool,
    /// A connection attempt is in flight.
    pub connecting: bool,
}

/// The published status.
///
/// Field order is upstream's object-literal order and is observable on
/// `/api/health`. `error` is **absent**, not null, when nothing is blocked —
/// upstream spreads `...(blockedError ? { error: blockedError } : {})`, and the
/// `unavailable` shape is deep-equality asserted in
/// `server/test/realtime-connection-status.test.mjs:21-30`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealtimeConnectionStatus {
    /// The provider key, verbatim.
    pub provider: String,
    /// The decided state.
    pub state: ConnectionState,
    /// Why the frontend is unavailable. Present only with
    /// [`ConnectionState::Unavailable`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Decide the realtime connection state.
///
/// External contract — `realtime-connection-status.mjs:1-20`. The `if` ladder is
/// reproduced in its exact order, because the order *is* the contract: a session
/// that is `ready` **and** `sleeping` reports `sleeping`, and one that is
/// `waking` **and** `ready` reports `waking`.
///
/// ```
/// use via_realtime::{ConnectionState, RealtimeConnectionInputs, realtime_connection_status};
///
/// let blocked = realtime_connection_status(&RealtimeConnectionInputs {
///     provider: "dashscope".into(),
///     ready: true,
///     sleeping: true,
///     blocked_error: "credential missing".into(),
///     ..Default::default()
/// });
/// assert_eq!(blocked.state, ConnectionState::Unavailable);
/// assert_eq!(blocked.error.as_deref(), Some("credential missing"));
/// ```
#[must_use]
pub fn realtime_connection_status(inputs: &RealtimeConnectionInputs) -> RealtimeConnectionStatus {
    let blocked = !inputs.blocked_error.is_empty();
    let state = if blocked {
        ConnectionState::Unavailable
    } else if inputs.sleeping {
        ConnectionState::Sleeping
    } else if inputs.waking {
        ConnectionState::Waking
    } else if inputs.ready {
        ConnectionState::Connected
    } else if inputs.connecting {
        ConnectionState::Connecting
    } else {
        ConnectionState::Disconnected
    };
    RealtimeConnectionStatus {
        provider: inputs.provider.clone(),
        state,
        error: blocked.then(|| inputs.blocked_error.clone()),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn base() -> RealtimeConnectionInputs {
        RealtimeConnectionInputs {
            provider: "dashscope".into(),
            ..Default::default()
        }
    }

    #[test]
    fn the_precedence_ladder_is_upstreams() {
        assert_eq!(
            realtime_connection_status(&base()).state,
            ConnectionState::Disconnected
        );
        assert_eq!(
            realtime_connection_status(&RealtimeConnectionInputs {
                connecting: true,
                ..base()
            })
            .state,
            ConnectionState::Connecting
        );
        assert_eq!(
            realtime_connection_status(&RealtimeConnectionInputs {
                ready: true,
                connecting: true,
                ..base()
            })
            .state,
            ConnectionState::Connected
        );
        assert_eq!(
            realtime_connection_status(&RealtimeConnectionInputs {
                ready: true,
                waking: true,
                ..base()
            })
            .state,
            ConnectionState::Waking
        );
        assert_eq!(
            realtime_connection_status(&RealtimeConnectionInputs {
                ready: true,
                waking: true,
                sleeping: true,
                ..base()
            })
            .state,
            ConnectionState::Sleeping
        );
    }

    #[test]
    fn a_blocked_error_overrides_every_other_input() {
        let status = realtime_connection_status(&RealtimeConnectionInputs {
            ready: true,
            sleeping: true,
            waking: true,
            connecting: true,
            blocked_error: "credential missing".into(),
            ..base()
        });
        assert_eq!(
            status,
            RealtimeConnectionStatus {
                provider: "dashscope".into(),
                state: ConnectionState::Unavailable,
                error: Some("credential missing".into()),
            }
        );
    }

    #[test]
    fn an_empty_blocked_error_is_not_blocked() {
        let status = realtime_connection_status(&RealtimeConnectionInputs {
            ready: true,
            blocked_error: String::new(),
            ..base()
        });
        assert_eq!(status.state, ConnectionState::Connected);
        assert_eq!(status.error, None);
    }

    #[test]
    fn the_unavailable_shape_carries_exactly_three_keys() {
        let status = realtime_connection_status(&RealtimeConnectionInputs {
            blocked_error: "nope".into(),
            ..base()
        });
        let json = serde_json::to_value(&status).expect("serialize");
        let object = json.as_object().expect("object");
        assert_eq!(
            object.keys().map(String::as_str).collect::<Vec<_>>(),
            ["provider", "state", "error"]
        );
    }

    #[test]
    fn a_healthy_shape_omits_the_error_key_entirely() {
        let status = realtime_connection_status(&base());
        let json = serde_json::to_value(&status).expect("serialize");
        let object = json.as_object().expect("object");
        assert_eq!(
            object.keys().map(String::as_str).collect::<Vec<_>>(),
            ["provider", "state"]
        );
    }

    #[test]
    fn the_states_order_by_precedence() {
        // `Ord` follows declaration order, which is the precedence order minus
        // the override — so a test that means "the stronger state wins" can say
        // so with `max` rather than restating the ladder.
        assert!(ConnectionState::Sleeping > ConnectionState::Waking);
        assert!(ConnectionState::Waking > ConnectionState::Connected);
        assert!(ConnectionState::Connected > ConnectionState::Connecting);
        assert!(ConnectionState::Connecting > ConnectionState::Disconnected);
        assert!(ConnectionState::Unavailable > ConnectionState::Sleeping);
    }
}
