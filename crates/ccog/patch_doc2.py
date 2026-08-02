"""Normalize an escaped rustdoc newline in an explicit Rust source file.

Kept as a compatibility entry point for historical automation. New callers
should use patch_doc.py, which also inserts the missing documentation.
"""

from __future__ import annotations

import argparse
from pathlib import Path

BROKEN = (
    "/// Verifies the signature of the RDF ontology projection graph.\\n"
    "pub fn verify_ontology_signature"
)
FIXED = (
    "/// Verifies the signature of the RDF ontology projection graph.\n"
    "pub fn verify_ontology_signature"
)


def repair(path: Path) -> bool:
    source = path.read_text(encoding="utf-8")
    if BROKEN not in source:
        return False
    path.write_text(source.replace(BROKEN, FIXED), encoding="utf-8")
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("path", type=Path, help="Path to insa-truthforge/src/lib.rs")
    args = parser.parse_args()

    target = args.path.resolve(strict=True)
    changed = repair(target)
    print(f"{'repaired' if changed else 'unchanged'}: {target}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
