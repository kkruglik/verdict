# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build                                       # Build all crates
cargo build --all-features                        # Build with all features (csv, parquet, etc.)
cargo test --all-features                         # Run all tests including feature-gated
cargo test -p verdict-core                        # Test core only
cargo test -p verdict-core --features csv         # Test core + csv
cargo test -p verdict-core --features parquet     # Test core + parquet
cargo test -p verdict-cli                         # Test CLI (all parsing tests)
cargo test -p verdict-core -- test_name           # Run specific test
cargo check                                       # Fast syntax/type check
```

### Python Extension (verdict-py)

```bash
cd crates/verdict-py
maturin develop                       # Build and install in current venv
maturin build --release               # Build wheel for distribution
```

## Architecture

Verdict is a data validation library with two crates:

```
verdict-core  ←  verdict-py
verdict-core  ←  verdict-cli
```

### verdict-core
Pure validation logic. Defines `Dataset`, `Schema`, `ValidationResult`, and validation rules. No I/O by default. Must remain usable as a standalone Rust library.

**Feature flags:**
- `csv` — enables `csv_loader` module with `DatasetCsvExt` trait (`Dataset::from_csv()`) and `CsvLoadingError`
- `parquet` — enables `parquet_loader` module with `DatasetParquetExt` trait (`DataFrame::from_parquet()`) and `ParquetLoadingError`

### verdict-py
PyO3 bindings exposing verdict to Python. Depends on `verdict-core` with `csv` feature enabled. The compiled library is named `verdict_py`.

**Note:** `verdict-py` does not yet support `TimeColumn` — it will fail to compile until separately updated.

**Python API (clean names via `#[pyclass(name = "...")]`):**
- `Dataset` — construct manually or load via `Dataset.from_csv(path, schema)`
- `Column` — typed constructors: `Column.integer(...)`, `Column.floating(...)`, `Column.string(...)`, `Column.boolean(...)`
- `Schema` — list of `(name, DataType)` tuples
- `DataType` — `DataType.integer()`, `DataType.float()`, `DataType.string()`, `DataType.boolean()`
- `Constraint` — factory for all column constraint variants
- `Rule(column, constraint)` — pairs a column name with a constraint
- `validate(dataset, rules) -> list[ValidationResult]` — main validation entry point
- `ValidationResult` — getters: `column`, `constraint`, `is_passed`, `failed_count`, `error`

**Python test scripts:**
- `crates/verdict-py/explore.py` — API exploration with small dataset
- `crates/verdict-py/benchmark.py` — verdict-only benchmark on 100k rows
- `crates/verdict-py/benchmark_pandas.py` — verdict vs pandas on 100k and 10M rows

### verdict-core Type Hierarchy

```
Dataset
├── columns: Vec<Column>
├── schema: Schema
│   └── fields: Vec<Field>
│       ├── name: String
│       └── dtype: DataType (Int, Float, Str, Bool, Date, DateTime, Time)
└── accessors: get_column_by_name, get_column_by_index, get_column_index, shape

Column (enum) — delegates to typed columns
├── common: len, is_empty, null_count, not_null_count, is_null
└── variants:
    ├── IntColumn (Vec<Option<i64>>)
    │   ├── NumericOps         → sum, min, max, mean, std, median
    │   ├── ComparableOps<i64> → gt, ge, lt, le, equal, between
    │   └── ComparableOps<f64> → gt, ge, lt, le, equal, between
    ├── FloatColumn (Vec<Option<f64>>)
    │   ├── NumericOps         → sum, min, max, mean, std, median
    │   └── ComparableOps<f64> → gt, ge, lt, le, equal, between
    ├── StrColumn (Vec<Option<String>>)
    │   ├── ComparableOps<&str> → gt, ge, lt, le, equal, between
    │   └── StringOps           → contains, starts_with, ends_with, matches_regex, length
    ├── BoolColumn (Vec<Option<bool>>)
    │   └── (common ops only)
    ├── DateColumn (Vec<Option<i32>>) — days since Unix epoch
    │   └── ComparableOps<&NaiveDate> / <i32> / <&str>
    ├── DateTimeColumn (Vec<Option<i64>>) — microseconds since Unix epoch
    │   └── ComparableOps<&NaiveDateTime> / <i64> / <&str>
    └── TimeColumn (Vec<Option<i64>>) — microseconds since midnight
        └── ComparableOps<&NaiveTime> / <i64> / <&str>
```

## Key Design Rules

- Core has no I/O dependencies by default
- CSV and parquet loading are behind feature flags, not separate crates
- Python knows core only
- No dependency cycles
- Validation rules live in core, never in py
- Time types are always stored in microseconds internally; loaders normalize on load
