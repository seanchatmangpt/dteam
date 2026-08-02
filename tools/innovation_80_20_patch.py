#!/usr/bin/env python3
"""Apply the audited 80/20 source repairs exactly once."""

from __future__ import annotations

from pathlib import Path

TELCO_TARGET = Path("capabilities/dteam-kernel/src/combinatorial.rs")
INNOVATION_TARGET = Path("capabilities/dteam-kernel/src/innovation.rs")

TELCO_OLD = """fn maximum_domain_disjoint(paths: &[TelcoPath]) -> usize {
    let mut selected: Vec<&TelcoPath> = Vec::new();
    'candidate: for path in paths {
        let internal = path.failure_domains.iter().cloned().collect::<BTreeSet<_>>();
        for existing in &selected {
            let existing_domains = existing.failure_domains.iter().cloned().collect::<BTreeSet<_>>();
            if !internal.is_disjoint(&existing_domains) { continue 'candidate; }
        }
        selected.push(path);
    }
    selected.len()
}
"""

TELCO_NEW = """fn maximum_domain_disjoint(paths: &[TelcoPath]) -> usize {
    let mut selected = Vec::<BTreeSet<String>>::new();
    'candidate: for path in paths {
        let internal = path
            .nodes
            .iter()
            .skip(1)
            .take(path.nodes.len().saturating_sub(2))
            .cloned()
            .collect::<BTreeSet<_>>();
        for existing in &selected {
            if !internal.is_disjoint(existing) {
                continue 'candidate;
            }
        }
        selected.push(internal);
    }
    selected.len()
}
"""

INNOVATION_REPAIRS = (
    (
        "let selected = selected_80_20.iter().collect::<BTreeSet<_>>();",
        "let selected = selected_80_20\n            .iter()\n            .map(String::as_str)\n            .collect::<BTreeSet<_>>();",
    ),
    (
        "selected.contains(&finding.id)",
        "selected.contains(finding.id.as_str())",
    ),
    (
        'assert_eq!(diff.regressed(), &["runtime-tracer-bullet"]);',
        'assert_eq!(diff.regressed(), &["runtime-tracer-bullet".to_owned()]);',
    ),
)


def replace_once(path: Path, old: str, new: str, label: str) -> bool:
    text = path.read_text(encoding="utf-8")
    if old in text:
        path.write_text(text.replace(old, new, 1), encoding="utf-8")
        print(f"patched {label}")
        return True
    if new in text:
        print(f"{label} already patched")
        return False
    raise SystemExit(f"refused: expected subject not found for {label}")


def main() -> int:
    replace_once(TELCO_TARGET, TELCO_OLD, TELCO_NEW, "telco transit disjointness")
    for index, (old, new) in enumerate(INNOVATION_REPAIRS, start=1):
        replace_once(INNOVATION_TARGET, old, new, f"innovation compile guard {index}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
