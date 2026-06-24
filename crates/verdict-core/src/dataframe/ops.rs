use std::str::FromStr;

use crate::dataframe::{
    Column,
    column::{DateColumn, DateTimeColumn, TimeColumn},
};

use super::{BoolColumn, FloatColumn, IntColumn, StringColumn};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};
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

impl ComparableOps<i32> for DateColumn {
    fn gt(&self, compare: i32) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x > compare)).collect()
    }

    fn ge(&self, compare: i32) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x >= compare)).collect()
    }

    fn lt(&self, compare: i32) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x < compare)).collect()
    }

    fn le(&self, compare: i32) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x <= compare)).collect()
    }

    fn equal(&self, compare: i32) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x == compare)).collect()
    }

    fn between(&self, lower: i32, upper: i32) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.map(|x| x >= lower && x <= upper))
            .collect()
    }
}

impl ComparableOps<&NaiveDate> for DateColumn {
    fn gt(&self, compare: &NaiveDate) -> Vec<Option<bool>> {
        let epochs = naive_date_to_i32(compare);
        self.0.iter().map(|v| v.map(|x| x > epochs)).collect()
    }

    fn ge(&self, compare: &NaiveDate) -> Vec<Option<bool>> {
        let epochs = naive_date_to_i32(compare);
        self.0.iter().map(|v| v.map(|x| x >= epochs)).collect()
    }

    fn lt(&self, compare: &NaiveDate) -> Vec<Option<bool>> {
        let epochs = naive_date_to_i32(compare);
        self.0.iter().map(|v| v.map(|x| x < epochs)).collect()
    }

    fn le(&self, compare: &NaiveDate) -> Vec<Option<bool>> {
        let epochs = naive_date_to_i32(compare);
        self.0.iter().map(|v| v.map(|x| x <= epochs)).collect()
    }

    fn equal(&self, compare: &NaiveDate) -> Vec<Option<bool>> {
        let epochs = naive_date_to_i32(compare);
        self.0.iter().map(|v| v.map(|x| x == epochs)).collect()
    }

    fn between(&self, lower: &NaiveDate, upper: &NaiveDate) -> Vec<Option<bool>> {
        let lower_epochs = naive_date_to_i32(lower);
        let upper_epochs = naive_date_to_i32(upper);
        self.0
            .iter()
            .map(|v| v.map(|x| x >= lower_epochs && x <= upper_epochs))
            .collect()
    }
}

impl ComparableOps<i64> for TimeColumn {
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

impl ComparableOps<&NaiveTime> for TimeColumn {
    fn gt(&self, compare: &NaiveTime) -> Vec<Option<bool>> {
        let micros = naive_time_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x > micros)).collect()
    }

    fn ge(&self, compare: &NaiveTime) -> Vec<Option<bool>> {
        let micros = naive_time_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x >= micros)).collect()
    }

    fn lt(&self, compare: &NaiveTime) -> Vec<Option<bool>> {
        let micros = naive_time_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x < micros)).collect()
    }

    fn le(&self, compare: &NaiveTime) -> Vec<Option<bool>> {
        let micros = naive_time_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x <= micros)).collect()
    }

    fn equal(&self, compare: &NaiveTime) -> Vec<Option<bool>> {
        let micros = naive_time_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x == micros)).collect()
    }

    fn between(&self, lower: &NaiveTime, upper: &NaiveTime) -> Vec<Option<bool>> {
        let lower_micros = naive_time_to_i64(lower);
        let upper_micros = naive_time_to_i64(upper);
        self.0
            .iter()
            .map(|v| v.map(|x| x >= lower_micros && x <= upper_micros))
            .collect()
    }
}

impl ComparableOps<&DateColumn> for DateColumn {
    fn gt(&self, compare: &DateColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }

