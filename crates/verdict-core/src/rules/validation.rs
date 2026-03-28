use std::{fmt::Display, str::FromStr};

use chrono::{NaiveDate, NaiveDateTime};

use crate::{
    dataframe::{
        Column, DataFrame, KeepStrategy, ValuesSet, i32_to_naive_date, i64_to_naive_datetime,
        naive_date_to_i32, naive_datetime_to_i64,
        ops::{ComparableOps, StringOps},
    },
    errors::ValidationError,
    rules::{ColumnConstraint, ColumnRule, Operand},
};

pub struct ValidatingConfig {
    pub max_failed_samples: usize,
}

impl Default for ValidatingConfig {
    fn default() -> Self {
        Self {
            max_failed_samples: 100,
        }
    }
}

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub column: String,
    pub constraint: String,
    pub passed: bool,
    pub failed_count: usize,
    pub error: Option<String>,
    pub failed_values: Option<Vec<(usize, String)>>,
}

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub passed: bool,
    pub total_rules: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub results: Vec<ValidationResult>,
}

impl ValidationReport {
    #[cfg(feature = "json")]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

impl Display for ValidationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.passed {
            writeln!(
                f,
                "Validation Report: PASSED ({}/{} rules passed)",
                self.passed_count, self.total_rules
            )?;
        } else {
            writeln!(
                f,
                "Validation Report: FAILED ({}/{} rules passed)",
                self.passed_count, self.total_rules
            )?;
            for result in self.results.iter().filter(|r| !r.passed) {
                writeln!(
                    f,
                    "  FAIL: column '{}' — {} — {} values failed: {}",
                    result.column,
                    result.constraint,
                    result.failed_count,
                    result.error.as_deref().unwrap_or("unknown error")
                )?;
                if let Some(values) = &result.failed_values {
                    for (idx, val) in values {
                        writeln!(f, "    row {}: {}", idx, val)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl ValidationResult {
    pub fn passed(rule: &ColumnRule) -> Self {
        ValidationResult {
            column: rule.column.clone(),
            constraint: format!("{}", rule.constraint),
            passed: true,
            failed_count: 0,
            error: None,
            failed_values: None,
        }
    }

    pub fn failed(
        rule: &ColumnRule,
        failed_count: usize,
        error: &str,
        failed_values: Option<Vec<(usize, String)>>,
    ) -> Self {
        ValidationResult {
            column: rule.column.clone(),
            constraint: format!("{}", rule.constraint),
            passed: false,
            failed_count,
            error: Some(error.to_string()),
            failed_values,
        }
    }
}

impl std::fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.passed {
            write!(f, "PASS: column '{}' — {}", self.column, self.constraint)
        } else {
            write!(
                f,
                "FAIL: column '{}' — {} — {} values failed: {}",
                self.column,
                self.constraint,
                self.failed_count,
                self.error.as_deref().unwrap_or("unknown error")
            )
        }
    }
}

pub fn validate(
    data: &DataFrame,
    rules: &[ColumnRule],
    config: ValidatingConfig,
) -> ValidationReport {
    let results: Vec<ValidationResult> = rules
        .iter()
        .map(|rule| {
            validate_with_rule(data, rule, &config)
                .unwrap_or_else(|e| ValidationResult::failed(rule, 0, &e.to_string(), None))
        })
        .collect();

    let passed_count = results.iter().map(|r| if r.passed { 1 } else { 0 }).sum();
    let total_rules = results.len();
    let failed_count = total_rules - passed_count;
    let passed = failed_count == 0;
    ValidationReport {
        passed,
        passed_count,
        total_rules,
        results,
        failed_count,
    }
}

fn validate_with_rule(
    data: &DataFrame,
    rule: &ColumnRule,
    config: &ValidatingConfig,
) -> Result<ValidationResult, ValidationError> {
    if let Some(column) = data.get_column_by_name(&rule.column) {
        let n = config.max_failed_samples;
        match &rule.constraint {
            ColumnConstraint::NotNull => Ok(check_not_null(column, rule, n)),
            ColumnConstraint::GreaterThan(operand) => match operand {
                Operand::Num(v) => Ok(check_greater_than_num(column, *v, rule, n)),
                Operand::Column(name) => {
                    if let Some(other) = data.get_column_by_name(name) {
                        Ok(check_greater_than_col(column, other, name, rule, n))
                    } else {
                        Err(ValidationError::ColumnNotFound {
                            name: name.to_string(),
                        })
                    }
                }
                Operand::Str(v) => Ok(check_greater_than_str(column, v, rule, n)),
            },
            ColumnConstraint::GreaterThanOrEqual(operand) => match operand {
                Operand::Num(v) => Ok(check_greater_than_or_equal_num(column, *v, rule, n)),
                Operand::Str(v) => Ok(check_greater_than_or_equal_str(column, v, rule, n)),
                Operand::Column(name) => resolve_col(data, name)
                    .map(|other| check_greater_than_or_equal_col(column, other, name, rule, n)),
            },
            ColumnConstraint::LessThan(operand) => match operand {
                Operand::Num(v) => Ok(check_less_than_num(column, *v, rule, n)),
                Operand::Str(v) => Ok(check_less_than_str(column, v, rule, n)),
                Operand::Column(name) => resolve_col(data, name)
                    .map(|other| check_less_than_col(column, other, name, rule, n)),
            },
            ColumnConstraint::LessThanOrEqual(operand) => match operand {
                Operand::Num(v) => Ok(check_less_than_or_equal_num(column, *v, rule, n)),
                Operand::Str(v) => Ok(check_less_than_or_equal_str(column, v, rule, n)),
                Operand::Column(name) => resolve_col(data, name)
                    .map(|other| check_less_than_or_equal_col(column, other, name, rule, n)),
            },
            ColumnConstraint::Equal(operand) => match operand {
                Operand::Num(v) => Ok(check_equal_num(column, *v, rule, n)),
                Operand::Str(v) => Ok(check_equal_str(column, v, rule, n)),
                Operand::Column(name) => resolve_col(data, name)
                    .map(|other| check_equal_col(column, other, name, rule, n)),
            },
            ColumnConstraint::Between { min, max } => match (min, max) {
                (Operand::Num(lo), Operand::Num(hi)) => {
                    Ok(check_between_num(column, *lo, *hi, rule, n))
                }
                (Operand::Str(lo), Operand::Str(hi)) => {
                    Ok(check_between_str(column, lo, hi, rule, n))
                }
                (Operand::Column(lo), Operand::Column(hi)) => {
                    let lo_col = resolve_col(data, lo)?;
                    let hi_col = resolve_col(data, hi)?;
                    Ok(check_between_cols(column, lo_col, hi_col, rule, n))
                }
                _ => Err(ValidationError::MismatchedTypes {
                    recieved: format!("recieved: {}, {}", min.type_name(), max.type_name()),
                    expected: "(num, num) | (str, str) | (col, col)".to_string(),
                }),
            },
            ColumnConstraint::MatchesRegex(p) => Ok(check_matches_regex(column, p, rule, n)),
            ColumnConstraint::Contains(p) => Ok(check_contains(column, p, rule, n)),
            ColumnConstraint::StartsWith(p) => Ok(check_starts_with(column, p, rule, n)),
            ColumnConstraint::EndsWith(p) => Ok(check_ends_with(column, p, rule, n)),
            ColumnConstraint::LengthBetween { min, max } => {
                Ok(check_length_between(column, *min, *max, rule, n))
            }
            ColumnConstraint::Unique => Ok(check_unique(column, rule, n)),
            ColumnConstraint::InSet(other) => Ok(check_is_in_set(column, other, rule, n)),
            ColumnConstraint::After(date_str) => match &column {
                Column::Date(_) => {
                    let naive_date = NaiveDate::from_str(date_str)?;
                    Ok(check_after_date(
                        column,
                        naive_date_to_i32(&naive_date),
                        rule,
                        n,
                    ))
                }
                Column::DateTime(_) => {
                    let naive_dt = NaiveDateTime::from_str(date_str)?;
                    Ok(check_after_datetime(
                        column,
                        naive_datetime_to_i64(&naive_dt),
                        rule,
                        n,
                    ))
                }
                _ => unreachable!("Only applied to date or datetime columns"),
            },
            ColumnConstraint::Before(date_str) => match &column {
                Column::Date(_) => {
                    let naive_date = NaiveDate::from_str(date_str)?;
                    Ok(check_before_date(
                        column,
                        naive_date_to_i32(&naive_date),
                        rule,
                        n,
                    ))
                }
                Column::DateTime(_) => {
                    let naive_dt = NaiveDateTime::from_str(date_str)?;
                    Ok(check_before_datetime(
                        column,
                        naive_datetime_to_i64(&naive_dt),
                        rule,
                        n,
                    ))
                }
                _ => unreachable!("Only applied to date or datetime columns"),
            },
            ColumnConstraint::BetweenDates { min, max } => match &column {
                Column::Date(_) => {
                    let naive_date_min = NaiveDate::from_str(min)?;
                    let naive_date_max = NaiveDate::from_str(max)?;
                    Ok(check_between_dates(
                        column,
                        naive_date_to_i32(&naive_date_min),
                        naive_date_to_i32(&naive_date_max),
                        rule,
                        n,
                    ))
                }
                Column::DateTime(_) => {
                    let naive_dt_min = NaiveDateTime::from_str(min)?;
                    let naive_dt_max = NaiveDateTime::from_str(max)?;
                    Ok(check_between_datetimes(
                        column,
                        naive_datetime_to_i64(&naive_dt_min),
                        naive_datetime_to_i64(&naive_dt_max),
                        rule,
                        n,
                    ))
                }
                _ => unreachable!("Only applied to date or datetime columns"),
            },
        }
    } else {
        Err(ValidationError::ColumnNotFound {
            name: rule.column.to_string(),
        })
    }
}

fn resolve_col<'a>(data: &'a DataFrame, name: &str) -> Result<&'a Column, ValidationError> {
    data.get_column_by_name(name)
        .ok_or(ValidationError::ColumnNotFound {
            name: name.to_string(),
        })
}

fn check_not_null(col: &Column, rule: &ColumnRule, n: usize) -> ValidationResult {
    let mask = col.is_null();
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| **val)
        .map(|(idx, _)| (idx, "null".to_string()))
        .take(n)
        .collect();

    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            "null values found",
            Some(failed_values),
        )
    }
}

