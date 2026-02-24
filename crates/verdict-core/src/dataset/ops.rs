use crate::dataset::Column;

use super::{BoolColumn, FloatColumn, IntColumn, StrColumn};
use regex::Regex;

pub trait NumericOps {
    type Item;
    fn sum(&self) -> Option<Self::Item>;
    fn min(&self) -> Option<Self::Item>;
    fn max(&self) -> Option<Self::Item>;
    fn mean(&self) -> Option<f64>;
    fn std(&self) -> Option<f64>;
    fn median(&self) -> Option<f64>;
}

pub trait ComparableOps<T> {
    fn gt(&self, compare: T) -> Vec<Option<bool>>;
    fn ge(&self, compare: T) -> Vec<Option<bool>>;
    fn lt(&self, compare: T) -> Vec<Option<bool>>;
    fn le(&self, compare: T) -> Vec<Option<bool>>;
    fn equal(&self, compare: T) -> Vec<Option<bool>>;
    fn between(&self, lower: T, upper: T) -> Vec<Option<bool>>;
}

pub trait StringOps {
    fn contains(&self, pat: &str) -> Vec<Option<bool>>;
    fn starts_with(&self, pat: &str) -> Vec<Option<bool>>;
    fn ends_with(&self, pat: &str) -> Vec<Option<bool>>;
    fn matches_regex(&self, pat: &str) -> Vec<Option<bool>>;
    fn length(&self) -> Vec<Option<usize>>;
}

impl NumericOps for IntColumn {
    type Item = i64;

    fn sum(&self) -> Option<Self::Item> {
        if self.not_null_count() == 0 {
            return None;
        }
        Some(self.0.iter().flatten().sum())
    }

    fn min(&self) -> Option<Self::Item> {
        self.0.iter().filter_map(|v| *v).min()
    }

    fn max(&self) -> Option<Self::Item> {
        self.0.iter().filter_map(|v| *v).max()
    }

    fn mean(&self) -> Option<f64> {
        let sum = self.sum()?;
        let count = self.not_null_count();
        Some(sum as f64 / count as f64)
    }

    fn std(&self) -> Option<f64> {
        let mean = self.mean()?;
        let count = self.not_null_count();
        if count < 2 {
            return None;
        }
        let sq_sum: f64 = self
            .0
            .iter()
            .filter_map(|v| *v)
            .map(|v| (v as f64 - mean).powi(2))
            .sum();
        Some((sq_sum / (count - 1) as f64).sqrt())
    }

    fn median(&self) -> Option<f64> {
        let mut vals: Vec<i64> = self.0.iter().filter_map(|v| *v).collect();
        if vals.is_empty() {
            return None;
        }
        vals.sort();
        let mid = vals.len() / 2;
        if vals.len().is_multiple_of(2) {
            Some((vals[mid - 1] + vals[mid]) as f64 / 2.0)
        } else {
            Some(vals[mid] as f64)
        }
    }
}

impl NumericOps for FloatColumn {
    type Item = f64;

    fn sum(&self) -> Option<Self::Item> {
        if self.not_null_count() == 0 {
            return None;
        }
        Some(self.0.iter().flatten().sum())
    }

    fn min(&self) -> Option<Self::Item> {
        self.0.iter().filter_map(|v| *v).reduce(f64::min)
    }

    fn max(&self) -> Option<Self::Item> {
        self.0.iter().filter_map(|v| *v).reduce(f64::max)
    }

    fn mean(&self) -> Option<f64> {
        let sum = self.sum()?;
        let count = self.not_null_count();
        Some(sum / count as f64)
    }

    fn std(&self) -> Option<f64> {
        let mean = self.mean()?;
        let count = self.not_null_count();
        if count < 2 {
            return None;
        }
        let sq_sum: f64 = self
            .0
            .iter()
            .filter_map(|v| *v)
            .map(|v| (v - mean).powi(2))
            .sum();
        Some((sq_sum / (count - 1) as f64).sqrt())
    }

