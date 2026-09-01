use kvd_rs::deserialize::from_str;
use kvd_rs::serialize::to_string;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Parse a KVD document and return its canonical serialization.
///
/// Raises `ValueError` if the input is not valid KVD.
#[pyfunction]
fn canonical(text: &str) -> PyResult<String> {
    let doc =
        from_str(text).map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))?;
    to_string(&doc).map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
}

#[pymodule]
fn pykvd(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(canonical, m)?)?;
    Ok(())
}
