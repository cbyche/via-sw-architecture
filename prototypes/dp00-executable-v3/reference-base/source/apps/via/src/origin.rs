//! `--url` and `--backend-url`: parse, restrict the scheme, keep the origin.
//!
//! Ported from `cli/src/arguments.mjs:41-52`:
//!
//! ```js
//! function cleanOrigin(value, label) {
//!   let url
//!   try { url = new URL(value) } catch { throw new Error(`无效的${label}：${value}`) }
//!   if (!['http:', 'https:'].includes(url.protocol)) {
//!     throw new Error(`${label}只支持 http 或 https`)
//!   }
//!   return url.origin
//! }
//! ```
//!
//! Three details are contract rather than incidental, and each has a test:
//!
//! 1. **The path is discarded.** `https://voice.example.com/path` becomes
//!    `https://voice.example.com`, which is what upstream's own test asserts
//!    (`cli/test/arguments.test.mjs:35`).
//! 2. **The default port is dropped.** `new URL('http://h:80/').origin` is
//!    `http://h`, and the lease's `origin` field is compared as a string — so
//!    a hand-rolled `scheme://host:port` formatter stops matching a running
//!    Gateway. `url::Url::origin()` is the same WHATWG algorithm `via-core`'s
//!    allow-list uses.
//! 3. **The Gateway label carries a leading space.** `' Gateway URL'`
//!    (`cli/src/arguments.mjs:259`), so the rendered sentence is
//!    `无效的 Gateway URL：<value>`. The catalogue calls this out explicitly:
//!    *"note the leading space baked into the Gateway label"*. It lives in the
//!    `via-i18n` value, not in a `format!` here.

use url::Url;

use via_i18n::{Key, Locale, keys, t};

use crate::error::{CODE_INVALID_ARGUMENT, CliError};

/// The two schemes an origin may use.
///
/// **External contract** — `cli/src/arguments.mjs:48`.
pub const ALLOWED_SCHEMES: [&str; 2] = ["http", "https"];

/// The Gateway address used when neither `--url` nor `VIA_URL` says otherwise.
///
/// **External contract** — `cli/src/arguments.mjs:103`
/// (`env.<PREFIX>_URL || 'http://127.0.0.1:3101'`). It agrees with
/// `via-core`'s `HOST`/`PORT` defaults by construction, which
/// `tests/contracts.rs` asserts rather than assumes.
pub const DEFAULT_GATEWAY_URL: &str = "http://127.0.0.1:3101";

/// Which address is being parsed — it selects the label the message names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlLabel {
    /// `--url`, labelled `" Gateway URL"` — leading space included.
    Gateway,
    /// `--backend-url`, labelled `"backend address"`.
    Backend,
}

impl UrlLabel {
    /// The `via-i18n` key carrying this label.
    #[must_use]
    pub const fn key(self) -> Key {
        match self {
            Self::Gateway => keys::CLI_LABEL_GATEWAY_URL,
            Self::Backend => keys::CLI_LABEL_BACKEND_URL,
        }
    }

    /// The label, rendered in `locale`.
    #[must_use]
    pub fn text(self, locale: Locale) -> &'static str {
        t(locale, self.key())
    }
}

/// Parse `value` and return its WHATWG origin.
///
/// # Errors
///
/// [`CliError::Refused`] with [`CODE_INVALID_ARGUMENT`] when `value` is not a
/// URL at all, or when its scheme is neither `http` nor `https`. The two cases
/// are distinguished by their message, exactly as upstream distinguishes them.
pub fn clean_origin(value: &str, label: UrlLabel, locale: Locale) -> Result<String, CliError> {
    let Ok(url) = Url::parse(value) else {
        return Err(CliError::refused_with(
            CODE_INVALID_ARGUMENT,
            locale,
            keys::CLI_INVALID_URL,
            &[("label", label.text(locale)), ("value", value)],
        ));
    };
    if !ALLOWED_SCHEMES.contains(&url.scheme()) {
        return Err(CliError::refused_with(
            CODE_INVALID_ARGUMENT,
            locale,
            keys::CLI_URL_SCHEME_UNSUPPORTED,
            &[("label", label.text(locale))],
        ));
    }
    Ok(url.origin().ascii_serialization())
}

