use crate::base::{Identified, TermId};
use std::fmt::Debug;

pub trait AltTermIdAware {
    type TermIdIter<'a>: Iterator<Item = &'a TermId>
    where
        Self: 'a;

    fn iter_alt_term_ids(&self) -> Self::TermIdIter<'_>;

    fn alt_term_id_count(&self) -> usize;
}

pub trait MinimalTerm: Identified + AltTermIdAware + Clone + Debug + PartialEq {
    fn name(&self) -> &str;

    fn is_current(&self) -> bool;

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
        alt_term_ids: Vec<TermId>,
        name: String,
        is_obsolete: bool,
    }

    impl SimpleMinimalTerm {
        pub fn new<T: ToString>(
            term_id: TermId,
            name: T,
            alt_term_ids: Vec<TermId>,
            is_obsolete: bool,
        ) -> Self {
            SimpleMinimalTerm {
                term_id,
                name: name.to_string(),
                alt_term_ids,
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
