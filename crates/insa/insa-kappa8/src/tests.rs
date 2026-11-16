use crate::*;
use crate::reduce_gap_gps::*;
use crate::precondition_strips::engine::*;
use crate::precondition_strips::schema::*;
use crate::precondition_strips::preconditions::{RequiredMask as StripsRequiredMask, ForbiddenMask as StripsForbiddenMask};
use crate::rule_mycin::engine::*;
use crate::rule_mycin::rule::*;
use crate::rule_mycin::MycinStatus;
use crate::ground_shrdlu::engine::*;
use crate::ground_shrdlu::symbol::*;
use crate::ground_shrdlu::GroundingStatus;
use crate::reconstruct_dendral::engine::*;
use crate::reconstruct_dendral::fragment::*;
use crate::reconstruct_dendral::DendralStatus;
use crate::reflect_eliza::engine::*;
use crate::reflect_eliza::pattern::*;
use crate::reflect_eliza::ReflectStatus;
use crate::fuse_hearsay::engine::*;
use crate::fuse_hearsay::blackboard::*;
use crate::fuse_hearsay::fusion_rule::*;
use crate::fuse_hearsay::FusionStatus;

use insa_instinct::*;

extern crate alloc;
use alloc::vec;
use alloc::boxed::Box;

#[test]
fn test_reduce_gap_gps_equivalence() {
    let goal = GoalState {
        required: FieldMask(0b111),
        forbidden: FieldMask(0b1000),
        completed: CompletedMask(0),
    };
    
    // Case 1: Gap width 0 (Success)
    let engine = ReduceGapGps {
        goal,
        operators: &[],
        max_depth: PlannerBudget(5),
    };
    let ctx = ClosureCtx {
        present: FieldMask(0b111),
        completed: CompletedMask(0),
        object: ObjectRef(0),
        policy: insa_types::PolicyEpoch(1),
        dictionary: DictionaryDigest([0; 32]),
    };
    let res = engine.evaluate(&ctx);
    assert_eq!(res.status, CollapseStatus::Success);
    assert_eq!(res.instincts, InstinctByte::SETTLE);
    assert_eq!(res.detail.gps, GpsByte::GOAL_KNOWN.union(GpsByte::PROGRESS_MADE));

    // Case 2: Gap detected, operator available
    let operators = vec![
        GapOperator {
            id: OperatorId(1),
            required_preconditions: FieldMask(0),
            resolves: FieldMask(0b011),
            emits: InstinctByte::ASK,
        }
    ];
    let engine = ReduceGapGps {
        goal,
        operators: Box::leak(operators.into_boxed_slice()),
        max_depth: PlannerBudget(5),
    };
    let ctx = ClosureCtx {
        present: FieldMask(0),
        completed: CompletedMask(0),
        object: ObjectRef(0),
        policy: insa_types::PolicyEpoch(1),
        dictionary: DictionaryDigest([0; 32]),
    };
    let res = engine.evaluate(&ctx);
    assert_eq!(res.status, CollapseStatus::Partial);
    assert_eq!(res.instincts, InstinctByte::ASK);
    assert!(res.detail.gps.contains(GpsByte::OPERATOR_AVAILABLE));
}

#[test]
fn test_precondition_strips() {
    let schema = ActionSchema {
        id: ActionId(1),
        required: StripsRequiredMask(FieldMask(0b11)),
        forbidden: StripsForbiddenMask(FieldMask(0b100)),
        add_effects: FieldMask(0b1000),
        clear_effects: FieldMask(0b10000),
        completes: CompletedMask(0),
        policy_epoch: crate::precondition_strips::schema::PolicyEpoch(1),
    };

    // Satisfied
    let res = PreconditionStrips::evaluate(&schema, FieldMask(0b11), crate::precondition_strips::schema::PolicyEpoch(1));
    assert_eq!(res.detail, StripsByte::EFFECTS_KNOWN.union(StripsByte::PRECONDITIONS_SATISFIED).union(StripsByte::ACTION_ENABLED));
    assert_eq!(res.emits, InstinctByte::empty());

    // Stale epoch
    let res = PreconditionStrips::evaluate(&schema, FieldMask(0b11), crate::precondition_strips::schema::PolicyEpoch(2));
    assert_eq!(res.detail, StripsByte::ACTION_BLOCKED);
    assert_eq!(res.emits, InstinctByte::AWAIT.union(InstinctByte::ESCALATE));
}

