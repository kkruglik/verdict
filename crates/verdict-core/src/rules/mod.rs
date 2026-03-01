use std::fmt::Display;

use crate::dataset::ops::{ComparableOps, StringOps};
use crate::{
    dataset::{Column, Dataset, FloatColumn, InSetValues},
    errors::ValidationError,
};

#[derive(Debug, Clone)]
pub enum Operand {
    Column(String),
    Literal(f64),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Literal(val) => write!(f, "{}", val),
            Operand::Column(name) => write!(f, "col({})", name),
        }
    }
}

impl From<f64> for Operand {
    fn from(value: f64) -> Self {
        Operand::Literal(value)
    }
}

impl From<i64> for Operand {
    fn from(value: i64) -> Self {
        Operand::Literal(value as f64)
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

pub struct ValidationResult {
    pub column: String,
    pub constraint: String,
    pub passed: bool,
    pub failed_count: usize,
    pub error: Option<String>,
}

impl ValidationResult {
    pub fn passed(rule: &Rule) -> Self {
        ValidationResult {
            column: rule.column.clone(),
            constraint: format!("{}", rule.constraint),
            passed: true,
            failed_count: 0,
            error: None,
        }
    }

    pub fn failed(rule: &Rule, failed_count: usize, error: &str) -> Self {
        ValidationResult {
            column: rule.column.clone(),
            constraint: format!("{}", rule.constraint),
            passed: false,
            failed_count,
            error: Some(error.to_string()),
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

pub fn validate(data: &Dataset, rules: &[Rule]) -> Vec<ValidationResult> {
    rules
        .iter()
        .map(|rule| {
            validate_with_rule(data, rule)
                .unwrap_or_else(|e| ValidationResult::failed(rule, 0, &e.to_string()))
        })
        .collect()
}

fn validate_with_rule(data: &Dataset, rule: &Rule) -> Result<ValidationResult, ValidationError> {
    if let Some(column) = data.get_column_by_name(&rule.column) {
        match &rule.constraint {
            Constraint::NotNull => Ok(check_not_null(column, rule)),
            Constraint::GreaterThan(operand) => match operand {
                Operand::Literal(v) => Ok(check_greater_than_val(column, *v, rule)),
                Operand::Column(name) => {
                    if let Some(other) = data.get_column_by_name(name) {
                        Ok(check_greater_than_col(column, other, name, rule))
                    } else {
                        Err(ValidationError::ColumnNotFound {
                            name: name.to_string(),
                        })
                    }
                }
            },
            Constraint::GreaterThanOrEqual(operand) => match operand {
                Operand::Literal(v) => Ok(check_greater_than_or_equal_val(column, *v, rule)),
                Operand::Column(name) => resolve_col(data, name)
                    .map(|other| check_greater_than_or_equal_col(column, other, name, rule)),
            },
            Constraint::LessThan(operand) => match operand {
                Operand::Literal(v) => Ok(check_less_than_val(column, *v, rule)),
                Operand::Column(name) => resolve_col(data, name)
                    .map(|other| check_less_than_col(column, other, name, rule)),
            },
            Constraint::LessThanOrEqual(operand) => match operand {
                Operand::Literal(v) => Ok(check_less_than_or_equal_val(column, *v, rule)),
                Operand::Column(name) => resolve_col(data, name)
                    .map(|other| check_less_than_or_equal_col(column, other, name, rule)),
            },
            Constraint::Equal(operand) => match operand {
                Operand::Literal(v) => Ok(check_equal_val(column, *v, rule)),
                Operand::Column(name) => {
                    resolve_col(data, name).map(|other| check_equal_col(column, other, name, rule))
                }
            },
            Constraint::Between { min, max } => match (min, max) {
                (Operand::Literal(lo), Operand::Literal(hi)) => {
                    Ok(check_between(column, *lo, *hi, rule))
                }
                _ => {
                    let len = column.len();
                    let lo: Column = match min {
                        Operand::Literal(v) => Column::Float(FloatColumn(vec![Some(*v); len])),
                        Operand::Column(name) => resolve_col(data, name)?.clone(),
                    };
                    let hi: Column = match max {
                        Operand::Literal(v) => Column::Float(FloatColumn(vec![Some(*v); len])),
                        Operand::Column(name) => resolve_col(data, name)?.clone(),
                    };
                    Ok(check_between_cols(column, &lo, &hi, rule))
                }
            },
            Constraint::MatchesRegex(p) => Ok(check_matches_regex(column, p, rule)),
            Constraint::Contains(p) => Ok(check_contains(column, p, rule)),
            Constraint::StartsWith(p) => Ok(check_starts_with(column, p, rule)),
            Constraint::EndsWith(p) => Ok(check_ends_with(column, p, rule)),
            Constraint::LengthBetween { min, max } => {
                Ok(check_length_between(column, *min, *max, rule))
            }
            Constraint::Unique => Ok(check_unique(column, rule)),
            Constraint::InSet(other) => Ok(check_is_in_set(column, other, rule)),
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

fn check_not_null(col: &Column, rule: &Rule) -> ValidationResult {
    let failed = col.null_count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(rule, failed, "null values found")
    }
}

fn check_greater_than_val(col: &Column, value: f64, rule: &Rule) -> ValidationResult {
    let failed = col
        .gt(value)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(rule, failed, &format!("values not greater than {}", value))
    }
}

fn check_greater_than_col(
    col: &Column,
    other: &Column,
    other_name: &str,
    rule: &Rule,
) -> ValidationResult {
    let failed = col
        .gt(other)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed,
            &format!("values not greater than other column: {}", other_name),
        )
    }
}

fn check_greater_than_or_equal_val(col: &Column, value: f64, rule: &Rule) -> ValidationResult {
    let failed = col
        .ge(value)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(rule, failed, &format!("values not >= {}", value))
    }
}

fn check_greater_than_or_equal_col(
    col: &Column,
    other: &Column,
    other_name: &str,
    rule: &Rule,
) -> ValidationResult {
    let failed = col
        .ge(other)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed,
            &format!("values not >= column: {}", other_name),
        )
    }
}

fn check_less_than_val(col: &Column, value: f64, rule: &Rule) -> ValidationResult {
    let failed = col
        .lt(value)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(rule, failed, &format!("values not less than {}", value))
    }
}

