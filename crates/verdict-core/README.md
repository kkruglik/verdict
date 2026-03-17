# verdict-core

High-performance data validation engine for tabular data, written in Rust.

Define typed schemas and validation rules, then run them against `Dataset` objects loaded from CSV or constructed in memory. All logic is pure Rust with no I/O dependencies by default.

## Usage

```toml
[dependencies]
verdict-core = "0.1.0"

# optional features
verdict-core = { version = "0.1.0", features = ["csv", "json"] }
```

## Features

- **14 constraint types** — null checks, uniqueness, numeric comparisons, string patterns, set membership, and more
- **Column-to-column comparisons** — validate that one column's values are greater than another's
- **Null-aware** — nulls are skipped in comparisons; use `not_null` to enforce presence
- **Failed row samples** — each result includes row index and value for up to N failed rows
- **Structured report** — `ValidationReport` with pass/fail summary, counts, and per-rule results
- **Zero I/O in core** — no filesystem dependencies by default

## Feature flags

| Feature | What it enables |
|---|---|
| `csv` | `Dataset::from_csv()` — load CSV files with typed schema enforcement |
| `json` | `#[derive(Serialize)]` on report types + `ValidationReport::to_json()` |

## Quick Start

```rust
use verdict_core::{Dataset, Schema, Field, DataType, Column, IntColumn};
use verdict_core::rules::{Rule, Constraint, validate, ValidateConfig};

let schema = Schema::new(vec![
    Field::new("age", DataType::Int),
]);

let dataset = Dataset::new(
    vec![Column::Int(IntColumn(vec![Some(25), None, Some(15)]))],
    schema,
);

let rules = vec![Rule::new("age", Constraint::Gt(18.0))];
let report = validate(&dataset, &rules, ValidateConfig::default());

println!("passed: {}", report.passed);
// failed_values contains (row_index, value_string) pairs
for r in &report.results {
    if let Some(ref vals) = r.failed_values {
        for (idx, val) in vals {
            println!("row {idx}: {val}");
        }
    }
}
```

## Constraints

| Constraint | Applies to | Description |
|---|---|---|
| `NotNull` | All | No null values |
| `Unique` | All | All values distinct |
| `Gt(v)` | Int, Float | Every value > v |
| `Ge(v)` | Int, Float | Every value >= v |
| `Lt(v)` | Int, Float | Every value < v |
| `Le(v)` | Int, Float | Every value <= v |
| `Equal(v)` | Int, Float, Str | Every value == v |
| `Between(min, max)` | Int, Float | min <= value <= max |
| `MatchesRegex(p)` | Str | Value matches regex |
| `Contains(s)` | Str | Value contains substring |
| `StartsWith(p)` | Str | Value starts with prefix |
| `EndsWith(s)` | Str | Value ends with suffix |
| `LengthBetween(min, max)` | Str | String length in [min, max] |
| `IsIn(values)` | All | Value is a member of the set |

## License

MIT
