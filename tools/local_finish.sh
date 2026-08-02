#!/usr/bin/env bash
set -euo pipefail
repo_root="${1:-$(pwd)}"
out="${2:-$repo_root/artifacts/local-finish}"
rm -rf "$out"
mkdir -p "$out"
bash "$repo_root/tools/local_polyglot_crown.sh" "$out/polyglot"
python3 "$repo_root/tools/local_process_evidence.py" "$out/polyglot/receipt.json" --output "$out/process-evidence.json"
python3 - "$out" <<'PY'
import hashlib
import json
import sys
from pathlib import Path
root = Path(sys.argv[1])
polyglot = json.loads((root / "polyglot/receipt.json").read_text(encoding="utf-8"))
process = json.loads((root / "process-evidence.json").read_text(encoding="utf-8"))
violations = []
if polyglot["standing"] != "ALIVE":
    violations.append("polyglot_not_alive")
if process["standing"] != "ALIVE":
    violations.append("process_not_alive")
if process["source_receipt_sha256"] != polyglot["receipt_sha256"]:
    violations.append("receipt_identity_mismatch")
if process["semantic_sha256"] != polyglot["semantic_sha256"]:
    violations.append("semantic_identity_mismatch")
receipt = {
    "schema": "urn:dteam:local-finish-crown:v1",
    "standing": "ALIVE" if not violations else "BUILD_BROKEN",
    "semantic_sha256": polyglot["semantic_sha256"],
    "polyglot_receipt_sha256": polyglot["receipt_sha256"],
    "process_evidence_sha256": process["evidence_sha256"],
    "implementation_count": polyglot["implementation_count"],
    "event_count": process["event_count"],
    "fitness": process["fitness"],
    "violations": violations,
}
raw = json.dumps(receipt, sort_keys=True, separators=(",", ":")).encode()
receipt["crown_sha256"] = hashlib.sha256(raw).hexdigest()
(root / "ALIVE.json").write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(json.dumps(receipt, sort_keys=True))
raise SystemExit(0 if receipt["standing"] == "ALIVE" else 1)
PY
echo "ALIVE local-finish crown=$out/ALIVE.json"
