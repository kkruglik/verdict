use verdict_core::{
    csv_loader::DatasetCsvExt,
    dataframe::{DataFrame, DataType, Field, Schema, ValuesSet},
    rules::{
        ColumnConstraint, ColumnRule, ColumnRuleBuilder, Operand, ValidationConfig, validate,
    },
};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, ValueEnum};
use serde::Deserialize;
use serde_json::Value;
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

    #[arg(
        long,
        default_value_t = 100,
        help = "Maximum number of failed samples to include in the report"
    )]
    max_failed_samples: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct SchemaConfig {
    pub columns: Vec<ColumnConfig>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct ColumnConfig {
    name: String,
    dtype: DtypeConfig,
    constraints: Option<Vec<ConstraintConfig>>,
    format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum DtypeConfig {
    Int,
    Float,
    Str,
    Bool,
    Date,
    DateTime,
}

impl From<DtypeConfig> for DataType {
    fn from(d: DtypeConfig) -> Self {
        match d {
            DtypeConfig::Int => DataType::Int,
            DtypeConfig::Float => DataType::Float,
            DtypeConfig::Str => DataType::String,
            DtypeConfig::Bool => DataType::Bool,
            DtypeConfig::DateTime => DataType::DateTime,
            DtypeConfig::Date => DataType::Date,
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

fn parse_str_array(value: &Value, constraint: &str) -> Result<(String, String)> {
    let arr = value
        .as_array()
        .ok_or_else(|| anyhow!("{}: expected array [min, max], got {}", constraint, value))?;
    if arr.len() != 2 {
        bail!("{}: expected 2 elements, got {}", constraint, arr.len());
    }
    let min = arr[0]
        .as_str()
        .ok_or_else(|| anyhow!("{}: min must be a string", constraint))?;
    let max = arr[1]
        .as_str()
        .ok_or_else(|| anyhow!("{}: max must be a string", constraint))?;
    Ok((min.to_string(), max.to_string()))
}

fn parse_is_in(value: &Value) -> Result<ValuesSet> {
    let arr = value
        .as_array()
        .ok_or_else(|| anyhow!("is_in: expected array, got {}", value))?;
    if arr.iter().all(|v| v.as_i64().is_some()) {
        Ok(ValuesSet::Int64Set(
            arr.iter().map(|v| v.as_i64().unwrap()).collect(),
        ))
    } else if arr.iter().all(|v| v.as_f64().is_some()) {
        Ok(ValuesSet::FloatSet(
            arr.iter().map(|v| v.as_f64().unwrap()).collect(),
        ))
    } else if arr.iter().all(|v| v.as_str().is_some()) {
        Ok(ValuesSet::StrSet(
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

fn parse_constraint(constraint: &str, value: &Value) -> Result<ColumnConstraint> {
    match constraint {
        "not_null" => Ok(ColumnConstraint::NotNull),
        "unique" => Ok(ColumnConstraint::Unique),
        "gt" => Ok(ColumnConstraint::GreaterThan(parse_operand(value)?)),
        "ge" => Ok(ColumnConstraint::GreaterThanOrEqual(parse_operand(value)?)),
        "lt" => Ok(ColumnConstraint::LessThan(parse_operand(value)?)),
        "le" => Ok(ColumnConstraint::LessThanOrEqual(parse_operand(value)?)),
        "eq" => Ok(ColumnConstraint::Equal(parse_operand(value)?)),
        "between" => {
            let (min, max) = parse_operand_array(value)?;
            Ok(ColumnConstraint::Between { min, max })
        }
        "is_in" => Ok(ColumnConstraint::InSet(parse_is_in(value)?)),
        "contains" => Ok(ColumnConstraint::Contains(parse_str_value(
            value, constraint,
        )?)),
        "starts_with" => Ok(ColumnConstraint::StartsWith(parse_str_value(
            value, constraint,
        )?)),
        "ends_with" => Ok(ColumnConstraint::EndsWith(parse_str_value(
            value, constraint,
        )?)),
        "matches_regex" => Ok(ColumnConstraint::MatchesRegex(parse_str_value(
            value, constraint,
        )?)),
        "length_between" => {
            let (min, max) = parse_length_between(value)?;
            Ok(ColumnConstraint::LengthBetween { min, max })
        }
        "after" => Ok(ColumnConstraint::After(parse_str_value(value, constraint)?)),
        "before" => Ok(ColumnConstraint::Before(parse_str_value(
            value, constraint,
        )?)),
        "between_dates" => {
            let (min, max) = parse_str_array(value, constraint)?;
            Ok(ColumnConstraint::BetweenDates { min, max })
        }
        _ => bail!(
            "unsupported constraint '{}'. valid: not_null, unique, gt, ge, lt, le, eq, between, is_in, contains, starts_with, ends_with, matches_regex, length_between",
            constraint
        ),
    }
}

#[cfg(test)]
mod tests;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if !cli.filename.exists() {
        bail!("dataset file not found: {}", cli.filename.display());
    }

    if !cli.schema.exists() {
        bail!("schema file not found: {}", cli.schema.display());
    }

    let is_yaml = cli
        .schema
        .extension()
        .map(|e| e == "yaml" || e == "yml")
        .unwrap_or(false);

    let schema_file = File::open(&cli.schema).context("failed to open schema file")?;
    let reader = BufReader::new(schema_file);
    let config: SchemaConfig = if is_yaml {
        serde_yaml::from_reader(reader).context("failed to parse schema file as YAML")?
    } else {
        serde_json::from_reader(reader).context("failed to parse schema file as JSON")?
    };

    let mut dataset_rules: Vec<ColumnRule> = Vec::new();

    for col_config in &config.columns {
        if let Some(constraints) = &col_config.constraints {
            let mut col_constraints: Vec<ColumnConstraint> = Vec::new();
            for c in constraints {
                col_constraints.push(parse_constraint(&c.constraint, &c.value).context(
                    format!(
                        "invalid constraint '{}' on column '{}'",
                        &c.constraint, &col_config.name
                    ),
                )?);
            }
            let col_rules = ColumnRuleBuilder {
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
            .map(|c| Field::new(&c.name, c.dtype.clone().into(), c.format.as_deref()))
            .collect(),
    );

    let data = DataFrame::from_csv(&cli.filename, &data_schema).context(format!(
        "failed to load dataset: {}",
        cli.filename.display()
    ))?;

    let report = validate(
        &data,
        &dataset_rules,
        ValidationConfig {
            max_failed_samples: cli.max_failed_samples,
        },
    );

    match cli.format {
        OutputFormat::Text => {
            println!("{}", report);
        }
        OutputFormat::Json => println!("{}", report.to_json()),
    }

    if !&report.passed {
        std::process::exit(1);
    }

    Ok(())
}
