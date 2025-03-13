/// Human Phenotype Ontology tests.
mod human_phenotype_ontology {
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::BufReader;

    use flate2::bufread::GzDecoder;
    use ontolius::io::OntologyLoaderBuilder;
    use ontolius::ontology::csr::CsrOntology;
    use ontolius::ontology::{HierarchyWalks, OntologyTerms};
    use ontolius::term::simple::{SimpleMinimalTerm, SimpleTerm};
    use ontolius::term::{MinimalTerm, Term};
    use ontolius::TermId;
    use rstest::{fixture, rstest};

    #[fixture]
    fn hpo() -> CsrOntology<u32, SimpleMinimalTerm> {
        let path = "resources/hp.v2024-08-13.json.gz";
        let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();

        loader.load_from_read(reader).unwrap()
    }

    #[rstest]
    fn check_children(hpo: CsrOntology<u32, SimpleMinimalTerm>) -> anyhow::Result<()> {
        let term_id = TermId::from(("HP", "0032677"));
        assert_eq!(
            hpo.term_by_id(&term_id).unwrap().name(),
            "Generalized-onset motor seizure"
        );

        let children_names: HashSet<_> = hpo
            .iter_child_ids(&term_id)
            .map(|i| hpo.term_by_id(i).map(MinimalTerm::name).unwrap())
            .collect();

        let expected = [
            "Bilateral tonic-clonic seizure with generalized onset",
            "Generalized atonic seizure",
            "Generalized clonic seizure",
            "Generalized myoclonic seizure",
            "Generalized myoclonic-atonic seizure",
            "Generalized myoclonic-tonic-clonic seizure",
            "Generalized tonic seizure",
            "Generalized-onset epileptic spasm",
        ]
        .into_iter()
        .collect();
        assert_eq!(children_names, expected);

        Ok(())
    }

    #[rstest]
    fn check_descendants(hpo: CsrOntology<u32, SimpleMinimalTerm>) -> anyhow::Result<()> {
        let term_id = TermId::from(("HP", "0002863"));
        assert_eq!(hpo.term_by_id(&term_id).unwrap().name(), "Myelodysplasia");

        let descendant_names: HashSet<_> = hpo
            .iter_descendant_ids(&term_id)
            .map(|i| hpo.term_by_id(i).map(MinimalTerm::name).unwrap())
            .collect();

        let expected = [
            "Single lineage myelodysplasia",
            // Child of Single lineage myelodysplasia
            "Refractory anemia with ringed sideroblasts",
            "Multiple lineage myelodysplasia",
            "Bilineage myelodysplasia",
        ]
        .into_iter()
        .collect();

        assert_eq!(descendant_names, expected);

        Ok(())
    }

    #[rstest]
    fn check_parents(hpo: CsrOntology<u32, SimpleMinimalTerm>) -> anyhow::Result<()> {
        let seizure = TermId::from(("HP", "0032677"));
        assert_eq!(
            hpo.term_by_id(&seizure).unwrap().name(),
            "Generalized-onset motor seizure"
        );

        let parent_names: HashSet<_> = hpo
            .iter_parent_ids(&seizure)
            .map(|i| hpo.term_by_id(i).map(MinimalTerm::name).unwrap())
            .collect();

        let expected = ["Generalized-onset seizure", "Motor seizure"]
            .into_iter()
            .collect();

        assert_eq!(parent_names, expected);

        Ok(())
    }

    #[rstest]
    fn check_ancestors(hpo: CsrOntology<u32, SimpleMinimalTerm>) -> anyhow::Result<()> {
        let term_id = TermId::from(("HP", "0002266"));
        assert_eq!(
            hpo.term_by_id(&term_id).unwrap().name(),
            "Focal clonic seizure"
        );

        let ancestor_names: HashSet<_> = hpo
            .iter_ancestor_ids(&term_id)
            .map(|i| hpo.term_by_id(i).map(MinimalTerm::name).unwrap())
            .collect();

        let expected = [
            "Focal motor seizure",
            "Focal-onset seizure",
            "Motor seizure",
            "Clonic seizure",
            "Seizure",
            "Abnormal nervous system physiology",
            "Abnormality of the nervous system",
            "Phenotypic abnormality",
            "All",
        ]
        .into_iter()
        .collect();

        assert_eq!(ancestor_names, expected);

        Ok(())
    }

    #[test]
    #[ignore = "just for interactive debugging"]
    fn load_full_data() {
        let path = "resources/hp.v2024-08-13.json.gz";
        let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();

        let hpo: CsrOntology<u32, SimpleTerm> = loader.load_from_read(reader).unwrap();

        for ft in hpo.iter_terms() {
            println!("{:?}", ft.definition())
        }
    }
}

/// Gene Ontology tests.
mod gene_ontology {

    use std::fs::File;
    use std::io::BufReader;

    use flate2::bufread::GzDecoder;
    use ontolius::common::go::{BIOLOGICAL_PROCESS, CELLULAR_COMPONENT, MOLECULAR_FUNCTION};
    use ontolius::io::OntologyLoaderBuilder;
    use ontolius::ontology::csr::MinimalCsrOntology;
    use ontolius::ontology::{HierarchyWalks, OntologyTerms};
    use ontolius::term::MinimalTerm;
    use ontolius::TermId;

    const TOY_GO_PATH: &str = "resources/go/go.toy.json.gz";

    fn load_toy_go() -> MinimalCsrOntology {
        OntologyLoaderBuilder::new()
            .obographs_parser()
            .build()
            .load_from_read(GzDecoder::new(BufReader::new(
                File::open(TOY_GO_PATH).unwrap(),
            )))
            .expect("Loading of the test file should succeed")
    }

    macro_rules! test_ancestors {
        ($($go: expr, $curie: expr, $expected: expr)*) => {
            $(
                let query: TermId = $curie.parse().unwrap();

                let mut names: Vec<_> = $go
                    .iter_ancestor_ids(&query)
                    .map(|tid| $go.term_by_id(tid).map(MinimalTerm::name).unwrap())
                    .collect();
                names.sort();
                assert_eq!(
                    names,
                    $expected,
                );
            )*
        };
    }

    #[test]
    fn iter_ancestor_ids() {
        let go = load_toy_go();

        test_ancestors!(
            go,
            "GO:0051146", // striated muscle cell differentiation
            &[
                "biological_process",
                "cell differentiation",
                "cellular developmental process",
                "cellular process",
                "developmental process",
                "muscle cell differentiation",
            ]
        );
        test_ancestors!(
            go,
            "GO:0052693", // epoxyqueuosine reductase activity
            &[
                "catalytic activity",
                "molecular_function",
                "oxidoreductase activity",
            ]
        );
        test_ancestors!(
            go,
            "GO:0005634", // nucleus
            &[
                "cellular anatomical structure",
                "cellular_component",
                "intracellular membrane-bounded organelle",
                "intracellular organelle",
                "membrane-bounded organelle",
                "organelle",
            ]
        );
    }

    #[test]
    fn we_get_expected_descendant_counts_for_go_subroots() {
        let go = load_toy_go();

        let mf_count = go.iter_descendant_ids(&MOLECULAR_FUNCTION).count();
        assert_eq!(mf_count, 3);
        let bp_count = go.iter_descendant_ids(&BIOLOGICAL_PROCESS).count();
        assert_eq!(bp_count, 8);
        let cc_count = go.iter_descendant_ids(&CELLULAR_COMPONENT).count();
        assert_eq!(cc_count, 7);
    }
}
