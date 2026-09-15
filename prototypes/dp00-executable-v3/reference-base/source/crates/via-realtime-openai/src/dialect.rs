//! Which Realtime *dialect* an endpoint speaks, and the four candidate URLs
//! that follow from it.
//!
//! Ported from ARGO `tinicore/src/llm/providers/openai_live.rs:45-130,300-560`
//! — `OPENAI_REALTIME_HOST`, `AZURE_REALTIME_API_VERSION`,
//! `AZURE_REALTIME_LEGACY_API_VERSION`, `host_is_azure_resource`, `QUERY_VALUE`,
//! `Dialect`, `candidate_urls`, `strip_scheme_and_path`.
//!
//! # The rule, stated once
//!
//! **Dialect follows the host, not the provider label** — ARGO bring-up §4.
//! Only a first-party resource (`*.openai.azure.com`, `*.azure-api.net`,
//! `*.cognitiveservices.azure.com`) is offered Azure's native routes first;
//! anything else is a gateway and gets `/v1/realtime?model=` first. A gateway
//! speaks OpenAI's shape whatever it forwards to.
//!
//! That distinction is not cosmetic. The environment ARGO was brought up
//! against is a litellm gateway forwarding to Azure: the app labelled the
//! provider `azure_openai`, and the wire protocol at that host was OpenAI's.
//! Asking for Azure's routes first wasted attempts — and worse, the gateway
//! **accepts a socket on routes it does not bridge**, so a wrong-order walk
//! ended on a 101 that never produced a frame.
//!
//! # The two dialects
//!
//! | | OpenAI | Azure |
//! |---|---|---|
//! | path | `/v1/realtime` | `/openai/realtime` |
//! | model | `?model=<id>` | `?deployment=<name>` (+ required `api-version`) |
//! | auth | `Authorization: Bearer` | `api-key` header |
//!
//! Everything after the upgrade — `session.update`, the event stream, tool
//! calls — is identical, so only the handshake branches.

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, PercentEncode, utf8_percent_encode};

/// Default OpenAI Realtime host.
///
/// External contract — ARGO `openai_live.rs:47`, and the host half of
/// [`via_catalog::realtime_provider::DEFAULT_OPENAI_REALTIME_URL`]. A plain proxy that speaks
/// OpenAI's own dialect is reachable by configuring a base URL.
pub const OPENAI_REALTIME_HOST: &str = "api.openai.com";

/// Default `api-version` for the Azure Realtime WebSocket route.
///
/// On Azure this single value decides three things at once: whether the route
/// exists, which models are addressable, and — because Azure pins the event
/// vocabulary to the `api-version` rather than to the model — whether the
/// stream speaks pre-GA or GA event names. `via-realtime`'s two dialects between
/// them decode both vocabularies, so the remaining job of this constant is just
/// to be new enough for the deployment in play. A version older than the
/// deployed model yields an explicit Azure rejection on the upgrade, not a
/// silent degrade.
///
/// External contract — ARGO `openai_live.rs:60`.
pub const AZURE_REALTIME_API_VERSION: &str = "2025-08-28";

/// The original realtime preview `api-version`, kept as a last-resort candidate
/// for resources pinned to the preview channel.
///
/// External contract — ARGO `openai_live.rs:113`.
pub const AZURE_REALTIME_LEGACY_API_VERSION: &str = "2024-10-01-preview";

/// The WebSocket subprotocol every candidate offers.
///
/// External contract — ARGO `openai_live.rs:466-497`, and ARGO bring-up §2 /
/// §4: **the upgrade succeeds without it but is not routed.** OpenAI's own
/// browser client negotiates it, and a gateway written against that client can
/// treat it as the signal that selects the realtime handler — accepting the
/// socket generically when it is absent and never binding anything upstream.
///
/// That is not a guess: probing the gateway ARGO was brought up against, a
/// handshake WITHOUT the header got a bare `101`, and the same handshake WITH it
/// got `101` plus `Sec-WebSocket-Protocol: realtime` echoed back. A server that
/// echoes a chosen subprotocol is one that reads it.
///
/// Deliberately **not** `openai-insecure-api-key.<key>`, the other value that
/// client sends: it exists so a browser can authenticate where headers cannot be
/// set, and putting a credential in a subprotocol name would write it into every
/// proxy log along the path.
pub const SUBPROTOCOL: &str = "realtime";

