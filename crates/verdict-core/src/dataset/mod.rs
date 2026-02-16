pub mod column;
pub mod ops;
pub mod schema;

pub use column::{BoolColumn, Column, FloatColumn, InSetValues, IntColumn, StrColumn};
pub use ops::NumericOps;
pub use schema::{DataType, Field, Schema};

pub struct Dataset {
    pub headers: Vec<String>,
    pub columns: Vec<Column>,
}

impl Dataset {
    pub fn new(headers: Vec<String>, columns: Vec<Column>) -> Self {
        Dataset { headers, columns }
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
