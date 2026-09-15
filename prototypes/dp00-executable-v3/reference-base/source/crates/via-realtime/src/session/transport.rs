//! The bytes under the session.
//!
//! Upstream constructs a `ws` socket inline inside `connect()`. VIA separates
//! the socket from the session for one concrete reason: everything interesting
//! about [`RealtimeSession`](crate::RealtimeSession) — correlation, the output
//! queue, the two watchdogs, the busy-retry ladder — is transport-independent,
//! and a test that has to stand up a TCP listener to reach any of it is a test
//! that will be written once and then avoided.
//!
//! So [`Transport`] is a pair of boxed halves, [`Transport::connect`] builds one
//! over a real WebSocket, and [`Transport::new`] builds one over anything else —
//! an in-memory duplex in a test, a replay file in `via-realtime-mock`, a pipe to
//! a local pipeline process.

use std::pin::Pin;

use futures::{Sink, SinkExt, Stream, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{HeaderName, HeaderValue};

use crate::error::RealtimeError;

/// The write half of a realtime transport.
///
/// The error type is `String` rather than `tungstenite::Error` so a transport
/// that is not a WebSocket can be plugged in without inventing a tungstenite
/// error to fail with — and because the only thing the session does with a
/// transport failure is put its text into [`RealtimeError::Transport`], whose
/// text is what a provider's `classify_error` reads.
pub type WsSink = Pin<Box<dyn Sink<Message, Error = String> + Send>>;

/// The read half of a realtime transport.
pub type WsStream = Pin<Box<dyn Stream<Item = Result<Message, String>> + Send>>;

/// A realtime transport: one write half, one read half.
pub struct Transport {
    /// Frames out.
    pub sink: WsSink,
    /// Frames in.
    pub stream: WsStream,
}

impl core::fmt::Debug for Transport {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Transport").finish_non_exhaustive()
    }
}

impl Transport {
    /// Wrap any sink and stream of WebSocket messages.
    ///
    /// The two halves may fail with different error types — a duplex built from
    /// two channels usually does — so only `Display` is required of either.
    pub fn new<Si, St, SinkError, StreamError>(sink: Si, stream: St) -> Self
    where
        Si: Sink<Message, Error = SinkError> + Send + 'static,
        St: Stream<Item = Result<Message, StreamError>> + Send + 'static,
        SinkError: core::fmt::Display + Send + 'static,
        StreamError: core::fmt::Display + Send + 'static,
    {
        Self {
            sink: Box::pin(sink.sink_map_err(|error| error.to_string())),
            stream: Box::pin(stream.map(|frame| frame.map_err(|error| error.to_string()))),
        }
    }

    /// Open a real WebSocket.
    ///
    /// `headers` are appended to the upgrade request in the order the provider
    /// listed them, which is why [`RealtimeProvider::headers`] is a `Vec` of
    /// pairs rather than a map.
    ///
    /// [`RealtimeProvider::headers`]: crate::RealtimeProvider::headers
    ///
    /// # Errors
    ///
    /// [`RealtimeError::Transport`] for an unusable URL, an unusable header, or
    /// a refused handshake. The text is the transport's own — an HTTP status
    /// line reaches a provider's `classify_error` as
    /// `Unexpected server response: 401`, which is one of the catalogued `fatal`
    /// patterns.
    pub async fn connect(url: &str, headers: &[(String, String)]) -> Result<Self, RealtimeError> {
        let mut request = url
            .into_client_request()
            .map_err(|error| RealtimeError::Transport {
                detail: error.to_string(),
            })?;
        for (name, value) in headers {
            let name = HeaderName::from_bytes(name.as_bytes()).map_err(|error| {
                RealtimeError::Transport {
                    detail: error.to_string(),
                }
            })?;
            let value = HeaderValue::from_str(value).map_err(|error| RealtimeError::Transport {
                detail: error.to_string(),
            })?;
            request.headers_mut().append(name, value);
        }
        let (socket, _response) =
            tokio_tungstenite::connect_async(request)
                .await
                .map_err(|error| RealtimeError::Transport {
                    detail: error.to_string(),
                })?;
        let (sink, stream) = socket.split();
        Ok(Self::new(sink, stream))
    }
}

#[cfg(test)]
mod tests {
    use futures::channel::mpsc;

    use super::*;

    #[tokio::test]
    async fn a_transport_can_be_built_over_any_channel() {
        let (outgoing, mut written) = mpsc::unbounded::<Message>();
        let (incoming_tx, incoming) = mpsc::unbounded::<Result<Message, mpsc::SendError>>();
        let mut transport = Transport::new(outgoing, incoming);

        transport
            .sink
            .send(Message::Text("hello".into()))
            .await
            .expect("send");
        assert_eq!(written.next().await, Some(Message::Text("hello".into())));

        incoming_tx
            .unbounded_send(Ok(Message::Text("world".into())))
            .expect("queue");
        assert_eq!(
            transport.stream.next().await,
            Some(Ok(Message::Text("world".into())))
        );
    }

    #[tokio::test]
    async fn an_unusable_url_is_a_transport_failure_carrying_its_own_text() {
        let error = Transport::connect("not a url", &[])
            .await
            .expect_err("refused");
        assert_eq!(error.code(), "VIA_REALTIME_TRANSPORT");
    }

    #[tokio::test]
    async fn an_unusable_header_name_is_refused_before_the_socket() {
        let error = Transport::connect(
            "ws://127.0.0.1:9/realtime",
            &[("bad header".into(), "v".into())],
        )
        .await
        .expect_err("refused");
        assert_eq!(error.code(), "VIA_REALTIME_TRANSPORT");
    }
}
