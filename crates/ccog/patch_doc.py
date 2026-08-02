"""Ensure the ontology-signature verifier has its canonical rustdoc.

The target file is supplied explicitly so the repair can run against any
checkout, generated tree, or temporary verifier fixture.
"""

from __future__ import annotations

import argparse
from pathlib import Path

SIGNATURE = "pub fn verify_ontology_signature"
DOC = "/// Verifies the signature of the RDF ontology projection graph."


def repair(path: Path) -> bool:
    source = path.read_text(encoding="utf-8")
    escaped = f"{DOC}\\n{SIGNATURE}"
    canonical = f"{DOC}\n{SIGNATURE}"

    if escaped in source:
        path.write_text(source.replace(escaped, canonical), encoding="utf-8")
        return True
    if canonical in source:
        return False
    if SIGNATURE not in source:
        raise ValueError(f"signature not found in {path}: {SIGNATURE}")

    path.write_text(source.replace(SIGNATURE, canonical, 1), encoding="utf-8")
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
