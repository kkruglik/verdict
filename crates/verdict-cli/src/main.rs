use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use verdict_core::{
    csv_loader::DatasetCsvExt,
    dataframe::{
        DataFrame, DataType, Field, Schema, ValuesSet, naive_date_to_i32, naive_datetime_to_i64,
        ops::naive_time_to_i64,
    },
    parquet_loader::DatasetParquetExt,
    rules::{
        ColumnConstraint, ColumnRule, ColumnRuleBuilder, Operand, TableConstraint, TableRule,
        TableRuleBuilder, ValidationConfig, column_checks::validate_columns,
        table_checks::validate_table,
    },
};

use anyhow::{Context, Ok, Result, anyhow, bail};
use clap::{Parser, ValueEnum};
use serde::Deserialize;
use serde_json::Value;
use std::{fs::File, io::BufReader, path::PathBuf, str::FromStr};

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
struct ValidationCliConfig {
    pub table: Option<TableConfig>,
    pub columns: Vec<ColumnConfig>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct TableConfig {
    constraints: Option<Vec<ConstraintConfig>>,
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
    Time,
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
            DtypeConfig::Time => DataType::Time,
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

fn parse_usize_value(value: &Value, constraint: &str) -> Result<usize> {
    let val = value
        .as_u64()
        .ok_or_else(|| anyhow!("expected usize value for constraint '{}'", constraint))?;
    Ok(val as usize)
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

fn parse_usize_array(value: &Value, constraint: &str) -> Result<(usize, usize)> {
    let arr = value
        .as_array()
        .ok_or_else(|| anyhow!("{}: expected array [min, max], got {}", constraint, value))?;
    if arr.len() != 2 {
        bail!("{}: expected 2 elements, got {}", constraint, arr.len());
    }
    let min = arr[0]
        .as_u64()
        .ok_or_else(|| anyhow!("{}: min must be a integer", constraint))?;
    let max = arr[1]
        .as_u64()
        .ok_or_else(|| anyhow!("{}: max must be a string", constraint))?;
    Ok((min as usize, max as usize))
}

fn parse_is_in(value: &Value, col_dtype: &DtypeConfig) -> Result<ValuesSet> {
    let arr = value
        .as_array()
        .ok_or_else(|| anyhow!("is_in: expected array, got {}", value))?;

    match col_dtype {
        DtypeConfig::Float => arr
            .iter()
            .map(|v| {
                v.as_f64()
                    .ok_or_else(|| anyhow!("is_in: expected float, got {}", v))
            })
            .collect::<Result<Vec<_>>>()
            .map(ValuesSet::FloatSet),
        DtypeConfig::Int => arr
            .iter()
            .map(|v| {
                v.as_i64()
                    .ok_or_else(|| anyhow!("is_in: expected integer, got {}", v))
            })
            .collect::<Result<Vec<_>>>()
            .map(ValuesSet::Int64Set),
        DtypeConfig::Str => arr
            .iter()
            .map(|v| {
                v.as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| anyhow!("is_in: expected string, got {}", v))
            })
            .collect::<Result<Vec<_>>>()
            .map(ValuesSet::StrSet),
        DtypeConfig::DateTime => {
            let t_arr = arr
                .iter()
                .map(|v| {
                    let s = v.as_str().ok_or_else(|| anyhow!("is_in: expected datetime string, got {}", v))?;
                    Ok(NaiveDateTime::from_str(s)?)
                })
                .collect::<Result<Vec<NaiveDateTime>>>()?;
            let output_arr = t_arr.iter().map(naive_datetime_to_i64).collect();
            Ok(ValuesSet::Int64Set(output_arr))
        }
        DtypeConfig::Date => {
            let t_arr = arr
                .iter()
                .map(|v| {
                    let s = v.as_str().ok_or_else(|| anyhow!("is_in: expected date string, got {}", v))?;
                    Ok(NaiveDate::from_str(s)?)
                })
                .collect::<Result<Vec<NaiveDate>>>()?;
            let output_arr = t_arr.iter().map(naive_date_to_i32).collect();
            Ok(ValuesSet::Int32Set(output_arr))
        }
        DtypeConfig::Time => {
            let t_arr = arr
                .iter()
                .map(|v| {
                    let s = v.as_str().ok_or_else(|| anyhow!("is_in: expected time string, got {}", v))?;
                    Ok(NaiveTime::from_str(s)?)
                })
                .collect::<Result<Vec<NaiveTime>>>()?;
            let output_arr = t_arr.iter().map(naive_time_to_i64).collect();
            Ok(ValuesSet::Int64Set(output_arr))
        }
        DtypeConfig::Bool => bail!("is_in is not supported for bool columns"),
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
fn parse_table_constraint(constraint: &str, value: &Value) -> Result<TableConstraint> {
    match constraint.to_lowercase().as_str() {
        "shape_equals" => {
            let (target_rows, target_cols) = parse_usize_array(value, constraint)?;
            Ok(TableConstraint::ShapeEquals {
                rows: target_rows,
                columns: target_cols,
            })
        }
        "rows_count_between" => {
            let (min, max) = parse_usize_array(value, constraint)?;
            Ok(TableConstraint::RowsCountBetween { min, max })
        }
        "rows_count_greater_or_equal" => Ok(TableConstraint::RowsCountGreaterOrEqual(
            parse_usize_value(value, constraint)?,
        )),
        "row_count_greater_than" => Ok(TableConstraint::RowCountGreaterThan(parse_usize_value(
            value, constraint,
        )?)),
        "rows_count_less_or_equal" => Ok(TableConstraint::RowsCountLessOrEqual(parse_usize_value(
            value, constraint,
        )?)),
        "row_count_less_than" => Ok(TableConstraint::RowCountLessThan(parse_usize_value(
            value, constraint,
        )?)),
        "columns_count_between" => {
            let (min, max) = parse_usize_array(value, constraint)?;
            Ok(TableConstraint::ColumnsCountBetween { min, max })
        }
        "columns_count_greater_or_equal" => Ok(TableConstraint::ColumnsCountGreaterOrEqual(
            parse_usize_value(value, constraint)?,
        )),
        "columns_count_greater_than" => Ok(TableConstraint::ColumnsCountGreaterThan(
            parse_usize_value(value, constraint)?,
        )),
        "columns_count_less_or_equal" => Ok(TableConstraint::ColumnsCountLessOrEqual(
            parse_usize_value(value, constraint)?,
        )),
        "columns_count_less_than" => Ok(TableConstraint::ColumnsCountLessThan(parse_usize_value(
            value, constraint,
        )?)),
        "columns_exist" => {
            let arr = value
                .as_array()
                .ok_or_else(|| anyhow!("{}: expected array of column names", constraint))?;
            let cols = arr
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(|s| s.to_string())
                        .ok_or_else(|| anyhow!("{}: column names must be strings", constraint))
                })
                .collect::<Result<Vec<String>>>()?;
            Ok(TableConstraint::ColumnsExist(cols))
        }
        _ => bail!("unknown table constraint: '{}'", constraint),
    }
}

fn parse_column_constraint(
    constraint: &str,
    value: &Value,
    col_dtype: &DtypeConfig,
) -> Result<ColumnConstraint> {
    match constraint.to_lowercase().as_str() {
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
        "is_in" => Ok(ColumnConstraint::InSet(parse_is_in(value, col_dtype)?)),
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
    let config: ValidationCliConfig = if is_yaml {
        serde_yaml::from_reader(reader).context("failed to parse schema file as YAML")?
    } else {
        serde_json::from_reader(reader).context("failed to parse schema file as JSON")?
    };

    let mut columns_rules: Vec<ColumnRule> = Vec::new();

    for col_config in &config.columns {
        if let Some(constraints) = &col_config.constraints {
            let mut col_constraints: Vec<ColumnConstraint> = Vec::new();
            for c in constraints {
                col_constraints.push(
                    parse_column_constraint(&c.constraint, &c.value, &col_config.dtype).context(
                        format!(
                            "invalid constraint '{}' on column '{}'",
                            &c.constraint, &col_config.name
                        ),
                    )?,
                );
            }
            let col_rules = ColumnRuleBuilder {
                column: col_config.name.clone(),
                constraint: col_constraints,
            }
            .build();
            columns_rules.extend_from_slice(&col_rules);
        }
    }

    let data_schema = Schema::new(
        config
            .columns
            .iter()
            .map(|c| Field::new(&c.name, c.dtype.clone().into(), c.format.as_deref()))
            .collect(),
    );

    let data = if cli.filename.extension().is_some_and(|s| s == "csv") {
        DataFrame::from_csv(&cli.filename, &data_schema).context(format!(
            "failed to load dataset: {}",
            cli.filename.display()
        ))?
    } else if cli.filename.extension().is_some_and(|s| s == "parquet") {
        DataFrame::from_parquet(&cli.filename).context(format!(
            "failed to load dataset: {}",
            cli.filename.display()
        ))?
    } else {
        bail!("Unsupported dataset format. Please use parquet or csv.")
    };

    let mut final_report = validate_columns(
        &data,
        &columns_rules,
        ValidationConfig {
            max_failed_samples: cli.max_failed_samples,
        },
    );

    if let Some(table_config) = &config.table {
        let mut table_rules: Vec<TableRule> = Vec::new();

        if let Some(constraints) = &table_config.constraints {
            let mut table_constraints: Vec<TableConstraint> = Vec::new();
            for c in constraints {
                table_constraints.push(
                    parse_table_constraint(&c.constraint, &c.value)
                        .context(format!("invalid constraint '{}' on table", &c.constraint))?,
                );
            }
            let rules = TableRuleBuilder {
                constraint: table_constraints,
            }
            .build();
            table_rules.extend_from_slice(&rules);
        }
        let table_report = validate_table(
            &data,
            &table_rules,
            ValidationConfig {
                max_failed_samples: cli.max_failed_samples,
            },
        );
        final_report = final_report.merge(table_report);
    }

    match cli.format {
        OutputFormat::Text => {
            println!("{}", final_report);
        }
        OutputFormat::Json => {
            println!("{}", final_report.to_json());
        }
    }

    if !&final_report.passed {
        std::process::exit(1);
    }

    Ok(())
}
