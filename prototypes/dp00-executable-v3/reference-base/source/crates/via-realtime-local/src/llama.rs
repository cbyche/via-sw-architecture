//! The real reasoning turn — `--features llama` only.
//!
//! `llama-cpp-2` over a Qwen3 GGUF, behind [`Responder`]. Same rule as
//! [`crate::sherpa`] and the same reason: `llama-cpp-sys-2` is a native C++
//! tree, `docs/adr/0001-placement.md` records that VIA's native dependencies
//! are exactly what keeps it out of ARGO's workspace, and a C toolchain in
//! every CI lane for a feature most builds never touch is that decision made
//! badly at a smaller scale.
//!
//! # Generation is blocking; the trait wants a droppable stream
//!
//! llama.cpp decodes on the calling thread. So one turn is one
//! [`spawn_blocking`](tokio::task::spawn_blocking) task writing into an
//! unbounded channel, and the [`ResponseStream`] the machine holds is the
//! receiving half. That gives the three properties [`crate::stages`] requires,
//! and each one falls out of the channel rather than being arranged:
//!
//! | Requirement | How |
//! | --- | --- |
//! | dropping the stream cancels the turn | the receiver is gone, the next `send` fails, the loop breaks |
//! | a dropped stream must not block | the drop closes a channel; the decode thread notices at its next token |
//! | a dropped stream must be inert | nothing can be sent to a closed channel |
//!
//! The **grain of the cancellation is one token**, which for a laptop-sized
//! model is tens of milliseconds. That is what makes barge-in feel immediate
//! even though llama.cpp has no interrupt of its own.
//!
//! # Two bounds, both deliberate
//!
//! [`LlamaOptions::max_tokens`] bounds a turn, and it is not a tuning knob: a
//! model that never emits its end-of-generation token would otherwise decode
//! until the context filled, holding the response slot and the speaker with it.
//! [`LlamaOptions::context_tokens`] bounds the KV cache, and the prompt is
//! **trimmed from the oldest message** to fit rather than being sent and
//! rejected — a context overflow arrives from llama.cpp as a decode error with
//! no indication that history was the cause.
//!
//! # Partial UTF-8 is buffered, not lossily decoded
//!
//! A token is a byte sequence, not a character: a CJK glyph or an emoji spans
//! two or three tokens. Decoding each token with `from_utf8_lossy` would put a
//! `U+FFFD` on the wire for every one of them — visible in the transcript, and
//! fed to the synthesizer. So bytes accumulate and only the valid UTF-8 prefix
//! is emitted.
//!
//! # What this stage does not do
//!
//! **It does not author tool calls.** A GGUF emits them as text in its own
//! model-specific format, and parsing that is a per-model concern rather than a
//! transport one — a wrong parser silently turns a tool call into spoken prose.
//! [`ResponseDelta::Tool`] is part of the stage *contract* and the machine
//! publishes it correctly (`response.output_item.added` plus
//! `response.function_call_arguments.done`); a deployment that wants tool calls
//! from a local model supplies a [`Responder`] that wraps this one and parses
//! its model's format. `docs/deviations/phase-8-via-realtime-local.md` records
//! it.
//!
//! # Not verified against real weights in this repository
//!
//! Written against `llama-cpp-2` 0.1.154's own sources. No GGUF is checked in
//! and no lane here builds the native library, so **this stage has not been run
//! against a real model in this repository.** The seam that has is
//! [`crate::scripted`].

use std::num::NonZeroU32;
use std::sync::{Arc, OnceLock};

use futures::stream;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;

use crate::error::{LocalError, Result};
use crate::stages::{Responder, ResponseDelta, ResponseStream, Stage, StageError, Turn, TurnRole};
use crate::weights::WeightsSet;

/// The longest turn this stage will generate, in tokens.
///
/// A bound, not a budget. See the module docs.
pub const DEFAULT_MAX_TOKENS: u32 = 512;

/// The KV-cache size a context is opened with, in tokens.
pub const DEFAULT_CONTEXT_TOKENS: u32 = 4_096;

/// Sampling temperature.
pub const DEFAULT_TEMPERATURE: f32 = 0.7;

/// Nucleus-sampling mass.
pub const DEFAULT_TOP_P: f32 = 0.9;

/// The sampler seed.
///
/// Fixed rather than drawn from the clock: two runs of the same session on the
/// same weights produce the same words, which is what makes a local pipeline
/// debuggable at all. A deployment that wants variety sets its own.
pub const DEFAULT_SEED: u32 = 0x5649_4100;

