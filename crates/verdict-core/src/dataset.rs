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

    pub fn shape(&self) -> (usize, usize) {
        let rows_count = self.columns.first().map_or(0, |c| c.len());
        (rows_count, self.columns.len())
    }
}

pub struct Field {
    pub name: String,
    pub dtype: DataType,
}

impl Field {
    pub fn new(name: impl Into<String>, dtype: DataType) -> Self {
        Field {
            name: name.into(),
            dtype,
        }
    }
}

pub enum DataType {
    Int,
    Str,
    Float,
    Bool,
}

pub struct Schema {
    pub fields: Vec<Field>,
}

impl Schema {
    pub fn new(fields: Vec<Field>) -> Self {
        Schema { fields }
    }
}

pub trait ColumnArray {
    fn len(&self) -> usize;
    fn null_count(&self) -> usize;
    fn not_null_count(&self) -> usize;
    fn is_empty(&self) -> bool;
}

pub struct IntColumn(pub Vec<Option<i64>>);
pub struct FloatColumn(pub Vec<Option<f64>>);
pub struct StrColumn(pub Vec<Option<String>>);
pub struct BoolColumn(pub Vec<Option<bool>>);

pub enum Column {
    Int(IntColumn),
    Float(FloatColumn),
    Str(StrColumn),
    Bool(BoolColumn),
}

impl Column {
    pub fn len(&self) -> usize {
        match self {
            Column::Int(col) => col.0.len(),
            Column::Float(col) => col.0.len(),
            Column::Str(col) => col.0.len(),
            Column::Bool(col) => col.0.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}
