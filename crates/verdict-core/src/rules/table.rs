use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum TableConstraint {
    RowsCountBetween { min: usize, max: usize },
    RowsCountGreaterOrEqual(usize),
    RowCountGreaterThan(usize),
    RowsCountLessOrEqual(usize),
    RowCountLessThan(usize),

    ColumnsCountBetween { min: usize, max: usize },
    ColumnsCountGreaterOrEqual(usize),
    ColumnsCountGreaterThan(usize),
    ColumnsCountLessOrEqual(usize),
    ColumnsCountLessThan(usize),

    ColumnsExist(Vec<String>),
    ShapeEquals { rows: usize, columns: usize },
}

impl Display for TableConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableConstraint::ShapeEquals { rows, columns } => {
                write!(f, "shape_equals({}, {})", rows, columns)
            }
            TableConstraint::ColumnsExist(cols) => write!(f, "columns_exists: {:?}", cols),
            TableConstraint::ColumnsCountLessThan(count) => {
                write!(f, "columns_count_less_than: {}", count)
            }
            TableConstraint::ColumnsCountLessOrEqual(count) => {
                write!(f, "columns_count_less_or_equal: {}", count)
            }
            TableConstraint::ColumnsCountGreaterThan(count) => {
                write!(f, "columns_count_greater_than: {}", count)
            }
            TableConstraint::ColumnsCountGreaterOrEqual(count) => {
                write!(f, "columns_count_greater_or_equal: {}", count)
            }
            TableConstraint::ColumnsCountBetween { min, max } => {
                write!(f, "columns_count_between({}, {})", min, max)
            }
            TableConstraint::RowCountLessThan(count) => write!(f, "row_count_less_than: {}", count),
            TableConstraint::RowsCountLessOrEqual(count) => {
                write!(f, "rows_count_less_or_equal: {}", count)
            }
            TableConstraint::RowsCountBetween { min, max } => {
                write!(f, "rows_count_between({}, {})", min, max)
            }
            TableConstraint::RowCountGreaterThan(count) => {
                write!(f, "row_count_greater_than: {}", count)
            }
            TableConstraint::RowsCountGreaterOrEqual(count) => {
                write!(f, "rows_count_greater_or_equal: {}", count)
            }
        }
    }
}

#[derive(Clone)]
pub struct TableRule {
    pub constraint: TableConstraint,
}

impl TableRule {
    pub fn new(constraint: TableConstraint) -> TableRule {
        TableRule { constraint }
    }
}

#[derive(Default)]
pub struct TableRuleBuilder {
    pub constraint: Vec<TableConstraint>,
}

impl TableRuleBuilder {
    pub fn shape_equals(mut self, rows: usize, columns: usize) -> Self {
        self.constraint
            .push(TableConstraint::ShapeEquals { rows, columns });
        self
    }

    pub fn build(self) -> Vec<TableRule> {
        self.constraint
            .into_iter()
            .map(|c| TableRule { constraint: c })
            .collect()
    }
}

pub fn table_rule() -> TableRuleBuilder {
    TableRuleBuilder { constraint: vec![] }
}
