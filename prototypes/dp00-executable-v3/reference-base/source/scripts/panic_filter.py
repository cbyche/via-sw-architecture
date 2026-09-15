"""Find production `panic!()` / `todo!()` / `unimplemented!()` sites.

Walks every `.rs` file under any `*/src/` tree and reports macro hits
that live OUTSIDE both `#[cfg(test)] mod ... { }` and `#[test]` /
`#[tokio::test]` standalone test functions. The cfg-test tracking
logic is shared with the other two audit scripts via
`scripts/_audit_walk.py` so a fix in one applies to all three.

Run from the repo root:

    python3 scripts/panic_filter.py

Originally written for the May 2026 panic audit. Re-runnable.
"""

import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _audit_walk import walk_production_lines, walk_rs_files  # noqa: E402

PATTERN = re.compile(r'\b(panic|todo|unimplemented)!\s*\(')


def find_hits(root_dir='.'):
    """Walk every `.rs` file under `root_dir` and return a list of
    `(path, lno, macro, line)` tuples for production-code panic
    sites. Z.ai PR-94 round 6 issue 5: factored out so a future
    richer CI gate can import this module and inspect individual
    hits programmatically — not just the `Total:` line. Z.ai
    PR-94 round 7 issue 3: file-walking moved to the shared
    `walk_rs_files` helper so the three audit scripts can't drift
    on the skip-list / `\\src\\` fragment match. Z.ai PR-94 round
    13 issue 2: docstring previously said "yield" but the function
    accumulates into a list and returns it (not a generator).
    """
    results = []
    for path in walk_rs_files(root_dir):
        for lno, line, line_no_comments in walk_production_lines(path):
            for m in PATTERN.finditer(line_no_comments):
                results.append((path, lno, m.group(1), line.rstrip()))
    return results


def main():
    results = find_hits()
    for path, lno, macro, line in results:
        print(f"{path}:{lno} [{macro}] {line[:120]}")
    # Z.ai PR-94 round 8 issue 7: standardised `Total
    # (script-name): N` format across all three filter scripts so
    # cross-referencing CI logs is uniform. The audit-gate regex
    # in `audit_panics_ci.py` accepts either the new
    # `Total (name.py): N` or the legacy
    # `Total[risky candidates]?: N hits…` for transitional
    # compat.
    print(f"\nTotal (panic_filter.py): {len(results)}")


if __name__ == "__main__":
    main()