/// The name of the subprotocol header.
pub const SUBPROTOCOL_HEADER: &str = "Sec-WebSocket-Protocol";

/// Characters that must be percent-encoded in a query value.
///
/// External contract — ARGO `openai_live.rs:118-129`. RFC 3986 §2.3 unreserved
/// characters (`-`, `.`, `_`, `~`) are deliberately left alone.
/// `NON_ALPHANUMERIC` on its own would encode them, and while `%2D` is a legal
/// spelling of `-` that a spec-compliant server decodes, API gateways have been
/// known to route on the raw query string — encoding a hyphen inside
/// `api-version=2025-08-28` is not worth the risk for zero gain.
const QUERY_VALUE: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

/// Percent-encode one query value, leaving the RFC 3986 unreserved characters
/// (`-`, `.`, `_`, `~`) literal.
///
/// A free function rather than a closure inside the URL builder: the borrow
/// checker cannot tie a closure's return lifetime to its argument here.
#[must_use]
pub fn encode_query_value(value: &str) -> PercentEncode<'_> {
    utf8_percent_encode(value, QUERY_VALUE)
}

/// Which Realtime dialect a provider dials in.
///
/// ARGO models this as two constructors on one provider
/// (`OpenAiRealtimeProvider::new` / `::azure`) because its `ProviderConfig`
/// already carried a provider *label*. VIA has one provider key, `openai`, so
/// the choice is a settings field — defaulted from the host by
/// [`OpenAiDialect::for_host`], which is the same rule stated in the same
/// direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OpenAiDialect {
    /// `/v1/realtime?model=`, `Authorization: Bearer`. One candidate.
    #[default]
    OpenAi,
    /// Azure's own routes plus the OpenAI-compatible ones, `api-key` auth. Four
    /// candidates, ordered by what the host is.
    Azure,
}

impl OpenAiDialect {
    /// The word ARGO's `[live] dialect=` marker prints.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Azure => "azure",
        }
    }

    /// Parse a configured dialect name, trimmed and case-insensitively.
    ///
    /// `azure`, `azure-openai` and `azure_openai` all select
    /// [`Azure`](Self::Azure) — the last two because that is the string ARGO's
    /// app used as its provider label, and an operator migrating a working
    /// configuration should not have to discover a third spelling.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "openai" => Some(Self::OpenAi),
            "azure" | "azure-openai" | "azure_openai" => Some(Self::Azure),
            _ => None,
        }
    }

    /// The dialect a host implies when nothing chose one.
    ///
    /// `api.openai.com` — and an empty host, which is what an unconfigured
    /// endpoint resolves to — is unambiguous and gets exactly one candidate.
    /// **Every other host is either a first-party Azure resource or a gateway**,
    /// and both need the four-candidate walk: the resource because its two
    /// surfaces are not discoverable from the endpoint string, the gateway
    /// because a 101 on a route it does not bridge is indistinguishable from a
    /// working session until the first-frame probe says otherwise.
    #[must_use]
    pub fn for_host(host: &str) -> Self {
        let host = host.trim();
        if host.is_empty() || host.eq_ignore_ascii_case(OPENAI_REALTIME_HOST) {
            Self::OpenAi
        } else {
            Self::Azure
        }
    }
}

