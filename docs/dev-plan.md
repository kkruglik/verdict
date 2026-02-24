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
- [x] Implement `Display` for human-readable output
- [ ] `Report` struct wrapping `Vec<ValidationResult>` with `all_passed()`, `failed()`

### Known Issues

- [x] `sum()` on all-null column returns `0.0` instead of `None` — fixed in `verdict-core` numeric ops

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

### 3.1 ~~Move CSV to verdict-csv~~ → Feature-gated CSV module (Done)

- [x] Created `csv_loader` module behind `#[cfg(feature = "csv")]` feature flag
- [x] `DatasetCsvExt` trait with `Dataset::from_csv(path, schema)`
- [x] `CsvLoadingError` owns all CSV errors (Io, Csv, Parse)
- [x] Removed `csv` dependency from core by default
- [x] Removed `DatasetError` from core (only `ValidationError` remains)
- [x] Core tests build datasets manually, CSV tests gated with `#[cfg(feature = "csv")]`
- [x] CI workflows updated with `--all-features`

---

## Phase 4: Python Bindings

### 4.1 Basic Bindings

- [x] Expose `Dataset`, `Schema`, `DataType` via PyO3 wrappers
- [x] `Dataset.from_csv(path, schema)`, `shape()`
- [x] `Column` with typed constructors: `integer`, `floating`, `string`, `boolean`
- [x] All column ops exposed: numeric, comparison, string, null checks, `is_in`
- [x] Dataset accessors: `get_column_by_name`, `get_column_by_index`, `get_column_index`
- [x] `__repr__` for `Dataset` and `Column`
- [x] Clean Python-facing names via `#[pyclass(name = "...")]`

### 4.2 Validation Bindings

- [x] Expose `Constraint` with all 14 variants as static constructors
- [x] Expose `Rule(column, constraint)`
- [x] Expose `validate(dataset, rules) -> list[ValidationResult]`
- [x] `ValidationResult` with getters: `column`, `constraint`, `is_passed`, `failed_count`, `error`
- [x] `__repr__` on `ValidationResult` using core `Display`

---

## Phase 5: CLI

**Goal:** zero-dependency data quality gate for CI pipelines. Drop a binary, point it at a CSV and a schema file, get exit code 0 or 1.

### 5.1 New crate: `verdict-cli`

- [ ] Add `crates/verdict-cli` as a binary crate depending on `verdict-core` (csv feature)
- [ ] `cargo build --release -p verdict-cli` produces a single static binary

### 5.2 Schema config file

- [ ] YAML schema format — column names + types + rules, no code required
- [ ] Example:
  ```yaml
  columns:
    user_id: integer
    score: float
    country: string

  rules:
    - column: user_id
      not_null: true
      unique: true
    - column: score
      between: [0, 100]
    - column: country
      is_in: [US, UK, DE, FR, JP]
  ```
- [ ] `SchemaConfig` and `RuleConfig` structs parsed with `serde` + `serde_yaml`

### 5.3 CLI interface

- [ ] `verdict validate <csv> <schema.yaml>` — main command
- [ ] Exit code 0 on all rules passed, 1 on any failure
- [ ] Human-readable output by default (table of results)
- [ ] `--format json` for machine-readable output (for CI log parsing)
- [ ] `--fail-fast` flag — stop on first failure
- [ ] `--quiet` — only print failures

### 5.4 CI integration

- [ ] Works as a GitHub Actions step with no setup beyond downloading the binary
- [ ] Example workflow snippet in README

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

## Optional: Split ComparableOps into EqualityOps + OrderOps

Currently `ComparableOps<T>` is a single trait with 6 methods: `gt`, `ge`, `lt`, `le`, `equal`, `between`. Every type that implements it must implement all 6.

**The problem:** `BoolColumn` has a legitimate use for `equal` — checking that two flag columns match row-by-row (`is_verified == is_active`). But `gt`, `ge`, `lt`, `le`, `between` on booleans are semantically meaningless in a validation context. Nobody writes a rule saying "column A must be greater than column B" for booleans. Right now we implement all 6 anyway because the trait requires it, which ships nonsensical operations as part of the public API.

**Proposed split:**
```rust
pub trait EqualityOps<T> {
    fn equal(&self, compare: T) -> Vec<Option<bool>>;
}

pub trait OrderOps<T>: EqualityOps<T> {
    fn gt(&self, compare: T) -> Vec<Option<bool>>;
    fn ge(&self, compare: T) -> Vec<Option<bool>>;
    fn lt(&self, compare: T) -> Vec<Option<bool>>;
    fn le(&self, compare: T) -> Vec<Option<bool>>;
    fn between(&self, lower: T, upper: T) -> Vec<Option<bool>>;
}
```

- `IntColumn`, `FloatColumn`, `StrColumn` implement both
- `BoolColumn` implements only `EqualityOps`
- `Constraint::Equal` requires `EqualityOps`, the rest require `OrderOps`

Do this when the bool `gt`/`le`/`between` dead weight becomes confusing — not before.

## ~~Optional: Generic ComparableOps~~ (Done)

Implemented `ComparableOps<T>` as generic trait. IntColumn supports both `i64` and `f64` comparison.
