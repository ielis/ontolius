use pyo3::prelude::*;
use crate::base::py as base;

#[pymodule]
fn ontolius(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    base::init_submodule(&py, m)?;
    Ok(())
}
