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
}

pub trait DatasetCsvExt {
    fn from_csv(path: &str, schema: &Schema) -> Result<Dataset, CsvLoadingError>;
}

enum ColBuilder {
    Int(Vec<Option<i64>>),
    Float(Vec<Option<f64>>),
    Str(Vec<Option<String>>),
    Bool(Vec<Option<bool>>),
}

impl DatasetCsvExt for Dataset {
    fn from_csv(path: &str, schema: &Schema) -> Result<Dataset, CsvLoadingError> {
        let mut reader = ReaderBuilder::new()
            .buffer_capacity(512 * 1024)
            .from_path(path)?;

        let headers: Vec<String> = reader.headers()?.iter().map(|s| s.to_string()).collect();

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

        let mut builders: Vec<ColBuilder> = schema
            .fields
            .iter()
            .map(|f| match f.dtype {
                DataType::Int => ColBuilder::Int(Vec::new()),
                DataType::Float => ColBuilder::Float(Vec::new()),
                DataType::Str => ColBuilder::Str(Vec::new()),
                DataType::Bool => ColBuilder::Bool(Vec::new()),
            })
            .collect();

        for (row_idx, record) in reader.records().enumerate() {
            let record = record?;
            for (col_idx, (builder, s)) in builders.iter_mut().zip(record.iter()).enumerate() {
                if s.is_empty() {
                    match builder {
                        ColBuilder::Int(v) => v.push(None),
                        ColBuilder::Float(v) => v.push(None),
                        ColBuilder::Str(v) => v.push(None),
                        ColBuilder::Bool(v) => v.push(None),
                    }
                    continue;
                }
                match builder {
                    ColBuilder::Int(v) => {
                        v.push(Some(s.parse::<i64>().map_err(|_| {
                            CsvLoadingError::ParseError {
                                column: field_names[col_idx].clone(),
                                row: row_idx,
                                value: s.to_string(),
                                expected: field_expected[col_idx].to_string(),
                            }
                        })?));
                    }
                    ColBuilder::Float(v) => {
                        v.push(Some(s.parse::<f64>().map_err(|_| {
                            CsvLoadingError::ParseError {
                                column: field_names[col_idx].clone(),
                                row: row_idx,
                                value: s.to_string(),
                                expected: field_expected[col_idx].to_string(),
                            }
                        })?));
                    }
                    ColBuilder::Str(v) => {
                        v.push(Some(s.to_string()));
                    }
                    ColBuilder::Bool(v) => {
                        v.push(Some(parse_bool(s).ok_or_else(|| {
                            CsvLoadingError::ParseError {
                                column: field_names[col_idx].clone(),
                                row: row_idx,
                                value: s.to_string(),
                                expected: field_expected[col_idx].to_string(),
                            }
                        })?));
                    }
                }
            }
        }

        let columns = builders
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
