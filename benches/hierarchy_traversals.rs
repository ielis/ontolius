use std::path::Path;

use std::str::FromStr;
use std::vec;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use curie_util::TrieCurieUtil;
use ontolius::io::obographs::ObographsParser;
use ontolius::ontology::csr::CsrOntology;
use ontolius::prelude::*;

fn hierarchy_traversals(c: &mut Criterion) {
    let path = Path::new("/home/ielis/data/ontologies/hpo/2023-10-09/hp.json");
    let loader = OntologyLoaderBuilder::new()
        .parser(ObographsParser::new(TrieCurieUtil::default()))
        .build();
    let ontology: CsrOntology<usize, _> = loader.load_from_path(path).unwrap();

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

    let mut group = c.benchmark_group("CsrOntologyHierarchy::parents_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(group, |term_id| hierarchy.parents_of(term_id), label, curie);
    }
    group.finish();

    let mut group = c.benchmark_group("CsrOntologyHierarchy::ancestors_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(
            group,
            |term_id| hierarchy.ancestors_of(term_id),
            label,
            curie
        );
    }
    group.finish();

    let mut group = c.benchmark_group("CsrOntologyHierarchy::children_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(
            group,
            |term_id| hierarchy.children_of(term_id),
            label,
            curie
        );
    }
    group.finish();

    let mut group = c.benchmark_group("CsrOntologyHierarchy::descendants_of");
    group.throughput(criterion::Throughput::Elements(1));
    for &(label, curie) in &payload {
        bench_traversal!(
            group,
            |term_id| hierarchy.descendants_of(term_id),
            label,
            curie
        );
    }
    group.finish();
}

criterion_group!(benches, hierarchy_traversals);
criterion_main!(benches);
