//! Doubles for the two things a realtime test needs: a provider, and a socket.
//!
//! Published rather than `#[cfg(test)]` because three crates need them —
//! `via-realtime-mock`, `via-realtime-dashscope` and `via-voice` all have to
//! drive a session, and none of them should have to reimplement a provider to do
//! it. [`TestProvider`] is upstream's own `testProvider()` helper
//! (`server/test/realtime-provider-registry.test.mjs:11-30`), grown the builder
//! methods its call sites pass as overrides.
//!
//! [`test_transport`] is the piece upstream does not have and does not need,
//! because `ws` is trivially stubbed in JavaScript: a pair of in-memory channels
//! that stands in for a WebSocket, so the whole session state machine — the two
//! watchdogs, the busy-retry ladder, correlation, the output queue — is
//! exercised with no listener, no port and no wall-clock time.

use std::sync::Arc;
use std::time::Duration;

use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use serde_json::{Map, Value, json};
use tokio_tungstenite::tungstenite::Message;
use via_catalog::ModelProfile;
use via_i18n::Locale;

use crate::capabilities::ProviderCapabilities;
use crate::error::RealtimeError;
use crate::event_error::{ErrorClass, is_recoverable_realtime_inactivity_error};
use crate::protocol::{RealtimeProtocol, ga_realtime_protocol, openai_compatible_protocol};
use crate::provider::{Injection, PermissionRequest, RealtimeProvider, SessionRequest, Visibility};
use crate::session::Transport;

/// The classification a [`TestProvider`] applies by default.
///
/// The union of the two shipped providers' recognisable phrases, so a test can
/// exercise both busy-retry paths without standing up either real provider.
#[must_use]
pub fn default_test_classification(message: &str) -> ErrorClass {
    let text = message.to_ascii_lowercase();
    if is_recoverable_realtime_inactivity_error(message) {
        return ErrorClass::Inactivity;
    }
    if text.contains("user is speaking") {
        return ErrorClass::InputBusy;
    }
    if text.contains("another response is in progress") {
        return ErrorClass::ResponseSlotBusy;
    }
    if text.contains("session_limit_reached") || text.contains("session slots") {
        return ErrorClass::CapacityBusy;
    }
    if text.contains("no active response") {
        return ErrorClass::NoActiveResponse;
    }
    if text.contains("invalid api-key") || text.contains("unexpected server response: 401") {
        return ErrorClass::Fatal;
    }
    ErrorClass::Other
}

/// A provider that answers everything and reaches nothing.
#[derive(Debug)]
pub struct TestProvider {
    key: String,
    label: String,
    aliases: Vec<String>,
    visibility: Visibility,
    input_sample_rate: u32,
    output_sample_rate: u32,
    response_start_timeout: Option<Duration>,
    configured: bool,
    model: Option<String>,
    voice: Option<String>,
    url: String,
    headers: Vec<(String, String)>,
    capabilities: ProviderCapabilities,
    model_profile: Option<ModelProfile>,
    model_catalog: Vec<ModelProfile>,
    configuration_signature: String,
    protocol: Box<dyn RealtimeProtocol>,
    preflight: Option<RealtimeError>,
    classify: fn(&str) -> ErrorClass,
    modalities: Vec<String>,
}

