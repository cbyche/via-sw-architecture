//! The test doubles themselves.
//!
//! A double that lies is worse than no double, and both of these ship in the
//! default build for `via-voice` and `via-app` to use. So their own behaviour
//! is pinned here: what they record, what they answer when their script runs
//! out, and that neither of them ever reaches a socket or a model.

use std::sync::Arc;

use pretty_assertions::assert_eq;
use via_wake_word::{FetchResponse, ModelFetch, ScriptedFetch, WakeWordError};

#[tokio::test]
async fn the_fetcher_answers_its_script_in_order_and_records_every_url() {
    let fetcher = ScriptedFetch::new();
    fetcher.push_ok(b"first".to_vec());
    fetcher.push_status(404);
    fetcher.push_ok(b"third".to_vec());
    assert_eq!(fetcher.remaining(), 3);

    let first = fetcher
        .fetch("https://a.invalid/1")
        .await
        .expect("answered");
    assert_eq!(first, FetchResponse::ok(b"first".to_vec()));
    assert!(first.is_ok());

    let second = fetcher
        .fetch("https://a.invalid/2")
        .await
        .expect("answered");
    assert_eq!(second.status, 404);
    assert_eq!(second.body, None);
    assert!(!second.is_ok());

    fetcher
        .fetch("https://a.invalid/3")
        .await
        .expect("answered");

    assert_eq!(
        fetcher.requested(),
        vec![
            "https://a.invalid/1".to_owned(),
            "https://a.invalid/2".to_owned(),
            "https://a.invalid/3".to_owned(),
        ]
    );
    assert_eq!(fetcher.requests(), 3);
    assert_eq!(fetcher.remaining(), 0);
}

#[tokio::test]
async fn a_request_the_script_did_not_arrange_names_the_url_it_did_not_expect() {
    let fetcher = ScriptedFetch::new();
    let error = fetcher
        .fetch("https://unexpected.invalid/model.tar.bz2")
        .await
        .expect_err("an empty script answers nothing");
    match &error {
        WakeWordError::Fetch { url, detail } => {
            assert_eq!(url, "https://unexpected.invalid/model.tar.bz2");
            assert!(detail.contains("no response left"));
        }
        other => panic!("expected a transport failure, got {other:?}"),
    }
    // The unexpected request is still recorded, so a test can see what asked.
    assert_eq!(fetcher.requests(), 1);
}

#[tokio::test]
async fn a_scripted_transport_failure_is_not_a_status() {
    let fetcher = ScriptedFetch::new();
    fetcher.push_failure("dns lookup failed");
    let error = fetcher
        .fetch("https://a.invalid/x")
        .await
        .expect_err("failed");
    assert!(matches!(error, WakeWordError::Fetch { .. }));
    assert!(error.to_string().contains("dns lookup failed"));
}

#[test]
fn the_success_range_is_the_javascript_one() {
    // `response.ok` is `200..=299`, which is what upstream branches on.
    for status in [200u16, 201, 204, 299] {
        assert!(FetchResponse::ok(Vec::new()).is_ok());
        assert!(
            FetchResponse {
                status,
                body: Some(Vec::new())
            }
            .is_ok(),
            "{status} should be a success"
        );
    }
    for status in [100u16, 199, 300, 301, 400, 404, 500, 503] {
        assert!(
            !FetchResponse::status(status).is_ok(),
            "{status} should not be a success"
        );
    }
}

#[tokio::test]
async fn the_fetcher_is_shareable_across_tasks() {
    // `ModelManager` holds it as `Arc<dyn ModelFetch>`, so `Send + Sync` is a
    // requirement rather than a convenience.
    let fetcher: Arc<dyn ModelFetch> = Arc::new(ScriptedFetch::serving(b"body".to_vec()));
    let clone = Arc::clone(&fetcher);
    let joined = tokio::spawn(async move { clone.fetch("https://a.invalid/x").await })
        .await
        .expect("the task completed");
    assert_eq!(joined.expect("answered").body, Some(b"body".to_vec()));
}
