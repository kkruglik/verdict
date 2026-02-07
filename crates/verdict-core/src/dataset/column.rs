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

    pub fn is_null(&self) -> Vec<bool> {
        match self {
            Column::Int(col) => col.0.iter().map(|v| v.is_none()).collect(),
            Column::Float(col) => col.0.iter().map(|v| v.is_none()).collect(),
            Column::Str(col) => col.0.iter().map(|v| v.is_none()).collect(),
            Column::Bool(col) => col.0.iter().map(|v| v.is_none()).collect(),
        }
    }

    pub fn null_count(&self) -> usize {
        match self {
            Column::Int(col) => col.0.iter().filter(|v| v.is_none()).count(),
            Column::Float(col) => col.0.iter().filter(|v| v.is_none()).count(),
            Column::Str(col) => col.0.iter().filter(|v| v.is_none()).count(),
            Column::Bool(col) => col.0.iter().filter(|v| v.is_none()).count(),
        }
    }

    pub fn not_null_count(&self) -> usize {
        self.len() - self.null_count()
    }
}
