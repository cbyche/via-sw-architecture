//! What a connected client can put *into* a turn.
//!
//! Ported from upstream `shared/client-input-capabilities.mjs:1-16`.
//!
//! The Gateway sends this profile to a client so the client knows whether to
//! render a composer at all. Upstream names five modules across server and web
//! that read it, which is why the four flag names, their values per client type
//! and the fall-back-to-`web` rule are all external contracts rather than a
//! presentation detail.
//!
//! The one that matters is `desktop`: the desktop shell is voice-only, so
//! `text`, `image` and `resource` are **false** there and only `audio` is true.
//! A client that renders a composer anyway is offering an input the Gateway
//! will not accept.
//!
//! # Key order is a contract
//!
//! The profile is serialized into a frame, and upstream's object literal fixes
//! the order `text, audio, image, resource`. Struct field order below *is* that
//! order — see [`CLIENT_INPUT_CAPABILITY_FIELDS`], which
//! `via-conformance` asserts against the catalogue.

use serde::{Deserialize, Serialize};

use crate::macros::wire_enum;

wire_enum! {
    /// The client forms upstream publishes an input profile for.
    ///
    /// **External contract** — the three keys of upstream's frozen `PROFILES`
    /// object (`shared/client-input-capabilities.mjs:1-5`), in that order. A
    /// `clientType` outside this set is not an error: it resolves to
    /// [`DEFAULT_CLIENT_TYPE`], which is what
    /// [`ClientInputCapabilities::for_wire`] reproduces.
    pub enum ClientType {
        /// `web` — the browser client. Every input form is available.
        Web = "web",
        /// `cli` — the terminal client. Same profile as [`Web`](Self::Web);
        /// upstream declares it separately rather than aliasing it, and so does
        /// this port, because the two are free to diverge.
        Cli = "cli",
        /// `desktop` — the voice-only shell. Audio in, nothing else.
        Desktop = "desktop",
    }
}

/// The `clientType` assumed when a client sends none, and the profile an
/// unrecognised `clientType` falls back to.
///
/// **External contract** — upstream's default parameter
/// `clientInputCapabilities(clientType = 'web')` together with
/// `PROFILES[clientType] || PROFILES.web`
/// (`shared/client-input-capabilities.mjs:7-10`). Both spellings of "I do not
/// know this client" land on the same profile.
pub const DEFAULT_CLIENT_TYPE: ClientType = ClientType::Web;

/// The four capability flag names, in serialization order.
///
/// **External contract** — the object literal order in upstream
/// `shared/client-input-capabilities.mjs:2-4`. Declared as a constant so the
/// order can be asserted directly instead of inferred from a serialized sample.
pub const CLIENT_INPUT_CAPABILITY_FIELDS: [&str; 4] = ["text", "audio", "image", "resource"];

/// Which input kinds a client may contribute to a turn.
///
/// **External contract — field order included.** `serde` serializes struct
/// fields in declaration order, so the declaration order below is the wire
/// order; do not reorder it. See [`CLIENT_INPUT_CAPABILITY_FIELDS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientInputCapabilities {
    /// Typed text.
    pub text: bool,
    /// Captured or streamed audio. True for every client form upstream ships.
    pub audio: bool,
    /// Image attachments.
    pub image: bool,
    /// Resource (file / URI) attachments.
    pub resource: bool,
}

impl ClientInputCapabilities {
    /// The `web` profile: everything.
    ///
    /// **External contract** — `shared/client-input-capabilities.mjs:2`.
    pub const WEB: Self = Self {
        text: true,
        audio: true,
        image: true,
        resource: true,
    };

    /// The `cli` profile: everything, and identical to [`Self::WEB`] today.
    ///
    /// **External contract** — `shared/client-input-capabilities.mjs:3`.
    pub const CLI: Self = Self {
        text: true,
        audio: true,
        image: true,
        resource: true,
    };

    /// The `desktop` profile: audio only.
    ///
    /// **External contract** — `shared/client-input-capabilities.mjs:4`. The
    /// three `false`s are the reason this contract exists.
    pub const DESKTOP: Self = Self {
        text: false,
        audio: true,
        image: false,
        resource: false,
    };

    /// The profile for a known client form.
    ///
    /// **External contract** — the `PROFILES` lookup
    /// (`shared/client-input-capabilities.mjs:7-10`). Upstream returns a fresh
    /// object (`{ ...profile }`) so a caller cannot mutate the frozen table;
    /// here the type is `Copy` and the constants are `const`, so the same
    /// guarantee holds without the clone.
    #[must_use]
    pub const fn for_client_type(client_type: ClientType) -> Self {
        match client_type {
            ClientType::Web => Self::WEB,
            ClientType::Cli => Self::CLI,
            ClientType::Desktop => Self::DESKTOP,
        }
    }

