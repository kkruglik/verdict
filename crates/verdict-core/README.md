# verdict-core

High-performance data validation engine for tabular data, written in Rust.

Define typed schemas and validation rules, then run them against `Dataset` objects loaded from CSV or constructed in memory. All logic is pure Rust with no I/O dependencies by default.

## Usage

```toml
[dependencies]
verdict-core = "0.1.0"

# with optional features
verdict-core = { version = "0.1.0", features = ["csv", "json"] }
```

## Features

- **14 constraint types** — null checks, uniqueness, numeric comparisons, string patterns, set membership
- **Column-to-column comparisons** — validate one column against another row-by-row
- **Null-aware** — nulls are skipped in comparisons; use `NotNull` to enforce presence
- **Failed row samples** — each result includes `(row_index, value_string)` pairs for failed rows
- **Structured report** — `ValidationReport` with pass/fail summary, counts, and per-rule results
- **Zero I/O in core** — no filesystem dependencies by default

## Feature flags

| Feature | What it enables |
|---|---|
| `csv` | `Dataset::from_csv()` — load CSV files with typed schema enforcement |
| `json` | `Serialize` on report types + `ValidationReport::to_json()` |

## Quick Start

```rust
use verdict_core::{Dataset, Schema, Field, DataType, Column, IntColumn};
use verdict_core::rules::{Rule, Constraint, Operand, validate, ValidateConfig};

let schema = Schema::new(vec![
    Field::new("age", DataType::Int),
]);

let dataset = Dataset::new(
    vec![Column::Int(IntColumn(vec![Some(25), None, Some(15)]))],
    schema,
);

let rules = vec![Rule::new("age", Constraint::GreaterThan(Operand::Num(18.0)))];
let report = validate(&dataset, &rules, ValidateConfig::default());

println!("passed: {}", report.passed);

for r in &report.results {
    if let Some(ref vals) = r.failed_values {
        for (idx, val) in vals {
            println!("row {idx}: {val}");
        }
    }
}
```

## ValidationReport

```rust
pub struct ValidationReport {
    pub passed: bool,
    pub total_rules: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub results: Vec<ValidationResult>,
}

pub struct ValidationResult {
    pub column: String,
    pub constraint: String,
    pub passed: bool,
    pub failed_count: usize,
    pub error: Option<String>,
    pub failed_values: Option<Vec<(usize, String)>>,  // (row_index, value)
}
```

## ValidateConfig

```rust
pub struct ValidateConfig {
    pub max_failed_samples: usize,  // default: 100
}
```

## Constraints

| Constraint | Applies to | Description |
|---|---|---|
| `NotNull` | All | No null values |
| `Unique` | All | All values distinct |
| `GreaterThan(op)` | Int, Float, Str | Every value > operand |
| `GreaterThanOrEqual(op)` | Int, Float, Str | Every value >= operand |
| `LessThan(op)` | Int, Float, Str | Every value < operand |
| `LessThanOrEqual(op)` | Int, Float, Str | Every value <= operand |
| `Equal(op)` | Int, Float, Str | Every value == operand |
| `Between { min, max }` | Int, Float, Str | min <= value <= max |
| `MatchesRegex(pattern)` | Str | Value matches regex |
| `Contains(substr)` | Str | Value contains substring |
| `StartsWith(prefix)` | Str | Value starts with prefix |
| `EndsWith(suffix)` | Str | Value ends with suffix |
| `LengthBetween { min, max }` | Str | String length in [min, max] |
| `InSet(values)` | Int, Float, Str | Value is member of set |

`Operand` can be a literal (`Operand::Num(f64)`, `Operand::Str(String)`) or a column reference (`Operand::Column(name)`) for row-by-row column comparison.

## License

MIT
