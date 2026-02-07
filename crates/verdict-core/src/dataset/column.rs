pub struct IntColumn(pub Vec<Option<i64>>);
pub struct FloatColumn(pub Vec<Option<f64>>);
pub struct StrColumn(pub Vec<Option<String>>);
pub struct BoolColumn(pub Vec<Option<bool>>);

impl IntColumn {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn not_null_count(&self) -> usize {
        self.0.iter().filter(|v| v.is_some()).count()
    }
}

impl FloatColumn {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn not_null_count(&self) -> usize {
        self.0.iter().filter(|v| v.is_some()).count()
    }
}

impl StrColumn {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn not_null_count(&self) -> usize {
        self.0.iter().filter(|v| v.is_some()).count()
    }
}

impl BoolColumn {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn not_null_count(&self) -> usize {
        self.0.iter().filter(|v| v.is_some()).count()
    }
}

pub enum Column {
    Int(IntColumn),
    Float(FloatColumn),
    Str(StrColumn),
    Bool(BoolColumn),
}

impl Column {
    pub fn len(&self) -> usize {
        match self {
            Column::Int(col) => col.len(),
            Column::Float(col) => col.len(),
            Column::Str(col) => col.len(),
            Column::Bool(col) => col.len(),
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
            Column::Int(col) => col.len() - col.not_null_count(),
            Column::Float(col) => col.len() - col.not_null_count(),
            Column::Str(col) => col.len() - col.not_null_count(),
            Column::Bool(col) => col.len() - col.not_null_count(),
        }
    }

    pub fn not_null_count(&self) -> usize {
        match self {
            Column::Int(col) => col.not_null_count(),
            Column::Float(col) => col.not_null_count(),
            Column::Str(col) => col.not_null_count(),
            Column::Bool(col) => col.not_null_count(),
        }
    }
}