impl core::fmt::Display for OpenAiDialect {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Whether a host is a first-party Azure OpenAI resource.
///
/// External contract — ARGO `openai_live.rs:69-75`. Distinguishes a real
/// resource from a gateway that merely *forwards* to one. The distinction
/// matters because the provider label and the endpoint can disagree: a
/// deployment reached through litellm is still configured as "azure" in a
/// client, but the wire protocol at that host is OpenAI's.
#[must_use]
pub fn host_is_azure_resource(host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    host.ends_with(".openai.azure.com")
        || host.ends_with(".azure-api.net")
        || host.ends_with(".cognitiveservices.azure.com")
}

/// The WebSocket scheme every candidate uses unless the endpoint says otherwise.
pub const SECURE_SCHEME: &str = "wss";

/// The scheme a plaintext endpoint uses.
///
/// **A VIA addition, not ARGO's.** ARGO's endpoint always arrived from a mobile
/// app's HTTPS configuration, so its URL builder hard-codes `wss://`. VIA's
/// configuration surface already ships a plaintext loopback default for another
/// provider (`ws://127.0.0.1:8765/v1/realtime`, the huggingface/speech-to-speech
/// endpoint), and a litellm gateway on loopback is an ordinary deployment.
/// Forcing TLS there would make a working configuration unreachable.
///
/// It is opt-in and explicit: only an endpoint configured as `ws://` or
/// `http://` gets it. Anything else — including a bare host — is `wss`.
pub const INSECURE_SCHEME: &str = "ws";

/// The scheme the candidates for `base` are built with.
#[must_use]
pub fn url_scheme(base: &str) -> &'static str {
    let base = base.trim();
    if base.starts_with("ws://") || base.starts_with("http://") {
        INSECURE_SCHEME
    } else {
        SECURE_SCHEME
    }
}

/// Reduce a base URL to its bare host.
///
/// External contract — ARGO `openai_live.rs:568-576`. Config surfaces commonly
/// store the endpoint with `/openai/v1` appended for the Responses API; realtime
/// must not inherit it and produce `/openai/v1/openai/v1/realtime`.
#[must_use]
pub fn strip_scheme_and_path(base: &str) -> String {
    let no_scheme = base
        .strip_prefix("https://")
        .or_else(|| base.strip_prefix("wss://"))
        .or_else(|| base.strip_prefix("http://"))
        .or_else(|| base.strip_prefix("ws://"))
        .unwrap_or(base);
    no_scheme
        .split('/')
        .next()
        .unwrap_or(no_scheme)
        .trim()
        .to_owned()
}

