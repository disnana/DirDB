use std::time::Duration;

use dirdb_core::{DirDb, Error, Options};
use pyo3::{
    exceptions::{PyFileNotFoundError, PyRuntimeError, PyValueError},
    prelude::*,
    types::{PyAny, PyDict, PyList, PyTuple},
};

#[pyclass(name = "NativeDirDB")]
struct PyDirDb {
    inner: DirDb,
}

#[pymethods]
impl PyDirDb {
    #[new]
    #[pyo3(signature = (root, cache_max_items=10_000, auto_reload=true, debounce_ms=100, verify_interval_seconds=Some(60)))]
    fn new(
        root: String,
        cache_max_items: usize,
        auto_reload: bool,
        debounce_ms: u64,
        verify_interval_seconds: Option<u64>,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: DirDb::open_with_options(
                root,
                Options {
                    cache_max_items,
                    auto_reload,
                    debounce: Duration::from_millis(debounce_ms),
                    verify_interval: verify_interval_seconds.map(Duration::from_secs),
                },
            )
            .map_err(to_py_error)?,
        })
    }
    fn get(&self, py: Python<'_>, key: String) -> PyResult<PyObject> {
        let inner = self.inner.clone();
        let entry = py.allow_threads(|| inner.get(&key)).map_err(to_py_error)?;
        value_to_py(py, &entry.value)
    }
    fn get_many(&self, py: Python<'_>, keys: Vec<String>) -> PyResult<Vec<PyObject>> {
        let inner = self.inner.clone();
        let entries = py.allow_threads(|| inner.get_many(&keys));
        entries
            .into_iter()
            .map(|entry| {
                let entry = entry.map_err(to_py_error)?;
                value_to_py(py, &entry.value)
            })
            .collect()
    }
    #[pyo3(signature = (key, value, expected_version=None))]
    fn set(
        &self,
        py: Python<'_>,
        key: String,
        value: &Bound<'_, PyAny>,
        expected_version: Option<u64>,
    ) -> PyResult<u64> {
        let value = value_from_py(value)?;
        let inner = self.inner.clone();
        Ok(py
            .allow_threads(|| inner.set(&key, &value, expected_version))
            .map_err(to_py_error)?
            .version)
    }
    fn set_many(&self, py: Python<'_>, values: &Bound<'_, PyDict>) -> PyResult<Vec<u64>> {
        let items = values
            .iter()
            .map(|(key, value)| Ok((key.extract::<String>()?, value_from_py(&value)?)))
            .collect::<PyResult<Vec<_>>>()?;
        let inner = self.inner.clone();
        py.allow_threads(|| inner.set_many(&items))
            .into_iter()
            .map(|entry| entry.map(|entry| entry.version).map_err(to_py_error))
            .collect()
    }
    #[pyo3(signature = (key, expected_version=None))]
    fn delete(&self, py: Python<'_>, key: String, expected_version: Option<u64>) -> PyResult<()> {
        let inner = self.inner.clone();
        py.allow_threads(|| inner.delete(&key, expected_version))
            .map_err(to_py_error)
    }
    fn exists(&self, py: Python<'_>, key: String) -> PyResult<bool> {
        let inner = self.inner.clone();
        py.allow_threads(|| inner.exists(&key)).map_err(to_py_error)
    }
    #[pyo3(signature = (prefix=""))]
    fn list(&self, py: Python<'_>, prefix: &str) -> PyResult<Vec<String>> {
        let inner = self.inner.clone();
        let prefix = prefix.to_owned();
        py.allow_threads(|| inner.list(&prefix))
            .map_err(to_py_error)
    }
    fn rebuild_index(&self, py: Python<'_>) -> PyResult<usize> {
        let inner = self.inner.clone();
        py.allow_threads(|| inner.rebuild_index())
            .map_err(to_py_error)
    }
    fn cache_stats(&self) -> (u64, u64, usize) {
        let stats = self.inner.cache_stats();
        (stats.hits, stats.misses, stats.entries)
    }
    fn stat(&self, key: String) -> PyResult<(bool, u64, Option<String>)> {
        let status = self.inner.stat(&key).map_err(to_py_error)?;
        Ok((
            status.file_valid,
            status.current_version,
            status.last_reload_error,
        ))
    }
}
fn to_py_error(error: Error) -> PyErr {
    match error {
        Error::NotFound(_) => PyFileNotFoundError::new_err(error.to_string()),
        Error::InvalidKey(_) => PyValueError::new_err(error.to_string()),
        _ => PyRuntimeError::new_err(error.to_string()),
    }
}

fn value_to_py(py: Python<'_>, value: &serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(value) => {
            Ok((*value).into_pyobject(py)?.to_owned().unbind().into())
        }
        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                Ok(value.into_pyobject(py)?.unbind().into())
            } else if let Some(value) = value.as_u64() {
                Ok(value.into_pyobject(py)?.unbind().into())
            } else {
                Ok(value
                    .as_f64()
                    .expect("valid JSON number")
                    .into_pyobject(py)?
                    .unbind()
                    .into())
            }
        }
        serde_json::Value::String(value) => Ok(value.as_str().into_pyobject(py)?.unbind().into()),
        serde_json::Value::Array(values) => {
            let output = PyList::empty(py);
            for value in values {
                output.append(value_to_py(py, value)?)?;
            }
            Ok(output.into_any().unbind())
        }
        serde_json::Value::Object(values) => {
            let output = PyDict::new(py);
            for (key, value) in values {
                output.set_item(key, value_to_py(py, value)?)?;
            }
            Ok(output.into_any().unbind())
        }
    }
}

fn value_from_py(value: &Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
    if value.is_none() {
        return Ok(serde_json::Value::Null);
    }
    if let Ok(value) = value.extract::<bool>() {
        return Ok(serde_json::Value::Bool(value));
    }
    if let Ok(value) = value.extract::<i64>() {
        return Ok(serde_json::Value::Number(value.into()));
    }
    if let Ok(value) = value.extract::<u64>() {
        return Ok(serde_json::Value::Number(value.into()));
    }
    if let Ok(value) = value.extract::<f64>() {
        let number = serde_json::Number::from_f64(value)
            .ok_or_else(|| PyValueError::new_err("NaN and infinity are not valid JSON values"))?;
        return Ok(serde_json::Value::Number(number));
    }
    if let Ok(value) = value.extract::<String>() {
        return Ok(serde_json::Value::String(value));
    }
    if let Ok(values) = value.downcast::<PyList>() {
        return values
            .iter()
            .map(|value| value_from_py(&value))
            .collect::<PyResult<Vec<_>>>()
            .map(serde_json::Value::Array);
    }
    if let Ok(values) = value.downcast::<PyTuple>() {
        return values
            .iter()
            .map(|value| value_from_py(&value))
            .collect::<PyResult<Vec<_>>>()
            .map(serde_json::Value::Array);
    }
    if let Ok(values) = value.downcast::<PyDict>() {
        let mut output = serde_json::Map::new();
        for (key, value) in values.iter() {
            output.insert(key.extract::<String>()?, value_from_py(&value)?);
        }
        return Ok(serde_json::Value::Object(output));
    }
    Err(PyValueError::new_err(
        "DirDB values must contain JSON-compatible dict, list, tuple, string, number, bool, or None values",
    ))
}
#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyDirDb>()?;
    Ok(())
}
