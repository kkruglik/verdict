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
