//! The model's token inventory, and the guard that stands in front of the
//! engine.
//!
//! # Why this module exists
//!
//! `sherpa-onnx` parses a keyword file line by whitespace. Each word is either a
//! symbol from the model's own `tokens.txt`, or a marker: `:` a per-keyword
//! boost, `#` a per-keyword threshold, `@` the display label. **A word that is
//! neither is a fatal error** — and not a returned one. `EncodeBase` logs
//! *"Cannot find ID for token … (Hint: Check the tokens.txt see if … in it)"*,
//! `InitKeywords` logs *"Encode keywords failed."*, and the C++ library then
//! ends the process. A `KeywordSpotter::create` that could return `None`
//! never gets the chance.
//!
//! There is a second way in. A `:score` or `#threshold` word is parsed with
//! `std::stof(word.substr(1))` and **no `try`**, so `:later` is not a rejected
//! score — it is an uncaught `std::invalid_argument`.
//!
//! That is the whole reason this module is here. VIA's wake phrase is
//! configuration (`docs/architecture.md` §16), so the keyword file is a value
//! an operator can get wrong, and getting it wrong must not take the Gateway
//! down with it. [`TokenInventory::validate`] reads the same file the engine
//! would read, applies the same rules, and answers
//! [`WakeWordError::UnknownTokens`](crate::WakeWordError::UnknownTokens) or
//! [`WakeWordError::MalformedMarker`](crate::WakeWordError::MalformedMarker) —
//! naming every word at fault — before the library is called at all.
//!
//! It runs at two points, deliberately: at **install**, so a keyword file that
//! would kill the Gateway is never written next to the model, and at **open**,
//! because a caller can assemble a [`DetectionConfig`](crate::DetectionConfig)
//! by hand.
//!
//! # Agreeing with the library in both directions
//!
//! Rejecting a keyword file the engine would accept turns a working wake word
//! off; accepting one it would reject ends the process. So the classification
//! follows `EncodeBase`'s own control flow — *inventory first, marker second,
//! otherwise out of vocabulary* — and it is lenient exactly where `std::stof`
//! is lenient. See [`TokenInventory::classify`] and [`has_numeric_prefix`].
//!
//! # The file
//!
//! `tokens.txt` is `<symbol> <id>`, one per line. The catalogued zh-en model
//! has 263 of them: ARPAbet phonemes with stress digits for English (`AY1`,
//! `AH1`, `T`), tone-marked pinyin initials and finals for Chinese (`n`, `ǐ`,
//! `h`, `ǎo`), plus `<blk>`, `<sos/eos>` and `<unk>`.

use std::collections::BTreeSet;
use std::path::Path;

use crate::error::{Result, WakeWordError};

/// The three characters that begin a keyword-file marker rather than a token.
///
/// **External contract** — `sherpa-onnx/csrc/utils.cc`, `EncodeBase`:
/// `':'` boost score, `'#'` trigger threshold, `'@'` display label. Everything
/// else must be a symbol in `tokens.txt`.
pub const KEYWORD_MARKERS: [char; 3] = [':', '#', '@'];

/// The symbols one model can encode.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TokenInventory {
    tokens: BTreeSet<String>,
}

