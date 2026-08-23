pub mod comparable;
pub mod numeric;
pub mod string;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};

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
