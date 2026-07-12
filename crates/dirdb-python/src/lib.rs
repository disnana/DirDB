use dirdb_core::{DirDb, Error};
use pyo3::{
    exceptions::{PyFileNotFoundError, PyRuntimeError, PyValueError},
    prelude::*,
    types::PyAny,
};

#[pyclass(name = "NativeDirDB")]
struct PyDirDb {
    inner: DirDb,
}

#[pymethods]
impl PyDirDb {
    #[new]
    fn new(root: String) -> PyResult<Self> {
        Ok(Self {
            inner: DirDb::open(root).map_err(to_py_error)?,
        })
    }
    fn get(&self, py: Python<'_>, key: String) -> PyResult<PyObject> {
        let inner = self.inner.clone();
        let entry = py.detach(|| inner.get(&key)).map_err(to_py_error)?;
        let encoded = serde_json::to_string(&entry.value)
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        Ok(py
            .import("json")?
            .call_method1("loads", (encoded,))?
            .unbind())
    }
    #[pyo3(signature = (key, value, expected_version=None))]
    fn set(
        &self,
        py: Python<'_>,
        key: String,
        value: &Bound<'_, PyAny>,
        expected_version: Option<u64>,
    ) -> PyResult<u64> {
        let encoded: String = py
            .import("json")?
            .call_method1("dumps", (value,))?
            .extract()?;
        let value: serde_json::Value = serde_json::from_str(&encoded)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let inner = self.inner.clone();
        Ok(py.detach(|| inner.set(&key, &value, expected_version)).map_err(to_py_error)?.version)
    }
    #[pyo3(signature = (key, expected_version=None))]
    fn delete(&self, py: Python<'_>, key: String, expected_version: Option<u64>) -> PyResult<()> {
        let inner = self.inner.clone();
        py.detach(|| inner.delete(&key, expected_version)).map_err(to_py_error)
    }
    fn exists(&self, py: Python<'_>, key: String) -> PyResult<bool> {
        let inner = self.inner.clone();
        py.detach(|| inner.exists(&key)).map_err(to_py_error)
    }
    #[pyo3(signature = (prefix=""))]
    fn list(&self, py: Python<'_>, prefix: String) -> PyResult<Vec<String>> {
        let inner = self.inner.clone();
        py.detach(|| inner.list(&prefix)).map_err(to_py_error)
    }
    fn rebuild_index(&self, py: Python<'_>) -> PyResult<usize> {
        let inner = self.inner.clone();
        py.detach(|| inner.rebuild_index()).map_err(to_py_error)
    }
}
fn to_py_error(error: Error) -> PyErr {
    match error {
        Error::NotFound(_) => PyFileNotFoundError::new_err(error.to_string()),
        Error::InvalidKey(_) => PyValueError::new_err(error.to_string()),
        _ => PyRuntimeError::new_err(error.to_string()),
    }
}
#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyDirDb>()?;
    Ok(())
}
