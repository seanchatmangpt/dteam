from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parents[1] / "chicago_validator.py"
SPEC = importlib.util.spec_from_file_location("chicago_validator", MODULE_PATH)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class ChicagoValidatorTests(unittest.TestCase):
    def test_public_module_inventory_is_exact_and_sorted(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "lib.rs"
            path.write_text("pub mod zebra;\npub mod alpha;\npub mod alpha;\n", encoding="utf-8")
            self.assertEqual(MODULE.public_modules(path), ("alpha", "zebra"))

    def test_structure_refuses_uncovered_capability(self) -> None:
        scenario = MODULE.Scenario("one", ("alpha",), ("true",))
        self.assertEqual(MODULE.validate_structure(("alpha", "beta"), (scenario,)), ("beta",))

    def test_standard_suite_covers_every_declared_kernel_module(self) -> None:
        root = Path(__file__).resolve().parents[2]
        manifest = root / "capabilities/dteam-kernel/Cargo.toml"
        modules = MODULE.public_modules(manifest.parent / "src/lib.rs")
        suite = MODULE.scenarios(manifest)
        self.assertEqual(MODULE.validate_structure(modules, suite), ())

    def test_every_profile_has_wizard_and_composition_scenarios(self) -> None:
        manifest = Path("capabilities/dteam-kernel/Cargo.toml")
        ids = {scenario.id for scenario in MODULE.scenarios(manifest)}
        for profile in MODULE.PROFILES:
            self.assertIn(f"wizard-{profile}", ids)
            self.assertIn(f"compose-{profile}", ids)

    def test_negative_crown_is_explicit_not_silently_green(self) -> None:
        manifest = Path("capabilities/dteam-kernel/Cargo.toml")
        crown = next(item for item in MODULE.scenarios(manifest) if item.id == "crown-negative-control")
        self.assertEqual(crown.expected_exit, 3)
        self.assertIn("VISION_2030", crown.required_text)

    def test_scenarios_forbid_mock_evidence(self) -> None:
        manifest = Path("capabilities/dteam-kernel/Cargo.toml")
        for scenario in MODULE.scenarios(manifest):
            self.assertIn("mock", scenario.forbidden_text)


if __name__ == "__main__":
    unittest.main()
