"""Find risky `unwrap()` / `expect()` sites in production code.

Targets the input-handling surface (parser/, format/, manifest, mcp/,
protocols/, plugin/, llm/, security/) plus keyword filters that hint
at parsing / deserialization / external IO. Result is a candidate
list; manual triage decides whether each hit is defensive (regex
group structure, prior is_some check, mutex-poison propagation) or
genuinely risky (unguarded parse of malformed input).

Run from the repo root:

    python3 scripts/risky_unwrap.py
"""

import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _audit_walk import (  # noqa: E402
    find_unmatched_subsystem_dirs,
    walk_production_lines,
    walk_rs_files,
)

# Word-boundary regex per keyword; substring matching produced false
# positives like `thread_read(...)` matching `read(`, or
# `split_once(...)` matching `split(`. Z.ai PR-92 round 5.
# Each alternative is followed by `\s*\(` so we only flag call sites
# (not type/identifier names that happen to share the substring).
#
# Z.ai PR-92 round 7 issue 7: added the deserialization /
# value-extraction / decode patterns most likely to be reachable
# from malformed input — `from_value` (serde_json), `from_utf8`
# (str/String), `as_f64` / `as_bool` / `as_i64` / `as_u64` /
# `as_i32` / `as_u32` (Value casts). `toml::from_str` and
# `serde_yaml::from_str` already match via the existing `from_str`
# alternative; `str::parse` already matches via `parse`. `Value::get`
# is intentionally omitted because `\bget\s*\(` would false-positive
# on every `Vec::get` / `HashMap::get` call. All new entries
# produce zero baseline hits on this branch but catch future drift
# without manual bumps.
#
# Z.ai PR-92 round 8 issue 3: replaced bare `read` with the
# specific `read_*` variants. `\bread\s*\(` was matching every
# `BufReader::read(`, `stdin().read(`, `Stream::read(` — internal
# I/O sites that don't take adversarial input and aren't this
# audit's target. The variants below are the malformed-input
# entry points we actually want to triage.
#
# Z.ai PR-92 round 9 issue 6: removed bare `split` and `trim`.
# `\bsplit\s*\(` and `\btrim\s*\(` matched every `str::split()` /
# `str::trim()` call in the audit-target subsystems, most of
# which operate on already-trusted internal data (config keys,
# tool names, response separators). `split_once` stays — the
# parsing-context idiom that's actually unguarded.
#
# The keyword list below is exposed via `--list-keywords` so a CI
# job can diff against a checked-in golden file (e.g. `scripts/
# risky_keywords.golden`) and force a deliberate ack on any
# addition. Without that, the list silently goes stale as new
# parsing-adjacent methods land in the codebase and the audit
# misses them. Z.ai PR-92 round 11 issue 7.
# Z.ai PR-92 round 13 issue 7: narrowed `request` and `response`.
# `\brequest\s*\(` matched `client.request(...)`, `tx.request(...)`,
# `peer.request(...)` — many internal-call sites that don't take
# adversarial input. `\bresponse\s*\(` had the same problem. The
# specific verbs below are the parse-malformed-input shapes the
# audit actually wants to triage: HTTP headers and bodies, RPC
# request envelopes from external peers, error responses parsed
# from third-party APIs.
RISKY_KEYWORDS = (
    'parse', 'from_str', 'from_slice', 'from_value', 'deserialize',
    'from_json', 'from_yaml', 'from_utf8',
    'read_to_string', 'read_to_end', 'read_exact', 'fs::read',
    'http_request', 'send_request', 'parse_request',
    'http_response', 'parse_response',
    'header', 'decode', 'extract', 'get_str',
    'as_str', 'as_object', 'as_array',
    'as_f64', 'as_bool', 'as_i64', 'as_u64', 'as_i32', 'as_u32',
    'split_once',
)
# `fs::read` note (Z.ai PR-92 round 15 issue 9 — confirms the
# round-13 anchoring is intentional): the leading `\b` fires
# before `f` in `fs::read`, then the engine matches the full
# literal `fs::read` as a single alternative. The `::` between
# `fs` and `read` contains a word boundary (`:` non-word, `r`
# word) but doesn't break THIS alternative because it's anchored
# at the leading `\b`. If a future refactor reorders the
# alternation or adds a prefix to `fs::read`, double-check that
# the leading `\b` still aligns at the `f` rather than at the
# inner `r` (which would silently leave bare `read` unanchored
# and re-introduce false positives like `BufReader::read(`).
RISKY_RE = re.compile(r'\b(?:' + '|'.join(RISKY_KEYWORDS) + r')\s*\(')
UNWRAP_RE = re.compile(r'\.(unwrap|expect)\s*\(')

# Z.ai PR-92 round 17 issue 7: the `--list-keywords` early exit
# used to live at module top-level (before `_check_golden_count`),
# which meant a CI step using `--list-keywords` for the golden diff
# would never trigger the line-count check. Both side-effecting
# entry points (`--list-keywords` and `main()`) now live inside the
# `if __name__ == '__main__'` block at the bottom AFTER
# `_check_golden_count()`, so the count check runs on every CLI
# invocation regardless of which mode is selected.

# Z.ai PR-92 round 14 issue 8: also verify that the golden file's
# line count matches the in-script keyword count. A diff catches
# any addition/removal/rename, but a TRUNCATED golden file (e.g.,
# accidentally written with fewer lines than the in-script tuple)
# could still pass the diff if both happen to align on the
# truncated prefix. Counting both sides catches that one drift
# class the diff misses. Run on direct script invocation only —
# Z.ai PR-92 round 15 issue 2: the previous unconditional
# top-level call meant `import risky_unwrap` from a future test or
# a downstream tool would trigger the check (and potentially
# `sys.exit(2)` on a stale golden file). Gating behind the
# `__name__ == '__main__'` block at the bottom of this file
# preserves CI behaviour (the audit gate runs the script as the
# main entry) while keeping the module safely importable.
def _check_golden_count():
    golden = os.path.join(
        os.path.dirname(os.path.abspath(__file__)),
        'risky_keywords.golden',
    )
    if not os.path.exists(golden):
        # Bootstrap or test runs without the golden file — silent.
        return
    with open(golden, encoding='utf-8') as f:
        golden_lines = sum(1 for line in f if line.strip())
    if golden_lines != len(RISKY_KEYWORDS):
        print(
            f"::error::risky_unwrap.py keyword count drift — "
            f"in-script tuple has {len(RISKY_KEYWORDS)} entries, "
            f"`risky_keywords.golden` has {golden_lines}. Re-run "
            f"`python3 scripts/risky_unwrap.py --list-keywords > "
            f"scripts/risky_keywords.golden` to refresh.",
            file=sys.stderr,
        )
        sys.exit(2)


