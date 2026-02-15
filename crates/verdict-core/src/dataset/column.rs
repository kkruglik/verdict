use std::collections::HashSet;

use crate::dataset::ops::{ComparableOps, NumericOps, StringOps};

#[derive(Debug)]
pub enum InSetValues {
    IntSet(Vec<i64>),
    FloatSet(Vec<f64>),
    StrSet(Vec<String>),
}

pub enum Column {
    Int(IntColumn),
    Float(FloatColumn),
    Str(StrColumn),
    Bool(BoolColumn),
}

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

    pub fn gt(&self, compare: f64) -> Vec<Option<bool>> {
        match self {
            Column::Int(col) => col.gt(compare),
            Column::Float(col) => col.gt(compare),
            _ => vec![None; self.len()],
        }
    }

    pub fn ge(&self, compare: f64) -> Vec<Option<bool>> {
        match self {
            Column::Int(col) => col.ge(compare),
            Column::Float(col) => col.ge(compare),
            _ => vec![None; self.len()],
        }
    }

    pub fn lt(&self, compare: f64) -> Vec<Option<bool>> {
        match self {
            Column::Int(col) => col.lt(compare),
            Column::Float(col) => col.lt(compare),
            _ => vec![None; self.len()],
        }
    }

    pub fn le(&self, compare: f64) -> Vec<Option<bool>> {
        match self {
            Column::Int(col) => col.le(compare),
            Column::Float(col) => col.le(compare),
            _ => vec![None; self.len()],
        }
    }

    pub fn equal(&self, compare: f64) -> Vec<Option<bool>> {
        match self {
            Column::Int(col) => col.equal(compare),
            Column::Float(col) => col.equal(compare),
            _ => vec![None; self.len()],
        }
    }

    pub fn equal_str(&self, compare: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.equal(compare),
            _ => vec![None; self.len()],
        }
    }

    pub fn between(&self, lower: f64, upper: f64) -> Vec<Option<bool>> {
        match self {
            Column::Int(col) => col.between(lower, upper),
            Column::Float(col) => col.between(lower, upper),
            _ => vec![None; self.len()],
        }
    }

    pub fn contains(&self, pat: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.contains(pat),
            _ => vec![None; self.len()],
        }
    }

    pub fn starts_with(&self, pat: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.starts_with(pat),
            _ => vec![None; self.len()],
        }
    }

    pub fn ends_with(&self, pat: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.ends_with(pat),
            _ => vec![None; self.len()],
        }
    }

    pub fn matches_regex(&self, pat: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.matches_regex(pat),
            _ => vec![None; self.len()],
        }
    }

    pub fn str_length(&self) -> Vec<Option<usize>> {
        match self {
            Column::Str(col) => col.length(),
            _ => vec![None; self.len()],
        }
    }
}
