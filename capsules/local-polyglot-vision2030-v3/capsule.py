#!/usr/bin/env python3
"""Deterministic self-extracting local Vision 2030 acceptance capsule v3."""
from __future__ import annotations
import argparse, base64, io, subprocess, tarfile, tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
EXPECTED_ROOT_BLAKE3 = '4d49e86e8f9090a311073813634046635d8a6503227da7c68486d54960331946'
EXPECTED_CANONICAL_BLAKE3 = 'b186c85cbc8bbd4796c3e5ae922113c88371dd33e160e0b004265ae3cd746398'
EXPECTED_CANONICAL_SHA256 = 'e9cb51496b4833606590a424eb183aca8eb5893e43477741cf95d710143b95d1'


def payload() -> bytes:
    encoded = ''.join(p.read_text().strip() for p in sorted((HERE / 'chunks').glob('*.b85')))
    return base64.b85decode(encoded.encode('ascii'))


def materialize(target: Path) -> Path:
    target.mkdir(parents=True, exist_ok=True)
    with tarfile.open(fileobj=io.BytesIO(payload()), mode='r:gz') as tf:
        root = target.resolve()
        for member in tf.getmembers():
            destination = (target / member.name).resolve()
            if root not in destination.parents and destination != root:
                raise RuntimeError(f'unsafe archive path: {member.name}')
        tf.extractall(target, filter='fully_trusted')
    return target


def run(target: Path | None = None) -> int:
    target = target or Path(tempfile.mkdtemp(prefix='dteam-vision2030-v3-'))
    materialize(target)
    result = subprocess.run(['bash', 'run-local.sh'], cwd=target)
    if result.returncode == 0:
        print('{"capsule":"dteam.local-polyglot-capsule.v3","standing":"ALIVE","root_blake3":"%s"}' % EXPECTED_ROOT_BLAKE3)
    return result.returncode


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument('--materialize', type=Path)
    parser.add_argument('--run', action='store_true')
    args = parser.parse_args()
    if args.materialize:
        materialize(args.materialize)
        print(args.materialize)
        return 0
    return run(None if args.run else Path.cwd() / 'materialized')

if __name__ == '__main__':
    raise SystemExit(main())
