use std::path::Path;

use crate::dataset::{
    BoolColumn, Column, DataType, Dataset, FloatColumn, IntColumn, Schema, StrColumn,
};
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
    fn from_csv(path: &Path, schema: &Schema) -> Result<Dataset, CsvLoadingError>;
}

enum ColBuilder {
    Int(Vec<Option<i64>>),
    Float(Vec<Option<f64>>),
    Str(Vec<Option<String>>),
    Bool(Vec<Option<bool>>),
}

impl DatasetCsvExt for Dataset {
    fn from_csv(path: &Path, schema: &Schema) -> Result<Dataset, CsvLoadingError> {
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
                DataType::Str => "Str",
                DataType::Bool => "Bool",
            })
            .collect();

        let mut columns_data: Vec<ColBuilder> = schema
            .fields
            .iter()
            .map(|f| match f.dtype {
                DataType::Int => ColBuilder::Int(Vec::with_capacity(4096)),
                DataType::Float => ColBuilder::Float(Vec::with_capacity(4096)),
                DataType::Str => ColBuilder::Str(Vec::with_capacity(4096)),
                DataType::Bool => ColBuilder::Bool(Vec::with_capacity(4096)),
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
                }
            }
        }

        let columns = columns_data
            .into_iter()
            .map(|b| match b {
                ColBuilder::Int(v) => Column::Int(IntColumn(v)),
                ColBuilder::Float(v) => Column::Float(FloatColumn(v)),
                ColBuilder::Str(v) => Column::Str(StrColumn(v)),
                ColBuilder::Bool(v) => Column::Bool(BoolColumn(v)),
            })
            .collect();

        Ok(Dataset { headers, columns })
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s {
        "true" | "True" | "TRUE" | "1" => Some(true),
        "false" | "False" | "FALSE" | "0" => Some(false),
        _ => None,
    }
}
