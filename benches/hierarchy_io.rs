use std::fs::File;
use std::io::{BufReader, Read};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use flate2::bufread::GzDecoder;
use ontolius::io::OntologyLoaderBuilder;
use ontolius::ontology::csr::MinimalCsrOntology;

fn load_csr_ontology(c: &mut Criterion) {
    let path = "resources/hp.v2024-08-13.json.gz";
    let mut reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
    let mut data = Vec::new();
    reader.read_to_end(&mut data).unwrap();

    let loader = OntologyLoaderBuilder::new().obographs_parser().build();

    let mut group = c.benchmark_group("CsrOntologyLoader");
    group.bench_function("CsrOntologyLoader::load", |b| {
        b.iter(|| {
            let ontology: MinimalCsrOntology = loader.load_from_buf_read(&*data).unwrap();
            black_box(ontology);
        })
    });
    group.finish();
}

criterion_group!(benches, load_csr_ontology);
criterion_main!(benches);
