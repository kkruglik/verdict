# Issue: Silent pass on bad string operands for temporal columns

## Problem

`GreaterThan(Operand::Str("bad-date"))` applied to a `DateColumn` silently reports
**passed** instead of an error. The same applies to `GreaterThanOrEqual`, `LessThan`,
`LessThanOrEqual`, `Equal`, and `Between` with `Operand::Str` on `Date`/`DateTime`/`Time`
columns.

The parse failure happens inside `ComparableOps<&str>` which returns `vec![None; n]`
when `NaiveDate::from_str` fails. The failure filter in `check_*_str` only looks for
`Some(false)`, so `None` rows are invisible — zero failures are reported and the
constraint is marked passed even though the operand is invalid.

## What already works correctly

`After`/`Before`/`BetweenDates` handle this correctly — they call `NaiveDate::from_str(s)?`
in `validate_cols_with_rule` and surface a `ValidationError::DateParseError` on bad input.
The generic comparison constraints are missing this guard.

## Fix

**File:** `crates/verdict-core/src/rules/column_checks.rs`

### 1 — Add helper before `validate_cols_with_rule`

```rust
fn validate_temporal_str(column: &Column, s: &str) -> Result<(), ValidationError> {
    match column {
        Column::Date(_) => { NaiveDate::from_str(s)?; }
        Column::DateTime(_) => { NaiveDateTime::from_str(s)?; }
        Column::Time(_) => { NaiveTime::from_str(s)?; }
        _ => {}
    }
    Ok(())
}
```

All required imports are already present.

### 2 — Guard each `Operand::Str` arm (5 single-operand constraints)

```rust
// Before
Operand::Str(v) => Ok(check_greater_than_str(column, v, rule, n)),

// After
Operand::Str(v) => {
    validate_temporal_str(column, v)?;
    Ok(check_greater_than_str(column, v, rule, n))
}
```

Apply to: `GreaterThan`, `GreaterThanOrEqual`, `LessThan`, `LessThanOrEqual`, `Equal`.

### 3 — Guard `Between` (two bounds)

```rust
(Operand::Str(lo), Operand::Str(hi)) => {
    validate_temporal_str(column, lo)?;
    validate_temporal_str(column, hi)?;
    Ok(check_between_str(column, lo, hi, rule, n))
}
```

### 4 — Update tests in `silent_parse_failure_tests` module

The three `test_validate_*` tests in `crates/verdict-core/tests/tests.rs` currently
assert `report.passed == true`. After the fix, change them to:

```rust
assert!(!report.passed);
assert!(report.results[0].error.is_some());
```

The four direct `ComparableOps<&str>` trait tests are unchanged — they correctly
document that the trait itself still returns `None` on parse failure (correct behavior
for null data rows at the raw API level).

## Preferred fix: associated `Output` type on `ComparableOps` (Polars pattern)

Rather than validating in `validate_cols_with_rule`, fix the root cause in the trait
itself. Polars uses the same design — `ChunkCompare` has an associated `type Item`
so typed impls return direct values while the untyped `Series` impl returns `Result`.

### 1 — Add `type Output` to the trait

```rust
pub trait ComparableOps<T> {
    type Output;
    fn gt(&self, compare: T) -> Self::Output;
    fn ge(&self, compare: T) -> Self::Output;
    fn lt(&self, compare: T) -> Self::Output;
    fn le(&self, compare: T) -> Self::Output;
    fn equal(&self, compare: T) -> Self::Output;
    fn between(&self, lower: T, upper: T) -> Self::Output;
}
```

### 2 — Add `type Output = Vec<Option<bool>>;` to all 24 typed impls

Every existing impl keeps its method bodies unchanged — Rust accepts the concrete
return type when it matches the declared `Output`. Only one line added per impl:

```rust
impl ComparableOps<i64> for IntColumn {
    type Output = Vec<Option<bool>>;   // ← add this
    fn gt(&self, compare: i64) -> Vec<Option<bool>> { ... }  // unchanged
    ...
}
```

Impls to update (all in `crates/verdict-core/src/dataframe/ops.rs`):
lines 147, 176, 205, 241, 270, 306, 374, 403, 439, 507, 576, 620,
751, 800, 843, 892, 936, 980, 1023, 1103, 1172, 1240, 1287, 1356.

### 3 — Rewrite `impl ComparableOps<&str> for Column` (line 656)

Change `Output` to `Result` and replace `map_or_else` with `?`:

```rust
impl ComparableOps<&str> for Column {
    type Output = Result<Vec<Option<bool>>, ValidationError>;

    fn gt(&self, compare: &str) -> Result<Vec<Option<bool>>, ValidationError> {
        match self {
            Column::Str(col) => Ok(col.gt(compare)),
            Column::Date(col) => Ok(col.gt(&NaiveDate::from_str(compare)?)),
            Column::DateTime(col) => Ok(col.gt(&NaiveDateTime::from_str(compare)?)),
            Column::Time(col) => Ok(col.gt(&NaiveTime::from_str(compare)?)),
            _ => Ok(vec![None; self.len()]),
        }
    }
    // ge, lt, le, equal — same pattern
    // between — parse both bounds with ?
}
```

Add `use crate::errors::ValidationError;` to ops.rs imports.

### 4 — Update `check_*_str` in `column_checks.rs`

Change return type from `ValidationResult` to `Result<ValidationResult, ValidationError>`
and add `?` to the mask call:

```rust
fn check_greater_than_str(...) -> Result<ValidationResult, ValidationError> {
    let mask = col.gt(value)?;   // ? propagates parse error
    ...
}
```

Six functions: `check_greater_than_str`, `check_greater_than_or_equal_str`,
`check_less_than_str`, `check_less_than_or_equal_str`, `check_equal_str`,
`check_between_str`.

In `validate_cols_with_rule`, remove the wrapping `Ok(...)`:
```rust
// before
Operand::Str(v) => Ok(check_greater_than_str(column, v, rule, n)),
// after
Operand::Str(v) => check_greater_than_str(column, v, rule, n),
```

### 5 — Update tests

In `silent_parse_failure_tests`, flip the three `test_validate_*` tests:
```rust
// before
assert!(report.passed);
assert_eq!(report.results[0].failed_count, Some(0));
// after
assert!(!report.passed);
assert!(report.results[0].error.is_some());
```

## Notes

- `ValidationError` already has `#[from] chrono::ParseError`, so `?` works with no new code.
- `validate_columns` (public) converts `Err` via `unwrap_or_else(|e| ValidationResult::failed(...))`,
  so errors surface as a failed result with an error message in the report.
- The four direct `ComparableOps<&str>` trait tests in `silent_parse_failure_tests`
  (`test_date_gt_bad_string_silently_passes` etc.) will no longer compile — `col.gt(...)`
  now returns `Result`, not `Vec`. Update them to `assert!(col.gt("bad").is_err())`.
- No other callers of `ComparableOps<&str> for Column` exist outside `check_*_str`
  and the test file — confirmed by grep across all crates.