fn check_greater_than_num(
    col: &Column,
    value: f64,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.gt(value);

    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Int(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Float(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            _ => unreachable!("gt(f64) on non-numeric column"),
        })
        .take(n)
        .collect();

    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not greater than {}", value),
            Some(failed_values),
        )
    }
}

fn check_greater_than_str(
    col: &Column,
    value: &str,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.gt(value);

    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            _ => unreachable!("gt(str) on non-str column"),
        })
        .take(n)
        .collect();

    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not greater than {}", value),
            Some(failed_values),
        )
    }
}

fn check_greater_than_col(
    col: &Column,
    other: &Column,
    other_name: &str,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.gt(other);

    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Int(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Float(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            Column::Bool(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::DateTime(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Date(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();

    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not greater than {}", other_name),
            Some(failed_values),
        )
    }
}

fn check_greater_than_or_equal_num(
    col: &Column,
    value: f64,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.ge(value);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Int(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Float(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            _ => unreachable!("ge(f64) on non-numeric column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not >= {}", value),
            Some(failed_values),
        )
    }
}

fn check_greater_than_or_equal_str(
    col: &Column,
    value: &str,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.ge(value);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            _ => unreachable!("ge(str) on non-str column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not >= {}", value),
            Some(failed_values),
        )
    }
}

fn check_greater_than_or_equal_col(
    col: &Column,
    other: &Column,
    other_name: &str,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.ge(other);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Int(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Float(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            Column::Bool(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::DateTime(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Date(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not >= column: {}", other_name),
            Some(failed_values),
        )
    }
}

fn check_less_than_num(col: &Column, value: f64, rule: &ColumnRule, n: usize) -> ValidationResult {
    let mask = col.lt(value);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Int(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Float(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            _ => unreachable!("lt(f64) on non-numeric column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not less than {}", value),
            Some(failed_values),
        )
    }
}

fn check_less_than_str(col: &Column, value: &str, rule: &ColumnRule, n: usize) -> ValidationResult {
    let mask = col.lt(value);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            _ => unreachable!("lt(str) on non-str column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not less than {}", value),
            Some(failed_values),
        )
    }
}

fn check_less_than_col(
    col: &Column,
    other: &Column,
    other_name: &str,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.lt(other);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Int(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Float(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            Column::Bool(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::DateTime(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Date(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not < column: {}", other_name),
            Some(failed_values),
        )
    }
}

fn check_less_than_or_equal_num(
    col: &Column,
    value: f64,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.le(value);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Int(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Float(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            _ => unreachable!("le(f64) on non-numeric column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not <= {}", value),
            Some(failed_values),
        )
    }
}

fn check_less_than_or_equal_str(
    col: &Column,
    value: &str,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.le(value);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            _ => unreachable!("le(str) on non-str column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not <= {}", value),
            Some(failed_values),
        )
    }
}

fn check_less_than_or_equal_col(
    col: &Column,
    other: &Column,
    other_name: &str,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.le(other);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Int(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Float(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            Column::Bool(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::DateTime(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Date(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not <= column: {}", other_name),
            Some(failed_values),
        )
    }
}

fn check_equal_num(col: &Column, value: f64, rule: &ColumnRule, n: usize) -> ValidationResult {
    let mask = col.equal(value);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Int(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Float(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            _ => unreachable!("equal(f64) on non-numeric column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not equal to {}", value),
            Some(failed_values),
        )
    }
}

fn check_equal_str(col: &Column, value: &str, rule: &ColumnRule, n: usize) -> ValidationResult {
    let mask = col.equal(value);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            _ => unreachable!("equal(str) on non-str column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not equal to {}", value),
            Some(failed_values),
        )
    }
}

fn check_equal_col(
    col: &Column,
    other: &Column,
    other_name: &str,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.equal(other);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Int(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Float(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            Column::Bool(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::DateTime(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Date(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not equal to column: {}", other_name),
            Some(failed_values),
        )
    }
}

fn check_between_num(
    col: &Column,
    min: f64,
    max: f64,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.between(min, max);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Int(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Float(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            _ => unreachable!("between(f64) on non-numeric column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not between {} and {}", min, max),
            Some(failed_values),
        )
    }
}

fn check_between_str(
    col: &Column,
    min: &str,
    max: &str,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.between(min, max);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            _ => unreachable!("between(str) on non-str column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not between {} and {}", min, max),
            Some(failed_values),
        )
    }
}

fn check_between_cols(
    col: &Column,
    lo: &Column,
    hi: &Column,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.between(lo, hi);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Int(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Float(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            Column::Bool(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::DateTime(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Date(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            "values not between column bounds",
            Some(failed_values),
        )
    }
}

fn check_matches_regex(
    col: &Column,
    pattern: &str,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.matches_regex(pattern);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            _ => unreachable!("matches_regex on non-str column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values don't match regex '{}'", pattern),
            Some(failed_values),
        )
    }
}

fn check_contains(col: &Column, pattern: &str, rule: &ColumnRule, n: usize) -> ValidationResult {
    let mask = col.contains(pattern);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            _ => unreachable!("contains on non-str column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values don't contain '{}'", pattern),
            Some(failed_values),
        )
    }
}

fn check_starts_with(col: &Column, pattern: &str, rule: &ColumnRule, n: usize) -> ValidationResult {
    let mask = col.starts_with(pattern);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            _ => unreachable!("starts_with on non-str column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values don't start with '{}'", pattern),
            Some(failed_values),
        )
    }
}

fn check_ends_with(col: &Column, pattern: &str, rule: &ColumnRule, n: usize) -> ValidationResult {
    let mask = col.ends_with(pattern);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            _ => unreachable!("ends_with on non-str column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values don't end with '{}'", pattern),
            Some(failed_values),
        )
    }
}

