use std::str::FromStr;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::{
    dataframe::{
        Column, DataFrame, KeepStrategy, ValuesSet, i32_to_naive_date, i64_to_naive_datetime,
        naive_date_to_i32, naive_datetime_to_i64,
        ops::{ComparableOps, StringOps, i64_to_naive_time, naive_time_to_i64},
    },
    errors::ValidationError,
    rules::{
        ColumnConstraint, ColumnRule, Operand, ValidationConfig, ValidationReport,
        ValidationResult, validation::CheckScope,
    },
};

pub fn validate_columns(
    data: &DataFrame,
    rules: &[ColumnRule],
    config: ValidationConfig,
) -> ValidationReport {
    let results: Vec<ValidationResult> = rules
        .iter()
        .map(|rule| {
            validate_cols_with_rule(data, rule, &config).unwrap_or_else(|e| {
                ValidationResult::failed(
                    rule.constraint.to_string().as_str(),
                    &e.to_string(),
                    Some(rule.column.as_str()),
                    None,
                    None,
                )
            })
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

fn validate_cols_with_rule(
    data: &DataFrame,
    rule: &ColumnRule,
    config: &ValidationConfig,
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
                Column::Time(_) => {
                    let naive_time = NaiveTime::from_str(date_str)?;
                    Ok(check_after_time(
                        column,
                        naive_time_to_i64(&naive_time),
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
                Column::Time(_) => {
                    let naive_time = NaiveTime::from_str(date_str)?;
                    Ok(check_before_time(
                        column,
                        naive_time_to_i64(&naive_time),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            "null values found",
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not greater than {}", value),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
            Column::Str(col) => (idx, col.0[idx].as_deref().unwrap_or("null").to_string()),
            Column::DateTime(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i64_to_naive_datetime(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            Column::Date(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i32_to_naive_date(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            Column::Time(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i64_to_naive_time(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            _ => unreachable!("gt(str) on non-str column"),
        })
        .take(n)
        .collect();

    if failed_values.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not greater than {}", value),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
                    .and_then(i64_to_naive_datetime)
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Date(c) => (
                idx,
                c.0[idx]
                    .and_then(i32_to_naive_date)
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Time(c) => (
                idx,
                c.0[idx]
                    .and_then(i64_to_naive_time)
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();

    if failed_values.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not greater than {}", other_name),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not >= {}", value),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
            Column::DateTime(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i64_to_naive_datetime(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            Column::Date(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i32_to_naive_date(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            Column::Time(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i64_to_naive_time(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            _ => unreachable!("ge(str) on non-str column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not >= {}", value),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
            Column::Time(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not >= column: {}", other_name),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not less than {}", value),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
            Column::DateTime(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i64_to_naive_datetime(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            Column::Date(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i32_to_naive_date(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            Column::Time(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i64_to_naive_time(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            _ => unreachable!("lt(str) on non-str column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not less than {}", value),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
                    .and_then(i64_to_naive_datetime)
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Date(c) => (
                idx,
                c.0[idx]
                    .and_then(i32_to_naive_date)
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Time(c) => (
                idx,
                c.0[idx]
                    .and_then(i64_to_naive_time)
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not < column: {}", other_name),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not <= {}", value),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
            Column::DateTime(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i64_to_naive_datetime(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            Column::Date(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i32_to_naive_date(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            Column::Time(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i64_to_naive_time(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            _ => unreachable!("le(str) on non-str column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not <= {}", value),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
            Column::Time(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not <= column: {}", other_name),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not equal to {}", value),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
            Column::DateTime(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i64_to_naive_datetime(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            Column::Date(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i32_to_naive_date(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            Column::Time(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i64_to_naive_time(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            _ => unreachable!("equal(str) on non-str column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not equal to {}", value),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
                    .and_then(i64_to_naive_datetime)
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Date(c) => (
                idx,
                c.0[idx]
                    .and_then(i32_to_naive_date)
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Time(c) => (
                idx,
                c.0[idx]
                    .and_then(i64_to_naive_time)
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not equal to column: {}", other_name),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not between {} and {}", min, max),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
            Column::DateTime(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i64_to_naive_datetime(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            Column::Date(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i32_to_naive_date(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            Column::Time(col) => (
                idx,
                col.0[idx]
                    .map(|val| {
                        if let Some(d) = i64_to_naive_time(val) {
                            d.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .unwrap_or_else(|| "null".to_string()),
            ),
            _ => unreachable!("between(str) on non-str column"),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not between {} and {}", min, max),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
                    .and_then(i64_to_naive_datetime)
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Date(c) => (
                idx,
                c.0[idx]
                    .and_then(i32_to_naive_date)
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
            Column::Time(c) => (
                idx,
                c.0[idx]
                    .and_then(i64_to_naive_time)
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            "values not between column bounds",
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values don't match regex '{}'", pattern),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values don't contain '{}'", pattern),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values don't start with '{}'", pattern),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values don't end with '{}'", pattern),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("string lengths not between {} and {}", min, max),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
            Column::Time(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();
    if failed_values.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("column values are not in set: {:?}", other),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
            Some(failed_values),
        )
    }
}

fn check_unique(col: &Column, rule: &ColumnRule, n: usize) -> ValidationResult {
    if col.duplicates_count() == 0 {
        return ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        );
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
            Column::Time(c) => (
                idx,
                c.0[idx]
                    .map(|v| v.to_string())
                    .unwrap_or("null".to_string()),
            ),
        })
        .take(n)
        .collect();
    ValidationResult::failed(
        rule.constraint.to_string().as_str(),
        "column values are not unique",
        Some(rule.column.as_str()),
        Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "values not after date {}",
                i32_to_naive_date(value)
                    .map(|d| d.to_string())
                    .unwrap_or(value.to_string())
            ),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "values not after datetime {}",
                i64_to_naive_datetime(value)
                    .map(|dt| dt.to_string())
                    .unwrap_or(value.to_string())
            ),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "values not before date {}",
                i32_to_naive_date(value)
                    .map(|d| d.to_string())
                    .unwrap_or(value.to_string())
            ),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "values not before datetime {}",
                i64_to_naive_datetime(value)
                    .map(|dt| dt.to_string())
                    .unwrap_or(value.to_string())
            ),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
            Some(failed_values),
        )
    }
}

fn check_before_time(col: &Column, value: i64, rule: &ColumnRule, n: usize) -> ValidationResult {
    let mask = col.lt(value);

    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Time(c) => (
                idx,
                c.0[idx]
                    .and_then(i64_to_naive_time)
                    .map(|time| time.to_string())
                    .unwrap_or_else(|| "null".to_string()),
            ),
            _ => unreachable!("check_before_time called on non-Time column"),
        })
        .take(n)
        .collect();

    if failed_values.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "values not before time {}",
                i64_to_naive_time(value)
                    .map(|dt| dt.to_string())
                    .unwrap_or(value.to_string())
            ),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
            Some(failed_values),
        )
    }
}

fn check_after_time(col: &Column, value: i64, rule: &ColumnRule, n: usize) -> ValidationResult {
    let mask = col.gt(value);

    let failed_values: Vec<(usize, String)> = mask
        .iter()
        .enumerate()
        .filter(|(_, val)| matches!(val, Some(false)))
        .map(|(idx, _)| match col {
            Column::Time(c) => (
                idx,
                c.0[idx]
                    .and_then(i64_to_naive_time)
                    .map(|time| time.to_string())
                    .unwrap_or_else(|| "null".to_string()),
            ),
            _ => unreachable!("check_after_time called on non-Time column"),
        })
        .take(n)
        .collect();

    if failed_values.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "values not after time {}",
                i64_to_naive_time(value)
                    .map(|dt| dt.to_string())
                    .unwrap_or(value.to_string())
            ),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "values not between dates {} and {}",
                i32_to_naive_date(min)
                    .map(|d| d.to_string())
                    .unwrap_or(min.to_string()),
                i32_to_naive_date(max)
                    .map(|d| d.to_string())
                    .unwrap_or(max.to_string())
            ),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
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
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            Some(rule.column.as_str()),
            CheckScope::Column,
        )
    } else {
        let min_str = i64_to_naive_datetime(min)
            .map(|dt| dt.to_string())
            .unwrap_or(min.to_string());
        let max_str = i64_to_naive_datetime(max)
            .map(|dt| dt.to_string())
            .unwrap_or(max.to_string());
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("values not between datetimes {} and {}", min_str, max_str),
            Some(rule.column.as_str()),
            Some(failed_values.len()),
            Some(failed_values),
        )
    }
}