/// The role string a system message carries into the chat template.
pub const ROLE_SYSTEM: &str = "system";

/// The role string a user message carries.
pub const ROLE_USER: &str = "user";

/// The role string an assistant message carries.
pub const ROLE_ASSISTANT: &str = "assistant";

/// What [`LlamaResponder`] is built with.
#[derive(Debug, Clone, PartialEq)]
pub struct LlamaOptions {
    /// GPU layers to offload. `0` is CPU-only, which is the shipping default on
    /// macOS arm64 where Metal is chosen by the build rather than by a count.
    pub gpu_layers: u32,
    /// KV-cache size, in tokens.
    pub context_tokens: u32,
    /// The longest turn, in tokens.
    pub max_tokens: u32,
    /// Decode threads. `0` lets llama.cpp choose.
    pub threads: i32,
    /// Sampling temperature.
    pub temperature: f32,
    /// Nucleus-sampling mass.
    pub top_p: f32,
    /// Sampler seed.
    pub seed: u32,
}

impl Default for LlamaOptions {
    fn default() -> Self {
        Self {
            gpu_layers: 0,
            context_tokens: DEFAULT_CONTEXT_TOKENS,
            max_tokens: DEFAULT_MAX_TOKENS,
            threads: 0,
            temperature: DEFAULT_TEMPERATURE,
            top_p: DEFAULT_TOP_P,
            seed: DEFAULT_SEED,
        }
    }
}

/// The process-wide llama.cpp backend.
///
/// `LlamaBackend::init` may be called **once per process** and answers
/// `BackendAlreadyInitialized` afterwards, so a second `LlamaResponder` in the
/// same Gateway would refuse to load a model for no reason a user could act on.
/// One `OnceLock` removes that failure mode; the error, if the first
/// initialization failed, is reported to every caller rather than to the first
/// one only.
fn backend() -> core::result::Result<&'static LlamaBackend, StageError> {
    static BACKEND: OnceLock<core::result::Result<LlamaBackend, String>> = OnceLock::new();
    BACKEND
        .get_or_init(|| LlamaBackend::init().map_err(|error| error.to_string()))
        .as_ref()
        .map_err(|detail| StageError::new(Stage::Reasoning, detail.clone()))
}

/// A Qwen3 GGUF behind [`Responder`].
pub struct LlamaResponder {
    model: Arc<LlamaModel>,
    options: LlamaOptions,
    path: std::path::PathBuf,
}

impl core::fmt::Debug for LlamaResponder {
    /// Hand-written because `LlamaModel` is a raw pointer into the C++ library
    /// and is not `Debug`.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LlamaResponder")
            .field("path", &self.path)
            .field("options", &self.options)
            .finish_non_exhaustive()
    }
}

impl LlamaResponder {
    /// Load the reasoning model named by `weights`.
    ///
    /// # Errors
    ///
    /// [`LocalError::ModelFileMissing`] when the GGUF is not on disk — checked
    /// before the library is asked, because `load_from_file` on an absent path
    /// is a `debug_assert` in a debug build and an opaque load failure in a
    /// release one. [`LocalError::Stage`] when the backend or the load refuses.
    pub fn open(weights: &WeightsSet, options: LlamaOptions) -> Result<Self> {
        if !weights.reasoning.is_file() {
            return Err(LocalError::model_file_missing(
                Stage::Reasoning,
                &weights.reasoning,
            ));
        }
        let backend = backend()?;
        let params = LlamaModelParams::default().with_n_gpu_layers(options.gpu_layers);
        let model =
            LlamaModel::load_from_file(backend, &weights.reasoning, &params).map_err(|error| {
                LocalError::from(StageError::new(
                    Stage::Reasoning,
                    format!("{}: {error}", weights.reasoning.display()),
                ))
            })?;
        Ok(Self {
            model: Arc::new(model),
            options,
            path: weights.reasoning.clone(),
        })
    }

    /// The options this stage was built with.
    #[must_use]
    pub fn options(&self) -> &LlamaOptions {
        &self.options
    }

