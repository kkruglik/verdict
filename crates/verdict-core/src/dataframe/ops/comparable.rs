use std::str::FromStr;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::dataframe::{
    BoolColumn, Column, DateColumn, DateTimeColumn, FloatColumn, IntColumn, StringColumn,
    TimeColumn,
    column::TypedColumn,
    ops::{naive_date_to_i32, naive_datetime_to_i64, naive_time_to_i64},
};

pub trait ComparableOps<T> {
    type Output;
    fn gt(&self, compare: T) -> Self::Output;
    fn ge(&self, compare: T) -> Self::Output;
    fn lt(&self, compare: T) -> Self::Output;
    fn le(&self, compare: T) -> Self::Output;
    fn equal(&self, compare: T) -> Self::Output;
    fn between(&self, lower: T, upper: T) -> Self::Output;
}

impl<T, M> ComparableOps<T> for TypedColumn<T, M>
where
    T: Copy + PartialOrd,
{
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: T) -> Self::Output {
        self.0.iter().map(|v| v.map(|x| x > compare)).collect()
    }

    fn ge(&self, compare: T) -> Self::Output {
        self.0.iter().map(|v| v.map(|x| x >= compare)).collect()
    }

    fn lt(&self, compare: T) -> Self::Output {
        self.0.iter().map(|v| v.map(|x| x < compare)).collect()
    }

    fn le(&self, compare: T) -> Self::Output {
        self.0.iter().map(|v| v.map(|x| x <= compare)).collect()
    }

    fn equal(&self, compare: T) -> Self::Output {
        self.0.iter().map(|v| v.map(|x| x == compare)).collect()
    }

    fn between(&self, lower: T, upper: T) -> Self::Output {
        self.0
            .iter()
            .map(|v| v.map(|x| x >= lower && x <= upper))
            .collect()
    }
}

impl ComparableOps<&NaiveDate> for DateColumn {
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &NaiveDate) -> Self::Output {
        let epochs = naive_date_to_i32(compare);
        self.0.iter().map(|v| v.map(|x| x > epochs)).collect()
    }

    fn ge(&self, compare: &NaiveDate) -> Self::Output {
        let epochs = naive_date_to_i32(compare);
        self.0.iter().map(|v| v.map(|x| x >= epochs)).collect()
    }

    fn lt(&self, compare: &NaiveDate) -> Self::Output {
        let epochs = naive_date_to_i32(compare);
        self.0.iter().map(|v| v.map(|x| x < epochs)).collect()
    }

    fn le(&self, compare: &NaiveDate) -> Self::Output {
        let epochs = naive_date_to_i32(compare);
        self.0.iter().map(|v| v.map(|x| x <= epochs)).collect()
    }

    fn equal(&self, compare: &NaiveDate) -> Self::Output {
        let epochs = naive_date_to_i32(compare);
        self.0.iter().map(|v| v.map(|x| x == epochs)).collect()
    }

    fn between(&self, lower: &NaiveDate, upper: &NaiveDate) -> Self::Output {
        let lower_epochs = naive_date_to_i32(lower);
        let upper_epochs = naive_date_to_i32(upper);
        self.0
            .iter()
            .map(|v| v.map(|x| x >= lower_epochs && x <= upper_epochs))
            .collect()
    }
}

impl ComparableOps<&NaiveTime> for TimeColumn {
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &NaiveTime) -> Self::Output {
        let micros = naive_time_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x > micros)).collect()
    }

    fn ge(&self, compare: &NaiveTime) -> Self::Output {
        let micros = naive_time_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x >= micros)).collect()
    }

    fn lt(&self, compare: &NaiveTime) -> Self::Output {
        let micros = naive_time_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x < micros)).collect()
    }

    fn le(&self, compare: &NaiveTime) -> Self::Output {
        let micros = naive_time_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x <= micros)).collect()
    }

    fn equal(&self, compare: &NaiveTime) -> Self::Output {
        let micros = naive_time_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x == micros)).collect()
    }

    fn between(&self, lower: &NaiveTime, upper: &NaiveTime) -> Self::Output {
        let lower_micros = naive_time_to_i64(lower);
        let upper_micros = naive_time_to_i64(upper);
        self.0
            .iter()
            .map(|v| v.map(|x| x >= lower_micros && x <= upper_micros))
            .collect()
    }
}

