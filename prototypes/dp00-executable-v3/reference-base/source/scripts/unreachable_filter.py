"""Find production `unreachable!()` sites.

Companion to `panic_filter.py`; same approach but for `unreachable!()`
specifically. `unreachable!()` is a panic at the call site if the
"unreachable" branch is ever reached, so it deserves the same
production-vs-test triage as `panic!()`.

Run from the repo root:

    python3 scripts/unreachable_filter.py
"""

import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _audit_walk import walk_production_lines, walk_rs_files  # noqa: E402

PATTERN = re.compile(r'\bunreachable!\s*\(')


def find_hits(root_dir='.'):
    """Walk every `.rs` file under `root_dir` and return a list of
    `(path, lno, line)` tuples for production-code unreachable
    sites. Z.ai PR-94 round 6 issue 5 + round 7 issue 3: file-
    walking factored to the shared `walk_rs_files` helper. Z.ai
    PR-94 round 13 issue 2: docstring previously said "yield" —
    this function accumulates into a list and returns it (not a
    generator).
    """
    results = []
    for path in walk_rs_files(root_dir):
        for lno, line, line_no_comments in walk_production_lines(path):
            if PATTERN.search(line_no_comments):
                results.append((path, lno, line.rstrip()))
    return results


def main():
    results = find_hits()
    for path, lno, line in results:
        print(f"{path}:{lno} {line}")
    # Z.ai PR-94 round 8 issue 7: standardised
    # `Total (script-name): N` format across all three filter
    # scripts.
    print(f"\nTotal (unreachable_filter.py): {len(results)}")


if __name__ == "__main__":
    main()
