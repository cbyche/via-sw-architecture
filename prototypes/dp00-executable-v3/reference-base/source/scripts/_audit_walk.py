"""Shared production-code line walker for the panic audit scripts.

Exposes one generator: `walk_production_lines(path)` that yields
`(lno, line, line_no_comments)` for lines OUTSIDE both (pass
`include_raw_stripped=True` for a 4th element — comments stripped,
string-literal content preserved — see the function's docstring):

- `#[cfg(test)] mod ... { ... }` blocks (whole-module test code), and
- `#[test]` / `#[tokio::test]` standalone test functions, AND
  `#[cfg(test)] fn helper()` shared test utilities at file scope
  (file-level test code).

## Feature-gated code is scanned unconditionally

The walker treats `#[cfg(feature = "foo")]` code as production. A
panic inside `#[cfg(feature = "ocr")] fn act(...)` ships to every
consumer that enables `ocr`, so it counts against the audit
baseline. The same logic applies to platform-gated code
(`#[cfg(target_os = "linux")]`, `#[cfg(not(target_arch = "wasm32"))]`,
…) — every cfg path is a path some user takes. The walker's only
job is to filter out test code (cfg-gated to NEVER reach a user)
from production code (cfg-gated to MAYBE reach a user). When you
add a feature-gated module behind a `feature = "..."` gate, expect
its panics to surface in the audit counts even when the feature is
off in your local build.

Brace-depth tracking is approximate — it ignores braces inside string
literals, raw strings, and `/* … */` comments — but adequate for the
canonical Rust formatting style this codebase uses. The walker is
deliberately conservative: when in doubt, it errs toward marking a
line as test code (suppression). The tradeoff is that genuine
production hits in unusual layouts may be missed; an over-eager
audit producing false-positive defensive sites that all need to be
manually re-categorised would be worse.

Z.ai PR-92 round 3 caught a bug in an earlier draft of `panic_filter.py`
where `next_line_is_test_fn` was set on `#[test]` but never reset,
suppressing every panic in the file after the first test attribute.
Z.ai PR-92 round 4 caught a sibling bug: the pre-mod `#[cfg(test)]`
attribute reset only happened in `panic_filter.py`, not in the other
two scripts. Centralising the logic here means both fixes apply
uniformly.

Z.ai PR-92 round 5 caught three more:

1. Single-line test-fn bodies (`#[test]\\nfn smoke() { assert!(true); }`)
   wedged the walker into "in_test_fn forever" mode, suppressing every
   subsequent production line in the file. Root cause: the body-depth
   threshold was set to `new_depth` (= depth + opens - closes), which
   collapsed to the entering depth when opens == closes, so the
   immediate-same-line exit check never fired. Fix: track the body's
   actual brace depth — `depth + 1` — which is robust whether the body
   spans one line or many.

2. `#[cfg(test)] fn helper()` (a shared test-utility fn at file scope,
   not inside a `#[cfg(test)] mod`) was treated as production code.
   Now `#[cfg(test)]` followed by a `fn` declaration enters test-fn mode,
   the same way `#[cfg(test)] mod tests {` enters test-mod mode.

3. Silent file-open failures (`except Exception: return`) hid real
   errors — non-UTF-8 files, permission denied, broken symlinks. Now
   logged to stderr.

Known blind spots:

A. `#[cfg(test)]` is detected ONLY before `mod` and `fn` declarations.
   Patterns like `#[cfg(test)] const X = ...;`, `#[cfg(test)] static`,
   `#[cfg(test)] use`, `#[cfg(test)] impl ...` at file scope are NOT
   recognised as test code; any `panic!`/`unwrap()` inside them will
   be reported as a production hit. In practice these patterns are
   rare (test-only declarations typically live inside the
   `#[cfg(test)] mod tests` block); manual triage should accept
   such hits as test code rather than chase a phantom bug.

B. External test modules (Z.ai PR-94 round 4 issue 7): the walker
   processes one file at a time, so a `#[cfg(test)] mod tests;`
   declaration that delegates to an external file (e.g.
   `src/foo/tests.rs` referenced from `src/foo/mod.rs`) does NOT
   suppress panics in the external file. The walker has no
   visibility into the parent file's cfg gate when scanning the
   child. If a future audit run produces a false positive in such
   an externalised test module, hoist the test code back inline
   under `#[cfg(test)] mod tests { ... }` or add the cfg gate
   redundantly on the helper inside the external file.

C. `paren_depth` counts parens inside char literals (Z.ai PR-94
   round 6 issue 2): `strip_comments` preserves char literals
   verbatim (`'('`, `')'`), so the per-line `count('(')` and
   `count(')')` see them. In balanced code (every `'('` paired
   with a matching `'.')'` later in the same scope) the counts
   net to zero, same as the brace-counting trade-off. The only
   failure mode is unpaired char-literal parens (e.g.
   `Some('(')` in a fn signature without a matching `Some(')')`
   nearby), which would shift `paren_depth` and possibly cause
   `next_line_is_test_fn` to be incorrectly preserved across
   the line. In practice fn signatures almost never carry char
   literals at all; the audit's conservative-on-doubt stance
   means even when this fires, the worst case is yielding a
   real test panic as a production hit (over-reporting), not
   the reverse.

D. Brace tracker counts braces inside char literals (Z.ai PR-94
   round 11 issue 2): same shape as blind spot C, but for the
   `opens` / `closes` brace counters that drive `in_test_mod` /
   `in_test_fn` exits. `strip_comments` preserves char literals
   verbatim, so `Some('{')` contributes 1 to `opens`, `Some('}')`
   contributes 1 to `closes`. In balanced production code the
   pair nets to zero, but a SOLO unpaired char-literal brace
   inside a test-mod body (e.g., `let opener = '{';` with no
   matching `'}'` inside the body) would inflate `opens` or
   `closes` and could prematurely exit `in_test_mod` or
   `in_test_fn`, causing later same-mod test panics to be
   reported as production hits. The audit's conservative-on-doubt
   stance means even when this fires, the worst case is
   over-reporting (real test panic surfaces as a production
   hit and gets manually triaged), not the reverse. A future
   tightening would have `strip_comments` also strip char-literal
   bodies — same trade-off as Blind Spot C, deferred for now.

State machine (Z.ai PR-94 round 10 issue 9):

   `walk_production_lines` carries 4 boolean flags + 2 integer
   depths through its line loop. The interactions are documented
   line-by-line, but the high-level flow is best summarised as
   a state diagram. Each box is a "current is-test-code stance"
   and arrows are line-pattern triggers:

      ┌──────────────────┐
      │  PRODUCTION code │  (initial state; yield lines)
      └─────────┬────────┘
                │ #[cfg(test)] alone on line
                │   → next_attr_starts_test_mod = True
                │   → next_attr_might_start_test_fn = True
                │
                │ #[test] / #[tokio::test] alone on line
                │   → next_line_is_test_fn = True
                │
                │ #[cfg(test)] mod / fn / impl ...   on ONE line
                │   → CFG_TEST_*_INLINE matches; transition direct
                │
                ▼
      ┌──────────────────────────────┐
      │  IN test_mod  /  IN test_fn  │  (suppress all yields)
      └─────────────┬────────────────┘
                    │ depth drops below entry brace level
                    │   → in_test_mod / in_test_fn = False
                    ▼
              back to PRODUCTION

   Pending flags (`next_*`) are short-lived hints — they consume
   on the next line that matches their trigger or get cleared by
   the catch-all in the same iteration. `paren_depth` keeps the
   `next_line_is_test_fn` flag alive across multi-line `fn foo<T>(\n
   x: T,\n)` signatures. `block_comment_depth` and `string_state`
   in `strip_comments` are independent of the test-mode tracking
   — they're purely about producing a clean `line_no_comments`
   for the regex matchers.

   The Z.ai-round-N references throughout the code are landmarks
   for "why this branch exists" rather than ad-hoc patches —
   each round closed a specific blind spot a contemporaneous
   review surfaced. New blind spots get a new round-N label;
   keep the cumulative log so future readers can read the
   accreted reasoning.

When (not) to switch to a real Rust parser (Z.ai PR-92 round 12 issue 6):

   This file is a manual character-level parser handling block
   comments, raw strings, byte raw strings, char literals, and
   regular strings with cross-line state — ~10 documented Z.ai
   round-N bug fixes worth of accreted complexity. The clean
   alternative is `tree-sitter-rust` (or `rustpython_ast`) for
   guaranteed-correct lexing.

   Reasons we haven't switched:

   - **Dependency footprint.** Tree-sitter ships a C library
     (~1MB) plus a Python binding; `rustpython_ast` pulls in
     the entire RustPython compiler. The audit is the only
     consumer in this repo. The complexity it would add to
     `requirements.txt` / wheel building / CI cache layers
     dwarfs the parser code we'd delete.

   - **Self-contained delta.** The walker has been stable across
     rounds 5-12 because each new bug surfaces a SPECIFIC blind
     spot that we close with a small change. A real parser
     wouldn't have those bugs but would introduce its OWN class
     of issues (version skew between tree-sitter grammar and
     `rustc`, error recovery behavior on incomplete files,
     transitive CVE pipeline).

   - **Conservative-by-default stance.** The walker errs toward
     suppression on doubt, so even when it gets test/production
     classification wrong, the failure mode is "miss a real
     production panic" (false negative) rather than "flag a
     test panic as production" (false positive that would block
     CI). The CI gate's baseline tolerance absorbs the small
     residual.

   Re-evaluate this trade-off when ANY of: the bug-fix cadence
   exceeds one fix per quarter; a confirmed false-NEGATIVE bites
   us in production (rather than the current false-positive
   pattern); or a downstream tool starts consuming the same
   walker (at which point shared maintenance amortizes the
   parser dep).
"""

