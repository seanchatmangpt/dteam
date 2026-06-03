import sys

chaos_monkey = """use insa_kappa8::fuse_hearsay::{FuseHearsay, RequiredMask, ConflictMask, RuleId, AuthorityMask};
use insa_kappa8::fuse_hearsay::{Blackboard, FusionRule};
use insa_types::FieldMask;
use insa_instinct::InstinctByte;

#[test]
fn gate_chaos_monkey_fuzz_rejection() {
    // Fix 7: "No Stubs" Blindspot - Ensure empty/contradictory states are violently rejected
    let mut board = Blackboard::default();
    board.present = FieldMask(0);
    board.conflicted = FieldMask(0xFF); // Pure contradiction
    board.stale = FieldMask(0xFF);  // Completely stale
    
    let rule = FusionRule {
        id: RuleId(1),
        required_sources: RequiredMask(FieldMask(0xFF)),
        conflict_mask: ConflictMask(FieldMask(0xFF)),
        authority_required: AuthorityMask(FieldMask(0)),
        emits_on_fail: InstinctByte::ESCALATE,
    };
    
    let res = FuseHearsay::fuse(&board, &rule);
    
    // The engine must NOT settle. It must escalate or ask.
    assert!(!res.emits.contains(InstinctByte::SETTLE), "Chaos monkey test failed: Engine settled on contradictory noise!");
}
"""
with open("/Users/sac/insa/insa-truthforge/tests/chaos_monkey.rs", "w") as f:
    f.write(chaos_monkey)

