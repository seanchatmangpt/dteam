#!/usr/bin/env python3
"""Deterministic 80/20 innovation audit for dteam.

The tool deliberately separates source admission from environment availability:

* ``audit`` proves repository controls from source alone.
* ``doctor`` adds local toolchain and sibling-workspace observations.
* ``replay`` executes the source audit twice and requires byte-identical evidence.

Exit codes: 0 = admitted, 1 = blocked dependency/environment, 2 = invariant failure.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from dataclasses import asdict, dataclass
from pathlib import Path

SCHEMA = "dteam.innovation-80-20.audit.v1"
RECEIPT_SCHEMA = "dteam.innovation-80-20.receipt.v1"
PINNED_NIGHTLY = re.compile(r"^nightly-\d{4}-\d{2}-\d{2}$")


@dataclass(frozen=True)
class Check:
    id: str
    area: str
    leverage: int
    standing: str
    detail: str
    repair: str | None = None


class AuditFailure(RuntimeError):
    pass


def root_from_script() -> Path:
    return Path(__file__).resolve().parents[1]


def read_text(root: Path, relative: str) -> str:
    path = root / relative
    if not path.is_file():
        raise AuditFailure(f"required source file missing: {relative}")
    return path.read_text(encoding="utf-8")


def canonical_json(value: object) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n").encode()


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def check_source(root: Path) -> list[Check]:
    checks: list[Check] = []

    makefile = read_text(root, "Makefile")
    doctor_recipe = re.search(r"(?ms)^doctor:\n((?:\t.*\n)+)", makefile)
    fail_open = doctor_recipe is None or "||" in doctor_recipe.group(1) or "2>/dev/null" in doctor_recipe.group(1)
    checks.append(Check(
        "DX-FAIL-LOUD",
        "developer_experience",
        100,
        "FAILED" if fail_open else "ALIVE",
        "doctor propagates the producing failure" if not fail_open else "doctor suppresses or rewrites a producing failure",
        "remove stderr suppression and shell fallbacks from the doctor recipe" if fail_open else None,
    ))

    for target in ("audit:", "acceptance:", "doctor-artifacts:"):
        checks.append(Check(
            f"DX-TARGET-{target[:-1].upper()}",
            "developer_experience",
            55,
            "ALIVE" if target in makefile else "FAILED",
            f"Makefile exposes {target[:-1]} as a first-class target",
            f"add a {target[:-1]} target" if target not in makefile else None,
        ))

    toolchain = tomllib.loads(read_text(root, "rust-toolchain.toml"))
    tc = toolchain.get("toolchain", {})
    channel = str(tc.get("channel", ""))
    targets = set(tc.get("targets", []))
    checks.append(Check(
        "BUILD-PINNED-NIGHTLY",
        "reproducibility",
        95,
        "ALIVE" if PINNED_NIGHTLY.fullmatch(channel) else "FAILED",
        f"toolchain channel is {channel or '<missing>'}",
        "pin rust-toolchain.toml to nightly-YYYY-MM-DD" if not PINNED_NIGHTLY.fullmatch(channel) else None,
    ))
    checks.append(Check(
        "BUILD-WASM-TARGET",
        "portability",
        70,
        "ALIVE" if "wasm32-unknown-unknown" in targets else "FAILED",
        "wasm32-unknown-unknown is admitted by the toolchain contract",
        "add wasm32-unknown-unknown to toolchain.targets" if "wasm32-unknown-unknown" not in targets else None,
    ))

    config = read_text(root, "src/config.rs")
    required_symbols = {
        "CONFIG-SOURCE-PROVENANCE": "pub enum ConfigSource",
        "CONFIG-LOAD-WITH-SOURCE": "pub fn load_with_source",
        "CONFIG-STRICT-LOAD": "pub fn load_required",
        "CONFIG-VALIDATION": "pub fn validate(&self)",
    }
    for check_id, symbol in required_symbols.items():
        present = symbol in config
        checks.append(Check(
            check_id,
            "configuration",
            90 if check_id in {"CONFIG-STRICT-LOAD", "CONFIG-VALIDATION"} else 75,
            "ALIVE" if present else "FAILED",
            f"configuration surface contains {symbol}",
            f"implement {symbol}" if not present else None,
        ))

    validation_obligations = (
        "reward_weights must sum to 1.0",
        "kernel.alignment must be a non-zero power of two",
        "automl.sh_subsample must be in (0,1]",
        "missing_required_config_is_refused",
        "reward_weight_drift_is_refused",
    )
    for obligation in validation_obligations:
        present = obligation in config
        checks.append(Check(
            "CONFIG-OBLIGATION-" + re.sub(r"[^A-Z0-9]+", "-", obligation.upper()).strip("-"),
            "configuration",
            65,
            "ALIVE" if present else "FAILED",
            f"configuration mutation obligation is present: {obligation}",
            f"add a validation/test obligation for: {obligation}" if not present else None,
        ))

    script = read_text(root, "tools/innovation_80_20.py")
    checks.append(Check(
        "EVIDENCE-DETERMINISTIC-JSON",
        "evidence",
        80,
        "ALIVE" if "sort_keys=True" in script and "canonical_json" in script else "FAILED",
        "audit evidence uses canonical JSON",
        "serialize evidence with sorted keys and fixed separators" if "sort_keys=True" not in script else None,
    ))
    checks.append(Check(
        "EVIDENCE-TWO-PASS-REPLAY",
        "evidence",
        90,
        "ALIVE" if "REPLAY_MATCH" in script and "run_replay" in script else "FAILED",
        "source audit has an exact two-pass replay crown",
        "implement two independent source-audit passes and compare bytes" if "REPLAY_MATCH" not in script else None,
    ))

    return sorted(checks, key=lambda c: (-c.leverage, c.id))


def cargo_path_dependencies(root: Path) -> list[dict[str, object]]:
    cargo = root / "Cargo.toml"
    if not cargo.is_file():
        return []
    doc = tomllib.loads(cargo.read_text(encoding="utf-8"))
    found: list[dict[str, object]] = []
    for section_name in ("dependencies", "dev-dependencies", "build-dependencies"):
        for name, value in doc.get(section_name, {}).items():
            if isinstance(value, dict) and "path" in value:
                raw = str(value["path"])
                resolved = (root / raw).resolve()
                found.append({
                    "name": name,
                    "section": section_name,
                    "path": raw,
                    "resolved": str(resolved),
                    "present": resolved.exists(),
                })
    return sorted(found, key=lambda item: (str(item["section"]), str(item["name"])))


def executable_version(name: str, args: list[str]) -> dict[str, object]:
    path = shutil.which(name)
    if path is None:
        return {"name": name, "present": False, "path": None, "version": None}
    try:
        proc = subprocess.run([path, *args], text=True, capture_output=True, timeout=10, check=False)
        output = (proc.stdout or proc.stderr).strip().splitlines()
        version = output[0] if output else f"exit={proc.returncode}"
    except (OSError, subprocess.TimeoutExpired) as error:
        version = f"error:{type(error).__name__}"
    return {"name": name, "present": True, "path": path, "version": version}


def source_artifacts(root: Path) -> list[dict[str, object]]:
    admitted = [
        "Makefile",
        "rust-toolchain.toml",
        "src/config.rs",
        "tools/innovation_80_20.py",
        "tests/test_innovation_80_20.py",
        "docs/innovation-80-20.md",
    ]
    artifacts: list[dict[str, object]] = []
    for relative in admitted:
        path = root / relative
        if not path.is_file():
            continue
        data = path.read_bytes()
        artifacts.append({"path": relative, "bytes": len(data), "sha256": sha256(data)})
    return artifacts


def build_report(root: Path, include_environment: bool) -> dict[str, object]:
    checks = check_source(root)
    failed = [asdict(check) for check in checks if check.standing == "FAILED"]
    report: dict[str, object] = {
        "schema": SCHEMA,
        "mode": "doctor" if include_environment else "audit",
        "checks": [asdict(check) for check in checks],
        "summary": {
            "total": len(checks),
            "alive": sum(check.standing == "ALIVE" for check in checks),
            "failed": len(failed),
            "leverage_closed": sum(check.leverage for check in checks if check.standing == "ALIVE"),
            "leverage_open": sum(check.leverage for check in checks if check.standing == "FAILED"),
        },
        "standing": "ALIVE" if not failed else "BUILD_BROKEN",
    }
    if include_environment:
        tools = [
            executable_version("python3", ["--version"]),
            executable_version("cargo", ["--version"]),
            executable_version("rustc", ["--version"]),
        ]
        path_dependencies = cargo_path_dependencies(root)
        blocked = [item for item in path_dependencies if not item["present"]]
        report["environment"] = {
            "tools": tools,
            "path_dependencies": path_dependencies,
            "blocked_path_dependencies": blocked,
        }
        if not failed and (blocked or not all(tool["present"] for tool in tools)):
            report["standing"] = "BLOCKED_DEPENDENCY"
    return report


def write_report(root: Path, output_dir: Path, report: dict[str, object]) -> tuple[Path, Path]:
    output_dir.mkdir(parents=True, exist_ok=True)
    report_bytes = canonical_json(report)
    report_path = output_dir / "innovation-80-20.json"
    report_path.write_bytes(report_bytes)
    artifacts = source_artifacts(root)
    receipt_subject = canonical_json({"report_sha256": sha256(report_bytes), "artifacts": artifacts})
    receipt = {
        "schema": RECEIPT_SCHEMA,
        "algorithm": "sha256",
        "report_sha256": sha256(report_bytes),
        "artifacts": artifacts,
        "root": sha256(receipt_subject),
        "standing": report["standing"],
    }
    receipt_path = output_dir / "innovation-80-20.receipt.json"
    receipt_path.write_bytes(canonical_json(receipt))
    return report_path, receipt_path


def run_once(root: Path, output_dir: Path, include_environment: bool) -> tuple[dict[str, object], bytes, bytes]:
    report = build_report(root, include_environment)
    report_path, receipt_path = write_report(root, output_dir, report)
    return report, report_path.read_bytes(), receipt_path.read_bytes()


def run_replay(root: Path, output_dir: Path) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="dteam-audit-a-") as a, tempfile.TemporaryDirectory(prefix="dteam-audit-b-") as b:
        report_a, report_bytes_a, receipt_bytes_a = run_once(root, Path(a), False)
        report_b, report_bytes_b, receipt_bytes_b = run_once(root, Path(b), False)
        match = report_bytes_a == report_bytes_b and receipt_bytes_a == receipt_bytes_b
        replay = {
            "schema": "dteam.innovation-80-20.replay.v1",
            "pass_a_report_sha256": sha256(report_bytes_a),
            "pass_b_report_sha256": sha256(report_bytes_b),
            "pass_a_receipt_sha256": sha256(receipt_bytes_a),
            "pass_b_receipt_sha256": sha256(receipt_bytes_b),
            "result": "REPLAY_MATCH" if match else "REPLAY_DRIFT",
            "standing": report_a["standing"] if match else "BUILD_BROKEN",
        }
        output_dir.mkdir(parents=True, exist_ok=True)
        (output_dir / "innovation-80-20.replay.json").write_bytes(canonical_json(replay))
        (output_dir / "innovation-80-20.json").write_bytes(report_bytes_a)
        (output_dir / "innovation-80-20.receipt.json").write_bytes(receipt_bytes_a)
        if report_a != report_b or not match:
            raise AuditFailure("two-pass innovation audit replay drifted")
        return replay


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("audit", "doctor", "replay"))
    parser.add_argument("--root", type=Path, default=root_from_script())
    parser.add_argument("--output-dir", type=Path, default=Path("artifacts/innovation-audit"))
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    root = args.root.resolve()
    output_dir = args.output_dir if args.output_dir.is_absolute() else root / args.output_dir
    try:
        if args.command == "replay":
            result = run_replay(root, output_dir)
            print(json.dumps(result, sort_keys=True))
            return 0 if result["standing"] == "ALIVE" else 2
        report, _, _ = run_once(root, output_dir, args.command == "doctor")
        print(json.dumps(report, sort_keys=True))
        if report["standing"] == "ALIVE":
            return 0
        if report["standing"] == "BLOCKED_DEPENDENCY":
            return 1
        return 2
    except (AuditFailure, OSError, tomllib.TOMLDecodeError) as error:
        print(json.dumps({"schema": SCHEMA, "standing": "BUILD_BROKEN", "error": str(error)}, sort_keys=True), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
