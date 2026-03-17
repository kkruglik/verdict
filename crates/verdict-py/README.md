# verdict-py

Python bindings for [verdict](https://github.com/kkruglik/verdict) — a high-performance data validation library written in Rust.

## Installation

```bash
pip install maturin
git clone https://github.com/kkruglik/verdict
cd verdict/crates/verdict-py
maturin develop --release
```

## Quick Start

```python
from verdict_py import Dataset, Schema, DataType, RuleBuilder, py_validate

schema = Schema([
    ("user_id", DataType.integer()),
    ("score",   DataType.float()),
    ("country", DataType.string()),
])

dataset = Dataset.from_csv("data.csv", schema)

rules = [
    *RuleBuilder("user_id").not_null().unique().build(),
    *RuleBuilder("score").between(0.0, 100.0).build(),
    *RuleBuilder("country").is_in(["US", "UK", "DE", "FR", "JP"]).build(),
]

report = py_validate(dataset, rules)

print(f"passed: {report.passed} ({report.passed_count}/{report.total_rules})")

for r in report.results:
    if not r.is_passed:
        print(f"FAIL: {r.column} / {r.constraint} — {r.failed_count} failures")
        for idx, val in (r.failed_values or []):
            print(f"  row {idx}: {val}")
```

## ValidationReport

`py_validate()` returns a `ValidationReport`:

| Field | Type | Description |
|---|---|---|
| `passed` | `bool` | `True` if all rules passed |
| `total_rules` | `int` | Total number of rules checked |
| `passed_count` | `int` | Number of rules that passed |
| `failed_count` | `int` | Number of rules that failed |
| `results` | `list[ValidationResult]` | Per-rule results |

Each `ValidationResult` has:

| Field | Type | Description |
|---|---|---|
| `column` | `str` | Column name |
| `constraint` | `str` | Constraint description |
| `is_passed` | `bool` | Whether this rule passed |
| `failed_count` | `int` | Number of failing rows |
| `error` | `str \| None` | Error message if failed |
| `failed_values` | `list[tuple[int, str]] \| None` | `(row_index, value)` for each failed row |

## Constraints

| Method | Applies to | Description |
|---|---|---|
| `not_null()` | All | No null values |
| `unique()` | All | All values distinct |
| `gt(value)` | Int, Float, Str, col | Every value > threshold |
| `ge(value)` | Int, Float, Str, col | Every value >= threshold |
| `lt(value)` | Int, Float, Str, col | Every value < threshold |
| `le(value)` | Int, Float, Str, col | Every value <= threshold |
| `equal(value)` | Int, Float, Str, col | Every value == target |
| `between(min, max)` | Int, Float, Str, col | min <= value <= max |
| `matches_regex(pattern)` | Str | Value matches regex pattern |
| `contains(substr)` | Str | Value contains substring |
| `starts_with(prefix)` | Str | Value starts with prefix |
| `ends_with(suffix)` | Str | Value ends with suffix |
| `length_between(min, max)` | Str | String length in [min, max] |
| `is_in(values)` | Int, Float, Str | Value is member of set |

Pass `col("name")` instead of a literal to compare two columns row-by-row:

```python
from verdict_py import col

RuleBuilder("high").gt(col("low")).build()     # validates high > low
RuleBuilder("end_date").ge(col("start_date")).build()
```

Nulls are skipped in all comparisons — they never count as failures. Use `not_null()` to enforce presence.

## API Reference

### Dataset

```python
# Load from CSV
dataset = Dataset.from_csv("data.csv", schema)

# Construct manually
from verdict_py import Column
dataset = Dataset(
    headers=["id", "score"],
    columns=[
        Column.integer([1, 2, None, 4]),
        Column.floating([9.5, 8.0, 7.5, None]),
    ]
)

dataset.shape()                     # (rows, cols)
dataset.get_column_by_name("id")    # Column | None
dataset.get_column_by_index(0)      # Column | None
```

### Schema

```python
schema = Schema([
    ("id",      DataType.integer()),
    ("score",   DataType.float()),
    ("name",    DataType.string()),
    ("active",  DataType.boolean()),
])
```

### RuleBuilder

Fluent builder that produces a list of `Rule` objects. Multiple constraints on the same column can be chained:

```python
rules = RuleBuilder("age").not_null().gt(0.0).between(18.0, 99.0).build()
# → 3 Rule objects
```

### Rule and Constraint (low-level)

```python
from verdict_py import Rule, Constraint, col

rules = [
    Rule("age",     Constraint.gt(18.0)),
    Rule("score",   Constraint.between(0.0, 100.0)),
    Rule("country", Constraint.is_in(["US", "UK", "DE"])),
    Rule("high",    Constraint.gt(col("low"))),
]
```
