//! Everything the realtime transport refuses.
//!
//! Upstream throws bare `Error`s carrying interpolated Chinese sentences from
//! three files — `providers/provider-registry.mjs`, `realtime-provider.mjs` and
//! `providers/registry.mjs`. Per `docs/fidelity.md` the *discriminant* and the
//! values a sentence interpolates live here, and the sentence itself is
//! `via-i18n`'s; [`RealtimeError::message`] renders it and the `Display` impls
//! are developer English no operator reads.
//!
//! # Two messages are deliberately not localized
//!
//! [`RealtimeError::NotConfigured`] and [`RealtimeError::ConnectTimeout`] carry a
//! string the **provider** supplied, because upstream reads
//! `provider.missingConfigurationMessage` and `provider.connectTimeoutMessage`
//! rather than composing one: the two shipped providers name different
//! environment variables and one of them interpolates its endpoint. The provider
//! localizes; this crate carries.
//!
//! [`RealtimeError::Transport`] is not localized either, and that one is a
//! contract rather than a convenience: a socket-level failure's text is what
//! `classifyError` matches on, and the catalogued `fatal` pattern includes
//! `unexpected server response: (?:401|403)` — the raw tungstenite sentence.
//! Translating it would silently stop an expired credential from being
//! recognised.

use via_i18n::{Locale, format, keys, t};

