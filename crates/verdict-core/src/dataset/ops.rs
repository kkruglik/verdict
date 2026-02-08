use super::{BoolColumn, FloatColumn, IntColumn, StrColumn};

pub trait NumericOps {
    type Item;
    fn sum(&self) -> Option<Self::Item>;
    fn min(&self) -> Option<Self::Item>;
    fn max(&self) -> Option<Self::Item>;
    fn mean(&self) -> Option<f64>;
    fn std(&self) -> Option<f64>;
    fn median(&self) -> Option<f64>;
}

pub trait ComparableOps {
    type Item;
    fn gt(&self, compare: Self::Item) -> Vec<bool>;
    fn ge(&self, compare: Self::Item) -> Vec<bool>;
    fn lt(&self, compare: Self::Item) -> Vec<bool>;
    fn le(&self, compare: Self::Item) -> Vec<bool>;
    fn equal(&self, compare: Self::Item) -> Vec<bool>;
    fn between(&self, lower: Self::Item, upper: Self::Item) -> Vec<bool>;
}

pub trait StringOps {
    fn contains(&self, pat: &str) -> Vec<bool>;
    fn starts_with(&self, pat: &str) -> Vec<bool>;
    fn ends_with(&self, pat: &str) -> Vec<bool>;
    fn matches_regex(&self, pat: &str) -> Vec<bool>;
    fn length(&self) -> Vec<Option<usize>>;
}

impl NumericOps for IntColumn {
    type Item = i64;

    fn sum(&self) -> Option<Self::Item> {
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
