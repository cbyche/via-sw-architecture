//! The origin allow-list: VIA's outermost security boundary.
//!
//! Ported from `server/src/core/request-security.mjs`. It is applied as the
//! **first** middleware, before identity issuance and before body parsing, so it
//! covers every route including `/livez` and `/readyz`
//! (`docs/reference/contracts.json`, *middleware order and limits*).
//!
//! # What it defends against
//!
//! A Gateway listening on `127.0.0.1:3101` is reachable from any web page the
//! user happens to have open, because a browser will happily send a
//! cross-origin request to loopback. Two attacks follow.
//!
//! **Cross-origin scripting.** `https://attacker.example` fetches
//! `http://127.0.0.1:3101/api/tasks`. The `Origin` header names the attacker, so
//! it is refused.
//!
//! **DNS rebinding.** The attacker controls `evil.example`, answers with a TTL-0
//! record pointing at `127.0.0.1`, and now the page's *own* origin resolves to
//! the Gateway — `Origin` and `Host` agree, and a naive "same origin" check
//! passes. Upstream's answer, reproduced here, is that the implicit same-origin
//! path is restricted to **literal loopback hosts**: `evil.example` is a public
//! hostname, so it must be named in `VIA_ALLOWED_ORIGINS` before it is trusted,
//! whatever it resolves to.
//!
//! # The rules, in order
//!
//! | `Host` | `Origin` | Outcome |
//! | --- | --- | --- |
//! | unparseable | anything | refused |
//! | loopback | absent | **allowed** — CLI and other non-browser clients |
//! | allow-listed host | absent | allowed |
//! | anything else | absent | refused |
//! | any | allow-listed, host matches | allowed |
//! | any | allow-listed, host differs | refused |
//! | loopback | loopback, host matches exactly | allowed |
//! | anything else | present | refused |
//!
//! # Why `url` and not string splitting
//!
//! Every comparison above is over WHATWG-normalised values: `Origin:
//! https://x.example:443` serialises to `https://x.example` with the default
//! port dropped, `Host: 127.1` normalises to hostname `127.0.0.1`, and
//! `Host: [0:0:0:0:0:0:0:1]:80` normalises to host `[::1]`. A hand-rolled
//! splitter gets all three wrong, and each one is a hole.

use url::{Origin, Url};

use crate::env::comma_list;

/// The hostnames that count as loopback.
///
/// **External contract** — `server/src/core/request-security.mjs:3`. `::1`
/// without brackets can never come out of WHATWG host parsing, but the set is
/// reproduced verbatim rather than trimmed to what is reachable.
pub const LOOPBACK_HOSTS: &[&str] = &["127.0.0.1", "localhost", "[::1]", "::1"];

/// Status code for a refused origin.
///
/// **External contract** — `server/src/core/request-security.mjs:76`.
pub const ORIGIN_REJECTION_STATUS: u16 = 403;

/// Body of a refused origin, byte for byte.
///
/// **External contract** — `server/src/core/request-security.mjs:76`
/// (`res.status(403).json({ error: 'origin not allowed' })`).
pub const ORIGIN_REJECTION_BODY: &str = "{\"error\":\"origin not allowed\"}";

/// The `error` field inside [`ORIGIN_REJECTION_BODY`].
pub const ORIGIN_NOT_ALLOWED: &str = "origin not allowed";

/// Raw bytes written to a WebSocket socket when the origin is refused.
///
/// **External contract** — `server/src/voice/realtime-gateway.mjs:65-68`.
/// Written before `destroy()`, so clients parse the status line off the wire.
pub const UPGRADE_ORIGIN_REJECTION: &str = "HTTP/1.1 403 Forbidden\r\nConnection: close\r\nContent-Type: text/plain\r\n\r\norigin not allowed";

/// Raw bytes written to a WebSocket socket when no identity is presented.
///
/// **External contract** — `server/src/voice/realtime-gateway.mjs:161-169`.
pub const UPGRADE_IDENTITY_REJECTION: &str = "HTTP/1.1 401 Unauthorized\r\nConnection: close\r\nContent-Type: text/plain\r\n\r\nidentity required";

/// Whether a WHATWG hostname is one of [`LOOPBACK_HOSTS`].
#[must_use]
pub fn is_loopback_host(hostname: &str) -> bool {
    LOOPBACK_HOSTS.contains(&hostname)
}

/// A parsed `Origin` header.
#[derive(Clone, Debug, PartialEq, Eq)]
enum OriginValue {
    /// No header, or one that does not parse as a URL. Upstream's `''`.
    Absent,
    /// A URL with an opaque origin — `data:`, `blob:`, a sandboxed document.
    ///
    /// Upstream reaches `new URL('null')` here and **throws**, which Express
    /// turns into a 500. VIA refuses instead: a 500 and a 403 are both a
    /// denial, and a security boundary should not have a crash path.
    /// Recorded in `docs/deviations/phase-1.md`.
    Opaque,
    /// A tuple origin, ASCII-serialised (default port omitted).
    Tuple(String),
}

fn normalized_origin(value: Option<&str>) -> OriginValue {
    let Some(raw) = value.filter(|text| !text.is_empty()) else {
        return OriginValue::Absent;
    };
    let Ok(url) = Url::parse(raw) else {
        return OriginValue::Absent;
    };
    match url.origin() {
        Origin::Opaque(_) => OriginValue::Opaque,
        origin @ Origin::Tuple(..) => OriginValue::Tuple(origin.ascii_serialization()),
    }
}