    /// The profile for a `clientType` string straight off the wire.
    ///
    /// **External contract** — `PROFILES[clientType] || PROFILES.web`
    /// (`shared/client-input-capabilities.mjs:8`). An unknown, misspelled or
    /// absent `clientType` resolves to [`DEFAULT_CLIENT_TYPE`] rather than
    /// failing: upstream's `||` fallback is what keeps a client from an
    /// unfamiliar host getting no composer at all.
    ///
    /// ```
    /// use via_protocol::ClientInputCapabilities;
    ///
    /// assert_eq!(ClientInputCapabilities::for_wire(Some("desktop")).text, false);
    /// assert_eq!(ClientInputCapabilities::for_wire(Some("tui")), ClientInputCapabilities::WEB);
    /// assert_eq!(ClientInputCapabilities::for_wire(None), ClientInputCapabilities::WEB);
    /// ```
    #[must_use]
    pub fn for_wire(client_type: Option<&str>) -> Self {
        let resolved = client_type
            .and_then(ClientType::from_wire)
            .unwrap_or(DEFAULT_CLIENT_TYPE);
        Self::for_client_type(resolved)
    }

    /// Whether the client should render a composer.
    ///
    /// **External contract** — upstream `supportsComposerInput`
    /// (`shared/client-input-capabilities.mjs:12-16`):
    /// `text || image || resource`. Note that `audio` is deliberately **not**
    /// in the disjunction — a voice-only client has audio and still gets no
    /// composer, which is exactly the `desktop` case.
    #[must_use]
    pub const fn supports_composer_input(self) -> bool {
        self.text || self.image || self.resource
    }
}

impl Default for ClientInputCapabilities {
    /// The [`DEFAULT_CLIENT_TYPE`] profile.
    fn default() -> Self {
        Self::for_client_type(DEFAULT_CLIENT_TYPE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_profiles_are_the_upstream_literals() {
        assert_eq!(
            ClientInputCapabilities::WEB,
            ClientInputCapabilities {
                text: true,
                audio: true,
                image: true,
                resource: true
            }
        );
        assert_eq!(ClientInputCapabilities::CLI, ClientInputCapabilities::WEB);
        assert_eq!(
            ClientInputCapabilities::DESKTOP,
            ClientInputCapabilities {
                text: false,
                audio: true,
                image: false,
                resource: false
            }
        );
    }

    #[test]
    fn audio_is_the_one_flag_every_client_has() {
        for client_type in ClientType::ALL {
            assert!(
                ClientInputCapabilities::for_client_type(*client_type).audio,
                "{client_type} must accept audio"
            );
        }
    }

    #[test]
    fn an_unknown_client_type_falls_back_to_web() {
        for unknown in ["", "tui", "WEB", "desktop ", "electron", "cli\u{0}"] {
            assert_eq!(
                ClientInputCapabilities::for_wire(Some(unknown)),
                ClientInputCapabilities::WEB,
                "`{unknown}` must fall back to the web profile"
            );
        }
        assert_eq!(
            ClientInputCapabilities::for_wire(None),
            ClientInputCapabilities::WEB
        );
    }

    #[test]
    fn only_desktop_hides_the_composer() {
        assert!(ClientInputCapabilities::WEB.supports_composer_input());
        assert!(ClientInputCapabilities::CLI.supports_composer_input());
        assert!(!ClientInputCapabilities::DESKTOP.supports_composer_input());
        // Audio alone never enables the composer, whatever the client form.
        assert!(
            !ClientInputCapabilities {
                text: false,
                audio: true,
                image: false,
                resource: false
            }
            .supports_composer_input()
        );
    }

    #[test]
    fn serialization_key_order_is_the_contract_order() {
        let json = serde_json::to_string(&ClientInputCapabilities::DESKTOP)
            .expect("a struct of four bools always serializes");
        assert_eq!(
            json,
            r#"{"text":false,"audio":true,"image":false,"resource":false}"#
        );
        let keys: Vec<&str> = CLIENT_INPUT_CAPABILITY_FIELDS.to_vec();
        assert_eq!(keys, ["text", "audio", "image", "resource"]);
    }

    #[test]
    fn client_type_round_trips_through_the_wire() {
        for client_type in ClientType::ALL {
            assert_eq!(
                ClientType::from_wire(client_type.as_str()),
                Some(*client_type)
            );
        }
        assert_eq!(ClientType::from_wire("tui"), None);
    }
}
