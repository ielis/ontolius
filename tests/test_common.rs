use ontolius::common::{hpo, go, maxo};

#[test]
fn hpo_commons_are_accessible() {
    assert_eq!(hpo::PHENOTYPIC_ABNORMALITY, ("HP", "0000118"));
    assert_eq!(hpo::ALL, ("HP", "0000001"));
    assert_eq!(hpo::CLINICAL_MODIFIER, ("HP", "0012823"));
}

#[test]
fn go_commons_are_accessible() {
    assert_eq!(go::BIOLOGICAL_PROCESS, ("GO", "0008150"));
    assert_eq!(go::CELLULAR_COMPONENT, ("GO", "0005575"));
    assert_eq!(go::MOLECULAR_FUNCTION, ("GO", "0003674"));
}

#[test]
fn maxo_commons_are_accessible() {
    assert_eq!(maxo::MEDICAL_ACTION, ("MAXO", "0000001"))
}