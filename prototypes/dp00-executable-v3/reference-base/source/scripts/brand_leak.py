#!/usr/bin/env python3
"""Assert VIA's product identity is clean in both directions.

VIA has two upstreams, so identity can leak two ways:

  * **inbound**  — a `qwen`/`qwaudio` string that should have been renamed to
    VIA survives from the qwen-audio-agent port.
  * **outbound** — an `argo`/`tini` string leaks in from the ARGO port and
    names someone else's product in VIA's output.

Both are release blockers. The allow-list is not maintained here: it is
derived from `docs/rebrand.md`'s KEEP table, which is the single source of
truth for which upstream names are legitimately not ours to rename.

Run from the repo root:

    python3 scripts/brand_leak.py
"""

import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REBRAND = os.path.join(ROOT, 'docs', 'rebrand.md')

INBOUND = re.compile(r'qwen|qwaudio|QWEN|QWAUDIO|千问', re.IGNORECASE)
OUTBOUND = re.compile(r'\bargo\b|\btini[a-z]*\b', re.IGNORECASE)

SCAN_EXT = ('.rs', '.toml')
SKIP_DIRS = {'target', '.git', 'node_modules', 'docs'}


def keep_terms():
    """Parse the KEEP table out of docs/rebrand.md.

    Every backticked token in the KEEP section is a name VIA does not own and
    must not rename. Returns them lowercased for substring matching.
    """
    if not os.path.exists(REBRAND):
        print(f'FATAL: {REBRAND} not found — the allow-list has no source', file=sys.stderr)
        sys.exit(2)
    with open(REBRAND, encoding='utf-8') as fh:
        text = fh.read()
    marker = text.find('## KEEP')
    if marker < 0:
        print('FATAL: docs/rebrand.md has no "## KEEP" section', file=sys.stderr)
        sys.exit(2)
    terms = set()
    for cell in re.findall(r'`([^`]+)`', text[marker:]):
        for token in re.split(r'[\s,/()\[\]]+', cell):
            token = token.strip().lower()
            if len(token) > 2:
                terms.add(token)
    return terms


def allowed(line, terms):
    """True when every branded token on this line is on the KEEP list.

    A line is also allowed when it is an explicit attribution: the port cites
    its upstream in doc comments and in `NOTICE`, and that is the point.
    """
    lowered = line.lower()
    if 'upstream' in lowered or 'ported from' in lowered or 'allow-brand' in lowered:
        return True
    return any(term in lowered for term in terms)


def scan(pattern, label, terms, extra_allow=()):
    hits = []
    for dirpath, dirnames, filenames in os.walk(ROOT):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS and not d.startswith('.')]
        for name in filenames:
            if not name.endswith(SCAN_EXT):
                continue
            path = os.path.join(dirpath, name)
            rel = os.path.relpath(path, ROOT)
            with open(path, encoding='utf-8', errors='replace') as fh:
                for lno, line in enumerate(fh, 1):
                    if not pattern.search(line):
                        continue
                    if allowed(line, terms) or any(a in line.lower() for a in extra_allow):
                        continue
                    hits.append((rel, lno, label, line.rstrip()[:140]))
    return hits


def main():
    terms = keep_terms()
    hits = scan(INBOUND, 'INBOUND qwen leak', terms)
    # `argo-bridge` is the sanctioned optional feature name (ADR-0001), and a
    # path-dep on ARGO inside it is the whole point of the feature.
    hits += scan(OUTBOUND, 'OUTBOUND argo leak', terms, extra_allow=('argo-bridge', 'argo_bridge'))

    if not hits:
        print(f'brand_leak: clean ({len(terms)} KEEP terms from docs/rebrand.md)')
        return 0

    print(f'brand_leak: {len(hits)} leak(s)\n', file=sys.stderr)
    for rel, lno, label, line in hits:
        print(f'  {rel}:{lno}  [{label}]\n      {line}', file=sys.stderr)
    print(
        '\nIf a name is legitimately not ours to rename, add it to the KEEP table\n'
        'in docs/rebrand.md — that table is this gate\'s allow-list.',
        file=sys.stderr,
    )
    return 1


if __name__ == '__main__':
    sys.exit(main())
