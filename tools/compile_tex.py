#!/usr/bin/env python3
"""Compile a TeX document deterministically from an explicit source path.

This replaces workstation-bound document scripts with one reusable capability.
The command never invents a checkout location and propagates compiler failure.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
from dataclasses import asdict, dataclass
from pathlib import Path


@dataclass(frozen=True)
class CompileReceipt:
    source: str
    output: str
    source_sha256: str
    output_sha256: str
    engine: str
    passes: int


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def compile_tex(source: Path, output_dir: Path, engine: str, passes: int) -> CompileReceipt:
    source = source.resolve(strict=True)
    if source.suffix.lower() != ".tex":
        raise ValueError(f"expected a .tex source: {source}")
    if passes < 1:
        raise ValueError("passes must be at least 1")

    executable = shutil.which(engine)
    if executable is None:
        raise FileNotFoundError(f"TeX engine not found: {engine}")

    output_dir = output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    command = [
        executable,
        "-interaction=nonstopmode",
        "-halt-on-error",
        f"-output-directory={output_dir}",
        source.name,
    ]
    for _ in range(passes):
        subprocess.run(command, cwd=source.parent, check=True)

    output = output_dir / f"{source.stem}.pdf"
    if not output.is_file():
        raise RuntimeError(f"compiler succeeded without producing {output}")

    return CompileReceipt(
        source=str(source),
        output=str(output),
        source_sha256=sha256(source),
        output_sha256=sha256(output),
        engine=engine,
        passes=passes,
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--engine", default="pdflatex")
    parser.add_argument("--passes", type=int, default=2)
    parser.add_argument("--receipt", type=Path)
    args = parser.parse_args()

    receipt = compile_tex(args.source, args.output_dir, args.engine, args.passes)
    encoded = json.dumps(asdict(receipt), indent=2, sort_keys=True) + "\n"
    if args.receipt:
        target = args.receipt.resolve()
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(encoded, encoding="utf-8")
    print(encoded, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
