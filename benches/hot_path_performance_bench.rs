use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dteam::utils::static_pkt::StaticPackedKeyTable;
use dteam::simd::SwarMarking;
use dteam::reinforcement::{Agent, SARSAAgent, WorkflowAction};
use dteam::utils::dense_kernel::KBitSet;
use dteam::{RlState, RlAction};
use dteam::io::xes::XESReader;

fn bench_static_pkt_lookup(c: &mut Criterion) {
    let mut pkt = StaticPackedKeyTable::<u32, u32, 64>::new();
    for i in 0..64 {
        let _ = pkt.insert(i as u64, i, i);
    }

    c.bench_function("StaticPkt_Lookup_N64", |b| {
        b.iter(|| {
            let val = pkt.get(black_box(32));
            black_box(val);
        })
    });
}

fn bench_swar_marking_fire(c: &mut Criterion) {
    let mut group = c.benchmark_group("SwarMarking_Fire");

    // K=64 (WORDS=1)
    let marking = SwarMarking::<1>::new(0b111);
    let req = [0b010];
    let out = [0b100];
    group.bench_function("Fire_K64", |b| {
        b.iter(|| {
            let (m, f) = marking.try_fire_branchless(black_box(&req), black_box(&out));
            black_box((m, f));
        })
    });

    // K=256 (WORDS=4)
    let marking = SwarMarking::<4>::new(0b111);
    let req = [0b010, 0, 0, 0];
    let out = [0b100, 0, 0, 0];
    group.bench_function("Fire_K256", |b| {
        b.iter(|| {
            let (m, f) = marking.try_fire_branchless(black_box(&req), black_box(&out));
            black_box((m, f));
        })
    });

    // K=512 (WORDS=8)
    let marking = SwarMarking::<8>::new(0b111);
    let req = [0b010, 0, 0, 0, 0, 0, 0, 0];
    let out = [0b100, 0, 0, 0, 0, 0, 0, 0];
    group.bench_function("Fire_K512", |b| {
        b.iter(|| {
            let (m, f) = marking.try_fire_branchless(black_box(&req), black_box(&out));
            black_box((m, f));
        })
    });

    group.finish();
}

fn bench_rl_hot_path(c: &mut Criterion) {
    let mut agent = SARSAAgent::<RlState<1>, RlAction>::new();
    let state = RlState {
        health_level: 1,
        event_rate_q: 0,
        activity_count_q: 0,
        spc_alert_level: 0,
        drift_status: 0,
        rework_ratio_q: 0,
        circuit_state: 0,
        cycle_phase: 0,
        marking_mask: KBitSet::zero(),
        activities_hash: 0,
        ontology_mask: KBitSet::zero(),
        universe: None,
    };

    c.bench_function("SARSA_SelectAction", |b| {
        b.iter(|| {
            let action = agent.select_action(black_box(state));
            black_box(action);
        })
    });

    let action = RlAction::from_index(0).unwrap();
    c.bench_function("SARSA_Update", |b| {
        b.iter(|| {
            agent.update(black_box(state), black_box(action), 1.0, black_box(state), false);
        })
    });
}

fn bench_xes_parsing(c: &mut Criterion) {
    let reader = XESReader::new();
    let xes_data = r#"<?xml version="1.0" encoding="UTF-8" ?>
<log xes.version="1.0" xes.features="arbitrary-depth" openxes.version="1.0RC7">
	<extension name="Concept" prefix="concept" uri="http://www.xes-standard.org/concept.xesext"/>
	<trace>
		<string key="concept:name" value="case_1"/>
		<event>
			<string key="concept:name" value="Activity A"/>
			<date key="time:timestamp" value="2023-01-01T00:00:00Z"/>
		</event>
		<event>
			<string key="concept:name" value="Activity B"/>
			<date key="time:timestamp" value="2023-01-01T00:01:00Z"/>
		</event>
	</trace>
</log>"#;

    c.bench_function("XESReader_ParseStr_Small", |b| {
        b.iter(|| {
            let log = reader.parse_str(black_box(xes_data));
            let _ = black_box(log);
        })
    });
}

criterion_group!(benches, bench_static_pkt_lookup, bench_swar_marking_fire, bench_rl_hot_path, bench_xes_parsing);
criterion_main!(benches);