    fn ge(&self, compare: &DateColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x >= y),
                _ => None,
            })
            .collect()
    }

    fn lt(&self, compare: &DateColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x < y),
                _ => None,
            })
            .collect()
    }

    fn le(&self, compare: &DateColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x <= y),
                _ => None,
            })
            .collect()
    }

    fn equal(&self, compare: &DateColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x == y),
                _ => None,
            })
            .collect()
    }

    fn between(&self, lower: &DateColumn, upper: &DateColumn) -> Vec<Option<bool>> {
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
impl ComparableOps<i64> for DateTimeColumn {
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

impl ComparableOps<&NaiveDateTime> for DateTimeColumn {
    fn gt(&self, compare: &NaiveDateTime) -> Vec<Option<bool>> {
        let ts = naive_datetime_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x > ts)).collect()
    }

    fn ge(&self, compare: &NaiveDateTime) -> Vec<Option<bool>> {
        let ts = naive_datetime_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x >= ts)).collect()
    }

    fn lt(&self, compare: &NaiveDateTime) -> Vec<Option<bool>> {
        let ts = naive_datetime_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x < ts)).collect()
    }

    fn le(&self, compare: &NaiveDateTime) -> Vec<Option<bool>> {
        let ts = naive_datetime_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x <= ts)).collect()
    }

    fn equal(&self, compare: &NaiveDateTime) -> Vec<Option<bool>> {
        let ts = naive_datetime_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x == ts)).collect()
    }

    fn between(&self, lower: &NaiveDateTime, upper: &NaiveDateTime) -> Vec<Option<bool>> {
        let lower_ts = naive_datetime_to_i64(lower);
        let upper_ts = naive_datetime_to_i64(upper);
        self.0
            .iter()
            .map(|v| v.map(|x| x >= lower_ts && x <= upper_ts))
            .collect()
    }
}

impl ComparableOps<&TimeColumn> for TimeColumn {
    fn gt(&self, compare: &TimeColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }

