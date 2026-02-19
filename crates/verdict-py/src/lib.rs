use pyo3::prelude::*;
use verdict_core::{
    csv_loader::DatasetCsvExt,
    dataset::{
        BoolColumn, Column, DataType, Dataset, Field, FloatColumn, InSetValues, IntColumn, Schema,
        StrColumn,
    },
    rules::{Constraint, Rule, ValidationResult, validate},
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

#[pyclass(name = "Column")]
struct PyColumn {
    inner: Column,
}

#[pyclass(name = "Dataset")]
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

    fn equal(&self, py: Python<'_>, compare: Py<PyAny>) -> Vec<Option<bool>> {
        if let Ok(v) = compare.extract::<String>(py) {
            self.inner.equal_str(&v)
        } else if let Ok(v) = compare.extract::<f64>(py) {
            self.inner.equal(v)
        } else {
            vec![None; self.inner.len()]
        }
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

    fn is_in(&self, py: Python<'_>, values: Vec<Py<PyAny>>) -> Vec<Option<bool>> {
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

#[pyclass(name = "DataType")]
struct PyDataType {
    inner: DataType,
}

#[pymethods]
impl PyDataType {
    #[staticmethod]
    fn integer() -> Self {
        PyDataType {
            inner: DataType::Int,
        }
    }

    #[staticmethod]
    fn float() -> Self {
        PyDataType {
            inner: DataType::Float,
        }
    }

    #[staticmethod]
    fn string() -> Self {
        PyDataType {
            inner: DataType::Str,
        }
    }

    #[staticmethod]
    fn boolean() -> Self {
        PyDataType {
            inner: DataType::Bool,
        }
    }
}

#[pyclass(name = "Schema")]
struct PySchema {
    inner: Schema,
}

#[pymethods]
impl PySchema {
    #[new]
    fn new(py: Python<'_>, fields: Vec<(String, Py<PyDataType>)>) -> Self {
        let core_fields = fields
            .iter()
            .map(|(name, dtype)| Field {
                name: name.clone(),
                dtype: dtype.borrow(py).inner.clone(),
            })
            .collect();
        PySchema {
            inner: Schema {
                fields: core_fields,
            },
        }
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

    #[staticmethod]
    fn from_csv(path: &str, schema: &PySchema) -> PyResult<Self> {
        let inner = Dataset::from_csv(path, &schema.inner)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(PyDataset { inner })
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

#[pyclass(name = "Constraint")]
struct PyConstraint {
    inner: Constraint,
}

#[pymethods]
impl PyConstraint {
    #[staticmethod]
    fn not_null() -> Self {
        PyConstraint {
            inner: Constraint::NotNull,
        }
    }

    #[staticmethod]
    fn unique() -> Self {
        PyConstraint {
            inner: Constraint::Unique,
        }
    }

    #[staticmethod]
    fn gt(compare: f64) -> Self {
        PyConstraint {
            inner: Constraint::GreaterThan(compare),
        }
    }

    #[staticmethod]
    fn ge(compare: f64) -> Self {
        PyConstraint {
            inner: Constraint::GreaterThanOrEqual(compare),
        }
    }

    #[staticmethod]
    fn lt(compare: f64) -> Self {
        PyConstraint {
            inner: Constraint::LessThan(compare),
        }
    }

    #[staticmethod]
    fn le(compare: f64) -> Self {
        PyConstraint {
            inner: Constraint::LessThanOrEqual(compare),
        }
    }

    #[staticmethod]
    fn eq(compare: f64) -> Self {
        PyConstraint {
            inner: Constraint::Equal(compare),
        }
    }

    #[staticmethod]
    fn between(min: f64, max: f64) -> Self {
        PyConstraint {
            inner: Constraint::Between { min, max },
        }
    }

    #[staticmethod]
    fn is_in(py: Python<'_>, values: Vec<Py<PyAny>>) -> PyResult<Self> {
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
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "is_in values must be all integers, floats, or strings",
            ));
        };
        Ok(PyConstraint {
            inner: Constraint::InSet(set),
        })
    }

    #[staticmethod]
    fn matches_regex(pattern: String) -> Self {
        PyConstraint {
            inner: Constraint::MatchesRegex(pattern),
        }
    }

    #[staticmethod]
    fn contains(pattern: String) -> Self {
        PyConstraint {
            inner: Constraint::Contains(pattern),
        }
    }

    #[staticmethod]
    fn starts_with(pattern: String) -> Self {
        PyConstraint {
            inner: Constraint::StartsWith(pattern),
        }
    }

    #[staticmethod]
    fn ends_with(pattern: String) -> Self {
        PyConstraint {
            inner: Constraint::EndsWith(pattern),
        }
    }

    #[staticmethod]
    fn length_between(min: usize, max: usize) -> Self {
        PyConstraint {
            inner: Constraint::LengthBetween { min, max },
        }
    }
}

#[pyclass(name = "Rule")]
struct PyRule {
    inner: Rule,
}

#[pymethods]
impl PyRule {
    #[new]
    fn new(py: Python<'_>, column: String, constraint: Py<PyConstraint>) -> Self {
        PyRule {
            inner: Rule {
                column,
                constraint: constraint.borrow(py).inner.clone(),
            },
        }
    }
}

#[pyclass(name = "ValidationResult")]
struct PyValidationResult {
    inner: ValidationResult,
}

#[pymethods]
impl PyValidationResult {
    #[getter]
    fn column(&self) -> &str {
        &self.inner.column
    }

    #[getter]
    fn constraint(&self) -> &str {
        &self.inner.constraint
    }

    #[getter]
    fn is_passed(&self) -> bool {
        self.inner.passed
    }

    #[getter]
    fn failed_count(&self) -> usize {
        self.inner.failed_count
    }

    #[getter]
    fn error(&self) -> Option<&str> {
        self.inner.error.as_deref()
    }

    fn __repr__(&self) -> String {
        self.inner.to_string()
    }
}

#[pyfunction]
fn py_validate(
    py: Python<'_>,
    data: Py<PyDataset>,
    rules: Vec<Py<PyRule>>,
) -> PyResult<Vec<PyValidationResult>> {
    let core_rules: Vec<Rule> = rules
        .into_iter()
        .map(|v| v.borrow(py).inner.clone())
        .collect();

    let results = validate(&data.borrow(py).inner, &core_rules)
        .into_iter()
        .map(|r| PyValidationResult { inner: r })
        .collect();
    Ok(results)
}

#[pymodule]
fn verdict_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDataset>()?;
    m.add_class::<PyColumn>()?;
    m.add_class::<PyConstraint>()?;
    m.add_class::<PyRule>()?;
    m.add_class::<PyValidationResult>()?;
    m.add_class::<PySchema>()?;
    m.add_class::<PyDataType>()?;
    m.add_function(wrap_pyfunction!(py_validate, m)?)?;
    Ok(())
}
