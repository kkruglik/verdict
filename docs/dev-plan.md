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
- [ ] `unique_values()` for each type

### 1.3 Column Ops Traits

- [ ] `NumericOps` — `sum`, `min`, `max`, `mean`, `std_dev`
- [ ] `ComparableOps` — `gt`, `ge`, `lt`, `le`, `between`, `equal` (Int, Float, Date, String)
- [ ] `DateTimeOps` — `year`, `month`, `day`, `between_dates`, `is_weekend`
- [ ] `StringOps` — `contains`, `starts_with`, `ends_with`, `matches_regex`, `length`

---

## Phase 2: Validation

### 2.1 Validation Results

- [ ] Define `ValidationResult` struct
- [ ] Track: passed/failed, which rows failed, failure reasons
- [ ] Implement `Display` for human-readable output

### 2.2 Expectation System

- [ ] Define `Expectation` trait/enum
- [ ] Column-level: `not_null`, `unique`, `in_set`, `in_range`, `matches_regex`
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

## Optional: Generic ComparableOps

Support comparing `IntColumn` against both `i64` and `f64` via generic trait parameter:

```rust
pub trait ComparableOps<T> {
    fn gt(&self, compare: T) -> Vec<bool>;
    // ...
}

impl ComparableOps<i64> for IntColumn { ... }
impl ComparableOps<f64> for IntColumn { ... }
```

Consider this when expectation system needs cross-type comparisons.
