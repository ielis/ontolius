use crate::base::{Identified, TermId};

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

pub trait Term: MinimalTerm {
    fn definition(&self) -> Option<&str>;

    fn comment(&self) -> Option<&str>;

    // TODO: add Xrefs and dbXrefs
}

pub mod simple {

    use super::{AltTermIdAware, MinimalTerm};
    use crate::base::{Identified, TermId};

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
        type TermIdIter<'a> = std::slice::Iter<'a, TermId>
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
}