/// The URLs a dialect dials, in the order it dials them.
///
/// External contract — ARGO `openai_live.rs:399-462`.
///
/// # Why Azure has more than one
///
/// Azure exposes **two** shapes for the same deployment and which one a resource
/// answers on is not discoverable from the endpoint string:
///
/// - the OpenAI-compatible **v1 surface** — `/openai/v1/realtime` with `model=`
///   selecting the deployment, and
/// - the **classic** route — `/openai/realtime` with `deployment=` and a dated
///   `api-version`.
///
/// The v1 surface takes **no `api-version` at all**. Microsoft's migration guide
/// is explicit that the parameter "is no longer supported in GA endpoint URLs"
/// and lists its presence as the cause of a `401 Unauthorized` — so passing one
/// there does not select a channel, it fails the request in a way that reads
/// exactly like a bad credential. `api_version` therefore applies only to the
/// classic routes.
///
/// A resource that does not serve the shape we picked answers `403` with an
/// empty body — its gateway rejecting an unknown route before the service sees
/// it — which is indistinguishable from an auth refusal. Guessing costs a full
/// round-trip per guess, so the walk tries them in order and reports every
/// rejection it collected.
#[must_use]
pub fn candidate_urls(
    dialect: OpenAiDialect,
    scheme: &str,
    host: &str,
    model: &str,
    api_version: &str,
) -> Vec<String> {
    let openai_shape = format!(
        "{scheme}://{host}/v1/realtime?model={}",
        encode_query_value(model)
    );
    if dialect == OpenAiDialect::OpenAi {
        return vec![openai_shape];
    }

    let azure_v1 = format!(
        "{scheme}://{host}/openai/v1/realtime?model={}",
        encode_query_value(model)
    );
    let azure_classic = format!(
        "{scheme}://{host}/openai/realtime?api-version={}&deployment={}",
        encode_query_value(api_version),
        encode_query_value(model)
    );
    let azure_legacy = format!(
        "{scheme}://{host}/openai/realtime?api-version={}&deployment={}",
        encode_query_value(AZURE_REALTIME_LEGACY_API_VERSION),
        encode_query_value(model)
    );

    // Order by what the HOST is, not by what the provider is labelled.
    if host_is_azure_resource(host) {
        vec![azure_v1, azure_classic, azure_legacy, openai_shape]
    } else {
        vec![openai_shape, azure_v1, azure_classic, azure_legacy]
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::schema::{RealtimeSchema, protocol_from_route};

    #[test]
    fn azure_host_detection_covers_the_real_domains_only() {
        for host in [
            "my-res.openai.azure.com",
            "MY-RES.OPENAI.AZURE.COM",
            "gw.azure-api.net",
            "r.cognitiveservices.azure.com",
        ] {
            assert!(host_is_azure_resource(host), "{host}");
        }
        for host in [
            "sr-aic-llm-proxy.lit.ovh",
            "api.openai.com",
            // The suffix check must be a suffix check: a look-alike domain that
            // merely *contains* the real one is a different party.
            "openai.azure.com.evil.example",
            "proxy.internal",
            "openai.azure.com",
            "",
        ] {
            assert!(!host_is_azure_resource(host), "{host}");
        }
    }

    #[test]
    fn the_dialect_a_host_implies() {
        assert_eq!(
            OpenAiDialect::for_host("api.openai.com"),
            OpenAiDialect::OpenAi
        );
        assert_eq!(
            OpenAiDialect::for_host("API.OPENAI.COM"),
            OpenAiDialect::OpenAi
        );
        for gateway in [
            "my-res.openai.azure.com",
            "sr-aic-llm-proxy.lit.ovh",
            "proxy.internal",
        ] {
            assert_eq!(
                OpenAiDialect::for_host(gateway),
                OpenAiDialect::Azure,
                "{gateway}"
            );
        }
        // An unconfigured endpoint resolves to OpenAI's own host, so an empty
        // host must not be read as a gateway that needs four candidates.
        for unconfigured in ["", "   "] {
            assert_eq!(
                OpenAiDialect::for_host(unconfigured),
                OpenAiDialect::OpenAi,
                "{unconfigured:?}"
            );
        }
    }

    #[test]
    fn a_dialect_name_parses_in_every_spelling_an_operator_might_carry_over() {
        for (value, expected) in [
            ("openai", OpenAiDialect::OpenAi),
            ("  OpenAI\n", OpenAiDialect::OpenAi),
            ("azure", OpenAiDialect::Azure),
            ("AZURE", OpenAiDialect::Azure),
            ("azure-openai", OpenAiDialect::Azure),
            ("azure_openai", OpenAiDialect::Azure),
        ] {
            assert_eq!(OpenAiDialect::parse(value), Some(expected), "{value}");
        }
        for value in ["", "  ", "gemini", "azureopenai", "openai-azure"] {
            assert_eq!(OpenAiDialect::parse(value), None, "{value}");
        }
    }

    #[test]
    fn the_dialect_prints_the_word_argos_marker_prints() {
        assert_eq!(OpenAiDialect::OpenAi.to_string(), "openai");
        assert_eq!(OpenAiDialect::Azure.to_string(), "azure");
        assert_eq!(OpenAiDialect::default(), OpenAiDialect::OpenAi);
    }

    #[test]
    fn the_openai_dialect_has_exactly_one_candidate() {
        assert_eq!(
            candidate_urls(
                OpenAiDialect::OpenAi,
                SECURE_SCHEME,
                OPENAI_REALTIME_HOST,
                "gpt-realtime-2.1-mini",
                AZURE_REALTIME_API_VERSION
            ),
            vec!["wss://api.openai.com/v1/realtime?model=gpt-realtime-2.1-mini"]
        );
    }

    #[test]
    fn a_gateway_host_is_offered_the_openai_shape_first() {
        // The case that actually bit: provider labelled azure, endpoint a
        // litellm gateway. The gateway serves OpenAI's shape, so asking for
        // Azure's routes first wastes attempts — and worse, a gateway may
        // ACCEPT the socket on a route it does not forward, which ends the
        // candidate walk on a session that never produces a frame.
        let candidates = candidate_urls(
            OpenAiDialect::Azure,
            SECURE_SCHEME,
            "proxy.example.internal",
            "gpt-realtime-2.1-mini",
            AZURE_REALTIME_API_VERSION,
        );
        assert_eq!(
            candidates[0],
            "wss://proxy.example.internal/v1/realtime?model=gpt-realtime-2.1-mini"
        );
        assert_eq!(candidates.len(), 4, "{candidates:?}");
        assert!(
            candidates[1].contains("/openai/v1/realtime"),
            "{candidates:?}"
        );
    }

    #[test]
    fn a_first_party_resource_is_offered_the_v1_surface_first() {
        // Ordering is the whole design: the v1 surface is the one with direct
        // evidence, so it must be attempt #1 or a resource that serves only v1
        // pays a wasted round-trip per launch.
        let candidates = candidate_urls(
            OpenAiDialect::Azure,
            SECURE_SCHEME,
            "my-res.openai.azure.com",
            "gpt-realtime-2.1",
            AZURE_REALTIME_API_VERSION,
        );
        assert_eq!(candidates.len(), 4, "{candidates:?}");
        assert_eq!(
            candidates[0],
            "wss://my-res.openai.azure.com/openai/v1/realtime?model=gpt-realtime-2.1"
        );
        // No `api-version` on the v1 surface: Microsoft lists the parameter's
        // presence there as the cause of a 401, so carrying one would fail the
        // most-likely candidate in a way indistinguishable from a bad key.
        assert!(!candidates[0].contains("api-version"), "{}", candidates[0]);
        assert!(
            candidates[1].contains("/openai/realtime?"),
            "{}",
            candidates[1]
        );
        assert!(
            candidates[1].contains("deployment=gpt-realtime-2.1"),
            "{}",
            candidates[1]
        );
        assert!(
            candidates[1].contains(&format!("api-version={AZURE_REALTIME_API_VERSION}")),
            "{}",
            candidates[1]
        );
        assert!(
            candidates[2].contains(&format!("api-version={AZURE_REALTIME_LEGACY_API_VERSION}")),
            "{}",
            candidates[2]
        );
        assert!(
            candidates[3].contains("/v1/realtime?model="),
            "{}",
            candidates[3]
        );
    }

    #[test]
    fn every_candidate_url_classifies_and_both_schemas_are_covered() {
        // The walk must not dial a route whose schema cannot be named. This is
        // the pin that fails if a new candidate shape is added without deciding
        // which generation it speaks.
        let candidates = candidate_urls(
            OpenAiDialect::Azure,
            SECURE_SCHEME,
            "my-res.openai.azure.com",
            "d",
            AZURE_REALTIME_API_VERSION,
        );
        assert!(
            candidates
                .iter()
                .any(|url| protocol_from_route(url) == RealtimeSchema::Ga)
        );
        assert!(
            candidates
                .iter()
                .any(|url| protocol_from_route(url) == RealtimeSchema::Preview)
        );
    }

    #[test]
    fn the_azure_candidates_are_all_distinct() {
        // Duplicates would silently turn the retry budget into repeated
        // identical attempts. The two classic routes differ only by
        // `api-version`, which is exactly the thing being varied.
        for host in ["r.openai.azure.com", "gateway.internal"] {
            let candidates = candidate_urls(
                OpenAiDialect::Azure,
                SECURE_SCHEME,
                host,
                "d",
                AZURE_REALTIME_API_VERSION,
            );
            let unique: std::collections::BTreeSet<&String> = candidates.iter().collect();
            assert_eq!(unique.len(), candidates.len(), "{host}: {candidates:?}");
        }
    }

    #[test]
    fn a_configured_api_version_reaches_only_the_classic_candidate() {
        let candidates = candidate_urls(
            OpenAiDialect::Azure,
            SECURE_SCHEME,
            "r.openai.azure.com",
            "d",
            "2024-12-17",
        );
        assert!(
            candidates[0].contains("/openai/v1/realtime"),
            "{candidates:?}"
        );
        assert!(!candidates[0].contains("api-version"), "{candidates:?}");
        assert!(
            candidates[1].contains("api-version=2024-12-17"),
            "{candidates:?}"
        );
        assert!(
            candidates[2].contains(&format!("api-version={AZURE_REALTIME_LEGACY_API_VERSION}")),
            "the legacy candidate is not overridable: {candidates:?}"
        );
    }

    #[test]
    fn an_endpoint_path_suffix_is_discarded() {
        // Config surfaces commonly store the endpoint with `/openai/v1`
        // appended for the Responses API.
        for base in [
            "https://my-res.openai.azure.com/",
            "https://my-res.openai.azure.com/openai",
            "https://my-res.openai.azure.com/openai/v1",
            "wss://my-res.openai.azure.com/openai/v1/realtime",
            "my-res.openai.azure.com",
        ] {
            let host = strip_scheme_and_path(base);
            assert_eq!(host, "my-res.openai.azure.com", "{base}");
            for url in candidate_urls(
                OpenAiDialect::Azure,
                SECURE_SCHEME,
                &host,
                "d",
                AZURE_REALTIME_API_VERSION,
            ) {
                assert!(url.matches("/openai/").count() <= 1, "{base} → {url}");
                assert_eq!(url.matches("/realtime").count(), 1, "{base} → {url}");
            }
        }
    }

    #[test]
    fn every_scheme_a_config_surface_writes_is_stripped() {
        for base in [
            "https://h.example/x",
            "http://h.example/x",
            "wss://h.example/x",
            "ws://h.example/x",
            "h.example/x",
            "  h.example  ",
        ] {
            assert_eq!(strip_scheme_and_path(base), "h.example", "{base}");
        }
    }

    #[test]
    fn only_an_explicitly_plaintext_endpoint_drops_tls() {
        for secure in [
            "",
            "  ",
            "api.openai.com",
            "https://r.openai.azure.com",
            "wss://gw.example/v1",
            // A host that merely *contains* the substring is not a scheme.
            "ws-gateway.example",
            "httpbin.example",
        ] {
            assert_eq!(url_scheme(secure), SECURE_SCHEME, "{secure:?}");
        }
        for plaintext in [
            "ws://127.0.0.1:8765/v1/realtime",
            "http://localhost:4000",
            "  http://gw.internal  ",
        ] {
            assert_eq!(url_scheme(plaintext), INSECURE_SCHEME, "{plaintext:?}");
        }
    }

    #[test]
    fn a_plaintext_endpoint_builds_plaintext_candidates() {
        let candidates = candidate_urls(
            OpenAiDialect::Azure,
            INSECURE_SCHEME,
            "127.0.0.1:8765",
            "d",
            AZURE_REALTIME_API_VERSION,
        );
        assert_eq!(candidates.len(), 4);
        for url in &candidates {
            assert!(url.starts_with("ws://127.0.0.1:8765/"), "{url}");
        }
    }

    #[test]
    fn query_values_keep_unreserved_characters_literal() {
        // A hyphen inside `api-version` and a dot inside a model id must survive
        // as themselves — over-encoding them to %2D / %2E relies on the gateway
        // decoding before it routes.
        let url = candidate_urls(
            OpenAiDialect::Azure,
            SECURE_SCHEME,
            "r.openai.azure.com",
            "gpt-realtime-2.1-mini",
            AZURE_REALTIME_API_VERSION,
        )
        .remove(1);
        assert!(url.contains("deployment=gpt-realtime-2.1-mini"), "{url}");
        assert!(url.contains("api-version=2025-08-28"), "{url}");
        assert!(!url.contains("%2D") && !url.contains("%2E"), "{url}");
        assert_eq!(encode_query_value("a-b.c_d~e").to_string(), "a-b.c_d~e");
    }

    #[test]
    fn query_values_still_escape_genuinely_unsafe_characters() {
        let url = candidate_urls(
            OpenAiDialect::OpenAi,
            SECURE_SCHEME,
            OPENAI_REALTIME_HOST,
            "a b&c=d",
            AZURE_REALTIME_API_VERSION,
        )
        .remove(0);
        assert!(url.ends_with("model=a%20b%26c%3Dd"), "{url}");
        // A query value cannot smuggle a second parameter or a path segment.
        assert_eq!(encode_query_value("../x?y#z").to_string(), "..%2Fx%3Fy%23z");
        assert_eq!(encode_query_value("한").to_string(), "%ED%95%9C");
    }
}