import os
import re
import sys


def force_utf8_streams():
    """Make stdout/stderr UTF-8 so non-ASCII output (em-dash `—`, `✗`) doesn't
    crash on a Windows cp949/cp1252 console. CI runs on UTF-8 Linux where this
    is a no-op, but Core devs run the audits locally on Windows too (repo
    CLAUDE.md § 6). Shared here so every audit script handles it identically."""
    for stream in (sys.stdout, sys.stderr):
        reconfigure = getattr(stream, "reconfigure", None)
        if reconfigure is not None:
            try:
                reconfigure(encoding="utf-8", errors="replace")
            except (ValueError, OSError):
                pass


CFG_TEST_MOD_START = re.compile(r'^\s*#\[cfg\(test\)\]\s*$')
CFG_ANY_TEST_MOD_START = re.compile(r'^\s*#\[cfg\(any\([^)]*test[^)]*\)\)\]\s*$')
# `#[cfg(test)] mod tests {` ALL on one line. Z.ai PR-94 round 5
# issue 3: rustfmt produces this shape for empty / very short test
# modules, and the round-3 + round-4 dispatchers required the
# attribute and the `mod` to be on separate lines (the prior
# CFG_TEST_MOD_START regex anchored to `]\s*$`). This pattern
# captures the whole inline shape so the brace tracker enters
# test-mod mode on the same line.
CFG_TEST_MOD_INLINE = re.compile(
    r'^\s*#\['
    r'(?:cfg\(test\)|cfg\((?:any|all)\([^)]*\btest\b[^)]*\)\))'
    r'\]\s*'
    r'(?:pub\s*(?:\([^)]*\))?\s+)?'
    r'mod\s+\w+.*\{'
)
# Symmetric inline pattern for `#[cfg(test)] fn helper() { … }` —
# the round-5 mod-inline branch closed the gap for inline test
# modules, but the same shape with `fn` instead of `mod` was
# still falling through every dispatcher branch (cfg attr alone
# doesn't match, mod regex doesn't match, the fn-decl branch
# only fires after a separate cfg-attr line). Z.ai PR-94 round 7
# issue 1.
CFG_TEST_FN_INLINE = re.compile(
    r'^\s*#\['
    r'(?:cfg\(test\)|cfg\((?:any|all)\([^)]*\btest\b[^)]*\)\))'
    r'\]\s*'
    r'(?:pub\s*(?:\([^)]*\))?\s+)?'
    r'(?:(?:async|const|unsafe|extern(?:\s+"[^"]*")?)\s+)*'
    r'fn\s+\w+.*\{'
)
# `#[cfg(test)] mod tests` on one line WITHOUT a `{` — Allman-style
# body brace lands on a later line. Z.ai PR-94 round 12 issue 1: the
# original CFG_TEST_MOD_INLINE required a trailing `{`, and the
# split-attr path (CFG_TEST_MOD_START) required the attribute to be
# alone on its line. Code formatted as
#     #[cfg(test)] mod tests
#     {
#         …
#     }
# fell through both branches: the entire test mod's panics were
# reported as production hits. This regex closes that gap by
# matching the attr+mod-without-brace shape; the dispatcher sets
# `next_line_might_open_test_mod` so the brace tracker enters
# test-mod mode on the subsequent `{`-bearing line. rustfmt
# normalises away from this shape, so the gap was rare in practice
# (failure mode was conservative — over-reporting, not under), but
# the fix removes the blind spot from the docstring.
CFG_TEST_MOD_INLINE_DEFERRED = re.compile(
    r'^\s*#\['
    r'(?:cfg\(test\)|cfg\((?:any|all)\([^)]*\btest\b[^)]*\)\))'
    r'\]\s*'
    r'(?:pub\s*(?:\([^)]*\))?\s+)?'
    r'mod\s+\w+\s*$'
)
CFG_TEST_FN_ATTR = re.compile(r'^\s*#\[(test|tokio::test)\b')