    fn median(&self) -> Option<f64> {
        let mut vals: Vec<f64> = self.0.iter().filter_map(|v| *v).collect();
        if vals.is_empty() {
            return None;
        }
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = vals.len() / 2;
        if vals.len().is_multiple_of(2) {
            Some((vals[mid - 1] + vals[mid]) / 2.0)
        } else {
            Some(vals[mid])
        }
    }
}

impl ComparableOps<i64> for IntColumn {
    fn gt(&self, compare: i64) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x > compare)).collect()
    }

    fn ge(&self, compare: i64) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x >= compare)).collect()
    }

    fn lt(&self, compare: i64) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x < compare)).collect()
    }

    fn le(&self, compare: i64) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x <= compare)).collect()
    }

    fn equal(&self, compare: i64) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x == compare)).collect()
    }

    fn between(&self, lower: i64, upper: i64) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.map(|x| x >= lower && x <= upper))
            .collect()
    }
}

impl ComparableOps<f64> for IntColumn {
    fn gt(&self, compare: f64) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.map(|x| (x as f64) > compare))
            .collect()
    }

    fn ge(&self, compare: f64) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.map(|x| (x as f64) >= compare))
            .collect()
    }

    fn lt(&self, compare: f64) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.map(|x| (x as f64) < compare))
            .collect()
    }

    fn le(&self, compare: f64) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.map(|x| (x as f64) <= compare))
            .collect()
    }

    fn equal(&self, compare: f64) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.map(|x| (x as f64) == compare))
            .collect()
    }

    fn between(&self, lower: f64, upper: f64) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.map(|x| (x as f64) >= lower && (x as f64) <= upper))
            .collect()
    }
}

impl ComparableOps<f64> for FloatColumn {
    fn gt(&self, compare: f64) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x > compare)).collect()
    }

    fn ge(&self, compare: f64) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x >= compare)).collect()
    }

    fn lt(&self, compare: f64) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x < compare)).collect()
    }

    fn le(&self, compare: f64) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x <= compare)).collect()
    }

    fn equal(&self, compare: f64) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x == compare)).collect()
    }

    fn between(&self, lower: f64, upper: f64) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.map(|x| x >= lower && x <= upper))
            .collect()
    }
}

impl ComparableOps<&str> for Column {
    fn gt(&self, compare: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.gt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn ge(&self, compare: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.ge(compare),
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.lt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.le(compare),
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.equal(compare),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: &str, upper: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.between(lower, upper),
            _ => vec![None; self.len()],
        }
    }
}

