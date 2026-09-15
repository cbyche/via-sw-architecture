//! A [`tracing_subscriber::Layer`] so the rest of VIA uses ordinary `tracing`
//! macros while this crate owns the wire format.
//!
//! Mapping rules, none of which are upstream contracts — upstream has no
//! `tracing` — but all of which are fixed here so records look the same
//! wherever they come from:
//!
//! | `tracing` | `via.log/v1` |
//! | --- | --- |
//! | `Level::{TRACE..ERROR}` | the same level name; `fatal` is reachable only through [`crate::Logger::fatal`] |
//! | the `message` field | `message` |
//! | a field literally named `event` | `event`, and removed from the field map |
//! | otherwise `Metadata::target` | `event` |
//! | span fields, root span outward | the context layer, between base fields and event fields |
//!
//! An enclosing [`crate::run_with_log_context`] still applies, and is merged
//! ahead of the span fields.

use crate::level::LogLevel;
use crate::logger::{Logger, current_log_context};
use crate::record::safe_event_name;
use crate::redact::ErrorRecord;
use serde_json::{Map, Number, Value};
use std::fmt;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

/// The field name that overrides the record's `event`.
pub const EVENT_FIELD: &str = "event";

/// The field name carrying a `tracing` macro's format string.
pub const MESSAGE_FIELD: &str = "message";

/// Bridges `tracing` events onto a [`Logger`].
#[derive(Clone, Debug)]
pub struct ViaLayer {
    logger: Logger,
}

impl ViaLayer {
    /// Send every event to `logger`.
    #[must_use]
    pub fn new(logger: Logger) -> Self {
        Self { logger }
    }

    /// The logger behind this layer.
    #[must_use]
    pub fn logger(&self) -> &Logger {
        &self.logger
    }
}

#[derive(Debug, Default)]
struct SpanFields(Map<String, Value>);

impl<S> Layer<S> for ViaLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            return;
        };
        let mut visitor = FieldVisitor::default();
        attrs.record(&mut visitor);
        span.extensions_mut().insert(SpanFields(visitor.fields));
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            return;
        };
        let mut visitor = FieldVisitor::default();
        values.record(&mut visitor);
        let mut extensions = span.extensions_mut();
        if let Some(existing) = extensions.get_mut::<SpanFields>() {
            existing.0.extend(visitor.fields);
        } else {
            extensions.insert(SpanFields(visitor.fields));
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let level = LogLevel::from_tracing(event.metadata().level());
        if !level.is_enabled_for(self.logger.level()) {
            return;
        }
        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);

        let mut context: Vec<Map<String, Value>> = vec![current_log_context()];
        if let Some(scope) = ctx.event_scope(event) {
            for span in scope.from_root() {
                if let Some(fields) = span.extensions().get::<SpanFields>() {
                    context.push(fields.0.clone());
                }
            }
        }

        let name = visitor
            .event_name
            .unwrap_or_else(|| event.metadata().target().to_owned());
        self.logger.emit_with_context(
            level,
            &safe_event_name(&name),
            &context,
            visitor.fields,
            visitor.message.as_deref().unwrap_or_default(),
        );
    }
}

#[derive(Default)]
struct FieldVisitor {
    fields: Map<String, Value>,
    message: Option<String>,
    event_name: Option<String>,
}

impl FieldVisitor {
    fn insert(&mut self, field: &Field, value: Value) {
        match field.name() {
            MESSAGE_FIELD => {
                self.message = Some(match value {
                    Value::String(text) => text,
                    other => other.to_string(),
                });
            }
            EVENT_FIELD => {
                // Consumed as the record's event name rather than kept as a
                // field, so the spine keeps its canonical position.
                self.event_name = Some(match value {
                    Value::String(text) => text,
                    other => other.to_string(),
                });
            }
            name => {
                self.fields.insert(name.to_owned(), value);
            }
        }
    }
}

impl Visit for FieldVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.insert(field, Value::String(value.to_owned()));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.insert(field, Value::Bool(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.insert(field, Value::Number(value.into()));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.insert(field, Value::Number(value.into()));
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        // `serde_json::Number` tops out at i64/u64 without
        // `arbitrary_precision`; anything wider is recorded as its decimal
        // text rather than silently truncated.
        match i64::try_from(value) {
            Ok(narrow) => self.insert(field, Value::Number(narrow.into())),
            Err(_) => self.insert(field, Value::String(value.to_string())),
        }
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        match u64::try_from(value) {
            Ok(narrow) => self.insert(field, Value::Number(narrow.into())),
            Err(_) => self.insert(field, Value::String(value.to_string())),
        }
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        match Number::from_f64(value) {
            // JSON has no NaN or infinity; their text form is the only lossless
            // option and matches `JSON.stringify`'s `null` no better, so the
            // text is kept.
            Some(number) => self.insert(field, Value::Number(number)),
            None => self.insert(field, Value::String(value.to_string())),
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.insert(field, ErrorRecord::from_error(value).to_value());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.insert(field, Value::String(format!("{value:?}")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::LoggerOptions;
    use crate::sink::MemorySink;
    use std::sync::Arc;
    use tracing_subscriber::layer::SubscriberExt;

    #[test]
    fn events_become_via_log_records() {
        let sink = Arc::new(MemorySink::new());
        let mut options = LoggerOptions::detached("gateway");
        options.level = LogLevel::Trace;
        let logger = Logger::with_sinks(options, vec![sink.clone()]);
        let subscriber = tracing_subscriber::registry().with(ViaLayer::new(logger));

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("turn", sessionId = "session-1");
            let _entered = span.enter();
            tracing::info!(
                event = "realtime.connected",
                durationMs = 42,
                "Realtime ready"
            );
        });

        let records = sink.records();
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record["event"], "realtime.connected");
        assert_eq!(record["component"], "gateway");
        assert_eq!(record["level"], "info");
        assert_eq!(record["sessionId"], "session-1");
        assert_eq!(record["durationMs"], 42);
        assert_eq!(record["message"], "Realtime ready");
    }

    #[test]
    fn event_name_falls_back_to_the_target() {
        let sink = Arc::new(MemorySink::new());
        let mut options = LoggerOptions::detached("gateway");
        options.level = LogLevel::Trace;
        let logger = Logger::with_sinks(options, vec![sink.clone()]);
        let subscriber = tracing_subscriber::registry().with(ViaLayer::new(logger));

        tracing::subscriber::with_default(subscriber, || {
            tracing::warn!(target: "via.gateway", "something");
        });

        let records = sink.records();
        assert_eq!(records[0]["event"], "via.gateway");
        assert_eq!(records[0]["level"], "warn");
    }
}