def strip_comments(line, state_in, keep_strings=False):
    """Strip Rust line (`// …`) AND block (`/* … */`, nested)
    comments from `line` AND drop the contents of string literals
    (regular `"..."`, raw `r#"..."#` with any number of `#`s, and
    char literals `'X'`). Returns `(stripped_line, state_out)`.

    `keep_strings=True` strips only comments and preserves string-literal
    content (and its delimiters) verbatim — comments are ALWAYS stripped
    regardless of this flag; it only changes whether STRING content survives.
    String-vs-comment state tracking is unaffected: a `//` or `/* */`
    sequence found INSIDE a string literal (e.g. `"http://example.com"`)
    still isn't treated as a comment opener, so callers matching a pattern
    against a string-literal argument (`env::var("HOME")`) see the real
    code with comments removed, not a raw line where a mention inside a
    trailing or preceding comment could also match.

    State is a `(block_depth, string_state)` tuple that
    propagates across lines:
      block_depth — int, current nesting depth of /* */ blocks.
      string_state — None (not in string), ("regular",), or
        ("raw", n_hashes).

    Why both kinds of state cross lines: Rust block comments and
    raw strings are both commonly multi-line (raw strings hold
    JSON test fixtures; block comments hold long doc preambles).
    Tracking either with single-line state silently breaks on
    real source. The earliest version of this helper had separate
    `strip_comments` (block-aware) and `strip_string_literals`
    (string-aware) functions which double-processed each line and
    gave inconsistent state — `strip_comments` would treat `//`
    inside `"https://x"` as a line comment opener whenever the
    string spanned a line boundary, dropping the closing `"` and
    leaving the next stripper convinced we were still in a
    string. Z.ai PR-94 round 3 issue 3.

    Why drop string content (not just preserve verbatim): the
    audit's brace tracker counts `{` / `}` on the post-strip
    line. Braces inside string literals (`format!("{{")`,
    multi-line JSON in `r#"…"#`) would shift the depth tracker
    and prematurely exit test-mod scopes, flagging real test
    panics as production hits. Dropping string content is safe
    for the regex matchers — `panic!("foo")` becomes `panic!()`
    which still matches `r'\\bpanic!\\s*\\('`, and the unwrap
    keyword matches don't need string content.

    Limitations:
      - Char literals are detected by the `'X'` / `'\\X'` /
        `'\\xNN'` / `'\\u{N…}'` shapes via small-window lookahead;
        the literal is preserved verbatim (so its `'` doesn't
        confuse subsequent state). Lifetimes (`'a`, `'static`)
        fall through and are kept as code.
      - Byte strings `b"..."` are handled via the `b` falling
        through as code and the following `"` opening a normal
        string — same handling.
    """
    out = []
    i = 0
    block_depth, string_state = state_in
    n = len(line)
    while i < n:
        c = line[i]
        nxt = line[i + 1] if i + 1 < n else ''
        if block_depth > 0:
            # Inside a block comment — look for `*/` (close) or
            # nested `/*` (open). Drop content.
            if c == '*' and nxt == '/':
                block_depth -= 1
                i += 2
            elif c == '/' and nxt == '*':
                block_depth += 1
                i += 2
            else:
                i += 1
        elif string_state is not None and string_state[0] == "regular":
            # Inside a regular `"..."` string. Honour `\X` escapes,
            # close on `"`. Drop content unless `keep_strings`.
            if c == '\\' and i + 1 < n:
                if keep_strings:
                    out.append(c)
                    out.append(nxt)
                i += 2
            elif c == '"':
                string_state = None
                if keep_strings:
                    out.append(c)
                i += 1
            else:
                if keep_strings:
                    out.append(c)
                i += 1
        elif string_state is not None and string_state[0] == "raw":
            # Inside a raw string — close on `"` followed by N `#`
            # chars (no escape processing). Drop content unless
            # `keep_strings`.
            if c == '"':
                want_hashes = string_state[1]
                if want_hashes == 0:
                    string_state = None
                    if keep_strings:
                        out.append(c)
                    i += 1
                    continue
                j = i + 1
                got = 0
                while j < n and got < want_hashes and line[j] == '#':
                    got += 1
                    j += 1
                if got == want_hashes:
                    string_state = None
                    if keep_strings:
                        out.append(line[i:j])
                    i = j
                    continue
                if keep_strings:
                    out.append(c)
                i += 1
            else:
                if keep_strings:
                    out.append(c)
                i += 1
        else:
            # Normal code — outside strings, outside comments.
            if c == '/' and nxt == '/':
                # Line comment — drop the rest of the line.
                break
            if c == '/' and nxt == '*':
                block_depth += 1
                i += 2
            elif c == "'":
                # Char literal vs lifetime disambiguator. Look
                # ahead for a closing `'` within ~10 chars (covers
                # `'X'`, `'\\X'`, `'\\xNN'`, `'\\u{NNNNNN}'`). If
                # found, copy the entire literal verbatim so its
                # `'` can't reopen state. If not, treat as a
                # lifetime tick — copy as a normal char.
                end = -1
                j = i + 1
                while j < n and j - i <= 13:
                    if line[j] == "'":
                        end = j
                        break
                    if line[j] == '\\' and j + 1 < n:
                        j += 2
                    else:
                        j += 1
                if end != -1:
                    out.extend(line[i : end + 1])
                    i = end + 1
                    continue
                out.append(c)
                i += 1
            elif c == 'b' and i + 1 < n and line[i + 1] == 'r':
                # Byte raw string `br"..."` or `br#"..."#`. Without
                # this branch the `b` would fall through to the
                # generic `out.append`, then the next iteration's
                # `r` branch would see `prev == 'b'` (alphanumeric)
                # and skip raw-string mode — leaving the `#"..."#`
                # tail to be processed character-by-character,
                # which can corrupt the string-state tracker on
                # any `"` inside the byte raw string. Z.ai PR-94
                # round 9 issue 4 (back-ported from PR-92 round 11
                # issue 1).
                #
                # Same prev-char gate as the regular raw-string
                # branch below so an identifier ending in `b` (like
                # `cb`, `dub`) doesn't accidentally trigger.
                prev = out[-1] if out else ''
                if not (prev.isalnum() or prev == '_'):
                    hashes = 0
                    j = i + 2
                    while j < n and line[j] == '#':
                        hashes += 1
                        j += 1
                    if j < n and line[j] == '"':
                        string_state = ("raw", hashes)
                        if keep_strings:
                            out.append(line[i : j + 1])
                        i = j + 1
                        continue
                out.append(c)
                i += 1
            elif c == 'r':
                # Raw string `r"..."` or `r#"..."#`. Need previous-
                # char gate so an identifier ending in `r` (like
                # `our`) doesn't trigger.
                hashes = 0
                j = i + 1
                while j < n and line[j] == '#':
                    hashes += 1
                    j += 1
                if j < n and line[j] == '"':
                    prev = out[-1] if out else ''
                    if not (prev.isalnum() or prev == '_'):
                        string_state = ("raw", hashes)
                        if keep_strings:
                            out.append(line[i : j + 1])
                        i = j + 1
                        continue
                out.append(c)
                i += 1
            elif c == '"':
                string_state = ("regular",)
                if keep_strings:
                    out.append(c)
                i += 1
            else:
                out.append(c)
                i += 1
    return ''.join(out), (block_depth, string_state)
