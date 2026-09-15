use std::{collections::HashSet, fs::File, hint::black_box, io::BufReader};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use flate2::bufread::GzDecoder;
use ontolius::{
    io::OntologyLoaderBuilder,
    ontology::{csr::MinimalCsrOntology, TaxonomyTraversal},
    TermId,
};
use roaring::RoaringBitmap;

const HPO_PATH: &str = "resources/hp.v2024-08-13.json.gz";

fn load_hpo(hpo_path: &str) -> MinimalCsrOntology {
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();
    let reader = GzDecoder::new(BufReader::new(
        File::open(hpo_path).expect("HPO file should exist"),
    ));
    loader
        .load_from_read(reader)
        .expect("HPO should be parsable")
}

fn roaring(c: &mut Criterion) {
    let hpo = load_hpo(HPO_PATH);

    let tid: TermId = "HP:0001166".parse().unwrap();
    let arach_bm = make_bitmap(&tid, &hpo);
    let arach_set = make_hashset(&hpo, &tid);

    let tid: TermId = "HP:0001250".parse().unwrap();
    let seizure_bm = make_bitmap(&tid, &hpo);
    let seizure_set = make_hashset(&hpo, &tid);

    let mut group = c.benchmark_group("Intersection");
    group.throughput(criterion::Throughput::Elements(1));

    group.bench_function(BenchmarkId::from_parameter("RoaringBitmap"), |bencher| {
        bencher.iter(|| {
            (&arach_bm & &seizure_bm).iter().for_each(|t| {
                black_box(t);
            });
        });
    });
    group.bench_function(BenchmarkId::from_parameter("HashSet"), |bencher| {
        bencher.iter(|| {
            (&arach_set & &seizure_set).iter().for_each(|t| {
                black_box(t);
            });
        });
    });
    group.finish();

    let mut group = c.benchmark_group("Union");
    group.throughput(criterion::Throughput::Elements(1));

    group.bench_function(BenchmarkId::from_parameter("RoaringBitmap"), |bencher| {
        bencher.iter(|| {
            (&arach_bm | &seizure_bm).iter().for_each(|t| {
                black_box(t);
            });
        });
    });
    group.bench_function(BenchmarkId::from_parameter("HashSet"), |bencher| {
        bencher.iter(|| {
            (&arach_set | &seizure_set).iter().for_each(|t| {
                black_box(t);
            });
        });
    });
    group.finish();

    let mut group = c.benchmark_group("Iteration");
    group.throughput(criterion::Throughput::Elements(1));

    group.bench_function(
        BenchmarkId::from_parameter("RoaringBitmap/Arachnodactyly"),
        |bencher| {
            bencher.iter(|| {
                black_box(&arach_bm).iter().for_each(|t| {
                    black_box(t);
                });
            });
        },
    );
    group.bench_function(
        BenchmarkId::from_parameter("RoaringBitmap/Seizure"),
        |bencher| {
            bencher.iter(|| {
                black_box(&seizure_bm).iter().for_each(|t| {
                    black_box(t);
                });
            });
        },
    );
    group.bench_function(
        BenchmarkId::from_parameter("HashSet/Arachnodactyly"),
        |bencher| {
            bencher.iter(|| {
                black_box(&arach_set).iter().for_each(|t| {
                    black_box(t);
                });
            });
        },
    );
    group.bench_function(BenchmarkId::from_parameter("HashSet/Seizure"), |bencher| {
        bencher.iter(|| {
            black_box(&seizure_set).iter().for_each(|t| {
                black_box(t);
            });
        });
    });
    group.finish();
}

fn make_bitmap<T>(tid: &TermId, hpo: T) -> RoaringBitmap
where
    T: TaxonomyTraversal<Idx = u32>,
{
    let idx = hpo.term_index(tid).unwrap();
    RoaringBitmap::from_iter(std::iter::chain(
        std::iter::once(idx),
        hpo.iter_ancestor_idxs(idx),
    ))
}

fn make_hashset<T>(hpo: T, tid: &TermId) -> HashSet<u32>
where
    T: TaxonomyTraversal<Idx = u32>,
{
    let mut set = HashSet::new();
    let idx = hpo.term_index(tid).clone().unwrap();
    set.insert(idx);
    set.extend(hpo.iter_ancestor_idxs(idx));
    set.shrink_to_fit();
    set
}

criterion_group!(benches, roaring);
criterion_main!(benches);