    fn ge(&self, compare: &TimeColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x >= y),
                _ => None,
            })
            .collect()
    }

    fn lt(&self, compare: &TimeColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x < y),
                _ => None,
            })
            .collect()
    }

    fn le(&self, compare: &TimeColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x <= y),
                _ => None,
            })
            .collect()
    }

    fn equal(&self, compare: &TimeColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x == y),
                _ => None,
            })
            .collect()
    }

    fn between(&self, lower: &TimeColumn, upper: &TimeColumn) -> Vec<Option<bool>> {
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
impl ComparableOps<&DateTimeColumn> for DateTimeColumn {
    fn gt(&self, compare: &DateTimeColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }

    fn ge(&self, compare: &DateTimeColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x >= y),
                _ => None,
            })
            .collect()
    }

    fn lt(&self, compare: &DateTimeColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x < y),
                _ => None,
            })
            .collect()
    }

    fn le(&self, compare: &DateTimeColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x <= y),
                _ => None,
            })
            .collect()
    }

    fn equal(&self, compare: &DateTimeColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x == y),
                _ => None,
            })
            .collect()
    }

    fn between(&self, lower: &DateTimeColumn, upper: &DateTimeColumn) -> Vec<Option<bool>> {
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

// TODO: when `compare` fails to parse for Date/DateTime/Time arms, we return vec![None; N].
// Column-check functions filter failures by `Some(false)`, so None rows are invisible and the
// constraint silently passes. This is only a problem if callers use generic `gt`/`ge`/`lt`/`le`/
// `eq`/`between` with a string operand on a temporal column instead of the dedicated
// `after`/`before`/`between_dates` constraints (which validate the string and surface an error).
// Fixing this properly requires either changing the trait return type to Result<Vec<Option<bool>>>
// or parsing the operand upstream before dispatch.
impl ComparableOps<&str> for Column {
    fn gt(&self, compare: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.gt(compare),
            Column::Date(col) => NaiveDate::from_str(compare)
                .map_or_else(|_| vec![None; self.len()], |date| col.gt(&date)),
            Column::DateTime(col) => NaiveDateTime::from_str(compare)
                .map_or_else(|_| vec![None; self.len()], |date| col.gt(&date)),
            Column::Time(col) => NaiveTime::from_str(compare)
                .map_or_else(|_| vec![None; self.len()], |date| col.gt(&date)),
            _ => vec![None; self.len()],
        }
    }

    fn ge(&self, compare: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.ge(compare),
            Column::Date(col) => {
                NaiveDate::from_str(compare).map_or_else(|_| vec![None; self.len()], |d| col.ge(&d))
            }
            Column::DateTime(col) => NaiveDateTime::from_str(compare)
                .map_or_else(|_| vec![None; self.len()], |d| col.ge(&d)),
            Column::Time(col) => {
                NaiveTime::from_str(compare).map_or_else(|_| vec![None; self.len()], |d| col.ge(&d))
            }
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.lt(compare),
            Column::Date(col) => {
                NaiveDate::from_str(compare).map_or_else(|_| vec![None; self.len()], |d| col.lt(&d))
            }
            Column::DateTime(col) => NaiveDateTime::from_str(compare)
                .map_or_else(|_| vec![None; self.len()], |d| col.lt(&d)),
            Column::Time(col) => {
                NaiveTime::from_str(compare).map_or_else(|_| vec![None; self.len()], |d| col.lt(&d))
            }
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.le(compare),
            Column::Date(col) => {
                NaiveDate::from_str(compare).map_or_else(|_| vec![None; self.len()], |d| col.le(&d))
            }
            Column::DateTime(col) => NaiveDateTime::from_str(compare)
                .map_or_else(|_| vec![None; self.len()], |d| col.le(&d)),
            Column::Time(col) => {
                NaiveTime::from_str(compare).map_or_else(|_| vec![None; self.len()], |d| col.le(&d))
            }
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.equal(compare),
            Column::Date(col) => NaiveDate::from_str(compare)
                .map_or_else(|_| vec![None; self.len()], |d| col.equal(&d)),
            Column::DateTime(col) => NaiveDateTime::from_str(compare)
                .map_or_else(|_| vec![None; self.len()], |d| col.equal(&d)),
            Column::Time(col) => NaiveTime::from_str(compare)
                .map_or_else(|_| vec![None; self.len()], |d| col.equal(&d)),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: &str, upper: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.between(lower, upper),
            Column::Date(col) => match (NaiveDate::from_str(lower), NaiveDate::from_str(upper)) {
                (Ok(lo), Ok(hi)) => col.between(&lo, &hi),
                _ => vec![None; self.len()],
            },
            Column::DateTime(col) => match (
                NaiveDateTime::from_str(lower),
                NaiveDateTime::from_str(upper),
            ) {
                (Ok(lo), Ok(hi)) => col.between(&lo, &hi),
                _ => vec![None; self.len()],
            },
            Column::Time(col) => match (NaiveTime::from_str(lower), NaiveTime::from_str(upper)) {
                (Ok(lo), Ok(hi)) => col.between(&lo, &hi),
                _ => vec![None; self.len()],
            },
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

impl ComparableOps<i32> for Column {
    fn ge(&self, compare: i32) -> Vec<Option<bool>> {
        match self {
            Column::Date(col) => col.ge(compare),
            _ => vec![None; self.len()],
        }
    }
    fn gt(&self, compare: i32) -> Vec<Option<bool>> {
        match self {
            Column::Date(col) => col.gt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: i32) -> Vec<Option<bool>> {
        match self {
            Column::Date(col) => col.lt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: i32) -> Vec<Option<bool>> {
        match self {
            Column::Date(col) => col.le(compare),
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: i32) -> Vec<Option<bool>> {
        match self {
            Column::Date(col) => col.equal(compare),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: i32, upper: i32) -> Vec<Option<bool>> {
        match self {
            Column::Date(col) => col.between(lower, upper),
            _ => vec![None; self.len()],
        }
    }
}

impl ComparableOps<i64> for Column {
    fn ge(&self, compare: i64) -> Vec<Option<bool>> {
        match self {
            Column::DateTime(col) => col.ge(compare),
            Column::Time(col) => col.ge(compare),
            _ => vec![None; self.len()],
        }
    }
    fn gt(&self, compare: i64) -> Vec<Option<bool>> {
        match self {
            Column::DateTime(col) => col.gt(compare),
            Column::Time(col) => col.gt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: i64) -> Vec<Option<bool>> {
        match self {
            Column::DateTime(col) => col.lt(compare),
            Column::Time(col) => col.lt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: i64) -> Vec<Option<bool>> {
        match self {
            Column::DateTime(col) => col.le(compare),
            Column::Time(col) => col.le(compare),
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: i64) -> Vec<Option<bool>> {
        match self {
            Column::DateTime(col) => col.equal(compare),
            Column::Time(col) => col.equal(compare),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: i64, upper: i64) -> Vec<Option<bool>> {
        match self {
            Column::DateTime(col) => col.between(lower, upper),
            Column::Time(col) => col.between(lower, upper),
            _ => vec![None; self.len()],
        }
    }
}

impl ComparableOps<&NaiveDate> for Column {
    fn gt(&self, compare: &NaiveDate) -> Vec<Option<bool>> {
        match &self {
            Column::Date(col) => col.gt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn ge(&self, compare: &NaiveDate) -> Vec<Option<bool>> {
        match &self {
            Column::Date(col) => col.ge(compare),
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: &NaiveDate) -> Vec<Option<bool>> {
        match &self {
            Column::Date(col) => col.lt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: &NaiveDate) -> Vec<Option<bool>> {
        match &self {
            Column::Date(col) => col.le(compare),
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: &NaiveDate) -> Vec<Option<bool>> {
        match &self {
            Column::Date(col) => col.equal(compare),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: &NaiveDate, upper: &NaiveDate) -> Vec<Option<bool>> {
        match &self {
            Column::Date(col) => col.between(lower, upper),
            _ => vec![None; self.len()],
        }
    }
}

impl ComparableOps<&NaiveTime> for Column {
    fn gt(&self, compare: &NaiveTime) -> Vec<Option<bool>> {
        match &self {
            Column::Time(col) => col.gt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn ge(&self, compare: &NaiveTime) -> Vec<Option<bool>> {
        match &self {
            Column::Time(col) => col.ge(compare),
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: &NaiveTime) -> Vec<Option<bool>> {
        match &self {
            Column::Time(col) => col.lt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: &NaiveTime) -> Vec<Option<bool>> {
        match &self {
            Column::Time(col) => col.le(compare),
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: &NaiveTime) -> Vec<Option<bool>> {
        match &self {
            Column::Time(col) => col.equal(compare),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: &NaiveTime, upper: &NaiveTime) -> Vec<Option<bool>> {
        match &self {
            Column::Time(col) => col.between(lower, upper),
            _ => vec![None; self.len()],
        }
    }
}

impl ComparableOps<&NaiveDateTime> for Column {
    fn gt(&self, compare: &NaiveDateTime) -> Vec<Option<bool>> {
        match &self {
            Column::DateTime(col) => col.gt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn ge(&self, compare: &NaiveDateTime) -> Vec<Option<bool>> {
        match &self {
            Column::DateTime(col) => col.ge(compare),
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: &NaiveDateTime) -> Vec<Option<bool>> {
        match &self {
            Column::DateTime(col) => col.lt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: &NaiveDateTime) -> Vec<Option<bool>> {
        match &self {
            Column::DateTime(col) => col.le(compare),
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: &NaiveDateTime) -> Vec<Option<bool>> {
        match &self {
            Column::DateTime(col) => col.equal(compare),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: &NaiveDateTime, upper: &NaiveDateTime) -> Vec<Option<bool>> {
        match &self {
            Column::DateTime(col) => col.between(lower, upper),
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
            (Column::Date(a), Column::Date(b)) => a.gt(b),
            (Column::DateTime(a), Column::DateTime(b)) => a.gt(b),
            (Column::Time(a), Column::Time(b)) => a.gt(b),
            _ => vec![None; self.len()],
        }
    }

    fn ge(&self, compare: &Column) -> Vec<Option<bool>> {
        match (self, compare) {
            (Column::Float(a), Column::Float(b)) => a.ge(b),
            (Column::Int(a), Column::Int(b)) => a.ge(b),
            (Column::Str(a), Column::Str(b)) => a.ge(b),
            (Column::Bool(a), Column::Bool(b)) => a.ge(b),
            (Column::DateTime(a), Column::DateTime(b)) => a.ge(b),
            (Column::Time(a), Column::Time(b)) => a.ge(b),
            (Column::Date(a), Column::Date(b)) => a.ge(b),
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: &Column) -> Vec<Option<bool>> {
        match (self, compare) {
            (Column::Float(a), Column::Float(b)) => a.lt(b),
            (Column::Int(a), Column::Int(b)) => a.lt(b),
            (Column::Str(a), Column::Str(b)) => a.lt(b),
            (Column::Bool(a), Column::Bool(b)) => a.lt(b),
            (Column::DateTime(a), Column::DateTime(b)) => a.lt(b),
            (Column::Time(a), Column::Time(b)) => a.lt(b),
            (Column::Date(a), Column::Date(b)) => a.lt(b),
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: &Column) -> Vec<Option<bool>> {
        match (self, compare) {
            (Column::Float(a), Column::Float(b)) => a.le(b),
            (Column::Int(a), Column::Int(b)) => a.le(b),
            (Column::Str(a), Column::Str(b)) => a.le(b),
            (Column::Bool(a), Column::Bool(b)) => a.le(b),
            (Column::DateTime(a), Column::DateTime(b)) => a.le(b),
            (Column::Time(a), Column::Time(b)) => a.le(b),
            (Column::Date(a), Column::Date(b)) => a.le(b),
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: &Column) -> Vec<Option<bool>> {
        match (self, compare) {
            (Column::Float(a), Column::Float(b)) => a.equal(b),
            (Column::Int(a), Column::Int(b)) => a.equal(b),
            (Column::Str(a), Column::Str(b)) => a.equal(b),
            (Column::Bool(a), Column::Bool(b)) => a.equal(b),
            (Column::DateTime(a), Column::DateTime(b)) => a.equal(b),
            (Column::Time(a), Column::Time(b)) => a.equal(b),
            (Column::Date(a), Column::Date(b)) => a.equal(b),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: &Column, upper: &Column) -> Vec<Option<bool>> {
        match (self, lower, upper) {
            (Column::Float(a), Column::Float(lo), Column::Float(hi)) => a.between(lo, hi),
            (Column::Int(a), Column::Int(lo), Column::Int(hi)) => a.between(lo, hi),
            (Column::Str(a), Column::Str(lo), Column::Str(hi)) => a.between(lo, hi),
            (Column::Bool(a), Column::Bool(lo), Column::Bool(hi)) => a.between(lo, hi),
            (Column::DateTime(a), Column::DateTime(lo), Column::DateTime(hi)) => a.between(lo, hi),
            (Column::Date(a), Column::Date(lo), Column::Date(hi)) => a.between(lo, hi),
            (Column::Time(a), Column::Time(lo), Column::Time(hi)) => a.between(lo, hi),
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

impl ComparableOps<&str> for StringColumn {
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

impl ComparableOps<&StringColumn> for StringColumn {
    fn gt(&self, compare: &StringColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }

    fn ge(&self, compare: &StringColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x >= y),
                _ => None,
            })
            .collect()
    }

    fn lt(&self, compare: &StringColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x < y),
                _ => None,
            })
            .collect()
    }

    fn le(&self, compare: &StringColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x <= y),
                _ => None,
            })
            .collect()
    }

    fn equal(&self, compare: &StringColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x == y),
                _ => None,
            })
            .collect()
    }

    fn between(&self, lower: &StringColumn, upper: &StringColumn) -> Vec<Option<bool>> {
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

impl StringOps for StringColumn {
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

pub fn naive_date_to_i32(naive_date: &NaiveDate) -> i32 {
    naive_date.to_epoch_days()
}

pub fn naive_time_to_i64(naive_time: &NaiveTime) -> i64 {
    let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    (*naive_time - midnight).num_microseconds().unwrap()
}

pub fn naive_datetime_to_i64(naive_datetime: &NaiveDateTime) -> i64 {
    naive_datetime.and_utc().timestamp_micros()
}

pub fn i64_to_naive_datetime(v: i64) -> Option<NaiveDateTime> {
    DateTime::from_timestamp_micros(v).map(|dt| dt.naive_utc())
}

pub fn i32_to_naive_date(v: i32) -> Option<NaiveDate> {
    NaiveDate::from_epoch_days(v)
}

pub fn i64_to_naive_time(v: i64) -> Option<NaiveTime> {
    if v < 0 {
        return None;
    }
    let secs = v / 1_000_000;
    let nanos = (v % 1_000_000) * 1000;
    NaiveTime::from_num_seconds_from_midnight_opt(secs as u32, nanos as u32)
}
