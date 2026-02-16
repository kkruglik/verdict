use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatasetError {
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

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Failed to validate column '{column}' for constraint '{constraint}'")]
    ColumnValidationError { column: String, constraint: String },

    #[error("Column '{name}' not found in dataset")]
    ColumnNotFound { name: String },

    #[error("Unknown constraint '{name}'")]
    UnknownConstraint { name: String },
}
