//! Ontology term models.
//!
//! The module includes traits and structs for modeling ontology terms.
use crate::{Identified, TermId};

/// Some terms have alternate identifiers,
/// e.g. the identifiers used to refer to the term in the past.
pub trait AltTermIdAware {
    type TermIdIter<'a>: Iterator<Item = &'a TermId>
    where
        Self: 'a;

    fn iter_alt_term_ids(&self) -> Self::TermIdIter<'_>;

    fn alt_term_id_count(&self) -> usize {
        self.iter_alt_term_ids().count()
    }
}

/// `MinimalTerm` describes the minimal requirements of an ontology term.
///
/// On top of inherited traits, such as [`Identified`], [`AltTermIdAware`], and others,
/// the term must have a name and it is either current or obsolete.
pub trait MinimalTerm: Identified + AltTermIdAware {
    /// Get the name of the term, e.g. `Seizure` for [Seizure](https://hpo.jax.org/browse/term/HP:0001250).
    fn name(&self) -> &str;

    /// Test if the term is *primary* and not obsolete.
    fn is_current(&self) -> bool;

    /// Test if the term is *obsolete*.
    fn is_obsolete(&self) -> bool {
        !self.is_current()
    }
}

pub trait CrossReferenced {
    fn xrefs(&self) -> &[TermId];
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SynonymCategory {
    Exact,
    Related,
    Broad,
    Narrow,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SynonymType {
    LaypersonTerm,
    Abbreviation,
    UkSpelling,
    ObsoleteSynonym,
    PluralForm,
    AllelicRequirement,
    SystematicSynonym,
    SyngoOfficialLabel,
    // We may not need these, unless we support ECTO (taken from hpo-toolkit)
    // IUPAC_NAME = enum.auto()
    // INN = enum.auto()
    // BRAND_NAME = enum.auto()
    // IN_PART = enum.auto()
    // SYNONYM = enum.auto()
    // BLAST_NAME = enum.auto()
    // GENBANK_COMMON_NAME = enum.auto()
    // COMMON_NAME = enum.auto()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Synonym {
    pub name: String,
    pub category: Option<SynonymCategory>,
    pub r#type: Option<SynonymType>,
    pub xrefs: Vec<TermId>,
}

pub trait Synonymous {
    fn synonyms(&self) -> &[Synonym];
}

impl CrossReferenced for Synonym {
    fn xrefs(&self) -> &[TermId] {
        &self.xrefs
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Definition {
    pub val: String,
    pub xrefs: Vec<String>,
}

pub trait Term: MinimalTerm {
    fn definition(&self) -> Option<&Definition>;

    fn comment(&self) -> Option<&str>;

    // TODO: add dbXrefs?
}

pub mod simple {

    use super::{
        AltTermIdAware, CrossReferenced, Definition, MinimalTerm, Synonym, Synonymous, Term,
    };
    use crate::{Identified, TermId};

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct SimpleMinimalTerm {
        term_id: TermId,
        alt_term_ids: Box<[TermId]>,
        name: String,
        is_obsolete: bool,
    }

    impl SimpleMinimalTerm {
        pub fn new<T, A>(term_id: TermId, name: T, alt_term_ids: A, is_obsolete: bool) -> Self
        where
            T: ToString,
            A: Into<Box<[TermId]>>,
        {
            SimpleMinimalTerm {
                term_id,
                name: name.to_string(),
                alt_term_ids: alt_term_ids.into(),
                is_obsolete,
            }
        }
    }

    impl Identified for SimpleMinimalTerm {
        fn identifier(&self) -> &TermId {
            &self.term_id
        }
    }

    impl AltTermIdAware for SimpleMinimalTerm {
        type TermIdIter<'a>
            = std::slice::Iter<'a, TermId>
        where
            Self: 'a;

        fn iter_alt_term_ids(&self) -> Self::TermIdIter<'_> {
            self.alt_term_ids.iter()
        }

        fn alt_term_id_count(&self) -> usize {
            self.alt_term_ids.len()
        }
    }

    impl MinimalTerm for SimpleMinimalTerm {
        fn name(&self) -> &str {
            self.name.as_str()
        }

        fn is_current(&self) -> bool {
            !self.is_obsolete
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct SimpleTerm {
        term_id: TermId,
        alt_term_ids: Box<[TermId]>,
        name: String,
        is_obsolete: bool,
        definition: Option<Definition>,
        comment: Option<String>,
        synonyms: Vec<Synonym>,
        xrefs: Vec<TermId>,
    }

    impl SimpleTerm {
        pub fn new<T, A>(
            term_id: TermId,
            name: T,
            alt_term_ids: A,
            is_obsolete: bool,
            definition: Option<Definition>,
            comment: Option<String>,
            synonyms: Vec<Synonym>,
            xrefs: Vec<TermId>,
        ) -> Self
        where
            T: ToString,
            A: Into<Box<[TermId]>>,
        {
            SimpleTerm {
                term_id,
                name: name.to_string(),
                alt_term_ids: alt_term_ids.into(),
                is_obsolete,
                definition,
                comment,
                synonyms,
                xrefs,
            }
        }
    }

    impl Identified for SimpleTerm {
        fn identifier(&self) -> &TermId {
            &self.term_id
        }
    }

    impl AltTermIdAware for SimpleTerm {
        type TermIdIter<'a>
            = std::slice::Iter<'a, TermId>
        where
            Self: 'a;

        fn iter_alt_term_ids(&self) -> Self::TermIdIter<'_> {
            self.alt_term_ids.iter()
        }

        fn alt_term_id_count(&self) -> usize {
            self.alt_term_ids.len()
        }
    }

    impl MinimalTerm for SimpleTerm {
        fn name(&self) -> &str {
            &self.name
        }

        fn is_current(&self) -> bool {
            !self.is_obsolete
        }
    }

    impl CrossReferenced for SimpleTerm {
        fn xrefs(&self) -> &[TermId] {
            &self.xrefs
        }
    }

    impl Synonymous for SimpleTerm {
        fn synonyms(&self) -> &[Synonym] {
            &self.synonyms
        }
    }

    impl Term for SimpleTerm {
        fn definition(&self) -> Option<&Definition> {
            self.definition.as_ref()
        }

        fn comment(&self) -> Option<&str> {
            self.comment.as_deref()
        }
    }
}
