//! Service addresses: normalisation, default ports, and the loopback test.
//!
//! Ported from `server/src/process/backend-drivers/shared.mjs:1-18` and
//! `server/src/process/managed-backend.mjs:11,71-73`.
//!
//! Every function here is WHATWG `new URL(...)` semantics, which is why the
//! `url` crate is a dependency rather than a `split(':')`: `origin`
//! serialization strips a default port, `hostname` renders IPv6 in bracket
//! form, and the special-scheme table is what makes `wss:` default to 443.
//!
//! # Two refusals that are security, not tidiness
//!
//! A scheme the driver does not admit, and a URL carrying a username or
//! password, are both hard errors. The second is the interesting one:
//! `docs/reference/contracts.json` (`error-code/service endpoint
//! rejections`) records the reason as *"Credentials in a URL would leak into
//! logs and process argv"* — the address is written into the child's
//! environment and echoed by `backend.process_started`, so a password inside
//! it is a password on disk.

use url::Url;

use crate::error::ProcessError;

/// Hosts the Gateway will launch a backend on.
///
/// **External contract** — `server/src/process/managed-backend.mjs:11`. The
/// set is compared against the WHATWG `hostname`, which renders IPv6 in
/// bracket form; both spellings are listed because upstream lists both.
pub const LOOPBACK_HOSTS: &[&str] = &["127.0.0.1", "localhost", "::1", "[::1]"];

/// The schemes `normalizeServiceEndpoint` admits when a driver names none.
///
/// **External contract** — `server/src/process/backend-drivers/shared.mjs:2`.
pub const DEFAULT_SERVICE_PROTOCOLS: &[&str] = &["http:", "https:", "ws:", "wss:"];

/// Parse a configured service address.
///
/// Split out of [`normalize_service_endpoint`] so the port and loopback
/// helpers can reuse it without re-deriving the error.
///
/// # Errors
///
/// [`ProcessError::InvalidServiceUrl`] when the value is not a URL. Upstream
/// lets `new URL`'s `TypeError` escape; see the variant's own documentation.
pub fn parse_service_endpoint(value: &str) -> Result<Url, ProcessError> {
    Url::parse(value).map_err(|_| ProcessError::InvalidServiceUrl {
        value: value.to_owned(),
    })
}

/// Reduce a configured service address to its origin, refusing what a driver
/// may not speak.
///
/// **External contract** — `server/src/process/backend-drivers/shared.mjs:1-14`.
/// Returns `target.origin`, so a path, query or fragment is dropped:
/// `http://localhost:4096/path` becomes `http://localhost:4096`. That is
/// deliberate and catalogued — `env-var/OPENCODE_BASE_URL` records that
/// `config.mjs` keeps the full URL while the driver reduces it to the origin,
/// "a divergence the port must preserve".
///
/// # Errors
///
/// - [`ProcessError::InvalidServiceUrl`] — not a URL.
/// - [`ProcessError::UnsupportedServiceProtocol`] — a scheme outside
///   `protocols`. The reported value keeps its trailing colon, matching
///   `new URL(x).protocol`.
/// - [`ProcessError::ServiceUrlHasCredentials`] — a username or password is
///   present.
pub fn normalize_service_endpoint(value: &str, protocols: &[&str]) -> Result<String, ProcessError> {
    let target = parse_service_endpoint(value)?;
    let protocol = format!("{}:", target.scheme());
    if !protocols.contains(&protocol.as_str()) {
        return Err(ProcessError::UnsupportedServiceProtocol { protocol });
    }
    // `username()` is `""` and `password()` is `None` when absent, exactly as
    // upstream's falsy test reads them.
    if !target.username().is_empty() || target.password().is_some_and(|value| !value.is_empty()) {
        return Err(ProcessError::ServiceUrlHasCredentials);
    }
    Ok(origin_of(&target))
}

/// The port a service address resolves to.
///
/// **External contract** — `server/src/process/backend-drivers/shared.mjs:16-19`.
/// An explicit port wins; otherwise `https:` and `wss:` are 443 and everything
/// else is 80. The `url` crate strips a port equal to the scheme's default, so
/// `http://host:80` and `http://host` both answer 80 — which is what
/// `new URL(...).port === ''` does too.
///
/// # Errors
///
/// [`ProcessError::InvalidServiceUrl`] when the value is not a URL.
pub fn service_endpoint_port(value: &str) -> Result<u16, ProcessError> {
    let target = parse_service_endpoint(value)?;
    Ok(port_of(&target))
}

