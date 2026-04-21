use dteam::reinforcement::{Agent, QLearning, StaticQValues};
use dteam::{RlAction, RlState};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let _profiler = dhat::Profiler::new_heap();

    println!("Starting Zero-Allocation RL Benchmark with StaticQValues and Reserved Capacity...");

    // 1. RL Hot Path with StaticQValues (Zero-Heap)
    // We pre-allocate the PackedKeyTable with enough capacity to avoid re-allocations of the entry vector.
    let agent = QLearning::<RlState<1>, RlAction, StaticQValues<3>>::with_capacity(1000);
    
    let state_template = RlState::<1> {
        health_level: 0,
        event_rate_q: 0,
        activity_count_q: 0,
        spc_alert_level: 0,
        drift_status: 0,
        rework_ratio_q: 0,
        circuit_state: 0,
        cycle_phase: 0,
        marking_mask: dteam::utils::dense_kernel::KBitSet::zero(),
        activities_hash: 1,
    };

    println!("Executing 1,000 RL updates with 1,000 DIFFERENT states...");
    for i in 0..1000 {
        let mut state = state_template;
        state.activities_hash = i as u64; // Different state each time
        let action = agent.select_action(state);
        agent.update(state, action, 1.0, state, false);
    }

    println!("Benchmark Complete. DHAT will now report allocations.");
}
