#!/usr/bin/env python3
"""Fast, dependency-free closure verifier for the dteam source tree.

The verifier deliberately checks properties that can be established without
building external sibling repositories. It is suitable as the first CI gate and
as a local preflight before the full Rust/ggen validation ladder.
"""

from __future__ import annotations

import argparse
import ast
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ABSOLUTE_WORKSTATION_PATHS = (
    re.compile(r"/Users/[A-Za-z0-9._-]+/"),
    re.compile(r"[A-Za-z]:\\\\Users\\\\[^\\\\]+\\\\"),
)
MUTATING_OPEN = re.compile(r"open\([^\n]+,[ ]*[\"'](?:w|a|x|w\+|a\+)[\"']")
UNCHECKED_SUBPROCESS = re.compile(r"subprocess\.run\((?![^\n]*check\s*=\s*True)")

IGNORED_DIRS = {".git", "target", ".venv", "venv", "node_modules", "__pycache__"}


@dataclass(frozen=True)
class Finding:
    code: str
    path: str
    line: int
    message: str


def source_files(root: Path, suffix: str):
    for path in root.rglob(f"*{suffix}"):
        if not any(part in IGNORED_DIRS for part in path.parts):
            yield path


def line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def verify_python(root: Path) -> list[Finding]:
    findings: list[Finding] = []
    for path in source_files(root, ".py"):
        text = path.read_text(encoding="utf-8")
        rel = str(path.relative_to(root))
        try:
            ast.parse(text, filename=rel)
        except SyntaxError as error:
            findings.append(
                Finding("PYTHON_SYNTAX", rel, error.lineno or 1, error.msg)
            )
            continue

        for pattern in ABSOLUTE_WORKSTATION_PATHS:
            for match in pattern.finditer(text):
                findings.append(
                    Finding(
                        "WORKSTATION_PATH",
                        rel,
                        line_number(text, match.start()),
                        "developer-specific absolute path",
                    )
                )

        for match in MUTATING_OPEN.finditer(text):
            findings.append(
                Finding(
                    "AMBIENT_MUTATION",
                    rel,
                    line_number(text, match.start()),
                    "direct mutating open(); use pathlib with explicit target and refusal semantics",
                )
            )

        for match in UNCHECKED_SUBPROCESS.finditer(text):
            findings.append(
                Finding(
                    "UNCHECKED_SUBPROCESS",
                    rel,
                    line_number(text, match.start()),
                    "subprocess.run must use check=True",
                )
            )
    return findings


def verify_rust(root: Path) -> list[Finding]:
    findings: list[Finding] = []
    malformed_match = re.compile(r"\}[ \t]+[A-Za-z_][A-Za-z0-9_:]*\s*=>")
    for path in source_files(root, ".rs"):
        text = path.read_text(encoding="utf-8")
        rel = str(path.relative_to(root))
        for match in malformed_match.finditer(text):
            findings.append(
                Finding(
                    "MALFORMED_MATCH_ARM",
                    rel,
                    line_number(text, match.start()),
                    "match arms appear fused on one line",
                )
            )
    return findings


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    root = args.root.resolve(strict=True)
    findings = sorted(
        verify_python(root) + verify_rust(root),
        key=lambda finding: (finding.path, finding.line, finding.code),
    )

    if args.json:
        print(json.dumps({"findings": [asdict(item) for item in findings]}, indent=2))
    elif findings:
        for item in findings:
            print(f"{item.path}:{item.line}: {item.code}: {item.message}")
        print(f"closure verification failed: {len(findings)} finding(s)", file=sys.stderr)
    else:
        print("closure verification passed")

    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