/// The `HOST` and `PORT` a Gateway origin implies.
///
/// **External contract** — `docs/reference/contracts.json` *env-var/gateway
/// child spawn environment*, `cli/src/runtime.mjs:294-316`:
/// `HOST=<listenHost || hostname, 'localhost' → '127.0.0.1'>` and
/// `PORT=String(listenPort || target.port || '80')`.
///
/// The `localhost` rewrite is not cosmetic. A Gateway that *binds* `localhost`
/// binds whatever the resolver returns first, which on a dual-stack host is
/// often `::1`; a client that then probes `127.0.0.1` finds nothing listening.
/// Upstream pins the literal, and so does this.
///
/// Returns `None` when the URL has no host — which [`clean_origin`] has
/// already excluded for `http`/`https`, so this is the unreachable-by-
/// construction arm rather than a case a caller must handle thoughtfully.
#[must_use]
pub fn listen_address(origin: &str) -> Option<(String, u16)> {
    let url = Url::parse(origin).ok()?;
    let host = url.host_str()?;
    let host = if host == "localhost" {
        "127.0.0.1"
    } else {
        host
    };
    // `Url::port()` is `None` for a default port, and upstream's
    // `target.port` is likewise the empty string there — hence the `|| '80'`.
    Some((host.to_owned(), url.port().unwrap_or(80)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("http://127.0.0.1:3101", "http://127.0.0.1:3101")]
    #[case("https://voice.example.com/path", "https://voice.example.com")]
    #[case("http://localhost:18888/path", "http://localhost:18888")]
    // The default port is dropped, both ways round.
    #[case("http://example.com:80/", "http://example.com")]
    #[case("https://example.com:443", "https://example.com")]
    #[case("http://example.com", "http://example.com")]
    // A query and a fragment are not part of an origin either.
    #[case("http://example.com:9/a?b=c#d", "http://example.com:9")]
    // WHATWG IPv4 shorthand and IPv6 bracket form, which a string splitter
    // gets wrong in both directions.
    #[case("http://127.1:3101", "http://127.0.0.1:3101")]
    #[case("http://[0:0:0:0:0:0:0:1]:3101", "http://[::1]:3101")]
    fn an_origin_is_scheme_host_and_non_default_port(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(
            clean_origin(input, UrlLabel::Gateway, Locale::En).expect("a valid http(s) URL"),
            expected
        );
    }

    #[rstest]
    #[case("")]
    #[case("127.0.0.1:3101")]
    #[case("not a url")]
    #[case("://missing-scheme")]
    fn an_unparseable_value_is_named_in_the_refusal(#[case] input: &str) {
        let error =
            clean_origin(input, UrlLabel::Gateway, Locale::Zh).expect_err("not a URL at all");
        assert_eq!(error.code(), CODE_INVALID_ARGUMENT);
        let message = error.message(Locale::Zh);
        assert_eq!(message, std::format!("无效的 Gateway URL：{input}"));
    }

    #[rstest]
    #[case("ws://127.0.0.1:3101")]
    #[case("wss://127.0.0.1:3101")]
    #[case("file:///etc/passwd")]
    #[case("data:text/plain,x")]
    #[case("javascript:alert(1)")]
    fn a_non_http_scheme_is_refused_with_the_scheme_message(#[case] input: &str) {
        let error = clean_origin(input, UrlLabel::Gateway, Locale::Zh)
            .expect_err("only http and https are origins VIA speaks");
        assert_eq!(
            error.message(Locale::Zh),
            " Gateway URL只支持 http 或 https"
        );
    }

    #[test]
    fn the_backend_label_is_a_different_sentence() {
        let error = clean_origin("nope", UrlLabel::Backend, Locale::Zh).expect_err("not a URL");
        assert_eq!(error.message(Locale::Zh), "无效的后台地址：nope");
    }

    #[test]
    fn the_gateway_label_keeps_its_leading_space_in_all_three_locales() {
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            assert!(
                UrlLabel::Gateway.text(locale).starts_with(' '),
                "{locale} lost the leading space upstream bakes into the label"
            );
        }
    }

    #[rstest]
    #[case("http://127.0.0.1:3101", "127.0.0.1", 3101)]
    #[case("http://localhost:18888", "127.0.0.1", 18888)]
    #[case("http://example.com", "example.com", 80)]
    // 80, not 443: upstream's `String(listenPort || target.port || '80')`
    // reads `.port` as the empty string whenever it is the scheme's default,
    // so a default-port https origin lands on 80 there too. Reproduced rather
    // than corrected — the value is what a phase-5 child will be handed.
    #[case("https://example.com", "example.com", 80)]
    #[case("https://example.com:8443", "example.com", 8443)]
    #[case("http://[::1]:3101", "[::1]", 3101)]
    fn a_listen_address_follows_the_child_spawn_rule(
        #[case] origin: &str,
        #[case] host: &str,
        #[case] port: u16,
    ) {
        assert_eq!(listen_address(origin), Some((host.to_owned(), port)));
    }

    #[test]
    fn the_default_gateway_url_is_a_valid_origin_of_itself() {
        assert_eq!(
            clean_origin(DEFAULT_GATEWAY_URL, UrlLabel::Gateway, Locale::En)
                .expect("the default must survive its own normalisation"),
            DEFAULT_GATEWAY_URL
        );
    }
}