impl ComparableOps<f64> for Column {
    fn ge(&self, compare: f64) -> Vec<Option<bool>> {
        match self {
            Column::Int(col) => col.ge(compare),
            Column::Float(col) => col.ge(compare),
            _ => vec![None; self.len()],
        }
    }
    fn gt(&self, compare: f64) -> Vec<Option<bool>> {
        match self {
            Column::Int(col) => col.gt(compare),
            Column::Float(col) => col.gt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: f64) -> Vec<Option<bool>> {
        match self {
            Column::Int(col) => col.lt(compare),
            Column::Float(col) => col.lt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: f64) -> Vec<Option<bool>> {
        match self {
            Column::Int(col) => col.le(compare),
            Column::Float(col) => col.le(compare),
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: f64) -> Vec<Option<bool>> {
        match self {
            Column::Int(col) => col.equal(compare),
            Column::Float(col) => col.equal(compare),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: f64, upper: f64) -> Vec<Option<bool>> {
        match self {
            Column::Int(col) => col.between(lower, upper),
            Column::Float(col) => col.between(lower, upper),
            _ => vec![None; self.len()],
        }
    }
}

impl ComparableOps<&Column> for Column {
    fn gt(&self, compare: &Column) -> Vec<Option<bool>> {
        match (self, compare) {
            (Column::Float(a), Column::Float(b)) => a.gt(b),
            (Column::Int(a), Column::Int(b)) => a.gt(b),
            (Column::Str(a), Column::Str(b)) => a.gt(b),
            (Column::Bool(a), Column::Bool(b)) => a.gt(b),
            _ => vec![None; self.len()],
        }
    }

    fn ge(&self, compare: &Column) -> Vec<Option<bool>> {
        match (self, compare) {
            (Column::Float(a), Column::Float(b)) => a.ge(b),
            (Column::Int(a), Column::Int(b)) => a.ge(b),
            (Column::Str(a), Column::Str(b)) => a.ge(b),
            (Column::Bool(a), Column::Bool(b)) => a.ge(b),
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: &Column) -> Vec<Option<bool>> {
        match (self, compare) {
            (Column::Float(a), Column::Float(b)) => a.lt(b),
            (Column::Int(a), Column::Int(b)) => a.lt(b),
            (Column::Str(a), Column::Str(b)) => a.lt(b),
            (Column::Bool(a), Column::Bool(b)) => a.lt(b),
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: &Column) -> Vec<Option<bool>> {
        match (self, compare) {
            (Column::Float(a), Column::Float(b)) => a.le(b),
            (Column::Int(a), Column::Int(b)) => a.le(b),
            (Column::Str(a), Column::Str(b)) => a.le(b),
            (Column::Bool(a), Column::Bool(b)) => a.le(b),
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: &Column) -> Vec<Option<bool>> {
        match (self, compare) {
            (Column::Float(a), Column::Float(b)) => a.equal(b),
            (Column::Int(a), Column::Int(b)) => a.equal(b),
            (Column::Str(a), Column::Str(b)) => a.equal(b),
            (Column::Bool(a), Column::Bool(b)) => a.equal(b),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: &Column, upper: &Column) -> Vec<Option<bool>> {
        match (self, lower, upper) {
            (Column::Float(a), Column::Float(lo), Column::Float(hi)) => a.between(lo, hi),
            (Column::Int(a), Column::Int(lo), Column::Int(hi)) => a.between(lo, hi),
            (Column::Str(a), Column::Str(lo), Column::Str(hi)) => a.between(lo, hi),
            (Column::Bool(a), Column::Bool(lo), Column::Bool(hi)) => a.between(lo, hi),
            _ => vec![None; self.len()],
        }
    }
}

impl ComparableOps<&IntColumn> for IntColumn {
    fn gt(&self, compare: &IntColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }

    fn ge(&self, compare: &IntColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x >= y),
                _ => None,
            })
            .collect()
    }

    fn lt(&self, compare: &IntColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x < y),
                _ => None,
            })
            .collect()
    }

    fn le(&self, compare: &IntColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x <= y),
                _ => None,
            })
            .collect()
    }

    fn equal(&self, compare: &IntColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x == y),
                _ => None,
            })
            .collect()
    }

    fn between(&self, lower: &IntColumn, upper: &IntColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(lower.0.iter())
            .zip(upper.0.iter())
            .map(|((x, lo), hi)| match (x, lo, hi) {
                (Some(x), Some(lo), Some(hi)) => Some(x >= lo && x <= hi),
                _ => None,
            })
            .collect()
    }
}

impl ComparableOps<&FloatColumn> for FloatColumn {
    fn gt(&self, compare: &FloatColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }
    fn ge(&self, compare: &FloatColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x >= y),
                _ => None,
            })
            .collect()
    }

    fn lt(&self, compare: &FloatColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x < y),
                _ => None,
            })
            .collect()
    }

    fn le(&self, compare: &FloatColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x <= y),
                _ => None,
            })
            .collect()
    }

    fn equal(&self, compare: &FloatColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x == y),
                _ => None,
            })
            .collect()
    }

    fn between(&self, lower: &FloatColumn, upper: &FloatColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(lower.0.iter())
            .zip(upper.0.iter())
            .map(|((x, lower), upper)| match (x, lower, upper) {
                (Some(x), Some(lower), Some(upper)) => Some(x >= lower && x <= upper),
                _ => None,
            })
            .collect()
    }
}