# Recognise a Rust fn declaration line. Handles `pub`, `pub(crate)`,
# `pub(in path)`, plus the `async`/`const`/`unsafe`/`extern "C"`
# qualifier prefixes in any combination. Used after `#[cfg(test)]`
# to distinguish a test-fn-helper from a test-mod or unrelated item.
#
# How `extern "X"` matches after `strip_comments` (Z.ai PR-94
# round 6 issue 1 — corrects a wrong claim from round 5):
# `strip_comments` DROPS both string content AND the surrounding
# quotes — `extern "C" fn foo()` becomes `extern  fn foo()`
# (two spaces, no quotes). The optional `\s+"[^"]*"` subpattern
# below therefore fails to match the (now-absent) quotes, the
# regex engine backtracks to just `extern`, the outer
# `(?:...\s+)*` repetition consumes the residual whitespace,
# and `fn\s+\w+` matches the rest. The behaviour works whether
# the quotes survive or not — the optional group is permissive,
# not load-bearing. If a future refactor changes how
# `strip_comments` handles the `"`s, this regex keeps matching;
# if a refactor adds or removes inter-keyword whitespace, the
# `\s+` outer quantifier still absorbs it.
FN_DECL_RE = re.compile(
    r'^\s*(?:pub\s*(?:\([^)]*\))?\s+)?'
    r'(?:(?:async|const|unsafe|extern(?:\s+"[^"]*")?)\s+)*'
    r'fn\s+\w+'
)
# Recognise a Rust `mod` declaration line, including visibility
# qualifiers. Z.ai PR-94 round 3 issue 1: bare `stripped.startswith
# ('mod ')` missed `pub mod tests {` and `pub(crate) mod tests {`,
# falling through to the catch-all branch and clearing the pending
# `#[cfg(test)]` flag — the entire visibility-qualified test
# module's panics got reported as production hits.
MOD_DECL_RE = re.compile(
    r'^\s*(?:pub\s*(?:\([^)]*\))?\s+)?mod\s+\w+'
)

# Z.ai PR-94 round 7 issue 3: shared file-walker so `panic_filter.py`,
# `unreachable_filter.py`, and `risky_unwrap.py` don't drift on the
# skip-list / `.rs` filter / `\src\` fragment match. A future
# directory addition to the skip list (e.g., a new build artifact
# dir) only needs one change here.
#
# Z.ai PR-94 round 8 issues 1+2: promoted from leading-underscore
# names to module-level public constants so other audit scripts
# can `from _audit_walk import SKIP_DIRS, SRC_DIR_FRAGMENT`
# instead of maintaining their own copies. The leading-underscore
# was wrong — these ARE the canonical values, not implementation
# detail. The directory-walk contract is now single-sourced.
# `.claude` holds the Claude-Code harness's nested git worktrees
# (one per active session) at `.claude/worktrees/<name>/`. Each is a
# full copy of the repo, so without skipping it the audits double-
# (or worse, N-) count every panic / unreachable / risky-unwrap site
# they find in the main tree. Skipping at the directory-walk level
# is the simplest fix and applies uniformly across all three audits.
SKIP_DIRS = {'target', 'node_modules', '.git', 'dist', '.claude'}
SRC_DIR_FRAGMENT = f'{os.sep}src{os.sep}'
# Backwards-compat aliases (any downstream caller that imported
# the leading-underscore names continues to work).
_SKIP_DIRS = SKIP_DIRS
_SRC_DIR_FRAGMENT = SRC_DIR_FRAGMENT


def walk_src_tree(root_dir='.'):
    """Yield `(root, dirs, files)` triples for every `<crate>/src/`
    directory under `root_dir`, with the skip-list dirs pruned in
    place so `os.walk` never descends into them.

    Z.ai PR-94 round 10 issue 8: extracted from `walk_rs_files` so
    `risky_unwrap.py::find_unmatched_dirs` (which wants to walk
    DIRECTORIES rather than files) can share the same pruning +
    skip-list logic. Future changes to the traversal contract
    (e.g., respecting `.gitignore`, skipping vendor sub-trees)
    only need to land here.
    """
    for root, dirs, files in os.walk(root_dir):
        # In-place mutation is the os.walk contract for pruning.
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        # Z.ai PR-94 round 12 issue 7: the `dirs[:]` pruning above
        # prevents `os.walk` from DESCENDING into a skip-dir, but
        # it does NOT skip the case where `root_dir` is itself
        # already inside a skip-dir (e.g., the caller passes
        # `walk_src_tree('./target/debug/build/foo-abc/out')` — a
        # rare but real scenario when running scripts from inside
        # `target/`). The `parts` check is a safety net for that
        # entry-point case; for a root_dir at the workspace top,
        # the check never fires.
        parts = root.split(os.sep)
        if any(p in SKIP_DIRS for p in parts):
            continue
        yield root, dirs, files