    /// The chat messages a turn becomes, oldest first.
    ///
    /// Public because it is the one part of this stage that is pure, and the
    /// one a deployment is most likely to want to assert: a turn whose
    /// `response_instructions` were dropped answers the *previous* user turn
    /// again, which is the failure mode of a realtime provider that ignores
    /// `response.create.instructions`.
    #[must_use]
    pub fn messages(turn: &Turn) -> Vec<(&'static str, String)> {
        let mut messages = Vec::new();
        if !turn.instructions.trim().is_empty() {
            messages.push((ROLE_SYSTEM, turn.instructions.clone()));
        }
        for message in &turn.history {
            let role = match message.role {
                TurnRole::User => ROLE_USER,
                TurnRole::Assistant => ROLE_ASSISTANT,
            };
            messages.push((role, message.text.clone()));
        }
        if let Some(instructions) = &turn.response_instructions {
            // A per-response instruction is a **system** turn, not a user one:
            // it is the Gateway telling the model how to speak, and a model that
            // read it as the user's words would answer it instead of obeying it.
            messages.push((ROLE_SYSTEM, instructions.clone()));
        }
        if !turn.transcript.trim().is_empty() {
            messages.push((ROLE_USER, turn.transcript.clone()));
        }
        messages
    }

    /// The prompt text a turn becomes.
    ///
    /// The model's own chat template when it has one — a Qwen3 GGUF does — and
    /// a plain `role: text` transcript when it does not. The fallback is
    /// deliberately dull: a template guessed wrong produces a model that answers
    /// its own system prompt, which reads as a broken persona rather than as a
    /// configuration error.
    fn prompt(&self, turn: &Turn) -> core::result::Result<(String, bool), StageError> {
        let messages = Self::messages(turn);
        let Ok(template) = self.model.chat_template(None) else {
            let rendered = messages
                .iter()
                .map(|(role, text)| format!("{role}: {text}"))
                .collect::<Vec<_>>()
                .join("\n");
            return Ok((format!("{rendered}\n{ROLE_ASSISTANT}: "), true));
        };
        let chat: Vec<LlamaChatMessage> = messages
            .into_iter()
            .filter_map(|(role, text)| LlamaChatMessage::new(role.to_owned(), text).ok())
            .collect();
        self.model
            .apply_chat_template(&template, &chat, true)
            .map(|prompt| (prompt, false))
            .map_err(|error| StageError::new(Stage::Reasoning, format!("chat template: {error}")))
    }
}

#[async_trait::async_trait]
impl Responder for LlamaResponder {
    async fn respond(&self, turn: Turn) -> core::result::Result<ResponseStream, StageError> {
        let (prompt, add_bos) = self.prompt(&turn)?;
        let model = Arc::clone(&self.model);
        let options = self.options.clone();
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<
            core::result::Result<ResponseDelta, StageError>,
        >();

        tokio::task::spawn_blocking(move || {
            if let Err(error) = generate(&model, &options, &prompt, add_bos, &sender) {
                let _ = sender.send(Err(error));
            }
        });

        Ok(Box::pin(stream::unfold(
            receiver,
            |mut receiver| async move { receiver.recv().await.map(|delta| (delta, receiver)) },
        )))
    }
}

