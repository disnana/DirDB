use dirdb_core::{DirDb, Error};
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
    fn new(root: String) -> PyResult<Self> {
        Ok(Self {
            inner: DirDb::open(root).map_err(to_py_error)?,
        })
    }
    fn get(&self, py: Python<'_>, key: String) -> PyResult<PyObject> {
        let inner = self.inner.clone();
        let entry = py.allow_threads(|| inner.get(&key)).map_err(to_py_error)?;
        value_to_py(py, &entry.value)
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
