use std::collections::HashSet;

use crate::dataset::ops::NumericOps;

#[derive(Debug, Clone)]
pub enum InSetValues {
    IntSet(Vec<i64>),
    FloatSet(Vec<f64>),
    StrSet(Vec<String>),
}

#[derive(Clone, Debug)]
pub enum Column {
    Int(IntColumn),
    Float(FloatColumn),
    Str(StrColumn),
    Bool(BoolColumn),
}

#[derive(Clone, Debug)]
pub struct IntColumn(pub Vec<Option<i64>>);

#[derive(Clone, Debug)]
pub struct FloatColumn(pub Vec<Option<f64>>);

#[derive(Clone, Debug)]
pub struct StrColumn(pub Vec<Option<String>>);

#[derive(Clone, Debug)]
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

    pub fn unique_count(&self) -> usize {
        match self {
            Column::Int(col) => col.0.iter().collect::<HashSet<_>>().len(),
            Column::Str(col) => col.0.iter().collect::<HashSet<_>>().len(),
            Column::Bool(col) => col.0.iter().collect::<HashSet<_>>().len(),
            Column::Float(col) => col
                .0
                .iter()
                .map(|v| v.map(|f| f.to_bits()))
                .collect::<HashSet<_>>()
                .len(),
        }
    }

    pub fn duplicates_count(&self) -> usize {
        self.len() - self.unique_count()
    }

    pub fn is_in(&self, other: &InSetValues) -> Vec<Option<bool>> {
        match (self, other) {
            (Column::Int(col), InSetValues::IntSet(set)) => col
                .0
                .iter()
                .map(|opt| opt.map(|v| set.contains(&v)))
                .collect(),
            (Column::Float(col), InSetValues::FloatSet(set)) => col
                .0
                .iter()
                .map(|opt| opt.map(|v| set.contains(&v)))
                .collect(),
            (Column::Str(col), InSetValues::StrSet(set)) => col
                .0
                .iter()
                .map(|opt| opt.as_ref().map(|v| set.contains(v)))
                .collect(),
            _ => vec![None; self.len()],
        }
    }

    pub fn sum(&self) -> Option<f64> {
        match self {
            Column::Int(col) => col.sum().map(|v| v as f64),
            Column::Float(col) => col.sum(),
            _ => None,
        }
    }

    pub fn mean(&self) -> Option<f64> {
        match self {
            Column::Int(col) => col.mean(),
            Column::Float(col) => col.mean(),
            _ => None,
        }
    }

    pub fn min(&self) -> Option<f64> {
        match self {
            Column::Int(col) => col.min().map(|v| v as f64),
            Column::Float(col) => col.min(),
            _ => None,
        }
    }

    pub fn max(&self) -> Option<f64> {
        match self {
            Column::Int(col) => col.max().map(|v| v as f64),
            Column::Float(col) => col.max(),
            _ => None,
        }
    }

    pub fn std(&self) -> Option<f64> {
        match self {
            Column::Int(col) => col.std(),
            Column::Float(col) => col.std(),
            _ => None,
        }
    }

    pub fn median(&self) -> Option<f64> {
        match self {
            Column::Int(col) => col.median(),
            Column::Float(col) => col.median(),
            _ => None,
        }
    }
}