/// Anything the realtime transport refuses.
///
/// Deliberately not `#[non_exhaustive]`, matching the other VIA error enums: a
/// crate above this one should be made to recompile when a new refusal appears
/// rather than fold it into a wildcard arm that already existed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RealtimeError {
    /// No registered provider answers to that key or alias.
    ///
    /// Upstream ``new Error(`不支持的 Realtime 前台：${name}（可选 ${keys.join('、')}）`)``,
    /// `providers/provider-registry.mjs:252-255`. Reachable from configuration
    /// **and** from a client's `connect` frame, so the available list is part of
    /// the message rather than only of the log line.
    #[error("unsupported realtime frontend: {requested} (available: {available:?})")]
    UnsupportedProvider {
        /// The value as requested, trimmed and lowercased.
        requested: String,
        /// Registered provider keys, in registration order.
        available: Vec<String>,
    },

    /// A second provider claimed a key or alias that is already registered.
    ///
    /// Upstream ``new Error(`Realtime Provider 名称已注册：${name}`)``,
    /// `provider-registry.mjs:239`, test-locked as `/已注册/`.
    #[error("realtime provider name is already registered: {name}")]
    ProviderNameTaken {
        /// The colliding key or alias, cleaned.
        name: String,
    },

    /// A provider offered an empty key or an empty label.
    ///
    /// Upstream `new Error('Realtime Provider 必须定义 key 和 label')`,
    /// `provider-registry.mjs:154`.
    #[error("a realtime provider must define key and label")]
    ProviderNeedsKeyAndLabel,

    /// A provider key that is not `^[a-z0-9][a-z0-9-]*$`.
    ///
    /// Upstream ``new Error(`Realtime Provider key 无效：${key}`)``,
    /// `provider-registry.mjs:157`. The shape is what a client may send on the
    /// `connect` frame, so it is validated at registration rather than at use.
    #[error("invalid realtime provider key: {key}")]
    ProviderKeyInvalid {
        /// The offending key, verbatim.
        key: String,
    },

    /// A provider with no usable input sample rate.
    ///
    /// Upstream ``new Error(`Realtime Provider ${key} 缺少 inputSampleRate`)``,
    /// `provider-registry.mjs:165`.
    #[error("realtime provider {key} has no input sample rate")]
    ProviderMissingInputSampleRate {
        /// The provider key.
        key: String,
    },

    /// A provider with no usable output sample rate.
    ///
    /// Upstream ``new Error(`Realtime Provider ${key} 缺少 outputSampleRate`)``,
    /// `provider-registry.mjs:168`.
    #[error("realtime provider {key} has no output sample rate")]
    ProviderMissingOutputSampleRate {
        /// The provider key.
        key: String,
    },

    /// A provider whose response-start timeout is zero.
    ///
    /// Upstream ``new Error(`Realtime Provider ${key} 的 responseStartTimeoutMs 必须是正数`)``,
    /// `provider-registry.mjs:177-179`.
    #[error("realtime provider {key} response start timeout must be positive")]
    ProviderResponseTimeoutInvalid {
        /// The provider key.
        key: String,
    },

    /// A provider whose alias list contains a blank entry.
    ///
    /// Upstream ``new Error(`Realtime Provider ${key} aliases 无效`)``,
    /// `provider-registry.mjs:215`.
    #[error("realtime provider {key} has invalid aliases")]
    ProviderAliasesInvalid {
        /// The provider key.
        key: String,
    },

    /// A provider whose model profile has a blank `id` or `label`.
    ///
    /// Upstream ``new Error(`Realtime Provider ${key} modelProfile 无效`)``,
    /// `provider-registry.mjs:91`.
    #[error("realtime provider {key} has an invalid model profile")]
    ProviderModelProfileInvalid {
        /// The provider key.
        key: String,
    },

    /// A provider whose model profile has a blank default voice.
    ///
    /// Upstream ``new Error(`Realtime Provider ${key} modelProfile.sessionDefaults 不完整`)``,
    /// `provider-registry.mjs:124-127`.
    #[error("realtime provider {key} has incomplete model profile session defaults")]
    ProviderSessionDefaultsIncomplete {
        /// The provider key.
        key: String,
    },

    /// The configured realtime model is one this provider cannot serve.
    ///
    /// Upstream ``new Error(`不支持的 Realtime 模型：${id}（${label}）`)``,
    /// `realtime-provider.mjs:135-139` — refused **before** the WebSocket is
    /// opened, which is what `server/test/realtime-provider.test.mjs:493-506`
    /// asserts by checking `frontend.ws === null` afterwards.
    #[error("unsupported realtime model: {id} ({label})")]
    UnsupportedModel {
        /// The model id, verbatim.
        id: String,
        /// The provider's label.
        label: String,
    },

    /// The provider is not configured, so there is nothing to connect to.
    ///
    /// Upstream `new Error(provider.missingConfigurationMessage)`,
    /// `realtime-provider.mjs:141-143`. The sentence is the provider's own — see
    /// the module docs.
    #[error("{provider} is not configured: {message}")]
    NotConfigured {
        /// The provider key.
        provider: String,
        /// The provider's own, already-localized sentence.
        message: String,
    },

    /// The connection did not become usable inside the connect budget.
    ///
    /// Upstream `new Error(provider.connectTimeoutMessage)` after a hard-coded
    /// 25 s timer, `realtime-provider.mjs:150-154`. The budget covers the whole
    /// handshake — socket open, `session.created`, `session.update`,
    /// `session.updated` — not just the TCP connect.
    #[error("{provider} connect timed out: {message}")]
    ConnectTimeout {
        /// The provider key.
        provider: String,
        /// The provider's own, already-localized sentence.
        message: String,
    },

    /// The socket closed.
    ///
    /// Upstream ``new Error(`${provider.label} 连接已关闭`)``,
    /// `realtime-provider.mjs:188`. Also what a call on a session whose owning
    /// task has already stopped answers with, because from the caller's side the
    /// two are the same fact.
    #[error("the {label} connection closed")]
    ConnectionClosed {
        /// The provider's label.
        label: String,
    },

    /// The provider never acknowledged a conversation item.
    ///
    /// Upstream ``new Error(`${provider.label} 未确认对话项 ${id}`)``,
    /// `realtime-provider.mjs:372`. The wait is bounded by the *response start*
    /// timeout, not by a separate one.
    #[error("{label} did not confirm conversation item {id}")]
    ItemUnconfirmed {
        /// The provider's label.
        label: String,
        /// The client-assigned item id.
        id: String,
    },

    /// The provider rejected a conversation item.
    ///
    /// Upstream ``new Error(event.error?.message || `${provider.label} 创建对话项失败`)``,
    /// `realtime-provider.mjs:566-571`, with `realtimeEvent = true` set on it so
    /// the failure is not reported twice — once as the response outcome and
    /// again as a session error.
    #[error("{label} rejected a conversation item")]
    ItemRejected {
        /// The provider's label.
        label: String,
        /// The provider's own sentence, when it gave one.
        detail: Option<String>,
    },

    /// A provider `error` event arrived before the session was ready.
    ///
    /// Upstream `throw new Error(realtimeEventErrorMessage(event))` from
    /// `handleProviderEvent`, `realtime-provider.mjs:216-220`. It rejects
    /// `connect()`, which is what makes a busy speech-to-speech session slot fail
    /// fast instead of hanging until the connect timeout.
    #[error("{message}")]
    ProviderRefused {
        /// The composed message — [`crate::realtime_event_error_message`].
        message: String,
    },

    /// Everything in flight was cancelled.
    ///
    /// Upstream `new Error('Realtime 请求已取消')`, `realtime-provider.mjs:438`.
    #[error("the realtime request was cancelled")]
    RequestCancelled,

    /// The session was reset — the socket closed or `close()` was called.
    ///
    /// Upstream `new Error('Realtime 会话已重置')`, `realtime-provider.mjs:776`.
    #[error("the realtime session was reset")]
    SessionReset,

    /// A second response start was attempted while one was still awaiting
    /// `response.created`.
    ///
    /// Upstream `new Error('Realtime 响应关联冲突：已有响应正在等待 response.created')`,
    /// `realtime-provider.mjs:501-503`, test-locked at
    /// `server/test/realtime-provider.test.mjs:350`. Correlation fails **closed**:
    /// with two unanswered starts there is no way to know which
    /// `response.created` belongs to which, so neither is guessed.
    #[error(
        "realtime response correlation conflict: a response is already awaiting response.created"
    )]
    ResponseCorrelationConflict,

    /// The socket itself failed.
    ///
    /// Not localized — see the module docs. `detail` is the transport's own
    /// sentence, which the provider's `classifyError` corpus matches on.
    #[error("realtime transport failure: {detail}")]
    Transport {
        /// The transport's own message, verbatim.
        detail: String,
    },
}

