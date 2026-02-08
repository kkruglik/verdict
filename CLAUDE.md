# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build                           # Build all crates
cargo test                            # Run all tests
cargo test -p verdict-core            # Test single crate
cargo test -p verdict-core -- test_name  # Run specific test
cargo check                           # Fast syntax/type check
```

### Python Extension (verdict-py)

```bash
cd crates/verdict-py
maturin develop                       # Build and install in current venv
maturin build --release               # Build wheel for distribution
```

## Architecture

Verdict is a data validation library with three crates following strict dependency direction:

```
verdict-core  ←  verdict-csv  ←  verdict-py
```

### verdict-core
Pure validation logic. Defines `Dataset`, `Expectation`, `ValidationResult`, and `VerdictError`. No I/O, no Python, no external data formats. Must remain usable as a standalone Rust library.

### verdict-csv
CSV file loading only. Converts CSV files into `Dataset` structs. Uses the `csv` crate. No validation logic.

### verdict-py
PyO3 bindings exposing verdict to Python. Wraps both core and csv functionality. The compiled library is named `verdict` (not `verdict-py`).

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

- Core knows nothing about CSV or Python
- CSV knows core only
- Python knows both
- No dependency cycles
- Each crate has exactly one responsibility
- Validation rules live in core, never in csv or py
