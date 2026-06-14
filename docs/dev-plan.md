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

- [ ] Works as a GitHub Actions step with no setup beyond downloading the binary
- [ ] Example workflow snippet in README

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

### Resolved design decisions

- **Dependency:** use `parquet` crate from arrow-rs — well-maintained, industry standard. Accept the binary size increase.
- **Schema:** auto-infer column types from Parquet file metadata (same approach as Pandera and Great Expectations). Schema file still required for constraints, but `dtype` field ignored for Parquet — the file already knows its types.
- **Loader signature:** `DataFrame::from_parquet(path)` — no schema argument, unlike CSV. Constraints come from the CLI schema file separately.

### Type mapping

| Parquet physical type | Parquet logical type | verdict `Column` | Conversion |
|---|---|---|---|
| `BOOLEAN` | — | `BoolColumn` | direct |
| `INT32` | none | `IntColumn(i64)` | widen i32→i64 |
| `INT64` | none | `IntColumn(i64)` | direct |
| `FLOAT` | — | `FloatColumn(f64)` | widen f32→f64 |
| `DOUBLE` | — | `FloatColumn(f64)` | direct |
| `BYTE_ARRAY` | `UTF8` / `STRING` | `StringColumn` | direct |
| `INT32` | `DATE` | `DateColumn(i32)` | direct — both use days since Unix epoch |
| `INT64` | `TIMESTAMP(millis)` | `DateTimeColumn(i64)` | × 1000 → micros |
| `INT64` | `TIMESTAMP(micros)` | `DateTimeColumn(i64)` | direct |
| `INT64` | `TIMESTAMP(nanos)` | `DateTimeColumn(i64)` | ÷ 1000 → micros, sub-microsecond precision lost |

Unsupported types (`INT96`, `DECIMAL`, `FIXED_LEN_BYTE_ARRAY`, `LIST`, `MAP`, `STRUCT`, `TIME`) return a clear error.

### Implementation scope

- [ ] Add `parquet` feature flag to `verdict-core/Cargo.toml`
- [ ] `parquet_loader.rs` — `DatasetParquetExt` trait with `DataFrame::from_parquet(path)`
- [ ] `ParquetLoadingError` — `IoError`, `ParquetError`, `UnsupportedType { column, parquet_type }` variants
- [ ] Type mapping implementation with logical type inspection
- [ ] `verdict-cli`: auto-detect `.parquet` extension, route to `from_parquet`, `dtype` field in schema ignored for Parquet files
- [ ] Python bindings: `Dataset.from_parquet(path)`
- [ ] Tests: nulls in every column type, unsupported type error, timestamp unit variants, edge date values — fixtures generated programmatically
- [ ] CI workflow update to run parquet-gated tests

---

## Phase 11: Chunked Loading

**Goal:** validate files too large to fit in memory by processing them in fixed-size chunks.

### How it works

Read N rows → build `DataFrame` → validate → drop → repeat. `ValidationReport::merge()` accumulates results across chunks. The existing columnar validation core is unchanged.

### Design decisions to resolve before starting

- **Chunk size:** fixed default (e.g. 100k rows) with a `--chunk-size` CLI flag, or auto-sized based on available memory?
- **Incompatible constraints:** `unique` requires a hash set of all seen values (partially defeats memory savings); `median_between` (Phase 12) is mathematically impossible without full data. Options: error at startup if ruleset contains these constraints in chunked mode, or document the limitation and skip them silently.

### Implementation scope

- [ ] `DatasetCsvChunkedExt` trait — `from_csv_chunked(path, schema, chunk_size)` returns an iterator of `DataFrame`
- [ ] Same for Parquet once Phase 10 is done: `DatasetParquetChunkedExt`
- [ ] CLI `--chunk-size N` flag; when set, validate chunk by chunk and merge reports
- [ ] Clear error at startup if `unique` constraint is used with `--chunk-size`
- [ ] Python: `Dataset.from_csv_chunked(path, schema, chunk_size)` iterator
- [ ] Tests: validate a multi-chunk file, assert results equal single-load validation

---

## Phase 12: Statistical Constraints

**Goal:** make verdict useful for ML pipelines and data scientists checking data distributions.

- [ ] `MeanBetween { min: f64, max: f64 }` — column mean in range (Int, Float)
- [ ] `StdLe(f64)` — standard deviation below threshold (Int, Float)
- [ ] `MedianBetween { min: f64, max: f64 }` (Int, Float)
- [ ] `NullRatioLe(f64)` — at most N% nulls (all types) — e.g. `null_ratio_le(0.05)` = max 5% nulls
- [ ] Expose all in Python and CLI schema

---

## Phase 13: Pandas Integration (Python)

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
