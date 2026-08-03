from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parents[1] / "verify_repository.py"
SPEC = importlib.util.spec_from_file_location("verify_repository", MODULE_PATH)
assert SPEC and SPEC.loader
VERIFY = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = VERIFY
SPEC.loader.exec_module(VERIFY)

TEST_POLICY = {
    "active_python_roots": ["."],
    "excluded_python_roots": [],
    "archived_python_roots": [],
}


class VerifyRepositoryTests(unittest.TestCase):
    def test_clean_tree_passes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "ok.py").write_text(
                "from pathlib import Path\nPath('out').write_text('ok')\n",
                encoding="utf-8",
            )
            (root / "ok.rs").write_text(
                "fn main() { match 1 { 1 => (), _ => () } }\n",
                encoding="utf-8",
            )
            self.assertEqual([], VERIFY.verify_python(root, TEST_POLICY))
            self.assertEqual([], VERIFY.verify_rust(root))

    def test_detects_workstation_path_and_ambient_mutation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "bad.py").write_text(
                "import subprocess\n"
                "path = '/Users/example/project/out.txt'\n"
                "with open(path, 'w') as handle:\n"
                "    handle.write('x')\n"
                "subprocess.run(['false'])\n",
                encoding="utf-8",
            )
            codes = {finding.code for finding in VERIFY.verify_python(root, TEST_POLICY)}
            self.assertEqual(
                {"WORKSTATION_PATH", "AMBIENT_MUTATION", "UNCHECKED_SUBPROCESS"},
                codes,
            )

    def test_detects_python_syntax_error_outside_active_roots(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "bad.py").write_text("def broken(:\n", encoding="utf-8")
            policy = {
                "active_python_roots": [],
                "excluded_python_roots": [],
                "archived_python_roots": ["."],
            }
            findings = VERIFY.verify_python(root, policy)
            self.assertEqual("PYTHON_SYNTAX", findings[0].code)

    def test_detects_fused_rust_match_arms(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "bad.rs").write_text(
                "match value { Variant::A => { 1 } Variant::B => 2 }\n",
                encoding="utf-8",
            )
            findings = VERIFY.verify_rust(root)
            self.assertEqual("MALFORMED_MATCH_ARM", findings[0].code)

    def test_policy_rejects_active_archive_overlap(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "tools").mkdir()
            policy = {
                "active_python_roots": ["tools"],
                "excluded_python_roots": [],
                "archived_python_roots": ["tools"],
            }
            findings = VERIFY.verify_policy(root, policy)
            self.assertEqual("POLICY_OVERLAP", findings[0].code)


if __name__ == "__main__":
    unittest.main()
