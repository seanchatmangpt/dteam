"""Repair an escaped newline in an INSA POWL64 source file.

The target is explicit: callers pass the file to mutate. This keeps the script
portable and prevents accidental writes to a developer-specific checkout.
"""

from __future__ import annotations

import argparse
from pathlib import Path

BROKEN = (
    "/// Fix 6: Byzantine Wire Assurance Cryptographic Signature\\n"
    "    pub signature: [u8; 32],"
)
FIXED = (
    "/// Fix 6: Byzantine Wire Assurance Cryptographic Signature\n"
    "    pub signature: [u8; 32],"
)


def repair(path: Path) -> bool:
    source = path.read_text(encoding="utf-8")
    if BROKEN not in source:
        return False

    path.write_text(source.replace(BROKEN, FIXED), encoding="utf-8")
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "path",
        type=Path,
        help="Path to insa-proof/src/powl64.rs",
    )
    args = parser.parse_args()

    target = args.path.resolve(strict=True)
    changed = repair(target)
    print(f"{'repaired' if changed else 'unchanged'}: {target}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
