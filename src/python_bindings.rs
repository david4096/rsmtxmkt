use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use crate::load_matrix_market;

/// Expose the load_matrix_market function to Python.
#[pyfunction]
fn py_load_matrix_market(filename: &str) -> PyResult<Py<PyAny>> {
    Python::with_gil(|py| {
        // Load the matrix from the file
        let result = tokio::runtime::Runtime::new().unwrap().block_on(load_matrix_market(filename));
        match result {
            Ok(matrix) => {
                let py_matrix = PyDict::new(py);

                // Convert indptr to Vec<usize>
                let indptr: Vec<usize> = matrix.indptr().as_slice().expect("Expected indptr to be available").to_vec();
                py_matrix.set_item("indptr", PyList::new(py, &indptr))?;

                // Convert indices to Vec<usize>
                let indices: Vec<usize> = matrix.indices().to_vec();
                py_matrix.set_item("indices", PyList::new(py, &indices))?;

                // Convert values to Vec<f64>
                let values: Vec<f64> = matrix.data().to_vec();
                py_matrix.set_item("values", PyList::new(py, &values))?;

                Ok(py_matrix.into())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string())),
        }
    })
}

/// Define the Python module.
#[pymodule]
fn rsmtxmkt(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_load_matrix_market, m)?)?;
    Ok(())
}