# Path-substring filter — must match the OS's path separator. On
# Windows, `os.path.join('tinicore', 'src', 'foo.rs')` produces
# `tinicore\src\foo.rs`, so a hard-coded `'/src/'` test would skip
# every file. Z.ai PR-92 round 6 issue 8.
SRC_DIR_FRAGMENT = f'{os.sep}src{os.sep}'
# Subsystem dir names. Each is anchored with the OS path separator
# on BOTH sides so substring matches require a real directory
# boundary — `'parser'` (no trailing slash) previously matched
# `.../tiniparser/foo.rs`, `'manifest'` matched
# `.../wasm-manifest-builder/...`, etc. Z.ai PR-94 round 4 issue 2.
SUBSYSTEM_DIRS = [
    # VIA phase 5: `via-realtime/src/protocol/` decodes every frame a realtime
    # provider sends — a third-party WebSocket service VIA does not control, so
    # adversarial input by definition. The baseline is unchanged (the dialects
    # use `Value::get` / `as_str` throughout and never unwrap a parse), but the
    # dir is now in scope so a future `.unwrap()` on a provider frame is caught.
    'protocol',
    # VIA phase 5: `via-app/src/realtime/` decodes every WebSocket frame a
    # client sends. The client is the outermost untrusted party the Gateway
    # has — any browser, any script, any process that can reach the port — so
    # the frame codec is adversarial input by definition. The baseline is
    # unchanged (`ClientFrame::decode` answers `None` for every shape it
    # cannot read and never unwraps a parse), but the dir is now in scope so a
    # future `.unwrap()` on a client frame is caught.
    'realtime',
    # VIA phase 5: `apps/via/src/chat/` is the text client, and every byte it
    # reads comes from a Gateway it does not control — `via chat --url
    # https://voice.example.com` parses that service's JSON bodies and its
    # WebSocket frames. A remote Gateway is a third party by definition, so the
    # client's decoding is adversarial input in exactly the sense
    # `via-app/src/realtime/` is, in the other direction. The baseline is
    # unchanged (every read is `Value::get` / `as_str` with a default, and an
    # unparseable frame is ignored rather than unwrapped); the listing puts a
    # future `.unwrap()` on a server frame in audit scope.
    'chat',
    'parser', 'format', 'manifest', 'mcp', 'protocols',
    'plugin', 'llm', 'security',
    # `lexer/` (tinish/src/lexer/) is the bash tokenizer — it consumes
    # untrusted shell-script text byte-by-byte (quoting, heredocs, escapes,
    # expansions), so it is adversarial input by definition and belongs in
    # the unwrap audit alongside its sibling `parser/`. It became a directory
    # (was `lexer.rs`) when its test module was split into `lexer/tests.rs`;
    # the tokenizer itself uses zero `unwrap()`/`expect()`, so the baseline is
    # unchanged (still 7).
    'lexer',
    # Z.ai PR-94 round 12 issue 2: `auth/` modules parse JWTs, OIDC
    # callback parameters, OAuth refresh tokens, session cookies,
    # and password material — every shape on this list takes
    # adversarial input. Both `tinicore/src/auth/` and
    # `argo-server/src/auth/` exist; the audit baseline is unchanged
    # (no new hits) but the dir is now in scope so future
    # `unwrap()` on token deserialization gets caught.
    'auth',
    # Z.ai PR-94 round 13 issue 5: `config/` parses operator-edited
    # configuration files (JSON / YAML / TOML); `gateway/` is a
    # network ingress point; `connectors/` deserializes external
    # service responses (Slack, GDrive, OAuth callbacks). All
    # three handle adversarial input by definition. Moving them
    # in surfaces 3 new defensive hits in
    # `connectors/messaging/slack.rs` and
    # `connectors/storage/gdrive.rs` — all
    # `Url::parse(STATIC_LITERAL).expect("static URL")` patterns,
    # same defensive shape as the existing regex-compile baseline.
    # Baseline bumped from 4 → 6 (see
    # PANIC_AUDIT_2026_05.md for the new entries).
    'config', 'gateway', 'connectors',
    # review round: `context_offload/` reads back
    # LLM-produced tool-call arguments, tool-name strings, and
    # serde-deserialized `OffloadStatus` enums from SQLite rows.
    # All three are adversarial inputs (the LLM is untrusted;
    # schema corruption is a real failure mode). Baseline
    # unchanged from the existing 7 — no new risky unwraps
    # landed, but the dir is now in scope so a future
    # `.unwrap()` on a row decode trips the gate.
    'context_offload',
    # Phase 3a (`tinicore/src/code_harness/`): the code-harness executor core
    # consumes adversarial LLM-authored JavaScript — the pre-flight scanner
    # byte-walks untrusted source (`scan_banned_constructs`), and the executor
    # evals it in the rquickjs sandbox. Adversarial input by definition, so it
    # belongs in the unwrap audit. Its only `unwrap()`s are compile-time-static
    # `Regex::new(LITERAL).unwrap()` compiles (the same defensive regex-compile
    # shape already in the baseline), which the audit does not count — the Rust
    # surface handles untrusted bytes through `Result`/`?` and no-unwrap byte
    # parsing — so the baseline is unchanged (still 7). Was previously part of
    # `tools/builtins/code_run.rs` (never in scope); the split promotes it in.
    'code_harness',
    # `templating/` is the new minijinja-backed engine wrapper.
    # User-supplied template strings flow through it (clipper
    # body templates, future Phase 7 skill templates, future
    # Phase 5 `.jinja2` prompt files). Any panic on a malformed
    # template would crash the host. Currently zero hits — the
    # wrapper only uses `?` and `Result` plumbing — so the
    # baseline is unchanged. Adding here keeps any future
    # `.unwrap()` in scope of the audit gate.
    #
    # Note: the legacy `template/` (tinicore/src/template.rs)
    # remains in `KNOWN_NON_ADVERSARIAL_DIRS` for now; that
    # exception is removed in PR-cleanup (Phase 4) once the
    # legacy regex code is gone.
    'templating',
    # `agents/` (argo-md/src-tauri/src/agents/) parses JSON-Lines from
    # 3P CLI agent stdout (Claude Code stream-json, Codex `exec --json`).
    # Adversarial input by definition — a hostile agent CLI could emit
    # crafted JSON to crash the host. Today every parse uses
    # `match serde_json::from_str(&line) { Ok | Err }` discipline so the
    # baseline is unchanged (zero hits), but this listing puts any future
    # `.unwrap()` on parsed JSON in audit scope. Promote to
    # SUBSYSTEM_FRAGMENTS-only matching if argo-md ever ships an in-tree
    # agent that takes its own untrusted bytes.
    'agents',
    # The HTTP submodule split landed in PRs #163/#165/#166/#168/
    # #170/#172/#173/#179. `argo-server/src/http/` parses every
    # axum-extracted query string, JSON body, multipart upload,
    # WS frame, and OAuth-provider callback — every shape on the
    # public network surface. Adding here surfaces one pre-existing
    # defensive hit (the base64 ASCII invariant in
    # `tinicapi/src/adapters/http.rs:92`), bumping the count from
    # 6 → 7; future risky patterns in handlers now get caught by
    # the audit gate.
    'http',
    # `workflow/` (tinicore/src/workflow/) evaluates untrusted JS/TS
    # workflow scripts and deserializes their `agent()` opts / result JSON
    # inside rquickjs. A script string and its per-`agent` opts are
    # adversarial input by definition, so any `.unwrap()` on parsed script
    # output belongs in audit scope. Today every parse uses
    # `unwrap_or_default` / `map_err` discipline, so the baseline is
    # unchanged (still 7). Issue #1564.
    'workflow',
    # Phase 2 token-level extraction (`tinicore/src/extractor/`).
    # `MockEntityExtractor` + `MockRedactionGate` run regexes over
    # user-controlled message text — NER for bootstrap, PII for
    # cloud-tier redaction. Adversarial input by design. All
    # `Regex::new(...).expect(...)` sites use HARDCODED LITERAL
    # patterns (same defensive shape as the existing regex-compile
    # baseline); the baseline did not bump. Future `.unwrap()` on
    # match data would be caught here.
    'extractor',
    # `exec_sandbox/` (tinicore/src/exec_sandbox/) is the OS-containment
    # backend for agent-authored subprocess runs. It decodes stdout/stderr
    # from an arbitrary — potentially hostile — child process, and its
    # argv/cwd/env inputs derive from LLM output. Both are adversarial by
    # definition, and this directory IS a security boundary, so it belongs
    # in strict audit scope rather than on the non-adversarial list. Today
    # the decode is `String::from_utf8_lossy` (cannot panic) and every
    # fallible step returns a typed error, so the baseline is unchanged
    # (still 7); a future `.unwrap()` on child output or on a
    # profile/path conversion now gets caught here.
    'exec_sandbox',
    # Phase 0 + Phase 1 multi-agent-coordinator identity
    # (`tinimesh/src/identity/`). FROST threshold signing, per-device
    # Ed25519 DIDs, quorum operation envelopes, social-recovery
    # wrapper, printable recovery seed (BIP-39). Every entry point
    # processes adversarial bytes — wire-format signatures, hex
    # decodes, BIP-39 checksums, attacker-supplied DID hex. All
    # current sites use `?` propagation or explicit defensive
    # `expect("...mutex poisoned")` shapes; no new unwraps in
    # production paths. Future `.unwrap()` on `from_bytes` / hex
    # decode would be caught here.
    'identity',
    # Phase 6a + Phase 6b operator surfaces
    # (`tinianchor/src/operator/`). Audit-export readers ingest
    # quorum-signed bundles whose envelope is attacker-influenceable
    # before signature verification (a malicious sender can forge a
    # 64-MiB payload — the size cap rejects it before the verifier
    # runs); TEE attestation tokens carry vendor-supplied bytes
    # decoded ahead of policy evaluation; the SDK/FFI bridge
    # serdes wire-form decisions and consent requests that round-trip
    # through Kotlin/Swift. Every entry point is adversarial. Phase 6
    # adds zero new `.unwrap()` sites; future `.unwrap()` on token
    # decode / decision parse / audit-bundle deser would be caught
    # here. Baseline unchanged at 7 (see PANIC_AUDIT_2026_05.md).
    'operator',
    # `tinicli/src/webui/` is the single-user WebUI gateway added by
    # the `tinicli-daemon-webui-gateway` change. It parses bearer /
    # `?token=` JWTs, `X-Argo-Single-User` headers, JSON request
    # bodies, and `ClientFrame` WebSocket frames — every byte is
    # network-adversarial. ADVERSARIAL by design. The handlers carry
    # zero production `unwrap()`/`expect()` (the four
    # `.expect("webui present after token check")` sites were removed
    # by threading the verified `WebuiState` out of `require_token`;
    # the only `.unwrap()` calls live in the sibling `tests.rs`, which
    # the audit walker excludes as test code). Baseline unchanged at 7.
    'webui',
    # Issue #209 follow-up PR 2.1 (#316) added
    # `tinianchor/src/persistence/`. Every entry point here
    # consumes adversarial bytes — `Store::get` returns
    # caller-tampered values, `BlobStore::get` returns
    # content-addressed bytes from a store the attacker may
    # have rewritten, the schema-header decoder rejects
    # malformed `MAGIC + version + payload` prefixes, and the
    # reflog deserialises JSON entries that may have been
    # corrupted on disk. The current PR uses `?` propagation
    # throughout (no production `.unwrap()` / `.expect(...)`
    # in `tinianchor/src/persistence/*.rs`). Future
    # `.unwrap()` on `decode_header` / `serde_json::from_slice`
    # / store-poison mutex paths would be caught here. PR 2.2
    # adds the file-backed `FsStore` / `FsBlobStore` / `FsReflog`
    # which exercise the same audit-pickup. Baseline unchanged
    # at 7.
    'persistence',
    # P0 AIDL layer (`tinicore/src/ipc/`) — the engine-side acceptance of
    # external-app tool bridges (`EngineServiceProvider` / `DefaultEngineService`).
    # The IPC boundary is a trust boundary by definition: the bridges it stores
    # proxy to external APKs over AIDL. The byte-level JSON parse of untrusted
    # tool I/O lives in `tiniffi/src/external_ipc.rs` (a file the audit walker
    # already scans), but the engine-side dir is in scope so any future
    # `.unwrap()` on a bridge response / registration path gets caught.
    # `DefaultEngineService` uses only `RwLock` poison-recovery
    # (`unwrap_or_else(PoisonError::into_inner)`), no `.unwrap()`/`.expect(...)`;
    # baseline unchanged at 7.
    'ipc',
    # PR #516 (refactor: argo-auth split flat *.rs into focused
    # submodules) introduced these three under `argo-auth/src/`:
    #   - `jwt/`     — parses inbound JWTs from untrusted callers
    #                  (access/refresh token validation, JWKS rotation).
    #   - `oidc/`    — OIDC client: ID-token validation, RFC-3339 ts
    #                  parsing from third-party providers, JWKS fetch.
    #   - `refresh/` — refresh-token persistence + rotation; user-
    #                  supplied refresh tokens are the entry point.
    # All three handle adversarial input by definition — same logic
    # as the existing `'auth'` entry above; PR #516 split the flat
    # `auth/*.rs` into nested submodules but the audit script wasn't
    # updated in lockstep. Adding them keeps any future
    # `.unwrap()` / `.expect(...)` on token deser / JWKS / cookie
    # parsing in scope. Baseline unchanged (the existing argo-auth
    # code remained `?`-propagation-clean across the split).
    'jwt', 'oidc', 'refresh',
    # `argo-auth/src/samsung/` — Samsung Account federated login
    # (issue #2081): validates the CLIENT-SUPPLIED `auth_server_url`
    # against the SSRF allowlist, parses Samsung's token-exchange
    # response (`userId`), and hygiene-checks `state`/`authcode`
    # strings from anonymous callers. Adversarial input by
    # definition — same reasoning as `jwt`/`oidc`/`refresh` above.
    # Baseline unchanged (the module is `?`/`Option`-propagation
    # clean; its only `unwrap_or_else` is mutex poison-recovery).
    'samsung',
    # `argo-wasm/src/wire/` — cell↔host wire envelopes. The
    # cell-side decoder consumes bytes the host wrote into the
    # cell's linear memory; on the host side `argo-server`
    # decodes the same envelopes back out of cell memory. Both
    # sides take adversarial input by design (a malicious wasm
    # cell could ship a malformed envelope; a misbehaving host
    # could write garbage). The PR #327 round-3 review
    # introduced `CellTurnRequestParse::JsonRejected` to surface
    # decode failures explicitly. Today all sites use
    # `serde_json::from_slice → Result` and `match` discipline;
    # baseline unchanged at 7. Future `.unwrap()` on `parse` /
    # `into_llm_request` would be caught here.
    'wire',
    # `argo-wasm/src/host_harness/` — reference wasmtime host
    # implementation reused by `argo-server`. The
    # `register_with_linker` closures deserialize JSON envelopes
    # the cell writes into its linear memory (`host_llm_dispatch_open`
    # → `WireLlmDispatchEnvelope`); `enumerate_skill_catalog_json`
    # reads operator-supplied `.install.json` files from disk
    # (TOCTOU-safe via byte-capped `Read::take`). Adversarial
    # input by design — a malicious cell can ship a malformed
    # envelope, a hostile shared-skills entry can plant a
    # multi-GB `.install.json`. Current sites use bounded reads
    # + `match` discipline; baseline unchanged at 7.
    'host_harness',
    # `tinish/src/executor/` — bash AST walker that executes user-
    # supplied scripts: argument expansion, command substitution,
    # arithmetic, glob, pipeline routing. Every entry point processes
    # operator-supplied bytes. Added when `tinish/src/executor.rs`
    # was split into the nested `executor/` module tree (refactor
    # PR); the audit baseline is unchanged because the split only
    # re-homed code with no new unwraps. ADVERSARIAL.
    'executor',
    # `tinivfs/src/vfs_session/` — VfsSession facade (mount, read,
    # write, journal). Reads bytes from worktree files whose content
    # was placed there by upstream `vfs_mount` / `file_write` tool
    # calls — operator-supplied bytes by definition. Journal entries
    # round-trip JSON between sessions. Added when `vfs_session.rs`
    # was split into the nested `vfs_session/` module tree; baseline
    # unchanged. ADVERSARIAL.
    'vfs_session',
    # `argo-home/src/smartthings/` — SmartThings REST backend: every
    # entry point parses network bytes (vendor JSON for devices /
    # status / scenes pages) fetched over HTTP — adversarial by
    # definition (a compromised cloud or MITM could emit crafted
    # payloads). The mapper skips malformed ITEMS and the client maps
    # malformed TOP-LEVEL bodies to `HomeError::Decode`; production
    # code carries zero `unwrap()`/`expect()` (the `expect`s live in
    # `#[cfg(test)]` modules, excluded by the walker). Baseline
    # unchanged. ADVERSARIAL.
    'smartthings',
    # PR #610 Phase 0–9 job/trigger/shortcut scheduler. `jobs/`
    # parses cron expressions, legacy condition strings (battery>,
    # network), and constraint specs from operator-edited manifests
    # — adversarial input. Currently zero production unwraps in
    # the parser paths (all use `?` propagation); adding here
    # keeps future `.unwrap()` on parse results in scope.
    'jobs',
    # `argo-pc/rust-backend/src/ondevice/` — Ollama on-device bridge.
    # Parses Ollama daemon HTTP responses (`/api/tags`,
    # `/v1/chat/completions`, `/v1/embeddings`) — external/adversarial
    # input. Currently zero production unwraps (all in `#[cfg(test)]`);
    # adding here keeps future `.unwrap()` on response parsing in scope.
    'ondevice',
    # `argo-pc/rust-backend/src/hand/` — the PC-exclusive Confluence
    # tools submodule (issue #1679). Every entry point parses network
    # bytes (Confluence REST JSON for search / page bodies / space +
    # child listings, plus XHTML storage-format bodies stripped to text)
    # — external/adversarial by definition (a compromised server or MITM
    # could emit crafted payloads). Same pattern as the sibling
    # `ondevice` bridge above. Production code carries zero
    # `unwrap()`/`expect()` (the sites live in the `#[cfg(test)]` module,
    # excluded by the walker), so the baseline is unchanged; adding here
    # keeps any future `.unwrap()` on a response parse in audit scope.
    # SCOPE NOTE (PR #1706 review): besides the intended
    # `…/hand/…` directory match, the `SUBSYSTEM_FILENAMES` matcher
    # ALSO pulls in every `hand.rs` file — today that is
    # `argo-pc/rust-backend/src/hand.rs` (the parent hand, fine) and
    # `tinicore/src/traits/hand.rs` (a CORE file: the Hand trait +
    # HandContext definitions). Both currently carry zero production
    # risky unwraps, so the golden count is unaffected — but a future
    # defensive `.expect()` added to the Core trait file lands in THIS
    # token's scope; document it against this entry, not a new one.
    'hand',
    # `tinicore/src/dispatchable_policy/` — unified-policy module.
    # The `audit` submodule scans untrusted skill / manifest text
    # (SKILL.md, MCP server descriptions, A2A-injected snippets)
    # against a regex rule set ported from runkids/skillshare. Every
    # byte that reaches `audit_text` originates from operator- or
    # network-supplied source. Currently zero production unwraps
    # (the rule compilation uses `OnceLock` + `filter_map`
    # error-handling rather than `.unwrap()`); adding here keeps
    # future `.unwrap()` on regex match / text slicing in scope.
    # ADVERSARIAL.
    'dispatchable_policy',
    # `tinicore/src/mcp_server/` — MCP server-mode dispatcher behind
    # the `mcp-server` Cargo feature. Parses inbound JSON-RPC 2.0
    # envelopes from external MCP peers (Claude Desktop, ChatGPT,
    # VS Code, third-party servers) — every byte is network-
    # adversarial. The dispatcher uses `match` + `Result`
    # discipline; the only production deserialise failures lift to
    # `error_codes::INVALID_PARAMS`. Future `.unwrap()` on
    # peer-controlled inputs would be caught here. ADVERSARIAL.
    'mcp_server',
    # Context compression. `tinicompress/src/smart_crusher/` crushes
    # untrusted JSON arrays (tool output, captured artifacts) and
    # `tinicore/src/compression/` runs the crushers + on-device distiller
    # over arbitrary tool output before it re-enters the prompt — every
    # byte is adversarial (a tool can return anything). The crushers parse
    # via `serde_json` with `match`/`?`/`unwrap_or` discipline and the
    # session store uses `.ok()?` throughout, so the baseline is unchanged
    # (zero production hits); listing them keeps any future `.unwrap()` on
    # parsed content in audit scope. ADVERSARIAL.
    'smart_crusher', 'compression',
    # `tinicore/src/proactive/` runs ONE LLM call against an
    # untrusted signals JSON (assembled by the Android worker from
    # device sources: notifications / files / photos / app-usage)
    # and parses the LLM's free-form response into a typed suggestion.
    # Both ends are adversarial in the audit sense: the signals
    # carry user-controlled text (notification bodies, file names,
    # contact identifiers) and the LLM response is a remote-emitted
    # string. `parse_response` uses `serde_json::from_str` with
    # `Result` discipline + a markdown-fence stripper, so the
    # baseline is unchanged (zero production hits). Listing keeps
    # any future `.unwrap()` on these inputs in audit scope.
    # ADVERSARIAL.
    'proactive',
    # `tinicore/src/mini_app/` (issue #1036) — the Mini-Apps package
    # model. Every entry point ingests an untrusted `.miniapp` bundle:
    # the package reader decompresses an attacker-controlled zip
    # (path-traversal + zip-bomb guards in `package.rs`), the manifest
    # layer parses operator/marketplace TOML, the signing layer verifies
    # a detached JWS over the Merkle root, and the scope catalog
    # validates declared capability strings. Every byte is adversarial
    # by design. Current sites use `?` propagation + `Result`
    # discipline throughout (no production `.unwrap()` / `.expect(...)`;
    # the `.unwrap()` calls live only in `#[cfg(test)] mod tests`, which
    # the audit walker excludes). Baseline unchanged at 7. Adding here
    # keeps any future `.unwrap()` on package/manifest/JWS parsing in
    # audit scope. ADVERSARIAL.
    'mini_app',
    # `tinihead/src/derp/` — the DERP relay + co-located STUN server
    # (PR-4). Decodes length-prefixed DERP frames and RFC 5389 STUN
    # binding datagrams straight off the wire from untrusted mesh
    # peers — every byte is network-adversarial. The frame/STUN
    # decoders bound the payload, reject unknown types, and lift all
    # malformed input to `Error::Derp`/`Error::Disco` via `?` (zero
    # production `.unwrap()`); the mutex guard maps a poisoned lock
    # to `Error::Derp` rather than unwrapping. Currently zero hits;
    # adding here keeps any future `.unwrap()` on peer-controlled
    # frame/STUN bytes in scope of the audit gate. ADVERSARIAL.
    'derp',
    # `tinihead/src/server/` (PR-13) — the control-plane HTTP API. Decodes
    # inbound RegisterRequest / MapRequest bodies (Ed25519-signed JSON) from
    # untrusted mesh clients; every byte is network-adversarial. Stub today
    # (zero hits); listed so a future `.unwrap()` on request bytes is in
    # scope of the audit gate. ADVERSARIAL.
    'server',
    # `tinihead/src/client/` (PR-14) — the node agent. Decodes MapResponse /
    # netmap deltas + DERP/disco frames received from the coordination server
    # and from peers; network-adversarial. Stub today (zero hits). ADVERSARIAL.
    'client',
    # `tinihead/src/db/` (PR-2) — persistence layer. Re-decodes stored JSON
    # (hostinfo / endpoints) and typed key/IP strings that originated from
    # network MapRequests; a future `.unwrap()` on a malformed stored row
    # should be caught. Currently zero production unwraps (all fallible paths
    # lift to `Error::Db` via `?`). ADVERSARIAL.
    'db',
    # `tinihead/src/wg.rs` (PR-10a) — the boringtun WireGuard engine.
    # `decapsulate()` decodes raw datagrams straight off the wire from
    # untrusted peers; every `TunnResult::Err` lifts to `Error::WireGuard`
    # (zero production `.unwrap()`). Single-file subsystem so a future
    # `.unwrap()` on peer-controlled ciphertext is caught. ADVERSARIAL.
    'wg',
    # `tinihead/src/conn.rs` (Wave C) — magicsock path mux (direct UDP <->
    # DERP). Will read untrusted UDP datagrams + DERP frames; network-
    # adversarial. Stub today (zero hits). ADVERSARIAL.
    'conn',
    # `tinihead/src/disco.rs` (PR-9/Wave C) — NAT-traversal/disco: parses STUN
    # binding responses + disco call-me-maybe/ping/pong off the wire; network-
    # adversarial. Stub today (zero hits). ADVERSARIAL.
    'disco',
    # `desktop_gui/` (argo-pc, EPIC #1229 P3) — the Windows desktop_gui host
    # hand. `helpers.rs` parses the `key(...)` chord DSL and the backend reads
    # the foreground app's UI-Automation tree (window titles / control labels /
    # field values), all influenced by on-screen content that a prompt-injection
    # attacker can shape — so it parses adversarial-ish input by definition.
    # This entry also pulls the existing `desktop_gui.rs` files (the tinicore
    # tool DSL parser + sub-agent) into scope via `SUBSYSTEM_FILENAMES`. All
    # three use `Result`/`?` end-to-end with zero production `.unwrap()`/
    # `.expect()` (the only such calls are in `#[cfg(test)]` modules, which the
    # walker excludes), so the baseline is unchanged (still 7). ADVERSARIAL.
    'desktop_gui',
]
SUBSYSTEM_FRAGMENTS = [f'{os.sep}{d}{os.sep}' for d in SUBSYSTEM_DIRS]

