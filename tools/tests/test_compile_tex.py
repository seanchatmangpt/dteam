from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

MODULE_PATH = Path(__file__).resolve().parents[1] / "compile_tex.py"
SPEC = importlib.util.spec_from_file_location("compile_tex", MODULE_PATH)
assert SPEC and SPEC.loader
COMPILE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(COMPILE)


class CompileTexTests(unittest.TestCase):
    def test_rejects_non_tex_source(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "document.txt"
            source.write_text("not tex", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "expected a .tex source"):
                COMPILE.compile_tex(source, root / "out", "pdflatex", 2)

    def test_rejects_zero_passes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "document.tex"
            source.write_text("tex", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "passes must be at least 1"):
                COMPILE.compile_tex(source, root / "out", "pdflatex", 0)

    @patch.object(COMPILE.shutil, "which", return_value=None)
    def test_reports_missing_engine(self, _which) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "document.tex"
            source.write_text("tex", encoding="utf-8")
            with self.assertRaisesRegex(FileNotFoundError, "TeX engine not found"):
                COMPILE.compile_tex(source, root / "out", "missing-tex", 1)


if __name__ == "__main__":
    unittest.main()