impl RealtimeError {
    /// A stable machine-readable code.
    ///
    /// VIA's own: upstream throws untyped `Error`s for every one of these.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedProvider { .. } => "VIA_REALTIME_PROVIDER_UNSUPPORTED",
            Self::ProviderNameTaken { .. } => "VIA_REALTIME_PROVIDER_NAME_TAKEN",
            Self::ProviderNeedsKeyAndLabel => "VIA_REALTIME_PROVIDER_INCOMPLETE",
            Self::ProviderKeyInvalid { .. } => "VIA_REALTIME_PROVIDER_KEY_INVALID",
            Self::ProviderMissingInputSampleRate { .. } => "VIA_REALTIME_PROVIDER_NO_INPUT_RATE",
            Self::ProviderMissingOutputSampleRate { .. } => "VIA_REALTIME_PROVIDER_NO_OUTPUT_RATE",
            Self::ProviderResponseTimeoutInvalid { .. } => "VIA_REALTIME_PROVIDER_TIMEOUT_INVALID",
            Self::ProviderAliasesInvalid { .. } => "VIA_REALTIME_PROVIDER_ALIASES_INVALID",
            Self::ProviderModelProfileInvalid { .. } => "VIA_REALTIME_PROVIDER_PROFILE_INVALID",
            Self::ProviderSessionDefaultsIncomplete { .. } => {
                "VIA_REALTIME_PROVIDER_SESSION_DEFAULTS_INCOMPLETE"
            }
            Self::UnsupportedModel { .. } => "VIA_REALTIME_MODEL_UNSUPPORTED",
            Self::NotConfigured { .. } => "VIA_REALTIME_NOT_CONFIGURED",
            Self::ConnectTimeout { .. } => "VIA_REALTIME_CONNECT_TIMEOUT",
            Self::ConnectionClosed { .. } => "VIA_REALTIME_CONNECTION_CLOSED",
            Self::ItemUnconfirmed { .. } => "VIA_REALTIME_ITEM_UNCONFIRMED",
            Self::ItemRejected { .. } => "VIA_REALTIME_ITEM_REJECTED",
            Self::ProviderRefused { .. } => "VIA_REALTIME_PROVIDER_REFUSED",
            Self::RequestCancelled => "VIA_REALTIME_REQUEST_CANCELLED",
            Self::SessionReset => "VIA_REALTIME_SESSION_RESET",
            Self::ResponseCorrelationConflict => "VIA_REALTIME_RESPONSE_CORRELATION_CONFLICT",
            Self::Transport { .. } => "VIA_REALTIME_TRANSPORT",
        }
    }

    /// Whether this failure came out of a provider event rather than out of VIA.
    ///
    /// Upstream's `error.realtimeEvent` flag (`realtime-provider.mjs:218`,
    /// `:569`). The session reports a response outcome for it but does **not**
    /// also emit it as a session error, so one provider refusal is surfaced once.
    #[must_use]
    pub const fn is_provider_event(&self) -> bool {
        matches!(
            self,
            Self::ProviderRefused { .. } | Self::ItemRejected { .. }
        )
    }

    /// The sentence a person reads.
    ///
    /// Every string comes from `via-i18n` except the three the module docs name.
    #[must_use]
    pub fn message(&self, locale: Locale) -> String {
        match self {
            Self::UnsupportedProvider {
                requested,
                available,
            } => format(
                locale,
                keys::REALTIME_UNSUPPORTED_FRONTEND_WITH_OPTIONS,
                &[
                    ("name", requested),
                    ("options", &join_provider_names(locale, available)),
                ],
            ),
            Self::ProviderNameTaken { name } => format(
                locale,
                keys::REALTIME_PROVIDER_NAME_TAKEN,
                &[("name", name)],
            ),
            Self::ProviderNeedsKeyAndLabel => {
                t(locale, keys::REALTIME_PROVIDER_NEEDS_KEY_AND_LABEL).to_owned()
            }
            Self::ProviderKeyInvalid { key } => {
                format(locale, keys::REALTIME_PROVIDER_KEY_INVALID, &[("key", key)])
            }
            Self::ProviderMissingInputSampleRate { key } => format(
                locale,
                keys::REALTIME_PROVIDER_MISSING_INPUT_SAMPLE_RATE,
                &[("key", key)],
            ),
            Self::ProviderMissingOutputSampleRate { key } => format(
                locale,
                keys::REALTIME_PROVIDER_MISSING_OUTPUT_SAMPLE_RATE,
                &[("key", key)],
            ),
            Self::ProviderResponseTimeoutInvalid { key } => format(
                locale,
                keys::REALTIME_PROVIDER_RESPONSE_TIMEOUT_INVALID,
                &[("key", key)],
            ),
            Self::ProviderAliasesInvalid { key } => format(
                locale,
                keys::REALTIME_PROVIDER_ALIASES_INVALID,
                &[("key", key)],
            ),
            Self::ProviderModelProfileInvalid { key } => format(
                locale,
                keys::REALTIME_PROVIDER_MODEL_PROFILE_INVALID,
                &[("key", key)],
            ),
            Self::ProviderSessionDefaultsIncomplete { key } => format(
                locale,
                keys::REALTIME_PROVIDER_SESSION_DEFAULTS_INCOMPLETE,
                &[("key", key)],
            ),
            Self::UnsupportedModel { id, label } => format(
                locale,
                keys::REALTIME_UNSUPPORTED_MODEL_WITH_PROVIDER,
                &[("id", id), ("label", label)],
            ),
            Self::NotConfigured { message, .. } | Self::ConnectTimeout { message, .. } => {
                message.clone()
            }
            Self::ConnectionClosed { label } => format(
                locale,
                keys::REALTIME_CONNECTION_CLOSED,
                &[("label", label)],
            ),
            Self::ItemUnconfirmed { label, id } => format(
                locale,
                keys::REALTIME_ITEM_UNCONFIRMED,
                &[("label", label), ("id", id)],
            ),
            Self::ItemRejected { label, detail } => detail.clone().unwrap_or_else(|| {
                format(
                    locale,
                    keys::REALTIME_ITEM_CREATE_FAILED,
                    &[("label", label)],
                )
            }),
            Self::ProviderRefused { message } => message.clone(),
            Self::RequestCancelled => t(locale, keys::REALTIME_REQUEST_CANCELLED).to_owned(),
            Self::SessionReset => t(locale, keys::REALTIME_SESSION_RESET).to_owned(),
            Self::ResponseCorrelationConflict => {
                t(locale, keys::REALTIME_RESPONSE_CORRELATION_CONFLICT).to_owned()
            }
            Self::Transport { detail } => detail.clone(),
        }
    }
}