# Z.ai PR-94 round 11 issue 7: explicit allowlist of `<crate>/src/`
# top-level directories that have been triaged as NOT taking
# adversarial input. Without this, the unmatched-dirs gate
# (`audit_panics_ci.py` exit code 3, `find_unmatched_dirs` below)
# would surface every uncategorised dir on every run, training
# operators to ignore the annotation. With the allowlist, the
# residual is empty in steady state — anything left is a NEW
# directory that needs an explicit ADVERSARIAL-or-NOT decision.
#
# To add a new dir: decide whether it takes adversarial input
# (parse / deserialize / decode user-controlled bytes). If YES,
# add to `SUBSYSTEM_DIRS` above. If NO, add here with a one-line
# rationale comment.
KNOWN_NON_ADVERSARIAL_DIRS = [
    # VIA phase 5: `via-voice/src/announcement/` is the Injection Gate and the
    # delivery ladder. Every value it sees is already typed by the time it
    # arrives — playback receipts are decoded by `via-app/src/realtime/` (which
    # IS listed as adversarial) and reach it as booleans and response ids, and
    # a backend result is bounded by `via_downstream::SessionEvent` before the
    # window ever weighs it. It decides *when* to speak, not *what* the bytes
    # were, so it takes no adversarial input of its own.
    'announcement',
    # Internal coordination / orchestration / dispatch.
    # Z.ai PR-94 round 12 issue 2: `auth` was previously here but
    # is now in `SUBSYSTEM_DIRS` (it parses JWT / OIDC / OAuth
    # tokens — adversarial input). If a future categorisation
    # decision lands a name here that turns out to handle
    # adversarial input later, move it to `SUBSYSTEM_DIRS` and
    # tighten the categorisation comment.
    # Z.ai PR-94 round 13 issue 5: `config`, `gateway`, `connectors`
    # were previously here but are now in `SUBSYSTEM_DIRS` (they
    # all handle adversarial input). `api` would also belong in
    # SUBSYSTEM_DIRS by the same logic, but the only `api/` dir on
    # this branch is `argo-webui/src/api/` which is TypeScript /
    # JavaScript code outside the Rust audit's scope. If a Rust
    # `api/` dir lands later, move it.
    # Phase 10 review (round 2): `audit` (`argo-server/src/audit/`)
    # and `cost` (`argo-server/src/cost/`) are sqlite-backed
    # internal recorders. They take typed structs from same-process
    # callers and persist via parameterised sqlite INSERTs — no
    # user-controlled bytes hit the unwrap surface. Promote to
    # `SUBSYSTEM_DIRS` if a future row reader deserialises external
    # SIEM payloads here.
    'adapters', 'agent',
    # Job #2 / migration #124 (`tinicore/src/history/`): mutable
    # conversation-history store contract + in-memory + SQLite impls.
    # The trait takes typed `HistoryMessage` values from same-process
    # callers (the agent loop and the threads HTTP adapter, both of
    # which validate inbound bytes upstream); the SQLite store uses
    # parameterised rusqlite queries and never deserialises an
    # operator-supplied or peer-supplied blob. Every site that would
    # otherwise `unwrap()` returns `HistoryError::Storage(String)`.
    'history',
    # PR #760 (`feat/auto-mode-orchestrator-wiring`): the
    # `tinicore/src/auto_mode/` bridge module hosts wire-up plumbing
    # only — `build_classify_request` assembles a typed
    # `ClassifyRequest` from already-typed `Principal` +
    # `ToolCallRequest` values, and `SubjectIdentity` (lands in PR-3b
    # per #764 — the trust-ledger gate) keys on the same typed values.
    # The adversarial seam is the LLM-classifier impl (which lives in
    # `tinicore/src/auto_mode/` under PR-3c — NOT PR-3b — but reaches
    # the LLM via the `LLMProvider` trait whose backends are already
    # in `SUBSYSTEM_DIRS` via `llm/`). No user-controlled bytes reach
    # `.unwrap()` in this module's code. Promote to `SUBSYSTEM_DIRS`
    # if a future revision deserialises raw classifier JSON / parses
    # untrusted manifest text here.
    'auto_mode',
    # PR #503 (`refactor(argo-anchor)` flat→submodule split): the
    # split landed three new top-level `<crate>/src/<dir>/`
    # directories whose code already lived in `argo-anchor/src/`
    # as flat files — no behaviour change, just a path-on-disk move.
    # The audit walker's `<crate>/src/X/` discovery surface flagged
    # them as uncategorised; rather than re-classify each on every
    # refactor, all three are listed here because their input
    # shapes are the same as the pre-split files:
    #   - `anchor/` — in-process orchestration over typed structs
    #     held by the parent daemon; no user-controlled bytes hit
    #     `.unwrap()`. Adversarial input enters via tinimesh's
    #     identity layer (already in SUBSYSTEM_DIRS via `identity/`).
    #   - `subsystem/` — internal coordination between the role,
    #     epoch, and federation subsystems; typed-struct in,
    #     typed-struct out.
    #   - `tracer/` — local-only event tracer that records typed
    #     spans into an in-memory ring buffer. No external bytes.
    'anchor',
    'api', 'audit', 'automation', 'background',
    # `benchmarks/` (argo-pc/rust-backend/src/benchmarks/) holds the
    # `bench-token-reduction`-gated benchmark runners `tr_bench` / `tcache_bench`.
    # They are dev-only harnesses, never compiled into the shipped app or MSI
    # (the gate is off in every release / default-CI build). The sources
    # previously lived in `src/bin/` (already on this list) and were moved out
    # only so Tauri's WiX bundler stops trying to bundle them — the
    # non-adversarial input shape is unchanged by the move. Promote to
    # SUBSYSTEM_DIRS only if a benchmark ever ships in a product artefact and
    # parses untrusted bytes.
    'benchmarks',
    'bin', 'binary_resolver',
    # The argo-server `cell_pool/` module wraps wasmtime cell
    # lifecycle (boot / dispatch / refuel). The actual wasm parsing
    # is delegated to wasmtime; this module only composes typed
    # `CellBootConfig` + drives the cell ABI through `TypedFunc`
    # calls. The bytes that reach `CellHandle::boot` are
    # operator-controlled (the wasm artefact path comes from server
    # config), not user-controlled — same boundary classification
    # as `bin/` and `boot/` already on this list. Promote to
    # SUBSYSTEM_DIRS if a future on-the-wire wasm-bytes endpoint is
    # added that takes user-uploaded modules.
    'cell_pool',
    # `boot/` is the argo-server runtime-startup wiring (loaded
    # before any HTTP listener binds — config validation, OAuth
    # provider construction, OIDC discovery cache priming). It
    # operates on already-validated `OrchestratorConfig` structs
    # (built by the typed env-var loader in `config/`); no
    # user-controlled bytes reach `.unwrap()` here. Promote to
    # SUBSYSTEM_DIRS if a future boot step parses third-party
    # discovery JSON inline.
    'boot',
    # Phase 2 bootstrap-bundle data types (`tinicore/src/bootstrap/`).
    # Pure typed structs + serde derives; no I/O, no parsing of
    # external bytes. Bootstrap *execution* (which reads platform
    # contacts/calendar) lives in ARGO, not tinicore — out of scope
    # here. Promote to SUBSYSTEM_DIRS if a future on-device bootstrap
    # bundle reader is added.
    'bootstrap',
    'broker', 'channels',
    # `argo-server/src/host_runtime/` — the v0.4 HostRuntimeBackend runner
    # (capability G / migration #124). Orchestration plumbing: it takes an
    # already-parsed gateway `Message`, builds an `AgentLoopConfig`, drives
    # `agent_loop`, and persists jobs. It parses no adversarial wire bytes —
    # the wire ↔ ServerFrame mapping lives in `channels/` + `wire.rs`, and
    # untrusted session-id path safety is delegated to `FsAgentJobStore`'s own
    # segment sanitiser. Production code uses match / `unwrap_or_else` (poison
    # recovery), no risky unwrap. Promote to SUBSYSTEM_DIRS if a future
    # revision adds a raw-bytes decoder here.
    'host_runtime',
    # Phase 0 / Phase 1 embedding-classifier substrate
    # (`tinicore/src/classifier/`). Internal trait + activation +
    # Hebbian arithmetic over typed structs; PrototypeSet I/O is
    # serde-mediated and errors propagate. No user-controlled bytes
    # reach unwrap. Promote to SUBSYSTEM_DIRS if a future
    # external-prototype-bundle loader is added that consumes
    # untrusted binary blobs.
    'classifier',
    # Phase 2.5 unified cognitive pipeline
    # (`tinicore/src/cognition/`). Pure trait + adapter layer over
    # the existing perceiver subsystems; no I/O, no parsing of
    # external bytes. Adversarial input (regex on message text)
    # already lands in `extractor` (SUBSYSTEM_DIRS); these
    # adapters just route typed outputs into a Perception enum.
    'cognition',
    'commands',
    'components', 'concepts',
    # Phase 2 contact registry (`tinicore/src/contacts/`). In-memory
    # HashMap + typed Contact structs + serde. ContactRegistry trait
    # is async but no impl in this crate touches untrusted input.
    # Production impls (ARGO) read the platform Contacts API and
    # would themselves classify as adversarial; the trait here is
    # not.
    'contacts',
    'context_policy', 'cost', 'cron', 'diff', 'extending',
    # Phase 5b inter-household federation (`tinianchor/src/federation/`).
    # The module composes tinimesh-typed `HouseholdVerifyingKey` /
    # `Signature` values through serde-derived shapes whose
    # malformed-input handling lives in tinimesh's identity layer
    # (already covered by SUBSYSTEM_DIRS via `identity/`). The
    # federation module itself only does in-memory struct
    # composition + delegates verification to
    # `HouseholdVerifyingKey::verify`. No user-controlled bytes
    # reach `.unwrap()` / `.expect()` outside `#[cfg(test)]`.
    # Promote to SUBSYSTEM_DIRS if a future revision adds a
    # `from_bytes` wire-decoder that takes adversarial input
    # directly.
    'federation',
    # G-02 GenUI widget infrastructure (`tinicore/src/genui/`). Two
    # files:
    #   - `validator.rs` — regex-based reject rules over LLM-generated
    #     JSX. All static `Regex::new(...).expect("static regex")`
    #     compilations operate on literal patterns, never user bytes.
    #     User input flows through `is_match()` / `captures()` which
    #     return `Option`; the only Index access on captures
    #     (`&cap[1]`) is gated by the regex structure (the pattern
    #     guarantees group 1 exists when the regex matches), so the
    #     panic is unreachable by construction.
    #   - `widget_store.rs` — rusqlite store using parameterised
    #     `params![...]` INSERTs over typed structs. No user-controlled
    #     bytes reach unwrap (errors propagate via `?` and
    #     `WidgetStoreError::Sqlite(#[from] rusqlite::Error)`).
    # Promote to SUBSYSTEM_DIRS if a future revision adds a raw-bytes
    # widget importer that bypasses the typed `NewWidget` struct.
    'genui',
    'hands', 'harness', 'heartbeat',
    # Migration #124 Job #5 (#124): `tinicore/src/history/` is a
    # marker-trait stub today — `ConversationHistoryStore` carries no
    # methods, so the module has zero functions and zero `unwrap`s.
    # Job #2 will add `InMemoryConversationHistoryStore` (in-process
    # DashMap port) and a `SqliteConversationHistoryStore` (rusqlite
    # over typed structs); when those land, re-classify: the in-memory
    # impl stays here (typed-struct in, typed-struct out, no external
    # bytes), but the sqlite impl moves to `SUBSYSTEM_DIRS` if its
    # `replace` / `get_for_turn` ever takes raw-byte transcripts from a
    # restore path.
    'history',
    'hooks', 'http', 'i18n', 'install',
    'integrating',
    # PR #713 rename: `harness_coord/` was renamed to `coord/` (the
    # module hosts CrossHarnessDispatch, HarnessRegistry, and the
    # Transport trait — typed-struct dispatch over peer devices, not
    # adversarial-byte parsing). Promote to SUBSYSTEM_DIRS if a future
    # revision adds a wire-decoder that takes user-controlled bytes.
    'coord',
    'license', 'managed',
    # `manager/` (via-work/src/manager/) is the Work subsystem's owning task:
    # a bounded `mpsc` of typed `Command` values, a lifecycle state machine
    # over `via_protocol::WorkStatus`, and the admission scheduler. Every byte
    # that could be adversarial is parsed before it gets here — `tasks.json` by
    # `via-store` (which quarantines rather than trusting), a backend's
    # `session/update` by `via-downstream`'s projection (whose input type has
    # nowhere to put a secret). This module receives typed values from
    # same-process callers only. Promote to SUBSYSTEM_DIRS if a future revision
    # deserialises a wire payload inline here.
    'manager',
    'markdown2ui', 'memory', 'mount',
    # `argo-pc/rust-backend/src/notifier/` — OS-native notification
    # facade (Windows toast / Linux freedesktop / stub) for cron-fire
    # events. Inputs are typed `(title, body, conversation_id)` strings
    # held by the in-process cron scheduler; no user-controlled bytes
    # or external deserialisation reach `.unwrap()`. Promote to
    # `SUBSYSTEM_DIRS` if a future backend ever parses notification
    # payloads received from another process / network endpoint.
    'notifier',
    # Tier-1 production-features PR: `observability` is the metrics +
    # health + doctor + request-id module. It only reads typed
    # internal state (atomics, counters, env vars) and emits text;
    # no user-controlled bytes hit unwrap.
    'observability',
    # Phase 2 OCEAN inference (`tinicore/src/ocean/`). Pure
    # arithmetic over `MessageSnapshot` text; no I/O, no parse, no
    # external bytes reach unwrap. The signals module uses only
    # std iterators + arithmetic.
    'ocean',
    # Phase 4b ML-runtime bridges (`tinicore/src/ml/`). Trait +
    # adapter substrate. MockBridge produces an in-memory
    # MockEmbedder; the four real-runtime bridges (candle / onnx /
    # coreml / tflite) are feature-gated stubs returning
    # `MlError::NotImplemented`. When a real loader lands its dir
    # SHOULD move to SUBSYSTEM_DIRS — it'll parse model files
    # (adversarial bytes from disk / network).
    'ml',
    # argo-pc desktop notifier (`argo-pc/rust-backend/src/notifier/`).
    # Platform-specific OS-notification dispatch — linux.rs (libnotify
    # / D-Bus), windows.rs (WinRT toast), stub.rs (no-op fallback) +
    # mod.rs wiring. Inputs are typed `Notification { title, body, .. }`
    # structs from same-process callers; outputs are calls into the
    # OS notification API. No user-controlled bytes reach `.unwrap()`.
    # Promote to SUBSYSTEM_DIRS if a future build adds a generic
    # notification-payload deserializer (e.g. accepting JSON from a
    # plugin).
    #
    # SECURITY: audited 2026-05-27 against
    # `argo-pc/rust-backend/src/notifier/{mod,linux,windows,stub}.rs`
    # on commit 99f0553e. Public surface is `show_info(title: &str,
    # body: &str, app_handle: &tauri::AppHandle)` and
    # `show_cron_fire(...)` (see `mod.rs:132,140`) — both accept owned
    # `&str` from same-process Rust callers and pass straight to the
    # OS notification API without any `.unwrap()` on the input path.
    # The only non-test `.expect()` sites in this directory are
    # construction-time `serde_json::to_value(&NotifyMetricsSnapshot)`
    # invariants (`mod.rs:318-319`, `linux.rs:1381-1382`) — fixed
    # struct shape, cannot fail. **If you add D-Bus inbound, IPC
    # message handlers, or a plugin/JSON deserializer to this
    # directory, this entry MUST move to SUBSYSTEM_DIRS** — the
    # safety claim only holds while inputs are typed Rust values
    # from same-process callers.
    'notifier',
    # Phase 6a operator inspection surface
    # (`tinianchor/src/operator/`). Read-only query layer over the
    # Phase 2/3/4 modules (audit / ledger / memory / sessions /
    # disclosure / brain). Inputs are typed structs from same-process
    # callers (the storage adapter walks its backing store and hands
    # a `Vec<AuditEntry>` / `Vec<TaggedRecord>` to the inspector); no
    # external bytes reach `.unwrap()`. The serde paths in
    # `audit_inspect::export_signed_log` / `import_signed_log` use
    # `?` propagation, not unwrap. Promote to `SUBSYSTEM_DIRS` if a
    # future operator import accepts raw user-uploaded bundles via
    # `unwrap` (today: `serde_json::from_slice` returns `Result`).
    'operator',
    'pipeline', 'playbook', 'prompt', 'rag', 'reference',
    'registry', 'resource', 'routines', 'runtime', 'schedule',
    'session', 'settings', 'shortcuts', 'skillhub', 'skills', 'slash_commands',
    # `sop/` is the SOP (Standard Operating Procedure) engine — a sibling of
    # `routines/` and `jobs/` (both already here). SOP definitions come from
    # the product manifest via a `SopSource` seam and runs are reloaded from
    # our own session DB rows — product-controlled config + self-authored
    # state, not adversarial network/file bytes. Same trust model as
    # `routines`/`triggers`. The runner uses `?`/`Result` plumbing for all
    # fallible paths; baseline unchanged (7). Promote to `SUBSYSTEM_DIRS` if a
    # future revision ever decodes an untrusted on-the-wire SOP manifest.
    'sop',
    'styles', 'subagents',
    # PR #503 (argo-anchor flat→submodule split) — see `anchor` above
    # for the per-dir rationale. `subsystem/` and `tracer/` ride
    # the same "in-process coordination over typed structs" shape.
    'subsystem',
    'summarize', 'summarizer', 'sync',
    'tools', 'traits',
    'tracer',
    'transport', 'triage', 'triggers', 'tui', 'types', 'ui',
    # The argo-server `users/` module (split from a flat `users.rs`
    # into focused submodules) is a SQLite-backed account store.
    # All SQL goes through parameterised `params![]` INSERTs /
    # SELECTs over typed `UserRecord` shapes — no user-controlled
    # bytes reach unwrap. Password material is mediated by argon2id
    # (`crate::auth::password::{hash_password, verify_password}`),
    # which already lives in `SUBSYSTEM_DIRS` via `auth`. The email
    # /password validation that runs in this module is shape-
    # checking (`.contains('@')`, `len() < 8`) and produces typed
    # `RegisterError` variants on rejection. Promote to
    # SUBSYSTEM_DIRS if a future revision adds a raw OIDC-token
    # parser (today the OIDC token decode happens in
    # `argo-auth::oidc`, which is already audited).
    'users',
    'vfs', 'workspace',
    # `argo-wasm/src/host_imports/` — cell-side safe wrappers over
    # the `host_*` extern "C" ABI. On wasm32 the cell calls them
    # to package its own outbound traffic; on native they stub to
    # `HostError::NotWired`. Neither side consumes adversarial
    # input — the cell IS the caller, and the data crossing the
    # ABI is bytes the cell produced. The `list_skills` wrapper
    # deserializes JSON produced by the trusted host. Promote to
    # SUBSYSTEM_DIRS only if a future ABI change has the cell
    # consume bytes from a different (untrusted) source.
    'host_imports',
    # `argo-wasm/src/wasm_runtime/` — cell-side dispatcher +
    # streaming-chunk wrapper. The dispatcher serializes a
    # typed `LlmRequest` for the host; `HostChunkStream::poll_next`
    # deserializes `LlmChunk` JSON returned BY the trusted host's
    # dispatcher. No untrusted bytes enter the unwrap surface
    # (host-produced chunks are not adversarial to the cell).
    # Promote to SUBSYSTEM_DIRS if a future seam has the cell
    # decode bytes from another wasm cell or an untrusted source.
    'wasm_runtime',
    # `tinicore/src/impact/` — the counterfactual "savings" ledger +
    # estimators. Inputs are all numeric measurements produced BY other
    # trusted Core subsystems (token counts, tool durations, byte deltas)
    # and the in-crate price table; nothing here parses external /
    # untrusted bytes, and the estimators are pure f64/u64 arithmetic.
    # `serde_json::Value` only flows OUTWARD as provenance (serialize-only;
    # `ImpactEvent` does not derive Deserialize). NOT ADVERSARIAL. Promote
    # to SUBSYSTEM_DIRS if a future revision deserialises an `ImpactEvent`
    # (or a baseline) from an untrusted source.
    'impact',
]
# Z.ai PR-92 round 14 issue 2: also match `<dir-name>.rs` filenames.
# A file like `tinicore/src/agent/sub_agents/manifest.rs` IS a
# parser-shaped file (it parses sub-agent manifest YAML) but lives
# OUTSIDE a `manifest/` directory, so the `\manifest\` fragment
# above doesn't match it. Adding a per-name `\manifest.rs` (and
# `\parser.rs`, etc.) suffix match catches single-file modules
# that share the audit subsystem's input-handling shape. Files
# that just happen to have the same basename in unrelated
# subsystems (`xmlparser/parser.rs`) still get included — that's
# fine: they're MORE likely to be audit-relevant than less.
SUBSYSTEM_FILENAMES = [f'{os.sep}{d}.rs' for d in SUBSYSTEM_DIRS]
# Z.ai PR-92 round 12 issue 7: scoping limitation. The fragments
# require each subsystem name to appear as a FULL PATH COMPONENT
# (`tinicore/src/protocols/...` matches `\protocols\`). This means
# nested-relocation patterns like `tinicore/src/net/protocols/` —
# protocols moved INSIDE net/ — would NOT match. The script is a
# triage tool, not a strict gate, so the trade-off is acceptable:
# adding a partial-path matcher would produce false positives on
# unrelated `net/protocols-doc/`-shaped paths. If a future
# refactor moves an audited subsystem under a parent directory,
# either:
#   (a) update SUBSYSTEM_DIRS to include the new parent
#       (e.g., add `'net'`), accepting that all of net/ is
#       audit-scoped now; or
#   (b) keep the audit anchored on the leaf subsystem name and
#       update CLAUDE.md's audit policy to call out the
#       relocation.

