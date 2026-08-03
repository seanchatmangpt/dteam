from __future__ import annotations

import importlib.util
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("innovation_80_20", ROOT / "tools/innovation_80_20.py")
assert SPEC and SPEC.loader
AUDIT = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = AUDIT
SPEC.loader.exec_module(AUDIT)


class Innovation8020Tests(unittest.TestCase):
    def fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temp = tempfile.TemporaryDirectory(prefix="dteam-8020-test-")
        target = Path(temp.name)
        for relative in (
            "Makefile",
            "rust-toolchain.toml",
            "src/config.rs",
            "tools/innovation_80_20.py",
            "tests/test_innovation_80_20.py",
            "docs/innovation-80-20.md",
        ):
            source = ROOT / relative
            destination = target / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
        return temp, target

    def assert_failed(self, root: Path, check_id: str) -> None:
        report = AUDIT.build_report(root, False)
        statuses = {check["id"]: check["standing"] for check in report["checks"]}
        self.assertEqual(statuses[check_id], "FAILED")
        self.assertEqual(report["standing"], "BUILD_BROKEN")

    def test_current_source_audit_is_alive(self) -> None:
        report = AUDIT.build_report(ROOT, False)
        self.assertEqual(report["standing"], "ALIVE")
        self.assertEqual(report["summary"]["failed"], 0)

    def test_fail_open_doctor_mutant_is_killed(self) -> None:
        temp, root = self.fixture()
        self.addCleanup(temp.cleanup)
        path = root / "Makefile"
        path.write_text(path.read_text().replace(
            '$(PYTHON) tools/innovation_80_20.py doctor --output-dir "$(AUDIT_DIR)"',
            'cargo run --example doctor 2>/dev/null || echo "diagnostics skipped"',
        ))
        self.assert_failed(root, "DX-FAIL-LOUD")

    def test_floating_nightly_mutant_is_killed(self) -> None:
        temp, root = self.fixture()
        self.addCleanup(temp.cleanup)
        path = root / "rust-toolchain.toml"
        path.write_text(path.read_text().replace("nightly-2026-06-02", "nightly"))
        self.assert_failed(root, "BUILD-PINNED-NIGHTLY")

    def test_missing_strict_config_mutant_is_killed(self) -> None:
        temp, root = self.fixture()
        self.addCleanup(temp.cleanup)
        path = root / "src/config.rs"
        path.write_text(path.read_text().replace("pub fn load_required", "pub fn load_optional_again"))
        self.assert_failed(root, "CONFIG-STRICT-LOAD")

    def test_missing_config_validation_mutant_is_killed(self) -> None:
        temp, root = self.fixture()
        self.addCleanup(temp.cleanup)
        path = root / "src/config.rs"
        path.write_text(path.read_text().replace("pub fn validate(&self)", "pub fn inspect_only(&self)"))
        self.assert_failed(root, "CONFIG-VALIDATION")

    def test_two_pass_replay_is_exact(self) -> None:
        with tempfile.TemporaryDirectory(prefix="dteam-8020-replay-") as output:
            replay = AUDIT.run_replay(ROOT, Path(output))
            self.assertEqual(replay["result"], "REPLAY_MATCH")
            self.assertEqual(replay["standing"], "ALIVE")
            report = json.loads((Path(output) / "innovation-80-20.json").read_text())
            self.assertEqual(report["standing"], "ALIVE")


if __name__ == "__main__":
    unittest.main()
