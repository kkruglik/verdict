use std::fmt::Display;

pub mod column;
pub mod column_checks;
pub mod table;
pub mod table_checks;
pub mod validation;

pub use column::{ColumnConstraint, ColumnRule, ColumnRuleBuilder, col};
pub use column_checks::validate_columns;
pub use table::{TableConstraint, TableRule, TableRuleBuilder};
pub use table_checks::validate_table;
pub use validation::{ValidationConfig, ValidationReport, ValidationResult};

#[derive(Debug, Clone)]
pub enum Operand {
    Column(String),
    Num(f64),
    Str(String),
}

impl Operand {
    pub fn type_name(&self) -> &'static str {
        match self {
            Operand::Column(_) => "col",
            Operand::Num(_) => "num",
            Operand::Str(_) => "str",
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Num(val) => write!(f, "{}", val),
            Operand::Column(name) => write!(f, "col({})", name),
            Operand::Str(val) => write!(f, "{}", val),
        }
    }
}

impl From<f64> for Operand {
    fn from(value: f64) -> Self {
        Operand::Num(value)
    }
}

impl From<i64> for Operand {
    fn from(value: i64) -> Self {
        Operand::Num(value as f64)
    }
}
