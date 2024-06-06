//! Module to support using Ontolius from Python.
use std::{ops::Deref, str::FromStr};

use pyo3::{exceptions::PyValueError, intern, prelude::*, pyclass::CompareOp, types::PyString};

use super::TermId;

/// `PyTermId` is a transparent wrapper around [`TermId`], a validated compact identifier (CURIE),
/// that works in Python.
///
/// In Python, the class is denoted as `TermId` and we can create an instance from a CURIE `str`
/// by running `TermId.from_curie(curie)`, where `curie` is e.g. `HP:0001250`.
///
/// When a Python object is sent to Rust, depending on context, `PyTermId`
/// can be created from a Python CURIE `str` (e.g. `HP:0001250`)
/// or from a Python object that has `prefix` and `id` properties/attributes
/// that return *prefix* (e.g. `HP`) and *id* (e.g. `0001250`) CURIE parts.
///
/// `PyTermId` implements `__eq__()`, `__hash__()`, `__richcmp__()`,
/// `__repr__()`, and `__str__()` Python magic methods.
///
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[pyclass(name = "TermId")]
pub struct PyTermId(TermId);

/// Get the inner [`TermId`].
impl Deref for PyTermId {
    type Target = TermId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[pymethods]
impl PyTermId {
    /// Create `TermId` from a CURIE string (e.g. `HP:0001250`).
    #[staticmethod]
    fn from_curie(curie: &str) -> PyResult<Self> {
        match TermId::from_str(curie) {
            Ok(t) => Ok(PyTermId(t)),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        Ok(op.matches(self.0.cmp(&other.0)))
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!("TermId('{}')", self.0)
    }
}

impl<'source> FromPyObject<'source> for PyTermId {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        if ob.is_instance_of::<PyString>() {
            // CURIE str
            PyTermId::from_curie(ob_to_py_string(ob)?)
        } else if ob.hasattr(intern!(ob.py(), "prefix"))? && ob.hasattr(intern!(ob.py(), "id"))? {
            // HPO toolkit `TermId`
            let prefix = ob.getattr("prefix")?;
            let id = ob.getattr("id")?;
            if prefix.is_instance_of::<PyString>() && id.is_instance_of::<PyString>() {
                Ok(PyTermId(TermId::from((
                    ob_to_py_string(prefix)?,
                    ob_to_py_string(id)?,
                ))))
            } else {
                Err(PyValueError::new_err("Cannot create `PyTermId` from an object with non-str `prefix` and `id` attributes"))
            }
        } else {
            Err(PyValueError::new_err(
                "Cannot create `PyTermId` from the provided object",
            ))
        }
    }
}

fn ob_to_py_string(ob: &PyAny) -> PyResult<&str> {
    ob.downcast::<PyString>()?.to_str()
}
