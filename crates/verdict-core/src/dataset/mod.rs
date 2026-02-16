pub mod column;
pub mod ops;
pub mod schema;

pub use column::{BoolColumn, Column, FloatColumn, InSetValues, IntColumn, StrColumn};
pub use ops::NumericOps;
pub use schema::{DataType, Field, Schema};

use crate::errors::DatasetError;
use csv::Reader;

pub struct Dataset {
    pub headers: Vec<String>,
    pub columns: Vec<Column>,
}

impl Dataset {
    pub fn new(headers: Vec<String>, columns: Vec<Column>) -> Self {
        Dataset { headers, columns }
    }

    pub fn from_csv(path: &str, schema: &Schema) -> Result<Self, DatasetError> {
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
                                    s.parse::<i64>().map_err(|_| DatasetError::ParseError {
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
                                    s.parse::<f64>().map_err(|_| DatasetError::ParseError {
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
                                    parse_bool(s).ok_or_else(|| DatasetError::ParseError {
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

    pub fn get_column_by_name(&self, name: &str) -> Option<&Column> {
        let col_idx = self.get_column_index(name);
        if let Some(idx) = col_idx {
            return Some(&self.columns[idx]);
        }
        None
    }

    pub fn get_column_by_index(&self, idx: usize) -> Option<&Column> {
        self.columns.get(idx)
    }

    pub fn get_column_index(&self, name: &str) -> Option<usize> {
        self.headers.iter().position(|h| h == name)
    }

    pub fn shape(&self) -> (usize, usize) {
        let rows_count = self.columns.first().map_or(0, |c| c.len());
        (rows_count, self.columns.len())
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}
