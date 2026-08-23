use std::{fs::File, path::Path};

use crate::dataframe::column::TimeColumn;
use crate::dataframe::{
    BoolColumn, Column, DataFrame, DateColumn, DateTimeColumn, FloatColumn, IntColumn, StringColumn,
};
use parquet::basic::{LogicalType, Type};
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParquetLoadingError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ParquetError(#[from] parquet::errors::ParquetError),

    #[error("unsupported column type in column '{column}': {type_name}")]
    UnsupportedType { column: String, type_name: String },

    #[error("type mismatch in column '{column}' at row {row}: expected {expected}, got {got}")]
    TypeMismatch {
        column: String,
        row: usize,
        expected: String,
        got: String,
    },
}

enum ColBuilder {
    Int(Vec<Option<i64>>),
    Float(Vec<Option<f64>>),
    Str(Vec<Option<String>>),
    Bool(Vec<Option<bool>>),
    Date(Vec<Option<i32>>),
    DateTime(Vec<Option<i64>>),
    Time(Vec<Option<i64>>),
}

impl ColBuilder {
    fn type_name(&self) -> &'static str {
        match self {
            ColBuilder::Int(_) => "Int",
            ColBuilder::Float(_) => "Float",
            ColBuilder::Str(_) => "Str",
            ColBuilder::Bool(_) => "Bool",
            ColBuilder::Date(_) => "Date",
            ColBuilder::DateTime(_) => "DateTime",
            ColBuilder::Time(_) => "Time",
        }
    }
}

pub trait DatasetParquetExt {
    fn from_parquet(path: &Path) -> Result<DataFrame, ParquetLoadingError>;
}

