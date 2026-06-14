pub mod column;
pub mod ops;
pub mod schema;

pub use column::{
    BoolColumn, Column, DateColumn, DateTimeColumn, FloatColumn, IntColumn, KeepStrategy,
    StringColumn, TimeColumn, ValuesSet,
};
pub use ops::{
    NumericOps, i32_to_naive_date, i64_to_naive_datetime, naive_date_to_i32, naive_datetime_to_i64,
};
pub use schema::{DataType, Field, Schema};

pub struct DataFrame {
    pub headers: Vec<String>,
    pub columns: Vec<Column>,
}

impl DataFrame {
    pub fn new(headers: Vec<String>, columns: Vec<Column>) -> Self {
        DataFrame { headers, columns }
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
