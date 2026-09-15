//! The extractor's one HTTP call, against a real socket.
//!
//! Ported from the `createExtractorLlmCall` half of
//! `server/test/memory-extractor.test.mjs`. The catalogued wire contract is
//! `POST {base_url}/chat/completions` with a `Bearer` credential and
//! `temperature: 0`, so it is asserted by reading the bytes off a listener
//! rather than by inspecting the client.

#![cfg(feature = "http")]

use std::time::Duration;

use pretty_assertions::assert_eq;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use via_conversation::extractor::{
    CHAT_COMPLETIONS_PATH, ExtractorError, ExtractorLlm, HttpExtractorLlm, create_extractor_llm,
};

/// One request, then one canned response. Returns the raw request bytes.
async fn one_shot(listener: TcpListener, status: u16, body: &'static str) -> String {
    let (mut socket, _) = listener.accept().await.expect("a connection");
    let mut request = Vec::new();
    let mut buffer = [0u8; 4096];
    loop {
        let read = socket.read(&mut buffer).await.expect("readable");
        if read == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..read]);
        let text = String::from_utf8_lossy(&request);
        // Stop once the whole body has arrived.
        if let Some((head, rest)) = text.split_once("\r\n\r\n") {
            let length: usize = head
                .lines()
                .find_map(|line| {
                    line.strip_prefix("content-length: ")
                        .or_else(|| line.strip_prefix("Content-Length: "))
                })
                .and_then(|value| value.trim().parse().ok())
                .unwrap_or(0);
            if rest.len() >= length {
                break;
            }
        }
    }
    let response = format!(
        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    socket
        .write_all(response.as_bytes())
        .await
        .expect("writable");
    socket.flush().await.expect("flushed");
    String::from_utf8_lossy(&request).into_owned()
}

#[test]
fn a_client_with_nothing_to_talk_to_is_never_built() {
    let timeout = Duration::from_secs(10);
    assert!(create_extractor_llm("https://example.com/v1", "", "qwen-flash", timeout).is_none());
    assert!(create_extractor_llm("", "key", "qwen-flash", timeout).is_none());
    assert!(create_extractor_llm("https://example.com/v1", "key", "", timeout).is_none());
    assert!(create_extractor_llm("https://example.com/v1", "key", "qwen-flash", timeout).is_some());
}

#[test]
fn the_endpoint_is_the_base_url_plus_the_catalogued_path() {
    let client = HttpExtractorLlm::new(
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        "key",
        "qwen-flash",
        Duration::from_secs(10),
    )
    .expect("a client");
    assert_eq!(
        client.endpoint(),
        format!("https://dashscope.aliyuncs.com/compatible-mode/v1{CHAT_COMPLETIONS_PATH}")
    );
}

#[tokio::test]
async fn the_request_carries_the_catalogued_method_path_headers_and_body() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bound");
    let address = listener.local_addr().expect("an address");
    let server = tokio::spawn(one_shot(
        listener,
        200,
        r#"{"choices":[{"message":{"content":"{\"changes\":[]}"}}]}"#,
    ));

    let client = HttpExtractorLlm::new(
        &format!("http://{address}/v1"),
        "test-key",
        "qwen-flash",
        Duration::from_secs(10),
    )
    .expect("a client");
    let answer = client.complete("SYSTEM", "USER").await.expect("answered");
    assert_eq!(answer, r#"{"changes":[]}"#);

    let request = server.await.expect("the server finished");
    let (head, body) = request.split_once("\r\n\r\n").expect("a request body");
    assert!(
        head.starts_with(&format!("POST /v1{CHAT_COMPLETIONS_PATH} HTTP/1.1")),
        "{head}"
    );
    assert!(
        head.to_lowercase()
            .contains("authorization: bearer test-key"),
        "{head}"
    );
    assert!(
        head.to_lowercase()
            .contains("content-type: application/json"),
        "{head}"
    );

    let payload: serde_json::Value = serde_json::from_str(body).expect("a JSON body");
    assert_eq!(payload["model"], serde_json::json!("qwen-flash"));
    assert_eq!(payload["temperature"], serde_json::json!(0.0));
    assert_eq!(
        payload["messages"],
        serde_json::json!([
            { "role": "system", "content": "SYSTEM" },
            { "role": "user", "content": "USER" },
        ]),
        "the system prompt first, then the transcript"
    );
}

#[tokio::test]
async fn a_non_success_status_is_surfaced_with_its_code() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bound");
    let address = listener.local_addr().expect("an address");
    let server = tokio::spawn(one_shot(listener, 429, "{}"));

    let client = HttpExtractorLlm::new(
        &format!("http://{address}/v1"),
        "test-key",
        "qwen-flash",
        Duration::from_secs(10),
    )
    .expect("a client");
    let error = client
        .complete("s", "u")
        .await
        .expect_err("429 is a failure");
    assert!(matches!(error, ExtractorError::Request { status: 429 }));
    assert_eq!(error.to_string(), "memory extractor request failed: 429");
    let _ = server.await;
}

#[tokio::test]
async fn an_answer_with_no_content_is_an_empty_string_not_an_error() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bound");
    let address = listener.local_addr().expect("an address");
    let server = tokio::spawn(one_shot(listener, 200, r#"{"choices":[]}"#));

    let client = HttpExtractorLlm::new(
        &format!("http://{address}/v1"),
        "test-key",
        "qwen-flash",
        Duration::from_secs(10),
    )
    .expect("a client");
    // The parser then refuses it, which is the layer that should.
    assert_eq!(client.complete("s", "u").await.expect("answered"), "");
    let _ = server.await;
}

#[tokio::test(start_paused = true)]
async fn a_silent_endpoint_times_out_rather_than_holding_the_session() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bound");
    let address = listener.local_addr().expect("an address");
    // Accept and then say nothing at all.
    let server = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.expect("a connection");
        std::future::pending::<()>().await;
        drop(socket);
    });

    let client = HttpExtractorLlm::new(
        &format!("http://{address}/v1"),
        "test-key",
        "qwen-flash",
        via_conversation::extractor::DEFAULT_REQUEST_TIMEOUT,
    )
    .expect("a client");
    let error = client
        .complete("s", "u")
        .await
        .expect_err("the request times out");
    assert!(
        matches!(error, ExtractorError::Timeout),
        "got {error:?}; session close must never wait on a silent endpoint"
    );
    server.abort();
}

#[tokio::test]
async fn an_unreachable_endpoint_is_a_transport_failure() {
    // Bind and immediately drop, so the port is almost certainly closed.
    let address = {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bound");
        listener.local_addr().expect("an address")
    };
    let client = HttpExtractorLlm::new(
        &format!("http://{address}/v1"),
        "test-key",
        "qwen-flash",
        Duration::from_secs(5),
    )
    .expect("a client");
    let error = client
        .complete("s", "u")
        .await
        .expect_err("nothing is listening");
    assert!(
        matches!(error, ExtractorError::Transport(_)),
        "got {error:?}"
    );
}
