#![doc = include_str!("../README.md")]
#![deny(unsafe_code)] // at least for now.. 👻

pub mod common;
pub mod io;
pub mod ontology;
#[cfg(feature = "pyo3")]
pub mod py;
pub mod term;
mod term_id;
#[cfg(all(test, feature = "csr"))]
pub(crate) mod test;

pub use term_id::{Identified, Prefix, TermId, TermIdParseError};
