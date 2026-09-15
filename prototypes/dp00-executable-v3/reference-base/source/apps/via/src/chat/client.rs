//! The HTTP half of `via chat`.
//!
//! Three requests and one cookie. `text-cli.mjs:78-94` is the whole of it:
//!
//! ```js
//! const healthResponse = await fetch(`${options.url}/api/health`)
//! const cookie = cookieFrom(healthResponse)
//! …
//! const api = async (path, init = {}) => {
//!   const response = await fetch(`${options.url}${path}`, {
//!     ...init, headers: { ...headers, ...init.headers },
//!   })
//!   const payload = await response.json()
//!   if (!response.ok) throw new Error(payload.error || `请求失败 (${response.status})`)
//!   return payload
//! }
//! ```
//!
//! # The cookie is the identity
//!
//! `/api/health` is the one route that **issues** the signed identity cookie;
//! the WebSocket upgrade only ever *resolves* one and answers 401 otherwise
//! (`docs/deviations/phase-5-via-app.md`). So the health call is not a
//! readiness check that happens to come first — it is how a client gets an
//! owner id at all, and the cookie has to be carried onto the socket and onto
//! every later request or they belong to nobody.
//!
//! Only the `name=value` pair is kept, which is
//! `raw.split(';', 1)[0]` — the attributes are the server's instructions to a
//! browser, and echoing `HttpOnly` back on a request header would be nonsense.

use std::time::Duration;

use via_i18n::{Locale, format, keys, t};
use via_work::PublicWork;

use crate::error::{CODE_INVALID_ARGUMENT, CliError};

/// How long a health probe waits before deciding nothing is there.
///
/// **External contract** — `shared/gateway-client.mjs:4`:
/// `AbortSignal.timeout(1500)`.
pub const HEALTH_TIMEOUT: Duration = Duration::from_millis(1500);

/// The header a cookie is offered on.
pub const SET_COOKIE: &str = "set-cookie";

/// Whether an `/api/health` payload identifies a VIA Gateway.
///
/// **External contract** — `shared/gateway-client.mjs:9`'s
/// `payload && typeof payload === 'object' && payload.backend`: JavaScript
/// truthiness on the `backend` field, not mere presence. `null`, `false`, `0`
/// and `""` are all falsy in JavaScript — an absent field reads the same way
/// `serde_json::Value::get` reads a missing key, as `None` — so a service that
/// happens to answer `{"backend": null}` on the same port is not mistaken for
/// a Gateway, which is the whole point of *http-route/health probe path and
/// identification gate*: *"distinguish a VIA gateway from an unrelated service
/// on the same port before reusing an instance."*
#[must_use]
pub fn identifies_a_gateway(payload: &serde_json::Value) -> bool {
    match payload.get("backend") {
        None | Some(serde_json::Value::Null) => false,
        Some(serde_json::Value::Bool(flag)) => *flag,
        Some(serde_json::Value::Number(number)) => number.as_f64().is_none_or(|value| value != 0.0),
        Some(serde_json::Value::String(text)) => !text.is_empty(),
        // An object or an array is truthy in JavaScript even when empty —
        // `{}` and `[]` are both `Boolean(x) === true`.
        Some(_) => true,
    }
}

/// A Gateway, as `via chat` talks to it.
#[derive(Debug, Clone)]
pub struct GatewayClient {
    http: reqwest::Client,
    origin: String,
    cookie: Option<String>,
    locale: Locale,
}