def find_unmatched_subsystem_dirs(root_dir, audited, allowlist):
    """Return a sorted set of top-level `<crate>/src/` subsystem
    directory names under `root_dir` that are in NEITHER `audited`
    nor `allowlist`. Generic helper extracted from
    `risky_unwrap.find_unmatched_dirs` so the dual-use pattern
    (`audit_panics_ci.py` runs `risky_unwrap.py` as a subprocess
    AND imports it as a module) is replaced by a single
    `_audit_walk` import. Z.ai PR-94 round 12 issue 4.
    `audited` and `allowlist` are passed as iterables; both are
    converted to sets internally for O(1) membership.
    """
    audited_set = set(audited)
    allowlist_set = set(allowlist)
    seen = set()
    # Z.ai PR-94 round 15 issue 2: previously the non-identifier
    # filter dropped hyphenated dirs into a stderr-only advisory,
    # which defeated the round-12 hard gate for any future
    # `my-module/`-shaped dir that handled adversarial input.
    # Now non-identifier dirs are folded into the same hard-gate
    # output as the identifier ones — `audit_panics_ci.py` exits
    # 3 and the operator must decide ADVERSARIAL or NOT and add
    # the literal name (hyphens preserved) to the corresponding
    # list in `risky_unwrap.py` (both lists are plain string sets,
    # so hyphenated dir names are accepted verbatim).
    #
    # Z.ai PR-94 round 14 issue 4: the previous round added the
    # advisory; round 15 issue 2 promotes it to a hard gate.
    #
    # IMPORTANT: also filters by Rust crate boundary. The walker
    # picks up ANY `src/` path component, including non-Rust ones
    # like `docs/book/src/getting-started/` (mdBook content).
    # Those are not Rust modules and can't contain Rust panics,
    # so requiring a sibling `Cargo.toml` at the crate root
    # filters them cleanly. Without this check, mdBook subdirs
    # would noise the unmatched-dirs gate.
    for root, _dirs, _files in walk_src_tree(root_dir):
        idx = root.find(SRC_DIR_FRAGMENT)
        if idx < 0:
            continue
        # Rust-crate boundary check: the dir immediately above
        # `<X>/src/` must contain `Cargo.toml`. Skips mdBook,
        # asset trees, etc. that happen to use a `src/` layout.
        crate_root = root[:idx]
        if not os.path.exists(os.path.join(crate_root, 'Cargo.toml')):
            continue
        rest = root[idx + len(SRC_DIR_FRAGMENT):]
        if not rest:
            continue
        first = rest.split(os.sep, 1)[0]
        if first in audited_set or first in allowlist_set:
            continue
        # Non-identifier dirs (hyphenated, dotted) are now
        # included so the hard gate catches them. If they're
        # genuinely not adversarial, add them to
        # KNOWN_NON_ADVERSARIAL_DIRS verbatim.
        seen.add(first)
    return sorted(seen)


def walk_rs_files(root_dir='.'):
    """Yield absolute-or-relative `.rs` file paths under `root_dir`
    that live in a `<crate>/src/` tree, skipping common build /
    vendor dirs (`target`, `node_modules`, `.git`, `dist`).

    Z.ai PR-94 round 9 issue 1: prunes `dirs` IN PLACE so `os.walk`
    never descends into the skip-list directories. The previous
    "filter results, not traversal" approach scanned every file
    in `target/` (a multi-GB Rust build artifact dir) before
    discarding it — significant wall-clock cost on a real
    monorepo.

    Z.ai PR-94 round 10 issue 8: now delegates to `walk_src_tree`
    so the pruning + skip-list logic stays single-sourced; this
    function adds the `.rs` filter + `<crate>/src/` substring
    match on top.
    """
    for root, _dirs, files in walk_src_tree(root_dir):
        for fname in files:
            if not fname.endswith('.rs'):
                continue
            path = os.path.join(root, fname)
            if SRC_DIR_FRAGMENT not in path:
                continue
            yield path