impl TestProvider {
    /// A provider named `key`, speaking the beta dialect, configured and ready.
    #[must_use]
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_owned(),
            label: key.to_owned(),
            aliases: Vec::new(),
            visibility: Visibility::Public,
            input_sample_rate: 16_000,
            output_sample_rate: 24_000,
            response_start_timeout: None,
            configured: true,
            model: Some("test-model".to_owned()),
            voice: None,
            url: "ws://127.0.0.1/realtime".to_owned(),
            headers: Vec::new(),
            capabilities: ProviderCapabilities::DEFAULT,
            model_profile: None,
            model_catalog: Vec::new(),
            configuration_signature: "test-signature".to_owned(),
            protocol: Box::new(openai_compatible_protocol()),
            preflight: None,
            classify: default_test_classification,
            modalities: vec!["text".to_owned(), "audio".to_owned()],
        }
    }

    /// A provider speaking the GA dialect with the capability set
    /// huggingface/speech-to-speech declares.
    #[must_use]
    pub fn ga(key: &str) -> Self {
        Self::new(key)
            .with_protocol(Box::new(ga_realtime_protocol()))
            .with_capabilities(ProviderCapabilities {
                acknowledges_session_update: false,
                single_response_slot: true,
                response_metadata_correlation: true,
                per_response_instructions: true,
                ..ProviderCapabilities::DEFAULT
            })
            .with_model(None)
            .with_modalities(["audio"])
    }

    /// Override the label.
    #[must_use]
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = label.to_owned();
        self
    }

    /// Override the aliases.
    #[must_use]
    pub fn with_aliases<I, S>(mut self, aliases: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.aliases = aliases.into_iter().map(Into::into).collect();
        self
    }

    /// Override the visibility.
    #[must_use]
    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Override the capture rate.
    #[must_use]
    pub fn with_input_sample_rate(mut self, rate: u32) -> Self {
        self.input_sample_rate = rate;
        self
    }

    /// Override the playback rate.
    #[must_use]
    pub fn with_output_sample_rate(mut self, rate: u32) -> Self {
        self.output_sample_rate = rate;
        self
    }

    /// Override the response-start budget.
    #[must_use]
    pub fn with_response_start_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.response_start_timeout = timeout;
        self
    }

    /// Override whether the provider is configured.
    #[must_use]
    pub fn with_configured(mut self, configured: bool) -> Self {
        self.configured = configured;
        self
    }

    /// Override the model id.
    #[must_use]
    pub fn with_model(mut self, model: Option<&str>) -> Self {
        self.model = model.map(str::to_owned);
        self
    }

    /// Override the voice id.
    #[must_use]
    pub fn with_voice(mut self, voice: Option<&str>) -> Self {
        self.voice = voice.map(str::to_owned);
        self
    }

    /// Override the endpoint.
    #[must_use]
    pub fn with_url(mut self, url: &str) -> Self {
        self.url = url.to_owned();
        self
    }

    /// Override the upgrade headers.
    #[must_use]
    pub fn with_headers<I>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = (String, String)>,
    {
        self.headers = headers.into_iter().collect();
        self
    }

    /// Override the declared capabilities.
    #[must_use]
    pub fn with_capabilities(mut self, capabilities: ProviderCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Override the active model profile.
    #[must_use]
    pub fn with_model_profile(mut self, profile: Option<ModelProfile>) -> Self {
        self.model_profile = profile;
        self
    }

    /// Override the model catalog.
    #[must_use]
    pub fn with_model_catalog<I>(mut self, catalog: I) -> Self
    where
        I: IntoIterator<Item = ModelProfile>,
    {
        self.model_catalog = catalog.into_iter().collect();
        self
    }

    /// Override the configuration signature.
    #[must_use]
    pub fn with_configuration_signature(mut self, signature: &str) -> Self {
        self.configuration_signature = signature.to_owned();
        self
    }

    /// Override the dialect.
    #[must_use]
    pub fn with_protocol(mut self, protocol: Box<dyn RealtimeProtocol>) -> Self {
        self.protocol = protocol;
        self
    }

    /// Make [`RealtimeProvider::preflight`] refuse.
    #[must_use]
    pub fn with_preflight_error(mut self, error: RealtimeError) -> Self {
        self.preflight = Some(error);
        self
    }

    /// Override the error classifier.
    #[must_use]
    pub fn with_classification(mut self, classify: fn(&str) -> ErrorClass) -> Self {
        self.classify = classify;
        self
    }

    /// Override the modalities every response body declares.
    #[must_use]
    pub fn with_modalities<I, S>(mut self, modalities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.modalities = modalities.into_iter().map(Into::into).collect();
        self
    }

    /// This provider behind an `Arc`, ready for a registry or a session.
    #[must_use]
    pub fn shared(self) -> Arc<dyn RealtimeProvider> {
        Arc::new(self)
    }

    fn modalities(&self) -> Value {
        Value::Array(
            self.modalities
                .iter()
                .cloned()
                .map(Value::String)
                .collect::<Vec<_>>(),
        )
    }
}