impl GatewayClient {
    /// A client for `origin`.
    ///
    /// # Errors
    ///
    /// [`CliError::Refused`] when the HTTP stack will not build, which on a
    /// rustls build means the platform has no usable certificate store.
    pub fn new(origin: &str, locale: Locale) -> Result<Self, CliError> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|error| Self::refused(locale, &error.to_string()))?;
        Ok(Self {
            http,
            origin: origin.trim_end_matches('/').to_owned(),
            cookie: None,
            locale,
        })
    }

    /// The identity cookie this client carries, once `/api/health` has issued
    /// one.
    #[must_use]
    pub fn cookie(&self) -> Option<&str> {
        self.cookie.as_deref()
    }

    /// The Gateway origin.
    #[must_use]
    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// `GET /api/health`, keeping the cookie it issues.
    ///
    /// `None` means *nothing answered* — the shape
    /// `shared/gateway-client.mjs` returns from its `catch`, and the signal
    /// that a Gateway has to be started. A Gateway that answered but refused is
    /// an `Err`, because a user can act on that.
    ///
    /// # Errors
    ///
    /// [`CliError::Refused`] when the Gateway answered a non-2xx, carrying
    /// `backend.error` when it named one and the catalogued
    /// *not ready* sentence otherwise.
    pub async fn health(&mut self) -> Result<Option<serde_json::Value>, CliError> {
        let response = match self
            .http
            .get(std::format!("{}/api/health", self.origin))
            .timeout(HEALTH_TIMEOUT)
            .send()
            .await
        {
            Ok(response) => response,
            Err(_) => return Ok(None),
        };
        // `getSetCookie()[0] || get('set-cookie') || ''`, then
        // `split(';', 1)[0]`.
        if let Some(raw) = response
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .find_map(|value| value.to_str().ok())
        {
            let pair = raw.split(';').next().unwrap_or_default().trim();
            if !pair.is_empty() {
                self.cookie = Some(pair.to_owned());
            }
        }
        let ok = response.status().is_success();
        let payload: serde_json::Value = response.json().await.unwrap_or(serde_json::Value::Null);
        if ok {
            // `payload && typeof payload === 'object' && payload.backend`.
            return Ok(identifies_a_gateway(&payload).then_some(payload));
        }
        Err(Self::refused(
            self.locale,
            payload
                .get("backend")
                .and_then(|backend| backend.get("error"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| t(self.locale, keys::CHAT_NOT_READY)),
        ))
    }

    /// `GET /api/tasks?sessionId=…`.
    ///
    /// # Errors
    ///
    /// [`CliError::Refused`], as [`Self::request`].
    pub async fn tasks(&self, session_id: &str) -> Result<Vec<PublicWork>, CliError> {
        let path = std::format!(
            "/api/tasks?sessionId={}",
            url::form_urlencoded::byte_serialize(session_id.as_bytes()).collect::<String>(),
        );
        let payload = self.request(reqwest::Method::GET, &path).await?;
        Ok(payload
            .get("tasks")
            .and_then(|tasks| serde_json::from_value(tasks.clone()).ok())
            .unwrap_or_default())
    }

    /// `DELETE /api/tasks/{id}`.
    ///
    /// # Errors
    ///
    /// [`CliError::Refused`], as [`Self::request`].
    pub async fn cancel(&self, work_id: &str) -> Result<serde_json::Value, CliError> {
        let path = std::format!(
            "/api/tasks/{}",
            url::form_urlencoded::byte_serialize(work_id.as_bytes()).collect::<String>(),
        );
        self.request(reqwest::Method::DELETE, &path).await
    }

    /// One request, with the cookie and upstream's refusal shape.
    ///
    /// # Errors
    ///
    /// [`CliError::Refused`] carrying the body's `error` when it has one, and
    /// the catalogued `request failed ({status})` otherwise. A transport
    /// failure carries its own sentence, which is what upstream's `fetch`
    /// rejection would have surfaced.
    pub async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
    ) -> Result<serde_json::Value, CliError> {
        let mut request = self
            .http
            .request(method, std::format!("{}{path}", self.origin));
        if let Some(cookie) = &self.cookie {
            request = request.header(reqwest::header::COOKIE, cookie);
        }
        let response = request
            .send()
            .await
            .map_err(|error| Self::refused(self.locale, &error.to_string()))?;
        let status = response.status();
        let payload: serde_json::Value = response.json().await.unwrap_or(serde_json::Value::Null);
        if status.is_success() {
            return Ok(payload);
        }
        Err(Self::refused(
            self.locale,
            payload
                .get("error")
                .and_then(serde_json::Value::as_str)
                .map_or_else(
                    || {
                        format(
                            self.locale,
                            keys::CHAT_REQUEST_FAILED,
                            &[("status", &status.as_u16().to_string())],
                        )
                    },
                    str::to_owned,
                )
                .as_str(),
        ))
    }

    fn refused(locale: Locale, message: &str) -> CliError {
        let _ = locale;
        CliError::Refused {
            code: CODE_INVALID_ARGUMENT,
            message: message.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn the_health_timeout_is_the_catalogued_one_and_a_half_seconds() {
        assert_eq!(HEALTH_TIMEOUT, Duration::from_millis(1500));
    }

    #[test]
    fn a_trailing_slash_is_stripped_so_paths_do_not_double_up() {
        let client = GatewayClient::new("http://127.0.0.1:3101///", Locale::En).expect("builds");
        assert_eq!(client.origin(), "http://127.0.0.1:3101");
    }

    #[test]
    fn a_client_carries_no_cookie_until_health_issues_one() {
        let client = GatewayClient::new("http://127.0.0.1:3101", Locale::En).expect("builds");
        assert_eq!(client.cookie(), None);
    }

    #[test]
    fn only_a_truthy_backend_field_identifies_a_gateway() {
        // Truthy `backend` values, of every JSON shape that can carry one —
        // including an empty object, which is truthy in JavaScript.
        for payload in [
            serde_json::json!({ "backend": { "ok": true } }),
            serde_json::json!({ "backend": {} }),
            serde_json::json!({ "backend": [] }),
            serde_json::json!({ "backend": true }),
            serde_json::json!({ "backend": 1 }),
            serde_json::json!({ "backend": "opencode" }),
        ] {
            assert!(identifies_a_gateway(&payload), "{payload}");
        }
        // Falsy `backend` values, and no `backend` field at all, are every one
        // of them "not a Gateway" — including `null`, which mere key
        // *presence* would have missed.
        for payload in [
            serde_json::json!({ "backend": null }),
            serde_json::json!({ "backend": false }),
            serde_json::json!({ "backend": 0 }),
            serde_json::json!({ "backend": "" }),
            serde_json::json!({ "ok": true }),
            serde_json::json!({}),
        ] {
            assert!(!identifies_a_gateway(&payload), "{payload}");
        }
    }

    #[tokio::test]
    async fn health_queries_get_api_health_with_the_catalogued_timeout() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        // **External contract** — `http-route/gateway health probe`:
        // `` GET `${baseUrl}/api/health` `` — asserted here at the byte level,
        // against a raw socket, rather than against whatever path a mock HTTP
        // client library happens to record.
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("an ephemeral port");
        let origin = std::format!("http://{}", listener.local_addr().expect("bound"));

        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("a connection");
            let mut buffer = [0u8; 1024];
            let read = socket.read(&mut buffer).await.expect("read the request");
            let request = String::from_utf8_lossy(&buffer[..read]).into_owned();
            let body = br#"{"ok":true,"backend":{"enabled":true}}"#;
            let response = std::format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n",
                body.len()
            );
            socket
                .write_all(response.as_bytes())
                .await
                .expect("write the status line");
            socket.write_all(body).await.expect("write the body");
            request
        });

        let mut client = GatewayClient::new(&origin, Locale::En).expect("builds");
        let payload = client
            .health()
            .await
            .expect("no transport error")
            .expect("a truthy `backend` field identifies a Gateway");
        assert_eq!(payload["backend"]["enabled"], true);

        let request = server.await.expect("the server task completed");
        assert!(
            request.starts_with("GET /api/health "),
            "the wrong path or method was queried: {request}",
        );
    }

    #[tokio::test]
    async fn health_returns_none_rather_than_mistaking_an_unrelated_service_for_a_gateway() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        // The other half of the identification gate: a service on the same
        // port that answers 200 with no `backend` field is not a Gateway.
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("an ephemeral port");
        let origin = std::format!("http://{}", listener.local_addr().expect("bound"));

        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("a connection");
            let mut buffer = [0u8; 1024];
            let _ = socket.read(&mut buffer).await.expect("read the request");
            let body = b"{\"ok\":true}";
            let response = std::format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n",
                body.len()
            );
            socket
                .write_all(response.as_bytes())
                .await
                .expect("write the status line");
            socket.write_all(body).await.expect("write the body");
        });

        let mut client = GatewayClient::new(&origin, Locale::En).expect("builds");
        let identified = client.health().await.expect("no transport error");
        assert_eq!(
            identified, None,
            "a 200 with no `backend` field must not be mistaken for a Gateway",
        );
        server.await.expect("the server task completed");
    }
}
