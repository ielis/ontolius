use std::fs::File;
use std::str::FromStr;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use flate2::read::GzDecoder;
use ontolius::ontology::csr::MinimalCsrOntology;
use ontolius::prelude::*;

fn hierarchy_traversals(c: &mut Criterion) {
    let path = "resources/hp.v2024-08-13.json.gz";
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();
    let reader = GzDecoder::new(File::open(path).expect("Missing ontology file"));
    let ontology: MinimalCsrOntology = loader.load_from_read(reader).unwrap();

    macro_rules! bench_traversal {
        ($group: expr, $func: expr, $name: expr, $curie: expr) => {
            $group.bench_function(BenchmarkId::from_parameter($name), |b| {
                let term_id = TermId::from_str($curie).expect("Curie should be parsable");
                let term_idx = ontology.id_to_idx(&term_id).expect("Should be there!");
                b.iter(|| {
                    $func(term_idx).for_each(|t| {
                        black_box(t);
                    });
                })
            });
        };
    }

    let payload = vec![
        ("Phenotypic abnormality", "HP:0000118"), // almost at the top of all terms
        ("Abnormality of the upper arm", "HP:0001454"),
        ("Arachnodactyly", "HP:0001166"), // 2 parents
        ("Seizure", "HP:0001250"),
        ("Short middle phalanx of the 3rd finger", "HP:0009439"), // 3 parents
    ];

    let hierarchy = ontology.hierarchy();

    let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_parents_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(
            group,
            |term_id| hierarchy.iter_parents_of(term_id),
            label,
            curie
        );
    }
    group.finish();
    let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_node_and_parents_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(
            group,
            |term_id| hierarchy.iter_node_and_parents_of(term_id),
            label,
            curie
        );
    }
    group.finish();

    let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_ancestors_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(
            group,
            |term_id| hierarchy.iter_ancestors_of(term_id),
            label,
            curie
        );
    }
    group.finish();
    let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_node_and_ancestors_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(
            group,
            |term_id| hierarchy.iter_node_and_ancestors_of(term_id),
            label,
            curie
        );
    }
    group.finish();

    let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_children_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(
            group,
            |term_id| hierarchy.iter_children_of(term_id),
            label,
            curie
        );
    }
    group.finish();
    let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_node_and_children_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(
            group,
            |term_id| hierarchy.iter_node_and_children_of(term_id),
            label,
            curie
        );
    }
    group.finish();

    let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_descendants_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(
            group,
            |term_id| hierarchy.iter_descendants_of(term_id),
            label,
            curie
        );
    }
    group.finish();
    let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_node_and_descendants_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(
            group,
            |term_id| hierarchy.iter_node_and_descendants_of(term_id),
            label,
            curie
        );
    }
    group.finish();
}

criterion_group!(benches, hierarchy_traversals);
criterion_main!(benches);
