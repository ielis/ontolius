use criterion::{black_box, criterion_group, criterion_main, Criterion};

use ontolius::io::OntologyLoaderBuilder;
use ontolius::ontology::csr::MinimalCsrOntology;

fn load_csr_ontology(c: &mut Criterion) {
    let path = "resources/hp.v2024-08-13.json.gz";

    let loader = OntologyLoaderBuilder::new().obographs_parser().build();

    let mut group = c.benchmark_group("CsrOntologyLoader");
    group.bench_function("CsrOntologyLoader::load", |b| {
        b.iter(|| {
            let ontology: MinimalCsrOntology = loader.load_from_path(path).unwrap();
            black_box(ontology);
        })
    });
    group.finish();
}

criterion_group!(benches, load_csr_ontology);
criterion_main!(benches);
