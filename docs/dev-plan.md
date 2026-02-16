# Verdict Development Plan

## Phase 1: Dataset Foundation

### 1.1 Dataset Accessors

- [x] `get_column_by_name(name: &str) -> Option<&Column>`
- [x] `get_column_by_index(idx: usize) -> Option<&Column>`
- [x] `get_column_index(name: &str) -> Option<usize>`
- [x] `shape() -> (usize, usize)`
- [ ] Typed getters: `get_int_column`, `get_str_column`, etc.

### 1.2 Column Utilities

- [x] `len()`
- [x] `is_empty()`
- [x] `null_count()` / `not_null_count()`
- [x] `is_null() -> Vec<bool>`
- [x] `unique_count()` / `duplicates_count()`
- [x] `is_in(InSetValues)` — typed set membership check
- [ ] `unique_values()` for each type

### 1.3 Column Ops Traits

- [x] `NumericOps` — `sum`, `min`, `max`, `mean`, `std`, `median` (IntColumn, FloatColumn)
- [x] `ComparableOps<T>` — `gt`, `ge`, `lt`, `le`, `equal`, `between` (IntColumn<i64,f64>, FloatColumn<f64>, StrColumn<&str>)
- [x] `StringOps` — `contains`, `starts_with`, `ends_with`, `matches_regex`, `length` (StrColumn)
- [x] Column enum delegation for all ops (returns f64 for numeric, None for unsupported types)
- [ ] `DateTimeOps` — `year`, `month`, `day`, `between_dates`, `is_weekend` (deferred, no DateTimeColumn yet)

---

## Phase 2: Validation

### 2.1 Validation Results

- [x] Define `ValidationResult` struct (column, constraint, passed, failed_count, error)
- [x] `ValidationResult::passed()` / `ValidationResult::failed()` constructors
- [x] Track: passed/failed, failed count, error message
- [ ] Implement `Display` for human-readable output
- [ ] `Report` struct wrapping `Vec<ValidationResult>` with `all_passed()`, `failed()`

### 2.2 Rules System

- [x] `Rule` struct (column name + constraint)
- [x] `Constraint` enum with 14 variants
- [x] `validate(dataset, rules) -> Vec<ValidationResult>` public API
- [x] `validate_col_with_rule` dispatch + check functions
- [x] `ValidationError` enum (ColumnNotFound, UnknownConstraint, ColumnValidationError)
- [x] `InSetValues` typed enum (IntSet, FloatSet, StrSet)

#### Column-level constraints (all implemented):
- [x] `NotNull`, `Unique`
- [x] `GreaterThan`, `GreaterThanOrEqual`, `LessThan`, `LessThanOrEqual`, `Equal`, `Between`
- [x] `MatchesRegex`, `Contains`, `StartsWith`, `EndsWith`, `LengthBetween`
- [x] `InSet` (typed via InSetValues)

#### Not yet implemented:
- [ ] Row-level: `column_pair_unique`, `column_a_gt_b`

---

## Phase 3: Architecture Cleanup

### 3.1 Move CSV to verdict-csv

- [ ] `load_csv(path, schema) -> Dataset`
- [ ] Remove `csv` dependency from core
- [ ] Split errors

---

## Phase 4: Python Bindings

### 4.1 Basic Bindings

- [ ] Expose `Dataset`, `Schema`, `Field`, `DataType`
- [ ] `load_csv`, `shape()`

### 4.2 Validation Bindings

- [ ] Expose expectations + results

---

## Optional: Generic Column Refactor

Refactor separate column structs into a single generic `TypedColumn<T>` to eliminate duplicated trait impls via blanket implementations:

```rust
pub struct TypedColumn<T>(pub Vec<Option<T>>);

pub type IntColumn = TypedColumn<i64>;
pub type FloatColumn = TypedColumn<f64>;
pub type StrColumn = TypedColumn<String>;
pub type BoolColumn = TypedColumn<bool>;

// One impl covers all types
impl<T> ColumnOps for TypedColumn<T> {
    fn len(&self) -> usize { self.0.len() }
    fn null_count(&self) -> usize {
        self.0.iter().filter(|v| v.is_none()).count()
    }
}

// One impl covers all numeric types
impl<T> NumericOps for TypedColumn<T>
where T: Copy + std::iter::Sum + PartialOrd
{
    type Item = T;
    fn sum(&self) -> Option<T> {
        let vals: Vec<T> = self.0.iter().filter_map(|v| *v).collect();
        if vals.is_empty() { None } else { Some(vals.into_iter().sum()) }
    }
}
```

Consider this when trait impl duplication becomes painful (4+ types).

## ~~Optional: Generic ComparableOps~~ (Done)

Implemented `ComparableOps<T>` as generic trait. IntColumn supports both `i64` and `f64` comparison.