/// One blocking decode loop.
///
/// Returns as soon as the receiver is gone, which is how a barge-in reaches a
/// library with no interrupt of its own.
fn generate(
    model: &LlamaModel,
    options: &LlamaOptions,
    prompt: &str,
    add_bos: bool,
    sender: &tokio::sync::mpsc::UnboundedSender<core::result::Result<ResponseDelta, StageError>>,
) -> core::result::Result<(), StageError> {
    use llama_cpp_2::model::AddBos;

    let fail = |detail: String| StageError::new(Stage::Reasoning, detail);

    let backend = backend()?;
    let context_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(options.context_tokens))
        .with_n_batch(options.context_tokens)
        .with_n_threads(options.threads);
    let mut context = model
        .new_context(backend, context_params)
        .map_err(|error| fail(format!("context: {error}")))?;

    let bos = if add_bos {
        AddBos::Always
    } else {
        AddBos::Never
    };
    let mut tokens = model
        .str_to_token(prompt, bos)
        .map_err(|error| fail(format!("tokenize: {error}")))?;

    // Trim from the oldest end so a long conversation degrades rather than
    // failing: llama.cpp reports a context overflow as a decode error with no
    // indication that history was the cause.
    let room = usize::try_from(
        options
            .context_tokens
            .saturating_sub(options.max_tokens)
            .max(1),
    )
    .unwrap_or(usize::MAX);
    if tokens.len() > room {
        tokens.drain(..tokens.len() - room);
    }
    if tokens.is_empty() {
        return Ok(());
    }

    let capacity = usize::try_from(options.context_tokens).unwrap_or(tokens.len());
    let mut batch = LlamaBatch::new(capacity.max(tokens.len()), 1);
    let last = tokens.len() - 1;
    for (index, token) in tokens.iter().enumerate() {
        let position = i32::try_from(index).map_err(|_| fail("prompt is too long".to_owned()))?;
        batch
            .add(*token, position, &[0], index == last)
            .map_err(|error| fail(format!("batch: {error}")))?;
    }
    context
        .decode(&mut batch)
        .map_err(|error| fail(format!("decode: {error}")))?;

    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::top_p(options.top_p, 1),
        LlamaSampler::temp(options.temperature),
        LlamaSampler::dist(options.seed),
    ]);

    let mut position = i32::try_from(tokens.len()).unwrap_or(i32::MAX);
    let mut pending: Vec<u8> = Vec::new();
    // The bound. A model that never emits its end-of-generation token stops
    // here rather than decoding until the context fills.
    for _ in 0..options.max_tokens {
        let token = sampler.sample(&context, batch.n_tokens() - 1);
        sampler.accept(token);
        if model.is_eog_token(token) {
            break;
        }

        let bytes = model
            .token_to_piece_bytes(token, 32, false, None)
            .map_err(|error| fail(format!("detokenize: {error}")))?;
        pending.extend_from_slice(&bytes);
        // Only the valid UTF-8 prefix: a CJK glyph spans two or three tokens,
        // and `from_utf8_lossy` per token would put a replacement character on
        // the wire for each one.
        let valid = match core::str::from_utf8(&pending) {
            Ok(_) => pending.len(),
            Err(error) => error.valid_up_to(),
        };
        if valid > 0 {
            let text = String::from_utf8_lossy(&pending[..valid]).into_owned();
            pending.drain(..valid);
            if sender.send(Ok(ResponseDelta::Text(text))).is_err() {
                // The machine dropped the stream: barge-in, or the session
                // closed. Nothing left to generate for.
                return Ok(());
            }
        }

        batch.clear();
        batch
            .add(token, position, &[0], true)
            .map_err(|error| fail(format!("batch: {error}")))?;
        position = position.saturating_add(1);
        context
            .decode(&mut batch)
            .map_err(|error| fail(format!("decode: {error}")))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::stages::TurnMessage;

    #[test]
    fn a_missing_gguf_is_named_before_the_library_is_asked() {
        let weights = WeightsSet::resolve("/nonexistent-model-root");
        assert_eq!(
            LlamaResponder::open(&weights, LlamaOptions::default()).err(),
            Some(LocalError::model_file_missing(
                Stage::Reasoning,
                &weights.reasoning
            ))
        );
    }

    #[test]
    fn a_turn_becomes_system_history_and_the_new_utterance_in_order() {
        let turn = Turn {
            instructions: "You are VIA.".to_owned(),
            response_instructions: None,
            transcript: "turn on the lights".to_owned(),
            history: vec![
                TurnMessage::user("hello"),
                TurnMessage::assistant("hi there"),
            ],
            tools: Vec::new(),
        };
        assert_eq!(
            LlamaResponder::messages(&turn),
            vec![
                (ROLE_SYSTEM, "You are VIA.".to_owned()),
                (ROLE_USER, "hello".to_owned()),
                (ROLE_ASSISTANT, "hi there".to_owned()),
                (ROLE_USER, "turn on the lights".to_owned()),
            ]
        );
    }

    #[test]
    fn per_response_instructions_are_a_system_turn_and_survive() {
        // The failure this prevents: a provider that drops
        // `response.create.instructions` answers the previous user turn again
        // instead of reading the finished result aloud.
        let turn = Turn {
            instructions: "You are VIA.".to_owned(),
            response_instructions: Some("Read the result aloud.".to_owned()),
            transcript: String::new(),
            history: vec![TurnMessage::user("build the thing")],
            tools: Vec::new(),
        };
        assert_eq!(
            LlamaResponder::messages(&turn),
            vec![
                (ROLE_SYSTEM, "You are VIA.".to_owned()),
                (ROLE_USER, "build the thing".to_owned()),
                (ROLE_SYSTEM, "Read the result aloud.".to_owned()),
            ]
        );
    }

    #[test]
    fn an_empty_turn_carries_no_blank_messages() {
        assert_eq!(LlamaResponder::messages(&Turn::default()), Vec::new());
        let whitespace = Turn {
            instructions: "  ".to_owned(),
            transcript: "\n".to_owned(),
            ..Turn::default()
        };
        assert_eq!(LlamaResponder::messages(&whitespace), Vec::new());
    }

    #[test]
    fn the_generation_bound_is_smaller_than_the_context() {
        assert!(DEFAULT_MAX_TOKENS < DEFAULT_CONTEXT_TOKENS);
        assert!(LlamaOptions::default().max_tokens > 0);
    }
}
