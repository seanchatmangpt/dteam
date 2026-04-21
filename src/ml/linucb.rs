//! Hyper-optimized LinUCB for autonomic drift adaptation.
//! ZERO HEAP ALLOCATIONS in select_action and update.
//! Optimized for auto-vectorization via slice primitives and constant sizing.

use crate::reinforcement::{Agent, AgentMeta, WorkflowAction, WorkflowState};

/// Branchless f32 selection based on a mask.
#[inline(always)]
fn select_f32(cond: bool, true_val: f32, false_val: f32) -> f32 {
    let mask = (cond as u32).wrapping_neg();
    let t_bits = true_val.to_bits();
    let f_bits = false_val.to_bits();
    f32::from_bits((mask & t_bits) | (!mask & f_bits))
}

/// Branchless usize selection based on a mask.
#[inline(always)]
fn select_usize(cond: bool, true_val: usize, false_val: usize) -> usize {
    let mask = (cond as usize).wrapping_neg();
    (mask & true_val) | (!mask & false_val)
}

/// Disjoint LinUCB Agent with separate models per arm.
/// D: Feature dimension
/// D2: D * D
/// ARMS: Number of arms
#[derive(Clone, Copy, Debug)]
pub struct LinUcb<const D: usize, const D2: usize, const ARMS: usize = 3> {
    pub alpha: f32,
    pub a_inv: [[f32; D2]; ARMS], // Stack-allocated fixed-size matrices
    pub b: [[f32; D]; ARMS],      // Stack-allocated fixed-size vectors
}

impl<const D: usize, const D2: usize, const ARMS: usize> LinUcb<D, D2, ARMS> {
    pub fn new(alpha: f32) -> Self {
        assert_eq!(D * D, D2, "D2 must be D * D");
        let mut a_inv = [[0.0; D2]; ARMS];
        for arm in 0..ARMS {
            for i in 0..D {
                a_inv[arm][i * D + i] = 1.0; // Identity initialization
            }
        }

        Self {
            alpha,
            a_inv,
            b: [[0.0; D]; ARMS],
        }
    }

    /// Selects an action based on context feature vector and upper confidence bound.
    /// Uses a pure branchless comparison for the best arm.
    #[inline(always)]
    pub fn select_action_raw(&self, context: &[f32; D], arms: usize) -> usize {
        let mut max_ucb = f32::NEG_INFINITY;
        let mut best_arm = 0;
        
        let actual_arms = arms.min(ARMS);

        for arm in 0..actual_arms {
            let mut theta_dot_x = 0.0;
            let mut x_a_inv_x = 0.0;
            
            let arm_a_inv = &self.a_inv[arm];
            let arm_b = &self.b[arm];

            // Matrix-vector multiplications optimized for auto-vectorization
            for i in 0..D {
                let offset = i * D;
                let row = &arm_a_inv[offset..offset + D];

                // theta = A_inv * b
                // theta_dot_x = context^T * (A_inv * b)
                let mut a_inv_row_dot_b = 0.0;
                for j in 0..D {
                    a_inv_row_dot_b += row[j] * arm_b[j];
                }
                theta_dot_x += a_inv_row_dot_b * context[i];

                // Variance = x^T * A_inv * x
                let mut row_dot_x = 0.0;
                for j in 0..D {
                    row_dot_x += row[j] * context[j];
                }
                x_a_inv_x += context[i] * row_dot_x;
            }

            let ucb = theta_dot_x + self.alpha * x_a_inv_x.max(0.0).sqrt();

            // Branchless best arm selection (AC 2)
            let is_better = ucb > max_ucb;
            best_arm = select_usize(is_better, arm, best_arm);
            max_ucb = select_f32(is_better, ucb, max_ucb);
        }
        best_arm
    }

    /// Sherman-Morrison rank-1 update for A_inv of a specific arm.
    /// Maintains Zero Heap Allocation by performing all math on the stack.
    #[inline(always)]
    pub fn update_arm(&mut self, arm: usize, context: &[f32; D], reward: f32) {
        if arm >= ARMS {
            return;
        }

        let arm_a_inv = &mut self.a_inv[arm];
        let arm_b = &mut self.b[arm];

        // b = b + r * x
        for i in 0..D {
            arm_b[i] += reward * context[i];
        }

        // a_inv_x = A_inv * x
        let mut a_inv_x = [0.0; D];
        for i in 0..D {
            let offset = i * D;
            let row = &arm_a_inv[offset..offset + D];
            for j in 0..D {
                a_inv_x[i] += row[j] * context[j];
            }
        }

        // Denominator = 1 + x^T * A_inv * x
        let mut x_a_inv_x = 0.0;
        for i in 0..D {
            x_a_inv_x += context[i] * a_inv_x[i];
        }
        let inv_denom = 1.0 / (1.0 + x_a_inv_x);

        // A_inv = A_inv - (a_inv_x * a_inv_x^T) / Denom
        // Add a bounding factor to prevent matrix collapse over 1M+ iterations (AC 5)
        let min_eigen = 1e-5;
        for i in 0..D {
            let offset = i * D;
            for j in 0..D {
                let update = (a_inv_x[i] * a_inv_x[j]) * inv_denom;
                let mut new_val = arm_a_inv[offset + j] - update;
                if i == j {
                    new_val = new_val.max(min_eigen); // Protect diagonal to maintain positive semi-definiteness
                }
                arm_a_inv[offset + j] = new_val.clamp(-1e4, 1e4); // Hard bound
            }
        }

        for b_val in arm_b.iter_mut() {
            *b_val = b_val.clamp(-1e5, 1e5);
        }
    }
}

impl<const D: usize, const D2: usize, const ARMS: usize, S, A> Agent<S, A> for LinUcb<D, D2, ARMS>
where
    S: WorkflowState,
    A: WorkflowAction,
{
    #[inline(always)]
    fn select_action(&self, state: S) -> A {
        let mut context = [0.0; D];
        state.write_features(&mut context);
        let arm_idx = self.select_action_raw(&context, A::ACTION_COUNT);
        A::from_index(arm_idx).unwrap()
    }

    #[inline(always)]
    fn update(&mut self, state: S, action: A, reward: f32, _next_state: S, _done: bool) {
        let mut context = [0.0; D];
        state.write_features(&mut context);
        self.update_arm(action.to_index(), &context, reward);
    }

    fn reset(&mut self) {
        let mut a_inv = [[0.0; D2]; ARMS];
        for arm in 0..ARMS {
            for i in 0..D {
                a_inv[arm][i * D + i] = 1.0;
            }
        }
        self.a_inv = a_inv;
        self.b = [[0.0; D]; ARMS];
    }
}

impl<const D: usize, const D2: usize, const ARMS: usize> AgentMeta for LinUcb<D, D2, ARMS> {
    fn name(&self) -> &'static str {
        "LinUCB"
    }

    fn exploration_rate(&self) -> f32 {
        self.alpha
    }

    fn decay_exploration(&mut self) {}
}
