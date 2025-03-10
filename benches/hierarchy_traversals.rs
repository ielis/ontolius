use std::fs::File;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use flate2::read::GzDecoder;
use ontolius::base::term::simple::SimpleMinimalTerm;
use ontolius::ontology::csr::BetaCsrOntology;
use ontolius::ontology::HierarchyTraversals;
use ontolius::prelude::*;

fn hierarchy_traversals(c: &mut Criterion) {
    let path = "resources/hp.v2024-08-13.json.gz";
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();
    let reader = GzDecoder::new(File::open(path).expect("Missing ontology file"));
    let hpo: BetaCsrOntology<u32, SimpleMinimalTerm> = loader.load_from_read(reader).unwrap();

    macro_rules! bench_traversal {
        ($group: expr, $func: expr, $name: expr, $curie: expr) => {
            $group.bench_function(BenchmarkId::from_parameter($name), |b| {
                let term_id = $curie.parse::<TermId>().expect("Curie should be parsable");
                let term_idx = hpo.term_index(&term_id).expect("Should be there!");
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

    let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_parents_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(group, |idx| hpo.iter_parent_idxs(idx), label, curie);
    }
    group.finish();
    // let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_node_and_parents_of");
    // group.throughput(criterion::Throughput::Elements(1));
    // for &(label, curie) in &payload {
    //     bench_traversal!(
    //         group,
    //         |term_id| hierarchy.iter_node_and_parents_of(term_id),
    //         label,
    //         curie
    //     );
    // }
    // group.finish();

    let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_ancestors_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(group, |idx| hpo.iter_ancestor_idxs(idx), label, curie);
    }
    group.finish();
    // let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_node_and_ancestors_of");
    // group.throughput(criterion::Throughput::Elements(1));
    // for &(label, curie) in &payload {
    //     bench_traversal!(
    //         group,
    //         |idx| hierarchy.iter_node_and_ancestors_of(idx),
    //         label,
    //         curie
    //     );
    // }
    // group.finish();

    let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_children_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(group, |idx| hpo.iter_child_idxs(idx), label, curie);
    }
    group.finish();
    // let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_node_and_children_of");
    // group.throughput(criterion::Throughput::Elements(1));
    // for &(label, curie) in &payload {
    //     bench_traversal!(
    //         group,
    //         |term_id| hierarchy.iter_node_and_children_of(term_id),
    //         label,
    //         curie
    //     );
    // }
    // group.finish();

    let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_descendants_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(group, |idx| hpo.iter_descendant_idxs(idx), label, curie);
    }
    group.finish();
    // let mut group = c.benchmark_group("CsrOntologyHierarchy::iter_node_and_descendants_of");
    // group.throughput(criterion::Throughput::Elements(1));
    // for &(label, curie) in &payload {
    //     bench_traversal!(
    //         group,
    //         |term_id| hierarchy.iter_node_and_descendants_of(term_id),
    //         label,
    //         curie
    //     );
    // }
    // group.finish();
}

criterion_group!(benches, hierarchy_traversals);
criterion_main!(benches);
