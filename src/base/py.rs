use std::str::FromStr;

use pyo3::{exceptions::PyValueError, intern, prelude::*, pyclass::CompareOp, types::PyString};

use super::TermId;

pub(crate) fn init_submodule(_py: &Python<'_>, m: &PyModule) -> PyResult<()> {
    // Make TermId available at the top-level module.
    m.add_class::<PyTermId>()?;
    Ok(())
}

/// `TermId` represents validated compact identifier (CURIE).
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[pyclass(name = "TermId")]
pub(crate) struct PyTermId(TermId);

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
                // TODO: better error string
                Err(PyValueError::new_err("Ooops"))
            }
        } else {
            // TODO: better error string
            Err(PyValueError::new_err("Ooops"))
        }
    }
}

fn ob_to_py_string(ob: &PyAny) -> PyResult<&str> {
    ob.downcast::<PyString>()?.to_str()
}
