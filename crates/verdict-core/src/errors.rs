use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Failed to validate column '{column}' for constraint '{constraint}'")]
    ColumnValidationError { column: String, constraint: String },

    #[error("Column '{name}' not found in dataset")]
    ColumnNotFound { name: String },

    #[error("Unknown constraint '{name}'")]
    UnknownConstraint { name: String },
}
