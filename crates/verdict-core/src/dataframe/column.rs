use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use crate::dataframe::ops::numeric::NumericOps;

#[derive(Debug, Clone, Copy)]
pub enum KeepStrategy {
    First,
    Last,
    None,
}

#[derive(Debug, Clone)]
pub enum ValuesSet {
    Int64Set(Vec<i64>),
    Int32Set(Vec<i32>),
    FloatSet(Vec<f64>),
    StrSet(Vec<String>),
}

#[derive(Clone, Debug)]
pub enum Column {
    Int(IntColumn),
    Float(FloatColumn),
    Str(StringColumn),
    Bool(BoolColumn),
    DateTime(DateTimeColumn),
    Date(DateColumn),
    Time(TimeColumn),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypedColumn<T, Marker>(pub Vec<Option<T>>, PhantomData<Marker>);

impl<T, Marker> TypedColumn<T, Marker> {
    pub fn new(data: Vec<Option<T>>) -> Self {
        TypedColumn(data, PhantomData)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_null(&self) -> Vec<bool> {
        self.0.iter().map(|v| v.is_none()).collect()
    }

    pub fn not_null_count(&self) -> usize {
        self.0.iter().filter(|v| v.is_some()).count()
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct IntMarker;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct FloatMarker;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct StringMarker;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct BoolMarker;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct DateMarker;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct DateTimeMarker;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct TimeMarker;

pub type IntColumn = TypedColumn<i64, IntMarker>;
pub type FloatColumn = TypedColumn<f64, FloatMarker>;
pub type BoolColumn = TypedColumn<bool, BoolMarker>;
pub type StringColumn = TypedColumn<String, StringMarker>;
pub type DateColumn = TypedColumn<i32, DateMarker>;
pub type DateTimeColumn = TypedColumn<i64, DateTimeMarker>;
pub type TimeColumn = TypedColumn<i64, TimeMarker>;

impl Column {
    pub fn len(&self) -> usize {
        match self {
            Column::Int(col) => col.len(),
            Column::Float(col) => col.len(),
            Column::Str(col) => col.len(),
            Column::Bool(col) => col.len(),
            Column::DateTime(col) => col.len(),
            Column::Date(col) => col.len(),
            Column::Time(col) => col.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_null(&self) -> Vec<bool> {
        match self {
            Column::Int(col) => col.is_null(),
            Column::Float(col) => col.is_null(),
            Column::Str(col) => col.is_null(),
            Column::Bool(col) => col.is_null(),
            Column::DateTime(col) => col.is_null(),
            Column::Date(col) => col.is_null(),
            Column::Time(col) => col.is_null(),
        }
    }

    pub fn null_count(&self) -> usize {
        match self {
            Column::Int(col) => col.len() - col.not_null_count(),
            Column::Float(col) => col.len() - col.not_null_count(),
            Column::Str(col) => col.len() - col.not_null_count(),
            Column::Bool(col) => col.len() - col.not_null_count(),
            Column::Date(col) => col.len() - col.not_null_count(),
            Column::DateTime(col) => col.len() - col.not_null_count(),
            Column::Time(col) => col.len() - col.not_null_count(),
        }
    }

    pub fn not_null_count(&self) -> usize {
        match self {
            Column::Int(col) => col.not_null_count(),
            Column::Float(col) => col.not_null_count(),
            Column::Str(col) => col.not_null_count(),
            Column::Bool(col) => col.not_null_count(),
            Column::DateTime(col) => col.not_null_count(),
            Column::Date(col) => col.not_null_count(),
            Column::Time(col) => col.not_null_count(),
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
            Column::DateTime(col) => col.0.iter().collect::<HashSet<_>>().len(),
            Column::Date(col) => col.0.iter().collect::<HashSet<_>>().len(),
            Column::Time(col) => col.0.iter().collect::<HashSet<_>>().len(),
        }
    }

    pub fn duplicates_count(&self) -> usize {
        self.len() - self.unique_count()
    }

    pub fn duplicated(&self, keep: KeepStrategy) -> Vec<bool> {
        match self {
            Column::Int(c) => duplicated(&c.0, keep),
            Column::Float(c) => {
                let bits_values: Vec<Option<u64>> =
                    c.0.iter().map(|v| v.map(|f| f.to_bits())).collect();
                duplicated(&bits_values, keep)
            }
            Column::Str(c) => duplicated(&c.0, keep),
            Column::Bool(c) => duplicated(&c.0, keep),
            Column::DateTime(c) => duplicated(&c.0, keep),
            Column::Date(c) => duplicated(&c.0, keep),
            Column::Time(c) => duplicated(&c.0, keep),
        }
    }

    pub fn is_in(&self, other: &ValuesSet) -> Vec<Option<bool>> {
        match (self, other) {
            (Column::Int(col), ValuesSet::Int64Set(set)) => col
                .0
                .iter()
                .map(|opt| opt.map(|v| set.contains(&v)))
                .collect(),
            (Column::Float(col), ValuesSet::FloatSet(set)) => col
                .0
                .iter()
                .map(|opt| opt.map(|v| set.contains(&v)))
                .collect(),
            (Column::Str(col), ValuesSet::StrSet(set)) => col
                .0
                .iter()
                .map(|opt| opt.as_ref().map(|v| set.contains(v)))
                .collect(),
            (Column::DateTime(col), ValuesSet::Int64Set(set)) => col
                .0
                .iter()
                .map(|opt| opt.map(|v| set.contains(&v)))
                .collect(),
            (Column::Date(col), ValuesSet::Int32Set(set)) => col
                .0
                .iter()
                .map(|opt| opt.map(|v| set.contains(&v)))
                .collect(),
            (Column::Time(col), ValuesSet::Int64Set(set)) => col
                .0
                .iter()
                .map(|opt| opt.map(|v| set.contains(&v)))
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

fn duplicated<T: Eq + std::hash::Hash>(vec: &[T], keep: KeepStrategy) -> Vec<bool> {
    let mut counts: HashMap<&T, usize> = HashMap::new();
    for val in vec {
        *counts.entry(val).or_insert(0) += 1;
    }

    let mut seen: HashMap<&T, usize> = HashMap::new();
    let mut mask = vec![false; vec.len()];

    match keep {
        KeepStrategy::First => {
            for (i, val) in vec.iter().enumerate() {
                let seen_count = seen.entry(val).or_insert(0);
                if *seen_count > 0 {
                    mask[i] = true;
                }
                *seen_count += 1;
            }
        }
        KeepStrategy::Last => {
            for (i, val) in vec.iter().enumerate().rev() {
                let seen_count = seen.entry(val).or_insert(0);
                if *seen_count > 0 {
                    mask[i] = true;
                }
                *seen_count += 1;
            }
        }
        KeepStrategy::None => {
            for (i, val) in vec.iter().enumerate() {
                if counts[val] > 1 {
                    mask[i] = true;
                }
            }
        }
    }

    mask
}
