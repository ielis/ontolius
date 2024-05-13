use std::str::FromStr;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ontolius::base::TermId;

fn bench_term_id(c: &mut Criterion) {
    // Bench parsing CURIE parts.
    let mut group = c.benchmark_group("TermId");
    group.bench_function(BenchmarkId::from_parameter("TermId::from known"), 
    |b| {
        b.iter(|| {
            black_box(TermId::from(("HP", "0001250")));
        })
    });

    group.bench_function(BenchmarkId::from_parameter("TermId::from random"), 
    |b| {
        b.iter(|| {
            black_box(TermId::from(("MP", "0001250")));
        })
    });

    // Bench parsing the entire CURIEs.
    group.bench_function(BenchmarkId::from_parameter("TermId::from_str known"), 
    |b| {
        b.iter(|| {
            black_box(TermId::from_str("HP:0001250").expect("This curie should be parsable!"));
        })
    });

    group.bench_function(BenchmarkId::from_parameter("TermId::from_str random"), 
    |b| {
        b.iter(|| {
            black_box(TermId::from_str("MP:0001250").expect("This curie should be parsable!"));
        })
    });
    group.finish();
}

criterion_group!(benches, bench_term_id);
criterion_main!(benches);
