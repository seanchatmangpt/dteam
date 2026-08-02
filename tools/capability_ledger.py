#!/usr/bin/env python3
"""Compile a deterministic capability ledger from repository source and policy.

The ledger makes completion enumerable. Each admitted capability is identified by
its path, language, digest, executable entry points, test coverage hints, and
closure status. Archived lineage remains visible but cannot silently acquire
production standing.
"""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import re
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable

SUPPORTED_SUFFIXES = {".py", ".rs", ".toml", ".json", ".yml", ".yaml", ".md"}
IGNORED_PARTS = {".git", "target", ".venv", "venv", "node_modules", "__pycache__"}
RUST_TEST = re.compile(r"#\s*\[\s*test\s*\]")
RUST_PUBLIC = re.compile(r"\bpub\s+(?:async\s+)?(?:fn|struct|enum|trait|mod|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)")


@dataclass(frozen=True)
class Capability:
    path: str
    classification: str
    language: str
    sha256: str
    bytes: int
    lines: int
    entry_points: tuple[str, ...]
    tests: int
    status: str
    reasons: tuple[str, ...]


@dataclass(frozen=True)
class Ledger:
    schema: str
    root: str
    policy: str
    capabilities: tuple[Capability, ...]
    counts: dict[str, int]
    digest: str


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def language(path: Path) -> str:
    return {
        ".py": "python",
        ".rs": "rust",
        ".toml": "toml",
        ".json": "json",
        ".yml": "yaml",
        ".yaml": "yaml",
        ".md": "markdown",
    }.get(path.suffix.lower(), "other")


def load_policy(root: Path, path: Path) -> dict:
    policy_path = path if path.is_absolute() else root / path
    payload = json.loads(policy_path.read_text(encoding="utf-8"))
    if not isinstance(payload.get("active_python_roots"), list):
        raise ValueError("policy.active_python_roots must be an array")
    if not isinstance(payload.get("archived_python_roots"), list):
        raise ValueError("policy.archived_python_roots must be an array")
    return payload


def under(relative: Path, roots: Iterable[str]) -> bool:
    text = relative.as_posix()
    return any(text == root.rstrip("/") or text.startswith(root.rstrip("/") + "/") for root in roots)


def classify(relative: Path, policy: dict) -> str:
    if relative.suffix == ".py":
        if under(relative, policy["active_python_roots"]):
            return "active"
        if under(relative, policy["archived_python_roots"]):
            return "archived"
        return "unclassified"
    return "active"


def python_surface(text: str) -> tuple[tuple[str, ...], int, tuple[str, ...]]:
    reasons: list[str] = []
    try:
        tree = ast.parse(text)
    except SyntaxError as error:
        return (), 0, (f"syntax:{error.lineno or 1}:{error.msg}",)

    entries: list[str] = []
    tests = 0
    has_main = False
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            if node.name.startswith("test_"):
                tests += 1
            elif not node.name.startswith("_"):
                entries.append(node.name)
        if isinstance(node, ast.If):
            rendered = ast.unparse(node.test) if hasattr(ast, "unparse") else ""
            if "__name__" in rendered and "__main__" in rendered:
                has_main = True
    if has_main:
        entries.append("__main__")
    if not entries:
        reasons.append("no-public-entry-point")
    return tuple(sorted(set(entries))), tests, tuple(reasons)


def rust_surface(text: str) -> tuple[tuple[str, ...], int, tuple[str, ...]]:
    entries = tuple(sorted(set(RUST_PUBLIC.findall(text))))
    tests = len(RUST_TEST.findall(text))
    reasons = () if entries else ("no-public-entry-point",)
    return entries, tests, reasons


def capability_for(root: Path, path: Path, policy: dict) -> Capability:
    relative = path.relative_to(root)
    data = path.read_bytes()
    text = data.decode("utf-8")
    classification = classify(relative, policy)
    lang = language(path)

    if lang == "python":
        entries, tests, reasons = python_surface(text)
    elif lang == "rust":
        entries, tests, reasons = rust_surface(text)
    else:
        entries, tests, reasons = (), 0, ()

    if classification == "archived":
        status = "ARCHIVED"
    elif any(reason.startswith("syntax:") for reason in reasons):
        status = "BUILD_BROKEN"
    elif classification == "unclassified":
        status = "UNKNOWN"
        reasons = reasons + ("python-surface-not-admitted-by-policy",)
    else:
        status = "ALIVE"

    return Capability(
        path=relative.as_posix(),
        classification=classification,
        language=lang,
        sha256=sha256_bytes(data),
        bytes=len(data),
        lines=text.count("\n") + (0 if not text or text.endswith("\n") else 1),
        entry_points=entries,
        tests=tests,
        status=status,
        reasons=tuple(sorted(set(reasons))),
    )


def source_paths(root: Path) -> Iterable[Path]:
    for path in sorted(root.rglob("*")):
        if not path.is_file() or path.suffix.lower() not in SUPPORTED_SUFFIXES:
            continue
        if any(part in IGNORED_PARTS for part in path.relative_to(root).parts):
            continue
        yield path


def compile_ledger(root: Path, policy_path: Path) -> Ledger:
    policy = load_policy(root, policy_path)
    capabilities = tuple(capability_for(root, path, policy) for path in source_paths(root))
    counts: dict[str, int] = {}
    for item in capabilities:
        counts[item.status] = counts.get(item.status, 0) + 1

    canonical = json.dumps(
        [asdict(item) for item in capabilities],
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    return Ledger(
        schema="urn:dteam:capability-ledger:v1",
        root=".",
        policy=policy_path.as_posix(),
        capabilities=capabilities,
        counts=dict(sorted(counts.items())),
        digest=sha256_bytes(canonical),
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--policy", type=Path, default=Path("closure-policy.json"))
    parser.add_argument("--output", type=Path)
    parser.add_argument("--fail-on", action="append", default=["BUILD_BROKEN", "UNKNOWN"])
    args = parser.parse_args()

    root = args.root.resolve(strict=True)
    ledger = compile_ledger(root, args.policy)
    payload = json.dumps(asdict(ledger), indent=2, sort_keys=True) + "\n"

    if args.output:
        output = args.output if args.output.is_absolute() else root / args.output
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(payload, encoding="utf-8")
    else:
        print(payload, end="")

    failing = set(args.fail_on)
    return 1 if any(item.status in failing for item in ledger.capabilities) else 0


if __name__ == "__main__":
    raise SystemExit(main())