def walk_production_lines(path, lines=None, include_raw_stripped=False):
    """Yield `(lno, line, line_no_comments)` for production-code lines.

    `lno` is 1-indexed. `line` is the raw line including trailing
    newline. `line_no_comments` is the line with `// …` comments
    stripped AND string-literal content dropped, suitable for regex
    matching without false positives on commented-out code (or on
    text that merely appears inside a string).

    `lines` lets a caller that ALREADY read the file (same UTF-8 /
    `errors='replace'` decode) pass its line list in to avoid a
    second read — `audit_core_product_leak.scan` needs every
    physical line for cross-line string state AND the production
    line numbers from here, so without this it would read each file
    twice. When `lines` is None the file is read here as before.

    `include_raw_stripped=True` yields 4-tuples instead of 3-tuples,
    appending `line_comments_stripped` — the line with ONLY comments
    removed and string-literal content (quotes and all) preserved
    verbatim. This is for patterns that key on a string-literal
    argument (e.g. `env::var("HOME")`) and so can't use
    `line_no_comments` (which drops the string content they need to
    match) but must not fall back to the fully raw `line` either — a
    mention inside a `//` comment would then be a false-positive hit.
    Computed via a second, independent `strip_comments` pass (with
    `keep_strings=True`) carrying its own cross-line state, so it
    doesn't disturb the existing `line_no_comments` / test-vs-
    production brace tracking below.
    """
    if lines is None:
        try:
            # `encoding='utf-8'` is explicit because Python's default
            # uses the system encoding on Windows (cp1252 / Shift-JIS /
            # etc. depending on locale). Rust source files are always
            # UTF-8, so on a non-UTF-8 Windows runner the implicit-
            # encoding open would fail on any non-ASCII content (or
            # mojibake-decode it), the audit would warn-and-skip
            # affected files, and we'd silently undercount production
            # panics in those files. `errors='replace'` keeps the
            # walker working even on a file with broken UTF-8 (rare
            # in audited Rust code, but possible in vendored fixtures)
            # — substituting the replacement character is fine for the
            # walker's regex purposes. Z.ai PR-94 round 4 issue 3.
            with open(path, encoding='utf-8', errors='replace') as f:
                lines = f.readlines()
        except Exception as e:
            # Surface real errors (permission denied, broken symlinks)
            # instead of silently dropping the file from the audit.
            # Z.ai PR-92 round 5.
            print(f"warning: skipping {path}: {e}", file=sys.stderr)
            return

    in_test_mod = False
    test_mod_brace_depth = 0
    in_test_fn = False
    test_fn_brace_depth = 0
    depth = 0
    next_attr_starts_test_mod = False
    # `#[cfg(test)]` could lead either a `mod` or a fn — set both
    # pending flags and let the next-line dispatcher pick the right
    # branch. (The `next_attr_starts_test_mod` flag stays for the
    # mod path; this companion drives the fn path.)
    next_attr_might_start_test_fn = False
    next_line_is_test_fn = False
    # Z.ai PR-94 round 3 issue 2: `#[cfg(test)]\nmod foo\n{` (Allman
    # body brace on its own line) — the `mod foo` line doesn't
    # carry a `{`, so we set this flag and let the next `{`-bearing
    # line enter test_mod mode. Rare in practice (rustfmt always
    # puts `{` on the same line) but documented as a walker
    # contract; ignoring it would leave a silent gap.
    next_line_might_open_test_mod = False
    # Z.ai PR-94 round 4 issue 1: a multi-line `#[test]` fn signature
    # like `async fn foo<T>(\n    x: T,\n) where T: Clone {` used to
    # clear `next_line_is_test_fn` on the parameter-continuation
    # lines (which aren't fn-decls and aren't comments), so the
    # body never entered test_fn mode and the test panics got
    # reported as production. Track paren depth so the catch-all
    # clear knows to wait until the signature parens close.
    paren_depth = 0
    # `strip_comments` state: (block_comment_depth, string_state)
    # propagated across lines. See its docstring for the state
    # enum. Z.ai PR-92 round 7 issue 5 + PR-94 round 3 issue 3.
    strip_state = (0, None)
    # Independent cross-line state for the `keep_strings=True` pass
    # (only computed when `include_raw_stripped`). Kept separate from
    # `strip_state` above so the two passes' string-vs-code tracking
    # can never cross-contaminate each other.
    raw_strip_state = (0, None)

    for lno, line in enumerate(lines, 1):
        # Strip line + block comments AND drop string-literal
        # contents in one pass. State (block depth + string mode)
        # propagates across lines so multi-line strings (JSON in
        # `r#"…"#`) and multi-line block comments are tracked
        # correctly. The output is suitable for both regex
        # matching (`panic!()` shape preserved) and brace
        # counting (no `{`/`}` inside strings).
        line_no_comments, strip_state = strip_comments(line, strip_state)
        if include_raw_stripped:
            line_comments_stripped, raw_strip_state = strip_comments(
                line, raw_strip_state, keep_strings=True
            )
        stripped = line_no_comments.strip()
        # Guard for the deferred-mod-open clear: we don't want the
        # `mod foo` line itself (which SETS the deferred flag) to
        # also CLEAR it in the same iteration. Only lines AFTER
        # the mod-decl can clear.
        #
        # Z.ai PR-94 round 13 issue 1: this variable is NOT a yield
        # suppressor (despite the superficial similarity to
        # `entered_test_fn_this_line` / `entered_test_mod_this_line`
        # below). It's a within-iteration sentinel consumed at the
        # `next_line_might_open_test_mod and not set_deferred_mod_this_line`
        # check downstream — see the elif near `if
        # next_line_might_open_test_mod and opens > 0` for the read
        # site. The deferred-mod declaration line itself (`#[cfg(test)]
        # mod tests` with no `{`) cannot contain a panic expression,
        # so suppressing the yield would be a no-op.
        set_deferred_mod_this_line = False
        # Update paren depth for THIS line up-front (Z.ai PR-94
        # round 4 issue 1). Matters for the catch-all branch
        # below, which needs to know whether we're inside an
        # open paren from a prior fn-signature line so it doesn't
        # prematurely clear `next_line_is_test_fn`.
        # Z.ai PR-94 round 5 issue 4: clamp at zero. If
        # `strip_comments` ever misparses an unbalanced `)`
        # (e.g., a raw string with mismatched quote escapes
        # corrupting state), the counted-`)` would over-shoot the
        # counted-`(` and `paren_depth` would go negative. The
        # catch-all branch below uses `paren_depth == 0` as a
        # signal to clear `next_line_is_test_fn`, so a negative
        # depth would FAIL that check and incorrectly preserve
        # the flag forever, suppressing real production code.
        # Zero-clamp keeps the condition meaningful even when
        # upstream parsing degrades.
        #
        # Z.ai PR-94 round 10 issue 4: char-literal parens are
        # ALSO a source of count drift — `'('` adds 1, `')'`
        # subtracts 1. See known blind spot C in the module
        # docstring for the trade-off. The clamp here protects
        # against negative drift; balanced char-literal parens
        # (the common case) net to zero and don't trigger it.
        new_paren_depth = max(
            0,
            paren_depth + line_no_comments.count('(') - line_no_comments.count(')'),
        )

        # Attribute / mod-decl tracking.
        if CFG_TEST_FN_INLINE.match(line_no_comments):
            # Z.ai PR-94 round 7 issue 1: `#[cfg(test)] fn helper() { … }`
            # all on one line — symmetric with the inline-mod branch
            # below. Set the test-fn pending flag so the brace
            # tracker enters test-fn mode when it sees the `{` on
            # this same line. The brace tracker's
            # `entered_test_fn_this_line` flag suppresses the
            # declaration line itself from being yielded.
            next_line_is_test_fn = True
        elif CFG_TEST_MOD_INLINE.match(line_no_comments):
            # Z.ai PR-94 round 5 issue 3: the entire
            # `#[cfg(test)] mod tests {` is on one line. Enter
            # test-mod mode immediately; the brace tracker below
            # will exit when depth drops below body level.
            if not in_test_mod:
                in_test_mod = True
                test_mod_brace_depth = depth + 1
        elif CFG_TEST_MOD_INLINE_DEFERRED.match(line_no_comments):
            # Z.ai PR-94 round 12 issue 1: `#[cfg(test)] mod tests`
            # on one line, brace on the next. Defer the test-mod
            # entry to the subsequent `{`-bearing line via the same
            # `next_line_might_open_test_mod` flag the Allman-style
            # `mod foo`-without-brace path uses. Clear the test-fn
            # pending flag in case a stray cfg-attr was buffered.
            next_line_might_open_test_mod = True
            set_deferred_mod_this_line = True
            next_attr_starts_test_mod = False
            next_attr_might_start_test_fn = False
            next_line_is_test_fn = False
        elif CFG_TEST_MOD_START.match(line_no_comments) or CFG_ANY_TEST_MOD_START.match(line_no_comments):
            next_attr_starts_test_mod = True
            next_attr_might_start_test_fn = True
        elif CFG_TEST_FN_ATTR.match(line_no_comments):
            next_line_is_test_fn = True
        elif MOD_DECL_RE.match(line_no_comments) and '{' in line_no_comments:
            # `mod foo {` (or `pub mod foo {`, `pub(crate) mod foo {`)
            # — body opens on this line. Z.ai PR-94 round 3 issue 1:
            # MOD_DECL_RE handles the visibility prefix that the old
            # bare `stripped.startswith('mod ')` missed.
            #
            # `'{' in line_no_comments` (Z.ai PR-94 round 5 issue 3
            # variant) instead of `stripped.endswith('{')` so we
            # also catch `mod foo { fn helper() { panic!() } }`
            # (entire body on the mod line). The brace tracker
            # below handles the same-line exit; the
            # `entered_test_mod_this_line` flag suppresses the
            # declaration line itself from being yielded.
            if next_attr_starts_test_mod and not in_test_mod:
                in_test_mod = True
                test_mod_brace_depth = depth + 1
            next_attr_starts_test_mod = False
            next_attr_might_start_test_fn = False
        elif next_attr_might_start_test_fn and FN_DECL_RE.match(line_no_comments):
            # `#[cfg(test)]` followed by a fn declaration → treat the
            # fn body as test code, same as a `#[test]` attribute.
            # (The brace-depth machinery below handles the body span;
            # we only need to set the same trigger flag here.)
            next_line_is_test_fn = True
            next_attr_starts_test_mod = False
            next_attr_might_start_test_fn = False
        elif MOD_DECL_RE.match(line_no_comments) and not stripped.endswith(';'):
            # `mod foo` (no `{` or `;` on this line) — Allman-style
            # body brace lands on a later line. Z.ai PR-94 round 3
            # issue 2. Defer the test-mod entry to the next
            # `{`-bearing line; clear the pending fn flag so a
            # subsequent fn-decl doesn't get confused with this mod.
            if next_attr_starts_test_mod:
                next_line_might_open_test_mod = True
                set_deferred_mod_this_line = True
            next_attr_starts_test_mod = False
            next_attr_might_start_test_fn = False
            # Z.ai PR-92 round 6 issue 3: defensive — if we never saw
            # a `{`-bearing fn body after `#[test]`, drop the pending
            # flag here too. Compilable code never hits this (rustc
            # rejects `#[test]` on non-fn items) but keeps the
            # walker self-consistent on partial / malformed input.
            next_line_is_test_fn = False
        elif stripped.startswith('//') or not stripped:
            # Line-comment-only or blank line (after block-comment
            # stripping). Z.ai PR-92 round 6 issue 2: a
            # `// section header` between `#[cfg(test)]` and
            # `mod tests {` was previously hitting the catch-all
            # branch below, clearing both pending flags and turning
            # the entire test mod into reported-as-production code.
            # Comments / blanks are inert — preserve all pending flags.
            pass
        elif not stripped.startswith('#') and stripped:
            # Any non-attribute, non-blank, non-comment line clears
            # the pending attributes. Without this, an intervening
            # line (use, blank, etc) between `#[cfg(test)]` and the
            # next `mod X {` would carry the flag forward and
            # falsely tag a non-test mod.
            next_attr_starts_test_mod = False
            next_attr_might_start_test_fn = False
            # Defensive reset for `next_line_is_test_fn` — same
            # reason as the `mod foo` branch above. Skip the clear
            # when ANY of:
            #   (a) the line itself is a fn-decl (handled by the
            #       next_attr_might_start_test_fn branch);
            #   (b) we're inside an open paren from a prior fn-
            #       signature line — Z.ai PR-94 round 4 issue 1:
            #       multi-line signatures like `async fn foo<T>(\n
            #       x: T,\n) where T: ... {` used to lose the flag
            #       on parameter-continuation lines;
            #   (c) the line opens at least one `{` — the brace
            #       tracker below is about to enter test_fn mode
            #       on this same line; clearing first would defeat
            #       the entry. This catches the Allman-style body
            #       brace on its own line (`async fn foo<T>(...)
            #       \nwhere T: Clone\n{`). Use the PRE-line
            #       `paren_depth` for (b) so a line that CLOSES
            #       the signature (`) where T: Clone {`) is still
            #       treated as in-signature on entry.
            if (
                not FN_DECL_RE.match(line_no_comments)
                and paren_depth == 0
                and line_no_comments.count('{') == 0
            ):
                next_line_is_test_fn = False

        # Brace tracking. `line_no_comments` already had comments
        # AND string-literal content removed by `strip_comments`
        # above (string content dropped to keep `{`/`}` inside
        # `format!("{{ }}")` and multi-line `r#"…"#` from shifting
        # the depth tracker). Char literals (`'{'`, `'}'`) are
        # preserved as-is, matching the existing brace-counting
        # trade-off.
        opens = line_no_comments.count('{')
        closes = line_no_comments.count('}')
        new_depth = depth + opens - closes

        # Track whether the test-fn declaration line itself fired here
        # (a single-line body opens AND closes on the same line, so by
        # end-of-line `in_test_fn` is False again). Without this flag,
        # the suppression check at the bottom would yield the
        # declaration line — a `fn smoke() { result.unwrap(); }` test
        # would falsely show up as a production `unwrap()` site.
        entered_test_fn_this_line = False
        # Same shape for inline test mods — Z.ai PR-94 round 5
        # issue 3. Two patterns trigger this: the all-on-one-line
        # `#[cfg(test)] mod tests { … }` (CFG_TEST_MOD_INLINE)
        # and the two-line `#[cfg(test)]\nmod tests { fn ... }`
        # where the cfg is on the previous line and the mod's
        # entire body is on the current line. Both share the
        # same problem: `in_test_mod` was just set for this line,
        # the brace tracker's same-line exit drops it again,
        # and without this flag the line itself is yielded.
        #
        # Z.ai PR-94 round 14 issue 3: this flag does NOT cover the
        # Allman-style deferred-mod path (`#[cfg(test)] mod tests`
        # then `{` on the next line) because the deferred-mod entry
        # happens BELOW this computation — `in_test_mod` is still
        # False when this runs for the bare-`{` line. The omission
        # is intentional: the bare-`{` line in Rust is structurally
        # panic-free (Rust module bodies can't have bare statements
        # at the brace level), so even if it were yielded, the
        # panic regex wouldn't match anything. If a future Rust
        # syntax change ever permits an expression on the same line
        # as a module-opening `{`, this flag's deferred-path
        # extension becomes load-bearing.
        entered_test_mod_this_line = in_test_mod and (
            CFG_TEST_MOD_INLINE.match(line_no_comments) is not None
            or (
                MOD_DECL_RE.match(line_no_comments) is not None
                and '{' in line_no_comments
                and '}' in line_no_comments
            )
        )

        # Z.ai PR-94 round 3 issue 2: handle the deferred Allman-
        # style `mod foo\n{` shape — when the prior `mod` line
        # didn't carry `{`, we set `next_line_might_open_test_mod`
        # and now the next `{`-bearing line enters test_mod mode.
        if next_line_might_open_test_mod and opens > 0 and not in_test_mod:
            in_test_mod = True
            test_mod_brace_depth = depth + 1
            next_line_might_open_test_mod = False
        # If we hit any non-blank, non-`{` line while a deferred mod
        # entry was pending, drop the flag — the layout isn't what
        # we expected and we shouldn't speculatively enter test_mod.
        # Skip when we just SET the flag on this same line (the
        # mod-decl line itself has opens=0 by definition).
        elif (
            next_line_might_open_test_mod
            and not set_deferred_mod_this_line
            and stripped
            and opens == 0
        ):
            next_line_might_open_test_mod = False

        # Enter test-fn body — a `{` after a `#[test]` / `#[tokio::test]`
        # attribute (or a `#[cfg(test)] fn` helper). Without checking
        # `opens > 0`, the flag would persist forever; without
        # checking the brace depth of the body, we couldn't tell when
        # we exit the fn.
        if next_line_is_test_fn and not in_test_fn and opens > 0:
            in_test_fn = True
            # The body's brace level is `depth + 1` regardless of how
            # many braces this line opens/closes. Using `new_depth`
            # here breaks single-line bodies like
            # `fn smoke() { assert!(true); }` — opens == closes makes
            # `new_depth == depth`, so the same-line exit check
            # `new_depth < test_fn_brace_depth` is `depth < depth`,
            # always false, and the walker stays "in test fn" forever
            # (suppressing every subsequent production line).
            # Z.ai PR-92 round 5.
            test_fn_brace_depth = depth + 1
            next_line_is_test_fn = False
            entered_test_fn_this_line = True

        # Exit test-fn body when depth drops below the fn body level.
        if in_test_fn and new_depth < test_fn_brace_depth:
            in_test_fn = False
            test_fn_brace_depth = 0

        # Exit test mod when depth drops below the mod body level.
        if in_test_mod and new_depth < test_mod_brace_depth:
            in_test_mod = False
            test_mod_brace_depth = 0

        depth = new_depth
        # Z.ai PR-94 round 4 issue 1: persist paren depth so the
        # next iteration's catch-all knows whether we're inside an
        # open fn signature.
        paren_depth = new_paren_depth

        # Yield only production lines. `entered_test_fn_this_line`
        # suppresses the test-fn declaration line itself when the
        # body fits on one physical line (open + close on the same
        # line); without it, the declaration would be yielded as
        # production code.
        if (
            not in_test_mod
            and not in_test_fn
            and not next_line_is_test_fn
            and not entered_test_fn_this_line
            and not entered_test_mod_this_line
        ):
            if include_raw_stripped:
                yield lno, line, line_no_comments, line_comments_stripped
            else:
                yield lno, line, line_no_comments

    # Z.ai PR-94 round 5 issue 9: surface a diagnostic if the file
    # ended mid-string or mid-block-comment. `strip_state` should
    # be back to `(0, None)` (no open block, no open string) at
    # EOF on syntactically valid Rust. A non-ground state means
    # either:
    #   - the file was truncated mid-construct (rare; likely a
    #     vendored fixture or a partial-write race);
    #   - `strip_comments` mis-tokenised something earlier in the
    #     file and never recovered (a stripper bug worth fixing).
    # Either way, a warning to stderr lets a future maintainer
    # spot the issue without re-running with prints. Doesn't fail
    # the audit — the lines that DID get walked are still valid.
    if strip_state != (0, None):
        block_depth, string_state = strip_state
        diag = []
        if block_depth:
            diag.append(f"block-comment depth={block_depth} at EOF")
        if string_state is not None:
            diag.append(f"string state={string_state!r} at EOF")
        print(
            f"warning: {path}: walker ended in non-ground state "
            f"({'; '.join(diag)}) — file may be truncated, or the "
            f"stripper mis-tokenised an earlier construct",
            file=sys.stderr,
        )

    # Mirror the strip_state EOF guard for raw_strip_state (the
    # keep_strings=True pass). It's an independent state machine with
    # the same truncation / mis-tokenisation failure modes; without
    # this guard a file ending mid-string in the strings-preserved
    # pass would silently lose the tail content in every yielded
    # `line_comments_stripped` with no diagnostic. Only runs when the
    # pass was actually computed (include_raw_stripped).
    if include_raw_stripped and raw_strip_state != (0, None):
        block_depth, string_state = raw_strip_state
        diag = []
        if block_depth:
            diag.append(f"block-comment depth={block_depth} at EOF")
        if string_state is not None:
            diag.append(f"string state={string_state!r} at EOF")
        print(
            f"warning: {path}: walker ended in non-ground state "
            f"in the keep_strings pass ({'; '.join(diag)}) — file "
            f"may be truncated, or the stripper mis-tokenised an "
            f"earlier construct",
            file=sys.stderr,
        )