fn check_less_than_col(
    col: &Column,
    other: &Column,
    other_name: &str,
    rule: &Rule,
) -> ValidationResult {
    let failed = col
        .lt(other)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed,
            &format!("values not < column: {}", other_name),
        )
    }
}

fn check_less_than_or_equal_val(col: &Column, value: f64, rule: &Rule) -> ValidationResult {
    let failed = col
        .le(value)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed,
            &format!("values not less than or equal to {}", value),
        )
    }
}

fn check_less_than_or_equal_col(
    col: &Column,
    other: &Column,
    other_name: &str,
    rule: &Rule,
) -> ValidationResult {
    let failed = col
        .le(other)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed,
            &format!("values not <= column: {}", other_name),
        )
    }
}

fn check_equal_val(col: &Column, value: f64, rule: &Rule) -> ValidationResult {
    let failed = col
        .equal(value)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(rule, failed, &format!("values not equal to {}", value))
    }
}

fn check_equal_col(
    col: &Column,
    other: &Column,
    other_name: &str,
    rule: &Rule,
) -> ValidationResult {
    let failed = col
        .equal(other)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed,
            &format!("values not equal to column: {}", other_name),
        )
    }
}

fn check_between(col: &Column, min: f64, max: f64, rule: &Rule) -> ValidationResult {
    let failed = col
        .between(min, max)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed,
            &format!("values not between {} and {}", min, max),
        )
    }
}

fn check_between_cols(col: &Column, lo: &Column, hi: &Column, rule: &Rule) -> ValidationResult {
    let failed = col
        .between(lo, hi)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(rule, failed, "values not between column bounds")
    }
}

fn check_matches_regex(col: &Column, pattern: &str, rule: &Rule) -> ValidationResult {
    let failed = col
        .matches_regex(pattern)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed,
            &format!("values don't match regex '{}'", pattern),
        )
    }
}

fn check_contains(col: &Column, pattern: &str, rule: &Rule) -> ValidationResult {
    let failed = col
        .contains(pattern)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(rule, failed, &format!("values don't contain '{}'", pattern))
    }
}

fn check_starts_with(col: &Column, pattern: &str, rule: &Rule) -> ValidationResult {
    let failed = col
        .starts_with(pattern)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed,
            &format!("values don't start with '{}'", pattern),
        )
    }
}

fn check_ends_with(col: &Column, pattern: &str, rule: &Rule) -> ValidationResult {
    let failed = col
        .ends_with(pattern)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed,
            &format!("values don't end with '{}'", pattern),
        )
    }
}

fn check_length_between(col: &Column, min: usize, max: usize, rule: &Rule) -> ValidationResult {
    let failed = col
        .length()
        .iter()
        .map(|opt| opt.is_some_and(|v| (v >= min) && (v <= max)))
        .filter(|v| !v)
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed,
            &format!("string lengths not between {} and {}", min, max),
        )
    }
}

fn check_is_in_set(col: &Column, other: &InSetValues, rule: &Rule) -> ValidationResult {
    let failed_count = col
        .is_in(other)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed_count == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed_count,
            &format!("column values are not in set: {:?}", other),
        )
    }
}

fn check_unique(col: &Column, rule: &Rule) -> ValidationResult {
    let failed_count = col.duplicates_count();
    if failed_count == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(rule, failed_count, "column values are not unique")
    }
}
