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

fn validate_col_with_rule(
    column: &Column,
    rule: &Rule,
) -> Result<ValidationResult, ValidationError> {
    match rule.constraint {
        Constraint::NotNull => {
            let (passed, failed_count) = check_not_null(column);
            Ok(ValidationResult {
                column: rule.column.clone(),
                constraint: format!("{:?}", rule.constraint),
                error: None,
                failed_count,
                passed,
            })
        }
        _ => Err(ValidationError::UnknownConstraint {
            name: "Some placeholder".to_string(),
        }),
    }
}

pub fn validate(data: &Dataset, rules: &[Rule]) -> Vec<ValidationResult> {
    rules
        .iter()
        .map(|rule| {
            let col = data.get_column_by_name(&rule.column);
            match col {
                Some(col) => {
                    validate_col_with_rule(&col, rule).unwrap_or_else(|e| ValidationResult {
                        column: rule.column.clone(),
                        constraint: format!("{:?}", rule.constraint),
                        passed: false,
                        error: Some(e.to_string()),
                        failed_count: 0,
                    })
                }
                None => {
                    let error = ValidationError::ColumnNotFound {
                        name: rule.column.clone(),
                    };
                    ValidationResult {
                        column: rule.column.clone(),
                        constraint: format!("{:?}", rule.constraint),
                        passed: false,
                        error: Some(error.to_string()),
                        failed_count: 0,
                    }
                }
            }
        })
        .collect()
}

fn check_not_null(col: &Column) -> (bool, usize) {
    let failed = col.null_count();
    (failed == 0, failed)
}

fn check_unique(col: &Column) -> (bool, usize);
fn check_greater_than(col: &Column, value: f64) -> (bool, usize);
fn check_greater_than_or_equal(col: &Column, value: f64) -> (bool, usize);
fn check_less_than(col: &Column, value: f64) -> (bool, usize);
fn check_less_than_or_equal(col: &Column, value: f64) -> (bool, usize);
fn check_equal(col: &Column, value: f64) -> (bool, usize);
fn check_between(col: &Column, min: f64, max: f64) -> (bool, usize);
fn check_in_set(col: &Column, values: &[String]) -> (bool, usize);
fn check_matches_regex(col: &Column, pattern: &str) -> (bool, usize);
fn check_contains(col: &Column, pattern: &str) -> (bool, usize);
fn check_starts_with(col: &Column, pattern: &str) -> (bool, usize);
fn check_ends_with(col: &Column, pattern: &str) -> (bool, usize);
fn check_length_between(col: &Column, min: usize, max: usize) -> (bool, usize);
