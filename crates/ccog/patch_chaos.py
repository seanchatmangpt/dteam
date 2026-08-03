"""Generate the INSA contradictory-state rejection test at an explicit path."""

from __future__ import annotations

import argparse
from pathlib import Path

CHAOS_MONKEY = """use insa_kappa8::fuse_hearsay::{FuseHearsay, RequiredMask, ConflictMask, RuleId, AuthorityMask};
use insa_kappa8::fuse_hearsay::{Blackboard, FusionRule};
use insa_types::FieldMask;
use insa_instinct::InstinctByte;

#[test]
fn gate_chaos_monkey_fuzz_rejection() {
    // Fix 7: \"No Stubs\" blind spot — contradictory states must be rejected.
    let mut board = Blackboard::default();
    board.present = FieldMask(0);
    board.conflicted = FieldMask(0xFF);
    board.stale = FieldMask(0xFF);

    let rule = FusionRule {
        id: RuleId(1),
        required_sources: RequiredMask(FieldMask(0xFF)),
        conflict_mask: ConflictMask(FieldMask(0xFF)),
        authority_required: AuthorityMask(FieldMask(0)),
        emits_on_fail: InstinctByte::ESCALATE,
    };

    let res = FuseHearsay::fuse(&board, &rule);

    assert!(
        !res.emits.contains(InstinctByte::SETTLE),
        \"engine settled on contradictory noise\"
    );
}
"""


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("path", type=Path, help="Output path for chaos_monkey.rs")
    parser.add_argument(
        "--force",
        action="store_true",
        help="Replace an existing file instead of refusing",
    )
    args = parser.parse_args()

    target = args.path.resolve()
    if target.exists() and not args.force:
        parser.error(f"refusing to overwrite existing file: {target}")

    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(CHAOS_MONKEY, encoding="utf-8")
    print(f"generated: {target}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
