//! The base building blocks for working with ontology data.
pub use term::simple::SimpleMinimalTerm;
pub use term::AltTermIdAware;
pub use term::{MinimalTerm, Term};
pub use term_id::{Identified, TermId};

mod term;
mod term_id;