fn check_length_between(
    col: &Column,
    min: usize,
    max: usize,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.length();
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, len)| !len.is_some_and(|v| v >= min && v <= max))
        .map(|(idx, _)| match col {
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            _ => unreachable!("length_between on non-str column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("string lengths not between {} and {}", min, max),
            Some(failed_values),
        )
    }
}

fn check_is_in_set(
    col: &Column,
    other: &ValuesSet,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.is_in(other);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Int(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Float(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            Column::Bool(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::DateTime(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Date(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("column values are not in set: {:?}", other),
            Some(failed_values),
        )
    }
}

fn check_unique(col: &Column, rule: &ColumnRule, n: usize) -> ValidationResult {
    if col.duplicates_count() == 0 {
        return ValidationResult::passed(rule);
    }
    let mask = col.duplicated(KeepStrategy::None);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| **val)
        .map(|(idx, _)| match col {
            Column::Int(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Float(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Str(c) => (idx, c.0[idx].as_deref().unwrap_or("null").to_string()),
            Column::Bool(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::DateTime(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Date(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();
    ValidationResult::failed(
        rule,
        failed_values.len(),
        "column values are not unique",
        Some(failed_values),
    )
}

fn check_after_date(col: &Column, value: i32, rule: &ColumnRule, n: usize) -> ValidationResult {
    let mask = col.gt(value);

    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Date(c) => (
                idx,
                c.0[idx]
                    .and_then(i32_to_naive_date)
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "null".to_string()),
            ),
            _ => unreachable!("gt(i32) on non-date column"),
        })
        .take(n)
        .collect();

    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!(
                "values not after date {}",
                i32_to_naive_date(value)
                    .map(|d| d.to_string())
                    .unwrap_or(value.to_string())
            ),
            Some(failed_values),
        )
    }
}

fn check_after_datetime(col: &Column, value: i64, rule: &ColumnRule, n: usize) -> ValidationResult {
    let mask = col.gt(value);

    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::DateTime(c) => (
                idx,
                c.0[idx]
                    .and_then(i64_to_naive_datetime)
                    .map(|dt| dt.to_string())
                    .unwrap_or_else(|| "null".to_string()),
            ),
            _ => unreachable!("gt(i64) on non-datetime column"),
        })
        .take(n)
        .collect();

    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!(
                "values not after datetime {}",
                i64_to_naive_datetime(value)
                    .map(|dt| dt.to_string())
                    .unwrap_or(value.to_string())
            ),
            Some(failed_values),
        )
    }
}