impl ComparableOps<&str> for StrColumn {
    fn gt(&self, compare: &str) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.as_str() > compare))
            .collect()
    }

    fn ge(&self, compare: &str) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.as_str() >= compare))
            .collect()
    }

    fn lt(&self, compare: &str) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.as_str() < compare))
            .collect()
    }

    fn le(&self, compare: &str) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.as_str() <= compare))
            .collect()
    }

    fn equal(&self, compare: &str) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.as_str() == compare))
            .collect()
    }

    fn between(&self, lower: &str, upper: &str) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| {
                v.as_ref()
                    .map(|s| s.as_str() >= lower && s.as_str() <= upper)
            })
            .collect()
    }
}

impl ComparableOps<&StrColumn> for StrColumn {
    fn gt(&self, compare: &StrColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }

    fn ge(&self, compare: &StrColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x >= y),
                _ => None,
            })
            .collect()
    }

    fn lt(&self, compare: &StrColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x < y),
                _ => None,
            })
            .collect()
    }

    fn le(&self, compare: &StrColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x <= y),
                _ => None,
            })
            .collect()
    }

    fn equal(&self, compare: &StrColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x == y),
                _ => None,
            })
            .collect()
    }

    fn between(&self, lower: &StrColumn, upper: &StrColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(lower.0.iter())
            .zip(upper.0.iter())
            .map(|((x, lo), hi)| match (x, lo, hi) {
                (Some(x), Some(lo), Some(hi)) => Some(x >= lo && x <= hi),
                _ => None,
            })
            .collect()
    }
}

impl ComparableOps<&BoolColumn> for BoolColumn {
    fn gt(&self, compare: &BoolColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }

    fn ge(&self, compare: &BoolColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x >= y),
                _ => None,
            })
            .collect()
    }

    fn lt(&self, compare: &BoolColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x < y),
                _ => None,
            })
            .collect()
    }

    fn le(&self, compare: &BoolColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x <= y),
                _ => None,
            })
            .collect()
    }

    fn equal(&self, compare: &BoolColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x == y),
                _ => None,
            })
            .collect()
    }

    fn between(&self, lower: &BoolColumn, upper: &BoolColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(lower.0.iter())
            .zip(upper.0.iter())
            .map(|((x, lo), hi)| match (x, lo, hi) {
                (Some(x), Some(lo), Some(hi)) => Some(x >= lo && x <= hi),
                _ => None,
            })
            .collect()
    }
}

impl StringOps for Column {
    fn contains(&self, pat: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.contains(pat),
            _ => vec![None; self.len()],
        }
    }

    fn starts_with(&self, pat: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.starts_with(pat),
            _ => vec![None; self.len()],
        }
    }

    fn ends_with(&self, pat: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.ends_with(pat),
            _ => vec![None; self.len()],
        }
    }

    fn matches_regex(&self, pat: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.matches_regex(pat),
            _ => vec![None; self.len()],
        }
    }

    fn length(&self) -> Vec<Option<usize>> {
        match self {
            Column::Str(col) => col.length(),
            _ => vec![None; self.len()],
        }
    }
}

impl StringOps for StrColumn {
    fn contains(&self, pat: &str) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.contains(pat)))
            .collect()
    }

    fn starts_with(&self, pat: &str) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.starts_with(pat)))
            .collect()
    }

    fn ends_with(&self, pat: &str) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.ends_with(pat)))
            .collect()
    }

    fn matches_regex(&self, pat: &str) -> Vec<Option<bool>> {
        let re =
            Regex::new(pat).unwrap_or_else(|e| panic!("invalid regex pattern '{}': {}", pat, e));
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| re.is_match(s)))
            .collect()
    }

    fn length(&self) -> Vec<Option<usize>> {
        self.0.iter().map(|v| v.as_ref().map(|s| s.len())).collect()
    }
}