/// Join provider names the way the reading locale expects.
///
/// Upstream joins with the ideographic comma `、` because its only locale is
/// `zh`; VIA has three, and `dashscope、mock` in an English sentence is wrong in
/// a way a reader notices. The `zh` rendering stays byte-identical to upstream's.
fn join_provider_names(locale: Locale, names: &[String]) -> String {
    let separator = match locale {
        Locale::Zh => "、",
        Locale::En | Locale::Ko => ", ",
    };
    names.join(separator)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn the_unsupported_provider_message_lists_what_is_available() {
        let error = RealtimeError::UnsupportedProvider {
            requested: "qwen3".into(),
            available: vec!["dashscope".into(), "speech-to-speech".into()],
        };
        assert_eq!(
            error.message(Locale::Zh),
            "不支持的 Realtime 前台：qwen3（可选 dashscope、speech-to-speech）"
        );
        assert_eq!(
            error.message(Locale::En),
            "unsupported Realtime frontend: qwen3 (available: dashscope, speech-to-speech)"
        );
    }

    #[test]
    fn a_provider_supplied_sentence_is_carried_not_translated() {
        let error = RealtimeError::NotConfigured {
            provider: "dashscope".into(),
            message: "请先配置 DASHSCOPE_API_KEY".into(),
        };
        assert_eq!(error.message(Locale::En), "请先配置 DASHSCOPE_API_KEY");
        assert_eq!(error.message(Locale::Ko), "请先配置 DASHSCOPE_API_KEY");
    }

    #[test]
    fn a_transport_sentence_survives_verbatim_so_classification_still_works() {
        let error = RealtimeError::Transport {
            detail: "Unexpected server response: 401".into(),
        };
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            assert_eq!(error.message(locale), "Unexpected server response: 401");
        }
    }

    #[test]
    fn a_rejected_item_prefers_the_providers_own_sentence() {
        let with_detail = RealtimeError::ItemRejected {
            label: "Qwen-Audio-Realtime".into(),
            detail: Some("invalid conversation item".into()),
        };
        assert_eq!(with_detail.message(Locale::En), "invalid conversation item");

        let without = RealtimeError::ItemRejected {
            label: "Qwen-Audio-Realtime".into(),
            detail: None,
        };
        assert_eq!(
            without.message(Locale::Zh),
            "Qwen-Audio-Realtime 创建对话项失败"
        );
    }

    #[test]
    fn only_provider_sourced_failures_are_marked_as_such() {
        assert!(
            RealtimeError::ProviderRefused {
                message: "boom".into()
            }
            .is_provider_event()
        );
        assert!(
            RealtimeError::ItemRejected {
                label: "L".into(),
                detail: None
            }
            .is_provider_event()
        );
        assert!(!RealtimeError::SessionReset.is_provider_event());
        assert!(
            !RealtimeError::Transport {
                detail: "boom".into()
            }
            .is_provider_event()
        );
    }

    #[test]
    fn every_variant_renders_in_every_locale_without_a_diagnostic() {
        let variants = [
            RealtimeError::UnsupportedProvider {
                requested: "x".into(),
                available: vec!["a".into()],
            },
            RealtimeError::ProviderNameTaken { name: "a".into() },
            RealtimeError::ProviderNeedsKeyAndLabel,
            RealtimeError::ProviderKeyInvalid { key: "A".into() },
            RealtimeError::ProviderMissingInputSampleRate { key: "a".into() },
            RealtimeError::ProviderMissingOutputSampleRate { key: "a".into() },
            RealtimeError::ProviderResponseTimeoutInvalid { key: "a".into() },
            RealtimeError::ProviderAliasesInvalid { key: "a".into() },
            RealtimeError::ProviderModelProfileInvalid { key: "a".into() },
            RealtimeError::ProviderSessionDefaultsIncomplete { key: "a".into() },
            RealtimeError::UnsupportedModel {
                id: "m".into(),
                label: "L".into(),
            },
            RealtimeError::NotConfigured {
                provider: "a".into(),
                message: "m".into(),
            },
            RealtimeError::ConnectTimeout {
                provider: "a".into(),
                message: "m".into(),
            },
            RealtimeError::ConnectionClosed { label: "L".into() },
            RealtimeError::ItemUnconfirmed {
                label: "L".into(),
                id: "i".into(),
            },
            RealtimeError::ItemRejected {
                label: "L".into(),
                detail: None,
            },
            RealtimeError::ProviderRefused {
                message: "m".into(),
            },
            RealtimeError::RequestCancelled,
            RealtimeError::SessionReset,
            RealtimeError::ResponseCorrelationConflict,
            RealtimeError::Transport { detail: "d".into() },
        ];
        for variant in &variants {
            for locale in [Locale::En, Locale::Zh, Locale::Ko] {
                let message = variant.message(locale);
                assert!(!message.is_empty(), "{variant:?} in {locale}");
                assert!(
                    !message.contains("<via-i18n:"),
                    "{variant:?} in {locale}: {message}"
                );
                assert!(!message.contains('{'), "{variant:?} in {locale}: {message}");
            }
        }
    }

    #[test]
    fn every_code_is_distinct_and_via_branded() {
        let codes = [
            RealtimeError::UnsupportedProvider {
                requested: String::new(),
                available: Vec::new(),
            }
            .code(),
            RealtimeError::ProviderNameTaken {
                name: String::new(),
            }
            .code(),
            RealtimeError::ProviderNeedsKeyAndLabel.code(),
            RealtimeError::ProviderKeyInvalid { key: String::new() }.code(),
            RealtimeError::ProviderMissingInputSampleRate { key: String::new() }.code(),
            RealtimeError::ProviderMissingOutputSampleRate { key: String::new() }.code(),
            RealtimeError::ProviderResponseTimeoutInvalid { key: String::new() }.code(),
            RealtimeError::ProviderAliasesInvalid { key: String::new() }.code(),
            RealtimeError::ProviderModelProfileInvalid { key: String::new() }.code(),
            RealtimeError::ProviderSessionDefaultsIncomplete { key: String::new() }.code(),
            RealtimeError::UnsupportedModel {
                id: String::new(),
                label: String::new(),
            }
            .code(),
            RealtimeError::NotConfigured {
                provider: String::new(),
                message: String::new(),
            }
            .code(),
            RealtimeError::ConnectTimeout {
                provider: String::new(),
                message: String::new(),
            }
            .code(),
            RealtimeError::ConnectionClosed {
                label: String::new(),
            }
            .code(),
            RealtimeError::ItemUnconfirmed {
                label: String::new(),
                id: String::new(),
            }
            .code(),
            RealtimeError::ItemRejected {
                label: String::new(),
                detail: None,
            }
            .code(),
            RealtimeError::ProviderRefused {
                message: String::new(),
            }
            .code(),
            RealtimeError::RequestCancelled.code(),
            RealtimeError::SessionReset.code(),
            RealtimeError::ResponseCorrelationConflict.code(),
            RealtimeError::Transport {
                detail: String::new(),
            }
            .code(),
        ];
        let mut sorted = codes.to_vec();
        sorted.sort_unstable();
        let mut unique = sorted.clone();
        unique.dedup();
        assert_eq!(sorted.len(), unique.len(), "codes must be distinct");
        for code in codes {
            assert!(code.starts_with("VIA_REALTIME_"), "{code}");
        }
    }
}
