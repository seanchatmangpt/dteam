//! Classical AI heuristics mapped to HDIT AutoML signals.
//! 
//! This module cross-pollinates the new classical AI implementations
//! (from `unibit-ai-classic` such as MYCIN, ELIZA, STRIPS) into the 
//! `dteam` AutoML loop. It exposes these classical expert systems as 
//! highly optimized, T0/T1 branchless signal generators for the 
//! `hdit_automl` pipeline.

use crate::ml::hdit_automl::SignalProfile;
use unibit_kernel::{UCell, UReceipt};

/// Generate an AutoML signal profile based on the MYCIN backward-chaining
/// expert system logic.
///
/// Traces are evaluated through `mycin::exec::infer` to determine if
/// the inference cycle successfully derives a target conclusion (e.g., STREP).
pub fn mycin_automl_signal(
    name: &str,
    trace_facts: &[u64], // Simulated facts for each trace
    anchor: &[bool],
) -> SignalProfile {
    use unibit_ai_classic::mycin::exec::{infer, org};

    let mut predictions = Vec::with_capacity(trace_facts.len());
    let mut total_timing_ns = 0u64;

    for &facts in trace_facts {
        // We simulate running the T0 kernel loop and measure logical timing
        let initial = UCell(facts);
        let (final_state, fired_rules, _) = infer(initial, UReceipt::default());
        let is_strep = (final_state.0 & org::STREP) != 0;
        predictions.push(is_strep);
        total_timing_ns += 20 + (fired_rules as u64 * 5);
    }

    let timing_us = (total_timing_ns / 1000).max(1);
    SignalProfile::new(name, predictions, anchor, timing_us)
}

/// Generate an AutoML signal profile based on the GPS/STRIPS planning logic.
///
/// Traces are evaluated to determine if a specific goal (e.g., picking up block A)
/// is achievable from the given initial state.
pub fn strips_automl_signal(
    name: &str,
    trace_states: &[u64],
    anchor: &[bool],
) -> SignalProfile {
    use unibit_ai_classic::strips::exec::{pickup_a, HOLDING_A};

    let mut predictions = Vec::with_capacity(trace_states.len());
    let mut total_timing_ns = 0u64;

    for &state in trace_states {
        let initial = UCell(state);
        // Try to execute PICKUP(A)
        let success = match pickup_a(initial, UReceipt::default()) {
            Some((final_state, _)) => (final_state.0 & HOLDING_A) != 0,
            None => false,
        };
        predictions.push(success);
        total_timing_ns += 15; // static cost for branchless ops
    }

    let timing_us = (total_timing_ns / 1000).max(1);
    SignalProfile::new(name, predictions, anchor, timing_us)
}

/// Generate an AutoML signal profile based on the ELIZA pattern matching logic.
///
/// Traces are evaluated to determine if a specific template (e.g., DREAM)
/// is triggered by the given input keywords.
pub fn eliza_automl_signal(
    name: &str,
    trace_keywords: &[u64],
    anchor: &[bool],
) -> SignalProfile {
    use unibit_ai_classic::eliza::exec::{eliza_turn, DOCTOR, tmpl};

    let mut predictions = Vec::with_capacity(trace_keywords.len());
    let mut total_timing_ns = 0u64;

    for &keywords in trace_keywords {
        let result = eliza_turn(keywords, &DOCTOR, UReceipt::default());
        predictions.push(result.matched && result.template_index == tmpl::DREAM);
        total_timing_ns += 10;
    }

    let timing_us = (total_timing_ns / 1000).max(1);
    SignalProfile::new(name, predictions, anchor, timing_us)
}

#[cfg(test)]
mod tests {
    use super::*;
    use unibit_ai_classic::mycin::exec::fact;
    use unibit_ai_classic::strips::exec::{CLEAR_A, ON_TABLE_A, ARM_EMPTY, HOLDING_A, CLEAR_B};
    use unibit_ai_classic::eliza::exec::{keyword_bit, kw};
    use crate::ml::hdit_automl::{run_hdit_automl, Tier};

    #[test]
    fn test_cross_pollinate_automl_with_all_classic_systems() {
        let anchor = vec![true, false, true, false];

        // MYCIN traces
        let patients = vec![
            fact::GRAM_POS | fact::AEROBIC | fact::FEVER | fact::RIGORS, // true
            fact::GRAM_NEG | fact::ANAEROBIC, // false
            fact::GRAM_POS | fact::AEROBIC | fact::FEVER | fact::RIGORS, // true
            fact::GRAM_POS | fact::AEROBIC | fact::BURN, // false
        ];
        let mycin_sig = mycin_automl_signal("mycin_strep", &patients, &anchor);

        // STRIPS traces
        let blocks = vec![
            CLEAR_A | ON_TABLE_A | ARM_EMPTY, // pickup_a succeeds -> true
            HOLDING_A | CLEAR_B, // pickup_a fails -> false
            CLEAR_A | ON_TABLE_A | ARM_EMPTY, // pickup_a succeeds -> true
            ON_TABLE_A | ARM_EMPTY, // pickup_a fails (not clear) -> false
        ];
        let strips_sig = strips_automl_signal("strips_pickup", &blocks, &anchor);

        // ELIZA traces
        let chats = vec![
            keyword_bit(kw::DREAM), // matches DREAM template -> true
            keyword_bit(kw::MOTHER), // matches FAMILY template -> false
            keyword_bit(kw::DREAM), // matches DREAM template -> true
            keyword_bit(kw::SORRY), // matches NOAPOL template -> false
        ];
        let eliza_sig = eliza_automl_signal("eliza_dream", &chats, &anchor);

        let candidates = vec![mycin_sig, strips_sig, eliza_sig];

        // The HDIT AutoML loop evaluates all classical AI heuristics and selects
        // an optimal combination (or just one if it's perfectly correlated).
        let plan = run_hdit_automl(candidates, &anchor, 2);

        // All of them correlate perfectly with the anchor, so it will pick the first one
        // and reject the others due to high correlation.
        assert!(!plan.selected.is_empty());
        let tier = plan.tiers.iter().find(|(name, _)| name == &plan.selected[0]).unwrap().1;
        assert_eq!(tier, Tier::T0, "Classical AI systems must be T0 tier");
        assert_eq!(plan.plan_accuracy, 1.0);
    }
}
