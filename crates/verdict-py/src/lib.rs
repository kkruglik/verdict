use std::path::Path;

use pyo3::prelude::*;
use verdict_core::{
    csv_loader::DatasetCsvExt,
    dataset::{
        BoolColumn, Column, DataType, Dataset, DateColumn, DateTimeColumn, Field, FloatColumn,
        InSetValues, IntColumn, Schema, StrColumn,
    },
    rules::{
        Constraint, Operand, Rule, RuleBuilder, ValidateConfig, ValidationReport, ValidationResult,
        validate,
    },
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

#[pyfunction(name = "col")]
fn py_col(name: &str) -> PyOperand {
    PyOperand {
        inner: Operand::Column(name.to_string()),
    }
}
fn extract_operand(py: Python<'_>, operand: &Py<PyAny>) -> PyResult<Operand> {
    if let Ok(s) = operand.extract::<String>(py) {
        Ok(Operand::Str(s))
    } else if let Ok(f) = operand.extract::<f64>(py) {
        Ok(Operand::Num(f))
    } else if let Ok(op) = operand.extract::<PyRef<PyOperand>>(py) {
        Ok(op.inner.clone())
    } else {
        Err(pyo3::exceptions::PyTypeError::new_err(
            "Expected float or column name string.",
        ))
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

    #[staticmethod]
    fn date() -> Self {
        PyDataType {
            inner: DataType::Date,
        }
    }

    #[staticmethod]
    fn datetime() -> Self {
        PyDataType {
            inner: DataType::DateTime,
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
                format: None,
            })
            .collect();
        PySchema {
            inner: Schema {
                fields: core_fields,
            },
        }
    }
}

#[pyclass(name = "Operand")]
struct PyOperand {
    inner: Operand,
}

#[pyclass(name = "Column")]
struct PyColumn {
    inner: Column,
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

    #[staticmethod]
    fn date(values: Vec<Option<i32>>) -> PyColumn {
        PyColumn {
            inner: Column::Date(DateColumn(values)),
        }
    }

    #[staticmethod]
    fn datetime(values: Vec<Option<i64>>) -> PyColumn {
        PyColumn {
            inner: Column::DateTime(DateTimeColumn(values)),
        }
    }

    // NOTE: column ops commented out — unclear if needed in a declarative validation library.
    // No competitor (Pandera, Great Expectations) exposes per-column ops.
    // Keeping for reference, revisit before public API release.

    // fn len(&self) -> usize { self.inner.len() }
    // fn is_null(&self) -> Vec<bool> { self.inner.is_null() }
    // fn is_empty(&self) -> bool { self.inner.is_empty() }
    // fn null_count(&self) -> usize { self.inner.null_count() }
    // fn not_null_count(&self) -> usize { self.inner.not_null_count() }
    // fn unique_count(&self) -> usize { self.inner.unique_count() }
    // fn duplicates_count(&self) -> usize { self.inner.duplicates_count() }
    // fn sum(&self) -> Option<f64> { self.inner.sum() }
    // fn mean(&self) -> Option<f64> { self.inner.mean() }
    // fn min(&self) -> Option<f64> { self.inner.min() }
    // fn max(&self) -> Option<f64> { self.inner.max() }
    // fn std(&self) -> Option<f64> { self.inner.std() }
    // fn median(&self) -> Option<f64> { self.inner.median() }
    // fn gt(&self, py: Python<'_>, compare: Py<PyAny>) -> PyResult<Vec<Option<bool>>> { ... }
    // fn ge(&self, compare: f64) -> Vec<Option<bool>> { self.inner.ge(compare) }
    // fn lt(&self, compare: f64) -> Vec<Option<bool>> { self.inner.lt(compare) }
    // fn le(&self, compare: f64) -> Vec<Option<bool>> { self.inner.le(compare) }
    // fn equal(&self, py: Python<'_>, compare: Py<PyAny>) -> Vec<Option<bool>> { ... }
    // fn between(&self, lower: f64, upper: f64) -> Vec<Option<bool>> { self.inner.between(lower, upper) }
    // fn contains(&self, pat: &str) -> Vec<Option<bool>> { self.inner.contains(pat) }
    // fn starts_with(&self, pat: &str) -> Vec<Option<bool>> { self.inner.starts_with(pat) }
    // fn ends_with(&self, pat: &str) -> Vec<Option<bool>> { self.inner.ends_with(pat) }
    // fn matches_regex(&self, pat: &str) -> Vec<Option<bool>> { self.inner.matches_regex(pat) }
    // fn str_length(&self) -> Vec<Option<usize>> { self.inner.length() }
    // fn is_in(&self, py: Python<'_>, values: Vec<Py<PyAny>>) -> Vec<Option<bool>> { ... }

    fn __repr__(&self) -> String {
        let (dtype, values) = match &self.inner {
            Column::Int(col) => ("i64", format_values(&col.0, |v: &i64| v.to_string())),
            Column::Float(col) => ("f64", format_values(&col.0, |v: &f64| v.to_string())),
            Column::Str(col) => (
                "str",
                format_values(&col.0, |v: &String| format!("\"{}\"", v)),
            ),
            Column::Bool(col) => ("bool", format_values(&col.0, |v: &bool| v.to_string())),
            Column::Date(col) => ("date", format_values(&col.0, |v: &i32| v.to_string())),
            Column::DateTime(col) => ("datetime", format_values(&col.0, |v: &i64| v.to_string())),
        };
        format!("[{}]: [{}]", dtype, values)
    }
}

#[pyclass(name = "Dataset")]
struct PyDataset {
    inner: Dataset,
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
        let inner = Dataset::from_csv(Path::new(path), &schema.inner)
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
            inner: Constraint::GreaterThan(Operand::Num(compare)),
        }
    }

    #[staticmethod]
    fn ge(compare: f64) -> Self {
        PyConstraint {
            inner: Constraint::GreaterThanOrEqual(Operand::Num(compare)),
        }
    }

    #[staticmethod]
    fn lt(compare: f64) -> Self {
        PyConstraint {
            inner: Constraint::LessThan(Operand::Num(compare)),
        }
    }

    #[staticmethod]
    fn le(compare: f64) -> Self {
        PyConstraint {
            inner: Constraint::LessThanOrEqual(Operand::Num(compare)),
        }
    }

    #[staticmethod]
    fn eq(compare: f64) -> Self {
        PyConstraint {
            inner: Constraint::Equal(Operand::Num(compare)),
        }
    }

    #[staticmethod]
    fn between(min: f64, max: f64) -> Self {
        PyConstraint {
            inner: Constraint::Between {
                min: min.into(),
                max: max.into(),
            },
        }
    }

    #[staticmethod]
    fn is_in(py: Python<'_>, values: Vec<Py<PyAny>>) -> PyResult<Self> {
        let set = if let Ok(v) = values
            .iter()
            .map(|v| v.extract::<i64>(py))
            .collect::<PyResult<Vec<_>>>()
        {
            InSetValues::Int64Set(v)
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

    #[staticmethod]
    fn after(date: String) -> Self {
        PyConstraint {
            inner: Constraint::After(date),
        }
    }

    #[staticmethod]
    fn before(date: String) -> Self {
        PyConstraint {
            inner: Constraint::Before(date),
        }
    }

    #[staticmethod]
    fn between_dates(min: String, max: String) -> Self {
        PyConstraint {
            inner: Constraint::BetweenDates { min, max },
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

#[pyclass(name = "RuleBuilder")]
struct PyRuleBuilder {
    inner: RuleBuilder,
}

#[pymethods]
impl PyRuleBuilder {
    #[new]
    fn new(column: String) -> Self {
        PyRuleBuilder {
            inner: RuleBuilder {
                column,
                constraint: vec![],
            },
        }
    }

    fn not_null(slf: Bound<'_, Self>) -> Bound<'_, Self> {
        let old = std::mem::take(&mut slf.borrow_mut().inner);
        slf.borrow_mut().inner = old.not_null();
        slf
    }

    fn unique(slf: Bound<'_, Self>) -> Bound<'_, Self> {
        let old = std::mem::take(&mut slf.borrow_mut().inner);
        slf.borrow_mut().inner = old.unique();
        slf
    }

    fn gt<'py>(
        slf: Bound<'py, Self>,
        py: Python<'_>,
        compare: Py<PyAny>,
    ) -> PyResult<Bound<'py, Self>> {
        let op = extract_operand(py, &compare)?;
        let old = std::mem::take(&mut slf.borrow_mut().inner);
        slf.borrow_mut().inner = old.gt(op);
        Ok(slf)
    }

    fn ge<'py>(
        slf: Bound<'py, Self>,
        py: Python<'_>,
        compare: Py<PyAny>,
    ) -> PyResult<Bound<'py, Self>> {
        let op = extract_operand(py, &compare)?;
        let old = std::mem::take(&mut slf.borrow_mut().inner);
        slf.borrow_mut().inner = old.ge(op);
        Ok(slf)
    }

    fn lt<'py>(
        slf: Bound<'py, Self>,
        py: Python<'_>,
        compare: Py<PyAny>,
    ) -> PyResult<Bound<'py, Self>> {
        let op = extract_operand(py, &compare)?;
        let old = std::mem::take(&mut slf.borrow_mut().inner);
        slf.borrow_mut().inner = old.lt(op);
        Ok(slf)
    }

    fn le<'py>(
        slf: Bound<'py, Self>,
        py: Python<'_>,
        compare: Py<PyAny>,
    ) -> PyResult<Bound<'py, Self>> {
        let op = extract_operand(py, &compare)?;
        let old = std::mem::take(&mut slf.borrow_mut().inner);
        slf.borrow_mut().inner = old.le(op);
        Ok(slf)
    }

    fn equal<'py>(
        slf: Bound<'py, Self>,
        py: Python<'_>,
        compare: Py<PyAny>,
    ) -> PyResult<Bound<'py, Self>> {
        let op = extract_operand(py, &compare)?;
        let old = std::mem::take(&mut slf.borrow_mut().inner);
        slf.borrow_mut().inner = old.equal(op);
        Ok(slf)
    }

    fn between<'py>(
        slf: Bound<'py, Self>,
        py: Python<'_>,
        min: Py<PyAny>,
        max: Py<PyAny>,
    ) -> PyResult<Bound<'py, Self>> {
        let min_op = extract_operand(py, &min)?;
        let max_op = extract_operand(py, &max)?;
        let old = std::mem::take(&mut slf.borrow_mut().inner);
        slf.borrow_mut().inner = old.between(min_op, max_op);
        Ok(slf)
    }

    fn is_in<'py>(
        slf: Bound<'py, Self>,
        py: Python<'_>,
        values: Vec<Py<PyAny>>,
    ) -> PyResult<Bound<'py, Self>> {
        let set = if let Ok(v) = values
            .iter()
            .map(|v| v.extract::<i64>(py))
            .collect::<PyResult<Vec<_>>>()
        {
            InSetValues::Int64Set(v)
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
                "in_set values must be all integers, floats, or strings",
            ));
        };
        let old = std::mem::take(&mut slf.borrow_mut().inner);
        slf.borrow_mut().inner = old.in_set(set);
        Ok(slf)
    }

    fn matches_regex(slf: Bound<'_, Self>, pattern: String) -> Bound<'_, Self> {
        let old = std::mem::take(&mut slf.borrow_mut().inner);
        slf.borrow_mut().inner = old.matches_regex(&pattern);
        slf
    }

    fn contains(slf: Bound<'_, Self>, pattern: String) -> Bound<'_, Self> {
        let old = std::mem::take(&mut slf.borrow_mut().inner);
        slf.borrow_mut().inner = old.contains(&pattern);
        slf
    }

    fn starts_with(slf: Bound<'_, Self>, pattern: String) -> Bound<'_, Self> {
        let old = std::mem::take(&mut slf.borrow_mut().inner);
        slf.borrow_mut().inner = old.starts_with(&pattern);
        slf
    }

    fn ends_with(slf: Bound<'_, Self>, pattern: String) -> Bound<'_, Self> {
        let old = std::mem::take(&mut slf.borrow_mut().inner);
        slf.borrow_mut().inner = old.ends_with(&pattern);
        slf
    }

    fn length_between(slf: Bound<'_, Self>, min: usize, max: usize) -> Bound<'_, Self> {
        let old = std::mem::take(&mut slf.borrow_mut().inner);
        slf.borrow_mut().inner = old.length_between(min, max);
        slf
    }

    fn build(slf: PyRef<'_, Self>) -> Vec<PyRule> {
        slf.inner
            .constraint
            .iter()
            .map(|c| PyRule {
                inner: Rule {
                    column: slf.inner.column.clone(),
                    constraint: c.clone(),
                },
            })
            .collect()
    }
}

#[pyclass(name = "ValidationResult")]
struct PyValidationResult {
    inner: ValidationResult,
}

#[pyclass(name = "ValidationReport")]
struct PyValidationReport {
    inner: ValidationReport,
}

#[pymethods]
impl PyValidationReport {
    #[getter]
    fn results(&self) -> Vec<PyValidationResult> {
        self.inner
            .results
            .iter()
            .map(|res| PyValidationResult { inner: res.clone() })
            .collect()
    }

    #[getter]
    fn passed(&self) -> bool {
        self.inner.passed
    }

    #[getter]
    fn total_rules(&self) -> usize {
        self.inner.total_rules
    }

    #[getter]
    fn passed_count(&self) -> usize {
        self.inner.passed_count
    }

    #[getter]
    fn failed_count(&self) -> usize {
        self.inner.failed_count
    }
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

    #[getter]
    fn failed_values(&self) -> Option<Vec<(usize, String)>> {
        self.inner.failed_values.clone()
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
) -> PyResult<PyValidationReport> {
    let core_rules: Vec<Rule> = rules
        .into_iter()
        .map(|v| v.borrow(py).inner.clone())
        .collect();

    let results = validate(
        &data.borrow(py).inner,
        &core_rules,
        ValidateConfig::default(),
    );
    let report = PyValidationReport { inner: results };
    Ok(report)
}

#[pymodule]
fn verdict_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDataset>()?;
    m.add_class::<PyColumn>()?;
    m.add_class::<PyConstraint>()?;
    m.add_class::<PyRule>()?;
    m.add_class::<PyRuleBuilder>()?;
    m.add_class::<PyValidationResult>()?;
    m.add_class::<PyValidationReport>()?;
    m.add_class::<PySchema>()?;
    m.add_class::<PyDataType>()?;
    m.add_function(wrap_pyfunction!(py_validate, m)?)?;
    m.add_function(wrap_pyfunction!(py_col, m)?)?;
    Ok(())
}
