//! The `wire_enum!` macro used by every frozen vocabulary in this crate.
//!
//! Upstream keeps its vocabularies as frozen JavaScript objects plus two `Set`s
//! used as direction gates (`shared/realtime-events.mjs:69-76`). In Rust the
//! direction is carried by the *type* — a [`GatewayServerEvent`] simply cannot
//! be sent by a client — but the runtime gate is still needed for the inbound
//! socket, so every vocabulary also gets `as_str` and `from_wire`.
//!
//! [`GatewayServerEvent`]: crate::GatewayServerEvent

/// Declares a frozen wire vocabulary.
///
/// Each variant is written `Variant = "wire.string"`. The literal is the
/// external contract; it becomes the `#[serde(rename = …)]` value, the
/// [`core::fmt::Display`] output, and the string `from_wire` matches on.
///
/// The generated type is `Copy` and totally ordered so it can be a map key or
/// live in a `const` slice, and it deliberately is **not** `#[non_exhaustive]`:
/// these vocabularies are frozen, and downstream crates are expected to match
/// them exhaustively so that adding a variant is a compile error rather than a
/// silently-ignored event.
macro_rules! wire_enum {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $wire:literal
            ),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                #[serde(rename = $wire)]
                $variant,
            )+
        }

        impl $name {
            #[doc = concat!(
                "Every [`", stringify!($name), "`] variant, in upstream ",
                "declaration order. The order is part of the contract: it is ",
                "the order the vocabulary is documented and asserted in."
            )]
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            #[doc = concat!(
                "The vocabulary's name, as it appears in a ",
                "[`ProtocolError::UnknownWireValue`](crate::ProtocolError::UnknownWireValue)."
            )]
            pub const VOCABULARY: &'static str = stringify!($name);

            /// The exact wire string for this variant.
            ///
            /// This value is an external contract; see the variant's own
            /// documentation for the upstream file it came from.
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $wire),+
                }
            }

            /// Parses a wire string, returning `None` when it is not a member
            /// of this vocabulary.
            ///
            /// This is the runtime direction gate — the Rust counterpart of
            /// upstream's `GATEWAY_CLIENT_EVENT_TYPES` /
            /// `GATEWAY_SERVER_EVENT_TYPES` `Set`s. An inbound frame whose
            /// `type` does not parse is rejected rather than ignored.
            pub fn from_wire(wire: &str) -> Option<Self> {
                match wire {
                    $($wire => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::core::str::FromStr for $name {
            type Err = $crate::error::ProtocolError;

            fn from_str(wire: &str) -> ::core::result::Result<Self, Self::Err> {
                Self::from_wire(wire).ok_or_else(|| {
                    $crate::error::ProtocolError::UnknownWireValue {
                        vocabulary: Self::VOCABULARY,
                        value: wire.to_owned(),
                    }
                })
            }
        }

        impl ::core::convert::AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }
    };
}

pub(crate) use wire_enum;
