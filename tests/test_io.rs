use std::collections::HashSet;

use ontolius::base::TermId;
use ontolius::io::OntologyLoaderBuilder;
use ontolius::ontology::csr::MinimalCsrOntology;
use ontolius::prelude::{
    AncestorNodes, ChildNodes, DescendantNodes, HierarchyAware, MinimalTerm, ParentNodes, TermAware,
};
use rstest::{fixture, rstest};

#[fixture]
fn hpo() -> MinimalCsrOntology {
    let path = "resources/hp.v2024-08-13.json.gz";
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();

    loader.load_from_path(path).unwrap()
}

#[rstest]
fn check_children(hpo: MinimalCsrOntology) -> anyhow::Result<()> {
    let term_id = TermId::from(("HP", "0032677"));
    assert_eq!(
        hpo.id_to_term(&term_id).unwrap().name(),
        "Generalized-onset motor seizure"
    );

    let term_idx = hpo.id_to_idx(&term_id).unwrap();
    let children_names: HashSet<_> = hpo
        .hierarchy()
        .iter_children_of(&term_idx)
        .flat_map(|i| hpo.idx_to_term(i).map(|t| t.name()))
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
fn check_descendants(hpo: MinimalCsrOntology) -> anyhow::Result<()> {
    let term_id = TermId::from(("HP", "0002863"));
    assert_eq!(hpo.id_to_term(&term_id).unwrap().name(), "Myelodysplasia");

    let term_idx = hpo.id_to_idx(&term_id).unwrap();
    let descendant_names: HashSet<_> = hpo
        .hierarchy()
        .iter_descendants_of(&term_idx)
        .flat_map(|i| hpo.idx_to_term(i).map(|t| t.name()))
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
fn check_parents(hpo: MinimalCsrOntology) -> anyhow::Result<()> {
    let seizure = TermId::from(("HP", "0032677"));
    assert_eq!(
        hpo.id_to_term(&seizure).unwrap().name(),
        "Generalized-onset motor seizure"
    );

    let seizure_idx = hpo.id_to_idx(&seizure).unwrap();
    let parents: HashSet<_> = hpo
        .hierarchy()
        .iter_parents_of(&seizure_idx)
        .flat_map(|i| hpo.idx_to_term(i).map(|t| t.name()))
        .collect();

    let expected = ["Generalized-onset seizure", "Motor seizure"]
        .into_iter()
        .collect();

    assert_eq!(parents, expected);

    Ok(())
}

#[rstest]
fn check_ancestors(hpo: MinimalCsrOntology) -> anyhow::Result<()> {
    let term_id = TermId::from(("HP", "0002266"));
    assert_eq!(
        hpo.id_to_term(&term_id).unwrap().name(),
        "Focal clonic seizure"
    );

    let term_idx = hpo.id_to_idx(&term_id).unwrap();
    let ancestor_names: HashSet<_> = hpo
        .hierarchy()
        .iter_ancestors_of(&term_idx)
        .flat_map(|i| hpo.idx_to_term(i).map(|t| t.name()))
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
