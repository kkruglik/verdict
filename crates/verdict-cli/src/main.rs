use verdict_core::{
    csv_loader::DatasetCsvExt,
    dataset::{DataType, Dataset, Field, InSetValues, Schema},
    rules::{Constraint, Operand, Rule, RuleBuilder, validate},
};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, ValueEnum};
use serde::Deserialize;
use serde_json::{Value, from_reader, json, to_string_pretty};
use std::{fs::File, io::BufReader, path::PathBuf};

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Parser, Debug)]
#[command(name = "verdict-cli")]
struct Cli {
    #[arg(help = "Dataset file path")]
    filename: PathBuf,

    #[arg(help = "Schema file path")]
    schema: PathBuf,

    #[arg(long, value_enum, default_value = "json", help = "Output format")]
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct ValidationConfig {
    pub columns: Vec<ColumnConfig>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct ColumnConfig {
    name: String,
    dtype: DtypeConfig,
    constraints: Option<Vec<ConstraintConfig>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum DtypeConfig {
    Int,
    Float,
    Str,
    Bool,
}

impl From<DtypeConfig> for DataType {
    fn from(d: DtypeConfig) -> Self {
        match d {
            DtypeConfig::Int => DataType::Int,
            DtypeConfig::Float => DataType::Float,
            DtypeConfig::Str => DataType::Str,
            DtypeConfig::Bool => DataType::Bool,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct ConstraintConfig {
    constraint: String,
    value: Value,
}

fn parse_operand(value: &Value) -> Result<Operand> {
    if let Some(v) = value.as_f64() {
        Ok(Operand::Num(v))
    } else if let Some(v) = value.as_str() {
        Ok(Operand::Str(v.to_string()))
    } else if let Some(v) = value.get("col").and_then(|col| col.as_str()) {
        Ok(Operand::Column(v.to_string()))
    } else {
        bail!(
            "invalid operand: {}. expected a number, string, or column reference {{\"col\": \"name\"}}",
            value
        )
    }
}

fn parse_operand_array(value: &Value) -> Result<(Operand, Operand)> {
    let arr = value
        .as_array()
        .ok_or_else(|| anyhow!("between: expected array [min, max], got {}", value))?;
    if arr.len() != 2 {
        bail!("between: expected 2 elements, got {}", arr.len());
    }
    Ok((parse_operand(&arr[0])?, parse_operand(&arr[1])?))
}

fn parse_str_value(value: &Value, constraint: &str) -> Result<String> {
    value
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("expected string value for constraint '{}'", constraint))
}

fn parse_is_in(value: &Value) -> Result<InSetValues> {
    let arr = value
        .as_array()
        .ok_or_else(|| anyhow!("is_in: expected array, got {}", value))?;
    if arr.iter().all(|v| v.as_i64().is_some()) {
        Ok(InSetValues::IntSet(
            arr.iter().map(|v| v.as_i64().unwrap()).collect(),
        ))
    } else if arr.iter().all(|v| v.as_f64().is_some()) {
        Ok(InSetValues::FloatSet(
            arr.iter().map(|v| v.as_f64().unwrap()).collect(),
        ))
    } else if arr.iter().all(|v| v.as_str().is_some()) {
        Ok(InSetValues::StrSet(
            arr.iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
        ))
    } else {
        bail!("is_in values must be all integers, floats, or strings")
    }
}

fn parse_length_between(value: &Value) -> Result<(usize, usize)> {
    let arr = value
        .as_array()
        .ok_or_else(|| anyhow!("length_between: expected array [min, max], got {}", value))?;
    if arr.len() != 2 {
        bail!("length_between: expected 2 elements, got {}", arr.len());
    }
    let min = arr[0]
        .as_u64()
        .ok_or_else(|| anyhow!("length_between: min must be an unsigned integer"))?
        as usize;
    let max = arr[1]
        .as_u64()
        .ok_or_else(|| anyhow!("length_between: max must be an unsigned integer"))?
        as usize;
    Ok((min, max))
}

fn parse_constraint(constraint: &str, value: &Value) -> Result<Constraint> {
    match constraint {
        "not_null" => Ok(Constraint::NotNull),
        "unique" => Ok(Constraint::Unique),
        "gt" => Ok(Constraint::GreaterThan(parse_operand(value)?)),
        "ge" => Ok(Constraint::GreaterThanOrEqual(parse_operand(value)?)),
        "lt" => Ok(Constraint::LessThan(parse_operand(value)?)),
        "le" => Ok(Constraint::LessThanOrEqual(parse_operand(value)?)),
        "eq" => Ok(Constraint::Equal(parse_operand(value)?)),
        "between" => {
            let (min, max) = parse_operand_array(value)?;
            Ok(Constraint::Between { min, max })
        }
        "is_in" => Ok(Constraint::InSet(parse_is_in(value)?)),
        "contains" => Ok(Constraint::Contains(parse_str_value(value, constraint)?)),
        "starts_with" => Ok(Constraint::StartsWith(parse_str_value(value, constraint)?)),
        "ends_with" => Ok(Constraint::EndsWith(parse_str_value(value, constraint)?)),
        "matches_regex" => Ok(Constraint::MatchesRegex(parse_str_value(
            value, constraint,
        )?)),
        "length_between" => {
            let (min, max) = parse_length_between(value)?;
            Ok(Constraint::LengthBetween { min, max })
        }
        _ => bail!(
            "unsupported constraint '{}'. valid: not_null, unique, gt, ge, lt, le, eq, between, is_in, contains, starts_with, ends_with, matches_regex, length_between",
            constraint
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- parse_operand ---

    #[test]
    fn parse_operand_returns_num_for_integer() {
        // Arrange
        let value = json!(42);
        // Act
        let result = parse_operand(&value).unwrap();
        // Assert
        assert!(matches!(result, Operand::Num(v) if v == 42.0));
    }

    #[test]
    fn parse_operand_returns_num_for_float() {
        // Arrange
        let value = json!(3.14);
        // Act
        let result = parse_operand(&value).unwrap();
        // Assert
        assert!(matches!(result, Operand::Num(v) if (v - 3.14).abs() < f64::EPSILON));
    }

    #[test]
    fn parse_operand_returns_str_for_string() {
        // Arrange
        let value = json!("hello");
        // Act
        let result = parse_operand(&value).unwrap();
        // Assert
        assert!(matches!(result, Operand::Str(s) if s == "hello"));
    }

    #[test]
    fn parse_operand_returns_column_for_col_object() {
        // Arrange
        let value = json!({"col": "user_id"});
        // Act
        let result = parse_operand(&value).unwrap();
        // Assert
        assert!(matches!(result, Operand::Column(s) if s == "user_id"));
    }

    #[test]
    fn parse_operand_errors_on_boolean() {
        // Arrange
        let value = json!(true);
        // Act
        let result = parse_operand(&value);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn parse_operand_errors_on_array() {
        // Arrange
        let value = json!([1, 2]);
        // Act
        let result = parse_operand(&value);
        // Assert
        assert!(result.is_err());
    }

    // --- parse_is_in ---

    #[test]
    fn parse_is_in_returns_int_set() {
        // Arrange
        let value = json!([1, 2, 3]);
        // Act
        let result = parse_is_in(&value).unwrap();
        // Assert
        assert!(matches!(result, InSetValues::IntSet(v) if v == vec![1, 2, 3]));
    }

    #[test]
    fn parse_is_in_returns_float_set() {
        // Arrange
        let value = json!([1.1, 2.2, 3.3]);
        // Act
        let result = parse_is_in(&value).unwrap();
        // Assert
        assert!(matches!(result, InSetValues::FloatSet(_)));
    }

    #[test]
    fn parse_is_in_returns_str_set() {
        // Arrange
        let value = json!(["a", "b", "c"]);
        // Act
        let result = parse_is_in(&value).unwrap();
        // Assert
        assert!(
            matches!(result, InSetValues::StrSet(v) if v == vec!["a".to_string(), "b".to_string(), "c".to_string()])
        );
    }

    #[test]
    fn parse_is_in_errors_on_mixed_types() {
        // Arrange
        let value = json!([1, "two", 3.0]);
        // Act
        let result = parse_is_in(&value);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn parse_is_in_errors_on_non_array() {
        // Arrange
        let value = json!(42);
        // Act
        let result = parse_is_in(&value);
        // Assert
        assert!(result.is_err());
    }

    // --- parse_length_between ---

    #[test]
    fn parse_length_between_returns_min_max() {
        // Arrange
        let value = json!([2, 10]);
        // Act
        let (min, max) = parse_length_between(&value).unwrap();
        // Assert
        assert_eq!(min, 2);
        assert_eq!(max, 10);
    }

    #[test]
    fn parse_length_between_accepts_zero_min() {
        // Arrange
        let value = json!([0, 5]);
        // Act
        let (min, _) = parse_length_between(&value).unwrap();
        // Assert
        assert_eq!(min, 0);
    }

    #[test]
    fn parse_length_between_errors_on_wrong_length() {
        // Arrange
        let value = json!([1, 2, 3]);
        // Act
        let result = parse_length_between(&value);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn parse_length_between_errors_on_float_values() {
        // Arrange
        let value = json!([1.5, 10.0]);
        // Act
        let result = parse_length_between(&value);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn parse_length_between_errors_on_non_array() {
        // Arrange
        let value = json!("not an array");
        // Act
        let result = parse_length_between(&value);
        // Assert
        assert!(result.is_err());
    }

    // --- parse_constraint ---

    #[test]
    fn parse_constraint_not_null() {
        let result = parse_constraint("not_null", &json!(true)).unwrap();
        assert!(matches!(result, Constraint::NotNull));
    }

    #[test]
    fn parse_constraint_unique() {
        let result = parse_constraint("unique", &json!(true)).unwrap();
        assert!(matches!(result, Constraint::Unique));
    }

    #[test]
    fn parse_constraint_gt_with_number() {
        let result = parse_constraint("gt", &json!(5)).unwrap();
        assert!(matches!(result, Constraint::GreaterThan(Operand::Num(v)) if v == 5.0));
    }

    #[test]
    fn parse_constraint_ge_with_column_ref() {
        let result = parse_constraint("ge", &json!({"col": "other"})).unwrap();
        assert!(
            matches!(result, Constraint::GreaterThanOrEqual(Operand::Column(s)) if s == "other")
        );
    }

    #[test]
    fn parse_constraint_between_two_numbers() {
        let result = parse_constraint("between", &json!([0, 100])).unwrap();
        assert!(matches!(
            result,
            Constraint::Between {
                min: Operand::Num(_),
                max: Operand::Num(_),
            }
        ));
    }

    #[test]
    fn parse_constraint_is_in_integers() {
        let result = parse_constraint("is_in", &json!([1, 2, 3])).unwrap();
        assert!(matches!(result, Constraint::InSet(InSetValues::IntSet(_))));
    }

    #[test]
    fn parse_constraint_contains_string() {
        let result = parse_constraint("contains", &json!("foo")).unwrap();
        assert!(matches!(result, Constraint::Contains(s) if s == "foo"));
    }

    #[test]
    fn parse_constraint_length_between() {
        let result = parse_constraint("length_between", &json!([1, 50])).unwrap();
        assert!(matches!(
            result,
            Constraint::LengthBetween { min: 1, max: 50 }
        ));
    }

    #[test]
    fn parse_constraint_errors_on_unknown_name() {
        let result = parse_constraint("nonexistent", &json!(null));
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("unsupported constraint")
        );
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if !cli.filename.exists() {
        bail!("dataset file not found: {}", cli.filename.display());
    }

    if !cli.schema.exists() {
        bail!("schema file not found: {}", cli.schema.display());
    }

    let config_json = File::open(cli.schema).context("failed to open schema file")?;
    let reader = BufReader::new(config_json);
    let config: ValidationConfig = from_reader(reader).context("failed to parse schema file")?;

    let mut dataset_rules: Vec<Rule> = Vec::new();

    for col_config in &config.columns {
        if let Some(constraints) = &col_config.constraints {
            let mut col_constraints: Vec<Constraint> = Vec::new();
            for c in constraints {
                col_constraints.push(parse_constraint(&c.constraint, &c.value).context(
                    format!(
                        "invalid constraint '{}' on column '{}'",
                        &c.constraint, &col_config.name
                    ),
                )?);
            }
            let col_rules = RuleBuilder {
                column: col_config.name.clone(),
                constraint: col_constraints,
            }
            .build();
            dataset_rules.extend_from_slice(&col_rules);
        }
    }

    let data_schema = Schema::new(
        config
            .columns
            .iter()
            .map(|c| Field::new(&c.name, c.dtype.clone().into()))
            .collect(),
    );

    let data = Dataset::from_csv(&cli.filename, &data_schema).context(format!(
        "failed to load dataset: {}",
        cli.filename.display()
    ))?;
    let results = validate(&data, &dataset_rules);
    let any_failed = results.iter().any(|r| !r.passed);

    match cli.format {
        OutputFormat::Text => {
            for r in &results {
                println!("{}", r);
            }
        }
        OutputFormat::Json => {
            let output: Vec<_> = results
                .iter()
                .map(|r| {
                    json!({
                        "column": r.column,
                        "constraint": r.constraint,
                        "passed": r.passed,
                        "failed_count": r.failed_count,
                        "error": r.error,
                    })
                })
                .collect();
            println!("{}", to_string_pretty(&output).unwrap());
        }
    }

    if any_failed {
        std::process::exit(1);
    }

    Ok(())
}
