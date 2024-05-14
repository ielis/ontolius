#[cfg(test)]
mod tests {
    use std::path::Path;

    use curie_util::TrieCurieUtil;
    use ontolius::io::{obographs::ObographsParser, OntologyLoaderBuilder};
    use ontolius::ontology::csr::CsrOntology;

    #[test]
    fn test_csr_ontology_loader() {
        let path = Path::new("resources/hp.small.json");

        let loader = OntologyLoaderBuilder::new()
            .parser(ObographsParser::new(TrieCurieUtil::default()))
            .build();

        let ontology: Result<CsrOntology<usize, _>, _> = loader.load_from_path(path);

        assert!(ontology.is_ok());
        // TODO: more tests?
    }
}