# `--all` shows every result; default truncates at 60 to keep the
# summary scannable. Z.ai PR-92 round 6 issue 1 — the previous
# summary printed the full `len(results)` even though only the
# first 60 were displayed, misleading operators into believing
# they'd seen the whole list.
DISPLAY_LIMIT = 60


def find_hits(root_dir='.'):
    """Walk every `.rs` file under `root_dir` whose path contains
    an audited subsystem dir / filename, and return a list of
    `(path, lno, kw, line)` tuples for production-code
    `unwrap()` / `expect()` sites that match a risky-keyword
    pattern. Z.ai PR-92 round 15 issue 2: factored out from
    the previous top-level walk so importing this module for
    testing or programmatic reuse doesn't trigger a full
    workspace scan + the side-effecting golden-file check. Z.ai
    PR-94 round 13 issue 2: docstring previously said "yield" —
    the function accumulates into a list and returns it (not a
    generator).
    """
    # Z.ai PR-94 round 14 issue 1: switched from raw `os.walk` +
    # post-filter to the shared `walk_rs_files` helper (which uses
    # `walk_src_tree`'s in-place `dirs[:]` pruning). The previous
    # shape walked into `target/` (a multi-GB Rust build artifact
    # dir) and `node_modules/` and filtered by path AFTER the
    # walker yielded every `.rs` file inside them — exactly the
    # perf gap that round 9 issue 1 closed for the other two
    # filter scripts. Now risky_unwrap.py shares the contract.
    results = []
    for path in walk_rs_files(root_dir):
        # Match either a subsystem directory (`/manifest/`) OR a
        # subsystem-shaped filename (`/manifest.rs`). The latter
        # catches single-file modules like
        # `agent/sub_agents/manifest.rs` that share the audit
        # subsystem's input-handling shape but live outside the
        # matching directory. Z.ai PR-92 round 14 issue 2.
        if not any(k in path for k in (*SUBSYSTEM_FRAGMENTS, *SUBSYSTEM_FILENAMES)):
            continue
        for lno, line, line_no_comments in walk_production_lines(path):
            unwrap_m = UNWRAP_RE.search(line_no_comments)
            if not unwrap_m:
                continue
            if RISKY_RE.search(line_no_comments):
                results.append(
                    (path, lno, unwrap_m.group(1), line.rstrip())
                )
    return results


