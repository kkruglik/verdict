use pyo3::prelude::*;
use verdict_core::dataset::{
    BoolColumn, Column, Dataset, FloatColumn, InSetValues, IntColumn, StrColumn,
};

fn format_values<T>(values: &[Option<T>], fmt: impl Fn(&T) -> String) -> String {
    let max_display = 10;
    let len = values.len();
    let items: Vec<String> = values
        .iter()
        .take(max_display)
        .map(|v| match v {
            Some(val) => fmt(val),
            None => "null".to_string(),
        })
        .collect();
    if len > max_display {
        format!("{}, ... ({} total)", items.join(", "), len)
    } else {
        items.join(", ")
    }
}

#[pyclass]
struct PyColumn {
    inner: Column,
}

#[pyclass]
struct PyDataset {
    inner: Dataset,
}


#[pymethods]
impl PyColumn {
    #[staticmethod]
    fn integer(values: Vec<Option<i64>>) -> PyColumn {
        let column = IntColumn(values);
        PyColumn {
            inner: Column::Int(column),
        }
    }

    #[staticmethod]
    fn floating(values: Vec<Option<f64>>) -> PyColumn {
        let column = FloatColumn(values);
        PyColumn {
            inner: Column::Float(column),
        }
    }

    #[staticmethod]
    fn boolean(values: Vec<Option<bool>>) -> PyColumn {
        let column = BoolColumn(values);
        PyColumn {
            inner: Column::Bool(column),
        }
    }

    #[staticmethod]
    fn string(values: Vec<Option<String>>) -> PyColumn {
        let column = StrColumn(values);
        PyColumn {
            inner: Column::Str(column),
        }
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_null(&self) -> Vec<bool> {
        self.inner.is_null()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn null_count(&self) -> usize {
        self.inner.null_count()
    }

    fn not_null_count(&self) -> usize {
        self.inner.not_null_count()
    }

    fn unique_count(&self) -> usize {
        self.inner.unique_count()
    }

    fn duplicates_count(&self) -> usize {
        self.inner.duplicates_count()
    }

    fn sum(&self) -> Option<f64> {
        self.inner.sum()
    }

    fn mean(&self) -> Option<f64> {
        self.inner.mean()
    }

    fn min(&self) -> Option<f64> {
        self.inner.min()
    }

    fn max(&self) -> Option<f64> {
        self.inner.max()
    }

    fn std(&self) -> Option<f64> {
        self.inner.std()
    }

    fn median(&self) -> Option<f64> {
        self.inner.median()
    }

    fn gt(&self, compare: f64) -> Vec<Option<bool>> {
        self.inner.gt(compare)
    }
    fn ge(&self, compare: f64) -> Vec<Option<bool>> {
        self.inner.ge(compare)
    }

    fn lt(&self, compare: f64) -> Vec<Option<bool>> {
        self.inner.lt(compare)
    }

    fn le(&self, compare: f64) -> Vec<Option<bool>> {
        self.inner.le(compare)
    }

    fn equal(&self, compare: f64) -> Vec<Option<bool>> {
        self.inner.equal(compare)
    }

    fn equal_str(&self, compare: &str) -> Vec<Option<bool>> {
        self.inner.equal_str(compare)
    }

    fn between(&self, lower: f64, upper: f64) -> Vec<Option<bool>> {
        self.inner.between(lower, upper)
    }

    fn contains(&self, pat: &str) -> Vec<Option<bool>> {
        self.inner.contains(pat)
    }

    fn starts_with(&self, pat: &str) -> Vec<Option<bool>> {
        self.inner.starts_with(pat)
    }

    fn ends_with(&self, pat: &str) -> Vec<Option<bool>> {
        self.inner.ends_with(pat)
    }

    fn matches_regex(&self, pat: &str) -> Vec<Option<bool>> {
        self.inner.matches_regex(pat)
    }

    fn str_length(&self) -> Vec<Option<usize>> {
        self.inner.str_length()
    }

    fn is_in(&self, py: Python<'_>, values: Vec<PyObject>) -> Vec<Option<bool>> {
        let set = if let Ok(v) = values
            .iter()
            .map(|v| v.extract::<i64>(py))
            .collect::<PyResult<Vec<_>>>()
        {
            InSetValues::IntSet(v)
        } else if let Ok(v) = values
            .iter()
            .map(|v| v.extract::<f64>(py))
            .collect::<PyResult<Vec<_>>>()
        {
            InSetValues::FloatSet(v)
        } else if let Ok(v) = values
            .iter()
            .map(|v| v.extract::<String>(py))
            .collect::<PyResult<Vec<_>>>()
        {
            InSetValues::StrSet(v)
        } else {
            return vec![None; self.inner.len()];
        };
        self.inner.is_in(&set)
    }

    fn __repr__(&self) -> String {
        let (dtype, values) = match &self.inner {
            Column::Int(col) => ("i64", format_values(&col.0, |v: &i64| v.to_string())),
            Column::Float(col) => ("f64", format_values(&col.0, |v: &f64| v.to_string())),
            Column::Str(col) => (
                "str",
                format_values(&col.0, |v: &String| format!("\"{}\"", v)),
            ),
            Column::Bool(col) => ("bool", format_values(&col.0, |v: &bool| v.to_string())),
        };
        format!("[{}]: [{}]", dtype, values)
    }
}

#[pymethods]
impl PyDataset {
    #[new]
    fn new(py: Python<'_>, headers: Vec<String>, columns: Vec<Py<PyColumn>>) -> Self {
        let core_columns = columns
            .into_iter()
            .map(|col| col.borrow(py).inner.clone())
            .collect();
        PyDataset {
            inner: Dataset {
                headers,
                columns: core_columns,
            },
        }
    }

    fn shape(&self) -> (usize, usize) {
        self.inner.shape()
    }

    fn get_column_by_name(&self, name: &str) -> Option<PyColumn> {
        self.inner
            .get_column_by_name(name)
            .map(|col| PyColumn { inner: col.clone() })
    }

    fn get_column_by_index(&self, idx: usize) -> Option<PyColumn> {
        self.inner
            .get_column_by_index(idx)
            .map(|col| PyColumn { inner: col.clone() })
    }

    fn get_column_index(&self, name: &str) -> Option<usize> {
        self.inner.get_column_index(name)
    }

    fn __repr__(&self) -> String {
        let (rows, cols) = self.inner.shape();
        format!("Dataset(rows={}, cols={})", rows, cols)
    }
}

#[pymodule]
fn verdict_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDataset>()?;
    m.add_class::<PyColumn>()?;
    Ok(())
}
