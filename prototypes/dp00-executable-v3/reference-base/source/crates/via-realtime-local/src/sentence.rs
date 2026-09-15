//! Cutting a token stream into utterances, one sentence at a time.
//!
//! # Why this is not an implementation detail
//!
//! A cascade is three request/response calls; a realtime session is full duplex.
//! The single largest part of that gap is *when synthesis starts*. Waiting for
//! the reasoning turn to finish before speaking a word adds the whole
//! generation to the time-to-first-audio, and on a laptop that is seconds, not
//! milliseconds — the difference between a voice agent and a form submission.
//!
//! So [`crate::machine`] hands each finished sentence to the speaker while the
//! reasoning stage is still generating the next one. This module is the boundary
//! finder, and it is separate because a boundary finder is exactly the kind of
//! thing that is easy to write, easy to get subtly wrong, and impossible to
//! debug from inside an async state machine.
//!
//! # The three rules
//!
//! 1. **A hard terminator ends a sentence immediately.** `\n`, and the CJK stops
//!    `。！？…` — none of them is ever used inside a word, so no lookahead is
//!    needed.
//! 2. **A soft terminator needs one character of lookahead.** `.`, `!` and `?`
//!    end a sentence only when what follows is whitespace. `3.14`, `e.g` and a
//!    URL all carry a `.` that is not a boundary, and speaking each fragment as
//!    its own utterance puts a pause in the middle of a number. When the
//!    lookahead is not there yet the text is **held**, which is why
//!    [`SentenceSplitter::flush`] exists: the end of the stream is the missing
//!    lookahead.
//! 3. **A sentence is bounded.** A model that emits no punctuation at all —
//!    a list, a code block, a language whose spacing the tokenizer lost — must
//!    not grow the buffer forever and must not delay speech forever, so at
//!    [`MAX_SENTENCE_CHARS`] the text is cut at the last space and, failing
//!    that, at the cap. It is the bound that makes "the pipeline eventually
//!    speaks" a property rather than a hope.

/// Terminators that end a sentence with no lookahead.
///
/// `…` is here rather than with the soft terminators because it is a single
/// character: unlike `...`, it cannot be the middle of a number.
pub const HARD_TERMINATORS: [char; 5] = ['\n', '。', '！', '？', '…'];

/// Terminators that end a sentence only when whitespace follows.
pub const SOFT_TERMINATORS: [char; 3] = ['.', '!', '?'];

/// The longest run of unpunctuated text that will be held before it is cut.
///
/// Roughly fifteen seconds of speech. Long enough that no ordinary sentence is
/// ever split by it, short enough that a model emitting an unpunctuated list
/// starts being spoken rather than buffering to the end of the turn.
pub const MAX_SENTENCE_CHARS: usize = 240;

/// An incremental sentence boundary finder.
///
/// Deterministic and allocation-frugal: one `String` that only ever shrinks from
/// the front.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SentenceSplitter {
    buffer: String,
}