/// A `Host` header split the way `new URL('http://' + host)` splits it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct HostParts {
    /// Host with its port, the port omitted when it is the scheme default.
    host: String,
    /// Host without its port; IPv6 keeps its brackets.
    hostname: String,
}

fn url_parts(url: &Url) -> Option<HostParts> {
    let hostname = url.host_str()?.to_owned();
    let host = match url.port() {
        Some(port) => format!("{hostname}:{port}"),
        None => hostname.clone(),
    };
    Some(HostParts { host, hostname })
}

fn parsed_host(value: Option<&str>) -> Option<HostParts> {
    let raw = value.filter(|text| !text.is_empty())?;
    let url = Url::parse(&format!("http://{raw}")).ok()?;
    url_parts(&url)
}

/// Filter a configured allow-list down to the entries that may be trusted.
///
/// **External contract** — `server/src/core/request-security.mjs:25-39`. An
/// entry survives when it parses as a URL **and** is either `https:` on any
/// host, or `http:` on a loopback host.
///
/// A plain-`http://` public entry is **silently dropped, not rejected**.
/// `docs/reference/contracts.json` calls this out by name: "a Rust port that
/// errors instead would break existing configs." Each surviving entry is
/// returned in its normalised origin form, which is what the comparisons below
/// use.
#[must_use]
pub fn trusted_origins(allowed_origins: &[String]) -> Vec<String> {
    allowed_origins
        .iter()
        .filter_map(|entry| match normalized_origin(Some(entry)) {
            OriginValue::Tuple(origin) => Some(origin),
            OriginValue::Absent | OriginValue::Opaque => None,
        })
        .filter(|origin| {
            Url::parse(origin).is_ok_and(|url| {
                url.scheme() == "https"
                    || (url.scheme() == "http" && url.host_str().is_some_and(is_loopback_host))
            })
        })
        .collect()
}

/// Parse `VIA_ALLOWED_ORIGINS` into the list [`is_allowed_origin`] takes.
///
/// **External contract** — `server/src/core/config.mjs:196-199`: comma
/// separated, each entry trimmed, empties dropped. The default is the **empty
/// list**, i.e. no cross-origin browser access at all. It is never `*`.
#[must_use]
pub fn parse_allowed_origins(value: Option<&str>) -> Vec<String> {
    comma_list(value)
}

/// Decide whether a request may proceed.
///
/// **External contract** — `server/src/core/request-security.mjs:41-72`. See
/// the module documentation for the full table.
#[must_use]
pub fn is_allowed_origin(
    host_header: Option<&str>,
    origin_header: Option<&str>,
    allowed_origins: &[String],
) -> bool {
    let Some(request_host) = parsed_host(host_header) else {
        return false;
    };
    let origin = normalized_origin(origin_header);
    let configured = trusted_origins(allowed_origins);
    let trusted_host = configured.iter().any(|entry| {
        Url::parse(entry)
            .ok()
            .and_then(|url| url_parts(&url))
            .is_some_and(|parts| parts.host == request_host.host)
    });

    let origin = match origin {
        // CLI and other non-browser clients send no Origin. They are accepted
        // only through a loopback address or an explicitly trusted proxy.
        OriginValue::Absent => return is_loopback_host(&request_host.hostname) || trusted_host,
        OriginValue::Opaque => return false,
        OriginValue::Tuple(origin) => origin,
    };

    let Some(origin_parts) = Url::parse(&origin).ok().and_then(|url| url_parts(&url)) else {
        return false;
    };

    if configured.contains(&origin) {
        return origin_parts.host == request_host.host;
    }

    // Comparing arbitrary Origin and Host values is vulnerable to DNS
    // rebinding, so the implicit same-origin path is limited to literal
    // loopback hosts. Public hostnames must be allow-listed.
    is_loopback_host(&request_host.hostname)
        && is_loopback_host(&origin_parts.hostname)
        && origin_parts.host == request_host.host
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allow(entries: &[&str]) -> Vec<String> {
        entries.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn loopback_without_an_origin_is_the_cli_path() {
        assert!(is_allowed_origin(Some("localhost:3101"), None, &[]));
        assert!(is_allowed_origin(
            Some("localhost:3101"),
            Some("http://localhost:3101"),
            &[]
        ));
        assert!(!is_allowed_origin(
            Some("localhost:3101"),
            Some("https://attacker.example"),
            &[]
        ));
    }

    #[test]
    fn a_lookalike_hostname_is_not_loopback() {
        assert!(!is_allowed_origin(
            Some("127.0.0.1.evil.com:3101"),
            None,
            &[]
        ));
        assert!(!is_allowed_origin(
            Some("127.0.0.1:3101"),
            Some("http://127.0.0.1.evil.com:3101"),
            &[]
        ));
    }

    #[test]
    fn an_http_public_allow_list_entry_is_dropped_not_rejected() {
        assert!(trusted_origins(&allow(&["http://voice.example.com"])).is_empty());
        assert_eq!(
            trusted_origins(&allow(&["https://voice.example.com", "not a url"])),
            vec!["https://voice.example.com".to_owned()]
        );
    }
}
