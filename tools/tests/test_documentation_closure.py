from __future__ import annotations

import importlib.util
import subprocess
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parents[1] / "documentation_closure.py"
SPEC = importlib.util.spec_from_file_location("documentation_closure", MODULE_PATH)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class DocumentationClosureTests(unittest.TestCase):
    def test_archive_path_preserves_original_path(self) -> None:
        self.assertEqual(
            MODULE.archive_path("docs/legacy/design.md"),
            "docs/archive/source/docs/legacy/design.md.txt",
        )

    def test_relative_link_is_stable(self) -> None:
        self.assertEqual(
            MODULE.relative_link(
                "crates/example/README.md",
                "docs/archive/source/crates/example/README.md.txt",
            ),
            "../../docs/archive/source/crates/example/README.md.txt",
        )

    def test_stub_points_to_archive_and_authority_map(self) -> None:
        stub = MODULE.superseded_stub(
            "notes/old.md",
            "docs/archive/source/notes/old.md.txt",
        )
        self.assertTrue(stub.startswith(MODULE.SUPERSEDED_MARKER))
        self.assertIn("../docs/archive/source/notes/old.md.txt", stub)
        self.assertIn("../docs/DOCUMENTATION_MAP.md", stub)

    def test_rewrite_is_idempotent_and_preserves_exact_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(("git", "init", "-q"), cwd=root, check=True)
            source = root / "legacy.md"
            original = b"# Legacy\n\nHistorical [link](missing.md).\n"
            source.write_bytes(original)
            subprocess.run(("git", "add", "legacy.md"), cwd=root, check=True)

            rewritten = MODULE.rewrite_noncanonical(root)
            self.assertEqual(rewritten, [("legacy.md", "docs/archive/source/legacy.md.txt")])
            self.assertEqual(
                (root / "docs/archive/source/legacy.md.txt").read_bytes(),
                original,
            )
            first_stub = source.read_bytes()

            self.assertEqual(MODULE.rewrite_noncanonical(root), [])
            self.assertEqual(source.read_bytes(), first_stub)
            self.assertEqual(
                (root / "docs/archive/source/legacy.md.txt").read_bytes(),
                original,
            )

    def test_parse_links_ignores_remote_and_anchor_targets(self) -> None:
        text = "[local](docs/a.md) [remote](https://example.com/x) [anchor](#section)"
        self.assertEqual(tuple(MODULE.parse_links(text)), ("docs/a.md",))


if __name__ == "__main__":
    unittest.main()
