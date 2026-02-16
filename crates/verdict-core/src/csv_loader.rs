use crate::dataset::{
    BoolColumn, Column, DataType, Dataset, FloatColumn, IntColumn, Schema, StrColumn,
};
use csv::Reader;

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
}

pub trait DatasetCsvExt {
    fn from_csv(path: &str, schema: &Schema) -> Result<Dataset, CsvLoadingError>;
}

impl DatasetCsvExt for Dataset {
    fn from_csv(path: &str, schema: &Schema) -> Result<Dataset, CsvLoadingError> {
        let mut reader = Reader::from_path(path)?;

        let headers: Vec<String> = reader.headers()?.iter().map(|s| s.to_string()).collect();
        let num_columns = headers.len();
        let mut raw_columns: Vec<Vec<Option<String>>> = vec![vec![]; num_columns];

        for record in reader.records() {
            let record = record?;
            for (i, field) in record.iter().enumerate() {
                let value = if field.is_empty() {
                    None
                } else {
                    Some(field.to_string())
                };
                raw_columns[i].push(value);
            }
        }

        let mut columns: Vec<Column> = Vec::with_capacity(schema.fields.len());

        for (col_idx, field) in schema.fields.iter().enumerate() {
            let raw_col = &raw_columns[col_idx];

            let column = match field.dtype {
                DataType::Int => {
                    let parsed: Result<Vec<Option<i64>>, _> = raw_col
                        .iter()
                        .enumerate()
                        .map(|(row_idx, val)| {
                            val.as_ref()
                                .map(|s| {
                                    s.parse::<i64>().map_err(|_| CsvLoadingError::ParseError {
                                        column: field.name.clone(),
                                        row: row_idx,
                                        value: s.clone(),
                                        expected: "Int".to_string(),
                                    })
                                })
                                .transpose()
                        })
                        .collect();
                    Column::Int(IntColumn(parsed?))
                }

                DataType::Float => {
                    let parsed: Result<Vec<Option<f64>>, _> = raw_col
                        .iter()
                        .enumerate()
                        .map(|(row_idx, val)| {
                            val.as_ref()
                                .map(|s| {
                                    s.parse::<f64>().map_err(|_| CsvLoadingError::ParseError {
                                        column: field.name.clone(),
                                        row: row_idx,
                                        value: s.clone(),
                                        expected: "Float".to_string(),
                                    })
                                })
                                .transpose()
                        })
                        .collect();
                    Column::Float(FloatColumn(parsed?))
                }

                DataType::Str => {
                    let parsed: Vec<Option<String>> = raw_col.clone();
                    Column::Str(StrColumn(parsed))
                }

                DataType::Bool => {
                    let parsed: Result<Vec<Option<bool>>, _> = raw_col
                        .iter()
                        .enumerate()
                        .map(|(row_idx, val)| {
                            val.as_ref()
                                .map(|s| {
                                    parse_bool(s).ok_or_else(|| CsvLoadingError::ParseError {
                                        column: field.name.clone(),
                                        row: row_idx,
                                        value: s.clone(),
                                        expected: "Bool".to_string(),
                                    })
                                })
                                .transpose()
                        })
                        .collect();
                    Column::Bool(BoolColumn(parsed?))
                }
            };

            columns.push(column);
        }

        Ok(Dataset { headers, columns })
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}
