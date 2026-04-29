// Chatman Equation hot-path budget: ≤500µs warm. Validate via `cargo bench -p ccog`.
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use ccog::{FieldContext, Construct8, process, process_with_hooks, HookRegistry};
use ccog::breeds::{eliza, mycin, strips};
use ccog::graph::GraphIri;
use ccog::hooks::{missing_evidence_hook, phrase_binding_hook};

const BENCH_NTRIPLES: &str = r#"
<http://example.org/claim/1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.org/Claim> .
<http://example.org/evidence/police_report> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://schema.org/DigitalDocument> .
<http://example.org/evidence/police_report> <http://purl.org/dc/terms/type> <http://example.org/police_report_concept> .
<http://example.org/police_report_concept> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/2004/02/skos/core#Concept> .
<http://example.org/police_report_concept> <http://www.w3.org/2004/02/skos/core#prefLabel> "police report" .
<http://example.org/missing_concept> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/2004/02/skos/core#Concept> .
<http://example.org/missing_concept> <http://www.w3.org/2004/02/skos/core#prefLabel> "missing" .
"#;

fn setup_field() -> FieldContext {
    let field = FieldContext::new("bench");
    field.graph.load_ntriples(BENCH_NTRIPLES).unwrap();
    field
}

fn bench_eliza_bind(c: &mut Criterion) {
    let field = setup_field();
    c.bench_function("eliza_bind_phrase", |b| {
        b.iter(|| eliza::bind_phrase(black_box("the police report is missing"), &field).unwrap())
    });
}

fn bench_mycin_evidence(c: &mut Criterion) {
    let field = setup_field();
    let bound_terms = eliza::bind_phrase("the police report is missing", &field).unwrap();
    c.bench_function("mycin_find_missing_evidence", |b| {
        b.iter(|| mycin::find_missing_evidence(black_box(&bound_terms), &field).unwrap())
    });
}

fn bench_strips_transition(c: &mut Criterion) {
    let field = setup_field();
    let iri = GraphIri::from_iri("https://schema.org/AskAction").unwrap();
    c.bench_function("strips_check_transition", |b| {
        b.iter(|| strips::check_transition(black_box(&iri), &field).unwrap())
    });
}

fn bench_construct8_from_sparql(c: &mut Criterion) {
    let field = setup_field();
    c.bench_function("construct8_from_sparql", |b| {
        b.iter(|| Construct8::from_sparql(&field.graph, black_box("CONSTRUCT { ?s a ?o } WHERE { ?s a ?o } LIMIT 8")).unwrap())
    });
}

fn bench_process(c: &mut Criterion) {
    c.bench_function("process_full", |b| {
        b.iter_batched(
            setup_field,
            |mut field| { process(black_box("the police report is missing"), &mut field).unwrap() },
            BatchSize::SmallInput,
        )
    });
}

fn bench_process_with_hooks(c: &mut Criterion) {
    c.bench_function("process_with_hooks_full", |b| {
        b.iter_batched(
            || {
                let mut reg = HookRegistry::new();
                reg.register(missing_evidence_hook());
                reg.register(phrase_binding_hook());
                (setup_field(), reg)
            },
            |(mut field, reg)| {
                process_with_hooks(black_box("the police report is missing"), &mut field, &reg).unwrap()
            },
            BatchSize::SmallInput,
        )
    });
}

fn bench_hook_fire(c: &mut Criterion) {
    let field = setup_field();
    let mut registry = HookRegistry::new();
    registry.register(missing_evidence_hook());
    registry.register(phrase_binding_hook());
    c.bench_function("hook_registry_fire_matching", |b| {
        b.iter(|| registry.fire_matching(black_box(&field)).unwrap())
    });
}

criterion_group!(benches,
    bench_eliza_bind,
    bench_mycin_evidence,
    bench_strips_transition,
    bench_construct8_from_sparql,
    bench_process,
    bench_process_with_hooks,
    bench_hook_fire);
criterion_main!(benches);
