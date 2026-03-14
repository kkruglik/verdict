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
- [ ] Mixed `Between` operands: `between(literal, col)` and `between(col, literal)` — 3 tests already written and ignored

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

- [x] Add `crates/verdict-cli` as a binary crate depending on `verdict-core` (csv feature)
- [x] `cargo build --release -p verdict-cli` produces a single static binary

### 5.2 Schema config file

- [x] JSON schema format — column names + types + constraints, no code required
- [x] Example: `config/entry.json`
- [x] `ValidationConfig`, `ColumnConfig`, `ConstraintConfig` structs parsed with `serde_json`
- [x] `constraints` field optional — declare all columns for schema, only constrain what matters

### 5.3 CLI interface

- [x] `verdict-cli <csv> <schema.json>` — positional args
- [x] Exit code 0 on all constraints passed, 1 on any failure
- [x] JSON output by default — machine-readable array of results
- [x] `--format text` for human-readable output
- [ ] `--fail-fast` flag — stop on first failure
- [ ] `--quiet` — only print failures

### 5.4 CI integration

- [ ] Works as a GitHub Actions step with no setup beyond downloading the binary
- [ ] Example workflow snippet in README

---

## Phase 6: Actionable Output

**Goal:** make validation failures debuggable. Right now verdict tells you *how many* rows failed. This phase makes it tell you *which* rows and *what* values.

### 6.1 Failed row samples in ValidationResult

- [ ] Add `failed_samples: Vec<(usize, String)>` to `ValidationResult` — row index + string-formatted value, capped at 5
- [ ] Populate samples in all `check_*` functions
- [ ] Expose samples in Python: `ValidationResult.failed_samples -> list[tuple[int, str]]`
- [ ] Include samples in CLI JSON output: `"failed_samples": [[3, "-1.5"], [7, "null"]]`
- [ ] Include samples in CLI text output: `FAIL: score / between(0, 100) — 42 failures (e.g. row 3: -1.5, row 7: -0.2)`

### 6.2 Report struct

- [ ] `Report` wrapping `Vec<ValidationResult>` in `verdict-core`
- [ ] `Report::all_passed() -> bool`
- [ ] `Report::failed() -> Vec<&ValidationResult>`
- [ ] `Report::passed_count() -> usize` / `Report::failed_count() -> usize`
- [ ] `Display` for summary: `"3/14 checks passed, 11 failed"`
- [ ] Expose `Report` in Python bindings
- [ ] `validate()` returns `Report` instead of `Vec<ValidationResult>`

---

## Phase 7: Date and Datetime Support

**Goal:** make verdict usable on real-world datasets. Most production data has timestamps.

### 7.1 DateColumn and DateTimeColumn types

- [ ] Add `DateColumn(Vec<Option<NaiveDate>>)` and `DateTimeColumn(Vec<Option<NaiveDateTime>>)` using `chrono`
- [ ] Add `DataType::Date` and `DataType::DateTime` variants
- [ ] Add `Column::Date` and `Column::DateTime` enum variants
- [ ] Common ops: `len`, `is_empty`, `null_count`, `is_null`, `unique_count`

### 7.2 Date constraints

- [ ] `After(date)` — all values after a given date
- [ ] `Before(date)` — all values before a given date
- [ ] `BetweenDates { min, max }` — values in date range
- [ ] `NotNull`, `Unique` already work via Column enum (wire up)
- [ ] Date format config for CSV parsing (default: `%Y-%m-%d` / `%Y-%m-%dT%H:%M:%S`)

### 7.3 Expose in Python and CLI

- [ ] `DataType.date()`, `DataType.datetime()` in Python
- [ ] `Column.date([...])`, `Column.datetime([...])` constructors
- [ ] `Constraint.after(date_str)`, `Constraint.before(date_str)`, `Constraint.between_dates(min, max)`
- [ ] `"date"` / `"datetime"` dtype in CLI JSON schema

---

## Phase 8: Table-level Constraints

**Goal:** validate the dataset itself, not just individual columns. Very common first check in GE/Soda.

- [ ] `TableConstraint` enum separate from column `Constraint`
- [ ] `RowCountBetween { min: usize, max: usize }` — dataset has expected number of rows
- [ ] `ColumnCountEquals(usize)` — dataset has expected number of columns
- [ ] `ColumnsExist(Vec<String>)` — named columns are present
- [ ] Wire into `validate()` or a separate `validate_table()` function
- [ ] Expose in Python and CLI schema format

---

## Phase 9: CLI DX

**Goal:** make the CLI pleasant to use day-to-day.

### 9.1 YAML schema support

- [ ] Add `serde_yaml` dependency to `verdict-cli`
- [ ] Auto-detect schema format by file extension (`.yaml`/`.yml` vs `.json`)
- [ ] Same `ValidationConfig` struct, just a different deserializer
- [ ] Add YAML example to README

### 9.2 CLI flags

- [ ] `--fail-fast` — stop on first constraint failure, exit 1
- [ ] `--quiet` — only print failed constraints (suppress passes)
- [ ] `--only-failed` equivalent in JSON output: filter to `passed: false` entries only

### 9.3 Severity levels

- [ ] Add `severity: "error" | "warn"` field to `ConstraintConfig` in schema (default: `"error"`)
- [ ] `warn` constraints appear in output but do not set exit code 1
- [ ] Reflect severity in `ValidationResult` output: `[WARN]` vs `[FAIL]`

---

## Phase 10: Parquet Support

**Goal:** make verdict usable in modern data stacks where Parquet is the default format.

- [ ] Add `parquet` feature flag to `verdict-core`
- [ ] `DatasetParquetExt` trait with `Dataset::from_parquet(path, schema)`
- [ ] `ParquetLoadingError` mirrors `CsvLoadingError`
- [ ] Support in `verdict-cli`: auto-detect by `.parquet` extension
- [ ] CI workflow: validate a sample `.parquet` file

---

## Phase 11: Statistical Constraints

**Goal:** make verdict useful for ML pipelines and data scientists checking data distributions.

- [ ] `MeanBetween { min: f64, max: f64 }` — column mean in range (Int, Float)
- [ ] `StdLe(f64)` — standard deviation below threshold (Int, Float)
- [ ] `MedianBetween { min: f64, max: f64 }` (Int, Float)
- [ ] `NullRatioLe(f64)` — at most N% nulls (all types) — e.g. `null_ratio_le(0.05)` = max 5% nulls
- [ ] Expose all in Python and CLI schema

---

## Phase 12: Pandas Integration (Python)

**Goal:** fit into existing Python data science workflows. Pandera's biggest advantage is zero-friction pandas adoption.

- [ ] `Dataset.from_pandas(df: pd.DataFrame) -> Dataset` — convert pandas DataFrame in Python bindings
- [ ] Auto-map pandas dtypes to verdict `DataType`
- [ ] `ValidationResult.to_pandas() -> pd.DataFrame` — convert results back to DataFrame for analysis
- [ ] Optional: `Report.to_pandas()` summary DataFrame

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
