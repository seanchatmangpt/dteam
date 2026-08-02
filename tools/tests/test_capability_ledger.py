from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parents[1] / "capability_ledger.py"
SPEC = importlib.util.spec_from_file_location("capability_ledger", MODULE_PATH)
assert SPEC and SPEC.loader
LEDGER = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = LEDGER
SPEC.loader.exec_module(LEDGER)


class CapabilityLedgerTests(unittest.TestCase):
    def make_policy(self, root: Path) -> Path:
        path = root / "closure-policy.json"
        path.write_text(
            json.dumps(
                {
                    "active_python_roots": ["tools", "scripts"],
                    "archived_python_roots": ["legacy"],
                }
            ),
            encoding="utf-8",
        )
        return path

    def test_compiles_active_and_archived_surfaces(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_policy(root)
            (root / "tools").mkdir()
            (root / "legacy").mkdir()
            (root / "src").mkdir()
            (root / "tools" / "doctor.py").write_text(
                "def diagnose():\n    return 'ok'\n\n"
                "if __name__ == '__main__':\n    print(diagnose())\n",
                encoding="utf-8",
            )
            (root / "legacy" / "patch.py").write_text(
                "def patch():\n    return True\n",
                encoding="utf-8",
            )
            (root / "src" / "lib.rs").write_text(
                "pub fn replay() {}\n#[test]\nfn replay_is_stable() {}\n",
                encoding="utf-8",
            )

            ledger = LEDGER.compile_ledger(root, Path("closure-policy.json"))
            by_path = {item.path: item for item in ledger.capabilities}

            self.assertEqual("ALIVE", by_path["tools/doctor.py"].status)
            self.assertEqual(("__main__", "diagnose"), by_path["tools/doctor.py"].entry_points)
            self.assertEqual("ARCHIVED", by_path["legacy/patch.py"].status)
            self.assertEqual("ALIVE", by_path["src/lib.rs"].status)
            self.assertEqual(("replay",), by_path["src/lib.rs"].entry_points)
            self.assertEqual(1, by_path["src/lib.rs"].tests)
            self.assertEqual(64, len(ledger.digest))

    def test_unclassified_python_is_unknown(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_policy(root)
            (root / "misc.py").write_text("def run():\n    return 1\n", encoding="utf-8")

            ledger = LEDGER.compile_ledger(root, Path("closure-policy.json"))
            item = next(item for item in ledger.capabilities if item.path == "misc.py")
            self.assertEqual("UNKNOWN", item.status)
            self.assertIn("python-surface-not-admitted-by-policy", item.reasons)

    def test_python_syntax_failure_is_build_broken(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_policy(root)
            (root / "tools").mkdir()
            (root / "tools" / "broken.py").write_text("def broken(:\n", encoding="utf-8")

            ledger = LEDGER.compile_ledger(root, Path("closure-policy.json"))
            item = next(item for item in ledger.capabilities if item.path.endswith("broken.py"))
            self.assertEqual("BUILD_BROKEN", item.status)
            self.assertTrue(any(reason.startswith("syntax:") for reason in item.reasons))

    def test_digest_is_deterministic_and_content_sensitive(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_policy(root)
            (root / "tools").mkdir()
            source = root / "tools" / "capability.py"
            source.write_text("def value():\n    return 1\n", encoding="utf-8")

            first = LEDGER.compile_ledger(root, Path("closure-policy.json"))
            second = LEDGER.compile_ledger(root, Path("closure-policy.json"))
            self.assertEqual(first.digest, second.digest)

            source.write_text("def value():\n    return 2\n", encoding="utf-8")
            changed = LEDGER.compile_ledger(root, Path("closure-policy.json"))
            self.assertNotEqual(first.digest, changed.digest)


if __name__ == "__main__":
    unittest.main()
