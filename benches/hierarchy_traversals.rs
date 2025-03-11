use std::fs::File;
use std::io::BufReader;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use flate2::bufread::GzDecoder;
use ontolius::io::OntologyLoaderBuilder;
use ontolius::ontology::csr::MinimalCsrOntology;
use ontolius::ontology::{HierarchyTraversals, HierarchyWalks};
use ontolius::TermId;

const HPO_PATH: &str = "resources/hp.v2024-08-13.json.gz";

const PAYLOAD: [(&str, &str); 5] = [
    ("Phenotypic abnormality", "HP:0000118"), // almost at the top of all terms
    ("Abnormality of the upper arm", "HP:0001454"),
    ("Arachnodactyly", "HP:0001166"), // 2 parents
    ("Seizure", "HP:0001250"),
    ("Short middle phalanx of the 3rd finger", "HP:0009439"), // 3 parents
];

fn load_hpo(hpo_path: &str) -> MinimalCsrOntology {
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();
    let reader = GzDecoder::new(BufReader::new(
        File::open(hpo_path).expect("HPO file should exist"),
    ));
    loader
        .load_from_read(reader)
        .expect("HPO should be parsable")
}

fn hierarchy_traversals(c: &mut Criterion) {
    let hpo = load_hpo(HPO_PATH);

    macro_rules! benchmark_hierarchy_traversal {
        ($group: expr, $func: expr, $name: expr, $curie: expr) => {
            $group.bench_function(BenchmarkId::from_parameter($name), |b| {
                let term_id = $curie.parse::<TermId>().expect("Curie should be valid");
                let idx = hpo.term_index(&term_id).expect("Should be there!");
                b.iter(|| {
                    $func(idx).for_each(|t| {
                        black_box(t);
                    });
                })
            });
        };
    }

    let mut group = c.benchmark_group("HierarchyTraversals::iter_parent_idxs");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &PAYLOAD {
        benchmark_hierarchy_traversal!(group, |idx| hpo.iter_parent_idxs(idx), label, curie);
    }
    group.finish();

    let mut group = c.benchmark_group("HierarchyTraversals::iter_ancestor_idxs");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &PAYLOAD {
        benchmark_hierarchy_traversal!(group, |idx| hpo.iter_ancestor_idxs(idx), label, curie);
    }
    group.finish();

    let mut group = c.benchmark_group("HierarchyTraversals::iter_child_idxs");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &PAYLOAD {
        benchmark_hierarchy_traversal!(group, |idx| hpo.iter_child_idxs(idx), label, curie);
    }
    group.finish();

    // Iterate descendants indices
    let mut group = c.benchmark_group("HierarchyTraversals::iter_descendant_idxs");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &PAYLOAD {
        benchmark_hierarchy_traversal!(group, |idx| hpo.iter_descendant_idxs(idx), label, curie);
    }
    group.finish();
}

fn hierarchy_walks(c: &mut Criterion) {
    let hpo = load_hpo(HPO_PATH);

    macro_rules! benchmark_hierarchy_walk {
        ($group: expr, $func: expr, $name: expr, $curie: expr) => {
            $group.bench_function(BenchmarkId::from_parameter($name), |b| {
                let term_id = $curie.parse::<TermId>().expect("Curie should be valid");
                b.iter(|| {
                    $func(&term_id).for_each(|tid| {
                        black_box(tid);
                    });
                })
            });
        };
    }

    let mut group = c.benchmark_group("HierarchyWalks::iter_parent_ids");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &PAYLOAD {
        benchmark_hierarchy_walk!(group, |idx| hpo.iter_parent_ids(idx), label, curie);
    }
    group.finish();

    let mut group = c.benchmark_group("HierarchyWalks::iter_ancestor_ids");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &PAYLOAD {
        benchmark_hierarchy_walk!(group, |idx| hpo.iter_ancestor_ids(idx), label, curie);
    }
    group.finish();

    let mut group = c.benchmark_group("HierarchyWalks::iter_child_ids");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &PAYLOAD {
        benchmark_hierarchy_walk!(group, |idx| hpo.iter_child_ids(idx), label, curie);
    }
    group.finish();

    let mut group = c.benchmark_group("HierarchyWalks::iter_descendant_ids");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &PAYLOAD {
        benchmark_hierarchy_walk!(group, |idx| hpo.iter_descendant_ids(idx), label, curie);
    }
    group.finish();
}

criterion_group!(benches, hierarchy_traversals, hierarchy_walks);
criterion_main!(benches);
