## 1. `verdict-core` — *the brain*

**Purpose**
Define *what* validation is, not *how* data is loaded or exposed.

**This crate must be usable without CSV, Python, or files.**

### Responsibilities

* Dataset abstraction
* Column type system and operations
* Expectation definitions
* Validation algorithm
* Error & result types

---

### Type Hierarchy

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

---

### Traits

* **NumericOps** — math operations for numeric columns (Int, Float)
* **ComparableOps\<T\>** — comparison operations, generic over compare type
* **StringOps** — string pattern matching and length

All trait methods on columns return `Vec<Option<bool>>` to preserve null information.

---

### What must NOT be here

* CSV parsing
* PyO3
* filesystem
* logging
* CLI

---

## 2. `verdict-csv` — *the data loader*

**Purpose**
Turn a CSV file into a `Dataset`.

### Responsibilities

* Read CSV files
* Normalize missing values
* Build `Dataset`

### API

```rust
pub fn load_csv(path: &Path, schema: &Schema) -> Result<Dataset, VerdictError>
```

### What must NOT be here

* Python bindings
* Expectation definitions
* Validation rules

This crate is replaceable.

---

## 3. `verdict-py` — *the bridge*

**Purpose**
Expose `verdict` to Python with a clean API.

### Responsibilities

* PyO3 wrappers
* Python-friendly classes
* Convert Rust results → Python objects
* Convert Rust errors → Python exceptions

### Python-facing API (conceptual)

```python
import verdict

ds = verdict.Dataset.from_csv("users.csv")

ds.expect_not_null("id")
ds.expect_unique("email")

result = ds.validate()
print(result.failures)
```

### What must NOT be here

* CSV parsing logic
* Validation rules
* Core data structures (owned elsewhere)

---

## 4. Dependency direction (critical)

```
verdict-core
↑
verdict-csv
↑
verdict-py
```

* Core knows nothing
* CSV knows core
* Python knows both

No cycles. Ever.

---

## 5. Execution flow (end-to-end)

1. Python calls `Dataset.from_csv`
2. `verdict-py` calls `verdict-csv::load_csv`
3. `verdict-csv` returns `Dataset`
4. Python adds expectations
5. Python calls `validate`
6. `verdict-core::validate` runs
7. Result is returned to Python

Every step has one owner.
