I’ll answer as a world-famous Rust library architect and data-validation systems engineer.

**TL;DR**
Each crate has **one clear responsibility**. If you respect these boundaries, the project stays simple now and scalable later. Below is the **exact high-level logic** and **what code belongs where**, no hand-waving.

---

## 1. `verdict-core` — *the brain*

**Purpose**
Define *what* validation is, not *how* data is loaded or exposed.

**This crate must be usable without CSV, Python, or files.**

### Responsibilities

* Dataset abstraction
* Expectation definitions
* Validation algorithm
* Error & result types

---

### Core concepts (must exist)

#### Dataset

Represents data in a backend-agnostic way.

```rust
pub struct Dataset {
    pub columns: HashMap<String, Vec<Option<String>>>,
    pub row_count: usize,
}
```

No CSV knowledge. No Python.

---

#### Expectation

Declarative rules.

```rust
pub enum Expectation {
    NotNull { column: String },
    Unique { column: String },
    Min { column: String, value: f64 },
}
```

This is your public API backbone.

---

#### Validation

Applies expectations to a dataset.

```rust
pub fn validate(
    dataset: &Dataset,
    expectations: &[Expectation],
) -> ValidationResult
```

This is pure logic.

---

#### Results & failures

Structured, inspectable output.

```rust
pub struct Failure {
    pub expectation: String,
    pub column: Option<String>,
    pub row: Option<usize>,
    pub message: String,
}

pub struct ValidationResult {
    pub passed: bool,
    pub failures: Vec<Failure>,
}
```

---

#### Errors

Typed library errors (`thiserror`).

```rust
pub enum VerdictError {
    UnknownColumn(String),
    TypeError(String),
}
```

---

### What must NOT be here

* CSV parsing
* PyO3
* filesystem
* logging
* CLI

---

## 2. `verdict-csv` — *the executor*

**Purpose**
Turn a CSV file into a `Dataset` and run core validation.

---

### Responsibilities

* Read CSV files
* Normalize missing values
* Build `Dataset`
* Call `verdict-core::validate`

---

### High-level API

```rust
pub fn load_csv(path: &Path) -> Result<Dataset, VerdictError>
```

Optionally:

```rust
pub fn validate_csv(
    path: &Path,
    expectations: &[Expectation],
) -> Result<ValidationResult, VerdictError>
```

---

### Internal logic

* Use `csv` crate
* Read headers
* Store values column-wise
* Empty string → `None`

---

### What must NOT be here

* Python bindings
* Expectation definitions
* Validation rules
* Business logic

This crate is replaceable.

---

## 3. `verdict-py` — *the bridge*

**Purpose**
Expose `verdict` to Python with a clean API.

---

### Responsibilities

* PyO3 wrappers
* Python-friendly classes
* Convert Rust results → Python objects
* Convert Rust errors → Python exceptions

---

### Python-facing API (conceptual)

```python
import verdict

ds = verdict.Dataset.from_csv("users.csv")

ds.expect_not_null("id")
ds.expect_unique("email")

result = ds.validate()
print(result.failures)
```

---

### Rust side structure

```rust
#[pyclass]
struct PyDataset {
    dataset: Dataset,
    expectations: Vec<Expectation>,
}
```

Methods:

* `from_csv(path)`
* `expect_*`
* `validate()`

All logic calls Rust functions; Python holds no state logic.

---

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

---

## 6. Why this design is correct

* You can delete Python → Rust still works
* You can delete CSV → core still works
* You can add Polars later → core unchanged
* You can unit test everything in isolation

This is *exactly* how mature Rust systems are built.

---

## Final answer

* `verdict-core` defines **truth and rules**
* `verdict-csv` **feeds data** into the rules
* `verdict-py` **exposes everything** to Python

Nothing overlaps. Nothing leaks.

If you want, next I can:

* write the **exact first `not_null` implementation**, or
* design the **Python API surface**, or
* show how to **evolve this when Polars is added**.
