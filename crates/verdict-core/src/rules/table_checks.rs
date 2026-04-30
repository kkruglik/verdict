use crate::{
    dataframe::DataFrame,
    errors::ValidationError,
    rules::{
        TableConstraint, TableRule, ValidationConfig, ValidationReport, ValidationResult,
        validation::CheckScope,
    },
};

pub fn validate_table(
    data: &DataFrame,
    rules: &[TableRule],
    config: ValidationConfig,
) -> ValidationReport {
    let results: Vec<ValidationResult> = rules
        .iter()
        .map(|rule| {
            validate_table_with_rule(data, rule, &config).unwrap_or_else(|e| {
                ValidationResult::failed(
                    rule.constraint.to_string().as_str(),
                    &e.to_string(),
                    None,
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

fn validate_table_with_rule(
    data: &DataFrame,
    rule: &TableRule,
    _: &ValidationConfig,
) -> Result<ValidationResult, ValidationError> {
    match &rule.constraint {
        TableConstraint::ShapeEquals { rows, columns } => {
            Ok(check_table_shape(data, rule, *rows, *columns))
        }
        TableConstraint::RowsCountBetween { min, max } => {
            Ok(check_table_rows_count_between(data, rule, *min, *max))
        }
        TableConstraint::RowsCountGreaterOrEqual(count) => {
            Ok(check_table_rows_ge(data, rule, *count))
        }
        TableConstraint::RowCountGreaterThan(count) => Ok(check_table_rows_gt(data, rule, *count)),
        TableConstraint::RowsCountLessOrEqual(count) => Ok(check_table_rows_le(data, rule, *count)),
        TableConstraint::RowCountLessThan(count) => Ok(check_table_rows_lt(data, rule, *count)),
        TableConstraint::ColumnsCountBetween { min, max } => {
            Ok(check_columns_count_between(data, rule, *min, *max))
        }
        TableConstraint::ColumnsCountGreaterOrEqual(v) => {
            Ok(check_table_columns_count_ge(data, rule, *v))
        }
        TableConstraint::ColumnsCountGreaterThan(v) => {
            Ok(check_table_columns_count_gt(data, rule, *v))
        }
        TableConstraint::ColumnsCountLessOrEqual(v) => {
            Ok(check_table_columns_count_le(data, rule, *v))
        }
        TableConstraint::ColumnsCountLessThan(v) => {
            Ok(check_table_columns_count_lt(data, rule, *v))
        }
        TableConstraint::ColumnsExist(target_cols) => {
            Ok(check_columns_exist(data, rule, target_cols))
        }
    }
}

fn check_table_shape(
    data: &DataFrame,
    rule: &TableRule,
    target_rows: usize,
    target_cols: usize,
) -> ValidationResult {
    let (rows, cols) = data.shape();
    if rows != target_rows {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("expected {} rows, got {}", target_rows, rows),
            None,
            None,
            None,
        )
    } else if cols != target_cols {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("expected {} columns, got {}", target_cols, cols),
            None,
            None,
            None,
        )
    } else {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            None,
            CheckScope::Table,
        )
    }
}

fn check_table_rows_count_between(
    data: &DataFrame,
    rule: &TableRule,
    min_rows: usize,
    max_rows: usize,
) -> ValidationResult {
    let rows = data.shape().0;
    if rows >= min_rows && rows <= max_rows {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            None,
            CheckScope::Table,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "expected rows between: ({}, {}). got: {}",
                min_rows, max_rows, rows
            ),
            None,
            None,
            None,
        )
    }
}

fn check_table_rows_ge(data: &DataFrame, rule: &TableRule, target_rows: usize) -> ValidationResult {
    let rows = data.shape().0;
    if rows >= target_rows {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            None,
            CheckScope::Table,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "expected rows greater or equal: {}. got: {}",
                target_rows, rows
            ),
            None,
            None,
            None,
        )
    }
}

fn check_table_rows_gt(data: &DataFrame, rule: &TableRule, target_rows: usize) -> ValidationResult {
    let rows = data.shape().0;
    if rows > target_rows {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            None,
            CheckScope::Table,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("expected rows greater: {}. got: {}", target_rows, rows),
            None,
            None,
            None,
        )
    }
}

fn check_table_rows_lt(data: &DataFrame, rule: &TableRule, target_rows: usize) -> ValidationResult {
    let rows = data.shape().0;
    if rows < target_rows {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            None,
            CheckScope::Table,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!("expected rows less than: {}. got: {}", target_rows, rows),
            None,
            None,
            None,
        )
    }
}

fn check_table_rows_le(data: &DataFrame, rule: &TableRule, target_rows: usize) -> ValidationResult {
    let rows = data.shape().0;
    if rows <= target_rows {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            None,
            CheckScope::Table,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "expected rows less or equal: {}. got: {}",
                target_rows, rows
            ),
            None,
            None,
            None,
        )
    }
}

fn check_columns_count_between(
    data: &DataFrame,
    rule: &TableRule,
    min_columns: usize,
    max_columns: usize,
) -> ValidationResult {
    let cols = data.shape().1;
    if cols >= min_columns && cols <= max_columns {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            None,
            CheckScope::Table,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "expected columns between: ({}, {}). got: {}",
                min_columns, max_columns, cols
            ),
            None,
            None,
            None,
        )
    }
}

fn check_table_columns_count_ge(
    data: &DataFrame,
    rule: &TableRule,
    target_columns: usize,
) -> ValidationResult {
    let cols = data.shape().1;
    if cols >= target_columns {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            None,
            CheckScope::Table,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "expected columns greater or equal: {}. got: {}",
                target_columns, cols
            ),
            None,
            None,
            None,
        )
    }
}

fn check_table_columns_count_gt(
    data: &DataFrame,
    rule: &TableRule,
    target_columns: usize,
) -> ValidationResult {
    let cols = data.shape().1;
    if cols > target_columns {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            None,
            CheckScope::Table,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "expected columns greater than: {}. got: {}",
                target_columns, cols
            ),
            None,
            None,
            None,
        )
    }
}

fn check_table_columns_count_le(
    data: &DataFrame,
    rule: &TableRule,
    target_columns: usize,
) -> ValidationResult {
    let cols = data.shape().1;
    if cols <= target_columns {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            None,
            CheckScope::Table,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "expected columns less or equal: {}. got: {}",
                target_columns, cols
            ),
            None,
            None,
            None,
        )
    }
}

fn check_table_columns_count_lt(
    data: &DataFrame,
    rule: &TableRule,
    target_columns: usize,
) -> ValidationResult {
    let cols = data.shape().1;
    if cols < target_columns {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            None,
            CheckScope::Table,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "expected columns less than: {}. got: {}",
                target_columns, cols
            ),
            None,
            None,
            None,
        )
    }
}

fn check_columns_exist(
    data: &DataFrame,
    rule: &TableRule,
    target_columns: &[String],
) -> ValidationResult {
    let missing: Vec<&String> = target_columns
        .iter()
        .filter(|&col| !data.headers.contains(col))
        .collect();
    if missing.is_empty() {
        ValidationResult::passed(
            rule.constraint.to_string().as_str(),
            None,
            CheckScope::Table,
        )
    } else {
        ValidationResult::failed(
            rule.constraint.to_string().as_str(),
            &format!(
                "missing columns: {}",
                missing
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            None,
            None,
            None,
        )
    }
}
