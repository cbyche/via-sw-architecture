//! A raw HTTP/1.1 client, so the transport tests see the bytes.
//!
//! Deliberately not a library client. The contract this crate is judged
//! against is *"404 with an **empty body**"*, *"405 with `Allow: POST`"* and
//! *"the token never appears in the URL"* — statements about a response, not
//! about what a convenience wrapper made of one. A client that helpfully
//! retried, followed a redirect or synthesised a body would hide exactly the
//! thing under test.

use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// A parsed HTTP response.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// The status code.
    pub status: u16,
    /// Every header, lower-cased name, in order.
    pub headers: Vec<(String, String)>,
    /// The body, as text.
    pub body: String,
}

impl HttpResponse {
    /// The first value of `name`, lower-cased comparison.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// The body parsed as JSON.
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body)
            .unwrap_or_else(|error| panic!("body is not JSON ({error}): {:?}", self.body))
    }

    /// Every `data:` payload of an `event: message` SSE frame, parsed.
    pub fn sse_messages(&self) -> Vec<serde_json::Value> {
        self.body
            .lines()
            .filter_map(|line| line.strip_prefix("data: "))
            .filter_map(|data| serde_json::from_str(data).ok())
            .collect()
    }
}

/// Send one raw request and read the whole response.
///
/// `Connection: close` is appended so the server ends the stream and the body
/// can be read to EOF without trusting a `Content-Length` the test is also
/// trying to verify.
pub async fn send(
    address: SocketAddr,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> HttpResponse {
    let mut request =
        format!("{method} {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n");
    for (name, value) in headers {
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    if let Some(body) = body {
        request.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    request.push_str("\r\n");
    if let Some(body) = body {
        request.push_str(body);
    }

    let mut stream = TcpStream::connect(address)
        .await
        .unwrap_or_else(|error| panic!("connect to {address}: {error}"));
    stream
        .write_all(request.as_bytes())
        .await
        .expect("write the request");
    stream.flush().await.expect("flush");

    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .await
        .expect("read the response");
    parse(&String::from_utf8_lossy(&raw))
}

/// A `POST /mcp` carrying a JSON-RPC message.
pub async fn post_mcp(address: SocketAddr, token: Option<&str>, body: &str) -> HttpResponse {
    let authorization = token.map(|token| format!("Bearer {token}"));
    let mut headers: Vec<(&str, &str)> = vec![("Content-Type", "application/json")];
    if let Some(authorization) = authorization.as_deref() {
        headers.push(("Authorization", authorization));
    }
    send(address, "POST", "/mcp", &headers, Some(body)).await
}

fn parse(raw: &str) -> HttpResponse {
    let (head, body) = raw
        .split_once("\r\n\r\n")
        .unwrap_or_else(|| panic!("no header terminator in {raw:?}"));
    let mut lines = head.split("\r\n");
    let status_line = lines.next().unwrap_or_default();
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or_else(|| panic!("no status code in {status_line:?}"));
    let headers = lines
        .filter_map(|line| line.split_once(": "))
        .map(|(name, value)| (name.to_ascii_lowercase(), value.to_owned()))
        .collect();
    HttpResponse {
        status,
        headers,
        body: body.to_owned(),
    }
}
