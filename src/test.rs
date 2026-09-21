use std::{fs::File, io::BufReader, sync::OnceLock};

use flate2::bufread::GzDecoder;

use crate::{io::OntologyLoaderBuilder, ontology::csr::MinimalCsrOntology};

const HPO_PATH: &str = "resources/hp.v2024-08-13.json.gz";

pub(crate) fn hpo() -> &'static MinimalCsrOntology {
    static ONTOLOGY: OnceLock<MinimalCsrOntology> = OnceLock::new();
    ONTOLOGY.get_or_init(|| {
        let reader = GzDecoder::new(BufReader::new(
            File::open(HPO_PATH).expect("Obographs JSON file should exist"),
        ));

        let loader = OntologyLoaderBuilder::new().obographs_parser().build();

        loader
            .load_from_read(reader)
            .expect("Obographs JSON should be well formatted")
    })
}
