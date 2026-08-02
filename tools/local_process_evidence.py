#!/usr/bin/env python3
"""Convert a local polyglot crown receipt into OCEL-style process evidence."""
from __future__ import annotations
import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path

ORDER = ("python", "node", "go", "java", "kotlin", "ruby", "php", "swift", "gcc", "clang", "cpp", "bash")


def digest(value: object) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("receipt", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    source = json.loads(args.receipt.read_text(encoding="utf-8"))
    events: list[dict[str, object]] = []
    sequence = 0

    def add(activity: str, objects: list[str], attributes: dict[str, object]) -> None:
        nonlocal sequence
        sequence += 1
        events.append({"id": f"e{sequence:03d}", "activity": activity, "time": sequence, "objects": objects, "attributes": attributes})

    run_id = source["receipt_sha256"][:16]
    add("run.started", [f"run:{run_id}"], {"implementations": source["implementation_count"]})
    for language in ORDER:
        item = source["outputs"][language]
        add("implementation.executed", [f"run:{run_id}", f"language:{language}"], {"language": language, "sha256": item["sha256"], "bytes": item["bytes"]})
        add("implementation.admitted", [f"run:{run_id}", f"language:{language}", f"semantics:{source['semantic_sha256']}"], {"equivalent": item["sha256"] == source["semantic_sha256"]})
    add("run.crowned", [f"run:{run_id}", f"semantics:{source['semantic_sha256']}"], {"standing": source["standing"], "byte_equivalent": source["byte_equivalent"]})

    traces = {language: ["implementation.executed", "implementation.admitted"] for language in ORDER}
    directly_follows: Counter[tuple[str, str]] = Counter()
    for trace in traces.values():
        directly_follows.update(zip(trace, trace[1:]))

    violations: list[str] = []
    if source["standing"] != "ALIVE":
        violations.append("source_not_alive")
    if not source["byte_equivalent"]:
        violations.append("semantic_divergence")
    if source["implementation_count"] != len(ORDER):
        violations.append("implementation_count")
    missing = sorted(set(ORDER) - set(source["outputs"]))
    if missing:
        violations.append("missing:" + ",".join(missing))

    report = {
        "schema": "urn:dteam:local-process-evidence:v1",
        "standing": "ALIVE" if not violations else "BUILD_BROKEN",
        "source_receipt_sha256": source["receipt_sha256"],
        "semantic_sha256": source["semantic_sha256"],
        "object_types": {"run": 1, "language": len(ORDER), "semantics": 1},
        "event_count": len(events),
        "traces": traces,
        "directly_follows": [{"from": left, "to": right, "count": count} for (left, right), count in sorted(directly_follows.items())],
        "fitness": 1.0 if not violations else 0.0,
        "violations": violations,
        "events": events,
    }
    report["evidence_sha256"] = digest(report)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({key: report[key] for key in ("standing", "event_count", "fitness", "evidence_sha256")}, sort_keys=True))
    return 0 if report["standing"] == "ALIVE" else 1


if __name__ == "__main__":
    raise SystemExit(main())
