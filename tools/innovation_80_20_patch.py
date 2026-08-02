#!/usr/bin/env python3
"""Apply the audited endpoint-tolerant telco redundancy repair exactly once."""

from __future__ import annotations

from pathlib import Path

TARGET = Path("capabilities/dteam-kernel/src/combinatorial.rs")

OLD = """fn maximum_domain_disjoint(paths: &[TelcoPath]) -> usize {
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

NEW = """fn maximum_domain_disjoint(paths: &[TelcoPath]) -> usize {
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


def main() -> int:
    text = TARGET.read_text(encoding="utf-8")
    if OLD in text:
        TARGET.write_text(text.replace(OLD, NEW, 1), encoding="utf-8")
        print("patched telco transit disjointness")
        return 0
    if NEW in text:
        print("telco transit disjointness already patched")
        return 0
    raise SystemExit("refused: expected telco disjointness subject not found")


if __name__ == "__main__":
    raise SystemExit(main())
