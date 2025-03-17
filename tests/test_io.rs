/// Human Phenotype Ontology tests.
mod human_phenotype_ontology {
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::BufReader;
    use std::sync::{Mutex, OnceLock};

    use flate2::bufread::GzDecoder;
    use ontolius::io::OntologyLoaderBuilder;
    use ontolius::ontology::csr::{CsrOntology, MinimalCsrOntology};
    use ontolius::ontology::{HierarchyWalks, OntologyTerms};
    use ontolius::term::simple::SimpleTerm;
    use ontolius::term::{MinimalTerm, Term};
    use ontolius::TermId;

    const HPO_PATH: &str = "resources/hp.v2024-08-13.json.gz";

    fn hpo() -> &'static Mutex<MinimalCsrOntology> {
        static HPO: OnceLock<Mutex<MinimalCsrOntology>> = OnceLock::new();
        HPO.get_or_init(|| {
            Mutex::new({
                let reader = GzDecoder::new(BufReader::new(
                    File::open(HPO_PATH).expect("Test HPO should exist"),
                ));
                let loader = OntologyLoaderBuilder::new().obographs_parser().build();

                loader.load_from_read(reader).expect("HPO should be OK")
            })
        })
    }

    macro_rules! test_hierarchy_walks {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (walk, query, expected) = $value;

                    let hpo = hpo().lock().unwrap();
                    let term_id: TermId = query.parse().expect("Query should be valid CURIE");

                    let names: HashSet<_> = (walk)(&*hpo, &term_id)
                        .map(|tid| hpo.term_by_id(tid).map(MinimalTerm::name).unwrap().clone())
                        .collect();

                    let expected: HashSet<_> = expected.iter().cloned().collect();

                    assert_eq!(names, expected);
                }
            )*
        };
    }

    test_hierarchy_walks! {
        // Generalized-onset motor seizure
        test_iter_child_ids: (HierarchyWalks::iter_child_ids, "HP:0032677", [
            "Bilateral tonic-clonic seizure with generalized onset",
            "Generalized atonic seizure",
            "Generalized clonic seizure",
            "Generalized myoclonic seizure",
            "Generalized myoclonic-atonic seizure",
            "Generalized myoclonic-tonic-clonic seizure",
            "Generalized tonic seizure",
            "Generalized-onset epileptic spasm",
        ]),
        // Generalized-onset motor seizure
        test_iter_term_and_child_ids: (HierarchyWalks::iter_term_and_child_ids, "HP:0032677", [
            "Generalized-onset motor seizure", // <- the query term's name
            "Bilateral tonic-clonic seizure with generalized onset",
            "Generalized atonic seizure",
            "Generalized clonic seizure",
            "Generalized myoclonic seizure",
            "Generalized myoclonic-atonic seizure",
            "Generalized myoclonic-tonic-clonic seizure",
            "Generalized tonic seizure",
            "Generalized-onset epileptic spasm",
        ]),

        // Myelodysplasia
        test_iter_descendant_ids: (HierarchyWalks::iter_descendant_ids, "HP:0002863", [
            "Single lineage myelodysplasia",
            // Child of Single lineage myelodysplasia
            "Refractory anemia with ringed sideroblasts",
            "Multiple lineage myelodysplasia",
            "Bilineage myelodysplasia",
        ]),
         // Myelodysplasia
         test_iter_term_and_descendant_ids: (HierarchyWalks::iter_term_and_descendant_ids, "HP:0002863", [
            "Myelodysplasia", // <- the query term's name
            "Single lineage myelodysplasia",
            // Child of Single lineage myelodysplasia
            "Refractory anemia with ringed sideroblasts",
            "Multiple lineage myelodysplasia",
            "Bilineage myelodysplasia",
        ]),

        // Generalized-onset motor seizure
        test_iter_parent_ids: (HierarchyWalks::iter_parent_ids, "HP:0032677", [
            "Generalized-onset seizure", "Motor seizure"
        ]),
        // Generalized-onset motor seizure
        test_iter_term_and_parent_ids: (HierarchyWalks::iter_term_and_parent_ids, "HP:0032677", [
            "Generalized-onset motor seizure",
            "Generalized-onset seizure", "Motor seizure"
        ]),
        // Focal clonic seizure
        test_iter_ancestor_ids: (HierarchyWalks::iter_ancestor_ids, "HP:0002266", [
            "Focal motor seizure",
            "Focal-onset seizure",
            "Motor seizure",
            "Clonic seizure",
            "Seizure",
            "Abnormal nervous system physiology",
            "Abnormality of the nervous system",
            "Phenotypic abnormality",
            "All",
        ]),
        // Focal clonic seizure
        test_iter_term_and_ancestor_ids: (HierarchyWalks::iter_term_and_ancestor_ids, "HP:0002266", [
            "Focal clonic seizure",
            "Focal motor seizure",
            "Focal-onset seizure",
            "Motor seizure",
            "Clonic seizure",
            "Seizure",
            "Abnormal nervous system physiology",
            "Abnormality of the nervous system",
            "Phenotypic abnormality",
            "All",
        ]),
    }

    #[test]
    #[ignore = "just for interactive debugging"]
    fn load_full_data() {
        let reader = GzDecoder::new(BufReader::new(File::open(HPO_PATH).unwrap()));
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
        ($($ontology: expr, $curie: expr, $expected: expr)*) => {
            $(
                let query: TermId = $curie.parse().unwrap();

                let mut names: Vec<_> = $ontology
                    .iter_ancestor_ids(&query)
                    .map(|tid| $ontology.term_by_id(tid).map(MinimalTerm::name).unwrap())
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

/// Medical Action Ontology (MAxO) tests.
mod medical_action_ontology {

    use std::fs::File;
    use std::io::BufReader;

    use flate2::bufread::GzDecoder;
    use ontolius::common::maxo::MEDICAL_ACTION;
    use ontolius::io::OntologyLoaderBuilder;
    use ontolius::ontology::csr::MinimalCsrOntology;
    use ontolius::ontology::{HierarchyWalks, OntologyTerms};
    use ontolius::term::MinimalTerm;
    use ontolius::TermId;

    const TOY_MAXO_PATH: &str = "resources/maxo/maxo.toy.json.gz";

    fn load_toy_maxo() -> MinimalCsrOntology {
        OntologyLoaderBuilder::new()
            .obographs_parser()
            .build()
            .load_from_read(GzDecoder::new(BufReader::new(
                File::open(TOY_MAXO_PATH).unwrap(),
            )))
            .expect("Loading of the test file should succeed")
    }

    macro_rules! test_ancestors {
        ($($ontology: expr, $curie: expr, $expected: expr)*) => {
            $(
                let query: TermId = $curie.parse().unwrap();

                let mut names: Vec<_> = $ontology
                    .iter_ancestor_ids(&query)
                    .map(|tid| $ontology.term_by_id(tid).map(MinimalTerm::name).unwrap())
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
        let maxo = load_toy_maxo();

        test_ancestors!(
            maxo,
            "MAXO:0000682", // cardiologist evaluation
            &[
                "diagnostic procedure",
                "internal medicine specialist evaluation",
                "medical action",
                "medical professional evaluation",
                "medical specialist evaluation",
            ]
        );
        test_ancestors!(
            maxo,
            "MAXO:0000185", // antiarrythmic agent therapy
            &[
                "cardiovascular agent therapy",
                "medical action",
                "pharmacotherapy",
                "therapeutic procedure"
            ]
        );
        test_ancestors!(
            maxo,
            "MAXO:0035118", // cardiac catheterization
            &[
                "cardiac device implantation",
                "catheterization",
                "implantation",
                "introduction procedure",
                "medical action",
                "medical device implantation",
                "surgical procedure",
                "therapeutic procedure"
            ]
        );
    }

    #[test]
    fn we_get_expected_descendant_counts_for_maxo_root() {
        let maxo = load_toy_maxo();

        let ma_count = maxo.iter_descendant_ids(&MEDICAL_ACTION).count();
        assert_eq!(ma_count, 16);
    }
}