impl ComparableOps<&DateColumn> for DateColumn {
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &DateColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }

    fn ge(&self, compare: &DateColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x >= y),
                _ => None,
            })
            .collect()
    }

    fn lt(&self, compare: &DateColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x < y),
                _ => None,
            })
            .collect()
    }

    fn le(&self, compare: &DateColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x <= y),
                _ => None,
            })
            .collect()
    }

    fn equal(&self, compare: &DateColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x == y),
                _ => None,
            })
            .collect()
    }

    fn between(&self, lower: &DateColumn, upper: &DateColumn) -> Self::Output {
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

impl ComparableOps<&NaiveDateTime> for DateTimeColumn {
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &NaiveDateTime) -> Self::Output {
        let ts = naive_datetime_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x > ts)).collect()
    }

    fn ge(&self, compare: &NaiveDateTime) -> Self::Output {
        let ts = naive_datetime_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x >= ts)).collect()
    }

    fn lt(&self, compare: &NaiveDateTime) -> Self::Output {
        let ts = naive_datetime_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x < ts)).collect()
    }

    fn le(&self, compare: &NaiveDateTime) -> Self::Output {
        let ts = naive_datetime_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x <= ts)).collect()
    }

    fn equal(&self, compare: &NaiveDateTime) -> Self::Output {
        let ts = naive_datetime_to_i64(compare);
        self.0.iter().map(|v| v.map(|x| x == ts)).collect()
    }

    fn between(&self, lower: &NaiveDateTime, upper: &NaiveDateTime) -> Self::Output {
        let lower_ts = naive_datetime_to_i64(lower);
        let upper_ts = naive_datetime_to_i64(upper);
        self.0
            .iter()
            .map(|v| v.map(|x| x >= lower_ts && x <= upper_ts))
            .collect()
    }
}

impl ComparableOps<&TimeColumn> for TimeColumn {
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &TimeColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }

    fn ge(&self, compare: &TimeColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x >= y),
                _ => None,
            })
            .collect()
    }

    fn lt(&self, compare: &TimeColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x < y),
                _ => None,
            })
            .collect()
    }

    fn le(&self, compare: &TimeColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x <= y),
                _ => None,
            })
            .collect()
    }

    fn equal(&self, compare: &TimeColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x == y),
                _ => None,
            })
            .collect()
    }

    fn between(&self, lower: &TimeColumn, upper: &TimeColumn) -> Self::Output {
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
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &DateTimeColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }

    fn ge(&self, compare: &DateTimeColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x >= y),
                _ => None,
            })
            .collect()
    }

    fn lt(&self, compare: &DateTimeColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x < y),
                _ => None,
            })
            .collect()
    }

    fn le(&self, compare: &DateTimeColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x <= y),
                _ => None,
            })
            .collect()
    }

    fn equal(&self, compare: &DateTimeColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x == y),
                _ => None,
            })
            .collect()
    }

    fn between(&self, lower: &DateTimeColumn, upper: &DateTimeColumn) -> Self::Output {
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
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: f64) -> Self::Output {
        self.0
            .iter()
            .map(|v| v.map(|x| (x as f64) > compare))
            .collect()
    }

    fn ge(&self, compare: f64) -> Self::Output {
        self.0
            .iter()
            .map(|v| v.map(|x| (x as f64) >= compare))
            .collect()
    }

    fn lt(&self, compare: f64) -> Self::Output {
        self.0
            .iter()
            .map(|v| v.map(|x| (x as f64) < compare))
            .collect()
    }

    fn le(&self, compare: f64) -> Self::Output {
        self.0
            .iter()
            .map(|v| v.map(|x| (x as f64) <= compare))
            .collect()
    }

    fn equal(&self, compare: f64) -> Self::Output {
        self.0
            .iter()
            .map(|v| v.map(|x| (x as f64) == compare))
            .collect()
    }

    fn between(&self, lower: f64, upper: f64) -> Self::Output {
        self.0
            .iter()
            .map(|v| v.map(|x| (x as f64) >= lower && (x as f64) <= upper))
            .collect()
    }
}

