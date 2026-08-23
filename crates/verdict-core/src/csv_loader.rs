use std::path::Path;

use crate::dataframe::{
    BoolColumn, Column, DataFrame, DataType, DateColumn, DateTimeColumn, FloatColumn, IntColumn,
    Schema, StringColumn, column::TimeColumn, naive_date_to_i32, naive_datetime_to_i64,
    ops::naive_time_to_i64,
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use csv::ReaderBuilder;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CsvLoadingError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    CsvError(#[from] csv::Error),

    #[error("Failed to parse column '{column}' row {row}: '{value}' is not a valid {expected}")]
    ParseError {
        column: String,
        row: usize,
        value: String,
        expected: String,
    },

    #[error("Schema does not match CSV: expected {expected} columns, found {found}")]
    ShapeError { expected: usize, found: usize },
}

pub trait DatasetCsvExt {
    fn from_csv(path: &Path, schema: &Schema) -> Result<DataFrame, CsvLoadingError>;
}

enum ColBuilder {
    Int(Vec<Option<i64>>),
    Float(Vec<Option<f64>>),
    Str(Vec<Option<String>>),
    Bool(Vec<Option<bool>>),
    Date(Vec<Option<i32>>, Option<String>),
    DateTime(Vec<Option<i64>>, Option<String>),
    Time(Vec<Option<i64>>, Option<String>),
}

impl DatasetCsvExt for DataFrame {
    fn from_csv(path: &Path, schema: &Schema) -> Result<DataFrame, CsvLoadingError> {
        let mut reader = ReaderBuilder::new()
            .buffer_capacity(512 * 1024)
            .from_path(path)?;

        let headers: Vec<String> = reader.headers()?.iter().map(|s| s.to_string()).collect();

        if schema.fields.len() != headers.len() {
            return Err(CsvLoadingError::ShapeError {
                expected: schema.fields.len(),
                found: headers.len(),
            });
        }

        let field_names: Vec<String> = schema.fields.iter().map(|f| f.name.clone()).collect();
        let field_expected: Vec<&'static str> = schema
            .fields
            .iter()
            .map(|f| match f.dtype {
                DataType::Int => "Int",
                DataType::Float => "Float",
                DataType::String => "Str",
                DataType::Bool => "Bool",
                DataType::DateTime => "DateTime",
                DataType::Date => "Date",
                DataType::Time => "Time",
            })
            .collect();

        let mut columns_data: Vec<ColBuilder> = schema
            .fields
            .iter()
            .map(|f| match f.dtype {
                DataType::Int => ColBuilder::Int(Vec::with_capacity(4096)),
                DataType::Float => ColBuilder::Float(Vec::with_capacity(4096)),
                DataType::String => ColBuilder::Str(Vec::with_capacity(4096)),
                DataType::Bool => ColBuilder::Bool(Vec::with_capacity(4096)),
                DataType::DateTime => {
                    ColBuilder::DateTime(Vec::with_capacity(4096), f.format.clone())
                }
                DataType::Date => ColBuilder::Date(Vec::with_capacity(4096), f.format.clone()),
                DataType::Time => ColBuilder::Time(Vec::with_capacity(4096), f.format.clone()),
            })
            .collect();

        for (row_idx, record) in reader.records().enumerate() {
            let record = record?;
            for (col_idx, val) in record.iter().enumerate() {
                if val.is_empty() {
                    match &mut columns_data[col_idx] {
                        ColBuilder::Int(v) => v.push(None),
                        ColBuilder::Float(v) => v.push(None),
                        ColBuilder::Str(v) => v.push(None),
                        ColBuilder::Bool(v) => v.push(None),
                        ColBuilder::DateTime(v, _) => v.push(None),
                        ColBuilder::Date(v, _) => v.push(None),
                        ColBuilder::Time(v, _) => v.push(None),
                    }
                    continue;
                }

                match &mut columns_data[col_idx] {
                    ColBuilder::Int(v) => {
                        v.push(Some(val.parse::<i64>().map_err(|_| {
                            CsvLoadingError::ParseError {
                                column: field_names[col_idx].clone(),
                                row: row_idx,
                                value: val.to_string(),
                                expected: field_expected[col_idx].to_string(),
                            }
                        })?));
                    }
                    ColBuilder::Float(v) => {
                        v.push(Some(val.parse::<f64>().map_err(|_| {
                            CsvLoadingError::ParseError {
                                column: field_names[col_idx].clone(),
                                row: row_idx,
                                value: val.to_string(),
                                expected: field_expected[col_idx].to_string(),
                            }
                        })?));
                    }
                    ColBuilder::Str(v) => {
                        v.push(Some(val.to_string()));
                    }
                    ColBuilder::Bool(v) => {
                        v.push(Some(parse_bool(val).ok_or_else(|| {
                            CsvLoadingError::ParseError {
                                column: field_names[col_idx].clone(),
                                row: row_idx,
                                value: val.to_string(),
                                expected: field_expected[col_idx].to_string(),
                            }
                        })?));
                    }
                    ColBuilder::DateTime(v, fmt) => {
                        let fmt_str = fmt.as_deref().unwrap_or("%Y-%m-%dT%H:%M:%S");
                        let naive_dt =
                            NaiveDateTime::parse_from_str(val, fmt_str).map_err(|_| {
                                CsvLoadingError::ParseError {
                                    column: field_names[col_idx].clone(),
                                    row: row_idx,
                                    value: val.to_string(),
                                    expected: field_expected[col_idx].to_string(),
                                }
                            })?;
                        v.push(Some(naive_datetime_to_i64(&naive_dt)));
                    }
                    ColBuilder::Date(v, fmt) => {
                        let fmt_str = fmt.as_deref().unwrap_or("%Y-%m-%d");
                        let naive_dt = NaiveDate::parse_from_str(val, fmt_str).map_err(|_| {
                            CsvLoadingError::ParseError {
                                column: field_names[col_idx].clone(),
                                row: row_idx,
                                value: val.to_string(),
                                expected: field_expected[col_idx].to_string(),
                            }
                        })?;
                        v.push(Some(naive_date_to_i32(&naive_dt)));
                    }
                    ColBuilder::Time(v, fmt) => {
                        let fmt_str = fmt.as_deref().unwrap_or("%H:%M:%S");
                        let naive_time = NaiveTime::parse_from_str(val, fmt_str).map_err(|_| {
                            CsvLoadingError::ParseError {
                                column: field_names[col_idx].clone(),
                                row: row_idx,
                                value: val.to_string(),
                                expected: field_expected[col_idx].to_string(),
                            }
                        })?;
                        v.push(Some(naive_time_to_i64(&naive_time)));
                    }
                }
            }
        }

        let columns = columns_data
            .into_iter()
            .map(|b| match b {
                ColBuilder::Int(v) => Column::Int(IntColumn::new(v)),
                ColBuilder::Float(v) => Column::Float(FloatColumn::new(v)),
                ColBuilder::Str(v) => Column::Str(StringColumn::new(v)),
                ColBuilder::Bool(v) => Column::Bool(BoolColumn::new(v)),
                ColBuilder::DateTime(v, _) => Column::DateTime(DateTimeColumn::new(v)),
                ColBuilder::Date(v, _) => Column::Date(DateColumn::new(v)),
                ColBuilder::Time(v, _) => Column::Time(TimeColumn::new(v)),
            })
            .collect();

        Ok(DataFrame { headers, columns })
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s {
        "true" | "True" | "TRUE" | "1" => Some(true),
        "false" | "False" | "FALSE" | "0" => Some(false),
        _ => None,
    }
}
