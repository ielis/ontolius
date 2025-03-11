/// Human Phenotype Ontology tests.
mod human_phenotype_ontology {
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::BufReader;

    use flate2::bufread::GzDecoder;
    use ontolius::io::OntologyLoaderBuilder;
    use ontolius::ontology::csr::CsrOntology;
    use ontolius::ontology::{HierarchyWalks, OntologyTerms};
    use ontolius::prelude::{MinimalTerm, Term};
    use ontolius::term::simple::{SimpleMinimalTerm, SimpleTerm};
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
            .flat_map(|i| hpo.term_by_id(i).map(MinimalTerm::name))
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
            .flat_map(|i| hpo.term_by_id(i).map(MinimalTerm::name))
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
            .flat_map(|i| hpo.term_by_id(i).map(MinimalTerm::name))
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
            .flat_map(|i| hpo.term_by_id(i).map(MinimalTerm::name))
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
    use ontolius::ontology::{HierarchyWalks, OntologyTerms};
    use ontolius::prelude::*;
    use ontolius::term::simple::{SimpleMinimalTerm, SimpleTerm};
    use ontolius::{io::OntologyLoaderBuilder, ontology::csr::CsrOntology};

    #[test]
    fn load_go() {
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();

        let path = "resources/go-basic.v2025-02-06.json.gz";
        let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));

        let go: CsrOntology<u32, SimpleMinimalTerm> = loader
            .load_from_read(reader)
            .expect("Loading of the test file should succeed");

        let pda = TermId::from(("GO", "0004738")); // pyruvate dehydrogenase activity
                                                   // let node = go
                                                   //     .id_to_idx(&pda)
                                                   //     .expect("Pyruvate dehydrogenase activity should be in GO");

        let names: Vec<_> = go
            .iter_ancestor_ids(&pda)
            .flat_map(|tid| go.term_by_id(tid).map(MinimalTerm::name))
            .collect();
        assert_eq!(
            names,
            &[
                "oxidoreductase activity, acting on the aldehyde or oxo group of donors",
                "oxidoreductase activity",
                "catalytic activity",
                "molecular_function",
            ]
        );
    }

    #[test]
    #[ignore = "just for interactive debugging"]
    fn load_full_go() {
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();

        let path = "resources/go-basic.v2025-02-06.json.gz";
        let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));

        let go: CsrOntology<u32, SimpleTerm> = loader
            .load_from_read(reader)
            .expect("Loading of the test file should succeed");

        println!("{:?}", go.len());
    }
}
