#![doc = include_str!("../README.md")]
#![deny(unsafe_code)] // at least for now.. 👻

mod base;
pub mod hierarchy;
pub mod io;
pub mod ontology;
pub mod prelude;
pub mod term;

pub use base::{Identified, TermId};
#[cfg(feature = "pyo3")]
pub mod py;
