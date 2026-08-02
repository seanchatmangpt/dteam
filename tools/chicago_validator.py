#!/usr/bin/env python3
"""Autonomous Chicago-style TDD validator for the dteam capability kernel."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable

PUBLIC_MODULE = re.compile(r"^pub mod ([a-zA-Z_][a-zA-Z0-9_]*);$", re.MULTILINE)
PROFILES = ("developer", "edge", "telco", "enterprise")


@dataclass(frozen=True)
class Scenario:
    id: str
    modules: tuple[str, ...]
    command: tuple[str, ...]
    expected_exit: int = 0
    required_text: tuple[str, ...] = ()
    forbidden_text: tuple[str, ...] = ("mock", "unimplemented", "todo!")
    timeout_seconds: int = 240


@dataclass(frozen=True)
class ScenarioResult:
    id: str
    modules: tuple[str, ...]
    command: tuple[str, ...]
    exit_code: int
    expected_exit: int
    elapsed_ms: int
    stdout_sha256: str
    stderr_sha256: str
    passed: bool
    failures: tuple[str, ...]


@dataclass(frozen=True)
class ValidationReceipt:
    schema: str
    repository: str | None
    sha: str | None
    manifest: str
    public_modules: tuple[str, ...]
    covered_modules: tuple[str, ...]
    uncovered_modules: tuple[str, ...]
    profiles: tuple[str, ...]
    scenario_count: int
    passed: int
    failed: int
    standing: str
    results: tuple[ScenarioResult, ...]
    digest: str


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def cargo(manifest: Path, *args: str) -> tuple[str, ...]:
    return ("cargo", *args, "--manifest-path", str(manifest))


def scenarios(manifest: Path) -> tuple[Scenario, ...]:
    all_modules = (
        "access", "broker", "combinatorial", "decision", "event_bus", "graph", "hash",
        "hook", "ledger", "model", "phase_change", "policy", "process", "provenance",
        "quota", "runtime", "saga", "scheduler", "schema", "state_machine", "store",
    )
    base = [
        Scenario("compile-all-targets", all_modules, cargo(manifest, "check", "--all-targets")),
        Scenario("access-state", ("access",), cargo(manifest, "test", "access::tests", "--", "--test-threads=1")),
        Scenario("identity-state", ("hash", "model"), cargo(manifest, "test", "hash::tests", "--", "--test-threads=1")),
        Scenario("schema-state", ("schema",), cargo(manifest, "test", "schema::tests", "--", "--test-threads=1")),
        Scenario("transaction-state", ("store",), cargo(manifest, "test", "store::tests", "--", "--test-threads=1")),
        Scenario("receipt-state", ("ledger",), cargo(manifest, "test", "ledger::tests", "--", "--test-threads=1")),
        Scenario("admission-state", ("policy",), cargo(manifest, "test", "policy::tests", "--", "--test-threads=1")),
        Scenario("decision-state", ("decision",), cargo(manifest, "test", "decision::tests", "--", "--test-threads=1")),
        Scenario("planning-state", ("graph",), cargo(manifest, "test", "graph::tests", "--", "--test-threads=1")),
        Scenario("broker-state", ("broker",), cargo(manifest, "test", "broker::tests", "--", "--test-threads=1")),
        Scenario("event-bus-state", ("event_bus",), cargo(manifest, "test", "event_bus::tests", "--", "--test-threads=1")),
        Scenario("hook-state", ("hook",), cargo(manifest, "test", "hook::tests", "--", "--test-threads=1")),
        Scenario("process-state", ("process",), cargo(manifest, "test", "process::tests", "--", "--test-threads=1")),
        Scenario("provenance-state", ("provenance",), cargo(manifest, "test", "provenance::tests", "--", "--test-threads=1")),
        Scenario("quota-state", ("quota",), cargo(manifest, "test", "quota::tests", "--", "--test-threads=1")),
        Scenario("saga-state", ("saga",), cargo(manifest, "test", "saga::tests", "--", "--test-threads=1")),
        Scenario("scheduler-state", ("scheduler",), cargo(manifest, "test", "scheduler::tests", "--", "--test-threads=1")),
        Scenario("machine-state", ("state_machine",), cargo(manifest, "test", "state_machine::tests", "--", "--test-threads=1")),
        Scenario(
            "runtime-system",
            ("runtime", "broker", "ledger", "policy", "graph"),
            cargo(manifest, "run", "--quiet", "--bin", "dteam-capabilities"),
            required_text=("standing", "completion_receipt"),
        ),
        Scenario(
            "dense-system",
            ("access", "event_bus", "saga", "runtime", "quota", "store"),
            cargo(manifest, "run", "--quiet", "--bin", "dteam-dense-demo"),
            required_text=("standing",),
        ),
        Scenario(
            "doctor-state",
            ("phase_change",),
            cargo(manifest, "run", "--quiet", "--bin", "dteam-doctor", "--", "--json"),
            required_text=("standing", "score", "digest"),
        ),
        Scenario("combinatorial-state", ("combinatorial",), cargo(manifest, "test", "combinatorial::tests", "--", "--test-threads=1")),
        Scenario(
            "telco-state",
            ("combinatorial",),
            cargo(manifest, "run", "--quiet", "--bin", "dteam-doctor", "--", "telco"),
            required_text=("standing=", "compliant_paths=", "digest="),
        ),
        Scenario(
            "crown-negative-control",
            ("phase_change",),
            cargo(manifest, "run", "--quiet", "--bin", "dteam-doctor", "--", "crown"),
            expected_exit=3,
            required_text=("VISION_2030",),
        ),
    ]
    for profile in PROFILES:
        base.extend(
            [
                Scenario(
                    f"wizard-{profile}",
                    ("combinatorial", "phase_change"),
                    cargo(manifest, "run", "--quiet", "--bin", "dteam-doctor", "--", "wizard", profile),
                    required_text=(f"wizard={profile}", "proof:"),
                ),
                Scenario(
                    f"compose-{profile}",
                    ("combinatorial", "phase_change"),
                    cargo(manifest, "run", "--quiet", "--bin", "dteam-doctor", "--", "compose", profile),
                    required_text=(f"profile={profile}", "lawful=", "pareto=", "digest="),
                ),
            ]
        )
    return tuple(base)


def public_modules(lib_rs: Path) -> tuple[str, ...]:
    return tuple(sorted(set(PUBLIC_MODULE.findall(lib_rs.read_text(encoding="utf-8")))))


def validate_structure(modules: tuple[str, ...], suite: Iterable[Scenario]) -> tuple[str, ...]:
    covered = {module for scenario in suite for module in scenario.modules}
    return tuple(sorted(set(modules) - covered))


def run_scenario(scenario: Scenario, cwd: Path) -> ScenarioResult:
    started = time.monotonic()
    try:
        completed = subprocess.run(
            scenario.command,
            cwd=cwd,
            text=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=scenario.timeout_seconds,
            check=False,
            env={**os.environ, "CARGO_INCREMENTAL": "0", "RUSTFLAGS": "-C debuginfo=0"},
        )
        exit_code = completed.returncode
        stdout = completed.stdout
        stderr = completed.stderr
    except subprocess.TimeoutExpired as error:
        exit_code = 124
        stdout = error.stdout or b""
        stderr = error.stderr or b""
    elapsed_ms = round((time.monotonic() - started) * 1000)
    combined = (stdout + b"\n" + stderr).decode("utf-8", errors="replace").lower()
    failures: list[str] = []
    if exit_code != scenario.expected_exit:
        failures.append(f"exit:{exit_code}!={scenario.expected_exit}")
    for text in scenario.required_text:
        if text.lower() not in combined:
            failures.append(f"missing:{text}")
    for text in scenario.forbidden_text:
        if text.lower() in combined:
            failures.append(f"forbidden:{text}")
    return ScenarioResult(
        id=scenario.id,
        modules=scenario.modules,
        command=scenario.command,
        exit_code=exit_code,
        expected_exit=scenario.expected_exit,
        elapsed_ms=elapsed_ms,
        stdout_sha256=sha256(stdout),
        stderr_sha256=sha256(stderr),
        passed=not failures,
        failures=tuple(failures),
    )


def canonical_digest(payload: dict) -> str:
    return sha256(json.dumps(payload, sort_keys=True, separators=(",", ":")).encode())


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--manifest", type=Path, default=Path("capabilities/dteam-kernel/Cargo.toml"))
    parser.add_argument("--output", type=Path, default=Path("artifacts/chicago/receipt.json"))
    parser.add_argument("--suite", action="append", help="run only matching scenario id; repeatable")
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--fail-fast", action="store_true")
    args = parser.parse_args()

    root = args.root.resolve(strict=True)
    manifest = (root / args.manifest).resolve(strict=True) if not args.manifest.is_absolute() else args.manifest.resolve(strict=True)
    lib_rs = manifest.parent / "src/lib.rs"
    modules = public_modules(lib_rs)
    all_scenarios = scenarios(manifest)
    selected = tuple(
        scenario for scenario in all_scenarios
        if not args.suite or any(pattern in scenario.id for pattern in args.suite)
    )

    if args.list:
        for scenario in selected:
            print(f"{scenario.id}\t{','.join(scenario.modules)}\t{' '.join(scenario.command)}")
        return 0

    uncovered = validate_structure(modules, all_scenarios)
    results: list[ScenarioResult] = []
    if not uncovered:
        for scenario in selected:
            result = run_scenario(scenario, root)
            results.append(result)
            marker = "PASS" if result.passed else "FAIL"
            print(f"{marker} {result.id} exit={result.exit_code} elapsed_ms={result.elapsed_ms}", flush=True)
            if args.fail_fast and not result.passed:
                break

    passed = sum(result.passed for result in results)
    failed = len(results) - passed
    standing = "ALIVE" if not uncovered and failed == 0 and len(results) == len(selected) else "BUILD_BROKEN"
    covered = tuple(sorted({module for scenario in all_scenarios for module in scenario.modules}))
    raw = {
        "schema": "urn:dteam:chicago-combinatorial-receipt:v1",
        "repository": os.environ.get("GITHUB_REPOSITORY"),
        "sha": os.environ.get("GITHUB_SHA"),
        "manifest": str(manifest.relative_to(root)),
        "public_modules": modules,
        "covered_modules": covered,
        "uncovered_modules": uncovered,
        "profiles": PROFILES,
        "scenario_count": len(selected),
        "passed": passed,
        "failed": failed,
        "standing": standing,
        "results": [asdict(result) for result in results],
    }
    receipt = ValidationReceipt(
        **{key: value for key, value in raw.items() if key != "results"},
        results=tuple(results),
        digest=canonical_digest(raw),
    )
    output = args.output if args.output.is_absolute() else root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(asdict(receipt), indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"standing={standing} scenarios={len(results)}/{len(selected)} modules={len(covered)}/{len(modules)} digest={receipt.digest}")
    return 0 if standing == "ALIVE" else 1


if __name__ == "__main__":
    raise SystemExit(main())