impl RealtimeProvider for TestProvider {
    fn key(&self) -> &str {
        &self.key
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn aliases(&self) -> Vec<String> {
        self.aliases.clone()
    }

    fn visibility(&self) -> Visibility {
        self.visibility
    }

    fn input_sample_rate(&self) -> u32 {
        self.input_sample_rate
    }

    fn output_sample_rate(&self) -> u32 {
        self.output_sample_rate
    }

    fn response_start_timeout(&self) -> Option<Duration> {
        self.response_start_timeout
    }

    fn is_configured(&self) -> bool {
        self.configured
    }

    fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    fn voice(&self) -> Option<&str> {
        self.voice.as_deref()
    }

    fn url(&self) -> Result<String, RealtimeError> {
        Ok(self.url.clone())
    }

    fn headers(&self) -> Vec<(String, String)> {
        self.headers.clone()
    }

    fn capabilities(&self) -> ProviderCapabilities {
        self.capabilities
    }

    fn model_profile(&self) -> Option<ModelProfile> {
        self.model_profile.clone()
    }

    fn model_catalog(&self) -> &[ModelProfile] {
        &self.model_catalog
    }

    fn configuration_signature(&self) -> String {
        self.configuration_signature.clone()
    }

    fn protocol(&self) -> &dyn RealtimeProtocol {
        self.protocol.as_ref()
    }

    fn preflight(&self) -> Result<(), RealtimeError> {
        match &self.preflight {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }

    fn classify_error(&self, message: &str) -> ErrorClass {
        (self.classify)(message)
    }

    fn build_session(&self, request: &SessionRequest<'_>) -> Value {
        let mut session = Map::new();
        session.insert(
            "instructions".to_owned(),
            Value::String(request.agent_context.instructions.clone()),
        );
        if !request.agent_context.tools.is_empty() {
            session.insert(
                "tools".to_owned(),
                Value::Array(request.agent_context.tools.clone()),
            );
        }
        if !request.configured {
            // The first update negotiates; every later one carries only
            // instructions and tools, exactly as `dashscope.mjs:70-92`.
            session.insert("modalities".to_owned(), self.modalities());
            session.insert("turn_detection".to_owned(), json!({ "type": "server_vad" }));
        }
        for (key, value) in &request.agent_context.extra {
            session.insert(key.clone(), value.clone());
        }
        Value::Object(session)
    }

    fn build_speak_response(&self, content: &str) -> Value {
        json!({
            "conversation": "none",
            "modalities": self.modalities(),
            "instructions": content,
        })
    }

    fn build_result_injection(&self, content: &str) -> Injection {
        Injection {
            item: json!({
                "type": "message",
                "role": "user",
                "content": [{ "type": "input_text", "text": content }],
            }),
            response: json!({
                "modalities": self.modalities(),
                "tool_choice": "none",
                "instructions": content,
            }),
        }
    }

    fn build_permission_injection(&self, permission: &PermissionRequest) -> Injection {
        let text = [
            "<backend_permission_request>".to_owned(),
            format!("authorization_id={}", permission.id),
            format!("operation={}", permission.summary),
            "</backend_permission_request>".to_owned(),
        ]
        .join("\n");
        Injection {
            item: json!({
                "type": "message",
                "role": "user",
                "content": [{ "type": "input_text", "text": text }],
            }),
            response: json!({
                "modalities": self.modalities(),
                "tool_choice": "none",
                "instructions": "ask about it",
            }),
        }
    }

    fn missing_configuration_message(&self, _locale: Locale) -> String {
        format!("{} is not configured", self.label)
    }

    fn connect_timeout_message(&self, _locale: Locale) -> String {
        format!("{} connect timed out", self.label)
    }
}

/// The far end of a [`test_transport`].
#[derive(Debug)]
pub struct TestPeer {
    to_session: Option<mpsc::UnboundedSender<Result<Message, String>>>,
    from_session: mpsc::UnboundedReceiver<Message>,
}

impl TestPeer {
    /// Deliver one provider frame to the session.
    ///
    /// # Panics
    ///
    /// If the session has already gone away — in a test that is the failure, not
    /// something to recover from.
    pub fn send(&self, event: Value) {
        let text = serde_json::to_string(&event).expect("a Value always serializes");
        self.to_session
            .as_ref()
            .expect("the peer is still open")
            .unbounded_send(Ok(Message::Text(text.into())))
            .expect("the session is still listening");
    }

    /// Deliver a raw text frame, valid JSON or not.
    ///
    /// # Panics
    ///
    /// If the session has already gone away.
    pub fn send_raw(&self, text: &str) {
        self.to_session
            .as_ref()
            .expect("the peer is still open")
            .unbounded_send(Ok(Message::Text(text.into())))
            .expect("the session is still listening");
    }

    /// Fail the transport, the way a broken socket does.
    ///
    /// # Panics
    ///
    /// If the session has already gone away.
    pub fn fail(&self, detail: &str) {
        self.to_session
            .as_ref()
            .expect("the peer is still open")
            .unbounded_send(Err(detail.to_owned()))
            .expect("the session is still listening");
    }

    /// End the transport, the way a closed socket does.
    pub fn close(&mut self) {
        self.to_session = None;
    }

    /// The next JSON frame the session wrote.
    ///
    /// `None` once the session has closed. A control frame — the `Close` a
    /// closing session writes, a keep-alive ping — is skipped rather than
    /// treated as content, so a test that reads frames after `close()` sees the
    /// end of the stream instead of a surprise.
    pub async fn next_frame(&mut self) -> Option<Value> {
        loop {
            match self.from_session.next().await? {
                Message::Text(text) => {
                    if let Ok(frame) = serde_json::from_str(&text) {
                        return Some(frame);
                    }
                }
                Message::Binary(bytes) => {
                    if let Ok(frame) = serde_json::from_slice(&bytes) {
                        return Some(frame);
                    }
                }
                Message::Close(_) => return None,
                Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => {}
            }
        }
    }

    /// The next frame whose `type` is `kind`, skipping everything before it.
    pub async fn next_frame_of(&mut self, kind: &str) -> Option<Value> {
        while let Some(frame) = self.next_frame().await {
            if frame.get("type").and_then(Value::as_str) == Some(kind) {
                return Some(frame);
            }
        }
        None
    }

    /// Every frame written so far, without waiting for more.
    pub fn drain_frames(&mut self) -> Vec<Value> {
        let mut frames = Vec::new();
        while let Ok(Message::Text(text)) = self.from_session.try_recv() {
            if let Ok(value) = serde_json::from_str(&text) {
                frames.push(value);
            }
        }
        frames
    }
}

/// A transport whose far end is a [`TestPeer`].
#[must_use]
pub fn test_transport() -> (Transport, TestPeer) {
    let (to_session, session_reads) = mpsc::unbounded::<Result<Message, String>>();
    let (session_writes, from_session) = mpsc::unbounded::<Message>();
    (
        Transport::new(
            session_writes.sink_map_err(|error| error.to_string()),
            session_reads,
        ),
        TestPeer {
            to_session: Some(to_session),
            from_session,
        },
    )
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn the_default_test_provider_validates() {
        assert_eq!(
            crate::provider::validate_realtime_provider(&TestProvider::new("test")),
            Ok(())
        );
    }

    #[test]
    fn the_ga_variant_declares_the_speech_to_speech_capability_set() {
        let provider = TestProvider::ga("ga");
        assert_eq!(
            provider.capabilities(),
            ProviderCapabilities {
                acknowledges_session_update: false,
                single_response_slot: true,
                response_metadata_correlation: true,
                per_response_instructions: true,
                conversation_item_id_echo: true,
            }
        );
        assert_eq!(provider.model(), None);
    }

    #[test]
    fn the_first_session_update_negotiates_and_later_ones_do_not() {
        let provider = TestProvider::new("test");
        let context = crate::provider::AgentContext {
            instructions: "hi".into(),
            tools: vec![json!({ "type": "function" })],
            ..Default::default()
        };
        let first = provider.build_session(&SessionRequest {
            configured: false,
            agent_context: &context,
        });
        assert_eq!(first["modalities"], json!(["text", "audio"]));
        assert_eq!(first["turn_detection"], json!({ "type": "server_vad" }));

        let later = provider.build_session(&SessionRequest {
            configured: true,
            agent_context: &context,
        });
        let keys: Vec<&str> = later
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["instructions", "tools"]);
    }

    #[test]
    fn the_default_classifier_knows_both_busy_phrases() {
        assert_eq!(
            default_test_classification("Cannot create response while user is speaking."),
            ErrorClass::InputBusy
        );
        assert_eq!(
            default_test_classification(
                "Cannot create response while another response is in progress."
            ),
            ErrorClass::ResponseSlotBusy
        );
        assert_eq!(
            default_test_classification(
                "All 1 session slots are in use. Disconnect an existing client first."
            ),
            ErrorClass::CapacityBusy
        );
        assert_eq!(
            default_test_classification(
                "Your session was closed because no response was generated for 180 seconds."
            ),
            ErrorClass::Inactivity
        );
        assert_eq!(
            default_test_classification("something else"),
            ErrorClass::Other
        );
    }

    #[tokio::test]
    async fn the_test_transport_carries_frames_both_ways() {
        let (mut transport, mut peer) = test_transport();
        peer.send(json!({ "type": "session.created" }));
        let inbound = transport.stream.next().await.expect("frame").expect("ok");
        assert_eq!(
            inbound,
            Message::Text(r#"{"type":"session.created"}"#.into())
        );

        transport
            .sink
            .send(Message::Text(r#"{"type":"session.update"}"#.into()))
            .await
            .expect("write");
        assert_eq!(
            peer.next_frame().await,
            Some(json!({ "type": "session.update" }))
        );
    }
}
