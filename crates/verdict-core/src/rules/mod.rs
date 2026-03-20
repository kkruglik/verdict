use std::fmt::Display;

pub struct ValidateConfig {
    pub max_failed_samples: usize,
}

impl Default for ValidateConfig {
    fn default() -> Self {
        Self {
            max_failed_samples: 100,
        }
    }
}

use crate::dataset::Keep;
use crate::dataset::ops::{ComparableOps, StringOps};
use crate::{
    dataset::{Column, Dataset, InSetValues},
    errors::ValidationError,
};

#[derive(Debug, Clone)]
pub enum Operand {
    Column(String),
    Num(f64),
    Str(String),
}

impl Operand {
    pub fn type_name(&self) -> &'static str {
        match self {
            Operand::Column(_) => "col",
            Operand::Num(_) => "num",
            Operand::Str(_) => "str",
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Num(val) => write!(f, "{}", val),
            Operand::Column(name) => write!(f, "col({})", name),
            Operand::Str(val) => write!(f, "{}", val),
        }
    }
}

impl From<f64> for Operand {
    fn from(value: f64) -> Self {
        Operand::Num(value)
    }
}

impl From<i64> for Operand {
    fn from(value: i64) -> Self {
        Operand::Num(value as f64)
    }
}

pub fn col(name: &str) -> Operand {
    Operand::Column(name.to_string())
}

#[derive(Debug, Clone)]
pub enum Constraint {
    // Null checks
    NotNull,
    Unique,

    // Numeric comparisons
    GreaterThan(Operand),
    GreaterThanOrEqual(Operand),
    LessThan(Operand),
    LessThanOrEqual(Operand),
    Equal(Operand),
    Between { min: Operand, max: Operand },

    // String checks
    InSet(InSetValues),
    MatchesRegex(String),
    Contains(String),
    StartsWith(String),
    EndsWith(String),
    LengthBetween { min: usize, max: usize },
}

impl std::fmt::Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constraint::NotNull => write!(f, "not_null"),
            Constraint::Unique => write!(f, "unique"),
            Constraint::GreaterThan(op) => write!(f, "gt({})", op),
            Constraint::GreaterThanOrEqual(op) => write!(f, "ge({})", op),
            Constraint::LessThan(op) => write!(f, "lt({})", op),
            Constraint::LessThanOrEqual(op) => write!(f, "le({})", op),
            Constraint::Equal(op) => write!(f, "eq({})", op),
            Constraint::Between { min, max } => write!(f, "between({}, {})", min, max),
            Constraint::InSet(_) => write!(f, "in_set"),
            Constraint::MatchesRegex(p) => write!(f, "matches_regex({})", p),
            Constraint::Contains(p) => write!(f, "contains({})", p),
            Constraint::StartsWith(p) => write!(f, "starts_with({})", p),
            Constraint::EndsWith(p) => write!(f, "ends_with({})", p),
            Constraint::LengthBetween { min, max } => write!(f, "length_between({}, {})", min, max),
        }
    }
}

#[derive(Clone)]
pub struct Rule {
    pub column: String,
    pub constraint: Constraint,
}

impl Rule {
    pub fn new(column: &str, constraint: Constraint) -> Rule {
        Rule {
            column: column.to_string(),
            constraint,
        }
    }
}

#[derive(Default)]
pub struct RuleBuilder {
    pub column: String,
    pub constraint: Vec<Constraint>,
}

impl RuleBuilder {
    pub fn not_null(mut self) -> Self {
        self.constraint.push(Constraint::NotNull);
        self
    }

    pub fn unique(mut self) -> Self {
        self.constraint.push(Constraint::Unique);
        self
    }

    pub fn gt(mut self, compare: impl Into<Operand>) -> Self {
        self.constraint
            .push(Constraint::GreaterThan(compare.into()));
        self
    }

    pub fn ge(mut self, compare: impl Into<Operand>) -> Self {
        self.constraint
            .push(Constraint::GreaterThanOrEqual(compare.into()));
        self
    }

    pub fn lt(mut self, compare: impl Into<Operand>) -> Self {
        self.constraint.push(Constraint::LessThan(compare.into()));
        self
    }

    pub fn le(mut self, compare: impl Into<Operand>) -> Self {
        self.constraint
            .push(Constraint::LessThanOrEqual(compare.into()));
        self
    }

    pub fn equal(mut self, compare: impl Into<Operand>) -> Self {
        self.constraint.push(Constraint::Equal(compare.into()));
        self
    }

    pub fn between(mut self, min: impl Into<Operand>, max: impl Into<Operand>) -> Self {
        self.constraint.push(Constraint::Between {
            min: min.into(),
            max: max.into(),
        });
        self
    }

    pub fn in_set(mut self, values: InSetValues) -> Self {
        self.constraint.push(Constraint::InSet(values));
        self
    }

    pub fn matches_regex(mut self, pattern: &str) -> Self {
        self.constraint
            .push(Constraint::MatchesRegex(pattern.to_string()));
        self
    }

    pub fn contains(mut self, pattern: &str) -> Self {
        self.constraint
            .push(Constraint::Contains(pattern.to_string()));
        self
    }

    pub fn starts_with(mut self, pattern: &str) -> Self {
        self.constraint
            .push(Constraint::StartsWith(pattern.to_string()));
        self
    }

    pub fn ends_with(mut self, pattern: &str) -> Self {
        self.constraint
            .push(Constraint::EndsWith(pattern.to_string()));
        self
    }

    pub fn length_between(mut self, min: usize, max: usize) -> Self {
        self.constraint.push(Constraint::LengthBetween { min, max });
        self
    }

