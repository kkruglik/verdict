# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build                           # Build all crates
cargo build --all-features            # Build with all features (csv, etc.)
cargo test --all-features             # Run all tests including feature-gated
cargo test -p verdict-core            # Test single crate (core only)
cargo test -p verdict-core --features csv  # Test core + csv
cargo test -p verdict-core -- test_name    # Run specific test
cargo check                           # Fast syntax/type check
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
```

### verdict-core
Pure validation logic. Defines `Dataset`, `Schema`, `ValidationResult`, and validation rules. No I/O by default. Must remain usable as a standalone Rust library.

**Feature flags:**
- `csv` — enables `csv_loader` module with `DatasetCsvExt` trait (`Dataset::from_csv()`) and `CsvLoadingError`

### verdict-py
PyO3 bindings exposing verdict to Python. Depends on `verdict-core` with `csv` feature enabled. The compiled library is named `verdict_py`.

**Python API (clean names via `#[pyclass(name = "...")]`):**
- `Dataset` — construct manually or load via `Dataset.from_csv(path, schema)`
- `Column` — typed constructors: `Column.integer(...)`, `Column.floating(...)`, `Column.string(...)`, `Column.boolean(...)`
- `Schema` — list of `(name, DataType)` tuples
- `DataType` — `DataType.integer()`, `DataType.float()`, `DataType.string()`, `DataType.boolean()`
- `Constraint` — factory for all 14 constraint variants
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
│       └── dtype: DataType (Int, Float, Str, Bool)
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
    └── BoolColumn (Vec<Option<bool>>)
        └── (common ops only)
```

## Key Design Rules

- Core has no I/O dependencies by default
- CSV loading is behind a feature flag, not a separate crate
- Python knows core only
- No dependency cycles
- Validation rules live in core, never in py
