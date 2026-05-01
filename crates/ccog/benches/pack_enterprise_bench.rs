//! Enterprise pack micro-bench (Phase 12). Tier: FullProcess (PROV-rich act fns).

use ccog::compiled::CompiledFieldSnapshot;
use ccog::field::FieldContext;
use ccog::multimodal::{ContextBundle, PostureBundle};
use ccog::packs::enterprise::BUILTINS;
use ccog::packs::TierMasks;
use ccog::runtime::ClosedFieldContext;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_enterprise(c: &mut Criterion) {
    let f = FieldContext::new("bench");
    let snap = CompiledFieldSnapshot::from_field(&f).expect("snap");
    let context = ClosedFieldContext {
        snapshot: &snap,
        posture: PostureBundle::default(),
        context: ContextBundle::default(),
        tiers: TierMasks::ZERO,
        human_burden: 0,
    };
    c.bench_function("pack_enterprise_act_first_slot", |b| {
        b.iter(|| {
            let delta = (BUILTINS[0].act)(black_box(&context)).expect("act");
            black_box(delta)
        })
    });
}

criterion_group!(benches, bench_enterprise);
criterion_main!(benches);