    pub fn build(self) -> Vec<Rule> {
        self.constraint
            .into_iter()
            .map(|c| Rule {
                column: self.column.clone(),
                constraint: c,
            })
            .collect()
    }
}

pub fn rule(col_name: &str) -> RuleBuilder {
    RuleBuilder {
        column: col_name.to_string(),
        constraint: vec![],
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
    pub fn passed(rule: &Rule) -> Self {
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
        rule: &Rule,
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

pub fn validate(data: &Dataset, rules: &[Rule], config: ValidateConfig) -> ValidationReport {
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
    data: &Dataset,
    rule: &Rule,
    config: &ValidateConfig,
) -> Result<ValidationResult, ValidationError> {
    if let Some(column) = data.get_column_by_name(&rule.column) {
        let n = config.max_failed_samples;
        match &rule.constraint {
            Constraint::NotNull => Ok(check_not_null(column, rule, n)),
            Constraint::GreaterThan(operand) => match operand {
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
            Constraint::GreaterThanOrEqual(operand) => match operand {
                Operand::Num(v) => Ok(check_greater_than_or_equal_num(column, *v, rule, n)),
                Operand::Str(v) => Ok(check_greater_than_or_equal_str(column, v, rule, n)),
                Operand::Column(name) => resolve_col(data, name)
                    .map(|other| check_greater_than_or_equal_col(column, other, name, rule, n)),
            },
            Constraint::LessThan(operand) => match operand {
                Operand::Num(v) => Ok(check_less_than_num(column, *v, rule, n)),
                Operand::Str(v) => Ok(check_less_than_str(column, v, rule, n)),
                Operand::Column(name) => resolve_col(data, name)
                    .map(|other| check_less_than_col(column, other, name, rule, n)),
            },
            Constraint::LessThanOrEqual(operand) => match operand {
                Operand::Num(v) => Ok(check_less_than_or_equal_num(column, *v, rule, n)),
                Operand::Str(v) => Ok(check_less_than_or_equal_str(column, v, rule, n)),
                Operand::Column(name) => resolve_col(data, name)
                    .map(|other| check_less_than_or_equal_col(column, other, name, rule, n)),
            },
            Constraint::Equal(operand) => match operand {
                Operand::Num(v) => Ok(check_equal_num(column, *v, rule, n)),
                Operand::Str(v) => Ok(check_equal_str(column, v, rule, n)),
                Operand::Column(name) => resolve_col(data, name)
                    .map(|other| check_equal_col(column, other, name, rule, n)),
            },
            Constraint::Between { min, max } => match (min, max) {
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
            Constraint::MatchesRegex(p) => Ok(check_matches_regex(column, p, rule, n)),
            Constraint::Contains(p) => Ok(check_contains(column, p, rule, n)),
            Constraint::StartsWith(p) => Ok(check_starts_with(column, p, rule, n)),
            Constraint::EndsWith(p) => Ok(check_ends_with(column, p, rule, n)),
            Constraint::LengthBetween { min, max } => {
                Ok(check_length_between(column, *min, *max, rule, n))
            }
            Constraint::Unique => Ok(check_unique(column, rule, n)),
            Constraint::InSet(other) => Ok(check_is_in_set(column, other, rule, n)),
        }
    } else {
        Err(ValidationError::ColumnNotFound {
            name: rule.column.to_string(),
        })
    }
}

fn resolve_col<'a>(data: &'a Dataset, name: &str) -> Result<&'a Column, ValidationError> {
    data.get_column_by_name(name)
        .ok_or(ValidationError::ColumnNotFound {
            name: name.to_string(),
        })
}

fn check_not_null(col: &Column, rule: &Rule, n: usize) -> ValidationResult {
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

fn check_greater_than_num(col: &Column, value: f64, rule: &Rule, n: usize) -> ValidationResult {
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

fn check_greater_than_str(col: &Column, value: &str, rule: &Rule, n: usize) -> ValidationResult {
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
    rule: &Rule,
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
    rule: &Rule,
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
    rule: &Rule,
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
    rule: &Rule,
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

fn check_less_than_num(col: &Column, value: f64, rule: &Rule, n: usize) -> ValidationResult {
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

fn check_less_than_str(col: &Column, value: &str, rule: &Rule, n: usize) -> ValidationResult {
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
    rule: &Rule,
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
    rule: &Rule,
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
    rule: &Rule,
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
    rule: &Rule,
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

fn check_equal_num(col: &Column, value: f64, rule: &Rule, n: usize) -> ValidationResult {
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

fn check_equal_str(col: &Column, value: &str, rule: &Rule, n: usize) -> ValidationResult {
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
    rule: &Rule,
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

fn check_between_num(col: &Column, min: f64, max: f64, rule: &Rule, n: usize) -> ValidationResult {
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
    rule: &Rule,
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
    rule: &Rule,
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

fn check_matches_regex(col: &Column, pattern: &str, rule: &Rule, n: usize) -> ValidationResult {
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

fn check_contains(col: &Column, pattern: &str, rule: &Rule, n: usize) -> ValidationResult {
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

fn check_starts_with(col: &Column, pattern: &str, rule: &Rule, n: usize) -> ValidationResult {
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

fn check_ends_with(col: &Column, pattern: &str, rule: &Rule, n: usize) -> ValidationResult {
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
    rule: &Rule,
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

fn check_is_in_set(col: &Column, other: &InSetValues, rule: &Rule, n: usize) -> ValidationResult {
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

fn check_unique(col: &Column, rule: &Rule, n: usize) -> ValidationResult {
    if col.duplicates_count() == 0 {
        return ValidationResult::passed(rule);
    }
    let mask = col.duplicated(Keep::None);
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