#[test]
fn test_rule_mycin() {
    let rules = vec![
        ExpertRule {
            id: ExpertRuleId(1),
            required: FieldMask(0b1),
            forbidden: FieldMask(0b10),
            emits: InstinctByte::SETTLE,
            confidence: Confidence(90),
            epoch: crate::rule_mycin::rule::PolicyEpoch(1),
        }
    ];

    let res = RuleMycin::evaluate(&rules, FieldMask(0b1), crate::rule_mycin::rule::PolicyEpoch(1));
    assert_eq!(res.status, MycinStatus::Fired);
    assert_eq!(res.emits, InstinctByte::SETTLE);
    assert!(res.detail.contains(MycinByte::CONFIDENCE_HIGH));
}

#[test]
fn test_ground_shrdlu() {
    let rules = vec![
        GroundingRule {
            symbol: SymbolId(1),
            required_context: FieldMask(0b1),
            expected_object: ObjectRef(42),
        }
    ];

    // Symbol resolved
    let res = GroundShrdlu::evaluate(&rules, true, FieldMask(0b1));
    assert_eq!(res.status, GroundingStatus::Resolved);
    assert_eq!(res.resolved_object, Some(ObjectRef(42)));

    // Missing symbol
    let res = GroundShrdlu::evaluate(&rules, false, FieldMask(0b1));
    assert_eq!(res.status, GroundingStatus::Missing);
    assert_eq!(res.resolved_object, None);
}

#[test]
fn test_reconstruct_dendral() {
    let rules = vec![
        ReconstructionRule {
            id: FragmentId(1),
            fragments_required: FieldMask(0b11),
            constraints_forbidden: FieldMask(0b100),
            generates_hypothesis: FieldMask(0),
            emits: InstinctByte::INSPECT,
        }
    ];

    let res = ReconstructDendral::evaluate(&rules, FieldMask(0b11));
    assert_eq!(res.status, DendralStatus::Unique);
    assert_eq!(res.emits, InstinctByte::INSPECT.union(InstinctByte::SETTLE));
}

#[test]
fn test_reflect_eliza() {
    let patterns = vec![
        ReflectPattern {
            id: PatternId(1),
            required_context: FieldMask(0b1),
            template_id: TemplateId(10),
            emits: InstinctByte::ASK,
            eliza_detail: ElizaByte::ASK_CLARIFYING,
            ask_kind: AskKind::Clarify,
            _padding: [0; 3],
        }
    ];

    // Matched
    let res = ReflectEliza::evaluate(&patterns, FieldMask(0b1), FieldMask(0));
    assert_eq!(res.status, ReflectStatus::Matched);
    assert_eq!(res.selected_pattern, Some(PatternId(1)));

    // Incomplete (missing slots)
    let res = ReflectEliza::evaluate(&patterns, FieldMask(0), FieldMask(0b10));
    assert_eq!(res.status, ReflectStatus::Incomplete);
}

#[test]
fn test_fuse_hearsay() {
    let board = Blackboard {
        present: FieldMask(0b11),
        conflicted: FieldMask(0),
        stale: FieldMask(0),
        ..Default::default()
    };
    let rule = FusionRule {
        id: crate::fuse_hearsay::fusion_rule::RuleId(1),
        required_sources: crate::fuse_hearsay::fusion_rule::RequiredMask(FieldMask(0b11)),
        conflict_mask: crate::fuse_hearsay::fusion_rule::ConflictMask(FieldMask(0b100)),
        authority_required: crate::fuse_hearsay::fusion_rule::AuthorityMask(FieldMask(0)),
        emits_on_fail: InstinctByte::empty(),
    };

    let res = FuseHearsay::fuse(&board, &rule);
    assert_eq!(res.status, FusionStatus::Complete);
    assert_eq!(res.emits, InstinctByte::SETTLE);
}
