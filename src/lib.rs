#![doc = include_str!("../README.md")]
#![deny(unsafe_code)] // at least for now.. 👻

pub mod hierarchy;
pub mod io;
pub mod ontology;
pub mod prelude;
pub mod term;
mod term_id;

pub use term_id::{Identified, TermId};
#[cfg(feature = "pyo3")]
pub mod py;

pub mod common;
