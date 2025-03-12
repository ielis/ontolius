#![doc = include_str!("../README.md")]
#![deny(unsafe_code)] // at least for now.. 👻

pub mod common;
#[deprecated(
    since = "0.5.0",
    note = "The hierarchy traits were moved into `ontolius::ontology` module",
)]
pub mod hierarchy;
pub mod io;
pub mod ontology;
#[deprecated(
    since = "0.5.0",
    note = "The number of traits was reduced and it seems better to perform explicit imports",
)]
pub mod prelude;
#[cfg(feature = "pyo3")]
pub mod py;
pub mod term;
mod term_id;

pub use term_id::{Identified, TermId};