impl DatasetParquetExt for DataFrame {
    fn from_parquet(path: &Path) -> Result<DataFrame, ParquetLoadingError> {
        let file = File::open(path)?;
        let reader = SerializedFileReader::new(file)?;

        let parquet_metadata = reader.metadata();
        let fields = parquet_metadata.file_metadata().schema().get_fields();
        let num_rows = parquet_metadata.file_metadata().num_rows();

        let mut headers: Vec<String> = vec![];
        let mut columns_data: Vec<ColBuilder> = vec![];

        for col in fields.iter() {
            let builder = if let Some(l_type) = col.get_basic_info().logical_type_ref() {
                match l_type {
                    LogicalType::String
                    | LogicalType::Uuid
                    | LogicalType::Json
                    | LogicalType::Enum
                    | LogicalType::List
                    | LogicalType::Map
                    | LogicalType::Bson
                    | LogicalType::Variant { .. } => {
                        ColBuilder::Str(Vec::with_capacity(num_rows as usize))
                    }
                    LogicalType::Date => ColBuilder::Date(Vec::with_capacity(num_rows as usize)),
                    LogicalType::Timestamp { .. } => {
                        ColBuilder::DateTime(Vec::with_capacity(num_rows as usize))
                    }
                    LogicalType::Integer { .. } => {
                        ColBuilder::Int(Vec::with_capacity(num_rows as usize))
                    }
                    LogicalType::Float16 | LogicalType::Decimal { .. } => {
                        ColBuilder::Float(Vec::with_capacity(num_rows as usize))
                    }
                    LogicalType::Time { .. } => {
                        ColBuilder::Time(Vec::with_capacity(num_rows as usize))
                    }
                    _ => {
                        return Err(ParquetLoadingError::UnsupportedType {
                            column: col.name().to_string(),
                            type_name: format!("{:?}", l_type),
                        });
                    }
                }
            } else {
                match col.get_physical_type() {
                    Type::BOOLEAN => ColBuilder::Bool(Vec::with_capacity(num_rows as usize)),
                    Type::INT32 | Type::INT64 => {
                        ColBuilder::Int(Vec::with_capacity(num_rows as usize))
                    }
                    Type::FLOAT | Type::DOUBLE => {
                        ColBuilder::Float(Vec::with_capacity(num_rows as usize))
                    }
                    Type::BYTE_ARRAY | Type::FIXED_LEN_BYTE_ARRAY => {
                        ColBuilder::Str(Vec::with_capacity(num_rows as usize))
                    }
                    Type::INT96 => {
                        return Err(ParquetLoadingError::UnsupportedType {
                            column: col.name().to_string(),
                            type_name: "INT96".to_string(),
                        });
                    }
                }
            };

            columns_data.push(builder);
            headers.push(col.name().to_string());
        }

        for (row_idx, row) in reader.get_row_iter(None)?.enumerate() {
            let row = row?;
            for (idx, (col_name, field)) in row.get_column_iter().enumerate() {
                match (&mut columns_data[idx], field) {
                    (ColBuilder::Str(v), Field::Str(val)) => v.push(Some(val.clone())),

                    // TODO: add proper list and map repr in verdict core
                    (ColBuilder::Str(v), Field::ListInternal(val)) => {
                        let fields_str: Vec<String> =
                            val.elements().iter().map(|v| v.to_string()).collect();
                        v.push(Some(fields_str.join(",")));
                    }
                    (ColBuilder::Str(v), Field::MapInternal(val)) => {
                        let fields_str: Vec<String> = val
                            .entries()
                            .iter()
                            .map(|(k, v)| format!("{}, {}", k, v))
                            .collect();
                        v.push(Some(fields_str.join(",")));
                    }
                    (ColBuilder::Str(v), Field::Bytes(val)) => {
                        let field = val.as_utf8()?;
                        v.push(Some(field.to_string()));
                    }
                    (ColBuilder::Int(v), Field::Byte(val)) => v.push(Some(*val as i64)),
                    (ColBuilder::Int(v), Field::Short(val)) => v.push(Some(*val as i64)),
                    (ColBuilder::Int(v), Field::Int(val)) => v.push(Some(*val as i64)),
                    (ColBuilder::Int(v), Field::Long(val)) => v.push(Some(*val)),
                    (ColBuilder::Int(v), Field::UByte(val)) => v.push(Some(*val as i64)),
                    (ColBuilder::Int(v), Field::UShort(val)) => v.push(Some(*val as i64)),
                    (ColBuilder::Int(v), Field::UInt(val)) => v.push(Some(*val as i64)),
                    (ColBuilder::Int(v), Field::ULong(val)) => v.push(Some(*val as i64)),
                    (ColBuilder::Float(v), Field::Float16(val)) => {
                        v.push(Some(f32::from(*val) as f64))
                    }
                    (ColBuilder::Float(v), Field::Float(val)) => v.push(Some(*val as f64)),
                    (ColBuilder::Float(v), Field::Double(val)) => v.push(Some(*val)),
                    (ColBuilder::Float(v), Field::Decimal(val)) => {
                        let data = val.data();
                        let n_bytes = data.len();
                        let unscaled = data.iter().fold(0i64, |acc, &b| (acc << 8) | b as i64);
                        // The fold zero-extends each byte. For n_bytes < 8 the i64
                        // sign bit is never reached, so sign-extend manually.
                        let unscaled = if n_bytes < 8 && (data[0] & 0x80) != 0 {
                            unscaled | (-1i64 << (n_bytes * 8))
                        } else {
                            unscaled
                        };
                        let f = unscaled as f64 / 10f64.powi(val.scale());
                        v.push(Some(f));
                    }
                    (ColBuilder::Bool(v), Field::Bool(val)) => v.push(Some(*val)),
                    (ColBuilder::Date(v), Field::Date(val)) => v.push(Some(*val)),
                    (ColBuilder::DateTime(v), Field::TimestampMicros(val)) => v.push(Some(*val)),
                    (ColBuilder::DateTime(v), Field::TimestampMillis(val)) => {
                        v.push(Some(*val * 1000))
                    }

                    // TODO: my lib support only seconds, but parquet spec supports milliseconds and
                    // microseconds
                    (ColBuilder::Time(v), Field::TimeMicros(val)) => v.push(Some(*val)),
                    (ColBuilder::Time(v), Field::TimeMillis(val)) => {
                        v.push(Some(*val as i64 * 1000))
                    }

                    // TODO: Group (nested struct) not supported — verdict-core has no nested column type
                    (ColBuilder::Str(v), Field::Group(_)) => v.push(None),
                    (builder, Field::Null) => match builder {
                        ColBuilder::Int(v) => v.push(None),
                        ColBuilder::Float(v) => v.push(None),
                        ColBuilder::Str(v) => v.push(None),
                        ColBuilder::Bool(v) => v.push(None),
                        ColBuilder::Date(v) => v.push(None),
                        ColBuilder::DateTime(v) => v.push(None),
                        ColBuilder::Time(v) => v.push(None),
                    },
                    (builder, field) => {
                        return Err(ParquetLoadingError::TypeMismatch {
                            column: col_name.to_string(),
                            row: row_idx,
                            expected: builder.type_name().to_string(),
                            got: format!("{:?}", field),
                        });
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
                ColBuilder::DateTime(v) => Column::DateTime(DateTimeColumn::new(v)),
                ColBuilder::Date(v) => Column::Date(DateColumn::new(v)),
                ColBuilder::Time(v) => Column::Time(TimeColumn::new(v)),
            })
            .collect();

        Ok(DataFrame { headers, columns })
    }
}
