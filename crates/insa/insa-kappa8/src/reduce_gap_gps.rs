use crate::{ClosureCtx, Cog8Support, CollapseEngine, CollapseResult, CollapseStatus};
use insa_instinct::{GpsByte, InstinctByte, KappaByte, KappaDetail16};
use insa_types::{CompletedMask, FieldMask};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperatorId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoalState {
    pub required: FieldMask,
    pub forbidden: FieldMask,
    pub completed: CompletedMask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gap {
    pub missing_required: FieldMask,
    pub present_forbidden: FieldMask,
    pub width: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GapOperator {
    pub id: OperatorId,
    pub required_preconditions: FieldMask,
    pub resolves: FieldMask,
    pub emits: InstinctByte,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlannerBudget(pub u8);

pub struct ReduceGapGps {
    pub goal: GoalState,
    pub operators: &'static [GapOperator],
    pub max_depth: PlannerBudget,
}

impl ReduceGapGps {
    pub fn compute_gap(current: FieldMask, goal: &GoalState) -> Gap {
        let missing = (current.0 & goal.required.0) ^ goal.required.0;
        let forbidden = current.0 & goal.forbidden.0;
        Gap {
            missing_required: FieldMask(missing),
            present_forbidden: FieldMask(forbidden),
            width: missing.count_ones() as u8 + forbidden.count_ones() as u8,
        }
    }
}

impl CollapseEngine for ReduceGapGps {
    fn evaluate(&self, ctx: &ClosureCtx) -> CollapseResult {
        let gap = Self::compute_gap(ctx.present, &self.goal);
        let mut detail = KappaDetail16::empty();
        detail.kappa = KappaByte::REDUCE_GAP;

        let is_zero = gap.width == 0;

        // Path A: gap.width == 0 (Success)
        let gps_a = GpsByte::empty().union(GpsByte::GOAL_KNOWN).union(GpsByte::PROGRESS_MADE);
        let instincts_a = InstinctByte::SETTLE;

        // Path B: gap.width > 0
        let is_small = gap.width < 4;
        let gap_size_bit = [GpsByte::GAP_LARGE, GpsByte::GAP_SMALL][is_small as usize];
        let mut gps_b = GpsByte::empty()
            .union(GpsByte::GOAL_KNOWN)
            .union(GpsByte::GAP_DETECTED)
            .union(gap_size_bit);

        let mut combined_instincts = InstinctByte::empty();
        let mut best_reduction = 0;

        for op in self.operators {
            let resolves_missing = op.resolves.0 & gap.missing_required.0;
            let resolves_forbidden = op.resolves.0 & gap.present_forbidden.0;
            let reduction = resolves_missing.count_ones() + resolves_forbidden.count_ones();

            let is_better = (reduction > 0) && (reduction >= best_reduction);
            let select_mask_reduction = (is_better as u32).wrapping_neg();
            best_reduction = (reduction & select_mask_reduction) | (best_reduction & !select_mask_reduction);

            let select_mask_instinct = (is_better as u8).wrapping_neg();
            combined_instincts = InstinctByte((op.emits.bits() & select_mask_instinct) | (combined_instincts.bits() & !select_mask_instinct));
        }

        let has_operator = best_reduction > 0;
        gps_b = gps_b.union([
            GpsByte::empty().union(GpsByte::OPERATOR_BLOCKED).union(GpsByte::NO_PROGRESS),
            GpsByte::empty().union(GpsByte::OPERATOR_AVAILABLE)
        ][has_operator as usize]);
        let status_b = [CollapseStatus::Failed, CollapseStatus::Partial][has_operator as usize];
        let combined_instincts_b = InstinctByte(combined_instincts.bits() | ([InstinctByte::ESCALATE.bits(), 0][has_operator as usize]));

        // Select between Path A and Path B
        let final_gps = GpsByte( (gps_a.bits() & (is_zero as u8).wrapping_neg()) | (gps_b.bits() & (!is_zero as u8).wrapping_neg()) );
        let instincts = InstinctByte( (instincts_a.bits() & (is_zero as u8).wrapping_neg()) | (combined_instincts_b.bits() & (!is_zero as u8).wrapping_neg()) );
        let status = [status_b, CollapseStatus::Success][is_zero as usize];

        detail.gps = final_gps;

        CollapseResult {
            detail,
            instincts,
            support: Cog8Support::new(ctx.present),
            status,
        }
    }
}