// TODO: when `compare` fails to parse for Date/DateTime/Time arms, we return vec![None; N].
// Column-check functions filter failures by `Some(false)`, so None rows are invisible and the
// constraint silently passes. This is only a problem if callers use generic `gt`/`ge`/`lt`/`le`/
// `eq`/`between` with a string operand on a temporal column instead of the dedicated
// `after`/`before`/`between_dates` constraints (which validate the string and surface an error).
// Fixing this properly requires either changing the trait return type to Result<Self::Output>
// or parsing the operand upstream before dispatch.
impl ComparableOps<&str> for Column {
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &str) -> Self::Output {
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

    fn ge(&self, compare: &str) -> Self::Output {
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

    fn lt(&self, compare: &str) -> Self::Output {
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

    fn le(&self, compare: &str) -> Self::Output {
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

    fn equal(&self, compare: &str) -> Self::Output {
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

    fn between(&self, lower: &str, upper: &str) -> Self::Output {
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
    type Output = Vec<Option<bool>>;
    fn ge(&self, compare: f64) -> Self::Output {
        match self {
            Column::Int(col) => col.ge(compare),
            Column::Float(col) => col.ge(compare),
            _ => vec![None; self.len()],
        }
    }
    fn gt(&self, compare: f64) -> Self::Output {
        match self {
            Column::Int(col) => col.gt(compare),
            Column::Float(col) => col.gt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: f64) -> Self::Output {
        match self {
            Column::Int(col) => col.lt(compare),
            Column::Float(col) => col.lt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: f64) -> Self::Output {
        match self {
            Column::Int(col) => col.le(compare),
            Column::Float(col) => col.le(compare),
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: f64) -> Self::Output {
        match self {
            Column::Int(col) => col.equal(compare),
            Column::Float(col) => col.equal(compare),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: f64, upper: f64) -> Self::Output {
        match self {
            Column::Int(col) => col.between(lower, upper),
            Column::Float(col) => col.between(lower, upper),
            _ => vec![None; self.len()],
        }
    }
}

impl ComparableOps<i32> for Column {
    type Output = Vec<Option<bool>>;
    fn ge(&self, compare: i32) -> Self::Output {
        match self {
            Column::Date(col) => col.ge(compare),
            _ => vec![None; self.len()],
        }
    }
    fn gt(&self, compare: i32) -> Self::Output {
        match self {
            Column::Date(col) => col.gt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: i32) -> Self::Output {
        match self {
            Column::Date(col) => col.lt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: i32) -> Self::Output {
        match self {
            Column::Date(col) => col.le(compare),
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: i32) -> Self::Output {
        match self {
            Column::Date(col) => col.equal(compare),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: i32, upper: i32) -> Self::Output {
        match self {
            Column::Date(col) => col.between(lower, upper),
            _ => vec![None; self.len()],
        }
    }
}

impl ComparableOps<i64> for Column {
    type Output = Vec<Option<bool>>;
    fn ge(&self, compare: i64) -> Self::Output {
        match self {
            Column::DateTime(col) => col.ge(compare),
            Column::Time(col) => col.ge(compare),
            _ => vec![None; self.len()],
        }
    }
    fn gt(&self, compare: i64) -> Self::Output {
        match self {
            Column::DateTime(col) => col.gt(compare),
            Column::Time(col) => col.gt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: i64) -> Self::Output {
        match self {
            Column::DateTime(col) => col.lt(compare),
            Column::Time(col) => col.lt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: i64) -> Self::Output {
        match self {
            Column::DateTime(col) => col.le(compare),
            Column::Time(col) => col.le(compare),
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: i64) -> Self::Output {
        match self {
            Column::DateTime(col) => col.equal(compare),
            Column::Time(col) => col.equal(compare),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: i64, upper: i64) -> Self::Output {
        match self {
            Column::DateTime(col) => col.between(lower, upper),
            Column::Time(col) => col.between(lower, upper),
            _ => vec![None; self.len()],
        }
    }
}

impl ComparableOps<&NaiveDate> for Column {
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &NaiveDate) -> Self::Output {
        match &self {
            Column::Date(col) => col.gt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn ge(&self, compare: &NaiveDate) -> Self::Output {
        match &self {
            Column::Date(col) => col.ge(compare),
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: &NaiveDate) -> Self::Output {
        match &self {
            Column::Date(col) => col.lt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: &NaiveDate) -> Self::Output {
        match &self {
            Column::Date(col) => col.le(compare),
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: &NaiveDate) -> Self::Output {
        match &self {
            Column::Date(col) => col.equal(compare),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: &NaiveDate, upper: &NaiveDate) -> Self::Output {
        match &self {
            Column::Date(col) => col.between(lower, upper),
            _ => vec![None; self.len()],
        }
    }
}

impl ComparableOps<&NaiveTime> for Column {
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &NaiveTime) -> Self::Output {
        match &self {
            Column::Time(col) => col.gt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn ge(&self, compare: &NaiveTime) -> Self::Output {
        match &self {
            Column::Time(col) => col.ge(compare),
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: &NaiveTime) -> Self::Output {
        match &self {
            Column::Time(col) => col.lt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: &NaiveTime) -> Self::Output {
        match &self {
            Column::Time(col) => col.le(compare),
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: &NaiveTime) -> Self::Output {
        match &self {
            Column::Time(col) => col.equal(compare),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: &NaiveTime, upper: &NaiveTime) -> Self::Output {
        match &self {
            Column::Time(col) => col.between(lower, upper),
            _ => vec![None; self.len()],
        }
    }
}

impl ComparableOps<&NaiveDateTime> for Column {
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &NaiveDateTime) -> Self::Output {
        match &self {
            Column::DateTime(col) => col.gt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn ge(&self, compare: &NaiveDateTime) -> Self::Output {
        match &self {
            Column::DateTime(col) => col.ge(compare),
            _ => vec![None; self.len()],
        }
    }

    fn lt(&self, compare: &NaiveDateTime) -> Self::Output {
        match &self {
            Column::DateTime(col) => col.lt(compare),
            _ => vec![None; self.len()],
        }
    }

    fn le(&self, compare: &NaiveDateTime) -> Self::Output {
        match &self {
            Column::DateTime(col) => col.le(compare),
            _ => vec![None; self.len()],
        }
    }

    fn equal(&self, compare: &NaiveDateTime) -> Self::Output {
        match &self {
            Column::DateTime(col) => col.equal(compare),
            _ => vec![None; self.len()],
        }
    }

    fn between(&self, lower: &NaiveDateTime, upper: &NaiveDateTime) -> Self::Output {
        match &self {
            Column::DateTime(col) => col.between(lower, upper),
            _ => vec![None; self.len()],
        }
    }
}

impl ComparableOps<&IntColumn> for IntColumn {
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &IntColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }

    fn ge(&self, compare: &IntColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x >= y),
                _ => None,
            })
            .collect()
    }

    fn lt(&self, compare: &IntColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x < y),
                _ => None,
            })
            .collect()
    }

    fn le(&self, compare: &IntColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x <= y),
                _ => None,
            })
            .collect()
    }

    fn equal(&self, compare: &IntColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x == y),
                _ => None,
            })
            .collect()
    }

    fn between(&self, lower: &IntColumn, upper: &IntColumn) -> Self::Output {
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
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &FloatColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }
    fn ge(&self, compare: &FloatColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x >= y),
                _ => None,
            })
            .collect()
    }

    fn lt(&self, compare: &FloatColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x < y),
                _ => None,
            })
            .collect()
    }

    fn le(&self, compare: &FloatColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x <= y),
                _ => None,
            })
            .collect()
    }

    fn equal(&self, compare: &FloatColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x == y),
                _ => None,
            })
            .collect()
    }

    fn between(&self, lower: &FloatColumn, upper: &FloatColumn) -> Self::Output {
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
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &str) -> Self::Output {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.as_str() > compare))
            .collect()
    }

    fn ge(&self, compare: &str) -> Self::Output {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.as_str() >= compare))
            .collect()
    }

    fn lt(&self, compare: &str) -> Self::Output {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.as_str() < compare))
            .collect()
    }

    fn le(&self, compare: &str) -> Self::Output {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.as_str() <= compare))
            .collect()
    }

    fn equal(&self, compare: &str) -> Self::Output {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.as_str() == compare))
            .collect()
    }

    fn between(&self, lower: &str, upper: &str) -> Self::Output {
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
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &StringColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }

    fn ge(&self, compare: &StringColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x >= y),
                _ => None,
            })
            .collect()
    }

    fn lt(&self, compare: &StringColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x < y),
                _ => None,
            })
            .collect()
    }

    fn le(&self, compare: &StringColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x <= y),
                _ => None,
            })
            .collect()
    }

    fn equal(&self, compare: &StringColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x == y),
                _ => None,
            })
            .collect()
    }

    fn between(&self, lower: &StringColumn, upper: &StringColumn) -> Self::Output {
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
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &BoolColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }

    fn ge(&self, compare: &BoolColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x >= y),
                _ => None,
            })
            .collect()
    }

    fn lt(&self, compare: &BoolColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x < y),
                _ => None,
            })
            .collect()
    }

    fn le(&self, compare: &BoolColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x <= y),
                _ => None,
            })
            .collect()
    }

    fn equal(&self, compare: &BoolColumn) -> Self::Output {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x == y),
                _ => None,
            })
            .collect()
    }

    fn between(&self, lower: &BoolColumn, upper: &BoolColumn) -> Self::Output {
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

impl ComparableOps<&Column> for Column {
    type Output = Vec<Option<bool>>;
    fn gt(&self, compare: &Column) -> Self::Output {
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

    fn ge(&self, compare: &Column) -> Self::Output {
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

    fn lt(&self, compare: &Column) -> Self::Output {
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

    fn le(&self, compare: &Column) -> Self::Output {
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

    fn equal(&self, compare: &Column) -> Self::Output {
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

    fn between(&self, lower: &Column, upper: &Column) -> Self::Output {
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
