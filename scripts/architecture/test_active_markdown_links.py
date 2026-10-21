#!/usr/bin/env python3
"""Regression checks for preserved-source link contexts; no evidence writes."""

from contextlib import ExitStack, redirect_stdout
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_active_markdown_links as links


class PresentationLinkContextsTest(unittest.TestCase):
    def setUp(self):
        repository = links.ROOT
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.root = Path(self.stack.enter_context(tempfile.TemporaryDirectory())).resolve()
        evidence = self.root / "results/architecture-evaluation/current"
        self.stack.enter_context(patch.object(links, "ROOT", self.root))
        self.stack.enter_context(patch.object(links, "CURRENT_EVIDENCE", evidence))
        self.stack.enter_context(patch.object(links, "ACTIVE_INPUTS", (self.root,)))
        self.stack.enter_context(patch.object(links, "restored_markdown_contexts", return_value=({}, [])))
        self.stack.enter_context(patch.object(links, "published_common15_contexts", return_value=({}, set(), [])))
        self.contexts = links.presentation_markdown_contexts()
        for snapshot, record in self.contexts.items():
            content = (repository / snapshot.relative_to(self.root)).read_bytes()
            for destination in (snapshot, record["context"]):
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(content)
        # Populate the real link destinations in the isolated fixture.
        for record in self.contexts.values():
            original = record["context"]
            for match in links.LINK.finditer(original.read_text()):
                target = links.local_target(match.group(1))
                if target is not None:
                    destination = (original.parent / target).resolve()
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    if not destination.exists():
                        destination.write_text("")
        self.snapshot = next(iter(self.contexts))

    def check(self):
        output = io.StringIO()
        with redirect_stdout(output):
            result = links.main()
        return result, output.getvalue()

    def test_all_fourteen_snapshots_are_checked_in_original_context(self):
        self.assertEqual(len(self.contexts), 14)
        self.assertEqual(self.check()[0], 0)

    def test_changed_snapshot_is_rejected(self):
        with self.snapshot.open("ab") as stream:
            stream.write(b"\nchanged evidence\n")
        result, output = self.check()
        self.assertEqual(result, 1)
        self.assertIn("SHA256 mismatch", output)

    def test_missing_link_target_is_still_rejected(self):
        target = self.root / "results/architecture-evaluation/current/four-dp-poc-evidence-20261019/README.md"
        target.unlink()
        result, output = self.check()
        self.assertEqual(result, 1)
        self.assertIn("missing: ../../../results/", output)

    def test_missing_original_context_is_rejected(self):
        self.contexts[self.snapshot]["context"].unlink()
        result, output = self.check()
        self.assertEqual(result, 1)
        self.assertIn("recorded Markdown context is missing", output)

    def test_unregistered_snapshot_does_not_gain_context(self):
        extra = self.snapshot.parents[6] / "unregistered/frozen/source/README.md"
        extra.parent.mkdir(parents=True)
        extra.write_bytes(self.snapshot.read_bytes())
        result, output = self.check()
        self.assertEqual(result, 1)
        self.assertIn("unregistered/frozen/source/README.md", output)
        self.assertIn("missing:", output)

    def test_ordinary_link_cannot_escape_repository(self):
        (self.root / "README.md").write_text("[outside](../outside.md)\n")
        result, output = self.check()
        self.assertEqual(result, 1)
        self.assertIn("outside repository", output)


if __name__ == "__main__":
    unittest.main()
