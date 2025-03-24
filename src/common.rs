//! The module with constants for working with various ontologies.

use crate::{
    term_id::{InnerTermId, Prefix},
    TermId,
};

/// A private function for streamlining creation of well-known term IDs.
const fn make_term_id(prefix: Prefix, id: u32, len: u8) -> TermId {
    TermId::from_inner(InnerTermId::Known(prefix, id, len))
}

// TODO: should in fact be static constants!
/// Constants for working with Human Phenotype Ontology (HPO).
pub mod hpo {
    use crate::{term_id::Prefix, TermId};

    use super::make_term_id;
    /// [All (HP:0000001)](http://purl.obolibrary.org/obo/HP_0000001)
    /// is the root of all terms in the HPO.
    pub static ALL: TermId = make_term_id(Prefix::HP, 1, 7);

    /// [Phenotypic abnormality (HP:0000118)](http://purl.obolibrary.org/obo/HP_0000118)
    /// is the root of the phenotypic abnormality sub-module of the HPO.
    pub static PHENOTYPIC_ABNORMALITY: TermId = make_term_id(Prefix::HP, 118, 7);

    /// [Clinical modifier (HP:0012823)](http://purl.obolibrary.org/obo/HP_0012823)
    /// is the root of HPO's submodule with terms to characterize
    /// and specify the phenotypic abnormalities defined in the Phenotypic abnormality subontology,
    /// with respect to severity, laterality, age of onset, and other aspects.
    pub static CLINICAL_MODIFIER: TermId = make_term_id(Prefix::HP, 12823, 7);
}

/// Constants for working with Medical Action Ontology (MAxO).
pub mod maxo {
    use crate::{term_id::Prefix, TermId};

    use super::make_term_id;
    /// [medical action (MAXO:0000001)](http://purl.obolibrary.org/obo/MAXO_0000001)
    /// is the root of all terms in the MAxO.
    pub static MEDICAL_ACTION: TermId = make_term_id(Prefix::MAXO, 1, 7);
}

/// Constants for working with Gene Ontology (GO).
pub mod go {
    use crate::{term_id::Prefix, TermId};

    use super::make_term_id;
    /// [biological process (GO:0008150)](http://purl.obolibrary.org/obo/GO_0008150)
    /// is one of three roots of the GO.
    pub static BIOLOGICAL_PROCESS: TermId = make_term_id(Prefix::GO, 8150, 7);
    /// [cellular component (GO:0005575)](http://purl.obolibrary.org/obo/GO_0005575)
    /// is one of three roots of the GO.
    pub static CELLULAR_COMPONENT: TermId = make_term_id(Prefix::GO, 5575, 7);
    /// [molecular function (GO:0003674)](http://purl.obolibrary.org/obo/GO_0003674)
    /// is one of three roots of the GO.
    pub static MOLECULAR_FUNCTION: TermId = make_term_id(Prefix::GO, 3674, 7);
}
