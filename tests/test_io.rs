#[cfg(test)]
mod tests {

    use ontolius::io::OntologyLoaderBuilder;
    use ontolius::ontology::csr::MinimalCsrOntology;

    #[test]
    fn test_csr_ontology_loader() {
        let path = "resources/hp.small.json.gz";

        let loader = OntologyLoaderBuilder::new().obographs_parser().build();

        let ontology: Result<MinimalCsrOntology, _> = loader.load_from_path(path);

        assert!(ontology.is_ok());
        // TODO: more tests?
    }
}
