use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wasm4pm::models::petri_net::{PetriNet, Place, Transition, Arc};
use wasm4pm::conformance::token_replay;
use wasm4pm::io::xes::XESReader;
use std::path::Path;

fn create_test_net() -> PetriNet {
    let mut net = PetriNet::default();
    net.places.push(Place { id: "p1".to_string() });
    net.places.push(Place { id: "p2".to_string() });
    net.transitions.push(Transition { id: "t1".to_string(), label: "A".to_string(), is_invisible: Some(false) });
    net.arcs.push(Arc { from: "p1".to_string(), to: "t1".to_string(), weight: Some(1) });
    net.arcs.push(Arc { from: "t1".to_string(), to: "p2".to_string(), weight: Some(1) });
    net.initial_marking.insert("p1".to_string(), 1);
    net
}

fn bench_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("Algorithms");
    
    // 1. XES Ingestion Benchmark
    let xes_path = Path::new("data/DomesticDeclarations.xes");
    if xes_path.exists() {
        group.bench_function("XES Ingestion (DomesticDeclarations)", |b| b.iter(|| {
            let reader = XESReader::new();
            let _log = reader.read(black_box(xes_path)).unwrap();
        }));
    }

    // 2. Structural Soundness Benchmark
    let net = create_test_net();
    group.bench_function("Structural Soundness Check", |b| b.iter(|| {
        black_box(&net).is_structural_workflow_net()
    }));

    // 3. Token-Based Replay Benchmark
    if xes_path.exists() {
        let reader = XESReader::new();
        let log = reader.read(xes_path).unwrap();
        group.bench_function("Token-Based Replay (1000 traces)", |b| b.iter(|| {
            token_replay(black_box(&log), black_box(&net))
        }));
    }

    group.finish();
}

criterion_group!(benches, bench_algorithms);
criterion_main!(benches);
