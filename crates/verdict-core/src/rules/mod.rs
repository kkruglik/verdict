use crate::{
    dataset::{Column, Dataset},
    errors::ValidationError,
};

pub struct Rule {
    pub column: String,
    pub constraint: Constraint,
}

#[derive(Debug)]
pub enum Constraint {
    // Null checks
    NotNull,
    Unique,

    // Numeric comparisons
    GreaterThan(f64),
    GreaterThanOrEqual(f64),
    LessThan(f64),
    LessThanOrEqual(f64),
    Equal(f64),
    Between { min: f64, max: f64 },

    // String checks
    InSet(Vec<String>),
    MatchesRegex(String),
    Contains(String),
    StartsWith(String),
    EndsWith(String),
    LengthBetween { min: usize, max: usize },
}

pub struct ValidationResult {
    pub column: String,
    pub constraint: String,
    pub passed: bool,
    pub failed_count: usize,
    pub error: Option<String>,
}

impl Rule {
    pub fn new(column: &str, constraint: Constraint) -> Rule {
        Rule {
            column: column.to_string(),
            constraint,
        }
    }
}

impl ValidationResult {
    pub fn passed(rule: &Rule) -> Self {
        ValidationResult {
            column: rule.column.clone(),
            constraint: format!("{:?}", rule.constraint),
            passed: true,
            failed_count: 0,
            error: None,
        }
    }

    pub fn failed(rule: &Rule, failed_count: usize, error: &str) -> Self {
        ValidationResult {
            column: rule.column.clone(),
            constraint: format!("{:?}", rule.constraint),
            passed: false,
            failed_count,
            error: Some(error.to_string()),
        }
    }
}

fn validate_col_with_rule(
    column: &Column,
    rule: &Rule,
) -> Result<ValidationResult, ValidationError> {
    match &rule.constraint {
        Constraint::NotNull => Ok(check_not_null(column, rule)),
        Constraint::GreaterThan(v) => Ok(check_greater_than(column, *v, rule)),
        Constraint::GreaterThanOrEqual(v) => Ok(check_greater_than_or_equal(column, *v, rule)),
        Constraint::LessThan(v) => Ok(check_less_than(column, *v, rule)),
        Constraint::LessThanOrEqual(v) => Ok(check_less_than_or_equal(column, *v, rule)),
        Constraint::Equal(v) => Ok(check_equal(column, *v, rule)),
        Constraint::Between { min, max } => Ok(check_between(column, *min, *max, rule)),
        Constraint::MatchesRegex(p) => Ok(check_matches_regex(column, p, rule)),
        Constraint::Contains(p) => Ok(check_contains(column, p, rule)),
        Constraint::StartsWith(p) => Ok(check_starts_with(column, p, rule)),
        Constraint::EndsWith(p) => Ok(check_ends_with(column, p, rule)),
        Constraint::LengthBetween { min, max } => {
            Ok(check_length_between(column, *min, *max, rule))
        }
        Constraint::Unique | Constraint::InSet(_) => Err(ValidationError::UnknownConstraint {
            name: format!("{:?}", rule.constraint),
        }),
    }
}

pub fn validate(data: &Dataset, rules: &[Rule]) -> Vec<ValidationResult> {
    rules
        .iter()
        .map(|rule| {
            let col = data.get_column_by_name(&rule.column);
            match col {
                Some(col) => validate_col_with_rule(&col, rule)
                    .unwrap_or_else(|e| ValidationResult::failed(rule, 0, &e.to_string())),
                None => {
                    let error = ValidationError::ColumnNotFound {
                        name: rule.column.clone(),
                    };
                    ValidationResult::failed(rule, 0, &error.to_string())
                }
            }
        })
        .collect()
}

fn check_not_null(col: &Column, rule: &Rule) -> ValidationResult {
    let failed = col.null_count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(rule, failed, "null values found")
    }
}

fn check_greater_than(col: &Column, value: f64, rule: &Rule) -> ValidationResult {
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

fn check_greater_than_or_equal(col: &Column, value: f64, rule: &Rule) -> ValidationResult {
    let failed = col
        .ge(value)
        .iter()
        .filter(|v| !matches!(v, Some(true)))
        .count();
    if failed == 0 {
        ValidationResult::passed(rule)
    } else {
        ValidationResult::failed(
            rule,
            failed,
            &format!("values not greater than or equal to {}", value),
        )
    }
}

fn check_less_than(col: &Column, value: f64, rule: &Rule) -> ValidationResult {
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

fn check_less_than_or_equal(col: &Column, value: f64, rule: &Rule) -> ValidationResult {
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

fn check_equal(col: &Column, value: f64, rule: &Rule) -> ValidationResult {
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
        .str_length()
        .iter()
        .map(|opt| opt.map_or(false, |v| (v >= min) && (v <= max)))
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
