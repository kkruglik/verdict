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
- [ ] `DateTimeOps` — `year`, `month`, `day`, `is_weekend` (deferred)

---

## Phase 2: Validation

### 2.1 Validation Results

- [x] Define `ValidationResult` struct (column, constraint, passed, failed_count, error)
- [x] `ValidationResult::passed()` / `ValidationResult::failed()` constructors
- [x] Track: passed/failed, failed count, error message
- [x] Implement `Display` for human-readable output
- [x] `ValidationReport` wrapping `Vec<ValidationResult>` with `passed`, `passed_count`, `failed_count`, `total_rules`

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
- [x] Expose `validate(dataset, rules) -> ValidationReport`
- [x] `ValidationReport` with getters: `passed`, `total_rules`, `passed_count`, `failed_count`, `results`
- [x] `ValidationResult` with getters: `column`, `constraint`, `is_passed`, `failed_count`, `error`, `failed_values`
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

- [x] `action.yml` GitHub Action — downloads pre-built binary, accepts CSV or Parquet, JSON/YAML schema
- [x] Example workflow snippet in README

---

## Phase 6: Actionable Output

**Goal:** make validation failures debuggable. Right now verdict tells you *how many* rows failed. This phase makes it tell you *which* rows and *what* values.

### 6.1 Failed row samples in ValidationResult

- [x] Add `failed_values: Option<Vec<(usize, String)>>` to `ValidationResult` — row index + string-formatted value, capped at `max_failed_samples`
- [x] Populate samples in all `check_*` functions
- [x] Expose samples in Python: `ValidationResult.failed_values -> list[tuple[int, str]] | None`
- [x] Include samples in CLI JSON output via serde: `"failed_values": [[3, "-1.5"], [7, "null"]]`
- [x] Include samples in CLI text output via `Display`: `row N: value` lines under each failure

### 6.2 Report struct

- [x] `ValidationReport` wrapping `Vec<ValidationResult>` in `verdict-core`
- [x] `passed: bool`, `passed_count: usize`, `failed_count: usize`, `total_rules: usize`
- [x] `Display` for summary: `"Validation Report: PASSED/FAILED (N/M rules passed)"`
- [x] JSON serialization via `to_json()` (behind `json` feature flag)
- [x] Expose `ValidationReport` in Python bindings
- [x] `validate()` returns `ValidationReport` instead of `Vec<ValidationResult>`
- [x] `ValidateConfig { max_failed_samples: usize }` controls sample cap (default 100)

---

## Phase 7: Date and Datetime Support ✅

**Goal:** make verdict usable on real-world datasets. Most production data has timestamps.

### 7.1 DateColumn and DateTimeColumn types

- [x] `DateColumn(Vec<Option<i32>>)` — epoch days; `DateTimeColumn(Vec<Option<i64>>)` — epoch microseconds
- [x] `DataType::Date` and `DataType::DateTime` variants
- [x] `Column::Date` and `Column::DateTime` enum variants
- [x] Common ops: `len`, `is_empty`, `null_count`, `is_null`, `unique_count`

### 7.2 Date constraints

- [x] `After(date)` — all values after a given date
- [x] `Before(date)` — all values before a given date
- [x] `BetweenDates { min, max }` — values in date range
- [x] `NotNull`, `Unique` wired up for Date/DateTime columns
- [x] Date format config for CSV parsing (`format` field in schema, default `%Y-%m-%d` / `%Y-%m-%dT%H:%M:%S`)

### 7.3 Expose in Python and CLI

- [x] `DataType.date()`, `DataType.datetime()` in Python
- [x] `Column.date([...])`, `Column.datetime([...])` constructors
- [x] `Constraint.after(date_str)`, `Constraint.before(date_str)`, `Constraint.between_dates(min, max)`
- [x] `"date"` / `"datetime"` dtype in CLI JSON/YAML schema

---

## Phase 8: Table-level Constraints ✅

**Goal:** validate the dataset itself, not just individual columns. Very common first check in GE/Soda.

- [x] `TableConstraint` enum separate from `ColumnConstraint` (12 variants)
- [x] Row count constraints: `RowsCountBetween`, `RowsCountGreaterOrEqual`, `RowCountGreaterThan`, `RowsCountLessOrEqual`, `RowCountLessThan`
- [x] Column count constraints: `ColumnsCountBetween`, `ColumnsCountGreaterOrEqual`, `ColumnsCountGreaterThan`, `ColumnsCountLessOrEqual`, `ColumnsCountLessThan`
- [x] `ColumnsExist(Vec<String>)` — named columns are present
- [x] `ShapeEquals { rows, columns }` — exact shape match
- [x] `validate_table()` as a separate function; `ValidationReport::merge()` to combine with column report
- [x] `ValidationResult.column` and `failed_count` made `Option` to share one type across column and table results
- [x] Exposed in Python (`py_validate_table`, `TableConstraint`, `TableRule`) and CLI schema (`table` block)

---

## Phase 9: CLI DX

**Goal:** make the CLI pleasant to use day-to-day.

### 9.1 YAML schema support ✅

- [x] `serde_yaml` dependency in `verdict-cli`
- [x] Auto-detect schema format by file extension (`.yaml`/`.yml` vs `.json`)
- [x] Same `ValidationConfig` struct, just a different deserializer
- [x] YAML example in README

### 9.2 CLI flags

- [ ] `--fail-fast` — stop on first constraint failure, exit 1
- [ ] `--quiet` — only print failed constraints (suppress passes)
- [ ] `--only-failed` equivalent in JSON output: filter to `passed: false` entries only

### 9.3 Severity levels

- [ ] Add `severity: "error" | "warn"` field to `ConstraintConfig` in schema (default: `"error"`)
- [ ] `warn` constraints appear in output but do not set exit code 1
- [ ] Reflect severity in `ValidationResult` output: `[WARN]` vs `[FAIL]`

---

## Phase 10: Parquet Support ✅

**Goal:** make verdict usable in modern data stacks where Parquet is the default format.

- [x] `parquet` feature flag in `verdict-core`
- [x] `DatasetParquetExt` trait — `DataFrame::from_parquet(path)` (no schema arg; types inferred from file metadata)
- [x] `ParquetLoadingError` — `IoError`, `ParquetError`, `UnsupportedType`, `TypeMismatch` variants
- [x] Type mapping: `BOOLEAN`→Bool, `INT32`/`INT64`→Int, `FLOAT`/`DOUBLE`→Float, `BYTE_ARRAY(UTF8)`→Str, `DATE`→Date, `TIMESTAMP(ms/us)`→DateTime, `TIME(ms/us)`→Time
- [x] Unit normalisation on load: `TimestampMillis×1000`→µs, `TimeMillis×1000`→µs
- [x] `verdict-cli`: auto-detect `.parquet` extension, route to `from_parquet`
- [x] `TimeColumn(Vec<Option<i64>>)` — microseconds since midnight; new `DataType::Time` variant
- [x] Time constraints: `After`, `Before`, `BetweenDates` wired to `TimeColumn`
- [x] `ComparableOps<&NaiveTime>`, `<i64>`, `<&str>` for `TimeColumn`; `is_in` for Time/Date/DateTime in CLI
- [x] Parquet unit tests: 15 loader tests + 10 `TimeColumn` constraint tests
- [ ] Python bindings: `Dataset.from_parquet(path)` — deferred (verdict-py needs `TimeColumn` update first)
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
