# Task: Phase 7 — Date and DateTime Column Support

## Goal

Add `Date` and `DateTime` as first-class column types in `verdict-core`, wire them into CSV loading, expose constraints, and surface them in the CLI schema format.

---

## Background decisions (already made)

- **Storage**: `DateColumn(Vec<Option<i32>>)` — epoch days. `DateTimeColumn(Vec<Option<i64>>)` — epoch microseconds.
- **Chrono**: always-on dependency (not feature-gated). Used only at boundaries — parsing and display.
- **Hot path**: all comparisons run on raw `i32`/`i64` integers, chrono never called during validation.
- **CSV schema config**: `format` (string) and `unit` (integer epoch) are both optional.

---

## Step 1 — Add `DateColumn` and `DateTimeColumn` to `verdict-core`

Create two new column structs:

```
DateColumn(Vec<Option<i32>>)      — epoch days
DateTimeColumn(Vec<Option<i64>>)  — epoch microseconds
```

Requirements:
- Implement common column ops: `len`, `is_empty`, `null_count`, `not_null_count`, `is_null`, `unique_count`
- Add `Column::Date` and `Column::DateTime` variants to the `Column` enum
- Add `DataType::Date` and `DataType::DateTime` variants
- Wire up enum delegation for all common ops (same pattern as existing column types)

Hint: look at how `IntColumn` and `FloatColumn` are structured. Date/DateTime follow the same pattern — the only difference is what the integers mean semantically.

---

## Step 2 — Chrono conversion utilities

Add internal helper functions (not pub, these are implementation details):

- `naive_date_to_i32(d: NaiveDate) -> i32` — days since epoch
- `i32_to_naive_date(v: i32) -> NaiveDate` — reverse
- `naive_datetime_to_i64(dt: NaiveDateTime) -> i64` — microseconds since epoch
- `i64_to_naive_datetime(v: i64) -> NaiveDateTime` — reverse

Hint: `chrono::NaiveDate` has `num_days_from_ce()` but that's days from year 1, not Unix epoch. You need days from `1970-01-01`. Look at how to get that offset correctly. Same problem for datetime — `timestamp_micros()` is what you want.

---

## Step 3 — Date constraints in `verdict-core`

Add three new `Constraint` variants:

```
After(String)                        — all values strictly after given date
Before(String)                       — all values strictly before given date
BetweenDates { min: String, max: String }
```

Requirements:
- Constraint values are stored as strings (e.g. `"2024-01-01"`)
- At validation time: parse the threshold string to `i32`/`i64` once, then compare column values as integers
- `NotNull` and `Unique` already work — they operate on `Column` enum level, wire them up for Date/DateTime
- Add `check_date` and `check_datetime` functions following the same pattern as existing `check_*` functions
- Add dispatch in `validate_col_with_rule` for the new column types

Hint: the threshold string needs a default format to parse against. Use `"%Y-%m-%d"` for Date and `"%Y-%m-%dT%H:%M:%S"` for DateTime as the canonical constraint format — this is separate from the CSV loading format.

---

## Step 4 — CSV loading for Date and DateTime

Extend the `csv_loader` module to handle the new types.

Schema config changes in `ColumnConfig`:

```
format: Option<String>   — chrono format string, e.g. "%Y-%m-%d"
unit:   Option<String>   — "s", "ms", "us", "ns" — for integer epoch columns
```

Parsing logic per column:

- `format` present → read cell as string, parse via `NaiveDate::parse_from_str` / `NaiveDateTime::parse_from_str` using the given format, convert to i32/i64
- `unit` present → read cell as integer, multiply/divide to normalize to epoch days (i32) or epoch microseconds (i64)
- neither → try a default format list, fail with a clear error if none match

Default format lists to try:
- Date: `["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y"]`
- DateTime: `["%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S"]`

Requirements:
- Add `DataType::Date` and `DataType::DateTime` to the schema JSON parsing (`"type": "date"`, `"type": "datetime"`)
- Parse errors should produce a clear `CsvLoadingError` variant — include row number, column name, and the offending value
- Null handling: empty cell → `None`, same as existing types

---

## Step 5 — CLI schema format

Update the JSON schema format in `verdict-cli` to support the new types.

Example valid configs:

```json
{ "name": "created_at", "type": "date",     "format": "%Y-%m-%d" }
{ "name": "created_at", "type": "date" }
{ "name": "ts",         "type": "datetime", "format": "%Y-%m-%dT%H:%M:%S" }
{ "name": "ts",         "type": "datetime", "unit": "ms" }
```

Constraints in CLI schema:

```json
{ "constraint": "after",         "value": "2024-01-01" }
{ "constraint": "before",        "value": "2024-12-31" }
{ "constraint": "between_dates", "min": "2024-01-01", "max": "2024-12-31" }
```

Requirements:
- `ConstraintConfig` needs new variants for the three date constraints
- Propagate `format` and `unit` fields through `ColumnConfig` into the CSV loader call

---

## Step 6 — Tests

Write tests for:

- `DateColumn` and `DateTimeColumn` common ops (null_count, unique_count, etc.)
- Each constraint: `After`, `Before`, `BetweenDates` — passing and failing cases, nulls, edge values (boundary inclusive/exclusive)
- CSV loading: string format, epoch integer, default format fallback, parse error on bad value
- Round-trip: parse from CSV → validate with date constraint → correct result

Tests for CSV loading go under `#[cfg(feature = "csv")]` same as existing CSV tests.

---

## Cargo.toml changes

Add to `verdict-core/Cargo.toml`:

```toml
chrono = { version = "0.4", default-features = false, features = ["std"] }
```

No feature flag — always-on.

---

## Definition of done

- [ ] `DateColumn` and `DateTimeColumn` exist with common ops
- [ ] `DataType::Date` and `DataType::DateTime` in schema
- [ ] `Constraint::After`, `Before`, `BetweenDates` implemented and tested
- [ ] CSV loader handles `date` and `datetime` types with `format` and `unit` options
- [ ] CLI schema format accepts the new types and constraints
- [ ] All existing tests still pass (`cargo test --all-features`)
- [ ] New tests cover happy path, nulls, and error cases