fn check_before_date(col: &Column, value: i32, rule: &ColumnRule, n: usize) -> ValidationResult {
    let mask = col.lt(value);

    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Date(c) => (
                idx,
                c.0[idx]
                    .and_then(i32_to_naive_date)
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "null".to_string()),
            ),
            _ => unreachable!("lt(i32) on non-date column"),
        })
        .take(n)
        .collect();

    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!(
                "values not before date {}",
                i32_to_naive_date(value)
                    .map(|d| d.to_string())
                    .unwrap_or(value.to_string())
            ),
            Some(failed_values),
        )
    }
}

fn check_before_datetime(
    col: &Column,
    value: i64,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.lt(value);

    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::DateTime(c) => (
                idx,
                c.0[idx]
                    .and_then(i64_to_naive_datetime)
                    .map(|dt| dt.to_string())
                    .unwrap_or_else(|| "null".to_string()),
            ),
            _ => unreachable!("lt(i64) on non-datetime column"),
        })
        .take(n)
        .collect();

    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!(
                "values not before datetime {}",
                i64_to_naive_datetime(value)
                    .map(|dt| dt.to_string())
                    .unwrap_or(value.to_string())
            ),
            Some(failed_values),
        )
    }
}
fn check_between_dates(
    col: &Column,
    min: i32,
    max: i32,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.between(min, max);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Date(c) => (
                idx,
                c.0[idx]
                    .and_then(i32_to_naive_date)
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "null".to_string()),
            ),
            _ => unreachable!("between(i32) on non-date column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!(
                "values not between dates {} and {}",
                i32_to_naive_date(min)
                    .map(|d| d.to_string())
                    .unwrap_or(min.to_string()),
                i32_to_naive_date(max)
                    .map(|d| d.to_string())
                    .unwrap_or(max.to_string())
            ),
            Some(failed_values),
        )
    }
}

fn check_between_datetimes(
    col: &Column,
    min: i64,
    max: i64,
    rule: &ColumnRule,
    n: usize,
) -> ValidationResult {
    let mask = col.between(min, max);
    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::DateTime(c) => (
                idx,
                c.0[idx]
                    .and_then(i64_to_naive_datetime)
                    .map(|dt| dt.to_string())
                    .unwrap_or_else(|| "null".to_string()),
            ),
            _ => unreachable!("between(i64) on non-datetime column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(rule)
    } else {
        let min_str = i64_to_naive_datetime(min)
            .map(|dt| dt.to_string())
            .unwrap_or(min.to_string());
        let max_str = i64_to_naive_datetime(max)
            .map(|dt| dt.to_string())
            .unwrap_or(max.to_string());
        ValidationResult::failed(
            rule,
            failed_values.len(),
            &format!("values not between datetimes {} and {}", min_str, max_str),
            Some(failed_values),
        )
    }
}
