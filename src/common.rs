//! The module with constants for working with various ontologies.

use crate::{
    term_id::{InnerTermId, KnownPrefix},
    TermId,
};

/// A private function for streamlining creation of well-known term IDs.
const fn make_term_id(prefix: KnownPrefix, id: u32, len: u8) -> TermId {
    TermId::from_inner(InnerTermId::Known(prefix, id, len))
}

// TODO: should in fact be static constants!
/// Constants for working with Human Phenotype Ontology (HPO).
pub mod hpo {
    use crate::{term_id::KnownPrefix, TermId};

    use super::make_term_id;
    /// [All (HP:0000001)](http://purl.obolibrary.org/obo/HP_0000001)
    /// is the root of all terms in the HPO.
    pub static ALL: TermId = make_term_id(KnownPrefix::HP, 1, 7);

    /// [Phenotypic abnormality (HP:0000118)](http://purl.obolibrary.org/obo/HP_0000118)
    /// is the root of the phenotypic abnormality submodule of the HPO.
    pub static PHENOTYPIC_ABNORMALITY: TermId = make_term_id(KnownPrefix::HP, 118, 7);

    /// [Clinical modifier (HP:0012823)](http://purl.obolibrary.org/obo/HP_0012823)
    /// is the root of HPO's submodule with terms to characterize
    /// and specify the phenotypic abnormalities defined in the Phenotypic abnormality subontology,
    /// with respect to severity, laterality, age of onset, and other aspects.
    pub static CLINICAL_MODIFIER: TermId = make_term_id(KnownPrefix::HP, 12823, 7);

    #[cfg(test)]
    pub mod test {
        use super::super::make_term_id;
        use crate::{term_id::KnownPrefix, TermId};
        /// Abnormality of the musculoskeletal system (HP:0033127)
        pub static ABNORMALITY_OF_MUSCULOSKELETAL_SYSTEM: TermId =
            make_term_id(KnownPrefix::HP, 33127, 7);
        /// Abnormality of limbs (HP:0040064)
        pub static ABNORMALITY_OF_LIMBS: TermId = make_term_id(KnownPrefix::HP, 40064, 7);
        /// Abnormality of the nervous system (HP:0000707)
        pub static ABNORMALITY_OF_THE_NERVOUS_SYSTEM: TermId =
            make_term_id(KnownPrefix::HP, 707, 7);
        /// Arachnodactyly (HP:0001166)
        pub static ARACHNODACTYLY: TermId = make_term_id(KnownPrefix::HP, 1166, 7);
        /// Clonic seizure (HP:0020221)
        pub static CLONIC_SEIZURE: TermId = make_term_id(KnownPrefix::HP, 20221, 7);
        /// Seizure (HP:0001250)
        pub static SEIZURE: TermId = make_term_id(KnownPrefix::HP, 1250, 7);
        /// Polydactyly (HP:0010442)
        pub static POLYDACTYLY: TermId = make_term_id(KnownPrefix::HP, 10442, 7);
        /// Hypertension (HP:0000822)
        pub static HYPERTENSION: TermId = make_term_id(KnownPrefix::HP, 822, 7);
    }
}

/// Constants for working with Medical Action Ontology (MAxO).
pub mod maxo {
    use crate::{term_id::KnownPrefix, TermId};

    use super::make_term_id;
    /// [medical action (MAXO:0000001)](http://purl.obolibrary.org/obo/MAXO_0000001)
    /// is the root of all terms in the MAxO.
    pub static MEDICAL_ACTION: TermId = make_term_id(KnownPrefix::MAXO, 1, 7);
}

/// Constants for working with Unit of Measurement Ontology (UO).
pub mod uo {
    use crate::{term_id::KnownPrefix, TermId};

    use super::make_term_id;
    /// [unit (UO:0000000)](http://purl.obolibrary.org/obo/UO_0000000)
    /// is the root of all terms in the UO.
    pub static UNIT: TermId = make_term_id(KnownPrefix::UO, 0, 7);
}

/// Constants for working with Gene Ontology (GO).
pub mod go {
    use crate::{term_id::KnownPrefix, TermId};

    use super::make_term_id;
    /// [biological process (GO:0008150)](http://purl.obolibrary.org/obo/GO_0008150)
    /// is one of three roots of the GO.
    pub static BIOLOGICAL_PROCESS: TermId = make_term_id(KnownPrefix::GO, 8150, 7);
    /// [cellular component (GO:0005575)](http://purl.obolibrary.org/obo/GO_0005575)
    /// is one of three roots of the GO.
    pub static CELLULAR_COMPONENT: TermId = make_term_id(KnownPrefix::GO, 5575, 7);
    /// [molecular function (GO:0003674)](http://purl.obolibrary.org/obo/GO_0003674)
    /// is one of three roots of the GO.
    pub static MOLECULAR_FUNCTION: TermId = make_term_id(KnownPrefix::GO, 3674, 7);
}

/// Constants for working with Mammalian Phenotype Ontology (MP).
pub mod mp {
    use crate::{term_id::KnownPrefix, TermId};

    use super::make_term_id;
    /// [mammalian phenotype (MP:0000001)](http://purl.obolibrary.org/obo/MP_0000001)
    /// is the of the MO.
    pub static MAMMALIAN_PHENOTYPE: TermId = make_term_id(KnownPrefix::MP, 1, 7);
}
