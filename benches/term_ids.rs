use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ontolius::{
    common::{go::CELLULAR_COMPONENT, hpo::PHENOTYPIC_ABNORMALITY, maxo::MEDICAL_ACTION},
    TermId,
};

fn bench_term_id_creation(c: &mut Criterion) {
    // Bench parsing CURIE parts.
    let mut group = c.benchmark_group("TermId::");
    group.bench_function(BenchmarkId::from_parameter("from known"), |b| {
        b.iter(|| {
            std::hint::black_box(TermId::from(("HP", "0001250")));
        })
    });

    group.bench_function(BenchmarkId::from_parameter("from random"), |b| {
        b.iter(|| {
            std::hint::black_box(TermId::from(("MP", "0001250")));
        })
    });

    // Bench parsing the entire CURIEs.
    group.bench_function(BenchmarkId::from_parameter("from_str known"), |b| {
        b.iter(|| {
            std::hint::black_box(
                "HP:0001250"
                    .parse::<TermId>()
                    .expect("This curie should be parsable!"),
            );
        })
    });

    group.bench_function(BenchmarkId::from_parameter("from_str random"), |b| {
        b.iter(|| {
            std::hint::black_box(
                "MP:0001250"
                    .parse::<TermId>()
                    .expect("This curie should be parsable!"),
            );
        })
    });
    group.finish();
}
criterion_group!(creation, bench_term_id_creation);

fn bench_term_id_util(c: &mut Criterion) {
    use std::hash::{DefaultHasher, Hash, Hasher};
    // Hash
    let terms = [
        PHENOTYPIC_ABNORMALITY.clone(),
        MEDICAL_ACTION.clone(),
        CELLULAR_COMPONENT.clone(),
        TermId::from(("RANDOM", "RAN-DOMC*HAR#ACTERS")),
    ];
    let mut group = c.benchmark_group("TermId::hash");
    for term_id in terms.clone() {
        group.bench_with_input(BenchmarkId::from_parameter(&term_id), &term_id, |b, tid| {
            b.iter(|| {
                let mut hasher = DefaultHasher::new();
                tid.hash(&mut hasher);
                std::hint::black_box(hasher.finish());
            });
        });
    }
    group.finish();

    // Eq
    let mut group = c.benchmark_group("TermId::eq");
    let seizure: TermId = TermId::from(("HP", "0001250"));
    let random = TermId::from(("RANDOM", "TERMwithNOinterest"));
    for term_id in terms.clone() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}-known", &term_id)),
            &term_id,
            |b, tid| b.iter(|| std::hint::black_box(tid == &seizure)),
        );
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}-random", &term_id)),
            &term_id,
            |b, tid| b.iter(|| std::hint::black_box(tid == &random)),
        );
    }
    group.finish();
}

criterion_group!(util, bench_term_id_util);
criterion_main!(creation, util);
