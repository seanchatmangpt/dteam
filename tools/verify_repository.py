#!/usr/bin/env python3
"""Fast, dependency-free closure verifier for the admitted dteam source tree.

All Python is syntax-checked. Portability and mutation rules apply only to
explicitly admitted capability roots from closure-policy.json. Historical
migration scripts remain readable lineage evidence without being mistaken for
supported production entry points.
"""

from __future__ import annotations

import argparse
import ast
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable

ABSOLUTE_WORKSTATION_PATHS = (
    re.compile(r"/Users/[A-Za-z0-9._-]+/"),
    re.compile(r"[A-Za-z]:\\\\Users\\\\[^\\\\]+\\\\"),
)
MUTATING_OPEN = re.compile(r"open\([^\n]+,[ ]*[\"'](?:w|a|x|w\+|a\+)[\"']")
UNCHECKED_SUBPROCESS = re.compile(r"subprocess\.run\((?![^\n]*check\s*=\s*True)")
IGNORED_DIRS = {".git", "target", ".venv", "venv", "node_modules", "__pycache__"}
DEFAULT_POLICY = {
    "active_python_roots": ["tools", "scripts"],
    "excluded_python_roots": ["tools/tests"],
    "archived_python_roots": [],
}


@dataclass(frozen=True)
class Finding:
    code: str
    path: str
    line: int
    message: str


def source_files(root: Path, suffix: str) -> Iterable[Path]:
    for path in root.rglob(f"*{suffix}"):
        if not any(part in IGNORED_DIRS for part in path.parts):
            yield path


def line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def load_policy(root: Path) -> dict[str, object]:
    path = root / "closure-policy.json"
    if not path.exists():
        return dict(DEFAULT_POLICY)
    data = json.loads(path.read_text(encoding="utf-8"))
    for key in ("active_python_roots", "excluded_python_roots", "archived_python_roots"):
        value = data.get(key, [])
        if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
            raise ValueError(f"{path}: {key} must be a list of strings")
    return data


def is_under(relative_path: Path, roots: Iterable[str]) -> bool:
    for root in roots:
        root_path = Path(root)
        if relative_path == root_path or root_path in relative_path.parents:
            return True
    return False


def verify_python(root: Path, policy: dict[str, object] | None = None) -> list[Finding]:
    policy = policy or DEFAULT_POLICY
    active = policy.get("active_python_roots", [])
    excluded = policy.get("excluded_python_roots", [])
    findings: list[Finding] = []

    for path in source_files(root, ".py"):
        text = path.read_text(encoding="utf-8")
        relative = path.relative_to(root)
        rel = str(relative)
        try:
            ast.parse(text, filename=rel)
        except SyntaxError as error:
            findings.append(Finding("PYTHON_SYNTAX", rel, error.lineno or 1, error.msg))
            continue

        if not is_under(relative, active) or is_under(relative, excluded):
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


def verify_policy(root: Path, policy: dict[str, object]) -> list[Finding]:
    findings: list[Finding] = []
    active = policy.get("active_python_roots", [])
    archived = policy.get("archived_python_roots", [])
    overlap = sorted(set(active).intersection(archived))
    for item in overlap:
        findings.append(
            Finding(
                "POLICY_OVERLAP",
                "closure-policy.json",
                1,
                f"path cannot be both active and archived: {item}",
            )
        )
    for item in active:
        if not (root / item).exists():
            findings.append(
                Finding(
                    "MISSING_ACTIVE_ROOT",
                    "closure-policy.json",
                    1,
                    f"active capability root does not exist: {item}",
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
    policy = load_policy(root)
    findings = sorted(
        verify_policy(root, policy) + verify_python(root, policy) + verify_rust(root),
        key=lambda finding: (finding.path, finding.line, finding.code),
    )

    if args.json:
        print(
            json.dumps(
                {
                    "policy": policy,
                    "findings": [asdict(item) for item in findings],
                },
                indent=2,
            )
        )
    elif findings:
        for item in findings:
            print(f"{item.path}:{item.line}: {item.code}: {item.message}")
        print(f"closure verification failed: {len(findings)} finding(s)", file=sys.stderr)
    else:
        print("closure verification passed")

    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