/// Whether the Gateway may launch a backend at this address.
///
/// **External contract** — `server/src/process/managed-backend.mjs:71-73`.
/// The test is on the WHATWG `hostname` against [`LOOPBACK_HOSTS`], so it is
/// exact-match and not a prefix or a "starts with 127." heuristic: `127.1`
/// and `127.0.0.2` are both refused, as upstream refuses them.
///
/// # Errors
///
/// [`ProcessError::InvalidServiceUrl`] when the value is not a URL.
pub fn is_local_backend(base_url: &str) -> Result<bool, ProcessError> {
    let target = parse_service_endpoint(base_url)?;
    Ok(LOOPBACK_HOSTS.contains(&hostname_of(&target).as_str()))
}

/// The WHATWG `hostname`: no port, IPv6 in brackets.
pub(crate) fn hostname_of(target: &Url) -> String {
    target.host_str().unwrap_or_default().to_owned()
}

/// The WHATWG `origin`, serialized.
///
/// `Origin::ascii_serialization` is the same algorithm `new URL(...).origin`
/// runs, default-port stripping included, and it answers the literal `"null"`
/// for an opaque origin just as JavaScript does. Every scheme a driver admits
/// is a special scheme, so the opaque arm is not reachable from
/// [`normalize_service_endpoint`].
pub(crate) fn origin_of(target: &Url) -> String {
    target.origin().ascii_serialization()
}

/// The port half of [`service_endpoint_port`], on a parsed URL.
pub(crate) fn port_of(target: &Url) -> u16 {
    if let Some(port) = target.port() {
        return port;
    }
    match target.scheme() {
        "https" | "wss" => 443,
        _ => 80,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduces_a_configured_address_to_its_origin() {
        assert_eq!(
            normalize_service_endpoint("http://localhost:4096/path", &["http:", "https:"]).unwrap(),
            "http://localhost:4096"
        );
    }

    #[test]
    fn keeps_a_default_port_out_of_the_origin() {
        assert_eq!(
            normalize_service_endpoint(
                "wss://agent.example.com/gateway",
                DEFAULT_SERVICE_PROTOCOLS
            )
            .unwrap(),
            "wss://agent.example.com"
        );
    }

    #[test]
    fn refuses_a_scheme_the_driver_does_not_admit() {
        let error = normalize_service_endpoint("wss://agent.example.com", &["http:", "https:"])
            .unwrap_err();
        assert!(matches!(
            error,
            ProcessError::UnsupportedServiceProtocol { ref protocol } if protocol == "wss:"
        ));
    }

    #[test]
    fn refuses_credentials_in_the_address() {
        assert!(matches!(
            normalize_service_endpoint(
                "wss://user:secret@agent.example.com",
                DEFAULT_SERVICE_PROTOCOLS
            )
            .unwrap_err(),
            ProcessError::ServiceUrlHasCredentials
        ));
        assert!(matches!(
            normalize_service_endpoint("wss://user@agent.example.com", DEFAULT_SERVICE_PROTOCOLS)
                .unwrap_err(),
            ProcessError::ServiceUrlHasCredentials
        ));
    }

    #[test]
    fn default_ports_follow_the_scheme() {
        assert_eq!(
            service_endpoint_port("http://127.0.0.1:18789").unwrap(),
            18789
        );
        assert_eq!(service_endpoint_port("http://127.0.0.1").unwrap(), 80);
        assert_eq!(service_endpoint_port("http://127.0.0.1:80").unwrap(), 80);
        assert_eq!(service_endpoint_port("https://example.com").unwrap(), 443);
        assert_eq!(service_endpoint_port("wss://example.com").unwrap(), 443);
        assert_eq!(service_endpoint_port("ws://example.com").unwrap(), 80);
    }

    #[test]
    fn only_the_four_loopback_spellings_are_local() {
        for host in [
            "http://127.0.0.1:1",
            "http://localhost:1",
            "http://[::1]:1",
            // WHATWG normalises IPv4 shorthand before `hostname` is read, so
            // this reaches the set as `127.0.0.1` — in JavaScript too.
            "http://127.1:1",
        ] {
            assert!(is_local_backend(host).unwrap(), "{host}");
        }
        for host in [
            "http://127.0.0.2:1",
            "http://example.com",
            "http://localhost.evil.example:1",
            "http://0.0.0.0:1",
        ] {
            assert!(!is_local_backend(host).unwrap(), "{host}");
        }
    }
}