impl TokenInventory {
    /// Parse a `tokens.txt`.
    ///
    /// Each line is `<symbol> <id>`; the symbol is the first whitespace-
    /// separated field. A line with a single field is the library's own
    /// encoding of the space symbol and carries no symbol name, so it is
    /// skipped rather than admitted as a token whose name is a number.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let tokens = text
            .lines()
            .filter_map(|line| {
                let mut fields = line.split_whitespace();
                let symbol = fields.next()?;
                // Require the id field, so a stray line cannot add a symbol.
                fields.next()?;
                Some(symbol.to_owned())
            })
            .collect();
        Self { tokens }
    }

    /// Read and parse a `tokens.txt` from disk.
    ///
    /// # Errors
    ///
    /// [`WakeWordError::Io`] if the file cannot be read.
    pub fn read(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| WakeWordError::io("read", path, &error))?;
        Ok(Self::parse(&text))
    }

    /// Whether `symbol` is in the inventory.
    #[must_use]
    pub fn contains(&self, symbol: &str) -> bool {
        self.tokens.contains(symbol)
    }

    /// How many symbols the model knows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Whether the inventory is empty — which a real `tokens.txt` never is.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Every word in `keywords_file` the model cannot encode, in order, without
    /// repeats.
    #[must_use]
    pub fn unknown_tokens(&self, keywords_file: &str) -> Vec<String> {
        self.collect(keywords_file, |word| matches!(word, Word::Unknown))
    }

    /// Every word in `keywords_file` that looks like a `:score` or `#threshold`
    /// marker but carries nothing `std::stof` could read.
    ///
    /// The second way a keyword file ends the process. `EncodeBase` reaches
    /// `std::stof(word.substr(1))` with no `try`, so `:later` is not a rejected
    /// score — it is an uncaught `std::invalid_argument`.
    #[must_use]
    pub fn malformed_markers(&self, keywords_file: &str) -> Vec<String> {
        self.collect(keywords_file, |word| matches!(word, Word::MalformedMarker))
    }

    /// Fail unless every word of `keywords_file` is one the library can read.
    ///
    /// # Errors
    ///
    /// - [`WakeWordError::UnknownTokens`], naming every word that is neither a
    ///   symbol nor a marker.
    /// - [`WakeWordError::MalformedMarker`], naming every `:`/`#` word whose
    ///   payload is not a number.
    ///
    /// Unknown tokens are reported first: they are far and away the common
    /// fault, and a file with both is more usefully described by the one an
    /// operator will recognise.
    pub fn validate(&self, keywords_file: &str) -> Result<()> {
        let unknown = self.unknown_tokens(keywords_file);
        if !unknown.is_empty() {
            return Err(WakeWordError::UnknownTokens { unknown });
        }
        let malformed = self.malformed_markers(keywords_file);
        if !malformed.is_empty() {
            return Err(WakeWordError::MalformedMarker { malformed });
        }
        Ok(())
    }

    /// Walk the file once, keeping the words of one class, in order, without
    /// repeats.
    fn collect(&self, keywords_file: &str, wanted: impl Fn(Word) -> bool) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for word in keywords_file.split_whitespace() {
            if !wanted(self.classify(word)) {
                continue;
            }
            if !out.iter().any(|seen| seen == word) {
                out.push(word.to_owned());
            }
        }
        out
    }

    /// What `EncodeBase` would make of one whitespace-separated word.
    ///
    /// The order of the checks is the library's, and it is load-bearing: the
    /// inventory is consulted **first**, so a model whose `tokens.txt` happened
    /// to contain a marker-shaped symbol like `#hash` encodes it as a symbol
    /// rather than trying to read `hash` as a threshold. Reversing the two
    /// would reject a keyword file the engine accepts, which turns a working
    /// wake word off.
    fn classify(&self, word: &str) -> Word {
        if self.contains(word) {
            return Word::Symbol;
        }
        match word.chars().next() {
            // `:` and `#` carry a number. `@` carries the display label, which
            // is any string at all.
            Some(':' | '#') => {
                if has_numeric_prefix(&word[1..]) {
                    Word::Marker
                } else {
                    Word::MalformedMarker
                }
            }
            Some('@') => Word::Marker,
            _ => Word::Unknown,
        }
    }
}

/// What one word of a keyword file is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Word {
    /// A symbol the model can encode.
    Symbol,
    /// A well-formed `:`, `#` or `@` marker.
    Marker,
    /// Neither — `EncodeBase` reports it and gives up.
    Unknown,
    /// A `:` or `#` whose payload `std::stof` would throw on.
    MalformedMarker,
}

/// Whether `std::stof` would read a number off the front of `payload`.
///
/// Deliberately an approximation, and deliberately the *lenient* one: `stof`
/// converts a valid prefix and ignores the rest, so `2.0dB` is a score of 2.0
/// to the library and must be one here too. What it rejects is the shape that
/// makes `stof` throw — no numeric prefix at all.
///
/// It does not accept `inf`, `nan` or C's hex-float syntax, which `stof` would.
/// Rejecting those over-rejects by exactly three spellings nobody writes as a
/// keyword boost, and over-rejection fails safe: it turns the wake word off
/// with a message naming the word, where under-rejection ends the process.
fn has_numeric_prefix(payload: &str) -> bool {
    let digits = payload.strip_prefix(['+', '-']).unwrap_or(payload);
    let mut characters = digits.chars();
    match characters.next() {
        Some(first) if first.is_ascii_digit() => true,
        Some('.') => characters.next().is_some_and(|next| next.is_ascii_digit()),
        _ => false,
    }
}
