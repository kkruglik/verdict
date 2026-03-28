use std::fmt::Display;

#[derive(Debug, Clone)]
enum TableConstraint {
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
        todo!()
    }
}

#[derive(Clone)]
pub struct TableRule {
    pub constraint: TableConstraint,
}

#[derive(Default)]
pub struct TableRuleBuilder {
    pub constraint: Vec<TableConstraint>,
}

impl TableRuleBuilder {
    fn shape_equals(mut self, rows: usize, columns: usize) -> Self {
        self.constraint
            .push(TableConstraint::ShapeEquals { rows, columns });
        self
    }
}