def find_unmatched_dirs(root_dir='.'):
    """Surface top-level `<crate>/src/` subsystem dirs under
    `root_dir` that are in NEITHER `SUBSYSTEM_DIRS` nor
    `KNOWN_NON_ADVERSARIAL_DIRS`. Thin wrapper over
    `_audit_walk.find_unmatched_subsystem_dirs` — the actual
    traversal lives in the shared module so `audit_panics_ci.py`
    can call the same function directly without a fragile
    "subprocess THEN import" dual-use pattern. Z.ai PR-94 round
    11 issue 7 + round 12 issue 4.
    """
    return find_unmatched_subsystem_dirs(
        root_dir, SUBSYSTEM_DIRS, KNOWN_NON_ADVERSARIAL_DIRS,
    )


def main():
    show_all = '--all' in sys.argv
    results = find_hits()
    shown = results if show_all else results[:DISPLAY_LIMIT]
    for path, lno, kw, line in shown:
        # Z.ai PR-94 round 4 issue 6 + round 6 issue 4: structured
        # tuple format aligned with `panic_filter.py`'s shape;
        # formatting happens here so a future programmatic
        # consumer can read the raw tuples.
        print(f"{path}:{lno} [{kw}] {line[:140]}")
    # Z.ai PR-94 round 13 issue 4: standardised
    # `Total (script-name): N` format across all three filter
    # scripts. The previous `Total risky candidates: N` was the
    # only outlier; the audit gate's regex used to need a
    # transitional-compat alternative for it. The truncation hint
    # moves to a separate line so the `Total:` matcher has the
    # same shape across all three scripts.
    if not show_all and len(results) > DISPLAY_LIMIT:
        print(
            f"\nTotal (risky_unwrap.py): {len(results)} "
            f"(showing first {DISPLAY_LIMIT}; rerun with --all to see the rest)"
        )
    else:
        print(f"\nTotal (risky_unwrap.py): {len(results)}")

    # Z.ai PR-94 round 11 issue 7: surface uncovered subsystem dirs
    # so a future `proto/` or `webhook/` addition that takes
    # adversarial input doesn't silently miss the audit. The hard
    # gate (exit code 3) lives in `audit_panics_ci.py`; this output
    # is the human-readable companion for CLI runs.
    unmatched = find_unmatched_dirs()
    if unmatched:
        print(
            f"\n::notice::risky_unwrap.py: {len(unmatched)} new "
            f"directorie(s) under `<crate>/src/` are in NEITHER "
            f"`SUBSYSTEM_DIRS` nor `KNOWN_NON_ADVERSARIAL_DIRS` — "
            f"add to one or the other in this script: "
            f"{', '.join(unmatched)}"
        )
        print(
            f"\nnew uncategorised directorie(s) under "
            f"`<crate>/src/` (decide ADVERSARIAL or NOT and add to "
            f"the corresponding list in this script):"
        )
        for d in unmatched:
            print(f"  {d}")


if __name__ == '__main__':
    # Z.ai PR-92 round 17 issue 7: `_check_golden_count()` runs
    # FIRST so `--list-keywords` (used by the golden diff) doesn't
    # bypass it. The previous order put the `--list-keywords` early
    # exit at module scope (above this guard), so a stale golden
    # file could pass the diff and still slip through CI.
    _check_golden_count()
    if '--list-keywords' in sys.argv:
        # Designed for CI to diff against a golden file:
        #   diff <(python3 scripts/risky_unwrap.py --list-keywords) \
        #        scripts/risky_keywords.golden
        # A diff means the keyword set changed without a
        # corresponding golden-file update — surfaces silent drift.
        # Z.ai PR-92 round 11 issue 7.
        for kw in RISKY_KEYWORDS:
            print(kw)
        sys.exit(0)
    main()