impl SentenceSplitter {
    /// An empty splitter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether anything is held.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.trim().is_empty()
    }

    /// What is held but not yet emitted.
    #[must_use]
    pub fn pending(&self) -> &str {
        &self.buffer
    }

    /// Append a delta and take every sentence it completed.
    ///
    /// The vector is usually empty (a delta is a token) and occasionally holds
    /// more than one (a delta that carried a whole paragraph).
    pub fn push(&mut self, delta: &str) -> Vec<String> {
        self.buffer.push_str(delta);
        let mut out = Vec::new();
        // Each `take` removes at least one byte from the front of the buffer,
        // so this terminates on a buffer of any length.
        while let Some(sentence) = self.take(false) {
            out.push(sentence);
        }
        out
    }

    /// Take whatever is left, terminated or not.
    ///
    /// The end of a reasoning turn is the lookahead a trailing soft terminator
    /// was waiting for, and a turn that ended mid-clause still has to be spoken.
    pub fn flush(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        while let Some(sentence) = self.take(true) {
            out.push(sentence);
        }
        out
    }

    /// Throw away everything held.
    ///
    /// Barge-in: the sentence the model was halfway through is not spoken, and
    /// must not surface on the next turn.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// One sentence off the front, if there is one.
    ///
    /// `at_end` means no more text is coming, which turns a held soft terminator
    /// into a boundary and an unterminated tail into a sentence.
    fn take(&mut self, at_end: bool) -> Option<String> {
        let cut = self.boundary(at_end)?;
        let rest = self.buffer.split_off(cut);
        let sentence = core::mem::replace(&mut self.buffer, rest);
        let trimmed = sentence.trim().to_owned();
        if trimmed.is_empty() {
            // Whitespace between two sentences is not an utterance. Recurse
            // rather than return `None`: the buffer strictly shrank, so this
            // cannot loop.
            return self.take(at_end);
        }
        Some(trimmed)
    }

    /// The byte index one past the end of the first sentence.
    fn boundary(&self, at_end: bool) -> Option<usize> {
        let mut chars = self.buffer.char_indices().peekable();
        let mut counted = 0usize;
        let mut last_space: Option<usize> = None;

        while let Some((index, character)) = chars.next() {
            counted += 1;
            if character.is_whitespace() {
                last_space = Some(index + character.len_utf8());
            }
            if HARD_TERMINATORS.contains(&character) {
                return Some(index + character.len_utf8());
            }
            if SOFT_TERMINATORS.contains(&character) {
                match chars.peek() {
                    // `!?` and `...` end together rather than one at a time.
                    Some((_, next)) if SOFT_TERMINATORS.contains(next) => continue,
                    Some((_, next)) if next.is_whitespace() => {
                        return Some(index + character.len_utf8());
                    }
                    // A letter or a digit after the dot: `3.14`, `e.g`, a URL.
                    Some(_) => continue,
                    // Nothing after it yet. At the end of the stream that is the
                    // boundary; otherwise hold and wait for the lookahead.
                    None if at_end => return Some(index + character.len_utf8()),
                    None => return None,
                }
            }
            if counted >= MAX_SENTENCE_CHARS {
                // The bound. Prefer a word boundary; a language the tokenizer
                // gave no spaces has none, and then the cap itself is the cut —
                // it is already on a char boundary because it came from
                // `char_indices`.
                return Some(last_space.unwrap_or(index + character.len_utf8()));
            }
        }

        // Nothing terminated. At the end of the stream the tail is a sentence.
        (at_end && !self.buffer.is_empty()).then_some(self.buffer.len())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn split(chunks: &[&str]) -> Vec<String> {
        let mut splitter = SentenceSplitter::new();
        let mut out = Vec::new();
        for chunk in chunks {
            out.extend(splitter.push(chunk));
        }
        out.extend(splitter.flush());
        assert!(splitter.is_empty(), "the splitter must end empty");
        out
    }

    #[test]
    fn one_sentence_arriving_token_by_token_is_emitted_once() {
        assert_eq!(
            split(&["On", " it", ".", " "]),
            vec!["On it.".to_owned()],
            "the delta boundaries must not become sentence boundaries"
        );
    }

    #[test]
    fn a_soft_terminator_waits_for_its_lookahead() {
        let mut splitter = SentenceSplitter::new();
        assert_eq!(splitter.push("Done."), Vec::<String>::new(), "held");
        assert_eq!(splitter.pending(), "Done.");
        assert_eq!(splitter.push(" Next"), vec!["Done.".to_owned()]);
        assert_eq!(splitter.flush(), vec!["Next".to_owned()]);
    }

    #[test]
    fn a_dot_inside_a_number_or_a_word_is_not_a_boundary() {
        for text in [
            "it is 3.14 exactly. ",
            "see example.com now. ",
            "version 1.2.3 shipped. ",
        ] {
            assert_eq!(
                split(&[text]),
                vec![text.trim().to_owned()],
                "a dot followed by a letter or a digit is inside a token"
            );
        }
    }

    /// The one case the lookahead rule gets wrong, stated rather than hidden.
    ///
    /// `e.g. ` is a dot followed by whitespace, so it reads as a boundary and
    /// the utterance is split. The alternative — an abbreviation dictionary —
    /// is per-locale, unbounded, and wrong in a different place; the cost of
    /// this one is an extra breath in the middle of a sentence, which is
    /// strictly better than a pause in the middle of `3.14`.
    #[test]
    fn an_abbreviation_followed_by_a_space_is_a_known_false_boundary() {
        assert_eq!(
            split(&["Bring a cable, e.g. USB-C. "]),
            vec!["Bring a cable, e.g.".to_owned(), "USB-C.".to_owned()]
        );
    }

    #[test]
    fn a_hard_terminator_needs_no_lookahead() {
        let mut splitter = SentenceSplitter::new();
        assert_eq!(splitter.push("好的。"), vec!["好的。".to_owned()]);
        assert_eq!(splitter.push("知道了！"), vec!["知道了！".to_owned()]);
        assert_eq!(splitter.push("真的？"), vec!["真的？".to_owned()]);
        assert_eq!(splitter.push("line\n"), vec!["line".to_owned()]);
        assert!(splitter.is_empty());
    }

    #[test]
    fn a_run_of_soft_terminators_ends_together() {
        assert_eq!(split(&["Really!? "]), vec!["Really!?".to_owned()]);
        assert_eq!(split(&["Wait... "]), vec!["Wait...".to_owned()]);
    }

    #[test]
    fn one_delta_carrying_a_paragraph_yields_every_sentence() {
        let mut splitter = SentenceSplitter::new();
        assert_eq!(
            splitter.push("First. Second! Third?\nFourth"),
            vec![
                "First.".to_owned(),
                "Second!".to_owned(),
                "Third?".to_owned()
            ]
        );
        assert_eq!(splitter.pending(), "Fourth");
    }

    #[test]
    fn whitespace_between_sentences_is_never_an_utterance() {
        assert_eq!(split(&["A.", "     ", "B. "]), vec!["A.", "B."]);
        assert_eq!(split(&["   \n\n  "]), Vec::<String>::new());
        assert_eq!(split(&[]), Vec::<String>::new());
    }

    #[test]
    fn an_unpunctuated_run_is_cut_at_the_bound_rather_than_held_forever() {
        // The property that has to have its own bound: a model that never
        // punctuates must still be spoken. `word ` is six characters, so the
        // cap falls inside the run rather than on it.
        let text = "word ".repeat(200);
        let mut splitter = SentenceSplitter::new();
        let sentences = splitter.push(&text);
        assert!(!sentences.is_empty(), "the bound must have fired");
        for sentence in &sentences {
            assert!(
                sentence.chars().count() <= MAX_SENTENCE_CHARS,
                "a cut sentence is still bounded: {} chars",
                sentence.chars().count()
            );
        }
        // And nothing was lost: every emitted piece plus the tail is the input.
        let rebuilt = format!("{} {}", sentences.join(" "), splitter.pending().trim());
        assert_eq!(
            rebuilt.split_whitespace().count(),
            text.split_whitespace().count()
        );
    }

    #[test]
    fn an_unpunctuated_run_with_no_spaces_is_cut_at_the_cap() {
        let text = "字".repeat(MAX_SENTENCE_CHARS * 2 + 7);
        let mut splitter = SentenceSplitter::new();
        let mut sentences = splitter.push(&text);
        sentences.extend(splitter.flush());
        assert_eq!(sentences.len(), 3, "{:?}", sentences.len());
        assert_eq!(sentences[0].chars().count(), MAX_SENTENCE_CHARS);
        assert_eq!(sentences[1].chars().count(), MAX_SENTENCE_CHARS);
        assert_eq!(sentences[2].chars().count(), 7);
        assert_eq!(sentences.concat(), text);
    }

    #[test]
    fn a_multibyte_cut_never_splits_a_character() {
        // Every cut comes from `char_indices`, so this is a property rather than
        // a lucky offset — but a panic here is a corrupted utterance, so it is
        // asserted directly.
        for filler in ["é", "字", "🎧"] {
            let text = filler.repeat(MAX_SENTENCE_CHARS + 13);
            let mut splitter = SentenceSplitter::new();
            let mut sentences = splitter.push(&text);
            sentences.extend(splitter.flush());
            assert_eq!(sentences.concat(), text, "{filler}");
        }
    }

    #[test]
    fn flush_takes_a_held_terminator_and_an_unterminated_tail() {
        let mut splitter = SentenceSplitter::new();
        assert!(splitter.push("Trailing.").is_empty());
        assert_eq!(splitter.flush(), vec!["Trailing.".to_owned()]);

        let mut splitter = SentenceSplitter::new();
        assert!(splitter.push("no terminator at all").is_empty());
        assert_eq!(splitter.flush(), vec!["no terminator at all".to_owned()]);
        assert_eq!(
            splitter.flush(),
            Vec::<String>::new(),
            "flush is idempotent"
        );
    }

    #[test]
    fn clear_discards_the_half_spoken_sentence() {
        let mut splitter = SentenceSplitter::new();
        assert!(splitter.push("I was going to say").is_empty());
        splitter.clear();
        assert!(splitter.is_empty());
        assert_eq!(splitter.flush(), Vec::<String>::new());
    }

    #[test]
    fn a_terminator_with_nothing_before_it_is_not_an_empty_utterance() {
        let mut splitter = SentenceSplitter::new();
        assert_eq!(splitter.push("\n\n。"), vec!["。".to_owned()]);
        assert_eq!(splitter.push("\n"), Vec::<String>::new());
    }
}
