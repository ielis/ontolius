use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ontolius::TermId;
use std::{
    hash::{DefaultHasher, Hash},
    hint::black_box,
};

fn bench_term_id(c: &mut Criterion) {
    // Bench parsing CURIE parts.
    let mut group = c.benchmark_group("TermId");
    group.throughput(criterion::Throughput::Elements(1));
    group.bench_function(BenchmarkId::from_parameter("TermId::from known"), |b| {
        b.iter(|| {
            black_box(TermId::from(("HP", "0001250")));
        })
    });

    group.bench_function(BenchmarkId::from_parameter("TermId::from random"), |b| {
        b.iter(|| {
            black_box(TermId::from(("MP", "0001250")));
        })
    });

    // Bench parsing the entire CURIEs.
    group.bench_function(BenchmarkId::from_parameter("TermId::from_str known"), |b| {
        b.iter(|| {
            black_box(
                "HP:0001250"
                    .parse::<TermId>()
                    .expect("This curie should be parsable!"),
            );
        })
    });

    group.bench_function(
        BenchmarkId::from_parameter("TermId::from_str random"),
        |b| {
            b.iter(|| {
                black_box(
                    "ABC:0001250"
                        .parse::<TermId>()
                        .expect("This curie should be parsable!"),
                );
            })
        },
    );

    group.bench_function(BenchmarkId::from_parameter("TermId::hash known"), |b| {
        let known: TermId = "HP:0001250".parse().expect("The curie is OK");
        b.iter(|| {
            let mut h = DefaultHasher::new();
            black_box(black_box(&known).hash(&mut h));
        });
    });

    group.bench_function(BenchmarkId::from_parameter("TermId::hash random"), |b| {
        let known: TermId = "ABC:0001250".parse().expect("The curie is OK");
        b.iter(|| {
            let mut h = DefaultHasher::new();
            black_box(black_box(&known).hash(&mut h));
        });
    });
    group.finish();
}

criterion_group!(benches, bench_term_id);
criterion_main!(benches);
