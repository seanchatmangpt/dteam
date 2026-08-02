"""Remove obsolete generated module declarations from an explicit CLI module."""

from __future__ import annotations

import argparse
from pathlib import Path

OBSOLETE_DECLARATIONS = (
    "pub mod generated_verbs;",
    "pub mod domain_traits;",
)


def repair(path: Path) -> tuple[str, ...]:
    source = path.read_text(encoding="utf-8")
    removed = tuple(decl for decl in OBSOLETE_DECLARATIONS if decl in source)
    if not removed:
        return ()

    lines = source.splitlines(keepends=True)
    filtered = [line for line in lines if line.strip() not in OBSOLETE_DECLARATIONS]
    path.write_text("".join(filtered), encoding="utf-8")
    return removed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("path", type=Path, help="Path to clap-noun-verb/src/cli/mod.rs")
    args = parser.parse_args()

    target = args.path.resolve(strict=True)
    removed = repair(target)
    status = ", ".join(removed) if removed else "unchanged"
    print(f"{status}: {target}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
