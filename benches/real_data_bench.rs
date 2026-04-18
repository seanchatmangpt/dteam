use criterion::{black_box, criterion_group, criterion_main, Criterion};
use process_mining::core::event_data::case_centric::xes::{import_xes, XESImportOptions};
use process_mining::core::event_data::case_centric::AttributeValue;
use wasm4pm::reinforcement::{
    Agent, DoubleQLearning, ExpectedSARSAAgent, QLearning, ReinforceAgent,
    SARSAAgent,
};
use wasm4pm::{RlAction, RlState};
use std::path::Path;
use std::fs::File;
use std::io::BufReader;

fn create_state(h: i32) -> RlState {
    RlState {
        health_level: h,
        event_rate_q: 0,
        activity_count_q: 0,
        spc_alert_level: 0,
        drift_status: 0,
        rework_ratio_q: 0,
        circuit_state: 0,
        cycle_phase: 0,
        marking_vec: Vec::new(),
        recent_activities: Vec::new(),
    }
}

fn map_activity_to_action(activity: &str) -> RlAction {
    let lower = activity.to_lowercase();
    if lower.contains("rejected") {
        RlAction::Rework
    } else if lower.contains("approved") || lower.contains("payment") || lower.contains("handled") {
        RlAction::Optimize
    } else {
        RlAction::Idle
    }
}

fn load_real_actions() -> Vec<RlAction> {
    let path = Path::new("data/DomesticDeclarations.xes");
    if !path.exists() {
        eprintln!("Data file not found at data/DomesticDeclarations.xes. Using mock data.");
        return vec![RlAction::Idle; 1000];
    }

    let file = File::open(path).expect("Failed to open XES file");
    let reader = BufReader::new(file);
    let log = import_xes(reader, XESImportOptions::default()).expect("Failed to import XES");
    let mut actions = Vec::new();

    for trace in log.traces {
        for event in trace.events {
            // Find concept:name attribute
            let name_attr = event.attributes.iter().find(|a| a.key == "concept:name");
            if let Some(attr) = name_attr {
                match &attr.value {
                    AttributeValue::String(s) => {
                        actions.push(map_activity_to_action(s));
                    }
                    _ => {}
                }
            }
        }
    }
    actions
}

fn bench_real_data_processing(c: &mut Criterion) {
    let actions = load_real_actions();
    println!("Loaded {} actions from real data", actions.len());

    let q = QLearning::<RlState, RlAction>::new();
    let sarsa = SARSAAgent::<RlState, RlAction>::new();

    let mut group = c.benchmark_group("RealDataProcessing");
    
    let chunk_size = 1000.min(actions.len());
    let actions_chunk = &actions[..chunk_size];

    group.bench_function("QLearning Real Data (1000 steps)", |b| b.iter(|| {
        let mut state = create_state(0);
        for action in actions_chunk {
            let next_h = match action {
                RlAction::Idle => state.health_level,
                RlAction::Optimize => state.health_level + 1,
                RlAction::Rework => (state.health_level - 1).max(0),
            };
            let next_state = create_state(next_h);
            let done = next_h >= 100;
            let reward = if done { 1.0 } else { 0.0 };
            q.update(black_box(&state), black_box(action), reward, black_box(&next_state), done);
            state = next_state;
            if done { state = create_state(0); }
        }
    }));

    group.bench_function("SARSA Real Data (1000 steps)", |b| b.iter(|| {
        sarsa.reset();
        let mut state = create_state(0);
        for action in actions_chunk {
            let next_h = match action {
                RlAction::Idle => state.health_level,
                RlAction::Optimize => state.health_level + 1,
                RlAction::Rework => (state.health_level - 1).max(0),
            };
            let next_state = create_state(next_h);
            let done = next_h >= 100;
            let reward = if done { 1.0 } else { 0.0 };
            sarsa.update(black_box(&state), black_box(action), reward, black_box(&next_state), done);
            state = next_state;
            if done { state = create_state(0); sarsa.reset(); }
        }
    }));

    group.finish();
}

criterion_group!(benches, bench_real_data_processing);
criterion_main!(benches);
